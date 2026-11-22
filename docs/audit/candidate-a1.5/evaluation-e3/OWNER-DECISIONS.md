# Evaluation candidate A1.5-E3: decisions for the owner

**Prepared, not decided.** Nothing here is approved, and nothing in E3
assumes an answer. Until the owner decides, the committed protocol
(`docs/evaluation/protocol-e3.json`) keeps `required_composition: null` and
`withheld_execution_permitted: false`: the runner refuses to run any set
under it and no score under it is a protocol run. Collection H was not
accessed in preparing this; no membership is proposed.

Each decision ends with what would change, where, if the owner chose it.
None of the choices needs a change to the recogniser.

## 1. Composition: 24 or 28 documents

**The facts.**

| Source | Composition |
|---|---|
| `../SET-H-APPROVAL.md` §2, `../set-h-decision-package.md` §3 (committed, owner-approved package) | **24** documents: 19 scored, 3 control, 2 mixed; several slots "either" authentic or generated; any slot may become exploratory before the seal |
| The independent evaluation audit's quotation of owner decision D5 (`../evaluation-audit/REPORT.md` §6, C2) | **28** documents: 23 scored, 3 control, 2 mixed, 0 exploratory; 16 authentic, 12 generated |

The text of D5 itself is not committed in this repository; only the audit's
quotation of it is. The two cannot both be the required composition.

**What the tools do now.** `check-set.py` and `score-set.py` compare the
declared composition (`composition.tsv`) with the protocol's
`required_composition` by **exact equality**, including authentic/generated
and exploratory. While it is `null`, no run is a protocol run (E2, kept).

**Options.**

* **A (no tool change).** The owner first confirms 24 or 28 as the target.
  The builder builds and declares a composition; before the seal, the owner
  reviews it and commits it, exactly, as `required_composition` in a
  protocol revision. Exact equality then holds by construction. Cost: the
  exact numbers are fixed after the builder's choices but before the seal
  and before any result.
* **B (tool change).** The protocol states fixed totals and minima (for
  example: total 24, control 3, mixed 2, scored + exploratory 19, authentic
  ≥ 8) and the check tests them as inequalities. Cost: a change to
  `check-set.py` and `score-set.py` (new SHA-256, a new review of that
  change).

*Recommendation for the owner's consideration:* decide 24 or 28 first
(the committed, approved package says 24); then A, which needs no new code.

## 2. D1 and D6: unsupported content is reported, not scored

**The facts.** As the audit quotes them, D1 requires scoring "in primary
numbering, including verse zero where present", and D6 requires limitations
to be reported and never used to change scoring. E2 (kept unchanged in E3)
does not score verse 0, chapters above the 66-book counts, books outside the
66 or unusual labels: it counts and lists them as **unsupported**, in every
score, beside the scorable figures, and never drops them. E3 adds that a
verse number covered twice in a chapter (a duplicate, or a bridge
overlapping another label) is unsupported too, and that a number inside a
sound bridge cannot be validated alone. The recogniser cannot hold any of
these; supporting them is not implied.

**Proposed wording (for the owner to accept, change or reject):**

> **D1 (amended).** Scoring uses each reference's primary numbering, as
> printed. Every verse is either *scorable* (a book of the 66, a chapter
> within its count, a verse 1–200 or a bridge, its number given once) or
> *unsupported* (verse 0, a chapter beyond the 66-book counts, a book outside
> the 66, an unusual label, a number given twice or covered by two labels).
> Unsupported verses are counted and listed in every score, by class, and
> never scored; a verified reference to a number inside a bridge, or to a
> number given twice, cannot be validated and fails criterion 2.

> **D6 (amended).** Limitations are reported and never change scoring.
> Unsupported reference content is reported, not scored: it is outside the
> scored denominator, and coverage of all printed verses is reported beside
> coverage of scorable ones. No limitation, and no unsupported content, is
> ever a reason to pass or to excuse a failure.

If the owner instead requires verse 0 (or any other class) to be scored,
that is a change of the recogniser's canon, outside this evaluation work.

## 3. The non-vacuous evidence requirement

**What E3 does (R2).** Safety and evidence are judged apart. The verdict is
*pass* only when the research gate (criteria 1–4, unchanged) holds **and**
the protocol's evidence floor is met; below the floor it is
*inconclusive*, never *pass*. The E3 protocol's floor (`aw-evidence-1`) is
the non-vacuity floor only: **at least one verse verified with faithful text
in a completed scored document.** A protocol without a readable floor gives
no pass. At the seal, a scored document whose reference prints no scorable
verse (or a declared book its reference does not print) is refused.

