# Native Original-view comparison and installer preparation

## Accepted diagnostic evidence

Independent review on 18 November 2026 accepts the first complete native comparison for the captured EPUB Reading → Original scenario on Windows 11 at 100% scaling. This is evidence for the tested diagnostic executable, not a claim that the future installer has been installed or that every format, DPI and monitor arrangement has been tested.

| Item | Recorded identity |
| --- | --- |
| Baseline application | `624ef9f357bcc956275e70bbd8ae371f1eba89f9` |
| Baseline executable SHA-256 | `0413927c456653deb2a30406a79ba192ae795af475928a837729ab73440716dc` |
| Corrected application and shared protocol | `32e8a52567c3863f9b2930125f711fba6b7bd9ef` |
| Corrected executable SHA-256 | `5161603e41618c8807a033b857c6e9b1ea9d4eb107c79862a15ab1b8e4c9e0f3` |
| Windows | Windows 11 Home 10.0.26300, x64 |
| WebView2 | 157.0.4341.46 |
| Display / native client | 1920×1080 / maximised 1920×1009 |
| Scaling | 96 DPI, DPR 1, browser zoom 1 |
| Native intervention | Disabled in actual recorded samples |

The first archive, `2026-11-18T22-18-51-726Z-7161b889-evidence.zip`, has SHA-256 `15522e8a4c69045462400e8ef840ffa6e5c108c74c7c8a9bb7262dcb6ccc3ccc`. It contains two baseline native screenshots and twelve corrected native screenshots, all visually reviewed. Archive CRC checks pass. Source and executable identities in the reports agree with the package manifest and the previously recorded build provenance. Executables and the complete 1,096-file distribution were not reuploaded in these evidence archives, so this review is not a new distribution checksum pass.

| Measurement | Baseline Reading | Baseline Original | Corrected, all 12 states |
| --- | ---: | ---: | ---: |
| Shell scrollTop | 0 | 206 | 0 |
| Workspace / pane top | 56 | −150 | 56 |
| All three pane bottoms | 1009 | 803 | 1009 |
| Shared bottom band | 0px | 206px | 0px |
| Application and reader controls | Visible | Hidden | Visible |
| Native client / browser height | 1009 | 1009 | 1009 |

The baseline preserves the first failed Original state and stops. Its native window and browser surface remain full-sized. The displacement is the application's outer scroll offset, corroborating the previously reproduced reader-navigation mechanism. No healing resize or native repair is logged after the failure.

The corrected sequence passes deep Reading/Original, keyboard footnote/backlink, Back/Forward, Contents footnotes/passage, highlight reveal, return to Reading/Original and Tab navigation. All three pane-scroll checks, edit value/caret preservation and saved-edit readback pass. The corrected screenshots visibly show the approved **People** heading.

Raw native logs contain only observed phases and `repairEnabled:false`. Per-state COM values are the latest preceding native-event sample, sequence 3, explicitly marked reused after the first capture. They are not twelve newly sampled COM observations. Fresh per-state HWND and DOM measurements corroborate the native screenshots.

The invented USFM and EPUB bytes were independently rehashed and match across both builds and both runs:

- USFM: `53375a3d9383439eb56d4007aadb9de2a26a641c39e9d1b47a0ca5d95cfbd046`.
- EPUB: `d979ce7473f96cf96557ef6aa979deca7e475f075af7e2d01ae648916d7d7d65`.

Owner screenshots, private desktop notifications and local owner paths are not published in this repository. A small notification overlaps part of the baseline screenshot's lower-right band; the shared boundary and most of the band remain clearly visible.

## Second run retained as incomplete native capture

`2026-11-18T22-21-10-875Z-81ff20b4-evidence.zip`, SHA-256 `dad671755ad1d541ec092e09df4daea30eb958551bba3f84a80a71d1208b3c77`, reproduces the same baseline defect. Corrected geometry and containment pass 12/12 and scrolling/editing/save checks pass, but all twelve native screenshots are absent. Each capture was rejected with `App must be unobscured in foreground on a clean test desktop`.

The runner correctly exits 1 and classifies this as a setup/measurement failure. The evidence does not identify why the foreground check failed. This is not another native pass and does not establish a layout regression. The first complete run already supplies the required evidence; no repeat is needed solely for the second run's missing captures.

## Production cleanup after the accepted diagnostic

The accepted comparison disabled the speculative Rust size intervention. Before preparing the normal installer, that intervention is removed from source:

- Delete its mismatch threshold, repair-mode/default setting, resizing call, repair readback and four obsolete repair-policy tests.
- Retain the existing read-only COM/native geometry collector. It requires `ABIDING_WORD_VIEWPORT_TRACE=1` before any diagnostic atomics, worker creation or geometry measurements. Ordinary launches use Tauri's normal sizing.
- Keep the log schema, with `repairEnabled:false` and phase `observed`; no setting can re-enable a removed resize setter.
- Keep the native diagnostic CLI compatible with `--mode observe`, reject repair mode, and require actual recorded proof that native repair is disabled. Preserve the historical baseline's observe environment setting so it remains a valid negative control.

The reader scrolling correction, four People translations, application configuration, dependency locks, Scripture resources, recognition logic, migrations and audience-window close behavior are preserved. Retaining the collector avoids weakening the existing acceptance requirement for actual COM bounds.

The cleanup creates a new source candidate. The Windows evidence above remains evidence for the unchanged frontend correction at `32e8a525`; it does not retroactively verify the new candidate's compilation or installed behavior.

## Normal installer pathway

Reuse the existing Windows-only NSIS workflow from successful run [42713667771](https://github.com/pistio/abidingword/actions/runs/42713667771), workflow/packaging revision `4bddc038d190cf42fd475b7c504430e96c014769`. Keep the reviewed NSIS preflight and cache probe before release compilation, and package the existing release executable into one unsigned NSIS installer. Preserve the exact source, historical 27-migration digest, updater-absence and checksum checks.

An inert workflow draft is prepared outside `.github/workflows`; final activation still requires the separately authorised installer build. Issue #49 stays open and PR #50 stays draft while the remaining checks and installed-release acceptance are pending. No new workflow run, installer, merge, signing, release or deployment is part of this preparation.
