#!/usr/bin/env python3
"""Write withheld-f.tsv and GROUND_TRUTH.md for set F (run once, before sealing)."""
import os, sys, hashlib, zipfile, re, collections
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import ROOT, DOCS, load_zip
import usfm

SRC = os.path.join(ROOT, 'src')
LOGS = os.path.join(ROOT, 'work', 'logs')


def sha(p):
    h = hashlib.sha256()
    with open(p, 'rb') as f:
        for blk in iter(lambda: f.read(1 << 20), b''):
            h.update(blk)
    return h.hexdigest()


def books_of(ref):
    with zipfile.ZipFile(os.path.join(DOCS, ref)) as z:
        codes = [re.match(r'\d+-([0-9A-Z]{3})', n).group(1) for n in z.namelist() if n.endswith('.usfm')]
    return codes


def omitted(key):
    p = os.path.join(LOGS, key + '.log')
    if not os.path.exists(p):
        return ''
    by = collections.OrderedDict()
    other = []
    for ln in open(p, encoding='utf-8'):
        m = re.match(r'(\S+) (\d+:\d+) empty in source', ln)
        if m:
            by.setdefault(m.group(1), []).append(m.group(2))
        elif 'pilcrow inside verse kept' in ln:
            other.append(ln.split(' pilcrow')[0])
    s = ''
    if by:
        n = sum(len(v) for v in by.values())
        s += '- Verses whose text is empty in the source (%d; left out of document and reference, so the numbering skips them): ' % n
        s += '; '.join('%s %s' % (b, ', '.join(v)) for b, v in by.items()) + '.\n'
    if other:
        s += ('- A pilcrow (¶) inside the verse text was kept as text in both document and reference (subscription '
              'notes after the last verse of the epistles): ' + '; '.join(other) + '.\n')
    return s


SCROLL = 'https://raw.githubusercontent.com/scrollmapper/bible_databases/master/formats/json/%s.json'
EB = 'https://ebible.org/Scriptures/%s_usfm.zip'

