# Scripture recognition v0.24.6 — general rules for set E's failure classes

v0.24.5 ended NOT READY (`docs/scripture-recognition-v0.24.5.md`, §7): its
independent set E, run once at the frozen recogniser `fc0f538`, verified 892
verses with wrong text and 13 verses the editions do not print. The owner
accepted that verdict as the development status and approved a v0.24.6
round on the disclosed failure classes, ending at a frozen recogniser and a
handover. **No set F was built or run**: the next independent evaluation
needs the owner's separate approval.

Branch: `claude/scripture-recognition-v0.24.6`, from `3a46578` (v0.24.5's
head; `claude/bold-noether-n8l5ds` is left at `3a46578`, unchanged). Nothing
is merged, released or published.

## 1. Kinds of evidence, kept apart

| Kind | What it is here | Independent? |
|---|---|---|
| **Original blind evaluations** | Sets B, C (v0.24.3), D (v0.24.4), E (v0.24.5): their first and only runs, kept unchanged in `docs/audit/` | Yes — and each failed. Never re-run as independent. |
| **Original disclosed evidence** | Set E's sealed files themselves, run again with v0.24.6 | No: its failures were disclosed and the rules were written with them in view. Regression only. |
| **Reconstructed regressions** | Documents rebuilt in the disclosed layouts of sets B, C and D (`scripts/recon/`, `recon-*.tsv`) | No: regression only, and not the sealed files. |
| **Development corpus** | 46 documents of 25 editions (`docs/corpus/editions.tsv`) | No: development. |

## 2. Evidence preserved (owner condition 2)

* `scripts/set-e/`: set E's generator scripts byte for byte (checksums in
  `tools.sha256`), the seal, `rebuild.sh` and a README with sources
  (eBible zip checksums), dependency versions, commands and limitations.
  Inspected first: no Scripture text, no secrets, no binary files. **No
  generated Bible and no eBible archive is committed** (three set E
  editions carry redistribution conditions). A rebuild from fresh downloads
  gave five documents byte-identical to the sealed ones and seven whose
  extracted text is identical (only embedded dates and identifiers differ).
* `scripts/recon/README.md`: commands, versions and source checksums for the
  reconstructions.
* A Python byte-code file committed by mistake in v0.24.5 (`81b27a2`) was
  removed; `__pycache__` is ignored.
* The original set E results (`docs/audit/v0.24.5/withheld-e/`) are
  unchanged (`git diff 3a46578 -- docs/audit/v0.24.5` is empty).

## 3. The rules (all general)

| Class | Rule | Direction |
|---|---|---|
| **Markdown footnotes** | The Markdown reader reads the common footnote syntax: a caller `[^id]` is marked as a declared note caller (as an EPUB's noteref is) and taken out of the verse; a definition `[^id]: …` is a note, never running text, even when it opens with a bold chapter and verse. A caller left as written leaves the verse unverified. | Structural (the document's own markup) + fail-closed backstop |
| **Greek alphabetic chapter numerals** | "ΚΕΦΑΛΑΙΟΝ ΙΓʹ" is chapter 13: Greek numerals are read only with their numeral sign and with places running downwards, so no word is ever read as a number. | Structural |
| **Hebrew numerals** (tested separately) | Hebrew numerals must run from the largest value down (ט״ו, ט״ז allowed; "אל" is no numeral). Hebrew-letter *verse* numbers inline in running text are still not read: they cannot be told from words, so such documents stay refused. | Fail-closed |
| **Unreadable chapter labels** | A line that is a word for "chapter" and a number that cannot be read ("CHAPTER ONE", "ΚΕΦΑΛΑΙΟΝ ϠΩʹ") refuses the chapter after it, never guessed (`bibleImport.documentChapterUnread`, six languages). | Fail-closed |
| **Running heads naming book and chapter** | A line naming a book and a chapter in any of the usual forms ("Matthew Chapter 5", "PSALMS 23", with a range or a page number) at a page's edge or as one of its first or last two lines is page furniture — where it also reads as the chapter's own title ("Psalm 23"), only when running text (lower case, or a verse number other than 1) follows it. A verse holding a book's name, a word for chapter and a number is not verified. | Fail-closed |
| **Words read with the other column (Selah)** | In the PDF layout, a word that ends before the right column's own left edge (the leftmost start shared by a fifth of the right side's lines) is the left column's — "Selah" set flush right at the left column's edge on a page of short poetry lines. A "Selah" in the middle of a verse, or set apart as if a heading while a verse is open, leaves the verse unverified. | Structural + fail-closed |
| **Cut verses and uncertain boundaries** | A body-type PDF line set apart only by its look (style, centring, shortness) that ends as a sentence does leaves the verse before unverified — it is as likely the verse's last line as a heading. A page where a line was held out after the page's notes leaves every verse on that page unverified (a last row read across both columns holds two verses). | Fail-closed |

