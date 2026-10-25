# Document Bibles you can navigate, German book names, other canons (v0.24)

v0.23 let a Bible be added from a PDF, EPUB, Word, Markdown or text file,
either by book, chapter and verse (when the structure was recognised without
doubt) or as a document to read. v0.24 makes a document Bible **findable by
passage** without pretending to know more than it does:

- verses **recognised and confirmed** can be found by passage, even when other
  chapters of the same document are doubtful (partial use);
- the place of a passage can be **marked by hand** (a PDF page, or a paragraph
  or part of an EPUB, Word or text file);
- a passage that is neither is shown as **not identified yet**, never filled
  from another Bible.

Also in this record: the German names of the books of Moses (Part E) and the
foundations for canons beyond the 66 books (Part G). Windows acceptance steps:
[windows-acceptance-v0.24.md](windows-acceptance-v0.24.md).

## 1. What is stored, and what is never changed

Three different things are kept apart:

| What | Where | Changed by v0.24? |
|---|---|---|
| The original document (the file and its extracted paragraphs) | `bible_documents`, `bible_document_segments` | No. Never rewritten, never re-imported. |
| Canonical references (`JHN.3.16`, verse key `book × 1 000 000 + chapter × 1 000 + verse`) used by notes, marks, studies, sources | `bible_verses` (the 66-book registry) | No. No reference is renumbered. |
| Where a passage is in *this edition* | `bible_verse_texts` + `bible_verse_origins` (recognised), `bible_passage_locations` (marked by hand) | Added to. |

A mark records only the canonical range and a place (page, paragraph position,
part of the document) plus, optionally, the words that were selected there as a
check. Notes, Scripture marks and references keep pointing at canonical verses:
adding, changing or removing a mark cannot change or lose them. Migration 0021
adds the one table (additive; `CHECK` constraints keep a range inside one book
and require a place).

## 2. Partial use: only what was found without doubt

`Look for verses…` (v0.23) reads the kept files again and shows what was found.
Before v0.24, one doubtful chapter meant nothing could be used. Now:

- **Whole recognition with no problems** — as before: the Bible becomes a
  normal Bible read by book, chapter and verse.
- **Some chapters doubtful** — the dialog offers *Use the N verses found
  without doubt*. `StagedContent::usable_chapters()` returns every recognised
  chapter that **no error names**. If any error does not name a book (so its
  reach is unknown), nothing is offered. Only those chapters' verses are
  stored (`bibles::insert_verses_where`), each with its origin in the document
  (`insert_origins_where`). The Bible **stays a document**: it is still read as
  the document, and its other passages say they are not identified yet.
- Doing it again replaces the earlier reading (verses, origins and Psalm
  titles are deleted first, in the same transaction); marks made by hand are
  kept.

## 3. Finding a passage in a document Bible

The document reader in the Scripture pane gains a line saying how much can be
found (*13 verses can be found by passage: John (3 of 21 chapters) · Romans
(1 of 16 chapters) · 1 place marked by hand*), a *Find* field, and *Mark a
passage here*.

Asking for a passage (typed there, or a reference clicked anywhere while the
document Bible is open — `ScriptureContext.show()` now keeps the open document
and asks it) calls `bible_document_locate`:

- verses found are shown with the edition's own text, and *Show it on page N*
  opens the document where the first verse begins;
- `complete` is true only when every verse of the standard count for the range
  has its text, checked verse by verse (a bridged verse "4–5" counts for both;
  extra or repeated verses never make up for a missing one). Otherwise the panel says only part has been found and offers
  *Read it in WEB* (the Bible read by passage); it never mixes the two;
- places marked by hand are listed (*Psalm 23, page 412*) and open the
  document there;
- with neither, the panel says *Where Psalm 23 is in this document has not
  been identified yet.* and offers to mark it.

## 4. Marking a place by hand

*Mark where it is* records where the reader is now:

- a **PDF** (original layout): the page shown;
- **EPUB / Word** (original layout): the part shown and the paragraph it starts with;
- **text view** (any format): the paragraph clicked, else the first shown.

The person types the passage there (read with the same reference parser and
language rules as everywhere else). The core checks that the document belongs
to that Bible, that the page exists (a page can be marked only in a document
that has pages: never in an EPUB or Word file), and that the paragraph exists
(`db::bible_documents::add_location`). Marks can be removed from the panel.

## 5. Recognition: what improved, and what cannot be

