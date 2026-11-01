"""engwebu -> two-column red-letter reference Bible PDF with ReportLab (two-pass for running heads/TOC)."""
import sys, re, html
sys.path.insert(0, '/root/aw-set-e/tools')
import usfm
from reportlab.lib.pagesizes import inch
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.lib.fonts import addMapping
from reportlab.lib.styles import ParagraphStyle
from reportlab.lib.enums import TA_JUSTIFY, TA_CENTER, TA_LEFT, TA_RIGHT
from reportlab.platypus import (BaseDocTemplate, PageTemplate, Frame, Paragraph, Spacer, PageBreak,
                                NextPageTemplate, FrameBreak, CondPageBreak, KeepTogether, Flowable)

SRC, OUT = sys.argv[1], sys.argv[2]
N = '/usr/share/fonts/truetype/noto/NotoSerif-'
for nm, f in [('NS', 'Regular'), ('NS-B', 'Bold'), ('NS-I', 'Italic'), ('NS-BI', 'BoldItalic')]:
    pdfmetrics.registerFont(TTFont(nm, N + f + '.ttf'))
addMapping('NS', 0, 0, 'NS'); addMapping('NS', 1, 0, 'NS-B'); addMapping('NS', 0, 1, 'NS-I'); addMapping('NS', 1, 1, 'NS-BI')

PW, PH = 6.14 * inch, 9.21 * inch
IM, OM, TM, BM = 0.62 * inch, 0.5 * inch, 0.72 * inch, 0.6 * inch
GUT = 0.22 * inch

body = ParagraphStyle('body', fontName='NS', fontSize=8.4, leading=10.2, alignment=TA_JUSTIFY, firstLineIndent=8)
bodym = ParagraphStyle('bodym', parent=body, firstLineIndent=0)
q1 = ParagraphStyle('q1', parent=body, alignment=TA_LEFT, leftIndent=16, firstLineIndent=-10)
q2 = ParagraphStyle('q2', parent=body, alignment=TA_LEFT, leftIndent=24, firstLineIndent=-6)
q3 = ParagraphStyle('q3', parent=body, alignment=TA_LEFT, leftIndent=30, firstLineIndent=-4)
pi = ParagraphStyle('pi', parent=body, leftIndent=10)
li = ParagraphStyle('li', parent=body, alignment=TA_LEFT, leftIndent=18, firstLineIndent=-10)
pc = ParagraphStyle('pc', parent=body, alignment=TA_CENTER, firstLineIndent=0)
dsty = ParagraphStyle('d', parent=body, fontName='NS-I', alignment=TA_LEFT, firstLineIndent=0, spaceAfter=2)
sp = ParagraphStyle('sp', parent=body, fontName='NS-I', fontSize=7.6, alignment=TA_LEFT, firstLineIndent=0, spaceBefore=3)
s1 = ParagraphStyle('s1', parent=body, fontName='NS-B', fontSize=8.6, alignment=TA_CENTER, firstLineIndent=0, spaceBefore=5, spaceAfter=2)
chap = ParagraphStyle('chap', fontName='NS-B', fontSize=20, leading=22, spaceBefore=6, spaceAfter=1)
psh = ParagraphStyle('psh', fontName='NS-B', fontSize=9, leading=11, alignment=TA_CENTER, spaceBefore=7, spaceAfter=2)
notes = ParagraphStyle('notes', fontName='NS', fontSize=6.4, leading=7.6, alignment=TA_LEFT, leftIndent=6, firstLineIndent=-6,
                       spaceBefore=1)
rule = ParagraphStyle('rule', fontName='NS', fontSize=4, leading=5)
title = ParagraphStyle('title', fontName='NS-B', fontSize=17, leading=21, alignment=TA_CENTER, spaceBefore=4)
title2 = ParagraphStyle('title2', fontName='NS-I', fontSize=10, leading=13, alignment=TA_CENTER)
intro = ParagraphStyle('intro', fontName='NS', fontSize=8.6, leading=11, alignment=TA_JUSTIFY, firstLineIndent=10, spaceAfter=2)
introh = ParagraphStyle('introh', fontName='NS-B', fontSize=10, leading=13, spaceBefore=8, spaceAfter=3)
toc = ParagraphStyle('toc', fontName='NS', fontSize=9.5, leading=12.5)
glo = ParagraphStyle('glo', parent=body, firstLineIndent=0, spaceAfter=2)

