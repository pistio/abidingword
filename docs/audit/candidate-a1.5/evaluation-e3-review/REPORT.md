# Independent review of evaluation candidate A1.5-E3

**Verdict: E3 INDEPENDENTLY VERIFIED — READY FOR OWNER PROTOCOL FINALISATION**

R1 and R2, the two blockers of the E2 review, are closed. I confirmed this
with my own material: my own USFM extractor, my own vectors and fuzzer, my
own sets in my own lab, a harness binary I rebuilt myself, and the E2 tools
and binary rebuilt for comparison. No new blocker was found. Six non-blocking
findings (§7) and the owner's decisions (§8) remain. None of the findings can
make an official verdict wrongly favourable on references built from eBible
USFM, the only approved source for Collection H.

Status: an independent review record for the owner. It approves nothing and
authorises nothing: no protocol change, no seal, no withheld run, no merge,
release or installer. I am not the E3 implementer, and this record approves
no work of mine.

## 0. In plain words

* **The reference text is now right.** Two E3 readers (Rust and Python) and
  a third extractor I wrote myself from the USFM 3 specification agree on
  the text of all **311 761** verses of the 13 bundled references, and on all
  **528** verses the E2 review found mis-spaced. For those 528 I also checked
  the raw source directly: at each of their 856 notes, the characters on
  either side of the note are joined exactly as the source joins them.
* **No valid text is lost or changed silently.** My 99 hand-written
  vectors, including malformed and Unicode cases, and 100 000 random books
  from my own generator found no silent loss. On every input E3 reads
  without a problem, my extractor reads the same text. Both E3 readers
  were identical on every input. One kind of **invalid** markup is not
  refused: a marker that is not a milestone, written in milestone form
  (`\c \*`, `\p\*`). It cannot occur in eBible USFM: the 716 USFM files of
  the 13 references contain no `\*` at all. Finding N1.
