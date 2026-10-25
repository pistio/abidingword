# PDF text in reading order (v0.24.1)

v0.24 read a PDF's text in the order its text layer draws it. On a page set
in two columns that order often runs across both columns line by line, so a
verse's lines were mixed with lines of the other column. The recogniser
detected this and refused the chapters affected — accurate, but on real
typeset Bibles only 11 % (World English Bible) and 23 % (Luther 1912) of the
verses could be used.

v0.24.1 reads **where** the text is on the page and puts it back in reading
order before anything else looks at it. The same layer serves the library
(Sources) and Bibles read from documents: there is one way to read a PDF's
text.

## 1. Three kinds of text, kept apart

| | What it is | Where it lives |
|---|---|---|
| **Document text** | the characters the PDF draws, each with its page, position and size | `pdf_extract` output, collected by `pdf_layout::Collector` |
| **Reading-order text** | the same characters, in spans, lines and blocks put in the order the page is read | `pdf_layout::PageLayout` (lines with positions, column, block, role and the index of their first character in drawing order); `PageLayout::text()` is what is stored as the document's segments |
| **Recognised Scripture** | verses read from the reading-order text by the conservative recogniser | `scripture::structure` — each verse a slice of that text, with its origin (document, page, segment) |

No character is added, dropped or repeated between the first two: each drawn
character is in exactly one span (tested), text set at an angle is kept at
the end of its page, and a page whose order cannot be established keeps its
drawing order.

## 2. How a page is put in order (`library/adapters/pdf_layout.rs`)

1. **Spans.** Characters drawn one after another on one baseline, with no gap
   wider than about a character, form a span. A space is put where the gap is
   wider than a narrow space. A span never crosses a column gutter.
2. **Running head and page number.** The first and last line of a page are
   taken apart when they lie in the outer eighth of the page with a clear
   space between them and the text (role `Head` / `Foot`).
3. **Notes.** Text smaller than the body's that stands entirely below all of
   the page's body-size text, with a little space above it, is the page's
   notes (role `Note`). It is read after the columns — never between them —
   whether it sits under one column or across the page.
4. **Columns.** A gutter is a vertical strip that no span crosses (or only a
   few: lines over both columns), with at least six spans entirely on each
   side, each side at least a fifth of the text's width, and the strip at
   least half a character wide. The search is a sweep over span edges
   (O(n log n) per page). A narrow column of verse numbers in a margin is
   never a column.
5. **Bands.** Lines that cross the gutter (a book title over both columns,
   a heading in the middle of a page) divide the page into bands; each band
   is read left column, then right column, then the crossing line. A
   crossing line must stand clear of the columns: if body-size text of a
   column stands beside it, the gutter is not a gutter there and the page is
   **uncertain** (below). Smaller text on the crossing line's own baseline
   (the sign before a footnote, a second footnote) belongs to it.
6. **Lines.** Within a column, spans whose baselines are within a fifth of
   a text size of each other form a line; a raised verse number, a raised
   note sign or a chapter number centred in the line stays in its line when
   it lies within the line's height,
   read left to right; lines are read top to bottom. A large initial (a
   chapter number set over two lines) opens the first line beside it. A wide
   space above a line starts a new block (paragraph).
7. **Nested columns.** A column is examined once more for a gutter (three
   columns), never deeper.
8. **The document's gutter.** A page with too little text to show its
   gutter (the last page of a book, half empty) uses the gutter that the
   document's other pages of the same side (odd or even) share, when at least
   five pages agree on it. A one-column line with a gap wider than two and a
   half characters inside it is never accepted as a line: the page is
   uncertain instead (this is what kept 1 Samuel 31:13 from losing words).
9. **Raised small letters** (one or two lower-case letters set smaller and
   higher than the text: cross-reference callers) are marked in a second
   copy of the line (`Line::marked`), never removed from the document text.

### Uncertain pages

A page is **uncertain** when:

- more than a few characters (over 20, or 2 % of the page) are set at an
  angle; or
- a line crosses the gutter with column text beside it; or
- its notes cannot be put in order.

Its text is kept in drawing order (as before), its lines are marked, and the
recogniser refuses every chapter that takes text from it with
`bibleImport.documentReadingOrder` (*Genesis 5: page 12 is laid out in a way
whose reading order could not be established. This chapter is not used.*).
Nothing is rearranged speculatively.

### Robustness

- The whole document is read at once; if `pdf_extract` fails or panics, each
  page is read on its own, so one bad page leaves only that page empty
  (reported), instead of — as before — losing every page after it.
- At most 40 million characters per file are examined.
- Encrypted and malformed files are refused with a message, as before; a PDF
  without a text layer is still read page by page and can be marked by hand.

## 3. What the recogniser does with it (`scripture/structure.rs`)

The recogniser keeps its v0.24 rules: it never guesses, and refuses a chapter
whose structure it cannot read. What v0.24.1 adds is **evidence** it may use
— always from the document itself:

