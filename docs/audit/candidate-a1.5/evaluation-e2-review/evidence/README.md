# Evidence of the independent A1.5-E2 review

Everything here was produced by the reviewer in one session, from the
reviewed commit `40d5d5970953b0546ac77e827b6d7d9d8f7c08d8` (E2 source
`97502d2`). No file of the repository outside this folder was changed.
Collection H was not accessed. Every synthetic sentence is the reviewer's
own invention ("REVIEW invented text, not Scripture"). Real verse text
appears only as short context windows around a difference, from the
bundled eBible references: Berean Standard Bible, World English Bible,
Louis Segond 1910 (public domain) and the SBL Greek New Testament
(© Society of Biblical Literature and Logos Bible Software, CC BY 4.0).

`SHA256SUMS` lists every other file here (`sha256sum -c SHA256SUMS` from
this folder).

## identity/

`identity-checks.txt`: HEAD, commit and tree identities, the recogniser
diff, every checksum manifest of E2 and A1.5, history paths unchanged, and
the test and assertion counts of the historical tool tests before and after
E2.

## build/

`harness-builds.md`: environment, commands and SHA-256 of three release
builds of the harness: the reviewed source at the pinned path (identical to
the pin), the same source at another path (different, path embedded), and
the A1.5 harness source (for the old-against-new comparison only). The
`../dist` result is in `REPORT.md` §8.

## probe/

A separate Cargo crate (not part of the repository's build) that includes
`src-tauri/tests/eval_support/fidelity.rs` and `reference.rs` read-only by
`#[path]` and depends on the app crate by path, so the reviewer can drive
the harness's own rule modules: `probe fidelity` (JSONL in, class and kind
out), `probe inventory <reference> [BOOKS]` (the harness's inventory and
every entry), `probe usfm <file>` (the app's reader). Built with the
repository's `Cargo.lock` copied beside it:
`CARGO_TARGET_DIR=<dir> cargo build` from this folder, after editing the two
`#[path]` lines and the dependency path to the checkout's location.

## fidelity/

* `make_vectors.py`: writes `vectors.json`, 159 adversarial vectors with
  the reviewer's expectation (`faithful` or `wrong`).
* `run_both.py <probe binary>`: classes each vector with
  `scripts/evaluation/fidelity.py` and with the probe (Rust), compares them
  with each other and with the expectation; writes `results.json`.

## inventory/

* `real_inventories.py <probe binary>`: both inventories on the 13 bundled
  references (`resources/scripture/*/*_usfm.zip`), compared with the
  scorer's own `reference_inventory.compare()`; feature counts; writes
  `real-inventories.json`.
* `indep_usfm.py`: the reviewer's independent USFM verse-text extractor
  (USFM 3 specification; no code or regex shared with the app's reader or
  the check).
* `compare_text.py <probe binary>`: app-reader text (as the E2 inventory
  stores it) against `indep_usfm.py`, every verse of the 13 references;
  `text-crosscheck-counts.json` holds the counts.
* `note-space-differences.json`: the 528 differing verses, with E2's kind
  and a context window from each reader.

## real-documents/

* `make_real_docs.py <out>`: Markdown, DOCX and PDF (LibreOffice 24.2.7)
  documents for 8 book selections of 8 real references, each reference
  checked against the SHA-256 in its `SOURCE.md`; the reference given to the
  harness is the book's USFM member, byte for byte
  (`manifest-selections.tsv`, `documents-selections.json` record files,
  members and hashes).
* `make_full_docs.py <out>`: whole-edition Markdown for engbsb, grcsbl and
  kor1961, against the bundled archives byte for byte
  (`manifest-full.tsv`).
* `run_harness.sh <binary> <docs> <manifest> <out>`: runs each key in its
  own process exactly as `run-withheld-set.sh` does (structural pipeline,
  4 GB, 30 minutes). Run with the pinned E2 binary and with the rebuilt A1.5
  binary (`e2-selections-RUN.txt`, `a15-selections-RUN.txt`).
* `summarise_e2.py <docs> <out>`: re-checks each E2 report with the
  scorer's own `observe_report()` and the Python inventory;
  `e2-selections-summary.json`.
* `a15-vs-e2.json`, `full-a15-vs-e2.json`: the comparison tables of
  `REPORT.md` §7. `e2-verified-mismatches.json`: every E2 verified mismatch
  of these runs, with its cause (reference reader, or document reading) and
  context windows.

The generated documents themselves are not committed (several MB, and
reproducible with the scripts from the bundled references).

## protocol/

* `lab.py`: a scratch Git repository holding the E2 tools byte for byte;
  builds sets of invented text, seals them with `seal-set.py --write`,
  commits the seal, runs `run-withheld-set.sh` once, scores with
  `score-set.py`.
* `scenarios.py <pinned binary> <other-path binary> <work>`: P1 (protocol
  as committed), overrides, P2 (a simulated owner revision committed in the
  scratch repository only), extra chapter/book/verse, every unsupported
  class, the shared-reader cases, the file-name case, another binary, a
  hand-made incomplete run record, a later change to a run record, seal
  refusals, and a committed modified scorer. Results: `results-scenarios.json`.
* `scenarios2.py <pinned binary> <work>`: the vacuous set, an empty
  reference zip, and four fake harnesses (the pinned binary with its report
  edited) scored officially and diagnostically. Results:
  `results-scenarios2.json`.
* `scores/`: the score JSON of each key run (`<set>.<mode>.json`). Two
  scores are only in the results files: the P2 clean official pass and the
  first, unmodified official score of `p2-extra` (their files were
  replaced by later scorings of the same set; both summaries are in
  `results-scenarios.json`). `p2-extra.official-by-modified-committed-scorer.json`
  is the score made by the deliberately modified scorer.
* `scratch-repo-log-*.txt`: the scratch repositories' commits.

## checks/

`checks-summary.md`: the results of every check of `REPORT.md` §8.
