# Set H: decision package for the owner

**Nothing in Set H has been created, selected, downloaded, inspected,
sealed or run.** This page asks for one decision: approve (or amend) the
protocol below, then separately authorise building and running Set H.
It merges the protocol of PR #51 (`docs/audit/candidate-a1.3/set-h-protocol.md`)
with the review amendments of PR #52 (`set-h-protocol-independent-review.md`)
and fixes what they left open: criteria, ceilings and the exact run.

## 1. The candidate

The frozen candidate SHA, its source checksums, toolchain and the
harness binary's SHA-256 are in `FREEZE.md` (this folder). The run uses
that binary and refuses any other (§6).

A pass is permission for research use and a later integration review. It
is not a release, not a claim that recognised Scripture is accurate, and
not approval of migration 0028.

## 2. Roles

| Role | Who (proposal) | Does | Must not |
|---|---|---|---|
| **Owner** | the owner | approves this page; authorises the build and the single run | — |
| **Document builder** | the owner, or a person the owner names, for authentic documents; an isolated agent session for generated ones | chooses documents and reference texts; records licence and provenance; seals | see the recogniser's code, tests, rules, reports or any earlier set's results; run the recogniser |
| **Ground-truth reviewer** | a second, separate session or person | checks each reference against its document before the seal (edition, revision, canon, versification, samples of text and boundaries); signs a pre-seal exception ledger | change anything after the seal |
| **Runner** | a Claude Code session | runs `run-withheld-set.sh` once; archives everything | edit code, harness, manifest, documents or references; rerun |
| **Assessor** | a session other than the runner's | scores with `score-set.py`; writes the case ledger | change the raw score |

Each role records which model or person filled it, and every path it
read. A model's name is not evidence of independence; the access records
are.

**Isolation that this environment can give.** An agent builder can be
denied the repository and the corpora by instruction and records, as for
Sets E–G. It cannot be proven unable to read them. Authentic documents
chosen by a person are the only fully independent part. That is why §3
asks for them.

## 3. Composition

| Dimension | Minimum |
|---|---|
| Documents | 24 |
| Authentic published files (not made from eBible USFM by a script) | 12, if the owner can supply them; otherwise as many as available, stated |
| Formats | EPUB 10, PDF 8, DOCX 4 |
| Languages | English, German, French, Spanish, Portuguese, Korean: 2 each; Greek, Hebrew, Latin: 1 each |
| Canon | 2 with deuterocanonical books; 2 New Testament only; 1 single book |
| No-Scripture controls | 3 (zero verses expected): e.g. a novel, a cookbook, a software manual |
| Mixed-content challenges | 2 (a commentary or lectionary quoting Scripture): the builder lists which passages each quotes; any other verified reference is a failure |
| Editions | none of the 113 disclosed editions; overlap of publisher or converter reported, not hidden |

The builder chooses layouts without advice. Scanned PDFs are left out
(not supported, `docs/handover-scripture-recognition-codex.md` §7).

## 4. Reference texts

* Each scored document has a reference of the **same edition, revision,
  canon and versification**. A similar translation is not a reference.
  Where no such reference exists, the document is declared *exploratory*
  before the seal: it is run and reported, but not scored for wrong text.
* **Generated documents:** `scripts/corpus/audit-derived-completeness.py`
  (version 2, qualified in `auditor-qualification.md`) must pass before the
  seal, for DOCX made from USFM. For PDF and EPUB derived from USFM there is
  **no** qualified auditor. The builder samples at least 20 verses per
  document by hand, records them, and lists the document's unaudited part.
* **Authentic documents:** the reviewer samples at least 20 verses per
  document against the reference and records every known divergence
  (missing or added verses, revisions, numbering) before the seal.

## 5. Criteria, fixed now

**Research gate (all must hold):**

1. **0** verified verses with wrong text against a matched reference;
2. **0** verified references the edition does not have;
3. **0** verified verses in no-Scripture controls, and **0** verified
   references outside the listed passages in mixed-content documents;
4. no document crashes, times out or exceeds the memory ceiling (§6)
   without refusing cleanly.

A divergence between reference and document found after the run counts
as a failure in the raw score. It may be explained in a separate
adjudication ledger, with source evidence, beside the raw figure, never
instead of it.

**Coverage** (verified correct ÷ printed) is reported per document and
group, with no pass mark. Proposed, for information only: below 50 % on a
format or a language is a finding to explain.

**Failing** the gate means the candidate is not ready. Set H is then
disclosed; corrections need a new candidate and a new withheld set.

## 6. Resource ceilings and the run

| Ceiling | Value |
|---|---|
| Time per document | 30 minutes (`timeout 1800`) |
| Address space per document | 4 GB (`ulimit -v`) |
| Peak reading memory to report | as measured; above 1 GB is a finding |

The exact run, by the runner, once:

```
scripts/evaluation/run-withheld-set.sh <sealed set> <harness binary> <binary sha256 from FREEZE.md> <new output folder>
python3 scripts/evaluation/score-set.py <output folder> <sealed set>/manifest.tsv --json=score.json --md=score.md
```

The runner refuses to start when the output folder exists, the binary is
not the frozen one, the manifest is not the sealed one, or any file
differs from its checksum. Each document runs in its own process, so one
failure is that document's result. Outputs are checksummed at the end.

## 7. Seal contents

Committed in one commit that changes nothing else, before the run:

* `manifest.tsv` (harness columns, plus a 13th `kind`: authentic or
  generated), with each file's SHA-256, licence and provenance;
* `checksums.sha256` for every document and reference;
* the completeness-audit outputs and sampled-verse records;
* the reviewer's pre-seal exception ledger;
* role and access records;
* `SEAL.txt` with the manifest's SHA-256 and the UTC time.

Documents stay outside Git. Copyrighted text never goes into the
repository or into public comments.

## 8. Before authorising

- [ ] Approve or amend §2-§7.
- [ ] Say whether authentic documents can be supplied, and by whom.
- [ ] Name the builder and reviewer arrangements.
- [ ] Then, separately: authorise building, sealing and the single run.
