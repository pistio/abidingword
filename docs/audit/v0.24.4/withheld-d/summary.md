# Withheld set D — blind, single evaluation

Built after the code freeze by an agent that saw neither the recogniser nor sets B and C (`docs/corpus/withheld-d.tsv`, committed 953c11c, before any Set D run); 12 documents, 7 formats, 8 languages. Run once with the recogniser frozen at **17398c6**; nothing was changed with knowledge of these documents, and nothing has been tuned against them since. Three earlier starts of this run (at 8a61c55, 771f40e and a first start at 17398c6) were cancelled before any output was read, because regression evidence still called for general fixes; their outputs were deleted unseen.

**Process note.** The harness crashed on `grcsr-story2col`: the app itself refused the PDF cleanly (`library.badPdf`, its text cannot be extracted — the embedded CMap makes the extractor panic on every page, which the app catches), but the report unwrapped that result and ended, so the three documents after it in the manifest were read in a second invocation of the same build (`withheld-d-rest.log`), and the rejection was recorded with a harness-only fix (856a81b, `tests/` only; the recogniser is unchanged). The recogniser build was the same for all twelve.

## Results

| Edition | Format | Printed | Detected (accepted) | Text-verified | False positives detected / verified | Verified with extra/changed/cut text | Location | Unverified reasons |
|---|---|---|---|---|---|---|---|---|
| deu1912-wpdfdropcap | .pdf | 31102 | 29824 (95.9 %) | 29380 (94.5 %) | 0 / 0 | 298 | 99.1 % | boundary 67, noteSign 8, possibleHeading 383, reference 8, unnumbered 1 |
| engbbe-fpdfmargin | .pdf | 31102 | 0 (0.0 %) | 0 (0.0 %) | 0 / 0 | 0 | — | boundary 847, noteSign 6, possibleHeading 14 |
| engbrenton-lohtml2col | .pdf | 22357 | 15750 (70.4 %) | 13963 (62.5 %) | 200 / 449 | 212 | 99.8 % | boundary 140, length 2, noteSign 1279, possibleHeading 745, unnumbered 7 |
| engdby-pandocdocx | .docx | 31099 | 30877 (99.3 %) | 30851 (99.2 %) | 0 / 0 | 0 | 100.0 % | boundary 3, noteSign 19, possibleHeading 4 |
| fralsg-rlabheads | .pdf | 31170 | 11613 (37.3 %) | 11349 (36.4 %) | 0 / 0 | 0 | 99.5 % | boundary 14, possibleHeading 333, reference 22 |
| grcbrent-txtref | .txt | 21949 | 16872 (76.9 %) | 16703 (76.1 %) | 200 / 449 | 2 | 100.0 % | boundary 71, length 7, possibleHeading 19, psalmTitle 116, reference 77 |
| hebwlc-storyrtl-est | .pdf | 167 | 0 (0.0 %) | 0 (0.0 %) | 0 / 0 | 0 | — |  |
| kor-wpdf2col | .pdf | 30991 | 30655 (98.9 %) | 29836 (96.3 %) | 0 / 0 | 568 | 99.9 % | boundary 118, possibleHeading 694, reference 11, unnumbered 2 |
| portft-mdcvbold | .md | 7898 | 189 (2.4 %) | 185 (2.3 %) | 0 / 0 | 31 | 100.0 % | boundary 2, possibleHeading 1, reference 2 |
| spaonbv-lohtmldocx-poet | .docx | 4648 | 2688 (57.8 %) | 2530 (54.4 %) | 0 / 0 | 16 | 100.0 % | boundary 200, length 31, possibleHeading 8, unnumbered 13 |
| spapddpt-epub2 | .epub | 31081 | 20818 (67.0 %) | 19603 (63.1 %) | 0 / 0 | 69 | 100.0 % | boundary 1367, length 85, noteSign 173, possibleHeading 141, reference 10 |
| grcsr-story2col | .pdf | — | not read | — | — | — | — | the app refused the file (`library.badPdf`): honest rejection, no text at all |

Totals over the eleven documents read: 243 564 verses printed  159 286 detected (65.4 %)  154 400 text-verified (63.4 %)  **898 verified verses the reference does not have  1 196 verified verses whose text is not the verse's**.

## Hard-safety failures, classified

Every verified verse whose text differs from the reference beyond whitespace and typography, by cause. Each of these is text the product would have stored as checked Scripture.

