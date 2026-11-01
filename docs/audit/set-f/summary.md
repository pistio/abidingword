# Withheld set F — blind, single evaluation of the frozen recogniser 324cfd1

## Construction, sealing and the run

* **Recogniser frozen** at `324cfd1` (v0.24.6). No file under
  `src-tauri/src/` or `src/` changed before or during set F
  (`git diff 324cfd1 -- src-tauri/src src` is empty at the evaluation
  commit). The harness gained only the line that reads `withheld-f.tsv`
  (`tests/`).
* **Independent builder**: a fresh agent, from the brief in
  `set-f-brief.md` (verbatim), which describes the general task and names no
  failure class, recogniser rule, earlier result or targeted weakness. It
  started 2026-11-01T19:54Z and sealed 2026-11-01T20:30:29Z. Its own report
  of what it read, the reuse, licences and two isolation incidents (a listing
  of top-level file names in the repository's `docs/`, a stray file written
  and deleted) is in `set-f-brief.md`, recorded before the run.
* **Sealed and committed before the run**: manifest
  `docs/corpus/withheld-f.tsv` (SHA-256 `1c9491fd…9071`), `GROUND_TRUTH.md`
  (SHA-256 `141cf6ad…054c`) and `SEAL.txt` with the checksums of all 13
  documents and 13 reference USFM zips, in commit `96f8657`. All checksums
  were verified immediately before the run.
* **Run once**, 2026-11-01T20:33:49Z, with the frozen build. Its output was
  copied read-only before being opened (digest of the 13 JSON reports:
  `5839f9ad…7237`). Nothing has been changed since with knowledge of it, and
  set F will not be re-run as an independent evaluation.
