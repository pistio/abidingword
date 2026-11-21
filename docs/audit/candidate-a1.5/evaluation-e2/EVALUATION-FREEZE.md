# Evaluation candidate A1.5-E2

**Status: a candidate for independent review of the evaluation machinery.**
It corrects how the frozen A1.5 recogniser is *measured*; it does not change
the recogniser. It is not a seal, a run, a release or a merge. No withheld
set has been built, sealed or run under it, and nothing here says anything
about Collection H, which was not accessed. The A1.5 freeze
(`../FREEZE.md`), its checksums, reports, rehearsals and disclosed results
are unchanged and stay the historical record.

It answers the independent audit (`../evaluation-audit/REPORT.md`): F1
(materially different text counted correct), B1–B3 (verse zero, other
reference content and nonexistent references outside the denominator or
the gate), and C1–C5 (scorer overrides, composition, file-name books, verse
maps, Unicode and reference integrity).

## 1. Identities

| | |
|---|---|
| **Evaluation source commit** | `97502d2c28873c790b2ed0b5cd225d9f178128b5` (tree `f007002652808898a58fab3c7e0c683c4820e3fb`), branch `claude/scripture-recognition-eval-fidelity-mokiah`, started from `767e22678eb7fccae53625df2c9bc435c4a6cd53`. Later commits on the branch add only this file |
| **Recogniser (unchanged)** | `08c6faac8c92c6c791ec1e7bbbe997362fa1059f`, tree `be296f2570dcbcc251789b2b1e1bbe2c9e7c2798` |
| Recogniser source identity | `git diff 08c6faa 97502d2 -- src-tauri/src src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/build.rs` is empty; `src-tauri/src` is tree `8cb224c1e059edc3a7750f4c1ea1649603c0de95` in both. `recogniser-source.sha256` (the 166 recogniser lines of the A1.5 `candidate-source.sha256`: `src-tauri/src`, `Cargo.toml`, `Cargo.lock`, `build.rs`, copied unchanged) verifies: 166 of 166 |
| A1.5 `candidate-source.sha256` | 278 of its 279 files verify at this commit. The one that does not is `src-tauri/tests/real_document_editions.rs`, the evaluation harness, which E2 changes on purpose. The A1.5 file is kept as it was: it describes `08c6faa` |
| **Evaluation files** | `evaluation-e2.sha256` (§3): every harness, rule, tool, protocol, test and fixture file of E2, with its SHA-256 |
| **Harness binary** | `real_document_editions`, release: SHA-256 **`800fb43c9d09e85e7c7475702ff3332ebc45a3ce44a0204220f3d64756454473`** (pinned in `docs/evaluation/protocol-e2.json`) |
| Its build | `cargo test --release --locked --test real_document_editions --no-run` in `/home/user/abidingword/src-tauri`, rustc 1.97.0 (2d8144b78), cargo 1.97.0, Linux x86-64 (Ubuntu 24.04), the `Cargo.lock` of `08c6faa`. Built twice, incrementally and again after `cargo clean --release -p abiding-word` (the app crate and the harness rebuilt from source): identical |
| Rule versions | report `aw-eval-2`, fidelity `aw-fidelity-2`, reference inventory `aw-reference-inventory-1` |
| Protocol | `docs/evaluation/protocol-e2.json`, SHA-256 `24e91c12e3c5f1e3e0698e4f594cdc107c803e0416f084eef6b20a4b9405aed4` |

**The A1.5 evaluation hashes are not valid for E2.** The A1.5 pins of
`../FREEZE.md` §5a (check-set `01a13dde…`, seal-set `e343202e…`, runner
`8515e92c…`, scorer `658de955…`, tests `8cc2e2d0…`, brief `bf7008d8…`) and
the A1.5 harness binary `920f7e28…` describe the A1.5 evaluation only. E2's
tools and binary have other hashes (§3), and the E2 scorer never trusts a report
of the A1.5 harness (it has no `evaluation_schema`): such a run is *not a protocol run*.

