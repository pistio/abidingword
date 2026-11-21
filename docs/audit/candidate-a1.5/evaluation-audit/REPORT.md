# Independent technical audit: the frozen evaluation system before the Collection H seal

**Verdict: EVALUATION RULES NOT CONFIRMED.**

Three exact blockers (§2): verse zero is never scored (B1); other
primary-numbered content of a sealed reference is dropped without a trace
(B2); and a verified verse in a chapter the sealed reference does not have
is never counted as a nonexistent reference (B3). Separately, the frozen
text-equivalence rule lets materially different wording count as verified
correct text (F1, §3). That rule is deterministic and was declared before
any result, so it does not breach D7, but the owner must accept it
explicitly or replace it. Five procedural conditions (§4) apply however the
blockers are settled.

Status: an audit record for the owner, stopped at owner review. It
authorises nothing: no seal, no run, no change to the frozen system, no
release, merge or tag.

## 1. What was audited, and how

| | |
|---|---|
| Branch audited | `claude/recognition-a1.5-evaluation-qualification` at `05b3e8b57eb74d02278b30d6b1e5cecc1134764f` (the expected head; verified) |
| Frozen recogniser | `08c6faac8c92c6c791ec1e7bbbe997362fa1059f`, tree `be296f2570dcbcc251789b2b1e1bbe2c9e7c2798`; an ancestor of the head; `git diff 08c6faa 05b3e8b -- src-tauri` is empty |
| Recogniser source checksums | `candidate-source.sha256` (279 files) verifies at the head; its own SHA-256 `fc2834d1…` is the one `FREEZE.md` records |
| Evaluation tools | All 11 SHA-256 pins of `FREEZE.md` §5a match the files at the head (`check-set.py` `01a13dde…`, `seal-set.py` `e343202e…`, `run-withheld-set.sh` `8515e92c…`, `score-set.py` `658de955…`, inventory `a9d83ac4…`, tests `8cc2e2d0…`, and the rest) |
| Harness | `src-tauri/tests/real_document_editions.rs` at `08c6faa` (blob `a9f94b13…`), with the reference importer it calls (`scripture/usfm.rs`, `scripture/import.rs`, `scripture/versification.rs`, `scripture/canon.rs`) and the document reader's verse-number rule (`scripture/structure.rs`) |
| Audit branch | `claude/recognition-evaluation-audit-bwxwv2`, started at the same head; this commit adds only this folder |

**Independence.** The private `pistio/collection-h` repository, its
documents, references, manifests and results were not opened, cloned,
fetched or inspected. No earlier withheld collection was accessed. Every
example below is invented text, built for this audit.

**Method.** Each code path was read at the pinned versions, then exercised
three ways:

1. **Comparison probe** (`evidence/compare-probe/`). The harness's
   `squash`, `dense`, `letters` and `difference_kind` (lines 45–105) and its
   class block (lines 712–720) were copied byte for byte from the frozen blob
   (the extracts are committed beside the probe) and run on 38 synthetic
   reference/document pairs.
2. **End-to-end synthetic sets** (`evidence/synthetic/`). Three invented
   sets (A, B, C) went through the pinned `seal-set.py --write`,
   `run-withheld-set.sh` and `score-set.py --run-record=…`, in a scratch Git
   repository outside this one. The harness binary was a release build of
   the frozen source with the frozen toolchain (rustc 1.97.0 2d8144b78).
3. **Control-path probe** (`evidence/synthetic/control-probe/`). The same
   harness run on four of set B's documents with no reference, so that it
   lists every verse it verified.

The local build's SHA-256 is `e396328c…`, not the frozen `920f7e28…`. The
source and toolchain are the same; the build environment evidently is not
(§7). Every behaviour reported here comes from the frozen source and the
pinned scripts. The frozen binary itself is not in this environment.

## 2. Blockers

### B1. Verse zero is never scored (D1; D6)

D1 requires scoring "including verse zero where present". The frozen system
cannot do that, on either side:

* **Reference.** `usfm::parse` reads `\v 0` as a verse labelled `0`.
  `StagedContent::place_verse` (`import.rs:781-802`) accepts only
  `1..=MAX_VERSE` (`reference.rs:27`, 200). It records
  `bibleImport.badVerseNumber` and **drops the verse**. The harness stages
  the reference with `import::stage` and keeps only `reference.content`
  (`real_document_editions.rs:577-582`). The reference's issues, this error
  included, are discarded.
