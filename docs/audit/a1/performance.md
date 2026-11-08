# A1 performance: frozen against structure-aware reading

Same machine, same unoptimised test build profile (`cargo test` with debug information off), one edition after another in one process per set. Seconds are per edition. **Peak MB is the process peak since the run started**, so it accumulates across the editions of a set and is not a per-edition figure.

| Set | Edition | Frozen s | Structural s | Frozen peak MB | Structural peak MB |
|---|---|---|---|---|---|
| g | G01 | 86.3 | 92.4 | 98 | 505 |
| g | G12 | 25.1 | 26.9 | 90 | 496 |
| g | G14 | 5.0 | 5.0 | 90 | 496 |
| f | F04 | 69.4 | 109.1 | 75 | 423 |
| f | F10 | 12.6 | 12.2 | 70 | 418 |
| f | F12 | 54.9 | 51.8 | 85 | 418 |
| e | engwyc2017-epubapoc | 11.3 | 10.9 | 34 | 64 |
| e | fra_fob-pandocepub | 33.4 | 33.9 | 44 | 155 |
| recon | spapddpt-repub2 | 88.1 | 88.6 | 71 | 356 |
| bc | englsv-rpandocepub | 106.4 | 104.6 | 394 | 427 |
| bc | francl-rebooklibepub | 116.3 | 107.5 | 98 | 316 |
| bc | frasbl-rpandocepub | 96.6 | 97.2 | 396 | 407 |
| dev | deu1912-epub | 95.3 | 137.6 | 355 | 419 |
| dev | engwebp-epub | 101.1 | 145.9 | 72 | 432 |
| dev | fob-epub | 99.1 | 140.0 | 388 | 420 |
| dev | fraLSG-epub | 145.3 | 183.5 | 355 | 490 |
| dev | frajnd-epub | 95.1 | 136.4 | 389 | 482 |
| dev | grctr-epub | 24.7 | 31.1 | 389 | 424 |
| dev | hboWLC-epub | 269.9 | 280.4 | 389 | 424 |
| dev | kjv-epub | 98.7 | 139.9 | 354 | 429 |
| dev | kor-epub | 124.5 | 159.6 | 389 | 437 |
| dev | latVUC-epub | 131.3 | 179.3 | 389 | 595 |
| dev | porbr2018-epub | 98.4 | 137.7 | 389 | 429 |
| dev | rv1909-epub | 92.3 | 133.8 | 389 | 417 |
| | **Total** | 2081 | 2545 | max 396 | max 595 |
