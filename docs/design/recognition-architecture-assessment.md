# Bible recognition: an architecture assessment after set G

**Status:** an assessment for the owner's decision. It is not a plan of
record. Nothing here is implemented, and no recognition change follows from
it without approval. It is written after set G (`docs/audit/set-g/summary.md`)
and uses the evidence of the independent sets B–G. Coordination: Codex owns
A.4 integration and the provisional Bible-import safeguards. Migration 0027
(PDF structure, A.3) is preserved, and 0028 is reserved for those
safeguards. Nothing proposed here takes a migration number before it is
agreed with Codex.

## 1. Requirements (owner, verbatim in substance)

1. Non-AI extraction remains the fully functional default.
2. AI assistance is explicitly opt-in.
3. Local AI is considered before cloud processing where practical.
4. No model-generated Scripture text is ever accepted as an original
   document quotation.
5. Every passage stays linked to its original evidence (document, page or
   flow, character range).
6. Automatically recognised Scripture stays provisional until the
   application's independent safety requirements are satisfied.

## 2. The current pipeline

```text
file ─► format adapter ─► units (text + style + place) ─► rule-based recogniser ─► checks ─► provisional import
        pdf-extract/lopdf   page, position, size,         structure.rs (~8 900 lines):   verse counts,
        EPUB/DOCX/MD DOM    bold, centred, edge …         furniture, headings, markers,  sequence, issues
                                                          book names, numbering
```

Strengths, shown by every set:

* **Strong refusal.** In six blind sets most failures were refusals, not
  wrong text.
* **Evidence-linked output.** Each verse is a slice of the document's own
  text with its place, so no text is generated.
* **Structured input.** The well-structured formats are read almost
  perfectly: Markdown, EPUBs with clean markup, and DOCX with styles. In set
  G, G11 verified 99.9 % and G12 68.9 %, with no wrong text.

Weaknesses, shown by every set:

| Evidence | What it shows |
|---|---|
| Each blind set (B, C, D, E, F, G) found **new** accepted-error classes. In G these were note callers left as hyperlink text, and callers read as verse numbers. | The recogniser decides locally, line by line, with rules. A new convention can satisfy all the local tests and still be wrong. Every revision fixed the disclosed classes; none prevented the next one. |
| Real published files cover little. In set G they verified 28 %, 0 %, 0 % and 0 %. | Rules written against rendered test documents generalise poorly to real typesetting: drop numerals, unnumbered first verses, right-to-left text, other languages' titles. |
| G05: a valid groff PDF could not be opened at all. | The PDF layer (`pdf-extract` 0.12 on `lopdf`) is the weakest component. Other engines read the same file. |
| Rules keep growing (v0.24.4 → v0.24.7: several hundred lines per round). | Each rule interacts with the others. Regressions were found only by rerunning whole corpora. |

**Conclusion.** The failure mode is architectural, not a matter of missing
rules. The design (1) discards structural evidence that the source formats
already carry, (2) reads geometry through a weak PDF layer, and (3) commits
to verse boundaries greedily instead of choosing a whole-chapter
interpretation and checking it.

## 3. Stronger non-AI techniques

They are ordered by expected value per unit of effort and risk. All are
deterministic, run offline and generate no text.

### 3.1 Use the structure the document already declares

* **EPUB / HTML:** treat what a hyperlink *points to* as evidence.
  * An `<a>` whose target is a note, an endnote file or an element
    containing note text is a note caller, whatever its visible text
    ("[1]", "*", "a"). That would have prevented all of set G's failures.
  * Also read `epub:type`, ARIA roles (`doc-noteref`, `doc-footnote`),
    CSS classes repeated with a role (`fnanchor`, `verse`, `chapter`), and
    `id` patterns that encode references ("Gen8-1").
  * Treat an id that encodes a reference as **corroborating** evidence,
    never as text.
* **DOCX:** use footnote and endnote references, paragraph and character
  style names, outline levels, fields and bookmarks.
* **PDF:**
  * the tagged structure tree (`StructTreeRoot`, `/ActualText`), when
    present;
  * the outline (bookmarks) for books and chapters;
  * link annotations that jump to notes;
  * page labels.

  Today these are read partially or not at all.

