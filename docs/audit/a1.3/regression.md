# Phase A1.3: regression on the 113 disclosed editions

Candidate `ccaad81` (A1.3 on A1.2; built at `7f7962e`, the same source before A1.2's docs commit was put under it), structural EPUB reading, the
frozen recogniser's successor for every other document. Compared with the
A1.1 structural run (its edition reports are identical to those of
A1.1's final code `5b2bbdd` and of A1.2, `docs/audit/a1.2/equivalence.md`)
and the frozen v0.24.7 recogniser
(`c96756d`) on identical, checksummed inputs (`CORPUS_OK`, `SETG_OK`).
Sets B–G are disclosed: these are regression measurements, not
independent evaluation. Set H was not created or run.

*Correct* = verified with the edition's text. *Refused* = not verified,
text correct (a conservative refusal). *Caught* = not verified, text
wrong.

## Totals

| | v0.24.7 | A1.1 | A1.3 |
|---|---:|---:|---:|
| Verified, correct text | 1 956 673 | 1 977 862 | 1 977 517 |
| **Verified, wrong text** | 4 172 | 20 | 9 |
| **Verified, not in the edition** | 7 | 2 | 0 |
| Refused (correct text, not verified) | 44 832 | 45 052 | 45 397 |
| Caught (wrong text, not verified) | 17 723 | 17 695 | 17 706 |

## Editions that changed from A1.1

| Set | Edition | Correct A1.1 → A1.3 | Wrong | Not in edition | Refused |
|---|---|---|---|---|---|
| e | `spav1602p-lodocx` | 3707 → 3707 (+0) | 2 → 0 | 2 → 0 | 4 → 4 (+0) |
| bc | `engasv-rrlab1col` | 27470 → 27470 (+0) | 1 → 0 | 0 → 0 | 518 → 518 (+0) |
| bc | `engbsb-rwpdf2col` | 22363 → 22133 (-230) | 0 → 0 | 0 → 0 | 2327 → 2557 (+230) |
| bc | `engrv-ebiblebook` | 30408 → 30406 (-2) | 0 → 0 | 0 → 0 | 90 → 92 (+2) |
| bc | `poronbv-rwpdfa5` | 29988 → 29985 (-3) | 0 → 0 | 0 → 0 | 863 → 866 (+3) |
| dev | `deu1912-pdf` | 30700 → 30693 (-7) | 0 → 0 | 0 → 0 | 402 → 409 (+7) |
| dev | `deutkw-lopdf` | 29819 → 29819 (+0) | 2 → 0 | 0 → 0 | 550 → 550 (+0) |
| dev | `deutkw-pdf` | 29719 → 29719 (+0) | 5 → 0 | 0 → 0 | 357 → 357 (+0) |
| dev | `engwebp-book` | 30112 → 30090 (-22) | 0 → 0 | 0 → 0 | 699 → 721 (+22) |
| dev | `fob-pdf` | 30782 → 30761 (-21) | 0 → 0 | 0 → 0 | 324 → 345 (+21) |
| dev | `fraLSG-luk` | 996 → 995 (-1) | 0 → 0 | 0 → 0 | 16 → 17 (+1) |
| dev | `fraLSG-pdf` | 29457 → 29439 (-18) | 0 → 0 | 0 → 0 | 1227 → 1245 (+18) |
| dev | `frajnd-pdf` | 29625 → 29612 (-13) | 0 → 0 | 0 → 0 | 811 → 824 (+13) |
| dev | `grcbyz-pdf` | 7921 → 7906 (-15) | 0 → 0 | 0 → 0 | 16 → 31 (+15) |
| dev | `latVUC-pdf` | 8527 → 8526 (-1) | 0 → 0 | 0 → 0 | 164 → 165 (+1) |
| dev | `porbr2018-pdf` | 29535 → 29533 (-2) | 1 → 0 | 0 → 0 | 579 → 581 (+2) |
| dev | `rv1909-pdf` | 30582 → 30580 (-2) | 0 → 0 | 0 → 0 | 190 → 192 (+2) |
| dev | `spabes-pdf` | 30240 → 30234 (-6) | 0 → 0 | 0 → 0 | 433 → 439 (+6) |
| dev | `spablm-pdf` | 29104 → 29102 (-2) | 0 → 0 | 0 → 0 | 370 → 372 (+2) |

Every other edition is identical to A1.1 in all four measures, including
every EPUB edition.

## Wrong verified verses left

| Edition | Verses |
|---|---|
| `spablm-docx` | EZR 4:11, EZR 4:17, EZR 5:7, EZR 7:12, DAN 4:1, DAN 4:19, DAN 6:25, DAN 9:4, DAN 9:11 |

All nine are `spablm-docx`'s text left out of the derived document by
the corpus derivation (`cases.md` §6). Rebuilt with the corrected
derivation script, that document reads with none.
