# Windows, Library, Teaching and Study integration

Controlled local integration, 28 October 2026. Owner review is required before any further integration or publication. This branch does not merge to `main`, publish installers, deploy, dispatch Actions, implement Teaching Stage 3, or change the separate Bible recognition work.

## Verified sources

All remote branch tips were fetched and checked with `git ls-remote` before integration.

| Role | Branch | Exact SHA |
| --- | --- | --- |
| Integration / Windows-compatible Library Stage 2 base | `integration/windows-library-teaching-study-review` | `b3120f54b0dcece6c078727c97240828680dd63c` |
| Complete Teaching, including PR #6 and #7 repairs | `feature/teaching-three-templates` | `e275a1d6a166eb2471a9bb23592a98767569036f` |
| Reviewed Study simplification | `feature/study-flow-and-teaching-simplification` | `981f4c45bb8650ae5f11fe35112089485f697496` |

The divergent `main` tip `a7bc798407dc3bd60598ef5991e7c04e8d4356f1` was observed but was not used as the integration base or merged. The Teaching history includes completion `756d0097a07cf7105ce7c12313c0a380503b707e` and correctness repairs `314db4d16b35e079f2c894ab93cef949a544fd8e` without cherry-picking duplicate implementations.

## Teaching conflict resolution

The local merge of the complete Teaching branch conflicted only in `docs/i18n/native-polish-review-v0.19.md` and `docs/i18n/native-polish-priority-v0.19.md`. Both were regenerated from the combined language packs using `npm run i18n:corpus`. `src/lib/api.ts` merged automatically with both Library and Teaching commands. Library Stage 2 collections, archive state, reader metadata and provenance remained in the integration tree.

Migrations 0001–0024 were compared byte-for-byte against the integration base. Migration 0025 is present exactly once and matches the complete Teaching source. No historical SQL was rewritten.

## Review delivery

PR #8 was confirmed through its GitHub page as a draft from `feature/teaching-three-templates` into `integration/windows-library-teaching-study-review`. Once the integration branch contains that source merge, PR #8 is superseded: its head is already an ancestor of its base. Close it as superseded; it is not a review PR for the combined integration branch. Any replacement PR must target an explicitly approved compatible base. In particular, do not replace it with an unapproved PR into divergent `main`.

GitHub CLI GraphQL and REST reads returned `Forbidden` in this environment. Native Git fetch access worked through the existing platform proxy; no new token was requested.

## Runtime and evidence

Validation uses Node 24.19.0, npm 11.9.0 and Rust stable 1.99.1 (`58cf8f9c3`, 2026-10-13), with locked dependency installs. Native checks run inside an isolated Debian 13 container with the documented Tauri libraries, debug information disabled and three Cargo jobs. Only synthetic/temp test databases are used.

## Merge sequence and Study resolution

Teaching merge: `bc03fa4f2b4b172b1edaa277dcb0423414f626a5`, created only after focused validation passed. Its first parent is the integration/Library base and its second parent is the complete Teaching branch.

Phase 1 results: 211 focused frontend tests in 15 files; 36 source-attestation/localisation tests in 3 files; 103 focused Rust tests passed with one pre-existing ignored `presentations::write_examples` fixture generator. Typecheck, lint, six-language checks, generated corpus check, formatting and frontend build passed. The initial native launch raced the ongoing compiler package installation (`cc` missing); rerunning after installation completed passed. An initial source-attestation test was blocked by sandbox `spawnSync git EPERM`; rerunning with subprocess permission passed. Neither required a source change or test weakening.

Study conflicts and decisions:

| Path | Resolution |
| --- | --- |
| `src/features/teaching/TeachingView.tsx` | Preserve the complete newer block-based implementation: three built-ins, blank and personal templates, version-1 compatibility. The old Study-side format/point start screen was superseded. |
| `src/features/settings/TeachingTemplateEditor.tsx` | Retain block editing, duplication and both version paths; combine imports with Study's simpler three-format selector. Existing stored kinds remain intact until the user changes formats. |
| `tests/ui/baselines.test.tsx` | Preserve modern start-screen assertions plus compatible Study changes. Obsolete format-filtered point-template expectations do not describe the block start screen. |
| `tests/ui/teaching-formats.test.tsx` | Keep modern three-built-in/blank assertions; retain Study's simpler legacy format labels and preservation checks; assert no second Talk question. |
| `tests/ui/teaching-templates.test.tsx` | Keep version-2 block builder, personal template and duplication assertions, alongside legacy version-1 coverage. |
| `tests/ui/teaching.test.tsx` | Keep block starts and explicit legacy-session fixtures, with the compatible simplified legacy information-panel selector. |
| Both `docs/i18n/native-polish-*-v0.19.md` files | Regenerate from all six merged language packs, preserving Study flow/builder namespaces and modern Teaching built-in/block/view/guidance namespaces. |

