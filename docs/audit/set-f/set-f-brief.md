# Set F — the independent builder's brief (verbatim)

Given at 2026-11-01T19:54:05Z to a fresh agent with no conversation history, after the recogniser was frozen at `324cfd1` (branch head `a256720`; no recogniser or interface change after `324cfd1`). SHA-256 of the brief text: `db79bf85dbb300390762f9d6b26879a7f169c8c860436288a988516343ce106d`. The brief describes the general task only: it names no failure class, no rule or behaviour of the recogniser, no earlier result and no targeted weakness; it lists the edition identifiers already used (so the set uses new editions), the reachable sources, the manifest columns and the ground-truth format the audit reads.

```text
You are an independent test builder. Your job is to prepare and seal a withheld test set ("set F") of Bible documents, with independent ground truth, for a one-time blind evaluation of a program that reads ordinary Bible files (PDF, EPUB, Word .docx, Markdown, plain text) and tries to find every book, chapter and verse in them. Such a program must never accept wrong text as a verse, so the set should contain the kinds of documents real people really have: varied layouts, varied producing programs, varied editions and languages.

ISOLATION RULES (strict; the evaluation's independence depends on them):
- Work ONLY inside /root/aw-set-f/ (create it). Generator scripts in /root/aw-set-f/tools/, documents and their reference files in /root/aw-set-f/docs/.
- Do NOT open, list, grep or read anything under /home/user/abidingword, /root/aw-base, /root/aw-recon, /root/aw-recon-bc, /root/aw-corpus, /root/aw-set-e, /root/e-rb, /root/e-rebuild, /root/audit, /root/.claude or /tmp. You must not see the program's code, its tests, its documentation or any earlier evaluation or test set. Design the set from your own knowledge of how Bibles are typeset, exported and shared.
- Do not run or test the program. You do not need it.
- Never kill, pkill or killall any process you did not start yourself (other jobs share this machine); stop your own processes only by the PID you recorded.

WHAT TO BUILD:
- 12 to 14 documents. Each is a Bible, a Testament, or a group of several books (at most two single-book documents).
- Sources. Reachable from here: ebible.org (USFM zips at https://ebible.org/Scriptures/<id>_usfm.zip; listing at https://ebible.org/Scriptures/), raw.githubusercontent.com (public repositories of Bible texts in OSIS, USFX, Zefania XML, JSON, CSV, SQL, plain text, e.g. seven1m/open-bibles or scrollmapper/bible_databases — check each one's licence), and the package registries (pip, npm). Other sites may be blocked; try them if you like. PREFER genuinely different sources: at least four documents must come from a source other than eBible.org USFM (a different publisher's or project's encoding of the text). Where nothing but eBible is available for a language, say so.
- Editions: do not use these edition identifiers already used in earlier sets (eBible ids): deu1912 deu1951 deuelbbk deuelo deutkw eng-Brenton eng-asv eng-kjv2006 eng-rv engBBE engDBY engDRA engbsb enggnv engjps engkjvcpb englsv engoebus engtnt engwebp engwebster engwebu engwyc2017 engylt fraLSG fra_fob frajnd francl frasbl grc-tisch grcbrent grcbyz grcmt grcsbl grcsr grctr hbo hboWLC heb hebwlc kor kor1961 latVUC porblt porbr2018 porbrbsl poronbv portft spaRV1909 spabes spabll spablm spaonbv spapddpt sparvg spav1602p spavbl. The same translation obtained from a different source/encoding (for example a KJV text from another project) counts as reuse of the translation: allowed only if you need it, in a completely different layout, and marked in the manifest. Record every unavoidable reuse honestly.
- Licences: use only texts that are public domain or whose licence allows this use (record the licence per document). Never use a text whose licence forbids copying or modification of layout for this purpose.
- Languages: cover English, German, French, Spanish, Portuguese and Korean (at least one document each), plus at least one of Greek, Hebrew or Latin; more languages are welcome.
- Formats and generators: at least 5 PDFs from at least 4 different producing programs or layout engines (for example LaTeX/XeLaTeX if installable, WeasyPrint, ReportLab, fpdf2, LibreOffice, pandoc, Typst, wkhtmltopdf, Ghostscript-produced, MuPDF Story — install what you can via pip/npm/apt), at least 2 EPUBs, at least 2 Word .docx, at least one Markdown and at least one plain text file. Prefer generators and methods of your own choosing, including ones less commonly used.
- Layouts: make them realistic and varied in your own way — page sizes, columns, running heads and page numbers, verse-number and chapter-number styles in many conventions, headings, notes and cross references in different placements, poetry, introductions, contents pages, study-edition features, right-to-left text, deuterocanonical books if present, anything real Bibles do. Choose freely; do not ask anyone which features to use.
- Every document must contain the publisher's words unchanged; only the layout is yours.

INDEPENDENT GROUND TRUTH (required for every document):
- The evaluation compares what the program reads with a reference text in USFM. For each document, put next to it in docs/ a ZIP of USFM files, one file per book, holding exactly the text the document prints as verses, with standard USFM book codes (\id GEN …), \c chapter markers and \v verse markers, and the same chapter/verse numbering as the document. For an eBible source this can be its own USFM zip (unchanged, or reduced to the books the document holds). For any other source, write your own converter from that source's format to USFM and keep it in tools/; do not invent text.
- Keep notes, headings and other non-verse material in the USFM only with their proper USFM markers (\f, \x, \s1, \d …), never inside verse text.
- In /root/aw-set-f/GROUND_TRUTH.md record per document: the source (URL, project, version/date, licence), the source file checksum, how the reference USFM was made (unchanged / reduced / converted by which script), the versification the document and reference follow, and any known gaps or oddities in the source text itself (missing verses, merged verses, misplaced markers) — do not fix those silently.

SEALING:
- Write /root/aw-set-f/withheld-f.tsv, tab-separated, header line exactly:
  key	files	reference	language	script	format	layout	role	source and licence	books	sha256	generator
  key = short unique id; files = document file name in docs/; reference = the reference USFM zip file name in docs/; language = ISO code; script = Latin/Hangul/Greek/Hebrew/…; format = pdf/epub/docx/md/txt; layout = a precise one-paragraph description of the layout; role = withheld-f; source and licence; books = comma-separated USFM codes the document holds, or "all"; sha256 = SHA-256 of the document file; generator = program and version, plus your script name.
- Then write /root/aw-set-f/SEAL.txt with the UTC time and the SHA-256 of withheld-f.tsv, of GROUND_TRUTH.md and of every file in docs/ (documents and reference zips).
- Change nothing in /root/aw-set-f after sealing.

REPLY with ONLY: the number of documents, the manifest path and its SHA-256, the seal time, the list of keys with format and source kind (eBible / other — no layout details), the reuse and licence notes, and an explicit statement of every path outside /root/aw-set-f you read or listed (should be none apart from network downloads, installed tools, fonts and package installs).
```

