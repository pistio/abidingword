# Set G — ground truth record

Built 2026-11-06 in /root/aw-set-g by an independent test builder. 14 documents, each with a
reference ZIP of USFM files (one file per book, `\id CODE`, `\c`, `\v`; notes as `\f`/`\x`,
headings as `\s1`, `\d`, `\cd`, `\qa`, `\rem`; never inside verse text). Converters and
renderers are in `tools/` (checksums in `tools.sha256`). Downloads, intermediate files,
renderer inputs and every check output are in `sources/` (check outputs in
`sources/checks/final/`, summary in `sources/checks/final/summary.txt`,
produced by `tools/run_final_checks.sh`).

## Conversion-check method (applies to all documents)

* (a) Structure: `tools/validate_usfm.py` (books, chapters, verses per chapter, duplicate or
  non-increasing numbers; `*_structure.json`) and the printed-number check of
  `tools/check_sequence.py --numcheck` (for every verse found, the last number printed before
  it must be its verse number; verse 1 may carry the chapter number or nothing).
* (b) Word-for-word: `tools/check_sequence.py` compares **every** reference verse, in order,
  with the document's own text (EPUB spine XHTML, DOCX `document.xml`, PDF text layer via
  `pdftotext`, plain text/Markdown as is), letters only (case-folded; punctuation, spacing and
  line-end hyphenation ignored). Results: *exact* (contiguous), *gapped* (all words in order,
  interrupted by running heads, notes or page breaks), *missing*. In addition
  `tools/check_sample_verbatim.py` tests a sample (first/last verse of every book, first/last
  verse of up to 12 chapters per book, at least 40 verses) **verbatim with punctuation**.
  For rendered PDFs the text sent to the typesetter (HTML, Typst, flat ODF) is also checked
  (`*_seqinput`), and page images were rendered with `pdftoppm` and read.
  The PDF text layer is used only for checking, never to build a reference.
* (c) Second encoding: `tools/compare_encodings.py` (verse-by-verse, letters only unless
  stated), where an independent encoding of the same edition was reachable.

Versification: every reference follows the numbering printed by its document (no remapping).

---

## G01 — Portuguese, Almeida (Lisbon 1911 printing), real EPUB, Project Gutenberg #62383

* Document: `G01_pt_pg62383_biblia_sagrada.epub` = https://www.gutenberg.org/ebooks/62383.epub3.images
  (bytes unchanged; sha256 4e9dad347361f6dd908c60b2d84d64d018b5c2babbb803e66036adc62ee9ebc4;
  dc:date 2020-06-12, dcterms:modified 2026-09-24). "A Biblia Sagrada, contendo o Velho e o
  Novo Testamento", tr. João Ferreira d'Almeida; produced by Júlio Reis, abi278, Sergio
  Queiroz and the Online Distributed Proofreading Team (images from Faithofgod.net).
* Licence: "Public domain in the USA" (Gutenberg catalogue and `dc:rights`); the Project
  Gutenberg licence/header inside the file is kept unchanged.
* Reference: converted by `tools/conv_pt_gutenberg62383.py` from the same publisher's
  structured HTML edition https://www.gutenberg.org/cache/epub/62383/pg62383-h.zip
  (sha256 5d368fdf9aa50742505be3e4aa27c6c60a01aad557e7add33e60ce631175e409):
  `<h2 id=GEN>` books, `<p id="Gen1-2">` verses, `<span class="first">` chapter numbers,
  `<h3>` chapter summaries → `\cd`, centred psalm titles → `\d`, Hebrew acrostic letters
  (Ps 119, Lamentations) → `\qa`, marginal chronology ("Antes de Christo 4004") → `\rem`,
  numbered footnotes (reference lists) → `\x`, lettered endnotes ([A]…[AAA], 2138 textual
  notes such as "Heb. todo o corno") → `\f`.
* Versification: as printed (Protestant/KJV-like); 66 books, 1189 chapters, 31104 verses.
* Checks: (a) structure clean; printed numbers 31103/31104 agree. (b) 31104/31104 verses
  exact in the EPUB text in order (note labels removed); verbatim sample 1045/1126 (the
  others differ only where note labels/italic runs split punctuation spacing; all are exact
  letter-for-letter). (c) No independent second encoding of this edition was reachable
  (github mazza/biblia-arc1911 is derived from this same Gutenberg file and covers 23 books;
  the Zefania "Portuguese version" is the later Almeida Atualizada) — not used.