**The binary identity is environment-specific.** In this environment a
release build of the *frozen* harness source gives `e396328c…`, the same as
the independent audit's build and not the frozen `920f7e28…`: the source
and toolchain are the same, the build environment is not. The harness also
compiles in its own path (`CARGO_MANIFEST_DIR`, to find `docs/corpus`). An
independent assessor who rebuilds elsewhere should expect another SHA-256;
the protocol must then be re-pinned to the assessor's build in a new,
reviewed protocol revision, never overridden at scoring time.

## 2. What changed, and why

### F1. Strict text fidelity (harness and scorer)

A1.5 counted a verified verse correct when its text equalled the
reference's after removing **all** white space, or ignoring case, or (as
`typography`) after dropping every character that is not alphanumeric,
which also dropped decomposed accents, breathings and cantillation. The
scorer then excused `typography` and `unicode`. So word joins and splits,
LORD/Lord, apostrophes, moved commas, question marks, editorial brackets
and lost marks all counted as correct text, and the class-1 cases were
listed nowhere.

E2 classes every verse's text by one rule, written twice and apart
(`src-tauri/tests/eval_support/fidelity.rs` in the harness,
`scripts/evaluation/fidelity.py` in the scorer):

| Class | Definition | Counts as |
|---|---|---|
| `exact` | the same code points, leading and trailing white space aside | faithful |
| `canonical` | the same after Unicode NFC (canonical equivalence only; compatibility forms are **not** equivalent) | faithful |
| `layout` | the same after NFC and white-space-run normalisation: each maximal run of Unicode White_Space (25 code points, listed explicitly in both implementations) becomes one U+0020, ends trimmed. No space is ever added or removed, so word boundaries are preserved | faithful |
| `mismatch` | anything else | **wrong text** |

Every verified `mismatch` fails criterion 1, whatever its kind. The kind is
a diagnostic label, chosen in this order: material (letters or digits
differ): `extra`, `missing`, `wrong_reference` (another reference verse's
words, any book, any status), `changed`; otherwise `case`,
`word_boundary`, `punctuation_spacing`, `format_characters`,
`compatibility`, `marks`, `word_boundary`, `sign`, `punctuation`.

The harness writes, for **every** usable verse whose text is not
byte-exact, both texts as they are (`fidelity_records`). The scorer classes
each again with its own implementation and recomputes the totals; any
disagreement makes the report untrusted and the run *not a protocol run*.
Exact verses are counted as verified minus the non-exact records, and that
too is checked.

### B1, B2. A neutral reference inventory

A1.5 read the reference through the app's importer, which keeps only what
the 66-book canon can hold and drops the rest (verse 0, chapters above the
canon's counts, other books, unusual labels, a second verse with the same
number) with errors the harness discarded; a verse map in the archive
renumbered it.

E2 reads the reference with a neutral inventory
(`src-tauri/tests/eval_support/reference.rs`): every verse of every USFM
book file, numbers as written, text as the app's USFM reader gives it
(`usfm::parse`, the same text the importer stores). Each verse is
*scorable* (a book of the 66, chapter within its count, verse 1–200 or a
bridge stored on its first verse, given once), *outside the declared
books*, or one of eleven **unsupported** classes: `verse_zero`,
`verse_label_unusual`, `verse_duplicate`, `verse_before_chapter`,
`chapter_zero`, `chapter_label_unusual`, `chapter_beyond_canon`,
`book_outside_canon`, `book_code_unknown`, `book_duplicate`,
`book_unidentified`. Unsupported verses are counted (printed and empty),
listed by label, never scored and never dropped. This implies no change to
the recogniser's canon: they are evaluation categories.

