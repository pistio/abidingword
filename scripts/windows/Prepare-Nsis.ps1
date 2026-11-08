[CmdletBinding()]
param(
  [string]$CacheDirectory,
  [string]$ToolsRoot = (Join-Path ([Environment]::GetFolderPath('LocalApplicationData')) 'tauri'),
  [string]$EvidenceDirectory,
  [switch]$LibraryOnly,
  [switch]$SkipExecutableProbe
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$script:Toolchain = Get-Content (Join-Path $PSScriptRoot 'nsis-toolchain.json') -Raw | ConvertFrom-Json

function Assert-ToolFile($Path, $Tool) {
  if (!(Test-Path -LiteralPath $Path -PathType Leaf)) { throw 'missing-file' }
  if ((Get-Item -LiteralPath $Path).Length -ne $Tool.size) { throw 'size-mismatch' }
  foreach ($algorithm in @('SHA1', 'SHA256')) {
    if ((Get-FileHash -LiteralPath $Path -Algorithm $algorithm).Hash -ine $Tool.$algorithm) { throw "$algorithm-mismatch" }
  }
}

function Invoke-ToolFetch($Uri, $Path, $MaximumBytes, $Accept) {
  # No token: these are public assets. Default TLS validation and environment
  # proxy remain enabled. Bound redirects, wall time, and streamed bytes.
  $handler = [System.Net.Http.HttpClientHandler]::new()
  $handler.MaxAutomaticRedirections = 5
  $client = [System.Net.Http.HttpClient]::new($handler)
  $client.Timeout = [TimeSpan]::FromSeconds(20)
  $response = $null; $inputStream = $null; $outputStream = $null
  $deadline = [Threading.CancellationTokenSource]::new([TimeSpan]::FromSeconds(20))
  try {
    $client.DefaultRequestHeaders.UserAgent.ParseAdd('AbidingWord-NSIS-preflight/1')
    $client.DefaultRequestHeaders.Accept.ParseAdd($Accept)
    $client.DefaultRequestHeaders.Add('X-GitHub-Api-Version', '2022-11-28')
    $response = $client.GetAsync($Uri, [System.Net.Http.HttpCompletionOption]::ResponseHeadersRead, $deadline.Token).GetAwaiter().GetResult()
    if (!$response.IsSuccessStatusCode) { throw "http-$([int]$response.StatusCode)" }
    $type = [string]$response.Content.Headers.ContentType.MediaType
    if ($Accept -eq 'application/octet-stream' -and $type -notin @('application/octet-stream','application/zip','application/x-zip-compressed','application/x-msdownload')) { throw 'unexpected-content-type' }
    $inputStream = $response.Content.ReadAsStreamAsync().GetAwaiter().GetResult()
    $outputStream = [IO.File]::Create($Path)
    $buffer = [byte[]]::new(65536); $total = 0
    # ReadAsync needs a cancellation token too: response-header timeout alone
    # does not bound a stalled streaming body.
    try {
      while (($read = $inputStream.ReadAsync($buffer,0,$buffer.Length,$deadline.Token).GetAwaiter().GetResult()) -gt 0) {
        $total += $read
        if ($total -gt $MaximumBytes) { throw 'oversized-response' }
        $outputStream.Write($buffer,0,$read)
      }
    } finally { $outputStream.Flush() }
  } finally {
    if ($outputStream) { $outputStream.Dispose() }
    if ($inputStream) { $inputStream.Dispose() }
    if ($response) { $response.Dispose() }
    $deadline.Dispose(); $client.Dispose(); $handler.Dispose()
  }
}

function Get-VerifiedTool($Tool, $Directory, $Events, [scriptblock]$Fetch = ${function:Invoke-ToolFetch}, [scriptblock]$Delay = { param($seconds) Start-Sleep -Seconds $seconds }) {
  New-Item -ItemType Directory -Force $Directory | Out-Null
  $target = Join-Path $Directory $Tool.name
  try { Assert-ToolFile $target $Tool; $Events.Add(@{tool=$Tool.name;route='cache';result='verified'}); return $target } catch { }
  # Never trust an Actions cache hit without verifying both pinned hashes.
  Remove-Item -LiteralPath $target -Force -ErrorAction SilentlyContinue
  $routes = [Collections.Generic.List[object]]::new()
  $routes.Add(@{name='release';url=$Tool.url})
  if ($Tool.PSObject.Properties.Name -contains 'apiUrl') { $routes.Add(@{name='asset-api';url=$Tool.apiUrl}) }
  else {
    # Public GitHub Release API: find the named asset, then request its binary
    # endpoint with Accept: application/octet-stream. Never trust a JSON body
    # masquerading as the asset; size and both hashes are mandatory afterwards.
    $metadata = Join-Path $Directory 'release-metadata.part'
    try {
      & $Fetch $Tool.releaseApiUrl $metadata 1048576 'application/vnd.github+json'
      $assets = (Get-Content $metadata -Raw | ConvertFrom-Json).assets
      $matches = @($assets | Where-Object { $_.name -ceq $Tool.name -and $_.size -eq $Tool.size })
      if ($matches.Count -eq 1 -and [string]$matches[0].id -match '^\d+$') {
        $routes.Add(@{name='asset-api';url=($Tool.assetApiPrefix + $matches[0].id)})
      }
    } catch { $Events.Add(@{tool=$Tool.name;route='release-api';result='unavailable'}) }
    finally { Remove-Item $metadata -Force -ErrorAction SilentlyContinue }
  }
  # Same official asset through GitHub's browser-download redirect with a fresh
  # query, avoiding a cached gateway failure. Hashes are identical for all routes.
  $routes.Add(@{name='release-fresh';url=($Tool.url + '?download=1&preflight=' + [Guid]::NewGuid().ToString('N'))})
  foreach ($route in $routes) {
    for ($attempt=1; $attempt -le 2; $attempt++) {
      $part = "$target.part"
      try {
        $uri = if ($route.name -eq 'release-fresh') { $Tool.url + '?download=1&preflight=' + [Guid]::NewGuid().ToString('N') } else { $route.url }
        & $Fetch $uri $part $Tool.size 'application/octet-stream'
        Assert-ToolFile $part $Tool
        Move-Item -LiteralPath $part -Destination $target -Force
        $Events.Add(@{tool=$Tool.name;route=$route.name;attempt=$attempt;result='verified';sha256=$Tool.sha256})
        return $target
      } catch {
        # Fixed error categories only: do not print response bodies, signed
        # redirect URLs, proxy configuration or credentials into Actions logs.
        $code = [string]$_.Exception.Message
        if ($code -notmatch '^(http-\d+|size-mismatch|SHA1-mismatch|SHA256-mismatch|oversized-response|unexpected-content-type)$') { $code='transport-failed' }
        $Events.Add(@{tool=$Tool.name;route=$route.name;attempt=$attempt;result=$code})
        Write-Warning "NSIS preflight: $($Tool.name), $($route.name), attempt ${attempt}: $code"
      } finally { Remove-Item -LiteralPath $part -Force -ErrorAction SilentlyContinue }
      if ($attempt -eq 1) { & $Delay 2 }
    }
  }
  throw "NSIS dependency preflight failed: $($Tool.name) unavailable or unverified after bounded official download routes. Application compilation has NOT started. Review nsis-preflight.json; do not rerun the full build automatically."
}

function Assert-NsisCache($Directory) {
  foreach ($relative in $script:Toolchain.requiredFiles) {
    if (!(Test-Path -LiteralPath (Join-Path $Directory $relative) -PathType Leaf)) { throw "NSIS cache missing required file: $relative" }
  }
  foreach ($relative in $script:Toolchain.pluginFiles) {
    if (!(Test-Path -LiteralPath (Join-Path $Directory "Plugins/x86-unicode/$relative") -PathType Leaf)) { throw "NSIS cache missing plugin: $relative" }
  }
  Assert-ToolFile (Join-Path $Directory 'Plugins/x86-unicode/additional/nsis_tauri_utils.dll') $script:Toolchain.tools[1]
}

function Assert-NoToolDownload($Log) {
  # makensis verbose output includes translated installer strings such as
  # "Error: Downloading WebView2 Failed"; those are not tool downloads.
  if ($Log -match '(?m)^\s*Downloading\b|NSIS directory (?:is missing|contains mis-hashed)') {
    throw 'Tauri attempted tool acquisition despite preflight; application compilation refused.'
  }
}

function Install-NsisTools($Cache, $Root, $Evidence, [switch]$SkipProbe) {
  $events = [Collections.Generic.List[object]]::new()
  New-Item -ItemType Directory -Force $Root,$Evidence | Out-Null
  $stage = Join-Path $Root ('nsis-stage-' + [Guid]::NewGuid().ToString('N'))
  try {
    $archive = Get-VerifiedTool $script:Toolchain.tools[0] $Cache $events
    $plugin = Get-VerifiedTool $script:Toolchain.tools[1] $Cache $events
    New-Item -ItemType Directory -Force $stage | Out-Null
    [IO.Compression.ZipFile]::ExtractToDirectory($archive,$stage)
    $prepared = Join-Path $stage 'nsis-3.11'
    $additional = Join-Path $prepared 'Plugins/x86-unicode/additional'
    New-Item -ItemType Directory -Force $additional | Out-Null
    Copy-Item -LiteralPath $plugin -Destination (Join-Path $additional 'nsis_tauri_utils.dll')
    Assert-NsisCache $prepared
    $installed = Join-Path $Root 'NSIS'
    # Always rebuild from verified archives, not mutable executable cache entries.
    if (Test-Path $installed) { Remove-Item -Recurse -Force $installed }
    Move-Item -LiteralPath $prepared -Destination $installed
    if (!$SkipProbe) {
      if (!$IsWindows) { throw 'The executable probe requires Windows; use SkipExecutableProbe only for explicit Linux fixture checks.' }
      $version = & (Join-Path $installed 'makensis.exe') /VERSION
      if ($LASTEXITCODE -ne 0 -or ([string]$version).Trim() -ne 'v3.11') { throw 'Cached makensis.exe did not report v3.11' }
    }
    $events.Add(@{result='installed';directory=$installed;executableProbe=(!$SkipProbe)})
    return $installed
  } finally {
    $events | ConvertTo-Json -Depth 8 -AsArray | Set-Content (Join-Path $Evidence 'nsis-preflight.json')
    if (Test-Path $stage) { Remove-Item -Recurse -Force $stage }
  }
}

if (!$LibraryOnly) {
  if (!$CacheDirectory -or !$EvidenceDirectory) { throw 'CacheDirectory and EvidenceDirectory are required.' }
  Install-NsisTools $CacheDirectory $ToolsRoot $EvidenceDirectory -SkipProbe:$SkipExecutableProbe
}
