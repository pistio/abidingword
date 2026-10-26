# Bible recognition across languages, editions and layouts (v0.24.2)

v0.24.1 read typeset PDFs in reading order, and the World English Bible and
the Luther Bible could be used by book, chapter and verse. The Louis Segond
1910 could not: **0 %** of its verses were found. Other editions in other
layouts fared little better (a third of the King James Version PDF, under
half of two Portuguese Bibles, nothing of the Hebrew Bible or the Latin
Vulgate). v0.24.2 makes recognition work across translations, languages,
editions and document layouts, and measures it on a corpus of real editions
(`docs/corpus/`).

The numbers before and after are in `docs/audit/v0.24.2/`.

## 1. Why the Louis Segond was 0 %

The French failure was not a French special case. It came from three general
weaknesses that the LSG happens to combine:

1. **Titles with articles.** "Le Lévitique", "La Genèse", "Les Nombres",
   "L’Évangile selon Matthieu": the title reading took off ordinals and words
   such as "book" or "gospel", but not articles, so French titles named no
   book. With no book, no verse was placed.
2. **A fixed distance between a title and chapter 1.** A title was accepted
   only when chapter 1 or verse 1 followed within three lines. The LSG opens
   each book with an introduction of one or several pages; its titles were
   all refused, even once they could be read.