* **On real text the measurement is clean.** I made a whole Berean Standard
  Bible document from the edition's own words and ran it through the
  rebuilt pinned binary: **0** wrong-text verses among 30 977 verified (E2's
  binary: 154, all of them the reference reader's). Through the official
  path it scored **PASS**. The SBL Greek New Testament keeps exactly **1**:
  ACT 28:28, a genuine recogniser error (E2: 335).
* **A pass now needs evidence, exactly as specified.** Vacuous references are
  refused at the seal. A set that verifies nothing gets **INCONCLUSIVE**: zero
  verified, every scored document refused whole, no recognisable Scripture,
  or only unsupported content. I could not obtain *pass* from any vacuous
  set. One faithful verse among 31 146 printed gives an official **PASS**. That
  is the protocol's stated default floor, and the verdict line shows the
  counts. Whether that floor is enough is the owner's decision.
* **The tools defend themselves.** Each of these gives *not a protocol run*:
  a modified scorer or reader, another binary, a fake harness that changes
  one verse's text digest, or a scorer on another Unicode version. Unknown
  options are refused. Two limits, both expected: a scorer and protocol
  rewritten together are caught only by the owner's own record of the
  protocol's SHA-256. The runner's rehearsal variable `AW_PROTOCOL` lets the
  documents be *executed* past the shut gate, though the result can never be
  official. Finding N2.
* **The pinned binary reproduces.** I rebuilt it in a fresh container at the
  pinned path and got `b4f51a62…`, identical to the pin. At another path it
  is `8cab9ddd…`. E2's binary `800fb43c…` also reproduced exactly.

## 1. What was reviewed, and how

| | |
|---|---|
| Reviewed branch and head | `claude/a1-5-e3-evaluation-corrections-jtt5cw` at `9640aea303c625c137c5cdbe1f4b8cc9738951fe` (the expected head; verified) |
| E3 implementation commit | `816e643fc3122f69d41710781575abe480922a09` (tree `2b982f0c…`); `9640aea` adds only the record, checksums and checks summary |
| Frozen recogniser | `08c6faac8c92c6c791ec1e7bbbe997362fa1059f`; `git diff 08c6faa 816e643` and `08c6faa HEAD` over `src-tauri/src`, `Cargo.toml`, `Cargo.lock`, `build.rs`: **empty**; `src-tauri/src` tree `8cb224c1…` in all three; `recogniser-source.sha256` 166/166 |
| Protocol | `docs/evaluation/protocol-e3.json` SHA-256 **`43f0468941af9ac04149318ea885f1a49c1f298d97a951132b12ea1e31a41937`**; gate `withheld_execution_permitted: false`; `required_composition: null`; floor `minimum_verified_faithful: 1`, three slots `null` |
| Tool hashes | all 8 `approved_tools` and all 4 `harness_build.sources_sha256` entries equal `sha256sum` of the files (`evidence/identity/`) |
| History | `git diff 7772d0b HEAD` touches none of FREEZE, checksums, SET-H-APPROVAL, decision package, regression, data, rehearsal, reviews, known weaknesses, evaluation audit, E2, E2 review, `docs/corpus`, `protocol-e2.json`, `src-tauri/src` |
| Review branch | `claude/a1-5-e3-independent-review-jtt5cw`, started at `9640aea`; its commit adds only this folder |

**Independence.** I did not read the E3 record's evidence drivers as a
model. I did not reuse any E3 vector, rehearsal set, fuzzer or test as test
material. The E3 tools, readers and binary were run only as the system under
test. The Rust reader was run through a probe crate that includes
`usfm_text.rs` read-only (`evidence/probe/`). From the E2 review I used only
data: the list of the 528 verses and their context windows. My extractor was
written without reading the E2 reviewer's extractor. Its whole-edition
documents came out byte-identical to the E2 reviewer's (§6), which is a
cross-check, not a copy. `pistio/collection-h` and every withheld document,
manifest, reference or result were not accessed. Nothing was sealed or run on
a withheld set. Every protocol revision, modified tool and fake harness lived
only in scratch repositories outside the project.

**Environment.** This session began in the owner-approved Collection H
environment: Limited network, `ebible.org` only, package managers off. The
Rust build and the npm checks need crates.io, npm and the Ubuntu archive. With
the owner's approval in the app, the session's network was moved to a new
personal environment, `abidingword review` (package managers on,
`ebible.org`). **The Collection H environment was not changed.** `ebible.org`
was reachable, but I downloaded nothing from it, because it is Collection H's
only approved source (§10).

## 2. Verdict and reasons

**E3 INDEPENDENTLY VERIFIED — READY FOR OWNER PROTOCOL FINALISATION.**

* R1 (reference text) is closed: §3.
* R2 (pass needs evidence) is closed and works exactly as specified: §4.
* The bounded findings of the E2 review are corrected, and integrity holds
  under attack: §5.
* The E2 failures reproduce under the E2 tools and binary, and the E3
  corrections hold on the same material and on real text: §6.
* The non-blocking findings N1–N6 (§7) do not stand in the way of
  finalising the protocol. Each comes with the owner action, if any, that
  makes it moot for Collection H without a new review cycle.
* The decisions only the owner can take are set out in §8, with my
  recommendations.

## 3. Task 1 — R1, the reference text

**Rules.** I read `usfm-text-rules.md` against the USFM 3 specification:
white space, notes, character spans and `+` nesting, attributes and default
attributes, milestones, `\va`/`\vp`/`\ca`, `~` and `//`, sidebars, tables,
chapter and verse markers. The rules are faithful to the specification
where it is clear, and they state their choices where it is not. Notes are
removed with nothing put in their place. Only one white space character
after an opening marker belongs to the marker. USFM white space is U+0020,
tab and line feed only. Every structural boundary closes what is open and
reports it. The two readers share no code with each other or with
`src-tauri/src/scripture/usfm.rs`. The Rust reader is a streaming state
machine; the Python reader is a lexer, a tree and a walk.

**Real references** (`evidence/r1/bundled-three-readers.json`). All 13
bundled references: 716 USFM members, **311 761 verses**. Labels and text are
identical in the E3 Python reader, the E3 Rust reader and my extractor. Both
E3 readers report 0 problems, and my extractor reports 0 trouble. The four
verses with a printed no-break space (engwebp JHN 5:11, ACT 20:35, ACT
22:21; porbr2018 ZEC 7:13) keep it in all three readers. USFM white space
does not include U+00A0, and the fidelity rule treats it as layout.

**The 528** (`evidence/r1/the-528.json`). I checked each verse against three
expectations that do not come from either E3 reader:

1. **The source itself.** At each of the **856** notes in these verses, the
   visible characters before and after the note must be adjacent in the
   verse text exactly as in the source.
2. **The E2 reviewer's committed context window.** It must occur in the
   text, and the app reader's spaced window must not.
3. **My extractor.**

All 528 pass all three, and the Rust reader agrees on all 528. By
edition: engbsb 156, engwebp 14, fraLSG 2, grcsbl 356.

**Bundled-reference tests.** `evaluation_references`: 4 passed, 1 ignored,
a dump helper. `test_e3_rules.BundledReferences`: 3 passed (§9).

**My vectors** (`evidence/r1/review-vectors.json`, results beside it). I wrote
99 vectors, each with an expectation written by hand from the
specification before running anything:

* notes: 14, including footnotes, cross references, endnotes, `\ef`/`\ex`,
  note parts, `+` spans and bars inside notes, a note beside a combining mark,
  and two adjacent notes;
* white space: 13, including NBSP, NNBSP, thin and ideographic spaces, `~`,
  `//`, tabs, CRLF, a marker followed by two spaces, glued spans, invisible
  format characters, BOM, paragraph and table boundaries;
* milestones and attributes: 8, including `\qt-s/e`, `\ts\*`, `\zaln`, `\w`
  with attributes, ruby, `\jmp`, `\fig` in both syntaxes;
* numbering: 10, including bridges, `\va`, `\vp`, `\ca`, `\cp`, `\cl` and
  `\d`, `\rq`, labels as written, duplicates, and a title numbered as verse 1;
* Unicode: 9, including German NFD, Korean syllables and jamo, polytonic
  Greek NFD, Hebrew with cantillation, RLM and maqaf, Persian ZWNJ, Syriac,
  Ethiopic, Devanagari, Chinese, Thai, astral characters, compatibility
  forms, U+2028/2029, and C1 NEL;
* identity and accounting: 6;
* malformed: 36, including mismatched and unclosed spans and notes, an
  unclosed `\x` before a later `\f`, a verse inside a note, `\wj` across a
  poetry line, unterminated and unpaired milestones, interrupted attributes,
  bars, unknown and `\z` markers, a lone backslash, text before verse 1, two
  `\id`, control characters, sidebars, `\v\*`, `\+v`, `\v1`, and seven
  markers written in milestone form.

Results:

* The two readers are **identical on all 99**.
* Of the 97 with an expectation, **90 meet it**.
* The **7 that do not** are all one class: non-milestone markers in
  milestone form (N1).
* My first expectation for a table header placed before verse 1 was wrong.
  It is text outside any verse, which the rules refuse, conservatively. The
  vector is kept as a refusal and noted in its file.
* Every malformed vector with a structural error is refused, and every `\v`
  still becomes a verse. The verses after the error read exactly; for
  example, an unclosed `\va` in RUT 1:1 still yields 1:2 and 2:1 intact.

**Fuzzing** (`evidence/r1/fuzz-review-100k.json`). I generated 100 000 books
from my own grammar of well-formed and malformed fragments: notes, spans,
milestones, attributes, headings, numbering, stray markers and seven
scripts. Results:

* The Rust and Python readings were identical on **100 000 of 100 000**
  (verses, lines, problem codes).
* 80 136 books were refused and 19 864 read cleanly.
* Among the clean ones, **0** silent losses: the `\v` count always matched.
* My extractor read the same text on all 18 773 clean inputs where it had no
  trouble of its own, and never a different text.
* Where it did report trouble and E3 did not, the cause was:
  * a marker in milestone form: 1 066 inputs (N1);
  * a bar in a heading, which is not verse text: 27;
  * sidebars, which my extractor does not implement: 2.

**Choices that are not refusals** (stated, judged acceptable):

* A `\d` title placed *after* `\v 1` is dropped like any heading. A title
  *with* `\v 1` inside it is read as verse 1. None of the 13 references uses
  either pattern. If an edition does, the effect is a false mismatch, which
  is fail-safe.
* A no-break space glued to a verse number (`\v 1 a`) makes an unusual
  label. It is reported as unsupported, not silently lost.

**Conclusion.** No valid source text was lost or changed silently in any
test. Ambiguous or malformed markup is refused, with every verse kept, in
every class tested except N1.

## 4. Task 2 — R2, a pass must contain evidence

All results below are official scores in my lab
(`evidence/lab/scenarios-e3.json`), produced with the real pinned binary
under a simulated owner revision (composition set, gate open, scratch
repository only).

| Attempt | Result |
|---|---|
| Empty reference zip | **refused at the seal** (`no_verses`) |
| Prose-only reference (`copr.htm` alone) | **refused at the seal** |
| Reference printing only Tobit | **refused at the seal** (`unsupported_only`) |
| Reference with only verse 0 and a chapter beyond Ruth | **refused at the seal** |
| Reference whose verses are all empty | **refused at the seal** (`numbered_but_empty`) |
| Declared books RUT,JON; reference prints only Ruth | **refused at the seal** (declared book not printed) |
| Two scored prose documents, sound references | **INCONCLUSIVE**: verified 0 of 133; 133 missing |
| Every scored document refused whole (unreadable PDFs) | **INCONCLUSIVE**: 0 completed, 2 refused whole |
| Valid reference, Ruth's words as unnumbered prose | **INCONCLUSIVE**: verified 0 of 85 |
| Document prints only verse 0 (reference also has 1:1) | **INCONCLUSIVE**; 1 printed unsupported |
| Scored prose + exploratory Jonah (47 faithful) | **INCONCLUSIVE**: exploratory evidence not counted |
| **One faithful verse in a very large collection**: Psalm 117 in prose, against the whole WEB (66 books), plus a second scored document verifying nothing | **PASS**: "verified 1 (1 faithful, 0 wrong text) of 31146 printed scorable verses; … 1 held back; … 31144 missing (not found); scored documents 2 completed …"; qualification limit stated; `documents_verifying_nothing: ps-b` |
| A real wrong word | **FAIL** (safety fail; evidence sufficient) |
| Faithful documents | **PASS** (132 of 133) |

The frozen recogniser verifies whole chapters only. 3, 5, 8 and 12 of Ruth
1's 22 verses verified nothing, and all 22 verified 22. So one verse is the
smallest unit it can produce, reached here because the second verse of
Psalm 117 was held back.

**The floor, parsed directly** (`evidence/r2/floor-unit.json`). I gave the
scorer's `evidence_floor()` 21 protocol variants. None of these allows a
pass: a missing floor, a floor that is not an object, 0, −1, `"1"`, `1.0`,
`true`, a missing base, an unknown (misspelt) measure, a string share, NaN,
or −∞. The document-spread, share and refused-whole slots behave as named.
The floor works **exactly as specified**: pass requires criteria 1–4 *and*
at least one faithful verse verified in a completed scored document;
otherwise the verdict is inconclusive, never pass.

**Presentation.** The safety verdict, evidence verdict, counts, coverage
and qualification limit are reported apart, and the verdict line always
carries the counts. No claim is false. Two wordings can be read as more than
they are (N3):

* With one faithful verse of 31 146, the line says "**evidence
  sufficient**". That is sufficient against the floor, which is a
  non-vacuity floor.
* The accuracy claim prints coverage "**0.0 %**" for 1/31 146 (0.003 %).

I did not introduce a coverage threshold. The choice of an additional floor
is the owner's (§8.3).

## 5. Task 3 — integrity and protocol

Results are in `evidence/lab/probes-e3.json`, `scenarios-e3.json` and
`deutero.json`.

| Probe | Result |
|---|---|
| Frozen recogniser source; tool, harness-source and protocol hashes | unchanged; all match (§1) |
| **Verse-by-verse comparison of the two readers by the scorer** | Real whole BSB and SBLGNT: no difference, no integrity issue (§6). A fake harness (the real binary, then **one** verse digest changed in its report), pinned in a simulated revision: **not a protocol run**, "1 verse(s) read differently … RUT 1:4" |
| Modified scorer committed (criterion 1 forced true), failing run | **not a protocol run** (`tools_are_the_approved_ones`) |
| Modified `usfm_text.py` committed before a run | **not a protocol run** (`run_used_the_approved_tools`, `tools_are_the_approved_ones`) |
| Modified scorer *and* protocol rewritten to approve it, committed **after** the run | **not a protocol run** (`run_under_this_protocol`) |
| … committed **before** the run (a consistent substitute toolset) | **official PASS on a failing run**, protocol SHA-256 `cd1bf718…` ≠ `43f04689…`. Expected and documented (`OWNER-DECISIONS.md` §4): only the owner's external record of the approved protocol hash catches it. The score prints the hash and says to check it |
| Unknown options (`--Protocol=`, `--Inventory=`, `--writ`, `--diagnostic=yes`, a valueless `--json`, a bare `--`) | refused, exit 2, every tool |
| Repeated `--name=value` options | refused, exit 2. **Repeated flags** (`--write --write`, `--diagnostic --diagnostic`) are accepted: harmless, but not "every repeated option" (N6) |
| `--protocol` without `--diagnostic` | refused |
| Outputs changed after the run | scorer refuses |
| Raw publisher archive (with `keys.asc`, `.css`) as the package | refused at the seal |
| `reference-sources.tsv`: wrong member hash / removed member present / kept row missing | each refused at the seal |
| A code point unassigned in Unicode 15.1 (U+16D40) in a reference | refused at the seal |
| … in a document's verified verse | **not a protocol run** (fidelity disagreement; unassigned code point distrusted) |
| Scorer run under Python 3.12 (Unicode 15.0) | **not a protocol run** (`unicode_versions_as_protocol`) |
| Harness Unicode version | `unicode-normalization` 0.1.25 `UNICODE_VERSION = (17, 0, 0)`; reports state 17.0.0 |
| Gate shut (committed protocol) | runner refuses before reading anything |
| Another binary (same source, other path), own hash / claiming the pin | runner refuses / runner refuses |
| **Gate shut, runner given `AW_PROTOCOL=<open copy>`** | **the runner runs the documents** (`.run-started` written, reports produced). The score is **not a protocol run** (`protocol_permits_withheld_execution`, `run_under_this_protocol`). See N2 |
| Binary reproducibility | pinned path, fresh target, new container: **`b4f51a62…` identical**. Other path: `8cab9ddd…` (the path is embedded once; 808 410 bytes differ). E2's `800fb43c…` rebuilt: identical |

## 6. Task 4 — regression and real evidence

**E2 failures and E3 corrections on my own sets.** I ran the same five sets,
officially, through E2's tools (`git archive 7772d0b`) with E2's binary,
which I rebuilt (`800fb43c…`), and through E3's (`evidence/lab/regress-*.json`).

| Set (invented text) | E2 | E3 |
|---|---|---|
| Notes against punctuation, faithful document | **FAIL**: 21 false mismatches (`punctuation_spacing`) | **PASS** 21/21 |
| An unclosed `\x` before a later `\f` | **FAIL**: printed 22 of 25 (3 verses vanished), 3 "outside the edition", 1 false mismatch, nothing flagged | **refused at the seal** (`note_unclosed`, line 8) |
| A USFM 3 milestone | **FAIL**: 1 false mismatch | **PASS** 25/25 |
| Vacuous (prose-only package, Tobit-only reference) | **PASS**, 0 verified | **refused at the seal** |
| Zero coverage, sound reference | **PASS**, 0 verified | **INCONCLUSIVE** |

**Real references, generated documents** (`evidence/real/`). These are
Markdown documents made from the editions' own words by my extractor. They
are **generated, not authentic published PDF, EPUB or DOCX files.** My
documents are byte-identical to the E2 reviewer's (SHA-256 `2153c588…`,
`3acba7ce…`), from extractors written apart. I ran them as the runner runs a
document, through each binary.

