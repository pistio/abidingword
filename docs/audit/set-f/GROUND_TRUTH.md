# Set F - independent ground truth

Builder: independent test builder (Claude), 2026-11-01. Everything here was made from public sources downloaded on 2026-11-01; the program under test was not seen or run.

General rules for all references:

- One USFM file per book in each `*.usfm.zip`, standard book codes, `\c` and `\v` numbering identical to the document. Verse text is the publisher's text unchanged (only white space collapsed).
- For eBible.org sources the reference files are the eBible USFM files themselves, copied byte for byte (only the books the document holds); they keep all of eBible's own markup (`\w ...|strong=...\w*`, `\f`, `\x`, `\s1`, `\d`, `\cl`, `\vp`, ...). Notes, headings and labels in them are marked as such and are not verse text.
- For other sources the reference was produced by `tools/scroll2usfm.py` from the source JSON: `\id`, `\h`, `\toc1-3`, `\mt1`, `\c`, (`\cp`), `\p`, `\v`. The source carries no book titles: the titles in `\h/\toc/\mt1` are standard printed titles in the language, supplied by the builder (`tools/booknames.py`), and are the titles the document prints. Verses whose source text is empty are left out of both document and reference (listed per document).
- Book titles, chapter headings ("Kapitel 3", "CHAPTER III.", "제 3 장", "ΚΕΦΑΛΑΙΟΝ Γʹ", ...), running heads, page numbers, contents lists, title pages, introductions and division pages are layout, not verse text.
- `tools/verify.py` (builder self-check) confirmed for every document that each reference verse text occurs in the document's extracted text in order (deviations only where a heading, a note caller or a footnote sits inside a verse, or right-to-left extraction order).

## F01 - F01-eng-nheb.pdf

- Reference: `F01-eng-nheb.usfm.zip` (29 books, 11328 verses).
- Source: New Heart English Bible (NHEB), Wayne A. Mitchell; text from scrollmapper/bible_databases (2025 schema, master branch, formats/json/NHEB.json, retrieved 2026-11-01); public domain (the NHEB is dedicated to the public domain by its editor; scrollmapper lists its licence as "Unknown"; repository code MIT).
- Source kind: other (not eBible USFM). URL: https://raw.githubusercontent.com/scrollmapper/bible_databases/master/formats/json/NHEB.json
- Source file SHA-256: `45c42baa7053244f244aed3fe89214dc10c13c70ffec6338b8d2d01087c7f6d4` (NHEB.json)
- Reference made: converted: tools/scroll2usfm.py NHEB.json eng PSA,PRO,<27 NT books> (one \p per chapter; book titles supplied, see note).
- Versification (document and reference): English (KJV-type) versification as in the source.
- Document generator: PyMuPDF 2.0 (MuPDF 1.28.5) Story layout engine; tools/make_f01_nheb_mupdf.py.
- Known gaps and oddities in the source text (not fixed):
  - Verses whose text is empty in the source (7; left out of document and reference, so the numbering skips them): MRK 11:26, 15:28; LUK 17:36, 23:17; ACT 15:34, 24:7; 3JN 1:15.
  - REV 14:1 begins "4I saw, ..." - a stray digit 4 is part of the source verse text; kept unchanged.
  - 13 verses of these books contain bracketed text [...] in the source (e.g. HEB 2:8); kept as printed.

## F02 - F02-eng-leeser.pdf

