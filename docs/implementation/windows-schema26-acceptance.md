# Windows schema-26 acceptance build

## Scope and source

Application source is fixed at **`6da1ea3d757d76b1072338f36c331dffc764b8ff`**, the integration branch's PR #13 merge. Build infrastructure is reviewed on `feature/windows-schema26-acceptance-build`, based on that exact commit, targeting `integration/windows-library-stage3-teaching-study`. Do not select `main`.

This prepares an acceptance installer only. **No workflow has been dispatched and no installer has been produced by this task.** Manual Windows acceptance remains outstanding. Claude Code's Bible PDF/EPUB recognition acceptance is a separate unfinished dependency; this change touches no recognition code, fixtures, tests, corpus or branches.

Read [schema-26 validation](schema26-validation-handover.md) for the complete migration hashes, rollback evidence, application coverage and known Windows gates. The older [v0.24 guide](../windows-acceptance-v0.24.md) describes schema 22 and publication; it does not authorise publication or opening real schema-19 data for this acceptance run.

## Build design and safeguards

Workflow: [`.github/workflows/windows-acceptance-build.yml`](../../.github/workflows/windows-acceptance-build.yml).

- Only `workflow_dispatch`, with a second event guard. Its single source choice and an explicit equality check allow only the full approved application SHA above, including when called through an API. The workflow revision and application revision are distinct and both are recorded.
- Read-only `contents` permission, no environment/signing/updater secrets, and checkout credentials are not persisted. There are no release, channel-manifest, deployment, write-token or workflow-dispatch steps. The existing publisher, updater config, application identifier and application version are unchanged.
- Windows checkout disables automatic CRLF conversion so SQL bytes match the committed blobs. Before dependency installation/compilation, existing `scripts/build-provenance.mjs` verifies exact HEAD, a clean tracked tree, schema 26 and the ordered 1–26 catalogue. The added inline guard pins SHA-256 of `JSON.stringify(proof.migrations)` to **`3a2159765b0c77874143ef3143b6b97bd785d26d5372aa9250dae590be30bded`**, independently derived at the approved commit. This covers every historical version, name, filename and SQL hash. It also rejects missing/unregistered SQL files and updater configuration. No historical checks or migrations were changed.
- Reuses supported Node 22, pinned Rust 1.99.1, `npm ci`, Cargo `--locked` and the existing ordinary NSIS bundling path. Runs on `windows-2025`; no updater overlay, version rewriting or signing is used. `--no-sign --ci` makes that intent explicit. A failed command fails the job; exactly one nonempty NSIS executable is required before upload, and missing artifact paths fail.
- Installer and provenance are separate Actions artifacts, retained **14 days**. Evidence includes original SQL hashes, exact application/workflow commits, runner image, tool versions and installer SHA-256. There is no `latest.json`, release asset, signature or update-channel change.

This is source/dependency reproducibility, not a claim of byte-for-byte deterministic Windows executables: hosted Windows images, Node 22 patch versions, third-party action tags and downloaded NSIS/WebView2 tooling can advance. The captured environment and checksum identify the actual result. The installer remains the ordinary `0.1.0` package; its filename/provenance, not the app's version display alone, identifies this acceptance source.

## Starting a build later — owner action only

Do not perform these steps until the owner explicitly authorises a run and reviews runner billing. Nothing in this task dispatches Actions.

GitHub normally requires a `workflow_dispatch` workflow to exist on the default branch before its Actions button/API dispatch is available. This workflow is deliberately delivered on a non-main review branch. If the button is absent, **do not merge to main or repurpose the publishing workflow to work around it**. The owner must choose a separately authorised way to register/run this reviewed workflow. Selecting a non-default ref does not remove GitHub's registration prerequisite. This is a possible availability blocker, not a reason to expand this task.

Once available and authorised:

1. Open Actions → **Windows schema 26 acceptance build** → Run workflow.
2. Select the reviewed workflow ref, not `main` or the publish workflow. Record its exact SHA.
3. Leave `source_sha` at `6da1ea3d757d76b1072338f36c331dffc764b8ff`. It is the only allowed source. A new application source requires another reviewed workflow/digest change; branch names and short SHAs are rejected.
4. Wait for a successful run. Inspect the source/catalogue verification and build steps. A failed run is not an accepted installer, even if an earlier upload exists.
5. Record run URL, attempt number, workflow SHA, application SHA and status. The existing publish workflow listens to successful **CI push-to-main** runs, not this differently named manual workflow; this job cannot trigger it.

## Download and verify

On the successful run's Summary → Artifacts, download **both**:

