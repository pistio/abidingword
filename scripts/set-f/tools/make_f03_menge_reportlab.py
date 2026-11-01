#!/usr/bin/env python3
"""F03: Menge-Bibel (1939) with the Apocrypha, complete.
Generator: ReportLab Platypus. B5 page, two columns with a rule, centred chapter
headings 'Kapitel N', small raised verse numbers, running heads in German
reference style ('1. Mose 3,4 - 4,2') taken from the verses actually drawn on
the page, outer page numbers, a contents table with page numbers, separate
divider pages for Old Testament, Apocrypha and New Testament."""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from reportlab.lib.pagesizes import B5
from reportlab.lib.units import mm
from reportlab.lib.styles import ParagraphStyle
from reportlab.lib.enums import TA_JUSTIFY, TA_CENTER
from reportlab.platypus import (BaseDocTemplate, PageTemplate, Frame, Paragraph, Spacer, NextPageTemplate,
                                PageBreak, FrameBreak, CondPageBreak)
from reportlab.platypus.tableofcontents import TableOfContents
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.lib.fonts import addMapping
from xml.sax.saxutils import escape
from common import load_zip, clean_items, DOCS, WORK, short_name, book_title
import reportlab

REF = os.path.join(DOCS, 'F03-deu-menge.usfm.zip')
OUT = os.path.join(DOCS, 'F03-deu-menge.pdf')
F = '/usr/share/fonts/truetype/noto/'
pdfmetrics.registerFont(TTFont('NS', F + 'NotoSerif-Regular.ttf'))
pdfmetrics.registerFont(TTFont('NS-B', F + 'NotoSerif-Bold.ttf'))
pdfmetrics.registerFont(TTFont('NS-I', F + 'NotoSerif-Italic.ttf'))
addMapping('NS', 0, 0, 'NS'); addMapping('NS', 1, 0, 'NS-B'); addMapping('NS', 0, 1, 'NS-I'); addMapping('NS', 1, 1, 'NS-B')

PW, PH = B5
ML, MR, MT, MB, GAP = 16 * mm, 14 * mm, 20 * mm, 17 * mm, 6 * mm

body = ParagraphStyle('body', fontName='NS', fontSize=8.4, leading=10.3, alignment=TA_JUSTIFY, firstLineIndent=0, spaceAfter=1)
chap = ParagraphStyle('chap', fontName='NS-B', fontSize=9, leading=12, alignment=TA_CENTER, spaceBefore=5, spaceAfter=2)
btitle = ParagraphStyle('bt', fontName='NS-B', fontSize=16, leading=20, alignment=TA_CENTER, spaceAfter=2)
bsub = ParagraphStyle('bs', fontName='NS-I', fontSize=9, leading=11, alignment=TA_CENTER)
div = ParagraphStyle('div', fontName='NS-B', fontSize=24, leading=30, alignment=TA_CENTER, spaceBefore=180)
toc1 = ParagraphStyle('toc1', fontName='NS', fontSize=9.5, leading=12.5)
tochead = ParagraphStyle('th', fontName='NS-B', fontSize=14, leading=18, alignment=TA_CENTER, spaceAfter=10)

state = {'marks': {}, 'names': {}}


def onverse(canv, kind, label):
    pg = canv.getPageNumber()
    state['marks'].setdefault(pg, []).append(label)


class Doc(BaseDocTemplate):
    def beforeDocument(self):
        state['marks'] = {}

    def afterFlowable(self, fl):
        if getattr(fl, '_toc_label', None):
            self.notify('TOCEntry', (0, fl._toc_label, self.page))


def fmt_ref(lbl):
    code, c, v = lbl.split('|')
    return state['names'][code], c, v


def page_end(canv, doc):
    pg = canv.getPageNumber()
    canv.saveState()
    odd = pg % 2 == 1
    marks = state['marks'].get(pg)
    canv.setFont('NS', 7.5)
    num = str(pg)
    if odd:
        canv.drawRightString(PW - MR, PH - 12 * mm, num)
    else:
        canv.drawString(ML, PH - 12 * mm, num)
    if marks:
        a, b = fmt_ref(marks[0]), fmt_ref(marks[-1])
        if a[0] == b[0]:
            if a[1] == b[1]:
                head = '%s %s,%s%s' % (a[0], a[1], a[2], '' if a[2] == b[2] else '-' + b[2])
            else:
                head = '%s %s,%s - %s,%s' % (a[0], a[1], a[2], b[1], b[2])
        else:
            head = '%s %s,%s - %s %s,%s' % (a[0], a[1], a[2], b[0], b[1], b[2])
        canv.setFont('NS-I', 7.5)
        canv.drawCentredString(PW / 2, PH - 12 * mm, head)
        canv.setLineWidth(0.4)
        canv.line(ML, PH - 13.5 * mm, PW - MR, PH - 13.5 * mm)
        # column rule
        canv.setLineWidth(0.3)
        x = ML + (PW - ML - MR) / 2
        canv.line(x, MB, x, PH - MT)
    canv.restoreState()