The check (`scripts/evaluation/reference_inventory.py`, written apart, in
Python, independent of the recogniser's canon) reads the same labels before
the run and records them in CHECK.json: the reference's complete coverage
and every unsupported label, declared before any result. The scorer
compares the two inventories (a SHA-256 of every label in reading order, the
counts per class, the scorable keys); any difference: not a protocol run.
The app's importer still runs on the reference beside it, for comparison
only (`importer_comparison`), so what E2 scores differently is visible.

Accounting, checked by the scorer on every report: every printed scorable
verse is exactly one of verified, held back (recognised but uncertain),
refused (in a refused chapter) or not found; every label read is exactly one
of scorable, unsupported (by class) or outside the declared books; every
verified reference is exactly one of verified against the reference,
absent, unvalidatable, or numbered but empty on both sides.

### B3. Full-set reference comparison

A1.5 looked for verified verses the edition lacks only inside chapters the
reference has, and in books it lacks entirely. A whole extra chapter
(Malachi 4 against a 3-chapter Malachi) was counted nowhere.

E2's harness lists every verified reference of the document
(`verified_references`) and looks each one up in the reference's complete
inventory: one the edition does not have is `verified_absent`, with its
reason (`verse_not_in_chapter`, `chapter_not_in_book`,
`book_not_in_reference`, `outside_declared_books`, `empty_in_edition`); one
the edition gives ambiguously (a number or book given twice) is
`verified_unvalidatable`. Both fail criterion 2. The scorer repeats the
lookup on its own against the check's scorable keys (from CHECK.json, made
before the run); if it finds a reference the harness did not report, the
report is untrusted. Verified mismatches whose words are those of an
unsupported reference verse (unsupported Scripture verified as another
reference) are named (`verified_misidentified_unsupported`).

### C1–C5. Seal and scoring safeguards

| | A1.5 | E2 |
|---|---|---|
| C1 overrides | `--frozen-binary` and `--inventory` silently changed run validity; the score recorded neither | Official scoring takes **no** option that can change the protocol: it reads the committed `protocol-e2.json` beside the scorer, and requires the scorer, its rule modules, the book table and the protocol to be committed and unchanged (`scorer_committed`). `--protocol`, `--frozen-binary`, `--inventory` are refused (exit 2) without `--diagnostic`; with it the score is marked DIAGNOSTIC, its overrides recorded, its verdict always *not a protocol run*, and what the criteria give is `diagnostic_verdict` |
| C2 composition | not enforced | `composition.tsv` in the seal (scored, control, mixed, exploratory, authentic, generated) must equal the manifest's counts (check-set refuses otherwise) and the protocol's `required_composition`. While the protocol has none (now), no run is a protocol run |
| C3 file-name books | `…_gen.md` with `all` was measured against Genesis only | A sealed manifest's `books` is the only selection (`all` or USFM codes; never empty; `-` only without a reference; codes checked against `docs/evaluation/usfm-books.tsv`). The harness never reads a book from a sealed set's file name; a report that did is untrusted. The disclosed corpus's own manifests keep the old rule, and each report says which rule chose its books |
| C4 verse maps | loaded, reference renumbered | A reference must be a zip of USFM books (plus prose, `.htm`) or one `.usfm`/`.sfm`; a verse map or any other member is refused at the seal and before the run; the harness never reads one and reports it as a reference problem (untrusted) |
| C5 Unicode, integrity | no normalisation; a decomposed reference let accent changes pass; non-UTF-8 read lossily | Comparison under NFC on both sides (canonical equivalence only); a non-UTF-8 reference, or a file with more than one `\id`, is refused and reported as a problem; a reference not in NFC is noted (it is compared canonically, so it hides nothing) |

The runner records the protocol it ran under (`protocol <sha256>`; rehearsal
protocols only through `AW_PROTOCOL`); the score records its mode, every
override, and the SHA-256 of the scorer, check, both rule modules and the
protocol, with the commit they are committed in.

## 3. Changed and new evaluation files

`git diff --stat 767e226 97502d2` lists every change. Recogniser source: none.

