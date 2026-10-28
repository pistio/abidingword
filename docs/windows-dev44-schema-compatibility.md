# Windows dev.44 schema compatibility repair

Execution: Codex cloud, repository `pistio/abidingword`, branch
`fix/windows-dev44-schema-compatibility`. All database tests use invented data
and temporary directories. No Windows installation, NiPogi host, real user
folder, release channel or installer was accessed or changed.

## Source evidence and cause

Inspected source snapshots:

- Common ancestor: `9533acbec47c5799bf4be6522542a2ff2cfd473b`.
- Merged template/main snapshot: `a7bc798407dc3bd60598ef5991e7c04e8d4356f1`
  (merge of the six-template feature; feature tip
  `fe57696ecea3eccf12d980e8ca0f84ac4b868418`).
- Recognition/development snapshot:
  `ac4d7f47a59679a4ffd7300e185cbd90c34748de`.
- Repair branch starting point:
  `296f418d762aaa4699dc0feb2f0383758e6987de`, whose only change from that
  development snapshot was the preliminary schema-19 regression.

The reported dev.43/schema-19 and dev.44/schema-10 results are the owner's
confirmed observations. The actual Windows executables and real database were
not supplied or inspected, so this report does not infer their exact binary
hashes or claim testing on the owner's database.

Both histories have byte-identical SQL for migrations 1–9. They then reuse
version **10 for different changes**:

| History | Version 10 | Subsequent history |
|---|---|---|
| Template/main | `section_status_and_fields`: `study_sections.status`, `study_sections.fields`, `study_blocks.field_key` | Ends at 10 |
| dev.43/development | `presentations`: tables `presentations`, `presentation_items` and their indexes | Continues through 23 |

These version numbers are the application’s `schema_versions` ledger, not
SQLite’s internal `PRAGMA schema_version`. This is a divergent history, not merely an out-of-date version constant.
Changing the supported maximum, relabelling schema rows, copying main's
migration 10 over the development migration, or marking skipped SQL as applied
would be unsafe. Main's older app correctly refuses a database newer than it
supports; removing that refusal would not supply the missing schema or code.

The development lineage has a `study_sections.phase` column from migration 13.
Its version-19 schema has no structured-field/progress columns. The compatible
result must have **both** phase support and the new structured-field support.

Object inventories from executing the historical SQL in disposable SQLite
connections (excluding `sqlite_%`, including FTS tables):

| Schema | Tables | Indexes | Triggers |
|---|---:|---:|---:|
| Main's alternate 10 | 46 | 42 | 18 |
| Development 19 | 62 | 50 | 20 |
| Development 23 | 72 | 61 | 20 |
| Compatible 24 | 72 | 61 | 20 |

## Historical SQL and associated commits

The following lists the last commit affecting each historical SQL file at the
starting development snapshot. The repair preserves every one of these files
byte for byte. Regression tests pin their original SHA-256 values.

