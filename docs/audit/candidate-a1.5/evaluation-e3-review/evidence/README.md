# Evidence of the independent review of evaluation candidate A1.5-E3

Everything here was written and run by the independent reviewer in one
session, from `9640aea303c625c137c5cdbe1f4b8cc9738951fe`. No E3 test, vector,
rehearsal set or driver was reused as test material; the E3 tools, readers and
the pinned harness binary were run only as the system under test. The E2
reviewer's committed outputs were used only as data (the list of the 528
verses and their context windows). `pistio/collection-h` and every withheld
document, manifest, reference and result were not accessed. Nothing was
sealed or run on a withheld set. Every scratch repository, protocol revision
and modified tool lived outside the project under `/tmp/claude-0/` and is not
committed. Every synthetic text is invented ("review invented text", "invented
prose"); real verse text comes only from the bundled public-domain references
(engbsb, engwebp; SBL Greek New Testament © SBL and Logos, CC BY 4.0).

`SHA256SUMS` lists every file here (`sha256sum -c SHA256SUMS` from this folder).

## identity/

* `identity-checks.txt`: commits and parents, recogniser diff and trees,
  `recogniser-source.sha256` (166/166), `evaluation-e3.sha256` (59/59), E3
  and E2-review `SHA256SUMS`, the protocol's SHA-256, every `approved_tools`
  and `harness_build.sources_sha256` entry against `sha256sum`, the gate and
  floor as committed, history unchanged since `7772d0b`.

## build/

* `harness-builds.md`: the pinned harness rebuilt from a fresh target in a new
  container at the pinned path (`b4f51a62…`, identical) and at another path
  (`8cab9ddd…`, path-dependent). The E2 harness rebuilt at the pinned path
  (`800fb43c…`, identical) is recorded in `../checks/checks-summary.md`.

## probe/

A Cargo crate (`cargo build --release --offline` with the project's locked
`serde`, `serde_json`, `sha2`) that includes `src-tauri/tests/eval_support/usfm_text.rs`
**read-only by path** and prints its reading as JSON. It is how the Rust
reader was run on the reviewer's inputs.

## r1/ — the reference text

| File | What | Reproduce |
|---|---|---|
| `rev_usfm.py` | The reviewer's own USFM extractor, written from the USFM 3 specification (regular-expression tokenizer, open-span list, notes cut at their own end marker); records `trouble` instead of recovering | — |
| `compare_bundled.py`, `rust_dump.py` → `bundled-three-readers.json` | The 13 bundled references: reviewer's extractor, E3 Python reader and E3 Rust reader (probe), every verse | `python3 -I rust_dump.py <repo> <probe> /tmp/rust.json; python3 -I compare_bundled.py <repo> bundled-three-readers.json /tmp/rust.json` |
| `check_528.py` → `the-528.json` | The review's 528 verses against three independent expectations: source adjacency at each of 856 notes, the E2 reviewer's context windows, the reviewer's extractor; plus the Rust reader | `python3 -I check_528.py <repo> the-528.json /tmp/rust.json` |
| `make_vectors.py` → `review-vectors.json`; `run_vectors.py` → `review-vectors.results.json` | 99 hand-written USFM vectors (notes, spans, white space, milestones, attributes, numbering, malformed structure, Unicode, identity) with hand-written expectations; both readers judged | `python3 -I make_vectors.py review-vectors.json; python3 -I run_vectors.py <repo> <probe> review-vectors.json review-vectors.results.json` |
| `fuzz_review.py` → `fuzz-review-100k.json` | 100 000 random books from the reviewer's grammar of well-formed and malformed fragments; Rust vs Python; silent loss; the reviewer's extractor as an oracle on every input E3 reads cleanly | `python3 -I fuzz_review.py <repo> <probe> 100000 7 fuzz-review-100k.json` |

## lab/ — R2 and integrity, end to end with the real binary

`lab.py` builds each set in a scratch Git repository holding a byte-for-byte
copy of the tools, commits a **simulated** owner revision of the protocol
there (the set's composition, gate open; pin and approved tools as
committed unless a probe says otherwise), seals with `seal-set.py --write`,
runs `run-withheld-set.sh` once with the real binary and scores officially.
`scenarios_lib.py` holds the shared material.

| File | What |
|---|---|
| `scenarios.py` → `scenarios-e3.json` | R2 (vacuous references, zero verified, unsupported only, numbered-but-empty, declared book not printed, all scored refused whole, no recognisable Scripture, **one faithful verse against a whole Bible**, exploratory, wrong text) and integrity (gate shut, raw publisher archive, `reference-sources.tsv` tampering, a Unicode 16 code point) |
| `probes.py` → `probes-e3.json` | Options (unknown, repeated, valueless), outputs changed after the run, scorer under Python 3.12, a modified scorer committed, a modified scorer with a rewritten protocol (after and before the run), a modified reader, the gate and `AW_PROTOCOL`, another binary, a fake harness changing one verse digest, a Unicode 16 code point in a document |
| `regress.py` → `regress-e2.json`, `regress-e3.json` | The same five reviewer sets through E2's tools (`git archive 7772d0b`) and rebuilt E2 binary, and through E3 |
| `deutero.py` → `deutero.json` | A scored document with a deuterocanonical book, by how `books` is written (approved slots 1 and 4) |
| `scores/` | Four official score reports as the scorer writes them (two-verse PASS, zero-verified INCONCLUSIVE, wrong-text FAIL, fake harness not a protocol run) |

Reproduce: `python3 -I scenarios.py <repo> <binary> <work> scenarios-e3.json`;
`python3 -I probes.py <repo> <binary> <other-path binary> <work> probes-e3.json`;
`python3 -I regress.py <E2 tools dir> e2 <E2 binary> <work> regress-e2.json` (and `<repo> e3 <E3 binary>`);
`python3 -I deutero.py <repo> <binary> <work> deutero.json`.

## r2/

* `floor_unit.py` → `floor-unit.json`: the scorer's `evidence_floor()` with 21
  protocol variants (missing, zero, negative, string, float, boolean,
  unknown measure, NaN, infinities, each slot met and unmet) and the shared
  option parser with repeated flags.

## real/ — generated documents from real references (not authentic files)

* `make_docs.py`: the whole Berean Standard Bible and the SBL Greek New
  Testament as Markdown, from the editions' own words read by the
  reviewer's extractor, with USFM-only packages of the editions' members.
  (Both documents came out byte-identical to the E2 reviewer's: SHA-256
  `2153c588…` and `3acba7ce…`.)
* `assess_real.py` → `real-references-e3-review.json`: the pinned E3 harness
  run as the runner runs a document; the scorer's own `observe_report()`
  with the check's inventory and per-verse records; each verified mismatch
  attributed; the reviewer's extractor against every reference verse digest.
* `real-references-e2-binary.json`: the same documents through the rebuilt E2
  binary, mismatches attributed.
* `real_official.py` → `bsb-official.json`, `real-bsb-official.score.md`: the
  whole BSB (less 2 John, a disclosed USFM book) through the official path.

## checks/

* `checks-summary.md`: every project check run, with results, and the
  environment note.

## Not here, and why

* The generated documents, run folders and scratch repositories (large, and
  reproducible from the drivers above and the bundled references).
* No authentic published PDF, EPUB or DOCX was used: none is in the
  repository, and `ebible.org`, although reachable, is Collection H's only
  approved source, so nothing was downloaded from it.
* The 113-edition disclosed corpus is not available; nothing about it is
  estimated.