| File | |
|---|---|
| `src-tauri/tests/real_document_editions.rs` | the harness: E2 comparison, inventory, full-set lookup, report schema `aw-eval-2` |
| `src-tauri/tests/eval_support/fidelity.rs`, `src-tauri/tests/eval_support/reference.rs` | new: the harness's rules |
| `src-tauri/tests/evaluation_rules.rs` | new: their tests (run by `cargo test`) |
| `scripts/evaluation/fidelity.py`, `scripts/evaluation/reference_inventory.py` | new: the scorer's and check's rules |
| `scripts/evaluation/check-set.py`, `seal-set.py`, `run-withheld-set.sh`, `score-set.py` | changed (§2) |
| `docs/evaluation/protocol-e2.json`, `docs/evaluation/usfm-books.tsv` | new: the protocol, the USFM book table |
| `docs/evaluation/builder-brief.md`, `docs/evaluation/README.md` | changed: what a builder must now deliver (composition, books, reference format) and receive |
| `tests/evaluation/test_withheld_set_tools.py` | changed: E2 report format, composition, test protocol, diagnostic scoring (each change listed in §6) |
| `tests/evaluation/test_e2_rules.py`, `fidelity-vectors.json`, `inventory-fixture/`, `inventory-fixture.expected.json`, `synthetic/` | new: E2 tests, golden vectors, fixture, synthetic regression sets |
| `docs/audit/candidate-a1.5/evaluation-e2/` | new: this record and its evidence |
| `docs/audit/candidate-a1.5/README.md` | one row added to its index |

Unchanged and still used: `scripts/evaluation/disclosed-inventory.py`,
`docs/evaluation/disclosed-documents.sha256` (`a9d83ac4…`, the canonical
inventory), `disclosed-editions.tsv`, `transcript-audit.md`,
`scripts/corpus/audit-derived-completeness.py`. `docs/corpus/README.md`
(in the A1.5 checksums) still describes the A1.5 counting; §2 describes E2's.

`evaluation-e2.sha256` lists the SHA-256 of every evaluation file;
`sha256sum -c` from the repository root verifies it.

## 4. Regression evidence, before and after

All synthetic, all invented (`evidence/README.md`). First the failing-before
evidence was taken with the frozen code; then the corrections were made and
the same material run again.

**Golden vectors** (68; `evidence/golden-vectors/`): the frozen comparison
lines, byte for byte, against E2. Of the 58 vectors that are wrong text
under E2, **36 were counted correct by A1.5**; the 22 others were caught by
both; the 10 faithful ones are faithful under both. Rust and Python agree on
all 68 (`evaluation_rules.rs`, `test_e2_rules.py`).

**Synthetic sets** through the real checker, runner, harness and scorer
(`evidence/synthetic/`, `comparison.md`):

| Set | Probes | A1.5 tools and harness | E2 |
|---|---|---|---|
| `e2-fid` | word joining and splitting, case, apostrophes, a moved comma, `?`, brackets, dashes, quotes, decomposed accents and breathings, cantillation (English, Spanish, Greek, Hebrew); a canonically equivalent document | **pass: 171 of 171 verified correct** | **fail: 18 verified mismatches** (punctuation 8, word boundary 4, marks 4, case 2), each listed; the canonically equivalent document faithful |
| `e2-mat` | words changed, a negation, a composed accent lost | fail, 3 wrong | fail, 3 mismatches |
| `e2-ref` | verse 0 (three ways), Joel 4, Tobit, 3 John 2, `12a`, `201`, a verse number twice | pass; printed 195; nothing reported | fail: printed 194 **+ 54 unsupported** listed by class; 3JN 1:7 verified against a number the edition gives twice: unvalidatable |
| `e2-abs` | Malachi 4 verified against a 3-chapter Malachi; Jude 26; Ruth against an Obadiah reference | fail on 23; **MAL 4:1–6 counted nowhere** | fail on **29**, MAL 4:1–6 as `chapter_not_in_book` |
| `e2-name` | Ruth in `probe_gen.md`, books `all` | fail: 22 false "outside the edition" | pass: 22 faithful |
| `e2-map` | a verse map in the reference | pass, Joel renumbered (printed 73) | refused at the seal |
| `e2-utf8` | a Latin-1 byte in the reference | read with U+FFFD, silently | refused at the seal |

