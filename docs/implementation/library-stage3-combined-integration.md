# Library Stage 3 combined integration

29 October 2026. Controlled development integration in Codex Cloud. Owner review only: no PR merge, main merge, installer publication, deployment, paid Actions dispatch, real user data or Teaching Stage 3 implementation.

## Verified sources and review

| Role | Branch | Exact commit |
| --- | --- | --- |
| Validated combined baseline / requested PR target | `integration/windows-library-teaching-study-review` | `fa4e1f9ddec01f91d5137884ee399b8f198ff069` |
| Starting working branch | `integration/windows-library-stage3-teaching-study` | `fa4e1f9ddec01f91d5137884ee399b8f198ff069` |
| Library Stage 3 source | `feature/library-stage3-navigation` | `67c72e1b7caa148ec24c31c6f0b54fe385851a37` |

All remote branches were fetched and the advertised tips above independently verified with `git ls-remote`. Neither source had advanced beyond the requested commit. The existing clean checkout was used; no worktree or divergent-main base was created.

PR #9 was reviewed through its GitHub page: draft Library Stage 3 navigation, head `feature/library-stage3-navigation`, base `fix/windows-dev44-schema-compatibility`. Its `docs/implementation/library-stage3-handover.md` was read at the exact source commit. PR #10 was reviewed through its GitHub page: draft combined Teaching/Study integration, head `integration/windows-library-teaching-study-review`, base `fix/windows-dev44-schema-compatibility`. Its `docs/implementation/windows-library-teaching-study-integration.md` was read in the combined baseline. Those source-specific historical test results are not substituted for this task's validation.

## Integration and conflicts

The complete Library Stage 3 branch was merged locally into the designated working branch. Conflicts occurred only in:

- `docs/i18n/native-polish-priority-v0.19.md`
- `docs/i18n/native-polish-review-v0.19.md`

Both documents were regenerated with `npm run i18n:corpus` from the combined six-language locale tree, retaining the newer Study/Teaching terminology corrections and new Library labels. The resulting corpus contains 3,706 strings / 3,847 plural-form keys with 243 advisory review findings; this is not native-speaker approval.

`src/lib/api.ts` merged the optional `collectionId` Library search argument while retaining all modern Teaching and Study APIs. `tests/support/memoryApi.ts` merged collection membership filtering while retaining atomic Study evidence creation and all combined fixtures. Other Stage 3 UI/native changes merged automatically. No Teaching or Study implementation file was changed relative to the combined baseline.

The merge retains collection-scoped browsing/name/text searches, direct nested collection membership, archive/Trash behavior, stale-result suppression, Library filter/query/page/scroll/focus return state, exact text/PDF/formatted-document positions, Scripture transfers and citation words/provenance. Navigation positions remain session-only; this does not add cross-restart navigation persistence.

Teaching still has its distinct block editor/storage model, all three built-in templates, blank/personal paths, legacy sessions/version-1 templates, deterministic role ownership and participant-private output filtering. Study retains its seven-/ten-step writing methods, legacy structured answers, personal/research blocks, exact edition/source provenance and atomic evidence saving. Original source handovers remain in the repository.

## Historical migrations

All migrations 0001–0025 were independently compared byte-for-byte with `fa4e1f9`; all 25 match. The Stage 3 source's historical 0001–0024 files also match. The migration registry is unchanged, schema 25 remains current, and no migration is added or duplicated.

SHA-256 manifest of the integrated files:

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

## Runtime and validation method

Validation uses the retained Node 24.19.0 / npm 11.9.0 installation and Rust stable 1.99.1. Dependency declarations and lockfiles are unchanged; installed dependencies and incremental outputs are reused. Native checks use `/workspace/.cloud-setup/native`, a fresh disposable Debian 13 container with the documented Tauri libraries, mounted workspace, current proxy/CA trust, three Cargo jobs and debug information disabled. Heavy frontend/native/performance validation is run sequentially to avoid four-CPU contention. Only synthetic/temp databases and repository-bundled licensed fixtures are used.

## Coverage and Windows acceptance

