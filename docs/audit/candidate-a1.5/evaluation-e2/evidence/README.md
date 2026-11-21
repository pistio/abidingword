# Evidence for evaluation candidate A1.5-E2

Everything here is SYNTHETIC (invented text, labelled as such in every
reference's `\id` line and every manifest row), or produced from it by the
frozen A1.5 tools and harness source ("before") or the E2 tools and harness
("after"). Nothing comes from Collection H, from any withheld collection, or
from a published translation. None of it is a seal, manifest or run record of
this repository: every `seal/` and `run/` folder below was made in a scratch
Git repository outside it. `SHA256SUMS` lists every file here.

## golden-vectors/: the comparison rule itself, before and after

| File | What it is |
|---|---|
| `extract-lines-45-105.rs`, `extract-lines-712-720.rs` | The A1.5 harness's `squash`, `dense`, `letters`, `difference_kind` and its class block, byte for byte from `08c6faac:src-tauri/tests/real_document_editions.rs` (the same SHA-256 as the independent audit's extracts: `d527532c…`, `16f43870…`) |
| `src/main.rs`, `Cargo.toml` | The before-probe: the two extracts pasted unchanged, run on every vector of `tests/evaluation/fidelity-vectors.json`, with the A1.5 scorer's rule (wrong = verified class 2 whose kind is not typography or unicode) |
| `results-a15.jsonl` | Its output: class, kind and scorer outcome per vector |
| `comparison.md` | Each vector under A1.5 and under E2 (`scripts/evaluation/fidelity.py`; the Rust implementation gives the same, `src-tauri/tests/evaluation_rules.rs`) |

Result: of the 58 vectors that are wrong text under E2, **36 were counted
correct by A1.5** (word joining and splitting, every space removed, case,
apostrophes, a moved comma, a question mark, brackets, dashes, quotation
marks, decomposed accents and breathings, cantillation, soft hyphen,
zero-width characters); 22 were caught by both. The 10 faithful vectors
(exact, canonical, white-space runs) are correct under both.

Reproduce: copy `golden-vectors/` and run `cargo run --offline` (it needs
`unicode-normalization 0.1.25`, as in `src-tauri/Cargo.lock`).

## synthetic/: seven sets through the real checker, runner, harness and scorer

`tests/evaluation/synthetic/make_e2_sets.py` writes the sets;
`tests/evaluation/synthetic/run_e2_sets.py old|new <harness> <folder>` seals
each one (seal-set.py --write), commits the seal alone, runs it once with the
runner (which commits its own run record) and scores it against that record.

* `before/`: `run_e2_sets.py old` with a release build of the frozen harness
  source (`cargo test --release --locked --test real_document_editions --no-run`
  at `767e226`, whose `src-tauri/` is `08c6faa`'s; SHA-256
  `e396328c20fef35ee12f62b880e554f676a93006ca5bc8054dc82d433f21ee43`, the
  same as the independent audit's local build) and the A1.5 tools exactly as
  frozen (`git show 767e226:<path>`), scored with `--frozen-binary`, which
  A1.5 accepted silently.
* `after/`: `run_e2_sets.py new` with the E2 harness
  (`800fb43c9d09e85e7c7475702ff3332ebc45a3ce44a0204220f3d64756454473`) and
  the E2 tools of this commit, scored with `--diagnostic` and a per-set
  rehearsal protocol (`rehearsal-protocol-<set>.json`: the committed
  protocol with that binary and the set's declared composition). Every
  official verdict is therefore *not a protocol run*; the diagnostic verdict
  is what the criteria give. `e2-fid/score-official.*` is the same run
  scored officially: *not a protocol run* (run under a rehearsal protocol; no
  required composition), and `score-official-override-refused.txt` the
  refusal of `--frozen-binary` without `--diagnostic`.

Each set folder holds `expected.json` (what each probe is for), `seal.log`,
`run.log`, `score.json`, `score.md` and `run-outputs/` (the runner's output
folder as it was; it verifies against its own `OUTPUTS.sha256`), or
`seal-refused.txt`. `sealed-sets/` holds each seal folder and committed run
record; `documents/` the documents and references; `scratch-repo-log.txt`
the scratch repository's commits. CHECK.json and score.json name the scratch
folders' paths.

`comparison.md` sets the two side by side, per set and per document.

| Set | Probes | A1.5 | E2 |
|---|---|---|---|
| `e2-fid` | F1: English, Spanish, Greek and Hebrew wording that differs only in ways A1.5 accepted; a decomposed (canonically equal) Greek document | **pass, 171 of 171 correct** | fail: 153 faithful, **18 mismatches** listed by kind (punctuation 8, word boundary 4, marks 4, case 2); the decomposed document all faithful |
| `e2-mat` | material changes (forty/four, a negation, a composed accent lost) | fail, 3 wrong | fail, 3 mismatches |
| `e2-ref` | B1/B2: verse 0 three ways, Joel in four chapters, Obadiah + Tobit, 3 John with a second chapter, `12a`, `201` and verse 7 twice | pass; printed 195, nothing reported | fail: printed 194 + **54 unsupported** (verse_zero 3, chapter_beyond_canon 25, book_outside_canon 22, verse_label_unusual 2, verse_duplicate 2); 3JN 1:7 verified but **unvalidatable** |
| `e2-abs` | B3: Malachi 4 verified against a 3-chapter reference; Jude 26; Ruth against an Obadiah reference | fail on 23; **MAL 4:1–6 counted nowhere** | fail on **29**: MAL 4:1–6 `chapter_not_in_book`, JUD 1:26, RUT 1:1–22 |
| `e2-name` | C3: Ruth in a file named `probe_gen.md`, books `all` | fail: printed 0, **22 false "outside the edition"** | pass: 22 faithful |
| `e2-map` | C4: Joel with a verse map in the reference archive | pass, printed **73** (renumbered) | **seal refused** (verse map) |
| `e2-utf8` | C5: a reference with one Latin-1 byte | fail on 1 (read with a replacement character, silently) | **seal refused** (not UTF-8) |

## direct-probe/: the harness itself on the two references the seal refuses

The `e2-map` and `e2-utf8` documents run directly (no seal, no runner) by
both harness builds: `manifest.tsv`, `docs/`, `out/frozen-source/` and
`out/e2/`. The A1.5 harness renumbers Joel by the verse map (printed 73) and
reads the Latin-1 byte as U+FFFD, reporting neither. The E2 harness never
reads the verse map (printed 52 + 21 unsupported, the importer comparison
showing the 21 renumbered verses), and both reports carry a reference
`problems` entry, which makes the scorer call the run *not a protocol run*.
Defence in depth behind the seal's refusal.

## python-tests/: the E2 tests before and after

`tests/evaluation/test_e2_rules.py` (28 tests) run against the A1.5 tools
(`AW_EVAL_TOOLS=<a copy of scripts/evaluation at 767e226>`): **27 fail**
(24 failures, 3 errors). The one that passes checks only the new rule module
(`fidelity.py`) on the golden vectors; its before-evidence is
`golden-vectors/`. On the E2 tools: 28 pass.
