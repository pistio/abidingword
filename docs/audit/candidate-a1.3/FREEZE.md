# Candidate freeze: Scripture recognition A1.3

**Status: a research-evaluation candidate, recorded for the owner's
approval.** This freeze is not app integration, not a release and not a
claim that recognised Scripture is accurate. Recognised text stays
provisional. Set H has not been created or run (`set-h-protocol.md`).

## 1. The candidate

| | |
|---|---|
| **Code** | `7c7565a7a3df8f3dcc9711c8701af8c94dbad6af` (branch `claude/recognition-a1.3-non-epub-safety`, draft PR #48) |
| Git tree | `1a1989a4f73aa5721e595e935005ba6935f633e0` |
| Source checksums | `candidate-source.sha256`: SHA-256 of the 180 files that decide the result at that commit (`src-tauri/src`, `Cargo.toml`, `Cargo.lock`, `build.rs`, the harness and A1 tests, the derivation script, `docs/corpus`). The list's own SHA-256 is `0b9499bea1461e94bf2ff5405cc57f050205b31146e05840eb7476df2513391b`. |
| Harness binary | `real_document_editions`, release build at that commit: SHA-256 `cc555ac6e745d6127dc8eef770d737c232990cc0e02d0c6aa73b061b6d43dc6a`. Built twice (at `7f7962e`, the same source, and at `7c7565a`): identical. |
| Toolchain | rustc 1.97.0 (2d8144b78), cargo 1.97.0, Linux x86-64; `--release --locked` |
| Pipeline | `AW_PIPELINE=structural`: the structure-aware reading for EPUB; the candidate's recogniser for every other format |

Lineage, oldest first (all draft, unmerged):

| Phase | Commits |
|---|---|
| Frozen comparison recogniser | `c96756d` (v0.24.7) |
| A1.1 (PR #45) | … `5b2bbdd` |
| A1.2 (PR #47) | `0e4c08e`, `4473951`, `e9c406a`, `eb78609` |
| A1.3 (PR #48) | `b97ee44`, `ccaad81`, `70f9017`, **`7c7565a`** |

This folder is on a later branch that adds evaluation metadata and corpus
tools only. Nothing under `src-tauri/` differs from `7c7565a`.

**"Immutable" here means recorded, not enforced.** No tag was made
(standing rule: no tags unless told to), and a branch can be moved.
The owner can make it binding with a tag or branch protection on
`7c7565a`. Until then the checksums above identify the candidate.

## 2. Inputs

| Input | Record |
|---|---|
| Development corpus and reconstructions | `docs/audit/v0.24.7/corpus-checksums.sha256` (178 files; its SHA-256 `6c943261…`), verified before the run (`CORPUS_OK`) |
| Set G | `docs/audit/set-g/SEAL.txt`, verified before the run (`SETG_OK`) |
| Manifests | `docs/corpus/*.tsv` at `7c7565a`, unchanged |
| `spablm-docx-derived-v2` | `docs/corpus/derived-v2.tsv`, `ledger.md`; scored separately |

No original manifest, seal, checksum record or earlier audit was edited.

## 3. Results at the candidate, all 113 disclosed editions

Rerun at exactly `7c7565a` for this freeze. All 113 edition reports and
all 113 provenance reports are identical to the A1.3 run reported in
PR #48 (ignoring only time and memory).

| | v0.24.7 (`c96756d`) | A1.1 | **Candidate** |
|---|---:|---:|---:|
| Verses in the editions | 2 541 344 | 2 541 344 | 2 541 344 |
| Verified, correct text | 1 956 673 | 1 977 862 | **1 977 517** |
| **Verified, wrong text** | 4 172 | 20 | **9** |
| **Verified, not in the edition** | 7 | 2 | **0** |
| Refused (correct text, not verified) | 44 832 | 45 052 | 45 397 |
| Caught (wrong text, not verified) | 17 723 | 17 695 | 17 706 |

* **The 9 are `spablm-docx` v1**: EZR 4:11, 4:17, 5:7, 7:12; DAN 4:1,
  4:19, 6:25, 9:4, 9:11. They are a divergence between the reference and
  the generated document, shown by an independent audit (`ledger.md`).
  They stay counted.
* **With v2 in place of v1 the figure is 0 wrong, 0 not in the edition**,
  by the same code. That is a fixture repair. It says nothing about the
  recogniser's ability to notice missing text: it has none.
* Sets B–G are disclosed. None of this is independent evidence.

### By format

| Format | Editions | Printed | Verified, correct | Wrong | Not in edition | Refused, correct | Caught | Peak MB, median / max |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| docx | 17 | 359 051 | 326 825 | 9 | 0 | 3 029 | 275 | 71 / 96 |
| epub | 24 | 586 995 | 505 989 | 0 | 0 | 2 612 | 1 735 | 120 / 249 |
| md | 8 | 135 566 | 109 288 | 0 | 0 | 270 | 252 | 43 / 77 |
| pdf | 56 | 1 243 153 | 895 687 | 0 | 0 | 39 052 | 15 260 | 192 / 566 |
| txt | 8 | 216 579 | 139 728 | 0 | 0 | 434 | 184 | 67 / 100 |

Peak memory is of the reading alone, from the 113-edition run (four
readings at a time; peaks are per process).

### By script

| Script | Editions | Printed | Verified, correct | Wrong | Not in edition | Refused, correct | Caught | Peak MB, median / max |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Cyrillic | 1 | 31 169 | 1 140 | 0 | 0 | 75 | 0 | 93 / 93 |
| Greek | 10 | 108 711 | 93 877 | 0 | 0 | 334 | 223 | 48 / 100 |
| Hangul | 11 | 287 876 | 234 591 | 0 | 0 | 7 577 | 318 | 96 / 368 |
| Hebrew | 7 | 68 579 | 22 910 | 0 | 0 | 6 | 0 | 30 / 162 |
| Latin | 84 | 2 045 009 | 1 624 999 | 9 | 0 | 37 405 | 17 165 | 120 / 566 |

By language, and per edition: `data/editions-candidate.json`. Zero wrong
and zero nonexistent in every language but Spanish (the 9).

### Conservative refusals

A1.3 refuses 345 correct verses that A1.1 verified (0.017 % of correct
verified verses). 230 are in `engbsb-rwpdf2col`, where a few verse
numbers are read as not raised in a document that raises them; a plain
number is then no proof of a verse. Why those numbers are read so was
not established. The others are spread over 14 PDFs
(`docs/audit/a1.3/regression.md`).

### Negatives kept

* JHN 7:52 in `spapddpt-repub2` (the verse an omitted 7:53's number was
  once folded into): not verified, counted as caught.
* Book-id ambiguity and contradiction, note callers, verse numbers read
  as words: the 18 A1 tests and 12 A1.1 tests pass.
* A1.3's six safety tests pass; each fails with its rule switched off
  (`docs/recognition-a1.3-non-epub-safety.md` §6).

## 4. Provenance, memory and time: the 24 EPUB editions

Release build, one reading at a time, idle machine. A1.2 and the
candidate were run in turn per edition; A1.1 in a pass of its own just
after. A digest of the whole provenance report (every verse's status,
spans, character and byte offsets, excluded ranges, reasons, id checks)
was added to the harness in a scratch build for this comparison; it is
not in the candidate.

| Reading alone | A1.1 (`0e4c08e`) | A1.2 (`e9c406a`) | Candidate |
|---|---:|---:|---:|
| Peak memory, highest | 596 MB | 248 MB | **249 MB** |
| Peak memory, median | 278 MB | 119 MB | **120 MB** |
| Time, all 24 | 267 s | 272 s | 276 s |
| Provenance digest equal to A1.1 | — | **24 of 24** (525 033 verses) | 23 of 24 |

* **A1.2's guarantee holds**: the same provenance as A1.1 for every
  verse of every EPUB, in under half the memory.
* **The candidate differs from A1.2 in one edition, in two refusal
  reasons, with no change of outcome.** In `francl-rebooklibepub`,
  JHN 10:22's reason changes from `reference` to `boundary`, and
  JHN 10:21 gains the doubt `boundary`. Both are in JHN 10, a chapter the
  recogniser already refuses for its verse numbers, so neither verse was
  usable before or after. The cause is A1.3's rule for a plain number in
  a document that raises its verse numbers, which applies to every
  format. Spans, byte offsets and excluded ranges are identical.
* **Correction to the A1.3 report.** It says the existing refusal reasons
  are unchanged and every EPUB edition is identical to A1.1. That is true
  of the four counted measures and of every span and offset. It is not
  true of these two reason labels. The frozen commit's text is left as it
  is; this is the correction.
* Time is the same within 3 % on this run. (A1.2's own report, measured
  on another day, shows the same +3 % against A1.1.)

Per edition:

| Set | Edition | Reading s: A1.1 / A1.2 / candidate | Reading peak MB: A1.1 / A1.2 / candidate | Digest: A1.2 = A1.1 | Digest: candidate = A1.2 |
|---|---|---|---|---|---|
| g | `G01` | 9.4 / 9.4 / 9.9 | 483 / 248 / 249 | yes | yes |
| g | `G12` | 2.0 / 2.0 / 2.1 | 72 / 40 / 41 | yes | yes |
| g | `G14` | 0.5 / 0.5 / 0.5 | 28 / 20 / 20 | yes | yes |
| f | `F04` | 11.5 / 11.9 / 11.7 | 312 / 139 / 140 | yes | yes |
| f | `F10` | 1.1 / 1.1 / 1.1 | 65 / 36 / 36 | yes | yes |
| f | `F12` | 6.6 / 6.8 / 6.7 | 213 / 93 / 93 | yes | yes |
| e | `engwyc2017-epubapoc` | 1.2 / 1.2 / 1.3 | 58 / 31 / 31 | yes | yes |
| e | `fra_fob-pandocepub` | 2.7 / 2.7 / 2.7 | 92 / 47 / 47 | yes | yes |
| recon | `spapddpt-repub2` | 8.8 / 8.4 / 8.6 | 264 / 118 / 120 | yes | yes |
| bc | `englsv-rpandocepub` | 7.9 / 7.6 / 8.0 | 310 / 120 / 121 | yes | yes |
| bc | `francl-rebooklibepub` | 10.1 / 10.4 / 10.4 | 361 / 143 / 144 | yes | **no**: 2 refusal reasons (below) |
| bc | `frasbl-rpandocepub` | 8.9 / 8.8 / 8.9 | 338 / 133 / 134 | yes | yes |
| dev | `deu1912-epub` | 13.4 / 13.4 / 13.9 | 278 / 117 / 118 | yes | yes |
| dev | `engwebp-epub` | 16.7 / 17.5 / 17.4 | 306 / 146 / 147 | yes | yes |
| dev | `fob-epub` | 13.7 / 13.8 / 13.8 | 286 / 118 / 118 | yes | yes |
| dev | `fraLSG-epub` | 22.0 / 23.0 / 22.6 | 375 / 178 / 180 | yes | yes |
| dev | `frajnd-epub` | 14.6 / 17.0 / 16.9 | 340 / 157 / 158 | yes | yes |
| dev | `grctr-epub` | 3.9 / 4.2 / 4.2 | 84 / 48 / 48 | yes | yes |
| dev | `hboWLC-epub` | 31.9 / 31.4 / 33.3 | 220 / 105 / 105 | yes | yes |
| dev | `kjv-epub` | 14.3 / 14.5 / 14.8 | 345 / 157 / 158 | yes | yes |
| dev | `kor-epub` | 18.6 / 19.4 / 19.7 | 212 / 127 / 127 | yes | yes |
| dev | `latVUC-epub` | 21.2 / 20.3 / 19.8 | 596 / 219 / 220 | yes | yes |
| dev | `porbr2018-epub` | 13.3 / 13.8 / 13.7 | 279 / 127 / 128 | yes | yes |
| dev | `rv1909-epub` | 13.0 / 13.3 / 13.5 | 265 / 115 / 116 | yes | yes |
**Outside EPUB, memory is not reduced.** A1.2 changed the EPUB reader
only. The largest PDF readings peak at 566 MB (`spavbl-rlofodt`), 493 MB
(`latVUC-pdf`) and about 400 MB (`engrv-ebiblebook`, `frajnd-lopdf`);
the PDF median is 192 MB. Those figures are the frozen PDF path's.

## 5. Checks at `7c7565a`

* `cargo fmt -- --check`: clean.
* `cargo clippy --all-targets --locked -- -D warnings`: clean.
* `cargo test --release --locked`: 1 086 passed, 0 failed, 8 ignored (the
  harnesses), in 80 binaries.
* Interface checks (`typecheck`, `lint`, `i18n:check`, `test`, `build`)
  passed on this source in A1.3; no interface file has changed since.

## 6. Freeze readiness

| Condition | State |
|---|---|
| No known incorrect or nonexistent verified verse on the disclosed tests | **Met with one recorded exception**: 9 on `spablm-docx` v1, explained by source evidence as reference/document divergence; 0 on v2. 0 nonexistent. 0 unexplained. |
| Memory materially reduced and measured | **Met for EPUB** (596 → 249 MB highest, 278 → 120 MB median). Not addressed for PDF. |
| Provenance and safe refusals preserved | **Met**, with the two reason labels above disclosed |
| No regression concealed by changed test expectations | **Met**: no existing test expectation changed; original corpus results reported as measured |
| Development corpus and Sets B–G rerun | **Met**, at the exact commit |

## 7. Unresolved safety limitations

1. **Missing text is invisible.** A verse verified from an incomplete
   document is verified. Verification is fidelity to the document read,
   not to the edition.
2. **No independent evidence.** Every figure here is on disclosed
   documents, almost all generated in this environment from eBible USFM.
3. **The A1.3 rules are measured on 113 editions.** A face change, a
   smaller size or an unraised number can mean something else elsewhere.
   Each rule only withholds verification.
4. **`engbsb`'s 230 refusals are not understood** at the level of why its
   numbers are read as not raised.
5. **Thin coverage in places.** Where little is verified, little is
   tested: Hebrew (22 910 verified of 68 579 printed, 7 documents), and
   one document each in Cyrillic, Swedish and Italian.
6. **PDF memory** up to 566 MB; scanned PDFs are not read at all.
7. **Markdown derivation** can drop paragraph classes the corpus does
   not exercise (`ledger.md` §2).
8. **PDFium** (PR #46) is a separate experiment, off, and no part of the
   candidate.

## 8. Files here

| File | What it holds |
|---|---|
| `candidate-source.sha256` | The candidate's source checksums |
| `ledger.md`, `data/v1-v2-ledger.json` | `spablm-docx` v1 and v2: identities, provenance, the nine verses, results |
| `data/spablm-docx-derived-v2.provenance.json` | v2's build record |
| `data/completeness-*.json` | Source-to-document audits: spablm v1 and v2, deutkw, kor1961, frajnd |
| `data/editions-candidate.json` | Every edition at the candidate: the five measures, reasons, memory |
| `data/epub-provenance-memory-time.json` | The 24 EPUBs: digests, memory and time for A1.1, A1.2 and the candidate |
| `set-h-protocol.md` | Proposed protocol for an independent Set H. Not authorised. |