The recogniser reads text in reading order. PDF text comes from `pdf_extract`
**without positions**, so a two-column page whose columns were read across
each other cannot be put back in order — it can only be detected. v0.24 adds
that detection: a verse number well ahead of the count (`> expected + 1`,
within the chapter's standard verse count) that appears **inside** a verse,
**and from which the count goes on** (its next number follows within the
next three numbers: "3 … 18 …", "4 … 19 …"), refuses the chapter with
`bibleImport.documentSequence` (*verse 12 appears inside an earlier verse
(perhaps two columns were read across each other). This chapter is not used.*).
A lone number in the text ("… of the field. 12 men came") is not refused.
With partial use, the other chapters can still be used.

Also added in v0.24, from a test on real typeset Bibles (below):

- **Running heads naming a passage** ("249 Josua 2:1", "Josua 4:1 252 Josua
  4:23") on the first or last line of 30 % of the pages or more are page
  furniture, left out and counted.
- **Footnotes at the foot of a page** (from the first line that starts with a
  note sign and a chapter and verse, "† 1:5 …", to the page end) are left out
  and reported (`bibleImport.documentFootnotes`).
- **Note signs** († ‡ § ¶ \*) raised against a word, set apart as a word of
  their own, or at the start of a verse are taken out of the verse text. They
  are never letters of the text.
- **The next chapter's number at the end of a line**, once the chapter has
  all its verses and verse 1 is numbered next ("… wandelten. 9" then
  "1 Und …"), begins the chapter; the number is not kept as verse text.
- **A book title printed over two lines** ("The Good News According to" /
  "Matthew") is the title: the first line, a whole short line after a verse
  that ended its sentence, is taken back out of that verse.

### Accuracy on real editions

`tests/real_document_editions.rs` (ignored by default; set
`AW_DOCUMENT_EDITIONS` to a folder holding the PDFs) reads two complete
typeset PDF Bibles from eBible.org, two columns per page, and compares every
recognised verse with the same edition's USFM bundled with Abiding Word. The
PDFs are not in the repository (about 9 MB each).

| Edition (eBible PDF) | Verses in the edition | Recognised | Identical to the edition | Usable by partial use | Usable but different |
|---|---|---|---|---|---|
| World English Bible, `engwebp_all.pdf` | 31 103 | 3 486 | 3 430 (36 of them differ only by a thin space between quotation marks) | 3 353 | **1** |
| Luther 1912, `deu1912_all.pdf` | 31 102 | 7 316 | 7 060 | 6 910 | **0** |

Before the v0.24 changes above, 391 (WEB) and 1 494 (Luther) usable verses
differed from the edition (running heads, note signs and footnotes inside
verses). The one difference left: Mark 9:44 keeps a trailing "\*" that the
PDF's text places after the verse.

**v0.24.1** reads PDFs in reading order from where their text is drawn
(`docs/pdf-reading-order-v0.24.1.md`). On the same two PDFs, compared with
the USFM of the same revision as the PDF (`docs/audit/v0.24.1/`):

| Edition | Usable verses v0.24 | Usable verses v0.24.1 | Exact | Differ only by whitespace | Usable but different |
|---|---|---|---|---|---|
| World English Bible | 3 353 (10.8 %) | 30 782 (99.0 %) | 30 185 | 864 | **0** |
| Luther 1912 | 6 910 (22.2 %) | 31 102 (100 %) | 31 102 | 0 | **0** |

The Mark 9:44 "\*" is gone: a note sign is taken out only where a note on its
page (or the page beside it) names that sign and that verse. Chapters still
refused in the WEB: Psalm 119 (acrostic headings *ALEPH*, *BETH*… inside the
chapter), Song of Songs 1, 2, 4–8 (speaker labels *Beloved*, *Lover*,
*Friends*), Revelation 8 (a chapter number read twice) and 22 (the glossary
printed after the last verse). They are refused, not guessed, and can be
marked by hand. A French LSG PDF (book introductions between title and text,
cross-reference letters) is refused as a whole.

Already in v0.23 and still true: page headers and page numbers repeated
across pages are left out and reported; footnotes in DOCX are notes, not verse
text; Psalm titles are kept as titles; verses that span a page break are
joined; edition-specific chapter numbering that differs from the standard
(Malachi with three chapters, Joel with four) is kept as written and
reported, never silently renumbered. Text that belongs to no verse (headings, introductions, notes) is
listed under *Left out*, with samples, so it can be audited.

**Words are never dropped, inserted, invented or moved.** Each stored verse is
a slice of the document's own text; `tests/bible_documents.rs` checks that
every verse text occurs verbatim in its origin paragraph.

Known limits:

- (v0.24) multi-column PDFs read across columns are refused per chapter —
  v0.24.1 puts columns in order; a page whose order cannot be established
  is still refused;
- a verse number printed without anything that marks it as a number (no
  superscript information survives PDF extraction) may be read as text, or a
  number in the text as a verse: the sequence checks catch most, not all;
- scanned PDFs have no text: they are read page by page, and pages can be
  marked by hand.

## 6. OCR (investigation only)

Not implemented. What a local-only design would need:

- an engine that runs offline on Windows (Tesseract 5 with `tessdata_best`
  language files for Hebrew, Greek, German, Korean… is the realistic choice;
  it is Apache-2.0, ~30–60 MB per script set, not bundled);
- every OCR result stored as **unverified** text, kept apart from text the
  PDF itself carries, never used to recognise verses without the person
  confirming, and labelled wherever shown;
- accuracy on Bible pages is poor exactly where it matters (small verse
  numbers, two columns, Hebrew points, Greek accents), so it would serve
  searching a scanned page, not navigation.

No cloud service is considered. Until then, scanned PDFs stay readable page by
page, and passages can be marked on their pages by hand.

## 7. German book names (Part E)

German shows the books of Moses as **1. Mose (Genesis)**, **2. Mose
(Exodus)**, **3. Mose (Levitikus)**, **4. Mose (Numeri)**, **5. Mose
(Deuteronomium)** in book lists, headings and pickers
(`src/i18n/locales/de/books.json`). In a passage the name in brackets is left
out (`passageBookName`): *1. Mose 1,1*, both in the interface and in exports
(presentation, Materials). Stored identifiers (`GEN`…`DEU`) are unchanged.

Searching and typing references accept all forms, as before: *1. Mose*, *1
Mose*, *1 Mo*, *Genesis*, *Gen*, *Exodus*, *Ex*, *Levitikus*, *Lev*,
*Numeri*, *Num*, *Deuteronomium*, *Dtn* (`scripture/aliases/de.rs`; tests in
`tests/reference_languages.rs`).

Places names appear, checked: the Scripture pane, book and chapter pickers,
passage labels, search results, concordance, Study and Teaching passages,
Materials and presentation exports (`exportLocale`), document-Bible panels,
import review (book names in issues come from `bookCode`).

German wording audit: every German string was searched for formal address
(*Sie, Ihnen, Ihr…*). The 12 hits are all third person (*sie* = it/she, *ihr*
= its/their); the interface consistently uses *du*. A test
(`tests/ui/german-address.test.ts`) keeps it so.

## 8. Canons beyond the 66 books (Part G)

`scripture/canons.rs` lays foundations and claims nothing more:

- **Canon, order and versification are separate concepts.** The 66-book
  registry stays the identity of every stored reference. Deuterocanonical
  books and the Greek additions (Tobit, Judith, Esther (Greek), Wisdom,
  Sirach, Baruch, Letter of Jeremiah, Song of the Three, Susanna, Bel and the
  Dragon, 1–4 Maccabees, 1 Esdras, Prayer of Manasseh, Psalm 151, Daniel
  (Greek)) have USFM identities and **reserved ordinals 67–84**, so adding
  them later can never renumber a Protestant reference.
- Three orders are described: Protestant (66), Catholic (73: Esther and Daniel
  with their Greek parts, Letter of Jeremiah as Baruch 6) and the Septuagint's
  (Greek Orthodox; other Orthodox churches differ in a few books). Greek Esther
  and Greek Daniel stand in place of Esther and Daniel and share their verses'
  references (`canons::contains`).
- **Supported today**: a Bible's files that contain these books import their
  66 books as before, and each other book is **named** in the review (*holds
  Tobit, which Abiding Word cannot show yet. It was left out; the other books
  are not affected.*) instead of being called unknown.
- **Not supported**: storing, showing or searching their text, references to
  them in notes and sources, Septuagint versification (Psalms numbering,
  Jeremiah order). Book names in that note are English.

## 9. Tests

- Rust: `tests/bible_documents.rs` (partial use keeps doubtful chapters out
  and the Bible a document; verse text equals document text with provenance;
  manual mark validation; two columns refused; a short second column refused;
  Malachi numbering; a lone number after a sentence end is not a column read
  across), `scripture::structure` unit test (note signs),
  `tests/real_document_editions.rs` (ignored; real editions, above),
  `tests/scripture.rs` (deuterocanonical books named and
  left out; a few files of another Bible among many refused; date stamps on
  `\id` lines are no evidence), `scripture::canons` unit tests (ordinals never reused, every
  tradition keeps the 66 books, NT order), `tests/reference_languages.rs`
  (German aliases), `tests/database.rs` (schema).
- Interface: `tests/ui/document-passages.test.tsx` (partial use, finding a
  passage, not identified, marking and removing a mark, invalid input),
  `tests/ui/i18n.test.tsx` (German names and passage labels).