* Source oddities (kept): Mark 4:34 is printed with the number "31" (id `Mar4-34`); the
  reference keeps 4:34. In Proverbs 31:10–31 the acrostic letter names ("Aleph." …) are
  printed after the verse numbers inside the verse and therefore stay in the verse text.
  Old Portuguese spelling (Moysés, Christo) as printed.

## G02 — Swedish, Bibeln 1917, real plain text, Project Gutenberg #2100

* Document: `G02_sv_pg2100_bibeln_1917.txt` = https://www.gutenberg.org/ebooks/2100.txt.utf-8
  (bytes unchanged, CRLF; sha256 2eec3ebd1125b40c785f4e208ec332dbb8b7d358fb693fc15d588fa188186f29;
  release 2000-03-01, updated 2020-12-31; produced by Project Runeberg).
* Licence: "Public domain in the USA"; Gutenberg licence text in the file unchanged.
* Reference: `tools/conv_pg2100_txt.py` from the file's own explicit verse markup
  ("CCC:VVV text", indented continuation lines, book titles from its contents list,
  "[n]" note anchors → `\f`). This is the only structured form this publisher offers; the
  converter reads markers, not layout. Corrections made explicitly and reported
  (`G02_converter_report.txt`): 24 verses printed with a wrong chapter number inside a
  running chapter (e.g. "010:017" for Ecc 9:17, "053:013–015" for Isa 52:13–15, Job
  "040:034–038" for 39:34–38, Song "004:001" for 5:1) are kept in the running chapter; two
  repeated verse numbers (Lam 1:21 printed twice → 1:22; Luke 20 "21, 23, 23" → 22) are
  renumbered. These corrections agree with the independent Zefania structure.
* Versification: Swedish 1917 (Hebrew-style OT numbering, e.g. Joel 4 chapters, Mal 3).
  66 books, 1189 chapters, 31168 verses, of which 17 are printed as a bare number with no
  text (Jos 21:36–37, Neh 7:68, Mt 17:21, 18:11, 21:44, 23:14, Mk 7:16, 9:44, 9:46,
  11:26, 15:28, Jn 5:4, Acts 8:37, 15:34, 24:7, Rom 16:24) — empty `\v` in the reference.
  Book order in the file: James follows 3 John (… 1–3 John, James, Jude, Revelation).
* Checks: (a) structure clean. (b) 31151 non-empty verses exact, in the file's book order;
  printed numbers agree except the 2 renumbered verses; verbatim sample 1126/1126.
  (c) Zefania "Swedish 1917 Version" (SF_2009-01-20_SWE_SVD, sha256 4ea1bea0…f0b0, from
  the Unbound Bible): 30989 common verses, letters agreement 98.78 %; differences are
  Zefania defects (stray words appended, e.g. Gen 21:34 "likhet med varandra."),
  `<b>` markup, a few wording variants (Gen 15:11 rovfåglar/rovfåglarna) and Zefania's
  KJV-style re-versification (179/159 verses only in one side).
* Oddities kept: notes "[1] V. 9-20 se …" (Mark 16:9) and "[2] 7:53-8:11 …" (John 7:53)
  → `\f` at the anchors; mid-verse paragraph breaks joined into the verse.

## G03 — English, King James Version, real PDF, vermiculus/bible (BibleTeX project)

* Document: `G03_en_bibletex_kjv.pdf` = `bible.pdf` of https://github.com/vermiculus/bible
  (commit 642de2d6da048f7d42fdda033c0d159e709e9ae3, 2018-08-19; bytes unchanged;
  sha256 36940c681aeaf44e2d16ca1289ceabae1f7da5b7ea8caec8e87b919f9a82e0b7; PDF created
  2015-05-17, "LaTeX with hyperref", xdvipdfmx 20140317, 816 Letter pages, two columns).
* Licence: README: KJV TeX content public domain (from Project Gutenberg; the PG licence is
  printed in the PDF), layout class CC BY-SA. **Translation reuse**: KJV (eng-kjv2006 used
  before) — taken deliberately as a real published file of the most common translation.
