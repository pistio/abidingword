# Evaluation candidate A1.5-E3

**Status: a candidate for one focused independent verification of the
evaluation machinery.** It corrects how the frozen A1.5 recogniser is
*measured*, after the independent review of E2
(`../evaluation-e2-review/REPORT.md`: E2 NOT READY, blockers R1 and R2, owner
decisions R3). It does not change the recogniser. It is not approved, not
sealed, not run on any withheld set, not merged and not released. Collection
H was not accessed. The A1.5 and E2 records, the review and every historical
result are unchanged. **This record is the author's; it is not an
independent approval.**

## 0. In plain words

* **R1, the reference text, is corrected.** E2 read each edition's verse
  text with the app's own USFM reader, which puts a space where a footnote
  was removed ("heel. ”" for the edition's "heel.”") and lets an unclosed
  note or alternate number hide the verses after it. E3 reads the text with
  two new readers of its own, written apart in Rust (the harness) and
  Python (the check), from the USFM 3 specification. They agree with each
  other on every one of the 311 761 verses of the 13 bundled references,
  and with the E2 reviewer's independent extractor on all but 4 (no-break
  spaces the editions print, which E3 keeps). All 528 verses the review
  found mis-spaced now read as the editions print them. Malformed
  references are refused at the seal; no verse is ever dropped silently
  (50 000 mutated inputs, both readers). The scorer compares the two
  readings verse by verse; any difference: not a protocol run.
* **End to end on real text:** a whole Berean Standard Bible made from the
  edition's own words had **154** verified "wrong text" verses under E2,
  every one the reader's; under E3, **0**. SBL Greek New Testament: 335 →
  **1**, the one genuine recogniser error the review found (ACT 28:28).
  Recognition itself is identical (same verified counts): only the
  measurement changed.
