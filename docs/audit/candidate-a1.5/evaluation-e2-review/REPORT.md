# Independent review of evaluation candidate A1.5-E2

**Verdict: E2 NOT READY.**

Two technical blockers remain (§2): the reference *text* that every verse is
compared with is not the edition's text in identifiable cases, and the
evidence of that is invisible to E2's own cross-checks (**R1**); and an
official **pass** can be obtained from a run that verifies nothing (**R2**).
Separately, the owner decisions that E2 itself leaves open (composition,
strictness, the amended meaning of D1) are still open (**R3**).

The E2 corrections themselves are real and they work: every one of the 11
concerns of the earlier audit is implemented in code, not only claimed, and
each was confirmed here with the reviewer's own material (§3). E2 caught
genuine recogniser errors on real text that A1.5 counted as correct (§7).
The blockers are in what E2 inherited (the app's USFM reader) and in what
the verdict means, not in the new rules.

Status: an independent review record for the owner. It authorises nothing:
no protocol change, no seal, no run, no merge, release or updater. Collection
H was not accessed and nothing here concerns it.

## 0. In plain words

* The new comparison rule is sound. 159 adversarial examples written for
  this review, in English, German, Spanish, Korean, polytonic Greek,
  Hebrew, Arabic and Unicode edge cases: no materially different text was
  ever counted faithful, by either the Rust or the Python implementation.
* The *reference* side is the weak point. E2 still reads the publisher's
  verse text with the app's own USFM reader. That reader puts a space where
  a footnote was, so "heel.”" becomes "heel. ”". On the 13 real eBible
  references bundled in the repository this affects 528 verses; on a whole
  Berean Standard Bible document made from the edition's own words, E2
  reports **154 verified verses with wrong text, every one of them caused
  by the reference reader**. Other USFM constructs make verses vanish from
  both "independent" inventories at once. E2's Python check copies the
  same regular expressions and never reads text, so it sees none of this.
* A run in which nothing at all is verified passes. A set whose scored
  documents verify 0 verses (one refused whole, one prose, one deutero-
  canonical, one whose "reference" contains no USFM at all) received the
  official verdict **pass**, with coverage 0.0 %.
* The binary pin was reproduced exactly (`800fb43c…`) by an independent
  rebuild at the same path with the same toolchain; at another path the
  hash differs, as E2 states.
* Outside the evaluation, for the product: the app's own importer stores
  the same text, so a person who imports these bundled Bibles reads
  "heel. ”" in 528 verses (§9, last item).

## 1. What was reviewed, and how

| | |
|---|---|
| Reviewed branch and head | `claude/scripture-recognition-eval-fidelity-mokiah` at `40d5d5970953b0546ac77e827b6d7d9d8f7c08d8` (the expected head; verified) |
| E2 evaluation source | `97502d2c28873c790b2ed0b5cd225d9f178128b5` (tree `f007002652808898a58fab3c7e0c683c4820e3fb`); `40d5d59` adds only `EVALUATION-FREEZE.md` |
| Frozen recogniser | `08c6faac8c92c6c791ec1e7bbbe997362fa1059f`, tree `be296f2570dcbcc251789b2b1e1bbe2c9e7c2798`; `src-tauri/src` tree `8cb224c1e059edc3a7750f4c1ea1649603c0de95` in `08c6faa`, `97502d2` and `40d5d59` |
| Earlier audit | `docs/audit/candidate-a1.5/evaluation-audit/REPORT.md` (F1, B1–B3, C1–C5) |
| Review branch | `claude/a1-5-e2-eval-review-2ldlj5`, started at `40d5d59`; this commit adds only this folder |

**Independence.** `pistio/collection-h`, its documents, references,
manifests and results were not accessed, attached, cloned or inspected. No
withheld set was built, sealed or run. Every synthetic text in this review
was invented for it; none of the author's golden vectors or synthetic sets
was reused as test material (the author's tools were run only as the
system under test). The rule modules were read, then exercised through a
separate probe crate (`evidence/probe/`) that includes them read-only, and
through the real pinned harness binary, rebuilt here.

**Method.** (1) identity and history checks; (2) the reviewer's own fidelity
vectors through both implementations; (3) both reference inventories, and a
reference-text extractor written for this review from the USFM
specification, on every real USFM reference available; (4) attacks on the
seal, run and score in a scratch Git repository, with the pinned binary,
committed and modified protocols, and fake harnesses; (5) documents made
from real references (Markdown, DOCX and PDF) through the E2 harness and a
rebuilt A1.5 harness; (6) the project's complete check suite.

## 2. Verdict: E2 NOT READY — the exact blockers

### R1. The reference text is single-sourced through the app's USFM reader, which is not faithful to the edition, and E2 cannot see it

E2's fidelity rule compares the document's verse with "the edition's own
text". That text is `usfm::parse` output (`eval_support/reference.rs`
calls it; the importer uses it too). Its notes regex replaces every note
with a space (`usfm.rs`, `notes.replace_all(source, " ")`), it matches a
note's opening marker to *any* closing note marker non-greedily, it does
not read USFM 3 milestones, and an unclosed `\va`/`\vp`/`\ca`/`\fig`/`\rq`/
`\cat` hides all following text of the book.

