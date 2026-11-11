# P0 owner Windows viewport cut-off — investigation and acceptance gate

**Issue:** [#49](https://github.com/pistio/abidingword/issues/49)  
**Draft code:** [PR #50](https://github.com/pistio/abidingword/pull/50), stacked on [PR #44](https://github.com/pistio/abidingword/pull/44)  
**Installed failing Windows application source:** `6ffed07de160b5168faf740cd4e1642ec5c29cf2`  
**Status:** **not Windows-validated; no new installer authorised.**

## Evidence

Owner's screenshot from 2026-11-09 is 1903 × 1019 physical pixels. All three vertically scrollable panes end at roughly y = 826, leaving approximately **193 px** of blank application background beneath them. This is not the result of short content: the Bible, POZETEK editor and Sources each have additional content below the cut-off.

The TSX source already uses `fixed inset-0 flex min-h-0` at the root (`src/app/Shell.tsx`), `flex-1 min-h-0` on the workspace and pane bodies, and `html, body, #root { height: 100% }`. These facts make a shared parent/native WebView2 viewport sizing problem plausible, **but they do not prove which layer is wrong**. A screenshot alone cannot distinguish the HTML layout viewport from the WebView2 controller's physical bounds or its native child HWND. DO NOT use a guessed `193px` offset, `height: 100vh` patch, screen-height multiplication or forced parent-window resizing.

## Narrow provisional repair under review

- Tauri v2 supports `WebviewWindow.as_ref(): &Webview`, `Webview::size`, `set_size`, and `set_auto_resize`. This branch restores auto-resize **from a worker**, reads both physical sizes, and requests a webview-only `set_size` *only on an actual difference greater than 2 physical pixels*. It does **not** force a resize on focus when the sizes match.
- Tauri's native focus/resize callbacks schedule delayed, coalesced checks outside the synchronous callback. No new Tauri frontend command permission. The audience window is never touched.
- Log output includes an initial `Main window geometry: client=..., webview=..., scale=...` snapshot (not writing/user/source data) and mismatch repairs/errors. It does not assert a successfully repainted WebView2 compositor just because `set_size` returned successfully.
- Pure policy tests cover the owner-shaped gap, DPI rounding, zero dimensions and width mismatch. These are not native WebView2 tests.
- `scripts/windows/Measure-WebviewGeometry.ps1` optionally records the parent-client and child-HWND physical rectangles, visibility, Win32 class and DPI, with no titles, URLs, document text, screenshots, paths or credentials. It **requires a running app on an interactive Windows desktop**, ideally using invented data. It is not usable as evidence if a CI session lacks such a desktop.

## Hard acceptance matrix — must complete before asking owner to smoke-test

Use a **disposable/invented** profile; do not instrument the owner's installed profile or collect its text.

1. **Verify actual API and build:** Codex Cloud independently verify the exact pinned Tauri 2.x/Wry APIs and threading behaviour; run Rust formatting/Clippy, full local Rust and frontend tests, plus Windows x64 compilation on a real Windows builder when approved. Check that coalescing never loses a final resize event, never runs when minimised, does not deadlock the Win32 message loop, and cannot resize the native window.
2. **Measure native independently:** for each scenario, record Win32 parent client rect, DPI, actual WebView2 child/controller rect if available, and the Tauri `Webview::size()` value. If the runtime reports already matching sizes despite the cut-off, the current repair is inadequate; investigate native compositor bounds and DOM instead. Do not keep a speculative resize.
3. **Measure DOM:** on the same actual Windows executable, record `window.innerWidth/innerHeight`, `devicePixelRatio`, `document.documentElement.clientHeight`, the rectangles of `[data-testid=app-shell]`, `[data-workspace-layout]`, `#pane-scripture`, `#pane-study`, `#pane-sources`, and all scroll roots. Compare bottom edges in physical screen coordinates using client-origin offsets and DPR.
4. **Exercise real Windows states:** initial launch; focus loss/restoration; maximise; restore; dragged taller/shorter; two/three panes; 100%, 125%, 150%, 200% Windows scaling (or explicitly report scenarios not tested). Repeat after opening an actual Source and editing an invented Study. Check scroll continues to the bottom and caret/focus/draft text remain unchanged.
5. **Proof:** save independent before/after geometry JSON and **privacy-safe** Windows screenshots with invented content. Every pane's bottom must reach the window's **client** bottom within a small physical-pixel tolerance. A Linux Chromium synthetic run alone is insufficient. The previous screenshot is the negative-control baseline. Include evidence of **no 193px gap**.
6. **Only then:** code-review and consider an owner-approved signed-off Windows installer in an isolated branch, reusing the safe pinned NSIS preflight from PR #39 with no updater/release. No owner MacArthur/Study smoke test until the viewport gate is independently shown green.

## Additional cautions

A Tauri `WebviewWindow` normally hosts one webview sized to its parent automatically. Avoid recursive `set_size` storms; verify that the new repair is necessary for this app's exact bug. Windows native HWND boundaries, browser visual viewport, and fixed/root CSS can differ at high DPI. This branch remains **draft and exploratory** until independent Windows evidence exists.

The already approved 10-step study, user writing, schema 27, historical migrations, separately frozen Scripture recogniser, and external PDFium helper must not be changed by this fix. No GitHub Actions paid dispatch, merge, release, deployment or updater from this work without owner approval.
