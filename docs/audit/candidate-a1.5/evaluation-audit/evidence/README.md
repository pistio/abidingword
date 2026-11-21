# Evidence for the independent evaluation audit

Everything here is invented for the audit, or produced from invented input
by the frozen source and the pinned evaluation scripts. Nothing comes from
Collection H or any other withheld collection. None of it is a seal, a
manifest or a run record of this repository. The folders named `seal/`
below were sealed and run only in a scratch repository outside this one.

`SHA256SUMS` lists every file in this folder.

## compare-probe/: the frozen comparison code on synthetic pairs

| File | What it is |
|---|---|
| `extract-lines-45-105.rs` | `squash`, `dense`, `letters`, `difference_kind`, copied byte for byte from `08c6faac:src-tauri/tests/real_document_editions.rs` (blob `a9f94b13…`), lines 45–105. SHA-256 `d527532cad3212657f5834b10d9c48f308094ce1700a191e6ae972aac796c794` |
| `extract-lines-712-720.rs` | the class block (exact / normalised / different), lines 712–720. SHA-256 `16f4387037874d9127ee80a4b42df544a51df52cc43cb192d5ce21d2c794c921` |
| `src/main.rs`, `Cargo.toml` | the probe: the two extracts pasted unchanged between a header and the test cases; `unicode-normalization = "=0.1.25"` as in `src-tauri/Cargo.lock` |
| `results.jsonl` | one line per pair: class, kind, the harness's counter, the scorer's outcome; then `char::is_alphanumeric` for 15 combining and joining characters |

Reproduce: `git show 08c6faac:src-tauri/tests/real_document_editions.rs | sed -n 45,105p`
must equal `extract-lines-45-105.rs` (and `sed -n 712,720p` the other);
then `cargo run` in a copy of `compare-probe/`.

## synthetic/: three sets through the pinned seal, runner and scorer

`make_sets.py <scratch repo> <documents root> [a,b,c]` writes, for each set,
a documents folder (a reference USFM zip and a Markdown document in the
layout of `scripts/build-derived-editions.py`) and a seal folder (14-column
manifest, checksums, provenance, allowed lists, ledger, access records). In
the scratch repository the steps were:

1. Commit copies of `check-set.py`, `seal-set.py`, `run-withheld-set.sh`,
   `score-set.py`, `docs/evaluation/disclosed-documents.sha256` and
   `docs/corpus/*.tsv`. Their SHA-256s are those `FREEZE.md` §5a pins.
2. `seal-set.py <seal> <documents> --write`; commit the seal folder alone.
3. `run-withheld-set.sh <seal> <seal commit> <documents> <harness> <its sha256> <out>`.
   The runner commits its own record.
4. `score-set.py <out> <seal> --run-record=<record commit> --json=… --md=…`,
   once as the protocol command and once with `--frozen-binary=<local build>`
   (and, for set C, `--inventory=…`).

The harness was `real_document_editions`, a release build of the frozen
source (`cargo test --release --locked --test real_document_editions --no-run`
in `src-tauri/`, rustc 1.97.0 2d8144b78), with SHA-256
`e396328c20fef35ee12f62b880e554f676a93006ca5bc8054dc82d433f21ee43`.
`scratch-repo-log.txt` lists the scratch repository's commits.

| Set | Contents | Result |
|---|---|---|
| `zz-a/` | English Ruth, Spanish Philemon and decomposed Greek 3 John, each with materially different wording in some verses (word boundaries, case, apostrophes, `?`, comma, brackets, the owner's `\add` case, decomposed marks); a control; a mixed document | `score-zz-a-default.md`: *not a protocol run* (local binary). `score-zz-a.md` with `--frozen-binary`: **pass**, 125 printed, 125 verified, 0 wrong, 10 `verified_typography` |
| `zz-b/` | 14 probes: verse zero three ways (`exo0a/b/c`); Malachi with a 3-chapter reference and a 4-chapter document (`mal`); Joel in four chapters (`jol`); Jude with a bridge, an empty verse, placeholders, an absent number and `\va`/`\vp` (`jud`), and the same with text printed for the empty and absent verses (`jud2`); Obadiah + Tobit (`tob`); a file named `probe_gen.md` (`gen`); the `\add` case with extra words to reveal the reference reading (`add`); genuinely wrong words (`wrong`); a composed accent lost (`accent`); Joel with a `VerseMap.xml` in the reference (`map`); an empty document (`empty`) | `score-zz-b.md`: **fail**, as designed (3 wrong, 24 outside the edition, 1 refused whole). Per-probe figures in `REPORT.md` §2, §4 and §5 |
| `zz-c/` | Obadiah scored; a control that holds Obadiah; a mixed document of Ruth 1–2 with `RUT 1:1-21` and `RUT 1:22` allowed. Run with a rehearsal-style inventory (`inventory-rehearsal.txt`) | `score-zz-c-noinv.md`: *not a protocol run*. `score-zz-c.md` with `--inventory`: `canonical_inventory: yes`, **fail** (control 21 verified, mixed 23 outside) |
| `control-probe/` | The `mal`, `jud`, `exo0a` and `tob` documents run as controls (no reference), harness called directly | MAL 4:1–6 verified; JUD 8 and 19 not verified; EXO 16:1–36; Obadiah only |

Each `run-outputs/` folder is the runner's output folder exactly as it was;
its files verify against its own `OUTPUTS.sha256`. The `.run-started` lock
the runner wrote into each documents folder was not copied. The
`CHECK.json` files name the scratch folders' paths.
