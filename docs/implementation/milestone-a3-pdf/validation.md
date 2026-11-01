# Validation record — A.3 Stage 2

All data, PDFs and profiles used here are disposable and invented. Real Windows and the original MacArthur PDF are unavailable. Neither is accepted by these checks.

## Focused regressions

- Actual PDF stage → import → index → Source locations → immutable reader window and original anchors: German embedded outlines and layout fallback; chapter start ahead of earlier incidental references; grounded book fallback.
- Tagged PDF headings (including RoleMap), quotations/lists, notes/artifacts, and a deliberately unaligned heading tag.
- Korean Unicode CMap PDF with multiple columns, book/chapter/verse headings and original coordinates. English headings and orphan numeric headings whose body mentions must not manufacture book context.
- Heading identity travels independently of the segment ID when multiple headings share an unchanged paragraph. Contents, initial Source result, Reading spans and Original targets use it. Separate outline book groups cannot inherit a preceding book accidentally; tied multicolumn destinations are rejected.
- Hash-bound, exact-original-line corrections; invalid/stale anchors refused; empty mappings stay empty; disabled parents cannot resurrect via raw outline titles; Reset/Undo and rebuild preserve text/IDs/locators.
- Full backup/restore retains corrections and Undo history, exact finding citations and highlights. Corrupt optional metadata leaves ordinary Reading usable. Permanent Source removal drops its metadata text.
- Populated schema 26 → 27: old Source/file/segment records and historical migration entries retained, one pre-migration backup, no FK errors; injected failure rolls back all new schema objects/columns.
- Full-reader UI tests exercise corrected Contents precedence over a later PDF.js outline, discoverable explicit build for older imports, correction payload/Undo, unchanged quotation text and narrow layouts.

## Full checks

Final counts and command logs are appended after the frozen runs. The first full frontend run found the expected stale generated localisation corpus and the schema-26-only build-provenance attestation. Both were updated for the new keys and additive schema 27; the original historical catalogue entries remain intact. No test timeout or ranking expectation was relaxed to hide a failure.

The browser harness uses a real invented PDF and actual Rust import/index metadata, with real PDF.js Original rendering. Its adapter is test-only. Screenshots and geometry checks are Linux Chromium evidence, not native Windows scaling evidence. The before Reading comparison is an isolated archive of the completed Stage 1 source with the same invented PDF/text manually opened at the corresponding physical page; it does not reproduce the unavailable MacArthur defect.

The first full Rust run identified missing main-window ACL grants for the four new native commands and the deliberately exact schema table inventory. The commands were added only to the main-window set, and the existing real Tauri capability resolver now tests their refusal to the audience window. The schema inventory explicitly includes the additive metadata table. Those checks and the full Rust suite are rerun; permissions are not broadened to presentation windows.

## Final results

- Full frontend: **1,189 passed / 107 files**. An additional pane-to-full-reader continuity regression was then added with no application frontend change; the focused PDF reader file passed **9/9**. All current frontend tests were exercised across these runs. The later typecheck/build and lint include that test.
- Full locked Rust rerun: **1,054 passed, zero failed, eight existing ignored / 89 result groups**. Includes migrations/backup, citation/highlight retention, Source relevance/performance, actual Tauri main/audience permissions and all new PDF cases. Existing timing thresholds were unchanged.
- Typecheck/build, ESLint, six-language consistency (3,819 English keys / 132 components), Rust formatting and all-target locked Clippy with warnings denied: passed. Vite retains its existing large-chunk advisory; no bundle-size limit was changed.
- Localisation corpus regenerated: 3,818 strings / 3,959 plural-inclusive keys, 251 editorial audit findings; automatic consistency is not native-language editorial approval.
- Browser: seven width/DPR cases at 640/900/1100/1440 and 1/1.25/1.5/2, German/Korean panels, two isolated Stage 1 before comparisons, plus a separate real-PDF.js original-bookmark/Contents check: passed.
- Native Linux: the actual Tauri app started on a disposable schema-27 profile, loaded Document structure through the native API, and opened the invented Acts 6 section in Original at physical page 2 / printed label 101. Profile was copied safely from the Stage 1 schema-26 smoke DB, upgraded/seeded through the real Rust services, with one pre-migration backup, integrity `ok`, no FK violations, unchanged managed PDF hash and retained root marker. Stopped intentionally after screenshots.
- Historical SQL 1–26, recognition and shared layout implementation: unchanged. Functional freeze inventory is retained; later amendments were the strict main-window ACL and corresponding schema/permission tests, a passing continuity regression and an opt-in disposable native smoke fixture.

Exact source SHA is recorded in the delivery handover. No real Windows or MacArthur acceptance has been inferred from these results.
