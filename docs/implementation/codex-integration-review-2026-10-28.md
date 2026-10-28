# Abiding Word integration review — 28 October 2026

## Decision and scope

Do not integrate Teaching or implement Stage 3 yet. Library/Sources PR #5 is
already squash-merged into the Windows compatibility branch. Teaching PR #6 is
open. Its inherited block implementation has confirmed output defects below.
The standalone Teaching design also needs an explicit foreign-key-safe rebuild
protocol before implementation approval.

This is a report-only review. No application code, tests, historical migrations,
real user databases, production systems, installers or releases were changed.
No integration branch was created. The report branch is
`review/codex-integration-report-2026-10-28`, based on
`b3120f54b0dcece6c078727c97240828680dd63c`.

## Evidence and current GitHub state

Remote heads and PR refs were fetched on 28 October 2026. The GitHub REST API
returned HTTP 403, so PR state, head/base SHAs and the Stage 3 design were read
from the GitHub PR pages' embedded structured data and rendered descriptions.
Read-only Git access worked through the existing authenticated HTTPS route after
using the supported sandbox network permission. No token was requested.

| Item | Exact commit | State |
| --- | --- | --- |
| Initial checkout, local `work`; remote `main` | `a7bc798407dc3bd60598ef5991e7c04e8d4356f1` | Divergent schema catalogue ending at 10 |
| Historical Windows baseline | `0551f22061b3b3219315930c5061be501e94f350` | Restored lineage through 24 |
| Windows compatibility branch now | `b3120f54b0dcece6c078727c97240828680dd63c` | Baseline plus squash merge of PR #5 |
| Library PR #5 head | `9d3d92a6ac24ce1cc69a155a31fcb05fb1233c8d` | MERGED, 28 October 2026 at 20:48:19 GMT |
| Teaching template parent branch | `66ae2df6dd9a86958e323ffbc704514df51d1bcd` | Initial block implementation plus handover documentation |
| Teaching PR #6 head | `756d0097a07cf7105ce7c12313c0a380503b707e` | OPEN; Stage 2 completion |
| Study simplification | `199f4ad958a0c946cf82df990edeeb51bfd95267` | Separate four-commit stream after baseline |
| Recognition branch | `ac4d7f47a59679a4ffd7300e185cbd90c34748de` | Already an ancestor of Windows baseline |

PR #5 targets `fix/windows-dev44-schema-compatibility`; its two commits are
`6adf1d35938af0a764c473f29f7511b9603e7c7c` and `9d3d92a…`. The squash commit
`b3120f5…` has exactly the same Git tree as PR #5's head
(`5fe57988ede34faa16de10e3dd9e1bfdc3311228`). Git ancestry alone would wrongly
suggest the feature was absent: the original head is not an ancestor after squash.

PR #6 targets `feature/teaching-three-templates`, not the Windows branch or
`main`. Its recorded base SHA is `4e41542067b5635bfab611467bcdbc9cb5873599`;
its one completion commit is `756d009…`. The target branch subsequently contains
two handover documentation commits (`05332bf…`, `66ae2df…`). The complete Teaching
review here covers both the initial block change `0551f22..4e41542` and the PR
completion `4e41542..756d009`, distinguishing inherited defects from completion work.

Study simplification commits after baseline are `d2d2c28` (design), `b2345e0`
(continuous writing / legacy answers), `429f29e` (research and writing blocks),
and `199f4ad` (Teaching consolidation / compatibility). They are pushed; they
are not included in the current Windows branch. No newer recognition head was
advertised by the remote. Unpushed work on another machine cannot be established
from GitHub; there were no initial local tracked or untracked changes here.

## Findings by severity

### P1 — private Scripture blocks bypass participant-output visibility

**Teaching initial block implementation, still present in PR #6.**
[`src/features/materials/derive.ts`](../../src/features/materials/derive.ts):
`passagesLine` at lines 127–131 filters only `pointId`; `scriptureBlocks` at
lines 92–96 does the same. Block outputs prepend those passages before checking
each block's `participantFacing` flag (lines 307–329, 346–350).