Automated coverage includes the new `library-stage3-navigation` UI regressions, Original PDF restoration in `source-viewer`, existing formatted EPUB/DOCX, reference/citation navigation, Teaching templates/privacy/materials/presentations, Study writing/research/evidence, native collection-scoped search, backup/project portability, and synthetic fresh/schema-19/schema-24/schema-25 migration/restart tests. The prior combined integration's schema-25 coexistence regression remains in the suite. All six locale packs are checked for parity, terminology and corpus freshness.

Codex Cloud is Linux. A real Windows desktop visual/accessibility acceptance run remains required before acceptance; no such run is claimed here. Use a disposable data folder and invented/licensed fixtures to check:

1. Browse parent/child collections by keyboard, confirm direct-membership semantics, and apply type/language/archive/name/text filters. Change collection while a later result page is pending; confirm no stale hits appear.
2. Open sources from list/search and return to the same collection, filters, page, query draft/submitted query, loaded results, scroll and focused opener. Delete or hide an opener and confirm the Library heading is a usable fallback.
3. Transfer citation targets and Scripture references between the full reader and Sources pane at narrow and wide widths. Confirm cited words/provenance remain exact and the Study's passage/answers are unchanged; Back to Library must restore its context.
4. Exercise PDF Reading/Original with page fractions, scrolling and zoom; formatted EPUB/DOCX with their anchors; text with paragraph/pixel offsets; Settings/reopening/back-forward history. Explicit citation/search targets must override remembered positions. Repeat with archived sources and missing original files.
5. Verify all three block Teaching templates, blank/personal starts and representative legacy sessions alongside simplified/legacy Study templates. Generate participant and leader outputs and confirm private content stays excluded where required.
6. Check six-language labels, long translated text, keyboard focus, bounded About disclosure and display scaling. Restart to confirm the documented session-only position scope, without expecting cross-restart position persistence.
7. On a disconnected Windows system, exercise installed Scripture, Library reader/search/citations, Study evidence, Teaching outputs, project export/import and a synthetic backup/restore. Do not open real user data or publish an installer as part of this task.

## Deferred milestone

Teaching Stage 3 is not implemented. Only after this integration is accepted should a separate owner-approved task prepare standalone Teaching sessions, optional Study links, project-level Teaching navigation, safe Study deletion/unlinking, and complete project export/import compatibility. No storage or navigation model for that milestone is introduced here.

## Validation results

| Check | Result |
| --- | --- |
| `npm run typecheck` | Passed. |
| `npm run lint` | Passed. |
| `npm run i18n:check` | Passed: six languages × nine areas, 3,707 English keys, 126 components. |
| `npm run i18n:corpus`, then corpus `--check` | Regenerated and passed freshness checks. |
| `npm run build` | Passed; existing >500 kB chunk-size advisory remains. |
| `npm test -- --maxWorkers=3` | 100 files / 1,134 tests passed; no failures or skips. |
| Native `cargo fmt -- --check` | Passed. |
| Native `cargo clippy --all-targets --locked -- -D warnings` | Passed; warnings denied. |

No frontend assertion or timeout was changed. A transient executor transport disconnect affected one read-only inspection; the connection recovered, the running validation continued, and its completion status was checked. This was not an application test failure.

Native application build: `cargo build --offline --locked` passed in the prepared container. This built the Linux development application and did not create or publish installer bundles. The complete native validation uses:

```sh
AW_NATIVE_NETWORK=none /workspace/.cloud-setup/native test --offline --locked -- --test-threads=1 --show-output
```

This keeps the existing assertions and performance thresholds intact and follows the Stage 3 source handover's sequential native-test method. No competing heavy build/test process runs during the suite.

Preservation was also verified by direct blob comparisons with the baseline: all 16 Study template JSON files, the Teaching template definitions, all 12 files under `src/features/teaching`, and all 35 files under `src/features/studies` are unchanged. The merge does not change dependency manifests, lockfiles, release workflows or the separate Scripture recognition implementation.

## Complete native result and delivery