E2's second inventory (`scripts/evaluation/reference_inventory.py`) is
described as "written apart" and "neutral", but it deliberately reads labels
"exactly as the app's USFM reader takes them": its `NOTES` and `MARKER`
regular expressions are copies of the reader's. It reads labels only, never
text. So the scorer's inventory comparison cannot detect a segmentation
error of the reader (both inventories make it), and nothing anywhere checks
the reference text.

Evidence:

* **Real references.** The reviewer's independent extractor
  (`evidence/inventory/indep_usfm.py`: a character tokenizer with an
  explicit stack; no shared code or regex) against `usfm::parse` on all 13
  bundled eBible references (311 761 verse labels): labels identical in
  every file; text identical in 311 233 verses; **528 verses differ**, every
  one by a space the reader inserted where a note was removed: engbsb 156,
  grcsbl 356, engwebp 14, fraLSG 2. Under E2 each is a `mismatch`
  (`punctuation_spacing` 527, `word_boundary` 1, e.g. ACT 5:21
  "Sanhedrin —the" against the edition's "Sanhedrin—the").
* **End to end, pinned binary, real references.** Documents made from each
  edition's own words: whole Berean Standard Bible **154** verified
  mismatches, all 154 from the reader; SBL Greek New Testament **335**, of
  which 334 from the reader (the remaining one, ACT 28:28, is a genuine
  recogniser error that A1.5 also counted); Daniel (WEBP) 1, Ephesians
  (LSG) 1, Matthew (SBLGNT) 38, Genesis (BSB) 8: all from the reader.
  Under criterion 1 a single such verse fails a run. The scorer re-classed
  all 567 non-exact records without one disagreement and raised nothing
  about them, because the false text is on both sides of its check (its
  only note, on the whole-edition runs, was the archives' `keys.asc` and
  `.css` members, §9).
* **Synthetic, end to end, official path.** In a sealed and run set
  (`evidence/protocol/results-scenarios.json`, `p2-shared`), with faithful
  documents: a note before a closing quotation mark gave 5 false mismatches;
  an unclosed `\x` before a later `\f…\f*` made 2JN 1:4–6 **vanish from both
  inventories** (`scorable_ranges` "2JN 1:1-3", "2JN 1:7-13"), after which
  the report says the edition does not have them; an unclosed `\va` turned
  13 printed verses into "numbered but empty in the edition"; a USFM 3
  milestone put `-s |who="Invented"\*…-e\*` into the reference text.
  Integrity problems flagged by the scorer: none.

Consequences: false *wrong text* and false *nonexistent reference*
failures on faithful documents; printed verses that silently leave the
denominator (when the document does not print them either); and, the
reverse case, a recogniser fault that happens to reproduce the reader's
artefact would be counted exact. The claim that "every reference verse is
either scored, explicitly unsupported or otherwise accounted for" does not
hold for verses the shared segmentation loses. E2 §7.3 discloses the shared
reader; the review shows the risk is realised on real data, not
theoretical.

**What closes R1** (after owner approval, as a new rule version; the frozen
recogniser need not change): an evaluation-owned reference-text reader
that follows the USFM specification for notes (removed without inserting
white space), milestones, character spans and `~`/`//`, and that refuses a
reference with an unclosed note or character span; a second, independently
written text reader in the check whose per-verse text (or a digest of it)
the scorer compares with the harness's; and a re-run of §5 and §7 of this
review on the corrected tools.

### R2. An official pass can be obtained from a run that verifies nothing

The verdict is decided by error counts only (criteria 1–4). Coverage, by
design, has no pass mark, a document refused whole is a clean result, and
the check accepts a scored document whose reference has no scorable verse.
With the pinned binary, a simulated owner-approved protocol (composition
set) and the official scorer, the set `q-vacuous`
(`evidence/protocol/scores/q-vacuous.official.json`) gave:

> verdict **pass**; scored: 3 completed (1 refused whole); printed 47
> scorable and 10 unsupported; verified **0**; coverage **0.0 %**.

It holds a blank document, a Tobit-only document against a Tobit-only
reference (all `book_outside_canon`), a prose document, and a document
whose reference zip holds only `copr.htm`: **a reference with no USFM at
all, and an empty zip, are both accepted at the seal**. The accuracy claim
beside the verdict does state 0 verified and 0.0 %, but "pass" alone
reads as success. A run with many unsupported verses or clean refusals is
therefore open to being described as generally successful.

Part of this is a stated design choice: "a document refused whole passes
the gate, and coverage has no pass mark" (`set-h-decision-package.md` §5;
earlier audit §7.4). Accepting a scored document whose reference has no
scorable verse is not a design choice but a gap in the check. Together they
let the official verdict carry no evidence at all; that is why it is
listed as a blocker for finalisation rather than as an observation.