* **Document.** A printed `0` is never a verse-number candidate
  (`structure.rs:2273-2275`). Its words are left out (`documentLeftOut`).
* **Evidence** (set B, `zzaudit-b-exo0a/b/c`). The reference is Exodus 16
  with `\v 0` and verses 1–36. The documents print verse 0 inline as "0 …",
  print its words unnumbered, or omit them. All three give the same result:
  **printed 36, verified 36, exact 36**. EXO 16:0 appears in no output file
  (checked by search). Its words are not attached to 16:1, which stays
  exact.

So EXO 16:0 can be neither printed, verified, wrong nor absent. It is not
reported either (D6). The rule is deterministic but is not the rule D1
states.

### B2. Other primary-numbered reference content is dropped without a trace (D1; D3 "unusual"; D6)

Same mechanism as B1: the reference importer refuses what the 66-book
English canon cannot hold, and the harness discards its errors. A sealed
reference loses, without any report:

| Reference content | Code | Synthetic evidence |
|---|---|---|
| Chapters above the canon's fixed count: JOL 4 in the Hebrew numbering (Joel in four chapters, as some German and Catholic editions number it), DAN 13–14, Vulgate EST 11–16, PSA 151 | `canon.rs:86-109` (JOL 3, MAL 4, EST 10, DAN 12, PSA 150); `import.rs:753-780` `chapterOutOfRange` | `zzaudit-b-jol`: Joel in four chapters on both sides; **printed 52**, not 73; JOL 4's 21 verses never printed; the document's JOL 4 refused |
| Verse labels other than an integer 1–200 or a bridge `N-M`: `0`, `12a`, `201` | `import.rs:781-802` `badVerseNumber` | B1 |
| A second verse with the same number | `import.rs:836-850` `verseTwice` | (code reading) |
| Any book outside the 66: TOB, JDT, ESG, WIS, SIR, BAR, 1MA, 2MA, DAG, PS2… | `canon.rs:159-164`; `import.rs:102-140` (skipped with a note the harness discards) | `zzaudit-b-tob`: Obadiah + Tobit; **printed 21** (Obadiah only); Tobit in no total |

The approved composition (`set-h-decision-package.md` §3, slots 1 and 4)
has two scored documents **with deuterocanonical books**. Under the frozen
rules their deuterocanonical part is never scored and never reported. The
document side refuses the corresponding chapters, so this is a defect of
completeness and reporting, not a safety leak. But "printed" and the
coverage denominator shrink silently, and D1 and D6 are not met as stated.

### B3. Nonexistent references in a chapter the reference lacks go uncounted (criterion 2; D1)

Criterion 2 is "0 verified references their edition does not have". The
harness looks for verified verses the edition lacks in only two places:

* chapters it iterates **from the reference** (`real_document_editions.rs:801-814`);
* books the reference does not hold **at all** (`real_document_editions.rs:821-837`).

A chapter the document has but the reference's book does not falls between
the two. Its verified verses are counted nowhere: not printed, not verified,
not wrong, not absent.

**Evidence** (`zzaudit-b-mal`). The reference has Malachi in three chapters
(3:19–24). The document prints four (4:1–6). The scored result is
**printed 55, verified 49, wrong 0, outside the edition 0**. The control-path
probe on the same document lists **MAL 4:1–6 as verified** (55 verified:
14 + 17 + 18 + 6). Six verified references the edition does not have are
invisible to the gate. The only trace is `"chaptersPrinted": 4` in the raw
report's numbering evidence, which the scorer does not read.

### What would resolve B1–B3 (the owner's decision, not made here)

Either **amend D1/D6** to state the frozen behaviour, and before the seal
declare every reference verse and chapter the frozen importer will discard
(this needs only the sealed references, never a result); or **issue new
evaluation tooling** (a changed harness is a new binary and a new freeze).
B3 cannot be closed by procedure for Markdown, DOCX or PDF documents: their
outputs do not list the verified verses of chapters the reference lacks.

## 3. Material finding F1: what "wrong text" means

**Question asked:** can the typography or Unicode classification let
materially incorrect Scripture text count as a successful verification?
**Yes.** So can the "normalised" class before it, which is never even
listed as a difference.

### The exact code path

