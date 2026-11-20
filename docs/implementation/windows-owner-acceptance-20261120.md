# Abiding Word — next Windows owner acceptance worksheet

**20 November 2026.** Draft application [PR #54](https://github.com/pistio/abidingword/pull/54). The build candidate is application commit `13b65b74903b7a02e1c8c0f6ef4a5a92c1814e54` (tree `50c15740b989c478900d86a31d606956fc3b9f93`). This worksheet is documentation-only; it does not change that candidate.

The next isolated Windows acceptance build was submitted on the pre-existing build branch at `229a268f2fc803d4cc134eafd4500fb719afeb7a`. **The build's success, run ID, installer checksum, installation and Windows acceptance must be recorded separately; submission proves none of these.** The preceding delivered Windows installer, application `bcf4858`, does not contain the Settings/alias/focus follow-ups.

## Rules for acceptance

- Windows is the primary native acceptance platform. Distinguish **implementation**, **automated test evidence** and **owner-observed Windows acceptance** for every item.
- Use a disposable or backed-up test profile for destructive upgrade/restore operations; never alter the owner's primary data without a separate plan. Avoid including private source contents, provider keys, participant names, copyrighted excerpts or screenshots with sensitive material in evidence.
- The owner already reported the supplied *Basisinformationen zur Bibel* correction working, accepted the source Original scrolling issue #49, confirmed the POZETEK seven-part structure and field resizing, and accepted Ten-Step methodology. Preserve those narrow decisions; they are not blanket acceptance of every document or Windows route.
- Report failing steps with the active version, precise UI route, expected/actual result, whether a fresh or upgraded disposable profile was used, scaling, and optionally a cropped screenshot without private text. Never mark a step passed solely because a Linux/JS test passed.
- Do not merge, release, sign, publish an updater or deploy from this worksheet.

## Native Windows acceptance matrix

Record **PASS / FAIL / NOT TESTED / NOT APPLICABLE** and evidence for each item. Every row starts **NOT TESTED** unless narrow acceptance has already been recorded.

| ID | Area | Owner check and expected result | Native status |
|---|---|---|---|
| W01 | Installation and history | Install the exact candidate on a disposable schema-27 profile; record version and installer checksum; existing backup and migration history remain usable. No unexpected updater or release. | NOT TESTED |
| W02 | Settings → Sources | On an active source, use **⋯ → Edit details**; edit title/language and save. The same source ID, text, annotations and citations remain. Language filter refreshes. Repeat with an archived source. | NOT TESTED |
| W03 | Source navigation | In a fresh unrelated PDF/EPUB with grounded headings, select book, chapter, and verse/range. Navigate only to actual confirmed containing destinations; if none exists, say so instead of guessing. | NOT TESTED |
| W04 | German and multilingual book names | Verify Sprüche/Sprichwörter/Sprueche/Sprichwoerter, Prediger/Kohelet, 1. Mose/Genesis, and an additional available non-German source label. Preserve meaningful ambiguities; no wrong-book jumps. | NOT TESTED |
| W05 | Existing PDFs | On a disposable or backed-up existing source with outdated analysis, choose **Chapters and headings → Rebuild structure** once. Text, segment IDs, annotations and existing manual citations are preserved. The supplied handbook correction is already owner-reported fixed; do not erase that report. | NOT TESTED |
| W06 | Reader position | Change selected Scripture while reading a source: the source remains at its position. **Go to current passage** is explicit; Back/Forward and Reading/Original mode behave predictably. Cross-restart persistence is a separate future requirement, not silently claimed here. | NOT TESTED |
| W07 | Whole-source coverage | Edit details → More details → Set coverage for a work on Romans 8. For Romans 8:28 show **Covers Romans 8** but do not invent a physical page. Open at the saved position; actual mentions remain optional explicit destinations. Test **No specific passage**. | NOT TESTED |
| W08 | Reader toolbar and scaling | Pane/full-reader one-row controls, meaningful overflow, pin/unpin, physical/printed page jump, tooltips/accessibility and no duplicated controls at 100%, 125%, 150% and 200% native scaling where possible. | NOT TESTED |
| W09 | Study writing and focus | Open information/help, click a writing field, type immediately, save and reopen. The exact authored text survives. Escape/Cancel only returns focus when no newer field is selected. | NOT TESTED |
| W10 | Study note ↔ Scripture | Save a Study note linked to John 15:2; immediately type another passage and navigate, then reopen the saved link. Both passages and the note remain correct, with no lost keystrokes. | NOT TESTED |
| W11 | POZETEK | The seven German parts are **Personen, Ort, Zeit, Ereignisse, Textzusammenhang, Erklärungen, Kern**, with optional Application separate. Kern has three compact sequential ideas and one writing field. Confirm writing-first layout and helpful optional hints. | PARTIALLY OWNER ACCEPTED — final native review open |
| W12 | Teaching editor | Start Teaching through a Study using the simplified workflow; save information and immediately type into Introduction. Close/reopen and verify both fields; preserve leader-only visibility, multiple sessions and legacy standalone records. | NOT TESTED |
| W13 | Q&A and export privacy | Test optional question/answer blocks, questions-only / with-answers / full outputs and leader/private content exclusions in a disposable Teaching. Never expose leader-only material to participant output. | NOT TESTED |
| W14 | Study → Teaching transfer | Explicit selection creates an editable, provenance-bearing snapshot; no silent overwrite or unintended auto-sync. Test failure/retry and original Study preservation. No new integration scope without owner review. | NOT TESTED |
| W15 | Presenter | After closing a panel press Next/Space/Enter; enter a passage in a different panel; close the audience window and leave/reopen Presenter. Keyboard acts on selected controls, not a stale opener. | NOT TESTED |
| W16 | Live DeepSeek Korean | Only with the owner's already-configured provider on Windows: preview, edit/apply and stale-target checks; no silent replacement, wrong-field edits or key disclosure. Mocked provider tests do not count as native acceptance. | NOT TESTED |
| W17 | Actual outputs | Inspect DOCX, PDF, PPTX and presentation with real Office/PDF software, including Korean typography, paragraph/verse content, copyright and participant visibility. | NOT TESTED |
| W18 | Safe preservation | On a disposable copy, check backup/restore, Source and Scripture citation references, retained Study/Teaching data and no loss during upgrade/rebuild. Preserve original source files and schema-27 migrations. | NOT TESTED |

## What is already automated

At application source `13b65b7`: 1,374/1,374 frontend tests pass, including focus regression cases; frontend typecheck, lint, six-language checks and production build pass. The preceding Settings/alias native baseline has 1,146 Rust tests passed, none failed, nine ignored, with focused source/PDF tests green. These are **not substitutes for native owner acceptance**. The build submission is not proof of installer success.

## Parallel but separate

Claude's Scripture recognizer remains frozen at `08c6faa` and awaits the independent Collection H evaluation. Its app integration is still open: migration 0028, provisional standing and downstream gating, provenance and printed numbering, fresh verification after corrections, resource-limited PDF worker, progress/cancellation and Windows end-to-end checks. Do not merge the research branch as part of this installer.

The full original vision and deferred roadmap remain in [continuity-20261119.md](continuity-20261119.md): **Word → Understanding → Teaching**, authored study with exact references, reusable methods, memory and faithful audience-specific outputs.
