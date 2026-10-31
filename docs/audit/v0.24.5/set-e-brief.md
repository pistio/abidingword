# Set E — the independent builder's brief (verbatim)

Given at 2026-10-31 22:58 UTC, after the recogniser was frozen at `fc0f538`, to a fresh agent with no conversation history. SHA-256 of the brief text: `f3aa0492ee0cd4ae858625839dcb6c8f4c9b9b1ca30dcf9593e9444ff3fb1f0d`. The brief names no rule of the recogniser, no failure class and no earlier result; it lists only the edition identifiers already used (so the set uses new editions) and the manifest columns the audit harness reads.

```text
You are an independent test builder. Your job is to prepare and seal a withheld test set ("set E") of Bible documents for a one-time blind evaluation of a document recogniser that finds books, chapters and verses in ordinary Bible files (PDF, EPUB, Word .docx, Markdown, plain text) and must never accept wrong text as Scripture.

ISOLATION RULES (strict; the evaluation's independence depends on them):
- Work ONLY inside /root/aw-set-e/ (create it). Put your generator scripts in /root/aw-set-e/tools/ and the documents in /root/aw-set-e/docs/.
- Do NOT open, list, grep or read anything under /home/user/abidingword, /root/aw-base, /root/aw-recon, /root/aw-recon-bc, /root/aw-corpus, /root/audit, or /tmp. You must not see the recogniser's code, its rules, its tests or any earlier evaluation. Design the set from your own knowledge of how real Bibles are typeset and exported.
- Do not try to run or test any recogniser. You do not need it.

WHAT TO BUILD:
- 12 documents, each a Bible (whole, a Testament, or a group of books — at least several books each, except at most one single-book document) generated from eBible.org USFM (download https://ebible.org/Scriptures/<id>_usfm.zip; browse https://ebible.org/Scriptures/ or https://ebible.org/find/ for ids). Only eBible.org and package registries (pip) can be reached.
- Use editions NOT in this list (already used in earlier sets): deu1912 deu1951 deuelbbk deuelo deutkw eng-Brenton eng-asv eng-kjv2006 eng-rv engBBE engDBY engDRA engbsb engjps engkjvcpb englsv engoebus engwebp engwebster engylt fraLSG fra_fob frajnd francl frasbl grc-tisch grcbrent grcbyz grcsbl grcsr grctr hbo hboWLC hebwlc kor kor1961 latVUC porblt porbr2018 porbrbsl poronbv portft spaRV1909 spabes spabll spablm spaonbv spapddpt spavbl. Prefer public-domain or openly licensed editions; record the licence.
- Languages: cover English, German, French, Spanish, Portuguese and Korean (at least one document each), plus at least one of Greek, Hebrew or Latin. If eBible has no unused edition for a language, you may reuse a source edition from the list ONLY in a completely different layout, and say so.
- Formats: at least 4 PDFs (from at least 3 different generators/layout engines), at least 2 EPUBs, at least 2 Word .docx, and at least one each of Markdown and plain text.
- Layouts: make them realistic and varied, the way publishers, conversion tools and people really produce Bible files. Vary freely, e.g.: page sizes, one/two/three columns, running heads and page numbers in various positions and wordings, verse-number styles (superscript, bold, inline, in brackets, in the margin, chapter:verse at line start), chapter styles (headings in various languages and wordings, drop caps, numbers only), section headings, footnotes and cross references (page-foot, end of book/document, inline), poetry indentation, introductions, tables of contents, title pages, study-edition features, speaker labels, Psalm titles, deuterocanonical books if the edition has them, right-to-left text. Choose combinations of your own design; do not ask anyone which features to use.
- Every document must contain the publisher's words unchanged (layout only is yours). You may relax nothing about the text itself.

SEALING:
- Write /root/aw-set-e/withheld-e.tsv, a tab-separated manifest with this header line and one row per document:
  key	files	reference	language	script	format	layout	role	source and licence	books	sha256	generator
  where: key = short unique id (edition-layout, e.g. engxyz-wpdf3col); files = the document's file name in /root/aw-set-e/docs/; reference = the eBible USFM zip file name used (e.g. engxyz_usfm.zip; copy that zip into /root/aw-set-e/docs/ as well); language = ISO 639-1/3 code; script = Latin/Hangul/Greek/Hebrew; format = pdf/epub/docx/md/txt; layout = a precise one-paragraph description of the layout you produced; role = withheld-e; source and licence = eBible id and licence; books = comma-separated USFM codes held, or "all"; sha256 = SHA-256 of the document file; generator = program and version, plus your script name.
- Then compute the SHA-256 of withheld-e.tsv itself and write it to /root/aw-set-e/SEAL.txt together with the UTC time.
- Do not change anything in /root/aw-set-e after sealing.

REPLY with ONLY: the number of documents, the path of the manifest, the manifest's SHA-256 and seal time, the list of keys (no layout details), and an explicit statement of every path outside /root/aw-set-e you read or listed (it should be none apart from ebible.org downloads and pip installs).
```
