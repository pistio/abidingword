## Verdict: **fail**

Run record `d50ba35a437f801ebbe5bd21fd22556e9642edae` (docs/evaluation/zz-b/run), seal `8769c0a27c7cd4f4916dfe4f66d231cb522e6f44`, scorer `658de955cc4594b5…`.

3 verified verses with wrong text and 24 verified references outside the edition, among 385 verified verses of 13 completed scored documents (0 authentic), which print 424 verses: coverage 90.1 % (verified correct / printed; no pass mark). Exploratory documents are not part of this figure. Not in the coverage figure, and not recognition coverage: 1 scored document(s) refused whole (zzaudit-b-empty) and 0 that failed to run, of 14 scored documents; no verse from them counts as verified.

**Qualification limit:** Scored authentic evidence (0 documents, 0 printed verses, formats none, languages none) is below the declared minimum (8 documents, 20000 verses, 2 formats, 4 languages). The result qualifies the candidate for the generated documents and for the authentic documents listed only; it makes no claim for authentic published files in general.

| Run validity | |
|---|---|
| anchored_and_intact | yes |
| run_complete | yes |
| every_document_has_a_record | yes |
| seal_matches | yes |
| frozen_binary | yes |
| protocol_limits | yes |
| canonical_inventory | yes |

| Criterion | Met |
|---|---|
| 1_no_wrong_verified_text | **no** |
| 2_no_verified_reference_outside_the_edition | **no** |
| 3_no_verified_reference_in_controls_or_outside_allowed | yes |
| 4_every_document_completed_or_refused_cleanly | yes |

| Execution | Documents |
|---|---:|
| completed | 13 |
| refused | 1 |

| Scored (completed, exact reference) | Value |
|---|---:|
| documents | 13 |
| printed | 424 |
| verified | 385 |
| verified_correct | 382 |
| verified_wrong | 3 |
| verified_typography | 0 |
| verified_absent | 24 |
| refused_correct | 2 |
| caught | 0 |
| refused_chapters | 4 |

| Other classes | Documents | Completed | Verified | Of which failures |
|---|---:|---:|---:|---:|
| control | 0 | 0 | 0 | 0 |
| mixed | 0 | 0 | 0 | 0 |
| exploratory | 0 | 0 | raw, per document | not judged |

| Scored by format | Docs | Printed | Correct | Wrong | Absent | Refused | Caught |
|---|---:|---:|---:|---:|---:|---:|---:|
| md | 13 | 424 | 382 | 3 | 24 | 2 | 0 |

| Scored by language | Docs | Printed | Correct | Wrong | Absent | Refused | Caught |
|---|---:|---:|---:|---:|---:|---:|---:|
| en | 11 | 374 | 334 | 1 | 24 | 2 | 0 |
| es | 2 | 50 | 48 | 2 | 0 | 0 | 0 |

| Scored by script | Docs | Printed | Correct | Wrong | Absent | Refused | Caught |
|---|---:|---:|---:|---:|---:|---:|---:|
| Latin | 13 | 424 | 382 | 3 | 24 | 2 | 0 |

| Scored by kind | Docs | Printed | Correct | Wrong | Absent | Refused | Caught |
|---|---:|---:|---:|---:|---:|---:|---:|
| generated | 13 | 424 | 382 | 3 | 24 | 2 | 0 |

| Document | Class | Kind | Execution | Exit | Printed | Verified | Wrong | Absent | Outside allowed | Peak MB |
|---|---|---|---|---:|---:|---:|---:|---:|---:|---:|
| `zzaudit-b-exo0a` | scored | generated | completed | 0 | 36 | 36 | 0 | 0 |  | 20 |
| `zzaudit-b-exo0b` | scored | generated | completed | 0 | 36 | 36 | 0 | 0 |  | 20 |
| `zzaudit-b-exo0c` | scored | generated | completed | 0 | 36 | 36 | 0 | 0 |  | 20 |
| `zzaudit-b-mal` | scored | generated | completed | 0 | 55 | 49 | 0 | 0 |  | 20 |
| `zzaudit-b-jol` | scored | generated | completed | 0 | 52 | 47 | 0 | 0 |  | 20 |
| `zzaudit-b-jud` | scored | generated | completed | 0 | 22 | 20 | 0 | 0 |  | 20 |
| `zzaudit-b-jud2` | scored | generated | completed | 0 | 22 | 22 | 0 | 2 |  | 20 |
| `zzaudit-b-tob` | scored | generated | completed | 0 | 21 | 21 | 0 | 0 |  | 20 |
| `zzaudit-b-gen` | scored | generated | completed | 0 | 0 | 0 | 0 | 22 |  | 20 |
| `zzaudit-b-add` | scored | generated | completed | 0 | 25 | 25 | 1 | 0 |  | 20 |
| `zzaudit-b-wrong` | scored | generated | completed | 0 | 21 | 21 | 1 | 0 |  | 20 |
| `zzaudit-b-accent` | scored | generated | completed | 0 | 25 | 25 | 1 | 0 |  | 20 |
| `zzaudit-b-map` | scored | generated | completed | 0 | 73 | 47 | 0 | 0 |  | 20 |
| `zzaudit-b-empty` | scored | generated | refused | 0 |  |  |  |  |  |  |