D = [
 dict(key='F01', files='F01-eng-nheb.pdf', ref='F01-eng-nheb.usfm.zip', lang='eng', script='Latin', fmt='pdf',
      src_kind='other', srcfile='scroll/NHEB.json', url=SCROLL % 'NHEB',
      source='New Heart English Bible (NHEB), Wayne A. Mitchell; text from scrollmapper/bible_databases (2025 schema, master branch, formats/json/NHEB.json, retrieved 2026-11-01); public domain (the NHEB is dedicated to the public domain by its editor; scrollmapper lists its licence as "Unknown"; repository code MIT)',
      gen='PyMuPDF 2.0 (MuPDF 1.28.5) Story layout engine; tools/make_f01_nheb_mupdf.py',
      layout='Pocket 5.5x8.5 in page, two justified columns (Caladea 8.6 pt), roman-numbered front matter: title page and a CONTENTS table with page numbers (plus PDF outline); books in the order Matthew-Revelation, then Psalms, Proverbs; each book opens with a spaced capital name and a full italic title; chapter number set as a large bold 19 pt numeral at the start of the chapter\'s paragraph IN PLACE OF verse number 1 (no printed "1"), except in Psalms, which get a centred "PSALM n" line and a superscript 1; all other verse numbers small bold dark-red superscripts; one paragraph per chapter; running head on every body page with the verse range on that page in capitals ("MATTHEW 7:22-8:27") on the outer side, page number on the inner side, rule under the head.',
      versif='English (KJV-type) versification as in the source.',
      method='converted: tools/scroll2usfm.py NHEB.json eng PSA,PRO,<27 NT books> (one \\p per chapter; book titles supplied, see note)',
      odd=['REV 14:1 begins "4I saw, ..." - a stray digit 4 is part of the source verse text; kept unchanged.',
           '13 verses of these books contain bracketed text [...] in the source (e.g. HEB 2:8); kept as printed.']),
 dict(key='F02', files='F02-eng-leeser.pdf', ref='F02-eng-leeser.usfm.zip', lang='eng', script='Latin', fmt='pdf',
      src_kind='eBible', srcfile='eb/englee_usfm.zip', url=EB % 'englee',
      source='Isaac Leeser Tanakh (1853), eBible.org englee, USFM zip generated 30 Oct 2026 from source files dated 12 Dec 2025; Public Domain',
      gen='LibreOffice Writer 24.2.7.2 (headless PDF export of a generated flat ODF .fodt); tools/make_f02_leeser_lo.py',
      layout='6.5x9.5 in page, two columns with a vertical rule (Liberation Serif 10 pt); title page with the 1853 title wording and an ORDER OF THE BOOKS list (no page numbers); books in Jewish (Tanakh) order with divider pages THE LAW / THE PROPHETS / THE HOLY WRITINGS; each book starts on a new page with a centred capital heading; chapters headed in small capitals with ROMAN numerals ("CHAPTER III.", Psalms "PSALM XLI."); one verse per paragraph with hanging indent, the verse number in bold followed by a full stop and a tab ("13. And the name ..."); empty verses are printed as a bare number; running head = current book name (spaced capitals, ODF chapter field) over a rule; page number centred in the footer.',
      versif='English versification; where the Hebrew numbering differs the source puts the Hebrew chapter:verse in parentheses at the start of the verse text, e.g. MAL 4:1 "(3:19) For, behold, ..."; JOL 3 chapters, MAL 4 chapters.',
      method='reduced: tools/reduce_ebible.py englee_usfm.zip (39 OT book files copied byte for byte; copr.htm and other non-USFM files left out)',
      odd=['JOS 21:36 and 21:37 are empty in the source (\\v 36 and \\v 37 with no text) and 21:38 begins "(21:36) And from the tribe of Gad"; the document prints "36." and "37." with nothing after them, as the reference has them.',
           'About 770 verses carry the Hebrew numbering in parentheses inside the verse text (see versification); they are verse text in the source and in the document.',
           'Double square brackets occur in the source text, e.g. GEN 2:23 "Woman, [[Ishah,]] because out of Man [[Ishi]]"; kept.']),
 dict(key='F03', files='F03-deu-menge.pdf', ref='F03-deu-menge.usfm.zip', lang='deu', script='Latin', fmt='pdf',
      src_kind='other', srcfile='scroll/GerMenge.json', url=SCROLL % 'GerMenge',
      source='Menge-Bibel (Hermann Menge, 1939 text) with Apocrypha, CrossWire module GerMenge via scrollmapper/bible_databases (formats/json/GerMenge.json, master, retrieved 2026-11-01); Public Domain (translator died 1939; licence field "Public Domain")',
      gen='ReportLab 5.0.1 Platypus (multiBuild, onDraw callbacks for running heads); tools/make_f03_menge_reportlab.py',
      layout='B5 page, two justified columns (Noto Serif 8.4 pt) with a column rule; title page, "Inhalt" contents table with page numbers; divider pages "Das Alte Testament", "Die Apokryphen" (between the testaments) and "Das Neue Testament"; books in Luther order (Hebrews and James after 3 John), each book on a new page with its full German title across both columns; chapters headed with centred bold "Kapitel N"; verse numbers as small grey raised figures (verse 1 printed); one paragraph per chapter; running head in German reference style with a comma between chapter and verse giving the first and last verse drawn on the page ("1. Mose 2,3 - 3,12"), page number at the outer head corner, rule under the head.',
      versif='The source (CrossWire) maps Menge onto a KJV-type versification with empty slots; the empty slots are not printed. Apocrypha: Tobit/Judith/Wisdom/Sirach/Baruch/1-2 Maccabees as in the module; "Stücke zu Ester" (ESG) in 7 chapters and "Stücke zu Daniel" (DAG) in 3 chapters as the module numbers them (not the Vulgate 10-16 / 13-14 chapter numbers); Gebet Manasses (MAN) 1 chapter.',
      method='converted: tools/scroll2usfm.py GerMenge.json deu all (one \\p per chapter; book titles supplied; book order kept)',
      odd=['Most empty slots are at the ends of chapters where the module\'s versification has more verses than Menge\'s text (Sirach alone has 225, e.g. SIR 1:28-38); a few are inside chapters, where the number is simply skipped.',
           '192 verses contain bracketed text [...] in the source; kept.']),
 dict(key='F04', files='F04-fra-martin.epub', ref='F04-fra-martin.usfm.zip', lang='fra', script='Latin', fmt='epub',
      src_kind='other', srcfile='scroll/FreBDM1744.json', url=SCROLL % 'FreBDM1744',
      source='Bible David Martin (1744), CrossWire module FreBDM1744 via scrollmapper/bible_databases (formats/json/FreBDM1744.json, master, retrieved 2026-11-01); Public Domain',
      gen='EbookLib 0.20 (EPUB 3 with NCX); tools/make_f04_martin_ebooklib.py',
      layout='Reflowable EPUB 3: title page, then one XHTML file per book; nested navigation (Le Vieux Testament / Le Nouveau Testament > books); each book starts with its French title and a grid of boxed chapter-number links (1 2 3 ...) before the text; chapters headed "Chapitre N"; one verse per paragraph, the verse number in bold dark red at the start of the paragraph followed by a small gap, verse 1 printed.',
      versif='French Protestant / KJV-type versification as in the source (Psalm titles are part of verse 1).',
      method='converted: tools/scroll2usfm.py FreBDM1744.json fra all',
      odd=['PSA 102:1 contains a merged following verse with its own number inside the text: "... devant l\'Eternel. 2.Eternel! écoute ma prière ..."; kept as the source has it (the number "2." is verse text of 102:1 in source and reference).',
           'OCR-type errors in the source text are kept: EXO 35:21 "1\'offrande", 1SA 9:9 "0r", 1SA 11:13 "0n", PSA 136:17 "I1", 1CO 3:15 "1\'oeuvre", HEB 10:7 "voic1", HEB 13:8 "aujourd\'hu1" (digit one / zero for letters).',
           'Slash characters and [bracketed] words occur in the source text and are kept.']),
 dict(key='F05', files='F05-spa-rv1865.docx', ref='F05-spa-rv1865.usfm.zip', lang='spa', script='Latin', fmt='docx',
      src_kind='other', srcfile='scroll/SpaRV1865.json', url=SCROLL % 'SpaRV1865',
      source='La Santa Biblia Reina-Valera (1865) con arreglos ortográficos, CrossWire module SpaRV1865 via scrollmapper/bible_databases (formats/json/SpaRV1865.json, master, retrieved 2026-11-01); Public Domain',
      gen='python-docx 1.2.0; tools/make_f05_rv1865_pythondocx.py',
      layout='Word document, US Letter; title page; one Word section per book (new page) with two columns separated by a line, the book short name in capitals in the section header and a PAGE field centred in the footer; book title as Heading 1, "EL NUEVO TESTAMENTO ..." line before Matthew; chapters as Heading 2 "CAPÍTULO N" (Psalms "SALMO N"); paragraphs follow the pilcrow paragraph marks of the source (the ¶ sign itself not printed), first-line indent; the first verse of every chapter is shown as a large bold chapter numeral instead of a verse number "1" (Psalms keep a superscript 1); other verse numbers dark-red superscripts followed by a no-break space.',
      versif='KJV-type versification as in the source.',
      method='converted: tools/scroll2usfm.py SpaRV1865.json spa all --pilcrow (a verse beginning with ¶ starts a new \\p and loses the sign)',
      odd=[]),
 dict(key='F06', files='F06-por-nva.pdf', ref='F06-por-nva.usfm.zip', lang='por', script='Latin', fmt='pdf',
      src_kind='other', srcfile='scroll/PorNVA.json', url=SCROLL % 'PorNVA',
      source='Bíblia Nova Versão de Acesso Livre (NVA), CrossWire module PorNVA via scrollmapper/bible_databases (formats/json/PorNVA.json, master, retrieved 2026-11-01); licence Creative Commons BY-SA 4.0 (attribution and licence notice printed on the title page; layout shared under the same licence)',
      gen='fpdf2 2.9.0; tools/make_f06_nva_fpdf2.py',
      layout='A5 page, single column, ragged right (Noto Serif 9 pt); title page with licence notice; "Índice" page with page numbers (fpdf2 TOC placeholder) and PDF outline; each book on a new page with a bold capital title; chapters headed by a centred coloured "— Capítulo N —"; verse numbers as bold green figures on the baseline followed by a space, verse 1 printed; one paragraph per chapter; running head on the outer side with book and the chapter in progress at the top of the page ("Mateus 24") over a rule; footer "Página N" centred.',
      versif='KJV-type NT versification as in the source.',
      method='converted: tools/scroll2usfm.py PorNVA.json por nt',
      odd=['MAT 10:5 begins its direct speech with a stray digit: "dizei primeiro: 1\'Que a paz ..." (source text, kept).',
           'ROM 4:19 contains a number in digits in the source text (kept).']),
 dict(key='F07', files='F07-kor-rv.pdf', ref='F07-kor-rv.usfm.zip', lang='kor', script='Hangul', fmt='pdf',
      src_kind='other', srcfile='scroll/KorRV.json', url=SCROLL % 'KorRV',
      source='개역성경 (Korean Revised Version, 1961 개역한글 text), CrossWire module KorRV via scrollmapper/bible_databases (formats/json/KorRV.json, master, retrieved 2026-11-01); licence field "Public Domain". REUSE NOTE: this is very probably the same translation as the earlier sets\' eBible id kor1961; it is used here only because no other Korean text is available, from a different source/encoding and in a completely different layout.',
      gen='Typst 0.15.0 (python package typst 0.15.0); tools/make_f07_korrv_typst.py',
      layout='128x188 mm page; title page "성 경 전 서 / 개 역", a 목차 contents page with dotted leaders and page numbers (Typst outline) and PDF bookmarks for books; "구 약 전 서" / "신 약 전 서" testament captions; two justified columns (Noto Serif CJK KR 8 pt); each book on a new page with its Korean name centred across both columns; chapters headed in dark red "제 N 장" (Psalms "제 N 편"); ONE VERSE PER LINE with the verse number in a fixed-width box and a hanging indent, verse 1 printed; running head with book name and page number (outer side) over a rule.',
      versif='Korean Protestant (KJV-type) versification as in the source.',
      method='converted: tools/scroll2usfm.py KorRV.json kor all',
      odd=['JOS 2:13 text is "12절과 상동" and ISA 7:9 text is "8절과 같음" ("same as verse 12/8") in the source; printed as verse text.',
           'Where the source leaves a verse empty the preceding verse presumably carries the merged text (e.g. ISA 30:1 is followed by 30:3); the reference simply has no \\v for the empty number.',
           'Bracketed passages, e.g. MRK 16:9-20 in [ ], are kept as in the source.']),
 dict(key='F08', files='F08-heb-salkinson.docx', ref='F08-heb-salkinson.usfm.zip', lang='heb', script='Hebrew', fmt='docx',
      src_kind='eBible', srcfile='eb/hebsg_usfm.zip', url=EB % 'hebsg',
      source='Salkinson-Ginsburg Hebrew New Testament (1886), eBible.org hebsg, USFM zip generated 30 Oct 2026 from source files dated 12 Dec 2025; Public Domain',
      gen='LibreOffice Writer 24.2.7.2 (headless export "MS Word 2007 XML" of a generated flat ODF .fodt); tools/make_f08_salkinson_lodocx.py',
      layout='Word document, 6.3x9.25 in page, RIGHT-TO-LEFT pointed Hebrew (Noto Serif Hebrew); title page and the edition\'s English introduction (left-to-right paragraphs with dates such as 1883, 1886) from its INT file; each book on a new page with a Heading 1 short name and the full pointed title; text in a two-column right-to-left section with a rule; chapter headings (Heading 2) are the source\'s own \\cl labels "פרק א"; verse numbers are the source\'s published Hebrew-letter numbers (\\vp: א, ב, ... יא, טו, טז) in bold dark red before each verse; running header = book short name (chapter field), page number in the footer.',
      versif='Standard (KJV-type) NT versification in the \\v markers; the document prints the \\vp Hebrew-letter verse numbers and \\cl/\\cp Hebrew-letter chapter labels that the reference also carries.',
      method='reduced: tools/reduce_ebible.py hebsg_usfm.zip nt (27 book files copied byte for byte; the INT introduction file, printed as front matter, holds no verses and is not in the reference)',
      odd=['3JN has 14 verses in this source.']),
 dict(key='F09', files='F09-grc-tcgnt.md', ref='F09-grc-tcgnt.usfm.zip', lang='grc', script='Greek', fmt='md',
      src_kind='eBible', srcfile='eb/grctcgnt_usfm.zip', url=EB % 'grctcgnt',
      source='The Text-Critical Greek New Testament: Byzantine Text (Robert Adam Boyd, based on Robinson-Pierpont Byzantine Textform 2018), eBible.org grctcgnt, USFM zip generated 31 Oct 2026; Public Domain. NOTE: the Greek text is closely related to the Robinson-Pierpont Byzantine text of earlier sets (grcbyz); this is a separate edition with its own text-critical apparatus.',
      gen='plain Python writer (CommonMark + footnote syntax); tools/make_f09_tcgnt_md.py',
      layout='Single UTF-8 Markdown file (LF): front matter and the English introduction with a sigla table (Markdown table) from the edition\'s FRT/INT files; "# BOOK" headings (Greek capitals, e.g. "# ΕΥΑΓΓΕΛΙΟΝ ΚΑΤΑ ΜΑΤΘΑΙΟΝ"), "## N" chapter headings with the bare number, "### ..." English section headings, verse numbers as inline HTML <sup>N</sup> directly before the word; the text-critical notes as Markdown footnotes [^n] (numbered through the whole file; each note begins with its own chapter:verse, e.g. "[^47]: 2:23 ναζαρετ ¦ ναζαρεθ ꜱᴄʀ") collected after every chapter; OT quotations (\\q1) as block-quote lines ending in two spaces; book-final "Ἀμήν" paragraphs in <div align="center">; "---" rule after each book.',
      versif='Byzantine/standard NT versification of the source: LUK 17:36, ACT 8:37, 15:34, 24:7 are absent (not in the Byzantine text); the Romans doxology stands at ROM 14:24-26 and ROM 16 ends at 16:24.',
      method='reduced: tools/reduce_ebible.py grctcgnt_usfm.zip nt (27 book files copied byte for byte; FRT/INT printed as front matter, appendix XXA not printed; neither is in the reference)',
      odd=['Book-final \\pc paragraphs ("Ἀμήν.") have no verse marker of their own: they belong to the last verse of the book (USFM continuation), and the document prints them after that verse as a centred line.',
           'Some section headings fall inside a verse (e.g. MRK 6:6, LUK 9:43, JHN 12:36, ACT 8:1); the heading is printed between the two parts of the verse, as in the source.']),
 dict(key='F10', files='F10-lat-sistine.epub', ref='F10-lat-sistine.usfm.zip', lang='lat', script='Latin', fmt='epub',
      src_kind='other', srcfile='scroll/VulgSistine.json', url=SCROLL % 'VulgSistine',
      source='Vulgata Sistina (Sixtine Vulgate, 1590), CrossWire module VulgSistine via scrollmapper/bible_databases (formats/json/VulgSistine.json, master, retrieved 2026-11-01); Public Domain. NOTE: a different edition from the Clementine Vulgate (latVUC) of earlier sets, but the same Latin Vulgate tradition (closely related text).',
      gen='hand-built EPUB 2.0.1 (Python zipfile: OPF 2.0, NCX, XHTML 1.1); tools/make_f10_sistine_epub2.py',
      layout='Reflowable EPUB 2: title page in Latin; one XHTML file per book (Isaias, Ieremias, Threni, Baruch, Ezechiel, Daniel, the Twelve) in Vulgate order; NCX with nested book > "Caput I" chapter entries; book heading in capitals ("PROPHETIA ISAIAE") with the short name below; chapters headed "CAPUT I." with ROMAN numerals; one verse per paragraph with the verse number floated into the left margin in small red figures.',
      versif='Vulgate versification: DAN 14 chapters (3:24-90 Song of the Three in chapter 3, Susanna = ch. 13, Bel and the Dragon = ch. 14), BAR 6 chapters (Letter of Jeremiah = ch. 6), JOL 3 chapters, MAL 4 chapters.',
      method='converted: tools/scroll2usfm.py VulgSistine.json lat ISA,JER,LAM,BAR,EZK,DAN,HOS,JOL,AMO,OBA,JON,MIC,NAM,HAB,ZEP,HAG,ZEC,MAL (Latin book titles supplied)',
      odd=[]),
 dict(key='F11', files='F11-ell-vamvas.txt', ref='F11-ell-vamvas.usfm.zip', lang='ell', script='Greek', fmt='txt',
      src_kind='other', srcfile='scroll/GreVamvas.json', url=SCROLL % 'GreVamvas',
      source='Η Αγία Γραφή, translation of Neophytos Vamvas (1850; Katharevousa Greek), CrossWire module GreVamvas via scrollmapper/bible_databases (formats/json/GreVamvas.json, master, retrieved 2026-11-01); Public Domain',
      gen='plain Python writer; tools/make_f11_vamvas_txt.py',
      layout='Plain text, UTF-8 with byte-order mark, CRLF line ends, hard-wrapped at 76 columns; title block and a ΠΕΡΙΕΧΟΜΕΝΑ list of book names (no numbers); testament banners between rows of asterisks; each book name in unaccented capitals centred between two rows of "=" signs; chapters headed "ΚΕΦΑΛΑΙΟΝ" followed by the Greek alphabetic numeral with keraia (Αʹ, Βʹ, ... ΣΤʹ, ΙΣΤʹ, ΡΙΘʹ, ΡΝʹ); each chapter one wrapped paragraph with the verse numbers as Arabic figures running inline (a verse number is never separated from its first word; two spaces usually precede it).',
      versif='KJV-type versification as in the source.',
      method='converted: tools/scroll2usfm.py GreVamvas.json ell all --cp-greek (\\cp carries the printed Greek numeral)',
      odd=['No empty verses and no digits in the source text.']),
 dict(key='F12', files='F12-rus-synodal.epub', ref='F12-rus-synodal.usfm.zip', lang='rus', script='Cyrillic', fmt='epub',
      src_kind='eBible', srcfile='eb/russyn_usfm.zip', url=EB % 'russyn',
      source='Russian Synodal Translation (Синодальный перевод), eBible.org russyn, USFM zip generated 30 Oct 2026 from source files dated 12 Dec 2025; Public Domain',
      gen='pandoc 3.1.3 (HTML -> epub3, --split-level=1 --toc --toc-depth=1); tools/make_f12_synodal_pandoc.py',
      layout='Reflowable EPUB 3 made by pandoc: title page from metadata, nav/NCX contents with the 66 book titles, one XHTML file per book; books as level-1 headings (source \\mt1), chapters as level-2 headings "Глава N" (Psalms "Псалом N"); verse numbers as bold dark-red superscripts followed by a no-break space; prose one paragraph per source paragraph; poetry (\\q1) as one paragraph per line inside a div.poem; psalm titles (\\d) as plain paragraphs before the first verse.',
      versif='Russian Synodal versification as in the source (Psalms with Synodal/LXX-based numbering and titles counted as verses where the source does so; PSA has 2526 verses).',
      method='reduced: tools/reduce_ebible.py russyn_usfm.zip all (66 book files copied byte for byte)',
      odd=[]),
 dict(key='F13', files='F13-ita-diodati.pdf', ref='F13-ita-diodati.usfm.zip', lang='ita', script='Latin', fmt='pdf',
      src_kind='eBible', srcfile='eb/ita1885_usfm.zip', url=EB % 'ita1885',
      source='La Sacra Bibbia, Giovanni Diodati, revised edition 1885 (Diodati Bibbia 1885), eBible.org ita1885, USFM zip generated 30 Oct 2026 from source files dated 3 May 2025; Public Domain',
      gen='WeasyPrint 70.0 (CSS paged media, float: footnote); tools/make_f13_diodati_weasy.py',
      layout='140x210 mm page, single column (Liberation Serif 9.6 pt), title page and INDICE with dotted leaders and page numbers; the poetical books Giobbe, Salmi, Proverbi, Ecclesiaste, Cantico, each starting on a right-hand page with the source title ("IL LIBRO DE\' SALMI"); chapter headings are the source\'s own \\cl labels with ROMAN numerals ("SALMO X", "CAPO 36") or "CAPO N"; bold centred section headings, italic psalm titles (\\d), translators\' added words (\\add) in italics, Selah "(Sela.)" in italics after the line; poetry one line per paragraph with the verse number in small bold figures and an en space; the edition\'s cross references (and few footnotes) as real page-bottom footnotes under a rule, called by superscript letters a, b, c ... restarting at every chapter, each note beginning with chapter:verse in bold ("i) 9:16 fatto Esod. 7.5; 14.4,10,31."); running heads book name (left) and the first chapter label on the page (right); page numbers in the outer bottom corner.',
      versif='Italian Protestant (KJV-type) versification as in the source; psalm titles are \\d headings, not verses.',
      method='reduced: tools/reduce_ebible.py ita1885_usfm.zip JOB,PSA,PRO,ECC,SNG (5 book files copied byte for byte)',
      odd=['In the source cross references the keyword is encoded with an empty \\xk followed by \\xt (e.g. "\\xk \\xt empi \\xt Prov. 4.14,15."); notes are not verse text.']),
]

