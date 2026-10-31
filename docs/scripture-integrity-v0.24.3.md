# Scripture integrity and cross-format recognition (v0.24.3)

v0.24.2 made recognition work across languages and layouts, and its
independent evaluation found what it could not yet do: three verses taken
from a footnote's apparatus as Scripture (the French Darby PDF), running
heads and section headings read into verses, verse numbers set against the
next word lost in converted Word and PDF files (35–68 % usable), and book
corrections that were forgotten whenever a document was read again.

v0.24.3 answers those findings with general evidence from the documents'
own layout, and adds three things the earlier milestones did not have:

1. **Finding a verse and verifying its text are separate questions.** A
   verse whose book, chapter and verse are recognised is *found*; only a
   verse whose text shows no sign of anything else read into it is
   *verified*. Only verified verses become the Bible's text. The others keep
   their place in the document, where they can be opened, with the reason
   their text was not verified.
2. **What is set apart is kept.** Footnotes, cross-references, headings,
   running heads, introductions and subscriptions are kept with where they
   stand and the verse they stand with, never mixed into a verse.
3. **The person's corrections are theirs.** A book chosen for a heading is
   stored with what the recogniser read and where the heading stands. Reading
   the same documents again applies it; a changed document, a heading read
   otherwise now, or a correction that no longer fits anything is shown to
   the person, never decided for them.

The figures are in `docs/audit/v0.24.3/` (§9). The verdict is in §11.

## 1. The three false positives

The Darby PDF prints Matthew 23:14, Acts 8:37 and Acts 15:34 only as notes
("a 23:14 Le « Texte reçu » ajoute le v. 14 : Malheur à vous …"). The notes
were never read as verses: the false positive came from the main text, which
prints the verse's number and nothing but the note's raised letter — "14 a".
The line has no other text, so the page's layout could not show that the
"a" was raised, and verse 23:14 was accepted with the text "a".

The general rule (`Reader::only_note_labels`): a verse whose whole text is
labels of notes that name *this very verse* (a letter or sign that begins a
note on its page, or the next, giving the same chapter and verse) is a verse
the edition prints only in its apparatus. It has no text; the note stays a
note and is kept with the verse it names. A label that names another verse
is text ("il a" stays). Nothing names these three references: editions that
print the verses in their text keep them (the King James Version and the
World English Bible print Matthew 23:14; both are read with it).

A second, new false positive appeared during the work and is fixed by a
general rule too: Darby's Numbers 20:29 ends "… pleura Aaron 30 jours", and
"30" was the next number expected. A number standing in the middle of a
sentence is never a verse beyond the chapter's standard count (Numbers 20
has 29); only a number the document sets apart (a line's start, after
punctuation, raised) may go beyond it, as editions do number some chapters
differently.

## 2. Structure from the layout

| Finding (v0.24.2) | Cause | General evidence used now |
|---|---|---|
| Running heads in verses ("Le premier livre de Samuel 1:1 364 …") | Heads were recognised by a word pattern of at most four words | A line the page layout sets apart at its top or bottom edge (`LineRole::Head`/`Foot`, now `Unit::edge`) that names a passage and ends with a number, without clause punctuation (`edge_furniture`) |
| Verse numbers against the word ("5auf Kain", "15y sean") in converted files | The Word reader flattened runs; the PDF reader only marked raised *letters* | Raised digits are kept as superscript digits for the recogniser: PDF spans set smaller and lifted (`raised_digits`), Word runs with `vertAlign="superscript"` or a raised `w:position` (`NewSegment::marked`); a raised number is a verse candidate against the next word, and glued to the word before it only as the very next verse |
| Headings ending with a full stop, unstyled (Textbibel) | Nothing but space sets them apart | Space above a line is the layout's own block (`Line::block`, `Unit::gap_before`); a centred line after space with a verse number under it is a heading; an uncentred one is kept but its verse is not verified (§5) |
| Titles over several lines ("ΠΕΤΡΟΥ" / "ΕΠΙΣΤΟΛΗ ΚΑΘΟΛΙΚΗ" / "ΔΕΥΤΕΡΑ") | Only two-line titles were joined | Three consecutive large lines that together, and not two of them, name a book with chapter 1 following |
| Alternative title conventions ("Le premier livre de Moïse dit la Genèse") | "dit" was not a joining word for a second name | `ALSO_CALLED`: dit, dite, nommé(e) |
| Book introductions with numbered lists ("1. Adresse et salutations 1-2", 3 John) | An introduction's prose numbers ("3 Jean", "(9-10)") made the list look longer than chapter 1 | Only strong numbers (a line's start, after a sentence) are compared with chapter 1's verses |
| Missing titles making the next book wrong | — | Unchanged rule, now with more titles read: a book that begins twice is refused, never merged |
| Quotation sources set flush right ("… Deus conosco.      Isaías 7:14") made 58 pages uncertain (Portuguese 2018) | The wide space looked like two columns run together | A short piece flush with the column's right edge, after a closing mark (`flush_right_reference`); the recogniser already takes the source out of the verse |
| Columns set so close that lines join across the gutter (Clementine Vulgate, 1 460 uncertain pages) | One run held both columns' text | The second column's edge, where at least six lines begin, cuts runs that stand alone on their baseline (`column_edge`, `spans_cut`); a line crossing the gutter beside column text still makes the page uncertain |
| Older verse-per-line book codes ("SOL", "JOE", "1JO") | Unknown codes | The VPL convention's codes (`legacy_code`) |
| Korean "¹⁴장래에 …" refused as "chapter 14" | The unit rule (장, 절, 편) applied to raised numbers too | A raised number is set off by its look |

