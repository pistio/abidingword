# Phase A1.3: the 20 incorrectly verified and 2 nonexistent verses

These are the verified verses whose text differs from the edition, and the
verified references the edition does not have. They come from the A1.1
structural run over all 113 disclosed editions (A1.1 `5b2bbdd`; the
frozen `c96756d` recogniser for every non-EPUB document). Every one comes
from a PDF or DOCX document. The EPUB path had none after A1.1.

Each case was traced from the edition's USFM, through the derived
document (`scripts/build-derived-editions.py`), to the app's units and
items (scratch dump tool) and the verse the Reader closed.

| # | Edition (set) | Verse(s) | Origin | What the app does now |
|---|---|---|---|---|
| 1 | `deutkw-pdf` (dev) | 1KI 8:12, 1KI 8:13, PSA 60:13, ACT 2:14, 2CO 6:16 | Segmentation | Refused (`possibleHeading` / `boundary`) |
| 2 | `deutkw-lopdf` (dev) | 2KI 4:25, LUK 9:43 | Segmentation | Refused |
| 3 | `porbr2018-pdf` (dev) | GAL 6:18 | Segmentation (heading merge) | Refused (`boundary`) |
| 4 | `engasv-rrlab1col` (bc) | HAB 3:19 | Segmentation | Refused |
| 5 | `spav1602p-lodocx` (e) | PSA 41:7, PRO 4:16 (wrong); PSA 41:8, PRO 4:17 (nonexistent) | Reference assignment | Not verified |
| 6 | `spablm-docx` (dev) | EZR 4:11, 4:17, 5:7, 7:12; DAN 4:1, 4:19, 6:25, 9:4, 9:11 | Omitted source text (corpus derivation) | Unchanged: the document lacks the text |

## 1–2. `deutkw`: headings in another face read as verse text (7 verses)

**Source.** The Konkordante / deutkw edition puts section headings
(`\s1`) in the middle of verses. Example: `\v 12 Damals sprach Salomo:`,
then `\s1`, then the poetic line. The derived PDFs set `\s1` as a line of
its own in a different face. Its letters mostly share the body's
widths.

**How it was accepted.** The line is neither `large` nor `heading`. Its
place does not set it apart: it starts at the left margin of the column,
like a paragraph. Before A1.3, a line had to be `styled` (a whole line in
another face or place) for `set_apart_line` to consider it, and the verse
had to end its sentence (`.`, `!`, `?` …). Here the verse before ends
with `:` or `,`, so the heading was read as the verse's own continuation.
Example: "Damals sprach Salomo: Die Sonne stellte er ans Himmelszelt, …"
for 1KI 8:12, whose edition text is only "Damals sprach Salomo:".

**Fix (`structure.rs`, `set_apart_line`; `pdf_layout.rs`).**

* `Line.partly_styled` records that at least 40 % of the line's measured
  glyphs (at least 6) have widths unlike the body's.
* A line both `styled` and `partly_styled`, after a line that was not
  styled, after a colon, comma or semicolon, is set apart. The verse is
  then not known to end there and is not verified.
* A `partly_styled` line that begins a block of its own (a new segment on
  the page) is considered as well. This is how `deutkw-lopdf`'s
  "Berg Karmel." / "Als sie aber der Mann Gottes …" is caught.

Exemptions, each found because the rule first refused correct verses:

