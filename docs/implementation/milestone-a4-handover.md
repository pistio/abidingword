# Abiding Word — Milestone A.4 owner handover

**Status: all five A.4 workstreams implemented and tested on the A.4 branch. Every owner gate (real MacArthur PDF, native Windows UI,
real DeepSeek) remains OPEN. Nothing merged, published or dispatched.**

- Repository `pistio/abidingword`, branch **`claude/friendly-ptolemy-s1s9ji`**, created fresh from the verified A.3 baseline
  `feature/windows-milestone-a3-pdf-structure` @ `3b59c28` (not `main`, not a Scripture-recognition branch).
- Requirement ledger: [milestone-a4/acceptance-ledger.md](milestone-a4/acceptance-ledger.md).
- Schema **27** unchanged: no migration added; historical migrations 0001–0027 byte-identical to the baseline. The next number
  (0028) is left to the provisional Bible-import lineage.
- No paid GitHub Actions run (`claude/**` matches no workflow trigger; commits carry `[skip ci]`), no updater, no NiPogi, no owner data,
  no paid AI calls (providers are scripted in tests).

## Stages (commits on the branch)

| Stage | Commit | What | Detail |
|---|---|---|---|
| A.4.0 + A.4.1 | `3c2029c` | MacArthur Acts 6 root cause reproduced and fixed; metadata-only diagnostic | [diagnosis](milestone-a4/a41-macarthur-diagnosis.md) |
| A.4.2 | `0e06137` | One compact, configurable reader toolbar | [toolbar](milestone-a4/a42-reader-toolbar.md) |
| A.4.3 | `40c6854` | Writing-first Bible Study **and** Bible Teaching | [editors](milestone-a4/a43-writing-first-editors.md) |
| A.4.4a | `d723730` | DeepSeek/Korean "Element nicht gefunden" classified and fixed | [#19](milestone-a4/a44-deepseek-korean-19.md) |
| A.4.4b | final commit | Q&A, safe exports, Add from Bible Study | [#27](milestone-a4/a44-qa-exports-transfer.md) |

## Most important findings

1. **MacArthur (#22)**: A.3 treated *every* line matching a margin running head as furniture, so a handbook's own book title
   ("Apostelgeschichte", also its running head) never became a heading; Acts 6 fell back to front matter (a motto verse or a Contents
   line). Reproduced on invented handbook PDFs and fixed. Whether this is the **actual** cause on the owner's PDF must be confirmed with
   the new local diagnostic (Document structure → Diagnostic report). Old imports need **Rebuild structure**.
2. **#19**: not DeepSeek or Korean. AI help on any block-teaching field failed before any provider call (a block body was sent as a
   "point" part without a point). Fixed, with actionable refusals for stale targets.
3. Two latent defects fixed on the way: a failed structured-field save was reported as "Saved"; deleting a teaching block had no
   confirmation.

## Validation (final)

Rust **1,076 passed / 0 failed / 8 ignored** (A.3: 1,054); frontend **1,215 passed / 112 files** (A.3: 1,189); fmt, Clippy `-D warnings`,
typecheck, lint, six-language i18n check and corpus, production build: all passed. Browser captures (real PDF.js, actual Rust metadata,
invented documents) at 640/900/1100/1440 px and 125/150/200 %, German and Korean; A.3 "before" captures from the baseline.

## Owner acceptance steps (Windows, disposable profile, keep the A.3 backup)

1. **MacArthur**: follow [the steps](milestone-a4/a41-macarthur-diagnosis.md#owner-steps-for-the-actual-macarthur-journey-windows):
   Rebuild structure, open Acts 6, click the result, save the diagnostic JSON, and report automatic vs manually corrected targeting separately.
2. **Toolbar**: check the reader pane and full reader at your usual width and scaling; Customise toolbar…; page jump; Narrator/NVDA.
3. **Editors**: a POZETEK study ("Römer 8") and an EVA teaching: writing first; Study options / part `⋯` / Teaching options discoverable;
   leader-only marks truthful.
4. **DeepSeek**: Korean text in an EVA block → AI-Hilfe → Grammatik und Rechtschreibung prüfen (see #19 doc).
5. **Q&A / exports / transfer**: add Q&A in Study and Teaching; question sheet with and without answers; Add from Bible Study.

Report each as pass/fail with screenshots. Until then: **#22, #25, #26, #19 and #27 stay open for owner acceptance**. A Windows
installer was not built here; the existing schema-27 manual acceptance workflow is pinned to the A.3 SHA and was **not** dispatched.
Building an A.4 review installer needs the owner's explicit instruction (paid runner).
