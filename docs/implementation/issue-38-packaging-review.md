# Issue 38: Windows installer dependency preflight

Status: packaging correction prepared; **no Windows Actions build dispatched**.
Owner approval is required before another paid attempt. This does not change
Windows owner-acceptance gates or application functionality.

## Source and failure evidence

- Parent: `codex/windows-owner-corrections-35`, verified tip
  `2b39fe838158072dfce9f30d6bd942916a408d43` (PR #37).
- Application source remains exactly
  `571b8fc0633b6909b6532e1c6b254e2dbb357f45`.
- Issue #38 records successful application compilation followed by HTTP 504
  fetching NSIS in both attempts of run `42466393853`: 09:26 UTC (8m28s
  compilation) and 09:56 UTC (12m06s compilation), 2026-11-08.
- Public attempt/job pages confirm failed installer-build steps and skipped
  artifact uploads (jobs `128686798597` and `128692623806`). Authenticated raw
  logs are currently unavailable because this Cloud proxy blocks
  `api.github.com`; the precise download error above comes from the owner's
  Issue #38 evidence. This is a packaging dependency failure, not evidence of
  an application compilation failure.

The previous build bundled immediately after compilation; the first NSIS
download therefore happened late. Successful compilation output was lost when
success-only artifact steps were skipped.

## Pinned upstream behaviour

`package-lock.json` pins Tauri CLI 2.12.0. Its upstream tag resolves to
`447fa9f3f993fe77724189e355078b38ce20baea`, containing tauri-bundler 2.10.0.
The reviewed sources are:

- [NSIS bundler](https://github.com/tauri-apps/tauri/blob/447fa9f3f993fe77724189e355078b38ce20baea/crates/tauri-bundler/src/bundle/windows/nsis/mod.rs)
- [HTTP/mirror handling](https://github.com/tauri-apps/tauri/blob/447fa9f3f993fe77724189e355078b38ce20baea/crates/tauri-bundler/src/utils/http_utils.rs)

With this application's unchanged default `useLocalToolsDir=false`, Windows
uses `%LOCALAPPDATA%\tauri\NSIS`. A missing directory or any of the bundler's
13 required files causes full tool-tree recreation/download. An incorrect
helper DLL SHA-1 causes a DLL redownload. `useLocalToolsDir=true` would instead
use `<cargo-target>/.tauri/NSIS`; this workflow does not enable it.

The archive's `nsis-3.11` directory becomes `NSIS`. It supplies standard
`NSISdl`, `StartMenu`, `System` and `nsDialogs` plugins; the separately pinned
Tauri helper belongs at `Plugins/x86-unicode/additional/nsis_tauri_utils.dll`.
The manifest records every required path, not just `makensis.exe`.

| Dependency | Official source | Bytes | SHA-256 |
| --- | --- | ---: | --- |
| NSIS 3.11 | [binary-releases archive](https://github.com/tauri-apps/binary-releases/releases/download/nsis-3.11/nsis-3.11.zip) | 2361546 | `c7d27f780ddb6cffb4730138cd1591e841f4b7edb155856901cdf5f214394fa1` |
| nsis_tauri_utils 0.5.3 | [helper DLL](https://github.com/tauri-apps/nsis-tauri-utils/releases/download/nsis_tauri_utils-v0.5.3/nsis_tauri_utils.dll) | 34304 | `5ba143b5db4a87d32d6e7802e033330aae56cbceabe0d1e3ba41948385ad4709` |

Both downloaded files match GitHub's published release-asset SHA-256 digests
and the SHA-1 hashes embedded in the pinned bundler, respectively
`ef7ff767e5cbd9edd22add3a32c9b8f4500bb10d` and
`75197fee3c6a814fe035788d1c34ead39349b860`.

## Workflow correction

1. Check out the immutable application SHA under `app`, and the workflow's
   own reviewed packaging scripts under `packaging`. Validate provenance and
   the historical migration catalogue before compilation.
2. Install locked npm dependencies, restore download archives only, then run
   `Prepare-Nsis.ps1`. Verify exact size, SHA-1 and SHA-256 before extraction;
   rebuild executable tools from verified bytes every time.
3. For each dependency, use the official release URL, then GitHub's supported
   release-asset API with `Accept: application/octet-stream`, then a fresh-query
   official browser download. Each route has at most two attempts, a shared
   20-second deadline per request including body streaming, five redirects,
   a maximum response size and a two-second retry delay. No third-party mirror,
   authentication token or TLS bypass is used. The whole step is capped at six
   minutes. Failures retain categorised evidence without response bodies or
   signed redirect URLs. These are bounded dependency retries, not build reruns.
4. Check all required files/plugins and run cached Windows `makensis /VERSION`.
   Then a two-minute, standalone Tauri probe bundles a tiny disposable PE using
   the exact installed CLI, with Cargo offline and GitHub tool download URLs
   redirected to closed loopback. Refuse application compilation if this fails
   or reports downloading/recreating tools. The probe is never distributed.
5. Only then compile Abiding Word with `--no-bundle --no-sign --ci -- --locked`.
   Upload the unsigned executable, adjacent DLLs and all configured resource
   destinations before packaging. Its manifest records hashes and missing
   resources; its README explicitly says **diagnostic only, not a validated
   portable application**.
6. Bundle the compiled application separately, rechecking the NSIS cache and
   blocking late GitHub tool downloads. Upload provenance and failure logs
   with `always()`, even when preflight or packaging fails.

The workflow stays manual-only, `contents: read`, immutable action revisions,
no additional secrets, no signing/updater/release/deployment. Source, locks,
schema 27 and migrations 0001–0027 are unchanged. No recurring trigger or
automatic build retry is added.

## Local verification and limits

Run from the repository root:

```sh
python3 scripts/windows/test-workflow.py
pwsh -NoProfile -File scripts/windows/Test-Packaging.ps1
# Optional verified real-archive extraction checks:
pwsh -NoProfile -File scripts/windows/Test-Packaging.ps1 -VerifiedArchives /path/to/archives
```

- Static workflow checks pass: exact source/CLI pins, immutable actions,
  execution order, failure artifacts, manual trigger, permissions, unchanged
  application/schema, and closed-loopback bundler guard.
- Thirteen offline PowerShell checks pass with the actual verified archives:
  bounded fallback/retry behaviour, corrupt caches/downloads, SHA checks,
  complete extraction, missing tools, corrupt helper, diagnostic resources,
  incomplete-resource reporting, traversal refusal, PowerShell parsing and a
  regression distinguishing real downloads from translated installer strings.
- `actionlint` 1.7.7 passes. All eleven embedded workflow PowerShell blocks
  parse successfully using PowerShell 7.4.13.
- Official fresh-query browser downloads succeeded locally and both hashes
  were verified. A live empty-cache PowerShell preflight encountered this
  Cloud environment's transport/gateway failures, failed within bounds and
  retained evidence **before any application compilation**. API fallback is
  covered by injected offline tests; live GitHub API access is blocked here.
- The real pinned Linux Tauri CLI successfully cross-bundled a disposable
  fixture using native Linux NSIS 3.11, the verified Windows tool tree and
  helper, Cargo offline and the closed-loopback GitHub mirror. No tool download
  occurred. This validates the bundler/cache path locally, not Windows runtime
  behaviour. The fixture is not Abiding Word and is not an acceptance artifact.
- Full existing application checks pass: typecheck, ESLint, localisation
  (6 languages × 9 areas), 1,231 frontend tests, production frontend build,
  Rust formatting, locked Clippy across all targets with warnings denied, and
  locked Rust tests (1,094 passed, eight existing ignored, zero failed). The
  production frontend retains its existing large-chunk warning.

Remaining owner gate: review this packaging revision, then explicitly approve
one manual Windows run. Do not rerun either failed attempt: it uses the old
workflow. Dispatch the corrected workflow revision, retaining the single allowed
application SHA. Inspect preflight/probe evidence before accepting its installer.
Actual Windows execution of the new probe is pending. Public hosting outages
can still make preflight fail safely. WebView2 bootstrapper acquisition and
unsigned SmartScreen/runtime/installer behaviour remain Windows acceptance
risks; they are not claims resolved by these static tests.
