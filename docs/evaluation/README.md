# Withheld evaluation sets: shared files

| File | What it is |
|---|---|
| `builder-brief.md` | The neutral brief for whoever builds a withheld set. Nothing in it describes the recogniser |
| `disclosed-editions.tsv` | Every edition used by the 113 disclosed editions (development corpus, Sets B–G, reconstructions): identity, language, source and licence as recorded. No layout, converter, rule or result |
| `transcript-audit.md` | How the builder's and reviewer's transcripts are audited (option A: procedural independence), and what invalidates a session. For the auditor, not the builder |
| `disclosed-documents.sha256` | The canonical inventory: 7 461 SHA-256 hashes of every disclosed document, reference, build intermediate and USFM book in them, and every hash the manifests record. No names |
| `protocol-e2.json` | (E2, historical) The E2 protocol. No tool reads it any longer |
| `protocol-e3.json` | (E3) The evaluation protocol the scorer applies: rule versions, the harness binary, ceilings, the canonical inventory, the required composition (unset until the owner sets it), the evidence floor, both Unicode versions, the approved tools by SHA-256, and the gate `withheld_execution_permitted` (false until the owner approves). Official scoring reads only this committed file |
| `usfm-text-rules.md` | (E3) Rules `aw-usfm-text-1`: how a reference's exact verse text is read from USFM, by two readers written apart (`src-tauri/tests/eval_support/usfm_text.rs`, `scripts/evaluation/usfm_text.py`), with the marker table both are tested against |
| `usfm-books.tsv` | (E2) The USFM book identifiers the neutral reference inventory knows: the 66 with their chapter counts, and every other identifier (counted as unsupported, never dropped) |

The tools are in `scripts/evaluation/`: `check-set.py` (the checks, before
running), `seal-set.py` (the same checks before the seal, and the seal),
`run-withheld-set.sh` (the one run; it commits the run record),
`score-set.py` (the score and assessment, anchored to the committed run
record with `--run-record`), `disclosed-inventory.py` (rebuilds the two lists
above). Their A1.5 checksums are pinned in
`docs/audit/candidate-a1.5/FREEZE.md` §5a, and the protocol is
`docs/audit/candidate-a1.5/set-h-decision-package.md`.

**Evaluation candidate A1.5-E2** changes these tools (and adds
`fidelity.py` and `reference_inventory.py`, the scorer's and the check's own
rules), so the A1.5 pins above describe the historical tools only. The new
identities, rules and evidence are in
`docs/audit/candidate-a1.5/evaluation-e2/EVALUATION-FREEZE.md`.

**Evaluation candidate A1.5-E3** corrects E2 after its independent review
(R1: reference text read by the evaluation's own USFM readers, compared verse
by verse; R2: a run that verifies nothing is never a pass; and the bounded
findings). It adds `usfm_text.py` and changes the other tools again; its
identities, rules, evidence and the owner decisions it prepares are in
`docs/audit/candidate-a1.5/evaluation-e3/`. E3 is a candidate for
independent verification: no set has been sealed or run under it, and its
protocol permits no withheld run.

## For the builder

Hand over these files and nothing else from this repository:

* `docs/evaluation/builder-brief.md`
* `docs/evaluation/disclosed-editions.tsv`
* `docs/evaluation/disclosed-documents.sha256`
* `scripts/evaluation/check-set.py`
* `scripts/evaluation/seal-set.py`
* `scripts/evaluation/reference_inventory.py`, `scripts/evaluation/usfm_text.py`
  and `docs/evaluation/usfm-books.tsv` (check-set.py reads references with
  them; a flat copy beside check-set.py is found)
* `docs/evaluation/protocol-e3.json` (E3: the required composition the seal
  is checked against; found beside check-set.py)
* `scripts/corpus/audit-derived-completeness.py` (only if Word documents
  are made from USFM)

## Rebuilding the inventory

```
python3 scripts/evaluation/disclosed-inventory.py docs/evaluation <every disclosed corpus folder>
```

The inventory was built from the corpus folders of this environment
(`aw-corpus`, `aw-derived-v2`, `aw-recon`, `aw-recon-bc`, `aw-set-e`,
`aw-set-f`, `aw-set-g`, `aw-usfm-bc`) and the hashes the manifests record.
The original files of Sets B, C and D are not in this environment; their
documents are covered by the hashes their manifests record and by the
edition list, and their reconstructions by the files themselves.
