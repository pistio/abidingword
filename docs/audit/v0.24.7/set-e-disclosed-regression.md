# Set E sealed files — disclosed regression: 324cfd1 → c96756d

Disclosed evidence, not independent.

Recogniser frozen at `c96756d`; baseline is v0.24.6 frozen at `324cfd1`. Same input files at both runs (`corpus-checksums.sha256`, verified before the final run).

## Results

| Edition | Printed | Detected before | Detected after | Verified before | Verified after | Δ verified | Verified FPs before / after | Verified wrong text before / after |
|---|---|---|---|---|---|---|---|---|
| deutkw-fpdfa5 | 5465 | 4735 | 4735 (86.6 %) | 0 | 0 (0.0 %) | +0 | 0 / 0 | 0 / 0 |
| enggnv-docxverse | 7953 | 7953 | 7953 (100.0 %) | 7950 | 7950 (100.0 %) | +0 | 0 / 0 | 0 / 0 |
| engtnt-wp3col | 7954 | 6308 | 6308 (79.3 %) | 6040 | 6040 (75.9 %) | +0 | 0 / 0 | 0 / 0 |
| engwebu-rl2col | 31098 | 7758 | 7758 (24.9 %) | 7499 | 7499 (24.1 %) | +0 | 0 / 0 | 0 / 0 |
| engwyc2017-epubapoc | 0 | 0 | 0 (—) | 0 | 0 (—) | +0 | 0 / 0 | 0 / 0 |
| fra_fob-pandocepub | 10423 | 10423 | 10423 (100.0 %) | 10387 | 10387 (99.7 %) | +0 | 0 / 0 | 0 / 0 |
| grcmt-txt | 7953 | 7953 | 7953 (100.0 %) | 7953 | 7953 (100.0 %) | +0 | 0 / 0 | 0 / 0 |
| heb-wprtl | 5852 | 0 | 0 (0.0 %) | 0 | 0 (0.0 %) | +0 | 0 / 0 | 0 / 0 |
| kor-lopdf | 7920 | 7428 | 7428 (93.8 %) | 65 | 65 (0.8 %) | +0 | 0 / 0 | 0 / 0 |
| porbrbsl-md | 11329 | 10925 | 10925 (96.4 %) | 10903 | 10903 (96.2 %) | +0 | 0 / 0 | 0 / 0 |
| sparvg-pandocdocx | 31102 | 31102 | 31102 (100.0 %) | 30992 | 30992 (99.6 %) | +0 | 0 / 0 | 0 / 0 |
| spav1602p-lodocx | 3713 | 3713 | 3713 (100.0 %) | 3709 | 3709 (99.9 %) | +0 | 2 / 2 | 2 / 2 |
| **Total** | 130762 | 98298 | 98298 (75.2 %) | 85498 | 85498 (65.4 %) | +0 | 2 / 2 | 2 / 2 |

### Every verified verse with wrong text or not in the edition, after

* spav1602p-lodocx PSA 41:8: verified, but the edition has no such verse
* spav1602p-lodocx PRO 4:17: verified, but the edition has no such verse
* spav1602p-lodocx PSA 41:7 (missing, page None): delete «­» → «»; delete « 8 Cosa pestilencial, dicen, se le ha pegado; y el que cayó » → «»
* spav1602p-lodocx PRO 4:16 (missing, page None): delete « 17 Porque comen pan de mal­dad, y beben vino de robos.» → «»

### Unverified reasons, change per document

| Doc | Change |
|---|---|
| **Total** |  |
