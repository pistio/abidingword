"""Write /root/aw-set-g/withheld-g.tsv (manifest of set G) with SHA-256 of each document.
Usage: python3 -I write_manifest.py
"""
import hashlib
import os

D = "/root/aw-set-g/docs"
HEADER = ["key", "files", "reference", "language", "script", "format", "layout", "role",
          "source and licence", "books", "sha256", "generator"]
ALL = "all"
NT = "MAT,MRK,LUK,JHN,ACT,ROM,1CO,2CO,GAL,EPH,PHP,COL,1TH,2TH,1TI,2TI,TIT,PHM,HEB,JAS,1PE,2PE,1JN,2JN,3JN,JUD,REV"
ROWS = [
    ("G01", "G01_pt_pg62383_biblia_sagrada.epub", "por", "Latin", "epub",
     "Project Gutenberg's own EPUB 3 (ebookmaker build, with images) of eBook #62383, transcribed by Distributed Proofreaders from the 1911 Lisbon printing: one XHTML flow per book with upper-case book titles, italic chapter summaries before each chapter, marginal chronology notes ('Antes de Christo 4004'), the chapter number in bold at the start of each chapter's first verse and every other verse a separate paragraph led by its number, centred psalm titles, Hebrew acrostic letters as headings (Ps 119) or before the number (Lamentations), about 20,000 numbered cross-reference footnote anchors '[n]' and 2138 lettered textual-note anchors '[A]'..'[AAA]' inside the verse text with the notes after each chapter/at the end, page-number anchors of the printed book, Gutenberg licence header and footer.",
     "[real file] [other] Project Gutenberg eBook #62383, Almeida translation (Lisbon 1911 printing), Public domain in the USA; Gutenberg licence kept",
     ALL, "Project Gutenberg ebookmaker EPUB3 (dcterms:modified 2026-09-24); reference by conv_pt_gutenberg62383.py"),
    ("G02", "G02_sv_pg2100_bibeln_1917.txt", "swe", "Latin", "txt",
     "Project Gutenberg plain-text file (UTF-8, CRLF, 5.1 MB) from Project Runeberg: Gutenberg licence header/footer, title page and contents list, books headed by their Swedish titles, every verse starting a line as 'CCC:VVV text' (zero-padded chapter:verse) with 8-space-indented continuation lines and blank lines between paragraphs, Psalms/poetry in indented half-lines, 17 verses printed as a bare reference with no text, two bracketed notes [1]/[2] (Mark 16:9-20, John 7:53-8:11), James placed after 3 John, some misprinted chapter numbers in the references.",
     "[real file] [other] Project Gutenberg eBook #2100, Bibeln 1917 (Project Runeberg transcription), Public domain in the USA; Gutenberg licence kept",
     ALL, "Project Gutenberg plain text (updated 2020-12-31); reference by conv_pg2100_txt.py"),
    ("G03", "G03_en_bibletex_kjv.pdf", "eng", "Latin", "pdf",
     "PDF typeset with XeLaTeX by the BibleTeX project (github vermiculus/bible): US Letter, 816 pages, two columns, running head with the book name and page number, large book titles, chapters 2.. introduced by a two-line drop numeral (lettrine), chapter 1 and every verse 1 unnumbered, superscript verse numbers run into continuous paragraphs, LORD in small capitals, divine and other speech in curly quotes (speaker colours), one chapter name heading ('In The Beginning'), Old/New Testament part pages, Gutenberg licence at the end; Zephaniah and Haggai absent.",
     "[real file] [other] vermiculus/bible commit 642de2d (bible.pdf created 2015-05-17); KJV text public domain (from Project Gutenberg), layout CC BY-SA; translation reuse (KJV)",
     "GEN,EXO,LEV,NUM,DEU,JOS,JDG,RUT,1SA,2SA,1KI,2KI,1CH,2CH,EZR,NEH,EST,JOB,PSA,PRO,ECC,SNG,ISA,JER,LAM,EZK,DAN,HOS,JOL,AMO,OBA,JON,MIC,NAM,HAB,ZEC,MAL," + NT,
     "LaTeX with hyperref, xdvipdfmx (20140317); reference by conv_vermiculus_kjv_tex.py"),
    ("G04", "G04_he_ebible_heblb_a4.pdf", "heb", "Hebrew", "pdf",
     "eBible.org's published A4 PDF (XeTeX pipeline) of the Biblica Open Hebrew Living New Testament: right-to-left modern Hebrew, single column, running heads with the book name and first/last reference, centred page numbers, large book titles, chapter numbers as big centred numerals, each verse a separate line/paragraph with an Arabic verse number at its right, some bridged verses ('6-5'), footnotes, roman-numbered front pages with the bilingual copyright/licence statement.",
     "[real file] [eBible] eBible.org heblb (build 2026-10-30), (c) 2009 Biblica, CC BY-SA 4.0",
     NT, "LaTeX with hyperref, xdvipdfmx (20250410) (eBible.org); reference = eBible heblb_usfm.zip unchanged"),
    ("G05", "G05_de_luther1545_auszug.pdf", "deu", "Latin", "pdf",
     "A5, 394 pages, Bookman, two columns: title page and book register; running head with the short book name, centred folio at the foot; book titles in old style ('Das Erste Buch Mose.'); chapter headings 'Das XII. Capitel.' / 'Das LXII. Psalm.' in Roman numerals; superscript bold verse numbers for every verse (verse 1 included) run into one paragraph per chapter; italic centred captions; Luther's marginal glosses and cross references as numbered footnotes at the foot of each column; a part title 'Apocrypha' before the deuterocanonical books.",
     "[rendered] [other] Luther 1545 (letzte Hand), lutherbibel.net digitisation via Zefania XML (kohelet-net-admin/zefania-xml-bibles); text public domain",
     "GEN,RUT,PSA,ROM,JDT,WIS,TOB,SIR,BAR,1MA,2MA,MAN",
     "GNU groff 1.23.0 (-ms, gropdf); render_g05_luther_groff.py"),
    ("G06", "G06_de_vaness_nt.docx", "deu", "Latin", "docx",
     "Word document, A4, title section then one section per book with two newspaper columns and a rule between, running header with the full book title, footer page number '– n –'; book title as Heading 1, chapter headings as ordinal words ('Siebzehntes Kapitel') in small capitals (Heading 2), van Ess's captions as italic Heading 3; verse numbers bold in parentheses '(12)' run into justified paragraphs; 592 notes as real Word footnotes (reference marks after the word).",
     "[rendered] [other] Leander van Ess, Die heiligen Schriften des Neuen Testamentes (Sulzbach 1859), digitised by Hans J. Herbst 2012, Zefania XML; public domain",
     NT, "docx 9.15.2 (npm) on Node 22.23.4; render_g06_vaness_docx.js"),
    ("G07", "G07_fr_fillion_deuterocanoniques.pdf", "fra", "Latin", "pdf",
     "6x9 in pages, single justified column (DejaVu Serif): half-title, contents page with dot leaders and page numbers; each book on a new page with a small-capitals title; chapters as large floated bold numerals with no 'chapitre' word, verse 1 unnumbered, other verses with small bold superscript numbers, one paragraph per chapter; running heads alternate book name and 'Book, chapter' with the page number in the outer corner; automatic hyphenation.",
     "[rendered] [other] La Sainte Bible, traduction de L.-Cl. Fillion, Zefania XML (magnificat.ca); public domain",
     "TOB,JDT,EST,WIS,SIR,BAR,DAN,1MA,2MA", "WeasyPrint 70.0; render_g07_fillion_weasy.py"),
    ("G08", "G08_es_rv1862_nt.docx", "spa", "Latin", "docx",
     "Word document saved by LibreOffice Writer from a hand-written flat ODF: A5, single column, header 'EL NUEVO TESTAMENTO', page-number footer; front matter with the valera1909.com notice and introduction; upper-case book titles centred on new pages; 'CAPITULO 5.' headings centred in small capitals (none for one-chapter books); every verse its own paragraph with a hanging indent and a bold verse number followed by a tab; closing subscriptions of the epistles in italics; supplied words in [brackets] and '^' correction marks as in the source text.",
     "[rendered] [other] Reina-Valera New Testament 1862 (SPCC; BFBS 1884 printing), Project Gutenberg eBook #5879 / valera1909.com; public domain",
     NT, "LibreOffice 24.2.7.2 (MS Word 2007 XML filter); render_g08_rv1862_lo.py"),
    ("G09", "G09_ko_kaeyok1938_selection.pdf", "kor", "Hangul", "pdf",
     "B5, 27 pages, Noto Sans CJK KR: title page and outline; two columns; running head with the book name under a rule, page number '— n —'; book titles large and centred; chapter headings in the 1938 edition's style '뎨일쟝' (archaic Sino-Korean ordinals); verse numbers in Chinese numerals (一, 二 ... 一〇) small and raised, verses run on; archaic Hangul (old jamo sequences) and the printed paragraph sign 〇 in the text; marginal notes as numbered footnotes.",
     "[rendered] [other] 셩경 개역 (Korean Revised Bible, Seoul: 聖書公會 1938), ko.wikisource transcription; public domain (PD-old-80)",
     "RUT,JON,MIC,PHP,COL,1TH,JAS,1JN,JUD", "Typst 0.15.0 (Python package typst); render_g09_kaeyok_typst.py"),
    ("G10", "G10_he_mam_megillot_jonah.pdf", "hbo", "Hebrew", "pdf",
     "17x24 cm, right-to-left, Noto Serif Hebrew with full vowel pointing and cantillation: Hebrew title page with a Latin attribution line; each book on a new page with a large Hebrew title; running head with the Hebrew book name, page number in the footer; chapter headings 'פרק ג'; verse numbers as Hebrew letters (א, ב ... יא) in small bold red type before each verse; text in justified prose paragraphs as divided in the edition; ketiv in parentheses before the qere; the PDF's text layer for pointed Hebrew is poorly ordered (LibreOffice export).",
     "[rendered] [other] Miqra according to the Masorah (S. A. Kadish; OSIS MAPM by D. Troidl, openscriptures/morphhb), CC BY-SA 3.0",
     "SNG,RUT,LAM,ECC,EST,JON", "LibreOffice 24.2.7.2 Writer PDF export; render_g10_mam_lo_pdf.py"),
    ("G11", "G11_grc_nestle1904.md", "grc", "Greek", "md",
     "Hand-built Markdown (UTF-8): YAML front matter, '# Ἡ ΚΑΙΝΗ ΔΙΑΘΗΚΗ' title, linked contents list, horizontal rule then '# ΚΑΤΑ ΜΑΘΘΑΙΟΝ'-style book headings (Greek titles of the edition), '## Κεφάλαιον 5' chapter headings, verse numbers as bold numerals '**12**' run into the edition's paragraphs, polytonic Greek with the edition's text-critical signs [[ ]] [ ] < > printed literally.",
     "[rendered] [other] Nestle 1904 Greek New Testament (base text D. R. dos Santos, public domain; XML biblicalhumanities.org, CC BY-SA 4.0)",
     "MAT,LUK,JHN,ACT,ROM,1CO,2CO,GAL,EPH,PHP,COL,1TH,2TH,1TI,2TI,TIT,PHM,HEB,JAS,1PE,2PE,1JN,2JN,3JN,JUD,REV",
     "hand-written by render_g11_nestle_md.py (Python 3.13)"),
    ("G12", "G12_en_noyes_poetry_gospels.epub", "eng", "Latin", "epub",
     "EPUB 3 made by pandoc from an HTML source: title page with translator and rights, navigation/TOC, one EPUB chapter per book with a centred small-capitals title, chapter headings 'Chapter 4' / 'Psalm 23' in spaced capitals, verse 1 unnumbered and the other verses with small coloured superscript numbers, indented justified paragraphs following the source's paragraph marks, psalm titles in quotation marks inside verse 1.",
     "[rendered] [eBible] eBible.org engnoy, The Holy Bible translated by George R. Noyes (1869), public domain",
     "JOB,PSA,PRO,ECC,SNG,MAT,MRK,LUK,JHN", "pandoc 3.1.3 (html -> epub3); render_g12_noyes_pandoc.py"),
    ("G13", "G13_en_ulb_selection.pdf", "eng", "Latin", "pdf",
     "US Letter, 28 pages, Liberation Serif: title page with the CC BY-SA licence; two columns with a vertical rule; running head with the verse range on the page ('Ruth 2:17 – 4:12') on the outer side and the book name on the inner side, page number in the bottom outer corner; book titles in a sans-serif heading; chapter numbers as large bold numerals on their own line; superscript bold verse numbers (verse 1 unnumbered); poetry as hanging-indent lines; translators' footnotes marked with superscript letters and collected under 'Notes' after each chapter with their chapter:verse.",
     "[rendered] [eBible] eBible.org engULB, Unlocked Literal Bible (c) 2017 Door43 World Missions Community, CC BY-SA 4.0",
     "RUT,JON,MRK,ROM,PHM,JUD", "ReportLab 5.0.1 (platypus); render_g13_ulb_reportlab.py"),
    ("G14", "G14_la_castellio_excerpta.epub", "lat", "Latin", "epub",
     "EPUB 3 built with EbookLib: navigation document and NCX, a licence/attribution page, one XHTML file per book with the project's long title ('Ēvangelium Auctōre Mārcō'), chapter headings 'CAPUT XII' in Roman numerals, verse numbers as small grey bracketed numbers '[12]' for every verse, each chapter one justified paragraph; classical Latin orthography with macrons, curly quotes.",
     "[rendered] [other] Biblia Sacra ex Sebastiani Castellionis interpretatione, transcription by the biblia-castellionis project (github njand, commit 0f93476, 2023), CC BY-SA 4.0",
     "GEN,MAT,MRK,LUK,JHN", "EbookLib 0.20 (Python); render_g14_castellio_ebooklib.py"),
]


def sha(p):
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def main():
    lines = ["\t".join(HEADER)]
    for key, fn, lang, script, fmt, layout, src, books, gen in ROWS:
        ref = os.path.splitext(fn)[0] + ".usfm.zip"
        assert os.path.exists(os.path.join(D, fn)) and os.path.exists(os.path.join(D, ref)), fn
        row = [key, fn, ref, lang, script, fmt, layout, "withheld-g", src, books, sha(os.path.join(D, fn)), gen]
        assert all("\t" not in c and "\n" not in c for c in row)
        lines.append("\t".join(row))
    open("/root/aw-set-g/withheld-g.tsv", "w", encoding="utf-8").write("\n".join(lines) + "\n")
    print(len(ROWS), "rows")


if __name__ == "__main__":
    main()
