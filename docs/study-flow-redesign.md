# Study flow and reusable template blocks

**Current decision: integration on hold.** See [Latest integration readiness](#latest-integration-readiness). Study implementation is complete; Teaching correctness repairs and owner approval are still required. Earlier integration orders below are conditional historical proposals, not authorisation to execute them.

The implementation and first validation below are historical records for `199f4ad958a0c946cf82df990edeeb51bfd95267`. See [Independent review and integration preparation](#independent-review-and-integration-preparation) for the subsequent verified state, confirmed defects, fixes, and unmerged integration risks.

The study-flow redesign starts at `d2d2c283`. Historical schema migrations 1–24 and the original ten built-in template resources remain unchanged. New template identities separate new defaults from legacy definitions. Studies keep independent JSON snapshots; changing or duplicating a template never transforms answers already written.

## Stage 1: writing

New POZETEK studies contain seven steps: Personen, Ort, Zeit, Ereignisse, Thema, Erkenntnisse, Konsequenzen. Other interface languages translate the method's purpose rather than its German acronym. Ten-Step retains ten steps. Each step has one spacious writing answer. Guidance is optional and collapsed. Continuous writing overrides folding without modifying saved section contents or the device's folded-section preferences. Legacy structured editors remain usable and labelled; no answers are concatenated or removed.

## Stage 2: reusable blocks

Personal templates can add, label, configure, duplicate, remove and reorder writing, guidance, headings, dividers, passage, concordance, cross-reference, word-lookup and concept/person-search blocks. Built-ins use exactly the same renderer and JSON definitions. Original field kinds remain supported. Default queries, editions and guidance are copied into study snapshots. Original built-in resources remain available for previously saved studies; six current starting templates are offered in the new-study catalogue.

Research uses existing local Bible services. Passage and user-supplied cross-reference lists require a selected installed structured edition. Concordance uses the edition's word/phrase search. Word lookup explicitly chooses a lemma/lexical entry returned by the edition. Concept/person search is textual search, with no inferred theological relationships. Kept evidence is a separate quotation block with the query, edition identity/name, canonical reference and source text in its metadata; lemma selections also retain their identifiers. Switching controls cannot relabel already returned evidence. Writing remains separate. Missing annotations and incomplete results are reported without silently substituting another edition.

No database migration is required: field definitions, configuration and source evidence use existing snapshot and block JSON columns. New enum values are additive; older binaries do not understand new templates and should not be used to edit databases containing them.

## Stage 3: Bible Study, then optional Teaching

The new-study form starts with a Bible Study. The separate Teaching preparation entry is removed. Teaching remains available from the study's existing Teaching view and next-step action, and its last-opened view is remembered.

Creation and information panels offer three formats: Sermon / Talk, Bible Study and Bible Teaching (German: Predigt / Vortrag, Bibelstudium, Bibelunterricht). The extra sermon-versus-talk question is removed. Stored kinds (`sermon`, `talk`, `bible_study`, `lesson`, `children`, `youth`, `other`) remain unchanged when existing work or templates are opened or saved without a format change. Personal templates of every legacy kind remain available under the corresponding format. Existing lesson starting points, manuscripts, leader/teacher guides, handouts, slides, presentations and presentation designs continue through the existing editor and materials pipeline.

## Compatibility and regression coverage

- The original ten template JSON resources and migrations 1–24 are byte-for-byte unchanged from `d2d2c283`. Six new definitions are installed alongside the originals; only the six current starting methods are offered for new studies. Built-in installation remains idempotent and cannot overwrite personal templates.
- Existing structured answers remain in their original keyed blocks, with their original field snapshots. The legacy editor is explicitly labelled and can be collapsed. No concatenation, destructive conversion, migration-history renumbering or fabricated applied-migration records is used.
- The existing schema-19 fixture verifies preservation of every historical object and row, migration ledger, repeated startup, safety copies and restore. Divergent schema histories still fail closed.
- New native fixtures cover all new block kinds and configuration, personal-template edits, arbitrary legacy field identifiers, complete and structure-only duplication, retained evidence, restart and full application backup/restore. Source text and personal conclusions are checked independently.
- UI regressions cover both new writing methods and old structured methods, continuous writing and remembered folding, block configuration/duplication/reordering/removal, each research service and edition/query/lemma provenance, all six interface languages and fallback, all seven legacy teaching kinds and the simplified navigation.
- An unrelated EPUB test exposed a readiness race while the suite was under load. It now waits for the asynchronous Contents button before interacting with it; reader behavior is unchanged.

## Validation in Codex cloud — 2026-10-28 UTC

All database fixtures and application test directories are synthetic. No Windows installation, real user database, NiPogi host, Claude recognition branch, release channel or deployment was used.

Repository root: `/workspace/abidingword`. Node 24.19, Rust 1.99.1; native dependency/sysroot setup is supplied by the cloud environment. Native commands use the following isolated configuration:

```sh
. /workspace/.onboarding/env.sh
export XDG_DATA_HOME=/workspace/.onboarding/study-flow/test-xdg/data
export XDG_CONFIG_HOME=/workspace/.onboarding/study-flow/test-xdg/config
export XDG_CACHE_HOME=/workspace/.onboarding/study-flow/test-xdg/cache
mkdir -p "$XDG_DATA_HOME" "$XDG_CONFIG_HOME" "$XDG_CACHE_HOME"
```

The environment sets `CARGO_BUILD_JOBS=3`, `CARGO_PROFILE_DEV_DEBUG=0` and `CARGO_PROFILE_TEST_DEBUG=0`. Tests/builds that need Node child processes run with the cloud execution sandbox exception. This is Linux validation, including synthetic Windows-database compatibility tests; it is not Windows acceptance testing.

| Exact command (from repository root) | Final result |
| --- | --- |
| `npm run typecheck` | Passed. |
| `npm run lint` | Passed. |
| `npm run i18n:check` | Passed: six languages × nine areas, 3,614 English keys, 124 components. |
| `npm run i18n:corpus` | Generated and checked by the complete frontend suite; 3,613 strings / 3,754 plural keys. |
| `npm test` | Passed: 95 files, 1,097 tests, no failures or skips. |
| `npm run build` | Passed; TypeScript and production Vite build. Vite reports a bundle-size warning. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | Passed. |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked -- -D warnings` | Passed with warnings denied. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked` | Passed: 1,007 tests, no failures, eight pre-existing ignored tests; 79 test targets. |
| `cargo build --manifest-path src-tauri/Cargo.toml --locked` | Passed: Linux development build; no Windows installer or publication. |
| `git diff --exit-code d2d2c283 -- src-tauri/migrations src-tauri/src/db/migrations.rs` | Passed; historical SQL and migration registry unchanged. |
| `git diff --check` | Passed. |

Detailed execution logs are retained in `/workspace/.onboarding/study-flow/`. Earlier runs exposed and resolved expected catalogue/order and navigation assertions, Korean terminology violations, a typed lexical-service fixture, and stale generated review documents. An interrupted environment run was restarted; only completed final runs are counted above.

### Review commits and remaining work

Stage 1 is `b2345e0` (single-answer templates and continuous writing). Stage 2 is `429f29e` (reusable blocks and evidence provenance). Stage 3 is the commit containing this report (Teaching simplification and final compatibility/test corrections). The final response records the full branch-tip SHA; `git rev-parse HEAD` identifies that source exactly.

- Owner review and separately approved Windows visual/acceptance testing remain. No Windows installer is built or published by this work.
- Cross-references are explicitly supplied passages because the current services provide no cross-reference corpus. Concept/person research is literal text search, not semantic inference. Annotation availability and truncated result sets depend on the selected installed edition and are reported in the UI.
- Older binaries do not understand the new JSON field kinds; use this application lineage for databases that contain the new templates.
- The standard Rust suite retains its eight pre-existing ignored manual fixture/report/corpus/performance tests. None were newly ignored: `document_units_dump`, `write_examples`, `real_document_editions_report`, `bundled_number_index_cost`, `oshb_corpus_reference_correctness`, `real_editions_report`, `write_interface_fixture`, `a_whole_generated_annotated_bible_is_measured`. The localisation review generator retains 237 audit findings for human review; the automated consistency, terminology and corpus checks are the required gates.
- The production bundle-size warning remains. This does not fail the build.

All three review commits use `[skip ci]`. The push targets only `feature/study-flow-and-teaching-simplification`; no PR, merge, GitHub workflow dispatch, tag, release, update publication or deployment is performed.


## Independent review and integration preparation

Reviewed in Codex cloud on **28 October 2026 (Europe/London)**. The starting tree was clean and matched the reported remote Study commit. The review fetched remote branch heads and `refs/pull/6/head`; PR approval/merge metadata was not queried. The newer branches were inspected as Git objects, not checked out, edited, merged, or represented as runtime-tested combinations.

This review changes only Study research persistence, exact citations, their command payload and regression fixtures, six translated explanatory strings, generated localisation documents, and this report. It does not implement standalone Teaching Stage 3, change the Teaching or Library workflows, or touch Bible recognition/import parsing. The historical “Stage 3” above means the older Study-flow entry simplification; it is distinct from the newer Teaching workstream’s unimplemented standalone-teaching Stage 3.

### Rechecked behavior versus the earlier claims

| Capability | Evidence and verified limit |
| --- | --- |
| Seven-step POZETEK | New `pozetek-writing-study` has exactly seven ordered writing steps; German mnemonic headings and purpose-based names in all six locales are checked by `writing-templates` and `template-catalogue` tests. Original eight-part structured POZETEK remains installed separately. |
| Ten-Step writing | `ten-step-writing-study` has ten ordered single-answer snapshots. Both methods share the writing renderer: ten rows and a 288px minimum height, growing with text. This is code/UI-test verification, not a Windows pixel/visual acceptance pass. |
| Optional guidance and writing views | New step guidance is closed by default; continuous mode reveals all steps and returning to step view restores saved folding. The full-app writing regression verifies saved text survives both changes. View preference is device-local; it does not transform study data. |
| Legacy structured studies | Original definitions and keyed answers remain unchanged; the labelled, initially expanded legacy detail editor stays usable. Legacy POZETEK/Ten-Step UI/native tests and schema-19 preservation tests pass. No existing study is converted into the new method. |
| Personal Study builder | Tests exercise block addition, title/configuration edits, duplication with distinct keys, ordering, removal and independent snapshots. Original built-ins remain recoverable and personal definitions are not overwritten by installation. |
| Reusable research | Passage, manually supplied cross-reference lists, concordance, lexical lookup and concept/person text search use the same renderer in built-in and personal Study templates. A text hit is explicitly evidence, not proof of an interpretation or a theological/historical connection. There is no automatic cross-reference corpus or semantic inference. |
| Exact source and availability | Before this review, split saves could leave blank evidence, partial passages could be labelled with their entire requested range, and wrong-edition search hits were silently discarded. These are confirmed defects in the original report’s provenance claim; fixes and reproductions are below. An unavailable configured edition disables lookup and is never silently substituted. Existing retained quotations remain stored snapshots; availability does not imply their source is currently installed. |
| Backup/restore | The native personal-template fixture now uses the actual atomic evidence-create path and compares template configuration, field snapshots, writing and provenance across restart, complete/structure-only duplication and full application backup/restore. Legacy backups are tested independently. |
| Three Teaching entry choices | Verified only on this Study branch’s existing editor: three format choices and preservation of all seven legacy stored kinds. This is **not** verification that the newer block-based Teaching editor is integrated. Its required mapping and conflicts are recorded below. |
| Localisation | Six-language key consistency, terminology, fallback/catalogue and generated-corpus tests run on the final source. This is not a fresh native-speaker review of all six languages. |

### Confirmed defects fixed here

1. **Non-atomic retained evidence.** The old UI created an empty quotation, then wrote its text and provenance with a second IPC request. A failed second request left an empty record; retry could create another. The existing `study_block_create` command now accepts optional metadata in its flat wire payload and commits quotation, provenance, ordering and study revision in one transaction. Old payloads remain valid. Invalid metadata or a simulated failure touching the study leaves the entire database state unchanged. The UI no longer performs a follow-up update and refreshes its section after success. There is no new migration or Teaching API.
2. **A requested range could overstate available source text.** The old passage renderer kept the complete requested reference even when the edition returned only some verses. Passage and supplied cross-reference results now cite each returned verse’s exact canonical identity. The original query still records the requested range/list. A six-language note explains that only available verses are shown/cited. A range containing verses 1–3 with verse 2 unavailable is tested: keeping verse 3 cites only verse 3, while retaining the original query. No recognition, parsing or imported Bible text is changed.
3. **Wrong-edition text-search responses were silently filtered.** Text search now reports the existing edition-mismatch error and offers no evidence to retain, matching the passage/concordance/lexical checks. It does not silently turn a mismatched response into “no results.”

`tests/ui/research-review.test.tsx` reproduced **four failures and two passes** against the original implementation before fixes; after fixes it and the existing research-block tests pass all 12 focused tests. The six new UI tests cover atomic keep, failed-save retry, partial passage, partial supplied cross-reference, edition mismatch and unavailable-edition non-substitution. Two new native tests check transactional rollback/validation and old/new command-payload compatibility. Native focused checks pass **11 tests** (`study_flow`: 4; `windows_dev44_compatibility`: 7).

### Compared integration heads

| Workstream | Fetched head | Relationship |
| --- | --- | --- |
| Study starting point | `199f4ad958a0c946cf82df990edeeb51bfd95267` | Three original Study-flow commits, including older Teaching entry changes. |
| Library Stage 2 / compatibility | `b3120f54b0dcece6c078727c97240828680dd63c` | Library Stage 2 is present; schema 24. |
| Teaching templates | `66ae2df6dd9a86958e323ffbc704514df51d1bcd` | Includes implementation `4e41542067b5635bfab611467bcdbc9cb5873599` plus two handover-document commits; schema 25. |
| PR #6 head | `756d0097a07cf7105ce7c12313c0a380503b707e` | Functional Stage 2 completion on the Teaching implementation; not an ancestor of the Teaching tip. |

Study, Library and Teaching implementation lines diverge at `0551f22061b3b3219315930c5061be501e94f350`. Teaching tip versus PR #6 has two documentation-only commits on the Teaching side and one functional commit on the PR side. The Teaching handover’s “Save as my template does not save blocks” statement is stale relative to PR #6: that PR adds version-2 block serialization, guidance/view/template work and portability regressions. The Library and Teaching heads are not ancestors of each other either.

A read-only `git merge-tree <merge-base> HEAD <remote-ref>` comparison was captured outside the repository; it does not create an integration branch, change the worktree/index, or perform a merge. The results describe the reported implementation (the review does not alter its Teaching conflicts):

- **Library: three overlapping paths.** `tests/support/memoryApi.ts` and both generated `docs/i18n/native-polish-{review,priority}-v0.19.md` documents. The generated documents conflict textually; the fixture auto-merges textually but still needs semantic review.
- **Teaching and PR #6: fourteen overlapping paths each.** Both generated localisation documents; `src/features/teaching/TeachingView.tsx`; `src/features/settings/TeachingTemplateEditor.tsx`; all six `src/i18n/locales/{en,de,ko,fr,es,pt}/studies.json` files; and `tests/ui/{baselines,teaching-formats,teaching-templates,teaching}.test.tsx`. Eight have direct textual conflicts: the two generated documents, two editors, and four test files. The six locale files auto-merge textually, which does not resolve terminology or behavior.

### Required integration resolutions — documented, not applied

| Area | Required resolution |
| --- | --- |
| Teaching editor ownership | Keep the newer `usesBlocks`, `BlockCanvas`, version-2 create/copy/save paths, legacy point-editor fallback, participant/private-note handling and handout/presentation pipeline. Do not resolve `TeachingView.tsx` or `TeachingTemplateEditor.tsx` by taking this Study branch’s entire older file. |
| Three simplified choices | Map Sermon / Talk → `sermon-talk` / legacy `sermon`; Bible Study discussion → `eva` / `bible_study`; Bible Teaching lesson → `bible-lesson` / `lesson`. Preserve `templateVersion: 2`, copied blocks and legacy kinds. The newer Blank starting option and personal templates must remain available; decide its placement without treating Blank as a fourth semantic Teaching type. This review changes none of that UI. |
| Version-2 personal templates | Retain PR #6’s `templateFromTeaching` → `version: 2` / `templateBlocksFromTeaching`, corresponding Save/Duplicate actions and portability tests. Taking the older serializer could downgrade a block template to points. Keep v1 personal templates and old sessions editable through the legacy path. |
| Study versus Teaching blocks | Study fields are section snapshots in existing JSON, with keyed `study_blocks` answers/evidence. Teaching uses its own `teaching_blocks`, parent hierarchy, roles and participant-facing configuration. Shared words such as “writing”, “guidance” and “block” do not imply interchangeable serialization. Keep both models; do not migrate Study answers into Teaching or force a unified builder in this integration. |
| Library fixture | Retain Library `Source.state`, `collectionNames`, `tagNames`, `filePresent`, filtered search and reader provenance, together with the Study catalogue/snapshots and new atomic-create metadata handling. A textually clean fixture merge can still omit one workstream’s behavior. No Library production code was changed here. |
| Terminology and locales | Merge Study `flow`/`builder`/new template keys alongside Teaching `builtin`/`blocks`/view/guidance keys. Preserve purpose-based Study names (POZETEK only as a German built-in name), all three Teaching purposes, established Korean terminology and legacy display aliases. Do not globally replace Hauskreis, Bibelstudium, Bibellehre or Bibelunterricht across unrelated contexts. Regenerate both corpus documents after merging locale sources; never select one generated document wholesale. |
| Tests and source attestation | Preserve new Teaching block/template/portability tests and all Study/Library preservation tests. This branch’s exact-schema-24 expectations and `scripts/build-provenance.mjs` hard-coded 24-entry catalogue check must be updated for the approved combined schema-25 source. Do not weaken history/hash validation or claim an untested combined build. |

### Migration and data lineage evidence

All **24** migration SQL files on this Study tree, the Library head, Teaching head and PR #6 head were compared byte-for-byte to `0551f22` and independently hashed. There are no differences in migrations 0001–0024. The existing native tests also enforce the pinned development-lineage hashes for 1–23, preserve schema-19 objects/rows and backups, reject the conflicting historical main migration 10, reject incomplete history without inventing applied records, and verify fresh/repeated startup at schema 24.

The per-file manifest is retained as `/workspace/.onboarding/study-review/original-migrations.sha256`; its SHA-256 is `8e2aef6d552b39143b018a6dfb9140aec8c0e8477b5f3eaf2c52c503f6f6411a`. Migration 0024 itself hashes to `7b8c3b31856ae40d2b6ad5c764515066066a20c0dc4dfac20ba5e62384a53252`.

Teaching and PR #6 both add the same `0025_teaching_blocks.sql`, SHA-256:

`68fd35adfff4af4d57ad5687249fd26442a72f470d6dbec54bb10caca6d29e36`

Library and this Study branch stop at 24. There is no competing Study migration 25 and no reason to renumber Teaching’s migration. Future integration must retain it exactly once, accept existing schema-25 databases, and test schema 19 → 25 plus repeated startup and full restore. This review’s runtime upgrade tests are **19 → 24 only**; the future combined 19 → 25 path was not executed. No historical ledger entries, SQL definitions, real database or stored legacy answers were rewritten.

### Fresh validation for this review

Logs and read-only comparison artifacts are in `/workspace/.onboarding/study-review/`. Native checks use the existing cloud sysroot setup and stable Rust **1.99.1**, with three build jobs and debug information disabled. Test data/config/cache directories are isolated under that review directory, not an installed user profile.

```sh
. /workspace/.onboarding/env.sh
export XDG_DATA_HOME=/workspace/.onboarding/study-review/test-xdg/data
export XDG_CONFIG_HOME=/workspace/.onboarding/study-review/test-xdg/config
export XDG_CACHE_HOME=/workspace/.onboarding/study-review/test-xdg/cache
mkdir -p "$XDG_DATA_HOME" "$XDG_CONFIG_HOME" "$XDG_CACHE_HOME"
```

| Exact command | Review result |
| --- | --- |
| `npx vitest run tests/ui/research-review.test.tsx` before fixes | Four failures / two passes; reproduced defects. |
| `npx vitest run tests/ui/research-review.test.tsx tests/ui/template-blocks.test.tsx` after fixes | 12 passed. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked --test study_flow --test windows_dev44_compatibility` | 11 passed. |
| `npm run typecheck` | Passed. |
| `npm run lint` | Passed. |
| `npm run i18n:check` | Passed: six languages × nine areas; 3,615 English keys / 124 components. |
| `npm run i18n:corpus` | Regenerated; 3,614 strings / 3,755 plural keys, 237 non-gating review findings. Corpus tests pass. |
| `npm test` | Passed: 96 files, 1,103 tests; no failures/skips. |
| `npm run build` | Passed; existing >500kB bundle-size warning remains. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | Passed. |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked -- -D warnings` | Passed; warnings denied. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked` | Passed: 1,009 tests; no failures; eight pre-existing ignored tests; 79 test targets. |
| `git diff --exit-code 199f4ad -- src-tauri/migrations src-tauri/src/db/migrations.rs resources/templates` | Passed: original templates, SQL and registry unchanged. Independent SHA-256 comparison additionally covered all 24 SQL files in four compared trees. |
| `git diff --check` | Passed. |

No Windows visual/acceptance run, installer build, production data check, paid service/Action, integration build or native-speaker language review was performed. Eight pre-existing ignored Rust manual/corpus/performance/fixture checks remain skipped, as listed in the original report; none were newly ignored. PR #6 and Library/Teaching branch tests are not reported as freshly run here. The full suites in this table validate **this Study branch**, not a hypothetical merged application.

### Proposed safe integration order — approval required before creating it

1. After owner approval, create an integration branch from the latest verified `fix/windows-dev44-schema-compatibility` (the reviewed head is `b3120f5`). Preserve Library Stage 2 and its compatibility lineage.
2. Merge the Teaching branch retaining implementation `4e41542`, migration 0025 and the handover records. Keep the newer editor and both v1/v2 storage paths.
3. Integrate PR #6’s Stage 2 completion `756d009`, which is missing from the Teaching branch tip. Recheck the current PR/head before integrating; avoid duplicating it if the branch has since advanced.
4. Bring in the Study branch including this review’s targeted fixes, resolving Teaching files in favor of the newer block implementation and reapplying only the compatible Study entry intent. Preserve Study snapshots, Library metadata, all locale namespaces and each workstream’s tests. Do not blindly cherry-pick the old Study Teaching editor changes over the new editor.
5. Reconcile schema-25 guards/source attestation and expected-version tests; regenerate localisation documents; run complete frontend/Rust/i18n/portability checks on the actual combined tree. Exercise fresh, v19, v24 and existing v25 synthetic databases, legacy structured studies and legacy teachings, v2 personal templates, saved evidence, restore, handouts and presentation/privacy behavior. Obtain separate approval before Windows acceptance, any main merge or publication.

No integration branch was created. No Teaching Stage 3 or additional Library feature was implemented. Only this designated Study branch receives the review/fix commit, with `[skip ci]`; no merge, PR creation, workflow dispatch, deployment, release or installer publication is part of delivery.


## Latest integration readiness

**Decision: HOLD — stop for owner approval; do not create an integration branch or merge.**

Checked 28 October 2026, 21:35 GMT in Codex Cloud/GitHub. This is a documentation-only readiness refresh of the reviewed Study source `3bb6dc02ec141adf3b2448bb5b4d9847c70dc593`. No Study redesign, Teaching repair, Library feature, migration edit, deployment, release, installer build or paid Action was performed. The Study and Teaching block models must remain separate.

### Pushed Study history confirmed

The local reviewed commit and fetched remote `feature/study-flow-and-teaching-simplification` both resolve to `3bb6dc02ec141adf3b2448bb5b4d9847c70dc593`. All five commits below are reachable from that remote head, so the design, implementation and final correctness review are pushed:

| Commit | Content |
| --- | --- |
| `d2d2c283bb05ab0b9f165d643b0e46d10369e790` | Accepted Study-flow design. |
| `b2345e06987dc206bbcb66f34b322f563438c30a` | Seven-step POZETEK / Ten-Step writing, continuous view and legacy answers. |
| `429f29e99e2ed02b9fe6857fe1b13db82086a351` | Personal Study builder, reusable research blocks and provenance. |
| `199f4ad958a0c946cf82df990edeeb51bfd95267` | Original simplified Teaching entry choices and compatibility checks. Its older Teaching editor code must not replace the newer editor during integration. |
| `3bb6dc02ec141adf3b2448bb5b4d9847c70dc593` | Final Study correctness review: atomic retained evidence, exact returned-verse citations and explicit edition-mismatch errors. |

The documentation commit containing this refresh is additional to that unchanged, completed Study source history.

### Latest remote and PR state

| Item | Current advertised head / state | Readiness meaning |
| --- | --- | --- |
| Library / Windows-compatible baseline | `fix/windows-dev44-schema-compatibility` → `b3120f54b0dcece6c078727c97240828680dd63c` | Includes Library Stage 2 via PR #5 squash; schema 24. Preserve it as the compatible baseline. |
| Teaching foundation | Implementation `4e41542067b5635bfab611467bcdbc9cb5873599`; `feature/teaching-three-templates` tip `66ae2df6dd9a86958e323ffbc704514df51d1bcd` | The two later commits are handover documentation; schema 25 with the new block editor. |
| Teaching completion / PR #6 | `cursor/teaching-stage-2-completion-9953` and `refs/pull/6/head` → `756d0097a07cf7105ce7c12313c0a380503b707e` | GitHub page reports **OPEN**, no merged time or merge commit. Targets `feature/teaching-three-templates`; page records base SHA `4e41542…`. The completion is still not in the foundation branch tip. |
| Teaching correctness repair | **No newer repair branch or PR head is advertised by this repository.** | The visible completion head is unchanged; it is not a replacement correctness repair. Unpushed/private work elsewhere cannot be inferred. No approved correction commit is established. |
| Separate integration/correctness assessment | `review/codex-integration-report-2026-10-28` → `4b2552e250eff1e8ad65e948cec438e9db998056` | Adds only `docs/implementation/codex-integration-review-2026-10-28.md` to the Library baseline. It is a report, not a repair. Its earlier Study head `199f4ad…` predates the now-pushed `3bb6dc0` fixes. |

PR status was read from the GitHub page's embedded structured payload: `state=OPEN`, `headSha=756d009…`, `baseBranch=feature/teaching-three-templates`, `mergeCommitSha=null`, `mergedTime=null`. The GraphQL request from `gh pr view` returned **Forbidden**; the available page payload did not establish current reviewer approval. Neither an open PR nor a fetched head is approval. `git ls-remote` advertised PR heads 1–6 and no later correctness PR. Do not interpret this as proof about unpublished work or as permission to integrate.

The separate assessment [at its exact report commit](https://github.com/pistio/abidingword/blob/4b2552e250eff1e8ad65e948cec438e9db998056/docs/implementation/codex-integration-review-2026-10-28.md) explicitly says not to integrate Teaching yet. Its pending output/model findings still apply to the unchanged published Teaching head. This refresh inspected the relevant source paths and branch identity; it did not repeat that assessment's runtime reproductions or repair those workstreams.

### Correctness gates that precede integration approval

- [ ] **Private Scripture output:** approved Teaching repairs must distinguish truly teaching-wide passages (`blockId == null`) from block-scoped passages, applying participant/leader visibility consistently to references and quoted text, including missing/deleted blocks. The assessment reproduced private block Scripture leaking into participant output through `passagesLine` / `scriptureBlocks` in `src/features/materials/derive.ts`.
- [ ] **Written headings:** block manuscripts must preserve a written outline heading together with its body. The assessment reproduced the non-section title being dropped when body text is present. Require preview/export regressions for the corrected source.
- [ ] **Role ownership and live sources:** the approved Teaching fix must define and test singleton-role duplication/deletion and mirrored preparation columns, or a consistent block-ID source-link policy. Editing duplicate introductions/conclusions must not make separate live slides follow whichever copy was saved last. The assessment's role finding was a static trace, not a fresh native reproduction in this readiness check.
- [ ] **Block-link provenance/ownership:** validate that linked blocks exist, are live and belong to the same preparation; test deletion and import/remapping. The assessment identified a command-boundary gap for `passage_add(block_id)`; no ordinary-UI cross-project leak was demonstrated. Keep Study evidence provenance distinct from Teaching block links.
- [ ] **Library Undo follow-up:** record a separately scoped fix or explicit owner disposition for archived → Trash → Undo restoring the original archived state rather than forcing active. The assessment identified this by static trace. Do not erase Library Stage 2 or implement that fix in the Study readiness task.
- [ ] **Approve the exact corrected Teaching source and its evidence.** Pin the correction branch/head, PR status and owner approval before authorising any integration branch or merge. None is pinned as approved by this refresh. Standalone Teaching Stage 3 and its proposed rebuild remain out of scope.

### Current pairwise conflict inventory

Fresh read-only comparisons used `git merge-tree <common-base> <left> <right>` and path intersections. They created no branch, index merge or combined application. Counts describe the pinned trees above; a future correctness branch requires comparison again.

| Pair | Common base | Overlapping paths / direct textual conflicts |
| --- | --- | --- |
| Library ↔ Teaching foundation | `0551f22…` | 3 / 2: `src/lib/api.ts` auto-merges textually; both generated localisation documents conflict. |
| Library ↔ Teaching completion | `0551f22…` | Same 3 / 2. |
| Library ↔ reviewed Study | `0551f22…` | 3 / 2: `tests/support/memoryApi.ts` auto-merges textually; both generated documents conflict. |
| Teaching foundation ↔ completion | `4e41542…` | 0 / 0: completion and foundation handover documents are separate deltas. This says nothing about correctness of their combined behavior. |
| Teaching foundation ↔ reviewed Study | `0551f22…` | 14 / 8; exact paths below. |
| Teaching completion ↔ reviewed Study | `0551f22…` | Same 14 / 8. |

The eight direct Teaching/Study conflicts are `src/features/teaching/TeachingView.tsx`, `src/features/settings/TeachingTemplateEditor.tsx`, `tests/ui/{baselines,teaching-formats,teaching-templates,teaching}.test.tsx`, and `docs/i18n/native-polish-{priority,review}-v0.19.md`. The other six overlaps are all `src/i18n/locales/{en,de,ko,fr,es,pt}/studies.json` files. A textual auto-merge is not a semantic approval.

### Concrete semantic resolution checklist — for a later approved integration

- [ ] **Preserve the complete Study change set.** Retain `pozetek-writing-study` (seven steps), `ten-step-writing-study` (ten), their shared spacious answer renderer, collapsed optional guidance, device-local continuous/step views, personal builder and reusable research fields. Keep all original structured template resources and saved field snapshots/answers; never concatenate, convert, drop or replace them with the new method definitions.
- [ ] **`TeachingView.tsx`: keep the newer implementation.** Retain `usesBlocks(prep)` → `BlockCanvas`, modern creation with `templateKey`, `templateVersion: 2` and copied `blocks`, plus legacy point-editor fallback. Do not take the Study side's entire file or restore its point-only `formatStarts` creation path. Carry only the compatible Study → optional Teaching entry intent into the newer workflow.
- [ ] **Keep all three v2 built-ins.** Map Sermon / Talk → `sermon-talk` / legacy `sermon`; Bible Study discussion → `eva` / `bible_study`; Bible Teaching lesson → `bible-lesson` / `lesson`. Preserve the modern Blank starting option and personal templates without inventing a fourth semantic Teaching type. Preserve all seven old stored kinds. Blank placement and any wording decision must be resolved on the approved Teaching design, not by reverting the editor.
- [ ] **`TeachingTemplateEditor.tsx`: preserve both versions.** Keep modern `blockMode`, the version-2 block builder, parent keys, privacy, roles and guidance; retain v1 point-template editing. Apply any compatible three-choice labeling without forcing every template through the Study side's older points-only form.
- [ ] **Serialization and guidance: retain PR #6.** Keep `templateFromTeaching` branching on `usesBlocks`, `version: 2`, `templateBlocksFromTeaching`, ordered stable keys/parent remapping, participant-facing/role/guidance configuration and omission of answer bodies from personal templates. Preserve `SaveTeachingTemplate`, `TeachingTemplates`, `BlockCanvas` guidance/view behavior and duplication support. Verify v2 Save → reopen stays v2; old templates remain v1. Never choose the older points serializer to clear a conflict.
- [ ] **Separate block models.** Study fields remain JSON snapshots with keyed `study_blocks`; Teaching remains its own `teaching_blocks` hierarchy with roles/participant configuration. Do not coerce either model into the other's schema, convert existing answers/sessions, or introduce a shared serialized block union during integration.
- [ ] **Study evidence API and persistence:** retain `CreateStudyBlockRequest`'s backward-compatible flat payload with optional metadata, `blocks::create_with_data`'s single transaction, validation and rollback, and the frontend's single create call. Keep exact returned-verse canonical references, original query, edition ID/name, source text and lexical/mode metadata, explicit edition-mismatch errors and unavailable-source behavior. Do not restore split create/update saves or cite an incomplete returned range as complete. Integrate the frontend and native atomic-save changes together.
- [ ] **`src/lib/api.ts`: retain both workstreams' signatures.** Keep Library search `sourceType`, `language`, `state` and pagination; also retain Teaching `templateKey`/`templateVersion`/`blocks` inputs, all five block CRUD/reorder/duplicate methods, `teachingPassageAdd.blockId` and its corrected ownership behavior. Study `CreateStudyBlock.data` remains a separate imported input type. A clean textual merge must not remove any command mapping.
- [ ] **`tests/support/memoryApi.ts`: preserve combined parity.** Keep all 16 installed Study definitions with the six-method new-study catalogue, snapshots and atomic metadata create; retain Library source state/collections/tags/file-presence and filtered search/reader provenance. Keep or supply modern Teaching block/v2 template fixtures consistent with actual native commands. Do not take a whole older fixture file to fix a type error.
- [ ] **UI and native tests: reconcile behavior, not assertions alone.** Retain Study `writing-templates`, `pozetek`, `ten-step`, `template-blocks`, `research-review`, `study_flow` and schema-19/backup tests. Resolve the four conflicting Teaching/baseline UI files using the modern three templates plus legacy compatibility; retain PR #6's block-view, guidance, v2 Save/Duplicate and portability coverage. Add the approved correctness regressions for private Scripture, headings, role copies/deletion and source links. Do not delete tests or restore outdated point-only UI expectations merely to obtain green results.
- [ ] **Six languages:** combine Study `flow`, `builder` and new Study-template keys with modern Teaching `builtin`, `blocks`, views and guidance keys in every language. Preserve purpose-based POZETEK names (German acronym only for the German built-in name), established Korean terms and user-authored wording. Resolve `Bibelstudium`/EVA, `Bibelunterricht`, `Predigt / Vortrag`, legacy `Hauskreis` and `Bibellehre` by context. Regenerate both native-polish documents from the final merged locale files; do not choose either generated side wholesale. Run six-language parity, terminology, fallback and corpus checks.
- [ ] **Schema and provenance attestation:** preserve 0001–0025 byte-for-byte and register version 25, name `teaching_blocks`, from `0025_teaching_blocks.sql` exactly once. Keep 1–24 history validation and reject divergent `main` migration 10; change only combined-version expectations/source-attestation logic that currently requires exactly 24. Require fresh/v19/v24/v25 synthetic migration, repeated startup, full backup/restore and project export/import coverage. Never alter SQL, renumber versions, falsify applied history or introduce standalone Stage 3 as a conflict resolution.
- [ ] **Outputs and preserved source links:** retain the corrected Teaching handout/manuscript/presentation pipeline, Library source/citation metadata and stable/remapped IDs. Verify participant versus leader output and live source links after edit/duplicate/delete/restore; source and edition absence must not cause silent substitution. Study text matches remain evidence, not asserted historical/theological connections.

### Migration and check evidence for this readiness snapshot

Fresh byte comparisons confirm migrations 0001–0024 on Library, Teaching foundation and completion match `0551f22` and reviewed Study. The foundation and completion both have exactly the same 0025 bytes, SHA-256 `68fd35adfff4af4d57ad5687249fd26442a72f470d6dbec54bb10caca6d29e36`. Study adds no competing migration. No migration 0001–0025 was modified. The preceding independent review contains the 24-file hash manifest and schema-19 evidence; it remains valid for unchanged Study application code.

Checks performed here: clean starting tree; fetch/prune and advertised branch/PR refs; all five Study commits reachable on the remote branch; live PR-page state/head/base/merge fields; exact review-report and source comparisons; all six pairwise path/three-way text comparisons; migration-byte/hash equality; documentation-only diff and whitespace checks. Artifacts are in `/workspace/.onboarding/study-readiness/`.

No application test suite or build was rerun for this documentation-only refresh. The previous source-specific results at unchanged `3bb6dc0` remain **1,103 frontend tests; 1,009 Rust tests with eight pre-existing ignores; typecheck/lint/i18n/Clippy/build passed**. Those are not fresh Teaching-correctness or combined-application results. PR reviewer approval is not verified; published correction commits are not available; neither is treated as a pass. No Windows acceptance or paid workflow was run.

### Owner approval stop

The earlier proposed merge order is superseded by this gate: **first obtain and review the published correctness fixes and their approved exact SHAs** (including PR #6's functional completion and the separate Library Undo disposition). Only after owner approval may a separate task create an integration branch from the latest approved Windows-compatible baseline, integrate the corrected Teaching stream, then integrate reviewed Study with the checklist above and validate the actual combined tree. Re-fetch all heads and PR status at that time.

This task stops with the readiness report. It creates no integration branch and performs no merge, repair implementation, deployment, release, paid Action or Windows installer build. The only new deliverable is a documentation commit on the designated Study branch, pushed with `[skip ci]`. **Await owner approval; do not proceed automatically.**