| Document | Printed | Verified | E2 binary: wrong text (reader / document) | E3 binary: wrong text | E3 scorer integrity |
|---|---:|---:|---:|---:|---|
| Berean Standard Bible, whole | 31 086 | 30 977 | 154 (154 / 0) | **0** | none |
| SBL Greek New Testament | 7 939 | 7 671 | 335 (334 / 1) | **1**: ACT 28:28 | none |

* The scorer's own `observe_report()` ran with the check's inventory and
  per-verse records: Rust and Python agree on every verse of both. My
  extractor's text matches every reference verse digest (31 086 and 7 939).
* ACT 28:28 is the recogniser's reading, not the reference's. The document
  prints the edition's text. The recogniser appended verses 30–31 and a
  stray `‘/book’` to verse 28; the SBLGNT has no verse 29.
* **Official path:** the whole BSB, less 2 John (its USFM member is a
  disclosed book, which the check refuses), gave **PASS**: 30 964 faithful,
  0 wrong text of 31 073, every validity condition met
  (`real/real-bsb-official.score.md`).
* Recognition counts are identical under E2 and E3 (30 977 and 7 671 verified
  under both): only the measurement changed.
* The app importer's comparison still shows **156** (BSB) and **356**
  (SBLGNT) verses differing. That is the product defect, outside this
  evaluation (`OWNER-DECISIONS.md` §7), confirmed.

