# Checks run by the independent reviewer

At `40d5d5970953b0546ac77e827b6d7d9d8f7c08d8`, Ubuntu 24.04.5, rustc 1.97.0, cargo 1.97.0, Python 3.13.18, Node v22.23.5, npm 10.9.9.
Rust checks with CI's `CARGO_PROFILE_DEV_DEBUG=0` and `CARGO_PROFILE_TEST_DEBUG=0`, from `src-tauri/`.

| Check | Exit | Result |
|---|---|---|
| `cargo fmt -- --check` | 0 | clean |
| `cargo clippy --all-targets --locked -- -D warnings` | 0 | clean |
| `cargo test --locked` | 0 | 1110 passed, 0 failed, 8 ignored, 82 result groups |
| `python3 -m unittest discover -s tests/evaluation -p 'test_*.py'` | 0 | Ran 74 tests in 43.182s, OK |
| `python3 -m unittest discover -s tests/corpus -p 'test_*.py'` | 0 | Ran 26 tests in 4.825s, OK |
| `npm ci` | 0 | |
| `npm run typecheck` | 0 | |
| `npm run lint` | 0 | |
| `npm run i18n:check` | 0 | i18n check: 6 languages × 9 areas, 2986 English keys, 121 components — all consistent. |
| `npm test` | 0 |  Test Files 89 passed (89);  Tests 1053 passed (1053) |
| `npm run build` | 0 | built |
| CI | — | not triggered |