ROLE = 'withheld-f'


def main():
    rows = []
    gt = ['# Set F - independent ground truth', '',
          'Builder: independent test builder (Claude), 2026-11-01. Everything here was made from public sources '
          'downloaded on 2026-11-01; the program under test was not seen or run.', '',
          'General rules for all references:', '',
          '- One USFM file per book in each `*.usfm.zip`, standard book codes, `\\c` and `\\v` numbering identical to the '
          'document. Verse text is the publisher\'s text unchanged (only white space collapsed).',
          '- For eBible.org sources the reference files are the eBible USFM files themselves, copied byte for byte '
          '(only the books the document holds); they keep all of eBible\'s own markup (`\\w ...|strong=...\\w*`, '
          '`\\f`, `\\x`, `\\s1`, `\\d`, `\\cl`, `\\vp`, ...). Notes, headings and labels in them are marked as such and are '
          'not verse text.',
          '- For other sources the reference was produced by `tools/scroll2usfm.py` from the source JSON: `\\id`, `\\h`, '
          '`\\toc1-3`, `\\mt1`, `\\c`, (`\\cp`), `\\p`, `\\v`. The source carries no book titles: the titles in `\\h/\\toc/\\mt1` '
          'are standard printed titles in the language, supplied by the builder (`tools/booknames.py`), and are the '
          'titles the document prints. Verses whose source text is empty are left out of both document and reference '
          '(listed per document).',
          '- Book titles, chapter headings ("Kapitel 3", "CHAPTER III.", "제 3 장", "ΚΕΦΑΛΑΙΟΝ Γʹ", ...), running heads, '
          'page numbers, contents lists, title pages, introductions and division pages are layout, not verse text.',
          '- `tools/verify.py` (builder self-check) confirmed for every document that each reference verse text occurs in '
          'the document\'s extracted text in order (deviations only where a heading, a note caller or a footnote sits '
          'inside a verse, or right-to-left extraction order).', '']
    for d in D:
        docp = os.path.join(DOCS, d['files'])
        books = books_of(d['ref'])
        bl = ','.join(books)
        if len(books) == 66 and 'DAG' not in books:
            bl = 'all'
        h = sha(docp)
        rows.append([d['key'], d['files'], d['ref'], d['lang'], d['script'], d['fmt'], d['layout'], ROLE,
                     d['source'] + ' [' + d['src_kind'] + ']', bl, h, d['gen']])
        nv = sum(len(usfm.verses_of(b)) for b in load_zip(os.path.join(DOCS, d['ref'])))
        gt.append('## %s - %s' % (d['key'], d['files']))
        gt.append('')
        gt.append('- Reference: `%s` (%d books, %d verses).' % (d['ref'], len(books), nv))
        gt.append('- Source: %s.' % d['source'].rstrip('.'))
        gt.append('- Source kind: %s. URL: %s' % ('eBible.org USFM' if d['src_kind'] == 'eBible' else 'other (not eBible USFM)', d['url']))
        gt.append('- Source file SHA-256: `%s` (%s)' % (sha(os.path.join(SRC, d['srcfile'])), os.path.basename(d['srcfile'])))
        gt.append('- Reference made: %s.' % d['method'])
        gt.append('- Versification (document and reference): %s' % d['versif'])
        gt.append('- Document generator: %s.' % d['gen'])
        om = omitted(d['key'])
        if om or d['odd']:
            gt.append('- Known gaps and oddities in the source text (not fixed):')
            if om:
                gt.extend('  ' + ln for ln in om.strip().split('\n'))
            for o in d['odd']:
                gt.append('  - ' + o)
        gt.append('')
    gt += ['## Reuse and licence summary', '',
           '- F07 (Korean): unavoidable reuse of the 1961 Korean Revised (개역한글) translation (= eBible kor1961 of earlier sets) - '
           'all Korean texts available here are either that translation, the 1910 text (kor, used) or a copyrighted text; '
           'taken from a different source (CrossWire KorRV via scrollmapper) and laid out completely differently.',
           '- F09 (grctcgnt) and F10 (Vulgata Sistina) are different editions but closely related texts to grcbyz and latVUC of earlier sets; recorded for transparency.',
           '- German, French, Spanish, Portuguese and Korean: every eBible.org USFM edition of these languages is either '
           'in the excluded list or restricted, so these documents use CrossWire texts distributed by scrollmapper/bible_databases.',
           '- Licences: all texts are public domain except F06 (PorNVA, CC BY-SA 4.0 - attribution and licence notice on the document\'s title page). '
           'F01 NHEB: public domain by its editor\'s dedication (the scrollmapper licence field says "Unknown").',
           '- Rejected during selection: grcf35 (CC BY-NC-ND, no derivatives), open-bibles por-almeida (edition and licence unclear).', '']
    hdr = ['key', 'files', 'reference', 'language', 'script', 'format', 'layout', 'role', 'source and licence', 'books', 'sha256', 'generator']
    with open(os.path.join(ROOT, 'withheld-f.tsv'), 'w', encoding='utf-8', newline='\n') as f:
        f.write('\t'.join(hdr) + '\n')
        for r in rows:
            assert not any('\t' in c or '\n' in c for c in r)
            f.write('\t'.join(r) + '\n')
    with open(os.path.join(ROOT, 'GROUND_TRUTH.md'), 'w', encoding='utf-8', newline='\n') as f:
        f.write('\n'.join(gt) + '\n')
    print('ok', len(rows))


if __name__ == '__main__':
    main()
