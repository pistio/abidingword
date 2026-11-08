# A1.1 performance: frozen against structure-aware reading, optimised build

`cargo test --release` (opt-level 3, no debug information), the real-document harness, **one process per edition and reading**, frozen then structural for each edition in turn on an otherwise idle 4-core machine. Seconds are the whole harness run for the edition (reading, recognition, comparison with the edition, report). Peak MB is that process's own peak resident memory (`peak_memory_mb`), so it is per edition.

| Set | Edition | Frozen s | Structural s | Δ | Frozen peak MB | Structural peak MB | Δ MB |
|---|---|---|---|---|---|---|---|
| g | G01 | 8.2 | 9.7 | +18 % | 88 | 544 | +456 |
| g | G12 | 1.9 | 2.1 | +12 % | 26 | 118 | +92 |
| g | G14 | 0.5 | 0.5 | +3 % | 19 | 29 | +10 |
| f | F04 | 5.3 | 12.4 | +134 % | 65 | 410 | +345 |
| f | F10 | 1.0 | 1.1 | +17 % | 25 | 73 | +48 |
| f | F12 | 6.7 | 7.1 | +5 % | 70 | 214 | +143 |
| e | engwyc2017-epubapoc | 1.2 | 1.4 | +11 % | 24 | 59 | +35 |
| e | fra_fob-pandocepub | 2.4 | 3.0 | +24 % | 28 | 143 | +115 |
| recon | spapddpt-repub2 | 8.1 | 8.8 | +9 % | 60 | 335 | +275 |
| bc | englsv-rpandocepub | 7.2 | 8.6 | +19 % | 58 | 405 | +347 |
| bc | francl-rebooklibepub | 9.5 | 10.4 | +9 % | 87 | 362 | +275 |
| bc | frasbl-rpandocepub | 7.5 | 9.0 | +20 % | 65 | 383 | +318 |
| dev | deu1912-epub | 6.4 | 13.9 | +117 % | 50 | 395 | +344 |
| dev | engwebp-epub | 9.5 | 16.9 | +78 % | 62 | 412 | +350 |
| dev | fob-epub | 6.6 | 13.8 | +108 % | 51 | 396 | +345 |
| dev | fraLSG-epub | 13.4 | 21.1 | +57 % | 78 | 466 | +388 |
| dev | frajnd-epub | 7.1 | 15.2 | +114 % | 64 | 454 | +389 |
| dev | grctr-epub | 3.0 | 4.2 | +43 % | 30 | 124 | +94 |
| dev | hboWLC-epub | 26.6 | 31.9 | +20 % | 67 | 323 | +256 |
| dev | kjv-epub | 6.6 | 14.1 | +116 % | 61 | 405 | +344 |
| dev | kor-epub | 11.8 | 18.9 | +61 % | 63 | 405 | +343 |
| dev | latVUC-epub | 10.3 | 19.8 | +93 % | 88 | 597 | +509 |
| dev | porbr2018-epub | 6.6 | 13.4 | +104 % | 54 | 406 | +352 |
| dev | rv1909-epub | 6.3 | 13.2 | +110 % | 49 | 392 | +343 |
| | **Total** | 174 | 271 | +56 % | max 88 | max 597 | |
