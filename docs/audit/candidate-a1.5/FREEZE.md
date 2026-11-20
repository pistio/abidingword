# Candidate freeze: Scripture recognition A1.5

**Status: a research-evaluation candidate, recorded for the owner.** Not
an integration, a release or a claim that recognised Scripture is
accurate. Recognised text stays provisional. Set H has **not** been built,
inspected, sealed or run (`set-h-decision-package.md`). The A1.3 freeze
(`docs/audit/candidate-a1.3/FREEZE.md`, `7c7565a`) is unchanged and stays
the historical record.

## 1. The candidate

| | |
|---|---|
| **Code** | `08c6faac8c92c6c791ec1e7bbbe997362fa1059f` (branch `claude/recognition-a1.5-evaluation-qualification`) |
| Git tree | `be296f2570dcbcc251789b2b1e1bbe2c9e7c2798` |
| Source checksums | `candidate-source.sha256`: 279 files at that commit (`src-tauri/src`, `Cargo.toml`, `Cargo.lock`, `build.rs`, `src-tauri/tests`, the derivation script, `scripts/corpus`, `docs/corpus`). Its own SHA-256: `fc2834d11f3b90bde8c8fcc3eb72d0e8438a4f061afb86026eb5692d041ef115` |
| Harness binary | `real_document_editions`, release build at that commit: SHA-256 `920f7e28e6c03f0c2fd7832bb78ce7c2077d6767541f0b86bb02f2fec1ed3a8e`. Built twice (before and after the commit, same source): identical |
| Toolchain | rustc 1.97.0 (2d8144b78), cargo 1.97.0, Linux x86-64, `cargo test --release --locked` |
| Dependencies | `src-tauri/Cargo.lock` unchanged since A1.3; pdf-extract 0.12.1, lopdf 0.42.0, quick-xml 0.42.0, zip 2.4.2 |
| Pipeline | `AW_PIPELINE=structural` |

Changes since the A1.3 candidate (`7c7565a`), oldest first:

| Commit | What |
|---|---|
| `16f100d`, `7a5ff0b`, `eb51969` | PR #52: protocol amendments, mutation suite, its record |
| `22e42a7` | Completeness auditor version 2, fail-closed (`auditor-qualification.md`) |
| `d1d97a2` | Derivation script stops on unknown markers and paragraph classes |
| `179f72f` | PDF layout: raised note letter and verse number as one run (`engbsb`) |
| `e35a920` | Harness: an extra manifest; control documents |
| `2536495` | Hardening after the independent security and canon reviews |
| `08c6faa` | `scripture::numbering`: where printed numbering departs |

Commits after `08c6faa` on this branch change only `docs/`,
`scripts/evaluation/` and `tests/evaluation/` (the evaluation tools and
their tests, pinned in §5a); no recogniser source or test, corpus manifest
or lock file.

**"Immutable" means recorded, not enforced.** No tag was made. A tag or
branch protection on `08c6faa` would make it binding; that is the owner's
call.

## 2. Inputs

Unchanged from A1.3: `docs/audit/v0.24.7/corpus-checksums.sha256` and
Set G's seal, both verified before the run (`CORPUS_OK`, `SETG_OK`).
`spablm-docx-derived-v2` as recorded in `docs/corpus/derived-v2.tsv`.

## 3. Results, all 113 disclosed editions

| | v0.24.7 | A1.3 | **A1.5** |
|---|---:|---:|---:|
| Verified, correct text | 1 956 673 | 1 977 517 | **1 980 257** |
| **Verified, wrong text** | 4 172 | 9 | **9** |
| **Verified, not in the edition** | 7 | 0 | **0** |
| Refused (correct text, not verified) | 44 832 | 45 397 | 45 376 |
| Caught (wrong text, not verified) | 17 723 | 17 706 | 17 518 |

* **The 9 are `spablm-docx` v1's**, a defect of that generated test
  document, not of the recogniser. They stay counted. On the corrected v2
  (`7e5c9080…aa4925`, run with `run-spablm-docx-derived-v2.sh` and the
  frozen binary): 31 098 printed, **31 072 verified, 0 wrong**, 0 not in
  the edition, 25 refused, 1 caught; the same as A1.3 on v2. This is a
  corrected test document, not a better recogniser.
* The change from A1.3 is one edition: `engbsb-rwpdf2col`, +2 740 verified,
  0 wrong (`known-weaknesses.md` §A). Details: `regression.md`.