STY = {'p': body, 'm': bodym, 'nb': bodym, 'mi': pi, 'pi': pi, 'pi1': pi, 'q': q1, 'q1': q1, 'q2': q2, 'q3': q3,
       'li': li, 'li1': li, 'li2': li, 'pc': pc, 'ip': intro, 'ili': li, 'ili1': li}


def esc(t):
    return html.escape(t, quote=False)


class Ref(Flowable):
    """Zero-size marker recording the reference reached in the text flow."""
    def __init__(self, book, ref):
        super().__init__(); self.book, self.ref = book, ref
    def wrap(self, w, h):
        return 0, 0
    def draw(self):
        pass


def letters(n):
    s = ''
    while True:
        s = chr(97 + n % 26) + s
        n = n // 26 - 1
        if n < 0:
            return s


class Doc(BaseDocTemplate):
    def __init__(self, fn, heads, **kw):
        super().__init__(fn, pagesize=(PW, PH), **kw)
        self.heads = heads          # page -> (first, last, bookname) from previous pass
        self.cur = {}               # page -> [first, last]
        self.bookpages = {}
        self.curbook = ''
        def frames(bookstart):
            out = []
            for side in (0, 1):
                pass
            return out
        def mk(id_, odd, start):
            l = IM if odd else OM
            r = OM if odd else IM
            w = PW - l - r
            cw = (w - GUT) / 2
            top = PH - TM
            fr = []
            if start:
                fr.append(Frame(l, top - 1.25 * inch, w, 1.25 * inch, id='t', leftPadding=0, rightPadding=0, topPadding=0, bottomPadding=0))
                top -= 1.25 * inch
            fr.append(Frame(l, BM, cw, top - BM, id='c1', leftPadding=0, rightPadding=0, topPadding=0, bottomPadding=0))
            fr.append(Frame(l + cw + GUT, BM, cw, top - BM, id='c2', leftPadding=0, rightPadding=0, topPadding=0, bottomPadding=0))
            return fr
        # odd/even handled by drawing; frames use average margins for simplicity of templates
        self.addPageTemplates([
            PageTemplate('front', [Frame(0.8 * inch, 0.8 * inch, PW - 1.6 * inch, PH - 1.6 * inch, id='f')], onPage=self.front),
            PageTemplate('intro', [Frame(IM, BM, PW - IM - OM, PH - TM - BM, id='i')], onPage=self.introp),
            PageTemplate('start', mk('s', True, True), onPage=self.page),
            PageTemplate('normal', mk('n', True, False), onPage=self.page),
        ])

    def front(self, c, d):
        pass

    def introp(self, c, d):
        c.saveState(); c.setFont('NS', 7.5)
        c.drawCentredString(PW / 2, 0.38 * inch, str(d.page))
        c.restoreState()

    def page(self, c, d):
        p = d.page
        first, last, bk = self.heads.get(p, ('', '', ''))
        c.saveState()
        c.setFont('NS-B', 7.6)
        y = PH - 0.5 * inch
        # gutter rule between columns
        l = IM; w = PW - IM - OM; cw = (w - GUT) / 2
        c.setLineWidth(0.3)
        top = PH - TM - (1.25 * inch if d.pageTemplate.id == 'start' else 0)
        c.line(l + cw + GUT / 2, BM, l + cw + GUT / 2, top)
        if d.pageTemplate.id != 'start':
            c.drawString(IM, y, first)
            c.drawRightString(PW - OM, y, last)
            c.setFont('NS', 7.6)
            c.drawCentredString(PW / 2, y, str(p))
            c.line(IM, y - 4, PW - OM, y - 4)
        else:
            c.setFont('NS', 7.6)
            c.drawCentredString(PW / 2, 0.38 * inch, str(p))
        c.restoreState()

    def afterFlowable(self, f):
        if isinstance(f, Ref):
            ent = self.cur.setdefault(self.page, [None, None, f.book])
            if ent[0] is None:
                ent[0] = f.ref
            ent[1] = f.ref
        if isinstance(f, Paragraph) and getattr(f, '_bookstart', None):
            self.bookpages.setdefault(f._bookstart, self.page)


