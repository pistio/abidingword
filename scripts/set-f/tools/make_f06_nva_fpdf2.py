#!/usr/bin/env python3
"""F06: Bíblia Nova Versão de Acesso Livre - Novo Testamento.
Generator: fpdf2. A5 page, single column, ragged right, chapter headings
'— Capítulo N —', bold coloured verse numbers on the baseline, running head
'Livro N' (the chapter in progress at the top of the page) with a rule, page
number 'Página N' in the footer, an index (Índice) with page numbers built by
fpdf2's table-of-contents placeholder."""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from fpdf import FPDF
import fpdf
from common import load_zip, clean_items, DOCS, short_name

REF = os.path.join(DOCS, 'F06-por-nva.usfm.zip')
OUT = os.path.join(DOCS, 'F06-por-nva.pdf')
F = '/usr/share/fonts/truetype/noto/'


class Bible(FPDF):
    cur = None

    def header(self):
        if self.page_no() <= 3 or not self.cur:
            return
        self.set_font('serif', 'I', 8)
        self.set_text_color(60, 60, 60)
        self.set_y(8)
        self.cell(0, 5, '%s %s' % self.cur, align='R' if self.page_no() % 2 else 'L')
        self.line(self.l_margin, 13.5, self.w - self.r_margin, 13.5)
        self.set_y(17)
        self.set_text_color(0, 0, 0)

    def footer(self):
        if self.page_no() <= 1:
            return
        self.set_y(-11)
        self.set_font('serif', '', 7.5)
        self.cell(0, 5, 'Página %d' % self.page_no(), align='C')


def render_toc(pdf, outline):
    pdf.set_font('serif', 'B', 13)
    pdf.cell(0, 10, 'Índice', align='C', new_x='LMARGIN', new_y='NEXT')
    pdf.set_font('serif', '', 9.5)
    for s in outline:
        pdf.cell(100, 5.4, s.name)
        pdf.cell(0, 5.4, str(s.page_number), align='R', new_x='LMARGIN', new_y='NEXT')


def main():
    books = load_zip(REF)
    pdf = Bible(format='A5')
    pdf.set_margins(15, 17, 13)
    pdf.set_auto_page_break(True, 15)
    pdf.add_font('serif', '', F + 'NotoSerif-Regular.ttf')
    pdf.add_font('serif', 'B', F + 'NotoSerif-Bold.ttf')
    pdf.add_font('serif', 'I', F + 'NotoSerif-Italic.ttf')
    pdf.set_title('Bíblia Nova Versão de Acesso Livre - Novo Testamento')
    pdf.set_author('Bíblia NVA')
    pdf.set_creator('set-F make_f06_nva_fpdf2.py')
    pdf.set_lang('pt-BR')
    pdf.add_page()
    pdf.set_font('serif', 'B', 20)
    pdf.set_y(70)
    pdf.cell(0, 12, 'NOVO TESTAMENTO', align='C', new_x='LMARGIN', new_y='NEXT')
    pdf.set_font('serif', '', 11)
    pdf.cell(0, 8, 'Bíblia Nova Versão de Acesso Livre', align='C', new_x='LMARGIN', new_y='NEXT')
    pdf.set_y(170)
    pdf.set_font('serif', '', 7)
    pdf.multi_cell(0, 3.6, 'Texto licenciado sob Creative Commons Atribuição-CompartilhaIgual 4.0 (CC BY-SA 4.0). '
                   'Fonte: scrollmapper/bible_databases (PorNVA). Diagramação: conjunto de teste F.', align='C')
    pdf.add_page()
    pdf.insert_toc_placeholder(render_toc, pages=1)
    LH = 4.6
    for b in books:
        name = short_name(b)
        pdf.add_page()
        pdf.start_section(name)
        pdf.cur = (name, '1')
        pdf.set_font('serif', 'B', 17)
        pdf.cell(0, 12, name.upper(), align='C', new_x='LMARGIN', new_y='NEXT')
        chap = None
        for bl in b['blocks']:
            if bl['kind'] == 'chapter':
                chap = bl['n']
                if pdf.get_y() > pdf.h - 35:
                    pdf.add_page()
                pdf.cur = (name, chap)
                pdf.ln(2)
                pdf.set_font('serif', 'B', 9)
                pdf.set_text_color(120, 40, 20)
                pdf.cell(0, 6, '— Capítulo %s —' % chap, align='C', new_x='LMARGIN', new_y='NEXT')
                pdf.set_text_color(0, 0, 0)
                continue
            if bl['kind'] != 'para':
                continue
            for it in clean_items(bl['items']):
                if it[0] == 'v':
                    pdf.set_font('serif', 'B', 7.5)
                    pdf.set_text_color(30, 100, 60)
                    pdf.write(LH, it[1] + ' ')
                    pdf.set_text_color(0, 0, 0)
                elif it[0] == 't':
                    pdf.set_font('serif', '', 9)
                    pdf.write(LH, it[1] + ' ')
            pdf.ln(LH + 0.8)
    pdf.output(OUT)
    print(OUT, pdf.page_no())


if __name__ == '__main__':
    main()
