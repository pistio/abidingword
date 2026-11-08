# Scripture recognition, Phase A1.1: no known wrong verified EPUB verse, removed references as exact ranges

**Status:** research prototype for the owner's review, approved for continued
development but **not for production integration**. Nothing here is merged,
released or switched on. The structure-aware reading
(`structural::stage_documents`) is still reached only from the harnesses. The
frozen recogniser's rules are unchanged except one reporting addition (§2.2),
which changes no verse.

**Every corpus used is disclosed** (Sets B–G, recon, development). These are
regression results on known documents, not evidence about unseen editions.
Recognised document-derived Bible text stays provisional. No AI was used.

## 1. Commits (branch `claude/recognition-a1.1-epub-accuracy`, from A1 `0e8c169`)

| SHA | What |
|---|---|
| `e96d241` | Every letter between one verse and the next is accounted for (`UnaccountedText`); the recogniser reports where it removes a reference (`Apparatus::within_verse`); grounded removals become exact excluded ranges |
| `47a4309` | Trailing removals; untraced verses judged from their own opening; classless inline elements measured by name; byte-order marks counted in byte ranges |
| the commits after it | Evidence (`docs/audit/a1.1/`), table generator (`scripts/a1.1/tables.py`), this report |

## 2. What changed

### 2.1 Unaccounted text withdraws verification

The tracer already placed each verse's text in the original as exact spans.
A1.1 adds a check of what lies **between** verses. Every letter from a
traced verse's last character to the next verse's first must be accounted
for:

* excluded from a verse (a note caller, a note sign, a grounded removal);
* inside an element the document's structure sets apart (`h1`–`h6`, a note
  container, `nav`/`aside`, hidden content, a floated block, a block a note
  caller leads to);
* inside an element whose class, or for classless inline elements whose
  name, is apart **by measurement in this document**: used at least three
  times and never around a verified verse's words.

If not, the verse before is withdrawn as `UnaccountedText`, stored as the
existing `boundary` reason, so no new database value is needed.

Where the next verse cannot itself be traced, the text is judged only up to
where that verse's own opening characters are found in the original.
Nothing is removed, rewritten or joined.

### 2.2 The recogniser reports what it removes

`Apparatus::within_verse` is the number of the verse's own non-space
characters before a piece the recogniser took out (an inline quotation
reference). The structure-aware reading accepts a removal only when it is
**grounded**:

* exactly one whole inline element in the verse's place holds exactly that
  text;
* the document sets that element apart (§2.1);
* the trace finds it after exactly the reported number of characters, or,
  for a reference after the verse's last word, right where the verse ends,
  with only note callers or note signs between.

It is then an excluded `Apparatus` range of the original, with character,
byte and element ranges. An ungrounded removal leaves the verse unverified.

In the frozen reading this changes no verse. Pieces with a position are
simply no longer merged, so the apparatus piece count changes in 9 of 113
editions (`docs/audit/a1.1/README.md`).

### 2.3 Byte ranges of byte-order-marked entries

12 disclosed EPUBs (all eBible.org builds) begin almost every entry with a
UTF-8 byte-order mark. quick-xml skips it without counting it, so every byte
range in those books was 3 bytes short in A1 and in `e96d241`. Character
ranges, tracing and verification were unaffected. Fixed and tested.

## 3. Results

Full tables: `docs/audit/a1.1/regression-tables.md`. Cases:
`docs/audit/a1.1/cases.md`.

| Measure, all disclosed EPUB-family editions | Frozen v0.24.7 | A1 (`205a453`) | A1.1 (`47a4309`) |
|---|---|---|---|
| Verified verses with wrong text | 4 152 | 8 | **0** |
| Verified verses the edition does not have | 5 | 0 | **0** |
| Verified verses | 488 952 | 506 162 | **505 989** |
| Of the 16 verses refused for inline references | — | refused | **16 verified, 165 removed ranges exact** |

**The 8.** All are withdrawn as `UnaccountedText`: Crampon's 7 section
titles and speaker labels, and the Portuguese subscription to Hebrews. In
each case the recogniser had left out text that the edition's own source
places in the verse (`cases.md` §1).

**The 16.** All are verified with their removed references as exact
excluded ranges; the id check agrees in 15. Every removed range in the book
(165) equals the stored file's bytes (`removed-ranges.md`).

**Coverage cost against A1: −173 verified verses (−0.03 %)**, all correct
verses withdrawn because unaccounted text follows them:

* frasbl: −117 (psalm titles set in `em`, which also holds *Sélah* in
  verses);