This sets **no coverage pass mark**: the approved policy that coverage has
none (decision package §5) is unchanged, and a clean refusal of a whole
document stays a clean *safety* result.

**For the owner: an additional quantitative floor, or none.** The protocol
has three slots, all `null` (not set). Each is evaluated by the E3 scorer if
set, so choosing one needs a protocol revision only, no code:

| Slot | Meaning | What it would change |
|---|---|---|
| `minimum_documents_with_verified_faithful` | at least N completed scored documents each verify one faithful verse | evidence spread over documents, not one document |
| `minimum_verified_faithful_share_of_printed_scorable` | verified faithful ÷ printed scorable ≥ X | **a coverage pass mark**: a change of the approved coverage policy |
| `maximum_scored_documents_refused_whole` | at most N scored documents refused whole | limits evidence lost to whole-document refusals |

Unknown slots, or values that are not numbers, give no pass (conservative).
The published-evidence minimum (8 authentic documents, 2 formats, 4
languages, 20 000 verses) stays a qualification limit, as approved, not a
verdict condition.

**Gate.** E3 keeps `withheld_execution_permitted: false` until the owner
has decided this (an additional floor, or explicitly none), the
composition (§1), D1/D6 (§2) and the tools (§4).

## 4. The approved scoring tools and binary

**What E3 does.** The protocol names every tool by SHA-256
(`approved_tools`). An official score requires the scorer, check, seal,
runner, both rule modules, the text reader and the book table in use, and
the runner, check, inventory and reader that RUN.txt records, to be exactly
those. A modified scorer committed to the checkout is therefore *not a
protocol run* (rehearsed: `evidence/after/rehearsal-e3.json`,
`modified_committed_scorer`). Every score states the protocol's own SHA-256
and asks the owner to check it.

**For the owner.** Approve (or not) the identities in
`EVALUATION-CANDIDATE.md` §1: the tools, the harness binary
`b4f51a62a51ef5085ca12900ba575ee04cc04f0b99f0fccf88db8890a141b63d` and the
protocol's own SHA-256. **Keep that protocol SHA-256 outside the
repository**: a scorer can be rewritten together with the protocol that
names it, and only the owner's own record of the approved protocol catches
that.

**The binary is environment-specific** (built at
`/home/user/abidingword/src-tauri`, rustc 1.97.0, webkit2gtk 2.52.6, Ubuntu
24.04.5, x86-64, empty `../dist`; the binary embeds its build path). An
assessor who rebuilds elsewhere will get another SHA-256; the protocol must
then be re-pinned in a reviewed revision, never overridden at scoring time.

## 5. Strictness (unchanged; for acceptance)

E3 keeps E2's strict fidelity rule (`aw-fidelity-2`) unchanged: exact,
canonical (NFC), white-space layout, or wrong text. Spacing beside
punctuation, invisible format characters, compatibility forms (ligatures)
and case are wrong text. The review supported it as the default. With the
reference text now read exactly (R1), every remaining mismatch is a real
difference between the document as read and the edition. Any relaxation
would be a new fidelity rule version.

## 6. How an approval would be recorded

One protocol revision, reviewed and committed before any seal, that sets
`required_composition`, any additional evidence floor (or leaves the slots
`null` with the owner's note that none is wanted), `approved_tools`,
`harness_binary_sha256` and `harness_build` for the assessor's build, D1/D6
wording in the approval record, and `withheld_execution_permitted: true`;
then the owner records that file's SHA-256 outside the repository.

## 7. Recorded for the owner's product backlog (outside this task)

* **The app's importer reads the bundled Bibles with the same space.** A
  person who imports the bundled engbsb, engwebp, fraLSG or grcsbl reads
  "heel. ”" in 528 verses (156, 14, 2 and 356). The harness's
  `importer_comparison` shows it on every run (whole BSB: 156 verses differ;
  SBL Greek NT: 356). The app's reader (`src-tauri/src/scripture/usfm.rs`)
  is part of the frozen recogniser and was not changed.
* **kor1961 Psalm titles hold control characters** (U+0001, U+0003; 18 in
  `19-PSAkor1961.usfm`, in `\d` titles, from the publisher's web pages).
  They are in no verse text, so the evaluation is unaffected; the app's
  reader keeps them in the superscription text it stores (how the interface
  shows them was not checked).