The required regression, *an entire additional chapter verified by the
recogniser but absent from the reference*, is `e2-abs-mal`: A1.5 printed 55,
verified 49, outside the edition 0; E2 lists MAL 4:1–6 as verified
references the edition does not have, and criterion 2 fails.

**Tests failing before, passing after**: `tests/evaluation/test_e2_rules.py`
(28) against the A1.5 tools: 27 fail (`evidence/python-tests/`); against E2:
28 pass. `src-tauri/tests/evaluation_rules.rs` (9) tests code that did not
exist before; its before-evidence is the golden-vector probe.

## 5. Historical against new metrics

**The 113 disclosed editions were not rerun.** They are not in this
environment; `ebible.org` is refused by its network policy; and most of them
could not be rebuilt here even with access (the withheld sets' generators
were kept outside Git; LibreOffice conversions are not checksum-reproducible;
eBible rebuilds its files). The committed A1.5 per-edition data
(`../data/editions.json`, `../regression.md`) records verified, correct and
wrong under the A1.5 rule, without the class-1 or typography split, so the
effect of E2 on those figures cannot be computed from committed evidence.
Nothing has been estimated or invented in its place.

What is known: the A1.5 figures (1 980 257 verified correct, 9 wrong, 0 not
in the edition, on 113 editions) were measured under the A1.5 rule and stay
as published; under E2, "correct" would exclude every case, word-boundary,
punctuation and mark difference, and every score would also report verse
0, deuterocanonical books and chapters above the canon's counts as printed
but unsupported, with coverage of all printed verses beside coverage of the
scorable ones. Whether that changes the figures by a little or a lot is
unmeasured. Rehearsal 4 reported
`verified_typography` 0; its class-1 count is not in any committed score.

The old-against-new comparison is therefore complete for the synthetic
evidence (§4) and absent for real documents. A rerun of the disclosed
editions with both harnesses, where they exist, is the first thing an
assessor should ask for (§8).

## 6. Changed test expectations (none hidden)

In `tests/evaluation/test_withheld_set_tools.py` (46 tests, all kept):

* the fake harness writes E2 reports (`report()` builds them, with a
  reference holding exactly the verses they claim) and each set declares a
  composition and has a test protocol; scoring is `--diagnostic` with that
  protocol, and `score()` asserts the official verdict is *not a protocol
  run* before handing the tests the diagnostic verdict;
* `test_a_clean_run_passes_and_counts_only_scored_documents`: S1's
  difference was `typography` and counted correct; under E2 that is wrong
  text, so the clean set now uses a canonically equivalent verse, and the
  group totals use E2's names. That typography is now wrong text is tested
  in `test_e2_rules.py`;
* the criteria are renamed (`1_no_verified_text_mismatch`,
  `2_no_verified_reference_outside_or_unvalidatable`);
* `test_a_reference_holding_a_disclosed_usfm_book` and the builder test use
  the generated 100-verse reference instead of a one-verse one;
* `AuthenticMinimum` calls the new `assess()` signature.

No assertion about the anchors, path containment, the runner's refusals,
execution classes or sealing was weakened.

## 7. Remaining limits and unsupported cases

1. **Required composition unset.** `protocol-e2.json` has
   `required_composition: null`, so no run under it can be a protocol run
   until the owner sets it (a reviewed protocol change). The audit's C2
   quotes D5 as 23 scored, 3 control, 2 mixed, 0 exploratory, 16 authentic,
   12 generated (28 documents); `../set-h-decision-package.md` §3 allocates
   24 documents. The owner must resolve which applies; E2 does not choose.