None names an edition, publisher or document. **Missing words are not
claimed to be detectable in general**: the rules leave a verse unverified
where the layout gives evidence that its end is uncertain; a word dropped
with no trace in the layout cannot be found without comparative evidence.

No new unverified-reason value was introduced (`boundary`,
`possibleHeading`, `noteSign` are reused), so nothing changes for the
provisional-import side's stored values; see
`docs/design/recognition-interface-contract-v0.24.6.md`.

## 4. Regression (baseline `fc0f538` → v0.24.6 frozen at `324cfd1`)

The baseline is v0.24.5's frozen recogniser on **exactly the same files**:
every corpus file's checksum is recorded in
`docs/audit/v0.24.6/corpus-checksums.sha256` and was checked again at the
final run. Tables in `docs/audit/v0.24.6/`.

| Corpus | Detected | Text-verified | Verified, not in edition | Verified with wrong text |
|---|---|---|---|---|
| Development (46 docs) | 1 253 558 → 1 253 558 | 1 241 524 → 1 240 796 (−728, −0.06 %) | 0 → 0 | 20 → 18 |
| Set D layouts, reconstructed (7) | 115 068 → 115 068 | 111 046 → 110 927 (−119) | 0 → 0 | 0 → 0 |
| Set B/C layouts, reconstructed (23) | 398 910 → 398 929 | 369 511 → 369 019 (−492) | 0 → 0 | 22 → 8 |
| Set E sealed files (**disclosed**, regression only) | 95 531 → 98 298 | 84 953 → 85 498 (+545) | 13 → 2 | 892 → 2 |

* **Development:** the 18 left are the same verses v0.24.4 and v0.24.5
  classified (7 Textbibel headings-as-Scripture in its USFM, 2 subscriptions
  kept apart by design, 9 words the converter dropped); two other
  subscriptions are now held back as unverified.
* **Set B/C reconstructions:** the 5 one-word "Selah" contaminations left open
  in v0.24.5 are gone (each "Selah" now stays with its own verse in the BSB
  layout). The 8 left are reference artefacts (ASV Habakkuk 3:19's rubric; 7
  headings the French USFM writes inside verses).
* **Set E's sealed files:** the Markdown footnotes (700 + 4), Greek chapter
  numerals (120 + 7), "Book Chapter N" heads (68) and both cut verses are
  addressed. The 2 + 2 left are the source artefact classified in v0.24.5
  (the edition's USFM lacks the `\v` marker the printed page shows for PSA
  41:8 and PRO 4:17). These figures are **not** an independent result.
* **Determinism:** three PDFs (Tyndale three-column, Korean LibreOffice,
  BSB two-column) read again in separate processes gave identical audit
  output (all verses, differences, refusals and reasons).
* A regression caught during the round: an intermediate version of the
  column rule split three-column pages wrongly (319 cut verses on the
  Tyndale PDF); fixed before the freeze (`324cfd1`) and covered by a test.

## 5. Effect on ordinary Scripture text

The owner asked specifically about genuine numbers, unusual punctuation and
short lines. What the runs and tests show:

* **Genuine numbers:** no v0.24.6 rule reads numbers inside verse text;
  the chapter-label rule ignores any line with digits. A test with census
  figures, "12,000", ages and "chapter 3" in a verse keeps all of them
  verified (`recognition_v0246.rs`).