1. Reference text: `usfm::parse` removes notes, markers and attribute lists
   (`usfm.rs:140-146`) and collapses white space (`usfm.rs:668-670`).
   `place_verse` stores the text as it is. **No Unicode normalisation**
   happens anywhere in the verse-text path (`nfc()` appears only in
   `concordance.rs`, `parser.rs` and `library/find.rs`).
2. Document text: the recogniser's text, also not normalised.
3. Class (`real_document_editions.rs:712-720`):
   * 0, exact: `want.trim() == text.trim()`;
   * 1, normalised: equal after collapsing white space (`squash`), **or
     after removing all white space** (`dense`), **or equal ignoring letter
     case**;
   * 2, different: anything else.
4. Kind, for class 2 only (`difference_kind`, lines 78–105):
   * `unicode`: NFC of the white-space-free texts is equal;
   * `typography`: `letters()` is equal (lower-cased `char::is_alphanumeric`
     characters only) and the document has no more note signs `*†‡§¶`;
   * then `sign`, `extra`, `missing`, `wrong_reference`, `changed`.
5. Harness counts (lines 744-758). Only class 2 enters `differences`
   (785-799). Class 1 is a bare count (`verified_normalised`,
   `usable_case_only`) with no reference listed.
6. Scorer (`score-set.py:196-210`): `wrong` is the verified differences whose
   kind is not `typography`/`unicode`; `verified_typography` is those that
   are; `verified_correct = verified − wrong`.

So **verified_correct = exact + normalised (white space, case) + typography
+ unicode**. `verified_typography` is a subset of it. The normalised verses
are in it but are not reported by the scorer at all.

**Tests.** `test_a_clean_run_passes_and_counts_only_scored_documents`
(`tests/evaluation/test_withheld_set_tools.py:355-383`) asserts that a
`typography` difference counts in `verified_correct`. Neither
`difference_kind` nor the class block has any unit test; the harness is
`#[ignore]` and holds the only copy.

### What passes as correct (comparison probe, frozen code; `evidence/compare-probe/results.jsonl`)