The complete offline Rust suite exited successfully: **1,022 passed, zero failed, eight pre-existing ignored checks** across 82 reported targets (including zero-test binary/doc targets). All synthetic fresh/schema-19/schema-24/schema-25 migration/restart checks, full backup/restore and project portability checks, collection filtering, legacy/new Teaching and Study checks, privacy/output checks, and bundled Scripture import/exact historical text checks completed. No test was removed, newly ignored, disabled or weakened.

Existing ignored manual/report/full-size fixture checks, not counted as passes:

- `document_units_dump`
- `write_examples`
- `real_document_editions_report`
- `bundled_number_index_cost`
- `oshb_corpus_reference_correctness`
- `real_editions_report`
- `write_interface_fixture`
- `a_whole_generated_annotated_bible_is_measured`

The six-language indexing performance assertion passed unchanged. Stage-plus-commit totals were approximately EN 2.96 s, DE 4.96 s, KO 3.37 s, FR 7.05 s, ES 7.54 s and PT 5.58 s, each below its existing ten-second limit.

Merge commit: `c514380e8e48a2565bbca708882956c2d081f448`.

- First parent / combined baseline: `fa4e1f9ddec01f91d5137884ee399b8f198ff069`.
- Second parent / complete Library Stage 3 source: `67c72e1b7caa148ec24c31c6f0b54fe385851a37`.
- Working/delivery branch: `integration/windows-library-stage3-teaching-study`.
- Requested new draft PR base: `integration/windows-library-teaching-study-review`.
- PRs #9 and #10 were reviewed only; this task does not merge or modify them.

Validation applies to the application tree in that merge commit. This handover is a subsequent documentation-only commit; its final pushed branch-tip SHA is reported in the task delivery message. Raw current-instance validation logs remain under `/workspace/.cloud-setup/stage3-logs`, outside tracked source.

## Remaining limitations and PR access

No application test failure remains. The existing bundle-size advisory, Windows visual/accessibility acceptance and real desktop GUI acceptance remain as stated above. Environment restoration in a new task and real external assistant providers were not tested or claimed.

Git fetch works through the supplied HTTPS route. GitHub API preflight returned `Forbidden`; current runtime status confirms restricted/enforced networking without `api.github.com`. The existing configuration draft already contains the required domain addition, verified during this task, but it has not been applied to the running policy. Saving a draft alone does not grant runtime access. No missing API credential has been established and no new secret was requested.

The new draft PR must use the exact head/base above, never main. Its body is prepared in `/workspace/.cloud-setup/stage3-pr-body.md`; the title is `Integrate Library Stage 3 navigation with combined Teaching and Study`. Creation is attempted after pushing the complete branch. A prepared body or compare URL is not an actual PR. The final task report records the resulting PR or concrete creation blocker.

Stop for owner review. No PR merge, main merge, installer publication, deployment, paid Actions dispatch, real user data access or Teaching Stage 3 work is authorized or performed by this delivery.

## Observed push and PR-creation outcome

The merge and handover were pushed successfully. The first verified delivery tip was `51d5f90fa6c7d159eb4fe4f1b987905778806189`; this final addendum is documentation only, with the final pushed tip recorded in the task delivery message. The requested PR base remained exactly `fa4e1f9ddec01f91d5137884ee399b8f198ff069`.

The actual authorized draft-creation command was:

```sh
gh pr create --repo pistio/abidingword --draft \
  --base integration/windows-library-teaching-study-review \
  --head integration/windows-library-stage3-teaching-study \
  --title 'Integrate Library Stage 3 navigation with combined Teaching and Study' \
  --body-file /workspace/.cloud-setup/stage3-pr-body.md
```

It failed with `Post "https://api.github.com/graphql": Forbidden`, returning no PR URL. No PR creation or PR merge is claimed. **Draft creation remains outstanding due to the enforced runtime network policy**, not an application/test failure. Apply the already-saved `api.github.com` allowance to the running environment and retry the prepared command; destination authentication can only be assessed after that access is available. Do not replace this with a main-targeted PR or claim a compare link is a created PR.

The pushed code and handover are available for owner review while that external prerequisite is resolved. All required local validation is complete; Windows acceptance remains pending as documented above.
