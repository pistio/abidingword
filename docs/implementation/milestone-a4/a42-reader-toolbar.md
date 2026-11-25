# A.4.2 — Compact, configurable Sources reader toolbar (#26)

**Status: implemented and tested in the browser harness; Windows owner acceptance OPEN.**

## Before / after (same 900 px three-pane workspace)

- Before (A.3): [reading-900-1.png](../milestone-a3-pdf/images/reading-900-1.png). The pane spent ≈ 700 px on ~13 rows:
  Back, Open full reader, history, Reading/Original, Bookmark/Note/Find, Contents, Document structure, Marks, then
  "Previous PDF page / PDF page [2] of 3 / Go to page / Next PDF page", a permanent page-number explanation and an
  "About this source" bar.
- After (A.4.2): [a42-reading-900-1.png](images/a42-reading-900-1.png), a single 48 px row:
  Back · Reading/Original · ‹ 8 / 15 › · pinned icons · `…`.

Other captures: [640](images/a42-reading-640-1.png), [1100](images/a42-reading-1100-1.png), [1440](images/a42-reading-1440-1.png),
[125 %](images/a42-reading-900-1.25.png), [150 %](images/a42-reading-1100-1.5.png), [200 %](images/a42-reading-1440-2.png),
[menu](images/a42-menu-1100-1.png), [jump to page](images/a42-jump-900-1.png), [Original](images/a42-original-1100-1.png),
[German](images/a42-reading-de-900.png), [Korean](images/a42-reading-ko-900.png). Results: [checks-a4.json](images/checks-a4.json).
These use the invented handbook PDF with real Rust metadata and real PDF.js. They are not Windows screenshots.

## Design

- `ReaderToolbar` (new) is the **one** toolbar for the Sources pane and the full reader. In the full reader the title sits beside Back.
- Always in the row: Back, Reading/Original, and (in Reading) the page indicator. Original keeps the viewer's own compact
  `◀ [n] / N ▶` and zoom row, so the page control is never duplicated.
- Actions: Find, Contents, history Back/Forward, Bookmark here, Note here, Marks, Document structure, Full reader,
  About this source. **Default pins: Find, Contents.** Everything else is in the `…` menu (`ActionMenu`: arrow keys, Escape,
  focus return). Disabled actions are announced with `aria-disabled` and do nothing.
- **Customise toolbar…** (in the menu) lists the actions as checkboxes; the choice is stored per device
  (`abiding-word.reader.toolbar.pins`); **Reset to default** clears it.
- **Never wraps.** The toolbar measures its room (`ResizeObserver`). Pinned icons that do not fit move into the menu; in very
  narrow panes Back and Reading/Original become icons (names kept for screen readers and as tooltips), and the page arrows fold
  into the jump popover, which then offers its own Previous/Next. The harness asserts one row and no horizontal overflow at
  640/900/1100/1440 and 125/150/200 % scaling.
- Icon buttons are 44 × 44 px, with `aria-label`, `title`, and a visible name tooltip on hover, **keyboard focus** and a long
  touch press. The long press is never the only route: every action is also a named menu item.
- Page indicator `8 / 15` opens a small "Go to page" form with a localised invalid-page message. The physical vs printed page
  explanation and the printed label appear there (and in the indicator's accessible name/tooltip), not as permanent text.
- About this source is collapsed by default (a dialog from the menu). An archived source or a missing original file is still
  shown in the reader without opening anything.

Preserved: reader positions, Reading/Original switching, history, selections/marks, citations, EPUB/DOCX Original, printed
labels, bookmarks, notes, Contents, structure targets and pane/full-reader transfers. The existing UI regressions were moved to
the new routes (helpers in `tests/support/readerActions.ts`), and 6 new toolbar tests were added (`tests/ui/a4-reader-toolbar.test.tsx`).

## Still for the owner (Windows)

Native 100/125/150/200 % scaling, Narrator/NVDA reading of the toolbar and menu, touch long-press on a Windows tablet, and the
judgement that the default pins are right.
