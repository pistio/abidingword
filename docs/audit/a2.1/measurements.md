# A2.1 measurements: current backend against PDFium glyphs through the app's layout

Same 57 checksummed PDFs as A2. Text coverage and order: share of the edition's verses found whole, and found in order, in the text the app's layout produced (A2's `score.py`; Hebrew with points and cantillation stripped, as in A2). Verification: the real-document harness on the same documents. Time and peak memory: release build, each reading in its own process (`clean_run.py`); for PDFium the peak is the larger of the app's and the helper's.

## G05 (Luther 1545, Fraktur forms)

| Document | Text coverage current / PDFium | In order current / PDFium | Verified current / PDFium | Verified wrong text current / PDFium | Verified not in edition current / PDFium | Current: time / peak | PDFium through layout: time / peak | Helper alone |
|---|---|---|---|---|---|---|---|---|
| G05 | — / 89.1 % | — / 46.6 % | — / 0 | — / 0 | — / 0 | — | 1.5 s / 28 MB | 1.1 s / 14 MB |

## Hebrew, right to left

| Document | Text coverage current / PDFium | In order current / PDFium | Verified current / PDFium | Verified wrong text current / PDFium | Verified not in edition current / PDFium | Current: time / peak | PDFium through layout: time / peak | Helper alone |
|---|---|---|---|---|---|---|---|---|
| hboWLC-pdf | 65.9 % / 99.4 % | 65.8 % / 99.4 % | 0 / 0 | 0 / 0 | 0 / 0 | 5.0 s / 154 MB | 8.0 s / 150 MB | 5.7 s / 26 MB |
| hbo-rwpdfrtl-rut | 95.3 % / 95.3 % | 95.3 % / 95.3 % | 0 / 0 | 0 / 0 | 0 / 0 | 0.0 s / 13 MB | 0.1 s / 13 MB | 0.0 s / 13 MB |
| heb-wprtl | 5.5 % / 5.5 % | 1.1 % / 1.1 % | 0 / 0 | 0 / 0 | 0 / 0 | 0.6 s / 15 MB | 0.8 s / 14 MB | 0.5 s / 13 MB |
| G04 | 99.5 % / 99.6 % | 99.5 % / 99.6 % | 0 / 0 | 0 / 0 | 0 / 0 | 1.6 s / 58 MB | 2.2 s / 54 MB | 1.7 s / 15 MB |
| G10 | 39.6 % / 73.4 % | 8.2 % / 25.9 % | 0 / 0 | 0 / 0 | 0 / 0 | 0.4 s / 25 MB | 0.3 s / 16 MB | 0.2 s / 13 MB |

## Korean

| Document | Text coverage current / PDFium | In order current / PDFium | Verified current / PDFium | Verified wrong text current / PDFium | Verified not in edition current / PDFium | Current: time / peak | PDFium through layout: time / peak | Helper alone |
|---|---|---|---|---|---|---|---|---|
| kor-pdf | 99.5 % / 99.5 % | 99.5 % / 99.5 % | 30917 / 30917 | 0 / 0 | 0 / 0 | 14.3 s / 76 MB | 7.2 s / 74 MB | 5.5 s / 35 MB |
| kor1961-lopdf | 100.0 % / 100.0 % | 34.9 % / 34.9 % | 30808 / 30808 | 0 / 0 | 0 / 0 | 22.6 s / 367 MB | 4.4 s / 350 MB | 2.4 s / 42 MB |
| kor-r2col | 99.0 % / 99.0 % | 98.8 % / 98.8 % | 30723 / 30723 | 0 / 0 | 0 / 0 | 10.7 s / 73 MB | 14.6 s / 75 MB | 13.3 s / 26 MB |
| kor-lopdf | 95.4 % / 95.4 % | 70.5 % / 70.5 % | 65 / 65 | 0 / 0 | 0 / 0 | 5.2 s / 110 MB | 1.3 s / 108 MB | 0.7 s / 21 MB |
| F07 | 99.0 % / 99.0 % | 29.2 % / 29.2 % | 0 / 1 | 0 / 0 | 0 / 0 | 10.1 s / 237 MB | 6.4 s / 160 MB | 4.8 s / 30 MB |
| G09 | 90.4 % / 90.6 % | 90.4 % / 90.6 % | 0 / 0 | 0 / 0 | 0 / 0 | 0.2 s / 18 MB | 0.2 s / 13 MB | 0.1 s / 13 MB |

## Greek

| Document | Text coverage current / PDFium | In order current / PDFium | Verified current / PDFium | Verified wrong text current / PDFium | Verified not in edition current / PDFium | Current: time / peak | PDFium through layout: time / peak | Helper alone |
|---|---|---|---|---|---|---|---|---|
| grctr-pdf | 98.3 % / 98.3 % | 13.6 % / 13.6 % | 7640 / 7640 | 0 / 0 | 0 / 0 | 2.0 s / 33 MB | 2.5 s / 32 MB | 1.9 s / 16 MB |
| grcbyz-pdf | 98.7 % / 98.7 % | 98.7 % / 98.7 % | 7921 / 7921 | 0 / 0 | 0 / 0 | 2.3 s / 30 MB | 2.8 s / 31 MB | 2.0 s / 17 MB |
| grcsbl-rfpdf-mrk | 96.9 % / 96.9 % | 96.9 % / 96.9 % | 0 / 0 | 0 / 0 | 0 / 0 | 0.1 s / 13 MB | 0.2 s / 13 MB | 0.1 s / 13 MB |

## Multi-column

| Document | Text coverage current / PDFium | In order current / PDFium | Verified current / PDFium | Verified wrong text current / PDFium | Verified not in edition current / PDFium | Current: time / peak | PDFium through layout: time / peak | Helper alone |
|---|---|---|---|---|---|---|---|---|
| engwebp-pdf | 98.2 % / 98.2 % | 94.7 % / 94.7 % | 29828 / 29828 | 0 / 0 | 0 / 0 | 7.5 s / 96 MB | 7.8 s / 93 MB | 5.6 s / 35 MB |
| engwebp-book | 98.6 % / 98.6 % | 32.1 % / 32.1 % | 30112 / 30112 | 0 / 0 | 0 / 0 | 7.4 s / 94 MB | 7.5 s / 91 MB | 6.2 s / 34 MB |
| kjv-pdf | 99.5 % / 99.5 % | 99.5 % / 99.5 % | 30868 / 30868 | 0 / 0 | 0 / 0 | 8.4 s / 271 MB | 8.7 s / 249 MB | 6.9 s / 40 MB |
| deu1912-pdf | 99.7 % / 99.7 % | 99.7 % / 99.7 % | 30700 / 30700 | 0 / 0 | 0 / 0 | 6.8 s / 216 MB | 7.2 s / 198 MB | 5.6 s / 33 MB |
| engbsb-rwpdf2col | 89.1 % / 89.1 % | 1.5 % / 1.5 % | 22363 / 22363 | 0 / 0 | 0 / 0 | 4.5 s / 81 MB | 6.9 s / 82 MB | 5.0 s / 25 MB |
| engjps-rdocx2lo | 96.4 % / 96.4 % | 2.1 % / 2.1 % | 20867 / 20867 | 0 / 0 | 0 / 0 | 4.0 s / 194 MB | 5.4 s / 197 MB | 3.3 s / 30 MB |
| spabll-rrlab2col | 80.9 % / 80.9 % | 19.2 % / 19.2 % | 0 / 0 | 0 / 0 | 0 / 0 | 3.7 s / 71 MB | 6.4 s / 68 MB | 4.2 s / 21 MB |
| engbrenton-rlo2col | 87.0 % / 87.0 % | 13.8 % / 13.8 % | 14409 / 14409 | 0 / 0 | 0 / 0 | 4.5 s / 275 MB | 6.2 s / 268 MB | 4.3 s / 38 MB |
| kor-r2col | 99.0 % / 99.0 % | 98.8 % / 98.8 % | 30723 / 30723 | 0 / 0 | 0 / 0 | 10.7 s / 73 MB | 14.6 s / 75 MB | 13.3 s / 26 MB |
| engtnt-wp3col | 98.7 % / 98.7 % | 98.7 % / 98.7 % | 6040 / 6040 | 0 / 0 | 0 / 0 | 1.0 s / 25 MB | 1.6 s / 24 MB | 1.1 s / 17 MB |
| F01 | 97.1 % / 97.4 % | 29.2 % / 29.2 % | 0 / 8541 | 0 / 0 | 0 / 0 | 2.9 s / 76 MB | 1.9 s / 29 MB | 1.4 s / 17 MB |
| F02 | 96.4 % / 96.4 % | 18.3 % / 18.3 % | 20487 / 20487 | 0 / 0 | 0 / 0 | 4.0 s / 228 MB | 5.3 s / 220 MB | 3.4 s / 33 MB |
| F03 | 98.0 % / 98.0 % | 10.4 % / 10.4 % | 10848 / 6107 | 0 / 0 | 0 / 0 | 4.8 s / 79 MB | 7.2 s / 81 MB | 4.7 s / 22 MB |
| G03 | 99.1 % / 99.1 % | 3.5 % / 3.5 % | 0 / 0 | 0 / 0 | 0 / 0 | 2.9 s / 62 MB | 5.4 s / 63 MB | 3.8 s / 22 MB |
| G05 | — / 89.1 % | — / 46.6 % | — / 0 | — / 0 | — / 0 | — | 1.5 s / 28 MB | 1.1 s / 14 MB |
| G09 | 90.4 % / 90.6 % | 90.4 % / 90.6 % | 0 / 0 | 0 / 0 | 0 / 0 | 0.2 s / 18 MB | 0.2 s / 13 MB | 0.1 s / 13 MB |
| G13 | 48.6 % / 48.6 % | 48.6 % / 48.6 % | 0 / 0 | 0 / 0 | 0 / 0 | 0.1 s / 13 MB | 0.3 s / 13 MB | 0.2 s / 13 MB |

## All 57

| Document | Text coverage current / PDFium | In order current / PDFium | Verified current / PDFium | Verified wrong text current / PDFium | Verified not in edition current / PDFium | Current: time / peak | PDFium through layout: time / peak | Helper alone |
|---|---|---|---|---|---|---|---|---|
| engwebp-pdf | 98.2 % / 98.2 % | 94.7 % / 94.7 % | 29828 / 29828 | 0 / 0 | 0 / 0 | 7.5 s / 96 MB | 7.8 s / 93 MB | 5.6 s / 35 MB |
| engwebp-book | 98.6 % / 98.6 % | 32.1 % / 32.1 % | 30112 / 30112 | 0 / 0 | 0 / 0 | 7.4 s / 94 MB | 7.5 s / 91 MB | 6.2 s / 34 MB |
| kjv-pdf | 99.5 % / 99.5 % | 99.5 % / 99.5 % | 30868 / 30868 | 0 / 0 | 0 / 0 | 8.4 s / 271 MB | 8.7 s / 249 MB | 6.9 s / 40 MB |
| deu1912-pdf | 99.7 % / 99.7 % | 99.7 % / 99.7 % | 30700 / 30700 | 0 / 0 | 0 / 0 | 6.8 s / 216 MB | 7.2 s / 198 MB | 5.6 s / 33 MB |
| deuelo-pdf | 99.7 % / 99.7 % | 99.7 % / 99.7 % | 30510 / 30510 | 0 / 0 | 0 / 0 | 7.6 s / 257 MB | 7.6 s / 207 MB | 6.1 s / 32 MB |
| fraLSG-pdf | 82.4 % / 82.4 % | 5.9 % / 5.9 % | 29457 / 29457 | 0 / 0 | 0 / 0 | 10.0 s / 350 MB | 9.4 s / 284 MB | 7.6 s / 47 MB |
| fraLSG-luk | 77.2 % / 77.2 % | 77.2 % / 77.2 % | 996 / 996 | 0 / 0 | 0 / 0 | 0.4 s / 25 MB | 0.4 s / 22 MB | 0.3 s / 13 MB |
| fob-pdf | 99.8 % / 99.8 % | 99.8 % / 99.8 % | 30782 / 30782 | 0 / 0 | 0 / 0 | 7.7 s / 289 MB | 8.1 s / 205 MB | 6.1 s / 34 MB |
| rv1909-pdf | 97.4 % / 97.4 % | 72.3 % / 72.3 % | 30582 / 30582 | 0 / 0 | 0 / 0 | 7.8 s / 224 MB | 7.0 s / 184 MB | 5.7 s / 32 MB |
| rv1909-lopdf | 100.0 % / 100.0 % | 100.0 % / 100.0 % | 30420 / 30420 | 0 / 0 | 0 / 0 | 3.1 s / 286 MB | 5.9 s / 276 MB | 3.8 s / 22 MB |
| spabes-pdf | 99.8 % / 99.8 % | 99.8 % / 99.8 % | 30240 / 30240 | 0 / 0 | 0 / 0 | 6.7 s / 206 MB | 6.8 s / 185 MB | 5.0 s / 33 MB |
| porbrbsl-pdf | 99.0 % / 99.0 % | 96.0 % / 96.0 % | 30199 / 30199 | 0 / 0 | 0 / 0 | 10.4 s / 313 MB | 10.0 s / 261 MB | 7.8 s / 44 MB |
| porbr2018-pdf | 99.9 % / 99.9 % | 99.9 % / 99.9 % | 29536 / 29536 | 1 / 1 | 0 / 0 | 7.1 s / 226 MB | 7.2 s / 193 MB | 5.7 s / 33 MB |
| kor-pdf | 99.5 % / 99.5 % | 99.5 % / 99.5 % | 30917 / 30917 | 0 / 0 | 0 / 0 | 14.3 s / 76 MB | 7.2 s / 74 MB | 5.5 s / 35 MB |
| grctr-pdf | 98.3 % / 98.3 % | 13.6 % / 13.6 % | 7640 / 7640 | 0 / 0 | 0 / 0 | 2.0 s / 33 MB | 2.5 s / 32 MB | 1.9 s / 16 MB |
| hboWLC-pdf | 65.9 % / 99.4 % | 65.8 % / 99.4 % | 0 / 0 | 0 / 0 | 0 / 0 | 5.0 s / 154 MB | 8.0 s / 150 MB | 5.7 s / 26 MB |
| latVUC-pdf | 96.0 % / 96.0 % | 8.6 % / 8.6 % | 8527 / 8527 | 0 / 0 | 0 / 0 | 15.0 s / 489 MB | 17.9 s / 414 MB | 13.7 s / 64 MB |
| deutkw-pdf | 99.6 % / 99.6 % | 58.3 % / 58.3 % | 29724 / 29724 | 5 / 5 | 0 / 0 | 7.8 s / 234 MB | 8.8 s / 215 MB | 5.8 s / 35 MB |
| frajnd-pdf | 84.9 % / 84.9 % | 4.5 % / 4.5 % | 29625 / 29804 | 0 / 0 | 0 / 0 | 10.3 s / 309 MB | 9.4 s / 254 MB | 7.6 s / 45 MB |
| spablm-pdf | 99.1 % / 99.1 % | 96.1 % / 96.1 % | 29104 / 29104 | 0 / 0 | 0 / 0 | 9.4 s / 305 MB | 10.1 s / 251 MB | 7.3 s / 43 MB |
| engylt-pdf | 99.9 % / 99.9 % | 99.9 % / 99.9 % | 31057 / 31057 | 0 / 0 | 0 / 0 | 7.1 s / 225 MB | 7.8 s / 196 MB | 5.7 s / 32 MB |
| grcbyz-pdf | 98.7 % / 98.7 % | 98.7 % / 98.7 % | 7921 / 7921 | 0 / 0 | 0 / 0 | 2.3 s / 30 MB | 2.8 s / 31 MB | 2.0 s / 17 MB |
| kor1961-lopdf | 100.0 % / 100.0 % | 34.9 % / 34.9 % | 30808 / 30808 | 0 / 0 | 0 / 0 | 22.6 s / 367 MB | 4.4 s / 350 MB | 2.4 s / 42 MB |
| deutkw-lopdf | 100.0 % / 100.0 % | 100.0 % / 100.0 % | 29821 / 29821 | 2 / 2 | 0 / 0 | 3.4 s / 391 MB | 6.0 s / 384 MB | 3.9 s / 29 MB |
| frajnd-lopdf | 100.0 % / 100.0 % | 96.4 % / 96.4 % | 26360 / 26356 | 0 / 0 | 0 / 0 | 3.6 s / 397 MB | 5.9 s / 387 MB | 3.8 s / 30 MB |
| engbsb-rwpdf2col | 89.1 % / 89.1 % | 1.5 % / 1.5 % | 22363 / 22363 | 0 / 0 | 0 / 0 | 4.5 s / 81 MB | 6.9 s / 82 MB | 5.0 s / 25 MB |
| engasv-rrlab1col | 97.1 % / 97.1 % | 28.3 % / 28.3 % | 27471 / 27471 | 1 / 1 | 0 / 0 | 5.2 s / 228 MB | 5.8 s / 218 MB | 3.8 s / 20 MB |
| spavbl-rlofodt | 88.2 % / 88.2 % | 49.7 % / 49.7 % | 3453 / 4020 | 0 / 0 | 0 / 0 | 5.6 s / 571 MB | 6.8 s / 550 MB | 4.3 s / 46 MB |
| grcsbl-rfpdf-mrk | 96.9 % / 96.9 % | 96.9 % / 96.9 % | 0 / 0 | 0 / 0 | 0 / 0 | 0.1 s / 13 MB | 0.2 s / 13 MB | 0.1 s / 13 MB |
| engrv-ebiblebook | 99.9 % / 99.9 % | 99.9 % / 99.9 % | 30408 / 30408 | 0 / 0 | 0 / 0 | 8.3 s / 277 MB | 9.8 s / 241 MB | 7.4 s / 37 MB |
| engkjvcpb-rwpdfmargin | 36.3 % / 36.3 % | 0.2 % / 0.2 % | 1 / 1 | 0 / 0 | 0 / 0 | 5.7 s / 246 MB | 7.6 s / 233 MB | 5.3 s / 27 MB |
| engjps-rdocx2lo | 96.4 % / 96.4 % | 2.1 % / 2.1 % | 20867 / 20867 | 0 / 0 | 0 / 0 | 4.0 s / 194 MB | 5.4 s / 197 MB | 3.3 s / 30 MB |
| spabll-rrlab2col | 80.9 % / 80.9 % | 19.2 % / 19.2 % | 0 / 0 | 0 / 0 | 0 / 0 | 3.7 s / 71 MB | 6.4 s / 68 MB | 4.2 s / 21 MB |
| poronbv-rwpdfa5 | 96.5 % / 96.5 % | 96.4 % / 96.4 % | 29988 / 29988 | 0 / 0 | 0 / 0 | 7.8 s / 268 MB | 7.9 s / 250 MB | 6.0 s / 30 MB |
| hbo-rwpdfrtl-rut | 95.3 % / 95.3 % | 95.3 % / 95.3 % | 0 / 0 | 0 / 0 | 0 / 0 | 0.0 s / 13 MB | 0.1 s / 13 MB | 0.0 s / 13 MB |
| engoebus-ebiblea4 | 99.0 % / 99.0 % | 98.9 % / 98.9 % | 13558 / 13558 | 0 / 0 | 0 / 0 | 3.1 s / 109 MB | 3.1 s / 90 MB | 2.2 s / 21 MB |
| engbrenton-rlo2col | 87.0 % / 87.0 % | 13.8 % / 13.8 % | 14409 / 14409 | 0 / 0 | 0 / 0 | 4.5 s / 275 MB | 6.2 s / 268 MB | 4.3 s / 38 MB |
| deu1912-rdropcap | 95.7 % / 95.7 % | 23.7 % / 23.7 % | 27284 / 27284 | 0 / 0 | 0 / 0 | 5.1 s / 232 MB | 7.4 s / 220 MB | 5.1 s / 26 MB |
| kor-r2col | 99.0 % / 99.0 % | 98.8 % / 98.8 % | 30723 / 30723 | 0 / 0 | 0 / 0 | 10.7 s / 73 MB | 14.6 s / 75 MB | 13.3 s / 26 MB |
| engwebu-rl2col | 94.9 % / 94.9 % | 3.3 % / 3.3 % | 7499 / 7307 | 0 / 0 | 0 / 0 | 5.9 s / 80 MB | 7.4 s / 80 MB | 4.6 s / 24 MB |
| engtnt-wp3col | 98.7 % / 98.7 % | 98.7 % / 98.7 % | 6040 / 6040 | 0 / 0 | 0 / 0 | 1.0 s / 25 MB | 1.6 s / 24 MB | 1.1 s / 17 MB |
| deutkw-fpdfa5 | 96.6 % / 96.6 % | 96.6 % / 96.6 % | 0 / 0 | 0 / 0 | 0 / 0 | 0.8 s / 54 MB | 1.4 s / 54 MB | 0.9 s / 13 MB |
| kor-lopdf | 95.4 % / 95.4 % | 70.5 % / 70.5 % | 65 / 65 | 0 / 0 | 0 / 0 | 5.2 s / 110 MB | 1.3 s / 108 MB | 0.7 s / 21 MB |
| heb-wprtl | 5.5 % / 5.5 % | 1.1 % / 1.1 % | 0 / 0 | 0 / 0 | 0 / 0 | 0.6 s / 15 MB | 0.8 s / 14 MB | 0.5 s / 13 MB |
| F01 | 97.1 % / 97.4 % | 29.2 % / 29.2 % | 0 / 8541 | 0 / 0 | 0 / 0 | 2.9 s / 76 MB | 1.9 s / 29 MB | 1.4 s / 17 MB |
| F02 | 96.4 % / 96.4 % | 18.3 % / 18.3 % | 20487 / 20487 | 0 / 0 | 0 / 0 | 4.0 s / 228 MB | 5.3 s / 220 MB | 3.4 s / 33 MB |
| F03 | 98.0 % / 98.0 % | 10.4 % / 10.4 % | 10848 / 6107 | 0 / 0 | 0 / 0 | 4.8 s / 79 MB | 7.2 s / 81 MB | 4.7 s / 22 MB |
| F06 | 96.1 % / 96.1 % | 94.8 % / 94.8 % | 4721 / 4721 | 0 / 0 | 0 / 0 | 0.9 s / 58 MB | 1.3 s / 58 MB | 0.9 s / 13 MB |
| F07 | 99.0 % / 99.0 % | 29.2 % / 29.2 % | 0 / 1 | 0 / 0 | 0 / 0 | 10.1 s / 237 MB | 6.4 s / 160 MB | 4.8 s / 30 MB |
| F13 | 69.4 % / 69.4 % | 63.3 % / 63.3 % | 7 / 7 | 0 / 0 | 0 / 0 | 1.0 s / 44 MB | 1.1 s / 41 MB | 0.8 s / 13 MB |
| G03 | 99.1 % / 99.1 % | 3.5 % / 3.5 % | 0 / 0 | 0 / 0 | 0 / 0 | 2.9 s / 62 MB | 5.4 s / 63 MB | 3.8 s / 22 MB |
| G04 | 99.5 % / 99.6 % | 99.5 % / 99.6 % | 0 / 0 | 0 / 0 | 0 / 0 | 1.6 s / 58 MB | 2.2 s / 54 MB | 1.7 s / 15 MB |
| G05 | — / 89.1 % | — / 46.6 % | — / 0 | — / 0 | — / 0 | — | 1.5 s / 28 MB | 1.1 s / 14 MB |
| G07 | 94.4 % / 94.4 % | 94.4 % / 94.4 % | 115 / 115 | 0 / 0 | 0 / 0 | 0.9 s / 45 MB | 1.3 s / 43 MB | 0.9 s / 13 MB |
| G09 | 90.4 % / 90.6 % | 90.4 % / 90.6 % | 0 / 0 | 0 / 0 | 0 / 0 | 0.2 s / 18 MB | 0.2 s / 13 MB | 0.1 s / 13 MB |
| G10 | 39.6 % / 73.4 % | 8.2 % / 25.9 % | 0 / 0 | 0 / 0 | 0 / 0 | 0.4 s / 25 MB | 0.3 s / 16 MB | 0.2 s / 13 MB |
| G13 | 48.6 % / 48.6 % | 48.6 % / 48.6 % | 0 / 0 | 0 / 0 | 0 / 0 | 0.1 s / 13 MB | 0.3 s / 13 MB | 0.2 s / 13 MB |
| **Total** | | | 896041 / 900392 | 9 / 9 | 0 / 0 | 305 s | 319 s | 234 s |