* Reference: `tools/conv_vermiculus_kjv_tex.py` from the project's own TeX sources at the
  same commit (`bible.tex` book list, `kjv/old|new/*.tex`): counters of `\Chapter`/`\Verse`,
  `\Chapter[name=…]` → `\s1`, `\LORD` → LORD, speaker macros (`\Father{…}`, `\Adam`,
  `\Eve`, `\Serpent`, 22 uses in Genesis) → the curly quotation marks that `\enquote`
  prints (‘…’ for the starred form).
* Versification: as printed. 64 books: **Zephaniah and Haggai are not in the PDF** (their
  files exist but bible.tex does not include them). 1184 chapters, 31011 verses. Verse 1 of
  every chapter and the number of chapter 1 are not printed (drop numerals for chapters ≥2).
* Checks: (a) structure clean; printed numbers 31004/31011 (the 7 others are book starts
  where only a page number precedes the unnumbered verse 1). (b) 31011/31011 verses exact,
  in order, in the PDF text layer; verbatim sample 852/1116 (failures are column/page
  breaks and hyphenation; all letter-exact); page 5 image read (Gen 3:24–5:20).
  (c) eBible eng-kjv USFM (sha256 12179046…cada): 31010 common, letters agreement 99.11 %;
  differences are Gutenberg-vs-Cambridge spelling variants (throughly/thoroughly,
  Sabtecha/Sabtechah) and the shift below.
* Source oddities (kept, printed so): **Genesis 25:34 is printed as 26:1, so Genesis 25 has
  33 and Genesis 26 has 36 verses** (all of Gen 26 is shifted by one); Gen 13:4 "which he
  had make there" (typo in the source).

## G04 — Hebrew (modern), Biblica Open Hebrew Living New Testament, real PDF, eBible.org

* Document: `G04_he_ebible_heblb_a4.pdf` = https://ebible.org/pdf/heblb/heblb_a4.pdf
  (bytes unchanged; sha256 7b22c964…d813; built 2026-10-30 by eBible's XeTeX pipeline,
  xdvipdfmx 20250410; A4, 399 pages, right-to-left).
* Licence: © 2009 Biblica, Inc., CC BY-SA 4.0 (copr.htm in the USFM zip).
* Reference: eBible's own USFM zip https://ebible.org/Scriptures/heblb_usfm.zip of the same
  build (2026-10-30), **unchanged** (sha256 ab98a531…89bf).
* Versification: eBible's (KJV-like with bridged verses, e.g. Mt 15:5-6 printed "6-5").
  27 books, 260 chapters, 7727 verse entries. The USFM also uses `\fl` and `\cp`.
* Checks: (a) structure from the USFM; printed numbers: 7392 agree, 268 differ only because
  bridged/RTL numbers ("6-5") and book starts are extracted differently. (b) 7660 exact +
  23 gapped of 7727 (99.4 %) in the text layer (pdftotext default mode); the 44 not located
  contain pointed names or LRM marks (e.g. Mt 4:18 ו‎אַנְדְּרֵי) whose glyph order the text
  layer scrambles; page images of pp. 5 and 8 read (Mt 1:1–20, 3:12–4:5 agree). Verbatim
  sample 2/376 only: the extracted RTL text reorders punctuation, so this test is not
  meaningful for this file. (c) No independent encoding of this edition reachable.
* Oddity: the PDF itself shows mirrored/misplaced parentheses in some lines (bidi).

## G05 — German, Luther 1545 (letzte Hand), rendered PDF (GNU groff)

* Source: Zefania XML "SF_2012-08-14_DEUTSCH_LUT_1545_LH" from
  https://github.com/kohelet-net-admin/zefania-xml-bibles (commit 7f2c4322d944932f08785931fc0463dcf2aa8d5b),
  sha256 131bb39d…9d88; digitisation of lutherbibel.net; rights field: "Umsonst habt ihrs
  empfangen, umsonst gebet es auch." Text of 1545: public domain. Not used before (Luther
  1912 was).
* Books: GEN, RUT, PSA, ROM and the Apocrypha JDT, WIS, TOB, SIR, BAR, 1MA, 2MA, MAN.
* Reference: `tools/conv_zefania.py` (VERS → `\v`, CAPTION → `\s1`, NOTE → `\f`, NOTE whose
  text is only references → `\x`): 594 glosses `\f`, 443 cross-reference notes `\x`, 7
  captions. 12 books, 358 chapters, 8930 verses. Versification: Luther/Hebrew
  (psalm titles are verse 1).
