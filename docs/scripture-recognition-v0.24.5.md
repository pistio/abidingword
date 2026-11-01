# Scripture recognition v0.24.5 — general rules for set D's failure classes

v0.24.4 ended NOT READY (`docs/scripture-recognition-v0.24.4.md`, §6): its
independent set D, run once at the frozen recogniser `17398c6`, verified
1 196 verses with wrong text in six of eleven documents and filed 898
Septuagint verses under Hebrew-canon book codes. Set D's original blind
evidence stays unchanged in `docs/audit/v0.24.4/withheld-d/`; from this
milestone on sets B, C and D are **disclosed regression sets**, and the
independent measure is a new set E (§6).

Branch: `claude/bold-noether-n8l5ds` (the session's designated development
branch), started from v0.24.4's verified head `2b5d4e2`. Nothing is merged,
published or deployed.

## 1. Evidence available for the disclosed sets

The generators of sets B, C and D were kept outside Git in the sessions that
made them and are gone; their manifests (`docs/corpus/withheld-{b,c,d}.tsv`)
and evaluations describe each document's layout precisely. eBible.org has
also rebuilt most of the development corpus since v0.24.4 (the fetch script
reports new checksums for most files), so the recorded v0.24.4 figures are
for other builds of the same editions.

What was done about it:

* **A fresh baseline.** The frozen v0.24.4 recogniser (worktree at
  `2b5d4e2`, recogniser code identical to `17398c6`) was run on the current
  corpus files; every v0.24.5 comparison below is against that baseline on
  the same files, never against the recorded v0.24.4 numbers.
* **Reconstructions of the disclosed layouts.** `scripts/recon/` rebuilds
  documents in set D's described layouts from the same eBible USFM
  (`docs/corpus/recon-d.tsv`, role `recon-d`) and, for sets B and C,
  `recon-bc.tsv` (§4.3). They are *not* the sealed files: other program
  versions, other checksums, sometimes another page size. Results on them
  are regression evidence only. They reproduce set D's failure classes at
  the baseline (table in §4.2), which is what makes them useful.

## 2. Set D's failure classes and the general rules now used

| Class (set D) | General evidence used now | Direction |
|---|---|---|
| **Running heads not set apart**: a long book title in capitals with its number ("DAS 1. BUCH MOSE (GENESIS)"), a work title over every page ("THE SEPTUAGINT VERSION … BRENTON"), a Korean "book c–book c" head | (a) a long line without figures, not ending as a clause, repeated word for word at the pages' edges is furniture (a refrain ends as a clause; a verse carries its number); (b) a line at a page's edge, not large, that names a book and nothing else (whatever figures its title has), on a later page than the book's title — or the first time, with running text going on in lower case or past verse 1 after it — is a running head; (c) heads naming the page's first and last chapter are read across a dash | fail-closed (text out of verses) |
| **Page numbers** read into text, or read as chapter numbers | a number alone that runs with the page count is the page's number wherever the page reads it (a footer set larger than the text and read between the columns) | structural; can raise coverage (the chapter is no longer cut) |
| **Verse numbers set against their first word** ("23que", "27»La", "11\`우리", "59aTheco", "… 30missing") | a verse whose text holds its own, its bridge's or the next verse's number against a word (or bare at its end) is found, not verified; its own number inside it puts the verse before in doubt; a line opening with a number out of the count leaves the open verse unverified | fail-closed |
| **Footnotes read into verses** in converted PDFs ("1993:2 a Gr. fire of flame.") | a line opening with a note's number run into its reference (a "chapter" above 150, no book has one) is a note to the end of its column | fail-closed |
| **Inline study / theme labels** ("*TEMA: …*") | the Markdown reader marks a paragraph written wholly in emphasis as set apart (as EPUB and Word already do), and the existing set-apart evidence applies | fail-closed |
| **Truncated verses, neighbours merged** | text taken out while a verse is open and does not end as a sentence leaves that verse unverified; a body-size line of two letters or one Korean syllable at a page's foot is text, not a note ("더라"); a line taken as a note with neither figure nor sign, only words ending a sentence, puts the verse before in doubt | fail-closed, except the foot-line rule (structural; restores text that was cut) |
| **Septuagint Esther and Daniel filed as `EST`/`DAN`** | §3 | fail-closed (books left out) |