* JHN 7:52 (`spapddpt-repub2`): still not verified. Book-id ambiguity and
  contradiction, note callers, verse numbers read as words: the A1 and
  A1.1 tests pass.

## 4. Memory and time

Both frozen harness binaries (A1.3 `cc555ac6…`, A1.5 `920f7e28…`: release
builds, same toolchain and `Cargo.lock`), structural pipeline, **one reading
at a time**, A1.3 and A1.5 back to back for each edition, on every EPUB and
PDF of the 113 (80 editions). "Seconds" and "peak" are the harness's own
measure of the reading (`read_seconds`, `read_peak_memory_mb`).

| Format | Editions | Peak MB, median A1.3 → A1.5 | Peak MB, highest A1.3 → A1.5 | Seconds, median A1.3 → A1.5 | Seconds, total A1.3 → A1.5 | Verified A1.3 → A1.5 |
|---|---:|---|---|---|---|---|
| EPUB | 24 | 120 → 119 | 248 → 249 (`G01`) | 13.9 → 14.6 | 350 → 354 | 505 989 → 505 989 |
| PDF | 56 | 185 → 185 | 572 → 566 (`spavbl-rlofodt`) | 44.8 → 45.7 | 2 874 → 2 893 | 895 687 → 898 427 |

* No edition's peak moved by more than 8 MB (largest: `engwebp-pdf`
  −8 MB); no reading took more than 1.12 times as long (`latVUC-epub`).
  The hardening costs about 1 % of reading time.
* PDF +2 740 verified is `engbsb-rwpdf2col` (§3).
* **Disclosed conditions:** the run was cut by a container restart after
  part of the list and resumed, skipping readings already written; for
  part of it, other single-threaded jobs ran on the machine's other cores
  (4 cores; the timing was then pinned to one core). Peaks are unaffected;
  seconds are indicative to a few per cent.
* Per edition: `data/memory-time-a13-a15.json` (peak MB, seconds,
  verified, for each build).

## 5. Checks at `08c6faa`

* `cargo fmt -- --check`: clean.
* `cargo clippy --all-targets --locked -- -D warnings`: clean.
* `cargo test --release --locked`: **1 101 passed, 0 failed**, 8 ignored
  (the corpus harnesses), in 81 binaries.
* Corpus-tool tests: `python3 -m unittest discover -s tests/corpus -p 'test_*.py'`:
  26 passed.
* Interface checks, same `package-lock.json` as the application's
  checkout: `npm run typecheck`, `npm run lint`, `npm run i18n:check`
  (6 languages, 2 986 English keys), `npm test` (**1 053 passed** in 89
  files) and `npm run build`: all pass. The recognition work changes no
  interface code.

## 5a. Evaluation tools, pinned

The recogniser is not part of this section and did not change. These are
the tools that check, seal, run and score a withheld set, as revised after
Codex's review of PR #53 and the follow-up review of `4dd9f8e`
(`docs/audit/candidate-a1.5/set-h-decision-package.md`).

| File | SHA-256 |
|---|---|
| `scripts/evaluation/check-set.py` | `01a13dde21510ad9e9f3b9c301d971e87feffbacdcaee0449d6c2f9180c3411f` |
| `scripts/evaluation/seal-set.py` | `e343202e2cbf6fb35b9fa70b74b6a457b214d5bc9ece9ecc2c101d57c0e23c8f` |
| `scripts/evaluation/run-withheld-set.sh` | `8515e92c329370236f5aab0495b4e08c4f6a76f64f6e8d0abbd59584181385a9` |
| `scripts/evaluation/score-set.py` | `658de955cc4594b5893ba84518ccbdfc7673b95da280edc5056c09835fb9a9bf` |
| `scripts/evaluation/disclosed-inventory.py` | `61afa5818d1e5fc053b80a5018024b5ba042a18fa06861cba01f04d6f2dbb9da` |
| `docs/evaluation/disclosed-documents.sha256` (the canonical inventory) | `a9d83ac45d356997d2c965d1635cb6a4280b26a988625c668aaeb13d28aac2b8` |
| `docs/evaluation/disclosed-editions.tsv` | `9c736eb2394803dfe44b938eee7c88f0881c28f2113bbff2541a9f695cb0801a` |
| `docs/evaluation/builder-brief.md` | `bf7008d84c2dcd2c3f4d8f5e5b58d01475ff8ead8425314c186cc181971be147` |
| `docs/evaluation/transcript-audit.md` | `6552c9095243a1320fb59d47b47ad12e873b9f72faa32737793b05797bd356a8` |
| `tests/evaluation/test_withheld_set_tools.py` | `8cc2e2d0659b91dd00ced7fe5d365d909e1f286a972e3ab13d13741fcded2140` |
| `scripts/corpus/audit-derived-completeness.py` (v2) | `d268175759f92add320530c9b8f5509f9f1b60ba726cc5f861501db5c90ab5f0` |

