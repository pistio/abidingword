# Abiding Word — instructions for Claude sessions

This file holds the standing development-workflow rules. The detail behind
each rule is in `docs/` (start with `docs/updates.md`).

## Windows builds and publication require owner approval

Windows is the primary visual acceptance platform. The consolidation in PR
#63 made `.github/workflows/publish-windows-test-update.yml` **manual only**.
A successful push, pull request, merge or CI run does not authorize a Windows
build, signing, publication or an updater-channel change. Ordinary Linux CI
continues to check source changes.

- Prepare a concrete tested source SHA and the intended build or release for
  owner approval. Respect the scope of an approval already given; do not ask
  again for the same authorized action. An unsigned acceptance installer and
  a signed development update are separate actions.
- Feature branches, pull requests and failed CI runs never publish. Stable
  releases are never automatic. Never expose the signing key, its password or
  `UPDATES_REPO_TOKEN` to anything but the `publish` job's signing / publishing
  steps; the `build` job must stay secret-free.
- The manual *Publish Windows test update* button (`workflow_dispatch`) is
  used only for an owner-authorized publication. A failed publish never
  blocks or reverts `main`.
- Do not change the updater public key, regenerate signing keys, or alter
  existing milestone tags. Do not tag a milestone unless told to.

## Completion checklist after merging to `main`

Every report after a merge must include:

1. the merged `main` SHA;
2. the `main` CI result for that exact SHA;
3. the exact source SHA of any separately authorized Windows build and its
   result, or a clear statement that no Windows build was run;
4. for an authorized publication only, the published dev version (`X.Y.Z-dev.N`);
5. for that publication, whether `test-channel/latest.json` in `jc-universe87/abidingword-updates`
   points to that version
   (`https://github.com/jc-universe87/abidingword-updates/releases/download/test-channel/latest.json`).

If an authorized publish failed, report the reason and repair the problem
within the approved scope. Do not turn a source-only completion task into a
new build or publication. A merge can finish with publication still pending.

## Checks before pushing

- Interface: `npm run typecheck`, `npm run lint`, `npm run i18n:check`,
  `npm test`, `npm run build`.
- Native core (from `src-tauri/`): `cargo fmt -- --check`,
  `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`.
- Rust CI builds without debug information (`CARGO_PROFILE_*_DEBUG=0` in
  `ci.yml`): the runner's disk fills up otherwise.