- Reference: `F02-eng-leeser.usfm.zip` (39 books, 23145 verses).
- Source: Isaac Leeser Tanakh (1853), eBible.org englee, USFM zip generated 30 Oct 2026 from source files dated 12 Dec 2025; Public Domain.
- Source kind: eBible.org USFM. URL: https://ebible.org/Scriptures/englee_usfm.zip
- Source file SHA-256: `6c0ffbe7d24fd9d4b0c6bc4f4a9ef3f0474dfcd82c7aab745bc3f420201d59a4` (englee_usfm.zip)
- Reference made: reduced: tools/reduce_ebible.py englee_usfm.zip (39 OT book files copied byte for byte; copr.htm and other non-USFM files left out).
- Versification (document and reference): English versification; where the Hebrew numbering differs the source puts the Hebrew chapter:verse in parentheses at the start of the verse text, e.g. MAL 4:1 "(3:19) For, behold, ..."; JOL 3 chapters, MAL 4 chapters.
- Document generator: LibreOffice Writer 24.2.7.2 (headless PDF export of a generated flat ODF .fodt); tools/make_f02_leeser_lo.py.
- Known gaps and oddities in the source text (not fixed):
  - JOS 21:36 and 21:37 are empty in the source (\v 36 and \v 37 with no text) and 21:38 begins "(21:36) And from the tribe of Gad"; the document prints "36." and "37." with nothing after them, as the reference has them.
  - About 770 verses carry the Hebrew numbering in parentheses inside the verse text (see versification); they are verse text in the source and in the document.
  - Double square brackets occur in the source text, e.g. GEN 2:23 "Woman, [[Ishah,]] because out of Man [[Ishi]]"; kept.

## F03 - F03-deu-menge.pdf

