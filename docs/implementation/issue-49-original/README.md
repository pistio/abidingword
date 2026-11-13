# Issue #49: Reading → Original scroll containment

Status: browser reproduction corrected and independently reviewed; **Windows acceptance remains open**. Work is in the development checkout, not the installed application or NiPogi. No build, installer, merge, release or deployment was started.

## Reproduced mechanism

The real React workspace, using invented formatted content through the memory API, reproduces the reported class of failure. The crucial condition is an open, populated Study alongside Scripture and Sources. A deep Reading position alone, without that Study, did not reproduce it; that unsuccessful probe is retained separately.

At 1920×1009 CSS pixels, selecting Original at paragraph 40 moves `app-shell.scrollTop` from 0 to 206. The shell rectangle and browser viewport remain unchanged, but workspace/pane top moves from 56 to −150 and every pane bottom from 1009 to 803. The reader toolbar moves to −102…−53 and disappears; the application toolbar also disappears. This leaves a shared 206px band. This is a real application browser reproduction, not an asserted measurement of the owner's 193px/201px Windows screenshots.

The formatted viewer's unrestricted `scrollIntoView` crosses its shadow root and scrolls the enclosing shell. An independent probe identifies the available shell scroll range: visually hidden Study labels have the fixed shell as their containing block and extend its scroll height to 1744 despite a 1009px client. See [contributing-layout.json](contributing-layout.json). Its temporary label-hiding counterfactual is diagnostic only; no production labels or Study CSS were changed.

## Correction and scope

The supplied candidate was reviewed and integrated at the two formatted EPUB/DOCX navigation sites: fragment arrival and highlight reveal. `scrollInViewport` keeps browser-native target alignment inside the reader and synchronously restores ancestors outside it, including across shadow roots and the document window. Explicit instant scrolling prevents later CSS smooth-scroll movement. Existing fragment-offset correction and keyboard `focus({preventScroll:true})` remain intact.

No hardcoded band height, CSS viewport-size change, forced window resize or global scroll handler is added. The prior Rust resize intervention is unchanged and receives no credit for this DOM correction. Native verification explicitly requires observation mode. PDF Original is a separate path: the exact format of the owner's document is still unconfirmed, so this evidence establishes the EPUB/DOCX defect, not every possible Original format.

The separate approved English wording change replaces “Persons” with “People” in exactly four POZETEK translation leaves (two headings, two descriptions). Internal `persons` identifiers, German wording, saved writing, Scripture recognition and migrations remain unchanged.

## Evidence and regression

The browser regression uses actual React components and real browser layout, with only backend data supplied by the existing test memory API. The fixture includes formatted paragraphs, a deep passage anchor, footnote/backlink, Contents entries and a highlight. No application CSS is modified to cause the failure. It records shell/workspace/pane top and bottom, toolbar visibility, ancestor/window scroll offsets and extents, reader location and visual viewport before assertions. For a repeat run, start the existing screenshot Vite server on port 1432, then run `PLAYWRIGHT_MODULE=/path/to/playwright node tests/screenshots/original-workspace.mjs`; default output goes to `work/original-workspace` so historical captures are preserved.

The first failing Original state is preserved before any navigation can restore the shell.

- [Before evidence](before/results.json): baseline formatted viewer from `624ef9f357bcc956275e70bbd8ae371f1eba89f9`; test fixtures and the separate wording correction were already present.
- [After evidence](after/results.json): corrected viewer, same interaction and geometry expectations.
- [Synthetic helper qualification](helper.json): 40 cases, including 24 unrestricted negative controls. These complement, and do not replace, the application reproduction.
- [Verification](verification.json): exact commands, suite totals and limitations.

**Final browser result: 8/8 cases, 96/96 recorded states pass** (EPUB/DOCX × 1920×1009 DPR1 and 1440×800 DPR1/1.5/2). Both populated-Study baseline cases fail; the unchanged no-Study control passes 11 steps. All measurement JSON and 26 representative after screenshots are retained; see [the evidence index](interaction-evidence-index.json).

The shared flow covers nonzero Reading → Original, return to Reading, keyboard footnote/backlink, Contents, Back/Forward, highlight reveal and retained Study writing. Browser DPR variants are emulation, not actual Windows OS scaling. The original generic resize-only kit is not acceptance evidence for this correction.

## Verification outcome

- Rustfmt/Clippy pass; Rust suite: **1107 passed, 8 existing ignored**.
- Full frontend run: **1297 passed, 1 failed** solely because the generated translation corpus was stale. Regenerated its four approved English entries; corpus retest **26/26 passed**. All **1298 distinct frontend tests** are verified on the final application code; no second full run is claimed.
- Typecheck, ESLint, translation consistency and production build pass. Existing bundle-size warning remains.
- Actual workspace: **2 expected baseline failures, 8 corrected cases / 96 states passing**. Helper: **40/40**, including 24 negative controls. Native EPUB import adapter probe: **1/1**.
- Full-suite coverage includes formatted reader navigation/marks, pane layout, export/print and audience/presentation checks. Native Windows screenshots for the corrected executable remain pending.

| Measured EPUB state (1920×1009) | Baseline Original | Corrected Original |
| --- | --- | --- |
| Shell scrollTop | 206 | 0 |
| Workspace / pane top | −150 | 56 |
| All three pane bottoms | 803 | 1009 |
| Shared blank band | 206px | 0px |
| App / reader toolbar | Hidden above viewport | Visible |

[Before screenshot](before/epub-1920-1009-1-original-deep.png) · [After screenshot](after/epub-1920-1009-1-original-deep.png). Both are Linux Chromium application evidence, not native Windows captures.

## One targeted Windows verification, pending approval

Reuse the established one-off plain-executable build pathway from run `43405180688` (`windows-viewport-build-20261113`, workflow `windows-viewport-single-build.yml`). Only its reviewed source pin needs updating to the final PR head after approval; do not create infrastructure or an installer. Build command remains `npx tauri build --debug --no-bundle --no-sign --ci -- --locked` with source and executable hashes retained. No dispatch or triggering workflow edit has occurred.

Use the same final protocol against the already archived baseline executable and the one corrected executable, in separate disposable invented-data profiles. The existing launcher now accepts:

```powershell
node <protocol-checkout>/scripts/windows/Test-WebviewViewport.mjs --scenario original --mode observe --exe <executable> --exe-sha256 <SHA256> --source-sha <executable-source-SHA> --expected-dpi 96 --out <NEW-output-directory>
```

Run with the matching clean application checkout as the current directory. The script records its own protocol SHA separately; use the same protocol for both executables. Start with the existing Windows 11/WebView2 diagnostic setup and actual 100% OS scaling. No installation, production profile reset or NiPogi access is part of this protocol.

The runner imports a real generated EPUB through the existing native importer, checks its format and deep anchor mapping, seeds a populated Study and highlight, and runs the shared interaction. Each recorded state includes independent Win32/COM dimensions and native client screenshots as well as DOM geometry, offsets and toolbar visibility. Preserve the baseline's first failure without resizing or repairing it. The corrected run must retain all toolbar/pane positions, scrolling, navigation, writing and saved edits. Independent native screenshot review remains mandatory even if geometry passes.

The generated EPUB was separately validated using the actual Rust adapter: 70 imported paragraphs, anchors 40 and 65, three Contents entries and two internal links. The new Windows scenario itself has not executed yet. If the baseline does not reproduce this interaction on Windows, retain that result and investigate the missing condition; do not call a passing corrected run proof of a Windows fix.