2. **Binary pin is environment-specific** (§1). Rebuilding elsewhere means a
   new protocol revision.
3. **Shared reader.** The reference's verse *text* is still read by the
   app's own USFM reader (`usfm::parse`, part of the frozen recogniser
   source). The labels are read twice and compared; the text is not. A
   defect in that reader's text extraction would affect app and evaluation
   alike. The harness also compares with the importer, which uses the same
   reader.
4. **Same author.** Both fidelity implementations, both inventories, the
   golden vectors and the synthetic sets were written in one session by one
   author. Their agreement shows consistency, not independent correctness.
5. **Unsupported content is never scored.** The recogniser cannot verify
   verse 0, deuterocanonical books or chapters above the 66-book counts;
   E2 counts and lists them, so coverage of all printed verses is lower than
   coverage of scorable ones (both are reported). Supporting them is not
   implied.
6. **Strictness choices for the owner.** E2 counts as wrong text: spacing
   beside punctuation, invisible format characters (soft hyphen,
   zero-width joiner) and compatibility forms (ligatures), not only material
   changes; and fails criterion 2 for a verse verified against a number the
   edition gives twice. Each is stated; any relaxation would be a new rule
   version.
7. **Diagnostic kinds are approximations** for combined differences (one
   label by precedence); the strict class is exact.
8. **Unicode versions.** Python 3.13 uses Unicode 15.1, the Rust crates
   Unicode 16. A character assigned in 16 could be classed differently;
   the cross-check would then make the run *not a protocol run* rather
   than hide it. Not tested on such characters.
9. **Edge labels.** `4-5a` is unsupported in E2 while the importer stores
   it as verse 4 (shown in the importer comparison). A bridge `4-5` and a
   separate `5` are both scorable (keys 4 and 5), as in the importer.
10. **Unchanged A1.5 limits**: missing text is invisible (verification is
    fidelity to the document read); no completeness auditor for PDF or
    EPUB; Hebrew right-to-left PDFs verify nothing; PDF memory is not
    bounded in-process.
11. Development scripts that read harness reports (`scripts/a1.1/tables.py`,
    `scripts/recon/compare_audits.py`) read the A1.5 format and were not
    changed; they are not evaluation tools.

## 8. Independent-review checklist

An assessor should, without relying on this record's claims:

1. **Identity.** Check out `97502d2c28873c790b2ed0b5cd225d9f178128b5`; confirm the recogniser diff to
   `08c6faa` is empty (§1), `sha256sum -c
   docs/audit/candidate-a1.5/evaluation-e2/recogniser-source.sha256` and
   `sha256sum -c docs/audit/candidate-a1.5/evaluation-e2/evaluation-e2.sha256`.
2. **History untouched.** `git diff 767e226 97502d2 --stat` shows no change to
   `../FREEZE.md`, `../candidate-source.sha256`, `../SET-H-APPROVAL.md`,
   `../set-h-decision-package.md`, `../regression.md`, `../data/`,
   `../rehearsal/`, `../evaluation-audit/` or `docs/corpus/`.
3. **Fidelity rule.** Read both implementations; write adversarial vectors of
   your own (not ours): heavy combining sequences, Hangul jamo, CJK, Arabic,
   Hebrew with and without points, Greek polytonic, Syriac, final sigma,
   dotted/dotless i, every White_Space and format character; confirm Rust and
   Python agree and that nothing material can be classed faithful.
4. **Inventory.** Run both inventories on real eBible USFM zips (notes,
   `\va`/`\vp`/`\ca`/`\cp`, `\qa`, BOM, CRLF, bridges) and confirm digests and
   class counts agree; review the dependence on `usfm::parse` for text
   (§7.3).
5. **Gate.** Try to obtain an official *pass* from a run that should fail:
   an unlisted mismatch, an unreported extra chapter, a dropped unsupported
   verse, a changed protocol, an override, a composition mismatch, a
   file-name book, a verse map, a non-UTF-8 reference. Each must be a
   refusal, a *fail* or *not a protocol run*.