- Reference: `F03-deu-menge.usfm.zip` (76 books, 35436 verses).
- Source: Menge-Bibel (Hermann Menge, 1939 text) with Apocrypha, CrossWire module GerMenge via scrollmapper/bible_databases (formats/json/GerMenge.json, master, retrieved 2026-11-01); Public Domain (translator died 1939; licence field "Public Domain").
- Source kind: other (not eBible USFM). URL: https://raw.githubusercontent.com/scrollmapper/bible_databases/master/formats/json/GerMenge.json
- Source file SHA-256: `de8be8743f521d535966f075577fcfb5b815c63c9e141c06179ca33cd9016b09` (GerMenge.json)
- Reference made: converted: tools/scroll2usfm.py GerMenge.json deu all (one \p per chapter; book titles supplied; book order kept).
- Versification (document and reference): The source (CrossWire) maps Menge onto a KJV-type versification with empty slots; the empty slots are not printed. Apocrypha: Tobit/Judith/Wisdom/Sirach/Baruch/1-2 Maccabees as in the module; "Stücke zu Ester" (ESG) in 7 chapters and "Stücke zu Daniel" (DAG) in 3 chapters as the module numbers them (not the Vulgate 10-16 / 13-14 chapter numbers); Gebet Manasses (MAN) 1 chapter.
- Document generator: ReportLab 5.0.1 Platypus (multiBuild, onDraw callbacks for running heads); tools/make_f03_menge_reportlab.py.
- Known gaps and oddities in the source text (not fixed):
  - Verses whose text is empty in the source (333; left out of document and reference, so the numbering skips them): EXO 35:19; NUM 25:19; 2KI 15:39; NEH 7:73; EZK 48:22; JDT 3:11, 3:12, 5:25, 5:26, 9:15, 12:21, 13:21, 13:22, 13:23, 13:24, 13:25, 13:26, 13:27, 13:28, 13:29, 13:30, 13:31, 15:14, 15:15, 15:16, 16:26, 16:27, 16:28, 16:29, 16:30, 16:31; WIS 2:25, 5:24, 6:27, 9:19, 17:21; TOB 1:23, 1:24, 1:25, 2:15, 2:16, 2:17, 2:18, 2:19, 2:20, 2:21, 2:22, 2:23, 3:18, 3:19, 3:20, 3:21, 3:22, 3:23, 3:24, 3:25, 4:22, 5:22, 5:23, 5:24, 5:25, 5:26, 5:27, 5:28, 5:29, 6:18, 6:19, 6:20, 6:21, 6:22, 6:23, 7:19, 7:20, 8:22, 8:23, 9:7, 9:8, 9:9, 9:10, 9:11, 9:12, 11:19, 11:20, 13:19, 13:20, 13:21, 13:22, 14:16, 14:17; SIR 1:28, 1:29, 1:30, 1:31, 1:32, 1:33, 1:34, 1:35, 1:36, 1:37, 1:38, 2:19, 2:20, 2:21, 2:22, 2:23, 3:30, 3:31, 3:32, 3:33, 3:34, 4:32, 4:33, 4:34, 4:35, 4:36, 5:17, 5:18, 6:37, 7:37, 7:38, 7:39, 7:40, 8:20, 8:21, 8:22, 9:19, 9:20, 9:21, 9:22, 9:23, 9:24, 9:25, 10:32, 10:33, 10:34, 11:32, 11:33, 11:34, 11:35, 12:19, 13:27, 13:28, 13:29, 13:30, 13:31, 13:32, 15:21, 16:29, 16:30, 17:28, 17:29, 17:30, 17:31, 18:33, 20:31, 20:32, 20:33, 21:29, 21:30, 21:31, 22:26, 22:27, 22:28, 22:29, 22:30, 22:31, 22:32, 22:33, 23:28, 23:29, 23:30, 23:31, 23:32, 23:33, 23:34, 23:35, 23:36, 23:37, 24:33, 24:34, 24:35, 24:36, 24:37, 24:38, 24:39, 24:40, 24:41, 24:42, 24:43, 24:44, 24:45, 24:46, 24:47, 25:26, 25:27, 25:28, 25:29, 25:30, 25:31, 25:32, 25:33, 25:34, 26:21, 26:22, 26:23, 26:24, 26:25, 26:26, 26:27, 26:28, 27:31, 27:32, 27:33, 28:27, 28:28, 28:29, 28:30, 29:29, 29:30, 29:31, 29:32, 29:33, 29:34, 29:35, 30:26, 30:27, 31:32, 31:33, 31:34, 31:35, 31:36, 31:37, 31:38, 31:39, 31:40, 32:24, 32:25, 32:26, 32:27, 32:28, 34:27, 34:28, 34:29, 34:30, 34:31, 35:21, 35:22, 35:23, 35:24, 35:25, 35:26, 36:27, 36:28, 37:32, 37:33, 37:34, 38:35, 38:36, 38:37, 38:38, 38:39, 39:36, 39:37, 39:38, 39:39, 39:40, 39:41, 40:31, 40:32, 41:25, 41:26, 41:27, 41:28, 41:29, 42:26, 43:34, 43:35, 43:36, 43:37, 44:24, 44:25, 44:26, 45:27, 45:28, 45:29, 45:30, 45:31, 45:32, 46:21, 46:22, 46:23, 47:26, 47:27, 47:28, 47:29, 47:30, 47:31, 48:26, 48:27, 48:28, 49:17, 49:18, 49:19, 49:20, 50:30, 50:31, 51:31, 51:32, 51:33, 51:34, 51:35, 51:36, 51:37, 51:38; BAR 3:38; 1MA 1:65, 1:66, 1:67, 1:68, 13:54; 2MA 2:33, 7:42, 12:46, 15:40; ESG 2:8, 3:12, 5:15, 5:16; MAN 1:16; MRK 9:44, 9:46; LUK 17:36; ACT 24:7.
  - Most empty slots are at the ends of chapters where the module's versification has more verses than Menge's text (Sirach alone has 225, e.g. SIR 1:28-38); a few are inside chapters, where the number is simply skipped.
  - 192 verses contain bracketed text [...] in the source; kept.

## F04 - F04-fra-martin.epub

