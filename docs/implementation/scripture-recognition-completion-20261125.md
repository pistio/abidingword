# Scripture recognition — product completion milestone
**Date:** 25 November 2026. **Status:** WORK IN PROGRESS, not implemented or accepted on this branch.  
**Owner objective:** *Use Abiding Word to study Scripture, retain reliable notes and produce faithful Teaching.* This milestone is delivered only when Scripture recognition works in the **Windows application**, not when an evaluation report passes.

**Implementation issue:** [#60](https://github.com/pistio/abidingword/issues/60). **Product importer defect:** [#59](https://github.com/pistio/abidingword/issues/59). **Separate restore P0:** [#57](https://github.com/pistio/abidingword/issues/57).

## 1. Freeze the completion scope

The first personally usable recognition release must support **EPUB, DOCX, Markdown/TXT and text-layer PDFs**, and honest handling of unsupported or ambiguous cases. No OCR, AI-based reconstruction, handwritten text, unsupported ancient canon/verse numbering, forced user cloud upload or universal recognition claim is required for this milestone.

For every accepted verse, fidelity means *the exact wording of the user's source document under its printed book/chapter/verse*, **not a claim that the source is an authentic Bible edition**. An unverified verse is a source location with a reason, never Bible text. Confirmation of edition identity is a separate owner action.

The point is completing a working vertical path:
1. Select a source file and preview book/chapter/verse findings and issues.
2. See exact original source locations (EPUB entry/spans/exclusions; other formats' place) and text only where verified by source evidence.
3. Make/restore a book-heading correction; rerun all verification gates; see the updated result.
4. Explicitly confirm the edition (or leave provisional), choose to add/keep as document, commit atomically.
5. Reopen after a cold restart. Read by book/chapter, search, navigate to the original, make a Study citation and then a Teaching/Materials output, without guessed or rejected verses.
6. Maintain source identity and status in backup, project transfer and restore; preserve all pre-existing work.

## 2. Exact technical integration location

**Integration starting branch:** `integration/scripture-recognition-completion-20261125` from PR #56 head `dc66257a50aa1b44141c80bacd2afb8062ae9cca` (tested application input `607e965d9c59a1dd93d23aaf95961944c11211a7`). Preserve its later save/deletion/recovery behaviour, Source→Original navigation and Teaching privacy.

**Frozen A1.5 recogniser to integrate:** `08c6faac8c92c6c791ec1e7bbbe997362fa1059f`. Detailed handover: `docs/handover-scripture-recognition-codex.md` at PR #53 / commit `05b3e8b57eb74d02278b30d6b1e5cecc1134764f`. That commit contains `src-tauri/src/scripture/structural.rs`, `provenance.rs` and `numbering.rs`. **Those modules are absent from the main PR #56 application**. Port only the compatible modules and dependencies into the latest app; do not merge frozen branch wholesale or override app changes.

**Existing app commands to complete, not replace:** `src-tauri/src/commands/bibles.rs` (preview, correction, commit), `commands/bible_documents.rs` (re-recognition, add verses), `src-tauri/src/db/bible_documents.rs`, `scripture/import.rs`, existing React Bible import preview/reader. They currently use `import::stage[_documents]` and do not invoke A1.5 `structural::stage_documents`.

**Persistence:** add only new migration **0028**. Existing migration registry in `src-tauri/src/db/migrations.rs` currently ends at 27 (`pdf_structure`). Do not edit migration 0001–0027 or mutate historical SQL bytes. Before migration, the app makes its existing verified safety copy; tests must cover a real pre-27 SQLite database and full backup. Store source and EPUB-entry hashes, per-verse provisional/verified-from-document status, confirmation date/actor, exact provenance/exclusions, reason, per-book numbering standing and engine identity. Preserve old rows and IDs. An existing translation with no such evidence remains **unconfirmed** until explicitly re-recognised; do not invent provenance.

## 3. Five non-negotiable correctness gates

### A. Every correction invalidates the preceding gate
A1.5 `StagedImport::correct` / `apply_saved` re-recognise and overwrite unverified flags. The EPUB `StructuralSource` and gate must survive/redo each pass, followed by fresh `StructuralReport` and `numbering::assess`. **Test both** saved correction and interactive correction with a verse that the gate originally withheld: it must stay unverified; stale old provenance must be impossible to commit.

### B. Every Scripture consumer is filtered
Verified text alone may become Scripture text in `bibles`, chapter/passage fetch, search, concordance, comparison, Study citations, Teaching/Materials/export, and optional assistant context. Unverified text stays inspectable **only at its source**, with reason and place. Edition confirmation and *from-document verification* are separate facts; unconfirmed is visible.

### C. Numbering cannot invent a parallel verse
Keep printed numbers and per-edition mapping. `withinReference` does not prove equivalence. If numbering differs or cannot be proven to agree, disable automatic parallel-verse alignment and make the limitation visible. Extra canon books/refused chapters cannot silently map into a Protestant canonical key.

### D. Untrusted PDFs have process limits
Never perform memory-unbounded PDF extraction in the GUI/Tauri process. Use a memory/time-limited worker (Windows Job Object and Linux RLIMIT_AS), crash/timeout isolation, progress events and actual cancellation. Staging cancellation or refusal changes no database/user file. Image-only scanned PDFs get a clear *OCR unsupported* explanation, not a false verified verse.

### E. App import versus published evaluation are independent
Claude's E3 evaluation and independent review are running separately. Neither the withheld `pistio/collection-h` repository nor sealed scores may be inspected for development. E3's extra reference reader is **evaluation-only**, not a replacement for the frozen recogniser. Product issue #59 (USFM footnote spacing on bundled Bibles) is a real separate app fidelity problem; resolve it through separately reviewed change and non-destructive handling of existing imported data, not by silently rebuilding all old editions.

## 4. Required tests and observations (deliver code, not another plan)

- **Rust actual-engine tests:** EPUB link-role and apparatus gate, provenance byte spans; correction and saved-correction re-gating; known exact/original texts; supported DOCX/text/PDF and honest refusals. Keep source bytes unchanged, including Ukrainian/Korean filenames/Unicode where relevant.
- **SQLite lifecycle:** import→commit→close/reopen, schema 27→28, old document Bible conservative backfill, unchanged study/citation IDs; portable backup and project export/import preserve status/hashes; restore failure safety coordination with issue #57. User data never used in tests.
- **React UI:** six-language preview labels, edition confirmation, unverified/source navigation, excluded verses not clickable as Scripture, cancellation/progress, editor/keyboard focus, assistant/context and export privacy.
- **Native Windows on disposable profile:** verify actual app executable, filesystem and Office DOCX output, source hash, restart, independent profile isolation with process-level `ABIDING_WORD_DATA_DIR` and `WEBVIEW2_USER_DATA_FOLDER`; do not rely on installer identity alone. **Require owner authorisation for a new Windows build**.
- **Core smoke exercise:** invented EPUB with John 3 and linked note → preview (verified/refused) → correct heading → previously gated verse still refused → commit → reopen → inspect source → Study note citing only verified passage → Teaching session → DOCX/Markdown participant output with no private Scripture/leader text. Repeat for invented DOCX and text-layer PDF. Explicitly record an image-only PDF refusal.
- **No false successes:** if the file is unsupported, unsafe or ambiguous, the app must tell the user and keep their original unchanged. A clean refusal counts as safe, **not as recognition coverage**.

## 5. Exit and communication discipline

**Milestone only passes when:** (1) code is integrated into application; (2) source status and provenance persist, rerun gate after all corrections; (3) unverified text cannot escape; (4) PDF worker limits and cancellation are verified; (5) all permitted Rust/React suites and exact-code CI pass; (6) installed, isolated Windows application passes the end-to-end owner smoke test; (7) findings from E3 evaluation and issues #57/#59 are reconciled honestly for a limited personal release.

Progress reports must start with **which user journey is now usable and demonstrably tested**; then blockers. New research, indefinite audits, fancy language/canon extensions, AI additions, cloud services and extra PR chains are not substitutes.

Do not merge or sign/publish/updater-install without owner approval. Do not use owner real documents, live profiles, private Collection H or paid CI. Codex/engineering works here; Claude continues only its already-running independent E3 review.

**The finish line is a Bible that can be imported, cited and used safely in a real Windows Abiding Word study session.**
