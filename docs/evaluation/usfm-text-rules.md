# Reference verse text: rules `aw-usfm-text-1`

Evaluation candidate A1.5-E3. How the evaluation reads a reference edition's
USFM to obtain each verse's **exact printed text**. Two readers implement
these rules apart, sharing no code, table or regular expression with each
other or with the app's USFM reader:

| Reader | Where | Method |
|---|---|---|
| Harness (Rust) | `src-tauri/tests/eval_support/usfm_text.rs` | one streaming pass, a stack of open spans |
| Check (Python) | `scripts/evaluation/usfm_text.py` | a lexer, a tree of paragraphs and spans, a walk |

The scorer compares the two readings verse by verse (label, status, line and
SHA-256 of the text); any difference makes a run *not a protocol run*. The
rules follow the USFM 3 specification (https://ubsicap.github.io/usfm/;
"Syntax Notes", "Milestones", "Word Level Attributes", "Chapters and
Verses", footnotes and cross references). Where the specification leaves a
choice, the choice is stated here and is the same in both readers.

Why: E2 read the reference through the app's reader, which puts a space
where a note was removed ("heel. ”" for the edition's "heel.”") and, after an
unclosed note or alternate number, hides verses. The independent E2 review
found 528 such verses in the 13 bundled references (R1).

## 1. Characters and white space

1. A leading byte-order mark is not text. CR and CRLF are read as LF.
2. **USFM white space is U+0020, U+0009 and LF only** (specification,
   "Whitespace"). Every other character is text and is kept exactly: a
   no-break space (U+00A0), a narrow no-break space (U+202F), a soft hyphen,
   joiners, marks, compatibility forms. Nothing is normalised (no NFC).
3. `~` is a no-break space (U+00A0). `//` (an optional line break) is
   nothing.
4. A verse's text is the concatenation of its pieces with each run of USFM
   white space made one U+0020 and both ends trimmed.

## 2. Markers

1. A marker is `\`, an optional `+` (nested span), a name of ASCII letters
   and digits, optionally `-s`, `-e` or `-` and digits, and an optional `*`
   (end marker). A backslash that starts no name is a problem.
2. **The one white space character after an opening marker belongs to the
   marker**, not to the text (specification: "the opening marker ends with
   the next space character"). End markers take none.
3. A marker followed (after that one character) by nothing, or by an
   attribute list `|...`, and then `\*` is a **milestone** (`\qt-s |who="A"\*`,
   `\ts\*`, `\zaln-s |x-strong="H1"\*`): not text. A `-s`/`-e` marker without
   `\*` is a problem; so is a `-s` without its `-e` (same name and level) in
   the file, an `-e` without its `-s`, and a `\*` that ends no milestone.
4. Markers by role (name without level digits and without `-` suffix):

```aw-usfm-text-1 markers
body: p m po pr cls pmo pm pmc pmr pi mi nb pc ph b q qr qc qm qd lh li lf lim tr
other: usfm ide sts rem h toc toca mt mte ms mr s sr r d sp sd cl cp cd qa lit periph restore imt is ip ipi im imi ipq imq ipr iq ib ili iot io iex imte ie
cell: th thc thr tc tcc tcr
note: f fe x ef ex
note part: fr fq fqa fk fl fw fp fv ft fdc fm efm xo xk xq xt xta xop xot xnt xdc
kept: add bk dc k nd ord pn png addpn qt sig sls tl wj em bd it bdit no sc sup ndx rb pro w wg wh wa jmp qs qac litl lik liv ior iqt
dropped: va vp ca fig rq cat
```

   `\id`, `\c`, `\v`, `\esb`, `\esbe` and `\pb` have their own rules (§3,
   §4). Any other marker is a problem (`marker_unknown`); a `\z` marker that
   is not a milestone is a problem (`marker_private`): its text cannot be
   placed.

## 3. Structure

1. `\id CODE`: the book is the first word, in ASCII upper case; the rest of
   the line is a description. More than one `\id` in a file is a problem.
2. `\c N`: the chapter is the first word of the text right after the
   marker. Before the chapter's first verse only white space, an alternate
   chapter number (`\ca`), notes and milestones may follow; anything else is
   a problem (`text_after_chapter_number`).
3. `\v N`: the verse is the first word of the text **right after** the
   marker; if a marker, a bar or only white space comes first, the number
   is missing (a problem) and the verse is still kept, unlabelled. A verse
   before any chapter is counted, not read. The verse's text runs to the
   next `\v`, `\c` or `\id`.
4. A **body** paragraph continues the open verse, with one space at its
   boundary; with no verse open in a chapter, its text is a problem
   (`text_outside_verse`). An **other** paragraph (title, heading, speaker,
   chapter label, introduction, `\qa` acrostic heading, `\d` psalm title) is
   never verse text, but a `\v` inside it starts a verse whose text is read
   (a title numbered as verse 1). A table **cell** and `\pb` are one space.
5. A sidebar (`\esb` ... `\esbe`) is not verse text; one left open, nested,
   or ended without a start is a problem.
6. **Every structural boundary closes what is open**: at `\v`, `\c`, `\id`,
   a paragraph, a cell and the end of the file, each open note, span or
   sidebar is closed, and each one (other than a note part) is a problem
   (`note_unclosed`, `char_unclosed`, `dropped_unclosed`,
   `sidebar_unclosed`). So an unclosed note or alternate number can never
   swallow the verses after it: they are read, and the reference is refused.

## 4. Notes, spans and attributes

1. **A note (`\f`, `\fe`, `\x`, `\ef`, `\ex`) is removed with its content and
   nothing is put in its place.** The white space the source has before and
   after it stays as it is (specification, "Whitespace Normalization" 5).
   So `heel.\f + \ft ...\f*”` reads `heel.”`, `word \f ...\f* next` reads
   `word next`.
2. A note ends at its own end marker. A note part (`\fr`, `\ft`, `\xt`...)
   closes the part before it and is closed by the note's end; a nested `+`
   span inside a note must be closed before it. A note inside a note, a
   note closed by another type's end marker, and an end marker with no
   note open are problems. A note part outside a note, in verse text, is a
   problem.
3. A **kept** span keeps its text; a **dropped** span (`\va`, `\vp`, `\ca`,
   `\fig`, `\rq`, `\cat`) does not. The text of `\va` and `\vp` is recorded
   beside its verse (alternate and published numbers), and of `\ca` beside
   its chapter; `\cp` is recorded as the chapter's published number. They
   never change a verse's or chapter's identity, which is always the `\v`
   or `\c` number.
4. A span ends at its own end marker, with the same name and `+`. Another
   end marker, an end marker with no span open, and a span opened inside
   another without `+` are problems.
5. Inside a span, `|` starts its attribute list, which runs to the span's
   own end marker and is not text (default attributes, `lemma`, `strong`,
   ruby glosses). Any other marker inside an attribute list is a problem;
   a `|` in verse text outside any span is a problem.

## 5. Accounting

1. Every `\v` marker becomes a verse or a verse before any chapter. Each
   reader also counts the `\v` markers in the source the plainest way (a
   backslash, `v`, and nothing that could continue a marker name) and
   reports `verse_markers_unaccounted` if the two differ.
2. A control character (Unicode Cc) in verse text is a problem.
3. A reference with any problem is refused by check-set.py before the seal
   and before the run, and is a reference problem in the harness's report
   (not a protocol run). A problem never drops a verse: the readers recover
   and go on, so that every problem in a file is reported.

## 6. What is outside these rules

* Book, chapter and verse **classification** (scorable, unsupported, outside
  the declared books; bridges; duplicates) is the inventory's
  (`aw-reference-inventory-2`), applied to the labels read here.
* Verse **comparison** with a document is the fidelity rule
  (`aw-fidelity-2`), unchanged from E2.
* The app's own importer and USFM reader are not changed: the review's
  finding that a person importing the bundled Bibles reads the inserted
  space in 528 verses is a product defect recorded for the owner, outside
  this evaluation and outside the frozen recogniser.
