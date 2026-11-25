# Source evidence and reader correction — stage #34

## Technical changes

Structured PDF navigation now orders evidence before specificity: explicit user
corrections, grounded embedded/printed TOC destinations, native/layout headings,
then literal body references. A TOC book opening can therefore outrank a narrower
incidental body citation. An actual book opening remains usable when no chapter
heading exists; its scope stays book-level. This is distinct from the unsuccessful
ranking-only approach: the #35 worker first has to extract and persist evidence.

Printed listings never become destination sections themselves. Three distinct
book entries must corroborate actual unique opening headings using a consistent
edition-specific folio offset, or an actual PDF page label must match. No
assumption about book order is used. Exact Greek NT titles are Source metadata
only. Ambiguous Esdras/Kingdoms/Jud labels and unsupported canonical variants stay
unmapped, with their original headings retained in Contents. This is conservative
support, not a complete registry of every canon or translation.

The Source header owns page navigation and PDF zoom commands; embedded PdfViewer
renders no second toolbar. Reading/Original share the live PDF page count, including
unextracted/scanned pages. Zoom survives view switches. Pinning remains available;
unpinned zoom/fit commands live in the named overflow menu with the current zoom.
Narrow panes use a compact View menu and current-page indicator. Reader menus and
page-jump overlays use viewport-clamped portals; PDF scroll width is contained.
The control is now **Kapitel und Überschriften / Chapters and headings** in all
six interface languages.

## Evidence and limits

- `source_evidence_levels.rs`: four regressions covering grounded printed indexes
  in noncanonical order, TOC-before-narrower-heading/body, real Unicode PDF Greek
  title extraction without creating Scripture, and ambiguous/unrepresented books.
- Focused frontend reader regressions: 121 passed in nine files.
- `owner-reader.mjs`: actual PDF.js and an invented Rust-derived PDF. Before/after
  screenshots and numeric geometry are in `images/34-*`. Widths 640/900/1100/1440;
  CSS viewport plus device scale 125/150/200%; zoom to 400%, mode switches,
  Contents, page jump, overlay bounds, resize and focus. This is Linux Chromium,
  not Windows WebView2 or Windows DPI acceptance.
- The owner's MacArthur PDF, Basisinformationen PDF and Greek NT file are not in
  this checkout. Their actual headings, TOC quality, opening targets and persistence
  must be tested on those originals. Counts alone do not establish recognition.
- Untargeted native outline bookmarks can still serve original-page navigation;
  a failed alignment does not manufacture an exact Reading anchor. Uncertain
  headings and broader book scopes remain visible in diagnostics.

Final stage verification: Rust 1,088 passed, eight existing ignored; frontend
1,217 passed with three stale corpus assertions, then all 26 corpus tests passed
after regeneration. Typecheck, ESLint, six-language consistency, production build
and Clippy with warnings denied passed. Browser checks also kept a populated
Study open and compared all textarea values through the interactions.

## Windows owner checks

Use the same copied schema-27 profile procedure as the main correction ledger.
Rebuild each original with the #35 progress panel; export a query report for Apg 6.
Confirm Contents hierarchy against actual printed/native headings. For Acts, check
that a book-level result opens the real introduction when no chapter-6 heading is
present; distinguish that from exact Acts 6 commentary. Test Greek NT separately
as a Source, with no claim of verified Bible text and no AI/network requirement.

At 100/125/150/200% Windows scaling and narrow side panes, test Contents → Original
→ 400% zoom → Reading → Original, page jump, dragging pane splitters, window resize,
keyboard focus and typing into an adjacent Study. Capture all three pane rectangles
and one-row reader controls. A Chromium screenshot does not close this gate.
