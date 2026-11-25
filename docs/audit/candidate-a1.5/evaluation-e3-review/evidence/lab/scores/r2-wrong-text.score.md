## Verdict: **fail**

**FAIL (safety fail; evidence sufficient): verified 85 (84 faithful, 1 wrong text) of 85 printed scorable verses; 0 verified references the edition lacks or cannot validate; 0 printed unsupported (not scored); 0 held back; 0 in refused chapters; 0 missing (not found); scored documents 1 completed, 0 refused whole, 0 failed to run, of 1.**

Safety (criteria 1-4): **fail**. Evidence (the protocol's floor, aw-evidence-1): **sufficient**.

Run record `0daaf6f61b4f7136bb42605d78fa63ffce98f8b6` (sets/r2-wrong-text/run), seal `317482d218b94f1d0073c4a23a902aa1aa48f852`, protocol `92f5e5698e23c3d2b45c8d7907d704a312dbefaaf9227a0f42bf557fc4a26e49`, scorer `16a84231be46ee42…` (rules aw-eval-3, aw-fidelity-2, aw-reference-inventory-2, aw-usfm-text-1). Before relying on this score, check with sha256sum that docs/evaluation/protocol-e3.json is 92f5e5698e23c3d2b45c8d7907d704a312dbefaaf9227a0f42bf557fc4a26e49 and that this is the protocol you approved; its approved_tools then name every tool by SHA-256.

1 verified verses with wrong text under the strict fidelity rule (changed 1), 0 verified references outside the edition and 0 the edition cannot validate, among 85 verified verses of 1 completed scored documents (0 authentic). They print 85 scorable verses and 0 verses the recogniser cannot hold (unsupported, by class: none). Faithful coverage (exact, canonical or layout, verified / printed; no pass mark): 98.8 % of scorable verses, 98.8 % of all printed verses. Exploratory documents are not part of this figure.

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
| run_under_this_protocol | yes |
| protocol_permits_withheld_execution | yes |
| composition_declared_and_matching | yes |
| composition_required_by_protocol_and_met | yes |
| check_report_is_e3 | yes |
| unicode_versions_as_protocol | yes |
| tools_are_the_approved_ones | yes |
| run_used_the_approved_tools | yes |
| reports_trusted | yes |
| scorer_committed | yes |

| Criterion (safety) | Met |
|---|---|
| 1_no_verified_text_mismatch | **no** |
| 2_no_verified_reference_outside_or_unvalidatable | yes |
| 3_no_verified_reference_in_controls_or_outside_allowed | yes |
| 4_every_document_completed_or_refused_cleanly | yes |

| Evidence floor | Floor | Observed | Met |
|---|---:|---:|---|
| minimum_verified_faithful | 1 | 84 | yes |

| Counts (completed scored documents) | |
|---|---:|
| scored_documents | 1 |
| completed | 1 |
| refused_whole | 0 |
| failed_to_run | 0 |
| printed_scorable | 85 |
| printed_unsupported | 0 |
| verified | 85 |
| verified_faithful | 84 |
| verified_mismatch | 1 |
| verified_absent_or_unvalidatable | 0 |
| held | 0 |
| refused_verses | 0 |
| missing | 0 |
| documents_verifying_nothing | none |

| Execution | Documents |
|---|---:|
| completed | 1 |

| Scored (completed, exact reference) | Value |
|---|---:|
| documents | 1 |
| printed | 85 |
| printed_unsupported | 0 |
| verified | 85 |
| verified_exact | 84 |
| verified_canonical | 0 |
| verified_layout | 0 |
| verified_faithful | 84 |
| verified_mismatch | 1 |
| verified_mismatch_material | 1 |
| verified_absent | 0 |
| verified_unvalidatable | 0 |
| held | 0 |
| held_faithful | 0 |
| held_mismatch | 0 |
| refused | 0 |
| not_found | 0 |
| refused_chapters | 0 |
| verified mismatch, changed | 1 |

| Other classes | Documents | Completed | Verified | Of which failures |
|---|---:|---:|---:|---:|
| control | 0 | 0 | 0 | 0 |
| mixed | 0 | 0 | 0 | 0 |
| exploratory | 0 | 0 | raw, per document | not judged |

| Scored by format | Docs | Printed | Unsupported | Verified | Faithful | Mismatch | Absent | Unvalidatable | Held faithful | Held mismatch | Refused | Missing |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| md | 1 | 85 | 0 | 85 | 84 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |

| Scored by language | Docs | Printed | Unsupported | Verified | Faithful | Mismatch | Absent | Unvalidatable | Held faithful | Held mismatch | Refused | Missing |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| en | 1 | 85 | 0 | 85 | 84 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |

| Scored by script | Docs | Printed | Unsupported | Verified | Faithful | Mismatch | Absent | Unvalidatable | Held faithful | Held mismatch | Refused | Missing |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Latin | 1 | 85 | 0 | 85 | 84 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |

| Scored by kind | Docs | Printed | Unsupported | Verified | Faithful | Mismatch | Absent | Unvalidatable | Held faithful | Held mismatch | Refused | Missing |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| generated | 1 | 85 | 0 | 85 | 84 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |

| Document | Class | Kind | Execution | Exit | Printed | Unsupported | Verified | Faithful | Mismatch | Absent | Unvalidatable | Held | Refused | Missing | Outside allowed | Peak MB |
|---|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `wt` | scored | generated | completed | 0 | 85 | 0 | 85 | 84 | 1 | 0 | 0 | 0 | 0 | 0 |  | 20 |

### Verified verses with wrong text

- `wt` RUT 1:2: changed
