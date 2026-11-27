# Scripture recognition — product completion milestone
**Date:** 25 November 2026. **Status:** integrated and verified on Linux (§6); **Windows acceptance not yet done** — not accepted, not releasable.  
**Owner objective:** *Use Abiding Word to study Scripture, retain reliable notes and produce faithful Teaching.* This milestone is delivered only when Scripture recognition works in the **Windows application**, not when an evaluation report passes.

**Current application candidate:** the [productive-use completion record](productive-use-completion.md)
combines this late Claude work with current main, Bible-details guidance,
USFM corrections and durable restore recovery. It records the product's
recogniser revision separately from the frozen research candidate below.
The dated verification records in this document remain historical evidence.

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

## 6. Verification record (25 November 2026, branch head after `3ced47d`)

**Usable and demonstrably tested (Linux, automated):** an invented EPUB, DOCX
and text-layer PDF Bible of Ruth is each previewed, corrected and gated again,
committed into a real profile, reopened after a cold restart, read by
chapter, searched, cited in a Study note, taught, presented and exported —
and its one withheld verse never becomes Scripture
(`src-tauri/tests/recognition_journey.rs`):

| Step | Observed, in all three formats |
|---|---|
| Preview | Ruth 1:1 verified; 1:2 withheld (`boundary`); the preview does not show 1:2's words |
| Correction | the book heading corrected: a new gated pass (generation +1); 1:2 still withheld |
| Commit | the kept copy byte-identical to the original and to the recorded SHA-256; one run, 84 verified evidence rows, 1:2 `unverified` with its reason, no Scripture text stored for it; edition confirmed only when the person confirmed it (EPUB yes, DOCX/PDF no) |
| Provenance | EPUB: entry path, byte spans, the linked note recorded as an exclusion; PDF: page; DOCX: paragraph; 1:2 inspectable at its source and findable in the document |
| Reopen, read | chapter 1 shows 21 verses and 1:2 as an unchecked place |
| Search, concordance | the verified word found; the withheld word never |
| Passage (Study, Teaching, assistant) | Ruth 1:1–3 gives 1 and 3 only |
| Study citation | a phrase note on 1:1's words saved and shown; an anchor in 1:2 refused |
| Teaching, presentation | slides show 1 and 3; the person preparing it is warned (`scripturePartial`, new) |
| Exports | Markdown, PowerPoint and PDF without the withheld words; Markdown and PowerPoint with the verified ones; Materials passages the same |
| Original file | unchanged |

Also: an image-only PDF yields no verse (OCR required) and is unchanged; the
PDF is read in the separate worker process (the app executable), as in the
app.

**Schema 27 → 28 and backups** (same file): a schema-27 profile holding a
document Bible (rows as the latest pre-0028 code stores them), a note on it
and its kept EPUB is opened: one pre-migration safety copy, which is the
exact v27 database; schema 28; **no row of any Bible table or of the notes
changed**; foreign keys sound; the Bible stays a document, edition
unconfirmed, and its words are kept out of Scripture until read again, while
the source stays searchable. Reading its kept file again restores the
verses with evidence (1:2 withheld) and keeps the note. A daily snapshot
carries every evidence row; after the Bible is removed, restoring the
snapshot brings them back.

**Checks at the branch head:** `cargo fmt -- --check` clean; `cargo clippy
--all-targets --locked -- -D warnings` clean; `cargo test --locked` (CI
settings) **1 199 passed, 0 failed, 9 ignored** (106 groups); `npm run
typecheck`, `lint`, `i18n:check` pass; `npm test` **1 455 passed** (136
files); `npm run build` passes.

**Fixed while verifying:** three native messages without translations
(`documentChapterUnmarked`, `documentChapterUnread`,
`documentSeptuagintNumbering`; translated in parallel by `f824d5a`, whose
wording is kept); two German strings used the formal address; the
six-language review corpus regenerated; a Teaching passage with a withheld
verse was shortened silently — the presentation now warns
(`scripturePartial`, six languages).

**Windows acceptance workflow:** `.github/workflows/windows-scripture-acceptance.yml`
runs only on a push to `build/pr61-windows-scripture-acceptance-20261125` with
the approval commit message. Its `SOURCE_SHA` (`056333b`) predates these
fixes; it should be re-pinned to the verified branch head when the owner
approves a Windows build.

**Not done, and required before acceptance (§4, §5):** the native Windows
smoke test on a disposable profile with an installed build (needs the
owner's authorisation for a Windows build); PDF worker memory/time limits
and cancellation on Windows (the Job Object path is compiled only there);
React-level checks of the import screen's progress, cancellation and focus
beyond the existing UI tests; reconciliation with issues #57 and #59. CI
was not run on this branch (it does not target `main` or a CI branch).

