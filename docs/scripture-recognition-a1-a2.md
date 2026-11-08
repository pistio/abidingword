# Scripture recognition, Phase A1 + A2: structure-aware EPUB reading, exact provenance, PDF engines

**Status:** prototype for the owner's review. Nothing here is merged, released
or switched on in the application. The frozen recogniser `c96756d` stays the
baseline, and all of its rules are unchanged.

**Every set used here (B–G and the development corpus) is disclosed.** The
numbers below are regression evidence on known documents. They are **not**
independent evaluation evidence and say nothing about unseen editions.

Recognised document-derived Bible text stays provisional whatever its
internal verification status. No AI was used to read, generate, correct or
normalise any text.

## 1. Branches and commits

| Item | Branch / SHA |
|---|---|
| Frozen recogniser (baseline) | `c96756d` |
| v0.24.7 branch tip with Set G evidence and the assessment | `claude/scripture-recognition-v0.24.7` at `ea5f944` (unchanged) |
| A1 branch | `claude/recognition-a1-structure`, from `ea5f944` |
| A1 failing tests first | `37e8822` |
| A1 structure-aware reading | `54dfc92` |
| A1 harness report | `ba09acb` |
| Optional-AI interface proposal (design only) | `57568b6` |
| A1 checks found by the regression (ids, swallowed verse numbers) | `8d02b97` |
| A1 note back-links are apparatus | `205a453` (the measured state) |
| A2 branch | `claude/recognition-a2-pdf-engines`, from `ea5f944`; see §6 |

## 2. What A1 changes, and what it does not

**Unchanged.** `documents::read`, the EPUB adapter's existing reading
(`epub::blocks`, `epub::read_book`), `take_out_note_links`, the recogniser
(`scripture::structure`) and every verification rule. `import::stage_documents`
calls the same code as before through a new seam
(`stage_documents_read(paths, origin, &documents::read)`). The frozen reading
was re-run on every EPUB of every corpus: its reports are **identical** to the
`c96756d` reports in every field except run time, peak memory and the
temporary reference file name (24 of 24 editions).

**New, isolated, used only when asked** (`scripture::structural::stage_documents`,
`AW_PIPELINE=structural` in the harness). Nothing in the application calls it
yet.

1. **A source model of the EPUB** (`library/adapters/epub_source.rs`). Every
   spine entry is read once with its exact text, and every character keeps its
   byte range in the original file (entities keep their whole `&…;` range).
   Every element keeps its name, id, classes, `epub:type`/`role` values,
   `href`, parent and text range.
2. **Link roles from evidence, never from the label.** Every `<a href>` is
   classified by where it leads:

   | Evidence | Role |
   |---|---|
   | Target inside a declared note container (`epub:type="footnote"`, `doc-endnotes`, `class="footnotes"` …) | note caller |
   | Target block links back, and block edges show which block is the note | note caller |
   | The link itself stands inside a note container, or is a declared back-link (`doc-backlink`) | navigation (apparatus) |
   | Declared a caller (`noteref`) but leads to text or a heading | **conflict**: excluded, and nothing beside it verified |
   | Caller-shaped label (`[3]`, `a`, `†`) leading nowhere, or a reciprocal pair with no direction | **ambiguous**: nothing beside it verified |
   | Anything else (contents, glossary, cross-references, headings) | navigation: its words stay |

   A visible `[1]` is therefore never assumed to be either a verse number or a
   note. Bracketed words and words linked to a glossary stay Scripture.
3. **Callers are marked, never cut.** Marked callers are handed to the
   recogniser as note marks, as declared callers always were. Blocks that the
   callers lead to are notes. A block whose class the book's stylesheet floats
   or positions absolutely (margin chronology, side notes) is apparatus.