### 3.2 A mature PDF engine, with geometry kept

* Read PDFs with a mature engine (pdfium or MuPDF through Rust bindings, or
  poppler), with the current reader as a fallback. Either engine opens files
  like G05 and gives each glyph a box, font, size, colour and rotation.
* Lay the page out with established, documented algorithms. Each is
  measurable on its own, unlike interleaved rules:
  * whitespace-cover column detection (Breuel) or recursive X-Y cut;
  * baseline clustering;
  * Docstrum-style line building;
  * the Unicode bidirectional algorithm (UAX #9) for right-to-left text;
  * glyph-gap statistics for letter-spacing;
  * font-size outliers for drop numerals.

### 3.3 Per-document style statistics

Cluster lines by font, size, weight, colour, indent and position. Note what
repeats from page to page at the same place: that is page furniture. The
document then tells which style is body text, verse number, heading, note
or running head, with no global thresholds. Most v0.24.x layout rules are
special cases of this.

### 3.4 Global interpretation of a chapter instead of greedy markers

* **Candidates.** Generate every candidate verse and chapter marker:
  numerals in any supported numeral system, with their style class.
* **Search.** Choose the most consistent sequence with dynamic programming
  (a Viterbi-style search). The constraints are:
  * numbers rise by one;
  * the versification table's chapter lengths;
  * verse-length plausibility;
  * the marker's style class matches its chapter's other markers;
  * no marker inside a declared note.
* **Ambiguity is refusal.** When the best and second-best interpretations
  give different text for any verse, refuse that chapter. This turns "a
  caller that happens to fit the sequence" (set G) and "a running head that
  happens to read as a verse number" (set F) into measured ambiguity,
  instead of new rules.

### 3.5 Independent comparative evidence

* **Second parse.**
  * Where the user imports two formats of one edition, or the document has
    two readable layers (an EPUB's text and its navigation; a PDF's text
    layer and its outline), accept a verse only when both agree.
  * Two different parsers reading one file (DOM-based and text-based) give
    the same check for structured formats.
* **Reference alignment, never substitution.**
  * If the user has a licensed structured text of the same edition, align
    the document against it and use the result only to *verify*. The stored
    text is always the document's own.
  * Across translations, length-based alignment (Gale–Church style) can
    flag implausible boundaries without needing the same text.

### 3.6 Language data as data

Move these out of code into tested tables per language:

* book titles and their variants;
* ordinal and chapter words;
* numeral systems: Roman, Greek, Hebrew, Han and Hangul ordinals, using the
  CLDR numbering-system definitions where they exist.

The cost of a new language (G02 Swedish, G09 archaic Korean) is then a
reviewed table, not new logic.

### 3.7 Expected effect on the observed failures

| Failure seen | Technique that addresses it |
|---|---|
| Set G: note callers in verses; callers read as verse numbers | 3.1 (link targets), 3.4 (marker inside a note; ambiguity refused) |
| G05 unreadable PDF; G03/G04/G13 real PDFs refused | 3.2, 3.1 (outline, tags) |
| Sets D–F: running heads, letter-spaced heads, drop numerals, columns | 3.2, 3.3 |
| Sets E–F: chapter labels in other numeral systems; wrong Kings/Samuel book | 3.6; 3.4 (verse-count constraints settle the book) |
| Cut verses, words read with the other column | 3.2, 3.4, 3.5 |

## 4. Where optional contextual AI would add measurable value

### 4.1 The rule: AI may only label and point, never write

An AI component's output is always a **proposal over the original
evidence**, made of:

* spans, as character ranges into the document's own text or glyph indexes
  into a page;
* labels, such as "note caller", "running head", "chapter 12 heading" or
  "title of the book RUT";
* a confidence;
* provenance: model, version, local or cloud, and a digest of the input.

The deterministic verifier then checks each proposal against the
constraints of §3.4. The stored verse text is always copied from the
document by range. Any proposal whose text is not a substring of the
original at that range is rejected. Accepted results stay provisional
(requirement 6), and a verse is marked when AI evidence took part in it.

### 4.2 Candidate uses, ranked by evidence of need

| Use | Why AI | Measurable value (on disclosed sets, then a new blind set) |
|---|---|---|
| **Book-title identification** in unknown languages or old spellings (G02 Swedish, G08 "EPÍSTOLA^", G14 Latin titles, G09) | Open-ended naming that tables never finish | Books found ↑; wrong-book verified = 0; proposals checked by chapter counts |
| **Role labelling of ambiguous lines** where the style statistics are inconclusive (heading, caption, note, furniture) | Context across lines and pages | Unverified `possibleHeading`/`boundary` ↓ with wrong text = 0 |
| **Reading order on complex pages** (local layout models over glyph boxes) | Columns, sidebars and notes interleaved beyond the classic algorithms | Reading-order refusals ↓ (G04, G13, set F's F07) |
| **Explaining a refusal** to the user and proposing a mapping for the user to confirm | Turns a silent 0 % into a guided import | Documents imported with user confirmation; no change to automatic acceptance |
| **OCR of scanned Bibles** (later; local OCR first) | No text layer exists | Its text is OCR-derived, never "original quotation"; stays a separate standing |

Not candidates:

* writing or "correcting" verse text;
* filling missing verses;
* normalising spelling;
* anything that replaces the document's characters.

These are excluded by requirement 4.

### 4.3 Local before cloud

* The application already has provider support for local servers (Ollama,
  LM Studio and compatible) as well as cloud services
  (`docs/architecture.md` §23). Recognition assistance would use the same
  abstraction.
* It would offer local models first. The labelling tasks above are small
  classification problems suited to small local models, or to classical
  learned classifiers that need no model server at all.
* Cloud processing would need a separate, per-document consent that names
  the provider. It would send only the minimal excerpt (the lines in
  question, not the whole Bible). It must respect the edition's licence:
  many modern translations forbid sending their text to third parties.

### 4.4 Opt-in and default

* The default pipeline (§3) must work completely without AI and must meet
  the accuracy standard on its own.
* AI assistance is a setting that is off by default and is enabled per
  import. Its contribution is recorded per verse.
* An import with AI assistance is never more trusted than one without:
  provisional standing is the same, and review is the same.

### 4.5 How value would be measured

Use the existing harness with the same corpora and checksums. Report AI-off
and AI-on side by side, recording accuracy and coverage separately as for
v0.24.7.

* **Acceptance gate:** verified wrong text and nonexistent verses must stay
  at 0 with AI on.
* **Value:** only coverage gain counts, measured on a blind set built after
  the AI component is frozen.
* **Additional reporting:** latency, memory and the share of proposals the
  verifier rejects.

## 5. Recommended direction

1. **Stop adding rules** to the greedy recogniser as the primary strategy.
   Keep it as the fallback and the regression baseline.
2. **Phase A (non-AI, highest value):** do 3.1 and 3.2 first.
   * Semantic note detection from links, tags and footnote references.
   * A mature PDF engine with the current reader as fallback.

   These address set G's only accepted errors and its unreadable file. Each
   is measured on all disclosed sets, then on a fresh blind set.
3. **Phase B:** 3.3 and 3.4. Style clustering and the global chapter
   interpretation with ambiguity refusal, replacing most layout rules.
4. **Phase C:** 3.6 and 3.5. Language tables as data, and two-parse or
   reference-alignment verification.
5. **Phase D (optional, opt-in):** the AI labelling interface of §4, local
   first, under the gates of §4.5.

Each phase ends as v0.24.x did:

* a frozen recogniser;
* disclosed-set regression, with accuracy and coverage recorded separately;
* an independent blind set built after the freeze;
* the owner's decision.

## 6. Interface and data notes for Codex

* Nothing here changes migration 0027. The provisional-import safeguards
  keep 0028.
* An AI-evidence flag per verse would need storage: an evidence-kind
  column, or a provenance row per proposal, recording model and version,
  local or cloud, input digest, and decision. That would be proposed to
  Codex as a later migration *after* 0028, numbered when agreed. It is not
  reserved here.
* The issue codes and unverified-reason values stay as they are. A
  global-interpretation refusal would reuse `boundary`. Any new value will be
  proposed before use, as the v0.24.6 contract states.
