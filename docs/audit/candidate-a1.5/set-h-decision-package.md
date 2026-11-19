# Set H: decision package for the owner

**Nothing in Set H has been created, selected, downloaded, inspected,
sealed or run.** This page asks for one decision: approve (or amend) the
protocol below, then separately authorise building and running Set H.
It merges the protocol of PR #51 (`docs/audit/candidate-a1.3/set-h-protocol.md`)
with the review amendments of PR #52 (`set-h-protocol-independent-review.md`)
and Codex's review of PR #53 (five points: document classes, execution
assessment, independence and one-run safeguards, composition and authentic
evidence, an independent builder arrangement). The recogniser is unchanged
at `08c6faa`; only the evaluation tools and this page changed.

## 1. The candidate

The frozen candidate SHA, its source checksums, toolchain and the
harness binary's SHA-256 are in `FREEZE.md` (this folder); the evaluation
tools' checksums are in `FREEZE.md` §5a. The run uses that binary and
refuses any other (§7).

A pass is permission for research use and a later integration review. It
is not a release, not a claim that recognised Scripture is accurate, and
not approval of migration 0028.

## 2. Roles, and how the documents are found

| Role | Who (proposal) | Does | Must not |
|---|---|---|---|
| **Owner** | the owner | approves this page, the source list and the final list of documents (titles, publishers, licences: metadata, not files); authorises the build and the single run | — |
| **Document builder** | a new cloud session in an environment of its own, or a person the owner names | finds and downloads lawfully available published files; makes references and any generated documents; records provenance; runs `seal-set.py` until it passes; seals with `seal-set.py --write` once the reviewer has signed and the owner has approved | receive or read the recogniser's code, rules, tests, failure reports or any earlier set's results; open this repository, its issues or pull requests; run any Bible-recognition software |
| **Ground-truth reviewer** | a second new session, or a person | checks each reference against its document before the seal (edition, revision, canon, versification, at least 20 sampled verses and the boundaries); checks each edition against the exclusion list; signs `review-ledger.md` | the same as the builder; change anything after the seal |
| **Runner** | a Claude Code session with this repository | runs `run-withheld-set.sh` once; commits the run record at once | edit code, harness, seal or documents; rerun |
| **Assessor** | a session other than the runner's | scores with `score-set.py`; writes the case ledger | change the raw score |

**The owner does not need to collect the files.** Proposed arrangement:

1. The owner creates a **private repository for the set** (for example
   `pistio/abidingword-set-h`) and two cloud environments, one for the
   builder and one for the reviewer, **with no repository of this project
   attached** and network access limited to the approved sources.
2. The builder session receives only the files listed in
   `docs/evaluation/README.md` §"For the builder": the neutral brief
   (`docs/evaluation/builder-brief.md`), the list of disclosed editions
   (`disclosed-editions.tsv`, identity only), the hash inventory
   (`disclosed-documents.sha256`), `check-set.py` and `seal-set.py`, and,
   for DOCX made from USFM, the completeness auditor. None of these describes the recogniser,
   its rules or how it fared.
3. The builder writes documents, references and the seal folder to the set
   repository; the reviewer reads them there and signs the ledger; the owner
   approves the list; the seal folder is then committed to this repository
   in one commit that changes nothing else (§8).
4. The runner reads the documents from the set repository into a folder
   outside Git and runs once.

**Copyright and cloud sessions.** The mandate forbids unapproved cloud AI
transmission of copyrighted text. The builder, reviewer and runner are
cloud sessions. Proposed: **public-domain texts only** by default; texts
under an open licence (Creative Commons and similar) only if the owner
approves that here; no all-rights-reserved text in any cloud session.

**Isolation this arrangement can give, and what it cannot.** A builder
session with no repository of this project attached has no copy of the
recogniser. Whether its credentials could still read this repository
depends on the GitHub app's installation scope, which the owner controls;
it cannot be proven from inside the session. Independence therefore rests
on (a) what is not attached, (b) the neutral brief, and (c) the access
records each role writes (every URL visited, every file read). Published
files that existed before the recogniser are independent of it by
construction; the *choice* among them is the builder's, which is why the
builder must not see the recogniser. A model's name is not evidence of
independence; the records are.

