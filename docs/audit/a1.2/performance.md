# Phase A1.2: time, memory and provenance on the 24 EPUB editions

Release build, one reading at a time on an otherwise idle 4-core Linux
machine. For each edition A1.1 (`0e4c08e` = `5b2bbdd` plus the harness's
reading-only fields) and then A1.2 (`e9c406a`) were run in turn, so drift
in the machine affects both alike.

* **Reading s / peak MB**: from the start of reading to the staged
  content and the provenance report (`read_seconds`,
  `read_peak_memory_mb`). This is what the app does on import.
* **Whole run peak MB**: the harness's own peak, including comparing every
  verse with the edition and writing the report afterwards. It is not app
  memory.
* **Every verse's provenance**: SHA-256 of the whole provenance report
  (every verse's status, spans with character and byte offsets, excluded
  ranges with their kinds, reasons and id checks), computed by a scratch
  addition to the harness after the reading's own measurements were
  taken. The same digest means the same provenance for every verse.

| Set | Edition | Reading s, A1.1 → A1.2 | Reading peak MB, A1.1 → A1.2 | Whole run peak MB, A1.1 → A1.2 | Every verse's provenance |
|---|---|---|---|---|---|
| g | `G01` | 14.9 → 14.0 | 483 → 248 (-49 %) | 549 → 496 | identical (25062 verses) |
| g | `G12` | 3.1 → 3.0 | 72 → 41 (-44 %) | 118 → 118 | identical (8400 verses) |
| g | `G14` | 1.0 → 0.9 | 29 → 20 (-30 %) | 29 → 26 | identical (773 verses) |
| f | `F04` | 18.3 → 18.0 | 309 → 140 (-55 %) | 410 → 434 | identical (31140 verses) |
| f | `F10` | 1.5 → 1.6 | 65 → 36 (-44 %) | 77 → 77 | identical (4403 verses) |
| f | `F12` | 9.4 → 9.8 | 214 → 93 (-57 %) | 214 → 93 | identical (2526 verses) |
| e | `engwyc2017-epubapoc` | 1.7 → 1.8 | 59 → 31 (-47 %) | 59 → 31 | identical (73 verses) |
| e | `fra_fob-pandocepub` | 4.1 → 4.0 | 92 → 47 (-49 %) | 150 → 150 | identical (10423 verses) |
| recon | `spapddpt-repub2` | 14.9 → 14.6 | 263 → 118 (-55 %) | 353 → 354 | identical (24837 verses) |
| bc | `englsv-rpandocepub` | 12.7 → 12.2 | 310 → 121 (-61 %) | 405 → 427 | identical (31104 verses) |
| bc | `francl-rebooklibepub` | 16.1 → 15.4 | 355 → 144 (-60 %) | 355 → 266 | identical (15918 verses) |
| bc | `frasbl-rpandocepub` | 13.0 → 12.6 | 339 → 134 (-61 %) | 384 → 383 | identical (28331 verses) |
| dev | `deu1912-epub` | 21.1 → 21.3 | 279 → 117 (-58 %) | 395 → 417 | identical (31092 verses) |
| dev | `engwebp-epub` | 24.7 → 26.0 | 306 → 146 (-52 %) | 412 → 417 | identical (31098 verses) |
| dev | `fob-epub` | 20.6 → 20.8 | 282 → 118 (-58 %) | 400 → 419 | identical (31107 verses) |
| dev | `fraLSG-epub` | 32.8 → 34.4 | 379 → 180 (-53 %) | 472 → 465 | identical (31170 verses) |
| dev | `frajnd-epub` | 21.2 → 23.9 | 341 → 157 (-54 %) | 460 → 454 | identical (30921 verses) |
| dev | `grctr-epub` | 5.4 → 5.7 | 84 → 48 (-42 %) | 130 → 131 | identical (7957 verses) |
| dev | `hboWLC-epub` | 44.0 → 44.1 | 220 → 105 (-52 %) | 344 → 345 | identical (23192 verses) |
| dev | `kjv-epub` | 22.1 → 24.3 | 346 → 158 (-54 %) | 430 → 428 | identical (31102 verses) |
| dev | `kor-epub` | 25.6 → 27.3 | 211 → 127 (-40 %) | 407 → 405 | identical (30990 verses) |
| dev | `latVUC-epub` | 29.2 → 30.8 | 599 → 219 (-63 %) | 599 → 445 | identical (31228 verses) |
| dev | `porbr2018-epub` | 19.7 → 21.3 | 276 → 127 (-54 %) | 406 → 428 | identical (31102 verses) |
| dev | `rv1909-epub` | 19.0 → 21.1 | 265 → 116 (-56 %) | 393 → 393 | identical (31084 verses) |
| | **Total / max / median** | 396 → 409 | max 599 → 248; median 278 → 120 | | all identical (525033 verses) |

The 12 development EPUBs from `deu1912-epub` to `rv1909-epub` are the
byte-order-marked ones (every XHTML entry starts with a BOM). G01, Hebrew
(`hboWLC-epub`), Korean (`kor-epub`) and the largest (`latVUC-epub`, 15 MB
of XHTML) are in the table.

Per-edition data: `data/epub-timing.json`.
