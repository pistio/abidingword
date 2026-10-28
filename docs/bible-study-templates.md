# Bible Study Templates

The new-study catalogue contains six methods for five subjects: Guided Passage
Study (POZETEK only in German), Ten-Step Bible Study, Biblical Character Study,
Biblical Word Study, Whole-Book Bible Study and Topical Bible Study. The two
passage methods share the Passage study subject and are selected in the Template
list. Character and Book studies accept a person's or book's name without
parsing it as a passage. Teaching preparation keeps its existing passage-study
workflow; this feature changes Bible Study only.

The four legacy definitions (`passage-study`, `biblical-theology-study`,
`systematic-theology-study`, `teaching-study`) remain installed and addressable.
They are hidden from the new catalogue, not removed from storage. Saved personal
templates, including legacy types, remain available. Backend callers omitting the
study type get the explicitly selected template's subject; an explicit study type
still takes precedence. Existing persisted studies are never retyped.

## Localisation

All six locales provide names, descriptions, headings, guidance and the passage
methods' structured fields, choices, goals and collapsible examples. Portuguese
uses the existing `pt` folder (Brazilian wording; system `pt-BR` maps to it).
German retains POZETEK and its mnemonic headings. Other languages use descriptive
names and headings, without German words in parentheses. Korean labels follow
the existing 성경공부 / 공부 terminology.

Known template identities disambiguate headings shared between methods.
Structured sections retain their translation source independently of whether the
original personal template still exists. Missing translations fall back to
English; unknown definitions and edited text keep the saved wording. Built-in
optional parts keep canonical stored text so language changes still translate
them. Personal template text remains as written.

## Compatibility

This completion adds no migration and does not edit existing migration files.
The branch's additive migration 0010 adds section progress, field snapshots and
block field keys. Version-9 studies, block text and saved personal templates
survive it unchanged; the automatically created version-9 backup remains readable
and can be restored to another path and upgraded again.

Studies copy their template's structure and structured fields. Refreshing or
removing a template never rewrites the copies. Full study duplication retains
answers, entry data, empty sections and progress; structure-only duplication
starts empty with progress reset. Choice-only entry answers require deletion
confirmation. Section deletion also protects structured entry data even when a
caller has not supplied corresponding display text. Keyless optional parts in
personal templates remain usable.

## Regression coverage

- `tests/ui/template-catalogue.test.tsx`: all six languages, template and field
  text, Character/Book creation and editing, both passage methods, catalogue
  membership, custom templates, optional parts, language switching, reopening,
  search, deleted source templates and fallback.
- `tests/ui/pozetek.test.tsx`: structured answers, choice-only deletion
  confirmation, progress and deferred structured parts.
- `src-tauri/tests/template_catalogue.rs`: all six methods, inferred subject,
  Character/Book filtering, search, customisation, optional parts, full and empty
  copies, template deletion, backup restoration and data-preserving deletion.
- `src-tauri/tests/pozetek.rs`: schema-9 upgrade and automatic backup restore,
  alongside existing structured-snapshot and custom-template checks.
- Existing frontend and Rust suites remain enabled, including Korean terminology,
  studies, database migration, source citations, teaching and updater tests.

See the implementation brief in
[Bible_Study_Template_Catalogue_Implementation.md](Bible_Study_Template_Catalogue_Implementation.md)
for the methodology and scope. Local checks are recorded in the final branch
report; Windows acceptance, CI execution, merge and publication are separate.

## Final local validation

Executed from `/workspace/abidingword` with Node 24.19.0, npm 11.9.0 and
Rust 1.99.1. Native shells first source `/workspace/.onboarding/env.sh`, which
sets the local libraries/compiler wrapper, three build jobs and
`CARGO_PROFILE_DEV_DEBUG=0` / `CARGO_PROFILE_TEST_DEBUG=0`.

| Exact command | Result |
|---|---|
| `npm run typecheck` | Passed |
| `npm run lint` | Passed |
| `npm run i18n:check` | Passed: six languages, eight areas, 1,879 keys, 90 components |
| `npm test -- --maxWorkers=3` | Passed: 720 tests, 60 files; no skipped tests |
| `npm run build` | Passed; bundle-size warning over 500 kB remains |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | Passed |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked --offline -- -D warnings` | Passed |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked --offline` | Passed: 577 tests; zero failed or ignored |
| `cargo build --manifest-path src-tauri/Cargo.toml --locked --offline` | Passed |
| `git diff --check` | Passed |

The final code also passed a Linux desktop smoke check on Xvfb display 100,
with `ABIDING_WORD_DATA_DIR=/workspace/.onboarding/feature-smoke-data` and the
workspace's `run-desktop.sh` helper. The rendered window offers Character Study,
Whole-Book Study and the passage-method selector; native logs report startup
complete. The disposable SQLite database reports integrity `ok`, schema 10 and
10 installed built-ins (six current, four legacy). Existing user data was not
opened or migrated for this check.

Earlier failures were fixed and the affected checks repeated: missing creation
labels, outdated catalogue/outline assertions, Korean terminology, preserving
progress in a full copy with no blocks, and waiting for asynchronous study
navigation in a UI test. The final complete runs above contain no failures.
Windows acceptance, deployments, releases and merges were not run. The push
commit uses `[skip ci]` so GitHub Actions are not started for this change.