## 3. Composition: 24 documents, overlaps shown

The minima of PR #51 and #52 kept, and shown to fit in 24. This is one
allocation that meets every minimum at once; the builder may choose
another that also meets them, and chooses titles and layouts without
advice.

| # | Class | Format | Language | Canon requirement met | Preferred kind |
|---:|---|---|---|---|---|
| 1 | scored | EPUB | English | with deuterocanonical books (1 of 2) | authentic |
| 2 | scored | PDF | English | New Testament only (1 of 2) | authentic |
| 3 | scored | EPUB | German | — | authentic |
| 4 | scored | PDF | German | with deuterocanonical books (2 of 2) | authentic |
| 5 | scored | EPUB | French | — | authentic |
| 6 | scored | DOCX | French | — | generated |
| 7 | scored | EPUB | Spanish | — | authentic |
| 8 | scored | PDF | Spanish | — | authentic |
| 9 | scored | EPUB | Portuguese | — | authentic |
| 10 | scored | DOCX | Portuguese | — | generated |
| 11 | scored | PDF | Korean | — | either |
| 12 | scored | EPUB | Korean | New Testament only (2 of 2) | either |
| 13 | scored | EPUB | Greek | — | either |
| 14 | scored | PDF | Hebrew | a single book | either |
| 15 | scored | DOCX | Latin | — | generated |
| 16 | scored | PDF | any | — | authentic |
| 17 | scored | EPUB | any | — | authentic |
| 18 | scored | PDF | any | — | either |
| 19 | scored | DOCX | any | — | either |
| 20 | control | EPUB | any | no Scripture (e.g. a novel) | authentic |
| 21 | control | PDF | any | no Scripture (e.g. a cookbook) | authentic |
| 22 | control | DOCX | any | no Scripture (e.g. a software manual) | authentic |
| 23 | mixed | EPUB | any | a commentary quoting Scripture | authentic |
| 24 | mixed | PDF | any | a lectionary quoting Scripture | authentic |

Totals: EPUB 10, PDF 9, DOCX 5 (minima 10, 8, 4); English, German, French,
Spanish, Portuguese and Korean twice each, Greek, Hebrew and Latin once
(slots 1–15); 2 with deuterocanonical books, 2 New Testament only, 1 single
book; 3 controls; 2 mixed. Slots 16–19 add scored evidence where it can be
found.

**Exploratory documents.** A slot whose document has no trustworthy
matching reference is declared *exploratory* before the seal. It is run
and reported, but it no longer counts toward the scored figures or the
authentic minimum below, and never toward the accuracy claim.

**Declared minimum of scored authentic Scripture-bearing evidence**
(fixed now, in `check-set.py` and `score-set.py`):

| | Minimum |
|---|---|
| Scored documents of kind *authentic* (in the manifest before the seal; completed in the run) | 8 |
| Formats among them | 2 |
| Languages among them | 4 |
| Verses (in their references before the seal; printed in the run) | 20 000 |

`check-set.py` reports the builder's figures before the seal (from the
references), and the scorer reports the run's (from the printed verses).
**If the minimum is not met**, the set may still be sealed and run, but the
result carries this qualification limit, verbatim from the scorer: *the
result qualifies the candidate for the generated documents and for the
authentic documents listed only; it makes no claim for authentic published
files in general.* If fewer than 4 authentic scored documents can be found,
Set H is a test of generated documents only, and should be described so.

**Known risks to the composition** (to be confirmed by the builder, not
assumed): public-domain Korean and Hebrew editions with a matching
reference may all be among the disclosed ones (`disclosed-editions.tsv`);
authentic DOCX Bibles are rare. Where a slot cannot be filled with an
undisclosed edition, the builder leaves it out or declares it exploratory,
and says so; it is never filled with a disclosed edition. Hebrew documents
are expected to verify little (`known-weaknesses.md` §B): safe, but no
coverage.

