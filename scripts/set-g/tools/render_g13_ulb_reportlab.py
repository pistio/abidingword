"""G13: render the Unlocked Literal Bible selection (eBible engULB: Ruth, Jonah, Mark,
Romans, Philemon, Jude) as a PDF with ReportLab's platypus layout engine.
Layout: US Letter, two columns with a rule between them; running head "Mark 4:12 – 5:3"
(first and last verse on the page) on the outer side and the book name on the inner side;
page number at the bottom outer corner; book title in a sans-serif band; chapter numbers
as large bold numerals on a line of their own above the chapter; superscript verse
numbers (verse 1 not printed); poetry (\\q1/\\q2) as indented lines; the translators'
footnotes marked with superscript letters and collected as "Notes" at the end of each
chapter, each note led by its chapter:verse reference.
Usage: python3 -I render_g13_ulb_reportlab.py REF.zip OUT.pdf
"""
import html
import re
import sys

sys.path.insert(0, "/root/aw-set-g/tools")
import usfmlib as U  # noqa: E402
from reportlab.lib.pagesizes import letter  # noqa: E402
from reportlab.lib.styles import ParagraphStyle  # noqa: E402
from reportlab.lib.units import inch  # noqa: E402
from reportlab.pdfbase import pdfmetrics  # noqa: E402
from reportlab.pdfbase.ttfonts import TTFont  # noqa: E402
from reportlab.platypus import BaseDocTemplate, Frame, PageTemplate, Paragraph, Spacer, CondPageBreak, PageBreak  # noqa: E402
from reportlab.platypus.flowables import Flowable  # noqa: E402

F = "/usr/share/fonts/truetype/liberation/"
pdfmetrics.registerFont(TTFont("LSerif", F + "LiberationSerif-Regular.ttf"))
pdfmetrics.registerFont(TTFont("LSerif-B", F + "LiberationSerif-Bold.ttf"))
pdfmetrics.registerFont(TTFont("LSerif-I", F + "LiberationSerif-Italic.ttf"))
pdfmetrics.registerFont(TTFont("LSans-B", F + "LiberationSans-Bold.ttf"))
pdfmetrics.registerFontFamily("LSerif", normal="LSerif", bold="LSerif-B", italic="LSerif-I", boldItalic="LSerif-B")

BODY = ParagraphStyle("body", fontName="LSerif", fontSize=9.6, leading=12, firstLineIndent=10, alignment=4, autoLeading="max")
FIRST = ParagraphStyle("first", parent=BODY, firstLineIndent=0)
POEM1 = ParagraphStyle("poem1", parent=BODY, firstLineIndent=-12, leftIndent=22, alignment=0)
POEM2 = ParagraphStyle("poem2", parent=BODY, firstLineIndent=-12, leftIndent=34, alignment=0)
TITLE = ParagraphStyle("title", fontName="LSans-B", fontSize=17, leading=22, alignment=1, spaceBefore=4, spaceAfter=10)
CHNUM = ParagraphStyle("chnum", fontName="LSerif-B", fontSize=17, leading=19, spaceBefore=2, spaceAfter=1, keepWithNext=1)
NOTEH = ParagraphStyle("noteh", fontName="LSans-B", fontSize=7, leading=9, spaceBefore=4)
NOTE = ParagraphStyle("note", fontName="LSerif", fontSize=7.4, leading=9, leftIndent=8, firstLineIndent=-8)

NAMES = {"RUT": "Ruth", "JON": "Jonah", "MRK": "Mark", "ROM": "Romans", "PHM": "Philemon", "JUD": "Jude"}


class Mark(Flowable):
    """Zero-size flowable recording the current verse reference for the running head."""

    def __init__(self, ref):
        super().__init__()
        self.ref = ref

    def wrap(self, *a):
        return (0, 0)

    def draw(self):
        doc = self.canv._doctemplate
        if doc.page_first is None:
            doc.page_first = self.ref
        doc.page_last = self.ref