def build():
    books = load_zip(REF, order='file')
    state['names'] = {b['code']: short_name(b) for b in books}
    colw = (PW - ML - MR - GAP) / 2
    hgt = PH - MT - MB
    two = [Frame(ML, MB, colw, hgt, id='l', leftPadding=0, rightPadding=0, topPadding=0, bottomPadding=0),
           Frame(ML + colw + GAP, MB, colw, hgt, id='r', leftPadding=0, rightPadding=0, topPadding=0, bottomPadding=0)]
    th = 26 * mm
    start = [Frame(ML, PH - MT - th, PW - ML - MR, th, id='t', leftPadding=0, rightPadding=0, topPadding=0, bottomPadding=0),
             Frame(ML, MB, colw, hgt - th, id='l2', leftPadding=0, rightPadding=0, topPadding=0, bottomPadding=0),
             Frame(ML + colw + GAP, MB, colw, hgt - th, id='r2', leftPadding=0, rightPadding=0, topPadding=0, bottomPadding=0)]
    one = [Frame(ML, MB, PW - ML - MR, hgt, id='o')]
    doc = Doc(OUT, pagesize=B5, title='Die Heilige Schrift - Menge-Bibel', author='Hermann Menge',
              creator='set-F make_f03_menge_reportlab.py', subject='Bibel mit Apokryphen')
    doc.addPageTemplates([PageTemplate('one', one, onPageEnd=page_end), PageTemplate('start', start, onPageEnd=page_end),
                          PageTemplate('two', two, onPageEnd=page_end)])
    st = []
    st.append(Spacer(1, 60 * mm))
    st.append(Paragraph('DIE HEILIGE SCHRIFT', ParagraphStyle('x', parent=btitle, fontSize=24, leading=30)))
    st.append(Paragraph('des Alten und Neuen Testaments', bsub))
    st.append(Spacer(1, 6 * mm))
    st.append(Paragraph('mit den Apokryphen', bsub))
    st.append(Spacer(1, 10 * mm))
    st.append(Paragraph('übersetzt von Hermann Menge', bsub))
    st.append(PageBreak())
    st.append(Paragraph('Inhalt', tochead))
    toc = TableOfContents()
    toc.levelStyles = [toc1]
    st.append(toc)
    dividers = {'GEN': 'Das Alte Testament', 'JDT': 'Die Apokryphen', 'MAT': 'Das Neue Testament'}
    for b in books:
        code = b['code']
        if code in dividers:
            st.append(NextPageTemplate('one'))
            st.append(PageBreak())
            p = Paragraph(dividers[code], div)
            st.append(p)
        st.append(NextPageTemplate('start'))
        st.append(PageBreak())
        st.append(NextPageTemplate('two'))
        p = Paragraph(escape(book_title(b)), btitle)
        p._toc_label = short_name(b)
        st.append(p)
        st.append(FrameBreak())
        chapn = None
        for bl in b['blocks']:
            if bl['kind'] == 'chapter':
                chapn = bl['n']
                st.append(CondPageBreak(30))
                st.append(Paragraph('Kapitel %s' % chapn, chap))
                continue
            if bl['kind'] != 'para':
                continue
            parts = []
            for it in clean_items(bl['items']):
                if it[0] == 'v':
                    parts.append('<onDraw name="verse" label="%s|%s|%s"/><super><font size="5.6" color="#555555">%s</font></super> '
                                 % (code, chapn, it[1], it[1]))
                elif it[0] == 't':
                    parts.append(escape(it[1]))
            st.append(Paragraph(''.join(parts), body))
    doc.multiBuild(st, canvasmaker=make_canvas)
    print(OUT)


def make_canvas(*a, **k):
    from reportlab.pdfgen.canvas import Canvas
    c = Canvas(*a, **k)
    c.setNamedCB('verse', onverse)
    return c


if __name__ == '__main__':
    build()
