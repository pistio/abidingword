# Set H: owner approval package

The one document for the owner's decision. Details, if needed, are in
`set-h-decision-package.md` (protocol), `FREEZE.md` (candidate) and
`data/isolation-test-2026-11-20/` (environment checks). **Set H has not been
selected, downloaded, inspected, built, sealed or run.** The evaluation is
not complete until it has been run and assessed.

## Readiness verdict: **READY FOR OWNER AUTHORISATION**

You have decided the arrangement (below), and the approved environment and
set repository are set up. Every material access route has been checked.
Separation of the builder and reviewer from the recogniser is
**procedural, with a complete transcript audit**. It is **not technically
enforced**: a session in the approved environment can still read public
GitHub repositories with `git`, and can use web search.

**The one remaining step is yours:** authorise selection and building, and
start the builder session with the message in §8. Sealing and the single
run stay separately authorised (§8, §9).

## Owner decisions (20 November 2026)

1. **Independence: option A.** Procedural independence with strict
   auditing: fresh, separately started builder and reviewer sessions;
   neutral repository names and instructions; no checkout of this project
   and no inherited context; complete transcript review
   (`docs/evaluation/transcript-audit.md`). Any access to the recogniser's
   source, tests, earlier results or other prohibited repositories
   invalidates the affected session. Separation is **not** described as
   technically enforced while public `git` access remains possible.
2. **Set repository and environment approved:** `pistio/collection-h`
   (private) and the environment `Collection H`, with **`ebible.org` as
   the only source host**. No further host is added automatically. The
   second isolation test's result (an invented file committed and pushed
   to a disposable private repository) is preserved.
3. **Licences:** public-domain texts. Open-licensed texts may be
   **proposed** for separate approval if authentic published evidence is
   insufficient. No all-rights-reserved material in cloud sessions without
   explicit permission.
4. **Published-evidence minimum accepted, never reduced to pass:** 8
   completed authentic scored Scripture documents, in 2 formats and 4
   languages, with 20 000 printed or reference verses. The report keeps
   apart measured accuracy, verified coverage, clean refusals, exploratory
   material and unsupported languages.

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
| `docs/evaluation/builder-brief.md` | `bf7008d84c2dcd2c3f4d8f5e5b58d01475ff8ead8425314c186cc181971be147` |
| `docs/evaluation/transcript-audit.md` | `6552c9095243a1320fb59d47b47ad12e873b9f72faa32737793b05797bd356a8` |
| `scripts/corpus/audit-derived-completeness.py` (v2) | `d268175759f92add320530c9b8f5509f9f1b60ba726cc5f861501db5c90ab5f0` |

The set repository's `builder/` folder (commit `4f84805`) holds copies of
the brief, both exclusion lists, `check-set.py`, `seal-set.py` and the
auditor, with these SHA-256 sums.

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
`set-h-decision-package.md` §3. A language that cannot be represented by an
undisclosed, accurately referenced document is left out, and the limit is
stated; it is never filled with disclosed material.

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

If the published-evidence minimum is not met, the scorer adds its
qualification limit, verbatim. The minimum is never lowered.

**Reported separately, never merged:**

| Measure | What it is |
|---|---|
| Measured accuracy | verified verses with wrong text, and verified references outside the edition, over completed scored documents |
| Verified coverage | verified correct ÷ printed, over completed scored documents; no pass mark |
| Abstentions | refused-correct and caught verses, and refused chapters, inside documents that were read |
| **Clean refusals** | whole documents the app refused. They are named in the accuracy claim as *not recognition coverage* and contribute no verse |
| Execution failures | each with its cause |
| Exploratory material | raw observations only, never in accuracy or coverage |
| Unsupported languages | languages or formats the set could not cover, stated as limits |

## 3. Builder and reviewer environment (set up)

| Setting | Value |
|---|---|
| Environment | `Collection H` (`env_01Kus7n5JxhUcNsGy6fHog2r`), personal |
| Network | **Limited**; allowed host **`ebible.org` only** (reachable, 200). Further hosts only with your separate approval. Project Gutenberg answers this cloud with 503 and is not used |
| Package managers / MCP servers | off / off |
| Setup script, variables, saved credentials | none |
| Repository | `pistio/collection-h`, private; `main` at `4f84805`: README and `builder/` |
| Sessions | **started by you**, new (not a fork or continuation), with the message in §8 as the first message |

## 4. What is enforced and what is procedure (as tested)

