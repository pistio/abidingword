# Set E's sealed files, run again — disclosed regression, NOT an independent evaluation

The original single evaluation of set E (fc0f538) is `docs/audit/v0.24.5/withheld-e/`, unchanged; it is the record. The rules of v0.24.6 were written with set E's disclosed failures in view, so the figures below say only that those failure classes are addressed on these files. "Before" is the original run's output, unmodified.

## Results

| Edition | Printed | Detected before | Detected after | Verified before | Verified after | Δ verified | Verified FPs before / after | Verified wrong text before / after |
|---|---|---|---|---|---|---|---|---|
| deutkw-fpdfa5 | 5465 | 2056 | 4735 (86.6 %) | 0 | 0 (0.0 %) | +0 | 0 / 0 | 0 / 0 |
| enggnv-docxverse | 7953 | 7953 | 7953 (100.0 %) | 7950 | 7950 (100.0 %) | +0 | 0 / 0 | 0 / 0 |
| engtnt-wp3col | 7954 | 6308 | 6308 (79.3 %) | 6039 | 6040 (75.9 %) | +1 | 0 / 0 | 68 / 0 |
| engwebu-rl2col | 31098 | 7758 | 7758 (24.9 %) | 7500 | 7499 (24.1 %) | -1 | 0 / 0 | 1 / 0 |
| engwyc2017-epubapoc | 0 | 0 | 0 (—) | 0 | 0 (—) | +0 | 0 / 0 | 0 / 0 |
| fra_fob-pandocepub | 10423 | 10423 | 10423 (100.0 %) | 10409 | 10387 (99.7 %) | -22 | 0 / 0 | 0 / 0 |
| grcmt-txt | 7953 | 7865 | 7953 (100.0 %) | 7640 | 7953 (100.0 %) | +313 | 7 / 0 | 120 / 0 |
| heb-wprtl | 5852 | 0 | 0 (0.0 %) | 0 | 0 (0.0 %) | +0 | 0 / 0 | 0 / 0 |
| kor-lopdf | 7920 | 7428 | 7428 (93.8 %) | 81 | 65 (0.8 %) | -16 | 0 / 0 | 1 / 0 |
| porbrbsl-md | 11329 | 10925 | 10925 (96.4 %) | 10621 | 10903 (96.2 %) | +282 | 4 / 0 | 700 / 0 |
| sparvg-pandocdocx | 31102 | 31102 | 31102 (100.0 %) | 31002 | 30992 (99.6 %) | -10 | 0 / 0 | 0 / 0 |
| spav1602p-lodocx | 3713 | 3713 | 3713 (100.0 %) | 3711 | 3709 (99.9 %) | -2 | 2 / 2 | 2 / 2 |
| **Total** | 130762 | 95531 | 98298 (75.2 %) | 84953 | 85498 (65.4 %) | +545 | 13 / 2 | 892 / 2 |

### Every verified verse with wrong text or not in the edition, after

* spav1602p-lodocx PSA 41:8: verified, but the edition has no such verse
* spav1602p-lodocx PRO 4:17: verified, but the edition has no such verse
* spav1602p-lodocx PSA 41:7 (missing, page None): delete «­» → «»; delete « 8 Cosa pestilencial, dicen, se le ha pegado; y el que cayó » → «»
* spav1602p-lodocx PRO 4:16 (missing, page None): delete « 17 Porque comen pan de mal­dad, y beben vino de robos.» → «»

| Edition | Change in unverified reasons |
|---|---|
| deutkw-fpdfa5 | boundary +1907, noteSign -1, unnumbered -6 |
| engtnt-wp3col | boundary -2, noteSign +8, possibleHeading -7 |
| engwebu-rl2col | boundary +10, noteSign -1, possibleHeading +13, reference -3 |
| fra_fob-pandocepub | boundary +22 |
| grcmt-txt | possibleHeading -233 |
| kor-lopdf | boundary +16 |
| porbrbsl-md | boundary -19, possibleHeading -47, reference -217 |
| sparvg-pandocdocx | boundary +10 |
| spav1602p-lodocx | boundary +2 |
| **Total** | boundary +1946, noteSign +6, possibleHeading -274, reference -220, unnumbered -6 |
