# Abiding Word — A.4 self-contained execution handover
Owner instruction: correct **all** A.4 interface and functionality defects in a controlled sequence, without silently dropping items. Prepared 2026-11-01 for Claude Code or a repaired Codex Cloud environment. This is a handover, **not** a claim of implementation.

## Verified repository baseline
- Repo: `pistio/abidingword`
- A.3 PDF delivery branch: `feature/windows-milestone-a3-pdf-structure`
- A.3 delivery tip: `3b59c28be44ad3cb2ef76d771c5a0d02bb428950` (tested application source: `15cac960a93f260385c868080ec580d4075e2c5f`).
- PR #23: Stage 1 home `fix/windows-milestone-a3-home` -> A.2; PR #24: Stage 2 PDF -> Stage 1. Both draft/review; do not merge into main.
- Schema **27** already uses `src-tauri/migrations/0027_pdf_structure.sql`. Never alter migrations 1–27 after application. Claude's separate Bible provisional-import plan must take the **next unused migration** (likely 0028) on the correct lineage; no concurrent 0027 file.
- Six languages: EN, DE, KO, FR, ES, PT (`pt` locale). Desktop offline-first, Rust/Tauri2/SQLite and React/TS; Windows owner acceptance primary.
- Do not merge unfinished Claude Scripture-recognition branches (recognition remains NOT READY); no Windows updater, production NiPogi deployment, paid GitHub Actions, release, real user profile access or unapproved API costs.

### Infrastructure fallback
The previous Codex workspace was `/workspace/abidingword` on local branch `work`, clean, but network outbound failed at proxy port 8080. The assistant independently verified the GitHub branch via its GitHub connection. A clean `work` checkout is **not proof** of the correct starting lineage. In any new environment, first check `pwd`, `git status`, `git rev-parse HEAD`, `git cat-file -t 3b59c28be44ad3cb2ef76d771c5a0d02bb428950` and `git branch -avv`. If the A.3 tip is not present locally, require a working authorised GitHub connection before coding. If present but proxy still fails, work and test locally only when possible; do not claim pushed PRs until confirmed remotely. Never bypass managed network policy or expose proxy credentials.

## Owner feedback and work tracking
- Master issue #28: https://github.com/pistio/abidingword/issues/28
- Actual MacArthur PDF failure #22: https://github.com/pistio/abidingword/issues/22
- Writing-first Study **and** Teaching UX #25: https://github.com/pistio/abidingword/issues/25
- Compact configurable Sources toolbar #26: https://github.com/pistio/abidingword/issues/26
- AI writing/DeepSeek Korean error #19: https://github.com/pistio/abidingword/issues/19
- Q&A, safe export, Study -> Teaching transfer #27: https://github.com/pistio/abidingword/issues/27
- Simplified Study/Project home #21: implemented, Windows owner passed; DO NOT regress.

On the owner's A.3 Windows schema-27 build, the homepage and the other A.2 smoke checks passed. **MacArthur Acts 6 still failed**. The Sources reader remained crowded with back/full-view, history, Reading/Original, bookmarks, notes, Search, Contents, Document structure, annotations, verbose PDF-page controls, permanent help and About controls. The editor still showed extra `Angaben`, Bible navigation, `Schritt für Schritt / Fortlaufend schreiben`, `Text / Verstehen / Anwendung`, `Hinweise (optional)`, `Deine Antwort`, explanatory text and repeated management controls instead of starting with a spacious writing field. These are NOT completed. Screenshot shows MacArthur original/Reading on PDF page 3/400, title/front matter, during the failed user journey; that image by itself does not prove which earlier click/target caused it.

## Deliverable A.4.0 — inventory and owner-facing acceptance ledger
First inspect A.3 handovers, code and tests. Create one table with every A.4 requirement and `implemented / automated tested / Windows accepted / blocker`. Keep real-file, AI and UI acceptance distinct. Do not substitute test counts for owner approval. Document exact baseline commit, worktree status, branch and migration history.

## Deliverable A.4.1 — real MacArthur navigation correctness (#22)
**Diagnose before rewriting ranking.** The actual John MacArthur German `Basisinformationen zur Bibel` PDF has 400 physical pages in the screenshot. Owner wants the app to understand study Bible and commentary document hierarchy and prefer Scripture-address headings over incidental body references:
1. Exact verse or passage/range heading.
2. Grounded chapter heading (e.g. Apostelgeschichte 6).
3. Grounded biblical book heading/section (Apostelgeschichte), with clearly broader scope.
4. Explicit body references and cross-references as weaker, honestly labelled mentions.
5. If no trustworthy structure exists, say so and permit reversible user correction; do NOT hallucinate a passage section or conflate references with theological discussion.

Inspect A.3 PDF structure pipeline: native PDF bookmarks/tagged content/layout headings, column order, `pdf_structure`, `generated_contents`, section indexing, Source results, selected heading IDs, Reader anchors and page/printed labels. Old PDF imports require **Dokumentstruktur -> Struktur erstellen** and must actually use rebuilt metadata. Test `Apostelgeschichte`, `Apg`, `Acts`, German chapter-only numbering, book context, running heads, front matter, footnotes, and title/Contents segmentation.

Create a **local exportable, metadata-only diagnostic** for owner: original managed file SHA, size/page count, structure status, outline/tag/layout source, heading count/types and nested addresses, candidate IDs/evidence/rank/physical page/printed page, exact target handed to Reader, and mode/page after click. Avoid copyrighted full PDF text, secrets and private writing in logs. Provide exact steps for owner to capture the ACTUAL MacArthur end-to-end journey. Synthetic fixtures are necessary but cannot close #22. The original PDF need not and should not be committed or uploaded without rights. Do not claim fix until Windows owner verifies correct automatic targeting; separately label manually corrected targeting.