* a line opening a quotation (eBible's red-letter PDFs);
* a whole paragraph set in another face (eBible book editions);
* a poem's centred line, which is `styled` by its place but not by its
  letters.

**Tests.**

* `a_whole_line_in_another_face_after_a_colon_withdraws_the_verse`, with
  poem and spoken-quote controls that stay verified;
* `a_line_perhaps_in_another_face_beginning_a_block_withdraws_the_verse`.

## 3. `porbr2018-pdf` GAL 6:18: the verse's end merged into the next book's heading

**Source.** The edition's GAL 6:18 is "A graça … irmãos. Amém!", followed
by the colophon "Escrita de Roma para os Gálatas". The derived PDF sets
"Amém!Escrita de Roma para os Gálatas" on the line after the verse,
before Ephesians' title.

**How it was accepted.** The Builder read "Amém!Escrita de Roma para os
Gálatas" and "Carta de Paulo aos Efésios" as one book heading. The verse
closed at "irmãos.", a sentence end, and was verified without "Amém!".

**Fix (`structure.rs`, `book()`).** When a new book's heading does not
begin with its own title line, it took in a line before the title. If
that line comes right after the open verse's last line, on its page, in
the body's size and not at the page's edge, the open verse is doubted
(`boundary`). A title over two lines whose first line is in the body's
size ("THE BOOK OF" / "EZRA" in `engasv`), on a page of its own or after a
gap, takes in nothing of a verse and is not affected.

**Test.** `a_line_after_a_book_s_last_verse_read_into_the_next_book_s_heading_withdraws_the_verse`.

## 4. `engasv-rrlab1col` HAB 3:19: the musical direction in smaller type

**Source.** HAB 3:19 ends "… upon my high places." It is followed by the
`\d`-like note "For the Chief Musician, on my stringed instruments.". The
derived PDF sets the note on the next line in smaller type.

**How it was accepted.** The note is a body-coloured line in a slightly
smaller size, not `styled`, not `heading`. It was joined to the verse.

**Fix (`documents.rs`, `set_apart_line`).** `Unit.small`: a text line at
most 0.96 × the body's size. After a verse has ended its sentence, such a
line is set apart and the verse is withdrawn.

**Test.** `a_line_in_smaller_type_after_the_verse_ended_withdraws_it`.

## 5. `spav1602p-lodocx`: a verse number the document does not mark

**Source.** In the DOCX, every verse number is a run with a style of its
own: smaller than the body and not superscript. At PSA 41:7 and PRO 4:16
the edition's own USFM lacks `\v 8` and `\v 17`. The numbers "8" and "17"
stand in the text as plain characters ("… diciendo de mí: 8 Cosa
pestilencial …"), so the comparison edition has no PSA 41:8 or PRO 4:17.

**How it was accepted.** The Reader takes a plain number inside a line as
a verse number when it is the expected next one. That made "8 Cosa …" a
verse 8 the edition does not have, and cut verse 7 short.

**Origin.** Reference assignment. The document's own evidence (the
number style) says these two numbers are not marked as verse numbers.

**Fix.**

* DOCX (`docx.rs`, `number_formats` / `mark_number_formats`): a run
  format used for at least 20 digit-only runs and no letters, and smaller
  than the body, is the document's verse-number style. Runs in it are
  marked as raised numbers.
* Reader (`raised_numbers`): in a document whose verse numbers are raised
  (at least 50 raised, at most one plain-in-line for every 19), a plain
  number inside a line is not taken as proof of a verse.

Both verses, and the two references, are now unverified. They are not
assigned differently: the app does not decide what the edition omitted.

**Tests.**

* `a_word_document_s_own_verse_number_style_is_the_evidence_for_its_verse_numbers`;
* `a_plain_number_inside_a_line_of_a_document_that_raises_its_numbers_is_not_shown_to_be_a_verse`.

## 6. `spablm-docx`: text left out of the document by the corpus derivation (9 verses)

**Source.** In the edition, each of these verses continues after a colon
in a `\mi` (or similar) paragraph: the letter, decree or prayer quoted.
Example: `\v 11 Esta es la copia de la carta que enviaron:` then
`\mi Al rey Artajerjes, …`. `scripts/build-derived-editions.py` did not
list `\mi` (and several other paragraph markers) as paragraph tags. The
continuation was **dropped from the derived DOCX**. The document the app
reads ends the verse at the colon.

**How it was accepted.** The document's text is exactly the verse up to
the colon, followed by the next verse number. Nothing in the document
shows that text is missing. The app read the document correctly.

**Origin.** Omitted source text, in the test corpus, not in the app.

**What was done.**

* The derivation script now lists `mi`, `pm*`, `pc`, `q3`, `qm` and the
  other paragraph markers.
* A rebuilt `spablm-docx` from the corrected script (scratch,
  SHA-256 `0cb50d10…`; the corpus file is `2ad1094b…`) reads with 0 wrong
  verified verses, on A1.1 and on A1.3 alike.
* **The corpus file itself is not changed.** It is checksummed evidence
  (`docs/audit/v0.24.7/corpus-checksums.sha256`). Replacing it is the
  owner's decision.

**Why the app does not refuse these verses.** It would mean refusing
every verse that ends with a colon before the next verse number. In the
same corpus that is thousands of correct verses ("… saying:"), and no
document evidence distinguishes them. No regression test is possible on
the app side; the script change is the correction.