| Route or guarantee | Status |
|---|---|
| No checkout, memory, `CLAUDE.md` or chat history from this project | **enforced** (demonstrated) |
| GitHub tools and API scoped to the session's own repository; no GitHub MCP tools | **enforced** (demonstrated) |
| Repository and scheduling tools (`list_repos`, `add_repo`, `check_repo_access`, scheduled tasks) | **not exposed** to the session (two probes) |
| Writing to the session's own repository | **works** (demonstrated: `f50b267`) |
| This repository's web page, raw files, API and archive | **refused** (403) |
| Server-side page fetches (WebFetch) of this repository | **refused** (host not in the allowlist) |
| Other private repositories over git | **refused** |
| Hosts outside the allowlist; package registries | **refused** |
| **`git` reads of any public repository, this one included** | **open**: procedure, audited |
| **Web search** | **open**: allowed only to find editions on the approved source; audited |
| Helper agents | run under the same rules; their calls appear in the parent's transcript (demonstrated) |
| Builder and reviewer not told the project | procedure: the handed-over files name no project (checked) |
| The model's general knowledge; the owner account's public profile | cannot be ruled out: the set repository's owner is `pistio`, whose public repositories include this one |

## 5. Set-up done, and clean-up

* **Done:** the set repository `pistio/collection-h` (private), the
  environment `Collection H`, and the builder files in `builder/`.
* **Optional clean-up** (nothing in them but invented text):
  * delete the repository `pistio/abidingword-seth-isolation-test` and the
    environment `Set H isolation test`;
  * archive the probe sessions `session_011iiPcVvhdecywvqehnFXRQ`,
    `session_011iiPcUd2VF4YHZiXU9gpCL`, `session_011iiPcSZEDbzb7FEUTXiBo7`
    and `session_011iiPcWVox1cFwyXNzRqa8y`.

## 6. Coverage limitations known in advance

* Korean and Hebrew: undisclosed public-domain editions with an exact
  reference may not exist. Such a slot is left out or declared
  exploratory, and the limit is stated.
* Hebrew right-to-left PDFs and Hebrew-letter numerals verify nothing.
  This is safe but gives no coverage.
* Authentic DOCX Bibles are rare.
* With `ebible.org` as the only source, many documents may come from one
  publisher's own conversions. Overlap of publisher or converter with the
  disclosed editions is reported, not hidden.
* There is no qualified completeness auditor for PDF or EPUB made from
  USFM.
* Missing text is invisible: verification means fidelity to the document
  read.

## 7. Copyright and licences

* Public-domain texts.
* Open-licensed texts only if proposed and then separately approved by you.
* No all-rights-reserved material in any cloud session without your
  explicit permission.
* Documents and outputs live only in the private set repository, never in
  this repository or any public comment.

## 8. Preparing, reviewing and sealing the set

1. **Authorise selection and building** (the remaining step). Then start a
   new session in Claude Code: environment **Collection H**, repository
   **`pistio/collection-h`**, with this first message:

   > You are the builder of a test collection. Read `builder/builder-brief.md` in this repository and follow it exactly. The approved source host is `ebible.org` only; use public-domain texts only. Work in this repository: put the documents and references in `documents/` and the seal folder in `seal/`, and commit and push your work to `main` as you go. Do not seal: run `python3 builder/seal-set.py seal documents --inventory=builder/disclosed-documents.sha256` until it passes, then tell me you are ready for review.

2. **Review.** When the builder reports ready, start a second new session
   in the same environment and repository, with:

   > You are the reviewer of the test collection in this repository. Read `builder/builder-brief.md`; its rules on what you must not seek apply to you as well. Do not change any document or reference. For every scored document, check that the reference is the same edition, revision, canon and versification; sample at least 20 verses against the document; check the first and last verse of each book; record every divergence. Check every edition against `builder/disclosed-editions.tsv`. Write and sign `seal/review-ledger.md`, commit and push it, and tell me the result.

3. **Audit.** A session that is neither the builder nor the reviewer (or
   you) reads both transcripts against
   `docs/evaluation/transcript-audit.md`, and records the result in
   `seal/access-records.md`. Any invalidating event: that session's work
   is not sealed, and you decide.
4. **You approve the list**: metadata only.
5. **Seal** (separate authorisation): the builder runs `seal-set.py --write`.
   The seal folder is committed alone to `docs/evaluation/set-h/seal/` in
   this repository.

## 9. The single run (separate authorisation)

1. A **runner** session, with this repository and the set repository,
   confirms the binary's SHA-256, then runs once:
   `scripts/evaluation/run-withheld-set.sh docs/evaluation/set-h/seal <seal commit> <documents folder> <harness binary> 920f7e28e6c03f0c2fd7832bb78ce7c2077d6767541f0b86bb02f2fec1ed3a8e <new output folder>`.
   * The runner refuses before reading anything if the seal, a document,
     a path, the binary, the inventory or an earlier run is wrong.
   * It commits the run record itself and prints that commit, which is
     pushed at once.
   * The outputs are copied to the set repository.
2. An **assessor** session (not the runner's) re-audits the transcripts,
   then runs
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
