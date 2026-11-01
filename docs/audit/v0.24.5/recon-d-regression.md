# Set D layouts, reconstructed — frozen v0.24.4 recogniser (baseline, 2b5d4e2) against v0.24.5 frozen at fc0f538

Regression evidence, not set D: documents rebuilt in set D's disclosed layouts by `scripts/recon/build_recon_d.py` (`docs/corpus/recon-d.tsv`). Same files, same harness, both runs. *Verified wrong text*: verified verses whose text differs from the edition's USFM beyond whitespace and typography. *Verified FPs*: verified verses the edition does not have (here: Septuagint Esther/Daniel filed as EST/DAN).

## Results

| Edition | Printed | Detected before | Detected after | Verified before | Verified after | Δ verified | Verified FPs before / after | Verified wrong text before / after |
|---|---|---|---|---|---|---|---|---|
| deu1912-rdropcap | 31102 | 26867 | 27922 (89.8 %) | 26211 | 27291 (87.7 %) | +1080 | 0 / 0 | 254 / 0 |
| engbrenton-rlo2col | 22358 | 8786 | 16365 (73.2 %) | 7937 | 14409 (64.4 %) | +6472 | 359 / 0 | 161 / 0 |
| grcbrent-rtxtref | 21949 | 16872 | 15144 (69.0 %) | 16703 | 15060 (68.6 %) | -1643 | 449 / 0 | 2 / 0 |
| kor-r2col | 30991 | 30798 | 30845 (99.5 %) | 30088 | 30824 (99.5 %) | +736 | 2 / 0 | 621 / 0 |
| portft-rmdcvbold | 7898 | 189 | 189 (2.4 %) | 185 | 154 (1.9 %) | -31 | 0 / 0 | 31 / 0 |
| spaonbv-rlodocx-poet | 4648 | 3785 | 3785 (81.4 %) | 3778 | 3778 (81.3 %) | +0 | 0 / 0 | 0 / 0 |
| spapddpt-repub2 | 31081 | 20818 | 20818 (67.0 %) | 19601 | 19530 (62.8 %) | -71 | 0 / 0 | 69 / 0 |
| **Total** | 150027 | 108115 | 115068 (76.7 %) | 104503 | 111046 (74.0 %) | +6543 | 810 / 0 | 1138 / 0 |

### Every verified verse with wrong text or not in the edition, after

None.