3. **Apparatus inside the chapters.** Section headings ("Naissance
   d’Ismaël"), lines of cross-references under them ("V. 7-16: cf. Ge
   21:9-21."), cross-reference letters raised at the same size as the text
   and footnotes ran into the verses, or made the chapters fail their checks.

Each is fixed by a rule that holds in every language (sections 3–5); the
fixes were checked on a second French edition (Ostervald) and on the other
languages, never only on the LSG.

## 2. Four stages

Recognition is one pipeline (`src-tauri/src/scripture/structure.rs`) over the
units every document reader produces (`scripture/documents.rs`: PDF lines in
reading order, EPUB and Word paragraphs, Markdown and text lines, each with
its place and its look).

| Stage | What it decides | Where |
|---|---|---|
| **A. Document structure** | what is front matter, a table of contents or navigation, a running head, a book's title, an introduction, a section heading, a line of cross-references, a note, a chapter's label — before any verse is read | `Builder` |
| **B. Language rules** | how titles are written in each language (articles, ordinals before or after the name, the words that describe a book, second names, medieval Latin spellings, Greek title formulas), how sentences end (Korean final endings) | `title_forms`, `book_named`, `greek_forms`, the alias dictionaries and the reference packs |
| **C. Sequence** | verse numbers in order, chapters in order and complete, books in canonical order, every chapter checked against the standard verse counts in both directions | `Reader`, `short_chapters`, `next_in_sequence` |
| **D. Verification** | every verse kept is a slice of the document's own text with its place; a chapter with any doubt is refused, never guessed, and the review says why | `Reader::close`, the issues |

A chapter the recogniser has any doubt about is **refused** and reported; its
verses are not used. Partial use (v0.24) lets the other chapters be used.

## 3. Stage A — the document's structure

* **Titles anywhere before chapter 1.** A title set apart as one (a heading
  style, large type, capitals) may be followed by an introduction of any
  length; the scan for chapter 1 passes lines of cross-references and
  sentences of the introduction that happen to name a book — one opening
  with a numbered book's name ("2 Samuel présente …" is not verse 2), a
  section heading beginning with the title's own book ("Josué à la tête du
  peuple") — and wrapped lines of prose, which are never titles even when
  they name a book ("Moïse. Ce dernier changea son nom Osée ou …"). A plain
  line naming a book must still be followed by chapter or verse 1 within a
  few lines.
* **A missing title is never papered over.** When a book's title is not
  found, its chapters would run on into the book before; that book then has
  a chapter twice and those chapters are refused, never merged.
* **Introductions.** Numbered text between a title and chapter 1 is the
  introduction: numbered paragraphs before the chapter 1 heading, and
  numbered lists that start again at 1 before chapter 1 is complete ("1. la
  guérison d’un démoniaque 4:33-36", "Adresse et salutations 1-2").
* **Section headings** between verses are left out when the document sets
  them apart: a heading style, large type, another typeface (PDF glyph
  widths, slanted italics), a block styled bold, italic or centred (EPUB CSS,
  Word paragraph properties), capitals, a line centred in its column (PDF),
  or the passage's cross-references right under it. A heading may run over
  two lines, end with a full stop when it has several parts, or end with its
  parallel passage in brackets ("…재판하다(신 1:9-18)"). A line that only
  *could* be a heading (no style, ending without punctuation) stays in its
  verse: "… vor Gott" may be the verse's last line.
* **Cross-reference lines** ("V. 18-24: cf. (1 Co 11:7-10.)", "Ps 70; 142.",
  "Ch. 31 à 34.") are never Scripture; nor are references to the source of a
  quotation that an edition sets after it, at a verse's end or inside it
  ("… sacrifício. Oseías 6:6 Porque eu não vim …", "… sequer. Salmos 14:1-3;
  53:1-3; Eclesiastes 7:20 Todos …"): a book with chapter and verse right
  after closing punctuation. Scripture never writes "book chapter:verse".
* **Notes** are recognised by their place (smaller text at the foot of a PDF
  page), by their form (a sign and a chapter:verse, even printed without
  spaces: "† 2:6there…") and by EPUB note links; a note's sign is taken out of
  the verse only where a note on the page names that verse.
* **Chapter labels.** A number alone in a block set apart by its look is that
  chapter's label, whatever the chapter's length — editions number some
  chapters differently (Louis Segond Job 38 has 38 verses, the standard
  numbering 41). Labels of a book's divisions ("BOOK 2", "LIVRO 3") are not
  verse numbers.

## 4. Stage B — language rules

Titles are reduced to a book's name by per-language tables (German, English,
French, Spanish, Portuguese, Latin, Greek): articles, ordinals (before or
after the name, as words or Roman numerals — never a figure after the name,
"Jó 1" is Job's chapter 1), the words that describe a book ("Brief", "Book",
"Épître", "Carta Universal", "Liber", "ΕΠΙΣΤΟΛΗ ΚΑΘΟΛΙΚΗ"), words after a
name ("of Solomon", "de Salomão", "prophetae"), and second names ("The First
Book of Samuel Otherwise Called the First Book of the Kings", "Liber Beresit
id est Genesis", "Ecclesiastes or, the Preacher") — read by the first name,
and never by a name one letter away when a title gives two. Names that differ
only by an accent ("Jó", Job, and "Jo", João) are decided by the document's
language as written. A title that is one letter or syllable is no title
(Korean reference abbreviations such as 마). **Latin** became a reference
language (OpenBible data, after the 100 population-ranked languages) for the
Vulgate and scholarly editions; medieval spellings (SAMUHELIS, ISAIÆ) are
read as such.

## 5. Stage C — sequence

* Verses in order within a chapter; a missing verse, a verse number standing
  inside the verse before, a verse far longer than any verse, a chapter that
  begins twice or out of order: the chapter is refused.
* **Chapters checked both ways** against the standard verse counts: a chapter
  much shorter than usual lost its end; one far longer than usual (by six
  verses, or half as many again) had another chapter or book read into it
  (`bibleImport.documentChapterLong`, new). Both are refused.
* A title that names one of numbered books without its number ("Crónicas")
  is the one that follows the book before it in the canon — only once the
  book before has some of its text (a subtitle right under "1 Crónicas" is
  not 2 Chronicles).
* A book that begins twice is refused rather than merged.

## 6. Stage D — verification and the person's view

Every verse kept is a slice of the document's text with its document, page
and position; nothing is filled in from another edition. The review keeps
four states apart: verses **recognised and usable**, chapters **refused**
with their reason, passages a person **marked** by hand (v0.24), and text
left **unresolved**. It now opens with one sentence: which books are ready to
use and how many chapters need checking ("Psalms and Romans are ready to use.
2 chapters need checking."). Reading the kept files again for verses ("Look
for verses…") needs no new import.

## 7. Layout reading (PDF) changes

* **Slanted italics** (LibreOffice, Word draw italics by shearing an upright
  face) were taken for rotated text: every glyph became its own line and the
  page's order was uncertain. A shear without rotation is upright text and
  counts as style evidence.
* **Centred lines** are style evidence in any script — bold Korean, Chinese
  or Japanese letters are as wide as the others, so glyph widths cannot tell.
  A centred line that ends as a sentence does (a poem's line) is not.
* **Word paragraphs** centred, or bold or italic throughout, carry style
  evidence (`NewSegment.styled`), as EPUB blocks do from their CSS.

## 8. What is not done

* **Right-to-left PDFs** (Hebrew): pdf_extract gives right-to-left text in an
  order that cannot be established, so those pages stay uncertain and their
  chapters are refused. The Hebrew Bible reads correctly from EPUB.
* **OCR.** Scanned Bibles have no text layer. A local, consented OCR path
  (an OCR engine installed by the person, its text marked unverified until
  checked) was investigated but not built in this milestone; scanned PDFs can
  still be added to read page by page.
* **Jerome's Latin titles** in the Vulgate ("INCIPIT LIBER MALACHIM ID EST
  REGUM TERTIUS ET QUARTUS", "LIBER DABREIAMIN", "HIEZECHIELIS",
  "ACTUUM APOSTOLORUM") are not read: Hebrew names, the old numbering of
  the four books of Kings (which clashes with the modern Latin names, where
  "1 Regum" is 1 Kings) and inflected spellings would need guessing. The
  chapters of those books are refused, never given to another book. The
  deuterocanonical books have no reference data yet.
* **Unstyled headings in scripts without sentence punctuation** cannot be
  told from a verse's last line; they stay in the verse.
* **Preserved corrections.** A re-run reads the kept files again; book
  corrections chosen earlier are not yet stored, so they are chosen again.