4. **Exact provenance** (`scripture/provenance.rs`). Every verse read from an
   EPUB is traced, in order, to the characters of its place in the original.
   Only white space, characters of known note callers and note signs (†‡§¶*)
   may stand between its words. Nothing is ever invented between spans. Each
   verse gets:
   * its file identity (document, file SHA-256, entry path, entry SHA-256);
   * its ordered source spans (character and byte ranges);
   * its excluded ranges (kind, range, the element's bytes, the text);
   * a deterministic quote of the spans;
   * a trace status (exact, spacing differs, untraced);
   * an id check (an id such as `Gen8-1`, `c8v1` or `RUT.1.4` at the verse's
     start, compared with the reference);
   * the reasons, in words.
5. **A structural gate that only withdraws verification**, stored under the
   existing reasons, with no new data or interface:

   | Cause | Stored as |
   |---|---|
   | Untraced | `boundary` |
   | An id at the verse's start names another verse | `boundary` |
   | The verse holds an element of the document's own verse-number class (measured: at least ten verses start right after an element of that class bearing their own number) | `boundary` |
   | An ambiguous or conflicting link in or next to the verse | `noteSign` |
   | A caller-shaped navigation link read as the verse's words | `noteSign` |

   It never verifies anything the recogniser did not.

No database migration was made. Provenance lives in the in-memory
`StructuralReport`. §5 proposes how Codex could store it.

## 3. Tests

`src-tauri/tests/recognition_a1_structure.rs`: 18 tests, all written before
the code that makes them pass. The first commit (`37e8822`) had 8 of 14
failing; each later check was added red first.

* numeric callers decided by their target, never by their label;
* the frozen reading reads numeric callers as verse numbers (kept, to show the
  difference);
* letter and sign callers into declared endnotes;
* bracketed words and glossary-linked words stay;
* linked verse numbers (to a contents page) stay verse numbers;
* a declared caller leading to a verse is a conflict;
* `[3]` leading nowhere is ambiguous and makes no verse;
* layered notes (notes with their own callers);
* a verse around a caller is two exact spans and one excluded range;
* entities and inline markup keep exact byte ranges;
* every verse of a book traced exactly, with ids agreeing;
* an id naming another verse withdraws verification;
* an id whose digits run a book's digit into the chapter (`R110_1`) is no
  evidence;
* a verse number read as words (`[[ <span class='v'>22</span>]]`) is never
  verified;
* a note's own back-link never withdraws the verses around it;
* tracing never bridges text it cannot name;
* caller shapes are short labels, never longer words;
* a floated block is never part of a verse.

Full checks at `205a453`: see §7.

## 4. Regression on the disclosed sets

Inputs are byte-identical to the v0.24.7 runs:
`docs/audit/v0.24.7/corpus-checksums.sha256` and the Set G seal
(`/root/aw-set-g/SEAL.txt`) both verified before every run. "Before" is the
stored `c96756d` report (for Set G, the original blind result). Only EPUBs
change; other formats are read as before. Full tables are in `docs/audit/a1/`:
* `regression-tables.md`;
* `refusals.md`;
* `provenance.md`;
* `performance.md`.

### 4.1 G01 (Project Gutenberg Almeida, the Set G failure)

| | Frozen `c96756d` | Structure-aware |
|---|---|---|
| Verified verses with wrong text | **4 144** | **0** |
| Verified verses the edition does not have | **5** | **0** |
| Verified with text cut or run on (boundary) | 141 | 0 |
| Verified coverage | 8 770 (28.2 %) | 24 305 (78.1 %) |
| Wrong readings correctly refused | 305 | 178 |
| Right readings refused | 2 085 | 180 |
| Chapters refused whole | 596 | 23 |
| Links classified | — | 21 360 note callers, 73 771 navigation, 0 ambiguous |
| Provenance | — | 24 917 exact, 145 untraced (text crossing two entries), 0 spacing |
| Ids | — | 24 742 agree, 166 contradict |

The 166 id contradictions are real catches. Job's title is not recognised, so
Job's text runs on as `EST 10:10…`, and the ids say `Job1-1`. Without the id
check, 35 of them would have been verified under the wrong reference. They are
withdrawn by evidence the frozen reading never had. The remaining refusals are the
unrecognised Job title and text that crosses entries.

### 4.2 All disclosed EPUBs

| Corpus | Verified before → after | Verified wrong text before → after | Verified not in edition before → after |
|---|---|---|---|
| Set G (14 editions; EPUBs G01, G12, G14) | 26 548 → 42 083 | 4 144 → 0 | 5 → 0 |
| Set F EPUBs (F04, F10, F12) | 35 979 → 35 979 | 0 → 0 | 0 → 0 |
| Set E EPUBs | 10 387 → 10 387 | 0 → 0 | 0 → 0 |
| Set D reconstruction EPUB | 19 519 → 21 210 | 0 → 0 | 0 → 0 |
| Sets B/C reconstruction EPUBs | 70 624 → 70 624 | 7 → 7 | 0 → 0 |
| Development corpus EPUBs (12) | 337 014 → 336 998 | 1 → 1 | 0 → 0 |
| **Total** | **500 071 → 517 281** | **4 152 → 8** | **5 → 0** |

The 8 remaining wrong verified verses are unchanged from `c96756d`, and none
is new:
* 7 in `francl` are section titles run into a verse: "PAROLES D'AGUR",
  "LE CHŒUR".
* 1 in `porbr2018` is a subscription after Hebrews 13:25.

They are headings in the text flow, not links, so A1 does not address them.

**Coverage cost.** 16 verified verses of `porbr2018` are now refused as
untraced. Their text is correct, but the original holds an inline quotation
reference (`<span class='rq'>Oseías 6:6 </span>`) that the recogniser took
out by a text pattern. The tracer will not skip named text that no element
rule excluded, and it is deliberately not loosened: skipping whole elements
could hide real Scripture being dropped. The fix belongs to the recogniser:
it should report the ranges it removes, so provenance can record them as
excluded.

### 4.3 What the regression found in A1 itself

The first run (`ba09acb`) had three faults. Each was fixed by a general rule
with a red-first test (`8d02b97`, `205a453`):

1. **Ids with a book digit run into the chapter.** eBible ids such as
   `R110_1` for 1 Chronicles 10:1 were read as chapter 110. About 6 100
   correct verses per development edition were withdrawn; none was wrong.
   Such ids are now no evidence either way.
2. **A new false verification (Set D reconstruction, JHN 7:52).** With the
   note caller after `[[ 53]]` no longer read as a verse number, the chapter
   was accepted. The recogniser then folded the omitted verse's number into
   7:52, which was verified. The measured verse-number-class check now
   withdraws it.
3. **Notes' back-links between chapters** were classified ambiguous, which
   withdrew 162 correct verses at chapter edges in `frasbl`.

Without the full regression, fault 2 would have been a false verification.
**The structural gate's value depends on running every disclosed corpus, not
only the set that motivated the change.**

### 4.4 Performance

Unoptimised test build, same machine. All EPUBs: 2 081 s frozen, 2 545 s
structure-aware (+22 %). The dev editions take about 40 % longer each;
editions with few links are unchanged. Peak process memory reached 595 MB
(against 396 MB) with editions read one after another in one process. The
source model keeps 12 bytes per character of every entry, about 55 MB for a
whole Bible, and is dropped after the import. An optimised build is expected
to be several times faster. It was not measured here.

## 5. Recommendation for Codex (integration interface)

A1 needs **no migration**. Migration 0027 belongs to the Sources library.
0028 is reserved for Codex's provisional Bible-import standing and safety
gates. The interface below is offered to Codex for that work. It is not
implemented in shared files.

1. **Call site.** Replace `import::stage_documents` with
   `structural::stage_documents` behind a setting that is off by default. It
   returns the same `StagedImport` plus a `StructuralReport`. With the setting
   off, nothing changes. A1's gate only adds entries to
   `DocumentStage.unverified` with the existing reasons (`boundary`,
   `noteSign`), so Codex's 0028 gates see A1 refusals as ordinary unverified
   verses.
2. **Provenance storage, if wanted (proposal for after 0028).** One row per
   verse place:
   * `document_sha256`, `entry_path`, `entry_sha256`;
   * `spans` as a JSON array of `[charStart, charEnd, byteStart, byteEnd]`;
   * `excluded` as a JSON array of `{kind, range, text}`;
   * `trace_status`, `id_check`, `reasons`.

   This is enough to show a person exactly where a verse came from, and to
   check later that the original file is unchanged. A1 assigns no number.
3. **Review surface.** The per-verse `reasons` are already in words, for "why
   was this verse accepted or refused". Their translation into the six
   interface languages belongs with the UI work. No user-visible string was
   added in A1.
4. **PDF.** A2's shared extraction interface (§6) gives PDFs the same span
   model, with glyph ranges in place of character ranges. That lets the same
   tracer and gate serve the Sources library and Bible import.

## 6. Phase A2: PDF engines

On branch `claude/recognition-a2-pdf-engines`, document
`docs/audit/a2/pdf-engines.md`. Its summary and recommendation are copied
into §8 of the handover once committed.

## 7. Checks at `205a453`

See the handover, §8.

## 8. Handover

See `docs/handover-recognition-a1-a2.md`.