Found on the set B/C reconstructions and fixed the same way: letter-spaced
chapter numbers ("C A P Í T U L O 1 3" was read as chapter 1); notes inline
in square brackets with a reference and words; a bare "7:1" read into a
verse; a line at a page's edge after the page's notes (held out, and the
verse it may end left unverified); "Selah" never page furniture and never
the first word of a verse; words broken at a line's end with U+2010 or a
soft hyphen joined (verified text read "inherit‐ ance").

Every rule is general: none names an edition, a publisher or a document.
Most move text from verified to unverified or a chapter to refused; four are
structural corrections that can also add verified text (page numbers read
anywhere on the page, the foot-line rule, four books of Kings, letter-spaced
numbers). An earlier commit message (`c275985`) said every rule only fails
closed; that was too broad, and this record is the correct statement. Every
added verse is covered by the regression runs below (no new contamination
or false positive).

## 3. The Septuagint: book identity, canon and versification

The model keeps three things apart (`canons.rs`): which books a tradition
counts, their order, and how chapters and verses are numbered. The app
stores verses under the 66-book registry (`canon.rs`) with Hebrew-canon /
English versification; books outside it (`TOB`, `1MA`, `ESG`, `DAG`, …) are
named and left out (v0.24.2), never stored.

What set D showed is an **attribution** error, not a textual one: Brenton's
Greek Esther (with its additions, lettered verses such as 1:1b) and Greek
Daniel (with the Song of the Three, Susanna and Bel; Daniel 3 has 97 verses)
were read as the Hebrew canon's Esther and Daniel because the canon has no
other Esther or Daniel and nothing in the recogniser recognised a Greek
edition. The recogniser now decides on evidence that does not depend on the
text of those books:

1. **A title that says so** ("ESTHER (GREEK)", "ΔΑΝΙΗΛ (ΕΛΛΗΝΙΚΆ)",
   "Daniel (grego)"): the Greek book, even where the Hebrew one has not
   appeared (before, only a second Esther with "Greek" beneath it counted).
2. **A document that declares itself a Septuagint** in a title line of its
   front matter or a running head — never in a note ("The Septuagint reads
   …") or under one book's title ("Ester / traduzido da Septuaginta grega",
   which names that one book as Greek).
3. **A Greek Old Testament**: three or more Old Testament books named in
   Greek ("ΓΕΝΕΣΙΣ", "ΒΑΣΙΛΕΙΩΝ Αʹ 1:1 …").

In such a document Esther and Daniel are `ESG` and `DAG`: named, their text
left out. Two further consequences of the Septuagint's own numbering, found
while inspecting the model:

* **Four books of Kings.** Brenton titles Samuel "KINGS I." and "KINGS II."
  and Kings "KINGS III." / "KINGS IV." (as the Septuagint and the Douay
  tradition do). The dictionary read "KINGS I." as 1 Kings, so 1 Samuel's
  chapters were filed under 1 Kings and verified where the verse counts
  happened to fit. Now, where the document has a title for a third or fourth
  book of Kings (the number beside the name, no chapter and verse), its first
  two are 1 and 2 Samuel.
* **Chapters the Greek numbers otherwise.** A Septuagint's Psalm 22 is the
  Hebrew Psalm 23; its Jeremiah 26–51 stand in another order; Proverbs 24–31
  and Exodus 35–40 move passages; 3 Kingdoms (1 Kings) adds, moves and swaps
  passages. Under the app's numbering those verses would be filed under
  other verses' numbers, so in a Septuagint those chapters (Psalms 9–147,
  Jeremiah 25–52, Proverbs 24–31, Exodus 35–40, all of 1 Kings) are refused
  with a message of their own (`bibleImport.documentSeptuagintNumbering`, six
  languages). This is a property of the tradition's versification, applied
  only on the evidence above — not an edition-specific exception.