| Edition | Verified false positives | Verified with wrong text | Causes (count) | Example |
|---|---|---|---|---|
| deu1912-wpdfdropcap | 0 | 298 | running head read into the verse (298) | running head read into the verse: GEN 1:17 «…DAS 1. BUCH MOSE (GENESIS)…» |
| engbrenton-lohtml2col | 449 | 212 | running head read into the verse (174); next verse merged (number set against its first word) (27); footnote (with the page number) read into the verse (11) | running head read into the verse: GEN 4:11 «…THE SEPTUAGINT VERSION OF THE OLD TESTAMENT — ENGLISH TRANSLATION BY SIR L. C. L. BRENTON…»<br>footnote (with the page number) read into the verse: EXO 3:12 «…1993:2 Gr. fire of flame.…»<br>next verse merged (number set against its first word): JOS 15:59 «…59aTheco, and Ephratha, this is Bæthleem, and Phagor, and Ætan, and Culon, and Tatam, and…» |
| grcbrent-txtref | 449 | 2 | next verse's words appended (verse-per-line text; 2) | PSA 103:32 «…Ἄσω τῷ Κυρίῳ ἐν τῇ ζωῇ μου, ψαλῶ τῷ Θεῷ μου ἕως ὑπάρχω. Ἡδυνθείη αὐτῷ ἡ διαλογή …» (the words of 103:33) |
| kor-wpdf2col | 0 | 568 | running head read into the verse (556); next verse merged (its number left at the end) (5); cut: words of the verse missing (4); next verse merged (number set against its first word) (3) | running head read into the verse: GEN 1:26 «…창세기 2–창세기 2…»<br>cut: words of the verse missing: GEN 11:12 «…아르박삿은 삼십 오세에 셀라를 낳았고…»<br>next verse merged (number set against its first word): DEU 22:29 «…30missing verse…» |
| portft-mdcvbold | 0 | 31 | theme label ("TEMA: …") read into the verse (31) | theme label ("TEMA: …") read into the verse: 1TH 1:1 «…TEMA: Sempre dou graças a Deus por vocês porque, ao orar, lembro-me constantemente da man…» |
| spaonbv-lohtmldocx-poet | 0 | 16 | next verse merged (number set against its first word) (16) | next verse merged (number set against its first word): JOB 5:26 «…27»La experiencia me ha enseñado la verdad de todo esto. Para bien tuyo, escucha mi conse…» |
| spapddpt-epub2 | 0 | 69 | next verse merged (number set against its first word) (64); next verse merged (its number left at the end) (3); cut: words of the verse missing (2) | next verse merged (number set against its first word): GEN 14:22 «…23que de todo lo que es tuyo, no tomaré ni un hilo ni una correa de sandalia para que no…»<br>cut: words of the verse missing: NUM 8:24 «…a arriba, entrarán para servir en el Tabernáculo de Reunión,…»<br>next verse merged (its number left at the end): NUM 8:25 «…años para arriba, entrarán para servir en el Tabernáculo de Reunión, 25…» |

### The 449 + 449 verified false positives (Brenton, English PDF and Greek text)

All 898 are verses of the editions' **Greek Esther and Greek Daniel** (USFM books `ESG`, `DAG`: 252 and 422 verses, Daniel 3 with 97 verses), present in the reference under the same chapter and verse. The documents head them "ESTHER" / "DANIEL" (and "ΕΣΘΗΡ" / "ΔΑΝΙΗΛ"); the recogniser's canon has only the Hebrew-canon Esther (`EST`) and Daniel (`DAN`), so it filed them there and verified them, including verses the Hebrew canon does not number (Daniel 3:31–97). No words were invented and every verse is where the document prints it, but a Bible read from such an edition would present Septuagint verses under the Hebrew book codes, and chapters longer than the canon were not refused. This is a canon-coverage and fail-closed gap, reported as its own class, not a reference artefact to be dismissed.

### What the failures have in common

* **Running heads not set apart** (German WeasyPrint PDF, 298; Korean two-column WeasyPrint PDF, 556; Brenton LibreOffice PDF, 174): heads whose form the layout evidence does not yet cover — a book's long title in capitals with its number ("DAS 1. BUCH MOSE (GENESIS)"), a Korean "book chapter–book chapter" head, and a work title repeated on every page ("THE SEPTUAGINT VERSION OF THE OLD TESTAMENT — ENGLISH TRANSLATION BY SIR L. C. L. BRENTON") — read as verse text at page breaks. In the Korean document the heads carry the chapter numbers.
* **A verse number set directly against its first word** ("23que", "27»La", "28ahe", "11`우리"; Spanish EPUB 2, 64 + 3 with the number left at the verse's end; Spanish LibreOffice DOCX, 16; Brenton PDF, 27; Korean PDF, 3 + 5): the number was not read as a verse number, so the next verse's words were verified as part of the verse before. Earlier work handled this only where the number is raised or superscript. The Greek verse-per-line text shows the same outcome twice (a following verse appended).
* **Footnotes on the page read as verse text** (Brenton PDF, 11): the note, with the page number before it, between two lines of a verse.
* **Theme labels inline** (Portuguese Markdown, 31): "TEMA: …" lines (an edition's own study apparatus) after a verse.
* **Cut verses** (Korean 4, Spanish EPUB 2): words of a verse missing from the verified text.

## Coverage and honest rejection

* `engbbe-fpdfmargin` (fpdf2 PDF with verse numbers in the margin): 0 % detected, 0 false positives — every chapter refused (`boundary` doubts on 847 lines): the margin-number evidence from set C generalised as intended, fail closed.
* `hebwlc-storyrtl-est` (right-to-left Hebrew booklet): 0 %, nothing invented.
* `grcsr-story2col` (MuPDF Story two-column PDF): the app refuses the file; no text is read.
* `fralsg-rlabheads` (A5 ReportLab PDF with running heads): 37.3 % detected, 36.4 % verified, no false positives and no wrong text — low coverage with honest rejection.
* `engdby-pandocdocx` (pandoc Word): 99.3 % detected, 99.2 % verified, clean.
* `portft-mdcvbold` (Markdown, chapter-verse in bold): 2.4 % detected — nearly everything refused — and the little that was verified carries the theme labels.

## Verdict for this set

The acceptance criteria — zero invented verses, zero wrong verse assignments and zero contaminated text among verified verses — are **not met**: six of the eleven documents read verify wrong text, and two file Septuagint books under Hebrew-canon codes. Four documents are clean or honestly rejected.
