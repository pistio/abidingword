# Phase A1.5: the known weaknesses, investigated

The owner's mandate named five. What was found, what was done, and what
remains. All evidence is on disclosed documents.

## A. `engbsb-rwpdf2col`: 230 correct verses refused — **resolved**

**Root cause** (glyph positions printed by a scratch tool from pdf-extract):
the document draws a note's letter and the next verse's number as **one**
small raised run: `high.ᵈ ¹⁶You` arrives as the run `"d 16"`, raised, at
7 pt against 12 pt text. `pdf_layout` tested a raised run either as
letters (a note's caller) or as digits (a verse number), never as both. The
run failed both tests, so the verse number was read as a **plain** number
inside a line. In a document that raises its verse numbers, A1.3's rule
treats a plain number as no proof of a verse. 344 numbers were drawn this
way, and the misreadings also cost 20 whole chapters.

**Fix** (`179f72f`): a raised run of one or two lower-case letters, an
optional space, and digits is split into its parts, each tested as it
would be alone, at the run's own size and baseline. The control (the
same characters on the baseline in the body's size) stays plain.

**Result:** `engbsb-rwpdf2col` 22 133 → 24 873 verified, 0 wrong, refused
chapters 67 → 47. No other of the 113 editions changes. This is evidence
from the document's own glyphs, not a broad heuristic.

## B. Hebrew: 22 910 of 68 579 printed verses verified — **explained, not resolved**

| Document | Format | Printed | Verified | Why |
|---|---|---:|---:|---|
| `hboWLC-epub` | EPUB | 23 192 | **22 910** | works; 10 chapters refused for Hebrew chapter divisions (Joel 4 …) |
| `hboWLC-pdf` | PDF | 23 192 | 0 | 183 pages whose right-to-left reading order the layout cannot establish; 923 chapters refused |
| `G04` | PDF | 7 508 | 0 | the same: 105 pages, 229 chapters refused |
| `G10` | PDF | 793 | 0 | the same: 11 pages |
| `F08` | DOCX | 7 957 | 0 | verse numbers are Hebrew letters (א, ב … טו); nothing is detected |
| `heb-wprtl` | PDF | 5 852 | 0 | Hebrew-letter numerals |
| `hbo-rwpdfrtl-rut` | PDF | 85 | 0 | Hebrew-letter numerals |

Two causes, both **safe** (0 wrong verified, 0 nonexistent on all seven):

1. **Right-to-left PDF reading order.** The layout refuses a page whose
   line order it cannot establish. In A2.1, PDFium's glyphs through the same
   layout raised `hboWLC-pdf`'s text coverage from 65.9 % to 99.4 %, which
   points at the extractor's right-to-left output, not the recogniser.
   Fixing it in the current backend means reordering pdf-extract's
   right-to-left glyph runs; it was not attempted here.
2. **Hebrew-letter numerals** (gematria) as verse and chapter numbers.
   Supporting them safely needs its own rules, because short Hebrew words
   are also valid numerals (כי, את). Not attempted.

Neither is a precision problem. Both are documented as unsupported in the
handover (§7).

## C. PDF memory — **bounded where possible, otherwise an integration requirement**

* Highest reading peak: 566 MB (`spavbl-rlofodt`), median about 190 MB
  (measured in `FREEZE.md`).
* The largest cost is pdf-extract and lopdf holding the whole document,
  its decoded streams and every glyph. lopdf inflates streams without a
  limit while loading (security review finding 5), which cannot be bounded
  in-process.
* Done: a document that would recurse without end is refused before it
  is drawn (it killed the process with a stack overflow; now a clean
  refusal); the whole PDF read is inside `catch_unwind`.
* Requirement for integration: read PDFs in a separate process with a
  memory cap (handover §6). PDFium stays experimental and separate.

## D. Markdown and USFM derivation — **fail-closed**

`scripts/build-derived-editions.py` now names what it leaves out on
purpose and **stops** on any other USFM marker or HTML paragraph class
(`d1d97a2`). Every existing derivative is reproduced unchanged. The
completeness auditor (version 2) fails on whatever it does not positively
understand (`auditor-qualification.md`).

## E. EPUB refusal reasons — **kept as documented**

The two `francl` JHN 10 refusal-reason differences reported in the A1.3
freeze stand: the A1.3 report's claim that reasons were unchanged was too
strong, and is corrected in `docs/audit/candidate-a1.3/FREEZE.md` §4. A1.5
changes `francl`'s link roles (2 533 links outside any verse become
conflicts) but no verse's outcome.
