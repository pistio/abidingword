# Checks run by the independent E3 reviewer

Checkout `9640aea303c625c137c5cdbe1f4b8cc9738951fe` at `/home/user/abidingword`, fresh container,
rustc 1.97.0, cargo 1.97.0, Python 3.13.18 (Unicode 15.1.0), Node 22.23.5, npm 10.9.9,
webkit2gtk-4.1 2.52.6, Ubuntu 24.04.5 x86-64, 4 CPUs. 2026-11-25.

| Check | Result |
|---|---|
| `cargo fmt -- --check` | clean |
| `cargo clippy --all-targets --locked -- -D warnings` (CI settings: `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`) | clean |
| `cargo test --locked` (CI settings) | **1 115 passed, 0 failed, 9 ignored, 83 result groups** |
| `evaluation_references` test binary alone (the bundled-reference tests) | 4 passed, 1 ignored (`read_texts_for_cross_checking`, a dump helper) |
| `python3 -m unittest discover -s tests/evaluation -p 'test_*.py'` | **122 passed** (115 s) |
| `python3 -m unittest tests.evaluation.test_e3_rules.BundledReferences` | 3 passed (the 13 references, the review's 528, Unicode editions) |
| `python3 -m unittest discover -s tests/corpus -p 'test_*.py'` | **26 passed** |
| `npm ci` | pass |
| `npm run typecheck`, `npm run lint` | pass, pass |
| `npm run i18n:check` | pass: 6 languages x 9 areas, 2 986 English keys, 121 components |
| `npm test` | **1 053 passed in 89 files** |
| `npm run build` | pass (`dist/` emptied again afterwards) |
| Harness binary, pinned path, fresh target | **`b4f51a62...` = the protocol pin** (build/harness-builds.md) |
| Harness binary, another path | `8cab9ddd...` (path-dependent, as stated) |
| E2 harness rebuilt at the pinned path (E2 sources of `7772d0b` checked out temporarily, then restored and verified) | **`800fb43c...` = E2's pin** (fourth reproduction) |
| Identity, checksums, tool and protocol hashes, history | all as recorded (identity/identity-checks.txt) |
| CI on GitHub | not triggered (review branch; no pull request) |

Environment note: this session started in the owner-approved **Collection H**
environment (Limited network: `ebible.org` only; package managers off). The
Rust and npm builds need crates.io, npm and the Ubuntu archive, so the session's
network was moved to a new personal environment, `abidingword review`
(package managers on, `ebible.org`), approved by the owner in the app. **The
Collection H environment itself was not changed.** `ebible.org` was reachable
but deliberately not used (it is Collection H's only approved source).