Final preparation (`a11fd0a`, 20 November 2026): the scorer names, beside
the coverage figure, every scored document refused whole or failed to run
(rehearsal 4 rescored: `rehearsal/protocol-2/run/score-final.*`); the
builder's files no longer name the project (brief, edition list); and
`check-set.py` changed in one comment only, so rehearsal 4's recorded
`check_set` (`d7290e28…`) differs from the pin above by that comment
(`git diff 5717388 a11fd0a -- scripts/evaluation/check-set.py`).

Owner decisions (20 November 2026, `SET-H-APPROVAL.md`): option A
(procedural independence, audited by `docs/evaluation/transcript-audit.md`);
the brief now states the access-route rules and the approved host.

Superseded pins, kept as recorded: brief `f4b63afa…` at `e2e9e62`; at `e73b5d3` check-set `d7290e28…`,
scorer `803a7adb…`, inventory script `e656f38e…`, editions `5241c244…`,
brief `7e1ea823…`, tests `b28e3a6f…`; at `4dd9f8e` check-set `66923a89…`,
seal-set `fa39c981…`, runner `d1b6a36c…`, scorer `13f9d4ae…`, brief
`a21be798…`, tests `df370b5e…`; at `f50c541` runner `66f20eb9…`, scorer
`740b0701…`.

**Follow-up review of `4dd9f8e` (19 November 2026).** It reproduced, with
the committed fake harness only, three ways to turn a failing run into a
pass (an allowed list widened after the run; a report, or an exit code,
changed and OUTPUTS.sha256 regenerated outside the committed record) and
three input paths that escaped the documents folder (`../`, an absolute
path, a link). Done, in `5717388`:

* The scorer now requires `--run-record=<commit>`, the one commit that
  added RUN.txt, OUTPUTS.sha256 and CHECK.json, unchanged since; checks the
  run folder against it and every output against the committed
  OUTPUTS.sha256; reads the seal (manifest, every allowed list) from the
  seal commit RUN.txt names, with the seal folder on disk identical to it.
  Anything missing or inconsistent refuses assessment. The runner commits
  the run record itself, alone, as soon as the outputs are checksummed.
* Every name must be a plain relative path, checked before any file is
  opened; no symbolic link is accepted in the seal, documents or run
  folder; every file must resolve inside its folder.
* Receipts in `data/review-2026-11-19/`: the review's probes reproduce all
  six cases at `4dd9f8e`; on the fixed tools its path probe exits 0 (all
  three refused), and its scorer cases, run step for step through the
  anchored interface, are each refused (the unchanged scorer probe stops at
  its first step, because the scorer refuses it; that stop is not counted).

**Qualification of these tools:**

* `tests/evaluation`: **46 tests** with a fake harness (no recogniser, no
  corpus): every refusal of the runner, every way a document's run can end,
  the four document classes, the scorer's refusals and anchors, path
  containment, the authentic minimum, and the builder's check and seal.
  The 17 added after the follow-up review include its three alterations
  (class `ScoringAnchors`) and three escapes (class `PathContainment`);
  eleven of their safeguards were removed one at a time, and each removal
  failed its test. Before that, thirteen safeguards were removed one at a
  time. The two of `seal-set.py`
  each failed their test. Of the other eleven, 8 failed their test in the
  first pass and 3 did not: two were real gaps (a seal changed in a later
  commit; exploratory documents kept out of the per-group tables), and
  both now have tests that fail without them; the third, the runner's
  no-clobber lock, is backed by the pre-run lock check, which its test
  does catch.
