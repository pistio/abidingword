# Main application continuation — writing and Presenter focus

20 November 2026. Environment: Codex Cloud / GitHub; Windows remains the native acceptance platform. This work does not deploy to NiPogi.

## Outcome and exact candidate

Closing an information panel, saving or cancelling a Study note, and leaving Presenter panels now preserve a newer keyboard-focus choice. Typing remains in the chosen Study/Teaching field or passage input. Normal Escape/Cancel return still works when no newer target has been selected.

- Published application commit: `13b65b74903b7a02e1c8c0f6ef4a5a92c1814e54`.
- Tested local application commit: `7c3a8c178aa753ac1c0d0f5af704521285c897bf`.
- Exact shared application tree: `50c15740b989c478900d86a31d606956fc3b9f93`.
- Base: `249a701e9193bc0257021ca32f8f02bbffda7285`; existing draft [PR54](https://github.com/pistio/abidingword/pull/54).
- [Complete verification](focus-restoration-20261120-verification.json).

The earlier Settings source editor, multilingual source hierarchy, global PDF structure recovery, explicit passage navigation and whole-source scope recommendations are retained in this candidate. The existing Windows ZIP remains the older `bcf4858` application and contains neither the later Settings/language work nor this focus correction.

## Demonstrated problem

An unconditional animation-frame callback returned focus to a button after the person had already clicked a different field. The writing itself was never received by that field. The synthetic tests distinguish this from a storage failure by inspecting visible text, actual save arguments and stored content.

| Route | Reproduced before its correction | Corrected behavior |
|---|---|---|
| Study information → writing | Outside click or Escape followed by writing lost focus; intended text and writing-save payload were absent. | The selected writer keeps focus and saves exactly what is typed. |
| Teaching information Save → Introduction | Information saved successfully, but the queued return prevented Introduction from receiving subsequent typing. | Information and subsequent authored writing both persist. |
| Reopen information / a Study note | An older close frame returned to the opener after the new draft opened. | A reopened draft retains focus and text. |
| Study note Save → Go to passage | The note and John 15:2 link saved correctly, but the passage input became empty/unfocused and the parser received no request. | The full passage is typed and parsed, the new chapter loads, and the saved finding link still returns to John 15. |
| Presenter panel → Next / another panel | Deferred return changed Enter's target or stole focus from the new Passage field. | The chosen control/input remains the keyboard target. |
| Close audience window / leave and reopen Presenter | A global delayed query could target a newer panel; the parent's delayed Start-button return could steal focus after reopening. | The original close target is retained; the parent restores during the closing render with a focus guard. |

The StudyEditor defect was present byte for byte in baseline `249a701`. A wider test run surfaced the affected path; controlled diagnostics establish the mechanism. The truncated earlier full-run log alone cannot establish its exact event order, and this work does not retrospectively claim to explain every historical Teaching or Presenter failure.

## Implementation boundaries

The shared InfoControl, StudyEditor, PresenterView and PresentationView check focus ownership, cancel obsolete queued returns, and preserve a newer chosen target. Reopening, Study changes and unmounting invalidate the relevant pending return. Information action content is rendered through a small component boundary so its close callback remains an event rather than being treated as a render-time ref access by the React lint rule.

Save APIs/debouncing, reference parsing, presentation key mappings, Scripture text and recognition are unchanged. No native code, migration, dependency, lockfile, locale wording or active workflow changed. Schema 27, source index 6 and PDF analysis 5 remain as recorded in the source/settings follow-up.

## Verification

- **1,374/1,374 frontend tests pass across 126 files**, including ten new tests and one strengthened existing test.
- ESLint, all six interface languages and the production build (including TypeScript checking) pass.
- The non-documentation input manifest remained unchanged throughout final verification: 890 files, SHA-256 `eb3272e8a583248e65dc2a53f03fab83a872b47e190fa3099c20d63657724e8c`.
- The focused information/Teaching run passed 67 tests; the focused Study-note file passed 16; all 48 presentation tests pass in the final complete run.
- No Rust/recognizer/corpus/timing run was repeated for frontend-only changes. The earlier 1,146-pass native evidence remains separate and unchanged.

The exact record retains the intermediate results, including the Study-note failure that led to its correction. Tests use controlled frames and actual data assertions, not added sleeps, relaxed assertions, retries or longer timeout allowances.

## Prepared Windows build — approval still required

[Inert installer recipe](windows-focus-followup-20261120.yml.in) pins application `13b65b74903b7a02e1c8c0f6ef4a5a92c1814e54` and reuses the successful workflow from `ca169b3`, with its verified packaging source `4bddc03`, Node 24.22.1 and Rust 1.99.1. Only application pins and descriptive labels differ from that workflow. The historical schema 27 migration catalogue remains pinned.

The file is under documentation with `.yml.in`; it is not an active workflow. After a new explicit approval for **one** installer build, activate it through the existing `build/abidingword-i35-i42-20261109` branch at `.github/workflows/windows-combined-owner-20261109.yml`, then retain the run's exact source/payload evidence. The previous one-build allowance was already used for the old ZIP. No activation, installer compilation, signing, merge, release or updater publication was performed here.

The expected result is one normal Windows installer ZIP containing the global Source work, Settings editing, multilingual hierarchy and these focus corrections. Native acceptance remains distinct from the successful Linux interface checks.

## Short acceptance in the normal Windows application

1. **Settings → Sources:** use ⋯ → Edit details on an active or archived source, save a title/language correction and confirm the same source remains available.
2. **Source navigation:** retain the already accepted handbook correction; rebuild older analysis once where offered. Check another available source, genuine book/chapter/verse hierarchy and whole-source relevance without invented pages. Changing Scripture must leave the reader in place until Go to current passage is requested.
3. **Study writing:** open information, change its draft, then click a writing field and type immediately. The information draft is discarded as indicated, while authored writing stays in its selected field and survives reopening.
4. **Study note → Scripture:** save a note with its current passage link, immediately enter another passage, and then follow the saved note link back. Confirm both passages and the note remain correct.
5. **Teaching:** save information, immediately write in Introduction, and reopen to confirm both values. Normal Cancel/Escape and reopened draft focus should remain predictable.
6. **Presenter:** close a panel, choose Next and use Space/Enter; open a different panel and type its passage; close the audience screen; leave and reopen Presenter. Keyboard input should act on the selected control.

These checks do not close the broader A.4 matrix: native scaling/accessibility, real Office output, owner-configured DeepSeek, visibility-safe materials and the remaining source/study/teaching observations retain their own evidence requirements.

## Main roadmap and recognition coordination

The objective remains **Word → Understanding → Teaching**: a local-first multilingual study with persistent authored work, exact sources and reusable audience-specific teaching. [The consolidated handover](continuity-20261119.md) preserves the complete known roadmap and the pause on new integration until the simpler Teaching workflow is accepted.

Claude's latest reviewed recognizer remains frozen at `08c6faa`. The 19 November review of `4dd9f8e` left a bounded evaluation-tool follow-up (committed evidence binding, path containment and accurate independent-role setup). The actual withheld set and application integration remain unfinished. No recognition process, research branch or actual Set H material was changed by this main-app continuation.
