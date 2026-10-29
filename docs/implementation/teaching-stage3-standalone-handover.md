# Teaching Stage 3 — Independent Teaching Workspace

## Review and lineage

Owner review required. No merge, release, installer publication, deployment, paid Actions, or real user databases are authorised by this handover.

- Repository: `pistio/abidingword`.
- Implementation branch: `feature/teaching-stage3-standalone`.
- Base / initial branch and target tip: `2cb7cb9483d305f10a3e8ad18ec5c6573c439722`.
- PR target: `integration/windows-library-stage3-teaching-study`; never `main`.
- This is an implementation on the validated combined lineage, with no source-branch merges or merge conflicts.
- Reviewed `CLAUDE.md`, Teaching Stage 2 and combined/Library Stage 3 handovers, and the independent review from `4b2552e250eff1e8ad65e948cec438e9db998056:docs/implementation/codex-integration-review-2026-10-28.md`. Its parent-drop cascade finding informed the migration protocol. The present user request authorises this Stage 3 implementation.

## Behaviour and design

Teaching belongs to a project; its Study relationship is optional. Project Teaching lists every live session, permits standalone creation, offers an optional same-project Study selector, and returns to the list. The Study entry remains available. Both routes use the same Teaching editor, template definitions, materials and presentation pipeline. The three built-ins remain modified EVA, Sermon / Talk, and Bible Lesson — Children / Youth. Blank and personal templates, including version 1, remain supported.

The Rust DTO returns `projectId` and nullable `studyId`. Creation accepts a project with omitted/null Study, and legacy Study-only input derives its project from that Study. The Rust creation struct retains its legacy String study field for source compatibility; a nullable JSON Study is normalised to the empty/absent value. Commands and the TypeScript API expose project listing, linking/unlinking, and Teaching restore. Lifecycle commands add Study/project soft delete, permanent delete and restore. They are granted only to the main window, not the audience window.

Cross-project links are rejected by the API and SQL triggers. A Study can have multiple teachings. Unchanged links are no-ops. Link changes update Teaching timestamps/revisions and keep its points, blocks, passages, materials and presentations. Soft or permanent Study deletion first detaches every linked preparation, including tombstones, in the same transaction. Restoring a Study does not silently relink Teaching. Archiving a Study retains its existing links. Soft project deletion hides its Teaching; project restore makes it available again. Permanent project deletion cascades through both live and trashed Teaching.

While linked, a Finding remains live. Unlinking/relinking or Study deletion captures the current Finding and raw citation provenance into a versioned saved-note envelope before clearing the live Finding FK. This preserves exact text, canonical Scripture references, quotation, title, author, locator, section, anchor, original source ID and content hash. A finding used only by a presentation is saved for that presentation without adding it to the editor's ordinary note list or assistant context. Existing slide source IDs continue resolving against saved notes. Citation reopening uses the existing exact citation locator. Missing/removed sources remain explicit; hash reconnection requires one exact match and refuses ambiguity. Deleted citations are not resurrected. A deleted live Finding stays deleted during unlinking and transfer; its cached provenance is not promoted into a visible note, and its link remains a tombstone.

Project packages select Teaching by project ownership, map optional Study links, preserve dependent IDs through the existing import map, and map saved Finding IDs used by presentations. Detached source references are included in package previews. Snapshot provenance records original Finding/Study IDs when current IDs are remapped. Saved citations receive separate locator IDs and retain their original citation IDs explicitly, so later edits to a still-live Study citation cannot redirect a saved copy. Legacy packages lacking Teaching project ownership are backfilled from the package's single mapped project; schema-26 Teaching rows missing the ownership field are rejected. Other malformed references retain the existing import refusal/left-out reporting policy. Backups include schema 26 and saved notes. Participant privacy remains enforced by the existing block visibility, materials and audience pipeline.

All six localisation packs include workspace, optional Study and saved-note wording. Derived native-language review documents were regenerated. Study and Teaching block storage remain separate. No Bible recognition logic was changed.

## Migration 0026 and preservation

The actual registry ended at 0025 at the starting tip. `0026_project_teaching.sql` is registered once as `project_teaching`. Historical SQL 0001–0025 was compared byte-for-byte with the exact base commit.

The dedicated runner requires autocommit, disables FK actions **before** BEGIN, verifies enforcement is off, and uses an exclusive transaction. It rebuilds Teaching ownership and nullable Study/Finding FKs, copying every old column and row, including tombstones. Before commit it compares original columns for all eight Teaching/dependent tables, seeds saved Finding provenance, checks `PRAGMA foreign_key_check`, and records the migration. Errors drop the transaction; FK enforcement is restored and verified outside it on success and failure. Unresolved project ownership fails rather than being guessed. Startup's existing verified pre-migration backup remains in place.