All Study research/evidence commands and models merged without storage conflicts. Study answers continue to use `study_blocks`; Teaching continues to use `teaching_blocks`. `study_flow.rs`'s assertion was advanced from schema 24 to the actual combined schema 25; no SQL or migration history was changed. The new `combined_integration.rs` regression covers schema-25 startup with pre-existing structured answers and a block Teaching, new template availability, repeated startup without a new migration backup, and independent Study editing.

The merged localisation corpus contains 3,702 strings / 3,843 plural-form keys. Its 242 audit findings are review guidance, not automated validation failures or a native-speaker approval claim.

## Historical migration SHA-256 manifest

These are the actual integrated SQL bytes, checked against the integration base (0001–0024) and Teaching source (0025).

```text
a0631bbc74b12b9900074ce4aaeb3437a0bec8c1814ec0848bf3379de2bbdcc6  0001_initial_schema.sql
7aa670dfddfeb164c579d6a2d2dbda8c59cff26bf121aee614746d439c8ec9e7  0002_scripture.sql
edbf84bb0d15f379a0e46e836f4082f4f9627332cbdb6378ffee67006d909ac5  0003_studies.sql
6a69101c45e35ddfd9492728f282f2cae7bd0ad3e7851523ac8fc37e03c34f49  0004_library.sql
4c2988058e5db2729a8ccd71abf4cbc750ca6805283e7e46f8f70db435e33819  0005_passage_sections.sql
d037f4a9f1afc755541a4245f439d649e4b979a3f70ab679e9a18c2ce21bcb87  0006_source_annotations.sql
2e643740eb7662b5af982e178d3d3af672ce0c93659a9ff766278219b4d1fede  0007_finding_source_quotes.sql
24358f2726358ab84c2be6edb381962d44d664686e212fb68ad3b5fb0cc19181  0008_teaching_preparations.sql
30b51c5269c4df0c041e54fdb7697ec48f99fad5723b1c3e907fcfad93cff26f  0009_teaching_step_kinds.sql
75a1d3100aed2daa2c61db1f0632c780a1b1dccdacae9b48f0cf77f4b7092c3e  0010_presentations.sql
cad7c6dbaa04546d6bb359948912511afc08679f6e196ee631162a616df0920e  0011_presentation_target.sql
4897e69f1772ecc4739d8b48b68cc67339c2fbf891d5bc7431ac503bd98ad600  0012_presentation_templates.sql
b21b8d7376bcefffd7af7c1612faae3e75b380f551d71be4a469646ed92d5752  0013_study_phases.sql
c2bb3d53a9534dc935bf92685a5ce89f1138955c108c6dd7d930c8b4a28a7f2f  0014_teaching_templates.sql
6fe4b7bf54c5e7bc24cf3dc5d162618868abe9ec65dbb381f735db503a1ebf5c  0015_verse_bridges.sql
9d8fc7cdbec5c5c7ce1b7c05e412d6396695d106d05e8e36f61cf4cf7424846a  0016_unresolved_citations.sql
0328b622162d1849e110c0e1d872ef678bb97dc41fd7b4fc68dfd317ef408f20  0017_word_annotations.sql
360fa1b64e04d6577414af653b2a75ae5e3e5e2167e37ac73341592734f2a0e2  0018_grammar_provenance.sql
9c5f902a99211c4c02e6789908639eb75542bbeab2a379092247728bcafe13d4  0019_bible_documents.sql
3c3b94a6448ffb8449a90161126cb8ea838cd5d53948ea62dabb656655dcc71c  0020_library_organisation.sql
58408192a7f7a8ee5570dff1b22c3a3227516cd1feb7bfe3f6f1d0d29d518cf1  0021_bible_passage_locations.sql
b77b53f8186ba795f8acdec482eac93250edf90e2ad140dd073fe175d3220521  0022_import_jobs.sql
beb1121b087d028b657c464ff65a351475cc307fe2b08430112ed608790c334a  0023_recognition_corrections.sql
7b8c3b31856ae40d2b6ad5c764515066066a20c0dc4dfac20ba5e62384a53252  0024_section_status_and_fields.sql
68fd35adfff4af4d57ad5687249fd26442a72f470d6dbec54bb10caca6d29e36  0025_teaching_blocks.sql
```