**What closes R2** (protocol change, owner's decision): (a) refuse at the
seal, or force to *exploratory*, any scored document whose reference has no
printed scorable verse (empty zip, prose only, every verse unsupported);
(b) a minimum of evidence for the verdict *pass*, fixed before the seal
(for example: verified faithful ≥ a stated share of printed scorable verses
over all scored documents, and no more than a stated number of scored
documents refused whole), below which the verdict is *inconclusive*, never
*pass*; (c) the verdict line itself always carries verified, printed
scorable, printed unsupported and refused-whole counts.

### R3. Owner decisions E2 leaves open (unchanged; required before finalisation)

1. **Required composition.** `protocol-e2.json` has
   `required_composition: null`; confirmed here that this makes every run
   *not a protocol run* (§6, P1). The audit quotes D5 as 23 scored, 3
   control, 2 mixed, 0 exploratory, 16 authentic, 12 generated (28
   documents); the committed `SET-H-APPROVAL.md` and
   `set-h-decision-package.md` §3 allocate **24** (19 scored, 3 control, 2
   mixed; several slots "either" authentic or generated; slots may become
   exploratory before the seal). The check compares the declared
   composition with the required one by **exact equality**, including
   authentic/generated and exploratory, so an exact composition cannot be
   fixed in the protocol until the builder has chosen. The owner must
   choose the numbers and the sequence (§10.3).
2. **Strictness** (E2 §7.6, §9 here): accept or relax each class of wrong
   text that is not a change of letters.
3. **D1/D6.** E2 does not score verse 0, chapters above the 66-book counts
   or books outside the 66; it counts and lists them as unsupported. That
   resolves the *reporting* defect of B1/B2, but D1 as written ("including
   verse zero where present") is still not met; the owner must amend D1
   explicitly or require support.

## 3. The 11 concerns of the earlier audit: implemented, or only claimed?

| | Concern | Implemented? | Confirmed here by | Residual |
|---|---|---|---|---|
| F1 | Materially different text counted correct | **Yes.** `fidelity.rs`/`fidelity.py`: exact, NFC, white-space runs; everything else wrong text, never excused by the scorer | 159 own vectors, 0 wrong-text vectors classed faithful (§4); real PDF errors caught that A1.5 excused (§7) | Unicode-version edge (§4); reference-side artefacts (R1) |
| B1 | Verse zero never scored or reported | **Reported, not scored.** `verse_zero` class, counted, listed, in CHECK.json before the run | `p2-unsup`: HAB 1:0 listed; Rust and Python agree | D1 amendment (R3.3) |
| B2 | Other reference content dropped silently | **Reported, not scored**: 11 unsupported classes | `p2-unsup`: all 11 classes populated and agreed; real references: 0 unsupported, both inventories agree on all 13 | Segmentation losses of the shared reader are not reported (R1) |
| B3 | Extra chapter verified, counted nowhere | **Yes.** Full-set lookup of every verified reference | `p2-extra`: JOL 3 `chapter_not_in_book` ×21, OBA `book_not_in_reference` ×21, `outside_declared_books` ×21, PHM 1:25 `verse_not_in_chapter`; fail | The scorer's own lookup cannot find a reference the harness omits from its list (§6, fake harness); the binary pin is the protection |
| C1 | Scorer overrides change validity silently | **Yes.** `--protocol`, `--frozen-binary`, `--inventory` refused (exit 2) without `--diagnostic`; diagnostic is always *not a protocol run* and labelled | P1 (all three refused; diagnostic labelled) | Unknown options are silently ignored (`--Protocol=` accepted, exit 0); a committed, modified scorer gives an official verdict (§6) |
| C2 | Composition not enforced | **Yes**, at seal, run and score; `null` blocks every protocol run | P1, seal refusals | Owner numbers (R3.1) |
| C3 | Books chosen from a file name | **Yes** for sealed sets | `p2-name`: `review_gen.md` holding Ruth, books `all`: 22 faithful, pass | — |
| C4 | Verse map renumbers the reference | **Yes**: any non-USFM, non-prose member refused | `r-map` (`OBA.VERSEMAP.XML`) refused | A `<verseMap>` inside a `.usfm` member is sealed (harmless: never read as a map); 12 of 13 real eBible zips are refused as published (`keys.asc`, `*.css`), so builders must repack (§9) |
| C5 | Unicode and integrity | **Yes**: NFC comparison; non-UTF-8 and two `\id` refused | `r-latin1`, `r-twoid` refused; NFD/NFC vectors | — |

**Historical test changes.** `tests/evaluation/test_withheld_set_tools.py`
keeps all 46 tests and 126 assertions (`767e226` against `97502d2`); each of
the five removed assertion lines has a renamed equivalent; no assertion
about anchors, path containment, runner refusals, execution classes or
sealing was weakened. One shift to note: the 46 tool tests now assert the
*diagnostic* verdict under a test protocol (A1.5 used the override options,
also non-official), and no test reaches an official pass. That is inherent
(the fake harness is not the pinned binary); this review's §6 P2 supplies
the missing end-to-end official pass with the real binary.

## 4. Independent adversarial fidelity evaluation

159 vectors (`evidence/fidelity/vectors.json`), each with the reviewer's own
expectation ("faithful" only for identical, canonically equivalent or
white-space-type-only text). Run through `fidelity.py` and, via the probe,
`fidelity.rs` (`evidence/fidelity/results.json`).

| Group | Vectors | Examples |
|---|---:|---|
| English | 32 | word joins/splits, LORD/Lord, apostrophes (straight/curly/dropped), moved comma, `.`→`?`, dashes, hyphen, NBSP/tab/U+2028/U+0085, leaked verse numbers and citations, footnote callers, Cyrillic/Greek homoglyphs, dotless ı |
| German, Spanish | 18 | NFD text, umlaut lost / as `ue`, ß/ss/ẞ, noun case, „“ quotes, `él`/`el` with composed and decomposed reference, `ñ`, `¿`, acute/grave, spacing tilde |
| Korean | 13 | syllables vs conjoining jamo, compatibility jamo, spacing added/removed, batchim, Hanja, half-width, old-Hangul arae-a |
| Greek | 19 | breathing changes (composed and NFD reference), iota subscript/adscript, oxia/tonos, ano teleia, `;`, final sigma, grave/acute, diaeresis, mark order, koronis, U+0340/0341/0344 |
| Hebrew | 18 | pointed/unpointed, dagesh order, shin/sin dot, cantillation lost/changed, maqaf, sof pasuq, final kaf, holam haser, meteg, RLM, presentation forms (canonical and compatibility), paseq |
| Arabic | 14 | harakat, shadda order, presentation forms, tatweel, wasla, Persian kaf/yeh, ZWNJ, Arabic comma and digits, superscript alef, madda |
| Unicode | 38 | ligatures, full width, superscripts, Roman numerals, Kelvin/Ohm/Ångström signs, soft hyphen, ZWSP/ZWJ/WJ/BOM/CGJ/VS/MVS, U+FFFD, private use, white-space set, mark reordering (canonical and not), Unicode 16 compositions, empty texts, USFM `~` and `//` left literal |
| Verse boundaries | 7 | next verse appended, another verse's words, number inside, citation, doubled verse, heading leaked, punctuation-only |

Results: **157 of 159 as expected and identical in Rust and Python. No
vector the reviewer judged wrong text was classed faithful by either
implementation.** The two exceptions are Unicode 16 canonical compositions
(Kirat Rai U+16D67 U+16D67 → U+16D68; Tulu-Tigalari U+11382 U+113C9 →
U+11383): Rust says *canonical*, Python *mismatch*. The scorer would record
a disagreement and declare *not a protocol run*: fail-safe, as E2 §7.8
predicts. Correction: the Rust crate (`unicode-normalization` 0.1.25) is
Unicode **17.0**, not 16; Python 3.13.18 is 15.1.

Observations: CJK compatibility ideographs are canonically equivalent to
unified ones (U+F914 樂 ≡ U+6A02), so Korean Hanja reading variants are
faithful under NFC (Unicode's definition; low practical risk). Every
invisible character, compatibility form, bidi mark and punctuation-spacing
change is wrong text; that is the intended strictness (§9).

Rust harness against Python scorer on real data: 567 non-exact records from
the real-reference runs re-classed by the scorer, 0 disagreements.

## 5. Reference completeness

* **Both inventories on real references.** All 13 bundled eBible references
  (deu1912, deu1951, deuelo, engbsb, engwebp, fraLSG, grcbyz, grcmt, grcsbl,
  grctr, kor1961, porbr2018, spaRV1909; 311 761 labels): label digests,
  per-class counts and scorable ranges agree in every case (the scorer's
  own `compare()`); 0 unsupported verses; 28 verses numbered but empty, each
  confirmed empty or note-only in the raw USFM (e.g. LUK 17:36, ACT 8:37 in
  engwebp; 18 bare `\v N` in spaRV1909). All USFM members are in NFC.
* **Unusual labels and books** (reviewer's own reference, `p2-unsup`): verse
  0, `12b`, `201`, a second verse 7, chapter 4 of Habakkuk, chapter `0` and
  `2a`, a verse before any chapter, SIR, an unknown code ZZZ, a file
  without `\id`, Jonah in two files. Every class was counted and listed by
  both inventories identically; the verified duplicate HAB 2:7 failed
  criterion 2 as *unvalidatable*. Coverage of all printed verses (88.4 %)
  is reported beside coverage of scorable ones (100 %).
* **Extra verses, chapters and books**: see §3 B3; each fails criterion 2
  with the right reason.
* **The shared reader** (R1): losses and text artefacts that the label
  cross-check cannot see, because the "independent" inventory copies the
  reader's regular expressions.
* **Accounting.** On every real and synthetic report the scorer's
  accounting checks held. They establish internal consistency of the
  harness's report, not completeness against the USFM file.

## 6. Protocol and scoring integrity

All with the pinned binary `800fb43c…` unless stated. "Simulated revision":
a scratch commit of `protocol-e2.json` with a required composition, to
stand in for the owner's future revision (sha `0d6baa86…`); never in this
repository.

| Attempt | Result | Correct? |
|---|---|---|
| P1 Clean set, protocol as committed | *not a protocol run*; the only failed condition is `composition_required_by_protocol_and_met` | yes: `null` blocks official scoring |
| `--protocol`, `--frozen-binary`, `--inventory` without `--diagnostic` | refused, exit 2 | yes |
| `--diagnostic` (with or without a rehearsal protocol) | always *not a protocol run*, labelled DIAGNOSTIC | yes |
| `--Protocol=<file>` (misspelt option) | silently ignored, exit 0 | harmless here; should be refused |
| P2 Clean set, simulated revision | official **pass** | yes: the official path works end to end with an independently built binary |
| Unreported text mismatch, by the real harness | not possible: a real wrong word gave *fail* (`q-real-error`) | yes |
| Unreported text mismatch, by a fake harness (record dropped, verse counted exact) | official: *not a protocol run* (binary); diagnostic: **pass** | the scorer's own re-check cannot see it; only the binary pin does |
| Edition text overwritten in a kept record (fake) | *not a protocol run*: fidelity disagreement | yes |
| Omitted unsupported label (fake) | *not a protocol run*: inventories differ | yes |
| Whole extra chapter hidden from `verified_references` (fake) | official: *not a protocol run* (binary); diagnostic: **pass** | the scorer's lookup is only of listed references; the binary pin protects |
| False references in an extra chapter, book, verse (real harness) | *fail*, criterion 2, reasons listed | yes |
| Composition declared unlike the manifest / unlike the protocol | refused at the seal | yes |
| Another build of the same source (`d3cdcd77…`), its hash given to the runner | runner accepts (it compares with its argument); scorer: *not a protocol run* (`frozen_binary`) | yes |
| Hand-made run record without `ended`, a document missing | *not a protocol run* (`run_complete`, `every_document_has_a_record`) | yes |
| Run record changed in a later commit | refused | yes |
| Changed seal after the seal commit | refused (covered by the author's tool tests, which pass here; not repeated with the reviewer's material) | yes |
| Verse map (`OBA.VERSEMAP.XML`) | refused at the seal | yes |
| `<verseMap>` content in a `.usfm` member | sealed; never read as a map | harmless |
| Non-UTF-8, two `\id`, `.txt` reference, lower-case or empty `books`, scored without reference | refused at the seal | yes |
| Empty reference zip; prose-only reference | **sealed**; a document that verifies text against it fails (`book_not_in_reference`), one that verifies nothing passes | no: R2 |
| File named `review_gen.md` holding Ruth, books `all` | measured against all; pass | yes |
| Vacuous set (0 verified) | official **pass**, coverage 0.0 % | no: R2 |
| Failing set scored by a modified scorer committed to the checkout (criteria forced true) | official **pass**; `anchors.scorer_sha256` `bd3f1270…` ≠ E2's `f38d797a…`; `tools_committed_and_unchanged` true | "official" means "the tools committed in this checkout", not "the approved tools": the owner must check the recorded hashes (§10.5) |

## 7. Real published documents

**Accessible.** The 113 disclosed editions' documents (eBible PDF, EPUB,
derived DOCX/PDF/Markdown) are **not** in this environment, and
`ebible.org` is refused by the network policy. The repository bundles 13
eBible USFM references; each matches the SHA-256 recorded in its
`SOURCE.md`, and for 10 of them every USFM member is byte-identical to the
committed disclosed inventory (`disclosed-documents.sha256`); the zips of
grcbyz, grctr and kor1961 are themselves in it. engbsb and engwebp are
later eBible builds (only 1 and 2 members in the inventory): they are real
published references, **not** the disclosed copies, and are not presented
as such. No historical figure is recomputed or rewritten.

**Documents.** Authentic published documents could not be used. The
reviewer generated documents from these real references (the editions' own
words, extracted independently; notes, headings and titles left out):
Markdown, and DOCX/PDF through LibreOffice 24.2.7. They are *generated*
documents. Results with the pinned E2 harness and an A1.5 harness rebuilt
from `08c6faa` (`evidence/real-documents/`):

| Document | Printed | Verified | A1.5 wrong | E2 mismatch | Reader artefact | Document reading |
|---|---:|---:|---:|---:|---:|---:|
| engbsb whole Bible, md | 31 086 | 30 977 | 0 | 154 | 154 | 0 |
| grcsbl NT, md | 7 939 | 7 671 | 1 | 335 | 334 | 1 |
| kor1961 whole Bible, md | 31 081 | 31 077 | 0 | 0 | 0 | 0 |
| engbsb Genesis, md / docx | 1 533 | 1 526 / 1 527 | 0 | 8 / 8 | 8 / 8 | 0 |
| engbsb Genesis, pdf | 1 533 | 1 517 | 0 | 15 | 8 | **7** |
| engwebp Daniel, md / pdf | 357 | 356 / 177 | 0 | 1 / 1 | 1 / 1 | 0 |
| grcsbl Matthew, md / pdf | 1 068 | 1 028 / 0 | 0 | 38 / 0 | 38 | 0 |
| fraLSG Ephesians, md / docx | 155 | 155 | 0 | 1 / 1 | 1 / 1 | 0 |
| kor1961 Ruth, md / pdf | 85 | 85 / 44 | 0 | 0 | 0 | 0 |
| deu1912 Ruth md; spaRV1909 Ruth md/docx; grcbyz 3 John md | 85 / 85 / 15 | all | 0 | 0 | 0 | 0 |

* Recognition is identical under both harnesses (same verified counts):
  only the measurement differs.
* The 7 PDF errors are genuine and E2 is right to fail them; A1.5 counted
  all as correct: a real hyphen removed at a line end
  ("Shaveh-kiriathaim" → "Shavehkiriathaim", "Abel-mizraim" →
  "Abelmizraim") and a space inserted after or before an em dash
  ("bird— everything"). This is F1 working on real text.
* Every other E2 mismatch on these documents is R1's reference artefact.
* The low PDF coverage (Daniel 177 of 357, Ruth 44 of 85, Matthew 0) is the
  recogniser's behaviour on LibreOffice PDFs, outside this review.

**Unmeasured.** The A1.5-against-E2 effect on the 113 disclosed editions
(1 980 257 verified correct under A1.5) remains unmeasured; no statistic
is estimated. The results above suggest the E2 figures would include the
reader's artefact wherever an edition places a note against punctuation,
and genuine PDF errors that A1.5 excused. Authentic EPUB and publisher PDF
layouts, Hebrew, Arabic, Latin and Portuguese documents were not run.

## 8. Build and verification

| Check | Result |
|---|---|
| `cargo fmt -- --check` | clean |
| `cargo clippy --all-targets --locked -- -D warnings` (CI settings) | clean |
| `cargo test --locked` (CI settings) | **1 110 passed, 0 failed**, 8 ignored (the corpus harnesses), 82 result groups; includes the 9 `evaluation_rules.rs` tests |
| `python3 -m unittest discover -s tests/evaluation -p 'test_*.py'` | **74 passed** |
| `python3 -m unittest discover -s tests/corpus -p 'test_*.py'` | **26 passed** |
| `npm ci`, `npm run typecheck`, `npm run lint`, `npm run i18n:check` | pass (i18n: 6 languages, 2 986 English keys) |
| `npm test` | **1 053 passed** in 89 files |
| `npm run build` | pass |
| `recogniser-source.sha256` | 166 of 166 |
| A1.5 `candidate-source.sha256` | 278 of 279 (the harness, changed on purpose) |
| `evaluation-e2.sha256` | 33 of 33 (it omits `docs/evaluation/README.md`, which E2 changed: minor) |
| E2 `evidence/SHA256SUMS` | 513 of 513 |
| Recogniser diff `08c6faa`..`97502d2` (src, Cargo.toml, Cargo.lock, build.rs) | empty |
| History (`FREEZE.md`, checksums, approval, decision package, regression, data, rehearsal, evaluation-audit, reviews, known-weaknesses, `docs/corpus`) | unchanged `767e226`..`40d5d59` |
| CI | not triggered (no push to a CI branch, no pull request) |

**Harness binary** (`evidence/build/harness-builds.md`). Rebuilt with
`cargo test --release --locked --test real_document_editions --no-run` in
`/home/user/abidingword/src-tauri`, fresh target directory, empty `../dist`,
rustc 1.97.0 (2d8144b78), cargo 1.97.0, Ubuntu 24.04.5, x86-64:
**`800fb43c9d09e85e7c7475702ff3332ebc45a3ce44a0204220f3d64756454473`,
identical to the pin.** The same commit built at another path gives
`d3cdcd77…`; the binary embeds its build path (`CARGO_MANIFEST_DIR`), as
E2 §1 states. The frontend build state does not matter: the same clone rebuilt after `npm run build` filled `../dist` (102 files; the app crate recompiled) gave `d3cdcd77…` again. No protocol override was used to obtain any
official verdict in this review.

## 9. The strict fidelity policy and its implications

E2 counts as wrong text everything except identical, canonically equivalent
and white-space-type differences. The review supports that as the default:
on real PDFs it caught real corruption that A1.5 excused (§7), and no
material change escapes it (§4). Its implications, for the owner to accept
or change by a new rule version:

* **It is only as faithful as the reference text.** Under the strict rule
  every artefact of the reference reader becomes a failure (R1). Before
  strictness is accepted, the reference text must be right.
* **Typographic and invisible characters**: soft hyphen, ZWSP, ZWJ/ZWNJ
  (orthographic in Persian and Indic scripts, so rightly strict there),
  LRM/RLM, BOM, ligatures (`ﬁ`, typical of PDF text), Arabic presentation
  forms, full-width forms: each is wrong text. Documents whose text layer
  holds these will fail even if the visible text is right; that is a
  correct signal about the recogniser's output, but the owner should
  expect it.
* **Case**: small-capital divine names (`\nd`) printed as capitals are
  wrong text against a mixed-case reference. None of the 13 bundled
  references uses `\nd`, so this was not observed.
* **Spacing beside punctuation** (French `:`, `;`, `!`, `?` with or without a
  narrow no-break space) is wrong text when absent or added, faithful when
  only the kind of space differs.
* **Real archives**: 12 of the 13 eBible USFM zips hold `keys.asc` and a
  `.css` and are refused as published; the builder brief says to take
  them out. The sealed reference is then not byte-identical to the
  publisher's archive; provenance should record the publisher's archive
  hash and each member's.
* **The product, not the evaluation.** The importer reads Bibles with the
  same `usfm::parse`; on the whole-edition runs its text agreed with the
  inventory's in every verse (`importer_comparison`: 31 086 of 31 086
  engbsb, 7 939 of 7 939 grcsbl). A person who imports the bundled
  engbsb, engwebp, fraLSG or grcsbl therefore reads the inserted space in
  the 528 verses of §2 R1. That is a Scripture-fidelity defect in the app,
  outside this review's remit and the frozen recogniser; it is recorded
  for the owner's backlog, not fixed here.

## 10. Recommended protocol changes (not implemented; after owner approval)

1. **Reference text (R1).** A new inventory rule version with an
   evaluation-owned USFM text reader that follows the specification (notes
   removed without a space; milestones; `~` as U+00A0; `//` removed;
   character spans closed by their own marker), refusing references with
   unclosed notes or spans as reference problems; and a second text reader
   in the check, written apart, whose per-verse text digests the scorer
   compares with the harness's. Re-run the real-reference comparison of
   this review; expected result: 0 reader-artefact mismatches.
2. **Pass means evidence (R2).** Refuse or force to exploratory any scored
   document whose reference prints no scorable verse; add an evidence floor
   below which the verdict is *inconclusive*; print verified, printed
   scorable, printed unsupported and refused-whole counts in the verdict
   line.
3. **Composition (R3.1).** Either (a) the protocol states minima and fixed
   counts that do not depend on the builder's choices (total, control 3,
   mixed 2, scored + exploratory 19, authentic ≥ 8), and the check tests
   them as inequalities; or (b) the builder declares the composition, the
   owner commits it as a protocol revision, and only then is the set
   sealed. Resolve 24 against 28 first.
4. **Strictness (R3.2) and D1 (R3.3)**: record the owner's acceptance in the
   protocol, with any relaxation as a new fidelity rule version.
5. **Tool identity.** Add to the protocol the SHA-256 of `score-set.py`,
   `check-set.py`, `fidelity.py`, `reference_inventory.py`,
   `usfm-books.tsv` and the runner, and have the owner check the score's
   `anchors` against them and against an owner-held record of the
   protocol's own hash (a committed modified tool otherwise gives an
   "official" verdict, §6). Refuse unknown command-line options.