* F12: −48 (psalm titles in the verses' own paragraph);
* francl: −12 correct, besides its 7 wrong;
* fraLSG: −2;
* latVUC and F10: −1 each.

Each was checked by edition from the markup (`cases.md` §3).

**Regression cases kept.** All 18 A1 tests pass, including:

* JHN 7:52 and its note-sign case;
* book-id ambiguity and contradiction;
* note callers;
* G01's 0 wrong at 24 305 verified (Set G, disclosed).

A1.1 adds 12 tests (`tests/recognition_a1_1_accuracy.rs`). Each fails on
the code before its rule and passes now.

**Unchanged elsewhere.** The frozen reading at `e96d241` gives reports
identical to v0.24.7 in every field but the apparatus count, on all 113
editions. The 79 editions not read by the EPUB reader are identical under
both settings.

## 4. Performance in the optimised build

Release build (`cargo test --release`, no debug information), one process
per edition and reading, frozen then structure-aware for each edition in
turn, on an otherwise idle 4-core machine. The whole harness run is timed:
reading, recognition, comparison and report. Per edition:
`docs/audit/a1.1/performance.md`.

| 24 EPUB-family editions | Frozen | Structure-aware |
|---|---|---|
| Total time | 174 s | 271 s (+56 %) |
| A whole Bible (e.g. kjv-epub, porbr2018-epub) | 6.6 s | 13.4–14.1 s |
| Peak memory, whole Bible | 48–88 MB | 390–470 MB |
| Peak memory, largest (latVUC-epub, 15 MB of XHTML) | 88 MB | **597 MB** |

**Long-EPUB memory is the main cost.** The source model keeps, for every
visible character of the book:

* the character;
* its byte range;
* its link role;
* its removal mark;
* its non-space index;
* its innermost element, computed twice.

That is roughly 60–80 bytes per character, plus the entry sources, for the
whole book at once. Holding one entry at a time, narrower types (a `u32`
element index instead of `Option<usize>`), and computing `owners` once
would cut it several-fold. That has not been done: it changes no result,
and it is the first optimisation to make before the reading is wired into
the application.

Time in a release build roughly doubles for a whole Bible: 6–7 s to
13–21 s; in the test build, A1 measured +22 %. Importing an EPUB is a one-off
step, so the time is acceptable; the memory is not yet, for a desktop app
importing long books.

## 5. Checks

At `47a4309`:

* `cargo fmt -- --check`: clean.
* `cargo clippy --all-targets --locked -- -D warnings`: clean.
* `cargo test --release --locked`: 1 077 passed, 0 failed, 8 ignored (the
  harnesses), in 79 test binaries.
* `npm run typecheck`, `lint`, `i18n:check`, `test` (1 053 tests) and
  `build`: all passed.

The Rust tests ran in the release profile because the session's disk is
too small for a debug build of every test binary. CI runs them in its own
profile.

## 6. Unresolved risks

1. **Generalisation is unproven.** Every rule was measured on disclosed
   documents, and three of A1.1's faults were found only by running the whole
   corpus. The evidence for "no wrong verified EPUB verse" is these 34
   editions and nothing else.
2. **The measured-class rule can be fooled.** A class or element used only
   for titles, never around verse words, is taken as set apart. A
   generator that styled the words of some verses with that same class
   would be caught, but one that never does while still placing real verse
   text in it (text the recogniser drops) would not.
3. **Coverage cost is real.** Psalm titles in the verses' own style are
   refused wholesale (F12 −48, frasbl −117). Accepting them would need
   positive evidence, such as the edition's own versification of psalm
   titles, not a looser rule.
4. **The frozen path still verifies 20 wrong verses** in non-EPUB editions
   (`spablm-docx` 9, `deutkw-pdf` 5, `deutkw-lopdf` 2, `porbr2018-pdf` 1,
   `engasv-rrlab1col` 1, `spav1602p-lodocx` 2), plus 2 verses that
   `spav1602p-lodocx` does not have. They predate A1 and are outside A1.1.
   They are why **no candidate may be frozen yet**.
5. **Untraced verses across entries** (G01: 145) are still refused.
6. **Interface strings.** Provenance reasons are English audit text; any
   review interface needs the six-language work.

## 7. Recommendation for the next owner decision

Do not freeze a release candidate; known wrong verified verses remain on
the frozen path (§6.4). Choose one of:

1. **Fix the frozen path's 20 wrong verses**, under the same rule: grounded
   evidence or refusal, nothing removed by look alone. Then freeze one
   candidate covering both readings and run a new, independently built,
   EPUB-heavy blind set. *Recommended.*
2. **Freeze the structure-aware EPUB reading alone as a candidate** for a
   blind EPUB set, accepting that the non-EPUB path is not part of it.
3. **Wire `structural::stage_documents` behind an off-by-default setting**
   for Codex once 0028's provisional standing exists. Its refusals arrive
   as ordinary `boundary` / `noteSign` reasons, so no schema change is
   needed. Persisting provenance would be a later, separate migration.

None of these needs Set H yet.
