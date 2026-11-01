#!/usr/bin/env python3
"""F05: Reina-Valera 1865 (con arreglos ortográficos), complete, as Word .docx.
Generator: python-docx. One Word section per book (book name in the header,
PAGE field in the footer), two text columns, book title as Heading 1,
'CAPÍTULO N' as Heading 2, paragraphs as the source's pilcrows divide them,
superscript verse numbers; verse 1 is shown as a bold large chapter numeral
at the start of each chapter's first paragraph."""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from docx import Document
from docx.shared import Pt, Cm, RGBColor
from docx.enum.text import WD_ALIGN_PARAGRAPH
from docx.enum.section import WD_SECTION
from docx.oxml.ns import qn
from docx.oxml import OxmlElement
from common import load_zip, clean_items, DOCS, short_name, book_title, NT

REF = os.path.join(DOCS, 'F05-spa-rv1865.usfm.zip')
OUT = os.path.join(DOCS, 'F05-spa-rv1865.docx')


def set_cols(section, n, space_cm=0.6):
    sp = section._sectPr
    for c in sp.findall(qn('w:cols')):
        sp.remove(c)
    cols = OxmlElement('w:cols')
    cols.set(qn('w:num'), str(n))
    cols.set(qn('w:space'), str(int(space_cm * 567)))
    if n > 1:
        cols.set(qn('w:sep'), '1')
    sp.append(cols)


def page_field(par):
    r = par.add_run()
    f1 = OxmlElement('w:fldChar'); f1.set(qn('w:fldCharType'), 'begin')
    it = OxmlElement('w:instrText'); it.set(qn('xml:space'), 'preserve'); it.text = ' PAGE '
    f2 = OxmlElement('w:fldChar'); f2.set(qn('w:fldCharType'), 'separate')
    t = OxmlElement('w:t'); t.text = '1'
    f3 = OxmlElement('w:fldChar'); f3.set(qn('w:fldCharType'), 'end')
    for e in (f1, it, f2, t, f3):
        r._r.append(e)


def main():
    books = load_zip(REF)
    doc = Document()
    st = doc.styles['Normal']
    st.font.name = 'Cambria'
    st.font.size = Pt(9.5)
    st.paragraph_format.space_after = Pt(2)
    st.paragraph_format.first_line_indent = Cm(0.4)
    st.paragraph_format.alignment = WD_ALIGN_PARAGRAPH.JUSTIFY
    for name, size in (('Heading 1', 18), ('Heading 2', 10)):
        h = doc.styles[name]
        h.font.name = 'Cambria'
        h.font.size = Pt(size)
        h.font.color.rgb = RGBColor(0x40, 0x10, 0x10)
        h.paragraph_format.alignment = WD_ALIGN_PARAGRAPH.CENTER
        h.paragraph_format.first_line_indent = Cm(0)
    doc.styles['Heading 2'].paragraph_format.space_before = Pt(8)
    doc.styles['Heading 2'].paragraph_format.space_after = Pt(3)
    doc.core_properties.title = 'La Santa Biblia — Reina-Valera 1865'
    doc.core_properties.author = 'Casiodoro de Reina; Cipriano de Valera'
    doc.core_properties.comments = 'Texto de dominio público (scrollmapper/bible_databases SpaRV1865). Maquetación: set-F.'
    sec = doc.sections[0]
    sec.page_width, sec.page_height = Cm(21.59), Cm(27.94)
    sec.left_margin = sec.right_margin = Cm(2.0)
    sec.top_margin, sec.bottom_margin = Cm(2.2), Cm(2.0)
    p = doc.add_paragraph()
    p.alignment = WD_ALIGN_PARAGRAPH.CENTER
    p.paragraph_format.space_before = Pt(200)
    r = p.add_run('LA SANTA BIBLIA'); r.bold = True; r.font.size = Pt(30)
    for line in ('QUE CONTIENE LOS SAGRADOS LIBROS DEL', 'ANTIGUO Y NUEVO TESTAMENTO', '',
                 'Versión de Cipriano de Valera', 'cotejada con los textos hebreo y griego', '1865'):
        q = doc.add_paragraph(line); q.alignment = WD_ALIGN_PARAGRAPH.CENTER
        q.paragraph_format.first_line_indent = Cm(0)
    first = True
    for b in books:
        code = b['code']
        sec = doc.add_section(WD_SECTION.NEW_PAGE)
        set_cols(sec, 2)
        sec.header.is_linked_to_previous = False
        sec.footer.is_linked_to_previous = False
        hp = sec.header.paragraphs[0]
        hp.text = short_name(b).upper()
        hp.alignment = WD_ALIGN_PARAGRAPH.CENTER
        hp.runs[0].font.size = Pt(8)
        fp = sec.footer.paragraphs[0]
        fp.alignment = WD_ALIGN_PARAGRAPH.CENTER
        page_field(fp)
        if code == 'MAT':
            np_ = doc.add_paragraph()
            np_.alignment = WD_ALIGN_PARAGRAPH.CENTER
            rr = np_.add_run('EL NUEVO TESTAMENTO DE NUESTRO SEÑOR JESU CRISTO'); rr.bold = True; rr.font.size = Pt(13)
        doc.add_heading(book_title(b), level=1)
        chap = None
        newchap = False
        for bl in b['blocks']:
            if bl['kind'] == 'chapter':
                chap = bl['n']
                doc.add_heading(('SALMO %s' if code == 'PSA' else 'CAPÍTULO %s') % chap, level=2)
                newchap = True
                continue
            if bl['kind'] != 'para':
                continue
            par = doc.add_paragraph()
            if newchap:
                par.paragraph_format.first_line_indent = Cm(0)
            for it in clean_items(bl['items']):
                if it[0] == 'v':
                    if newchap and it[1] == '1' and code != 'PSA':
                        rn = par.add_run(chap + ' ')
                        rn.bold = True
                        rn.font.size = Pt(16)
                        rn.font.color.rgb = RGBColor(0x40, 0x10, 0x10)
                    else:
                        rn = par.add_run(it[1])
                        rn.font.superscript = True
                        rn.font.color.rgb = RGBColor(0x80, 0x20, 0x20)
                        par.add_run(' ')
                    newchap = False
                elif it[0] == 't':
                    par.add_run(it[1])
    doc.save(OUT)
    print(OUT)


if __name__ == '__main__':
    main()
