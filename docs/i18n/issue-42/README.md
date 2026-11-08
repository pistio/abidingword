# Issue #42 — complete AI editorial review for owner assessment

PR #44 remains a **draft**, stacked on PR #43 (`codex/owner-followup-35-33`, `38c785be723f4ff4ca7ba8c885aeabff3a89df7a`). The continuation baseline is `64dadc0d1bb263362aa384951612e9395d705725`. Issue #42 remains open. No human native-speaker approval, owner acceptance or real Windows acceptance is claimed.

The six separate AI language reviewers read **every baseline locale string**, including formerly unchanged and legacy keys, in all nine namespaces. They recorded retain/edit/protected/question decisions individually, then reviewed 30 new display keys per language discovered by the source audit. Independent consistency passes checked all edits plus retained safety/method contexts; the final 180 additions also received a separate six-language consistency review. Automated inventory/ledger validation verifies that evidence matches the files; it is not presented as the linguistic review itself.

## Complete coverage and decision evidence

The final corpus has **4,045 logical English strings / 4,186 English leaves including plural forms**. The 4,015 baseline logical strings were all included, not only the previous 269 edited keys. All **25,396 current locale leaves** have explicit decisions and before/after evidence. This continuation makes **564 existing-leaf edits and 180 new display-label additions**; retain/protected decisions cover everything else, including unresolved questions left unchanged.

| Language | Current leaves | Existing edits | Added leaves | First-pass report and individual decisions | Distinct consistency reviewer |
| --- | ---: | ---: | ---: | --- | --- |
| en-GB | 4,186 | 70 | 30 | [English](review-en.md) · [ledger](review-en-decisions.json) | [German reviewer](review-de-en-consistency.md) |
| de, du | 4,186 | 75 | 30 | [German](review-de.md) · [ledger](review-de-decisions.json) | [French reviewer](review-fr-second-de.md) |
| ko, respectful | 4,046 | 46 | 30 | [Korean](review-ko.md) · [ledger](review-ko-decisions.json) | [Portuguese reviewer](review-pt-second-ko.md) |
| fr, vous | 4,326 | 166 | 30 | [French](review-fr.md) · [ledger](review-fr-decisions.json) | [Spanish reviewer](review-es-second-fr.md) |
| es, tú | 4,326 | 131 | 30 | [Spanish](review-es.md) · [ledger](review-es-decisions.json) | [Korean reviewer](review-ko-second-es.md) |
| pt-BR, você | 4,326 | 76 | 30 | [Brazilian Portuguese](review-pt.md) · [ledger](review-pt-decisions.json) | [Coordinating reviewer](review-root-second-pt.md) |

Each ledger contains key/path, feature context, exact baseline/result text, disposition and reason. New keys have explicit addition decisions. Counts include every plural branch, not just a representative form. [Final additions review](review-fr-final-additions.md) documents locator, format, additional-book, PowerPoint and Word labels in all six languages. The complete side-by-side corpus and automated terminology flags remain in [the generated corpus](../native-polish-review-v0.19.md) and [priority report](../native-polish-priority-v0.19.md). Their remaining heuristic flags are explained by the individual decisions; they are not human-review verdicts.

## Feature and source-context matrix

| Priority / area | Completed AI editorial scope and concrete result | Remaining assessment boundary |
| --- | --- | --- |
| P0 Errors/native dialogs/accessibility | All errors and import warnings, dynamic parameters, plural-only/unknown-code handling, labels/ARIA/keyboard announcements, native picker titles/filters. German timeout no longer exposes English. | OS-owned dialog chrome, assistive-technology software and Windows WebView2 require platform assessment. |
| P0 Sources/PDF/EPUB/DOCX | All locale copy, diagnostics, contents/TOC, source annotations, progress/cancellation/retry and generated evidence; original/reading surfaces and hardcoded paths traced. | Real MacArthur 400-page processing, persistence and explicit Apg 6 target remain #35 owner gates; synthetic results do not establish them. |
| P0 Scripture/import/canon | Complete book/reference/morphology/edition/import-warning wording; separate code-based additional-book labels and known format parameters localised. Greek/Latin distinctions and uncertainty retained. | Imported Scripture, actual source/edition copyright/licence data, canon identities and recogniser preserved verbatim; no new canon/theological approval. |
| P0 Study/POZETEK/Ten-Step | All templates, sections/fields, findings, Q&A, repair/Undo, help and writing flows read. Seven POZETEK sections and subject/statement/theme Kern triplet preserved. Spanish Kern edits only change usted to established tú. | Accepted Ten-Step wording unchanged; two genuine French syntax occurrences are separately flagged below. Original unpublished guide/workshop not directly available. |
| P0 Teaching/Materials/privacy | Complete EVA/sermon-talk/lesson, leader/participant text, template builders, transfer/Q&A and privacy labels read; no added theology or new method summary. | Primary owner material, live AI response and actual external Word/PowerPoint rendering are separate assessment contexts. |
| P1 Exports/presentations | Generated manuscript/sheets/Markdown/Word/PDF/PowerPoint labels and error paths traced. British/Brazilian proofing tags, empty Q&A filename, PowerPoint selection/accessibility names and nine custom Word style names fixed. | Built-in Word style identities and package schema/IDs remain invariant; actual audience/projector/Windows/Office acceptance still needed. |
| P0 AI/settings/backup | Every provider/consent/model/image/offline/privacy/settings/destructive/restore string read. Structured AI preview/evidence facts and generated image credits remove English wrappers without rewriting provenance. | No live service requests, billing, API-key use or real restore performed. Local-installation explanatory wording has the limited question below. |
| P1 Common/navigation/legacy | All nine namespaces, deliberate product names, defaults, typography, register, dates/numbers, unused/duplicate/legacy strings individually read. | Sample screenshots demonstrate selected contexts, not every possible rendered state or qualified native-speaker approval. |