* A line in a book-chapter-verse document that names its book is read as a
  verse however far beyond the chapter's standard count it goes ("ΨΑΛΜΟΙ
  103:33"): read as text, it joined the verse before it; read as a verse,
  the chapter's length refuses it. A line naming a book outside the 66
  ("ΤΩΒΙΤ 1:1 …") begins that book and is left out with it.

Original edition identifiers and historical data are untouched: no
identifier, table, column or migration is added; existing Bibles keep their
rows; an edition whose own USFM files Greek portions under `DAN`/`EST` (the
Clementine Vulgate) is unaffected, because nothing in it declares a
Septuagint — its chapters longer than the canon are refused as before.
A Greek translation made from the Hebrew would lose its Esther and Daniel
(coverage lost, never text misfiled).

## 4. Regression evidence (recogniser frozen at `fc0f538`)

All against the fresh baseline on the same files (§1). Tables and every
remaining difference: `docs/audit/v0.24.5/`.

### 4.1 Development corpus (46 documents) — `dev-regression.md`

| | Baseline (v0.24.4 code) | v0.24.5 |
|---|---|---|
| Detected | 1 253 560 | 1 253 558 |
| Text-verified | 1 241 934 | 1 241 524 (−410, −0.03 %) |
| Verified verses the edition does not have | 0 | 0 |
| Verified with wrong text | 20 | 20 — the same verses: 7 Textbibel headings-as-Scripture in its USFM, 4 subscriptions kept apart by design, 9 words the converter dropped (`verified-differences.md`) |

The 410 verses are fail-closed costs, spread over 23 editions (largest:
Darby French PDF −99, Vulgate PDF −79, Darby LibreOffice PDF −51, WEB PDF
−44).

### 4.2 Set D layouts, reconstructed (7 documents) — `recon-d-regression.md`

| | Baseline | v0.24.5 |
|---|---|---|
| Detected / text-verified | 108 115 / 104 503 | 115 068 / 111 046 |
| Verified verses the edition does not have | **810** (Septuagint Esther/Daniel as `EST`/`DAN`, both Brenton editions) | **0** |
| Verified with wrong text | **1 138** (German 254, Brenton 161, Korean 621, Portuguese 31, Spanish EPUB 69, Greek 2) | **0** |

Coverage gained: Brenton PDF +6 472 (page numbers no longer read as chapter
numbers, four books of Kings), German +758, Korean +738. Coverage lost
honestly: Greek Septuagint text −1 643 (its Septuagint-numbered chapters and
Greek Esther/Daniel are now refused or left out), Spanish EPUB −71 and
Portuguese Markdown −31 (verses holding a glued number or a theme line are
no longer verified). The reconstructed Spanish DOCX reproduced no failure.

### 4.3 Sets B and C layouts, reconstructed (21 + 2 eBible originals) — `recon-bc-regression.md`

| | Baseline | v0.24.5 |
|---|---|---|
| Detected / text-verified | 399 182 / 369 163 | 398 910 / 369 512 |
| Verified verses the edition does not have | 13 | 0 |
| Verified with wrong text | 120 | 22 |

The reconstructions exposed failures already present in the v0.24.4
recogniser on layouts close to B and C's (Revelation 13–18 shifted in the
Portuguese A5 PDF by a letter-spaced chapter number; notes in brackets; a
note's last line at a page's foot); those were fixed generally (§2). Of the
22 left, 17 are reference-encoding artefacts of the kinds v0.24.4 already
classified, and **5 are genuine one-word contamination** in the BSB
reconstruction: right-aligned "Selah" lines near the column gutter read with
the other column (3 verses lose their "Selah", 2 gain one mid-sentence).
This was classified after the freeze and is left as an open finding.

