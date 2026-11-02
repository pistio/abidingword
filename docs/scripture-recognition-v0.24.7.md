# Scripture recognition v0.24.7: general rules for set F's failure classes

Set F was the independent blind evaluation of v0.24.6, run once at the
frozen recogniser `324cfd1`. It ended FAILED / NOT READY: 98 verified verses
the editions do not print and 505 verified with wrong text
(`docs/audit/set-f/`). The owner accepted that verdict and approved a v0.24.7
round on the disclosed failure classes, ending at a frozen recogniser and a
stop for approval. **Set F is now disclosed regression evidence. No run
against it is reported as independent. No set G was built.**

Branch: `claude/scripture-recognition-v0.24.7`, from `ff6762c`, the head of
`claude/scripture-recognition-v0.24.6`, which carries the sealed set F record
on top of the frozen `324cfd1`. That branch is unchanged.
`claude/bold-noether-n8l5ds` is unchanged at `3a46578`. Nothing is merged,
released or published, and no paid Actions run was started (`claude/*`
pushes run no CI).

## 1. Kinds of evidence, kept apart

| Kind | What it is here | Independent? |
|---|---|---|
| **Original blind evaluations** | Sets B and C (v0.24.3), D (v0.24.4), E (v0.24.5) and F (v0.24.6). Each was run once, and the record is kept unchanged in `docs/audit/`. | Yes, and each one failed. None is ever re-run as independent. |
| **Disclosed regression** | Set F's and set E's sealed files themselves, run again with v0.24.7 | No. Their failures were disclosed, and the rules were written with them in view. |
| **Reconstructed regressions** | Documents rebuilt in the disclosed layouts of sets B, C and D (`scripts/recon/`) | No. These are regression only, not the sealed files. |
| **Development corpus** | 46 documents (`docs/corpus/editions.tsv`) | No. These are development material. |

Every result below comes from this material. **No figure in this record is
independent evidence of correctness.**

## 2. Evidence preserved

* The original set F record (`docs/audit/set-f/`: the seal, ground truth,
  harness summary and verified failures) is unchanged since `ff6762c`.
  `git diff ff6762c HEAD -- docs/audit/set-f docs/audit/v0.24.6
  docs/audit/v0.24.5` is empty.
* The frozen v0.24.6 implementation stays at `324cfd1` on its own branch.
* Set F's generators and rebuild steps are preserved (`46ce75b`).
* `docs/audit/v0.24.7/corpus-checksums.sha256` holds the SHA-256 of every
  regression input: 178 files. These are v0.24.6's 152 files plus set F's 26
  documents. All were verified identical before the final runs, so the
  baseline and v0.24.7 read the same bytes.

## 3. The rules (all general)

The synthetic tests were written first and committed red (`967c6a5`). Each
later rule also started from a test confirmed red at the commit before it
(`src-tauri/tests/recognition_v0247.rs`, 16 tests). No rule names an edition,
publisher or document.

| Class | Rule | Direction |
|---|---|---|
| **1. Samuel/Kings naming** | A Kings-type title is read against the whole document. "Βασιλειῶν", "Regum", "Kingdoms", "Царств" and similar forms denote four books in the Septuagint and Vulgate numbering. When the document shows that numbering (a third or fourth such book, or Septuagint evidence), the titles are 1–2 Samuel and 1–2 Kings. When the document names Samuel itself, "Βασιλειῶν Α/Β" is 1–2 Kings. When it shows neither, the title names **no book**, because 1 Samuel and 1 Kings are equally possible. "1 Kings" and "Rois" keep the Hebrew Bible's numbering, as in v0.24.5. A trailing Greek numeral counts as an ordinal only with its numeral sign ("Βασιλειῶν Γʹ"), so "João 1" is never read as 1 John. | Fail-closed |
| **2. Latin chapter labels, no count inference** | "CAPUT XII" and other chapter words with Roman numerals give their chapters, including when set inside ornaments ("— Capítulo 22 —"). A label that also gives another numbering in brackets ("Psaume 20 (Vulg. XIX)") is read by its own number. A bracket that names the Hebrew number instead leaves the label unread. A chapter is **no longer inferred** from verse numbers that start again at 1 without a label. Such a chapter is refused with the new issue `bibleImport.documentChapterUnmarked` (Warning, six languages). Two structural forms are still read: a chapter's own number alone on a line directly before verse 1, and after a refused chapter, the next chapter's own number when it is followed by verse 1. A book-and-chapter running head above the next chapter's label is page furniture and never opens a chapter again. | Structural + fail-closed |
| **3. Markdown raw HTML** | Text inside a raw HTML block is now read, so a verse such as "Ἀμήν." in a `<p>` is no longer silently dropped. Every tag is taken out and the common character references are decoded. Script, style, template and iframe elements and comments are dropped with their content. Nothing is interpreted or executed. A block using more than simple formatting (`p`, `div`, `span`, `b`, `i`, `sup` …) is set apart and never read as running verse text; examples are headings, tables, lists, links and unknown elements. A note caller left inside the HTML (an unparsed `[^n]`) leaves the verse unverified (`noteSign`). | Structural + fail-closed |
| **4. Letter-spaced running heads** | A running head with spaced letters ("O B A D I A H", "1 S A M U E L") is read with the spaces closed. It is page furniture like any repeated head, including when it begins with a figure. | Fail-closed |
| **5. False Septuagint classification** | Greek book names alone no longer make a document a Septuagint. The Septuagint's shape is also needed: four books of Kings or one of its own books. A "LXX" after a Psalm or chapter word ("SALMO LXX") is the number 70, not a Septuagint mark. A contents page of book titles with page numbers (four or more lines, each naming a different book, with numbers that never fall) names no book and is skipped. | Fail-closed |

