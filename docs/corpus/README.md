# The real-edition corpus (v0.24.2)

Abiding Word reads Bibles that people already have as PDF, EPUB, Word,
Markdown or text files. Whether that works is measured on real editions, not
on invented samples: this folder describes the corpus the recognition audit
(`src-tauri/tests/real_document_editions.rs`) reads.

Nothing in the corpus is committed to Git. The files are downloaded and
checked by `scripts/fetch-real-editions.sh`, and the converted variants are
built by `scripts/build-derived-editions.py`.

## Files

| File | What it says |
|---|---|
| `editions.tsv` | One row per edition and format: its key, the document file(s), the reference text it is compared with (the same edition's USFM), language, script, format, layout, role and source/licence. |
| `sources.tsv` | Where each file comes from (an eBible.org address, a file of this repository, or `derived`) and the SHA-256 of the build the audit was run on. |

## What is in it

* **Interface languages, at least two editions each:** English (World English
  Bible, King James Version), German (Luther 1912, Elberfelder 1905), French
  (Louis Segond 1910, Ostervald), Spanish (Reina-Valera 1909, Español
  Sencillo), Portuguese (World Portuguese Bible, Bíblia Livre 2018), Korean
  (Korean Bible 1910, 개역한글 1961).
* **Original languages and Latin:** Textus Receptus (Greek), Westminster
  Leningrad Codex (Hebrew, right to left), Clementine Vulgate (Latin, with the
  deuterocanon).
* **Formats and layouts:** PDFs from eBible.org in letter, A4 and book sizes,
  one and two columns, with footnotes, book introductions, cross-reference
  lines and raised cross-reference letters; a single-book PDF (Luke); EPUBs;
  a one-verse-per-line text export; and **converted** documents made by a
  different program (LibreOffice Writer) from the publisher's own HTML or USFM
  — Word documents, PDFs with superscript verse numbers, and Markdown — so the
  recogniser is not tuned to one producer's layout.
* **Hidden editions** (role `hidden`): six editions — German Textbibel, French
  Darby (PDF and EPUB), Spanish Free Bible, Young's Literal Translation and the
  Byzantine Greek New Testament — that are not looked at while the recogniser
  is developed. The audit leaves them out unless `AW_INCLUDE_HIDDEN=1`; they
  are read once, for the final evaluation.

## Licences

All editions are public domain except the Bíblia Livre 2018 (CC BY 4.0,
attribution in `editions.tsv`) and 개역한글 1961, whose economic copyright has
expired; its integrity and attribution are respected (see
`resources/scripture/kor1961/SOURCE.md`). The converted documents keep the
publishers' words unchanged: only the layout is new.

## Running the audit

```sh
scripts/fetch-real-editions.sh ~/aw-corpus
# converted variants (LibreOffice needed), as the script prints at its end
cd src-tauri
AW_DOCUMENT_EDITIONS=~/aw-corpus AW_AUDIT_OUT=~/aw-corpus/audit \
  cargo test --release --test real_document_editions -- --ignored --nocapture
```

`AW_EDITIONS=key,key` limits the run; `AW_INCLUDE_HIDDEN=1` adds the hidden
editions. The audit writes `summary.md` and, per edition, a Markdown report and
a JSON file with every difference. The recogniser never sees the reference
text: it is read only after the document has been recognised.

## What the audit counts

Per edition and per book:

* **printed** — verses the edition has (the reference USFM, empty verses left
  out and counted as absent);
* **usable** — verses in chapters the recogniser accepts;
* **unmapped** — verses not found or in refused chapters;
* **exact / whitespace only / case only / different** — usable verses whose
  text is identical, differs only in spacing, only in letter case (small
  capitals), or differs otherwise (a **genuine mismatch**);
* **false positives** — usable verses the edition does not have;
* refused chapters with their reason (reading order, verse numbers, other);
* books not found, time and peak memory.

The acceptance gates are 98 % usable coverage per qualifying edition and zero
genuine mismatches, false positives and wrong books, chapters or boundaries
among usable verses. They are never loosened to make an edition pass.