## Failures investigated during combined validation

- The first full frontend run executed 1,124 tests: 1,122 passed and two terminology regressions failed. New Teaching errors contained formal German `Wählen Sie` and four Korean uses of `블록`. These were corrected to informal German and the established Korean terminology. Assertions were retained. The affected German, Korean, corpus and Teaching-format checks then passed all 43 tests, and six-language parity passed again. Derived review documents were regenerated after the text repairs.
- The first full native run stopped at `passage_languages::indexing_a_moderately_large_source_in_each_language_is_quick`. Its French stage/commit measurement was 5.126 + 7.292 seconds against an unchanged ten-second limit, while frontend tests and compilation shared the four-CPU quota. The failed attempt is retained separately in the local logs. The complete native suite was rerun without competing heavy work, with networking disabled, and passed. No test or performance assertion was removed or relaxed.

## Combined frontend and tooling results

| Check | Result |
| --- | --- |
| `npm run typecheck` | Passed. |
| `npm run lint` | Passed. |
| `npm run i18n:check` after terminology repairs | Passed: six languages × nine areas, 3,703 English keys, 126 components. |
| `node scripts/i18n-corpus.mjs --check` | Passed after regeneration. |
| `npm test -- --maxWorkers=3` on corrected integrated source | 99 files, 1,124 tests passed; no failures or skips. |
| `npm run build` after terminology repairs | Passed; >500 kB chunk-size warning remains. |
| `cargo fmt -- --check` through native wrapper | Passed. |
| `cargo clippy --all-targets --locked -j 1 -- -D warnings` through native wrapper | Passed; warnings denied. |
| Development HTTP smoke | Application HTML contains the Abiding Word title and React entry; transformed `/src/main.tsx` serves `createRoot`. The smoke-test server was stopped afterward. |
| Install-script repeatability | `/workspace/.cloud-setup/install.sh` reran successfully with locked npm/Cargo dependencies and Rust 1.99.1. The native helper was verified in a fresh disposable container. |

All 10 original Study template JSON files were independently compared byte-for-byte with `b3120f5` and remain unchanged. Six new Study definitions are additive. The corrected Teaching materials, presentation and native block implementation remain unchanged by the Study merge.

## Coverage map for owner review

| Requirement | Existing and added coverage exercised |
| --- | --- |
| Fresh database and schema 19/24/25 startup | `database`, `migration_v024`, `migration_v25`, `windows_dev44_compatibility`, added `combined_integration`. |
| Historical SQL, divergent-history refusal, repeated startup | Independent 25-file manifest above; native historical migration guards and synthetic restart/backup assertions. |
| Backup/restore and project portability | `portability`, `study_flow`, `teaching_templates`, Teaching role/source remapping regressions and legacy migration tests. |
| Legacy and new Study templates | POZETEK, Ten-Step, catalogue, writing-template, reusable-block and research-review suites; original template bytes preserved. |
| Legacy and new Teaching templates | Teaching, Teaching blocks/templates, personal version-1/version-2 definitions, all three built-ins, optional blank start, Save/Duplicate and guidance UI tests. |
| Participant privacy and outputs | Teaching output-integrity, material Scripture/DOCX fixtures, manuscript heading assertions, native presentation/private-block checks. |
| Library collections, reader, search and citations | Library management/import/search, reader/UI, source navigation, passage-language and citation suites; Library Stage 2 metadata remains in the source. |
| Exact Scripture and edition provenance | Atomic evidence and failed-save rollback, explicit edition mismatch, partial-passage exact verse citations, stored source snapshots and native Scripture/material tests. |
| Six languages | Parity, language closure, German address, Korean terminology, fallback/corpus tests; review findings remain advisory. |
| Offline use | Complete native suite is run through the disposable-container helper with `AW_NATIVE_NETWORK=none` and Cargo `--offline --locked`; frontend tests use synthetic local APIs. |

No real user database was opened or migrated. Test data is synthetic/temp, with repository-bundled/licensed Scripture fixtures where the existing tests require them. No changes were made to `src-tauri/src/scripture`, `scripts/scripture` or `resources/scripture` relative to the integration base; the separate Bible recognition workstream is untouched.

## Final native result and merge commits