**Not reproduced, and why.** No authentic published document was used: none
is in the repository, and I chose not to download from `ebible.org` (§10).
The 113-edition disclosed corpus is unavailable; nothing about it is
estimated, and the historical A1.5 figures stand as published.

## 7. Findings (non-blocking)

**N1. A non-milestone marker written in milestone form is silently ignored
(both readers; low).** This covers `\c \*`, `\c |n="2"\*`, `\p\*`, `\q2\*`,
`\id\*`, `\f\*` and `\nd\*`. Effects:

* the chapter boundary disappears: the next chapter's verses join the
  previous chapter, as duplicates (reported as unsupported) or, if
  numbering continues, as silently renumbered verses;
* a paragraph boundary loses its space ("line one,line two");
* a missing `\id` makes the book unidentified.

The input is invalid USFM. Both readers agree, so it is not caught by their
comparison. `\v\*` *is* caught by the verse accounting.

Exposure: none in the 716 USFM files of the 13 eBible references, which
contain no `\*` at all. Consequences are fail-safe unless a malformed
reference and a matching recogniser error coincide.

Owner action, no code change: before the seal, the Collection H reviewer
confirms that no sealed reference holds a non-milestone marker in milestone
form. Either count `\*` in the USFM members and expect 0 for eBible
packages, or run `evidence/r1/rev_usfm.py` and expect no "written as a
milestone" trouble. Code fix, in any later reader version: allow milestone
syntax only for `-s`/`-e` names and standalone milestones such as `\ts`.

