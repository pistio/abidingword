# Study flow and reusable template blocks

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
