# Teaching templates handover

Status of `feature/teaching-three-templates` after the three block-based templates, plus where the Library work went. Stage 3 (a teaching that is not attached to a study) has not been started. This document does not merge branches or publish a build.

Checked 28 October 2026, 20:52–20:56 GMT. The working tree was clean before this file was added. Nothing uncommitted was discarded. No secrets, credentials, private source documents, local databases, or build artefacts are included.

## Branch and commits

| Line | Branch | Commit | What it contains |
| --- | --- | --- | --- |
| Teaching templates | `feature/teaching-three-templates` | `4e41542067b5635bfab611467bcdbc9cb5873599` | Three block templates, migration 0025, block editor, handout and slide filtering. Parent is `0551f22061b3b3219315930c5061be501e94f350`. |
| This handover | `feature/teaching-three-templates` | `05332bf64b758d618e4056ddee2432d4a74ee399` | This document. Parent is `4e41542`. Resume from the branch tip if a later commit only records this hash. |
| Library Stage 2 | `feature/library-sources-workflow` | `9d3d92a6ac24ce1cc69a155a31fcb05fb1233c8d` (parent `6adf1d3`) | Collections, tags, archive state, paged library, grouped search, reader provenance. |
| Windows compatibility line | `fix/windows-dev44-schema-compatibility` | `b3120f54b0dcece6c078727c97240828680dd63c` | Library Stage 2, merged from pull request [#5](https://github.com/pistio/abidingword/pull/5) at 20:48 GMT on 28 October 2026. It does not contain migration 0025. |

Both feature branches diverge from `0551f22`. Neither contains the other’s commits. Do not rebase one onto the other. A later integration has to be a real merge that keeps Library Stage 2 and migration 0025.

Links:

- Teaching branch: https://github.com/pistio/abidingword/tree/feature/teaching-three-templates
- Implementation commit: https://github.com/pistio/abidingword/commit/4e41542067b5635bfab611467bcdbc9cb5873599
- Library branch: https://github.com/pistio/abidingword/tree/feature/library-sources-workflow
- Windows compatibility branch: https://github.com/pistio/abidingword/tree/fix/windows-dev44-schema-compatibility

Do not merge this branch into `main`. A review pull request against `fix/windows-dev44-schema-compatibility` is a place to read the diff. Integrating the two lines later has to be a merge that keeps Library Stage 2 and migration 0025. The histories diverged at `0551f22`.

## What is implemented

Commit `4e41542` (“Add three block-based Teaching templates”) is the whole implementation. A new teaching, still opened from a study, can start from:

- Bible Study (EVA), key `eva`, legacy kind `bible_study`
- Sermon/Talk, key `sermon-talk`, legacy kind `sermon`
- Children and youth Bible lesson, key `bible-lesson`, legacy kind `lesson`
- A blank block teaching, key `blank`, kind `other`
- One of the person’s own templates. Version 2 copies blocks. An older template still starts the point editor.

The three built-in structures live in `resources/teaching-templates.json` as `version: 2`. Titles come from the six locale files at creation time. The teaching copies the blocks and is not linked back to the template.

`src/features/teaching/BlockCanvas.tsx` edits a teaching whose `templateVersion` is at least 2, or which already has blocks (`usesBlocks` in `src/features/teaching/teachingTemplates.ts`). Sections hold one level of child blocks. The person can add, edit, reorder, duplicate, and remove blocks, and can mark a block as for participants.

Block types: `section`, `writing`, `scripture`, `teaching_notes`, `discussion_questions`, `explanation`, `illustration`, `activity`, `application`, `objectives`, `time`, `presentation_notes`, `handout`, `outline_point`.

Settings can create and edit a version-2 template as blocks (`src/features/settings/TeachingTemplateEditor.tsx`). A version-1 template still edits points.

Roles such as introduction and conclusion are mirrored onto the existing teaching columns and points so older presentation code can still follow them (`src-tauri/src/db/teaching_blocks.rs`).

## What is unfinished

- Stage 3, below, is not started. `study_id` is still required.
- “Save as my template” from a teaching (`src/features/teaching/SaveTeachingTemplate.tsx`) still saves points through `templateFromTeaching`. It does not save the block list. A block teaching saved that way can come back as an older point template and open the point editor. Building a block template in Settings is a separate path and does store blocks.
- This branch and `fix/windows-dev44-schema-compatibility` have not been combined.
- No pull request, no merge, no installer, and no desktop walkthrough of the new editor in this wrap-up.
- Full `npm test`, ESLint, Clippy, and the rest of the Rust suite were not re-run here.

## Database migrations

Added: `src-tauri/migrations/0025_teaching_blocks.sql`, registered in `src-tauri/src/db/migrations.rs`.

It adds nullable `template_key` and `template_version` on `teaching_preparations`, the `teaching_blocks` table, and nullable `block_id` on `teaching_passages`, `teaching_finding_links`, and `teaching_materials`.

Migrations `0001` through `0024` are unchanged from `0551f22`. `git diff 0551f22..4e41542 -- src-tauri/migrations` shows only the new file.

`study_id` on `teaching_preparations` stays required. The migration comment says a teaching with no study can be added later without reshaping `teaching_blocks`.

`windows_dev44_compatibility.rs` now expects a fresh database and an upgraded schema-19 database to reach version 25, and expects the preserved rows to survive. The byte-for-byte pin in that file still covers migrations 1–23 only (`development_migrations_one_through_twenty_three_are_byte_identical`). Migrations 24 and 25 are outside that hash list. Migration 24 was already outside it before this commit.

## Tests

Toolchain note: the default Rust on this machine is 1.83.0. `cargo test` with it failed immediately because current crates.io packages need Cargo edition 2024 (`idna_adapter` v1.2.2). The Rust results below used `rustup run stable` (rustc and cargo 1.99.1).

### Passed in this wrap-up

Frontend:

- `npx tsc --noEmit` — passed.
- `npm run i18n:check` — passed. 6 languages, 9 areas, 3406 English keys, 123 components, all consistent.
- Vitest, 28 October 2026, 20:52 GMT: `teaching-blocks`, `teaching-formats`, `teaching-templates`, `teaching`, `baselines`, `build-provenance`. 6 files, 90 tests, all passed.
- Vitest, 28 October 2026, 20:55 GMT: `materials`, `material-scripture`, `presentation`, and `teaching-blocks` again. 4 files, 72 tests, all passed.

Rust, stable 1.99.1, all passed except one pre-existing ignore:

| Test target | Result |
| --- | --- |
| `migration_v25` | 1 passed |
| `portability` | 11 passed |
| `presentations` | 25 passed, 1 ignored (`write_examples`) |
| `teaching` | 6 passed |
| `teaching_blocks` | 2 passed |
| `teaching_templates` | 3 passed |
| `windows_dev44_compatibility` | 7 passed |
| `material_docx` | 8 passed |
| `material_scripture` | 5 passed |

### Failed

No test in the runs above failed.

### Not run

- `npm test` (the full Vitest suite)
- `npm run lint` / ESLint
- `cargo clippy`
- `cargo test` for the whole crate (only the targets listed above)
- A browser or desktop pass of the new teaching form, block editor, handout, or slides
- Installers, release builds, or an upgrade of a real installed database

Library Stage 2 was already merged before this wrap-up. Its earlier test run is not repeated here. Do not treat this document as a fresh verification of that merge.

## Known defects and compatibility risks

- The two development lines are split. Shipping either branch alone drops the other feature. Teaching-only builds lack the Library Stage 2 interface. The Windows compatibility branch lacks migration 0025, so it cannot open a database that has already applied 0025 (`A database written by a newer version of the app is refused`).
- “Save as my template” from the teaching information panel does not copy blocks. See unfinished work above.
- Rust 1.83 cannot resolve the current lockfile. Use stable 1.99.1 or newer.
- `presentations` `write_examples` is ignored. That ignore was already in the suite; this wrap-up did not investigate it.
- Hash pinning still stops at migration 23. A later edit to migration 24 or 25 would not be caught by `development_migrations_one_through_twenty_three_are_byte_identical`.

## Legacy data

Migration 0025 does not rewrite existing teachings. A schema-24 teaching gains the new columns and no block rows. `migration_v25::a_schema_24_teaching_is_kept_and_gains_no_blocks` passed.

`usesBlocks` is false when `templateVersion` is absent and `blocks` is empty, so those teachings stay on the point editor. The Windows compatibility test upgrades a synthetic schema-19 database to version 25 and checks that the seeded study, teaching, presentation, and template rows remain. That test passed. No installed user database was opened.

Project export includes `teaching_blocks` and the new `block_id` columns (`src-tauri/src/portability/project.rs`). The portability suite passed, including the checks that a package is data-only and that a newer backup is refused.

## Presentation and export

Proposed slides for a block teaching come from `propose_blocks` in `src-tauri/src/db/presentations.rs`. A block is proposed only when it is marked for participants. `teaching_notes` and `presentation_notes` are not turned into slides. Introduction, conclusion, and central-message roles still point at the preparation fields older slides already follow.

Handouts in `src/features/materials/derive.ts`:

- Manuscript includes a block that is not for participants only when notes are requested (`includeNotes`).
- Question sheets show non-participant blocks on the leader’s version. The group sheet keeps participant-facing discussion questions and handout text.
- Worksheets skip blocks that are not for participants.

The materials and presentation tests listed above passed. They were not walked in the running app.

## Localisation

The template and block strings were added in `src/i18n/locales/{en,de,fr,es,pt,ko}/studies.json` in `4e41542`. `i18n:check` passed on this branch, so the six packs agree on keys.

`docs/i18n/native-polish-priority-v0.19.md` and `docs/i18n/native-polish-review-v0.19.md` were edited in the same commit. This wrap-up did not repeat a native-speaker review. Key parity is not a review of German, French, Spanish, Portuguese, or Korean wording.

## Stage 3 — a teaching without a study

Not started. Leave it for the next piece of work, after the branch split above is resolved.

What the code requires today:

- `teaching_preparations.study_id` is still `NOT NULL`.
- `teaching::list` and `teaching::create` load the study and refuse a missing one (`src-tauri/src/db/teaching.rs`).
- The new-teaching form is given a `studyId` and always sends it (`NewTeaching` in `src/features/teaching/TeachingView.tsx`).
- Findings linked to a teaching are compared with the teaching’s study.

Migration 0025 already says this can be added later without reshaping `teaching_blocks`. A Stage 3 change would relax `study_id`, allow create and list without a study, and decide what happens to study findings, scripture links, and project export when there is no study. It must not rewrite migrations 0001–0025. Existing teachings keep their study.

## How to resume

1. Check out `feature/teaching-three-templates` and confirm the tip is this handover on top of `4e41542`.
2. Fetch `fix/windows-dev44-schema-compatibility` and confirm it is still `b3120f5` or a descendant that contains Library Stage 2 and not a rewritten migration 0025.
3. Integrate with a merge. Keep both the Library commits and `0025_teaching_blocks.sql`. Do not rebase away either side, and do not edit migrations 0001–0024.
4. Build Rust with stable 1.99.1 or newer (`rustup run stable cargo …`). Cargo 1.83 fails on edition 2024 crates.
5. Before calling the combined branch ready, run `npm run check` (typecheck, ESLint, i18n, Vitest, Clippy, full Rust tests) and open the new teaching form, a legacy teaching, a handout, and a proposed slide deck in the app.
6. Fix “Save as my template” so a block teaching stores its blocks at version 2, or hide that action until it does.
7. Only then consider Stage 3. Do not merge to `main`, tag a release, or publish installers as part of resuming this handover.
