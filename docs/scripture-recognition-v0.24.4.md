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

Recogniser frozen at **17398c6** (`src-tauri/src/scripture/structure.rs`,
`library/adapters/{pdf_layout,epub,mod}.rs`; later commits touch only
`tests/` and `docs/`). Evidence in `docs/audit/v0.24.4/`.

### Checks at the frozen commit

`cargo fmt -- --check`, `cargo clippy --all-targets --locked -- -D warnings`
and `cargo test --locked --no-fail-fast` (every suite, migrations included)
pass; `npm run typecheck`, `lint`, `i18n:check`, `test` (1 053 tests) and
`build` pass. One timing-sensitive front-end test failed once while the Rust
suite and the audit chain were loading the machine and passed on a clean
re-run of the same code. Reading the same document twice gives the same
result (the Berean PDF, read twice at 771f40e and again at 17398c6:
identical refused chapters, counts and verses).

### Regression sets (disclosed; not independent)

* **Set C** (`set-c-regression.md`; blind figures from v0.24.3 beside each):
  no verified false positive in any document (blind: 73). Verified verses with
  wrong text: 1, the shorter ending of Mark appended to 16:20 in the OEB PDF
  (a reference artefact; blind: 6 032). The KJV margin PDF: 56.5 % detected,
  0 % verified — every verse found is left unverified because its boundaries
  cannot be settled from the layout (blind: 38.3 % detected, 37.2 % verified,
  4 886 wrong). pandoc EPUB 89.5 / 89.3 % clean (blind 84.5 %, 929 wrong);
  Portuguese A5 PDF 99.4 / 97.7 % clean (blind 75.5 %, 199 wrong). Two
  documents stay at 0 % (no verse numbers; Hebrew-letter numerals): refused,
  nothing invented.
* **Set B** (`set-b-regression.md`): no verified false positive (first
  evaluation: 23); 28 verified differences, all classified in
  `verified-differences.md` as reference artefacts (acrostic headings and
  "Selah" placement in the USFM, headings inside verse text), two numbers the
  PDF generator dropped, and the ASV's closing rubric of Habakkuk 3:19 — no
  contamination (the four of 8a61c55 — page numbers and a note letter read
  into Berean verses — are gone, §2). Berean PDF 77.3 / 71.2 % (first
  evaluation 62.9 / 59.3 %, 41 wrong).
* **Development corpus** (`dev-regression.md`, 46 documents): no verified
  or detected false positive anywhere. Detection equal to v0.24.3's final run
  in 43 documents, higher in two (Vulgate PDF +1 832, Segond PDF +176) and
  lower in one (Segond EPUB −136: four chapters of Jeremiah and Ezekiel now
  refused because "Ainsi parle l'Éternel:" on a line of its own, repeated,
  stands between two sentences — a fail-closed cost of the label rule, §2).
  Verified text 95 % or more of the printed verses in 39 of 46 documents;
  against v0.24.3 it moved between +1.6 % (porbr2018 PDF) and −5.7 % (Vulgate
  PDF, whose chapters were refused before and are now found but partly
  unverified) — the intended cost of the new fail-closed rules.

The 20 remaining verified differences of the development corpus
(`verified-differences.md`) are 7 reference artefacts (the Textbibel's USFM
encoding Scripture as headings), 4 subscriptions kept apart by design and 9
verses whose words the converter dropped; the two contaminations v0.24.3 left
there (Textbibel headings in the body's type, the Song of Songs label) are
gone. Across the development corpus, set B and set C together: zero verified
false positives, zero contaminated verified text.

### Withheld set D (independent; blind; run once)

`withheld-d/summary.md`. Twelve documents built after the freeze by an agent
that saw neither the recogniser nor sets B and C: new generators (fpdf2,
LibreOffice from HTML, MuPDF Story, pandoc DOCX, hand-written EPUB 2,
Markdown with bold chapter-verse, verse-per-line text), new languages for the
sets (Septuagint Greek, Hebrew) and new layouts (drop caps, running heads
with the chapter, right-to-left). Three editions reuse source texts of the
development corpus (Luther 1912, Segond, Korean) in new layouts; the set's
generator relaxed its own sanity checks for two two-column PDFs; one PDF the
app refuses outright.

Result: **NOT clean.**

| | |
|---|---|
| Documents | 12 (11 read; 1 refused by the app as unreadable) |
| Clean or honestly rejected | 5: pandoc Word 99.3 / 99.2 %; ReportLab A5 PDF 37.3 / 36.4 %; margin-number PDF 0 % (every chapter refused); Hebrew booklet 0 %; MuPDF PDF refused |
| Verified verses with wrong text | 1 196 in 6 documents |
| Verified verses under a book code the edition's reference does not use | 898 (Greek Esther and Daniel filed as `EST`/`DAN`, two Brenton editions) |

The causes are few and general: **running heads not set apart** (1 028: a
long book title in capitals, a Korean "book chapter–book chapter" head, a
work title on every page), **a verse number set against its first word**
("23que", "27»La", "28ahe": 120 verses merged with the next), **footnotes
read as verse text** (11), **inline theme labels** (31), and **Septuagint
books outside the canon table** (898, with chapters longer than the canon
not refused). The margin-number evidence from set C generalised (fpdf2 PDF:
0 % with no false positive); the A5 ReportLab PDF is clean at low coverage.

The first evaluation of each withheld set has now failed hard safety three
times in a row (B: 23 false positives, 1 734 wrong; C: 73 and 6 032; D: 898
and 1 196). Each round fixed its set's causes generally and left the earlier
sets clean, and each new set found new layouts. That is the pattern §4's
safety architecture exists for.

## 6. Verdict

**NOT READY** for text read from documents to enter a Bible as verified
Scripture without the product-level safeguard.

Supporting evidence: zero verified false positives and zero contamination in
the development corpus and in sets B and C at 17398c6, with coverage
unchanged or higher except where fail-closed rules were added deliberately;
but the independent set D verifies wrong text in six of eleven documents
read, and files Septuagint books under Hebrew-canon codes.

What this milestone delivers, and what is recommended:

1. **Merge-ready recognition improvements** on
   `claude/scripture-recognition-v0.24.4` (17398c6 code; `856a81b`+ docs):
   general evidence for margin numbers, declared note callers, stacked
   titles, letter-spaced labels, running heads by book name, the Textbibel
   and Song of Songs contaminations, page numbers and note letters in a
   verse, deterministic running-head reading. Every rule fails closed. The
   owner decides when to merge; nothing has been merged or published.
2. **The provisional-import architecture** (`docs/design/`) should be
   implemented by Codex on the schema-26 integration branch before any Bible
   read from a document is offered as verified: set D shows that new layouts
   keep arriving faster than general rules can be proven on them, and the
   product must hold such text as *provisional* until a person has reviewed
   it.
3. **Next recognition round, if the owner wants one**: the set D causes are
   general and small in number (running-head forms; a number set against a
   word; notes in the LibreOffice PDF; theme labels; Septuagint canon
   codes), to be addressed on general layout evidence and proven on a set E
   built after that freeze. Set D is a regression set from now on.
