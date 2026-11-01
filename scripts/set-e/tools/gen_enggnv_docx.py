"""enggnv (Geneva 1599) New Testament -> Word .docx with python-docx.
One verse per paragraph, bold chapter:verse at line start, 'CHAP. I.' Roman chapter headings,
section per book with its own running header, PAGE field footer, TOC field."""
import sys
sys.path.insert(0, '/root/aw-set-e/tools')
import usfm
from docx import Document
from docx.shared import Pt, Inches, RGBColor
from docx.enum.text import WD_ALIGN_PARAGRAPH, WD_BREAK
from docx.enum.section import WD_SECTION
from docx.oxml.ns import qn
from docx.oxml import OxmlElement

SRC, OUT = sys.argv[1], sys.argv[2]


def roman(n):
    vals = [(1000, 'M'), (900, 'CM'), (500, 'D'), (400, 'CD'), (100, 'C'), (90, 'XC'), (50, 'L'), (40, 'XL'),
            (10, 'X'), (9, 'IX'), (5, 'V'), (4, 'IV'), (1, 'I')]
    s = ''
    for v, r in vals:
        while n >= v:
            s += r; n -= v
    return s


def field(par, instr, placeholder=''):
    r = par.add_run()
    b = OxmlElement('w:fldChar'); b.set(qn('w:fldCharType'), 'begin'); r._r.append(b)
    r2 = par.add_run()
    it = OxmlElement('w:instrText'); it.set(qn('xml:space'), 'preserve'); it.text = instr; r2._r.append(it)
    r3 = par.add_run()
    s = OxmlElement('w:fldChar'); s.set(qn('w:fldCharType'), 'separate'); r3._r.append(s)
    if placeholder:
        par.add_run(placeholder)
    r4 = par.add_run()
    e = OxmlElement('w:fldChar'); e.set(qn('w:fldCharType'), 'end'); r4._r.append(e)


doc = Document()
st = doc.styles
st['Normal'].font.name = 'Garamond'
st['Normal'].font.size = Pt(10.5)
st['Normal'].element.rPr.rFonts.set(qn('w:eastAsia'), 'Garamond')
pf = st['Normal'].paragraph_format
pf.space_after = Pt(0); pf.space_before = Pt(0); pf.line_spacing = 1.08
for name, size in [('Title', 30), ('Heading 1', 18), ('Heading 2', 11)]:
    s = st[name]; s.font.name = 'Garamond'; s.font.size = Pt(size); s.font.color.rgb = RGBColor(0x20, 0x20, 0x20)
st['Heading 2'].paragraph_format.alignment = WD_ALIGN_PARAGRAPH.CENTER
st['Heading 2'].paragraph_format.space_before = Pt(9)
st['Heading 2'].paragraph_format.space_after = Pt(3)
vs = st.add_style('Verse', 1)
vs.base_style = st['Normal']
vs.paragraph_format.left_indent = Inches(0.38)
vs.paragraph_format.first_line_indent = Inches(-0.38)
vs.paragraph_format.space_after = Pt(2)

sec = doc.sections[0]
sec.page_width, sec.page_height = Inches(8.5), Inches(11)
sec.left_margin = sec.right_margin = Inches(1)
sec.top_margin = sec.bottom_margin = Inches(1)

p = doc.add_paragraph('THE NEWE TESTAMENT', style='Title'); p.alignment = WD_ALIGN_PARAGRAPH.CENTER
p = doc.add_paragraph(); p.alignment = WD_ALIGN_PARAGRAPH.CENTER
p.add_run('Geneva Bible 1599').italic = True
p = doc.add_paragraph(); p.alignment = WD_ALIGN_PARAGRAPH.CENTER
p.add_run('The Geneva Bible in Old English of 1599. Public Domain. This digital copy is freely available world-wide, '
          'with no copyright restrictions, courtesy of eBible.org. Note that the spelling used is the original spelling.').font.size = Pt(9)
p.add_run().add_break(WD_BREAK.PAGE)
doc.add_paragraph('Contents', style='Heading 1')
p = doc.add_paragraph()
field(p, 'TOC \\o "1-1" \\h \\z \\u', 'Right-click to update the table of contents.')

books = usfm.load(SRC)
first = True
for bk in usfm.ordered(books, usfm.NT):
    ns = doc.add_section(WD_SECTION.NEW_PAGE)
    ns.header.is_linked_to_previous = False
    ns.footer.is_linked_to_previous = False
    hp = ns.header.paragraphs[0]
    hp.text = bk.meta.get('h', bk.name).strip()
    hp.alignment = WD_ALIGN_PARAGRAPH.RIGHT
    for r in hp.runs:
        r.font.small_caps = True; r.font.size = Pt(9)
    fp = ns.footer.paragraphs[0]; fp.alignment = WD_ALIGN_PARAGRAPH.CENTER
    fp.add_run('Page ')
    field(fp, 'PAGE', '1')
    doc.add_paragraph(bk.long_name.strip(), style='Heading 1').alignment = WD_ALIGN_PARAGRAPH.CENTER
    ch = None
    for b in bk.blocks:
        if b[0] == 'c':
            ch = b[1]
            doc.add_paragraph('CHAP. %s.' % roman(int(ch)), style='Heading 2')
            continue
        if b[0] != 'p':
            continue
        par = None
        for it in b[2]:
            if it[0] == 'v':
                par = doc.add_paragraph(style='Verse')
                r = par.add_run('%s:%s' % (ch, it[1])); r.bold = True
                par.add_run('\t')
            elif it[0] == 't':
                if par is None:
                    par = doc.add_paragraph(style='Verse')
                txt = it[1]
                if par.runs and par.runs[-1].text == '\t':
                    txt = txt.lstrip()
                r = par.add_run(txt)
                if it[2] & {'add', 'em', 'it'}:
                    r.italic = True
doc.core_properties.title = 'The Newe Testament (Geneva Bible 1599)'
doc.core_properties.author = 'eBible.org enggnv'
doc.save(OUT)
print('ok')