The complete command was:

```sh
AW_NATIVE_NETWORK=none /workspace/.cloud-setup/native test --offline --locked -- --show-output
```

It exited successfully: **1,021 passed, zero failed, eight pre-existing ignored checks**, across 82 reported targets including zero-test binary/doc targets. The new schema-25 coexistence regression is included in this complete run. All bundled Bible imports and exact historical text comparisons completed successfully offline.

The Library timing regression passed unchanged in all six languages. Stage plus commit totals were EN 3.19 s, DE 4.87 s, KO 3.63 s, FR 7.92 s, ES 8.06 s and PT 6.08 s, each below the existing ten-second limit. Together with the unchanged source and isolated rerun, this supports resource contention as the cause of the initial timing failure.

Pre-existing ignored checks, not counted as passes:

- `document_units_dump`
- `write_examples`
- `real_document_editions_report`
- `bundled_number_index_cost`
- `oshb_corpus_reference_correctness`
- `real_editions_report`
- `write_interface_fixture`
- `a_whole_generated_annotated_bible_is_measured`

These are optional dump/report/fixture/corpus/full-size measurement checks; no ignore was added by this integration.

| Merge | Exact commit | Parents |
| --- | --- | --- |
| Teaching into Windows/Library | `bc03fa4f2b4b172b1edaa277dcb0423414f626a5` | `b3120f54b0dcece6c078727c97240828680dd63c`, `e275a1d6a166eb2471a9bb23592a98767569036f` |
| Study into validated Teaching integration | `17b5220d7a5d4911b0180f3e8bcba538d2e04f79` | `bc03fa4f2b4b172b1edaa277dcb0423414f626a5`, `981f4c45bb8650ae5f11fe35112089485f697496` |

Application validation applies to the source in the second merge above. This handover is delivered in a subsequent documentation-only commit; the final pushed branch-tip SHA is reported in the task delivery message.

## Environment and remaining boundaries

Reusable `install_script` and `start_skill` fields were saved successfully in the environment configuration draft. Helpers, the native image/rebuild Dockerfile, cached dependencies and raw validation logs are under `/workspace/.cloud-setup` (or the local Docker store), outside tracked application source. Saving the draft did not publish the environment snapshot or prove restoration in a fresh task. Review/save the configuration in environment settings, then publish the environment snapshot when approved.

GitHub API access was diagnosed as a proxy CONNECT rejection (HTTP 403) for `api.github.com`, before destination authentication. The running allowlist did not include that host. Its domain addition was saved in the draft while preserving existing custom domains and package-manager presets; it is not applied merely by saving the draft. No credential value was requested or exposed, and new API authentication has not been validated. Normal Git HTTPS fetch access was verified separately.

No application/test failure remains after the documented corrections and rerun. The bundle-size warning remains. Real Windows visual acceptance, a live Tauri desktop GUI run, installer generation/publication, real external assistant providers and environment restoration in a fresh task were not performed or claimed. No main merge, deployment, paid Actions dispatch, Teaching Stage 3 or separate recognition work was performed. Stop here for owner review.

The first push was rejected by GitHub GH007 because the environment configured a private commit email. Only the two newly created, unpushed merge commits were recreated using the existing repository owner’s GitHub noreply address. Their tree IDs were verified identical; all source-branch parents remain exact. The remote branch was unchanged by the rejection, and subsequent delivery uses an ordinary fast-forward push. No account privacy setting or remote history was changed.

## Pushed delivery and PR disposition

The application integration was pushed successfully as `17b5220d7a5d4911b0180f3e8bcba538d2e04f79` to `integration/windows-library-teaching-study-review`, and `git ls-remote` independently confirmed that exact remote SHA. The next commit contains this handover only; its final pushed SHA is provided in the delivery message.

After that push, PR #8's GitHub page still reported **DRAFT**. Its Teaching head `e275a1d6a166eb2471a9bb23592a98767569036f` is now an ancestor of its integration base. **Close #8 as superseded and replace it for combined integration review**, using the integration branch as the new head and an owner-approved compatible review base. Do not target divergent `main` without a separate explicit decision. No replacement PR was created and no PR metadata edit is claimed: the API route remains blocked by the running network policy. Apply the saved `api.github.com` domain addition in environment settings before API authentication/edit access can be tested; no new secret is established as necessary.

The code is ready for owner review on the pushed integration branch. Windows acceptance and any further merge/publication remain separate owner decisions.