- Reference: `F04-fra-martin.usfm.zip` (66 books, 31166 verses).
- Source: Bible David Martin (1744), CrossWire module FreBDM1744 via scrollmapper/bible_databases (formats/json/FreBDM1744.json, master, retrieved 2026-11-01); Public Domain.
- Source kind: other (not eBible USFM). URL: https://raw.githubusercontent.com/scrollmapper/bible_databases/master/formats/json/FreBDM1744.json
- Source file SHA-256: `d1e3f2e4bf0af1546ff76388988c7213ff2a6d7ee216ed65c5a08a53f59a7ea9` (FreBDM1744.json)
- Reference made: converted: tools/scroll2usfm.py FreBDM1744.json fra all.
- Versification (document and reference): French Protestant / KJV-type versification as in the source (Psalm titles are part of verse 1).
- Document generator: EbookLib 0.20 (EPUB 3 with NCX); tools/make_f04_martin_ebooklib.py.
- Known gaps and oddities in the source text (not fixed):
  - Verses whose text is empty in the source (4; left out of document and reference, so the numbering skips them): PSA 69:37, 102:29, 108:14, 140:14.
  - PSA 102:1 contains a merged following verse with its own number inside the text: "... devant l'Eternel. 2.Eternel! écoute ma prière ..."; kept as the source has it (the number "2." is verse text of 102:1 in source and reference).
  - OCR-type errors in the source text are kept: EXO 35:21 "1'offrande", 1SA 9:9 "0r", 1SA 11:13 "0n", PSA 136:17 "I1", 1CO 3:15 "1'oeuvre", HEB 10:7 "voic1", HEB 13:8 "aujourd'hu1" (digit one / zero for letters).
  - Slash characters and [bracketed] words occur in the source text and are kept.

## F05 - F05-spa-rv1865.docx

- Reference: `F05-spa-rv1865.usfm.zip` (66 books, 31098 verses).
- Source: La Santa Biblia Reina-Valera (1865) con arreglos ortográficos, CrossWire module SpaRV1865 via scrollmapper/bible_databases (formats/json/SpaRV1865.json, master, retrieved 2026-11-01); Public Domain.
- Source kind: other (not eBible USFM). URL: https://raw.githubusercontent.com/scrollmapper/bible_databases/master/formats/json/SpaRV1865.json
- Source file SHA-256: `bce26de39969fbf16d7970ba29be7724ac738c5ac7639aaa55ea3f20819d75bb` (SpaRV1865.json)
- Reference made: converted: tools/scroll2usfm.py SpaRV1865.json spa all --pilcrow (a verse beginning with ¶ starts a new \p and loses the sign).
- Versification (document and reference): KJV-type versification as in the source.
- Document generator: python-docx 1.2.0; tools/make_f05_rv1865_pythondocx.py.
- Known gaps and oddities in the source text (not fixed):
  - Verses whose text is empty in the source (4; left out of document and reference, so the numbering skips them): 1CH 3:24; 2CH 33:25; PSA 13:6; JER 39:18.
  - A pilcrow (¶) inside the verse text was kept as text in both document and reference (subscription notes after the last verse of the epistles): ROM 16:27; 1CO 16:24; 2CO 13:14; GAL 6:18; EPH 6:24; PHP 4:23; COL 4:18; 1TH 5:28; 2TH 3:18; 1TI 6:21; 2TI 4:22; TIT 3:15; PHM 1:25; HEB 13:25.

## F06 - F06-por-nva.pdf

- Reference: `F06-por-nva.usfm.zip` (27 books, 7949 verses).
- Source: Bíblia Nova Versão de Acesso Livre (NVA), CrossWire module PorNVA via scrollmapper/bible_databases (formats/json/PorNVA.json, master, retrieved 2026-11-01); licence Creative Commons BY-SA 4.0 (attribution and licence notice printed on the title page; layout shared under the same licence).
- Source kind: other (not eBible USFM). URL: https://raw.githubusercontent.com/scrollmapper/bible_databases/master/formats/json/PorNVA.json
- Source file SHA-256: `636555248a52398aa138015fa02e459f7dc30a38120ce636f6f1fb70550db04f` (PorNVA.json)
- Reference made: converted: tools/scroll2usfm.py PorNVA.json por nt.
- Versification (document and reference): KJV-type NT versification as in the source.
- Document generator: fpdf2 2.9.0; tools/make_f06_nva_fpdf2.py.
- Known gaps and oddities in the source text (not fixed):
  - Verses whose text is empty in the source (10; left out of document and reference, so the numbering skips them): MAT 17:21, 18:11, 23:14; MRK 7:16, 15:28; JHN 5:4; ACT 15:34, 24:7, 28:29; ROM 16:24.
  - MAT 10:5 begins its direct speech with a stray digit: "dizei primeiro: 1'Que a paz ..." (source text, kept).
  - ROM 4:19 contains a number in digits in the source text (kept).