The populated synthetic gate passed before downstream implementation began. It includes a live and trashed preparation, every dependent table, non-default revisions, exact IDs and multilingual text. Fault injection covers an SQL error, an otherwise-valid child deletion detected by preservation comparison, an introduced dangling FK, and missing ownership. The tests compare schema/data/history and FK enforcement after rollback. No real user database is used.

## Migration SHA-256

| SQL file | SHA-256 |
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

## Validation

Validated on 2026-10-29 with Node **24.19.0**, npm **11.9.0**, and Rust **1.99.1 (58cf8f9c3, 2026-10-13)**. Native checks used the prepared Debian 13 container, `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`, and `CARGO_BUILD_JOBS=3`. All native tests ran with Docker network **none**, Cargo offline/locked, one test thread, and disposable data. Heavy validation steps were run sequentially for the final results.

| Check | Final result |
| --- | --- |
| `npm run typecheck` | Passed |
| `npm run lint` | Passed |
| `npm run i18n:check` | Passed: six languages, nine areas, 3,721 English keys, 126 components |
| `npm test -- --maxWorkers=1` | **1,143 passed, 101 files passed** |
| `npm run build` | Passed; existing large-chunk warning remains (no limit or bundle setting changed) |
| `cargo fmt -- --check` | Passed |
| `cargo clippy --offline --locked --all-targets -- -D warnings` | Passed |
| `cargo test --offline --locked --no-fail-fast -- --test-threads=1` | **1,031 passed, 1 timing failure, 8 pre-existing ignored tests**, across 84 result summaries including unit/integration/doc harnesses |
| `cargo build --offline --locked` | Passed: Linux native development build; no installer produced/published |
| Migration, portability and standalone suites in the complete Rust run | **23/23 passed**: migration v26 (3), portability (14), standalone Teaching (6) |
| Historical SQL verification | **25/25 byte-identical** to the exact base; 0026 registered once |
| Derived localisation documents | Regenerated after UI changes; localisation check passed again |

The complete native run is **not entirely green**. Its sole failure is the unchanged `passage_languages::indexing_a_moderately_large_source_in_each_language_is_quick` debug timing assertion: French stage **4.922012582 s** + commit **5.556192171 s** = **10.478204753 s**, over its existing **10 s** limit. The test file and Library parsing/indexing algorithms were not changed.

For comparison, a clean disposable checkout of the exact base `2cb7cb9483d305f10a3e8ad18ec5c6573c439722` passed the same isolated test in all six languages (French **9.781625297 s**). The final candidate also passed the identical isolated invocation without code or threshold changes (French **9.360934319 s**; all six languages passed). These measurements show timing variability near the limit, not a demonstrated functional failure. They do **not** turn the full-run failure into a green result or establish its cause conclusively. Review the timing margin on the Windows acceptance machine before merging/releasing.

The earlier generated-annotation timing failure cleared in the final complete run without changes to Bible recognition, annotation code or thresholds. An early contended frontend run was stopped; the final sequential frontend run passed every test. The portability citation-count assertion initially caught duplicate counting of a live link's provenance cache; the importer was fixed and the original assertion retained. No test was disabled, removed or weakened.

Preservation coverage includes fresh schema 26, populated upgrades from 19/24/25, repeat startup, original migration history and pre-upgrade backups, all eight Teaching/dependent tables, revisions/tombstones, failure rollback/FK restoration, Study/project lifecycle, legacy sessions/version-1 templates, all three built-ins, blank/personal paths, standalone link/unlink and cross-project rejection, project packages and full backup restore. The complete suites also cover participant privacy, manuscript/presentation/DOCX generation, Library browsing/search/citations/readers, exact Scripture/edition handling, research evidence/provenance and offline behaviour. The new interface tests open the standalone workspace in every language.

Detailed local evidence is retained under `/workspace/.cloud-setup/teaching3/`: `checks.json`, `frontend-final.log`, `native-final.log`, `focused-final.log`, both isolated Library timing logs, build/lint/localisation logs, and `migration-hashes.json`. These logs contain synthetic fixture data, not real user databases.

## Delivery