6. **Binary pin.** Re-pin in a protocol revision to the assessor's own
   build, recording path, toolchain and `../dist` state; or, in a later
   harness version, find `docs/corpus` at run time so the pin does not
   depend on the path.
7. **Unicode versions.** State both (Python 15.1, Rust 17.0) and make the
   scorer refuse (as a reference problem) text with code points unassigned
   in its own Unicode version, rather than relying on a disagreement.
8. **Wording.** E2 §2 says the scorer finds "a reference the harness did not
   report"; it can only re-check references the harness lists. Say that
   omissions are guarded by the binary pin.

## 11. Reproduction limits

* The disclosed corpus (113 editions) and every authentic published
  document were unavailable; real coverage is 13 real references and
  documents generated from them (§7).
* Hebrew and Arabic were tested with vectors only (no real reference).
* The fake-harness experiments edit the pinned harness's output; they test
  the scorer's own checks, not a realistic harness defect.
* The review used the same toolchain and path as E2's build; its exact
  binary match does not show reproducibility in another environment (the
  other-path build differs, as expected).
* Windows was not built or tested.

## 12. Boundaries kept

No production recognition code, evaluation script, scoring rule, protocol,
frozen binary, historical evidence or expected result in this repository
was changed. The simulated protocol revision and the modified scorer
existed only in scratch repositories outside it. Collection H was not
accessed; no seal of a withheld set was made; no withheld recognition was
run. Nothing was merged, released, tagged or published; no updater, no
paid provider call, no CI run. This review stops at owner review and does
not authorise sealing or running Collection H.

## Evidence index

`evidence/README.md` lists every file, how it was produced and how to
reproduce it. `evidence/SHA256SUMS` checksums them.
