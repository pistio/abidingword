# Set H: owner approval package

The one document for the owner's decision. Details, if needed, are in
`set-h-decision-package.md` (protocol), `FREEZE.md` (candidate) and
`data/isolation-test-2026-11-20/` (environment check). **Set H has not been
selected, downloaded, inspected, built, sealed or run.** The evaluation is
not complete until it has been run and assessed.

## Readiness verdict: **NOT READY**

**The remaining blocker:** a builder or reviewer session cannot be
*enforced* to stay away from the recogniser's public source.

In the tested restricted environment, a fresh session cannot open
`pistio/abidingword`'s web page, raw files, API or archive. It **can** read
the repository with plain `git clone`, as it can any public repository.
The proposed isolation was therefore partly procedural, not enforced as
the decision package claimed. You must choose how separation is provided
(§4: option A, B or C).

**Also outstanding:** one message from you completes the environment check
(§5, step 0). It covers the write path to the session's own repository,
and whether server-side fetching reaches the repository. A session started
by another session will not push without a person's go-ahead.

Once you have chosen A or C and the write path is shown, nothing else
stands between this package and your authorisation. No further engineering
is needed. Option B needs a re-run of the probe after the change.

## 1. Identities

| | |
|---|---|
| Recogniser (frozen) | `08c6faac8c92c6c791ec1e7bbbe997362fa1059f`, tree `be296f2570dcbcc251789b2b1e1bbe2c9e7c2798` |
| Source checksums | `candidate-source.sha256`: 279 files, all re-verified on this branch; its own SHA-256 `fc2834d11f3b90bde8c8fcc3eb72d0e8438a4f061afb86026eb5692d041ef115` |
| Harness binary | `real_document_editions`, release: `920f7e28e6c03f0c2fd7832bb78ce7c2077d6767541f0b86bb02f2fec1ed3a8e` |
| Toolchain | rustc and cargo 1.97.0; `Cargo.lock` unchanged since A1.3; `AW_PIPELINE=structural` |
| `scripts/evaluation/check-set.py` | `01a13dde21510ad9e9f3b9c301d971e87feffbacdcaee0449d6c2f9180c3411f` |
| `scripts/evaluation/seal-set.py` | `e343202e2cbf6fb35b9fa70b74b6a457b214d5bc9ece9ecc2c101d57c0e23c8f` |
| `scripts/evaluation/run-withheld-set.sh` | `8515e92c329370236f5aab0495b4e08c4f6a76f64f6e8d0abbd59584181385a9` |
| `scripts/evaluation/score-set.py` | `658de955cc4594b5893ba84518ccbdfc7673b95da280edc5056c09835fb9a9bf` |
| `docs/evaluation/disclosed-documents.sha256` (canonical inventory, 7 461 hashes) | `a9d83ac45d356997d2c965d1635cb6a4280b26a988625c668aaeb13d28aac2b8` |
| `docs/evaluation/disclosed-editions.tsv` | `9c736eb2394803dfe44b938eee7c88f0881c28f2113bbff2541a9f695cb0801a` |
| `docs/evaluation/builder-brief.md` | `f4b63afab9c1ecfb70c692b2b4b95618cb8cc7801039ac04d9aa46e3c3459b1b` |
| `scripts/corpus/audit-derived-completeness.py` (v2) | `d268175759f92add320530c9b8f5509f9f1b60ba726cc5f861501db5c90ab5f0` |

**Historical evidence, unchanged:** on the 113 disclosed editions,
1 980 257 verified correct, **9 verified wrong** (all from the defective
original `spablm-docx` fixture) and 0 verified references absent from the
edition. On the corrected v2 fixture, 31 072 verified and 0 wrong. The
80-edition timing comparison, and every earlier report and raw score, are
also unchanged.

## 2. Protocol and pass/fail

**Set:** 24 documents. The scored ones have an exact matching reference:
the same edition, revision, canon and versification. The set includes 3
controls with no Scripture and 2 mixed-content documents, each with a
sealed list of the passages it may verify. Documents with no trustworthy
reference are declared *exploratory* before the seal and are never scored.
No disclosed edition is used. The allocation is in
`set-h-decision-package.md` §3.

**Published-evidence minimum:** at least 8 *completed* scored authentic
documents, in at least 2 formats and 4 languages, printing at least 20 000
verses between them. These are printed verses, not verified ones. If the
minimum is not met, the scorer adds a qualification limit, verbatim.