## F07 - F07-kor-rv.pdf

- Reference: `F07-kor-rv.usfm.zip` (66 books, 31084 verses).
- Source: 개역성경 (Korean Revised Version, 1961 개역한글 text), CrossWire module KorRV via scrollmapper/bible_databases (formats/json/KorRV.json, master, retrieved 2026-11-01); licence field "Public Domain". REUSE NOTE: this is very probably the same translation as the earlier sets' eBible id kor1961; it is used here only because no other Korean text is available, from a different source/encoding and in a completely different layout.
- Source kind: other (not eBible USFM). URL: https://raw.githubusercontent.com/scrollmapper/bible_databases/master/formats/json/KorRV.json
- Source file SHA-256: `ffdafc000a0555f87c889c03638b631ff27d033fe9aed0ef89b38d04b576c4bf` (KorRV.json)
- Reference made: converted: tools/scroll2usfm.py KorRV.json kor all.
- Versification (document and reference): Korean Protestant (KJV-type) versification as in the source.
- Document generator: Typst 0.15.0 (python package typst 0.15.0); tools/make_f07_korrv_typst.py.
- Known gaps and oddities in the source text (not fixed):
  - Verses whose text is empty in the source (20; left out of document and reference, so the numbering skips them): PSA 72:20; ISA 30:2, 48:2; JER 21:2; EZK 24:5; MAT 18:11; MRK 9:44, 9:46, 11:26, 15:28; LUK 17:36, 23:17; ACT 8:37, 15:26, 15:34, 28:29; ROM 9:2, 16:24; 2CO 13:14; REV 12:18.
  - JOS 2:13 text is "12절과 상동" and ISA 7:9 text is "8절과 같음" ("same as verse 12/8") in the source; printed as verse text.
  - Where the source leaves a verse empty the preceding verse presumably carries the merged text (e.g. ISA 30:1 is followed by 30:3); the reference simply has no \v for the empty number.
  - Bracketed passages, e.g. MRK 16:9-20 in [ ], are kept as in the source.

## F08 - F08-heb-salkinson.docx

- Reference: `F08-heb-salkinson.usfm.zip` (27 books, 7957 verses).
- Source: Salkinson-Ginsburg Hebrew New Testament (1886), eBible.org hebsg, USFM zip generated 30 Oct 2026 from source files dated 12 Dec 2025; Public Domain.
- Source kind: eBible.org USFM. URL: https://ebible.org/Scriptures/hebsg_usfm.zip
- Source file SHA-256: `71f137fd188b81b12e1379e52389bb4ab32efee6deb863a0ebad35102c535986` (hebsg_usfm.zip)
- Reference made: reduced: tools/reduce_ebible.py hebsg_usfm.zip nt (27 book files copied byte for byte; the INT introduction file, printed as front matter, holds no verses and is not in the reference).
- Versification (document and reference): Standard (KJV-type) NT versification in the \v markers; the document prints the \vp Hebrew-letter verse numbers and \cl/\cp Hebrew-letter chapter labels that the reference also carries.
- Document generator: LibreOffice Writer 24.2.7.2 (headless export "MS Word 2007 XML" of a generated flat ODF .fodt); tools/make_f08_salkinson_lodocx.py.
- Known gaps and oddities in the source text (not fixed):
  - 3JN has 14 verses in this source.

## F09 - F09-grc-tcgnt.md