- Installer artifact: `Abiding-Word-schema26-6da1ea3d757d76b1072338f36c331dffc764b8ff-Windows-x64`.
- Evidence artifact: `Abiding-Word-schema26-6da1ea3d757d76b1072338f36c331dffc764b8ff-provenance`.

Extract each ZIP into its own directory. The installer is:

`Abiding-Word-schema26-6da1ea3d757d76b1072338f36c331dffc764b8ff-x64-setup.exe`

The other artifact contains `build-provenance.json`, `build-environment.txt` and `installer-sha256.txt`. Check the JSON commit equals the full approved source, `dirty` is false, schemaVersion is 26 and all 26 migrations are present. Verify the executable before running it:

```powershell
$installer = (Resolve-Path 'C:\Acceptance\installer\Abiding-Word-schema26-6da1ea3d757d76b1072338f36c331dffc764b8ff-x64-setup.exe').Path
$expectedHash = ((Get-Content 'C:\Acceptance\evidence\installer-sha256.txt').Trim() -split '\s+')[0]
if ((Get-FileHash -LiteralPath $installer -Algorithm SHA256).Hash -ine $expectedHash) {
    throw 'Installer checksum differs from the run evidence'
}
```

Adjust only the download directories. Preserve evidence outside the disposable app-data folder. The hash links the file to this run; it is not a publisher signature. This is unsigned test software: Windows/SmartScreen may warn. Verify the run/source/checksum before following the organisation's approved test-software process; do not disable antivirus or organisation policy.

## Install without affecting the existing application

**Use a clean disposable Windows VM with no real Abiding Word installation/data. Leave the existing host application installed and untouched.** Take a VM snapshot first. A dedicated separate physical test machine is equivalent. Do not install this ordinary same-identifier NSIS package alongside an existing installation in the same account: it can replace binaries, shortcuts and uninstall registration. A custom destination or data override does not isolate installation identity. A separate Windows account alone is insufficient if an all-users installation could be affected; use the VM in that case.

The application has a verified source-level data override: `src-tauri/src/lib.rs::resolve_data_root` uses a nonempty **`ABIDING_WORD_DATA_DIR`** before the OS per-user path. All app-owned database/source/backup/export/log directories are derived from that root. An ordinary shortcut does **not** set this variable. Do not assume a new installer automatically uses a new profile. Actual Windows child-process inheritance and the displayed path must still be checked at acceptance.

In one PowerShell session **inside the VM**, before launching the installer:

```powershell
$acceptanceRoot = Join-Path $env:LOCALAPPDATA ('AW-schema26-acceptance-' + [guid]::NewGuid())
$profile = Join-Path $acceptanceRoot 'Data with spaces - 한글'
New-Item -ItemType Directory -Path $profile -Force | Out-Null
$env:ABIDING_WORD_DATA_DIR = $profile
if (-not (Test-Path -LiteralPath $env:ABIDING_WORD_DATA_DIR -PathType Container)) {
    throw 'Disposable profile missing'
}
# $installer is the previously checksum-verified executable.
Start-Process -FilePath $installer -Wait
```

Choose a per-user test installation **in the VM**. Do not uninstall the host's existing app. Untick **Run Abiding Word** at the end, if offered. The installer may create shortcuts; leave those unused. If it auto-launches, close it before continuing. The VM has no real data even if an installer launch fails to inherit the override.

Record the installed executable path, then start it explicitly from the same PowerShell session:

```powershell
$exe = Read-Host 'Full installed Abiding Word executable path (without surrounding quotes)'
if (-not (Test-Path -LiteralPath $exe -PathType Leaf)) { throw 'Executable not found' }
$env:ABIDING_WORD_DATA_DIR = $profile
Start-Process -FilePath $exe -WorkingDirectory (Split-Path -Parent $exe) -Wait
```

Before entering data, open Settings → Data and backup and compare the displayed data directory with `$profile`. Confirm the log's `data directory:` line matches and that `$profile\database\abiding-word.db` exists. If paths differ, stop and report failure. Settings must show no configured updater; do not install any offered update. Keep using this explicit launcher and **the same recorded profile** for restart tests. A new shell needs the same environment assignment; do not double-click a shortcut. Do not set a permanent machine-wide override.

For another fresh-install case, close every instance in the VM, create a new disposable root and repeat the explicit launch. Never point this binary at the host profile, a cloud-synchronised personal folder or a real backup. Installer removal is not a data rollback mechanism.

## Fresh and historical database checks