**Pass requires all of the following:**
1. 0 verified verses with wrong text;
2. 0 verified references their edition does not have;
3. 0 verified references in controls, and 0 outside the allowed lists in
   mixed documents;
4. 0 execution failures: every document either completes or is refused
   cleanly;
5. both anchors hold: the seal commit and the runner's own run-record
   commit;
6. a protocol run: the frozen binary, the 4 GB and 30-minute ceilings, and
   the canonical inventory.

If 1–4 do not all hold, the verdict is **fail**. If 5 does not hold, the
scorer refuses to assess at all. If 6 does not hold, the verdict is
**not a protocol run**.

**Reported separately, never merged:**
* verified correct verses, and coverage (verified correct ÷ printed, over
  completed scored documents), with no pass mark;
* abstentions within read documents: refused-correct and caught verses,
  and refused chapters;
* **clean refusals of whole documents**: named in the accuracy claim as
  *not recognition coverage*, contributing no verse;
* execution failures, with their cause;
* exploratory documents, raw only;
* any language or format the set could not cover with an undisclosed,
  accurately referenced document. This is stated as a limit, never filled
  with disclosed material.

## 3. Builder and reviewer environment

| Setting | Value |
|---|---|
| Network | **Limited** |
| Allowed hosts | proposed: `ebible.org`. Others only with your approval. **Not** `www.gutenberg.org`: it answers this cloud with `503`, retry after 6 hours, so Gutenberg files can only come from you |
| Package managers / MCP servers | off / off |
| Setup script, variables, saved credentials | none |
| Repositories | the private set repository only (§5) |
| Sessions | **started by you**, new (not a fork or continuation), with the brief as the first message |

## 4. Enforced and procedural guarantees

Demonstrated in `data/isolation-test-2026-11-20/`:

| Guarantee | Status |
|---|---|
| No checkout, memory, `CLAUDE.md` or chat history from this project | **enforced** (demonstrated) |
| GitHub tools and API scoped to the session's own repository; no GitHub MCP tools | **enforced** (demonstrated) |
| This repository's web page, raw files, API and zip archive | **refused** (demonstrated) |
| Other private repositories of the account, over git | **refused** (demonstrated) |
| Hosts outside the allowlist; package registries | **refused** (demonstrated) |
| **This repository over `git`** (and any public repository) | **NOT enforced**: `git clone` works (demonstrated) |
| WebFetch (server-side) and `add_repo` of this repository | **untested**: needs your message (§5, step 0) |
| Writing to the session's own repository | **not yet demonstrated**: needs your message |
| Builder and reviewer not told the project | procedure: the handed-over files no longer name the project or its location (checked) |
| No access to the recogniser by any route | procedure: the whole transcript is reviewed before the seal and again by the assessor before scoring |
| The model's general knowledge | cannot be ruled out |

**Choose one** (the blocker):

* **A. Procedural separation, audited (recommended).** Fresh sessions;
  neutral repository and file names; the builder is never told the project.
  Every transcript is reviewed for any `git`, web or tool access to a
  repository other than the set repository. Any such access voids that
  session's work, and you decide what follows. Sets E–G were built under
  similar procedural separation.
* **B. Enforced for the build and review window.** Make `pistio/abidingword`
  private while the builder and reviewer sessions are open; private
  repositories outside a session's scope were refused over git (D4). Re-run
  the probe first, and restore visibility after the seal. GitHub's
  confirmation dialog lists the side effects. A private repository's CI
  uses the account's Actions minutes, and the mandate forbids paid Actions
  without approval. Check the test-update channel and the PR #54 work
  first.
* **C. A person builds and reviews**, outside any session with access to
  this project. This is procedural, on that person's conduct.

## 5. Private repository and environment set-up

0. **Finish the check:** open
   `https://claude.ai/code/session_011iiPcUd2VF4YHZiXU9gpCL` and reply:
   *"Confirmed. Run all four steps, including the push to `main` of this
   test repository."* Then delete `pistio/abidingword-seth-isolation-test`
   and the environment `Set H isolation test`.
