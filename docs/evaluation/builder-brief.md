# Brief for the builder of a withheld test set

You are asked to assemble a test collection of Bible documents in common
file formats, each with a reference text it can be compared with, plus a
few documents with no Scripture and a few that quote it. Someone else will
later run software on the collection once; you will not run it, see it,
or learn how it works. That is deliberate: the collection is useful only
if whoever chooses the documents knows nothing about the software.

## Before anything else

Record in `access-records.md`: your session link (or your name), the
environment, and the repositories your session description says you were
given (do not list or look for others). Your whole session is kept
as a record: every command, every address you open and every file you
read will be reviewed before the collection is used.

## What you receive, and what you must not seek

You receive only: this brief; `disclosed-editions.tsv` (editions that must
not be used); `disclosed-documents.sha256` (fingerprints of files that must
not be used); `check-set.py` and `seal-set.py` (check your folder, and seal
it), with `reference_inventory.py`, `usfm_text.py`, `usfm-books.tsv` and
`protocol-e3.json`, which they read and which you keep beside them; and, if
you make Word documents from USFM, `audit-derived-completeness.py`.

Do not open, search or ask for, by any means (web pages, `git`, APIs,
search engines, other sessions): any software that reads Bibles from
documents, its source code, its rules, tests, issues, pull requests or
reports, or any earlier test collection or its results. Use `git` only
for the repository you were given. Do not run any
Bible-recognition software on the documents. If you come across any of
these by accident, stop and record it in `access-records.md`.

Also not allowed, because they reach beyond your repository: listing,
attaching or checking access to other repositories (tools such as
`list_repos`, `add_repo`, `check_repo_access`); creating scheduled tasks,
other sessions or messages to other sessions; adding network hosts or
connectors. Web search is allowed only to find editions on the approved
sources. Anything a helper agent you start does is part of your record,
under the same rules. Breaking any of these rules invalidates your
session's work.

## Sources and licences

* Lawfully available published files only, with their licence recorded.
* **Public domain only**, unless the owner's approval says that texts
  under an open licence may also be used. No all-rights-reserved text.
* No scanned page images: every document's text must be selectable.
* The approved sources are those the owner names in your first message
  (initially `ebible.org` only); your network reaches no other host. Record
  every URL you visit.

## The collection

24 documents with keys `H01` … `H24`. The table below is one way to meet
every requirement at once; you may choose another that also meets them.
Choose titles, publishers and layouts freely.

| Requirement | Minimum |
|---|---|
| Formats | EPUB 10, PDF 8, DOCX 4 |
| Languages, among Scripture documents | English, German, French, Spanish, Portuguese, Korean 2 each; Greek, Hebrew, Latin 1 each |
| Canon | 2 with deuterocanonical books; 2 New Testament only; 1 a single book |
| Controls (no Scripture at all) | 3: e.g. a novel, a cookbook, a software manual |
| Mixed (Scripture quoted in other text) | 2: e.g. a commentary, a lectionary |
| **Authentic** documents (published by someone else, not made by you) among those with an exact reference | at least 8, in at least 2 formats and 4 languages, with at least 20 000 verses in their references |

If a requirement cannot be met with editions that are not excluded, leave
the slot out or declare the document *exploratory* (below), and write down
why. Never fill it with an excluded edition.

## Exclusions

* An edition listed in `disclosed-editions.tsv` must not be used **in any
  format, from any publisher**: the text, not the file, is what is
  excluded. If you are unsure whether two editions are the same, treat them
  as the same and record the question.
* `check-set.py` refuses any file, or any USFM book inside a reference,
  whose fingerprint is in `disclosed-documents.sha256`. That catches only
  identical files; the edition check above is yours and the reviewer's.

## Classes and references

Every document is one of:

* **scored**: it has an exact matching reference: the same edition,
  revision, canon and versification, in USFM, zipped. Preferably the
  publisher's own source text of the same release. If you convert a
  publisher's text to USFM, do not change a word, a number or a book;
  record the method. Keep every book, chapter and verse as the edition
  numbers it, including verse 0, books outside the 66 and chapters the
  usual numbering does not have: the check lists them and they are
  counted, never dropped.
