# Synthetic regression sets: the A1.5 tools against E2

Every set is SYNTHETIC (invented text; `tests/evaluation/synthetic/make_e2_sets.py`), sealed, run once and
scored in a scratch Git repository by `tests/evaluation/synthetic/run_e2_sets.py`. **Before**: the A1.5
tools as frozen (`767e226`) and a release build of the frozen harness source (SHA-256 `e396328c…`, the
same as the independent audit's local build), scored with `--frozen-binary` as A1.5 allowed. **After**:
the E2 tools and the E2 harness (`800fb43c…`), scored with `--diagnostic` and a per-set rehearsal
protocol, so the official verdict is always *not a protocol run*; the diagnostic verdict is shown.

| Set | A1.5 verdict | A1.5 printed / verified / correct / wrong / absent | E2 diagnostic verdict | E2 printed + unsupported / verified / faithful / mismatch / absent / unvalidatable |
|---|---|---|---|---|
| `e2-fid` | pass | 171 / 171 / 171 / 0 / 0 | fail | 171 + 0 / 171 / 153 / 18 / 0 / 0 |
| `e2-mat` | fail | 46 / 46 / 43 / 3 / 0 | fail | 46 + 0 / 46 / 43 / 3 / 0 / 0 |
| `e2-ref` | pass | 195 / 189 / 189 / 0 / 0 | fail | 194 + 54 / 188 / 188 / 0 / 0 / 1 |
| `e2-abs` | fail | 101 / 95 / 95 / 0 / 23 | fail | 101 + 0 / 95 / 95 / 0 / 29 / 0 |
| `e2-name` | fail | 0 / 0 / 0 / 0 / 22 | pass | 22 + 0 / 22 / 22 / 0 / 0 / 0 |
| `e2-map` | pass | 73 / 47 / 47 / 0 / 0 | — | seal refused: e2-map-jol: reference VerseMap.xml: a verse map, which would renumber the reference; never read |
| `e2-utf8` | fail | 21 / 21 / 20 / 1 / 0 | — | seal refused: e2-utf8-oba: reference 41-OBA.usfm: not UTF-8 (read with replacement characters) |

## Per document

| Document | A1.5: printed / verified / correct / wrong (of which typography) / absent | E2: printed + unsupported / faithful / mismatch (kinds) / absent / unvalidatable |
|---|---|---|
| `e2-fid-3jn` | 15 / 15 / 15 / 0 (2) / 0 | 15 + 0 / 13 / 2 (marks 2) / 0 / 0 |
| `e2-fid-jud-nfd` | 25 / 25 / 25 / 0 (25) / 0 | 25 + 0 / 25 / 0 / 0 / 0 |
| `e2-fid-oba-he` | 21 / 21 / 21 / 0 (1) / 0 | 21 + 0 / 20 / 1 (marks 1) / 0 / 0 |
| `e2-fid-phm` | 25 / 25 / 25 / 0 (1) / 0 | 25 + 0 / 22 / 3 (marks 1, word_boundary 2) / 0 / 0 |
| `e2-fid-rut` | 85 / 85 / 85 / 0 (8) / 0 | 85 + 0 / 73 / 12 (case 2, punctuation 8, word_boundary 2) / 0 / 0 |
| `e2-mat-oba` | 21 / 21 / 19 / 2 (0) / 0 | 21 + 0 / 19 / 2 (changed 2) / 0 / 0 |
| `e2-mat-phm` | 25 / 25 / 24 / 1 (0) / 0 | 25 + 0 / 24 / 1 (marks 1) / 0 / 0 |
| `e2-ref-3jn` | 14 / 13 / 13 / 0 (0) / 0 | 13 + 8 (chapter_beyond_canon 4, verse_duplicate 2, verse_label_unusual 2) / 12 / 0 / 0 / 1 |
| `e2-ref-exo0a` | 36 / 36 / 36 / 0 (0) / 0 | 36 + 1 (verse_zero 1) / 36 / 0 / 0 / 0 |
| `e2-ref-exo0b` | 36 / 36 / 36 / 0 (0) / 0 | 36 + 1 (verse_zero 1) / 36 / 0 / 0 / 0 |
| `e2-ref-exo0c` | 36 / 36 / 36 / 0 (0) / 0 | 36 + 1 (verse_zero 1) / 36 / 0 / 0 / 0 |
| `e2-ref-jol` | 52 / 47 / 47 / 0 (0) / 0 | 52 + 21 (chapter_beyond_canon 21) / 47 / 0 / 0 / 0 |
| `e2-ref-tob` | 21 / 21 / 21 / 0 (0) / 0 | 21 + 22 (book_outside_canon 22) / 21 / 0 / 0 / 0 |
| `e2-abs-jud` | 25 / 25 / 25 / 0 (0) / 1 | 25 + 0 / 25 / 0 / 1 / 0 |
| `e2-abs-mal` | 55 / 49 / 49 / 0 (0) / 0 | 55 + 0 / 49 / 0 / 6 / 0 |
| `e2-abs-rut` | 21 / 21 / 21 / 0 (0) / 22 | 21 + 0 / 21 / 0 / 22 / 0 |
| `e2-name-gen` | 0 / 0 / 0 / 0 (0) / 22 | 22 + 0 / 22 / 0 / 0 / 0 |
| `e2-map-jol` | 73 / 47 / 47 / 0 (0) / 0 | not sealed (refused) |
| `e2-utf8-oba` | 21 / 21 / 20 / 1 (0) / 0 | not sealed (refused) |

A1.5 never lists its class-1 ("normalised") verses: in `e2-fid` the raw reports count `verified_normalised` 4 (Ruth) and 2 (Philemon), which no score names.
