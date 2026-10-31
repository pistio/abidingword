# Withheld set C — final (blind, single) evaluation

Run once at commit 01ac499, after set B had been disclosed and used as a regression set; nothing was changed with knowledge of these documents. Manifest, sources and checksums: `docs/corpus/withheld-c.tsv` (frozen in d4444e5).

| Edition | Format | Printed | Detected (accepted) | Text-verified | False positives detected / verified | Verified with extra/changed/cut text | Location | Unverified reasons |
|---|---|---|---|---|---|---|---|---|
| deu1951-pydocxdrop | .docx | 31100 | 30754 (98.9 %) | 28550 (91.8 %) | 0 / 0 | 0 | 100.0 % | noteSign 2, possibleHeading 106, reference 1361, unnumbered 1037 |
| engjps-docx2lo | .pdf | 23145 | 22663 (97.9 %) | 21106 (91.2 %) | 0 / 0 | 6 | 99.5 % | boundary 1, possibleHeading 63, reference 1960 |
| engkjvcpb-wpdfmargin | .pdf | 31102 | 11900 (38.3 %) | 11569 (37.2 %) | 26 / 25 | 4886 | 99.8 % | boundary 7, noteSign 188, possibleHeading 432 |
| engoebus-ebiblea4 | .pdf | 13875 | 13870 (100.0 %) | 13615 (98.1 %) | 0 / 0 | 2 | 99.9 % | boundary 4, possibleHeading 252 |
| engwebster-txtnonum | .txt | 31102 | 0 (0.0 %) | 0 (0.0 %) | 0 / 0 | 0 | — |  |
| frasbl-pandocepub | .epub | 31093 | 26283 (84.5 %) | 26175 (84.2 %) | 18 / 16 | 929 | 100.0 % | boundary 48, length 22, noteSign 5, possibleHeading 99, reference 2 |
| grctisch-mdbold | .md | 7939 | 7939 (100.0 %) | 7920 (99.8 %) | 0 / 0 | 0 | 100.0 % | boundary 19 |
| hbo-wpdfrtl-rut | .pdf | 85 | 0 (0.0 %) | 0 (0.0 %) | 0 / 0 | 0 | — |  |
| kor-txtfull | .txt | 30991 | 30928 (99.8 %) | 30783 (99.3 %) | 0 / 0 | 0 | 98.8 % | boundary 14, psalmTitle 116, reference 15 |
| poronbv-wpdfa5 | .pdf | 31105 | 23469 (75.5 %) | 22254 (71.5 %) | 44 / 32 | 199 | 99.3 % | boundary 164, noteSign 187, possibleHeading 1501 |
| spabll-rlab2col | .pdf | 31093 | 9757 (31.4 %) | 9494 (30.5 %) | 1 / 0 | 10 | 99.8 % | boundary 1375, length 5, noteSign 464, possibleHeading 663, reference 2 |

## What failed, by cause

* **engkjvcpb-wpdfmargin** (verse numbers set in the margin, not in the line): verse boundaries
  fall a line early or late; 4 886 verified verses carry a neighbour's words, and 25 verified
  verses the edition does not have (a chapter run on, 2 Kings 1:19–24). Hard-safety failure.
* **frasbl-pandocepub** (footnote callers as plain digits after words, "Dieu1",
  "SEIGNEUR3"): 929 verified verses keep the callers; 16 verified false positives where chapters
  ran on (Esther 10:4–6, Job 42:18, Hosea 14:10, Mark 16:21).
* **poronbv-wpdfa5**: 32 verified false positives (Genesis 11:33–37 and others, chapters run
  on) and 199 verified verses with another verse's or a cut text.
* **engjps-docx2lo**: running-head words read inside verses ("… unto thee, Habakkuk and
  filthiness …"); 6 verified.
* **spabll-rlab2col** (two columns, tight): 31 % read; quotation boundaries misplaced in 10
  verified verses.
* **engwebster-txtnonum** (no verse numbers at all) and **hbo-wpdfrtl-rut** (Hebrew-letter
  verse numbers): nothing read — correctly refused, nothing invented.
* Read well: deu1951 Word (98.9 % detected, no errors), engoebus eBible PDF (100 %), grctisch
  Markdown (100 %), kor plain text (99.8 %).

