[CmdletBinding()]
param([string]$VerifiedArchives)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'Prepare-Nsis.ps1') -LibraryOnly
$root = Join-Path ([IO.Path]::GetTempPath()) ('aw-packaging-tests-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Force $root | Out-Null
$count=0
function Assert($Condition, $Message) { if (!$Condition) { throw $Message } }
function Must-Fail([scriptblock]$Operation, $Pattern) {
  $caught=$false
  try { & $Operation | Out-Null } catch { $caught=$true; Assert ($_.Exception.Message -match $Pattern) "Unexpected failure: $($_.Exception.Message)" }
  Assert $caught "Expected failure: $Pattern"
}
function Hash-Bytes([byte[]]$Bytes, $Algorithm) {
  $h=[Security.Cryptography.HashAlgorithm]::Create($Algorithm)
  try { return [Convert]::ToHexString($h.ComputeHash($Bytes)).ToLowerInvariant() } finally { $h.Dispose() }
}
try {
  $good=[Text.Encoding]::UTF8.GetBytes('verified fixture')
  $bad=[Text.Encoding]::UTF8.GetBytes('corrupt! fixture')
  $fake=[PSCustomObject]@{name='fixture.zip';url='https://github.com/example/tools/releases/download/v1/fixture.zip';apiUrl='https://api.github.com/repos/example/tools/releases/assets/1';size=$good.Length;sha1=(Hash-Bytes $good SHA1);sha256=(Hash-Bytes $good SHA256)}
  $events=[Collections.Generic.List[object]]::new();$calls=[Collections.Generic.List[string]]::new()
  $fetch={param($uri,$path,$maximum,$accept) $calls.Add($uri); if($uri -eq $fake.url){throw 'http-504'};[IO.File]::WriteAllBytes($path,$good)}.GetNewClosure()
  $download=Get-VerifiedTool $fake "$root/download" $events $fetch {}
  Assert ($calls.Count -eq 3) 'Expected bounded primary retries, then asset API fallback'
  Assert ($events[-1].route -eq 'asset-api') 'API fallback not recorded'
  Assert-ToolFile $download $fake
  $count++
  $noNetwork={throw 'Cache validation unexpectedly used network'}
  Get-VerifiedTool $fake "$root/download" $events $noNetwork {} | Out-Null
  $count++
  [IO.File]::WriteAllBytes($download,$bad)
  $calls.Clear()
  Get-VerifiedTool $fake "$root/download" $events $fetch {} | Out-Null
  Assert ($calls.Count -eq 3) 'Poisoned restored archive cache was not replaced'
  $count++
  $calls.Clear()
  $corrupt={param($uri,$path,$maximum,$accept) $calls.Add($uri);[IO.File]::WriteAllBytes($path,$bad)}.GetNewClosure()
  Must-Fail {Get-VerifiedTool $fake "$root/failure" $events $corrupt {}} 'Application compilation has NOT started'
  Assert ($calls.Count -eq 6) 'All routes must have bounded two-attempt retries'
  Assert (!(Test-Path "$root/failure/fixture.zip")) 'Unverified download survived'
  Assert (!(Test-Path "$root/failure/fixture.zip.part")) 'Partial download survived'
  $count++
  # Valid-size JSON/error payload or corrupted bytes must still fail hashes.
  [IO.File]::WriteAllBytes("$root/wrong.bin",$bad)
  Must-Fail {Assert-ToolFile "$root/wrong.bin" $fake} 'SHA1-mismatch'
  $count++
  Assert-NoToolDownload 'LangString: "webview2DownloadError" 1033 "Error: Downloading WebView2 Failed - $0"'
  Must-Fail {Assert-NoToolDownload ' Downloading [tauri_bundler::utils::http_utils] https://127.0.0.1:9/tool.zip'} 'tool acquisition'
  Must-Fail {Assert-NoToolDownload ' Warn [tauri_bundler::bundle::windows::nsis] NSIS directory is missing some files. Recreating it.'} 'tool acquisition'
  $count++
  # Match the exact Windows cache gate of pinned bundler 2.10.0, and exercise
  # extraction against the real upstream archives when supplied explicitly.
  if ($VerifiedArchives) {
    $installed=Install-NsisTools $VerifiedArchives "$root/tauri" "$root/evidence" -SkipProbe
    Assert-NsisCache $installed
    Assert ((Get-ChildItem $installed -Recurse -File).Count -eq 442) 'Unexpected upstream archive inventory'
    Remove-Item (Join-Path $installed 'Include/MUI2.nsh')
    Must-Fail {Assert-NsisCache $installed} 'missing required file'
    $installed=Install-NsisTools $VerifiedArchives "$root/tauri" "$root/evidence" -SkipProbe
    [IO.File]::AppendAllText((Join-Path $installed 'Plugins/x86-unicode/additional/nsis_tauri_utils.dll'),'x')
    Must-Fail {Assert-NsisCache $installed} 'size-mismatch'
    $count+=3
  }
  $app=Join-Path $root 'synthetic app';$out=Join-Path $root 'diagnostic'
  New-Item -ItemType Directory -Force "$app/src-tauri/target/release" | Out-Null
  $config=Get-Content (Join-Path $PSScriptRoot '../../src-tauri/tauri.conf.json') -Raw | ConvertFrom-Json
  $config | ConvertTo-Json -Depth 20 | Set-Content "$app/src-tauri/tauri.conf.json"
  [IO.File]::WriteAllText("$app/src-tauri/target/release/abiding-word.exe",'synthetic PE stand-in, never execute')
  [IO.File]::WriteAllText("$app/src-tauri/target/release/loader.dll",'synthetic DLL stand-in')
  foreach($resource in $config.bundle.resources.PSObject.Properties){
    $path=[IO.Path]::GetFullPath((Join-Path "$app/src-tauri" $resource.Name))
    New-Item -ItemType Directory -Force (Split-Path $path) | Out-Null
    [IO.File]::WriteAllText($path,'Invented resource fixture')
  }
  & (Join-Path $PSScriptRoot 'Save-DiagnosticExecutable.ps1') -AppDirectory $app -Destination $out
  $manifest=Get-Content "$out/manifest.json" -Raw | ConvertFrom-Json
  Assert $manifest.complete 'Complete diagnostic fixture reported missing resources'
  Assert ($manifest.classification -eq 'diagnostic-only-not-portable') 'Unsafe diagnostic classification'
  Assert ($manifest.files.Count -eq 16) 'Expected exe, DLL, disclaimer and all 13 resources'
  $count++
  Remove-Item "$app/resources/scripture/grcsbl/grcsbl_usfm.zip"
  & (Join-Path $PSScriptRoot 'Save-DiagnosticExecutable.ps1') -AppDirectory $app -Destination "$root/incomplete"
  $manifest=Get-Content "$root/incomplete/manifest.json" -Raw | ConvertFrom-Json
  Assert (!$manifest.complete -and $manifest.missing.Count -eq 1) 'Incomplete diagnostics were mislabeled complete'
  $count++
  $config.bundle.resources.PSObject.Properties.Remove('../resources/scripture/grcsbl/grcsbl_usfm.zip')
  $config.bundle.resources | Add-Member -NotePropertyName '../resources/scripture/engbsb/engbsb_usfm.zip' -NotePropertyValue '../escape.zip' -Force
  $config | ConvertTo-Json -Depth 20 | Set-Content "$app/src-tauri/tauri.conf.json"
  Must-Fail {& (Join-Path $PSScriptRoot 'Save-DiagnosticExecutable.ps1') -AppDirectory $app -Destination "$root/unsafe"} 'Unsafe resource destination'
  $count++
  Get-ChildItem $PSScriptRoot -Filter '*.ps1' | ForEach-Object {
    $parseErrors=@();[void][Management.Automation.Language.Parser]::ParseFile($_.FullName,[ref]$null,[ref]$parseErrors)
    Assert ($parseErrors.Count -eq 0) "PowerShell syntax error in $($_.Name)"
  }
  $count++
  Write-Output "$count packaging checks passed. Windows native execution and Tauri cache probe are separate, not claimed on Linux."
} finally {Remove-Item -Recurse -Force $root}
