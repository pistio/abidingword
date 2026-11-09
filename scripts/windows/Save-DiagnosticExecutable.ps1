[CmdletBinding()]
param([Parameter(Mandatory)][string]$AppDirectory, [Parameter(Mandatory)][string]$Destination)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
New-Item -ItemType Directory -Force $Destination | Out-Null
$config = Get-Content (Join-Path $AppDirectory 'src-tauri/tauri.conf.json') -Raw | ConvertFrom-Json
$release = Join-Path $AppDirectory 'src-tauri/target/release'
$exe = Join-Path $release 'abiding-word.exe'
$missing = [Collections.Generic.List[string]]::new()
@'
DIAGNOSTIC COMPILATION OUTPUT — NOT AN INSTALLER, NOT VALIDATED AS PORTABLE
Source: 6ffed07de160b5168faf740cd4e1642ec5c29cf2; schema 27.
Unsigned application executable, target-adjacent DLLs and every configured Tauri
resource are preserved for diagnosis if packaging fails. This is not a release.
Windows launch, resource resolution, WebView2/VC++ runtime prerequisites and profile
handling still require validation. Do not launch against the primary owner profile.
The preflight's synthetic probe executable is intentionally excluded.
See manifest.json: complete=false means files are missing, never a portable app.
'@ | Set-Content (Join-Path $Destination 'DIAGNOSTIC-ONLY.txt')
if (Test-Path -LiteralPath $exe) {
  Copy-Item -LiteralPath $exe -Destination $Destination
  Get-ChildItem -LiteralPath $release -Filter '*.dll' -File | Copy-Item -Destination $Destination
} else { $missing.Add('abiding-word.exe (application compilation did not produce an executable)') }
if ($config.bundle.resources -isnot [PSCustomObject]) { throw 'Pinned resource map changed; diagnostic staging requires review.' }
foreach ($resource in $config.bundle.resources.PSObject.Properties) {
  $source = [IO.Path]::GetFullPath((Join-Path "$AppDirectory/src-tauri" $resource.Name))
  $target = [IO.Path]::GetFullPath((Join-Path $Destination ([string]$resource.Value)))
  $prefix = [IO.Path]::GetFullPath($Destination) + [IO.Path]::DirectorySeparatorChar
  if (!$target.StartsWith($prefix,[StringComparison]::OrdinalIgnoreCase)) { throw 'Unsafe resource destination.' }
  if (Test-Path -LiteralPath $source -PathType Leaf) {
    New-Item -ItemType Directory -Force (Split-Path $target) | Out-Null
    Copy-Item -LiteralPath $source -Destination $target
  } else { $missing.Add([string]$resource.Value) }
}
$files = @(Get-ChildItem -LiteralPath $Destination -Recurse -File | ForEach-Object {
  @{path=[IO.Path]::GetRelativePath($Destination,$_.FullName);bytes=$_.Length;sha256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()}
})
@{source='6ffed07de160b5168faf740cd4e1642ec5c29cf2';schema=27;classification='diagnostic-only-not-portable';complete=($missing.Count -eq 0);missing=@($missing);files=$files} |
  ConvertTo-Json -Depth 8 | Set-Content (Join-Path $Destination 'manifest.json')
if ($env:GITHUB_OUTPUT) { "available=$((Test-Path -LiteralPath $exe).ToString().ToLowerInvariant())" | Out-File $env:GITHUB_OUTPUT -Append -Encoding utf8 }