1. In the fresh installed profile, verify schema 26, first launch/restart, bundled local Bible reading, and successful project/Study/Teaching creation. Record startup errors and data path.
2. On the same test Windows machine with development tools, check out the **exact application SHA**, prepare Node 22/Rust 1.99.1 MSVC dependencies/WebView2, and run the synthetic migration matrix below. It creates fresh temporary on-disk schemas **19, 24 and 25**, seeds Studies/answers and the available legacy/block Teaching graph, opens them with the actual schema-26 startup code, checks FK/integrity/history/data and checks that three starts produce exactly one verified migration backup. The rollback tests inject failure and verify full rollback. No user database or edited ledger is required.

```powershell
git checkout --detach 6da1ea3d757d76b1072338f36c331dffc764b8ff
npm ci
npm run build
$env:CARGO_PROFILE_DEV_DEBUG = '0'
$env:CARGO_PROFILE_TEST_DEBUG = '0'
cargo +1.99.1 test --manifest-path src-tauri/Cargo.toml --locked --test schema26_review --test migration_v26 --test windows_dev44_compatibility --test migration_v25 --test portability -- --test-threads=1 --show-output
if ($LASTEXITCODE -ne 0) { throw 'Historical upgrade/rollback/portability matrix failed' }
```

3. Preserve the complete test output, exact counts and ignored tests. This exercises the native application's migration service on Windows, **not the installed executable's GUI upgrade path**. Tests clean up their temp databases; they are not persistent GUI fixture profiles. A separate installed-executable upgrade run still needs reviewed, wholly synthetic 19/24/25 profile packages (with source files and expected row counts) from the owner/test maintainer. This task does not provide those packages or claim that GUI upgrade acceptance is complete. Do not substitute a real user's backup or fabricate an older schema by deleting history rows. Report this gate as pending until those profiles are supplied and each is copied to a fresh disposable root, launched explicitly and compared before/after.
4. For each approved synthetic GUI fixture: with the app closed copy its **complete** folder into a new disposable root; preserve an untouched original. Set the override to the copy, launch the acceptance executable, compare expected Studies/answers/Teaching IDs/descendants/tombstones/quotes and outputs, verify schema 26 and one pre-migration backup, then restart twice and confirm no extra migration backup. Exercise study archive/delete/restore/unlink and project delete/restore. Restore only into disposable roots.
5. Schema 26 must not be edited by old binaries. Rollback requires closing all processes and restoring the complete untouched old folder/backup with its matching old binary. Never patch migration SQL/history to downgrade. Windows locks, WAL, denied destinations and interrupted restore must fail safely; preserve the untouched fixture for recovery.

## Manual functional acceptance

Use invented study text and redistributable public-domain sample documents. Record each row as PASS/FAIL/PENDING with evidence; a partial run is not a full pass.

| Area | Checks |
| --- | --- |
| Bible Study | Every offered method; seven-step POZETEK and ten-step writing; collapsed guidance, continuous writing, restart/autosave, personal templates and reusable research blocks. Verify exact edition/verse/quote provenance and atomic evidence retry; unavailable/mismatched editions must not silently substitute. |
| Teaching | Standalone from project; linked from Study; multiple sessions; link/unlink/relink; all three built-ins plus blank/personal and legacy v1 templates. Same Teaching remains accessible after Study deletion; restoration does not silently relink it. |
| Privacy and outputs | Put distinct public/private Scripture and a `LEADER ONLY` sentinel in notes. Inspect manuscript, worksheet, actual DOCX, saved/live presentation and second-display output. Participant/audience content must exclude private words and references. |
| Library and sources | Collections, nested filtering, tags and title/author/body search; archived → Trash → Undo remains archived; active Undo remains active. Reopen citations, PDF Reading/Original, EPUB and DOCX; traverse Sources/Scripture/Study and check in-session reading/return state. Missing sources retain exact saved quotes with explicit unavailability. |
| Portability | Back up synthetic work; export/import linked and standalone projects into a second disposable profile; compare relationships, citations, materials and presentations. Restore a full backup and compare. Test spaces/Unicode, cancelled native dialogs, denied paths and locked files. |
| Accessibility and languages | Tab/Shift+Tab, visible focus, Enter/Escape/cancel and focus restoration; narrow windows, 100/125/150/200% scaling and second screen. Check English, Deutsch, 한국어, Français, Español and Português for clipping/untranslated keys. |
| Offline | Prepare local Bibles/documents first, close/reopen with external networking disconnected, then read, navigate Library, edit Study/Teaching, autosave and export. Unavailable online downloads/assistant services must fail gracefully without blocking local work. No paid provider call is required. |

Document imported-Bible interface/provenance defects for Claude with a minimal redistributable example. Do not change recognition algorithms or treat this checklist as final Bible recognition acceptance.

