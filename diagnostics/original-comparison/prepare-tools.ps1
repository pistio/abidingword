$ErrorActionPreference = 'Stop'
$dest = Join-Path $PWD 'work/portable-tools'
New-Item -ItemType Directory -Force $dest | Out-Null
$metadata = Get-Content (Join-Path $PSScriptRoot 'tool-provenance.json') -Raw | ConvertFrom-Json
function Download-Verified($item, $name) {
  $file = Join-Path $PWD "work/$name"
  Invoke-WebRequest -Uri $item.url -OutFile $file
  $actual = (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant()
  if ($actual -cne $item.archiveSha256) { throw "Tool archive hash mismatch: $name" }
  return $file
}
$nodeArchive = Download-Verified $metadata.node 'node-win.zip'
Expand-Archive -LiteralPath $nodeArchive -DestinationPath 'work/node-unpacked'
New-Item -ItemType Directory -Force "$dest/node" | Out-Null
Copy-Item 'work/node-unpacked/node-v24.22.3-win-x64/node.exe' "$dest/node/node.exe"
Copy-Item 'work/node-unpacked/node-v24.22.3-win-x64/LICENSE' "$dest/node/LICENSE"
$psArchive = Download-Verified $metadata.powershell 'pwsh-win.zip'
Expand-Archive -LiteralPath $psArchive -DestinationPath "$dest/pwsh"
$pwArchive = Download-Verified $metadata.playwrightCore 'playwright-core.tgz'
New-Item -ItemType Directory -Force 'work/pw-unpacked' | Out-Null
& tar -xzf $pwArchive -C 'work/pw-unpacked'
if ($LASTEXITCODE -ne 0) { throw 'Cannot extract Playwright client' }
Move-Item 'work/pw-unpacked/package' "$dest/playwright-core"
Copy-Item (Join-Path $PSScriptRoot 'tool-provenance.json') "$dest/tool-provenance.json"
