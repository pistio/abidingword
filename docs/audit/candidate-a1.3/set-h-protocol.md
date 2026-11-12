# Set H: protocol for an independent evaluation of the frozen candidate

**Status: a proposal for the owner's approval. Set H does not exist.**
Nothing has been generated, selected, downloaded or run. Approving this
protocol, and separately authorising Set H, are the owner's decisions.

The candidate is `7c7565a7a3df8f3dcc9711c8701af8c94dbad6af`
(`FREEZE.md`). A freeze is permission for research evaluation only. It is
not app integration, a release, or a claim that recognised Scripture is
accurate. Recognised text stays provisional whatever Set H shows.

## 1. Why Sets B–G are not enough

* Sets B–G are disclosed. The candidate's rules were written after their
  results were read, so its results on them are regression evidence.
* Every document in B–G was generated in this environment from eBible.org
  USFM by scripted tools. The sets vary layout and generator, not the
  source of the words or the kind of builder.
* The `spablm-docx` case shows a third weakness: a generated document can
  differ from the reference it is scored against, and nobody notices until
  the recogniser is blamed (`ledger.md`).

Set H should answer each of these.

## 2. Roles, kept apart

| Role | Does | Must not |
|---|---|---|
| **Owner** | Approves this protocol, the composition and the pass criteria before anything is built; authorises the single run | — |
| **Builder** | Selects or generates the documents and their reference texts; audits them; seals the set | See the recogniser's code, rules, tests, reports or any earlier set's results; run any recogniser |
| **Runner** | Runs the frozen harness once on the sealed set; publishes every output unmodified | Change code, harness, manifests or documents; rerun after seeing results |
| **Judge** | The frozen harness's comparison with each edition's reference text, at the candidate SHA | Be adjusted after the seal |

The builder should not be the session that wrote the recogniser. Two
ways to arrange that, strongest first:

1. **Owner-curated authentic documents.** The owner (or someone the owner
   asks) collects real published files: publishers' own EPUBs, PDFs and
   Word documents, not made from eBible USFM by a script. The builder
   only pairs each with a reference text and seals. This is the only
   arrangement that tests documents of other people's making.
2. **An isolated builder session**, as for Sets E–G: a neutral written
   brief, a working folder of its own, no read access to the repository,
   the corpora or the audits, and a statement of every path it read.
   Weaker: same environment, same kind of builder, eBible-sourced words.

A mixed set is reasonable: authentic documents where a trustworthy
reference exists, generated ones for scripts and layouts that cannot be
found otherwise. Each document's kind is recorded in the manifest and the
results are reported per kind.

## 3. Composition (to be fixed by the owner before building)

Proposed minimum, 24 documents:

| Dimension | Proposal |
|---|---|
| Formats | EPUB at least 10, PDF at least 8, DOCX at least 4; EPUB-heavy is acceptable |
| Authentic documents | At least half, if arrangement 1 is possible |
| Languages | English, German, French, Spanish, Portuguese, Korean, at least two each; Greek, Hebrew and Latin, at least one each |
| Scripts | Latin, Hangul, Greek, Hebrew (right to left, pointed and unpointed) |
| Canon | At least two with deuterocanonical books; at least two New Testament only; at least one single book |
| Layouts | Chosen by the builder without advice: columns, notes, cross-references, headings, poetry, introductions, contents pages, study apparatus |
| Editions | None used in the development corpus or Sets B–G (list supplied to the builder as names only) |
| Negatives | At least three documents that are not Bibles or hold no verses (a commentary, a lectionary, a hymnal): nothing in them may be verified |

No document may hold altered Scripture. The words are the publisher's.

## 4. Reference texts and the completeness audit

Each document is scored against a reference text of the same edition.

* **Generated documents:** before sealing, the builder runs a
  source-to-document completeness audit of its own (or
  `scripts/corpus/audit-derived-completeness.py`, which shares no code
  with any generator). Every verse's words must be in the document, in
  order. A document that fails is repaired or left out **before** the
  seal, and the audit's output is sealed with the set.
* **Authentic documents:** the reference is the same edition's text from
  an independent source. Where the published file and the reference are
  known to differ (a revision, an omitted verse), the builder records it
  at seal time. A difference found only after the run is reported as a
  recogniser finding first, and reclassified only with source evidence
  and the original result kept (§7).

## 5. Sealing

Before the run, committed to the repository in one commit that changes
nothing else:

* the manifest: key, files, reference, language, script, format, layout,
  kind (authentic or generated), source and licence, books, SHA-256,
  generator and version;
* the SHA-256 of every reference text and source file;
* the completeness-audit outputs;
* the builder's ground-truth notes (what each document is meant to test),
  encrypted or hashed if the owner wants them unread until after the run;
* `SEAL.txt`: the manifest's SHA-256, UTC time, and the builder's
  statement of every path read.

Documents stay outside Git, as in every earlier set.

## 6. The run

* One run. Release build of the harness at the candidate SHA; the
  binary's SHA-256 recorded (`FREEZE.md` has the expected value on this
  toolchain).
* `AW_PIPELINE=structural`: the structure-aware reading for EPUB, the
  candidate's reading for everything else.
* Every input's checksum is verified immediately before the run.
* Every output is archived unmodified. No output is read before the run
  is complete for all documents.
* A crash, a timeout or an unreadable document is a result, not a reason
  to rerun.

## 7. What is reported

For the set, and per format, language, script and kind:

| Measure | Meaning |
|---|---|
| **Verified, wrong text** | Verified verses whose text differs from the reference |
| **Verified, not in the edition** | Verified references the edition does not have |
| Verified, correct | Coverage |
| Refused, correct | Conservative refusals |
| Caught | Not verified, text wrong |
| Refused chapters and documents; books not found | — |
| Anything verified in a negative document | Counted as wrong |
| Reading time and peak memory | Release build |

Every wrong verified verse and nonexistent reference is listed and
classified: extraction, segmentation, reference assignment, omitted
source text, verification, or reference divergence. **Reference
divergence may be claimed only with a source-to-document audit, and the
verse stays in the count as originally measured**, with the
reclassification beside it.

## 8. Pass criteria (to be fixed by the owner before the seal)

Proposed:

* **0** verified references the edition does not have;
* **0** verified verses with wrong text, with no exception argued after
  the fact (reference divergence is reported separately, §7, and the
  owner decides what it means);
* **0** verified verses in negative documents;
* no crash or unbounded memory on any document;
* coverage is reported, with no pass mark. A low figure on a class of
  document is a finding. (A coverage threshold invites tuning for it.)

Failing any of these means the candidate is not ready. It does not mean
the rules are patched and Set H is run again: after its single run Set H
is disclosed, becomes regression evidence like B–G, and a later candidate
needs a new withheld set.

## 9. Not covered

* Scanned PDFs and OCR.
* Sources, the database (migration 0028 is Codex's), the interface, and
  Windows behaviour.
* PDFium: the A2.1 helper is a separate experiment and takes no part.
* The truth of a translation. The judge checks a document against its own
  edition's text, nothing more.
