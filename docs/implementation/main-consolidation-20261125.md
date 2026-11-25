# Abiding Word — reconcile historical main into the current application

**25 November 2026. Internal source-control consolidation; not a public release.**

## Inputs and preserved history

- Historical `main`: `ceba01ff62c1bbeffc27982cb71a846788ee3533`.
- Current integrated application PR #61: `c7a464860de4c2696d6e83f09239722ccbe79226`, descended from the tested Windows acceptance source `056333b68e74360225aea6b05cfe896c6ae83220`.
- Common ancestor: `9533acbec47c5799bf4be6522542a2ff2cfd473b`. GitHub reports 42 historical `main` commits not ancestors of PR61 and 348 current-application commits not ancestors of `main`.
- GitHub's normal reconciliation PR #62 reports `mergeable_state=dirty` due to overlapping template/study files. Its history is **not** thrown away. A deliberate two-parent reconciliation records both lineages.

## Reviewed conflict resolution

The 42 main-only commits are the older POZETEK / Ten-Step / six-method Study-template catalogue, its six translations and documentation, plus a historical schema-26 Windows acceptance workflow and voluntary-support README links. Main's `0010_section_status_and_fields.sql` is the same three `ALTER TABLE` statements as the later application's `0024_section_status_and_fields.sql`; the only difference in that SQL file is the numbered comment. **Both migrations must never run**: the current application correctly assigns migration version 10 to presentations and version 24 to Study section fields.

Choose the **current PR61 application tree** as the authoritative result of resolving the older branch's overlapping source changes. It already contains POZETEK/Ten-Step functionality, structured fields, Study progress and six-language Study presentation, plus tests (`src-tauri/tests/pozetek.rs`, `ten_step.rs`, UI tests). **Do not transplant old main's migrations or overwrite later source code.** The pre-consolidation `main` parent remains in Git history for retrieval and audit. No existing user DB is rewritten or migrated by this Git operation.

Preserve historical main's **voluntary support links** in the consolidated `README.md`. Other historical main-only template draft docs and old schema26 workflow are recoverable from `ceba01ff`; do not republish them as current instructions when superseded by later app code.

Because databases created by the historical main schema10 lineage and the later schema28 lineage have **different version-10 identities**, an existing historical database requires an explicit tested migration/recovery plan; never silently reopen or rewrite it. Migration-integrity guards must remain fail-closed. Do not treat this source-control reconciliation as a blanket upgrade guarantee.

## Merge/release isolation

- `.github/workflows/native-builds.yml`: **manual `workflow_dispatch` only**. No Windows or Linux installer on a push to main, feature branches or tags.
- `.github/workflows/publish-windows-test-update.yml`: **manual `workflow_dispatch` only**. Remove the automatic `workflow_run` trigger that otherwise signs/publishes an update after successful main CI. Preserve the separate manual build/sign/publish pathway.
- Retain source CI on PRs and main pushes; GitHub-hosted CI reports are distinct from Windows owner acceptance.
- The independent, successful unsigned Windows installer is GitHub Actions run [45705226399](https://github.com/pistio/abidingword/actions/runs/45705226399), application source `056333b68e`. **Compilation passed; owner-native Scripture smoke is not yet accepted.** Do not trigger another install/build automatically.
- No signing, updater publication, deployment, user data access, release, or main-force-push.
- P0 backup restoration fault tolerance [#57](https://github.com/pistio/abidingword/issues/57) and legacy USFM source spacing [#59](https://github.com/pistio/abidingword/issues/59) remain open. Do not silently incorporate divergent PR #58.

## Required acceptance to promote as main's source baseline

1. The reconciliation commit must list **both parents** (the current application and old main) and use the application-plus-safety tree; verify both branches are now ancestors.
2. Verify historical SQL 0001–0027 identities and migration 0028, six-language resources, scripts and local-only behavior; run full PR CI and inspect any failures before merge. Source-index timing fluctuations require evidence, not dismissal.
3. Review the resulting PR to main; merge by a history-preserving method, **never squash** and never use a force update. No auto updater/native build after merge.
4. Keep PR #53 recognition evaluation and #58 incomplete full-backup work separate from app code. After main carries the application history, archive redundant #56/#61 stacked drafts and retain or delete branch refs only after checking open dependencies.

**Resulting main is a current development baseline, not a signed public stable Windows release.**
