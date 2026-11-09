# Scripture recognition, Phase A1.3: no known wrong verified verse outside EPUB

**Status:** candidate for the owner's review, on branch
`claude/recognition-a1.3-non-epub-safety`, on top of A1.2 (`e9c406a` and its docs commit `eb78609`).
There is no migration, no change to `Cargo.lock`, no interface change and
no production integration. The frozen `c96756d` recogniser and its
baseline reports are untouched and remain the comparison evidence.

All 113 editions are disclosed (the development corpus and Sets B–G).
These numbers are regression evidence on known documents, not
independent evaluation. Set H was not created or run.

## 1. Commits

| SHA | What |
|---|---|
| `b97ee44` | Five safety rules, their regression tests (`tests/recognition_a1_3_safety.rs`) |
| `ccaad81` | Corpus derivation script: keep every paragraph marker's text |
| the commit after it | Evidence (`docs/audit/a1.3/`), this report; the red-letter control test; a duplicated test attribute removed |

## 2. Result on the 113 editions

| | v0.24.7 (`c96756d`) | A1.1 | **A1.3** |
|---|---:|---:|---:|
| Verified, correct text | 1 956 673 | 1 977 862 | **1 977 517** |
| **Verified, wrong text** | 4 172 | 20 | **9** |
| **Verified, not in the edition** | 7 | 2 | **0** |
| Refused (correct text, not verified) | 44 832 | 45 052 | 45 397 |
| Caught (wrong text, not verified) | 17 723 | 17 695 | 17 706 |

* All 11 wrong verified verses the app could know about are now
  unverified, and so are both references the edition does not have.
* The 9 left are `spablm-docx`'s. The **test corpus's** derivation left the
  text out of that document; the document the app reads has no trace of
  it (§4).
* The cost is 345 correct verses no longer verified: 0.017 % of correct
  verified verses. 230 of them are in one edition (`engbsb-rwpdf2col`).
* Every EPUB edition is identical to A1.1. 94 of the 113 editions are
  identical in every measure.

Per edition: `docs/audit/a1.3/regression.md`, `data/editions.json`.

## 3. The cases

`docs/audit/a1.3/cases.md` traces each verse from the edition's USFM,
through the derived document, to the units and the verse the reader
closed.

| Edition | Verses | Origin | Rule |
|---|---|---|---|
| `deutkw-pdf`, `deutkw-lopdf` | 7 | Segmentation: a mid-verse heading in another face read as the verse's text | A whole line in another face after `:` `,` `;`; such a line beginning a block |
| `porbr2018-pdf` | GAL 6:18 | Segmentation: the verse's last line read into the next book's heading | The heading took in the line right after the verse |
| `engasv-rrlab1col` | HAB 3:19 | Segmentation: a smaller-type note joined to the verse | A line in smaller type after the verse ended |
| `spav1602p-lodocx` | 2 wrong, 2 not in the edition | Reference assignment: a number the document does not style as a verse number | The DOCX's own verse-number style; a plain number in a document that raises its numbers |
| `spablm-docx` | 9 | Omitted source text (corpus derivation) | None in the app; the derivation script is corrected |

Every case was a verse ending sooner or later than it should, or a number
taken as a verse. None came from verification itself: each wrong text
was compared correctly with what the reader assembled.

Each rule makes a verse **unverified**. It never assigns text or a
reference differently, so source spans, excluded ranges, byte offsets
and the existing refusal reasons are unchanged (`boundary`,
`possibleHeading`).

## 4. `spablm-docx`: why it is not refused

* `scripts/build-derived-editions.py` did not list `\mi` and several other
  paragraph markers, so their text never reached the derived DOCX.
* The document ends each of these verses at a colon, followed by the next
  verse number. That is exactly how thousands of correct verses end
  ("… diciendo:").
* No document evidence separates them, and refusing them all would trade
  thousands of correct verses for nine.
* The script is corrected. A document rebuilt with it (scratch, SHA-256
  `0cb50d10…`) reads with no wrong verse, on A1.1 and A1.3 alike.