**No new unverified-reason value** was introduced; `boundary`,
`possibleHeading`, `noteSign` and `reference` are reused. The only interface
addition is the issue code `bibleImport.documentChapterUnmarked`. See
`docs/design/recognition-interface-contract-v0.24.7.md`.

## 4. Regression on identical inputs (baseline `324cfd1` → frozen `c96756d`)

Every run used a copy of the test binary built at the frozen commit, so a
later rebuild could not change a run. The full tables, the lists of verified
wrong verses and the changes in unverified reasons are in
`docs/audit/v0.24.7/`.

### 4.1 Accuracy (verified verses not in the edition / verified with wrong text)

| Corpus | Before (`324cfd1`) | After (`c96756d`) | Change |
|---|---|---|---|
| Set F, disclosed (13 docs) | 98 / 505 | **0 / 0** | −98 / −505 |
| Set E, disclosed (12) | 2 / 2 | 2 / 2 | none: the same verses, a source artefact (§7) |
| Set D reconstructions (7) | 0 / 0 | 0 / 0 | none |
| Set B/C reconstructions (22) | 0 / 8 | 0 / 8 | none: the same verses |
| Development (46) | 0 / 18 | 0 / 18 | none: the same verses |

**No accuracy loss on any corpus.** The lists of verified wrong verses after
the change are identical to the baseline's in sets E, B/C and development.

### 4.2 Coverage (verified verses)

| Corpus | Before | After | Change |
|---|---|---|---|
| Set F, disclosed | 111 269 | 129 438 | **+18 169**: F06 +4 721, F10 +3 142, F11 +8 666, F12 +1 188, F02 +455, F13 +7, F09 −10 |
| Set E, disclosed | 85 498 | 85 498 | 0 |
| Set D reconstructions | 110 927 | 110 927 | 0 |
| Set B/C reconstructions | 369 019 | 368 381 | **−638** |
| Development | 1 240 796 | 1 240 053 | **−743** |

### 4.3 Coverage losses, by cause

All the losses below are fail-closed. Each is a chapter that v0.24.6 numbered
only by counting verse numbers that started again at 1, which v0.24.7 now
refuses (`documentChapterUnmarked`). No verse moved to the wrong place. In
every case the verses were either verified before and correct, or are now
left out.

| Edition | Δ verified | Cause |
|---|---|---|
| francl (B/C) | −417 | PSA −315: Psalm 73 has no label, only a section head "Psaumes - Livre Troisième (73-89)", so it and the psalms that depend on it are refused (150 → 137 chapters). EST −87: chapter 5's number is merged into a note line (10 → 4 chapters). JHN −31. |
| engjps (B/C) | −181 | GEN: seven chapters with no readable label (50 → 43). |
| engasv (B/C) | −40 | ACT: one "CHAPTER 16" label set at a page edge is read as furniture (26 → 25). |
| rv1909-lopdf (dev) | −533 | 16 books lose one to three chapters each: the chapter number is pushed to the page foot and read as a note sign (for example GEN 35, LUK). It was −6 982 before `c96756d`, when each refusal also lost the rest of the book. |
| deutkw-pdf (dev) | −109 | GEN −55, 1SA −23, 2KI −20, JER −13: labels not read. |
| fraLSG-luk (dev) | −81 | LUK: two chapters (24 → 22). |
| fraLSG-pdf (dev) | −20 | 2KI −37 (one chapter); 1TI +16. |
| F09 (set F) | −10 | An unparsed note caller inside raw HTML now leaves the verse unverified (`noteSign`). The verse is no longer dropped, and its text is kept. |

