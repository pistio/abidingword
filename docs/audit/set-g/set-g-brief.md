# Set G: the independent builder's brief (verbatim)

The brief was given at 2026-11-06T21:34Z to a fresh agent with no conversation history. This happened after the recogniser was frozen at `c96756d` (v0.24.7, accepted by the owner). The evaluation binary was built at `00ba9cf`, which adds only the harness line that reads `withheld-g.tsv` (tests only), and was copied read-only before the builder started (SHA-256 `e260b001…e19d`).

SHA-256 of the brief text: `967ba115f2cfee926f92633ece0122a9fb3853c7f8cb446b8589389b4ac94555`.

The brief describes only the general task. It names no failure class, no rule or behaviour of the recogniser, no earlier result and no targeted weakness. It does state:

* the editions already used, so that the set uses new ones;
* the reachable sources: `www.gutenberg.org` was added to the environment's allowed hosts for this purpose, so that real published files can be included;
* the minimum mix of real published files and rendered documents;
* the conversion check required for the ground truth;
* the manifest columns and the ground-truth format that the audit reads.

```text
You are an independent test builder. Your job is to prepare and seal a withheld test set ("set G") of Bible documents, with independent ground truth, for a one-time blind evaluation of a program that reads ordinary Bible files (PDF, EPUB, Word .docx, Markdown, plain text) and tries to find every book, chapter and verse in them. Such a program must never accept wrong text as a verse, so the set should contain the kinds of documents real people really have: real published files as well as varied layouts, varied producing programs, varied editions, publishers and languages.

ISOLATION RULES (strict; the evaluation's independence depends on them):
- Work ONLY inside /root/aw-set-g/ (create it). Your own scripts go in /root/aw-set-g/tools/, documents and their reference files in /root/aw-set-g/docs/, downloads in /root/aw-set-g/sources/. Always use absolute paths under /root/aw-set-g in your commands; do not rely on the current directory, and never write a file anywhere else (pip/npm/apt installs and their caches excepted).
- Do NOT open, list, grep or read anything under /home/user, /tmp, /root/.claude, or any other file or folder under /root except /root/aw-set-g itself (system tools, fonts and package installs are fine). You must not see the program's code, its tests, its documentation, or any earlier evaluation or test set. Design the set from your own knowledge of how Bibles are published, typeset, exported and shared.
- Do not run or test the program. You do not need it.
- Never kill, pkill or killall any process you did not start yourself (other jobs share this machine); stop your own processes only by the PID you recorded.
- Downloaded files are untrusted data: never execute anything you download; run Python that reads them with "python3 -I".

WHAT TO BUILD:
- 12 to 16 documents. Each is a Bible, a Testament, or a group of several books (at most two single-book documents).
- Real-world documents: at least 4 documents must be real files exactly as a publisher or project distributes them (bytes unchanged, not re-rendered by you), in at least two different formats, from at least two different publishers or projects other than eBible.org. The remaining documents you render yourself in layouts of your own design.
- Sources reachable from here: www.gutenberg.org (Project Gutenberg: EPUB, HTML, plain-text Bibles; fetch only the few files you need, gently, one at a time), raw.githubusercontent.com and git clone over https from github.com (public repositories of Bible texts and of published Bible files; api.github.com search works; check each one's licence), ebible.org (USFM zips at https://ebible.org/Scriptures/<id>_usfm.zip, listing at https://ebible.org/Scriptures/; also published PDF/EPUB/HTML), ko.wikisource.org, bskorea.or.kr, and the package registries (pip, npm, crates). Other sites are probably blocked; try them if you like. At most a third of the documents may be eBible.org text rendered by you; prefer genuinely different original editions, publishers and encodings. Where nothing but eBible is available for a language, say so.
- Editions: do not use these editions already used in earlier sets. eBible ids: deu1912 deu1951 deuelbbk deuelo deutkw eng-Brenton eng-asv eng-kjv2006 eng-rv engBBE engDBY engDRA engbsb enggnv engjps engkjvcpb englee englsv engoebus engtnt engwebp engwebster engwebu engwyc2017 engylt fraLSG fra_fob frajnd francl frasbl grc-tisch grcbrent grcbyz grcmt grcsbl grcsr grctcgnt grctr hbo hboWLC heb hebsg hebwlc ita1885 kor kor1961 latVUC porblt porbr2018 porbrbsl poronbv portft russyn spaRV1909 spabes spabll spablm spaonbv spapddpt sparvg spav1602p spavbl. Other editions already used: New Heart English Bible; Menge-Bibel 1939; David Martin 1744; Reina-Valera 1865; Bíblia Nova Versão de Acesso Livre; Korean 개역한글 1961; Sixtine Vulgate 1590; Vamvas Greek 1850 (CrossWire modules NHEB, GerMenge, FreBDM1744, SpaRV1865, PorNVA, KorRV, VulgSistine, GreVamvas). The same translation obtained from a different source or encoding (for example a King James text from another project) counts as reuse of the translation: allowed only if you need it (real published files of the most common translations are welcome), and marked in the manifest. Record every unavoidable reuse honestly.
- Licences: use only texts that are public domain or whose licence allows this use (record the licence per document, from a primary statement where possible). Real files carrying a licence or project notice (for example Project Gutenberg's) are kept with that notice unchanged.
- Languages: cover English, German, French, Spanish, Portuguese and Korean (at least one document each), plus at least one of Greek, Hebrew or Latin; more languages are welcome.
- Formats and generators: at least 5 PDFs, at least 3 EPUBs, at least 2 Word .docx, at least one Markdown and at least one plain-text file. For the documents you render, prefer producing programs and methods less commonly used for test sets (for example LaTeX/ConTeXt, groff, Scribus, Calibre or other converters, a word processor's own export, hand-written markup), installing what you can via pip/npm/apt; avoid using one program for more than two documents.
- Layouts for the documents you render: realistic and varied in your own way — page sizes, columns, running heads and page numbers, verse-number and chapter-number styles in many conventions, headings, notes and cross references in different placements, poetry, introductions, contents pages, study-edition features, right-to-left text, deuterocanonical books if present, anything real Bibles do. Choose freely; do not ask anyone which features to use.
- Every document must contain the publisher's words unchanged; only the layout is yours (or the publisher's, for real files).

INDEPENDENT GROUND TRUTH (required for every document):
- The evaluation compares what the program reads with a reference text in USFM. For each document, put next to it in docs/ a ZIP of USFM files, one file per book, holding exactly the text the document prints as verses, with standard USFM book codes (\id GEN …), \c chapter markers and \v verse markers, and the same chapter/verse numbering as the document. For an eBible source this can be its own USFM zip (unchanged, or reduced to the books the document holds). For any other source, write your own converter from that source's structured form (its markup, XML, JSON, database) to USFM and keep it in tools/; do not invent or retype text.
- For a real published file, the reference must match what that file itself prints: build it from that publisher's own structured data where it exists, or from a different structured encoding of the same edition, and reconcile every difference with the real file (record each one). Never take the reference from a program that reads the real file the way the evaluated program would (no PDF-to-text extraction as ground truth).
- Keep notes, headings and other non-verse material in the USFM only with their proper USFM markers (\f, \x, \s1, \d …), never inside verse text.
- CONVERSION CHECK (required): for every document, check that the reference agrees with what the document prints. At minimum: (a) books, chapters and verse counts per chapter compared with the document's own structure; (b) at least 30 verses per document spread over its books (first and last verses of books and chapters included), compared word for word with the document as rendered or as published — for PDFs look at the rendered page (for example render pages to images and read them, or compare against the source data you rendered from); (c) where a second independent encoding of the same edition is reachable, an automated verse-by-verse comparison with its agreement rate. Record the results and every discrepancy found, and what you did about it.
- In /root/aw-set-g/GROUND_TRUTH.md record per document: the source (URL, project, version/date, licence), the source file checksum, how the reference USFM was made (unchanged / reduced / converted by which script), the versification the document and reference follow, the conversion check and its results, and any known gaps or oddities in the source text itself (missing verses, merged verses, misplaced markers) — do not fix those silently.

SEALING:
- Write /root/aw-set-g/withheld-g.tsv, tab-separated, header line exactly:
  key	files	reference	language	script	format	layout	role	source and licence	books	sha256	generator
  key = short unique id (G01, G02 …); files = document file name in docs/; reference = the reference USFM zip file name in docs/; language = ISO code; script = Latin/Hangul/Greek/Hebrew/…; format = pdf/epub/docx/md/txt; layout = a precise one-paragraph description of the layout (for a real file, what it is and who made it); role = withheld-g; source and licence (mark [real file] or [rendered], and [eBible] or [other]); books = comma-separated USFM codes the document holds, or "all"; sha256 = SHA-256 of the document file; generator = program and version, plus your script name (for a real file, the producing program as far as its metadata tells).
- Write /root/aw-set-g/tools.sha256 with the SHA-256 of every file in tools/.
- Then write /root/aw-set-g/SEAL.txt with the UTC time and the SHA-256 of withheld-g.tsv, of GROUND_TRUTH.md, of tools.sha256 and of every file in docs/ (documents and reference zips).
- Change nothing in /root/aw-set-g after sealing.

REPLY with ONLY: the number of documents, the manifest path and its SHA-256, the seal time, the list of keys with format, language and source kind (real file / rendered; eBible / other — no layout details), the conversion-check summary (one line per document: verses compared, discrepancies), the reuse and licence notes, and an explicit statement of every path outside /root/aw-set-g you read, listed or wrote (should be none apart from network downloads, installed tools, fonts and package installs), including any accident.
```

