# Scripture recognition: handover for integration review (Codex)

**Status: engineering handover for review, not an integration.** Nothing
here is wired into the application beyond what `main` already has. The
interface, the database and Windows remain Codex's. Migration 0028 is
reserved for Codex and is **not** written or consumed here; §8 is a
proposal for it.

The candidate is recorded in `docs/audit/candidate-a1.5/FREEZE.md` (code
SHA, source checksums, toolchain, binary digests). Its independent
evaluation (Set H) has **not** been run; see §11.

## 1. What the engine does

Given a Bible's documents (PDF with a text layer, EPUB, DOCX, Markdown,
plain text), it finds books, chapters and verses and returns:

* each verse's text exactly as the document has it, never written,
  completed, translated or repaired;
* whether each verse is **verified** (its text and reference are shown by
  the document's own evidence) or **unverified**, with a reason;
* where each verse is in the document;
* for EPUB, exact provenance: the entry, its SHA-256, the character and
  byte ranges of every run of the verse, and every excluded range
  (note callers, note signs, apparatus) with its kind;
* what it set apart (headings, notes, apparatus) and every issue.

No AI is used anywhere in it.

## 2. Entry points

| Function | Use |
|---|---|
| `scripture::structural::stage_documents(&[PathBuf], ImportOrigin) -> AppResult<(StagedImport, StructuralReport)>` | **The candidate pipeline.** Structure-aware reading for EPUB; the candidate recogniser for every other format. |
| `scripture::import::stage_documents(&[PathBuf], ImportOrigin) -> AppResult<StagedImport>` | The reading `main` uses today (no EPUB provenance). Same recogniser for non-EPUB formats. |
| `scripture::numbering::assess(&StagedContent) -> Vec<BookNumbering>` | Where the printed numbering departs from the reference numbering, per book (§5). |

Both staging functions read every file, then recognise. They are
synchronous and CPU-bound; call them off the interface thread.

## 3. Output contract

### `StagedImport`

* `content.books[ordinal].chapters[chapter].verses[verse] -> String`: every
  verse found, verified or not. `bridges[first] = last` for "4-5".
* `content.issues: Vec<ImportIssue>`: `severity` (`Error` refuses its
  chapter, or its book, or with no book everything), `book`, `chapter`,
  `code` (an i18n key) and `params`. `content.usable_chapters()` applies
  the rule.
* `documents: Option<DocumentStage>`:
  * `unverified: BTreeMap<(u8, u16, u16), &'static str>`: every verse found
    whose text is **not** verified, and the reason (§4);
  * `origins: BTreeMap<(u8, u16, u16), Origin>`: each verse's first and
    last `Place` (document, page, segment, character offset);
  * `left_out`, `apparatus`: text set apart, with places;
  * `books: Vec<FoundBook>`: each book heading, as found and as used.

### `StructuralReport` (EPUB)

`provenance: BTreeMap<"BOOK c:v", VerseProvenance>`:

| Field | Meaning |
|---|---|
| `reference` | (book ordinal, chapter, verse) |
| `file` | document index, file SHA-256, entry path, entry SHA-256 |
| `text` | the verse's text as staged |
| `spans` | `SourceRange { charStart, charEnd, byteStart, byteEnd }` per run; non-contiguous verses have several |
| `excluded` | `ExcludedRange { kind, range, elementBytes, text }`: `noteCaller`, `conflictingCaller`, `noteSign`, `apparatus` |
| `quote` | the spans' text joined; equals `text` when `status` is `exact` |
| `status` | `exact`, `spacingDiffers`, `untraced` |
| `idCheck` | `none`, `agrees`, `contradicts { chapter, verse }` |
| `reasons` | why the verse was accepted or refused, in words |

`gated`: verses whose verification the structural gate withdrew, and why.

**Byte offsets point into the stored entry** (identified by its SHA-256),
counted from its first byte, byte-order mark included.
`provenance::original_bytes` reads them back. The engine does not keep
the entry's bytes after reading: whoever re-reads a span must have the
original file.

## 4. Verification semantics

**Only a verified verse is Scripture text the app may store and show as
the edition's verse.** An unverified verse is a place in the document,
with a reason; its text must not be shown as Scripture. Unverified text
can contain another verse's words, a heading, a note or another
chapter's text under this reference (independent canon review, finding 3).

| Reason | Meaning for the user |
|---|---|
| `possibleHeading` | It may contain a heading or a label. |
| `reference` | It contains a Bible reference. |
| `noteSign` | It contains a note sign, a private-use character or an unreadable character (U+FFFD). |
| `unnumbered` | Its end is not certain. |
| `length` | It is far longer than the verses around it. |
| `psalmTitle` | It may include the psalm's title. |
| `boundary` | Where it begins or ends is not certain. |

A verse in a chapter refused by an `Error` issue is never verified, even
if not in `unverified`.

**"Verified" is source-document fidelity, not edition authenticity.** It
means: this text is what this document shows at this printed number. It
does not mean the document is a complete or faithful copy of the edition.
A document missing words shows them missing, and the verse verifies
without them (the `spablm-docx` case, `docs/audit/candidate-a1.3/ledger.md`).
Every recognised verse should stay **provisional** in the app until the
person confirms the edition; migration 0028 is to enforce that.

## 5. Numbering and canon

* References are the document's **printed** numbers, filed under the
  66-book canon's ordinals. The engine never renumbers.
* A Hebrew-numbered Psalter keeps its titles as verse 1; a Vulgate or
  Synodal Psalter keeps its own psalm numbers; Malachi 3:19-24 stays there.
  These verify: their text is the document's at those numbers.
* **Do not join verses of different editions by (book, chapter, verse)**
  unless both editions' numbering is known to agree. `numbering::assess`
  marks a book `departs` when a chapter or a verse is printed beyond the
  reference numbering (66 books, KJV-tradition chapter counts, the largest
  verse count over five English systems). `withinReference` is not proof
  of English numbering.
* Books outside the 66 (Tobit, Sirach, Maccabees, Greek Esther and Daniel,
  Psalm 151 …) are recognised by name and left out, never merged into a
  canonical book. Catholic Esther 10-16 and Daniel 13-14, Joel 4,
  Septuagint numbering and Hebrew chapter divisions are refused as errors:
  coverage, not misfiled text.
* Book names are known in English, German, French, Spanish, Portuguese,
  Korean, Hebrew, Greek and Latin. Documents in other languages find
  nothing, and say so.

## 6. Resources

| Limit | Value | Where |
|---|---|---|
| Files per Bible | 100 | `documents::MAX_FILES` |
| File size | 200 MB | `documents::MAX_FILE_BYTES` |
| All files | 400 MB | `documents::MAX_TOTAL_BYTES` |
| EPUB entry, inflated | 20 MB, and never more than it declares | `epub::read_entry` |
| EPUB spine text, inflated | 120 MB per book | `epub::MAX_SPINE_BYTES` |
| PDF characters | 40 million | `pdf_layout::MAX_CHARS` |
| PDF forms drawing forms | 32 deep; cycles refused | `pdf::recursion_hazard` |

Measured on the disclosed corpus (release build, reading alone):

* EPUB median 119 MB, highest 249 MB;
* PDF median 185 MB, highest 566 MB (`spavbl-rlofodt`).

Times and per-edition figures: `docs/audit/candidate-a1.5/FREEZE.md` §4.

**Requirements for integration:**

1. **Run PDF reading in a separate process with a memory cap** (Linux
   `RLIMIT_AS`, Windows Job Object). lopdf inflates streams without a
   limit while loading (independent security review, finding 5), and that
   cannot be bounded in-process. The A2.1 PDFium helper (PR #46) shows the
   pattern; PDFium itself is **not** part of this candidate.
2. **There is no cancellation or progress hook.** A long import can only
   be abandoned by ending the process that runs it. The places a flag
   would be checked are listed in
   `docs/audit/candidate-a1.5/reviews/security-review.md`.
3. Treat any panic as a refused document. The PDF reader catches its own;
   the others return errors.
4. **Unfinished integration work: every rerun of recognition must pass
   the structural gate again and regenerate provenance and numbering
   evidence.** Applying saved
   corrections (`StagedImport::apply_saved`), making new ones
   (`StagedImport::correct`) or reading the documents again for any other
   reason calls `import::recognise`, which **replaces**
   `DocumentStage::unverified` (with `origins`, `books`, `left_out` and
   `apparatus`) by the recogniser's fresh result. The gate
   (`structural::gate`) runs only inside `structural::stage_documents`,
   and needs each EPUB's `StructuralSource`, which that function drops when
   it returns. So, as the candidate stands, after a correction every
   withdrawal the gate made is gone, a verse it withheld can be shown as
   verified, and the `StructuralReport` and `numbering::assess` describe
   the reading before the correction. The integration must:
   * keep each EPUB's `StructuralSource` with the staged import (or read
     the files again);
   * after **every** call of `recognise`, run `structural::gate` again,
     replace the `StructuralReport` (provenance, link roles, gated
     verses) and recompute `numbering::assess`;
   * never show or store a verse as verified from a staged import whose
     report predates its latest recognition;
   * test it: an EPUB with a verse the gate withdraws stays unverified
     after `apply_saved` and after `correct` (for instance a book heading
     re-assigned), and its provenance and numbering reflect the correction.

   This is application work. The frozen recogniser (`08c6faa`) is not
   changed for it, and no figure in the evaluation covers a corrected
   import: the harness reads each document once, without corrections.

## 7. Not supported

* Scanned PDFs (no text layer): refused as `scanned`.
* PDF text the page does not show (invisible render mode, off the page):
  read as text. pdf-extract gives no visibility information.
* EPUB text hidden by CSS `display:none`: read as text. (HTML `hidden`,
  `aria-hidden`, `<template>` and SVG titles are discussed in the security
  review.)
* Hebrew-letter verse and chapter numerals (א, ב … טו): not read, so
  documents numbered only so find no verse.
* Right-to-left PDFs whose reading order cannot be established: those
  pages are refused.
* Languages beyond the nine in §5.

## 8. Proposal for migration 0028 (Codex)

Not written here. What the engine's output needs stored, so that an
imported verse stays inspectable and provisional:

* **Edition document identity:** per file, its name, SHA-256 and format;
  per EPUB entry, its path and SHA-256.
* **Verse status:** `verified` or `unverified` with its reason, and a
  separate `confirmed_by_person` (null until the person confirms the
  edition). Only verified verses carry text the app shows as Scripture.
* **Provenance:** for EPUB, `spans` and `excluded` as JSON (they are small);
  for other formats, the `Origin` places.
* **Numbering:** per book, `numbering::assess`'s `agreement` and its
  differences, so that the app never aligns a `departs` book with another
  edition by number without a mapping.
* **Engine version:** the candidate SHA that produced the reading, so a
  later engine can re-read and compare.

`bible_verse_places` (0023) already keeps unverified places with their
reasons; the proposal extends, not replaces, it.

## 9. User-facing behaviour (proposal)

* Say "Recognised from your document" for verified verses, never
  "verified Scripture" alone; keep every verse provisional until the
  person confirms the edition.
* Show unverified verses as places to open in the document, with the
  reason, never as text.
* Show refused chapters and books with their issues.
* For a book whose numbering `departs`, say that its verse numbers
  follow another tradition, and do not offer parallel reading with
  another edition by number.

## 10. Integration tests for the app

From this repository, runnable without the corpus:

* `cargo test --release --locked`: every unit and integration test,
  including `recognition_a1_structure`, `recognition_a1_3_safety` and the
  A1.5 hardening tests.
* `python3 -m unittest discover -s tests/corpus -p 'test_*.py'`: the
  completeness auditor and derivation tests.
* `python3 -m unittest discover -s tests/evaluation -p 'test_*.py'`: the
  withheld-set tools (check, seal, run, score).
* To be written by the integration: the correction test of §6,
  requirement 4.

With the disclosed corpus (outside Git): `src-tauri/tests/real_document_editions.rs`,
`AW_PIPELINE=structural`, as in `FREEZE.md`.

## 11. Evaluation status and limitations

* All figures are on 113 **disclosed** editions (the development corpus
  and Sets B–G). They are regression evidence, not independent evidence.
* An independent, withheld **Set H** has not been built or run. Its
  protocol awaits the owner's approval (`docs/audit/candidate-a1.5/set-h-decision-package.md`).
* Known wrong verified verses on the disclosed corpus: the nine of
  `spablm-docx` v1, a defect of that test document (`ledger.md`). On the
  corrected v2, none.
* The rules are measured on these 113 editions. They only withhold
  verification, so an unseen layout costs coverage, not correctness, as
  far as the disclosed evidence shows.
* No figure covers an import after corrections: the harness reads each
  document once. Until requirement 4 of §6 is met, a corrected import can
  show as verified a verse the gate had withheld.

## 12. Open integration work (main application, PR #54)

Assigned to the main Abiding Word integration work, not to recognition
development. The frozen recogniser (`08c6faa`) is not changed for any of
it, and a Set H pass does not complete any of it.

| # | Work | Where it is specified |
|---:|---|---|
| 1 | Run the structural verification gate again after every saved correction, new correction or re-recognition | §6, requirement 4 (**unfinished**) |
| 2 | Regenerate provenance and per-book numbering evidence after such changes | §6, requirement 4 (**unfinished**) |
| 3 | Migration 0028: provisional recognition standing and its evidence | §8 (proposal only) |
| 4 | Unverified text never enters Scripture display, comparison, search, Study citations, Teaching, AI context or exports as verified Bible text | §4, §9 |
| 5 | PDF processing in a memory-limited worker process | §6, requirement 1 |
| 6 | Progress and cancellation | §6, requirement 2 |
| 7 | Edition-specific numbering kept; cross-edition alignment only through validated mappings | §5 |
| 8 | Windows integration and end-to-end acceptance | the application's own acceptance process (`docs/updates.md`) |