### 4.4 Superseded intermediate runs

Runs at `04f0055`, `d8dccb3`, `b478d7b` and `bab49bc` found regressions that
were fixed before the freeze, each with a red test first:

* F06's unread "— Capítulo N —" labels (178 wrong, 8 false at `04f0055`);
* "João 1" read as 1 John (porbrbsl-md −243);
* F13's "SALMO LXX" read as a Septuagint mark;
* the per-book coverage loss in rv1909-lopdf.

Their outputs are kept outside the repository as superseded. None is reported
as a result.

## 5. Checks at the frozen commit

Recogniser frozen at **`c96756d`**. No file under `src-tauri/src/` or `src/`
changes after it; later commits touch `docs/` only.

* `cargo fmt -- --check`: pass.
* `cargo clippy --all-targets --locked -- -D warnings`: pass.
* `cargo test --locked --no-fail-fast`: 77 test binaries, 1 047 passed, 0
  failed, 8 ignored (the ignored tests are the corpus harnesses run above).
* `npm run typecheck`, `lint`, `i18n:check`, `test` (1 053 tests), `build`:
  pass.
* The test helper `verified()` in the v0.24.6 and v0.24.7 test files now
  also requires that no Error issue refuses the verse's chapter (tests
  only).
* **No migration, schema, stored status value or user data is touched.**
  Migration 0027 belongs to the A.3 PDF structure work. The provisional
  Bible safeguards take the next free number, provisionally **0028**, on the
  A.3 lineage (Codex). v0.24.7 adds nothing that 0028 must store.

## 6. Status

**Development complete for this round. NOT READY.** v0.24.7 has had no
independent evaluation. Set F's 0 / 0 is disclosed regression evidence on
documents the rules were written against, and it says nothing independent.
Recognised document-derived Bible text remains **provisional** in the
application, whatever any evaluation or per-verse check shows.

| | |
|---|---|
| Frozen recogniser | **`c96756d`** |
| Branch | `claude/scripture-recognition-v0.24.7` (from `ff6762c`) |
| Evidence | `docs/audit/v0.24.7/`: set F and set E disclosed regressions, set D and B/C reconstructions, development, corpus checksums |
| Preserved | `docs/audit/set-f/` and the earlier audits, unchanged; v0.24.6 frozen at `324cfd1` |

## 7. Unresolved errors and limitations

1. **Set E's two verified differences** (PSA 41:8 and PRO 4:17 in the Valera
   1602 Purificada) are unchanged. The printed page has the verse marker and
   the edition's USFM does not.
2. **Set B/C's 8 and development's 18 verified wrong verses** are unchanged
   from v0.24.6, and they are recorded per verse in the tables. Examples:
   colophons at the end of Hebrews and Galatians in porbr2018; Ezra 4's
   letter openings in spablm-docx; words from neighbouring lines in
   deutkw.
3. **Coverage limits in set F, kept fail-closed:**
   * F01: the chapter number is set in place of verse 1, with reading-order
     refusals.
   * F03: German two-column, mostly `boundary`.
   * F07: Korean PDF, reading order and chapter range.
   * F08: right-to-left pointed Hebrew, no book found.
   * F12: Russian EPUB, chapter lengths refused.
   * F13: raised letters in the Italian PDF.

   None of these was made to verify by weakening a safeguard.
4. **Refusing count-inferred chapters costs coverage**, as §4.3 shows:
   * chapters whose label is missing or merged into another line;
   * section heads in place of psalm labels;
   * numbers pushed to a page foot.

   This is the intended direction.
5. **Words lost with no trace in the layout** cannot be detected without
   comparative evidence.
6. **The reconstructions are not the sealed sets B–D**: different program
   versions may give different results on the originals.

**Stop for owner approval.** Nothing further will be done until the owner
decides: no set G, merge, release, installer, paid Actions or deployment.