* **R2, a pass needs evidence, is corrected.** A run that verifies nothing
  now gets **inconclusive**, never *pass*; safety (criteria 1–4) and
  evidence (the protocol's floor: by default one faithful verified verse)
  are judged apart, and every verdict line carries its counts. A scored
  document whose reference prints no scorable verse is refused at the seal.
  The review's vacuous set, which E2 passed, is now refused at the seal; a
  zero-coverage set with sound references is *inconclusive*.
* **The bounded findings are corrected:** unknown or repeated options are
  refused by every tool; the protocol names every tool by SHA-256 (a
  modified scorer committed to the checkout no longer gives an official
  pass); publisher archives are recorded apart from the USFM-only packages;
  the Unicode versions are stated correctly (harness 17.0.0, scorer 15.1.0)
  and enforced; the binary's environment-specific identity is recorded.
* **Owner decisions are prepared, not taken** (`OWNER-DECISIONS.md`): 24 or
  28 documents, the D1/D6 wording, any additional evidence floor, the tools
  and binary. Until then the protocol's gate stays shut
  (`withheld_execution_permitted: false`, `required_composition: null`).
* No authentic published document was available, so **no new recognition
  statistic is given**.

## 1. Identities

| | |
|---|---|
| Starting point | `7772d0b06bcf766d7f3f5154fe76566396a6d444` (branch `claude/a1-5-e2-eval-review-2ldlj5`, the E2 review) |
| **E3 source commit** | `816e643fc3122f69d41710781575abe480922a09` (tree `2b982f0c25378628196559e60404d92c19d4fe18`), branch `claude/a1-5-e3-evaluation-corrections-jtt5cw`; the commit after it adds only this record, `evaluation-e3.sha256`, `recogniser-source.sha256`, `evidence/checks/` and `evidence/SHA256SUMS` |
| **Recogniser (unchanged)** | `08c6faac8c92c6c791ec1e7bbbe997362fa1059f`. `git diff 08c6faa 816e643 -- src-tauri/src src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/build.rs` is empty; `src-tauri/src` is tree `8cb224c1e059edc3a7750f4c1ea1649603c0de95` in both. `recogniser-source.sha256` (E2's, copied unchanged): 166 of 166 |
| No production Scripture file changed | nothing under `src-tauri/src/` (and so nothing in `src-tauri/src/scripture/`) |
| **Harness binary** | `real_document_editions`, release: **`b4f51a62a51ef5085ca12900ba575ee04cc04f0b99f0fccf88db8890a141b63d`**, pinned in `protocol-e3.json` with its build (`evidence/build/harness-builds.md`): `/home/user/abidingword/src-tauri`, rustc/cargo 1.97.0, webkit2gtk-4.1 2.52.6, Ubuntu 24.04.5 x86-64, empty `../dist`. **Environment-specific**: the binary embeds its build path |
| E2 binary, reproduced | `800fb43c…` rebuilt here before any change: identical to E2's pin (the third reproduction) |
| Rule versions | report `aw-eval-3`; fidelity `aw-fidelity-2` (**unchanged**); inventory `aw-reference-inventory-2`; text reader `aw-usfm-text-1` (`docs/evaluation/usfm-text-rules.md`); evidence `aw-evidence-1` |
| **Protocol** | `docs/evaluation/protocol-e3.json`, SHA-256 **`43f0468941af9ac04149318ea885f1a49c1f298d97a951132b12ea1e31a41937`**. `protocol-e2.json` is kept, unchanged and unused |
| Unicode | harness NFC: Rust `unicode-normalization` 0.1.25 = **Unicode 17.0.0**; scorer: Python 3.13.18 `unicodedata` = **Unicode 15.1.0** (E2 said 16 for Rust: wrong; corrected) |

**Approved-tool identities proposed** (in `protocol-e3.json`,
`approved_tools`; the owner approves them by approving the protocol's
SHA-256 above):

| Tool | SHA-256 |
|---|---|
| `scripts/evaluation/score-set.py` | `16a84231be46ee425ec10dc7114bdd7f80abc816633d58e568ba389dfc8bf9e1` |
| `scripts/evaluation/check-set.py` | `8059b0910d5c2e6036b2975dedbbdd07ab448efc538684e6430365eca110814a` |
| `scripts/evaluation/seal-set.py` | `7e49b1b1abcf308c7411b86c38a7277da3d9eb31d44d9d1268f35b135b3839da` |
| `scripts/evaluation/run-withheld-set.sh` | `0930fd5cecd551609fdc67f81872289996333f68a0bfc5959d89f6eed9c5dd33` |
| `scripts/evaluation/fidelity.py` | `421a8dd29ba96f72f3d5fa6b939045011e4def86fdc7541c0aae51fabde4fa33` |
| `scripts/evaluation/reference_inventory.py` | `40221f22d83b82cae3daddeadedd7b4dff5bbd4a01103ab0333251523f90a5c8` |
| `scripts/evaluation/usfm_text.py` | `6f733a37cb3a719462e6f9f3ba387ac8c55a30889a9cbfdf7d625fdc993d73fe` |
| `docs/evaluation/usfm-books.tsv` | `4925aa88f5ce46cb4de1d0ad18d27a1256bde3c63cb959b265a64a1269819248` |

`evaluation-e3.sha256` lists the SHA-256 of every file E3 adds or changes
(and the unchanged E3 inputs); `sha256sum -c` from the repository root.

## 2. R1: the reference text

**The defect** (review §2 R1). E2's inventory took verse text from
`usfm::parse` (the app's reader), which replaces each note with a space, ends
a note at *any* note end marker, ignores USFM 3 milestones, and lets an
unclosed `\va`, `\vp`, `\ca`, `\fig`, `\rq` or `\cat` hide the rest of a
book. E2's second inventory copied that reader's regular expressions and
read labels only, so the scorer's cross-check could not see any of it.

**The correction.**

| | E3 |
|---|---|
| Rules | `docs/evaluation/usfm-text-rules.md` (`aw-usfm-text-1`), from the USFM 3 specification: USFM white space is U+0020, tab and newline only (a no-break space is text); the one space after an opening marker is markup; **a note is removed with its content and nothing is put in its place**; milestones, attribute lists, `\va`/`\vp`/`\ca`/`\fig`/`\rq`/`\cat` are not text (alternate and published numbers are recorded beside the verse or chapter, never in its identity); `~` is U+00A0, `//` nothing; paragraph and table-cell boundaries are one space; nothing is normalised |
| Harness reader | `src-tauri/tests/eval_support/usfm_text.rs`: one streaming pass with a stack of open spans; no regular expression; no code from the app |
| Check reader | `scripts/evaluation/usfm_text.py`: a lexer, a tree of paragraphs and spans, a walk; no code, table or expression from the Rust reader or the app |
| Malformed structure | a problem, never a guess: an unclosed or mismatched note or span, a nested note, a span opened inside another without `+`, an unterminated or unpaired milestone, an attribute list interrupted, a bar outside a span, an unknown or private marker, a verse or chapter number missing, text after a chapter number or outside any verse, a control character in verse text, two `\id`. **Every structural boundary closes what is open (and reports it), so nothing can swallow the verses after it.** A reference with any problem is refused at the seal and before the run |
| No silent loss | each reader counts the `\v` markers apart from its reading and reports a problem if any is not a verse |
| Inventories | `reference.rs` and `reference_inventory.py` (`aw-reference-inventory-2`) read labels and text with their own reader; every verse is recorded with its text's SHA-256 and status; a verse number covered twice (a duplicate, or a bridge overlapping another label) is ambiguous (`verse_duplicate`); a number inside a sound bridge cannot be validated (`inside_bridge`) |
| Scorer | compares the harness's per-verse records with the check's (written before the run to `check-verses/`, covered by the committed OUTPUTS.sha256, and bound by CHECK.json's text digest): label, status, line and text digest of every verse. **Any difference makes the run not a protocol run.** Code points unassigned in the scorer's Unicode version in a reference are refused at the seal; in a fidelity record they make the report untrusted |

**Evidence** (all reproducible; `evidence/`):

| Required evidence | Where | Result |
|---|---|---|
| 1. The footnote-adjacent punctuation cases from real references | `test_e3_rules.BundledReferences.test_the_review_528…` and `evaluation_references.rs` (both in CI); `evidence/readers/bundled-crosscheck.json`; `evidence/after/real-references-e3.json` | all **528** of the review's verses read with the edition's own context and without the app's space, in both readers; real documents: 0 reference artefacts (§6) |
| 2. All 13 bundled editions | `tests/evaluation/bundled-references.expected.json` (archive SHA-256 checked against each `SOURCE.md`, verses, empty verses, an aggregate digest of every verse), asserted by **both** readers in CI | 311 761 verses, 0 problems, every `\v` a verse; Rust = Python on all 311 761 (labels, lines, text digests, alternate numbers) |
| 3. Malformed cross-references and character spans | `tests/evaluation/usfm-text-vectors.json` (59 vectors, both readers in CI); `test_e3_rules.SealRefusesMalformedReferences`; rehearsal `malformed` | refused with the named problem, every verse kept; the review's unclosed `\x` and `\va` and its milestone case refused at the seal |
| 4. Missing, duplicated and unusual labels | vectors (`verse-number-missing`, `labels-as-written`...); `test_e3_rules.Labels`; `evaluation_rules.rs` (E2 fixture, bridges) | kept as written, classed; duplicates and overlapping bridges ambiguous; numbers inside bridges unvalidatable; a gap is a verse the edition lacks |
| 5. Unicode in English, German, Korean, Greek, Hebrew and others | vectors `unicode-*` (English, German NFD, Korean jamo, polytonic Greek NFD, Hebrew with cantillation and RLM, Arabic/Persian ZWNJ, French/Spanish/Portuguese, Latin); the 13 editions (English, German, French, Spanish, Portuguese, Korean, Greek) | kept code point for code point; no normalisation; the 4 verses with a printed no-break space keep it |
| 6. Neither reader can silently omit a verse | both readers' accounting; property tests in both languages (CI); `evidence/readers/fuzz-50k.json` | 50 137 inputs (38 333 refused, 11 804 clean): **0 silent losses** in either reader; **50 137 of 50 137** identical between the readers (verses, lines, problem codes) |

## 3. R2: a successful evaluation must contain evidence

| Requirement | E3 |
|---|---|
| Refuse at the seal scored references with no scorable verse | `check-set.py`: a scored document whose reference prints no scorable verse in its declared books (an empty package, prose only, only unsupported or only empty verses) is refused; so is a declared book the reference does not print |
| Classify unsupported-only references before sealing | each reference's `evidence_class` (`scorable`, `numbered_but_empty`, `unsupported_only`, `outside_declared_books_only`, `no_verses`) and printed counts in the check report (`reference_evidence`); exploratory documents may hold such references (noted) |
| Separate safety/fidelity and evidence outcomes | `safety_verdict` (criteria 1–4, unchanged) and `evidence_verdict` (the floor), reported apart |
| INCONCLUSIVE when no faithful verse is verified | verdict = *not a protocol run* / *fail* (any criterion) / **inconclusive** (criteria hold, evidence below the floor) / *pass* (both) |
| Counts beside every result | the verdict line: verified (faithful, wrong text), printed scorable, verified references the edition lacks, printed unsupported, held, in refused chapters, missing, documents completed / refused whole / failed; per document and per group: held, refused, missing columns added |
| A protocol-defined floor, conservative by default | `evidence_floor` in the protocol, fixed before any set: **at least one faithful verified verse** (`aw-evidence-1`). No readable floor, an unknown measure or a non-numeric value: no pass. **No percentage threshold is introduced and the approved coverage policy (no pass mark) is unchanged**; three owner slots (documents with evidence, faithful share, documents refused whole) are `null` |
| A gate preventing final withheld execution | `withheld_execution_permitted: false`: the runner refuses; no score under it is a protocol run |

**Demonstrated** (synthetic, real binary, official path under a simulated
revision): the review's vacuous set (blank, Tobit-only, prose, a reference
without USFM) was **pass** under E2 and is **refused at the seal** under E3;
a zero-coverage set with sound references was **pass** (0 verified, 0.0 %)
under E2 and is **inconclusive** under E3 ("INCONCLUSIVE (safety pass;
evidence insufficient): verified 0 (0 faithful, 0 wrong text) of 25 printed
scorable verses; … 25 missing (not found); scored documents 1 completed, 1
refused whole, 0 failed to run, of 2"). `test_e3_rules.Evidence` covers each
floor slot, a protocol without a floor, and wrong text with no evidence
(*fail*, not *inconclusive*).

## 4. The bounded findings (review §6, §10)

| Finding | Correction | Shown by |
|---|---|---|
| Unknown scorer options silently ignored (`--Protocol=`) | `score-set.py`, `check-set.py` and `seal-set.py` refuse any unknown or repeated option (exit 2) | `test_e3_rules.Options`; rehearsal `unknown_option` (E2 exit 0, E3 exit 2) |
| "Official" meant "the tools committed here", not "the approved tools" | `approved_tools` in the protocol; official scoring requires every tool in use, and the runner, check, inventory and reader recorded in RUN.txt, to match; every score states the protocol's SHA-256 for the owner to check | `test_e3_rules.ToolIdentity`; rehearsal `modified_committed_scorer`: E2 **pass** on a failing run, E3 *not a protocol run* (`tools_are_the_approved_ones`) |
| Runner accepted any expected binary hash | it refuses one that is not the protocol's pin, and refuses while the gate is shut | `test_withheld_set_tools` (`test_another_binary`, `…_under_the_pinned_hash`), `test_e3_rules.ExecutionGate` |
| Publisher archives not recorded apart from stripped USFM-only packages | `reference-sources.tsv` in every seal: the publisher's archive and its SHA-256, every kept member (checked against the package byte for byte), every removed member | `test_e3_rules.ReferenceSources`; builder brief updated |
| Python/Rust Unicode versions misstated | 17.0.0 (Rust) and 15.1.0 (Python), in the protocol; reports state theirs; any other is not a protocol run; unassigned code points refused or distrusted | `test_e3_rules.UnicodeVersions`, `ExecutionGate.test_the_committed_protocol…` |
| Binary reproducibility | recorded (`evidence/build/harness-builds.md`): the path is embedded; the E2 pin reproduced exactly; E3 built incrementally and again after `cargo clean --release -p abiding-word` | §1 |
| Wording: the scorer "finds a reference the harness did not report" | the scorer re-checks the references the harness lists; omissions from that list are guarded by the binary pin (unchanged; stated here as the review asked) | review §6 |

## 5. The 528 reference-text artefacts, investigated

* **Cause, confirmed.** In each of the 528 verses an edition places a
  footnote or cross reference directly against punctuation (`heel.\f …\f*”`,
  `Γαλιλαίᾳ\f …\f*,`, `gerahs\x …\x*)`, `(\x …\x*c’est`). The USFM
  specification keeps the white space around a note and adds none; the
  app's reader puts a space in its place. Distribution, as the review
  found: engbsb 156, grcsbl 356, engwebp 14, fraLSG 2 (E2 kinds:
  `punctuation_spacing` 527, `word_boundary` 1).
* **Resolved.** Both E3 readers read every one with the edition's own
  context (checked against the review's own context windows, in CI). The
  harness's comparison with the app's importer (kept, comparison only) still
  shows these verses differing (`importer_comparison.text_differs`: 156 in
  the whole BSB, 356 in the SBLGNT): that is the app, not the evaluation.
* **Four further differences, explained.** Against the reviewer's extractor,
  E3 differs in 4 verses (engwebp JHN 5:11, ACT 20:35, ACT 22:21; porbr2018
  ZEC 7:13), each a no-break space (U+00A0) in the edition's verse text.
  USFM white space does not include it, so E3 keeps it; the reviewer's
  extractor and the app make it a space. The fidelity rule treats the two
  as the same layout (faithful), so no verdict depends on it.
* **The product defect is recorded, not fixed** (outside this task and the
  frozen recogniser): `OWNER-DECISIONS.md` §7.

## 6. Before and after

Synthetic sets through seal, run and official score with the real binaries
(`evidence/before/rehearsal-e2.json`, `evidence/after/rehearsal-e3.json`):

| Set | E2 tools and binary | E3 tools and binary |
|---|---|---|
| clean (faithful documents) | pass | pass (every validity condition met) |
| notes against punctuation, faithful document | **fail: 23 false mismatches** | **pass: 0** |
| unclosed `\x` before a later `\f`; unclosed `\va` | fail: 2JN 1:4–7 vanished from both inventories; HAG 1:4–2:23 read as empty (3 of 38 printed); nothing flagged | **refused at the seal** (`note_unclosed`, `dropped_unclosed`) |
| a USFM 3 milestone | fail: its attributes in the reference text | pass |
| vacuous (blank, Tobit only, prose, no USFM) | **pass, 0 verified** | **refused at the seal** |
| zero coverage, sound references | **pass, 0 verified, 0.0 %** | **inconclusive** |
| a real wrong word | fail | fail |
| `--Protocol=` given to an official score | ignored, exit 0 | refused, exit 2 |
| a modified scorer committed, on a failing run | **official pass** | not a protocol run |

Real references, documents made from the editions' own words (by the
reviewer's extractor), run through each binary
(`evidence/before/real-references-e2.json`, `evidence/after/real-references-e3.json`):

| Document | Verified (both) | E2 mismatches (reader) | E3 mismatches |
|---|---:|---:|---:|
| engbsb whole Bible | 30 977 | 154 (154) | **0** |
| grcsbl New Testament | 7 671 | 335 (334) | **1** (ACT 28:28, the document's reading) |
| engbsb Genesis | 1 526 | 8 (8) | 0 |
| grcsbl Matthew | 1 028 | 38 (38) | 0 |
| engwebp Daniel | 356 | 1 (1) | 0 |
| fraLSG Ephesians | 155 | 1 (1) | 0 |
| kor1961 Ruth, deu1912 Ruth, grcbyz 3 John | 85, 85, 15 | 0 | 0 |

In every E3 run the scorer's verse-by-verse comparison of the two readings
found no difference and no integrity issue. These are generated documents;
they qualify the measurement, not the recogniser.

## 7. Files, and changed test expectations (none hidden)

`git diff --stat 7772d0b 816e643` lists every change. New: the
two readers, `usfm-text-rules.md`, `protocol-e3.json`,
`evaluation_references.rs`, `test_e3_rules.py`, the vectors and bundled
expectations, `synthetic/rehearse.py`, this folder. Changed: the harness
(`real_document_editions.rs`: schema `aw-eval-3`, the reader, Unicode
version, `inside_bridge`), `reference.rs`, `evaluation_rules.rs` (the reader
module; one new test), the Python tools, the builder brief and README, the
audit index (two rows).

Changes to existing tests, each a format or wording change, none weakening
an assertion:

* `test_withheld_set_tools.py`: the fake reports are E3's (`aw-eval-3`,
  inventory `-2`, reader rules, Unicode version, per-verse records); seals
  hold `reference-sources.tsv`; the test protocol approves the tools in use,
  states both Unicode versions, opens the gate and sets the default floor;
  `score()` also swaps the verdict line; the group totals gained `held`
  (10 = 8 + 2; no value changed); `test_another_binary` now expects the
  runner's earlier refusal (an expected hash that is not the protocol's
  pin), and a **new** test keeps the old path (another binary under the
  pinned hash: "is not the frozen").
* `test_e2_rules.py`: "not a report of the E2 harness" reads "E3".
* `evaluation_rules.rs`: includes the reader module; the E2 inventory
  fixture's expected digests and counts are unchanged and still pass.

No existing test that expected *pass* now gets *inconclusive*: none had a
scored set verifying nothing.

## 8. Historical results, and real documents

* Unchanged since `7772d0b`: `../FREEZE.md`, `../candidate-source.sha256`,
  `../SET-H-APPROVAL.md`, `../set-h-decision-package.md`, `../regression.md`,
  `../data/`, `../rehearsal/`, `../reviews/`, `../known-weaknesses.md`,
  `../evaluation-audit/`, `../evaluation-e2/`, `../evaluation-e2-review/`,
  `docs/corpus/`, `docs/evaluation/protocol-e2.json`, `src-tauri/src/`.
* E2's `evaluation-e2.sha256` now verifies 22 of 33: the 11 others are E2
  files E3 changes on purpose (as E2's record says of A1.5's harness).
* **The 113 disclosed editions were not rerun** and no authentic published
  document was available (not in this environment; `ebible.org` is not
  reachable). The historical A1.5 figures (1 980 257 verified correct, 9
  wrong) stand as published. No new recognition statistic is estimated; the
  real-reference runs above use generated documents and measure the
  measurement.

## 9. Remaining limitations

1. **Owner decisions open** (`OWNER-DECISIONS.md`): composition 24/28, D1/D6
   wording, any additional evidence floor, approval of the tools and
   binary, acceptance of the strict fidelity rule. The gate stays shut until
   then.
2. **Same author.** Both readers, both inventories, the vectors, the
   rehearsals and this record were written by one author in one session.
   The readers were written apart in different designs and agree on
   311 761 real verses and 50 137 adversarial inputs, and the reviewer's
   independent extractor agrees on 311 757 real verses; that is consistency
   and an external cross-check, not independent correctness.
3. **The marker table is a choice.** Which markers are verse text follows
   the specification and the app's established practice (e.g. `\qr`, `\qd`,
   `\qs` are text; `\rq`, `\d` titles and `\qa` are not). Markers outside the
   table refuse the reference rather than guess; a real Collection H
   reference may be refused for markup it legitimately uses (the builder
   then needs another release or the table a reviewed extension).
4. **Strict refusals may cost references.** Unclosed spans, unknown markers
   and unpaired milestones refuse a reference even where the verse text
   would be unaffected. Conservative by design; none of the 13 bundled
   references is refused.
5. **The binary pin is environment-specific** (§1): an assessor elsewhere
   needs a reviewed re-pin.
6. **Fidelity rule unchanged** (`aw-fidelity-2`): its strictness choices
   (review §9) stand for the owner.
7. **The scorer's lookup covers the references the harness lists**; an
   omission from that list is guarded only by the binary pin (unchanged).
8. **The app's importer defect** (528 verses) is outside this evaluation
   and the frozen recogniser.
9. **Unchanged A1.5/E2 limits**: missing text is invisible to verification;
   no completeness auditor for PDF or EPUB; Hebrew right-to-left PDFs verify
   little; PDF memory is not bounded in-process; Windows was not built or
   tested.

## 10. Checks run

The full table, with every figure, is `evidence/checks/checks-summary.md`.

| Check | Result |
|---|---|
| `cargo fmt -- --check`; `cargo clippy --all-targets --locked -- -D warnings` (CI settings) | clean; clean |
| `cargo test --locked` (CI settings) | **1 115 passed, 0 failed**, 9 ignored, 83 groups (E2: 1 110 / 8 / 82) |
| Python: `tests/evaluation`; `tests/corpus` | **122 passed**; 26 passed |
| `npm ci`, typecheck, lint, i18n check, `npm test`, `npm run build` | all pass; 1 053 tests in 89 files |
| Readers on the 13 references; against the reviewer's extractor; the 528 | 311 761 identical; 311 757 identical (4 no-break spaces); 528 of 528 |
| Readers on mutated inputs | 50 137 identical, 0 silent losses |
| Harness binaries | E2 `800fb43c…` reproduced; E3 `b4f51a62…` twice, identical |
| Recogniser identity; history | unchanged (§1, §8) |
| `evaluation-e3.sha256`; `evidence/SHA256SUMS`; `recogniser-source.sha256` | all verify; all verify; 166 of 166 |
| CI | not triggered (no matching branch, no pull request); every step run locally |

## 11. Independent-review acceptance checklist

A verifier should, without relying on this record:

1. **Identity.** Check out `816e643fc3122f69d41710781575abe480922a09`; confirm `git diff 08c6faa
   816e643 -- src-tauri/src src-tauri/Cargo.toml src-tauri/Cargo.lock
   src-tauri/build.rs` is empty; `sha256sum -c` this folder's
   `recogniser-source.sha256` and `evaluation-e3.sha256`; compare
   `approved_tools` in `protocol-e3.json` with `sha256sum` of each tool.
2. **History.** `git diff 7772d0b 816e643 --stat` touches none of
   the files listed in §8.
3. **R1, rules.** Read `docs/evaluation/usfm-text-rules.md` against the USFM
   3 specification (notes and white space, milestones, attributes,
   chapters and verses); challenge the marker table.
4. **R1, independence.** Read both readers; confirm they share no code or
   expressions with each other or with `src-tauri/src/scripture/usfm.rs`.
5. **R1, real data.** Run `cargo test --test evaluation_references` and
   `python3 -m unittest tests.evaluation.test_e3_rules`; run your own
   extractor on the 13 references against E3's (`evidence/readers/`); expect
   only the 4 no-break-space verses to differ from an extractor that
   collapses them.
6. **R1, adversarial.** Write your own malformed USFM (notes, spans,
   milestones, sidebars, labels, Unicode); confirm each is either read as the
   specification says or refused, that no verse is dropped without a
   problem, and that both readers agree (`fuzz_readers.py` takes any
   inputs).
7. **R1, end to end.** Rebuild the harness (expect another SHA-256 at
   another path); run `run_real.py --full --e3` or your own documents; try a
   fake harness whose verse texts differ from the check's: it must be *not a
   protocol run*.
8. **R2.** Try to obtain *pass* from a set that verifies nothing, verifies
   only unsupported content, or whose references hold no scorable verse;
   each must be refused at the seal or *inconclusive*. Check the counts in
   every verdict line.
9. **Integrity.** Unknown options; a modified committed scorer; a runner or
   check other than the approved one; another Unicode version; the gate
   shut; a package member not the recorded one: each refused or *not a
   protocol run*.
10. **Owner decisions.** Check `OWNER-DECISIONS.md` states the 24/28 facts,
    the D1/D6 wording and the floor options neutrally and finalises none.
11. Run §10's checks.

## 12. Readiness and independence

**Ready for one focused independent verification** of R1, R2 and the
bounded findings, from `816e643fc3122f69d41710781575abe480922a09`. **Not ready for any withheld-set
run**: the owner's decisions are open and the protocol's gate is shut.
Nothing was sealed, run on a withheld set, merged, tagged, released or
published; no updater, paid provider or CI run was used (this branch
triggers no workflow; every CI step was run locally with CI's settings).

E3 was written after reading the E2 review and its evidence, by one author,
with knowledge of the recogniser's behaviour on disclosed and synthetic
documents. The reviewer's extractor was used only as a third reading and to
make documents, never copied. This record shows that E3's tools no longer
make the errors the review found; it is not independent evidence that they
make no others.
