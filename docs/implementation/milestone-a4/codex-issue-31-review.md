# Codex follow-up: atomic Study → Teaching transfer (#31)

Application source: **`35526df382495f3000e7a2aea7b26de842211b9f`**, on
`codex/issue-31-atomic-study-transfer`, directly descended from the exact A.4
handover `b85b9fc7b6a4dfee02bd1ae07a5a05fe7597f669`.
The initial push was refused by GitHub's private-email protection. Only the two
unpublished Codex commits were reauthored with `Codex <noreply@openai.com>`; the
application tree remains exactly `0808328c3033bd5e3a96654d69f42f5d09b0b9a0`,
which is the tree validated by the full suites recorded below. Transfer and capability
smokes were also run on the final application commit. No source change or privacy
setting change was used to resolve the push rejection.

Claude's branch and draft PR #30 are preserved. This is a technical follow-up,
not Windows owner acceptance or permission to merge/publish/build on paid runners.

## Correction and regression evidence

`add_from_study` now starts one SQLite `IMMEDIATE` transaction before reading the
Teaching or its Study. It validates the live Study/project relationship and all
selected source items inside that transaction. Internal block/passage routines
accept the existing transaction instead of starting nested transactions. They
retain the same privacy, provenance, role reconciliation, positions, compatibility
columns and revision behavior. The response is read before committing; there is
no compensating soft-delete loop and no ignored cleanup failure.

Copies still start leader-only, remain editable independent snapshots, and keep
server-read source text and provenance. No AI or network is involved in transfer.
No migration, application version, updater, recognition lineage or export format
was changed. Migrations 0001–0027 and the original acceptance ledger are unchanged;
0028 remains reserved for the independent provisional Bible-import safeguards.

The existing six transfer/Q&A tests are retained. Two additional regression tests
exercise seven late failures using real SQLite triggers and a deferred foreign-key
constraint, without production failure switches:

- second copied block, after an earlier complete copy;
- third copied block, after a complete Scripture block and passage;
- passage attachment after the Scripture block has been inserted;
- Teaching revision update after passage insertion;
- Study timestamp update after passage insertion;
- decoding the final response;
- commit, after all statements otherwise succeeded.

Every case compares raw rows of **every persisted table**, including tombstones,
JSON origins and privacy, positions, compatibility columns, revisions, timestamps
and index side effects. The mid-operation cases also compare the visible Teaching,
check autocommit was restored, and successfully retry after removing the fault.
Preexisting writing, a private snapshot, a passage, public visibility and deleted
rows are present before the fault. Invented data only.

Both new regression tests **fail on the original A.4 database implementation**
(`response-read` / `second-block`: raw database changed), and all eight transfer
suite tests pass with the correction. The original code was restored immediately
after this controlled comparison; it is not part of the resulting source.

## Combined A.4 review

All five workstreams remain in the original lineage and are retained:

| Workstream | Review |
| --- | --- |
| MacArthur navigation (#22) | Margin/size-aware furniture and Contents-listing guards; metadata-only diagnostic and rebuild advice; synthetic native/UI regressions retained. Original Acts 6 acceptance remains open. |
| Compact reader (#26) | Shared toolbar, overflow/custom pins, page navigation and accessibility contracts retained. |
| Writing-first editors (#25) | Study/Teaching writing surface, autosave/error state, contextual menus, sessions, private marks and legacy route retained. |
| DeepSeek Korean target (#19) | Block IDs/stale-target refusal, explicit preview/apply and scripted-provider regressions retained; no real provider call made. |
| Q&A, exports, transfer (#27) | Q&A data, visibility-safe question/answer exports, snapshot provenance and leader-only default retained; transaction correction above. |

The EPUB Contents test had a global rectangle mock that also made the adaptive
toolbar 100px wide, as wide as each of its fixed controls. It could remove Contents
between pointer-down and click, depending on React's measurement timing. The mock
now applies only to the reading viewport/paragraphs; navigation assertions remain
unchanged. No application reader/editor code was modified in this follow-up.

## Validation

Validation is on Linux with Rust 1.99.1 and Node 24.22.1, using locked dependencies.
| Check | Result |
| --- | --- |
| `cargo test --locked` | **1,078 passed / 0 failed / 8 ignored**; all targets completed; zero doc tests are separate from the executed suites |
| `npm test -- --maxWorkers=2` | **1,215 passed / 112 files** |
| `cargo fmt -- --check` | passed |
| `cargo clippy --all-targets --locked -- -D warnings` | passed |
| `npm run typecheck`, `npm run lint` | passed |
| `npm run i18n:check`, `npm run i18n:corpus` | passed; six-language consistency; corpus reproduced the existing 254 audit findings without tracked changes, not linguistic owner acceptance |
| `npm run build`, `cargo build --locked` | passed; existing Vite bundle-size advisory |
| Complete reusable install script | repeated successfully: verified APT packages, local sysroot, `npm ci`, frontend and native builds; saved `install_script` and `start_skill` to the environment draft |
| Vite functional smoke | `/` and transformed `/src/main.tsx` returned expected application content |
| Windows workflow | `actionlint`, manual-only/pin/permission checks, source and migration attestation passed; Windows execution unrun |

The original full frontend run had 1,214 passes and one Contents fixture failure;
an isolated rerun passed, and the lower-concurrency rerun also reproduced the
failure. The final complete run passes after the scoped rectangle-mock correction.
The first Rust run stopped at the capability test's read-only-home failure; the
final complete run passes with writable XDG paths. The eight preexisting ignored
Rust tests were not enabled or counted as passes.

The fresh cloud execution environment and HTTPS Git fetch worked. GitHub web
access allowed reading Issue #31 and the latest PR #30 owner handover. GitHub API
requests were separately denied by the proxy; `api.github.com` was added to the
configuration draft, which requires review/save/publication in environment
settings before assuming access. No token was requested or copied.

The native build libraries are installed in a local verified Debian sysroot;
Rust, npm cache and XDG directories use writable workspace paths. The Tauri
capability test initially failed on the read-only default home directory and
passes with these XDG settings. This was an environment prerequisite, not a code
permission failure. Reusable setup is outside the repository under
`/workspace/.setup`, with instructions saved to the environment configuration.

## Prepared Windows acceptance workflow — not dispatched

`.github/workflows/windows-a4-acceptance-build.yml` is a separate manual-only
workflow. Its sole source choice and guard pin the application SHA above. Rust
1.99.1, Node 24.22.1 and action commits are pinned. It verifies source cleanliness,
all 27 migration hashes and absence of updater configuration before compiling.
It produces an unsigned NSIS installer, SHA-256 checksum, source provenance and
build-environment artifacts, with read-only repository permission and no signing,
release, updater or deployment steps. The old A.3 acceptance workflow is preserved.

Local `actionlint`, YAML/source-pin checks and source/migration attestation pass.
This does **not** establish a Windows installer build or Windows runtime success.
Only after separate owner approval should the workflow at the reviewed branch
revision be dispatched, with source SHA `35526df382495f3000e7a2aea7b26de842211b9f`.
Record the workflow/run SHA, installer checksum and exact source with the result.

Use a disposable/copied schema-27 profile, retain the complete A.3 backup and its
matching executable, and set `ABIDING_WORD_DATA_DIR` explicitly for acceptance.
Follow the existing [owner handover](../milestone-a4-handover.md) and
[acceptance ledger](acceptance-ledger.md), in order:

1. Original MacArthur Acts 6, rebuilding structure; record automatic and manual targeting separately with the diagnostic.
2. Compact reader, page jump/pins, native scaling and assistive technology.
3. Minimalist Study/Teaching, writing retention, autosave and private marks.
4. Real owner-configured DeepSeek Korean correction with explicit preview/apply.
5. Q&A, question-only/answer/full exports, snapshot transfer/citation provenance and participant privacy.

**Every existing owner gate remains OPEN.** Do not close #19/#22/#25/#26/#27 or
change the ledger to Windows-accepted on the strength of automated tests.
Scripture recognition remains isolated and **NOT READY after Set F**; AI remains
optional and the architecture offline/non-AI-first. No merge to main, release,
automatic updater or NiPogi production deployment is authorized.
