# Withheld set E — blind, single evaluation at fc0f538

## Construction and isolation

* **Recogniser frozen** at `fc0f538` (2026-10-31, before 22:58 UTC); no file
  under `src-tauri/src/` changed after it (`git diff fc0f538 -- src-tauri/src`
  is empty at the evaluation commit; the harness gained only the line that
  reads `withheld-e.tsv`, in `tests/`).
* **Built by an independent agent** started at 22:58 UTC with the brief in
  `../set-e-brief.md` (verbatim), which names no recogniser rule, failure
  class or earlier result. It worked only in `/root/aw-set-e/` and reported
  reading nothing of the repository, the recogniser, the corpora or earlier
  evaluations (only ebible.org downloads, installed tools and fonts). This is
  its own statement; the container has no access log to prove it. The main
  session did not look at the documents or the manifest's layouts before the
  run.
* **Sealed** at 2026-10-31T23:17:25Z; manifest SHA-256
  `18d145453f7236595a9b22619aaf3dc02b061e447ac22ba14157d97c9a7d5846`,
  committed as `docs/corpus/withheld-e.tsv` (`61d98ad`) before the run; all
  twelve documents' checksums verified against it; nothing in the folder
  newer than the seal.
* **Run once**, 23:20 UTC, with the frozen build. The output was copied
  read-only before it was opened. Nothing has been changed since with
  knowledge of it.
* **Caveats.** Four source editions are reused from earlier sets in new
  layouts (deutkw, fra_fob, porbrbsl, kor: eBible has no unused
  redistributable edition in those languages; marked in the manifest). Only
  eBible.org USFM could be reached, so the set varies the generator and the
  layout, not the publisher.

## Results

| Edition | Format | Printed | Detected | Text-verified | Verified, not in edition | Verified with wrong text |
|---|---|---|---|---|---|---|
| engwebu-rl2col | pdf, 2 columns | 31 098 | 7 758 (24.9 %) | 7 500 (24.1 %) | 0 | 1 |
| enggnv-docxverse | docx, c:v per paragraph | 7 953 | 7 953 (100 %) | 7 950 (100 %) | 0 | 0 |
| engwyc2017-epubapoc | epub, Apocrypha only | 0 (no book of the 66) | 0 | 0 | 0 | 0 |
| engtnt-wp3col | pdf, 3 columns | 7 954 | 6 308 (79.3 %) | 6 039 (75.9 %) | 0 | **68** |
| deutkw-fpdfa5 | pdf, margin numbers | 5 465 | 2 056 (37.6 %) | 0 (0 %) | 0 | 0 |
| fra_fob-pandocepub | epub | 10 423 | 10 423 (100 %) | 10 409 (99.9 %) | 0 | 0 |
| sparvg-pandocdocx | docx | 31 102 | 31 102 (100 %) | 31 002 (99.7 %) | 0 | 0 |
| spav1602p-lodocx | docx | 3 713 | 3 713 (100 %) | 3 711 (99.9 %) | 2 | 2 |
| porbrbsl-md | md | 11 329 | 10 925 (96.4 %) | 10 621 (93.8 %) | **4** | **700** |
| kor-lopdf | pdf, 2 columns | 7 920 | 7 428 (93.8 %) | 81 (1.0 %) | 0 | 1 |
| grcmt-txt | txt | 7 953 | 7 865 (98.9 %) | 7 640 (96.1 %) | **7** | **120** |
| heb-wprtl | pdf, right to left | 5 852 | 0 | 0 | 0 | 0 |
| **Total** | | **130 762** | **95 531 (73.1 %)** | **84 953 (65.0 %)** | **13** | **892** |

Every failing verse is listed in `verified-failures.md`; the harness's own
table is `harness-summary.md`.

## Every failure, classified

| Edition | Count | Class | What happened |
|---|---|---|---|
| porbrbsl-md | 700 wrong | **Notes: Markdown footnote callers** | Pandoc-style footnote references `[^psa-1-1]` stay in the verse text (the Markdown reader does not know footnote syntax). The verse's own words are otherwise intact; the callers are apparatus marked verified. |
| porbrbsl-md | 4 not in edition | **Notes read as verses** | Footnote definitions `[^luk-17-1]: **17:36** Alguns manuscritos gregos acrescentam: …` read as verses LUK 17:36, ACT 8:37, ACT 15:34, ACT 24:7 — textual-variant notes verified as Scripture the edition does not print. |
| grcmt-txt | 120 wrong + 7 not in edition | **Wrong verse assignment: chapter numerals** | Chapter headings in Greek alphabetic numerals ("ΚΕΦΑΛΑΙΟΝ ΙΓʹ") misread around Revelation 13–18: chapter 13 holds chapter 14's verses and so on; verses beyond the real ends (REV 13:19, 16:22–23, 17:19–20, 18:25–26) verified. |
| engtnt-wp3col | 68 wrong | **Running heads** | A head of the form "Matthew Chapter 5" (book, the word "Chapter", the page's last chapter) read into verses at page breaks. |
| spav1602p-lodocx | 2 wrong + 2 not in edition | **Source artefact** | The edition's USFM lacks the `\v` marker for PSA 41:8 and PRO 4:17 (their numbers stand inside the verse before, "… diciendo de mí: 8 Cosa …"); the document prints the number and the recogniser reads it as a verse. The words are the edition's and in their printed order; the verse division differs from the edition's own file. |
| engwebu-rl2col | 1 wrong | **Cut verse** | PRO 5:5 verified without its second line ("Her steps lead straight to Sheol."). |
| kor-lopdf | 1 wrong | **Cut verse** | 1PE 1:8 verified with a clause missing. |

Book identity: no Esther, Daniel or other book was misfiled. In the
Apocrypha EPUB the title "Epistle Of Jeremiah" was read as Jeremiah; its one
chapter was refused for its length, so nothing was verified under `JER`
(caught, but by the length check, not by the title).

## Coverage and honest rejection

* `heb-wprtl` (Hebrew-letter numerals, right to left) and
  `engwyc2017-epubapoc` (only books outside the 66): nothing verified,
  nothing invented — honest rejection.
* `deutkw-fpdfa5` (verse numbers in the margin): 37.6 % found, 0 % verified —
  the margin-number rule fails closed as designed.
* `kor-lopdf`: 93.8 % found but only 1.0 % verified (6 229 `boundary`, 1 297
  `possibleHeading`): fail-closed rules in this layout cost almost all
  verified coverage.
* `engwebu-rl2col`: 24.9 % found — most chapters refused in this two-column
  trade layout.
* Clean at high coverage: Geneva DOCX (100 %), Ostervald EPUB (99.9 %),
  Reina Valera Gómez DOCX (99.7 %), Reina Valera 1602 DOCX (apart from the
  source artefact).

## Verdict for this set

**NOT READY.** The acceptance criteria — zero verified verses the edition
does not have, zero wrong verse assignments, zero contaminated verified
text — are not met: 13 verified verses not in the editions and 892 verified
verses with wrong text, in four documents (three if the source artefact is
set aside). New classes appeared (Markdown footnotes, Greek alphabetic
chapter numerals, a "Book Chapter N" head) that no earlier set showed.
Per the brief, the recogniser is not retuned against set E and no later run
on it will be called independent; set E is a disclosed regression set from
now on.
