# Evidence of evaluation candidate A1.5-E3

Everything here was produced in one session by the E3 author, from the
starting commit `7772d0b` (the E2 review) and the E3 working tree. Collection H
was not accessed. Every synthetic text is invented ("SYNTHETIC", "invented
text, not Scripture"). Real verse text appears only as short context windows
from the 13 references bundled in `resources/scripture/` (public domain,
apart from the SBL Greek New Testament: © Society of Biblical Literature and
Logos Bible Software, CC BY 4.0; and the Korean Revised Version 1961, whose
economic copyright has expired). Paths of the scratch folder are written
`<work>`. `SHA256SUMS` lists every other file here (`sha256sum -c SHA256SUMS`
from this folder).

The before (E2) and after (E3) runs use the same material and the same
drivers. "E2" is a byte-for-byte copy of the tools at `7772d0b`
(`git archive 7772d0b scripts/evaluation docs/evaluation docs/corpus`) and
the E2 harness binary rebuilt here, identical to its pin `800fb43c…`
(`build/harness-builds.md`). "E3" is this candidate's tools and its binary
`b4f51a62…`.

## before/ and after/: end-to-end rehearsals (synthetic) and real references

* `rehearsal-e2.json`, `rehearsal-e3.json`: `tests/evaluation/synthetic/rehearse.py`
  (in the repository; its docstring says how it works). Seven sets of
  invented documents, each built in a scratch Git repository holding the
  tools, sealed with `seal-set.py --write`, the seal committed alone, run
  once with `run-withheld-set.sh` and the real harness binary, and scored
  with `score-set.py` officially under a **simulated** owner revision of the
  protocol (scratch repository only: the set's composition, the binary,
  E3's gate and approved tools). Plus two probes: a misspelt option given
  to an official score (`unknown_option`), and a scorer modified to pass
  everything and committed (`modified_committed_scorer`). Reproduce:

  ```
  python3 tests/evaluation/synthetic/rehearse.py --tools=<E2 copy> --version=e2 --binary=<E2 binary> --work=<w> --out=rehearsal-e2.json
  python3 tests/evaluation/synthetic/rehearse.py --tools=. --version=e3 --binary=<E3 binary> --work=<w> --out=rehearsal-e3.json
  ```

* `real-references-e2.json`, `real-references-e3.json`:
  `real-references/run_real.py <binary> <work> <out> --full [--e3]`.
  Markdown documents made from the editions' own words (Genesis BSB,
  Matthew SBLGNT, Daniel WEBP, Ephesians LSG, Ruth KOR 1961 and Luther 1912,
  3 John Byzantine; and the whole BSB and the whole SBLGNT New Testament),
  run through each harness binary as the runner runs a document. The
  document's words come from a **third** reader, the E2 reviewer's
  extractor, so that agreement with E3's readers is not circular. Each
  verified mismatch is attributed to the reference reader or to the
  document's reading. With `--e3`, the scorer's own `observe_report()` also
  compares the harness's per-verse text digests with the Python check's.

## readers/: the two E3 readers

* `crosscheck_bundled.py <rust dump> <out>`: the 13 bundled references read
  by the Rust reader (a dump from the `evaluation_references` test with
  `AW_EVAL_TEXT_DUMP`), the Python reader and the E2 reviewer's extractor;
  and the review's 528 verses as E3 reads them. Result:
  `bundled-crosscheck.json`.
* `fuzz_readers.py <evaluation_references test binary> <out> [count]`: both
  readers on 50 000 mutated USFM texts (and their 137 seeds); every input
  either refused by a problem or read with every `\v` as a verse, by a third
  count; and the two readers' verses, lines and problem codes compared.
  Result: `fuzz-50k.json`.

## build/

`harness-builds.md`: the environment, commands, source hashes and SHA-256 of
the E2 reproduction and the E3 builds.

## checks/

`checks-summary.md`: every check of `../EVALUATION-CANDIDATE.md` §10.

## Not here, and why

* No authentic published document: none is available in this environment,
  and `ebible.org` is not reachable. No new recognition statistic is given.
* The generated documents and the run folders are not committed (they are
  reproducible with the drivers above from the bundled references and the
  invented text).
