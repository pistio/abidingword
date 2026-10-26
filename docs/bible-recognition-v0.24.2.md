# Bible recognition across languages, editions and layouts (v0.24.2)

v0.24.1 read typeset PDFs in reading order, and the World English Bible and
the Luther Bible could be used by book, chapter and verse. The Louis Segond
1910 could not: **0 %** of its verses were found. Other editions in other
layouts fared little better (a third of the King James Version PDF, under
half of two Portuguese Bibles, nothing of the Hebrew Bible or the Latin
Vulgate). v0.24.2 makes recognition work across translations, languages,
editions and document layouts, and measures it on a corpus of real editions
(`docs/corpus/`).

The numbers before and after are in `docs/audit/v0.24.2/` (§9, §10). On the
development corpus no usable verse is one the edition does not have; on the
hidden editions, which were not used to write any rule, **three are** (§9):
recognition is not ready to be trusted without the person's review.

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
  up to four lines in another typeface, its later lines going on in lower
  case and its last one perhaps a single word too short for its style to
  show ("… Un démoniaque" / "guéri"); it may end with a full stop when it
  has several parts, or with its parallel passage in brackets
  ("…재판하다(신 1:9-18)", "셈의 후예(대상 1:24-27)" — such a bracket alone sets
  a line apart, as Scripture never writes one). A paragraph of its own that
  the document marks as a heading (an EPUB or Word heading style) is a
  heading however long and whatever its last mark, and also inside a verse,
  between two of its sentences ("… changé l’eau en vin." / "Guérison du fils
  d’un officier du roi" / "Il y avait …"). A paragraph merely bold, italic
  or centred is one only when the passage's cross-references follow it:
  italic poetry and quotations are styled paragraphs too, and stay in their
  verses ("My God, in whom I trust!”"). A line in another typeface that ends as a sentence does
  stays Scripture: poetry and spoken words are set in other typefaces too
  ("Bénissez l’Éternel!", "Escúchenlo."). A line that only *could* be a
  heading (no style, ending without punctuation) stays in its verse: "… vor
  Gott" may be the verse's last line.
* **Closing words stay.** "Selah" is never a heading; a note's sign after a
  psalm's closing words ("Praise the LORD!†") does not hide that they end
  a sentence.
* **Books outside the 66** (Tobit, Judith, Wisdom, Sirach, Baruch, the
  Maccabees, 1–2 Esdras, the Prayer of Manasseh, Psalm 151, and the Greek
  text of Esther or Daniel printed as a book of its own) are named by their
  titles in the interface languages, Latin and Korean, and their text is
  left out with a note (`bibleImport.documentAdditionalBook`), as the USFM
  importer already does: their chapters no longer run on into the
  canonical book before them (Tobit into Malachi).
* **Subscriptions.** An epistle's subscription after its last verse ("Πρὸς
  Ῥωμαίους ἐγράφη ἀπὸ Κορίνθου …", "Written to the Romans from Corinthus …")
  is set aside even without a style, read over the lines it is wrapped on;
  a verse that merely begins "Πρὸς" is kept.
* **Cross-reference lines** ("V. 18-24: cf. (1 Co 11:7-10.)", "Ps 70; 142.",
  "Ch. 31 à 34.") are never Scripture; nor are references to the source of a
  quotation that an edition sets after it, at a verse's end or inside it
  ("… sacrifício. Oseías 6:6 Porque eu não vim …", "… sequer. Salmos 14:1-3;
  53:1-3; Eclesiastes 7:20 Todos …"): a book with chapter and verse right
  after closing punctuation. Scripture never writes "book chapter:verse".
  A reference line wrapped onto a line of bare numbers ("… Jé 6:6-15," /
  "27-30.") is part of it. In Hangul, Chinese and Japanese a word of two
  syllables is prose, never a reference abbreviation, so a Korean line such
  as "7:28) 너는 가증한 것을 …" stays in its verse.
* **Notes** are recognised by their place (smaller text at the foot of a PDF
  page), by their form (a sign and a chapter:verse, even printed without
  spaces: "† 2:6there…", "g 9:542R1:10" for note g of 9:54) and by EPUB note
  links; a note's sign or raised letter is taken out of the verse only where
  a note on the page names that verse — after the word it marks or before
  it ("Χριστοῦ, †ᾧ").
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

* **The document's language.** A document that states no language, or one
  that its own titles contradict (LibreOffice writes its interface language,
  "en-US", into the PDF of a Korean Bible), is read again in the language
  whose own book names its titles use — when that language names at least
  three titles and three in five, more than any other, and the stated one
  fewer than one in five. The running text is never used to guess.
  Names that differ only by an accent are decided on the title's name ("O
  Livro de Jó" is Job, not João).
* **The Vulgate's own titles** that general rules cannot read — Hebrew
  names (MALACHIM, DABREIAMIN), the old numbering of the four books of Kings
  ("REGUM TERTIUS"), inflected spellings (HIEZECHIELIS, ACTUUM) — are read
  from an explicit, edition-aware table, for Latin and for whole titles
  only. "Regum primus" alone is not in it: 1 Samuel in the Vulgate,
  1 Kings in the Nova Vulgata, and nothing in such a title tells them apart.

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
* **Hyphens at a line's end.** A word broken at a line's end is joined
  ("gover-" / "nor"). The hyphen is kept only on the document's own
  evidence: the document writes the hyphenated word inside a line elsewhere
  ("Hazezón-tamar"), or the word already carries a hyphen before the break.
  A hyphen written nowhere inside a line is taken for the line break's and
  dropped; where a name's hyphen is lost this way it shows as a typography
  difference in the audit (rv1909-lopdf: 6), never as changed words.
* **Right-to-left pages** whose glyphs a PDF draws in visual order (eBible's
  Hebrew Bible) are put back in reading order from the glyphs' places, so
  they can be read and searched; see §8 for why no verse is taken from
  them.

## 8. What is not done

* **Right-to-left PDFs** (Hebrew) — a limit of the source document, not of
  the reading: eBible's Hebrew PDF draws its glyphs in visual order, and
  reading them back right to left finds the books, chapters and verses
  (88.9 % of verses in chapters that would be usable, no false positive),
  but such a PDF records where each vowel point and accent is drawn, not
  which letter it belongs to (a holam drawn before its vav, an accent
  between two letters, U+034F joiners added): only 426 of 20 623 verses
  match the edition even after Unicode normalisation, 60 % on their
  consonants alone. Those pages stay uncertain and their chapters are
  refused. The Hebrew Bible reads correctly from EPUB (98.8 %).
* **OCR.** Scanned Bibles have no text layer. A local, consented OCR path
  (an OCR engine installed by the person, its text marked unverified until
  checked) was investigated but not built in this milestone; scanned PDFs can
  still be added to read page by page.
* **The Vulgate's numbering.** Its psalms (and some chapters elsewhere)
  follow the Greek numbering, which the standard verse counts do not: those
  chapters are refused, never renumbered. Its deuterocanonical books are
  named and left out (no reference data yet).
* **Psalm titles inside a verse's own line.** A one-verse-per-line export
  ("PSA 3:1 A Psalm by David, when he fled … LORD, how my adversaries …")
  gives no mark between the title and the verse: the line is kept as the
  document writes it (138 such verses in the World English Bible export).
* **Unstyled headings in scripts without sentence punctuation** cannot be
  told from a verse's last line; they stay in the verse.
* **Preserved corrections.** A re-run reads the kept files again; book
  corrections chosen earlier are not yet stored, so they are chosen again.

## 9. Hidden editions

Ten documents of five editions in five languages were kept out of
development (role `hidden` in `docs/corpus/editions.tsv`): no rule was
written or tuned on them, and their results were read only after the
development corpus was final (`docs/audit/v0.24.2/hidden/`). They are the
Textbibel 1906, the French Darby 1885, the Spanish Free Bible (with its
deuterocanon), Young's Literal Translation and the Byzantine Greek New
Testament, in eBible page sizes and typesetting not in the development
corpus (A4, book size), one EPUB, and four LibreOffice Word and PDF
conversions made from the editions' USFM. One subscription fix (§3, a
2 Timothy subscription wrapped over five lines) was made on the
development corpus after the hidden run; it does not touch the rules the
hidden results below depend on.

| Edition | Usable | Different (typography / extra / cut / wrong ref. / changed) | False positives |
|---|---|---|---|
| deutkw-pdf | 97.7 % | 294 (0 / 281 / 1 / 0 / 12) | 0 |
| frajnd-pdf | 80.1 % | 921 (359 / 482 / 5 / 0 / 75) | **3** |
| frajnd-epub | 78.1 % | 0 | 0 |
| spablm-pdf | 94.8 % | 2 (0 / 1 / 1 / 0 / 0) | 0 |
| engylt-pdf | 100 % | 1 (0 / 0 / 1 / 0 / 0) | 0 |
| grcbyz-pdf | 99.7 % | 20 (0 / 15 / 0 / 0 / 5) | 0 |
| deutkw-docx | 54.4 % | 1679 (0 / 1667 / 1 / 0 / 11) | 0 |
| deutkw-lopdf | 54.3 % | 1669 (0 / 1657 / 1 / 0 / 11) | 0 |
| spablm-docx | 67.6 % | 1285 (0 / 1278 / 6 / 0 / 1) | 0 |
| frajnd-lopdf | 35.4 % | 1151 (107 / 1042 / 1 / 0 / 1) | 0 |

What they show, by cause (parser limitations unless said otherwise):

* **Footnotes taken as verses (false positives).** The Darby PDF prints
  three verses only in its footnotes ("a 23:14 Le « Texte reçu » ajoute le
  v. 14 : Malheur à vous …"; Acts 8:37 and 15:34 likewise); its notes are not
  set smaller, and the recogniser accepted them as Matthew 23:14 and Acts
  8:37 and 15:34. Apparatus accepted as Scripture: the most serious result
  of this milestone.
* **Running heads inside verses** ("Le premier livre de Samuel 1:1 364 Le
  premier livre de Samuel 1:17", "ΠΡΟΣ ΤΙΜΟΘΕΟΝ Α΄ 4:13 232 …"): a page's
  head with its page number, joined to the verse that runs over the page.
  Most of the "extra" and all the "changed" in frajnd-pdf and grcbyz-pdf.
* **Unstyled section headings ending in a full stop** ("Das Paradies und
  der Sündenfall.", "Davids Volkszählung. (Vergl. …") stay in the verse
  before them or take verse 1's place (deutkw-pdf).
* **Verse numbers written against the word** ("5auf Kain", "15y sean"): the
  LibreOffice conversions from USFM run two verses together (most of the
  "extra" in the four converted documents).
* **Small capitals** (the Darby's "ÉTERNEL" for "Éternel"): the PDF draws
  capitals; 5876 verses differ only in case (counted usable; a property of
  the source).
* **Pentateuch titles** of the Darby are not recognised in any of its three
  formats, so Genesis to Deuteronomy are not found (frajnd-epub's 78 %).

## 10. Audit evidence

`docs/audit/v0.24.2/` holds:

* `summary-baseline.md` — the corpus at the start of this work;
* `final/summary.md` — the development corpus at its end, every edition;
* `final/differences.md` — every difference in a usable verse, with its kind,
  page, and the found text beside the edition's own;
* `hidden/summary.md`, `hidden/differences.md` — the same for the hidden
  editions (the first 40 differences of each, with the counts by kind).

The false-positive count is the audit's own check (a usable verse whose
reference the edition's USFM does not have), listed at the end of each
`differences.md`.

