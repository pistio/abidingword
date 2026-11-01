# Abiding Word — Milestone A.4 acceptance ledger

Master tracking: Issue #28. One row per requirement. **Implemented** and **automated tested** describe this branch;
**Windows accepted** can only be set by the owner. Real-file, AI-provider and UI acceptance are separate gates.
Test counts are evidence of regression safety, not owner approval.

## Baseline (A.4.0)

| Fact | Value |
|---|---|
| Repository | `pistio/abidingword` |
| Development baseline | `feature/windows-milestone-a3-pdf-structure` @ `3b59c28be44ad3cb2ef76d771c5a0d02bb428950` (A.3 tested application source `15cac96`) |
| A.4 branch | `claude/friendly-ptolemy-s1s9ji` (the session's designated branch, created fresh from the baseline; not `main`, not a Scripture-recognition branch) |
| Worktree at start | clean; baseline verified by `git cat-file` and the remote ref |
| Handover read | `handover/abidingword-a4-20261101:docs/implementation/milestone-a4-execution-handover.md` (doc-only commit `b7711c1` on top of the baseline) |
| Migrations | 27 files, 0001–0027; `0027_pdf_structure.sql` unchanged. A.4 adds **no migration** so far. A later need takes the next unused number (0028), coordinated with the separate provisional-Bible-import lineage |
| Environment | Claude Code cloud; GitHub fetch/push working; Linux build deps installed as in `ci.yml`; Rust 1.97, Node 22 |
| Not done, by instruction | no merge to `main`, no paid Actions dispatch, no updater publication, no NiPogi deployment, no owner data access, no paid AI calls |

## Ledger

States: ✅ done · ◐ partial · ☐ not started · ⛔ blocked/owner gate.

| # | Requirement (issue) | Implemented | Automated tested | Windows accepted | Notes / blocker |
|---|---|---|---|---|---|
| A4.0-1 | Baseline, branch, migration lineage recorded (#28) | ✅ | n/a | n/a | This document |
| A4.0-2 | Single requirement ledger, real-file/AI/UI gates distinct (#28) | ✅ | n/a | n/a | Updated per stage |
| A4.1-1 | Diagnose before re-ranking; trace result → reader target (#22) | ✅ | ✅ `macarthur_shape` | ⛔ owner | [Diagnosis](a41-macarthur-diagnosis.md): A.3 furniture rule removed handbook book titles |
| A4.1-2 | Running heads/folios/footnotes excluded without deleting real titles (#22) | ✅ | ✅ | ⛔ owner | Margin-only, size-aware; number-insensitive only at body size |
| A4.1-3 | Contents/title/front-matter segmentation never stands in for a section (#22) | ✅ | ✅ | ⛔ owner | Listing pages (≥3 books) map no passage; listing mentions rank last |
| A4.1-4 | Priority exact > chapter > book > mentions, honestly labelled (#22) | ✅ | ✅ Rust + UI | ⛔ owner | "Book section … (broader)", "Chapter heading …" in 6 languages |
| A4.1-5 | German `Apostelgeschichte`/`Apg`, `Acts`, chapter-only numbering under book context (#22) | ✅ (A.3 dictionaries + A.4 guards) | ✅ | ⛔ owner | Diagnostic accepts all forms |
| A4.1-6 | Old imports use rebuilt metadata; rebuild prompt (#22) | ✅ | ✅ | ⛔ owner | `analysis` revision; A.3 docs still load; notice in panel |
| A4.1-7 | Local exportable metadata-only diagnostic (#22) | ✅ | ✅ privacy assertions | ⛔ owner | Copy / Save .json; includes observed reader mode/page |
| A4.1-8 | No hallucinated section; reversible correction (#22) | ✅ (A.3 corrections retained) | ✅ | ⛔ owner | Verdict "No matching section identified" |
| A4.1-9 | **Real MacArthur PDF end-to-end** (#22) | — | — | ⛔ **OPEN** | Synthetic fixtures cannot close it. Owner steps in the diagnosis doc |
| A4.2-1 | One compact adaptive toolbar for pane and full reader (#26) | ✅ | ✅ UI + browser harness | ⛔ owner | `ReaderToolbar`; [notes](a42-reader-toolbar.md) |
| A4.2-2 | Icon-first, `…` overflow, persistent per-device pin/unpin + reset (#26) | ✅ | ✅ | ⛔ owner | Defaults: Find, Contents |
| A4.2-3 | Accessible names, tooltip on hover/focus, 44 px, no long-press-only (#26) | ✅ | ✅ | ⛔ owner (Narrator/NVDA, touch) | |
| A4.2-4 | PDF prev/next icons + `3 / 400` jump popover; contextual page explanation (#26) | ✅ | ✅ | ⛔ owner | Original keeps viewer's compact control (no duplicate) |
| A4.2-5 | About collapsed by default; no multi-row wrap at narrow widths (#26) | ✅ | ✅ 640–1440, 125–200 % | ⛔ owner (native scaling) | |
| A4.2-6 | Preserve history, modes, selections, citations, EPUB, labels, bookmarks, notes, Contents, full-reader transitions (#26) | ✅ | ✅ full UI suite (1,199) | ⛔ owner | Tests moved to new routes, none deleted |
| A4.3-1 | Study title + two tabs Bibelstudium/Bibellehre (#25) | ✅ | ✅ | ⛔ owner | [notes](a43-writing-first-editors.md) |
| A4.3-2 | Section heading then spacious writing field; continuous default (#25) | ✅ | ✅ | ⛔ owner | |
| A4.3-3 | Remove repeated "Deine Antwort", hints, status, phase tabs, duplicated controls (#25) | ✅ | ✅ | ⛔ owner | Phase outline/headers removed; phase kept per part |
| A4.3-4 | Hints on demand; section actions in one contextual menu; templates/attribution preserved (#25) | ✅ | ✅ | ⛔ owner | Attribution moved to end of study |
| A4.3-5 | Quiet save, conspicuous pending/error; stable autosave/focus/scroll (#25) | ✅ | ✅ | ⛔ owner | Fixed: failed field save was reported as saved |
| A4.3-6 | Teaching: same writing-first surface; truthful leader-private indicator; multiple sessions; legacy route (#25) | ✅ | ✅ | ⛔ owner | Delete now confirms by name |
| A4.3-7 | Six languages; 640/900/1100/1440; scaling/keyboard/screen reader (#25) | ◐ | ✅ strings, widths (browser) | ⛔ owner (scaling, Narrator/NVDA) | |
| A4.4-1 | DeepSeek Korean "Element nicht gefunden": traced, classified, fixed (#19) | ✅ | ✅ mocked DeepSeek (Rust 6 + UI 2) | ⛔ owner + real DeepSeek | [diagnosis](a44-deepseek-korean-19.md): block body sent as point part without point; provider never called |
| A4.4-2 | Suggestions never silently replace; stale target refused (#19) | ✅ | ✅ | ⛔ owner | `assistant.staleTarget`; Study help classified too |
| A4.4-3 | Optional repeatable Q&A blocks in Study and Teaching (#27) | ✅ | ✅ Rust + UI | ⛔ owner | [notes](a44-qa-exports-transfer.md); no migration |
| A4.4-4 | Exports: questions only / with answers / full, visibility-safe (#27) | ✅ | ✅ privacy assertions | ⛔ owner | Teaching materials + Study Markdown export |
| A4.4-5 | Explicit "Add from Bible Study" snapshots with provenance (#27) | ✅ | ✅ | ⛔ owner | Leader-only by default; all or nothing; own study only |
| A4.X-1 | #21 home not regressed | ✅ (untouched) | ✅ suite green | owner passed A.3 | |
| A4.X-2 | Schema 27, data, citations, annotations, privacy preserved | ✅ (0 migration files changed) | ✅ suite green | ⛔ owner | No A.4 migration; 0028 left to the provisional Bible import lineage |

## Stage log

| Stage | Commit | Rust | Frontend | Other checks |
|---|---|---|---|---|
| A.4.1 | `3c2029c` | 1,063 passed / 0 failed / 8 ignored | 1,193 passed / 108 files | fmt, Clippy -D warnings, typecheck, lint, i18n, build |
| A.4.2 | `0e06137` | unchanged code (review export test is opt-in) | 1,199 passed / 109 files | typecheck, lint, i18n, build; browser harness 10/10 scenes |
| A.4.3 | `40c6854` | unchanged | 1,206 passed / 110 files | typecheck, lint, i18n, build; editor captures 4/4 widths + A.3 baseline |
| A.4.4a | `d723730` | 1,070 passed / 0 failed / 8 ignored | 1,208 passed / 111 files | fmt, Clippy -D warnings, typecheck, lint, i18n |
| A.4.4b | see `git log` | 1,076 passed / 0 failed / 8 ignored | 1,215 passed / 112 files | fmt, Clippy -D warnings, typecheck, lint, i18n, build; migrations unchanged |