- **Notes known from the layout** (`Unit::note`) are left out with the
  pattern-found page footnotes and counted (`documentFootnotes`).
- **Note signs** (`†`, `‡`, `§`, `¶`, `*`) are taken out of a verse **only when
  a note on the same page, or the page before or after, names that sign and
  that verse** ("‡ 9:44 …"). A run of signs (`*‡`) is one unit. A sign with no
  such note stays in the text as written. This fixes Mark 9:44 (regression
  test `mark_9_44_a_cluster_of_note_signs_is_taken_out_when_the_notes_name_the_verse`).
- **Raised letters** are taken out only when a note names the letter and the
  verse ("a 3:16 …"); a raised letter without one refuses its chapter
  (`documentRaisedLetters`) — it is never left glued to a word.
- **Uncertain pages** refuse their chapters (`documentReadingOrder`).
- **Tables of contents.** A line with dot leaders to a page number
  ("Exodus . . . . . 60") is never a heading.
- **Large type** (1.15 × the page's body size or more) is a heading, never
  verse text ("BOOK 2" in the Psalms).
- **Running heads** naming a passage are checked against what was read: a
  page whose head says Revelation 17 while the recogniser is in chapter 16
  shows a lost chapter, and the chapter is refused (`documentRunningHead`).
  eBible's known head slip (the head keeps the previous chapter at a chapter
  start) is tolerated, and heads that run backwards are ignored.
- **Chapter resynchronisation.** A number alone on its line, followed by
  verse 1, greater than the current chapter and within the book, begins that
  chapter even when the previous chapter is short (an edition with 17 verses
  where the standard count is 18); a jump of more than one chapter is
  reported (`documentChapterJump`).
- **Labels inside verses.** Short capitalised words repeated as lines of
  their own (speaker labels "Beloved", "Lover"; acrostic headings
  "ALEPH") refuse the chapter (`documentLabel`): they are not Scripture and
  cannot be told apart from it safely.
- **Book titles** are matched in their printed forms: "The Second Book of
  Samuel", "Paul's Letter to the Romans", "The Acts of the Apostles", "Das
  erste Buch Mose (Genesis)", two-line titles with a lower-case second line,
  and a one-letter misprint of a name of six letters or more ("Chonik").
  Lines naming a book and a chapter count as Bible lines only when the
  document is written that way (a preface citing "Luke 17:36" is not).
- **Hyphenation.** "Lob- und Dankopfer" keeps its hyphen and space: a hyphen
  before *und / oder / and / or …* is a suspended compound, not a line-end
  break.

### Results (`docs/audit/v0.24.1/`)

Each edition's PDF compared with the USFM of the **same revision** (audit
only; see 4):

| | World English Bible | Luther 1912 |
|---|---|---|
| Verses in the edition | 31 103 | 31 102 |
| Usable v0.24 | 3 353 (10.8 %) | 6 910 (22.2 %) |
| **Usable v0.24.1** | **30 782 (99.0 %)** | **31 102 (100 %)** |
| Exact | 30 185 | 31 102 |
| Differ only by whitespace (thin spaces, line ends) | 864 | 0 |
| Genuine differences in usable verses | **0** | **0** |
| Genuine differences in refused chapters (never shown) | 49 | 0 |
| Pages with uncertain order | 0 | 0 |

WEB book-layout PDF (`engwebp_book.pdf`): the same figures. Compared with the
older WEB USFM bundled with Abiding Word instead, three usable verses differ
(Exodus 24:3, Deuteronomy 6:10, 28:29): the PDF has the newer wording.

**Gaps (WEB):** Psalm 119 (176 verses, acrostic headings), Song of Songs 1, 2,
4–8 (106 verses, speaker labels), Revelation 8 and 22 (34 verses: a chapter
number read twice; the glossary printed after 22:21), and five verses never
found (Luke 17:36, Acts 8:37, 15:34, 24:7, Romans 16:24: verses this edition
does not print in its text). **French LSG** (`fraLSG_all.pdf`) is refused as a whole: book
introductions stand between each title and its first chapter and verses carry
cross-reference letters and "V. …" notes; nothing wrong is accepted. Seven of
its pages are uncertain.

## 4. Independence from reference texts

The USFM of the same edition is used **only** by the audit test, after
recognition, to measure accuracy. The layout and the recogniser never see it:
a person's own PDF Bible is read the same way without any matching USFM.

## 5. Precise places

Every verse keeps its origin (document, page, segment). **Show it on page N**
now also marks where the verse begins on its page: the viewer looks for the
verse's first words in that page's own text and marks and scrolls to them
only when they occur exactly once there; otherwise it opens the page.

## 6. Sources

New PDF sources are stored in reading order too (better search and reading
of two-column books). Sources already in the library keep their stored text;
nothing is extracted again. The original PDF view, zoom, page search and
marks are unchanged; a mark whose words cross a column boundary is placed on
the page only where the page's own text shows them in that order.
