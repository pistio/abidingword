# Issue #49: Windows viewport investigation and acceptance gate

> 13 November update: the actual Source → Original interaction now reproduces a shared blank band in the real React workspace. See [the scroll-containment investigation and targeted Windows protocol](issue-49-original/README.md). The earlier resize-only results below are historical and do not exercise that trigger. Native acceptance of the new correction remains pending.

[Draft PR #50](https://github.com/pistio/abidingword/pull/50), stacked on [PR #44](https://github.com/pistio/abidingword/pull/44).

**2026-11-12: NOT FIXED / Windows acceptance BLOCKED.** This execution environment is Linux, with no attached Windows desktop or WebView2 runtime. No native Windows measurements or before/after screenshots were obtained. No paid Actions, installer, signing, release, deployment or merge was started. The reproducible Windows runner below is prepared but has not executed on Windows.

Reviewed requested commit `d9e4ce21bad16f26af6c4164c69641c8b25716bf`; retained its subsequent logger-order and client-origin corrections through `4f2ff35b61a327b37dc81b6a17f1027bc53f68b6`. This is a review and instrumentation correction, not evidence that the owner-visible defect is repaired.

## Findings and implementation corrections

Owner evidence is an installed build from `6ffed07de160b5168faf740cd4e1642ec5c29cf2`: a 1903×1019 screenshot with panes ending near y826 and approximately 193px blank below. These are owner-reported screenshot observations, not this run's native measurements. The roughly 806px rendered surface is only suggestive of the configured 800 **logical** pixel height. Unknown DPI, title-bar/client geometry and controller bounds prevent that inference from becoming a diagnosis.

Reviewed the actual locked crate sources: Tauri/tauri-runtime/tauri-runtime-wry **2.12.0**, Wry **0.57.0**, Tao **0.37.1**, webview2-com **0.39.1**, windows **0.62.2**. No dependency changes.

- `Webview::size()` and `bounds()` dispatch requests and synchronously wait for replies. Keep these calls on the background worker, outside the synchronous window-event callback. Actual COM access now uses `with_webview` on the owning UI thread; only plain JSON crosses back through a channel with a bounded timeout.
- Wry's `WindowContent` bounds getter reads controller bounds but returns a **zero position**, losing actual left/top. The new diagnostic reads actual COM `Bounds`, `ParentWindow` and `ZoomFactor`; independent Win32 capture identifies that parent HWND and its physical client origin/size. Matching widths/heights alone cannot exclude a displaced or clipped child HWND.
- `set_auto_resize(true)` is documented as a no-op for `WebviewWindow`; the earlier removal is correct. Upstream already handles parent WM_SIZE and sizes its child `WRY_WEBVIEW` host/controller.
- `set_size` queues work; an `Ok` return is not backend `SetBounds` success or repaint evidence. The worker now records subsequent measurements without logging a successful fix. In the pinned implementation this changes the embedded controller/child host, not the outer application window.
- The provisional repair remains limited to a measured difference over two physical pixels. Added a stale-revision/client recheck and explicit `ScaleFactorChanged` scheduling. Minimized/zero-size windows and the audience window remain excluded. Coalescer handoff was reviewed for the final-event race; real Windows event-storm/deadlock behaviour remains untested. Its 40ms debounce is not a height or paint workaround.
- `ABIDING_WORD_VIEWPORT_MODE=observe` disables correction for an A/B run. `repair` enables the existing measured correction; unset preserves that draft default, unknown explicit values fail closed to observation. `ABIDING_WORD_VIEWPORT_TRACE=1` records dimension-only samples for each scheduled observation. Neither mode changes native outer-window size.
- The PowerShell collector now establishes/restores per-monitor-v2 thread DPI awareness, fixes incorrect interpretation of `EnumChildWindows`' return value, records monitor geometry, and optionally captures the **actual native client** from the screen. It refuses clipped/off-monitor captures. A CDP screenshot alone cannot prove the Windows compositor painted the full client.

No production CSS/layout, migrations, recogniser, Study/Teaching logic, Scripture resources, permissions, updater or workflow was changed. Test fixture changes live only under `tests/`; invented USFM content explicitly says it is not Scripture.

## Verification performed

Full commands and results are recorded in [verification.json](issue-49/verification.json) and the adjacent logs.

| Check | Result | Limit |
| --- | --- | --- |
| Rust formatting | PASS | `cargo fmt -- --check` |
| Linux Clippy, all targets, warnings denied | PASS | No Windows runtime |
| Full Rust tests | 1107 passed, 0 failed, 8 existing ignored | Linux execution |
| Windows GNU target `cargo check --all-targets --locked` | PASS | Cross check, not Windows execution or MSVC linking |
| Windows GNU target Clippy, warnings denied | PASS | Compiles Windows-only API/COM code, not a repaint test |
| Frontend typecheck, ESLint, i18n audit, production build | PASS | Existing bundle-size warning remains |
| Full frontend tests | 1297 passed, 0 failed | Includes 8 geometry-policy cases, not native evidence |
| Original vertical browser contract before edits | 10/10 passed | Already green; cannot be the native failure reproduction |
| Expanded vertical contract | 40/40 passed | Chromium, emulated DPR 1/1.25/1.5/2; widths 900/1440; heights 800→1019→670→1130→800 |
| Collector C# / PowerShell syntax; runner Node syntax | PASS | Static checks on Linux PS 7.5.4, no Win32 invocation |
| Windows launch/maximise/restore/resize/OS DPI | **NOT RUN** | No Windows executor attached |
| Native before/after screenshots and geometry | **MISSING** | Critical acceptance blocker |

The expanded browser test loads long invented Scripture/Study/Source fixtures, checks all pane bottoms and real overflowing scroll containers, and retains edited text/focus/caret during resizing. [JSON and 40 screenshots](issue-49/browser-contract.json) are browser evidence only. [Initial browser evidence](issue-49/browser-initial.json) demonstrates the pre-existing browser contract was already green. There is no native before/after comparison to infer from these images.

## Reproducible native Windows protocol (prepared, NOT executed)

Use an existing approved disposable interactive Windows 11 desktop with sufficient resolution at each scaling setting, WebView2 installed, Rust/MSVC+Windows SDK, Node/npm, Git and PowerShell 7.2+. Do not run on the owner's installed profile. The runner creates a new profile, verifies `app_info` uses it, checks schema 27 and updater disabled, then imports only invented data through the existing application commands. It never reuses another process or existing output directory.

On an approved Windows builder, check out the final PR head SHA and keep application sources clean. Build a plain debug executable with embedded frontend through the Tauri CLI, **no bundle/installer/signing**:

```powershell
$Pinned = '<full reviewed PR head SHA>'
git checkout --detach $Pinned
npm ci
New-Item -ItemType Directory -Force work | Out-Null
node scripts/build-provenance.mjs --expected-sha $Pinned --require-clean --out work/viewport-source.json
npx tauri build --debug --no-bundle --no-sign --ci -- --locked
npm install --prefix work/viewport-tools playwright@1.65.0
$env:PLAYWRIGHT_MODULE = (Resolve-Path work/viewport-tools/node_modules/playwright).Path
$Exe = (Resolve-Path src-tauri/target/debug/abiding-word.exe).Path
$Hash = (Get-FileHash -Algorithm SHA256 $Exe).Hash.ToLowerInvariant()
node scripts/windows/Test-WebviewViewport.mjs --exe $Exe --exe-sha256 $Hash --source-sha $Pinned --mode observe --expected-dpi 96 --out work/native-96-observe
node scripts/windows/Test-WebviewViewport.mjs --exe $Exe --exe-sha256 $Hash --source-sha $Pinned --mode repair --expected-dpi 96 --out work/native-96-repair
```

Run each command separately and retain failed observation evidence. Repeat with actual **Windows** display scaling 125%, 150%, 200% and expected DPI 120, 144, 192, each in a new output directory. Do not use Chromium DPR emulation or force-device-scale-factor as a substitute. Keep the app fully on one monitor and an unobscured, clean test desktop. Localhost CDP is enabled only for the launched test process and closed with it; never use the owner's production profile. Windows executable hash and caller-pinned clean source SHA are recorded; retain the local build log to associate that binary with source.

**Wide-layout prerequisite:** the P0 screenshot is a three-column desktop defect. The shorter/taller native height transitions therefore retain an outer width of 1440 logical pixels at the measured scaling factor. A narrow/stacked layout has intentionally different pane-height behaviour and cannot be used as positive three-pane viewport evidence. The comparator now records `data-workspace-layout` and explicitly refuses a `stacked` result. On high-DPI monitors, use a large enough clean desktop for the requested physical width; report insufficient resolution instead of counting an unintended narrow layout as successful native acceptance.

The runner measures launch, maximise, restore, shorter/taller native outer sizes, maximise again and restore again. It records:

- independently DPI-aware Win32 main client/child HWND rectangles, screen origins, visibility, classes and monitor;
- actual UI-thread COM controller rectangle/parent/zoom and Tauri physical client/webview samples;
- browser viewport/DPR/visualViewport/document height, shell/workspace/all three pane rectangles and scroll roots;
- native client PNGs plus separate browser PNGs, real overflowing-pane scroll checks, editor text/focus/caret retention and saved-edit readback;
- event sequence growth, missing native measurements, DPI mismatches and screenshot failures as failed gates.

A green geometry report still requires independent **native screenshot review**. The screenshots must show all three panes painted to the native client bottom, with no grey band, and scrolling/editing must pass. If the observe mode does not reproduce the actual defect, repair-mode success is not before/after proof. Also manually exercise focus loss/restoration, dragged resizes, two/three-pane transitions, cross-monitor DPI changes and sustained resize bursts; the automated stages alone do not cover these acceptance cases. Preserve all logs and describe any untested scenario.

## Diagnosis decision and remaining blockers

1. If native client is taller than actual controller/child host, evaluate the measured correction using observe/repair evidence. Determine whether its height is actually near 800 logical pixels using measured DPI, not screenshot guesses.
2. If native/controller sizes match, **do not apply repeated resize requests**. Compare child-host clipping/origins, browser innerHeight, visual viewport and document/root/pane bottoms. The geometry comparator rejects a clipped native host even when controller and client sizes agree.
3. If all geometry matches but native screenshots retain the grey band, investigate WebView2 composition/paint rather than claiming a layout fix. Compare independent native client captures with CDP images and collect Windows/WebView2 version/graphics information before choosing another implementation.

Currently neither the 800px hypothesis nor a DOM/compositor cause has been established. The prepared runner's native IPC, GDI capture, focus behaviour and actual WebView2 integration remain unvalidated. A real Windows environment is required to make the requested before/after evidence and complete acceptance. Keep Issue #49 open and PR #50 draft; no owner teaching-material smoke test, installer, paid Actions, merge or release follows from the checks above.
