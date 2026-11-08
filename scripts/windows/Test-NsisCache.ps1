[CmdletBinding()]
param([Parameter(Mandatory)][string]$AppDirectory, [Parameter(Mandatory)][string]$EvidenceDirectory)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
if (!$IsWindows) { throw 'The real Tauri cache probe must run on Windows.' }
$probeEvidence = $EvidenceDirectory
. (Join-Path $PSScriptRoot 'Prepare-Nsis.ps1') -LibraryOnly
$EvidenceDirectory = $probeEvidence
$cliPackage = Join-Path $AppDirectory 'node_modules/@tauri-apps/cli/package.json'
if ((Get-Content $cliPackage -Raw | ConvertFrom-Json).version -cne $script:Toolchain.cliVersion) { throw 'Unexpected Tauri CLI version; review the pinned NSIS toolchain first.' }
$tools = Join-Path ([Environment]::GetFolderPath('LocalApplicationData')) 'tauri/NSIS'
Assert-NsisCache $tools
$probe = Join-Path $EvidenceDirectory 'cache-probe'
New-Item -ItemType Directory -Force "$probe/src", "$probe/target/release" | Out-Null
@'
[package]
name = "nsis-cache-probe"
version = "1.0.0"
edition = "2021"
[workspace]
'@ | Set-Content "$probe/Cargo.toml"
'fn main() {}' | Set-Content "$probe/src/main.rs"
@{productName='NSIS cache probe';version='1.0.0';identifier='app.abidingword.nsiscacheprobe';bundle=@{active=$true;targets=@('nsis');icon=@();windows=@{webviewInstallMode=@{type='skip'}}}} |
  ConvertTo-Json -Depth 8 | Set-Content "$probe/tauri.conf.json"
# This standalone, dependency-free probe is NOT Abiding Word compilation and
# is never distributed or executed. It only supplies a valid PE for bundling.
& rustc --crate-name nsis_cache_probe --edition 2021 "$probe/src/main.rs" -o "$probe/target/release/nsis-cache-probe.exe"
if ($LASTEXITCODE -ne 0) { throw 'Tiny NSIS probe PE compilation failed; application compilation has not started.' }
$oldTarget = $env:CARGO_TARGET_DIR; $oldOffline = $env:CARGO_NET_OFFLINE
$oldMirror = $env:TAURI_BUNDLER_TOOLS_GITHUB_MIRROR_TEMPLATE
$oldBaseMirror = $env:TAURI_BUNDLER_TOOLS_GITHUB_MIRROR
try {
  $env:CARGO_TARGET_DIR = "$probe/target"
  $env:CARGO_NET_OFFLINE = 'true'
  # The exact installed CLI must accept the prepared cache, not recover by
  # downloading tools. Any attempted GitHub tool download fails immediately.
  $env:TAURI_BUNDLER_TOOLS_GITHUB_MIRROR = $null
  $env:TAURI_BUNDLER_TOOLS_GITHUB_MIRROR_TEMPLATE = 'https://127.0.0.1:9/<owner>/<repo>/<version>/<asset>'
  Push-Location $probe
  try {
    & node (Join-Path $AppDirectory 'node_modules/@tauri-apps/cli/tauri.js') -vv bundle --bundles nsis --no-sign --no-binary-patching --ci 2>&1 |
      Tee-Object -FilePath (Join-Path $EvidenceDirectory 'nsis-cache-probe.log')
    $code = $LASTEXITCODE
  } finally { Pop-Location }
  if ($code -ne 0) { throw 'Tauri NSIS cache probe failed before application compilation. Inspect nsis-cache-probe.log.' }
  $log = Get-Content (Join-Path $EvidenceDirectory 'nsis-cache-probe.log') -Raw
  Assert-NoToolDownload $log
  $setups = @(Get-ChildItem "$probe/target/release/bundle/nsis/*-setup.exe" -File)
  if ($setups.Count -ne 1 -or $setups[0].Length -eq 0) { throw 'Tauri probe produced no NSIS installer.' }
  Assert-NsisCache $tools
  @{cli=$script:Toolchain.cliVersion;bundler=$script:Toolchain.bundlerVersion;result='passed';githubToolDownloadsBlocked=$true;probeInstallerSha256=(Get-FileHash $setups[0].FullName -Algorithm SHA256).Hash} |
    ConvertTo-Json | Set-Content (Join-Path $EvidenceDirectory 'nsis-cache-probe.json')
} finally {
  $env:CARGO_TARGET_DIR = $oldTarget; $env:CARGO_NET_OFFLINE = $oldOffline
  $env:TAURI_BUNDLER_TOOLS_GITHUB_MIRROR_TEMPLATE = $oldMirror
  $env:TAURI_BUNDLER_TOOLS_GITHUB_MIRROR = $oldBaseMirror
  # Keep logs/hashes only. This synthetic probe must never become an app artifact.
  if (Test-Path $probe) { Remove-Item -Recurse -Force $probe }
}