Reproduction executed against the exact PR #6 archive: create a version-2
Teaching with a Scripture block marked private and a passage with
`pointId = null`, `blockId = private-block-id`; derive the ordinary participant
question sheet. It still contains the passage reference. Enabling passage text
uses the same unfiltered selection. This concerns the Scripture reference/content,
not a demonstrated leak of the private prose note body.

**Fix:** distinguish genuinely teaching-wide passages (`blockId == null`) from
block-scoped ones and select block passages using the output's visibility rules.
Apply the rule consistently to references and quoted Scripture, including deleted
or missing blocks. Test private, participant, leader, deleted-block and legacy
passages across handouts, manuscripts and presentations. No migration rewrite is needed.

### P2 — block manuscript loses written outline headings

**Teaching initial block implementation, still present in PR #6.**
`src/features/materials/derive.ts`, `blockManuscript`, lines 311–316.
For a non-section block with both a title and body, the body branch prevents the
title branch from running. A sermon outline point titled `UNIQUE HEADING` with
body `Body words` produces only `Body words` in the manuscript. This was reproduced
by calling the actual derivation function, without changing repository tests.

**Fix:** emit the outline-point heading followed by its body; define which other
block types need both fields, avoiding redundant section labels. Verify preview
and each format that consumes the derived document. Existing legacy manuscript
tests do not cover this version-2 case.

### P2 — duplicating role-bearing blocks makes live presentation text ambiguous

**Teaching initial block implementation, still present in PR #6; static trace.**
`src-tauri/src/db/teaching_blocks.rs`: `duplicate` at lines 635–649 retains the
original `role`; `mirror_role` at approximately lines 308–341 writes a single
preparation column for introduction/conclusion/message/notes. The UI offers
Duplicate on these blocks in `src/features/teaching/BlockCanvas.tsx`.

Duplicate the introduction writing block, then edit the copies to different
texts. Both blocks remain in the editor, but the single preparation introduction
contains whichever copy was saved last. `db/presentations.rs::propose_blocks`
creates preparation/introduction source links for both, so both slides follow
that same last value. Deleting a role block also does not reconcile its mirrored
column (`teaching_blocks.rs::delete`, lines 594–617). This is an observable model
conflict, not a hypothetical migration collision. Native reproduction remains unrun.

**Fix:** choose and enforce role ownership, e.g. clear singleton roles when
duplicating, reject duplicate roles on creation/import, and reconcile mirrors on
deletion. Alternatively make block presentations follow block IDs consistently
and define legacy column projection. Test distinct copy edits and existing live slides.

### P2 — Library Undo changes an archived source to active

**PR #5, already merged; static trace.**
`src/features/library/LibraryView.tsx`, lines 158–170. Trash remembers only ID
and title, and Undo unconditionally calls `librarySetState(id, "active")`.
Trashing an archived source and choosing Undo therefore removes its archived
state. In an Archived-only list it also disappears after Undo.

**Fix:** retain and restore the source's previous state for this undo operation;
add an archived → Trash → Undo regression. This changes organization, not source
content or citations. No new schema is needed.

## PR #5 independent assessment

The change is primarily a Library navigation/organization and provenance UI
improvement. Search now consistently applies source-type, language and archive
filters to segment FTS; names/authors use `library_manage::list`. Existing stored
collections, tags and archive state are exposed in the source details, and the
command checks the managed original file without exposing its path.

No SQL migration, citation model, project package format, presentation generator,
Teaching storage or backup/restore code is changed by these two commits.
Metadata is read from existing tables; the command checks file existence using
the existing verified path helper. No source-reference integrity regression was
identified in this reviewed delta. Existing citation tests were adjusted for the
expanded Rust input struct; that adjustment alone is not new citation coverage.

All six locale files receive the new keys. Mechanical parity validation passed;
this does not establish native-speaker translation quality or Windows visual
acceptance. The list keeps search and ordinary browsing separate, with advanced
management still in Settings; avoid expanding this into another management UI.
The Undo defect above should be fixed on the Windows lineage, without re-merging
the already squash-merged feature branch.

## PR #6 independent assessment

Migration 0025 is additive: two preparation columns, a block table and three
nullable block-link columns. It leaves `study_id NOT NULL` and adds no standalone
Teaching. Existing sessions remain on the legacy canvas when they have neither
blocks nor version-2 template metadata. That is the correct separation for Stage 2.
The completion commit adds view choice, template saving/duplication and guidance;
the continuous/outline switch uses the same stored blocks.

