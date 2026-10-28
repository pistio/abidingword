# Draft PR: restore the compatible development schema lineage and six study templates [skip ci]

Base candidate: `main` (review the full lineage difference, not just migration 24).
Head: `fix/windows-dev44-schema-compatibility`.
Status: draft text only; GitHub API access is forbidden in this environment.

## Problem and resulting behaviour

The six-template main build stops at an incompatible migration 10 and therefore
refuses the owner's schema-19 database. This repair keeps the complete
compatible development application and its historical migrations 1–23, then
adds the template columns at migration 24. A development schema-19 database
gets a verified backup, executes real migrations 20–24 and keeps its original
objects and data. Fresh databases get the full schema and six current methods.

Divergent, incomplete or unrecorded schemas fail before speculative upgrades;
no migration is falsely marked as applied. Personal templates and studies are
preserved. Windows build routes check their exact expected checkout SHA and
record the complete migration catalogue/hashes before compilation.

## Review scope

This head descends from the later development/recognition lineage, unlike
current main. A PR to main therefore includes that lineage's existing later
application features as well as this repair. It must remain a draft for a full
source/compatibility review. Do not automatically merge or publish it. The
recognition branch itself is unchanged by this task.

See [the detailed validation report](windows-dev44-schema-compatibility.md) for
source SHAs, migration differences, preservation tests, exact local commands,
results and limitations. The separate main schema-10 database lineage remains
explicitly refused pending a reviewed converter.

## Execution restrictions

No Actions run is requested. Do not run a paid workflow, merge, sign, publish,
deploy, start Windows acceptance or touch real user data without explicit owner
approval. The head commit includes `[skip ci]`. Creating a remote PR is deferred
because GitHub API access returned `Forbidden`; do not work around that by
changing credentials or triggering a workflow.