A pre-existing provenance fault was found by the new location check (§9):
when a long paragraph is kept as several segments, every line pointed at the
first segment. Lines are now placed in the segment that holds them, with
their character offset (`documents::line_units`, `Place::offset`).

## 3. Cross-format results

The converted Word and PDF files (LibreOffice from the editions' USFM) were
at 35–68 % because their verse numbers are raised by position, not by a
superscript style, and written against the next word. Comparing the
extracted units of `deutkw_usfm_lo.docx` with the Textbibel's eBible PDF
showed the same words, paragraphs and headings, with "1Im Anfang" where the
PDF has "¹ Im Anfang". Reading the raised runs brings these files to the
level of the publisher's own PDFs (§9).

## 4. Corrections the person owns

* **Stored** (`bible_recognition_corrections`): the document's SHA-256 and
  position, the heading as written, its page and segment, what the
  recogniser read it as, and the book chosen (or none: left out).
* **Kept at once** for a Bible already added (`bible_import_correct` with an
  `Existing` origin), and at commit for a new one — whether it is added by
  its verses or as a document.
* **Applied when read again** (`StagedImport::apply_saved`): by document,
  wording and place; when the document changed, by wording alone if found
  once. Each applied correction has a status the review shows: applied;
  earlier (the same documents added before as another Bible); document
  changed (applied, to be checked); read otherwise now (the person's choice
  kept, to be checked); not found (nothing applied, the correction kept).
* **Distinguishable**: a found book carries `corrected`; a verse found under
  a correction has `basis = 'corrected'` in `bible_verse_origins` and
  `bible_verse_places`.
* **Removed** only when the person sets the heading back to what was found.

## 5. Verification

A verse is verified when it is in a chapter recognised without doubt and its
text has none of these signs (`Recognition::unverified`):

| Reason | What it means |
|---|---|
| `possibleHeading` | A line that could be a heading stands in it: a whole line after a sentence's end, beginning with a capital and not ending as a sentence does, as its last line; a whole line in another typeface or larger after a sentence's end; a styled paragraph of its own (Word, EPUB); a line with space above it inside the verse; a word in capitals closing it ("… forsake me. BETH"); a label the document writes inside its lines ("… rooms. Friends We will …") |
| `reference` | A passage reference in it ("Samuel 1:17"), or the tail of a reference line ("11-15.)") |
| `noteSign` | A note's sign that no note explains |
| `unnumbered` | Verse 1 without a number of its own |
| `length` | More than five times its chapter's median length and over 500 characters |
| `psalmTitle` | A psalm's verse 1 on a line that names book, chapter and verse: an export may write the title into it |
| `boundary` | Where the verse begins or ends rests on weak evidence: a raised number against a word, a number alone on a PDF line, the next verse's number missing |

Further evidence for `possibleHeading`, added after the first full audit
showed headings left in verified verses: a heading over two lines (the lines
after a heading-like one never end a sentence before the verse ends); the
first line of running text on a page after a sentence's end; a short line,
flush with the column's left edge, after a short line that ended a sentence
(a paragraph of its own without a verse number, not indented as poetry); a
whole line just before a heading or a line of cross-references; several
words in capitals, or a number alone ("30."), closing a verse after a
sentence's end; a book's name alone closing it ("… mandou. Zacarias", a
quotation's source); a styled paragraph of a Word or EPUB file split by a
number ("맺는 말(1)").

Unverified verses are stored as places (`bible_verse_places`): the review
says how many and why; a passage in a document Bible lists them with their
page and a button to open them. A Bible read by its verses stays one: its
unverified verses are left out of its verses and shown in the chapter
reader in their place, marked as not checked, with their page.

## 6. Apparatus kept

`bible_document_apparatus` keeps every footnote, cross-reference line,
heading, subscription, running head, introduction and the text before any
book, with its document, page and segment and the verse it stands with
(notes are attached to the verse their chapter and number name on their
page). A passage in a document Bible shows its notes and headings, labelled
as the document's, never as verse text.

## 7. Difficult formats

See §9 for the figures.

* **Hebrew PDF**: still refused. The typesetter (XeLaTeX) inserted U+034F
  joiners after letters with dagesh, and draws the holam of holam male where
  it belongs geometrically to the letter before the vav ("מֹו" for the
  source's "מוֹ"). Both encodings look identical, so positions cannot decide.
  The Hebrew Bible reads correctly from EPUB.
* **Clementine Vulgate PDF**: see §2 (tight columns). Its psalms and some
  chapters follow the Greek numbering, which the standard counts do not:
  those chapters are refused, never renumbered.
* **Portuguese 2018 PDF**: see §2 (flush-right quotation sources).
* **Greek TR PDF**: three-line titles (§2).
* **Korean 1961 Word**: raised numbers before 장 (§2).
* **French single-book PDF** (Luke): one page whose reading order cannot be
  established; its chapter is refused.
* **One-verse-per-line export**: legacy codes (§2); psalm titles written
  into verse 1 cannot be separated (`psalmTitle`, found not verified).
* **OCR**: not built. Scanned Bibles are kept as documents to read page by
  page; no text is taken from them and none is substituted.

## 8. Database (migration 0023)

Additive: three tables (`bible_recognition_corrections`,
`bible_verse_places`, `bible_document_apparatus`) and two columns on
`bible_verse_origins` (`basis`, default `'automatic'`; `first_offset`).
Existing rows keep their meaning. Every new table cascades with its Bible.
Tests: a schema-22 database with a document Bible upgrades with a checked
copy made first and its origins read as automatic
(`tests/recognition_integrity.rs`); a failing migration after 0023 rolls back
completely and leaves 0023 intact.

## 9. Audit

All figures and per-verse evidence are in `docs/audit/v0.24.3/`:

| File | What it holds |
|---|---|
| `summary-baseline.md` | v0.24.2's recogniser (aa8fdd9) on the development and regression corpora |
| `final/summary.md`, `final/summary-run.md` | the final run on the same corpora, against the baseline |
| `final/verified-differences.md` | every verified verse whose text differs from the reference, classified |
| `withheld-b-first/` | withheld set B, its one blind evaluation, unchanged |
| `withheld-b-regression/` | set B after disclosure, used to find and fix general faults |
| `withheld-c/` | withheld set C, its one blind evaluation (the final generalisation test) |

**Development and regression corpora (46 documents, 21 editions).** No verse the edition
does not have, detected or verified. No edition below its baseline detection except where
v0.24.2 accepted wrong text: the French Segond PDF (30 550 against 30 568), where chapters
whose verse 1 an introduction's "1 Corinthiens …" took are now refused. Detection ≥ 98 % in
34 of 46 documents; verified text 85–100 % except the Hebrew PDF (refused) and the
Clementine Vulgate PDF (21.9 %); verification costs 0–4 %. The
converted files went from 35–68 % to 85–100 %. Remaining verified text that differs: 24
verses — 7 reference artefacts (the Textbibel's USFM encodes Scripture as `\s1`), 4 subscriptions
kept apart by design, 9 words a converter dropped, and **4 genuine contaminations** (3
Textbibel headings with no layout evidence, 1 inline speaker label). Location accuracy
97.5–100 %, except the French Segond PDFs (95.4 %; the Luke booklet 93.7 %).

**Withheld set B (11 documents, other generators and layouts), first evaluation:** 23
verified false positives in three documents, 1 734 verified verses with wrong text, and
coverage of 0–37 % in four documents. After disclosure and general fixes (set B used as a
regression set): no false positives, 58 verified differences (8 of them reference
artefacts), detection 70–100 % except the French Crampon EPUB (39.5 %), the title-less
Greek booklet (0 %, refused) and two plain-text/eBible files (83–86 %).

**Withheld set C (11 documents), final evaluation, run once:** read well in five documents
(German Word, eBible English PDF, Greek Markdown, Korean text: 98.9–100 % detected, no
errors), correctly refused two it cannot read (no verse numbers; Hebrew-letter numerals),
but **failed hard safety in three**: 73 verified false positives (KJV with verse numbers in
the margin, the Segond 21-like pandoc EPUB, the Portuguese A5 PDF) and 6 032 verified verses
with another verse's or a note's words, chiefly where verse numbers stand in the margin
(4 886) and where note callers are plain digits after words (929). Details:
`withheld-c/summary.md`.

**Checks.** `cargo fmt --check`, `cargo clippy --all-targets --locked -D warnings` and
`cargo test --locked` (all suites) pass; `npm run typecheck`, `lint`, `i18n:check`, `test`
(1 053 tests) and `build` pass. Library, Hebrew/Greek research, import cancellation,
migrations (0023 forward and rollback), batch and document workflows are covered by those
suites and unchanged in result.

## 10. Limitations and their effect on ordinary users

What remains, and what a person using the app sees because of it:

* **Hebrew Bible as PDF** (§7): refused; the person is told the pages'
  reading order or letters could not be established and is offered to keep
  it as a document. The EPUB of the same edition reads in full.
* **Layouts not yet handled safely** (withheld set C, §9): verse numbers in the margin,
  note callers written as plain digits after words, and some A5 PDFs. Verses can be accepted
  with a neighbour's words. Until this is fixed, a Bible added from such a document can hold
  wrong text marked as checked; the document itself is always kept and shown as printed.
* **Books without a title, verses without numbers, Hebrew-letter verse numbers**: refused
  (nothing read), never guessed.
* **Clementine Vulgate PDF** (22 % of verses): its pages set two columns so
  close, with a commentary beneath, that most pages' reading order cannot be
  established; their chapters are refused, never guessed. The EPUB reads
  95 %. The PDF stays readable as a document.
* **Headings in the body's own type with nothing to set them apart.** Where
  a heading differs from Scripture only by its words (no other typeface the
  file records, no space, not centred, not short after prose), it can be
  read as the end of the verse before it. Every rule in §5 narrows this; the
  remaining cases in the corpora are listed in `docs/audit/v0.24.3/` (two
  italic headings in the Textbibel PDF whose italic face has the same
  character widths as its upright face). The verse is then shown with the
  heading's words; the document view shows where it stands.
* **Source files that encode Scripture as headings.** The Textbibel's USFM
  marks some continuations of verses as section headings (`\s1`); the
  audit's reference text therefore lacks those words and counts the verses
  as "extra". They are the edition's printed text, not headings; they are
  listed separately in the audit and not counted as contamination.
* **Speaker labels written inline once** in a verse-per-line export of the
  Song of Songs ("… Lover Why do you desire …") cannot be told from a
  sentence's first word when a label occurs only once; one verse in the
  corpus keeps such a label.
* **Subscriptions** (colophons after a letter's last verse, "Escrita de
  Roma …") are kept as apparatus, not verse text; the audit's reference
  includes them in the verse, so these verses show as "missing" words. This
  is deliberate.
* **Words a converter left out.** LibreOffice's Word export of the Reina
  Valera 1960 lacks some letters' openings (Ezra 4–7, Daniel 4): the verses
  hold what the file holds; nothing is substituted.
* **Verification costs coverage.** Verses whose text is in doubt are kept
  as places, not as the Bible's text: 0–4 % of verses in the corpora. They
  remain readable in the document and in the chapter reader, marked as not
  checked.
* **Corrections are anchored to a document's place among the Bible's
  documents.** Re-ordering the documents when adding them again makes a
  correction "not found" (shown, kept), not silently misapplied.
* **Apparatus kept per Bible is capped** (4,000 characters per piece, 40
  million in all) so a commentary-heavy edition cannot exhaust the
  database. Beyond the cap the apparatus is not kept (the document's own
  text, which holds it, always is); the review does not yet say so.
* **OCR is not built**: scanned PDFs are kept as documents only.

## 11. Verdict

**NOT READY.**

The development and regression corpora meet the acceptance criteria except for four verified
contaminations in two editions, and every v0.24.2 finding (the Darby false positives, running
heads, raised numbers in converted files, lost corrections) is fixed. But the final
independent test, withheld set C, shows that the recogniser does not yet generalise safely:
on layouts it was not developed against (verse numbers in the margin, note callers as plain
digits, an A5 PDF with italic running heads) it accepts verses the edition does not have and
verifies text that belongs to another verse. The hard safety requirements — zero false
positives, zero wrong verse assignments and zero contamination in verified passages — are
not met there, and set B's first evaluation showed the same.

The owner's decision is needed on how to proceed: in particular whether a further milestone
should (a) add the missing layout evidence (margin verse numbers; plain-digit note callers
in EPUB and Word; drop numerals) and evaluate on a new withheld set, and/or (b) change the
product so that text recognised from a document is never used as a Bible's verses without
the person confirming a sample of it, as a safety net for layouts the recogniser has not
seen. Sets B and C are now disclosed and become regression sets; a new withheld set is
needed for the next independent test.