class Doc(BaseDocTemplate):
    def __init__(self, fn):
        super().__init__(fn, pagesize=letter, leftMargin=0.7 * inch, rightMargin=0.7 * inch,
                         topMargin=0.85 * inch, bottomMargin=0.75 * inch,
                         title="Unlocked Literal Bible — Selections", author="Door43 World Missions Community")
        w = (self.width - 0.3 * inch) / 2
        frames = [Frame(self.leftMargin, self.bottomMargin, w, self.height, id="c1"),
                  Frame(self.leftMargin + w + 0.3 * inch, self.bottomMargin, w, self.height, id="c2")]
        self.addPageTemplates([PageTemplate(id="front", frames=[Frame(self.leftMargin, self.bottomMargin, self.width, self.height)]),
                               PageTemplate(id="two", frames=frames, onPageEnd=self.decorate)])
        self.page_first = None
        self.page_last = None
        self.cur_book = ""

    def beforePage(self):
        self.page_first = None

    def decorate(self, canv, doc):
        canv.saveState()
        canv.setStrokeGray(0.6)
        x = self.leftMargin + self.width / 2
        canv.line(x, self.bottomMargin, x, self.bottomMargin + self.height)
        canv.setFont("LSerif", 8.5)
        rng = ""
        if self.page_first:
            rng = self.page_first if self.page_first == self.page_last else f"{self.page_first} – {self.page_last.split(' ', 1)[-1] if self.page_last.split(' ')[0] == self.page_first.split(' ')[0] else self.page_last}"
        y = letter[1] - 0.55 * inch
        odd = doc.page % 2 == 1
        if odd:
            canv.drawRightString(letter[0] - self.rightMargin, y, rng)
            canv.drawString(self.leftMargin, y, self.cur_book)
            canv.drawRightString(letter[0] - self.rightMargin, 0.45 * inch, str(doc.page))
        else:
            canv.drawString(self.leftMargin, y, rng)
            canv.drawRightString(letter[0] - self.rightMargin, y, self.cur_book)
            canv.drawString(self.leftMargin, 0.45 * inch, str(doc.page))
        canv.restoreState()


class BookStart(Flowable):
    def __init__(self, name):
        super().__init__()
        self.name = name

    def wrap(self, *a):
        return (0, 0)

    def draw(self):
        self.canv._doctemplate.cur_book = self.name


def esc(s):
    return html.escape(s, quote=False)


def main(ref, out):
    books = U.read_usfm_zip(ref)
    doc = Doc(out)
    story = [Spacer(1, 2.5 * inch),
             Paragraph("Unlocked Literal Bible", ParagraphStyle("t", parent=TITLE, fontSize=26, leading=32)),
             Paragraph("Ruth · Jonah · Mark · Romans · Philemon · Jude", ParagraphStyle("s", parent=BODY, alignment=1, fontSize=12, leading=16)),
             Spacer(1, 2 * inch),
             Paragraph("The Holy Bible in English, Unlocked Literal Bible translation, copyright © 2017 Door43 World Missions "
                       "Community. Licensed under the Creative Commons Attribution-ShareAlike 4.0 International License. "
                       "Text from eBible.org (engULB); typeset with ReportLab.", ParagraphStyle("c", parent=BODY, alignment=1, fontSize=8, leading=10)),
             PageBreak()]
    from reportlab.platypus.doctemplate import NextPageTemplate
    story.insert(-1, NextPageTemplate("two"))
    for b in books:
        name = NAMES[b["code"]]
        story += [CondPageBreak(2.2 * inch), BookStart(name), Paragraph(esc(name).upper(), TITLE)]
        ch = 0
        cur = []
        style = FIRST
        notes = []
        letter_i = 0

        def flush(next_style=BODY):
            nonlocal cur, style
            if cur:
                story.append(Paragraph("".join(cur), style))
            cur = []
            style = next_style

        def end_chapter():
            nonlocal notes
            flush()
            if notes:
                story.append(Paragraph("Notes", NOTEH))
                for lab, refv, nt in notes:
                    story.append(Paragraph(f"<super>{lab}</super> <b>{refv}</b> {esc(nt)}", NOTE))
            notes = []

        for it in b["items"]:
            if it[0] == "c":
                end_chapter()
                ch = it[1]
                letter_i = 0
                story.append(Spacer(1, 4))
                story.append(Paragraph(f"{ch}", CHNUM))
                cur = []
                style = FIRST
            elif it[0] == "p":
                if cur and not (len(cur) == 1 and cur[0].startswith("<font name='LSerif-B' size='19'>")):
                    flush(BODY)
                elif not cur:
                    style = BODY
            elif it[0] == "q":
                if cur and not (len(cur) == 1 and cur[0].startswith("<font name='LSerif-B' size='19'>")):
                    flush(POEM1 if it[1] == 1 else POEM2)
                else:
                    style = POEM1 if it[1] == 1 else POEM2
            elif it[0] == "v":
                refv = f"{ch}:{it[1]}"
                parts = re.split(r"\x01(\d+)\x01", it[2])
                txt = esc(parts[0])
                for k in range(1, len(parts), 2):
                    lab = "abcdefghijklmnopqrstuvwxyz"[letter_i % 26]
                    letter_i += 1
                    notes.append((lab, refv, it[3][int(parts[k])][1]))
                    txt += f"<super>{lab}</super>" + esc(parts[k + 1])
                vn = "" if str(it[1]) == "1" else f"<super><b>{it[1]}</b></super>"
                cur.append(vn + txt + " ")
                story.append(Mark(f"{name} {refv}"))
        end_chapter()
    doc.build(story)
    print("written", out)


if __name__ == "__main__":
    main(*sys.argv[1:3])