Teaching remains distinct from Bible Study data: it references findings rather
than moving them. `passage_add` validates point ownership, but accepts `block_id`
without checking that the block exists and belongs to the same preparation
(`db/teaching.rs`, lines 635–668). Migration 0025's link columns also have no SQL
foreign keys. This is an enforcement gap at the command boundary; ordinary UI
calls pass the current block. Add block-ownership validation and lifecycle tests
before expanding this API in Stage 3; no cross-project leak through normal UI was
demonstrated in this review.

`portability/project.rs` includes blocks and remaps preparation, parent, point
and block references. Presentation source IDs are remapped through the import ID
maps. Its new round-trip test checks block identity metadata/body, not a complete
presentation/handout/privacy round trip. Backup stores the full database, so it
does not require a separate block-table export list; Rust backup/restore execution
was not repeated here. Presentations and handouts need the fixes above before
claiming output correctness. Ordinary private prose notes were excluded by the
executed existing frontend tests; the Scripture visibility case was absent from
that evidence.

All six `studies.json` dictionaries contain the added keys, and parity checks
passed. Usability still needs Windows acceptance at narrow widths, keyboard
navigation and template editing. The view toggle mainly changes writing-area
size; it should not be presented as a new content hierarchy. Do not add further
workflow controls until the output/data correctness fixes are in place.

## Integration risks and proposed order — approval required

1. Keep `b3120f5…` as the Windows-compatible starting point. It already includes
   Library/Sources. Never merge it blindly onto the divergent `main` history.
2. Apply the Library Undo fix on that lineage in a separate coding task.
3. Fix and revalidate Teaching's block output/model defects. Review PR #6 into
   its intended template parent branch, preserving the handover documentation.
4. Integrate the corrected complete Teaching stream into a temporary integration
   branch, then integrate Study simplification with explicit semantic conflict
   review and full legacy data/Teaching tests.
5. Recognition `ac4d7f4…` is already included through the baseline. Do not merge
   it again. Fetch and review any subsequently published recognition work before
   adding it as the last stream; no unknown future commit is approved here.

If useful after approval, create **`integration/windows-library-teaching-study-review`**
from **`b3120f54b0dcece6c078727c97240828680dd63c`**. Proposed historical merge sequence:
`feature/teaching-three-templates@66ae2df…`, then
`cursor/teaching-stage-2-completion-9953@756d009…`, then
`feature/study-flow-and-teaching-simplification@199f4ad…`. Corrective commits must
be pinned and substituted/added in the approved plan before execution. Do not
create this branch or perform its merges as part of this review.

PR #5 and complete Teaching overlap in `src/lib/api.ts` and two native-polish
documents. Teaching and Study simplification overlap in `TeachingView.tsx`,
`TeachingTemplateEditor.tsx`, all six `studies.json` files, the native-polish
documents and four Teaching/baseline UI test files. Library and simplification
overlap in `tests/support/memoryApi.ts` plus those documents. Resolve behavior,
not just conflict markers: template version selection, legacy editor fallback,
study-to-teaching entry points, translations and in-memory API parity all matter.
No trial merge was performed or assumed conflict-free.

## Migration compatibility

`main` assigns migration **10** to `section_status_and_fields`; the Windows
lineage assigns **10** to `presentations` and **24** to
`section_status_and_fields`. Those files are not interchangeable even when their
features sound similar. The restored runner validates recorded version/name
history and detects certain pre-existing column collisions before upgrading.
It deliberately refuses incompatible histories instead of guessing a repair.
This review did not create a converter for databases from divergent `main`.

Byte comparisons established that baseline, current Windows branch, PR #5 and
Study simplification share exactly migrations 0001–0024; complete Teaching has
the same bytes plus 0025. Recognition has the same 0001–0023 and is ancestral.
`main` shares 0001–0009 and then diverges. SHA-256 inventory follows below.
Do not edit, renumber or regenerate historical files. A Stage 3 migration might
be 0026 with today's history, but **reserve no number until the final integrated
catalogue has been fetched and checked**.

## Stage 3 readiness and implementation plan — design only

