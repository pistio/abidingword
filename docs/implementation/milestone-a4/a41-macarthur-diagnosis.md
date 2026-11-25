# A.4.1 — MacArthur Acts 6 structural navigation: diagnosis, correction, owner diagnostic

**Status: root cause reproduced and corrected on invented handbook-shaped PDFs.
Real MacArthur / native Windows acceptance remains OPEN (Issue #22).**

## What the owner saw

A.3 Windows build (application SHA `15cac96`): selecting Acts 6 and opening the
MacArthur *Basisinformationen zur Bibel* Source showed **Reading on PDF page 3 / 400**,
front matter, instead of the Acts material.

## Trace (A.3 code, unchanged)

1. A Source result's button calls `onOpen(sourceId, best.segmentId, best.structureNodeId)`
   (`src/features/library/PassageSourceList.tsx`). The reader then opens exactly that segment with
   `libraryOpenSegment(segmentId)` (`SourceReader.tsx`), so a page-3 landing means the native
   ranking chose a page-3 segment as the **best place**. The click and reader worked as designed.
2. Ranking (`db/library.rs::collect_places` + `consolidate`): a structured PDF's book-scope
   section is class 3, a literal body mention class ≥ 4, and ties are broken by position, so the
   **earliest** wins.
3. Structure extraction (`library/pdf_structure/mod.rs::extract`), A.3 rule: text repeated in the top/bottom
   10 % margin of ≥ 3 pages is running furniture. **But the reclassification was applied to every line
   with that text anywhere in the document.**

## Root cause (reproduced)

Bible handbooks typically use the book name as their running head ("Apostelgeschichte" on every page of
the Acts introduction). Under the A.3 rule, the **24-pt opening title "Apostelgeschichte" in the body of
the book's first page was reclassified as `header`** and never became a heading. With no grounded Acts
section left, the best place became the earliest literal Acts 6 mention, which sits in the front
matter (a motto or dedication verse, an imprint, or the Contents). Second path: when a Contents page
sets bare book names in a heading-like size, A.3 mapped each line to a whole-book section, and the
earliest one, on the Contents page, won the position tie-break.

`tests/macarthur_shape.rs` builds invented handbook PDFs (title page, imprint with an *Apg 6,4* motto,
Contents, one introduction per book with running heads, page numbers, incidental *Apg 7,58*). Against
the unchanged A.3 code:

| Shape | A.3 first Acts 6 place | A.4.1 first place |
|---|---|---|
| Running head = book name (typical) | PDF p. 2, front-matter mention | p. 8, Acts book section |
| + Contents lists bare book titles | p. 2 | p. 8 |
| Opening title inside the top margin | p. 2 | p. 8 |
| Running heads with chapter ("Apostelgeschichte 7") | p. 8 (title text differs, so it survived) | p. 8 |
| All of the above | **p. 3, Contents line as "book section"** | p. 8 |

This explains a front-matter landing like the owner's. It is **not proof** that the real PDF fails for this
reason: the metadata-only diagnostic below exists to confirm it on the original file.

## Corrections

- **Furniture is margin-only and size-aware** (`classify_furniture`). Only margin occurrences are furniture;
  a margin line set > 1.2 × that furniture's usual size (a chapter-opening title at the top of its page) stays text.
  Numbers are ignored for running heads ("Apostelgeschichte 6/7/8") only when the furniture is set no
  larger than the body text, so headings such as "Kapitel 6 / 7" near the top of a page are **not** removed.
  The same guard was applied to the generated-Contents path (`generated_contents::headings`).
- **Contents listings** (`resolve::effective`): inferred headings on a page naming ≥ 3 different biblical books stay in the
  hierarchy/Contents but receive scope `contents` and **no passage mapping**. Literal references on such a page get the role
  `contents` and rank last (class 6). User mappings are never overridden.
- **Honest labels**: a book-scope result now reads "Book section Acts 1–28 (broader)", and a chapter heading reads "Chapter heading Acts 6"
  (six languages). Mentions remain "Mentions …".
- **Backward compatibility**: stored document JSON stays version 1. The new `analysis` field defaults to 0 for A.3 documents, which still load;
  the structure panel then shows **"built with earlier analysis rules — Rebuild structure"**. Rebuild keeps segment IDs/text, corrections,
  citations and annotations (existing A.3 guarantees, unchanged). No migration; schema stays **27**.

Priority now delivered by the ranking: exact passage/range heading → chapter heading → book section (broader) →
literal mentions → notes → Contents entries. If no heading exists, results stay labelled mentions, and the diagnostic says
"No matching section identified". Manual correction via Document structure remains available and is reported as `user` basis.

## Metadata-only diagnostic (owner tool)

**Sources → open the PDF → Document structure → Diagnostic report (metadata only)**. Enter a passage (`Apg 6`, `Apostelgeschichte 6`,
`Acts 6`, `ACT.6`), then **Create report**, **Copy report** or **Save report… (.json)**.

Contents (`abidingword.pdf-structure-diagnostic/1`): app version; managed file SHA-256 and size; segment/page counts; structure status
(`none / ready / stale / corrupt`), analysis revision and rebuild recommendation; heading counts by basis/scope/level; Contents-listing pages;
block-role counts; derived-link counts; each heading's id, level, depth, basis, scope, canonical mapping, physical/printed page, vertical
position, type size and title (≤ 80 chars); up to 25 ranked candidates (class, origin, evidence label, scope, basis, role, node id,
physical/printed page, matched reference words ≤ 80 chars); the **exact Reader target**; the Reader's **observed** mode/page/opened
heading at report time; and a verdict.

It never contains paragraph text, notes, Study writing, file names/paths, keys or provider settings (asserted in
`the_diagnostic_is_metadata_only`). Heading titles are structural labels but still come from the book, so review the report before sharing.

## Owner steps for the ACTUAL MacArthur journey (Windows)

Use a disposable/copied schema-27 profile (see the A.3 owner check for backup/rollback). Do not upload the PDF.

1. Open the MacArthur Source → **Document structure**. If you see "built with earlier analysis rules", choose **Rebuild structure** and wait.
2. In Scripture, open **Acts 6**. In the Sources pane, note the MacArthur result's first line (e.g. "Book section Acts 1–28 (broader)").
3. Click it. Note the Reader mode and the page indicator (physical page and printed label), and take a full-window screenshot.
4. Without navigating away, open **Document structure → Diagnostic report**, type `Apg 6`, **Create report**, then **Save report…**.
5. Switch to Original and back; repeat step 4 if the page changed.
6. If the result is wrong, correct the Acts heading in Document structure (title/level/mapping) and repeat steps 2–4. Report that run as
   **manual correction**, not automatic recognition.
7. Return the JSON reports and screenshots. Expected automatic result: the Acts introduction (book section, broader), or a chapter/passage
   heading if the book has one, and never front matter.

## Validation (this stage)

Rust 1,063 passed / 0 failed / 8 ignored (A.3: 1,054); `macarthur_shape` 9/9; frontend 1,193 passed (incl. 3 new UI tests); fmt,
Clippy `-D warnings`, typecheck, lint, i18n check; build passed. See the ledger for exact SHAs.
