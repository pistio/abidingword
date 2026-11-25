# Schema 26 independent validation and Windows readiness

## Baseline and scope

- Repository: `pistio/abidingword`.
- Existing review branch: `review/schema26-recognition-handover`.
- Exact starting review and integration baseline: **`5f4edc0f8906df9bc1db9437f4cb4a30561eff22`** (PR #12 merge). Remote tips were independently verified before checkout. The local tracking checkout uses that existing remote branch; no replacement remote branch was created.
- Target for review only: `integration/windows-library-stage3-teaching-study`. No merge to that branch or divergent `main` is performed.
- Final validated implementation SHA: **`a4980a22f8ed4ae2efc3c16f8928f406c8e4f6c1`**. The subsequent delivery commit changes this handover only; its exact pushed SHA is reported with delivery and resolves as `git rev-parse origin/review/schema26-recognition-handover`.

The required `CLAUDE.md`, Teaching Stage 3 handover, Library Stage 3 combined handover, Windows/Library/Teaching/Study integration handover, Study-flow redesign and Windows v0.24 acceptance guide were read. Their earlier holds, schema-22/25 descriptions and publication instructions are historical. This task authorises schema-26 review and non-recognition repairs, not any publication or recognition development.

Git ancestry verifies Library Stage 2 (`b3120f54b0dcece6c078727c97240828680dd63c`), Library Stage 3 (`67c72e1b7caa148ec24c31c6f0b54fe385851a37`), Teaching correctness repairs (`314db4d16b35e079f2c894ab93cef949a544fd8e`), complete simplified Study (`981f4c45bb8650ae5f11fe35112089485f697496`), and standalone Teaching (`795d482222ad014d1140835acc27f7bfabac382f`) are all ancestors of the exact baseline. The current migration registry contains historical versions 1–26.

## Confirmed defect and correction

**Library archived → Trash → Undo incorrectly changed the source to active.** This also meant an archived source could unexpectedly reappear in the active Sources pane. The reproduced UI/API regression passed for an initially active source and failed for an archived source (`expected archived`, `received active`). The baseline Library Undo handler always requested `active`.

`LibraryView` now captures the displayed source's previous state before the Trash operation and restores that state on Undo. No new control, API, localisation string or database migration is introduced. Regression cases exercise both original states through the public API and verify collection names, tags, content hash and segment count are retained. The new cases and existing Library organisation/navigation tests passed **16/16**; the complete frontend run passed **1,145/1,145**.

No database preservation defect was reproduced. No recognition algorithm, recognition-specific test, corpus, evaluation tool, development branch or source fixture is changed. The only production-code change is the Library Undo state restoration.

## Database integrity evidence

All data used here is invented in temporary databases. No installed user database was opened, copied, restored or migrated.

The added `src-tauri/tests/schema26_review.rs` independently exercises **on-disk** upgrades from 19, 24 and 25 using a Unicode/space-containing path. It captures original columns and values for Studies, sections, structured answers/data, findings/citations, personal templates, legacy Teaching, block Teaching, points, blocks, passages, materials, presentations and presentation items. The fixture includes a deleted preparation and deleted block, non-default revisions and speaker-only writing. Every original value is compared after actual `Database::open` startup, not just after SQL execution. Original ledger names/timestamps remain identical. Three startups per source version keep exactly one pre-migration backup, whose original schema/data are checked separately.

Its second case archives a Study, unlinks/relinks Teaching, soft-deletes/restores/permanently removes the Study, restores trashed Teaching, soft-deletes/restores the project and finally removes the owning project permanently. All eight Teaching-table ID sets survive until the intentional permanent project removal. Saved quotations remain available after the Study is gone. FK enforcement, `foreign_key_check` and `integrity_check` are checked throughout.

Existing `migration_v26` tests also populate the complete dependency graph and inject SQL failure, valid-but-destructive child deletion, a dangling FK, and unresolved ownership. They verify schema/data/history rollback and FK restoration. The v26 runner was inspected: FK actions are disabled outside the transaction, the rebuild runs exclusively, original values are compared, and FK checks precede the history row/commit. It restores and verifies enforcement after success/error. No ineffective in-transaction FK toggle, subquery CHECK, migration renumbering or ledger fabrication was added.

The focused native command selected `schema26_review`, `migration_v26`, `teaching_standalone`, `portability`, `windows_dev44_compatibility`, `migration_v25`, `study_flow` and `combined_integration`: **38 passed, zero failed/ignored**. This includes actual fresh schema 26, synthetic 19/24/25 upgrades, rollback, repeat startup, divergent-history refusal, old/new Study and Teaching compatibility, full backup restore, standalone/saved-provenance project transfer and malformed-package protection.

### Immutable migration manifest

All 26 SQL blobs were compared byte-for-byte with the exact starting commit. No registry or migration SQL is changed. SHA-256:

| File | SHA-256 |
| --- | --- |
| `0001_initial_schema.sql` | `a0631bbc74b12b9900074ce4aaeb3437a0bec8c1814ec0848bf3379de2bbdcc6` |
| `0002_scripture.sql` | `7aa670dfddfeb164c579d6a2d2dbda8c59cff26bf121aee614746d439c8ec9e7` |
| `0003_studies.sql` | `edbf84bb0d15f379a0e46e836f4082f4f9627332cbdb6378ffee67006d909ac5` |
| `0004_library.sql` | `6a69101c45e35ddfd9492728f282f2cae7bd0ad3e7851523ac8fc37e03c34f49` |
| `0005_passage_sections.sql` | `4c2988058e5db2729a8ccd71abf4cbc750ca6805283e7e46f8f70db435e33819` |
| `0006_source_annotations.sql` | `d037f4a9f1afc755541a4245f439d649e4b979a3f70ab679e9a18c2ce21bcb87` |
| `0007_finding_source_quotes.sql` | `2e643740eb7662b5af982e178d3d3af672ce0c93659a9ff766278219b4d1fede` |
| `0008_teaching_preparations.sql` | `24358f2726358ab84c2be6edb381962d44d664686e212fb68ad3b5fb0cc19181` |
| `0009_teaching_step_kinds.sql` | `30b51c5269c4df0c041e54fdb7697ec48f99fad5723b1c3e907fcfad93cff26f` |
| `0010_presentations.sql` | `75a1d3100aed2daa2c61db1f0632c780a1b1dccdacae9b48f0cf77f4b7092c3e` |
| `0011_presentation_target.sql` | `cad7c6dbaa04546d6bb359948912511afc08679f6e196ee631162a616df0920e` |
| `0012_presentation_templates.sql` | `4897e69f1772ecc4739d8b48b68cc67339c2fbf891d5bc7431ac503bd98ad600` |
| `0013_study_phases.sql` | `b21b8d7376bcefffd7af7c1612faae3e75b380f551d71be4a469646ed92d5752` |
| `0014_teaching_templates.sql` | `c2bb3d53a9534dc935bf92685a5ce89f1138955c108c6dd7d930c8b4a28a7f2f` |
| `0015_verse_bridges.sql` | `6fe4b7bf54c5e7bc24cf3dc5d162618868abe9ec65dbb381f735db503a1ebf5c` |
| `0016_unresolved_citations.sql` | `9d8fc7cdbec5c5c7ce1b7c05e412d6396695d106d05e8e36f61cf4cf7424846a` |
| `0017_word_annotations.sql` | `0328b622162d1849e110c0e1d872ef678bb97dc41fd7b4fc68dfd317ef408f20` |
| `0018_grammar_provenance.sql` | `360fa1b64e04d6577414af653b2a75ae5e3e5e2167e37ac73341592734f2a0e2` |
| `0019_bible_documents.sql` | `9c5f902a99211c4c02e6789908639eb75542bbeab2a379092247728bcafe13d4` |
| `0020_library_organisation.sql` | `3c3b94a6448ffb8449a90161126cb8ea838cd5d53948ea62dabb656655dcc71c` |
| `0021_bible_passage_locations.sql` | `58408192a7f7a8ee5570dff1b22c3a3227516cd1feb7bfe3f6f1d0d29d518cf1` |
| `0022_import_jobs.sql` | `b77b53f8186ba795f8acdec482eac93250edf90e2ad140dd073fe175d3220521` |
| `0023_recognition_corrections.sql` | `beb1121b087d028b657c464ff65a351475cc307fe2b08430112ed608790c334a` |
| `0024_section_status_and_fields.sql` | `7b8c3b31856ae40d2b6ad5c764515066066a20c0dc4dfac20ba5e62384a53252` |
| `0025_teaching_blocks.sql` | `68fd35adfff4af4d57ad5687249fd26442a72f470d6dbec54bb10caca6d29e36` |
| `0026_project_teaching.sql` | `cbcef5c4d83a571230b6001d3ebc6d9f4060709f1aae148bc2ea4456598ef95c` |

## Workflow and offline evidence

The full React suite uses the application shell and synthetic local APIs; native suites use actual SQLite/filesystem/reader/export services in disposable data directories. This is layered application evidence, not a claim that a real Windows Tauri GUI or native dialog was exercised in Linux.

| Journey | Exercised coverage and expected behaviour |
| --- | --- |
| Study methods | Current catalogue/writing-template, POZETEK, Ten-Step, structured-legacy and personal-template suites; seven/ten writing steps, collapsed optional guidance, continuous writing and preserved keyed answers. |
| Research | Reusable fields, exact returned Scripture/edition identity, incomplete ranges, atomic retained-evidence create/rollback/retry, explicit edition mismatch and unavailable-edition non-substitution. Text matches remain evidence, not inferred theological connections. |
| Teaching | Standalone/project and linked Study entry; optional same-project link/unlink/relink, multiple sessions, three built-ins, blank and personal v1/v2 templates; the same stored Teaching remains available after Study deletion. Study restore does not silently relink it. |
| Outputs/privacy | Manuscript headings, worksheet/material derivation, DOCX, live/saved presentation sources and participant-versus-leader visibility; private leader writing and private Scripture remain excluded from participant/audience content. |
| Library | Nested collections/direct membership, tags, title/author/content searches, archive/missing-source behaviour, citation words/anchors, PDF/EPUB/DOCX transitions, Sources/Scripture/Study transfers and stale-result protection. Reading/navigation return state is session-only as documented by Library Stage 3; cross-restart persistence is not claimed. |
| Portability | Full backup/restore, migration safety copies and project packages; optional Study IDs and Teaching descendants remap, standalone sessions transfer and saved citations remain explicit when sources are unavailable. |
| Offline | Native execution uses Docker network `none` and Cargo offline/locked. UI tests cover installed local reading/editing, autosave, reader navigation and exports, plus unavailable updater/assistant/research service responses. Real external paid providers are not contacted or accepted. |

## Complete check results

Fresh checks ran sequentially on the final implementation using Node 24.19.0, npm 11.9.0 and Rust 1.99.1. Native commands used the Debian 13 dependency container, network `none`, locked/offline dependencies and three build jobs. Raw logs and machine-readable `checks.json` are retained under `/workspace/.cloud-setup/schema26-review/`; previous task logs are not counted.

| Check | Result | Seconds |
| --- | --- | ---: |
| `npm run typecheck` | PASS | 79.04 |
| `npm run lint` | PASS | 77.34 |
| `npm run i18n:check` | PASS: six languages × nine areas, 3,721 English keys, 126 components | 1.90 |
| `node scripts/i18n-corpus.mjs --check` | PASS | 0.80 |
| `npm test -- --maxWorkers=2` | **1,145 passed, zero failed; 102 files** | 510.22 |
| `npm run build` | PASS; existing large-chunk advisory remains | 84.22 |
| `cargo fmt -- --check` | PASS | 35.29 |
| `cargo clippy --offline --locked --all-targets -- -D warnings` | PASS | 26.18 |
| `cargo test --offline --locked --no-fail-fast -- --test-threads=1 --show-output` | **1,034 passed, zero failed, eight ignored** | 682.78 |
| `cargo build --offline --locked` | PASS, development binary | 34.45 |

Rust counts sum all 85 result summaries, including doc-tests. The eight pre-existing ignored tests remain ignored: `document_units_dump`, `write_examples`, `real_document_editions_report`, `bundled_number_index_cost`, `oshb_corpus_reference_correctness`, `real_editions_report`, `write_interface_fixture`, and `a_whole_generated_annotated_bible_is_measured`. No test was disabled, weakened or newly ignored. They are not counted as passes. The original archived-Undo reproduction failed before correction; all final full-suite checks passed after correction.

## Library timing investigation

The existing `passage_languages::indexing_a_moderately_large_source_in_each_language_is_quick` test and its **10-second** stage-plus-commit threshold are unchanged. Each language must also retain 5,000 segments and 10,000 references; correctness assertions are never relaxed. Heavy jobs are serialized, and native tests run with one test thread. The managed CPU quota is four cores (`cpu.max = 400000 100000`). CPU accounting is retained before/after the complete native run; aggregate throttling cannot by itself prove the cause of a particular case's latency.

The review changes no native production code or Library indexing implementation. The unchanged timing test **passed in this complete Rust run**, with the following stage-plus-commit measurements (seconds):

| Language | Stage | Commit | Total |
| --- | ---: | ---: | ---: |
| English | 2.174703 | 0.710699 | 2.885403 |
| German | 1.765547 | 2.997909 | 4.763456 |
| Korean | 1.196179 | 2.314290 | 3.510469 |
| French | 3.546028 | 4.567031 | 8.113059 |
| Spanish | 3.718910 | 4.350156 | 8.069066 |
| Portuguese | 2.734672 | 4.088655 | 6.823327 |

No isolated rerun was needed to replace a failed current result. This run does not reproduce an indexing regression; limited headroom in French/Spanish still warrants measurements on the acceptance Windows machine. Resource contention is plausible for historical intermittency but is not established as its cause. The historical Teaching handover's timing failures/successes are context, not fresh evidence or proof of environmental causation.

## Recognition boundary and Claude handoff

Claude Code retains exclusive responsibility for multilingual Bible PDF/EPUB recognition. Existing application regression tests may run unchanged here. This review does not alter verse boundaries, headings, footnotes, verification rules, recognition fixtures/corpora/evaluation tools or Claude's development branch.

No new recognition-related regression was reproduced by the existing complete application suite. No new recognition defect is handed off from this run; ignored reports and unexecuted external-corpus acceptance remain outstanding for Claude. Any later recognition defect must be recorded with exact fixture/command and expected/actual results for Claude, without changing recognition logic or weakening tests here. Passing existing regressions does **not** establish final Bible recognition acceptance. The separate recognition project remains an **unfinished dependency**, with its own corpus/evaluation and owner/Claude acceptance still required.

## Windows-specific readiness and update compatibility

Linux success does not establish Windows compatibility. Remaining checks include WebView2/native rendering, physical second-screen presentation, native open/save/folder dialogs, UTF-16/Unicode and space-containing paths, NTFS ACL failures, locked files/antivirus interference, rollback/WAL handling and realistic data-volume memory/latency.

`resolve_data_root` uses nonempty `ABIDING_WORD_DATA_DIR` before the OS per-user data directory. The existing `test` updater configuration still names `jc-universe87/abidingword-updates`; its public-key SHA-256 is `1d6c49a406a5b4597d4866c37f83b311606b65f2373192f8db0ed5561cdd23c7`. Neither that routing, signing key, updater logic nor release workflows were changed. Before any future approved installer, its owner must verify the intended repository/channel routing, signed manifest, monotonically newer development version and exact source/schema provenance. No live update/install was attempted here. Normal development builds gate out the updater; unavailable-service behaviour is checked by the existing tests.

Schema 26 is not safe to open for editing in schema-19/24/25 binaries. Rollback means quitting all app processes and restoring a verified complete pre-upgrade folder/backup; never delete migration-ledger rows or rewrite historical SQL to force a downgrade. The native rollback tests are automated evidence; Windows locks and real installer restart/rollback still require acceptance.

## Reproducible Windows acceptance checklist

Use a clean Windows VM or dedicated disposable account with **no real user library/studies**. Check out the exact final reviewed source, use Node 24 (or compatible >=20.19), Rust 1.99.1/compatible MSVC tooling and WebView2. Prepare locked dependencies before disconnecting; do not invoke release/publisher workflows.

```powershell
$reviewRoot = Join-Path $env:TEMP ("AW-schema26-review-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $reviewRoot | Out-Null
$env:ABIDING_WORD_DATA_DIR = Join-Path $reviewRoot "Data with spaces - 한글"
if ([string]::IsNullOrWhiteSpace($env:ABIDING_WORD_DATA_DIR)) { throw "Missing disposable data directory" }
$env:CARGO_PROFILE_DEV_DEBUG = "0"
$env:CARGO_PROFILE_TEST_DEBUG = "0"
npm ci
cargo +1.99.1 test --manifest-path src-tauri/Cargo.toml --locked --test schema26_review --test migration_v26 --test windows_dev44_compatibility --test migration_v25 --test portability -- --test-threads=1
npm run tauri dev
```

The targeted Rust command generates its own synthetic historical databases and checks on-disk upgrades/backups/restarts on Windows. It does not require an old user database. Confirm Settings shows the disposable data location before entering any data. Once dependencies are prepared, repeat ordinary GUI journeys with external networking disconnected (localhost dev-server access remains available).

- [ ] Record exact git SHA, Rust/Node versions, Windows/WebView2 version, app-reported schema 26 and actual data path. Run the complete checks and preserve all failed/ignored results and timing measurements.
- [ ] Fresh start and restart; run the synthetic 19/24/25 upgrade/rollback matrix above. Confirm backups contain the old ledger/data, repeated startup makes no extra migration backup and no Teaching descendant/tombstone disappears.
- [ ] Create every method offered by New Study; explicitly count seven POZETEK and ten writing steps. Toggle guidance/continuous view, type/reopen, and inspect legacy structured/personal studies without converting answers.
- [ ] Exercise each reusable research field against a local licensed structured edition. Keep evidence, trigger a failed save/retry, verify returned canonical verses/query/edition/text remain exact, and confirm missing/wrong editions do not silently substitute.
- [ ] From project Teaching create all three built-ins plus blank/personal sessions; reopen a legacy version-1 session/template. Link/unlink/relink within the project, create two sessions from one Study and reject a different-project Study.
- [ ] Soft-delete/restore/archive and permanently delete a synthetic Study through supported lifecycle operations. The same Teaching, writing and cited material must remain in project Teaching; restoring a Study must not relink it automatically. Restore trashed Teaching and soft-delete/restore a project; only explicit permanent project removal removes its Teaching graph.
- [ ] Use distinct public/private Scripture and a `LEADER ONLY` sentinel in private writing. Generate manuscript, worksheets, DOCX and slides; inspect actual files and a second display. Neither private Scripture references/words nor leader-only text may enter participant/audience output.
- [ ] Browse/filter collections/tags; search title/author/body, open/reopen cited words, move between all reader modes/panes and return to the same in-session shelf/filter/page/position. Verify archived → Trash → Undo remains archived and active → Trash → Undo remains active. Keep marks, collections, tags, original files and citations.
- [ ] Remove a synthetic source or its managed file: retained quotes/provenance stay visible with explicit unavailable status. Exercise PDF Reading/Original, EPUB and DOCX; imported-Bible UI/edition issues may be recorded, but recognition quality is for Claude's separate acceptance.
- [ ] Export/import standalone and linked Teaching projects into a second disposable profile; verify all relationships/outputs. Back up everything, change/delete synthetic work, restore and compare. Test Unicode/space paths, cancelled dialogs, read-only destinations, locked-file failure and complete-folder rollback with the app closed.
- [ ] Repeat key reading, Study/Teaching/autosave and export paths offline. Online-service failures must not stop local work. Check six languages, keyboard focus, high-DPI/narrow layout and native dialogs. Do not install an offered update or publish a channel during acceptance.

## Installer recommendation and delivery

The integrated schema-26 source is ready for controlled Windows acceptance and owner review. All available automated checks passed, migrations are unchanged and the one reproduced non-recognition defect is corrected. This is **not approval to publish the next development installer**: complete the real Windows checklist and obtain the separate recognition/owner decision first. Windows acceptance and Claude's recognition acceptance remain separate gates. This task never authorises publication, automatic updates, deployment, a main merge or paid Actions/services.

### Delivery record

Implementation/test commit `a4980a22f8ed4ae2efc3c16f8928f406c8e4f6c1` was committed and pushed to the existing review branch. Clean-tree build provenance verified that exact source, schema 26 and 26 migrations. Immediately before delivery, the integration target was still `5f4edc0f8906df9bc1db9437f4cb4a30561eff22`. No integration/main branch was pushed or merged. Commit messages include `[skip ci]`; no Actions run was requested.

Creating a draft PR using `gh pr create --repo pistio/abidingword --head review/schema26-recognition-handover --base integration/windows-library-stage3-teaching-study --draft` failed with `Post "https://api.github.com/graphql": Forbidden`. The effective cloud network allowlist omits `api.github.com`; Git HTTPS push works. No PR number or created-PR URL can therefore be claimed. The prepared body is retained at `/workspace/.cloud-setup/schema26-review/pr-body.md`.

Owner can create the draft manually from [this comparison](https://github.com/pistio/abidingword/compare/integration/windows-library-stage3-teaching-study...review/schema26-recognition-handover?expand=1), targeting the integration branch explicitly. This is a comparison/creation link, not an existing PR. Stop for owner review; the API/network limitation is the remaining delivery blocker, alongside the explicitly separate Windows and recognition acceptance gates.