## Construction record (written before the evaluation run)

* **Built** by the independent agent from 2026-11-01T19:54Z; **sealed**
  2026-11-01T20:30:29Z (`SEAL.txt`): manifest SHA-256
  `1c9491fd82488f3cb8954d544bb7cee7b090549a0ff37fa0ade9c8349a7a9071`,
  `GROUND_TRUTH.md` SHA-256
  `141cf6adc4a83a176a6639ebfd43031c748bacd2e6cfe917721e8c666383054c`, and all
  26 files in its `docs/` (13 documents, 13 reference USFM zips). Verified by
  the main session before the run: every checksum in `SEAL.txt` and in the
  manifest matches; no file in the set is newer than the seal.
* **Sources** (builder's report): 8 documents from a source other than
  eBible USFM (CrossWire texts as distributed by the scrollmapper
  bible_databases repository, converted to USFM ground truth by the
  builder's own scripts), 5 from eBible USFM. 6 PDFs from 6 engines (MuPDF
  Story, LibreOffice, ReportLab, fpdf2, Typst, WeasyPrint), 3 EPUBs
  (EbookLib, hand-built EPUB 2, pandoc), 2 DOCX (python-docx, LibreOffice),
  1 Markdown, 1 text. 11 languages (en, de, fr, es, pt, ko, he, grc, la, el,
  ru, it).
* **Reuse recorded by the builder:** F07 Korean is almost certainly the 1961
  revision (개역한글) used before as eBible `kor1961`, from another source and
  in a new layout; F09 (Text-Critical Greek NT) is built on the same
  Byzantine base text as `grcbyz`; F10 (Sixtine Vulgate 1590) is in the
  Clementine tradition of `latVUC`; the builder did not check every
  German/French/Spanish/Portuguese CrossWire title against earlier sets
  (by identifier none of them was used before).
* **Licences** (builder's report): public domain except F06 (CC BY-SA 4.0,
  attribution on its title page); F01 (NHEB) is listed by its distributor as
  licence "Unknown" and treated as public domain on the builder's
  understanding of the editor's dedication, not checked against a primary
  source. Nothing of set F's documents or ground-truth zips is committed.
* **Isolation incidents (builder's own report):** (1) one command meant for
  the set's own folder ran in the default working directory and listed the
  names of the repository's top-level `docs/` entries (about 30 names, e.g.
  `pdf-reading-order-v0.24.1.md`, `bible-recognition-v0.24.2.md`, `audit`,
  `corpus`); no file was opened, and by the builder's account the sources,
  editions and ground truth were already chosen and two documents rendered;
  (2) a `pip download` wrote a wheel file into the repository's root, which
  was deleted at once by its exact path (the working tree was clean when
  checked). The main session's judgement: file names disclose no failure
  class, rule or result; the set is treated as independent, and the owner is
  told of the incidents.
