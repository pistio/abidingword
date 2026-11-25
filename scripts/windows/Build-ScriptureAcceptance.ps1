# Requires a separately approved, clean Windows x64 build environment.
[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidatePattern('^[a-f0-9]{40}$')][string]$SourceSha,
    [Parameter(Mandatory)][string]$OutputDirectory,
    [switch]$Approved
)
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true
if (!$Approved -or !$IsWindows -or [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture -ne 'X64') {
    throw 'This build requires explicit approval and Windows x64.'
}
$OutputDirectory = [IO.Path]::GetFullPath($OutputDirectory)
if (Test-Path -LiteralPath $OutputDirectory) { throw 'Use a new output directory.' }
$priorMode = $env:ABIDING_WORD_ACCEPTANCE_BUILD
$priorTarget = $env:CARGO_TARGET_DIR
Push-Location (Join-Path $PSScriptRoot '../..')
try {
    $null = New-Item -ItemType Directory -Path $OutputDirectory
    $evidence = Join-Path $OutputDirectory 'build-provenance.json'
    node scripts/build-provenance.mjs --expected-sha $SourceSha --require-clean --out $evidence
    @'
    import assert from 'node:assert/strict';
    import { createHash } from 'node:crypto';
    import { readFileSync, readdirSync } from 'node:fs';
    const proof = JSON.parse(readFileSync(process.argv[2], 'utf8'));
    assert.equal(proof.schemaVersion, 28);
    assert.equal(createHash('sha256').update(JSON.stringify(proof.migrations)).digest('hex'), 'f05e1c5356d49ef5f6e0b4c4be7dedddef78d9a494746bf2dff6d9ed51d0f4a6');
    assert.deepEqual(readdirSync('src-tauri/migrations').filter(f => f.endsWith('.sql')).sort(), proof.migrations.map(m => m.file).sort());
    const config = JSON.parse(readFileSync('src-tauri/tauri.conf.json', 'utf8'));
    assert.equal(config.identifier, 'app.abidingword.desktop');
    assert.equal(config.bundle.resources['../docs/release/THIRD-PARTY-NOTICES.md'], 'licenses/THIRD-PARTY-NOTICES.md');
    assert.equal(config.plugins?.updater, undefined);
    assert.ok(!config.bundle.createUpdaterArtifacts);
    assert.ok(!config.bundle.windows?.signCommand);
'@ | node --input-type=module - $evidence
    $overlay = Join-Path $OutputDirectory 'acceptance.conf.json'
    @{
        productName = 'Abiding Word Scripture Acceptance'
        mainBinaryName = 'abiding-word-scripture-acceptance'
        identifier = 'app.abidingword.scripture.acceptance'
        app = @{ windows = @(@{
            title = 'Abiding Word Scripture Acceptance'
            width = 1280; height = 800; minWidth = 640; minHeight = 520
            center = $true; backgroundColor = '#f6f1e6'
        }) }
        bundle = @{
            createUpdaterArtifacts = $false
            windows = @{ nsis = @{ installMode = 'currentUser' } }
        }
    } | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $overlay -Encoding utf8NoBOM
    $env:ABIDING_WORD_ACCEPTANCE_BUILD = 'scripture'
    $env:CARGO_TARGET_DIR = Join-Path $OutputDirectory 'cargo-target'
    @(
        "Source: $SourceSha"
        "Workflow: $env:GITHUB_WORKFLOW_SHA"
        "Run: $env:GITHUB_RUN_ID attempt $env:GITHUB_RUN_ATTEMPT"
        "OS: $([Environment]::OSVersion.VersionString)"
        "Node: $(node --version)"
        "npm: $(npm --version)"
        "$(rustc --version)"
        "$(cargo --version)"
    ) | Set-Content (Join-Path $OutputDirectory 'build-environment.txt')
    npm ci
    npm run release:resources
    npx --no-install tauri build --target x86_64-pc-windows-msvc --bundles nsis --config $overlay --no-sign --ci -- --locked
    $built = Join-Path $env:CARGO_TARGET_DIR 'x86_64-pc-windows-msvc/release'
    $installers = @(Get-ChildItem (Join-Path $built 'bundle/nsis/*-setup.exe') -File)
    if ($installers.Count -ne 1 -or $installers[0].Length -eq 0) { throw 'Expected exactly one nonempty NSIS package.' }
    $installedBinary = Join-Path $built 'abiding-word-scripture-acceptance.exe'
    $installer = Join-Path $OutputDirectory "Abiding-Word-Scripture-Acceptance-$SourceSha-x64-setup.exe"
    Copy-Item -LiteralPath $installers[0].FullName -Destination $installer
    foreach ($binary in @($installer, $installedBinary)) {
        if ((Get-AuthenticodeSignature -LiteralPath $binary).Status -ne 'NotSigned') { throw 'Expected unsigned acceptance binaries.' }
    }
    @{
        sourceSha = $SourceSha
        installer = Split-Path -Leaf $installer
        installerSha256 = (Get-FileHash -LiteralPath $installer).Hash.ToLowerInvariant()
        executable = Split-Path -Leaf $installedBinary
        executableSha256 = (Get-FileHash -LiteralPath $installedBinary).Hash.ToLowerInvariant()
        identifier = 'app.abidingword.scripture.acceptance'
        architecture = 'x86_64-pc-windows-msvc'
        updater = $false
    } | ConvertTo-Json | Set-Content (Join-Path $OutputDirectory 'acceptance-build.json')
    foreach ($name in @('Test-ScriptureAcceptance.mjs', 'scripture-acceptance-fixtures.mjs', 'original-epub-fixture.mjs', 'ScriptureAcceptance.md')) {
        Copy-Item -LiteralPath (Join-Path $PSScriptRoot $name) -Destination $OutputDirectory
    }
} finally {
    $env:ABIDING_WORD_ACCEPTANCE_BUILD = $priorMode
    $env:CARGO_TARGET_DIR = $priorTarget
    Pop-Location
}