### 4.4 Determinism

Three PDFs (Brenton two-column, Korean two-column, German drop-cap
reconstructions) read twice at `fc0f538` in separate processes: the audit
JSON (every verse, difference, refused chapter, apparatus count and
unverified reason) is identical; only the reference file's temporary name,
which carries the process id, differs.

### 4.5 Checks at the frozen commit

CHECKS_PENDING

## 5. Interface contract with the provisional-import work

`docs/design/recognition-interface-contract-v0.24.5.md` reviews what the
recogniser hands over and what Codex's migration 0027 and consumer gate must
keep distinct: structured-source verses, recognised provisional verses,
individually unverified verses (places) and the original document text. No
migration is implemented here; the integration lineage ends at 0026
(`e2564b5`) at this review.

## 6. Independent evaluation (set E) — `docs/audit/v0.24.5/withheld-e/`

Recogniser frozen at `fc0f538`; set E built afterwards by an independent
agent from a brief that names no rule or earlier result
(`set-e-brief.md`, verbatim), sealed at 23:17 UTC (manifest SHA-256
`18d14545…5846`, committed `61d98ad` before the run), run once.

| | |
|---|---|
| Documents | 12 (5 PDF, 2 EPUB, 3 DOCX, 1 Markdown, 1 text; 8 languages) |
| Printed / detected / text-verified | 130 762 / 95 531 (73.1 %) / 84 953 (65.0 %) |
| **Verified verses the edition does not have** | **13** (Greek text 7, Portuguese Markdown 4, Spanish DOCX 2) |
| **Verified verses with wrong text** | **892** (Portuguese Markdown 700, Greek text 120, Tyndale PDF 68, Spanish DOCX 2, WEB PDF 1, Korean PDF 1) |
| Clean | Geneva DOCX, Ostervald EPUB, Reina Valera Gómez DOCX; honest rejection: Hebrew RTL PDF, Apocrypha-only EPUB, margin-number PDF |

Failure classes, all new relative to earlier sets: **Markdown footnotes**
(pandoc `[^id]` callers left in 700 verses; four footnote definitions
"**17:36** Alguns manuscritos gregos acrescentam: …" verified as verses the
edition does not print), **Greek alphabetic chapter numerals** ("ΚΕΦΑΛΑΙΟΝ
ΙΓʹ": Revelation 13–18 shifted, 120 + 7), a **"Book Chapter N" running
head** (68), two **cut verses**, and two verses where the edition's own USFM
lacks a `\v` marker the printed page shows (source artefact). No Esther,
Daniel or other book was misfiled. Set E is a disclosed regression set from
now on; no later run on it is independent.

## 7. Verdict

**NOT READY.**

* The six set D failure classes are addressed by general rules: on
  reconstructions of set D's layouts the recogniser goes from 1 138 wrong
  verified verses and 810 misfiled Septuagint verses to none, without
  contamination or new false positives in the development corpus (same 20
  classified differences, −0.03 % verified).
* But the independent set E, built after the freeze and run once, verifies
  892 verses with wrong text and 13 verses the editions do not have, through
  three classes no earlier set showed; and 5 one-word contaminations remain
  on the set B/C reconstructions. Five consecutive first evaluations (B, C,
  D, E) have now each found new layouts the rules did not cover.

Recommendations:

1. Treat text read from documents as **provisional**, as the v0.24.4 design
   requires (`design/provisional-bible-imports-v0.24.4.md`; contract review in
   `design/recognition-interface-contract-v0.24.5.md`), before any such Bible
   is offered; this milestone's evidence strengthens that case.
2. The v0.24.5 recognition commits are merge candidates for the recognition
   line (every regression set improved; owner's decision; nothing merged).
3. A next round, if wanted, has concrete general targets: Markdown footnote
   syntax (the reader should know `[^id]` callers and definitions as
   notes), alphabetic chapter numerals with keraia, heads of the form
   "Book Chapter N", and lines placed across a column gutter.