**N2. The gate stops official runs, not execution (process; low).** The
runner's rehearsal variables (`AW_PROTOCOL`, and likewise
`AW_DISCLOSED_INVENTORY` and the limits) let it run a sealed set while the
committed protocol's gate is shut. The set is then marked run, and its
reports, which quote the documents, are written. The result can never be
official. The gate is therefore a guard against an *official* premature
run, not a barrier to execution, and the E3 record's phrase "a gate
preventing final withheld execution" overstates it. Anyone holding the
documents and the binary could run them anyway, so the real control is
procedural: access to Collection H. Owner action: the final run instructions
state that the runner is invoked with no `AW_*` variables. The scorer already
enforces this through RUN.txt. Optional code fix: the runner refuses any
override while the committed gate is shut.

**N3. Wording of the evidence verdict (presentation; low).** "evidence
sufficient" for 1 of 31 146, and coverage printed "0.0 %" when it is not
zero (§4). The counts are exact and adjacent, so nothing is false. Suggest,
in any later scorer revision, "evidence floor met (1 ≥ 1)" and a coverage
precision that never rounds a non-zero figure to 0.0.

**N4. Deuterocanonical books in a scored document (builder guidance; low).**
Approved slots 1 and 4 are scored documents with deuterocanonical books.
Under E3, writing a deuterocanonical code in `books` (e.g. `RUT,TOB`) refuses
the seal, explicitly: "outside the 66-book canon". `books = all` seals, and
the book's verses are reported as unsupported. Listing only the 66-book codes
seals, and the book is reported as outside the declared books. The builder
brief does not say this (`evidence/lab/deutero.json`). Owner action: tell the
builder to write `all`, or only the 66-book codes, for such documents (see
§8.2).