[Frontend/source audit](review-frontend-source.md) and [1,786 frontend candidate dispositions](review-frontend-source-decisions.json) distinguish visible copy from CSS/logs/developer-only diagnostics. [Native audit](review-de-native-audit.md) and [1,978 lexical dispositions](review-de-native-source-decisions.json) classify initial native-source candidates, with final display fixes reviewed in their diffs and dedicated tests. [The refreshed inventory](inventory.json) fingerprints 377 final production source files and covers 216 statically named native message codes in all six languages. Its broad regular-expression candidates include SQL/XML/test/protocol false positives; the context audits supersede those candidates. Key mapping alone is not the editorial evidence.

The source fixes use transient display metadata, not changes to stored studies or writing. Source titles, authors, quotations, actual attribution, printed page labels, imported text and stable model/command identities stay exact. [Word mapping evidence](review-pt-word-styles.md) shows the nine distinct custom style names in every language. Owner-defined technical diagnostic JSON retains its requested English evidence/schema while visible verdicts are localised.

## Context screenshots and checks

The final [72 editor/settings cases](images/full-review/checks.json) and [12 Source/Scripture/timeout cases](images/full-review/source-checks.json) provide **96 screenshots** across all six languages at 900px (1.5 device scale) and 1440px (1.0 scale). The synthetic profile exercises Study, Teaching, deletion with Escape/focus return, Settings, AI/privacy, backup, source reading, missing Scripture text and a native timeout carrying the witnessed English fallback. No page error or pane/dialog horizontal overflow was observed in those samples. Hot reload is disabled for the editor capture to keep the sampled copy stable.

| Language | Study / Teaching (900px) | Source / timeout (900px) | AI / backup (900px) |
| --- | --- | --- | --- |
| en-GB | [Study](images/full-review/en-study-900.png) · [Teaching](images/full-review/en-teaching-900.png) | [Source](images/full-review/en-source-900.png) · [timeout](images/full-review/en-timeout-900.png) | [AI](images/full-review/en-ai-settings-900.png) · [backup](images/full-review/en-backup-settings-900.png) |
| de | [Study](images/full-review/de-study-900.png) · [Teaching](images/full-review/de-teaching-900.png) | [Source](images/full-review/de-source-900.png) · [timeout](images/full-review/de-timeout-900.png) | [AI](images/full-review/de-ai-settings-900.png) · [backup](images/full-review/de-backup-settings-900.png) |
| ko | [Study](images/full-review/ko-study-900.png) · [Teaching](images/full-review/ko-teaching-900.png) | [Source](images/full-review/ko-source-900.png) · [timeout](images/full-review/ko-timeout-900.png) | [AI](images/full-review/ko-ai-settings-900.png) · [backup](images/full-review/ko-backup-settings-900.png) |
| fr | [Study](images/full-review/fr-study-900.png) · [Teaching](images/full-review/fr-teaching-900.png) | [Source](images/full-review/fr-source-900.png) · [timeout](images/full-review/fr-timeout-900.png) | [AI](images/full-review/fr-ai-settings-900.png) · [backup](images/full-review/fr-backup-settings-900.png) |
| es | [Study](images/full-review/es-study-900.png) · [Teaching](images/full-review/es-teaching-900.png) | [Source](images/full-review/es-source-900.png) · [timeout](images/full-review/es-timeout-900.png) | [AI](images/full-review/es-ai-settings-900.png) · [backup](images/full-review/es-backup-settings-900.png) |
| pt-BR | [Study](images/full-review/pt-study-900.png) · [Teaching](images/full-review/pt-teaching-900.png) | [Source](images/full-review/pt-source-900.png) · [timeout](images/full-review/pt-timeout-900.png) | [AI](images/full-review/pt-ai-settings-900.png) · [backup](images/full-review/pt-backup-settings-900.png) |