| Id | Reference | Document | Class / kind | Scorer |
|---|---|---|---|---|
| M01 | set **apart** | set **a part** | 1 normalised | correct |
| M02 | God is **now here** | God is **nowhere** | 1 normalised | correct |
| M03 | **si no** por gracia | **sino** por gracia | 1 normalised | correct |
| M04 | the **Spirit** of God | the **spirit** of God | 1 normalised (case) | correct |
| M05 | the **LORD** | the **Lord** | 1 normalised (case) | correct |
| M06–M08 | **It's** / **He'll** / **We're** | **Its** / **Hell** / **Were** | 2 typography | correct |
| M09 | Truly I tell you**,** today you will be… | Truly I tell you today**,** you will be… | 2 typography | correct |
| M10 | You are the Christ**.** | You are the Christ**?** | 2 typography | correct |
| M11 | **[[**Jesus wept.**]]** (text-critical brackets) | Jesus wept. | 2 typography | correct |
| N01 | el que cree | e◌́l que cree (decomposed: *él*) | 2 typography | correct |
| N02 | e◌́l (decomposed *él*) | el | 2 typography | correct |
| N06 | *ἤ* "or" (decomposed: smooth breathing) | *ἥ* "which" (decomposed: rough breathing) | 2 typography | correct |
| T04 | que élirá (the owner's case, §5) | queélirá | 1 normalised | correct |
| **Caught** | ***él*** (composed) / ***ἤ*** (composed) / forty / 40 / דָּבָר | *el* / ἥ (decomposed) / four / 4 / דָבָר | 2 changed | **wrong** |

Why: `char::is_alphanumeric` is false for the Latin, Greek and Cyrillic
combining accents and breathings (U+0301, U+0313, U+0314, U+0342), for
Hebrew cantillation (U+0596) and for the spacing acute (U+00B4). It is true
for Hebrew points (U+05B0–U+05BC, U+05C1), Arabic vowel signs and U+0345.
Decomposed accents and breathings vanish from `letters()`; Hebrew vowel
points do not. An accent lost between two **composed** texts is caught. It
is missed when the reference is decomposed, or when a decomposed mark
appears in the document over an unaccented reference letter in a verse
with no other composed accent.

**End to end** (set A, `evidence/synthetic/zz-a/`). The same kinds of
deviation sit in an English Ruth, a Spanish Philemon and a decomposed Greek
3 John, beside a control and a mixed document. They went through the pinned
seal, runner and scorer. Result: **verdict pass; 125 printed, 125 verified,
125 correct, 0 wrong**, coverage 100.0 %. Ten of the deviations show as
`verified_typography` (RUT 1:1–5, 1:10, 1:11; PHM 1:2; 3JN 1:1, 1:2). Six
(RUT 1:6–9, PHM 1:1, PHM 1:3) are class 1 and are listed nowhere. The raw
report says only `verified_normalised: 4` and `2`.

**Assessment.** The rule is deterministic, was documented before any result
(`docs/corpus/README.md:97-107`: "differs only in spacing, only in letter
case"; "typography (the same letters and digits; punctuation or quotes
differ)"), and is frozen. The adjudication ledger can only explain it,
never change it, so it is no D7 breach. But a *pass* with "0 verified
verses with wrong text" does **not** exclude:

* word-boundary errors;
* case errors such as LORD/Lord or Spirit/spirit;
* apostrophe, question-mark, comma or bracket errors;
* lost decomposed accents.

The documented wording does not say that "spacing" includes merging or
splitting words, or that "letters" excludes decomposed marks. The owner
must either accept this definition explicitly, with the accuracy claim
qualified, or replace it (a new freeze). In the project's own fourth
rehearsal `verified_typography` was 0. Its class 1 count is not visible in
any committed score.

## 4. Conditions (procedural; no code change; required whatever is decided above)

| | Condition | Why (code) | Evidence |
|---|---|---|---|
| C1 | The assessor runs exactly the command of `SET-H-APPROVAL.md` §9.2, with no `--frozen-binary` and no `--inventory`. The owner checks that `score.json` `run.binary` is `920f7e28…` and `run.inventory` is `a9d83ac4…` | Both options override run validity after the run (`score-set.py:473-474`, used at 303-305). The score records neither the option nor its value (`anchors` holds only commits and the scorer and check-set hashes) | Set A: default *not a protocol run*, `--frozen-binary` → *pass*; set C: default *not a protocol run*, `--inventory=<rehearsal inventory>` → `canonical_inventory: yes` |
| C2 | Before the run, the sealed manifest is confirmed to hold D5's 23 scored / 3 control / 2 mixed / 0 exploratory and 16 authentic / 12 generated | The tools fix each document's class at the seal and read it from the seal commit (`score-set.py:447-459`), but enforce no numbers. `CHECK.json` `classes` gives the class counts; kinds must be counted from the manifest | (code reading) |
| C3 | No scored row whose `books` is empty or `all` has a first file named `…_<book code>.<ext>`, case-insensitive: `_gen`, `_rev`, `_job`, `_act`, `_num`, `_mat`… | `real_document_editions.rs:642-657` then measures the document against that one book only | `zzaudit-b-gen` (`probe_gen.md`, Ruth): printed 0, **22 false "outside the edition"** |
| C4 | No sealed reference archive holds a file whose name ends in `versemap.xml` and contains `<verseMap`, and every reference is USFM, not OSHB OSIS or MorphGNT | `import.rs:1677-1681` loads it; `release_held` (`import.rs:886-`) **renumbers the reference** silently, breaching D2 | `zzaudit-b-map`: the same Joel reference plus a map; printed **73** instead of 52, the reference's JOL 3 now holding the Hebrew 4's words |
| C5 | If F1 is accepted as frozen: every sealed reference USFM is in NFC (`unicodedata.is_normalized("NFC", text)`) | A decomposed reference lets accent and breathing changes pass as `typography` (N02, N06) | compare probe |

## 5. The unusual cases, one by one

For each case below: the behaviour as implemented, where it lives in the
source, the synthetic evidence, and whether D1–D7 leave any discretionary
choice. "None" means none at assessment: the outcome is fixed by the frozen
code and the sealed bytes.

**Primary verse zero, EXO 16:0**

* Behaviour: dropped from the reference (`badVerseNumber`); never a verse
  marker in the document; its words left out; never printed, verified,
  wrong or absent.
* Source: `import.rs:781-802`; `structure.rs:2273`; `real_document_editions.rs:577-582`.
* Evidence: `zzaudit-b-exo0a/b/c`: printed 36, verified 36, exact 36.
* Discretion: none, but contrary to D1 (**B1**).

**Empty reference verse** (`\v 9` with no text, or only a note)

* Behaviour: `absent_in_edition`, not printed. A verified document verse
  with text there is `verified_absent` (criterion 2).
* Source: `import.rs:851-865` (kept empty, warning);
  `real_document_editions.rs:694-697`, 801-814.
* Evidence: `zzaudit-b-jud` (absent_in_edition 1); `zzaudit-b-jud2`:
  JUD 1:9 verified → **outside the edition**.
* Discretion: none. D3 met.

**Placeholder text** (`—`, `[Omitted in the earliest copies]`)

* Behaviour: ordinary printed text, compared as published.
* Source: as for any verse.
* Evidence: `zzaudit-b-jud`: JUD 1:12 and 1:14 verified exact.
* Discretion: none. D3 met. (Probe P01: a punctuation-only placeholder
  against an empty document verse would class as `typography`, which is
  theoretical.)

**Merged verses** (`\v 4-5`)

* Behaviour: the text is stored on 4 (bridge 4→5), and 5 has no entry. A
  document's separate 5 would be `verified_absent`; its 4 alone would be
  `missing`.
* Source: `import.rs:781-834`, 874-881; `usfm.rs:706-710` (test).
* Evidence: `zzaudit-b-jud`: "4-5" verified on 4, exact.
* Discretion: none. D3 met.

**Absent verse number** (no `\v 20`)

* Behaviour: not printed. A document's verified 20 is `verified_absent`;
  the verse before the gap is held back as `boundary`.
* Source: `real_document_editions.rs:801-814`.
* Evidence: `zzaudit-b-jud`: JUD 8 and 19 unverified `boundary`;
  `zzaudit-b-jud2`: JUD 1:20 → outside the edition.
* Discretion: none. D3 met.

**Alternate numbering** (`\va`, `\vp`, `\ca`; `\cp`)

* Behaviour: their content is dropped and the primary `\c`/`\v` used.
  Never substituted. The document's printed departures are recorded in the
  raw report (`numbering`), which the scorer does not read.
* Source: `usfm.rs:206` (`cp`), 219-221; `numbering::assess` via
  `real_document_editions.rs:583`.
* Evidence: `zzaudit-b-jud`: JUD 21 with `\va 22\va* \vp 21a\vp*` verified
  exact under 21.
* Discretion: none. D2 met, provided C4 holds.

**Vulgate and other numbering systems**

* Behaviour: scored under the edition's own primary numbers while within
  the canon's chapter counts and verses 1–200. Beyond them, dropped (B2). A
  verse map in the reference renumbers it (C4).
* Source: `canon.rs:86-109`; `import.rs:753-780`, 1155-1178, 1677-1681.
* Evidence: `zzaudit-b-jol`, `zzaudit-b-mal`, `zzaudit-b-map`.
* Discretion: none at assessment, but **B2**, **B3** and **C4**.

**Punctuation and Unicode differences**

* Behaviour: see F1. Canonical equivalence (NFC) is genuinely the same
  text. White space, case, punctuation, symbols and decomposed marks are
  all counted correct.
* Source: `real_document_editions.rs:45-105`, 712-758;
  `score-set.py:196-210`.
* Evidence: compare probe; set A.
* Discretion: none, but **F1**.

**USFM opening character-marker delimiter** (the space after `\add`)

* Behaviour: implemented. An opening marker swallows exactly one following
  space; a closing marker (`\add*`) swallows none.
* Source: `usfm.rs:144-146` (marker regex), 302-499.
* Evidence: `zzaudit-b-add`: the reference `que \add*él\add irá` is read as
  **"Y dijo que élirá a la ciudad al amanecer."** (the harness's own
  `edition` text).
* Discretion: none.

**The owner's case:** `que \add*él\add irá` (read "que élirá") against
"queélirá"

* Behaviour: class 1, *normalised* (`dense` equal). Counted
  `verified_correct`, listed in no difference. "que élirá", "queélirá" and
  "que él irá" all score identically. The reference stays as published.
* Source: `real_document_editions.rs:712-720`.
* Evidence: compare probe T04 and T05; set A, PHM 1:3 (verified,
  normalised).
* Discretion: none. D3 met; the outcome is fixed by the white-space rule
  (F1).

**Controls**

* Behaviour: every verified reference is a failure (criterion 3).
* Source: `real_document_editions.rs:451-517`; `score-set.py:316`, 321.
* Evidence: set C: a control holding Obadiah → 21 verified → **fail**; set
  A: a plain control → 0.
* Discretion: none. D4 met.

**Mixed documents, allowed lists**

* Behaviour: the list is read from the seal commit. `BOOK C`, `BOOK C:V`
  and `BOOK C:V-W` are accepted, with `#` comments; any other line refuses
  the seal. Every verified reference outside the list is a failure.
* Source: `check-set.py:64`, 183-207; `score-set.py:230-232`, 270-278, 317.
* Evidence: set C: Ruth 1–2 against `RUT 1:1-21` and `RUT 1:22`: 22
  allowed, **23 outside** (RUT 2:1–23) → **fail**.
* Discretion: none. D4 met.

**Document failures**

* Behaviour: a non-zero exit, a timeout, memory, abort, kill, panic, no
  output or no RUN.txt line is a *failure* (criterion 4 → fail). A report
  left behind is kept as `diagnostic`, never counted.
* Source: `score-set.py:180-193`, 255-269, 322; runner 80-90.
* Evidence: existing tests `test_failures_are_never_completions` and
  `test_a_timeout_is_a_failure_and_not_a_protocol_run`; the project's
  forced-failure and memory rehearsals.
* Discretion: none. Met.

**Clean refusals**

* Behaviour: exit 0 with the app's refusal of the whole document. Not a
  failure. Named beside the coverage figure and never counted as coverage.
* Source: `real_document_editions.rs:553-571`; `score-set.py:259-260`,
  353-362.
* Evidence: `zzaudit-b-empty` (empty text file): *refused*, named in the
  claim.
* Discretion: none. Met.

**Incomplete runs**

* Behaviour: no `ended` line or a missing document line → *not a protocol
  run*. If the runner dies before committing its record, assessment is
  refused, and the set cannot be re-run from that folder (`.run-started`).
* Source: `score-set.py:297-306`, 109-127; runner 61-62, 92-99.
* Evidence: existing tests in classes `ScoringAnchors` and `RunnerRefuses`.
* Discretion: none in the code. A hand-committed partial record can only
  yield *not a protocol run*.

**Nonexistent references**

* Behaviour: counted for a reference chapter's extra verses and for books
  the reference lacks. **Not** for a chapter the reference's book lacks.
* Source: `real_document_editions.rs:801-837`.
* Evidence: `zzaudit-b-jud2` (2 counted), `zzaudit-b-gen` (22 counted),
  `zzaudit-b-mal` (**6 uncounted**).
* Discretion: none, but **B3**.

**`verified_wrong`, `verified_typography`, `verified_correct`**

* Behaviour: wrong is verified class 2 of kind extra, sign, missing,
  wrong_reference or changed. Typography is verified class 2 of kind
  typography or unicode. Correct = verified − wrong, so it includes
  typography and the unlisted class 1. Absent is separate and never inside
  `verified`.
* Source: `score-set.py:196-210`; `real_document_editions.rs:744-758`, 846.
* Evidence: set A; set B `zzaudit-b-wrong` (forty→four) and
  `zzaudit-b-accent` (composed él→el) each **1 wrong**.
* Discretion: none, but **F1**.

## 6. D1–D7, rule by rule

| Rule | Deterministic, fixed before results? | Implemented as stated? |
|---|---|---|
| D1 primary numbering incl. verse zero | Yes | **No**: B1 (verse zero), B2 (content above canon counts, odd labels, books outside the 66), B3 (chapters the reference lacks) |
| D2 alternate numbering recorded, never substituted | Yes | Yes for USFM references without a verse map; **C4** |
| D3 empty, placeholder, merged, unusual as published; nothing excluded after execution | Yes | Empty, placeholder, merged, absent: yes. "Unusual": **no** for verse 0, odd labels, out-of-range chapters, deuterocanon (B2). Nothing is excluded after execution: yes (the scorer has no exclusion path; `verified_extra_verses` is never truncated, lines 845–869) |
| D4 controls none; mixed only allowed | Yes | Yes |
| D5 classes fixed (23/3/2/0; 16/12) | Yes: fixed at the seal, read from the seal commit | The numbers are not enforced by the tools: **C2** |
| D6 limitations reported, never used to change scoring | Yes | The scorer's qualification limit never changes the verdict: yes. Reference-side discards (B1, B2) and class 1 normalisations (F1) are **not reported** |
| D7 no exemptions or discretionary choices | Yes, apart from two scorer options closed by the approved command (**C1**); the adjudication ledger is beside the raw score, never instead of it | Yes, with C1 |

**Separate treatment of outcomes.** Correct verification, wrong text,
clean refusals, execution failures and coverage are kept apart as the
protocol says, and the per-document execution layer never feeds the
accuracy figures. Nonexistent references are kept apart too, **except for
B3**. Coverage is `verified_correct ÷ printed` for completed scored
documents. Its numerator includes F1's equivalences; its denominator omits
B1 and B2's discards.

## 7. Observations (no action required for the verdict)

1. **Binary reproducibility.** The local release build of
   `real_document_editions` from the verified frozen source gave
   `e396328c20fef35ee12f62b880e554f676a93006ca5bc8054dc82d433f21ee43`;
   `FREEZE.md` records `920f7e28…` ("built twice … identical"). `FREEZE.md` records
   `cargo test --release --locked`; this audit built only the one target
   (`cargo test --release --locked --test real_document_editions --no-run`
   in `src-tauri/`), and the frozen build's path is not recorded. The runner and scorer would refuse or disqualify this build,
   as they should. The owner may wish the frozen build's environment to be
   recorded.
2. **Untested code.** `difference_kind`, the class block and verse zero
   have no tests. The behaviours in §3 and §5 rest on code reading plus
   this audit's probes.
3. **Fails closed.** `check-set.py` and the runner's `read` trim manifest
   cells, but the harness uses them untrimmed
   (`real_document_editions.rs:163-171`). A key or reference with
   surrounding spaces passes the check and then fails as "no output". A
   failure, not a false pass.
4. **By design.** A document refused whole passes the gate, and coverage
   has no pass mark. This is how `set-h-decision-package.md` §5 states it.

## 8. Checks run

| Check | Result |
|---|---|
| Branch head, frozen commit ancestry, `git diff 08c6faa 05b3e8b -- src-tauri` | head `05b3e8b…` as expected; ancestor; empty |
| `sha256sum -c docs/audit/candidate-a1.5/candidate-source.sha256` | 279 files OK |
| `FREEZE.md` §5a pins (11 files) | all match |
| `python3 -m unittest discover -s tests/evaluation -p 'test_*.py'` | 46 passed |
| `python3 -m unittest discover -s tests/corpus -p 'test_*.py'` | 26 passed |
| `cargo fmt -- --check` (src-tauri) | clean |
| `cargo test --locked --lib -- scripture::` (frozen source, debug) | 100 passed, 0 failed |
| `cargo test --release --locked --test real_document_editions --no-run` | built; SHA-256 `e396328c…` (§7.1) |
| Comparison probe, frozen lines 45–105 and 712–720 (extract SHA-256 `d527532c…`, `16f43870…`) | 38 pairs, results in `evidence/compare-probe/results.jsonl` |
| Sets A, B, C through the pinned `seal-set.py --write`, `run-withheld-set.sh`, `score-set.py` | A pass (with C1's override); B fail as designed; C fail as designed; outputs verify against their committed `OUTPUTS.sha256` |
| Control-path probe (MAL, JUD, EXO, TOB documents with no reference) | MAL 4:1–6 verified (B3) |
| Working tree after all builds and runs | clean (build output only in the ignored `src-tauri/target/`) |

Not run: the interface checks (`npm run typecheck`, `lint`, `i18n:check`,
`test`, `build`), `cargo clippy` and the full `cargo test --release`. This
commit adds documentation and evidence only, and the recogniser source is
unchanged and verified by its checksums. To build the harness, the GTK and
WebKit development packages named in `ci.yml` were installed in this
container; nothing in the repository depends on that.

## 9. Boundaries kept

No recogniser source, evaluation script, reference semantics, existing test,
expected outcome, manifest, seal or frozen binary was changed. Collection H
was not accessed, run or sealed. The synthetic sets were sealed and run only
in a scratch repository outside this one, whose commits are listed in
`evidence/synthetic/scratch-repo-log.txt`. Nothing was released, merged or
tagged. This audit stops at owner review.

## Evidence index

`evidence/README.md` lists every file, how it was produced, and how to
reproduce it.