- Reference: `F09-grc-tcgnt.usfm.zip` (27 books, 7954 verses).
- Source: The Text-Critical Greek New Testament: Byzantine Text (Robert Adam Boyd, based on Robinson-Pierpont Byzantine Textform 2018), eBible.org grctcgnt, USFM zip generated 31 Oct 2026; Public Domain. NOTE: the Greek text is closely related to the Robinson-Pierpont Byzantine text of earlier sets (grcbyz); this is a separate edition with its own text-critical apparatus.
- Source kind: eBible.org USFM. URL: https://ebible.org/Scriptures/grctcgnt_usfm.zip
- Source file SHA-256: `b112e596999cf981e7cb79cb9cbbc9eea0bc1eb64223f9f4372a1682769f81b7` (grctcgnt_usfm.zip)
- Reference made: reduced: tools/reduce_ebible.py grctcgnt_usfm.zip nt (27 book files copied byte for byte; FRT/INT printed as front matter, appendix XXA not printed; neither is in the reference).
- Versification (document and reference): Byzantine/standard NT versification of the source: LUK 17:36, ACT 8:37, 15:34, 24:7 are absent (not in the Byzantine text); the Romans doxology stands at ROM 14:24-26 and ROM 16 ends at 16:24.
- Document generator: plain Python writer (CommonMark + footnote syntax); tools/make_f09_tcgnt_md.py.
- Known gaps and oddities in the source text (not fixed):
  - Book-final \pc paragraphs ("Ἀμήν.") have no verse marker of their own: they belong to the last verse of the book (USFM continuation), and the document prints them after that verse as a centred line.
  - Some section headings fall inside a verse (e.g. MRK 6:6, LUK 9:43, JHN 12:36, ACT 8:1); the heading is printed between the two parts of the verse, as in the source.

## F10 - F10-lat-sistine.epub

- Reference: `F10-lat-sistine.usfm.zip` (18 books, 5876 verses).
- Source: Vulgata Sistina (Sixtine Vulgate, 1590), CrossWire module VulgSistine via scrollmapper/bible_databases (formats/json/VulgSistine.json, master, retrieved 2026-11-01); Public Domain. NOTE: a different edition from the Clementine Vulgate (latVUC) of earlier sets, but the same Latin Vulgate tradition (closely related text).
- Source kind: other (not eBible USFM). URL: https://raw.githubusercontent.com/scrollmapper/bible_databases/master/formats/json/VulgSistine.json
- Source file SHA-256: `4bf645056a9f81529d2f015a3e83866c74073f75af11fa5d458b69922ee01443` (VulgSistine.json)
- Reference made: converted: tools/scroll2usfm.py VulgSistine.json lat ISA,JER,LAM,BAR,EZK,DAN,HOS,JOL,AMO,OBA,JON,MIC,NAM,HAB,ZEP,HAG,ZEC,MAL (Latin book titles supplied).
- Versification (document and reference): Vulgate versification: DAN 14 chapters (3:24-90 Song of the Three in chapter 3, Susanna = ch. 13, Bel and the Dragon = ch. 14), BAR 6 chapters (Letter of Jeremiah = ch. 6), JOL 3 chapters, MAL 4 chapters.
- Document generator: hand-built EPUB 2.0.1 (Python zipfile: OPF 2.0, NCX, XHTML 1.1); tools/make_f10_sistine_epub2.py.
- Known gaps and oddities in the source text (not fixed):
  - Verses whose text is empty in the source (1; left out of document and reference, so the numbering skips them): ISA 45:26.

## F11 - F11-ell-vamvas.txt