- Application/test implementation commit: **`795d482222ad014d1140835acc27f7bfabac382f`**.
- Implementation tree: `d0ba48151aae8a9fd66a98bbab43d3f644725644`.
- Pushed to `origin/feature/teaching-stage3-standalone`; the remote SHA was verified after the push.
- The target remained exactly **`2cb7cb9483d305f10a3e8ad18ec5c6573c439722`** at delivery verification. No merge was performed.
- A following documentation-only commit records this delivery outcome; the completion report supplies its final pushed SHA. It changes no application, migration or test source.
- Clean-source provenance verification passed for the implementation commit: schema 26, exactly 26 migrations, compatible historical catalogue.
- Commits use `[skip ci]`; no Actions workflow was manually started, and no paid Actions, installer publication, deployment or real user-data changes were performed.

**Draft PR creation is blocked.** After the source push, the authorised command below returned `Post "https://api.github.com/graphql": Forbidden`. This attempt did not create a new PR. The running enforced network policy excludes `api.github.com`. The environment configuration draft (revision 4) already includes that host, but the available tools cannot activate the draft. No credential values were displayed or requested. Resolve the network restriction before retrying.

```sh
gh pr create --repo pistio/abidingword \
  --base integration/windows-library-stage3-teaching-study \
  --head feature/teaching-stage3-standalone --draft \
  --title "Teaching Stage 3: independent project workspace" \
  --body-file /workspace/.cloud-setup/teaching3/pr-body.md
```

[Manual PR creation link](https://github.com/pistio/abidingword/compare/integration/windows-library-stage3-teaching-study...feature/teaching-stage3-standalone?expand=1). Keep it a draft targeting the integration branch. The prepared description is retained in the local body file and reproduced here for review:

> Teaching can now start independently in a project or link to a Study in that project. Study deletion retains the Teaching manuscript, blocks, materials, presentation references and saved citation provenance. Both routes use the existing editor, three built-in templates, blank/personal templates and participant-privacy pipeline.
>
> Migration 0026 performs an FK-off exclusive rebuild with populated preservation/rollback checks; migrations 0001–0025 remain byte-for-byte unchanged. Project packages now include standalone Teaching, optional links and saved provenance.
>
> Validation: 1,143 frontend tests passed; typecheck, lint, six-language checks, formatting, Clippy and frontend/native builds passed. The complete Rust run recorded 1,031 passed, one unchanged Library timing assertion failure, and eight pre-existing ignored tests. The timing case passed unchanged in isolation on both the original base and final candidate; the full-run timing margin still requires owner review. All 23 migration/portability/standalone tests passed.
>
> Complete handover: `docs/implementation/teaching-stage3-standalone-handover.md`. Draft for owner review. No merge, installer publication, deployment, paid Actions or real user-data changes.

Owner review is the stopping point. Do not merge or release until the remaining timing margin and Windows acceptance requirements are reviewed.

## Boundaries and operational notes

- Study/project deletion and restore are backend/API lifecycle operations in this milestone; it does not add a general trash-management interface. Use those operations to capture the latest Finding/citation state atomically; raw SQL deletion bypasses application lifecycle hooks.
- Saved notes keep their old content after unlinking, relinking or later Study edits. Remove a saved note as a whole from Teaching; its citation provenance is read-only there. Restore does not silently recreate Study relationships.
- The migration preservation comparator and saved-citation lookup materialise Teaching data in memory. Very large synthetic Teaching histories still need Windows memory/latency acceptance.
- No changes were made to the separate Bible recognition work. Existing Library performance thresholds remain unchanged.

## Windows acceptance before release

1. Use a disposable Windows data directory; exercise fresh install and representative schema-19/24/25 upgrades, inspect the automatic backup, and restart repeatedly. Verify Teaching descendants, tombstones, templates, exact Bible editions and citation anchors. Never point this acceptance build at the owner's sole real database.
2. In all six languages, navigate to project Teaching with no Study, create all three built-ins plus blank/personal/version-1 templates, edit/reopen, link/unlink, and open multiple linked teachings through their Study.
3. Delete and restore a Study/project using supported lifecycle calls; confirm Teaching survival after Study deletion and complete cleanup only after permanent project deletion. Restore a trashed Teaching after its Study was deleted. Study restore must not silently relink it.
4. Export/import a project with standalone and linked Teaching, saved citations and presentation sources onto a second disposable profile; confirm exact-file reconnection and unavailable-source behaviour. Restore a full backup and compare records/outputs.
5. Check manuscript, DOCX and participant outputs; private blocks, private Scripture and speaker notes must stay out of participant/audience content. Check PDF/EPUB/DOCX Library transitions and reading positions.
6. Exercise WebView keyboard/focus, narrow layout, native dialogs, offline startup, and a real second display/presenter window. Linux automation does not validate Windows/WebView or physical displays.

No installer is published by this task. Owner acceptance and a separately authorised release workflow remain required.