* **The checksummed corpus file (`2ad1094b…`) is not replaced.** That is
  the owner's decision, because it changes the frozen comparison evidence.

## 5. The cost, rule by rule

From a build that logged each rule as it fired (scratch, not committed);
indicative, not exact:

| Rule | Where it refuses correct verses | Why they are refused |
|---|---|---|
| Plain number in a document that raises its numbers | `engbsb` (most of its 230), `rv1909`, `frajnd`, `deutkw` | In those PDFs a few verse numbers are read as not raised (why was not established). A plain number is then no proof of a verse: that is `spav`'s failure. |
| Whole line in another face after `:` `,` `;` | `engwebp-book`, `fob`, `fraLSG-pdf`, `frajnd`, `grcbyz`, `deu1912`, `spabes`, `poronbv`, `engrv` | Italic or red words of the verse itself on a line of their own, which the document cannot tell from a heading in italic. |
| Line in smaller type after the verse ended | `engasv` | — |
| Verse's line read into the next book's heading | `porbr2018`, `latVUC-pdf` | — |

Four narrowing steps were measured on all 113 editions before this
candidate; each is in a comment beside its rule:

* the heading rules apply to paginated documents only (EPUB and text
  editions had lost 79 correct verses);
* the line taken into a heading must come right after the verse (`engasv`'s
  two-line titles, 35);
* a line continuing a quotation that began on the line before is exempt
  (red-letter editions);
* a fifth rule (a passage set aside before the next book) was removed: it
  was needed for no case and refused book-final verses in Pauline epistles
  with subscriptions.

## 6. Regression tests

`tests/recognition_a1_3_safety.rs`, 6 tests. Each was run with its rule
switched off in a scratch build:

| Rule switched off | Test that fails |
|---|---|
| Whole line in another face after `:` | `a_whole_line_in_another_face_after_a_colon_withdraws_the_verse` |
| Such a line beginning a block | `a_line_perhaps_in_another_face_beginning_a_block_withdraws_the_verse` |
| Smaller type | `a_line_in_smaller_type_after_the_verse_ended_withdraws_it` |
| Line read into the next heading | `a_line_after_a_book_s_last_verse_read_into_the_next_book_s_heading_withdraws_the_verse` |
| Plain number in a raised document | `a_plain_number_inside_a_line_…` and `a_word_document_s_own_verse_number_style_…` |
| DOCX verse-number style | `a_word_document_s_own_verse_number_style_…` |
| The exemption for a quotation begun on the line before | `a_whole_line_in_another_face_after_a_colon_…` (its red-letter control) |

With every rule on, all 6 pass. The exemption's row shows a narrowing
is tested too: without it, the red-letter control is refused. Each test also checks a control that
stays verified (a poem's centred line, a spoken quotation, the same
book without the line in question, a document without raised numbers).

No existing test's expectation was changed. The only edits to old tests
add the two new `Unit` fields (`partly_styled: false, small: false`) to
their literals.

## 7. Checks

On the candidate (the rules and tests as committed):

* `cargo fmt -- --check`: clean.
* `cargo clippy --all-targets --locked -- -D warnings`: clean.
* `cargo test --release --locked`: 1 086 passed, 0 failed, 8 ignored (the
  harnesses), in 80 binaries.
* `npm run typecheck`, `lint`, `i18n:check`, `test` (1 053 tests) and
  `build`: all passed. No interface file changed.
* The 113-edition harness: release build, SHA-256 `cc555ac6e745d612…`,
  inputs checked against their checksums before the run.

## 8. Risks

* The rules are measured on these 113 disclosed editions. A face change,
  a smaller size or an unraised number can mean something else in a
  document not seen. Each rule only withholds verification, so an unseen
  case costs coverage, not correctness.
* `partly_styled` depends on pdf-extract's glyph widths. A PDFium backend
  (A2.1) would need these rules re-measured.
* `spablm-docx` stays at 9 wrong verses in the frozen corpus until the
  owner decides on the corrected document.
