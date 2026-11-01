# A.2 validation and delivery

Baseline: `1b61656d4407a7511b3615b2c91c8411d8c1faff`.
Branch: `fix/windows-milestone-a2`; review base: `fix/windows-milestone-a1`.

Tested implementation SHA: `450bd95ef9bedabc248e1746fbbf8585fe19e72a`. The delivery commit adds only review documentation and the pinned build-workflow selection; application and test code remain exactly this snapshot.

## Validation scope

The complete frontend suite includes lifecycle, Teaching privacy/export, optional Study linking, import, reading position/history and previous milestone regressions. The complete native suite includes historical migrations, backup/restore, lifecycle/snapshots, source indexing and integrity regressions. No test time limits were weakened.

All 26 migration files match `milestone-a/migration-sha256.json`; no migrations were added. The multilingual recognition implementation remains unchanged.

The A.2 browser script passed for 640/900/1100/1440 px, DPR 1/1.25/1.5/2, using an invented real PDF and EPUB with full workspace components. The A.1 browser script also passed against current code; its source selector was updated to include the match's accessible location text. Screenshots are in `images/` and `previous-regressions/`.

## Development failures excluded from final evidence

An earlier frontend run contained the obsolete expectation that returning from a Source focuses the section heading. The final behavior restores focus to the live equivalent result button and its test now checks that behavior. Early browser fixture setup accidentally supplied the old harness PDF to the new fixture; this was corrected before accepted screenshots by routing the fixture's own bytes.

The first complete native run, concurrent with other CPU-heavy checks, exceeded the existing `indexing_a_moderately_large_source_in_each_language_is_quick` bound for Spanish (5.044 s stage + 6.282 s commit). This result is retained as a failure, not counted as a passing complete run. Final native verification is repeated without concurrent build/test load; neither production indexing nor the time limit was altered.

The next full frontend run exposed a pre-existing asynchronous Teaching test that queried the linked-Study label before Teaching loaded. Its assertion now waits for the label; preservation checks are unchanged. Review also found and fixed stale descendant Study rows after Project Trash/Undo; the regression confirms immediate row removal and restoration. These changes are included in the final full frontend rerun.

## External acceptance limits

Linux Chromium with emulated DPR does not validate Windows WebView2, file pickers or OS scaling. The actual MacArthur file is absent. Its Acts 6 navigation and genuine discussion ranking remain unaccepted pending the procedure in the owner handover. PDF structural reading is a later independent milestone.

The manual Windows workflow will be pinned to the exact tested application SHA, retaining source/migration checks and unsigned NSIS artifacts. No workflow dispatch, Windows installer build, signing, updater overlay or release publication occurred here. A new installer therefore remains outstanding.

GitHub API requests are rejected by the active managed network policy (`Forbidden`). Public GitHub pages and normal Git transport work. Draft PR creation will be attempted after push; no bypass or credential change is authorized or used.

## Completed static and application checks

- `npm run build`: passed (TypeScript and Vite production build; existing large-chunk advisory remains).
- `npm run lint`: passed.
- `npm run i18n:check`: passed, six languages × nine areas, 3,773 English keys and 130 components.
- `npm run i18n:corpus`: completed, 3,772 strings / 3,913 keys, 249 existing audit findings retained for editorial review. This is consistency validation, not a native-speaker verdict.
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check`: passed.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`: passed.
- Native Linux Tauri dev build/startup: passed in a fresh `/workspace/.aw-tools/milestone-a2/native-data` profile. [Startup screenshot](images/native-linux-startup.png); SQLite integrity `ok`, no foreign-key violations. This is a startup smoke check, not a native Windows workflow test.
- `git diff --check`: passed before delivery.

Final complete frontend: `npm test -- --maxWorkers=2` — **105 files, 1,177 tests passed**, 310.07 s. Reduced worker concurrency does not alter the selected tests or assertions.

Final complete native: `cargo test --manifest-path src-tauri/Cargo.toml --locked --no-fail-fast -- --test-threads=1` — **1038 passed, 0 failed, 8 existing ignored**, 86 result groups including doctests. The previously failing indexing time-limit test passed unchanged. Final Clippy and Rust formatting also passed.

[Captured checks](checks/) are normalised for ANSI escapes and trailing whitespace. Earlier failures are not included in final passing totals.

## Windows review build preparation

The existing manual-only `.github/workflows/windows-acceptance-build.yml` now defaults to implementation SHA `450bd95ef9bedabc248e1746fbbf8585fe19e72a`, with an exact two-SHA guard retaining the previous reviewed schema-26 source. The pinned migration catalogue digest remains `3a2159765b0c77874143ef3143b6b97bd785d26d5372aa9250dae590be30bded`. Clean source provenance verifies schema 26 and all 26 migrations; [captured provenance](build-provenance.json).

The workflow prepares unsigned NSIS review artifacts and checksums without updater/release publication. It has not run. No Windows installer is available from this session. Owner-controlled Windows execution and the original-document acceptance remain required.