* **exploratory**: no exact reference can be found or shown to match.
  Give the closest reference you have, or `-`.
* **control**: no Scripture; reference `-`.
* **mixed**: reference `-`, and `allowed/<key>.txt` listing every passage
  it quotes, one per line: `JHN 3:16`, `PSA 23:1-6`, or `PSA 23` for a
  whole chapter. Book codes are the usual three-character USFM codes.

A reference is a `.zip` of USFM book files (`.usfm` or `.sfm`, UTF-8, one
`\id` line each), with nothing else in it but prose such as `copr.htm`;
or a single `.usfm` file. A verse map, OSIS, MorphGNT or any other file
in it is refused: take it out.

**Keep the publisher's archive and make a USFM-only package from it.** A
publisher's archive often holds other files (a signature `keys.asc`, a
style sheet `*.css`): copy the USFM book files (and prose, if you like)
**byte for byte** into a new `.zip`, change nothing in them, and record both
in `reference-sources.tsv` (below): the publisher's archive and its
SHA-256, each member you kept with its SHA-256, and each member you left
out with its SHA-256. `check-set.py` checks the package against the kept
rows.

`check-set.py` also refuses a reference it cannot read without guessing:
a footnote, cross reference or character span that is never closed or is
closed by another marker, an alternate verse number left open, a
milestone without its `\*`, a marker that is not standard USFM 3, text in
a chapter outside any verse, a verse marker without a number, or verse
text with a character its Unicode version does not know. Every refusal
names the file, the line and the reason. Use another release, or another
edition; never edit the publisher's text to make the check pass.

A **scored** document's reference must print the books the document
holds: `check-set.py` refuses a scored document whose reference prints no
verse that can be scored (an empty package, prose only, only books outside
the 66 such as Tobit, only empty verses), or a book in its `books` column
that the reference does not print. Such a document can only be
*exploratory*.

A document you make yourself from a reference (a Word file from USFM, say)
is *generated*: record the program and settings. For a Word file made
from USFM, `audit-derived-completeness.py` must pass. For every scored
document, sample at least 20 verses against the reference by hand and
record them.

## What you deliver

A documents folder (the files only) and a seal folder:

* `manifest.tsv`: the 14 columns `key, files, reference, language,
  script, format, layout, role, source and licence, books, sha256,
  generator, kind, class` (`kind`: authentic or generated; `role`: `set-h`;
  `books`: the USFM codes of the books the document holds, or `all`,
  never empty; `-` only for a document with no reference);
* `composition.tsv`: `measure<TAB>documents`, one line each for `scored`,
  `control`, `mixed`, `exploratory`, `authentic` and `generated`: the
  numbers you declare. They must be those of the manifest, and those the
  protocol requires;
* `checksums.sha256`: every file of the documents folder (`sha256sum`
  format, names relative to that folder);
* `provenance.tsv`: `key, publisher, edition, revision, retrieved_from,
  retrieved_utc, licence, document_sha256, reference_source,
  reference_method, identity_checked_by`;
* `reference-sources.tsv`: `key, reference, publisher_archive,
  publisher_archive_sha256, member, member_sha256, role` (`role`: `kept`
  for each member of your package, `removed` for each member of the
  publisher's archive you left out); a header only if no document has a
  reference;
* `allowed/<key>.txt` for each mixed document;
* `access-records.md`: who you are (session or person), the environment,
  every URL visited, every file read;
* `review-ledger.md`: left for the reviewer, who checks your references
  and editions and signs it.

Every file name you write (in the manifest, `checksums.sha256` and the
seal) must be a plain relative path inside its folder: no absolute path,
no `..`, no backslash. Put real files in both folders; symbolic links are
refused.

Run `python3 seal-set.py <seal folder> <documents folder>
--inventory=disclosed-documents.sha256` until it passes, then hand over.
After the reviewer has signed `review-ledger.md` and the owner has
approved the list, the same command with `--write` writes `SEAL.txt`; it
refuses while the ledger or the access records are missing, and never
seals a folder twice.

Do not post any document or its text anywhere public.