6. **Rebuild** the harness (§1) and compare its SHA-256; if it differs,
   propose a protocol revision rather than an override.
7. **Owner decisions**: the required composition (§7.1); the strictness
   choices (§7.6); whether to accept E2 as the evaluation for any future
   withheld set.
8. **Real documents.** Where the disclosed editions exist, run both harnesses
   on them and publish the old-against-new counts beside the A1.5 figures,
   which must stay as published.
9. Run the checks of §9.

## 9. Checks run

| Check | Result |
|---|---|
| `cargo fmt -- --check` (`src-tauri/`) | clean |
| `cargo clippy --all-targets --locked -- -D warnings` (with CI's `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`) | clean |
| `cargo test --locked` (same settings) | **1 110 passed, 0 failed**, 8 ignored (the corpus harnesses), in 82 result groups. A1.5 recorded 1 101; the 9 added are `evaluation_rules.rs` |
| `npm run typecheck`, `npm run lint`, `npm run i18n:check` | pass (6 languages, 2 986 English keys) |
| `npm test` | **1 053 passed** in 89 files (as A1.5; no interface code changed) |
| `npm run build` | pass |
| `python3 -m unittest discover -s tests/evaluation -p 'test_*.py'` | **74 passed** (46 tool tests, 28 E2 tests) |
| `python3 -m unittest discover -s tests/corpus -p 'test_*.py'` | 26 passed |
| E2 tests on the A1.5 tools (before) | 27 of 28 fail (`evidence/python-tests/`) |
| Golden vectors through the frozen comparison code (before) | 36 of 58 wrong-text vectors counted correct |
| Synthetic sets, A1.5 tools and E2 tools | §4; `evidence/synthetic/comparison.md` |
| Recogniser identity at `97502d2` | `git diff` to `08c6faa` empty; `src-tauri/src` tree `8cb224c1…` in both; `recogniser-source.sha256` 166 of 166; A1.5 `candidate-source.sha256` 278 of 279 (the harness) |
| History untouched | `git diff 767e226 97502d2` is empty for `../FREEZE.md`, `../candidate-source.sha256`, `../SET-H-APPROVAL.md`, `../set-h-decision-package.md`, `../regression.md`, `../data/`, `../rehearsal/`, `../evaluation-audit/`, `docs/corpus/` |
| `evaluation-e2.sha256` | 33 of 33 |
| `evidence/SHA256SUMS` | 513 of 513; every `run-outputs/OUTPUTS.sha256` verifies |
| Harness binary | `800fb43c…` from the incremental build and again after `cargo clean --release -p abiding-word` |
| Frozen harness source rebuilt here | `e396328c…`: the audit's local build, not the frozen `920f7e28…` (§1) |
| CI | **not run**: `ci.yml` runs on pushes to `main`, `feature/**`, `fix/**`, `chore/**` and on pull requests to `main`; this branch triggers none, and no pull request was opened. Every CI step was run locally above, with CI's settings |
| Not run | the 113 disclosed editions (§5); Windows builds or installers; anything on Collection H |

## 10. Readiness

**Ready for independent assessment of the evaluation machinery**: its
rules, code, tests and synthetic evidence can be reviewed and reproduced
from this commit.

**Not ready for any withheld-set run.** The protocol has no required
composition, the binary pin is this environment's, no real document has been
evaluated under E2, and independent review has not happened. Collection H
readiness is not claimed. Nothing was sealed, run, merged, signed, released
or published.

**Independence, stated plainly.** E2 was written after reading the
independent audit and its synthetic probes, by one author, with knowledge of
the A1.5 recogniser's behaviour on disclosed and synthetic documents. Its
evidence is synthetic, made by the same author. It shows that the tools no
longer hide the defects the audit found; it is not independent evidence
that they hide no others, and it says nothing new about the recogniser's
accuracy on unseen documents.