The existing design is in [PR #6's description](https://github.com/pistio/abidingword/pull/6),
section “Stage 3 standalone Teaching”. It calls for required project ownership,
nullable same-project study linkage, several teachings per study, project-level
navigation and survival when a study is deleted. These are suitable requirements;
the current design is not yet migration-ready.

**Required design corrections, demonstrated with disposable in-memory SQLite:**

- A naive copy → drop old parent → rename replacement with foreign keys ON
  cascade-deletes dependent rows. A minimal populated parent/child reproduction
  left zero children, even though the replacement kept the parent's ID.
- `PRAGMA foreign_keys=OFF` inside an active transaction is ineffective (readback
  stayed 1). The current runner starts a transaction before executing SQL in
  `db/migrations.rs::apply`. Putting the pragma in a new SQL file cannot solve it.
- The suggested `CHECK (... EXISTS (SELECT ...))` is invalid SQLite: subqueries
  are prohibited in CHECK constraints. Use validated commands plus appropriate
  triggers or an explicitly designed relational constraint instead.

Plan the implementation as follows, after owner approval:

1. Inventory every inbound foreign key with `PRAGMA foreign_key_list`, every
   index/trigger/view and the presentation source-ID links at the final schema.
   Include points, blocks, passages, finding links, materials, presentations and
   presentation items; do not assume indirect descendants are unaffected.
2. Extend migration orchestration for this explicitly identified rebuild: verify
   backup first; disable foreign keys **before** beginning the exclusive migration
   transaction on the migration connection; assert the setting; create the new
   parent and copy every column and ID; drop/rename/recreate indexes/triggers;
   run `foreign_key_check` and preservation assertions before committing; re-enable
   foreign keys outside the transaction on success and failure. Preserve rollback
   and history validation. Do not rename the old parent first and accidentally
   retarget inbound foreign keys. Do not use deferred foreign keys as a substitute
   for preventing DROP-triggered cascades.
3. Backfill project ownership for **all retained rows**, including soft-deleted
   teachings, not just live ones. Refuse unresolved ownership; preserve tombstones,
   revisions, timestamps, template fields, positions and every dependent ID.
4. Use `project_id NOT NULL REFERENCES projects(id) ON DELETE CASCADE` and nullable
   `study_id ... ON DELETE SET NULL`; enforce same-project links on creation,
   relinking and study project moves. Keep study linkage non-unique. Index project
   and study lists. Project ownership survives unlinking.
5. Audit application soft-delete/Trash/restore paths as well as physical DELETE.
   `ON DELETE SET NULL` alone does not run when an application merely updates
   `deleted_at`. Define and test immediate unlinking, restoration behavior and
   project deletion. Define what happens to live finding references when their
   study is deleted or a teaching changes study; preserve source attribution and
   avoid silently pointing a teaching at a different study's findings.
6. Update Rust/TypeScript domain and command types, project-scoped lists, optional
   study assistant context, same-project link picker and project-level Teaching
   navigation. Preserve existing study-local entry points and multiple teachings.
7. Select package Teaching rows by project ownership, remap required project and
   optional study IDs, preserve preparation IDs through the import maps and test
   presentation/block/citation links. Plan schema/package-version compatibility:
   older packages need project ownership backfilled; older apps cannot safely
   consume the new schema. Recheck project removal counts and preview counts,
   which currently derive teaching membership through studies.
8. Test generated version-19/24/25 databases with populated legacy and block
   teachings, every dependent table and tombstones; verify exact counts/IDs,
   integrity/FKs, rollback after injected failure, backup/restore, repeat startup,
   multiple teachings per study, study hard/soft deletion, standalone project
   export/import, presentation links and private output. Validate all six languages
   and perform Windows manual acceptance before considering integration/release.

**Recommended next implementation task:** a focused Stage 2 correctness PR on
the Windows-compatible lineage covering participant Scripture visibility,
manuscript headings and singleton role/duplicate/delete semantics, with native
and frontend regressions. Fix Library Undo separately or as an explicitly scoped
follow-up. Do not start Stage 3 until those fixes and the rebuild protocol are approved.

## Validation performed in this review

Exact PR trees were extracted into `/tmp/aw-review/pr5` and `pr6` with `git archive`;
these are disposable validation copies, not integration branches/worktrees.
`npm ci` installed the frozen lockfile. Both PR lockfiles are byte-identical to
the initial checkout lockfile. No real database was opened.

- Both heads: TypeScript typecheck, ESLint and six-language parity passed.
  PR #5: 3356 English keys; PR #6: 3414; six languages × nine areas each.
- Both heads: frontend production build passed (bundle-size warning only).
- Both heads: all historical SQL files applied in sequence to an empty Python
  SQLite in-memory database with foreign keys enabled; integrity returned `ok`,
  `foreign_key_check` returned no rows. This checks SQL execution, not the Rust
  migration runner, populated Windows upgrades or backup behavior.
- Two isolated frontend reproductions confirmed the Scripture privacy and
  manuscript-heading defects using actual PR #6 functions. They assert the
  observed bad behavior, so their passing is evidence of defects, not fixes.
- SQLite rebuild, transaction pragma and invalid CHECK reproductions executed
  as described above. No proposed Stage 3 migration was applied.
- Frontend full-suite outcomes and focused reruns are recorded below.

### Fresh frontend suite results and failure diagnosis

| Exact source | Full suite in archived source | Focused reruns at exact Git checkout |
| --- | --- | --- |
| PR #5 `9d3d92a…` | 94 files: 91 passed, 3 failed; 1092 tests passed, 3 failed | Two script files: all 30 tests passed. Failed Teaching assertion: 1 passed, 54 intentionally skipped by name filter. |
| PR #6 `756d009…` | 94 files: 92 passed, 2 failed; 1097 tests passed, 2 failed | Two script files: all 30 tests passed. |

Both initial suite runs failed `build-provenance.test.ts` and the updater CLI
subprocess test. The first explicitly returned `spawnSync git EPERM`; the second
lacked expected child-process stderr. The source archives also lacked Git metadata,
which provenance requires. Rerunning in the actual checkout, at each exact head,
with supported subprocess permission passed every test in those two files. These
are local fixture checks; they do not publish an updater or contact a paid service.

PR #5's other failure was the floating-AI-panel Teaching test: expected `Abide.`
in Conclusion, observed `Abid` during two concurrent full suites. The single-test
rerun without that contention passed. The relevant TeachingField, useAutosave and
Teaching test files are unchanged between baseline and PR #5. This is a transient
UI/autosave-test failure; one passing rerun does not prove the underlying timing
behavior impossible. Do not attribute it to the Library delta or call the original
full-suite run green. PR #6 did not exhibit that failure. React `act` warnings
and a PDF standard-font warning also appeared in the full-suite logs.

The two review-specific function reproductions ran separately: one test file,
two tests passed confirming bad output. They were created only in the disposable
PR #6 archive after the full suite had begun; the full-suite counts above are the
repository's original 1099 tests, not these added reproductions. No assertions were
disabled and no historical test files were changed. No complete full-suite rerun
was performed after the focused diagnoses.

Previously reported evidence is separate: PR #5 reports 94 files / 1095 frontend
tests and native fmt/clippy/tests passing; PR #6 reports 94 files / 1099 frontend
tests and native checks. Those are author reports, not fresh native results.
Rust is absent from PATH in this instance, so Rust fmt/clippy/tests, native GUI,
populated Rust migration/backup/portability tests and Windows acceptance were
not rerun. No installed desktop application, paid service or installer was started.
Frontend passes and author reports do not cover the confirmed missing cases.

## Reproducibility and remaining limits

The report and evidence do not certify the five-repository cloud environment as
fully onboarded or validate a fresh-task restore. Node 24.19.0/npm 11.9.0 and
locked frontend dependencies were available/installed; Rust/Tauri system setup
and desktop startup remain outstanding. No install/start configuration was saved
because a complete tested desktop setup was not established during this review.
No other repository was changed. Review and implementation remain separate.

## Full fetched remote branch inventory

Branches below are advertised heads, not evidence that each is active or reviewed.

| Remote branch | Full HEAD | SQL migration files |
| --- | --- | --- |
| `origin` | `a7bc798407dc3bd60598ef5991e7c04e8d4356f1` | 10 |
| `chore/auto-windows-test-updates` | `b296c7739ac628bf573ab74bbf00494ea37b10c9` | 4 |
| `chore/github-ci` | `8dd83fcb0fcbc3f24c51c935640464d000ccce1c` | 4 |
| `chore/verify-via-api` | `516226c6159f8432355b48097e5dc595b25be06a` | 4 |
| `chore/verify-wait-longer` | `78d9125bdf628cc771b7279a3d06da187844f7cd` | 4 |
| `claude/admiring-thompson-j2n6k9` | `38c345f017d081c7b06d2233b2f03f9e915f42ae` | 9 |
| `claude/beautiful-curie-vm0eve` | `aa8fdd92bf8e2492f1e5ee50a4610d59b5bddcbc` | 22 |
| `claude/scripture-integrity-cross-format-4lo6i8` | `ac4d7f47a59679a4ffd7300e185cbd90c34748de` | 23 |
| `cursor/teaching-stage-2-completion-9953` | `756d0097a07cf7105ce7c12313c0a380503b707e` | 25 |
| `feature/bible-study-templates` | `fe57696ecea3eccf12d980e8ca0f84ac4b868418` | 10 |
| `feature/concordance-v0.12` | `1dbb42db9f313a9f24ca13f107951952852bfc10` | 5 |
| `feature/formatted-epub-v0.8` | `9940a079bf3e95ed12f5fca153e2d4b6c5035e60` | 4 |
| `feature/i18n-foundation` | `fbc218d394cb8f530bf6724e1220f6efc062ae36` | 4 |
| `feature/library-sources-workflow` | `9d3d92a6ac24ce1cc69a155a31fcb05fb1233c8d` | 24 |
| `feature/library-v0.4` | `61a901374d48227223bb630bebacacc71cb3bd92` | 4 |
| `feature/passage-aware-sources-v0.9` | `c32f941a736c9235d4f53eb771a46251088a5dce` | 5 |
| `feature/scripture-annotations-v0.5` | `7ef3c5fc8067ced546cae364c469bfc5df78417b` | 4 |
| `feature/scripture-links-v0.11` | `99118d526eb6c2bba79d6f3cac23b1a537b8e772` | 5 |
| `feature/source-navigation-v0.7` | `21d2f9747230d52ccaa77f915ac1cc52bc081b5c` | 4 |
| `feature/study-flow-and-teaching-simplification` | `199f4ad958a0c946cf82df990edeeb51bfd95267` | 24 |
| `feature/study-workspace-refinement-v0.10` | `6b86af30f55ad509b62f9210ea9fc7880a21a3e6` | 5 |
| `feature/study-workspace-ux-v0.6` | `66ee623c7a3492df33125b74e114434d7a71c314` | 4 |
| `feature/teaching-three-templates` | `66ae2df6dd9a86958e323ffbc704514df51d1bcd` | 25 |
| `fix/v0.11-contents-focus` | `9e7bc1612e28ba8f7366a3f88150c4de2651a0c2` | 5 |
| `fix/v0.11-german-malachi` | `a012271008d67dcacf5873173185dca6a0d81c16` | 5 |
| `fix/v0.9-more-locations` | `bf415ff3344a348b9cdd8f9fa1f503b47d4f7e56` | 5 |
| `fix/v0.9-source-relevance` | `7be97cd617544bc25b1574111cc1cfbac977c7e5` | 5 |
| `fix/windows-dev44-schema-compatibility` | `b3120f54b0dcece6c078727c97240828680dd63c` | 24 |
| `main` | `a7bc798407dc3bd60598ef5991e7c04e8d4356f1` | 10 |

## Exact ordered migration byte inventory

For baseline, current Windows, PR #5 and Study simplification use rows 0001–0024;
for PR #6 and its template parent use all rows; for recognition use 0001–0023.
These are SHA-256 digests of committed SQL bytes, not migration runner checksums.

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

`main` uses the same 0001–0009 bytes and instead ends with:

| File | SHA-256 |
| --- | --- |
| `0010_section_status_and_fields.sql` | `0b7d7c9ba293e573d8b417f8e92f9e3e0ce762f373d8c407d765cd4c91846366` |

The first report-only push was rejected by GitHub email privacy protection. Its
local commit is retained on `review/codex-integration-2026-10-28`; no remote history
was rewritten. The publishable report branch uses a Codex noreply commit identity.
