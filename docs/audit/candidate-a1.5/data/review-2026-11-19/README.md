# Follow-up review of `4dd9f8e` (19 November 2026): receipts

The owner's independent review package (its own `SHA256SUMS` kept here as
`review-package-SHA256SUMS`; its scripts are not copied) reproduced three
ways to turn a failing synthetic run into a pass, and three input paths
that escaped the documents folder. All synthetic: the committed fake
harness, no recogniser, no Scripture, nothing withheld.

| File | What |
|---|---|
| `before-4dd9f8e-scorer-probe.json` | The review's `probe_scorer_integrity.py` rerun here at `4dd9f8e`: all three alterations turn **fail into pass** (reproduced) |
| `before-4dd9f8e-path-probe.json` | The review's `probe_path_containment.py` at `4dd9f8e`: all three escapes **accepted**, the fake harness ran |
| `after-path-probe.json` | The same unchanged probe on the fixed tools: exit 0, all three **refused** before the harness ran |
| `after-original-scorer-probe-stderr.txt` | The unchanged scorer probe on the fixed tools stops at its first alteration: the scorer refuses it (`allowed/M1.txt differs from the seal commit`), so the probe's own assertion that scoring succeeds fails. An invocation stop is not counted as evidence; hence: |
| `probe_scorer_integrity_anchored.py`, `after-scorer-probe-anchored.json` | The review's three alterations, step for step, through the anchored interface (the runner commits the run record; the scorer is given that commit). Before each alteration the run scores `fail`; after it the scorer **refuses** (exit 2): `allowed/M1.txt differs from the seal commit` (and again after re-sealing), `OUTPUTS.sha256 in the run folder differs from the committed run record`, `RUN.txt in the run folder differs from the committed run record`. Exit 0 |

The same cases, and more, are regression tests in
`tests/evaluation/test_withheld_set_tools.py` (classes `ScoringAnchors`
and `PathContainment`).
