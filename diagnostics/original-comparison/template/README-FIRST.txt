ABIDING WORD — ISSUE 49: READING -> ORIGINAL COMPARISON

This ZIP contains two plain unsigned DEBUG executables and the SAME final
comparison protocol. It is not an installer and does not update the installed
Abiding Word application. It does not connect to NiPogi, use SSH, install Rust,
or copy a personal profile. No network access/download is needed by the launcher.

1. Extract the ENTIRE ZIP to a writable local folder. Do not run inside the ZIP.
2. Use the existing Windows 11 desktop at its current 100% display scaling.
   Keep the app visible and unobscured. Close confidential overlapping windows.
3. Double-click Run-Comparison.cmd. Allow the two automated runs to finish;
   do not resize, click, type into or cover the app while they run.
4. The launcher prints one evidence ZIP path under results. Retain/share that
   ZIP for independent engineering review. A baseline failure is expected only
   when the complete measurements demonstrate the actual layout defect.

The baseline executable is the already-built debug source:
  624ef9f357bcc956275e70bbd8ae371f1eba89f9
  GitHub run 43405180688 (reused, not rebuilt)
The corrected debug executable is the single authorised build of:
  32e8a52567c3863f9b2930125f711fba6b7bd9ef
Exact executable SHA-256, build/source provenance and package inventory are in
package-manifest.json and provenance/.

Both runs explicitly use --scenario original --mode observe. The runner sets
ABIDING_WORD_VIEWPORT_MODE=observe and requires the native diagnostic log to
confirm repairEnabled=false. A native resize correction cannot earn acceptance.
Each app receives a new disposable profile containing identical invented USFM,
EPUB, Study writing and a highlight. The profiles remain under results locally;
the evidence ZIP excludes WebView/profile caches and includes input fixtures,
measurements, allowlisted raw native geometry-log lines, native screenshots and verification reports.

The actual path is populated Study -> formatted Source -> deep Reading ->
Original. The baseline's FIRST failing Original state is captured before any
resize or recovery. On a valid corrected run the protocol continues through
footnotes/backlinks, Contents, Back/Forward, highlight reveal, Reading return,
Tab navigation, all three pane scrollers, and saved-writing verification.

Interpretation:
- SETUP_OR_MEASUREMENT_FAILURE means missing/invalid setup or evidence; it is
  NOT a reproduction of the blank band.
- BASELINE_DID_NOT_REPRODUCE means the blank band was not demonstrated in this
  run. A corrected pass then cannot establish Windows acceptance.
- A measured baseline reproduction and corrected pass are still PENDING
  independent native screenshot review. The launcher never declares Issue 49 fixed.

If setup fails, keep the evidence/logs; do not install tools, reset profiles,
change display scaling or run additional builds to work around the failure.
The package uses portable bundled Node, PowerShell and Playwright's CDP client;
it reuses the system's existing WebView2 runtime. It does not install a browser.

COM timing: Win32 and DOM geometry are captured at each interaction. COM bounds
come from the latest logged UI-thread native-event sample (often maximisation),
which may be reused across interactions. Reports retain sequence/phase and
reused-sample flags plus raw timestamped geometry lines. No resize or focus
change is forced to refresh COM before preserving the first failure.