* Document: `tools/render_g05_luther_groff.py` → groff 1.23.0 `-ms -Tpdf`, A5, 394 pages.
* Checks: (a) clean. (b) 8930/8930 found in the PDF text (8381 exact, 549 interrupted by
  footnotes/column breaks), printed numbers agree for 8379 of the 8381 checked (2 where a
  footnote mark follows the number); verbatim sample 216/247 (others broken by hyphenation/footnotes);
  page images pp. 3 and 120 read. (c) Zefania 2009 encoding of the same digitisation
  (SF_2009-01-20_GER_LUT_1545_LH): 8928 common, letters agreement 95.07 % — the 2009 file
  prints the marginal notes inside the verse text; 2 verses only in the reference.
* Source oddities (kept): **1 Macc 5:41 carries the text of 2 Macc 15:22** ("HERR Gott / Da
  dich die Boten des königes Sennaherib lesterten …") and the true 5:41 follows as 5:42;
  Ps 96:14 is absent from the 2009 file. Old spelling (vnd, auff, Himel) as in 1545.

## G06 — German, Leander van Ess New Testament (Sulzbach 1859), rendered DOCX (npm docx)

* Source: Zefania "SF_2012-07-12_GER_VANESS_(LEANDER VAN ESS_REV2)" (same repository/commit),
  sha256 470af3be…0776; source field: "Verlag der I.E. v. Seidelschen Buchhandlung 1859
  Sulzbach; digitalisiert: Hans J. Herbst 2012"; rights: "public domain".
* Reference: `tools/conv_zefania.py`; 27 books, 260 chapters, 7949 verses; 528 notes `\f`,
  47 `\x`, 234 captions `\s1`.
* Document: `tools/render_g06_vaness_docx.js` (node 22.23.4, npm docx 9.15.2): 592 real
  Word footnotes.
* Checks: (a) clean; printed numbers 7949/7949. (b) 7949/7949 exact; verbatim 375/376 (one
  verse split by a footnote reference run). (c) none reachable.

## G07 — French, Fillion Bible (deuterocanonical books and Greek parts), rendered PDF (WeasyPrint)

* Source: Zefania "SF_2010-05-15_FRE_FILL_(LA BIBLE FILLION DE LOUIS CLAUDE FILLION)",
  sha256 3099d429…e4a6e541; source www.magnificat.ca; L.-Cl. Fillion (1843–1927) — public
  domain. (The Vigouroux file was rejected: its books are mislabelled, e.g. "Tobit" holds
  Wisdom, and Daniel 3:24–90 is crammed into one verse.)
* Books: EST (16 ch.), DAN (14 ch.), TOB, JDT, WIS, SIR, BAR, 1MA, 2MA; Vulgate numbering
  (Dan 3 has 100 verses). 167 chapters, 5165 verses.
* Reference: `tools/conv_zefania.py`; 4 verse-number typing errors in the source corrected
  and reported (Est 9 "30, 3, 32" → 31; Dan 3 "43, 45, 45" → 44; Sir 14 "26, 17" → 27;
  1 Mac 13 "39, 46, 41" → 40).
* Source oddity (kept): Sir 29:16 reads ", 17, 18. Mieux que le bouclier …" — the source
  joins verses 16–18 under number 16 and leaves the numbers in the text; there is no 29:17–18.
* Document: `tools/render_g07_fillion_weasy.py` → WeasyPrint 70.0, 6×9 in.
* Checks: (a) clean. (b) PDF text layer: 5116/5165 found (4892 exact, 224 gapped); 49 not
  located are text-layer artefacts of hyphenated words at page breaks and running heads;
  the HTML given to WeasyPrint: 5165/5165 exact, numbers 5164/5165 (Sir 29:16, below); verbatim sample 212/212 (renderer input); page 120 image
  read (Sir 8:7–9:11). (c) none reachable.

## G08 — Spanish, Reina-Valera New Testament 1862 (SPCC; BFBS printing 1884), rendered DOCX (LibreOffice)

* Source: Project Gutenberg #5879 https://www.gutenberg.org/ebooks/5879.txt.utf-8
  (sha256 3a178acf…1337; updated 2012-10-25), "Public domain in the USA"; transcription
  from valera1909.com whose notice ("no tiene derechos reservados") and introduction are
  reproduced on the first pages as they ask. Related to, but not the same edition as, the
  used Reina-Valera 1865/1909.
* Reference: `tools/conv_pg5879_txt.py` from the file's markup (title blocks, "CAPITULO n.",
  numbered verse paragraphs). Reported (`G08_converter_report.txt`): 20 misprinted verse
  numbers renumbered by sequence (e.g. Mt 1:24 printed "21", Acts 4:35 "357", John 18:27
  "2'7"); Rev 18:21 printed "2l" inside verse 20 — split there; Mt 15:1 printed without a
  number; 14 closing subscriptions of the epistles ("Fué escrita de Corinto …") → `\s1`
  (not verses); 16 unnumbered mid-verse paragraphs joined to their verse.
  Kept as printed: supplied words in brackets [ ], the transcribers' correction marks "^".