* **Rehearsal with the frozen binary on disclosed documents**
  (`rehearsal/`): three sealed sets, each committed alone, run once and
  recorded. With the canonical inventory the runner refused the protocol
  set before reading anything: every reused disclosed document, reference
  and USFM book was named (`rehearsal/protocol/refused-canonical-inventory.txt`).
  With a rehearsal inventory (`rehearsal/inventory-removed.sha256` lists the
  hashes taken out):
  * protocol set (4 scored, 1 control, 1 mixed, 1 exploratory): 3 scored
    completed with 33 548 verified, 0 wrong, 0 outside the edition; the
    unreadable PDF refused cleanly; control and mixed verified nothing; the
    exploratory document (G08 against a *different* Spanish edition) shows
    2 343 differing verified verses as raw observations, outside the gate;
  * forced-failure set (10 s ceiling): two timeouts (exit 124), one
    completed;
  * memory set (384 MB ceiling): a real allocation failure under the
    ceiling (exit 134, "memory allocation of 632 bytes failed") classed as
    a memory failure, nothing counted; the other document completed.

  Every verdict is "not a protocol run", as it must be: rehearsal
  inventory, and test ceilings for the failure sets. A second attempt was
  refused by the documents folder's lock, and a copy of the set without the
  lock by the committed run record.

  These three rehearsals were run with the tools of `f1d0d7f` and scored
  before their record was committed. After the follow-up review they were
  **rescored with the anchored scorer** against their committed run record
  (`deeca37`): verdicts, criteria, execution and every document's
  observations identical (`rehearsal/*/run/score-anchored.*`).
* **Fourth rehearsal, end to end with the final tools** (`rehearsal/protocol-2/`):
  the same seven disclosed documents, sealed in `8f9dffa`; the canonical
  inventory refused it before reading; with the rehearsal inventory it ran
  once and the runner committed its own record (`5509668`, three files
  only); scored with `--run-record=5509668`: 33 548 verified, 0 wrong, 0
  outside the edition, coverage 70.5 % of 47 610 printed verses in the
  three completed scored documents; results identical to the first
  rehearsal. On this real run the scorer refused another commit named as
  the record, a missing `--run-record`, and an altered report whose
  OUTPUTS.sha256 had been regenerated.

**First dry run (tools at `f50c541`, superseded, kept as recorded):** four
disclosed Set G documents under new keys and the README as a control;
results identical to the run without ceilings; the heaviest disclosed
reading (`spavbl-rlofodt`, 570 MB) identical under the 4 GB ceiling
(`data/dryrun/`).

## 6. What the independent reviews found, and what was done

Three reviewer agents, each separated from the code's author and from
each other's results:

| Review | Found | Done |
|---|---|---|
| Completeness auditor (`auditor-qualification.md`) | 33 ways a damaged document passed version 1 | Version 2, fail-closed: 78 of 83 probes as the reviewer expected; the 5 others named, with reasons |
| Security and resources (`reviews/security-review.md`) | 6 high, 6 medium, 4 low (and one suspected) | 12 fixed with tests, each shown failing without its fix; 1 an integration requirement; the rest documented |
| Canon and versification (`reviews/canon-review.md`) | Printed numbering of other traditions verifies under the canon's keys; one title misread | Numbering evidence per book; title fixed; the handover forbids joining editions by printed number |

## 7. Freeze readiness

| Condition | State |
|---|---|
| No unexplained wrong or nonexistent verified verse on the disclosed tests | **Met.** 9 explained by source evidence (v1 fixture defect); 0 on v2; 0 nonexistent |
| Memory bounded and measured | **EPUB met** (median 119 MB, highest 249 MB). **PDF not bounded in-process**: median 185 MB, highest 566 MB; integration must isolate PDF reading |
| Provenance and safe refusals preserved | **Met**, with the `francl` link-role and JHN 10 reason changes disclosed |
| No regression concealed by changed expectations | **Met**: no existing test expectation changed |
| Development corpus and Sets B–G rerun | **Met**, at the exact commit's source |
| Evaluation tools qualified | **Met for USFM-to-DOCX only**; no auditor exists for PDF or EPUB |
| Independent blind evaluation | **Not done.** Awaits the owner's decision |

## 8. Unresolved limitations

1. Missing text is invisible: a verse read faithfully from an incomplete
   document is verified. Verification is fidelity to the document read.
2. No independent evidence yet. All figures are on disclosed documents,
   almost all generated in this environment from eBible USFM.
3. Hebrew: right-to-left PDFs and Hebrew-letter numerals verify nothing
   (`known-weaknesses.md` §B). Safe, but no coverage.
4. PDF memory is not bounded in-process; no cancellation anywhere.
5. Printed numbering is kept. A book of another tradition verifies under
   its own numbers; the numbering evidence must be used before any
   cross-edition alignment.
6. EPUB text hidden by CSS or `hidden`/`aria-hidden`, and PDF text the
   page does not show, are read as text.
7. The rules are measured on 113 editions; each only withholds
   verification, so an unseen layout costs coverage, not correctness, as
   far as this evidence shows.