**N5. A consistent substitute toolset passes officially (inherent;
documented).** A modified scorer and a protocol rewritten to approve it,
committed before the run, give an official verdict (§5). This is the limit
`OWNER-DECISIONS.md` §4 states. It makes the owner's external record of the
approved protocol SHA-256 **essential**, not optional.

**N6. Repeated flags accepted (nit).** `--write --write` and
`--diagnostic --diagnostic` are accepted. This is idempotent and harmless,
but the record says every repeated option is refused.

Also confirmed, outside the evaluation: the app importer's 528-verse
spacing defect.

Observation outside E3: when I looked up the Collection H environment
(read-only, unchanged), it showed "MCP servers allowed: true".
`SET-H-APPROVAL.md` §3 records "Package managers / MCP servers: off / off".
The owner may want to check that setting.

## 8. Task 5 — owner decisions (`OWNER-DECISIONS.md`)

`OWNER-DECISIONS.md` states the facts neutrally and finalises nothing. It is
accurate: I checked §1 against `SET-H-APPROVAL.md` §2, the decision package
§3 and the evaluation audit's quotation of D5. My recommendations follow.
The decisions are the owner's.

**8.1 Composition: the 24-document specification and the reviewed
28-document collection.**

* The facts:
  * The committed, owner-approved texts say "Set: 24 documents". The
    decision package calls its table "one allocation that meets every
    minimum at once; the builder may choose another that also meets them".
  * The 28 (23 scored, 3 control, 2 mixed, 0 exploratory; 16 authentic, 12
    generated) exists in this repository only as the audit's quotation of
    D5.
  * The tools compare the declared composition with the protocol's by exact
    equality.
