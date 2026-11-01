"""deutkw (Textbibel 1906) prophets -> A5 PDF with fpdf2. Verse numbers hung in the outer margin,
one verse per line start, 'Kapitel N' headings, section headings in italics, running head with book name
and page number on the outer edge, Inhalt (contents) at the end."""
import sys
sys.path.insert(0, '/root/aw-set-e/tools')
import usfm
from fpdf import FPDF

SRC, OUT = sys.argv[1], sys.argv[2]
CODES = 'ISA JER LAM EZK DAN HOS JOL AMO OBA JON MIC NAM HAB ZEP HAG ZEC MAL'.split()
books = usfm.load(SRC)
seq = [books[c] for c in CODES]
L = '/usr/share/fonts/truetype/liberation/LiberationSerif-'


class P(FPDF):
    head = ''

    def header(self):
        if self.page_no() <= 2 or not self.head:
            return
        self.set_font('LS', 'I', 8)
        self.set_y(8)
        odd = self.page_no() % 2 == 1
        if odd:
            self.set_x(12); self.cell(0, 4, self.head, align='L')
            self.set_xy(12, 8); self.set_font('LS', 'B', 8); self.cell(self.w - 24, 4, str(self.page_no()), align='R')
        else:
            self.set_x(12); self.set_font('LS', 'B', 8); self.cell(10, 4, str(self.page_no()), align='L')
            self.set_xy(12, 8); self.set_font('LS', 'I', 8); self.cell(self.w - 24, 4, self.head, align='R')
        self.set_draw_color(120); self.set_line_width(0.2); self.line(12, 13, self.w - 12, 13)
        self.set_y(17)


pdf = P(format='A5')
pdf.add_font('LS', '', L + 'Regular.ttf'); pdf.add_font('LS', 'B', L + 'Bold.ttf')
pdf.add_font('LS', 'I', L + 'Italic.ttf'); pdf.add_font('LS', 'BI', L + 'BoldItalic.ttf')
pdf.set_auto_page_break(True, 14)
pdf.set_margins(19, 17, 12)
pdf.set_title('Die Propheten - Textbibel von Kautzsch und Weizsäcker')
pdf.set_author('Kautzsch und Weizsäcker')
pdf.set_creator('fpdf2 / gen_deutkw_fpdf.py')

pdf.add_page()
pdf.set_font('LS', 'B', 24); pdf.set_y(60); pdf.cell(0, 12, 'Die Propheten', align='C', new_x='LMARGIN', new_y='NEXT')
pdf.set_font('LS', '', 12); pdf.cell(0, 8, 'nach der Textbibel von Kautzsch und Weizsäcker', align='C', new_x='LMARGIN', new_y='NEXT')
pdf.set_font('LS', 'I', 10); pdf.cell(0, 8, 'Jesaja bis Maleachi', align='C', new_x='LMARGIN', new_y='NEXT')
pdf.add_page()
pdf.set_y(150); pdf.set_font('LS', '', 8)
pdf.multi_cell(0, 4, 'The Holy Bible in German, translation by Kautzsch und Weizsäcker 1906. Public Domain.\n'
               'Quelle: eBible.org (deutkw). Textbibel.de (Hans J. Herbst Mitarbeit bei der Digitalisierung).')
toc = []
for bk in seq:
    pdf.head = bk.meta.get('h', bk.name).strip()
    pdf.add_page()
    toc.append((bk.long_name.strip(), pdf.page_no()))
    pdf.set_font('LS', 'B', 16)
    pdf.multi_cell(0, 8, bk.long_name.strip(), align='C', new_x='LMARGIN', new_y='NEXT')
    pdf.ln(3)
    ch = None
    for b in bk.blocks:
        if b[0] == 'c':
            ch = b[1]
            if pdf.get_y() > pdf.h - 35:
                pdf.add_page()
            pdf.ln(2)
            pdf.set_font('LS', 'B', 10.5)
            pdf.cell(0, 6, 'Kapitel %s' % ch, align='C', new_x='LMARGIN', new_y='NEXT')
            continue
        if b[0] == 'h':
            pdf.set_font('LS', 'I', 9.2)
            pdf.ln(1)
            pdf.multi_cell(0, 4.4, usfm.plain(b[2]), align='L', new_x='LMARGIN', new_y='NEXT')
            continue
        # split paragraph into verses; first verse of a paragraph gets a small extra space
        cur_v = None; buf = []
        segs = []
        for it in b[2]:
            if it[0] == 'v':
                if cur_v is not None or buf:
                    segs.append((cur_v, ''.join(buf)))
                cur_v, buf = it[1], []
            elif it[0] == 't':
                buf.append(it[1])
        segs.append((cur_v, ''.join(buf)))
        pdf.ln(0.8)
        for v, t in segs:
            t = t.strip()
            if v is None and not t:
                continue
            if pdf.get_y() > pdf.h - 20:
                pdf.add_page()
            y = pdf.get_y()
            if v is not None:
                pdf.set_font('LS', 'B', 7)
                pdf.set_xy(11, y + 0.3)
                pdf.cell(7, 4, v, align='R')
                pdf.set_xy(19, y)
            pdf.set_font('LS', '', 9.6)
            pdf.multi_cell(0, 4.5, t, align='J', new_x='LMARGIN', new_y='NEXT')
pdf.head = 'Inhalt'
pdf.add_page()
pdf.set_font('LS', 'B', 14); pdf.cell(0, 8, 'Inhalt', align='C', new_x='LMARGIN', new_y='NEXT'); pdf.ln(3)
pdf.set_font('LS', '', 10)
for name, pg in toc:
    pdf.cell(95, 6, name); pdf.cell(0, 6, str(pg), align='R', new_x='LMARGIN', new_y='NEXT')
pdf.output(OUT)
print('ok', pdf.page_no())