1. **Set repository.** Private, with a neutral name that does not name the
   project, for example `pistio/collection-h`, initialised with a README.
   It can be created for you with the GitHub tool: creating the disposable
   test repository that way worked, and the Claude GitHub App could reach
   it at once. Or create it yourself: GitHub → New repository → owner
   `pistio` → the name → **Private** → *Add a README* → Create. Then check
   that Settings → Applications → Claude → Repository access includes it.
   Never make it public.
2. **Builder files.** Commit to its `builder/` folder exactly the files
   listed in `docs/evaluation/README.md` §"For the builder".
3. **Environment.** As §3, named neutrally (for example "Collection H"),
   repository: the set repository.

## 6. Coverage limitations known in advance

* Korean and Hebrew: undisclosed public-domain editions with an exact
  reference may not exist. Such a slot is left out or declared
  exploratory, and the limit is stated.
* Hebrew right-to-left PDFs and Hebrew-letter numerals verify nothing.
  This is safe but gives no coverage.
* Authentic DOCX Bibles are rare.
* Project Gutenberg is unreachable from cloud sessions.
* There is no qualified completeness auditor for PDF or EPUB made from
  USFM.
* Missing text is invisible: verification means fidelity to the document
  read.

## 7. Copyright and licences

* Public-domain texts only.
* Open licences only if you approve them.
* No all-rights-reserved text in any cloud session.
* Documents and outputs live only in the private set repository, never in
  this repository or any public comment.

## 8. Preparing, reviewing and sealing the set

1. You approve this package (§4 option, §3 hosts, §7 licences), and the
   repository and environment are set up (§5).
2. You start the **builder** session. It builds the 24 documents and
   references, records provenance, and runs `seal-set.py` until it passes.
3. You start the **reviewer** session, fresh. It checks every reference
   (edition, revision, canon, versification, at least 20 sampled verses,
   the boundaries) and every edition against `disclosed-editions.tsv`, and
   signs `review-ledger.md`.
4. The builder's and reviewer's transcripts are reviewed for access
   outside the set repository (option A).
5. You approve the list of documents: metadata only.
6. The builder seals with `seal-set.py --write`.
7. The seal folder is committed alone to `docs/evaluation/set-h/seal/` in
   this repository.

## 9. The single run

1. A **runner** session, with this repository and the set repository,
   confirms the binary's SHA-256, then runs once:
   `scripts/evaluation/run-withheld-set.sh docs/evaluation/set-h/seal <seal commit> <documents folder> <harness binary> 920f7e28e6c03f0c2fd7832bb78ce7c2077d6767541f0b86bb02f2fec1ed3a8e <new output folder>`.
   * The runner refuses before reading anything if the seal, a document,
     a path, the binary, the inventory or an earlier run is wrong.
   * It commits the run record itself and prints that commit, which is
     pushed at once.
   * The outputs are copied to the set repository.
2. An **assessor** session (not the runner's) runs
   `python3 scripts/evaluation/score-set.py <output folder> docs/evaluation/set-h/seal --run-record=<record commit> --json=score.json --md=score.md`.
   * It refuses anything not anchored to the committed seal and run record.
   * The assessor compares RUN.txt's runner and check-set hashes with §1.
   * It writes the case ledger.
   * The score is committed beside the record.

## 10. What the result permits

* **Pass:** research use of the frozen candidate, and the start of its
  integration review. It is **not** a release, not a general claim that
  recognised Scripture is accurate, and not migration 0028. The accuracy
  claim is the scorer's sentence, with its coverage, refusals and any
  qualification limit.
* **Fail:** the candidate is not ready. Set H becomes disclosed. A fix needs
  a new candidate and a new withheld set; Set H is never rerun and
  presented as independent.
* **Not a protocol run:** no conclusion. If the documents were exposed, a
  new set is needed.

## Integration work (main application, PR #54; not recognition development)

The full list is `docs/handover-scripture-recognition-codex.md` §12. Each
item stays open whatever Set H shows:

1. The structural gate must run again after every saved correction or
   re-recognition.
2. Provenance and per-book numbering evidence must be regenerated after
   such changes.
3. Migration 0028 for provisional standing and evidence.
4. Unverified text must never reach display, comparison, search, Study
   citations, Teaching, AI context or exports as verified Bible text.
5. PDF processing in a memory-limited worker process.
6. Progress and cancellation.
7. Edition-specific numbering, with validated cross-edition mappings only.
8. Windows integration and end-to-end acceptance.