def inline_markup(inl, chapter, noteacc, counter, refs_out, book):
    out = []
    for it in inl:
        if it[0] == 't':
            t = esc(it[1])
            st = it[2]
            if 'wj' in st:
                t = '<font color="#9b111e">%s</font>' % t
            if st & {'add', 'em', 'it', 'qs', 'bk', 'tl'}:
                t = '<i>%s</i>' % t
            if st & {'nd', 'sc'}:
                t = '<font size="7.4">%s</font>' % t
            if 'k' in st or 'bd' in st:
                t = '<b>%s</b>' % t
            out.append(t)
        elif it[0] == 'v':
            refs_out.append(it[1])
            out.append('<font name="NS-B" size="5.6"><super>%s</super></font> ' % esc(it[1]))
        elif it[0] == 'n':
            ref, txt = usfm.note_text(it[3])
            mark = letters(counter[0]); counter[0] += 1
            sym = mark if it[1].startswith('f') else '†' + mark
            out.append('<font size="5.2"><super><i>%s</i></super></font>' % sym)
            noteacc.append((sym, ref, txt))
    return ''.join(out)


def nd_fix(markup):
    return markup


def build(heads, bookpages_prev, total_prev):
    books = usfm.load(SRC)
    seq = usfm.ordered(books)
    story = []
    doc = Doc(OUT, heads, title='The Holy Bible: World English Bible Updated', author='Public Domain',
              subject='World English Bible Updated (eBible.org engwebu)', creator='aw-set-e gen_engwebu_rl2col.py')
    story.append(NextPageTemplate('front'))
    story.append(Spacer(1, 2.2 * inch))
    story.append(Paragraph('THE HOLY BIBLE', ParagraphStyle('tt', fontName='NS-B', fontSize=26, leading=32, alignment=TA_CENTER)))
    story.append(Spacer(1, 0.25 * inch))
    story.append(Paragraph('World English Bible Updated', ParagraphStyle('tt2', fontName='NS-I', fontSize=14, leading=18, alignment=TA_CENTER)))
    story.append(Spacer(1, 0.12 * inch))
    story.append(Paragraph('with the Deuterocanonical Books', ParagraphStyle('tt3', fontName='NS', fontSize=10, leading=14, alignment=TA_CENTER)))
    story.append(Spacer(1, 0.12 * inch))
    story.append(Paragraph('Red Letter Reference Edition', ParagraphStyle('tt3', fontName='NS', fontSize=10, leading=14, alignment=TA_CENTER)))
    story.append(PageBreak())
    story.append(Spacer(1, 4.6 * inch))
    cp = ParagraphStyle('cp', fontName='NS', fontSize=7.5, leading=10, alignment=TA_LEFT)
    for line in ['The World English Bible is in the Public Domain. That means that it is not copyrighted.',
                 'Source text: eBible.org, engwebu USFM. Words of Christ are printed in red.',
                 'Footnotes and cross references are gathered at the end of each chapter.']:
        story.append(Paragraph(line, cp))
    story.append(NextPageTemplate('intro'))
    story.append(PageBreak())
    story.append(Paragraph('Contents', ParagraphStyle('ch', fontName='NS-B', fontSize=14, leading=18, spaceAfter=8)))
    groups = [('The Old Testament', usfm.CANON[1:40]), ('The Deuterocanonical Books', usfm.CANON[40:59]),
              ('The New Testament', usfm.NT)]
    for gname, codes in groups:
        story.append(Paragraph('<b>%s</b>' % gname, ParagraphStyle('tg', parent=toc, spaceBefore=6)))
        for c in codes:
            if c in books:
                pg = bookpages_prev.get(c, '')
                story.append(Paragraph('%s <font color="#777777">%s</font> %s' % (esc(books[c].long_name), '.' * 8, pg), toc))
    if 'GLO' in books:
        story.append(Paragraph('<b>Glossary</b> %s %s' % ('.' * 8, bookpages_prev.get('GLO', '')), ParagraphStyle('tg', parent=toc, spaceBefore=6)))
    story.append(PageBreak())
    # preface
    if 'FRT' in books:
        fb = books['FRT']
        story.append(Paragraph(esc(fb.mt[0][1]) if fb.mt else 'Preface', ParagraphStyle('ph', fontName='NS-B', fontSize=14, leading=18, spaceAfter=8)))
        for b in fb.blocks:
            m = inline_markup(b[2], '', [], [0], [], fb)
            story.append(Paragraph(m, introh if b[0] == 'h' else intro))
    current_group = None
    for bk in seq:
        if bk.code in ('FRT', 'GLO'):
            continue
        grp = 'OT' if bk.code in usfm.CANON[1:40] else ('DC' if bk.code in usfm.CANON[40:59] else 'NT')
        if grp != current_group:
            current_group = grp
            story.append(NextPageTemplate('front'))
            story.append(PageBreak())
            story.append(Spacer(1, 3 * inch))
            story.append(Paragraph({'OT': 'THE OLD TESTAMENT', 'DC': 'THE DEUTEROCANONICAL BOOKS', 'NT': 'THE NEW TESTAMENT'}[grp],
                                   ParagraphStyle('gt', fontName='NS-B', fontSize=18, leading=24, alignment=TA_CENTER)))
        story.append(NextPageTemplate('start'))
        story.append(PageBreak())
        mts = sorted(bk.mt, key=lambda x: x[0])
        tp = None
        for lvl, t in bk.mt:
            p = Paragraph(esc(t), title if lvl == 1 else title2)
            if tp is None:
                p._bookstart = bk.code
                tp = p
            story.append(p)
        if tp is None:
            p = Paragraph(esc(bk.long_name), title); p._bookstart = bk.code; story.append(p)
        story.append(NextPageTemplate('normal'))
        story.append(FrameBreak())
        head = bk.meta.get('h', bk.name)
        chapter = None
        noteacc = []
        counter = [0]
        def flush_notes():
            if noteacc:
                story.append(Paragraph('<font color="#999999">%s</font>' % ('—' * 6), rule))
                for sym, ref, txt in noteacc:
                    story.append(Paragraph('<i>%s</i> <b>%s</b> %s' % (esc(sym), esc(ref), esc(txt)), notes))
                noteacc.clear()
        pending_heads = []
        for b in bk.blocks:
            if b[0] == 'c':
                flush_notes(); counter[0] = 0
                chapter = b[3] or b[1]
                lab = b[2] or (('%s %s' % (bk.chapter_label, chapter)) if bk.chapter_label else None)
                if lab:
                    pending_heads.append(Paragraph(esc(lab).upper(), psh))
                else:
                    pending_heads.append(Paragraph(esc(chapter), chap))
                continue
            if b[0] == 'h':
                st = {'d': dsty, 'sp': sp}.get(b[1], s1)
                pending_heads.append(Paragraph(inline_markup(b[2], chapter, noteacc, counter, [], bk), st))
                continue
            if b[1] == 'b':
                story.append(Spacer(1, 4)); continue
            refs = []
            m = inline_markup(b[2], chapter, noteacc, counter, refs, bk)
            st = STY.get(b[1], body)
            para = Paragraph(m, st)
            items = pending_heads + [para]
            pending_heads = []
            if len(items) > 1:
                story.append(KeepTogether(items[:-1] + [para]) if len(items) < 4 else items[0])
                if len(items) >= 4:
                    story.extend(items[1:])
            else:
                story.append(para)
            ch = chapter or ''
            for r in refs[:1]:
                story.append(Ref(head, '%s %s:%s' % (head, ch, r)) if ch else Ref(head, head))
            if refs[1:]:
                story.append(Ref(head, '%s %s:%s' % (head, ch, refs[-1])) if ch else Ref(head, head))
            elif not refs and ch:
                story.append(Ref(head, '%s %s' % (head, ch)))
        story.extend(pending_heads)
        flush_notes()
    if 'GLO' in books:
        g = books['GLO']
        story.append(NextPageTemplate('intro'))
        story.append(PageBreak())
        p = Paragraph(esc(g.mt[0][1] if g.mt else 'Glossary'), ParagraphStyle('gh', fontName='NS-B', fontSize=14, leading=18, spaceAfter=8))
        p._bookstart = 'GLO'
        story.append(p)
        for b in g.blocks:
            story.append(Paragraph(inline_markup(b[2], '', [], [0], [], g), introh if b[0] == 'h' else glo))
    doc.build(story)
    heads2 = {pg: (v[0] or '', v[1] or '', v[2]) for pg, v in doc.cur.items()}
    return heads2, doc.bookpages, doc.page


h, bp, total = build({}, {}, 0)
h, bp2, total = build(h, bp, total)
if bp2 != bp:
    h, bp2, total = build(h, bp2, total)
print('pages', total)