* Recommendation, in this order:
  1. Commit the owner's D5 text, or an owner note superseding "24" in
     SET-H-APPROVAL §2, so the authority for the number is in the record.
     If the reviewed collection has 28 documents, the rationale is that the
     package's minima are binding and 24 was one allocation of them. The
     owner should state that explicitly.
  2. Run the **E3** check (`seal-set.py` without `--write`) on the reviewed
     collection **before** fixing any number. E3 refuses more than E2 did: a
     scored reference with no scorable verse; a declared deuterocanonical
     code (N4); markup the strict reader refuses. A document moved to
     *exploratory* changes the counts.
  3. Confirm the final set still meets every minimum of the decision package
     §3 and the published-evidence minimum: 8 authentic documents, 2
     formats, 4 languages, 20 000 verses.
  4. Then take option A, as `OWNER-DECISIONS.md` recommends: commit the
     exact composition as `required_composition`. No code change is needed.
* I could not inspect the collection, so I cannot say whether it meets the
  minima.

**8.2 D1/D6: unsupported verses.**

* I support the proposed amendments: report them, do not score them. The
  frozen recogniser cannot hold verse 0, chapters beyond the 66-book counts
  or books outside the 66. Scoring them would mean changing the recogniser,
  which is outside a frozen evaluation.
* E3 counts and lists every class before the run and in every score. It
  reports coverage of all printed verses beside coverage of scorable ones,
  and treats references to bridged or duplicated numbers as unvalidatable,
  which fails criterion 2 (§4; `r2-document-only-unsupported`).
* Add one sentence for N4: a scored document holding books outside the 66
  writes `books = all` (those books are then counted as unsupported) or
  lists only its 66-book codes (they are then counted as outside the
  declared books). Either way they are never scored and never dropped.

**8.3 The non-vacuous evidence requirement.**

* The default floor (at least one faithful verified verse) is correct,
  conservative in parsing, and works exactly as specified. It closes R2:
  nothing vacuous passes.
* It deliberately does not make PASS mean *broad* evidence. One verse of 31
  146 passes, and so would one scored document verifying a chapter while
  every other scored document verifies nothing. The verdict line, the
  "documents verifying nothing" list and the qualification limit show this.
* The owner should decide explicitly and record it in the protocol revision:
  * **either** keep the default and require that any statement of the
    verdict quote the full verdict line;
  * **or** set `minimum_documents_with_verified_faithful` and/or
    `maximum_scored_documents_refused_whole`. These are spread and refusal
    floors, not coverage pass marks, and need no code.
* I recommend **not** setting
  `minimum_verified_faithful_share_of_printed_scorable` unless the owner
  intends to change the approved "no coverage pass mark" policy, because it
  is a coverage pass mark.
* I set no numbers.