## Diagnostics and reporting

- Record run URL/attempt, source/workflow SHA, installer SHA-256, Windows build, architecture, WebView2 version, display scaling, language, profile path (redact username) and test status.
- Local logs normally live in **`$profile\logs`** (`abiding-word` rotation files). If startup cannot create that root, logging falls back to the OS application log directory; use the startup error/Settings-reported log location rather than guessing a host path. Capture the relevant error/time window only.
- Logs, paths and screenshots can contain titles, names, document text, queries or prompts. Review/redact a copy before sharing; do not upload the whole profile, DB, backup, source PDFs/EPUBs or licensed Bible text. Prefer invented text, a minimal screenshot and a minimal reproducible sequence. Keep original diagnostic evidence private.
- Report: `case | PASS/FAIL/PENDING | expected | actual | reproduction | evidence filename`. Include commands and exact failing/ignored tests; an isolated success never erases a full-run failure. Mark native/installer/GUI checks separately.
- Stop on wrong data path, loss of descendants/answers, participant privacy leak, corrupt backup, unsafe rollback or unexpected updater. Keep the synthetic failing profile private for diagnosis and restore the clean VM snapshot if needed. No publication/update/production deployment is authorised by acceptance.

## Local validation and delivery

Local workflow validation passed:

- `actionlint` 1.7.7 on the new workflow: no findings.
- YAML/static policy checks: only manual trigger, read-only permission, pinned checkout, two artifact uploads, no secret/publish/dispatch/overlay paths.
- Actual embedded Node verifier: approved catalogue accepted; changed SQL hash, missing record, renamed historical record, extra SQL and configured updater rejected.
- PowerShell 7.4.6 on Linux: all run steps parsed; the real source-selection guard accepted the approved SHA and rejected empty/different SHAs. The actual provenance/catalogue command passed against the clean approved baseline. The actual staging step rejected missing, empty and multiple installers, and staged/checksummed one synthetic nonempty file. These files were invented bytes, not a Windows build.
- Changed-file boundary: only the new workflow and this guide; all application, recognition, historical migration, updater and publisher files are byte-identical to the required baseline.

Fresh repository checks ran sequentially on Node 24.19.0/npm 11.9.0 and Rust 1.99.1. Native checks used the existing Debian 13 dependency container with network disabled, locked/offline dependencies, debug information disabled and one test thread. Application source is the exact approved baseline; only these infrastructure/docs additions differ. Node 22 remains the supported existing workflow version and is selected for the Windows build. Evidence is retained in `/workspace/.cloud-setup/windows-acceptance/` (`workflow-check.log`, `powershell-check.log`, per-check logs and `checks.json`). Windows packaging, NSIS, actual Windows PowerShell execution, native dialogs and GUI execution remain untested until an authorised workflow/manual run; Linux PowerShell parsing/guard tests do not establish Windows acceptance.


| Check | Result | Seconds |
| --- | --- | ---: |
| TypeScript typecheck | PASS | 81.89 |
| ESLint | PASS | 57.71 |
| Six-language localisation | PASS | 1.98 |
| Derived localisation corpus | PASS | 0.98 |
| Full frontend suite (`--maxWorkers=2`) | 1,145 passed / 102 files; zero failed | 469.50 |
| Frontend production build | PASS; existing large-chunk advisory | 43.93 |
| Rust formatting | PASS | 30.35 |
| Clippy (`--all-targets -- -D warnings`) | PASS | 19.24 |
| Full Rust suite (`--no-fail-fast -- --test-threads=1 --show-output`) | 1,034 passed, zero failed, eight existing ignored | 623.79 |
| Rust development build | PASS | 35.28 |

All Rust summaries (85, including doc-tests) are included. Existing ignored tests were `document_units_dump`, `write_examples`, `real_document_editions_report`, `bundled_number_index_cost`, `oshb_corpus_reference_correctness`, `real_editions_report`, `write_interface_fixture`, and `a_whole_generated_annotated_bible_is_measured`; none is counted as passed. No tests or thresholds changed. The unchanged six-language Library indexing timing test passed in the complete run. No recognition acceptance is inferred.

This branch is ready for owner review of build preparation. The installer build, GitHub dispatch availability, real Windows/GUI migration acceptance and recognition acceptance remain pending. The two new files are committed with `[skip ci]` to avoid requesting automatic CI/native-build runs; no merge or Actions dispatch is part of delivery. The exact pushed infrastructure commit SHA is reported in the delivery response. The selected application commit remains the fixed SHA at the top of this guide.
