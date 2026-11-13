#requires -Version 7.2
param([Parameter(Mandatory)][string]$InputDirectory,[Parameter(Mandatory)][string]$OutputPath)
$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.IO.Compression.FileSystem
$root=(Resolve-Path -LiteralPath $InputDirectory).Path
$archive=[System.IO.Compression.ZipFile]::Open($OutputPath,[System.IO.Compression.ZipArchiveMode]::Create)
try {
  foreach($file in Get-ChildItem -LiteralPath $root -File -Recurse) {
    $relative=[System.IO.Path]::GetRelativePath($root,$file.FullName)
    # Disposable app/WebView profiles remain local. Evidence contains inputs,
    # copied diagnostic logs, measurements and native screenshots, not caches.
    if(($relative -split '[\\/]') -contains 'invented-profile'){continue}
    [System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile($archive,$file.FullName,$relative.Replace('\','/'),[System.IO.Compression.CompressionLevel]::Optimal) | Out-Null
  }
} finally {$archive.Dispose()}
