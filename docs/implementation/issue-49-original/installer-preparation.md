# Windows Original-view installer preparation — 18 November 2026

**The single approved Windows installer build has completed successfully.** See the [installer download and verification](installer-build-20261118.md). This work belongs in the cloud development/review environment. It requires no commands on NiPogi or the owner's Windows desktop at this stage.

## Exact candidate

Application cleanup commit: `539b54d65c2cecf010cd1573b8bd51c91ac640bd`, based on the accepted diagnostic source `32e8a52567c3863f9b2930125f711fba6b7bd9ef`.

The cleanup removes the native size setter and repair policy, preserves opt-in read-only COM measurements, restricts diagnostics to observe mode, and fixes diagnostic artifact source metadata. Production frontend assets, the reader correction, People translations, dependency locks, schema/migrations, resource mappings and application identity remain unchanged from the accepted diagnostic source.

The candidate has passed the required frontend and Linux Rust verification gates. Rust and helper checks ran at local checkout `0021c18a7b754b487b5ca9e1e189f6d4a7c50db7`; its application, diagnostic and test files are identical to the published candidate. Publication through the authenticated GitHub integration assigned a new commit ID while preserving the complete candidate Git tree `f06b161d5017d105980ce9c94dc4337e58e6ee53`. The verification record preserves both the tested local identity and the canonical published identity. The final workflow was repinned and its PowerShell blocks revalidated. The earlier Windows evidence applies to the unchanged frontend correction with repair disabled, not to a newly built installer.

## Prepared workflow

The complete [installer workflow draft](installer-workflow.yml.in) pins all six application references to the candidate SHA. It is deliberately outside `.github/workflows` and cannot run from this location.

The owner-approved build activated the following existing pathway:

| Field | Value |
| --- | --- |
| Repository | `pistio/abidingword` |
| Existing activation branch | `build/abidingword-i35-i42-20261109` |
| Existing active workflow path | `.github/workflows/windows-combined-owner-20261109.yml` |
| Previous successful workflow / NSIS tools | `4bddc038d190cf42fd475b7c504430e96c014769` |
| Jobs | One Windows job on `windows-2025`, 60-minute limit |
| Application build | Locked unsigned release executable, no bundling during compilation |
| Packaging | NSIS only, using the already compiled release executable |
| Result | One unsigned Windows x64 setup executable, SHA-256 and source evidence |
| Signing, updater, release, deployment, automatic retry | None |

Do not rerun the old installer job: its old source pins are intentional. The prepared draft is a reviewed adaptation of that pathway, with exact candidate pins, the same migration catalogue guard, and the established NSIS acquisition/cache probe before application compilation.

The old packaging helper hardcoded the previous `6ffed07…` source in its diagnostic notice and manifest. The candidate therefore includes a narrowly updated `Save-DiagnosticExecutable.ps1` which checks the actual application checkout against the expected full SHA before writing files. The draft calls that candidate helper; NSIS preparation and probing continue to use the previously verified fixed packaging revision.

## Verification completed before the Windows build

The [verification record](installer-preparation-verification.json) records the tested file hashes and limits.

- Full frontend suite: **1,299 passed, zero failed, 121 test files**.
- Typecheck, ESLint, translation consistency and production frontend build pass. The existing chunk-size warning remains.
- Updated Node diagnostic syntax and help output pass.
- All 28 archived geometry verdicts remain unchanged under the strengthened observation-only comparator. This is evidence replay, not new Windows execution.
- Independent source and workflow review found and resolved the diagnostic source-label error.
- Final draft YAML, six exact source pins, trigger scope, one-job limit, packaging order, migration digest, fixed tool checkout, diagnostic helper call and unsigned NSIS output checks pass locally.
- Rust formatting and all-target Clippy with `--locked -- -D warnings` pass.
- Full locked Rust suite: **1,103 passed, zero failed, 8 existing ignored**. Four obsolete native repair-policy tests were deleted with that intervention, accounting for the reduction from the earlier 1,107. No additional test is ignored. [Rust verification details](installer-rust-validation.json) record every result group, command, actual checkout, environment and log hashes.
- PowerShell 7.5.4 validates all four helpers and all eleven PowerShell blocks from the exact resolved workflow. Five synthetic staging checks pass: correct source in notice/manifest, correct file hashes/sizes, truthful completeness, and wrong/malformed SHA rejection before output creation. The invented executable/DLL fixture files were never run. [PowerShell verification details](installer-powershell-validation.json) retain the checked input hashes and runtime verification.
- The complete migration catalogue is unchanged: schema 27, exactly 27 registered SQL files, guard SHA-256 `acb4707b448735285346834b3cab04caa61fe3696079b84a7e5d9c35b4cd4e13`.

Rust 1.99.1 and required Linux development libraries were set up in the owner-authorised cloud workspace. These Linux checks completed the repository's pre-push gate. The Windows-only native collector subsequently compiled successfully in the approved Windows build. PowerShell parsing and synthetic helper execution do not establish Windows packaging or installation success.

The reviewed workflow SHA-256 is `dcf0a088dcee5b39a5c419b80a4d2ed8795d68a340e50b8afc05d21407a8b4c8`. This template remains under documentation; identical bytes were activated at `711f6dbe07c531fb410a5010fb25c0fd08cd80be` on the existing build branch after the owner approved one build. Run `44421251924` succeeded on attempt 1, with one application compilation and one NSIS packaging operation. [Installer verification](installer-build-20261118.md) records the downloaded files, checksums and exact expected packaging change. No retry, merge, signing, updater, release or deployment occurred. Installation and installed-release acceptance remain pending. No further diagnostic repeat is required solely to replace the already accepted first comparison.
