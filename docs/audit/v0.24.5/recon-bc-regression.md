# Sets B and C layouts, reconstructed — baseline (2b5d4e2) against v0.24.5 frozen at fc0f538

Regression evidence, not sets B and C: 21 documents rebuilt in their disclosed layouts by `scripts/recon/build_recon_bc.py` plus the two eBible originals in their current builds (`docs/corpus/recon-bc.tsv`). Every remaining wrong verified verse is classified in `verified-differences.md`.

## Results

| Edition | Printed | Detected before | Detected after | Verified before | Verified after | Δ verified | Verified FPs before / after | Verified wrong text before / after |
|---|---|---|---|---|---|---|---|---|
| deu1951-rpydocxdrop | 31101 | 30754 | 30754 (98.9 %) | 28535 | 28535 (91.7 %) | +0 | 0 / 0 | 0 / 0 |
| deuelbbk-rpandocdocx | 31144 | 30353 | 30353 (97.5 %) | 30198 | 30197 (97.0 %) | -1 | 0 / 0 | 1 / 0 |
| engasv-rrlab1col | 31086 | 28054 | 28054 (90.2 %) | 27522 | 27522 (88.5 %) | +0 | 0 / 0 | 9 / 9 |
| engbsb-rwpdf2col | 31086 | 26404 | 26047 (83.8 %) | 22767 | 22428 (72.1 %) | -339 | 0 / 0 | 7 / 5 |
| engdra-rtxtbrk | 31233 | 26007 | 26007 (83.3 %) | 25880 | 25880 (82.9 %) | +0 | 0 / 0 | 0 / 0 |
| engjps-rdocx2lo | 23145 | 22599 | 22599 (97.6 %) | 21052 | 21052 (91.0 %) | +0 | 0 / 0 | 0 / 0 |
| engkjvcpb-rwpdfmargin | 31102 | 16471 | 16453 (52.9 %) | 0 | 1 (0.0 %) | +1 | 0 / 0 | 0 / 0 |
| englsv-rpandocepub | 31104 | 31104 | 31104 (100.0 %) | 31063 | 31063 (99.9 %) | +0 | 0 / 0 | 0 / 0 |
| engoebus-ebiblea4 | 13894 | 13870 | 13870 (99.8 %) | 13565 | 13560 (97.6 %) | -5 | 0 / 0 | 1 / 1 |
| engrv-ebiblebook | 31086 | 30519 | 30519 (98.2 %) | 30416 | 30415 (97.8 %) | -1 | 0 / 0 | 0 / 0 |
| engwebster-rtxtnonum | 31102 | 0 | 0 (0.0 %) | 0 | 0 (0.0 %) | +0 | 0 / 0 | 0 / 0 |
| francl-rebooklibepub | 31220 | 12379 | 12379 (39.7 %) | 12298 | 12203 (39.1 %) | -95 | 0 / 0 | 9 / 7 |
| frasbl-rpandocepub | 31098 | 27814 | 27814 (89.4 %) | 27789 | 27789 (89.4 %) | +0 | 0 / 0 | 0 / 0 |
| grcsbl-rfpdf-mrk | 673 | 0 | 0 (0.0 %) | 0 | 0 (0.0 %) | +0 | 0 / 0 | 0 / 0 |
| grctisch-rmdbold | 7939 | 7939 | 7939 (100.0 %) | 7920 | 7920 (99.8 %) | +0 | 0 / 0 | 0 / 0 |
| hbo-rwpdfrtl-rut | 85 | 0 | 0 (0.0 %) | 0 | 0 (0.0 %) | +0 | 0 / 0 | 0 / 0 |
| kor-rmdcv | 30991 | 18477 | 18477 (59.6 %) | 18428 | 18425 (59.5 %) | -3 | 0 / 0 | 1 / 0 |
| kor-rtxtfull | 30991 | 30928 | 30928 (99.8 %) | 30783 | 30781 (99.3 %) | -2 | 0 / 0 | 0 / 0 |
| porblt-rpydocx | 7942 | 7942 | 7942 (100.0 %) | 7939 | 7939 (100.0 %) | +0 | 0 / 0 | 0 / 0 |
| poronbv-rwpdfa5 | 31105 | 30790 | 30894 (99.3 %) | 29518 | 30344 (97.6 %) | +826 | 13 / 0 | 92 / 0 |
| spabll-rrlab2col | 31098 | 3188 | 3187 (10.2 %) | 0 | 0 (0.0 %) | +0 | 0 / 0 | 0 / 0 |
| spavbl-rlofodt | 31086 | 3590 | 3590 (11.5 %) | 3490 | 3458 (11.1 %) | -32 | 0 / 0 | 0 / 0 |
| **Total** | 551311 | 399182 | 398910 (72.4 %) | 369163 | 369512 (67.0 %) | +349 | 13 / 0 | 120 / 22 |

### Every verified verse with wrong text or not in the edition, after

* engasv-rrlab1col PSA 119:24 (missing, page 732): delete « ד DALETH.» → «»
* engasv-rrlab1col PSA 119:32 (missing, page 732): delete « ה HE.» → «»
* engasv-rrlab1col PSA 119:56 (missing, page 733): delete « ח HHETH.» → «»
* engasv-rrlab1col PSA 119:72 (missing, page 734): delete « י YODH.» → «»
* engasv-rrlab1col PSA 119:80 (missing, page 734): delete « כ KAPH.» → «»
* engasv-rrlab1col PSA 119:112 (missing, page 736): delete « ס SAMEKH.» → «»
* engasv-rrlab1col PSA 119:136 (missing, page 737): delete « צ TSADHE.» → «»
* engasv-rrlab1col PSA 119:152 (missing, page 738): delete « ר RESH.» → «»
* engasv-rrlab1col HAB 3:19 (extra, page 1066): insert «» → « For the Chief Musician, on my stringed instruments.»
* engbsb-rwpdf2col PSA 81:7 (missing, page 472): delete « Selah» → «»
* engbsb-rwpdf2col PSA 140:3 (missing, page 499): delete « Selah» → «»
* engbsb-rwpdf2col PSA 140:5 (missing, page 499): delete « Selah» → «»
* engbsb-rwpdf2col PSA 141:6 (changed, page 499): insert «» → «Selah »
* engbsb-rwpdf2col PSA 141:9 (changed, page 499): insert «» → «Selah »
* engoebus-ebiblea4 MRK 16:20 (missing, page 414): delete « But all that had been revealed to them they reported briefl» → «»
* francl-rebooklibepub PRO 22:16 (missing, page None): delete « PAROLES DES SAGES — 1-INTRODUCTION —» → «»
* francl-rebooklibepub PRO 24:22 (missing, page None): delete « AUTRES PAROLES DES SAGES.» → «»
* francl-rebooklibepub PRO 29:27 (missing, page None): delete « PAROLES D’AGUR.» → «»
* francl-rebooklibepub PRO 30:33 (missing, page None): delete « PAROLES DU ROI LAMUEL.» → «»
* francl-rebooklibepub SNG 1:7 (missing, page None): delete « LE CHŒUR.» → «»
* francl-rebooklibepub SNG 3:5 (missing, page None): delete « LE CHŒUR.» → «»
* francl-rebooklibepub SNG 5:8 (missing, page None): delete « LE CHŒUR.» → «»