* Versification: as printed; 27 books, 260 chapters, 7956 verses (Acts 19 has 40 verses —
  the words of 19:41 stand unnumbered at the end of 19:40; 2 Cor 13 has 13; 3 John 15).
* Document: `tools/render_g08_rv1862_lo.py` writes flat ODF; LibreOffice 24.2.7.2 saves it
  as Word 2007 XML.
* Checks: (a) clean; printed numbers 7956/7956 in the DOCX. (b) 7956/7956 exact in the DOCX;
  against the source text file: 7956/7956 exact, the 20 renumbered verses show as number
  mismatches there; verbatim sample 376/376. (c) none reachable.

## G09 — Korean, 셩경 개역 (Korean Revised Bible, Seoul 1938), rendered PDF (Typst)

* Source: ko.wikisource proofread pages "페이지:셩경 개역.pdf/446–452, 1426–1437, 1850–1867,
  1912–1918, 1934–1941, 1946–1947" fetched as raw wikitext (`tools/fetch_wikisource_pages.sh`;
  concatenated sha256 e32d0458…c86a). Edition: printed 1 Sept 1938, publisher 聖書公會
  (British and Foreign Bible Society, Keijō). Wikisource licence section: public domain
  (PD-old-80). Not used before (the 1961 개역한글 and the 1910/11 Bible were).
* Page quality: Ruth and the NT pages are level 1 (transcribed, not proofread); Jonah and
  Micah level 3 (proofread). Transcription errors cannot be excluded for level-1 pages.
* Books: RUT, JON, MIC, PHP, COL, 1TH, JAS, 1JN, JUD — 39 chapters, 764 verses; chapter and
  verse structure identical to the KJV counts.
* Reference: `tools/conv_wikisource_kaeyok1938.py`: `{{절|C|V}}` (Chinese numerals) → `\c`/`\v`;
  `<ref>` (named and reused) → `\f`; underline/size templates dropped (text kept); `{{upe}}`
  at a page end joins the word across the page; the printed paragraph sign 〇 kept in the text;
  archaic Hangul (U+1100 jamo sequences, e.g. ᄯᅡ) kept as transcribed.
* Document: `tools/render_g09_kaeyok_typst.py` → Typst 0.15.0 (Python package), B5, 27 pages;
  verse numbers printed in Chinese numerals (一〇 = 10) as in the 1938 printing, chapter
  headings 뎨일쟝 … (generated in the edition's style; the transcription has one variant
  heading "뎨샴쟝").
* Checks: (a) clean. (b) PDF text: 743/764 found (21 not located: archaic-jamo sequences
  extracted differently); Typst source: 764/764 (708 exact, 56 interrupted only by inline
  footnote text), Chinese-numeral verse numbers 707 of 708 checked (Jas 2:11: the numerals of
  a preceding cross-reference note precede it); verbatim 75/78 (3 split by notes); page 4 image read
  (Ruth 2:7–3:13, archaic syllables rendered). (c) none reachable (the 1961 revision is a
  different edition).

## G10 — Hebrew, Miqra according to the Masorah (MAM), rendered PDF (LibreOffice Writer)

* Source: https://github.com/openscriptures/morphhb `MAPM/MAPM.xml` (commit
  3d15126fb1ef74867fc1434be1942e837932691f; sha256 2cfda0e0…68d0; OSIS by David Troidl,
  2014-02-19; text by Seth (Avi) Kadish), CC BY-SA 3.0 (readme and `<rights>`).
