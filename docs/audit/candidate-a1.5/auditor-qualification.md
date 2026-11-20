# Completeness auditor: qualification record (Phase A1.5)

**Result: version 1 of the auditor was not fit to qualify a test set.
Version 2 replaces it.** The auditor is `scripts/corpus/audit-derived-completeness.py`
(USFM source to derived DOCX). It is still a USFM-to-DOCX tool only; see
"Not qualified" below.

## 1. The PR #52 mutation suite, executed

| | |
|---|---|
| Commit | `eb51969938cce2a6ec4dbfc03144356472dbec8f` (PR #52 head) |
| Command | `python3 -m unittest discover -s tests/corpus -p 'test_audit_mutations.py' -v` |
| Python | 3.13.18, Linux x86-64 |
| Auditor (v1) SHA-256 | `5c7eeaa68b3e5ed126bea3819fed7d7a9149dded6a9fad19072670779910c248` |
| Suite SHA-256 | `14ca30f54bc2a69df3ac72ba909e5c991271b41a83f8b9e0268dc602a9e6133a` |
| Result | 11 tests, 11 passed |

All 11 passed against version 1. That showed the suite and the auditor
agreed, and nothing more: both were written by parties who knew what the
auditor checked.

## 2. Independent adversarial review

A separate reviewer agent was given the auditor's stated claim and told
to break it. It wrote its own fixtures (invented words, standard library
only), did not see the suite above, and was not allowed to change the
repository.

| 83 probes against version 1 | |
|---|---:|
| Correct verdict | 35 |
| **Escapes** (damaged document reported complete) | **33** |
| False alarms (complete document reported incomplete) | 15 |

Its raw results: `data/independent-probes-auditor-v1.json`. The escapes
fell into six groups:

1. **No floor on the source.** An empty archive, a source of unknown or
   repeated books, a UTF-16 file and a `\v1` without a space all passed.
2. **A space-insensitive fallback.** Merged words ("blimnarf") passed.
3. **Order was never checked.** Re-ordered verses, chapters and books,
   and interleaved verse text, passed.
4. **The DOCX reader ignored visibility.** Hidden, white, tiny and
   tracked-change text counted as present.
5. **Heading mode swallowed verse words** after `\rq`, `\ip`, `\d`, `\sp`;
   an unclosed note swallowed whole verses.
6. **Unaudited zones in the document.** Italic and bold-centred
   paragraphs and text before the first verse could hold anything.

None of these changes an earlier result: the corpus documents have none
of these defects (§4). They are ways a *future* damaged fixture could have
been sealed as complete.

## 3. Version 2

Rewritten to fail closed. Per book, the source and the document are each
one ordered stream of chapters, verse numbers and words, and the two must
be equal token for token. Headings are compared too, and a source heading
must be set as a heading. What the script does not positively understand
fails. A generator's deliberate omissions are a named, declared profile
(`--profile=build-derived-editions-usfm`, `--books=66`), counted in the
report, never silent. The script's header lists every rule.

| The same 83 probes against version 2 | |
|---|---:|
| Correct verdict | 78 |
| Escapes | 1, a declared omission (below) |
| False alarms | 4, each a deliberate refusal (below) |

* **The one "escape"**: a speaker's name (`\sp`) inside a verse, dropped.
  The generator profile declares it does not print `\sp`. The words are
  counted and listed by verse. Under the default `strict` profile the
  same document fails. `spablm` has 33 such words in 27 places.
* **The four refusals**: a raised run that is not a verse number; a
  hyphen written as an element; a chapter heading with two numbers;
  alternate content. Each document is in fact complete; the auditor
  refuses because it cannot be sure.

A defect of version 2's own first draft was found on the real documents,
not by any test: a word running across a character marker
(`\w blim|…\w*,`) was read as two words. It is fixed and has a test.

**Tests now**, 21, all passing:

```
python3 -m unittest discover -s tests/corpus -p 'test_audit_*.py' -v
```

| File | SHA-256 | What |
|---|---|---|
| `scripts/corpus/audit-derived-completeness.py` (v2) | `d2681757…0ab5f0` | the auditor |
| `tests/corpus/test_audit_mutations.py` | `14ca30f5…9e6133a` | PR #52's 11, unchanged |
| `tests/corpus/independent_auditor_probes.py` | `f1b2acb7…01983537` | the reviewer's 83 probes, as written |
| `tests/corpus/test_audit_independent_probes.py` | `1cd9cfb4…e17c2b64` | the probes as a test; 5 cases added after the review |

No expectation of PR #52's suite or of the reviewer's probes was changed.
The five probes whose verdict differs from the reviewer's are named in
the test with their reasons, so a change to any of them is seen.

## 4. The real derived documents under version 2

`--profile=build-derived-editions-usfm --books=66`; outputs in `data/completeness-v2-*.json`.

| Document | Verses complete | Words missing | Verdict |
|---|---:|---:|---|
| `spablm-docx` v1 (original, sealed) | 31 094 of 31 103 | 130, all `\mi` | **incomplete: the same nine verses** |
| `spablm-docx-derived-v2` | 31 103 of 31 103 | 0 | complete |
| `deutkw_usfm_lo.docx` | 31 157 of 31 157 | 0 | complete |
| `kor1961_usfm_lo.docx` | 31 081 of 31 081 | 0 | complete |
| `frajnd` (DOCX rebuilt from its USFM) | 31 170 of 31 170 | 0 | complete |

The verdicts are those of version 1 (PR #51). Version 2 additionally
compares section headings and psalm titles, which version 1 skipped.

One new finding: `kor1961`'s USFM has 36 unprintable control characters
(U+0001, U+0003) inside psalm titles. LibreOffice drops them. They are
counted, not compared.

## 5. Not qualified

* **Only USFM to DOCX.** There is no auditor for PDF or EPUB output, and
  none for an authentic published file, where there is no source to
  compare with. Set H's protocol must not rely on this tool beyond
  generated DOCX.
* **Passing these tests is not independence.** The probes are one
  reviewer's ideas. A defect nobody thought of is not tested.
* A declared profile is trusted: if a generator is declared not to print
  `\sp`, the auditor does not check that it should.