* **Ground truth** is independent: for the 8 non-eBible documents (CrossWire
  texts distributed by scrollmapper/bible_databases) the builder wrote its
  own converters to USFM; the 5 eBible documents use their edition's USFM,
  reduced to the books printed. Documents and reference zips are not
  committed (rebuildable from the builder's record; one CC BY-SA text).

## Results

| Doc | Language, format, generator | Printed | Detected | Text-verified | Verified, not in edition | Verified, wrong text | Unverified: clean / contaminated (caught) |
|---|---|---|---|---|---|---|---|
| F01 | en, PDF, MuPDF Story, 2 columns | 11 328 | 0 | 0 | 0 | 0 | 0 / 0 |
| F02 | en (Leeser Tanakh), PDF, LibreOffice | 23 143 | 21 999 (95.1 %) | 20 032 (86.6 %) | 0 | **6** | 1 821 / 146 |
| F03 | de (Menge), PDF, ReportLab | 31 142 | 11 025 (35.4 %) | 10 848 (34.8 %) | 0 | 0 | 173 / 4 |
| F04 | fr (Martin 1744), EPUB, EbookLib | 31 166 | 31 115 (99.8 %) | 30 987 (99.4 %) | 0 | 0 | 102 / 26 |
| F05 | es (RV 1865), DOCX, python-docx | 31 098 | 25 246 (81.2 %) | 25 210 (81.1 %) | 0 | 0 | 36 / 0 |
| F06 | pt (NVA), PDF, fpdf2 | 7 949 | 0 | 0 | 0 | 0 | 0 / 0 |
| F07 | ko (개역한글), PDF, Typst | 31 084 | 18 | 0 | 0 | 0 | 18 / 0 |
| F08 | he (Salkinson NT), DOCX, LibreOffice, RTL | 7 957 | 0 | 0 | 0 | 0 | 0 / 0 |
| F09 | grc (TCGNT), Markdown | 7 954 | 3 219 (40.5 %) | 3 046 (38.3 %) | 0 | **11** | 2 / 171 |
| F10 | la (Sixtine Vulgate), EPUB 2 | 5 556 | 712 (12.8 %) | 662 (11.9 %) | **41** | **199** | 6 / 44 |
| F11 | el (Vamvas), text | 31 102 | 20 518 (66.0 %) | 20 484 (65.9 %) | **57** | **289** | 31 / 3 |
| F12 | ru (Synodal), EPUB 3, pandoc | 31 169 | 0 | 0 | 0 | 0 | 0 / 0 |
| F13 | it (Diodati), PDF, WeasyPrint | 4 784 | 0 | 0 | 0 | 0 | 0 / 0 |
| **Total** | 13 documents, 12 languages | **255 432** | **113 852 (44.6 %)** | **111 269 (43.6 %)** | **98** | **505** | 2 189 / 394 |

Every failing verse: `verified-failures.md`; the harness's own table:
`harness-summary.md`.

## The failures, by the owner's categories

### Incorrect verified verse text: 505, in 4 documents

| Doc | Count | Class | What happened |
|---|---|---|---|
| F11 | 289 | **Wrong book** | This Hebrew-based (Vamvas, 1850) Greek Bible names 1–2 Samuel "ΣΑΜΟΥΗΛ Α΄/Β΄" and 1–2 Kings "ΒΑΣΙΛΕΩΝ Α΄/Β΄". The recogniser's Greek names read "ΒΑΣΙΛΕΩΝ Α΄" in the Septuagint sense, as 1 Samuel, and do not know "ΣΑΜΟΥΗΛ Α΄". 1 Kings' text was verified as 1 Samuel. |
| F10 | 199 | **Wrong chapter** | Chapters are headed "CAPUT I." (Latin); the recogniser has no Latin chapter word, so the headings were not read and chapters were inferred from verse numbers starting again at 1, checked against the standard verse counts. With the Vulgate's counts the inference drifted: Ezekiel, Amos, Micah and Zechariah chapters hold other chapters' verses. |
| F09 | 11 | **Cut verse** | Each book's closing "Ἀμήν." (part of the last verse in the edition) is printed in a raw HTML block (`<div align="center">`), which the Markdown reader never interprets; the last verse was verified without it. |
| F02 | 4 | **Cut verse / wrong boundary** | A letter-spaced running head beginning with a figure ("1 S A M U E L") at a page's top was read as verse 1 of the next chapter, cutting the end of the verse before (1 Samuel 9:27, 22:23; 1 Chronicles 12:40; 2 Chronicles 1:17). |
| F02 | 2 | **Running head read in** | Letter-spaced heads "O B A D I A H", "H A G G A I" read into Obadiah 1:18 and Haggai 2:4. |

### False-positive verses (verified, not in the edition): 98

F11 57 (1 Samuel 2:37 ff., 3:22 ff. …: 1 Kings' longer chapters under 1
Samuel numbers) and F10 41 (Ezekiel 4:18, 5:18–22, 7:28, 8:19–21, 13:24–27 …:
drifted chapters). Both follow from the wrong book / wrong chapter above.
(Detected but unverified false positives: 102.)

### Wrong book, chapter or verse assignment

* Book: F11 (Samuel/Kings naming, 289 + 57).
* Chapter: F10 (unread Latin chapter labels, 199 + 41).
* Verse boundary: F02 (letter-spaced head with a figure read as verse 1; 4).

### Contamination by headings, notes or page furniture

* Page furniture: F02, 2 verses (letter-spaced book-name running heads).
* Notes and headings: none verified. 394 verses whose text was not the edition's
  were caught and held back as unverified (F09 171, F02 146, F10 44, F04 26,
  others 7).

### Coverage

Detected 44.6 %, text-verified 43.6 % of 255 432 printed verses. High
coverage and clean: French Martin EPUB (99.4 %), Spanish RV 1865 DOCX
(81.1 %), German Menge (34.8 %, clean but low); Leeser PDF 86.6 % with 6
errors.

### Correct refusals and unverified verses

Six documents produced no verified verse and no error — honest refusals,
though each is a coverage gap:

* F01 (English, MuPDF two-column) and F06 (Portuguese, fpdf2): book titles
  were taken from contents entries with page numbers ("Matthew 1", "Mark
  32"), so books "began twice" and every chapter was refused.
* F07 (Korean, Typst): the reading order of its pages could not be
  established (1 187 reading-order refusals).
* F08 (Hebrew, right-to-left DOCX): only Matthew's title was recognised.
* F12 (Russian, pandoc EPUB): no Russian book names are known to the
  recogniser; nothing was found.
* F13 (Italian, WeasyPrint): chapters refused (raised note letters
  unexplained; Psalms refused under the Septuagint numbering rule — that
  rule was triggered in an Italian document, a false trigger that failed
  closed).

In F11 the same Septuagint rule fired on a Hebrew-based Greek Bible (Greek
book names): its Esther was left out and 181 chapters refused — the
fail-closed cost v0.24.5 predicted.

### Differences caused by the reference source itself

None of the 603 failures. The builder recorded the sources' own oddities
(Leeser's Hebrew numbering in parentheses inside verses, empty verses in
Leeser, the Sixtine Vulgate and other texts, OCR digits in Martin, the
Korean "12절과 상동" verses, book-final "Ἀμήν." paragraphs in the TCGNT); they
explain held-back verses (Leeser's 1 898 `reference` doubts) but no
verified failure.

## Verdict for this set

**FAILED — NOT READY.** 98 verified verses that the editions do not have
and 505 verified verses with wrong text, in four of thirteen documents,
through classes none of the earlier sets showed in this form: a modern
Greek Bible's Samuel/Kings names, Latin chapter labels with chapters then
inferred from verse counts, raw HTML blocks in Markdown, and letter-spaced
running heads beginning with a figure. Set F is now disclosed; it will not
be re-run as an independent evaluation.
