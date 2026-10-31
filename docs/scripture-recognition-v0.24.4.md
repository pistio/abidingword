# Scripture recognition v0.24.4 — general fixes from withheld sets B and C, and a safety architecture

v0.24.3 ended NOT READY (`docs/scripture-integrity-v0.24.3.md`, §11): the
development and regression corpora were clean, but the two withheld sets
showed the recogniser accepting wrong text on layouts it had not seen. The
owner decided to proceed on two fronts: general recognition improvements for
the structural problems those sets disclosed, and a product-level safety
architecture for Bibles read from documents. This record covers both.

Sets B and C are **disclosed** from this milestone on: their first, blind
results stay unchanged in `docs/audit/v0.24.3/withheld-b-first/` and
`docs/audit/v0.24.3/withheld-c/`; everything measured on them here is a
regression figure, not an independent one. The independent measure for
v0.24.4 is a new set D, built after the code freeze and run once (§5).

Branch: `claude/scripture-recognition-v0.24.4`, from v0.24.3's verified head
`c7176d2`.

## 1. What set C disclosed, and the general evidence used now

| Finding (set C, blind) | Cause | General evidence used now |
|---|---|---|
| KJV PDF with verse numbers in the margin: 4 886 verified verses with a neighbour's words, 25 verified verses that do not exist | A number in the margin marks the line a verse begins in, not where; it was read as a number at the line's start | `pdf_layout`: a line's first span of digits that ends left of the column's text (`MARGIN` mark). Such a verse is found but neither boundary is verified (`boundary`); where ten lines or more are so marked, every line-opening raised number counts as a margin number (poetry sets them inside the column). Coverage: 52.8 % found, 21.9 % verified — honest rejection of what the layout cannot settle |
| The same PDF's stacked titles ("THE SECOND BOOK OF SAMUEL," / "OTHERWISE CALLED," / "THE SECOND BOOK OF THE KINGS.") read as Kings | The title's last line decided the book | The whole stack decides (`title_start` accepts heading lines and lines ending in a comma; `book_named` on the joined title). Untitled stacks are joined before being offered for correction ("THE FIRST BOOK OF THE" / "CHRONICLES."); hyphenated titles are one title ("LAMENTATIONS OF JEREMI‐" / "AH."); "S." as an abbreviation of Saint |
| pandoc EPUB: note callers as plain digits after words ("Dieu1"): 929 verified verses with callers; 16 verified verses from "some manuscripts add" notes | The EPUB reader kept no inline semantics | `epub.rs` marks what the document itself declares: `epub:type="noteref"` / `role="doc-noteref"` / pandoc's `footnote-ref` links (`NOTE_OPEN`…`NOTE_CLOSE`, taken out of the verse and kept as apparatus), `<sup>` digits as superscripts, footnote sections (`epub:type="footnote(s)"`, `role="doc-endnotes"`, `class="footnotes"`) as notes |
| Portuguese A5 PDF: chapters run together, 32 verified verses that do not exist | "C A PÍ TU LO 20": letter-spaced labels broke their word; a bold 8.5 pt number ran into its 10 pt verse as one span, so the body size was misjudged and continuation lines became "large" headings | Letter-spaced chapter and psalm labels are joined before reading; a span is text of one size (±7 %), so the line's size is its text's |
| JPS PDF: a book's name over a page read into a verse ("… unto thee, Habakkuk and filthiness …") | The name was on too few pages to count as a running head | A page's first line that is a book's name alone, followed by running text (lower case, or a verse number other than 1) or by the book's own large title, is a running head whatever its count |
| Pieces of a two-column page read into one line ("volaban.“¡Santo …") | — | A sentence's end run straight into an opening quotation mark: `boundary`, not verified |
| Editor's notes in brackets quoting readings; speaker labels in brackets | — | Not verified (`noteSign`, `possibleHeading`) |

## 2. The two remaining contaminations of v0.24.3's own corpora

* **Textbibel PDF headings in the body's type.** The italic face has the
  upright face's letter widths, so the layout records nothing. New rule: a
  short, flush-left line that is a sentence of its own after a full line that
  ended one, as the verse's last line, is not verified. It costs some genuine
  short closing sentences their verification; that is the intended trade.
* **A speaker label written once** (WEB verse-per-line, Song of Songs). New
  rule: where a document writes inline labels, a capitalised word in a label's
  place that is rare in the whole document (twice at most) and never begins a
  sentence is a label too.

### Set B's remaining contaminations, and a reading that varied

Set B's regression run at the first freeze (8a61c55) still verified four
contaminated verses in the Berean two-column PDF: three with the page's own
number read into the line ("… a disaster 761 from which …", on page 761) and
one with a note's letter whose raising the page lost ("… in their seasons e
or lead out …"). Two general rules now leave such verses unverified: a line
holding the number of its own page (`boundary`), and a single lowercase letter
standing as a word mid-verse that is not one of the language's one-letter
words — "a" in English, "e"/"o" in Portuguese, every vowel where the
language is unknown (`noteSign`).

Running the same PDF twice with the same build then gave different refused
chapters. The running-head check gathered the page furniture from a hash set,
so the "first head" chosen for a page varied, and a cross-reference note at a
page's foot ("d 9:9 Deuteronomy 25:4") could be taken for the head and refuse
the chapter. The lines are now read in the document's order and notes are
never heads; the same document reads the same every time (checked by running
the PDF twice at the frozen commit, §5).

## 3. Fail closed

Every rule above moves a verse from verified to found-but-unverified, or a
chapter to refused, on evidence; none moves text the other way. Where book,
chapter or verse cannot be established the chapter is refused (two Set C
documents remain at 0 %: one without verse numbers, one with Hebrew-letter
numerals — nothing is invented for them).

## 4. Safety architecture

`docs/design/provisional-bible-imports-v0.24.4.md`: a `text_standing`
(verified / provisional) on every Bible, decided by its source; one gate in
`db/bibles.rs` that every consumer (reader, search, research evidence, AI
context, Teaching materials, presentations, concordance, annotations) goes
through; an optional guided sample review that records problems but never
promotes; migration 0027, additive, on schema 26. To be implemented by Codex
on the integration branch; nothing of it is on this branch.

## 5. Evaluation

*To be completed: regression runs on the development corpus, set B and set C
at the frozen commit; set D built after the freeze and run once; checks.*

## 6. Verdict

*To be completed.*
