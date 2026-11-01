# Verified verses whose text differs from the reference — v0.24.5 frozen at fc0f538

Every text-verified verse whose text is not identical to the edition's USFM
(beyond whitespace and typography), in every regression run of this
milestone, with its classification. **Contamination** means text marked
verified that is not that verse's own (or that lacks part of it) because of
the recogniser: a hard-safety failure. Reference artefacts are differences
between how the edition's USFM encodes its text and how the document prints
it, where the recogniser's reading is the printed page's.

## Set D layouts, reconstructed (7 documents)

None. (Baseline at the v0.24.4 recogniser: 1 138 wrong verified verses and
810 verified verses the editions do not have.)

## Sets B and C layouts, reconstructed (23 documents): 22

| Edition | Verse(s) | Classification |
|---|---|---|
| engasv-rrlab1col | PSA 119:24, 32, 56, 72, 80, 112, 136, 152 | Reference artefact: the ASV USFM puts the acrostic heading (`\qa`, "ד DALETH.") inside the verse; the document prints it as a heading and the recogniser leaves it out (v0.24.4: the same) |
| engasv-rrlab1col | HAB 3:19 | Reference artefact: the closing rubric ("For the Chief Musician, on my stringed instruments.") is `\d` after the verse in the USFM and printed as the verse's last sentence (v0.24.4: the same) |
| engoebus-ebiblea4 | MRK 16:20 | Reference artefact: the eBible PDF prints the shorter ending of Mark apart from 16:20; the USFM appends it to the verse (v0.24.4: the same) |
| francl-rebooklibepub | PRO 22:16, 24:22, 29:27, 30:33; SNG 1:7, 3:5, 5:8 | Reference artefact: the edition's USFM writes section headings and speaker labels ("PAROLES D'AGUR.", "LE CHŒUR.") at the end of the verse before; the document sets them as headings and the recogniser leaves them out |
| engbsb-rwpdf2col | PSA 81:7, 140:3, 140:5 (missing "Selah") and PSA 141:6, 141:9 ("Selah" read into the middle of the verse) | **Contamination (one word, open finding).** The reconstruction sets "Selah" right-aligned (`\qr`) at the left column's inner edge; the layout reads those lines with the right column, so the word leaves its verse and enters another. Present already at the v0.24.4 baseline (7 such verses there). Not fixed: the recogniser was frozen before this was classified. |

## Development corpus (46 documents): 20, the same 20 as at the baseline

| Edition | Verse(s) | Classification |
|---|---|---|
| deutkw-lopdf, deutkw-pdf | 2KI 4:25, LUK 9:43; 1KI 8:12, 8:13, PSA 60:13, ACT 2:14, 2CO 6:16 | Reference artefact: the Textbibel's USFM marks these words of Scripture as headings (`\s1`/`\sp`/`\qa`); the documents print them as text (v0.24.4: the same) |
| porbr2018-epub, porbr2018-pdf | HEB 13:25; ROM 16:27, GAL 6:18, 1TI 6:21 | By design: the subscription (colophon) is kept apart as apparatus; the USFM appends it to the verse (v0.24.4: the same) |
| spablm-docx | EZR 4:11, 4:17, 5:7, 7:12; DAN 4:1, 4:19, 6:25, 9:4, 9:11 | Source: the converted document lacks these words; nothing is substituted (v0.24.4: the same) |

No contamination, no verified verse the edition does not have.