## Deliverable A.4.2 — compact Sources/reader toolbar (#26)
Same adaptive toolbar in pane and full view, predominantly **icons**; Back and mode remain discoverable, Search/Contents and page navigation accessible, with overflow `…` for lower-frequency controls (History, notes, bookmarks, document structure, marks/annotations, full view and details). Allow simple persistent per-device pin/unpin of toolbar items with reset/default, not overly complex draggable UI. Accessible `aria-label`, title on focus/hover, appropriate long-press hints without making long press the sole operation; keyboard and touch 44px targets.
For PDF pages replace multirow "Vorherige PDF-Seite"/"PDF-Seite"/"Zur Seite"/"Nächste..." permanent text with unobtrusive previous/next icons and an interactive `3 / 400` indicator opening jump-to-page. Explanations of physical vs printed page numbers are contextual. Keep About collapsed by default. In 3-pane narrow view the toolbar must not grow into repeated rows that displace text. Keep reading/original history, positions, selections, citations, EPUB, page labels, bookmarks, notes, Contents and full-reader transitions intact.

## Deliverable A.4.3 — writing-first Bible Study and Teaching (#25)
**Do both editors**, not only Study.
Normal main surface, minimal visible chrome:
- Study title, e.g. **Römer 8**, not the organisational Project title.
- Two distinct tabs: **Bibelstudium | Bibellehre**.
- Immediately a section heading, e.g. POZETEK **Personen** or Teaching **Erkennen**.
- A spacious editable writing field, then **Ort** and subsequent sections.
- Continuous writing **default**. One-step / outline mode available from a discreet settings action.
- No repeated `Deine Antwort`, long explanatory hint, section status, phase tabs, extra labels/headers, duplicated participation visibility controls before every field.
- Preserve original POZETEK (DE-only) and Ten-Step exegesis sections, attribution, optional Hinweise, field keys/order, free writing and template restoration. Optional hints under a disclosure/tooltip, not lost. Section actions such as reorder/rename/duplicate/delete/add and optional AI move into one contextual ellipsis/menu.
- Save indicator quiet on success but conspicuous on pending/error. Stable autosave, focus, scroll, selection and drafts through view switching. Never overwrite.
- Teaching privacy must remain truthful: show a subtle but clear indicator for leader-private content; keep participant-facing content safe. Retain independent Study and Teaching data and multiple Teaching sessions per Study; preserve historical standalone manuscripts through their legacy management route.
- All six language packs; 640/900/1100/1440 widths and Windows scaling/keyboard/screen-reader acceptance. Full-workspace before/after screenshots, not just components.

## Deliverable A.4.4 — functional bugs/features (#19, #27)
**DeepSeek #19:** owner configured DeepSeek and tried to correct Korean text in German UI, but received `Dieses Element wurde nicht gefunden. Möglicherweise wurde es gelöscht.`. Trace selected editor block/section references and lifecycle, request preview, chosen provider/model, provider errors, suggestion application and stale target. Do not presume API failure. No secrets in logs, no paid AI calls in CI; mock provider regression plus owner-initiated Windows check. Suggestion never silently replaces original; explicitly approve/reject, avoid applying to wrong/revised content.

**Q&A and transfer #27:** optional repeatable paired Question + Answer writing units in both Study and Teaching; reorder; no imposed short character limit. Questions-only, questions+answers, full export, observing audience/private visibility and biblical edition standing. Explicit **Add from Bible Study** into Teaching for selected findings, observations, passages, Q&A, interpretations, application and source citations. Copy editable snapshots with provenance, never silent sync/overwrite, never leak private notes. Preserve citations, studies, Teaching sessions, history, backups, Trash/restore. Add additive schema changes only if needed, on unique next migration after schema 27 and verified branch lineage.

## Separate integrity backlog — track, do not silently merge
Codex should eventually implement provisional Bible import safety (Bible-level `text_standing`, downstream gate for reading, search, research, AI, export, presentations, revision tracking and optional sampling) on a separate controlled lineage; Claude recognition v0.24.6 is NOT READY for user-verified Scripture. Twelve-verse sampling cannot certify a Bible edition. Additional theological library/copyright catalogue, original language resources, Nextcloud/WebDAV and cloud sync, optional digital bookshop are later projects and not substitutes for A.4 acceptance.

## Delivery discipline
- Implement autonomously in **small, reviewable branches** or commits per workstream; maintain the master acceptance ledger as stages proceed. Prefer to start with MacArthur diagnostic to resolve the key blocked real-file pathway, then reader toolbar and editor simplification; run DeepSeek/Q&A in independent reviewable branches when dependencies allow.
- Before each push, record exact tested SHA and result; run full feasible Rust/frontend/i18n/build/Clippy/lint, migrations/backup/privacy, desktop and real browser checks.
- Do not amend applied migration 0027 or 1–26; do not merge into `main`, publish updater, dispatch paid Actions, access owner production data or deploy to NiPogi.
- If GitHub network blocked: only work offline if the base SHA is locally verified and dependencies are present; commit locally and report **not pushed**, never claim a PR. Do not use unapproved proxy-bypass tricks.
- Deliver all A.4 requirements or clearly evidence-blocked items in a final owner handover. Real MacArthur and Windows owner acceptance remain explicit gates.