## 4. Document classes and references

| Class | Reference | What is judged | Counts toward the accuracy claim |
|---|---|---|---|
| **scored** | the exact matching reference: same edition, revision, canon and versification | wrong verified text; verified references the edition does not have | yes |
| **control** | none (`-`) | every verified reference is a failure | no (gate only) |
| **mixed** | none (`-`); a sealed `allowed/<key>.txt` lists the passages it quotes | every verified reference outside the list is a failure | no (gate only) |
| **exploratory** | none, or one that is not an exact match | nothing; raw observations only | **never** |

* **Generated documents:** `scripts/corpus/audit-derived-completeness.py`
  (version 2) must pass before the seal, for DOCX made from USFM. For PDF
  and EPUB derived from USFM there is **no** qualified auditor. The builder
  samples at least 20 verses per document by hand, records them, and lists
  the document's unaudited part.
* **Authentic documents:** the reviewer samples at least 20 verses per
  document against the reference and records every known divergence
  (missing or added verses, revisions, numbering) before the seal. A
  reference that cannot be shown to match is exploratory.

## 5. Criteria, fixed now

The scorer keeps three layers apart: the raw observations (never
reclassified), the execution of each document, and this assessment.

**Execution**, per document: *completed* (exit 0 and a report), *refused*
(exit 0 and the app's clean refusal of the whole document), or *failure*
(timeout, memory, aborted, killed, crashed, panicked, no output, not run).
A report a failed process left behind is kept as a diagnostic and never
counted.

**Research gate (all must hold):**

1. **0** verified verses with wrong text in completed scored documents;
2. **0** verified references their edition does not have;
3. **0** verified references in controls, and **0** outside the allowed
   list in mixed documents;
4. **0** execution failures: every document completed or refused cleanly.

**Protocol run:** the frozen binary, the protocol ceilings (§7), the
canonical disclosed-document inventory, a complete RUN.txt for every
document, outputs unchanged since the run, and the seal the run names.
Anything else is "not a protocol run" whatever the counts.

**Verdict:** *pass*, *fail*, or *not a protocol run*. The **accuracy
claim** is only: *N verified verses with wrong text and M outside the
edition, among V verified verses of D completed scored documents (A
authentic)*. Exploratory documents are not part of it. **Coverage**
(verified correct ÷ printed) is reported per document and group, with no
pass mark.

A divergence between reference and document found after the run counts as
a failure in the raw score. It may be explained in a separate
adjudication ledger, with source evidence, beside the raw figure, never
instead of it. **Failing** the gate means the candidate is not ready. Set H
is then disclosed; corrections need a new candidate and a new withheld set.

## 6. Safeguards: what the code enforces, what is procedure

| Safeguard | Enforced by code | Procedure (who records it) |
|---|---|---|
| The frozen binary | runner compares its SHA-256 and refuses; RUN.txt records it; the scorer checks it | — |
| The pinned tools | RUN.txt records the runner's and `check-set.py`'s SHA-256 | the assessor compares them with `FREEZE.md` §5a |
| The seal | `seal-set.py --write` seals only a folder that passes every check, with the reviewer's ledger and the access records present, and never twice; `check-set.py` at the run: the seal folder is exactly what one commit added, that commit changes nothing else, nothing changed since, every file as in `SEAL.txt` | the builder commits it; the owner approves before |
| The documents | every file in the documents folder listed in `checksums.sha256` and unchanged; nothing unlisted; every manifest file listed | — |
| Provenance | a complete row per document; its file hash the checksummed one | the reviewer checks that each row is **true** and signs `identity_checked_by` |
| Not a disclosed document | a file, or a USFM book inside a reference, byte-identical to anything in the canonical inventory (7 461 hashes) is refused | the same edition re-saved, re-zipped, in another format or from another publisher is **not** caught by hashes: the builder checks `disclosed-editions.tsv`, the reviewer confirms each edition and signs |
| Not a disclosed key | a key a disclosed manifest uses is refused (the harness would also run that edition's files) | it proves nothing about whether an edition is unseen |
| One run | the output folder must not exist; the documents folder gets `.run-started` before the first document is read, and a folder that has it is refused; a committed run record (`<seal>/../run/`) makes every later attempt from a checkout that has it refuse | the runner commits RUN.txt, OUTPUTS.sha256 and the score **at once**. A copy of the documents made before the run, used from a checkout without the run record, cannot be stopped by code: access records |
| The run as scored | the scorer refuses outputs changed or added since the run, or a RUN.txt of another seal, and declares a run with other ceilings, binary or inventory "not a protocol run" | — |
| Independence of builder and reviewer | — | no repository of this project attached, neutral brief only, access records (§2) |
| No copyrighted text in this repository | the run record holds hashes, checks and scores only; reports that quote text stay outside Git | public-domain sources by default (§2) |

## 7. Ceilings and the exact run

| Ceiling | Value |
|---|---|
| Time per document | 30 minutes (`timeout 1800`) |
| Address space per document | 4 GB (`ulimit -v 4194304`) |
| Peak reading memory to report | as measured; above 1 GB is a finding |

The exact run, by the runner, once (no `AW_RUN_*` or `AW_DISCLOSED_INVENTORY`
variables set: they exist for rehearsals only and make the run "not a
protocol run"):

```
scripts/evaluation/run-withheld-set.sh docs/evaluation/set-h/seal <seal commit> <documents folder> <harness binary> 920f7e28e6c03f0c2fd7832bb78ce7c2077d6767541f0b86bb02f2fec1ed3a8e <new output folder outside Git>
```

then, at once, the run record (`docs/evaluation/set-h/run/`: RUN.txt,
OUTPUTS.sha256, CHECK.json) committed; then, by the assessor:

```
python3 scripts/evaluation/score-set.py <output folder> docs/evaluation/set-h/seal --json=score.json --md=score.md
```

and `score.json` and `score.md` added to the run record.

## 8. Seal contents

The seal folder `docs/evaluation/set-h/seal/`, committed in one commit
that changes nothing else, before the run:

* `manifest.tsv`: 14 columns, the harness's 12 plus `kind` (authentic or
  generated) and `class` (scored, control, mixed, exploratory);
* `checksums.sha256`: every document and reference;
* `provenance.tsv`: per document, publisher, edition, revision, where and
  when it was retrieved, licence, its SHA-256, the reference's source and
  method, and who checked its identity;
* `allowed/<key>.txt` for each mixed document;
* `review-ledger.md`: the reviewer's signed pre-seal ledger, with the
  sampled verses, the completeness-audit results and every known
  divergence;
* `access-records.md`: who filled each role, and what each read;
* `SEAL.txt`: the UTC time and the SHA-256 of every other file.

Documents stay outside this repository. Copyrighted text never goes into
it or into public comments.

## 9. Rehearsed on disclosed documents, not run on Set H

The tools were qualified by 29 synthetic tests and three sealed rehearsal
sets of disclosed documents run with the frozen binary (`FREEZE.md` §5a,
`rehearsal/`). The rehearsal shows every class, a clean refusal, a
timeout, a real memory failure under a lowered ceiling, the refusal of
disclosed documents by the canonical inventory, and the refusal of a
second attempt. Every rehearsal verdict is "not a protocol run", as it
must be.

## 10. Before authorising

- [ ] Approve or amend §2–§8.
- [ ] Approve the builder arrangement (§2): the private set repository,
      the two environments and the source list; or name people instead.
- [ ] Decide the licences allowed in cloud sessions: public domain only
      (proposed), or also open licences.
- [ ] Accept the authentic minimum and the qualification limit (§3),
      including the risk that Korean or Hebrew slots cannot be filled.
- [ ] Then, separately: authorise building, sealing and the single run.
