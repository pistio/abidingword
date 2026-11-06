# Withheld set G: blind, single evaluation of the frozen recogniser c96756d

**Verdict: FAILED. NOT READY.** One real published file (G01, Project
Gutenberg's EPUB of the Almeida Bible) produced:

* **4 144 verified verses with wrong text**;
* **5 verified verses the edition does not have**.

The other 13 documents verified no wrong text. However, 7 of them verified
nothing, and one of those could not be read at all.

## Construction, sealing and the run

* **Recogniser frozen** at `c96756d` (v0.24.7, accepted by the owner). No
  file under `src-tauri/src/` or `src/` changed afterwards. Since then the
  harness has gained only the line that reads `withheld-g.tsv` (`00ba9cf`,
  tests only). The evaluation binary was built there and copied read-only
  before the builder started (SHA-256 `e260b001…e19d`, checked again at the
  run).
* **Independent builder:** a fresh agent working from the neutral brief in
  `set-g-brief.md` (verbatim, committed in `805a124` before the set existed).
  The brief names no failure class, recogniser rule, earlier result or
  targeted weakness. It asked for at least four real published files and for
  a conversion check of the ground truth.
  * Built from 2026-11-06T21:34Z; sealed 2026-11-06T23:10:36Z.
  * Its report on reuse, licences and isolation is in `set-g-brief.md`,
    recorded before the run.
* **Sealed and committed before the run** (`0a9eae1`):
  * manifest `docs/corpus/withheld-g.tsv` (SHA-256 `d0318e02…883b`);
  * `GROUND_TRUTH.md`, `SEAL.txt` and `tools.sha256`;
  * all 28 checksums (14 documents, 14 reference USFM zips) verified
    immediately before the run.
* **Run once**, from 2026-11-06T23:12:10Z to 23:25:57Z, exit 0.
  * The output was made read-only and copied before it was opened. The
    digest of the 13 JSON reports is `103343f0…f9e`.
  * Nothing has been changed with knowledge of the results.
  * Set G will never be re-run as an independent evaluation.

## Results

The harness counts the 66 books. Deuterocanonical books (G05, G07) are
left out by design.

| Doc | Lang, format, source | Printed | Detected | Verified | Verified, not in edition | Verified, wrong text | Unverified: clean / caught |
|---|---|---|---|---|---|---|---|
| G01 | pt, EPUB, **real file** (Gutenberg) | 31 104 | 23 399 (75.2 %) | 8 770 (28.2 %) | **5** | **4 144** | 11 / 258 |
| G02 | sv, text, **real file** (Gutenberg) | 31 151 | 0 | 0 | 0 | 0 | 0 / 0 |
| G03 | en (KJV), PDF, **real file** (BibleTeX/XeLaTeX) | 31 011 | 0 | 0 | 0 | 0 | 0 / 0 |
| G04 | he, PDF, **real file** (eBible, XeTeX), RTL | 7 508 | 5 076 | 0 | 0 | 0 | 0 / 0 |
| G05 | de (Luther 1545), PDF, groff | (not read) | — | — | — | — | — |
| G06 | de (van Ess), DOCX, npm docx | 7 949 | 396 (5.0 %) | 391 (4.9 %) | 0 | 0 | 5 / 0 |
| G07 | fr (Fillion), PDF, WeasyPrint | 601 | 176 | 115 (19.1 %) | 0 | 0 | 0 / 8 |
| G08 | es (RV 1862), DOCX, LibreOffice | 7 956 | 3 363 (42.3 %) | 3 354 (42.2 %) | 0 | 0 | 9 / 0 |
| G09 | ko (1938), PDF, Typst | 764 | 0 | 0 | 0 | 0 | 0 / 0 |
| G10 | hbo, PDF, LibreOffice, RTL | 793 | 11 | 0 | 0 | 0 | 0 / 0 |
| G11 | grc (Nestle 1904), Markdown | 7 269 | 7 269 (100 %) | 7 259 (99.9 %) | 0 | 0 | 10 / 0 |
| G12 | en (Noyes), EPUB, pandoc | 8 550 | 8 400 (98.2 %) | 5 889 (68.9 %) | 0 | 0 | 200 / 0 |
| G13 | en (ULB), PDF, ReportLab | 1 288 | 16 | 0 | 0 | 0 | 0 / 0 |
| G14 | la (Castellio), EPUB, EbookLib | 2 244 | 773 (34.4 %) | 770 (34.3 %) | 0 | 0 | 3 / 0 |
| **Total** | 14 documents, 11 languages | **138 188** | **48 879 (35.4 %)** | **26 548 (19.2 %)** | **5** | **4 144** | 238 / 266 |

Every failing verse is listed in `verified-failures.md`. The harness's own
table is in `harness-summary.md`.

## By the owner's categories

### 1. Incorrect verified text: 4 144, all in G01

The real Gutenberg EPUB puts note links into the verse text as bracketed
callers, "[1]" … "[AAA]". They are plain hyperlinks
(`<a class="fnanchor">[77]</a>`, with the visible text "[1]"), not declared
EPUB note references.

* **3 993 verses: right words, note callers left in.** The verse's words
  equal the edition once the callers are removed, but the stored verse
  would carry "[1]", "[OR]" and the like.
* **151 verses: wrong boundary.**
  * Where a caller's number fitted the verse sequence, the caller was read
    as a verse number. In GEN 8:1, "E [1] lembrou-se Deus de Noé, e [2] de
    toda a besta …" gave a verse 1 "lembrou-se Deus de Noé, e" and moved the
    rest of 8:1 into 8:2.
  * The real verse numbers that followed were then held as text ("… 23 E o
    Senhor t'as dará …").
  * Result: **133 verses cut** (verified with part of their text missing)
    and **18 shifted** (verified with another verse's text).

### 2. Invented or nonexistent verses

The same shift ran five chapters past their end, and those verses were
verified: **DEU 7:27, DEU 7:29, DAN 12:16, AMO 1:16, AMO 1:18**. A further 6
such verses (DEU 7:28, DAN 12:14–15, AMO 1:17, 1:19–20) were detected and
caught unverified.

### 3. Wrong book or chapter assignment

* **Book:** none. No verse was verified under a wrong book, and every book
  code found is one the document holds (G07, G08 and G14 leave some titles
  unnamed).
* **Chapter:** none (`verified_wrong_reference` is 0 in every document).
* **Verse:** the 151 boundary errors and the 5 nonexistent verses above are
  verses assigned the wrong number within the right chapter.

### 4. Genuine refusals (nothing accepted, nothing wrong)

| Doc | What happened |
|---|---|
| G02 (real, Swedish text) | No book found (`documentNoBook`): the Swedish book titles were not recognised. |
| G03 (real, KJV PDF) | No book found (64 not found). The titles were not read, and chapter 1 and every verse 1 are unnumbered (drop numerals). |
| G04 (real, Hebrew PDF) | 23 books found. All 229 chapters were refused for reading order and verse sequence in the right-to-left text layer. |
| G09 (Korean 1938 PDF) | No book found. It uses archaic ordinals ("뎨일쟝") and Chinese-numeral verse numbers. |
| G10 (Hebrew PDF) | 6 books found. Chapters were refused because the verse numbers are Hebrew letters and the text layer is poorly ordered (a recorded property of the document). |
| G13 (ULB PDF) | 2 of 6 books found. Their 2 chapters were refused (reading order, chapter short). |

Partial coverage, also by refusal:
* **G06:** ordinal chapter words ("Siebzehntes Kapitel") were not read, so
  those chapters were refused as unmarked. 13 books were not found.
* **G08:** 15 book titles not named; some carry the source's "^" marks.
* **G12:** 150 psalms were refused (psalm titles inside verse 1).
* **G14:** the Latin Gospel titles were not named.
* **G07:** only Esther and Daniel are in the 66.

### 5. Coverage

Overall, 19.2 % of printed verses were verified and 35.4 % detected.

* **Full or high:** G11 (Markdown, 99.9 %); G12 (EPUB, 68.9 %).
* **Real published files:** G01 28.2 %, with the failures above; G02, G03
  and G04 0 %.

### 6. Text lost through parsing

* **G05: the whole document.** The app's PDF reader refused it
  (`library.badPdf`: "could not be read as a PDF"). The file is a valid
  394-page groff/gropdf PDF: poppler and pypdf read every page. The loss is
  explicit, not silent, but none of the document can be imported.
* **G01: 133 verses cut**, and those cuts were verified (§1). This is
  **silent** loss.
* Elsewhere, no printed verse is missing from an accepted chapter
  (`missing_in_usable_chapter` is 0 in every document).
* The text layers of G04 and G10 lose or scramble characters. The builder
  recorded this as a property of those documents.

## Ground truth

The builder's conversion checks (`GROUND_TRUTH.md`) compared every
reference verse in order with what the document prints. Where a second
independent encoding was reachable, agreement was 95–99.9 %.

The verified verses support those checks: in the 12 documents with verified
verses other than G01, every verified verse is exact or differs only in
spacing or case.

G01's reference leaves the note callers out, correctly: they are notes, not
verse text. Its exact verses (4 626) confirm the reference matches the
printed words.

## Status

Set G is now **disclosed**. It must never be reported again as independent
evidence. No recognition change has been made, and none will be made before
the owner's review. The original output is kept read-only
(`/root/audit/withheld-g.ORIGINAL`), with this record. The documents and
reference zips are not committed (several are CC BY-SA); they are
rebuildable from `GROUND_TRUTH.md` and `scripts/set-g/`.
