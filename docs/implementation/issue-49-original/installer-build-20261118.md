# Windows Original-view installer — approved build completed

**The single approved Windows x64 NSIS build succeeded on 18 November 2026, on attempt 1.** The downloaded installer and its application/resources have been independently verified. On 19 November the owner reported that the bottom-border problem no longer recurs; [the owner acceptance](owner-acceptance-20261119.md) closes that reported symptom. The build-time review below retains its original evidence and scope.

## Download and exact identities

[Download the installer ZIP](https://github.com/pistio/abidingword/actions/runs/44421251924/artifacts/14481669640) · [Successful build run](https://github.com/pistio/abidingword/actions/runs/44421251924) · [Complete verification record](installer-build-verification.json)

The GitHub artifact is retained until 2 December 2026. The owner also receives the same verified ZIP in chat. Extract the ZIP to obtain the normal setup executable; it is an unsigned Windows acceptance installer.

| Item | Identity |
| --- | --- |
| Application source | `539b54d65c2cecf010cd1573b8bd51c91ac640bd` |
| Workflow activation | `711f6dbe07c531fb410a5010fb25c0fd08cd80be` |
| Workflow content SHA-256 | `dcf0a088dcee5b39a5c419b80a4d2ed8795d68a340e50b8afc05d21407a8b4c8` |
| Fixed NSIS tools | `4bddc038d190cf42fd475b7c504430e96c014769` |
| Run / job | `44421251924` / `135214534225` |
| Attempt / application compilations / application packages | 1 / 1 / 1 |
| Schema / registered migrations | 27 / 27 |
| Installer size | 33,616,012 bytes |
| Installer SHA-256 | `1ade97d555b39bd6c215d1b6739cc4b23563b6e20eff6c0f9d2afcc867fffbbf` |
| Download ZIP size | 33,607,228 bytes |
| Download ZIP SHA-256 | `fbebcc5b8cb1aa635355a82bc49b0ab78806b36b3d87050ea650583fbd19ff60` |

Installer filename:

```text
Abiding-Word-Original-View-schema27-539b54d65c2cecf010cd1573b8bd51c91ac640bd-x64-setup.exe
```

The activation changed only the existing workflow file on `build/abidingword-i35-i42-20261109`. Exact-source and historical-migration checks passed before compilation. The NSIS preflight and cache probe also passed first. The application was compiled once with locked dependencies and signing disabled, then that output was packaged once. No workflow or application build retry occurred. The separate tiny NSIS toolchain probe is not an application compilation.

Build environment: Windows image `win25-vs2026 / 20261103.298.1`, x64; Node 24.22.1, npm 11.19.1, Rust 1.99.1. This is build-runner evidence, not execution on the owner's Windows 11 desktop.

## Artifact and payload verification

All three downloaded artifact ZIPs match GitHub's recorded sizes and SHA-256 digests, and their ZIP CRC checks pass. The installer ZIP contains exactly one setup executable. Its SHA-256 agrees with the checksum produced by the Windows build.

Read-only extraction on Linux verified the packaged x64 application and all **13 configured Scripture resources**. Every resource matches both the diagnostic manifest and exact application source. All **15 diagnostic inventory files** also match their recorded sizes and hashes. No application DLLs are listed or missing. The Tauri NSIS utility plugin matches its pinned preflight hash.

The packaged application is PE32+/x64. The NSIS bootstrap is PE32/x86, which does not change the application's x64 architecture. Neither PE has an embedded Authenticode certificate; signing was explicitly disabled in the build.

### Expected packaging change to the executable

The diagnostic executable is saved before Tauri adds the bundle type. Direct comparison of the actual prepackaging and packaged executables shows exactly three changed bytes, at offsets 24,663,916–24,663,918:

```text
__TAURI_BUNDLE_TYPE_VAR_UNK → __TAURI_BUNDLE_TYPE_VAR_NSS
```

This is the NSIS marker documented in the locked Tauri source and explicitly recorded in the bundling log. Both executables are 29,800,960 bytes; every other byte is identical. Their different raw hashes are retained, not treated as a mismatch or silently equated:

- Prepackaging executable: `ac79673d67134e66cc64a73bff0aa44d515fc828c57e8146d6880c21a0f8bf85`.
- Packaged executable: `36b7fb210d4d75f0c26c54afc5c642ac1e6ad3566834e59a1c752456e251d648`.

## Limits and recorded warnings

No Windows installer or application was launched during the 18 November build review. At that time installed-release acceptance was pending. The owner subsequently accepted the reported symptom on 19 November, as linked above. The accepted native comparison of diagnostic source `32e8a525` remains evidence for the captured EPUB/100% scenario, not a new installed-release measurement. Issue #49 is closed on the owner's report; PR #50 remains draft.

- `npm ci` reported 14 existing dependency vulnerabilities: 11 moderate and 3 high. Dependencies were not changed; advisory applicability was not assessed in this focused build/provenance review.
- Vite emitted the existing chunk-size warning. Pinned Actions emitted Node deprecation warnings. The synthetic NSIS probe reported an unused WebView label. All required steps completed successfully.
- 7-Zip reported `BadCmd=13` in its NSIS subtype metadata. Extraction completed successfully and direct byte/hash comparisons validate the application/resource contents; the installation script was not interpreted or executed.
- The NSIS utility DLL metadata lookup used the existing fallback to its official release asset, verified against the pinned hash on attempt 1. This was not an application-build retry.

This approval produced the installer. It did not authorise installation, merge, signing, updater publication, release or deployment, and it has not been reused for another build.