**8.4 Strict fidelity (`aw-fidelity-2`).** Accept it as the default. With
the reference read exactly, its strictness no longer produces reader-caused
failures: 0 on the whole BSB. What remains are real differences between
document and edition, such as ACT 28:28 here and the E2 review's PDF hyphen
and dash errors. Its known costs stand: invisible characters, ligatures and
the case of small-capital divine names count as wrong text. Any relaxation
must be a new rule version, decided before any withheld result is seen.

**8.5 Exact tool and binary approval.**

* Approve the eight tool hashes in `protocol-e3.json` (all verified equal to
  the files).
* Approve the binary pin `b4f51a62…` only together with its build
  conditions: `/home/user/abidingword/src-tauri`, rustc 1.97.0, Cargo.lock of
  `08c6faa`, webkit2gtk-4.1 2.52.6, Ubuntu 24.04.5 x86-64, empty `../dist`.
  I reproduced it under exactly these conditions in a fresh container, so an
  assessor can too. Elsewhere, re-pin in a reviewed revision.
* The finalising revision will change the protocol file: composition, gate,
  any floor slots, any wording. **Its** SHA-256 is the one the owner must
  record outside the repository and check against every score (N5).
* Run instructions: no `AW_*` variables (N2), and the N1 pre-seal check.

## 9. Checks run

The full table is in `evidence/checks/checks-summary.md`.

| Check | Result |
|---|---|
| `cargo fmt -- --check`; `cargo clippy --all-targets --locked -- -D warnings` (CI settings) | clean; clean |
| `cargo test --locked` (CI settings) | **1 115 passed, 0 failed, 9 ignored, 83 groups** (as E3 records) |
| `evaluation_references` alone; `test_e3_rules.BundledReferences` | 4 passed, 1 ignored; 3 passed |
| Python `tests/evaluation`; `tests/corpus` | **122 passed**; **26 passed** |
| `npm ci`, typecheck, lint, i18n check, `npm test`, `npm run build` | all pass; **1 053 tests in 89 files** |
| Harness binaries | E3 `b4f51a62…` reproduced (pinned path); `8cab9ddd…` at another path; E2 `800fb43c…` reproduced |
| Three readers on 13 references; the 528; 99 vectors; 100 000 fuzzed books | identical, 0 silent losses; 528/528 against three independent expectations; 90/97 as expected (7 = N1); Rust = Python 100 000/100 000 |
| Lab: R2 15 scenarios, integrity 6 + probes 11, regression 5 × 2, deutero 3, real BSB official | as reported in §§4–6 |
| Identity, checksums, history | all verify (§1) |
| CI on GitHub | not triggered (no pull request) |

## 10. Limitations of this review

* **No authentic published document.** `ebible.org` was reachable, but it is
  the only approved source for Collection H. Downloading and running eBible
  PDFs or EPUBs could have touched documents of the withheld set, so I
  downloaded nothing. Every document here is generated (Markdown) or
  synthetic, plus junk PDFs used only to cause whole-document refusals.
* **Real USFM coverage** is the 13 bundled references: English, German,
  French, Spanish, Portuguese, Korean and Greek. Hebrew, Arabic, Syriac and
  other scripts were tested with vectors and fuzzing only.
* **The inventories.** The Rust inventory (`reference.rs`) was exercised
  through the real binary and the scorer's comparison, not in isolation.
* **The 113 disclosed editions** were not rerun; nothing is estimated about
  them.
* **My extractor** shares its specification with the E3 readers, so a
  misreading of the specification common to all three would not show. The
  source-adjacency check on the 528 and the hand-written vector
  expectations are the parts that do not rest on any reader.
* **Windows** was not built or tested.
* **The lab's protocol revisions are simulations.** No owner decision is
  implied by them.

## 11. Boundaries kept

* No production recognition code, evaluation rule, tool, protocol, binary,
  existing test or historical record in this repository was changed. The E2
  harness sources were checked out temporarily to rebuild E2's binary, then
  restored and verified by hash before anything else was built or run.
* `pistio/collection-h` was not accessed. Nothing was sealed, run on a
  withheld set, merged, tagged, released or published. No installer, updater
  or paid provider was used.
* The Collection H environment was not modified (§1).
* This review stops at owner protocol finalisation. It does not authorise
  sealing or running Collection H.

## Evidence index

`evidence/README.md` lists every file, how it was produced and how to
reproduce it. `evidence/SHA256SUMS` checksums them.