| Migration | Associated commit |
|---|---|
| 0001_initial_schema.sql | be98075 Adopt frozen docs and refined brand package; fix lockup and clippy |
| 0002_scripture.sql | da05366 Scripture v0.2 core: canon, references, schema, import, commands |
| 0003_studies.sql | 554fac9 Studies v0.3 core: types, findings, templates, relations, search |
| 0004_library.sql | 7129f06 Library v0.4 core: migration 0004, importers, provenance, search, citations |
| 0005_passage_sections.sql | c3380aa Passage-aware sources: section links, ranking and grouping in the core |
| 0006_source_annotations.sql | c71338f Source annotations core: highlights, notes and bookmarks anchored in stored source text |
| 0007_finding_source_quotes.sql | 3fc6d81 Sources to Study core: migration 0007 and findings made from a source |
| 0008_teaching_preparations.sql | d14e045 Teaching Preparation core and a provider-neutral teaching assistant |
| 0009_teaching_step_kinds.sql | abbfeba Bible Teaching: a canvas for each format, lesson steps with kinds (migration 0009) |
| 0010_presentations.sql | a4b6a30 Presentation & Delivery core: migration 0010, one document pipeline, PowerPoint/PDF/Markdown/package |
| 0011_presentation_target.sql | e0b2e10 Presentation Pareto polish: Overview, blank audience screen, pointer and highlighter, target time, layouts, recovery |
| 0012_presentation_templates.sql | 2d744f4 Church presentation templates: model, storage and one layout for PowerPoint and PDF |
| 0013_study_phases.sql | c0cdc4a v0.18.7 independent review: fix every finding |
| 0014_teaching_templates.sql | 753d613 v0.18.7: user-designed Study and Bible Teaching templates |
| 0015_verse_bridges.sql | 0fe46c0 v0.18.8 independent review: fix every finding |
| 0016_unresolved_citations.sql | ef5983f Hardening: imports not blocked by old rows, tolerant migration 0016, docs, screenshots |
| 0017_word_annotations.sql | 264d100 v0.21 A+B: read USFM word annotations; store and search them per translation |
| 0018_grammar_provenance.sql | 9008ab6 v0.22 D: grammar provenance (fit vs the edition's own word), Robinson and MorphGNT decoders, migration 0018 |
| 0019_bible_documents.sql | b65bed8 v0.23 B/C core: Bibles from PDF, EPUB, Word, Markdown and text |
| 0020_library_organisation.sql | 0b1b411 v0.24 A/B: recoverable deletion and Settings → Sources |
| 0021_bible_passage_locations.sql | b6cb792 v0.24 D/E/G: navigable document Bibles, German book names, canon foundations |
| 0022_import_jobs.sql | 6bb6ed8 v0.24 C/F: batch import with persistent jobs, edition check, original-language catalogue |
| 0023_recognition_corrections.sql | f24a262 Review fixes: doubts anywhere in a verse, around missing numbers and raised numbers against words, dotted references, leftover note numbers; Bibles stay readable by verse with unchecked verses shown in place; corrections undone or agreed with kept right, carried across Bibles, no duplicate-anchor failure; tighter layout guards; apparatus index and CHECKs |

The template feature's conflicting migration 10 arrived in the separate
`994e810` handover lineage. Its original SQL is retained only as a test fixture;
it is **not** added as development migration 10.

## Integration and preservation strategy

1. Keep the repair branch on the compatible development application lineage,
   retaining its presentations, personal teaching templates, study phases,
   portability, word annotations and Bible-document support.
2. Preserve development migrations 1–23, their numbers, names, order and bytes.
3. Append **0024_section_status_and_fields.sql** with the three additive columns
   required by the six-template feature. Old sections get `not_started`, no field
   snapshot, and old blocks get no field key. No old writing is rewritten.
4. Integrate the six current study methods and all six interface locales.
   Preserve later-lineage phase metadata, historical wording aliases,
   owner-approved locale terms, the existing design system and personal-template
   workflows. Saving a structured study as a personal template retains its field
   structure and translation source, not its answers.
5. Keep four legacy definitions loadable; new selection offers six current
   methods. Built-in refresh only updates rows with `is_builtin = 1`. Personal
   template definitions and already-created studies are not replaced.
6. Validate the recorded `(version, name)` history before configuring WAL,
   upgrading or accepting a backup. Missing, renamed or divergent history rows
   cause a refusal. Unexpected structured columns before version 24 also cause
   refusal **before any pending migration or backup is attempted**.
7. Keep transactions and the existing verified pre-migration backup. New history
   rows are inserted only in the same transaction as their actual successful
   SQL. No historical row is deleted, renumbered, renamed or fabricated.

For a normal development schema 19, the actual pending SQL is 20–24: library
organisation, Bible passage locations, persistent import jobs, recognition
corrections/apparatus, then structured study fields. These changes are additive.
The earlier citation-table rebuild in 16 and grammar update in 18 are **not
replayed** on a version-19 database.

The existing startup passage-index maintenance can refresh a source's derived
`passage_index_version` and derived passage links. The preservation regression
separates this existing maintenance from migration: it first compares every old
column value immediately after migration, then tests maintenance/startup and
reopening separately. Source titles, original text, user citations, writing and
personal template definitions remain unchanged.

One recognition-source edit is a behaviour-equivalent pattern-match spelling
required by Clippy 1.99 (`redundant_guards`); no recognition algorithm is changed.
The recognition branch itself is not modified or pushed.

## Regression evidence

`src-tauri/tests/windows_dev44_compatibility.rs` covers:

- Fresh databases: exact history 1–24, presentations and structured fields both
  present, all six study templates installed.
- Historical SQL: SHA-256 pin for every development migration 1–23.
- A populated historical schema 19: every original object remains present and
  every captured original row/column value is preserved by migration. Fixture
  data includes personal templates/studies, notes, citations, source records,
  presentation designs/slides/speaker notes, teaching templates, Bible text,
  word annotations, grammar provenance, documents and verse origins.
- Original history names and timestamps stay identical; actual new rows are
  exactly 20–24. The verified pre-migration backup is still version 19 and
  contains the original rows/history.
- Repeated startup: no further migrations or backups, no template revision or
  timestamp churn, no personal data overwrite.
- Restoring the pre-migration copy at a different path, then upgrading again.
- Full `.awbackup` creation/inspection/restoration: field snapshots, answers,
  progress, phases and managed files survive, with a pre-restore safety copy.
- Main's alternate version-10 database is refused with its original schema and
  history unchanged. An incomplete history is not backfilled. An unrecorded
  pre-24 structured column is refused before migrations 20–24 can run.

The existing migration, rollback, capabilities, project portability,
recognition, structured-template and UI suites remain enabled. Type fixtures
were extended for the added optional template data and section defaults.
The generated six-language review corpus was refreshed; its class-A audit has
zero findings. Other editorial review findings remain visible, not suppressed.

## Windows build source verification

`scripts/build-provenance.mjs` is an offline pre-build check. It requires an
explicit full expected SHA, verifies `git rev-parse HEAD`, refuses tracked
changes with `--require-clean`, checks the complete contiguous 1–24 catalogue,
requires development migration 10 (`presentations`) and migration 24
(`section_status_and_fields`), and records a SHA-256 for every included SQL file.
Migration SQL is pinned to LF in `.gitattributes`, including on Windows.

Both Windows build routes run this check before compilation and retain the JSON
as a source-evidence artifact. The signed development workflow already checks
out `needs.gate.outputs.sha`; the guard compares to that same value. The native
workflow compares its checkout to `github.sha`. No triggers, gate conditions,
signing keys, updater keys or release destinations were changed.

The source guard's regression verifies a matching local SHA, all 24 migration
entries and hashes, the two conflicting-number positions, and refusal of a
wrong SHA. Both edited workflows parse as YAML.

**No Windows executable was built or inspected in this task.** This confirms the
source configuration and its local guard, not a produced Windows binary. An
approved future Windows build must retain the source-evidence JSON, confirm its
commit equals the reviewed repair commit, and verify its binary before any
installer publication or acceptance test.

## Remaining boundaries

- The alternate main/template **schema-10 lineage is deliberately refused**.
  There is no speculative converter, no fake application of development
  migrations, and no history rewrite. Supporting that separate database lineage
  needs a separately reviewed conversion strategy.
- The real schema-19 database has not been read. Compatibility evidence uses the
  pinned historical schema and invented representative data; it is not a claim
  of Windows acceptance on the user's installation.
- GitHub API access returned `Forbidden`. Draft PR text is supplied locally; no
  remote PR was opened. The existing PR-triggered CI must not be started without
  the owner's approval. The repair commit uses `[skip ci]` for any branch push.
- No main merge, release, deployment, installer publication, paid Actions,
  Windows acceptance or real user-data change is part of this repair.

## Final local validation commands and results

From `/workspace/abidingword`, using Node 24.19.0, npm 11.9.0 and Rust 1.99.1.
Native shells first activated the prepared workspace toolchain/libraries and
kept test-created application files away from the home directory:

```sh
. /workspace/.onboarding/env.sh
export XDG_DATA_HOME=/workspace/.onboarding/schema-repair/test-xdg/data
export XDG_CONFIG_HOME=/workspace/.onboarding/schema-repair/test-xdg/config
export XDG_CACHE_HOME=/workspace/.onboarding/schema-repair/test-xdg/cache
```

The toolchain environment uses three build jobs and disables native debug
information, as CI does. Network/child-process permissions were enabled only
through the command runner's supported route where required.

| Exact command | Final result |
|---|---|
| `npm run typecheck` | Passed |
| `npm run lint` | Passed |
| `npm run i18n:check` | Passed: six languages, nine areas, 3,343 English keys, 122 components |
| `npm run i18n:corpus` | Review documents refreshed; zero class-A findings |
| `npm test -- --maxWorkers=3` | 1,089 passed, 93 files; no skipped tests |
| `npm run build` | Passed; existing large-chunk warning remains |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | Passed |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked --offline -- -D warnings` | Passed |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked --offline --no-fail-fast` | 1,005 passed, zero failed, eight pre-existing ignored tests |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked --offline --test portability --test windows_dev44_compatibility` | Passed: 11 portability and seven focused compatibility tests |
| `cargo build --manifest-path src-tauri/Cargo.toml --locked --offline` | Passed: native Linux debug build |
| `git diff --check HEAD` | Passed |

The complete native suite was repeated after the final schema preflight and
backup fixture adjustment; this table reports the final successful run, not an
earlier failed run. Earlier failures were resolved rather than disabled:
legacy count/outline assumptions, later UI/provider interfaces, approved locale
terms/typography, generated corpus freshness, disposable XDG paths for Tauri's
mock permission test, and the intentionally tampered rollback fixture now
reaching the intended restore stage under the stronger preflight.

The eight existing ignored tests were not executed or changed into passing
checks: `document_units_dump`, `write_examples`, `real_document_editions_report`,
`bundled_number_index_cost`, `oshb_corpus_reference_correctness`,
`real_editions_report`, `write_interface_fixture`, and
`a_whole_generated_annotated_bible_is_measured`.

After saving the repair commit, the clean source can be attested without
building or publishing Windows:

```sh
node scripts/build-provenance.mjs \
  --expected-sha "$(git rev-parse HEAD)" --require-clean \
  --out /workspace/.onboarding/schema-repair/final-build-provenance.json
```

The delivered source-evidence JSON records the final full commit SHA and all
24 SQL hashes. It is not a Windows binary attestation. No dependency lockfile or
historical migration 1–23 was changed. The only new migration is 24.
