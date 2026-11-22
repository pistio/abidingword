# Checks of evaluation candidate A1.5-E3

All run in this environment on the E3 source commit's tree (Rust with CI's
`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`).

| Check | Result |
|---|---|
| `cargo fmt -- --check` (`src-tauri/`) | clean |
| `cargo clippy --all-targets --locked -- -D warnings` | clean |
| `cargo test --locked` | **1 115 passed, 0 failed**, 9 ignored, 83 result groups (E2: 1 110 / 8 / 82; added: `evaluation_references.rs`, 4 tests and 1 opt-in evidence hook, and 1 test in `evaluation_rules.rs`) |
| `python3 -m unittest discover -s tests/evaluation -p 'test_*.py'` | **122 passed** (47 tool tests, 28 E2 tests, 47 E3 tests) |
| `python3 -m unittest discover -s tests/corpus -p 'test_*.py'` | 26 passed |
| `npm ci`, `npm run typecheck`, `npm run lint`, `npm run i18n:check` | pass (6 languages, 2 986 English keys, 121 components) |
| `npm test` | **1 053 passed** in 89 files (as E2; no interface code changed) |
| `npm run build` | pass (`../dist` emptied again afterwards; the harness pin was built with it empty) |
| Rust and Python readers, 13 bundled references | 311 761 verses identical (label, line, text digest, alternate numbers); both reproduce `tests/evaluation/bundled-references.expected.json` in CI |
| Against the E2 reviewer's extractor | 311 757 of 311 761 identical; the 4 others are no-break spaces E3 keeps (`../readers/bundled-crosscheck.json`) |
| The review's 528 mis-spaced verses | 528 of 528 read as the editions print them (both readers, in CI) |
| Both readers on 50 137 mutated inputs | 50 137 identical; 0 silent losses in either (`../readers/fuzz-50k.json`) |
| Rehearsals (seal, run, official score; real binaries) | `../before/rehearsal-e2.json`, `../after/rehearsal-e3.json`: EVALUATION-CANDIDATE.md §6 |
| Real references (generated documents) | `../before/real-references-e2.json`, `../after/real-references-e3.json`: whole BSB 154 → 0 mismatches, SBLGNT 335 → 1 (ACT 28:28, the document's reading) |
| E2 harness rebuilt before any change | `800fb43c…`: identical to E2's pin |
| E3 harness | `b4f51a62…`, built incrementally and again after `cargo clean --release -p abiding-word`: identical |
| Recogniser identity | `git diff 08c6faa <E3> -- src-tauri/src src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/build.rs` empty; `src-tauri/src` tree `8cb224c1e059edc3a7750f4c1ea1649603c0de95`; `recogniser-source.sha256` 166 of 166 |
| History untouched since `7772d0b` | no change to `FREEZE.md`, `candidate-source.sha256`, `SET-H-APPROVAL.md`, `set-h-decision-package.md`, `regression.md`, `data/`, `rehearsal/`, `reviews/`, `known-weaknesses.md`, `evaluation-audit/`, `evaluation-e2/`, `evaluation-e2-review/`, `docs/corpus/`, `protocol-e2.json`, `src-tauri/src/` |
| E2's `evaluation-e2.sha256` | 22 of 33: the 11 others are E2 files E3 changes on purpose |
| `evaluation-e3.sha256` | every file verifies (see the record) |
| `evidence/SHA256SUMS` | every file verifies |
| A builder's flat copy of the tools | `check-set.py` passes a synthetic set, reading the committed `protocol-e3.json` beside it, and notes the gate |
| CI | **not triggered**: workflows run on `main`, `feature/**`, `fix/**`, `chore/**`, tags and pull requests to `main`; this branch matches none and no pull request was opened. Every CI step was run locally above |
| Not run | the 113 disclosed editions (not available); authentic published documents (none available); Windows builds; anything on Collection H |
