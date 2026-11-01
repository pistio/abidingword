# A.3 Stage 1 validation

Baseline: `c4d045f4c63586a0a351509a050adc458259394c` (`fix/windows-milestone-a2`).
Review branch: `fix/windows-milestone-a3-home`; intended PR base: `fix/windows-milestone-a2`.

PRs #20, #18 and #17 were read through their public GitHub pages and remain drafts on the documented stack. Issues #21 and #22 were read; only #21 is implemented here. API access remains subject to the active managed network policy. Delivery SHAs and actual PR result are recorded after validation/push.

## Development findings excluded from final passing totals

- Old tests assumed a single named Project was flattened, all Studies appeared at root, empty views immediately displayed the form, and a Study always returned directly to Studies. Tests now explicitly navigate the file/folder hierarchy. The old last-used-Project selection rule and organiser component were removed, with new tests for fixed root ownership and named folders.
- Deleting the last Study initially unmounted its list and immediate Undo. The list now stays mounted; the lifecycle regression verifies Undo remains reachable.
- Changing interface language initially reloaded root ownership and unmounted the active Study/presentation. Root identity loading now stays stable across language changes. Existing presentation/guidance language tests and a same-editor/pending-writing regression cover this.
- Imported/example Project count assertions now include the implicit root container created by home initialisation. Import cancellation, native errors, preview and idempotence assertions remain in place.
- The localisation corpus was regenerated after all six packs changed. Earlier stale-corpus failures are not final results.
- Archived baseline screenshots initially could not serve fonts from the shared dependency directory. The baseline-only harness allow-list was corrected and every comparison was recaptured. Baseline application code remains the exact A.2 source.

## Verification scope

Native tests use disposable data and include atomic move rollback, same-workspace validation, stable Study and Teaching identities, active/trashed Teaching, standalone preservation, findings/citations, blocks, presentations, root marker and full backup/restore. The complete native suite retains migration, backup, schema-26 and historical import regressions. All 26 historical migration hashes match the independent Milestone A manifest. No migration was added or changed.

The complete frontend suite retains private export, optional Study linking, existing source/reader, historical/custom method and writing-mode regressions. New home acceptance tests use the actual root route, cover six languages, named folder counts, root/folder separation, creation, moves, rename, deletion/Undo and legacy settings management. Existing editor fixtures enter their named folder through actual UI actions.

The screenshot script passed full-workspace checks at 640/900/1100/1440 pixels and DPR 1/1.25/1.5/2, including owner-shaped and mixed-folder profiles, German/Korean layouts, move with retained writing, safe confirmation and exact legacy manuscript access. This is Linux Chromium evidence, not Windows acceptance or OS scaling proof.

Final command results, source inventory verification and exact implementation SHA will be appended once the frozen runs finish. No paid Actions, merge, release/update publication, NiPogi access or real Windows profile access is authorised by this patch.

## Final frontend and static results

- Complete frontend: **106 files, 1,181 tests passed**, no failed tests, `npm test -- --maxWorkers=2`.
- TypeScript (`tsc --noEmit`, including production build), ESLint and Vite production build: passed. The existing large-chunk advisory remains.
- Six-language localisation: passed, nine areas, 3,785 English keys and 131 components.
- Regenerated corpus: 3,784 strings / 3,925 plural-inclusive keys, 250 editorial audit findings. These are not a native-language approval verdict.
- Final complete-workspace browser script: passed, including corrected baseline fonts, the owner's one-implicit-Project shape, mixed folders/root documents, German/Korean labels, seven width/DPR cases and long unbroken folder names without horizontal overflow.
- Breadcrumb accessibility: 44-pixel minimum touch target retained. The earlier complete run's sole remaining failure was this fixed static accessibility check; it is excluded from the passing total.

The final backend root-initialisation recheck and concurrency fixture were added before native validation began. The frontend source inventory remained unchanged. Native ownership fixtures also assert the actual restored settings marker before allowing name-based fallback, so the backup test proves persistence rather than accidental re-adoption.

## Final native and compatibility results

- Complete Rust: **1,041 passed, zero failed, eight existing ignored**, 87 result summaries including doctests; `cargo test --manifest-path src-tauri/Cargo.toml --locked --no-fail-fast -- --test-threads=1`.
- Rust formatting, all-target locked Clippy with warnings denied, and locked native development build: passed. Existing indexing timing limits were unchanged and passed.
- Native Linux startup: passed with a fresh task-only profile, schema 26, one root marker and one implicit Project, integrity `ok`, no foreign-key violations. [Startup evidence](images/native-linux-startup.png) and [database check](checks/native-startup.txt).
- Existing A.1 and A.2 complete-workspace browser scripts: passed after explicit folder-entry/menu-label adaptations. Reading/Original, real invented PDF/EPUB, Contents, positions, splitter geometry, exact reopen, method restoration and lifecycle checks remain exercised. Their images are retained locally under `/workspace/.aw-tools/milestone-a3/previous-a1` and `previous-a2`; the committed A.3 comparison images are the owner review packet.
- The 1,019-file source inventory stayed unchanged except the two standalone browser scripts' necessary entry adaptations after Vitest completed. Those scripts were separately executed against current application code. Native/application/Vitest sources did not drift during accepted validation.
- All 26 historical migration files and the recognition implementation are unchanged. No schema change.

Captured check text normalises ANSI escapes and trailing whitespace. Failed development runs are not counted in final totals. Windows WebView2 and actual OS scaling still require owner review. No Windows installer or release was produced here.