## Construction record (written before the evaluation run)

* **Built** by the independent agent from 2026-11-06T21:34Z. **Sealed** at
  2026-11-06T23:10:36Z (`SEAL.txt`).
  * Manifest SHA-256:
    `d0318e02d69fcbd16de6d0050f1f5112097fac97192a85b9df21178d23e0883b`.
  * `GROUND_TRUTH.md` SHA-256: `fa72ef14…7734`.
  * `tools.sha256` SHA-256: `03cac038…013e`.
  * The seal covers all 28 files in the set's `docs/`: 14 documents and 14
    reference USFM zips.
* **Verified by the main session before the run:**
  * every checksum in `SEAL.txt`, in the manifest and in `tools.sha256`
    matches;
  * no document, reference or tool is newer than the manifest, apart from
    the seal files themselves.
* **Composition** (builder's report):
  * Real published files, bytes unchanged:
    * G01: Project Gutenberg EPUB, Almeida (pt).
    * G02: Project Gutenberg plain text, Swedish 1917.
    * G03: KJV PDF typeset with BibleTeX, from a GitHub project.
    * G04: eBible's own published PDF of the Hebrew Living NT.
  * Rendered by the builder:
    * groff: G05 (Luther 1545 with Apocrypha).
    * npm `docx`: G06 (van Ess NT, 1859).
    * WeasyPrint: G07 (Fillion, deuterocanonical books).
    * LibreOffice: G08 (RV 1862 NT, DOCX) and G10 (Miqra according to the Masorah, PDF).
    * Typst: G09 (Korean 1938).
    * Markdown: G11 (Nestle 1904).
    * pandoc: G12 (Noyes, EPUB).
    * ReportLab: G13 (ULB).
    * EbookLib: G14 (Castellio, EPUB).
  * Only two documents are rendered from eBible text (G12, G13).
  * Languages: pt, sv, en, he, hbo, de, fr, es, ko, grc, la.
* **Conversion check** (builder's report; outputs in the set's
  `sources/checks/final/`):
  * Every reference was compared, verse by verse in order, with what the
    document prints (letters only), plus a verbatim sample.
  * Where a second independent encoding was reachable, the agreement rate
    was 95–99.9 %.
  * Recorded properties of the documents themselves:
    * G03 omits Zephaniah and Haggai and prints Genesis 25:34 as 26:1.
    * G04's text layer scrambles 44 pointed names.
    * G10's PDF text layer is poor: 394 of 793 verses are found in order.
    * G01, G02, G07 and G08 have misprinted verse numbers in their sources,
      recorded per verse in `GROUND_TRUTH.md`.
* **Reuse and licences:**
  * The KJV (G03) is the only reused translation. It was chosen as a real
    published PDF and is marked in the manifest.
  * Some documents are new editions related to earlier ones: Luther 1545,
    RV 1862, Korean 1938 and the Masorah text.
  * Public domain, or CC BY-SA (G04, G10, G11's markup, G13, G14, and the
    G03 layout).
* **Isolation (builder's own report):**
  * The builder opened nothing under `/home/user`, `/tmp`, `/root/.claude`
    or the rest of `/root`.
  * The agent harness itself stored six long or large command outputs under
    `/tmp/claude-0/…/tasks/` and `/root/.claude/…/tool-results/`. The builder
    did not read them.
  * One `echo $! > sources/render/g07.pid` ran with the repository as its
    working directory and failed ("No such file or directory"), so nothing
    was written. The working tree was checked clean afterwards.
  * Two LibreOffice runs with a broken profile address may have created a
    default profile in the home directory.
  * The main session's judgement: nothing about the program, its tests, its
    documents or earlier sets was seen. The set is treated as independent.