* Books: SNG, RUT, LAM, ECC, EST, JON — 43 chapters, 793 verses (Hebrew versification).
* Reference: `tools/conv_mapm_osis.py`: verse milestones → `\v`; ketiv segment printed in
  parentheses before the qere "(K) Q" (MAM's display convention) — 58 cases; editorial
  symbol ♫ (Ecc and Est, 2) not printed and not in the reference; full pointing and
  cantillation kept.
* Document: `tools/render_g10_mam_lo_pdf.py` (re-runs the converter and requires byte
  equality with the reference) → flat ODF → LibreOffice 24.2 PDF export, 17×24 cm, RTL,
  Hebrew-letter chapter and verse numbers.
* Checks: (a) clean. (b) Flat-ODF input: 793/793 exact, Hebrew-letter numbers 790/793 (3
  verses beginning with a parenthesised ketiv confuse the number pattern); verbatim
  86/86; page images pp. 3 and 20 read. **PDF text layer**: LibreOffice's text layer for
  pointed Hebrew is poor — `pdftotext -raw` gives fragments in visual order; default mode
  gives logical order but detaches points; letters-without-marks check finds 394/793
  verses in order and the consonantal skeleton of 696/793 verses somewhere. This is a
  property of the document, noted for the evaluation. (c) Zefania
  "SF_2014-07-09_HEB_MAPM" (from the same MAM project, unaccented): 793 common, agreement
  99.87 % without cantillation marks; the one difference is Lam 5:22, where the Zefania file
  appends the liturgical repetition of 5:21.

## G11 — Greek, Nestle 1904 New Testament (all books except Mark), rendered Markdown

* Source: https://github.com/biblicalhumanities/Nestle1904 `xml/` (commit
  713f28a3b7d4d66132f5aa809fa223fe79762e5d; tree 3bd64c9f…); XML markup CC BY-SA 4.0
  (J. Robie); base text by Diego Renato dos Santos, declared public domain. Not used before.
* Reference: `tools/conv_nestle1904_xml.py` (milestones → `\v`, `<pc>` attached, text-critical
  signs `[[ ]] [ ] < >` printed as in the edition). Mark is excluded because the XML encodes
  the shorter ending ("ΑΛΛΩΣ") as an unnumbered chapter "99". 26 books, 244 chapters,
  7269 verses.
* Document: `tools/render_g11_nestle_md.py` writes the Markdown itself (re-converts and requires
  byte equality with the reference).
* Checks: (a) clean; (b) 7269/7269 exact, numbers 7269/7269, verbatim 351/351. (c) dos Santos'
  XHTML base text (same repository, `tools/parse_nestle_xhtml.py`): 7203 common, letters
  agreement 99.26 %; the 66 verses only in the reference and most differences come from
  verse numbers the XHTML does not mark separately (verses at section starts, e.g. Mt 2:1,
  3:13, 4:1), so neighbouring verses are merged there.

## G12 — English, George R. Noyes' translation, rendered EPUB (pandoc) — eBible

* Source: https://ebible.org/Scriptures/engnoy_usfm.zip (sha256 5dc89d98…e9e5c, 2026-10-30),
  "The Holy Bible, translated into English by George Noyes (1869)", public domain.
* Reference: the eBible zip reduced to JOB, PSA, PRO, ECC, SNG, MAT, MRK, LUK, JHN
  (`tools/reduce_ebible_zip.py`, files byte-identical, plus copr.htm). 332 chapters,
  8563 verses, 13 empty in the source.