These are **Linux Chromium with a generated PDF and invented memory profile**, not Windows WebView2/high-DPI, actual owner files, native screen-reader operation, real OS dialogs, a live DeepSeek Korean response or Windows acceptance. The installed Scripture fixture has no imported text; its missing-text warning is intentional. The earlier [before/after timeout evidence](images/browser-checks.json) and [parent Kern evidence](../../implementation/owner-followup/images/browser-checks.json) remain historical comparisons.

## Owner questions retained without invention

1. **Protected French Ten-Step syntax:** both the legacy context-field help and current step-04 guidance contain “Que précède et suit ce passage ?”. Suggested genuine linguistic correction: “Qu’est-ce qui précède et suit ce passage ?”. Both remain unchanged and individually flagged; no accepted method wording was silently altered.
2. **Template sample quotation provenance:** “Abiding is not a technique but a friendship.” / “A pastor” was retained. Confirm whether this is deliberately invented sample copy or a source-backed quotation before changing its attribution or theological wording.
3. **Local-installation explanatory scope:** the local-provider installation help says text never leaves the computer. The actual provider help and consent path correctly evaluate the current address (verified in source); the separate installation explanation could be made explicitly about the default loopback setup when a person later changes Address. Existing wording is retained pending that contextual policy decision; no new privacy promise is introduced.
4. **Unavailable primary owner source:** the private YCC/Campus Connect/EVA workshop and unpublished original method guide were not directly available. Existing owner-provided/accepted text and issue excerpts were preserved; this review does not invent source access or certify new methodological fidelity. Apg 6 in English diagnostic help remains the owner’s explicit acceptance query, deliberately retained rather than treated as an English leak.

Qualified human native-speaker proofreading and owner assessment have not occurred. These questions and platform gaps remain explicit; the individual AI review and recorded verification gates are complete and ready for owner assessment.

## Verification and preservation

Final verification results and resolved earlier failures are recorded in [verification.json](verification.json), with complete frontend case results and native logs. **1,289 frontend tests passed (120 files); 1,103 Rust tests passed, with 8 existing ignored tests.** Typecheck, ESLint, locale/corpus/individual-ledger/preservation/query checks, Rust fmt, all-target locked Clippy, frontend production build and Linux locked release compilation passed. The release compile took 6m10s; the frontend retains its existing large-chunk warning. No earlier partial result substitutes for these final results.

Reproduction (Node >=20.19, stable Rust with rustfmt/Clippy and the usual Tauri Linux development libraries):

```sh
npm ci
node scripts/i18n-editorial-inventory.mjs --inventory-only
npm run i18n:corpus
node scripts/test-editorial-review.mjs
python3 scripts/test-editorial-contract.py
python3 scripts/test-pdf-index-query.py
npm run typecheck
npm run lint
npm run i18n:check
npm test -- --maxWorkers=1
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --locked
cargo build --manifest-path src-tauri/Cargo.toml --release --locked
```

Browser reproduction uses the screenshot Vite harness on port 1432 with `SCREENSHOT_DATA` pointing to the generated `pdf_structure` fixture (`AW_PDF_REVIEW_DIR`), then `tests/screenshots/editorial-full.mjs` and `editorial-sources.mjs` with Playwright/Chromium. The test fixture generator and all UI profiles are synthetic. Workspace-local XDG directories are needed in a read-only-home Cloud sandbox for Tauri mock capability tests; this does not alter app capabilities.

The preservation gate keeps all baseline keys, plural branches, placeholder names, accepted Ten-Step text, Bible-name registry bytes, resources, schema 27, historical migrations, reserved 0028, recogniser/importers, stored manuscripts and user source material intact. Its narrow native exception list covers only transient display facts, generated export accessibility/style labels and the exact generated fixture/optional-format test adaptation. No workflow/NSIS/updater change, paid GitHub Actions, installer, merge, release, deployment or live AI request was performed. Feature commits carry `[skip ci]`; the draft remains based on PR #43.

Historical first-tranche edit reports `en.md`, `de.md`, `ko.md`, `fr.md`, `es.md`, `pt.md` are retained for the prior PR history. The `review-*` reports/ledgers above supersede their incomplete-review status for this continuation.