- Reference: `F11-ell-vamvas.usfm.zip` (66 books, 31102 verses).
- Source: Η Αγία Γραφή, translation of Neophytos Vamvas (1850; Katharevousa Greek), CrossWire module GreVamvas via scrollmapper/bible_databases (formats/json/GreVamvas.json, master, retrieved 2026-11-01); Public Domain.
- Source kind: other (not eBible USFM). URL: https://raw.githubusercontent.com/scrollmapper/bible_databases/master/formats/json/GreVamvas.json
- Source file SHA-256: `2f72c1e19619237b972d97429b3af91dbdf5b0a8b5187afdf0cec2eb521a9063` (GreVamvas.json)
- Reference made: converted: tools/scroll2usfm.py GreVamvas.json ell all --cp-greek (\cp carries the printed Greek numeral).
- Versification (document and reference): KJV-type versification as in the source.
- Document generator: plain Python writer; tools/make_f11_vamvas_txt.py.
- Known gaps and oddities in the source text (not fixed):
  - No empty verses and no digits in the source text.

## F12 - F12-rus-synodal.epub

- Reference: `F12-rus-synodal.usfm.zip` (66 books, 31169 verses).
- Source: Russian Synodal Translation (Синодальный перевод), eBible.org russyn, USFM zip generated 30 Oct 2026 from source files dated 12 Dec 2025; Public Domain.
- Source kind: eBible.org USFM. URL: https://ebible.org/Scriptures/russyn_usfm.zip
- Source file SHA-256: `9af20fe73fff9370acc8fd03cc0ffa718e1cf107d83fc401d41dcadc6c4e4a88` (russyn_usfm.zip)
- Reference made: reduced: tools/reduce_ebible.py russyn_usfm.zip all (66 book files copied byte for byte).
- Versification (document and reference): Russian Synodal versification as in the source (Psalms with Synodal/LXX-based numbering and titles counted as verses where the source does so; PSA has 2526 verses).
- Document generator: pandoc 3.1.3 (HTML -> epub3, --split-level=1 --toc --toc-depth=1); tools/make_f12_synodal_pandoc.py.

## F13 - F13-ita-diodati.pdf

- Reference: `F13-ita-diodati.usfm.zip` (5 books, 4784 verses).
- Source: La Sacra Bibbia, Giovanni Diodati, revised edition 1885 (Diodati Bibbia 1885), eBible.org ita1885, USFM zip generated 30 Oct 2026 from source files dated 3 May 2025; Public Domain.
- Source kind: eBible.org USFM. URL: https://ebible.org/Scriptures/ita1885_usfm.zip
- Source file SHA-256: `509c769e98c1de3c963dc84d77643581359d6e1acbccaa926b3c8a2271c61000` (ita1885_usfm.zip)
- Reference made: reduced: tools/reduce_ebible.py ita1885_usfm.zip JOB,PSA,PRO,ECC,SNG (5 book files copied byte for byte).
- Versification (document and reference): Italian Protestant (KJV-type) versification as in the source; psalm titles are \d headings, not verses.
- Document generator: WeasyPrint 70.0 (CSS paged media, float: footnote); tools/make_f13_diodati_weasy.py.
- Known gaps and oddities in the source text (not fixed):
  - In the source cross references the keyword is encoded with an empty \xk followed by \xt (e.g. "\xk \xt empi \xt Prov. 4.14,15."); notes are not verse text.

## Reuse and licence summary

- F07 (Korean): unavoidable reuse of the 1961 Korean Revised (개역한글) translation (= eBible kor1961 of earlier sets) - all Korean texts available here are either that translation, the 1910 text (kor, used) or a copyrighted text; taken from a different source (CrossWire KorRV via scrollmapper) and laid out completely differently.
- F09 (grctcgnt) and F10 (Vulgata Sistina) are different editions but closely related texts to grcbyz and latVUC of earlier sets; recorded for transparency.
- German, French, Spanish, Portuguese and Korean: every eBible.org USFM edition of these languages is either in the excluded list or restricted, so these documents use CrossWire texts distributed by scrollmapper/bible_databases.
- Licences: all texts are public domain except F06 (PorNVA, CC BY-SA 4.0 - attribution and licence notice on the document's title page). F01 NHEB: public domain by its editor's dedication (the scrollmapper licence field says "Unknown").
- Rejected during selection: grcf35 (CC BY-NC-ND, no derivatives), open-bibles por-almeida (edition and licence unclear).