* Document: `tools/render_g12_noyes_pandoc.py` → pandoc 3.1.3 EPUB 3.
* Checks: (a) clean; printed numbers 8548/8550 (Mt 5:4–5, below). (b) 8550/8550 non-empty
  exact; verbatim 215/215. (c) Zefania "SF_2009-01-20_ENG_NOYES" (Online Bible origin):
  8563 common, letters agreement 99.94 % (5 differences: e.g. Psalm book headings "BOOK I"
  inside eBible's Ps 1:1 and 42:1).
* Source oddities (kept): Mt 5:4 and 5:5 carry "(5:5)"/"(5:4)" in the text (Noyes' order of
  the beatitudes); psalm titles are in quotation marks inside verse 1.

## G13 — English, Unlocked Literal Bible (selection), rendered PDF (ReportLab) — eBible

* Source: https://ebible.org/Scriptures/engULB_usfm.zip (sha256 fc8c949f…d278, 2026-10-30),
  © 2017 Door43 World Missions Community, CC BY-SA 4.0 (copr.htm).
* Reference: eBible zip reduced to RUT, JON, MRK, ROM, PHM, JUD (byte-identical files).
  42 chapters, 1294 verses (6 empty in the source), 18 footnotes.
* Document: `tools/render_g13_ulb_reportlab.py` → ReportLab 5.0.1, Letter, two columns.
* Checks: (a) clean; numbers 1264/1264. (b) 1288/1288 non-empty found (1264 exact, 24
  interrupted by column/page breaks or note letters); verbatim 70/70; page 3 image read
  (Ruth 2:15–4:12). (c) none reachable.

## G14 — Latin, Castellio's Latin Bible (1551), transcription with macrons, rendered EPUB (EbookLib)

* Source: https://github.com/njand/biblia-castellionis LaTeX sources at commit
  0f934763c92c824fe25f0ce56bf73194878d43fc (2023-04-02, before the project's 2026 changes);
  files: genesis.tex 1079e671…0b80, matthaeus.tex 19e7e8c4…d2f7, marcus.tex 5238cd5e…a3b,
  lucas.tex 188009fb…b568, johannes.tex 3c6255df…a2c8. Licence: CC BY-SA 4.0 (repository
  licence); attribution page in the EPUB.
* Books/extent as transcribed: Genesis 1–27, Matthew 1–13, Mark 1–5, Luke 1–13, John 1–4
  (62 chapters, 2244 verses).
* Reference: `tools/conv_castellio_latex.py` (`\Caput`/`\Versus` counters, `\emph` removed with
  brace matching, TeX quotes → “ ” ‘ ’, `\-` removed, commented summaries/footnotes not printed).
* Versification: the project's own verse division, which differs from the KJV in 10 Genesis
  chapters (e.g. Gen 11 has 34 verses, Gen 14 25).
* Document: `tools/render_g14_castellio_ebooklib.py` → EbookLib 0.20.
* Checks: (a) clean; numbers 2244/2244. (b) 2244/2244 exact; verbatim 93/93. (c) the project's
  own plain-text files of the same commit (`TXT/4 - Macronized`, `tools/parse_castellio_txt.py`):
  1674 common verses, letters agreement 99.04 %; the TXT lags the LaTeX for the NT (570
  verses only in the reference) and has Genesis 28–50 (267 only in the TXT); 16 textual
  differences are verse-boundary shifts (e.g. Gen 15:18–20).

---

## Reuse and licence summary

* Translation reuse: G03 KJV (eng-kjv2006 used earlier; this is a different encoding and a
  real published PDF). Related but different editions: G05 Luther 1545 (Luther 1912 used),
  G08 Reina-Valera 1862 (RV 1865 and 1909 used), G09 개역 1938 (its 1961 revision used),
  G10 MAM (WLC used; a different Masoretic edition). No eBible id from the exclusion list
  is used; eBible-derived documents: G04 (real file), G12 and G13 (rendered) — 2 of 14
  rendered from eBible text.
* Non-eBible publishers of real files: Project Gutenberg (G01, G02), BibleTeX/vermiculus (G03).
* Licences: public domain G01, G02, G03 (text; layout CC BY-SA), G05, G06, G07, G08, G09, G12;
  CC BY-SA: G04 (4.0), G10 (3.0), G11 (markup 4.0, text PD), G13 (4.0), G14 (4.0).

## Build environment notes

* Programs installed for this set: groff (apt), Python packages reportlab, typst, ebooklib,
  lxml (pip), npm package docx (in sources/nodeenv). Already present: pandoc 3.1.3,
  LibreOffice 24.2.7.2, WeasyPrint 70.0, poppler-utils.
* Network sources: www.gutenberg.org, ebible.org, github.com (git clone), api.github.com
  (search), raw.githubusercontent.com, ko.wikisource.org. es.wikisource.org was refused by
  the proxy; ko.wikisource's API was rate-limited, so raw page wikitext was fetched instead.
* Rejected candidates: Zefania Vigouroux 1902 (mislabelled books, crammed Daniel 3),
  Zefania Antoniades (unaccented Strong's-tagged text), Gutenberg #10 KJV txt (KJV already
  represented by G03), OET Ruth PTXPrint proofs (two parallel versions per verse),
  the 2026 regenerated Castellio EPUB (G14 uses the 2023 LaTeX instead).