* **Unusual punctuation:** quotations, dashes and question marks are not
  evidence for any new rule; the same test keeps them verified.
* **Short lines:** the one rule that concerns short lines applies only to a
  PDF line in the body's size that the recogniser already set apart as a
  heading by its look, and only when it ends as a sentence does; poetry
  lines that stay in their verse are not affected.
* **Where coverage was lost (dev −728):** each edition loses 3–24 verses to
  "Selah" read as a separate line (now held back instead of verified
  without its "Selah") or standing mid-verse (Habakkuk 3:3/3:9 in some
  translations); the Textbibel LibreOffice PDF loses 181 and the KJV PDF
  121 to the look-only and notes-page rules; the LSG PDF 70. All are
  fail-closed costs; per-edition reasons are in `dev-regression.md`.
* **Not improved:** the Korean LibreOffice PDF of set E still verifies under
  1 % (its hanging-indent verse numbers look like margin numbers, which the
  v0.24.4 safeguard doubts); Hebrew-letter verse numbers remain unread;
  Hebrew right-to-left and margin-number PDFs stay refused.

## 6. Checks at the frozen commit

Recogniser frozen at **`324cfd1`**: no file under `src-tauri/src/` or
`src/` changes after it (later commits touch `tests/` and `docs/` only).

* `cargo fmt -- --check`: pass.
* `cargo test --locked --no-fail-fast`: 76 test binaries, 1 031 tests, 0
  failures (migrations untouched).
* `cargo clippy --all-targets --locked -- -D warnings`: failed first on an
  unused helper in the new test file (`tests/recognition_v0246.rs`); the
  helper was removed (tests only) and Clippy then passed.
* `npm run typecheck`, `lint`, `i18n:check`, `test` (1 053 tests), `build`:
  pass. The interface change is one localised message
  (`bibleImport.documentChapterUnread`, six languages; i18n review corpus
  regenerated).
* No migration, schema, stored status value or user data is touched;
  migration 0027 is not implemented here.

## 7. Handover

**Status: development complete for this round; NOT READY for release** —
v0.24.6 has no independent evaluation. Every figure above comes from
development, reconstructed or disclosed material, on which the rules were
developed. The v0.24.5 NOT READY verdict stands until an independent set F,
built after this freeze, says otherwise.

| | |
|---|---|
| Frozen recogniser | `324cfd1` |
| Branch | `claude/scripture-recognition-v0.24.6` (from `3a46578`); `claude/bold-noether-n8l5ds` unchanged at `3a46578` |
| Evidence | `docs/audit/v0.24.6/` (dev, set D and B/C reconstructions, set E disclosed regression, corpus checksums); original blind records unchanged |
| Preserved tooling | `scripts/set-e/`, `scripts/recon/` |

**Unresolved errors and limitations**

1. Set E's two source-artefact verses (PSA 41:8, PRO 4:17 in the Valera
   1602 Purificada): the recogniser follows the printed page, the
   edition's USFM lacks the verse marker. Not a recogniser error, but a
   verse division that differs from the edition's own file.
2. Coverage: the Korean LibreOffice PDF (hanging-indent numbers read as
   margin numbers) and margin-number PDFs verify almost nothing; Hebrew
   right-to-left PDFs and Hebrew-letter verse numbers stay refused; the
   Portuguese TFT Markdown's long titles are not read as book names.
3. Words lost with no trace in the layout cannot be detected without
   comparative evidence; the rules act only where the layout shows that a
   verse's end is uncertain.
4. The reconstructions are not the sealed sets: different program versions
   may change results on the originals.
5. Recognised text must remain provisional in the integrated application
   (Codex: migration 0027 and the consumer gate), whatever the recogniser
   marks as checked per verse; no sample review certifies an edition.

**Next step (owner's decision):** authorise set F — built by an independent
agent after this freeze, from a brief that names no rule or earlier result,
sealed and committed before one run at `324cfd1`, with no tuning against
it. Nothing of set F has been built or run.
