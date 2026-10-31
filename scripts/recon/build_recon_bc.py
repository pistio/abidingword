#!/usr/bin/env python3
"""Reconstructions of the disclosed set B and set C layouts.

The generator scripts of sets B and C (tools/b*.py, tools/c*.py,
tools/textexports.py) were kept outside Git and are lost; their manifests
(docs/corpus/withheld-b.tsv, docs/corpus/withheld-c.tsv) describe each
document's layout. These scripts rebuild documents *in those layouts* from
the same eBible USFM with the same generator families (WeasyPrint, ReportLab,
fpdf2, python-docx, pandoc, LibreOffice, EbookLib, plain Python). They are
not the sealed files (different checksums, program versions and line
breaks): results on them are regression evidence, never an independent
measure. The two eBible-original rows (engrv-ebiblebook, engoebus-ebiblea4)
are not rebuilt; download_ebible() fetches eBible's current PDFs instead.

    python3 scripts/recon/build_recon_bc.py <usfm-folder> <out-folder> [key ...]

The USFM folder must hold the <name>_usfm.zip files named in the manifests'
reference column (https://ebible.org/Scriptures/<name>_usfm.zip).
RECON_TEST_BOOKS=GEN,PSA limits every job to those books (quick checks only).
"""

import html
import os
import re
import shutil
import subprocess
import sys
import tempfile
import textwrap
import urllib.request
import zipfile

sys.path.insert(0, os.path.dirname(__file__))
from usfm_blocks import note_parts, note_plain, note_ref, plain, read_zip  # noqa: E402

SRC, OUT = sys.argv[1], sys.argv[2]
ONLY = set(sys.argv[3:])
os.makedirs(OUT, exist_ok=True)
TEST_BOOKS = set(filter(None, os.environ.get("RECON_TEST_BOOKS", "").split(",")))

esc = html.escape
NT = ["MAT", "MRK", "LUK", "JHN", "ACT", "ROM", "1CO", "2CO", "GAL", "EPH", "PHP", "COL", "1TH", "2TH",
      "1TI", "2TI", "TIT", "PHM", "HEB", "JAS", "1PE", "2PE", "1JN", "2JN", "3JN", "JUD", "REV"]
OT = ["GEN", "EXO", "LEV", "NUM", "DEU", "JOS", "JDG", "RUT", "1SA", "2SA", "1KI", "2KI", "1CH", "2CH",
      "EZR", "NEH", "EST", "JOB", "PSA", "PRO", "ECC", "SNG", "ISA", "JER", "LAM", "EZK", "DAN", "HOS",
      "JOL", "AMO", "OBA", "JON", "MIC", "NAM", "HAB", "ZEP", "HAG", "ZEC", "MAL"]
POETRY = {"q", "q1", "q2", "q3", "q4", "qm1", "qm2"}
SERIF = "/usr/share/fonts/truetype/liberation/LiberationSerif-{}.ttf"
FREESERIF = "/usr/share/fonts/truetype/freefont/FreeSerif{}.ttf"


def load(zipname, only=None):
    """Books of one eBible USFM zip (rich pieces); `only` keeps those books
    (front matter FRT / INT only when named)."""
    if TEST_BOOKS:
        only = (set(only) & (TEST_BOOKS | {"FRT", "INT"})) if only else TEST_BOOKS
    return read_zip(os.path.join(SRC, zipname), set(only) if only else None, rich=True)


def roman(n, upper=True):
    vals = [(1000, "m"), (900, "cm"), (500, "d"), (400, "cd"), (100, "c"), (90, "xc"), (50, "l"),
            (40, "xl"), (10, "x"), (9, "ix"), (5, "v"), (4, "iv"), (1, "i")]
    out = ""
    for v, s in vals:
        while n >= v:
            out += s
            n -= v
    return out.upper() if upper else out


def hebrew_numeral(n):
    ones = ["", "א", "ב", "ג", "ד", "ה", "ו", "ז", "ח", "ט"]
    tens = ["", "י", "כ", "ל", "מ", "נ", "ס", "ע", "פ", "צ"]
    hundreds = ["", "ק", "ר", "ש", "ת"]
    out = hundreds[n // 100] if n < 500 else "ת" + hundreds[(n - 400) // 100]
    n %= 100
    if n == 15:
        return out + "טו"
    if n == 16:
        return out + "טז"
    return out + tens[n // 10] + ones[n % 10]


def letters(n):
    """1 -> a, 26 -> z, 27 -> aa ..."""
    s = ""
    while n:
        n, r = divmod(n - 1, 26)
        s = chr(97 + r) + s
    return s


def ref_with(ref, sep):
    """A note's c:v / c.v / c,v reference with the chapter separator `sep`."""
    return re.sub(r"^(\d+)[:.,]", lambda m: m.group(1) + sep, ref)


def split_note(val, chap, verse, sep=":"):
    """(is cross reference, reference, text parts) of a note piece."""
    xref = val.startswith("x:")
    body = val[2:] if xref else val
    ref, rest = note_ref(body)
    ref = ref_with(ref, sep) if ref else f"{chap}{sep}{verse}"
    return xref, ref, note_parts(rest)


def note_html(parts):
    return "".join(f"<i>{esc(t)}</i>" if it else esc(t) for it, t in parts)


# HTML for styled runs (WeasyPrint / EPUB).
HTML_STYLE = {"add": ("<i>", "</i>"), "it": ("<i>", "</i>"), "em": ("<i>", "</i>"), "tl": ("<i>", "</i>"),
              "bk": ("<i>", "</i>"), "qs": ("<i>", "</i>"), "sls": ("<i>", "</i>"),
              "bd": ("<b>", "</b>"), "bdit": ("<b><i>", "</i></b>"),
              "nd": ("<span class='sc'>", "</span>"), "sc": ("<span class='sc'>", "</span>"),
              "sup": ("<sup>", "</sup>"), "rq": ("<span class='rq'>", "</span>")}


def run_html(kind, val, styles=HTML_STYLE):
    a, b = styles.get(kind, ("", ""))
    return a + esc(val) + b


def intro_html(blocks, para_tag="p"):
    out = []
    for bl in blocks:
        if bl[0] != "intro":
            continue
        marker, pieces = bl[1], bl[2]
        text = "".join(run_html(k, v) for k, v in pieces if k not in ("v", "note")).strip()
        if marker == "ib" or not text:
            out.append("<div class='ib'></div>")
        elif marker.startswith(("is", "iot", "imt")):
            out.append(f"<h3 class='is'>{text}</h3>")
        else:
            out.append(f"<{para_tag} class='{marker}'>{text}</{para_tag}>")
    return "\n".join(out)


def weasy(html_text, out):
    from weasyprint import HTML
    HTML(string=html_text).write_pdf(out)


# =================================================================== B01
def engbsb_wpdf2col():
    """BSB, US Letter, two CSS columns with a rule; running head "Book c:v –
    Book c:v" top centre and folio top outer, none on book openings; drop
    chapter numerals replacing verse 1 (Psalms: small-caps "Psalm N");
    superscript bold verse numbers; page-foot footnotes lettered per chapter
    opening with a bold c:v."""
    books = load("engbsb_usfm.zip")
    css = """
    @page { size: letter; margin: 0.85in 0.65in 0.75in 0.65in;
      @top-center { content: string(vr, first) " – " string(vr, last); font: 8.5pt 'Liberation Serif'; }
      @footnote { border-top: 0.4pt solid #000; padding-top: 3pt; margin-top: 6pt; } }
    @page :left { @top-left { content: counter(page); font: 8.5pt 'Liberation Serif'; } }
    @page :right { @top-right { content: counter(page); font: 8.5pt 'Liberation Serif'; } }
    @page :first { @top-center { content: none } @top-left { content: none } @top-right { content: none } }
    @page :nth(1 of book) { @top-center { content: none } }
    body { font: 9.5pt/1.3 'Liberation Serif'; text-align: justify; hyphens: auto; }
    .titlepage { text-align: center; margin-top: 3in; font-size: 34pt; letter-spacing: 0.06em; }
    .titlepage small { display: block; font-size: 13pt; margin-top: 0.3in; letter-spacing: 0; }
    section.book { page: book; break-before: page; }
    h1 { text-align: center; font-size: 22pt; letter-spacing: 0.08em; margin: 0.3in 0 0.25in; font-weight: normal; }
    .cols { column-count: 2; column-gap: 0.28in; column-rule: 0.5pt solid #555; }
    .ch { counter-reset: footnote; }
    .dc { float: left; font-size: 27pt; line-height: 0.9; font-weight: bold; margin: 1pt 4pt 0 0; string-set: vr attr(data-r); }
    sup.v { font-weight: bold; font-size: 6pt; vertical-align: super; line-height: 0; string-set: vr attr(data-r); }
    p { margin: 0 0 2pt; text-indent: 0.15in; }
    p.m, p.pmo, p.nb, p.mi { text-indent: 0; }
    p.q1, p.q, p.q3 { margin: 0 0 0 1.4em; text-indent: -1em; }
    p.q2 { margin: 0 0 0 2.4em; text-indent: -1em; }
    p.li1 { margin-left: 1.2em; text-indent: -0.8em; } p.li2 { margin-left: 2.2em; text-indent: -0.8em; }
    p.qr { text-align: right; } p.pc { text-align: center; text-indent: 0; }
    p.b { height: 4pt; margin: 0; }
    h4.s { font-size: 9.5pt; font-weight: bold; margin: 6pt 0 1pt; text-align: left; break-after: avoid; }
    h4.s2 { font-style: italic; }
    p.r { font-style: italic; text-indent: 0; margin: 0 0 3pt; break-before: avoid; }
    h3.ms { text-align: center; font-size: 11pt; margin: 8pt 0 0; letter-spacing: 0.05em; }
    p.mr { text-align: center; font-style: italic; text-indent: 0; margin-bottom: 4pt; }
    h3.ps { text-align: center; font-variant: small-caps; font-size: 10.5pt; font-weight: normal; margin: 7pt 0 2pt; }
    p.d { font-style: italic; text-indent: 0; } p.qa { font-weight: bold; text-indent: 0; margin-top: 3pt; }
    .sc { font-variant: small-caps; }
    .fn { float: footnote; font-size: 7.5pt; line-height: 1.2; text-align: left; text-indent: 0; font-style: normal; font-weight: normal; }
    .fn::footnote-call { content: counter(footnote, lower-alpha); font-size: 6pt; vertical-align: super; line-height: 0; font-style: italic; }
    .fn::footnote-marker { content: counter(footnote, lower-alpha) " "; font-style: italic; }
    """
    parts = [f"<html lang='en'><head><meta charset='utf-8'><title>The Holy Bible</title><style>{css}</style></head><body>",
             "<div class='titlepage'>THE HOLY BIBLE<small>Berean Standard Bible</small></div>"]
    for b in books:
        name = b["h"] or b["title"]
        title = (b["mt"][0] if b["mt"] else name).upper()
        psalm = b["code"] == "PSA"
        parts.append(f"<section class='book'><h1>{esc(title)}</h1><div class='cols'>")
        chap, verse, pending, open_ch = 0, "1", None, False
        for bl in b["blocks"]:
            if bl[0] == "chapter":
                if open_ch:
                    parts.append("</div>")
                parts.append("<div class='ch'>")
                open_ch = True
                chap, verse = bl[1], "1"
                if psalm:
                    parts.append(f"<h3 class='ps'>Psalm {chap}</h3>")
                else:
                    pending = chap
            elif bl[0] == "heading":
                m, t = bl[1], esc(bl[2])
                if m in ("s", "s1", "s2", "s3"):
                    parts.append(f"<h4 class='s {m}'>{t}</h4>")
                elif m == "r":
                    parts.append(f"<p class='r'>{t}</p>")
                elif m.startswith("ms"):
                    parts.append(f"<h3 class='ms'>{t}</h3>")
                elif m == "mr":
                    parts.append(f"<p class='mr'>{t}</p>")
                elif m == "d":
                    parts.append(f"<p class='d'>{t}</p>")
                elif m == "qa":
                    parts.append(f"<p class='qa'>{t}</p>")
                elif m == "sp":
                    parts.append(f"<p class='qa'>{t}</p>")
            elif bl[0] == "para":
                out = []
                if pending is not None:
                    out.append(f"<span class='dc' data-r='{esc(name)} {pending}:1'>{pending}</span>")
                for kind, val in bl[2]:
                    if kind == "v":
                        verse = val
                        if pending is not None and val == "1":
                            pending = None
                            continue
                        out.append(f"<sup class='v' data-r='{esc(name)} {chap}:{esc(val)}'>{esc(val)}</sup>")
                    elif kind == "note":
                        xref, ref, nparts = split_note(val, chap, verse)
                        if xref:
                            continue
                        out.append(f"<span class='fn'><b>{esc(ref)}</b> {note_html(nparts)}</span>")
                    else:
                        out.append(run_html(kind, val))
                pending = None
                parts.append(f"<p class='{bl[1]}'>{''.join(out).strip()}</p>")
        if open_ch:
            parts.append("</div>")
        parts.append("</div></section>")
    parts.append("</body></html>")
    weasy("\n".join(parts), os.path.join(OUT, "engbsb-rwpdf2col.pdf"))


# =================================================================== ReportLab helpers
def reportlab_fonts():
    from reportlab.lib.fonts import addMapping
    from reportlab.pdfbase import pdfmetrics
    from reportlab.pdfbase.ttfonts import TTFont
    for name, f in (("Serif", "Regular"), ("Serif-Bold", "Bold"), ("Serif-Italic", "Italic"), ("Serif-BoldItalic", "BoldItalic")):
        pdfmetrics.registerFont(TTFont(name, SERIF.format(f)))
    addMapping("Serif", 0, 0, "Serif")
    addMapping("Serif", 1, 0, "Serif-Bold")
    addMapping("Serif", 0, 1, "Serif-Italic")
    addMapping("Serif", 1, 1, "Serif-BoldItalic")


def xesc(text):
    return text.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


def small_caps(text, size):
    """Emulated small capitals: lower-case letters as reduced capitals."""
    out = []
    for run in re.findall(r"[^\W\d_]+|[\W\d_]+", text):
        for chunk in re.findall(r"[a-zß-ÿ]+|[^a-zß-ÿ]+", run):
            if chunk.islower():
                out.append(f"<font size='{size * 0.78:.1f}'>{xesc(chunk.upper())}</font>")
            else:
                out.append(xesc(chunk))
    return "".join(out)


def rl_run(kind, val, size):
    if kind in ("nd", "sc"):
        return small_caps(val, size)
    if kind in ("add", "it", "em", "tl", "bk", "qs", "sls"):
        return f"<i>{xesc(val)}</i>"
    if kind == "bd":
        return f"<b>{xesc(val)}</b>"
    if kind == "bdit":
        return f"<b><i>{xesc(val)}</i></b>"
    if kind == "sup":
        return f"<super>{xesc(val)}</super>"
    return xesc(val)


def head_canvas(draw):
    """A canvas class that collects <onDraw name="verse" label=...> labels
    of a page and calls draw(canvas, labels, page) before the page is shown."""
    from reportlab.pdfgen.canvas import Canvas

    class HeadCanvas(Canvas):
        def __init__(self, *a, **k):
            super().__init__(*a, **k)
            self.labels = []
            if hasattr(self, "_namedCB"):
                self._namedCB["verse"] = self.verse

        def verse(self, canv, kind, label):
            self.labels.append(label)

        def showPage(self):
            draw(self, self.labels, self.getPageNumber())
            self.labels = []
            super().showPage()

    return HeadCanvas


# =================================================================== B02
def engasv_rlab1col():
    """ASV, 6x9 in one column justified (ReportLab): title page and PREFACE,
    stacked book titles, CHAPTER N / PSALM N, bold verse numbers + nbsp,
    running head "GENESIS 18:30–19:16" with a rule and the folio outer;
    centred folio at the foot of pages without verses; notes after each
    chapter with asterisk callers."""
    from reportlab.lib.enums import TA_CENTER, TA_JUSTIFY, TA_LEFT
    from reportlab.lib.styles import ParagraphStyle
    from reportlab.lib.units import inch
    from reportlab.platypus import Frame, PageBreak, PageTemplate, Paragraph, Spacer
    from reportlab.platypus.doctemplate import BaseDocTemplate
    reportlab_fonts()
    W, H = 6 * inch, 9 * inch
    books = load("eng-asv_usfm.zip", {"FRT", "INT"} | set(OT) | set(NT))

    def draw(c, labels, page):
        c.saveState()
        c.setFont("Serif", 8.5)
        if labels:
            first, last = labels[0], labels[-1]
            bk, fcv = first.rsplit(" ", 1)
            lcv = last.rsplit(" ", 1)[1]
            c.drawCentredString(W / 2, H - 0.5 * inch, f"{bk} {fcv}–{lcv}")
            if page % 2 == 0:
                c.drawString(0.6 * inch, H - 0.5 * inch, str(page))
            else:
                c.drawRightString(W - 0.6 * inch, H - 0.5 * inch, str(page))
            c.setLineWidth(0.4)
            c.line(0.6 * inch, H - 0.55 * inch, W - 0.6 * inch, H - 0.55 * inch)
        elif page > 1:
            c.drawCentredString(W / 2, 0.4 * inch, str(page))
        c.restoreState()

    st = {
        "body": ParagraphStyle("body", fontName="Serif", fontSize=9.5, leading=12, alignment=TA_JUSTIFY, firstLineIndent=12),
        "m": ParagraphStyle("m", fontName="Serif", fontSize=9.5, leading=12, alignment=TA_JUSTIFY),
        "q1": ParagraphStyle("q1", fontName="Serif", fontSize=9.5, leading=12, alignment=TA_LEFT, leftIndent=24, firstLineIndent=-12),
        "q2": ParagraphStyle("q2", fontName="Serif", fontSize=9.5, leading=12, alignment=TA_LEFT, leftIndent=36, firstLineIndent=-12),
        "qc": ParagraphStyle("qc", fontName="Serif", fontSize=9.5, leading=12, alignment=TA_CENTER),
        "d": ParagraphStyle("d", fontName="Serif-Italic", fontSize=9, leading=11.5, alignment=TA_LEFT, spaceAfter=2),
        "chap": ParagraphStyle("chap", fontName="Serif", fontSize=10, leading=13, alignment=TA_CENTER, spaceBefore=8, spaceAfter=4),
        "ms": ParagraphStyle("ms", fontName="Serif-Bold", fontSize=10, leading=13, alignment=TA_CENTER, spaceBefore=8, spaceAfter=2),
        "note": ParagraphStyle("note", fontName="Serif", fontSize=7.5, leading=9, alignment=TA_JUSTIFY, spaceBefore=1),
        "is": ParagraphStyle("is", fontName="Serif-Italic", fontSize=10, leading=13, alignment=TA_CENTER, spaceBefore=8, spaceAfter=3),
        "ip": ParagraphStyle("ip", fontName="Serif", fontSize=9.5, leading=12, alignment=TA_JUSTIFY, firstLineIndent=12, spaceAfter=2),
        "front": ParagraphStyle("front", fontName="Serif", fontSize=9, leading=11.5, alignment=TA_CENTER, spaceAfter=1),
    }
    mt_size = {"mt": 22, "mt1": 22, "mt2": 13, "mt3": 11, "mt4": 9}
    story = []

    def title_lines(mts, scale=1.0):
        for m, t in mts:
            size = mt_size.get(m, 11) * scale
            story.append(Paragraph(xesc(t), ParagraphStyle("t", fontName="Serif", fontSize=size, leading=size * 1.3, alignment=TA_CENTER, spaceAfter=size * 0.25)))

    for b in books:
        if b["code"] == "FRT":
            story.append(Spacer(1, 1.2 * inch))
            title_lines(b["mts"])
            story.append(PageBreak())
            for bl in b["blocks"]:
                if bl[0] == "intro":
                    if bl[1] == "ib" or not bl[2]:
                        story.append(Spacer(1, 10))
                    else:
                        story.append(Paragraph("".join(rl_run(k, v, 9) for k, v in bl[2] if k not in ("v", "note")), st["front"]))
            story.append(PageBreak())
            continue
        if b["code"] == "INT":
            title_lines(b["mts"], 0.8)
            story.append(Spacer(1, 8))
            for bl in b["blocks"]:
                if bl[0] == "intro":
                    text = "".join(rl_run(k, v, 9.5) for k, v in bl[2] if k not in ("v", "note")).strip()
                    if bl[1].startswith("is"):
                        story.append(Paragraph(text, st["is"]))
                    elif bl[1] == "ib" or not text:
                        story.append(Spacer(1, 6))
                    else:
                        story.append(Paragraph(text, st["ip"]))
            story.append(PageBreak())
            continue
        name = (b["h"] or b["title"]).upper()
        psalm = b["code"] == "PSA"
        story.append(Spacer(1, 0.3 * inch))
        title_lines(b["mts"], 0.75)
        story.append(Spacer(1, 6))
        chap, verse, notes = 0, "1", []

        def flush_notes():
            for i, (ref, parts) in enumerate(notes):
                star = "*" * (i + 1)
                body = "".join(f"<i>{xesc(t)}</i>" if it else xesc(t) for it, t in parts)
                story.append(Paragraph(f"{star}{xesc(ref)} {body}", st["note"]))
            notes.clear()

        for bl in b["blocks"]:
            if bl[0] == "chapter":
                flush_notes()
                chap, verse = bl[1], "1"
                story.append(Paragraph(f"{'PSALM' if psalm else 'CHAPTER'} {chap}", st["chap"]))
            elif bl[0] == "heading":
                if bl[1] == "d":
                    story.append(Paragraph(xesc(bl[2]), st["d"]))
                elif bl[1].startswith("ms"):
                    story.append(Paragraph(xesc(bl[2]), st["ms"]))
            elif bl[0] == "para":
                m = bl[1]
                if m == "b":
                    story.append(Spacer(1, 5))
                    continue
                out = []
                for kind, val in bl[2]:
                    if kind == "v":
                        verse = val
                        out.append(f"<onDraw name='verse' label='{name} {chap}:{xesc(val)}'/><b>{xesc(val)}</b>&nbsp;")
                    elif kind == "note":
                        xref, ref, parts = split_note(val, chap, verse)
                        notes.append((ref, parts))
                        out.append(f"<super>{'*' * len(notes)}</super>")
                    else:
                        out.append(rl_run(kind, val, 9.5))
                text = "".join(out).strip()
                if not text:
                    continue
                style = st["q1"] if m in ("q", "q1", "q3") else st["q2"] if m == "q2" else st["qc"] if m == "qc" \
                    else st["m"] if m in ("m", "nb", "mi") else st["body"]
                story.append(Paragraph(text, style))
        flush_notes()
        story.append(PageBreak())
    path = os.path.join(OUT, "engasv-rrlab1col.pdf")
    doc = BaseDocTemplate(path, pagesize=(W, H), leftMargin=0.6 * inch, rightMargin=0.6 * inch,
                          topMargin=0.75 * inch, bottomMargin=0.65 * inch, title="American Standard Version")
    frame = Frame(doc.leftMargin, doc.bottomMargin, doc.width, doc.height, id="f", leftPadding=0, rightPadding=0)
    doc.addPageTemplates([PageTemplate(id="p", frames=[frame])])
    doc.build(story, canvasmaker=head_canvas(draw))


# =================================================================== Markdown helpers
def md_esc(text):
    return re.sub(r"([\\`*_\[\]{}<>#|^~$@])", r"\\\1", text)


MD_STYLE = {"add": ("*", "*"), "it": ("*", "*"), "em": ("*", "*"), "tl": ("*", "*"), "bk": ("*", "*"),
            "qs": ("*", "*"), "sls": ("*", "*"), "bd": ("**", "**"), "bdit": ("***", "***"),
            "nd": ("[", "]{.smallcaps}"), "sc": ("[", "]{.smallcaps}"), "sup": ("^", "^")}


def md_run(kind, val):
    a, b = MD_STYLE.get(kind, ("", ""))
    if not a:
        return md_esc(val)
    lead = val[:len(val) - len(val.lstrip())]
    trail = val[len(val.rstrip()):]
    core = val.strip()
    if not core:
        return val
    if kind == "sup":
        core = core.replace(" ", "\\ ")
    return lead + a + md_esc(core) + b + trail


def pandoc(src, out, *args):
    subprocess.run(["pandoc", "-f", "markdown-smart", src, "-o", out, *args], check=True, timeout=7200)


# =================================================================== B03
GERMAN_TITLES = {
    "GEN": "Das erste Buch Mose", "EXO": "Das zweite Buch Mose", "LEV": "Das dritte Buch Mose",
    "NUM": "Das vierte Buch Mose", "DEU": "Das fünfte Buch Mose", "JOS": "Das Buch Josua",
    "JDG": "Das Buch der Richter", "RUT": "Das Buch Ruth", "1SA": "Das erste Buch Samuel",
    "2SA": "Das zweite Buch Samuel", "1KI": "Das erste Buch der Könige", "2KI": "Das zweite Buch der Könige",
    "1CH": "Das erste Buch der Chronika", "2CH": "Das zweite Buch der Chronika", "EZR": "Das Buch Esra",
    "NEH": "Das Buch Nehemia", "EST": "Das Buch Esther", "JOB": "Das Buch Hiob", "PSA": "Die Psalmen",
    "PRO": "Die Sprüche Salomos", "ECC": "Der Prediger Salomo", "SNG": "Das Hohelied Salomos",
    "ISA": "Der Prophet Jesaja", "JER": "Der Prophet Jeremia", "LAM": "Die Klagelieder Jeremias",
    "EZK": "Der Prophet Hesekiel", "DAN": "Der Prophet Daniel", "HOS": "Der Prophet Hosea",
    "JOL": "Der Prophet Joel", "AMO": "Der Prophet Amos", "OBA": "Der Prophet Obadja",
    "JON": "Der Prophet Jona", "MIC": "Der Prophet Micha", "NAM": "Der Prophet Nahum",
    "HAB": "Der Prophet Habakuk", "ZEP": "Der Prophet Zephanja", "HAG": "Der Prophet Haggai",
    "ZEC": "Der Prophet Sacharja", "MAL": "Der Prophet Maleachi",
    "MAT": "Das Evangelium nach Matthäus", "MRK": "Das Evangelium nach Markus",
    "LUK": "Das Evangelium nach Lukas", "JHN": "Das Evangelium nach Johannes",
    "ACT": "Die Apostelgeschichte", "ROM": "Der Brief an die Römer",
    "1CO": "Der erste Brief an die Korinther", "2CO": "Der zweite Brief an die Korinther",
    "GAL": "Der Brief an die Galater", "EPH": "Der Brief an die Epheser", "PHP": "Der Brief an die Philipper",
    "COL": "Der Brief an die Kolosser", "1TH": "Der erste Brief an die Thessalonicher",
    "2TH": "Der zweite Brief an die Thessalonicher", "1TI": "Der erste Brief an Timotheus",
    "2TI": "Der zweite Brief an Timotheus", "TIT": "Der Brief an Titus", "PHM": "Der Brief an Philemon",
    "HEB": "Der Brief an die Hebräer", "JAS": "Der Brief des Jakobus", "1PE": "Der erste Brief des Petrus",
    "2PE": "Der zweite Brief des Petrus", "1JN": "Der erste Brief des Johannes",
    "2JN": "Der zweite Brief des Johannes", "3JN": "Der dritte Brief des Johannes",
    "JUD": "Der Brief des Judas", "REV": "Die Offenbarung",
}


def deuelbbk_pandocdocx():
    """Elberfelder (bibelkommentare.de), pandoc Markdown -> docx: Title +
    subtitle, Heading 1 = traditional German book titles, Heading 2 =
    Kapitel N / Psalm N, superscript verse numbers, real Word footnotes
    numbered through the document, LORD (\\nd) in small capitals."""
    books = load("deuelbbk_usfm.zip")
    md = ["---", "title: Die Heilige Schrift", "subtitle: Elberfelder Übersetzung (Version von bibelkommentare.de)",
          "lang: de-DE", "---", "",
          md_esc("Copyright © 2019 Verbreitung des christlichen Glaubens e.V. – Creative Commons "
                 "Namensnennung – Nicht kommerziell – Keine Bearbeitungen 4.0 (CC BY-NC-ND 4.0). "
                 "Quelle: eBible.org (deuelbbk). Nur das Satzbild wurde neu gestaltet; der Wortlaut ist unverändert."), ""]
    for b in books:
        md += [f"# {md_esc(GERMAN_TITLES.get(b['code'], b['toc1'] or b['title']))}", ""]
        psalm = b["code"] == "PSA"
        chap, verse = 0, "1"
        for bl in b["blocks"]:
            if bl[0] == "chapter":
                chap, verse = bl[1], "1"
                md += [f"## {'Psalm' if psalm else 'Kapitel'} {chap}", ""]
            elif bl[0] == "para":
                out = []
                for kind, val in bl[2]:
                    if kind == "v":
                        verse = val
                        out.append(f"^{md_esc(val)}^ ")
                    elif kind == "note":
                        xref, ref, parts = split_note(val, chap, verse)
                        body = "".join(md_run("it" if it else "t", t) for it, t in parts)
                        out.append(f"^[{md_esc(ref)} {body.strip()}]")
                    else:
                        out.append(md_run(kind, val))
                text = re.sub(r"[ \t]+", " ", "".join(out)).strip()
                if text:
                    md += [text, ""]
    work = tempfile.mkdtemp()
    src = os.path.join(work, "deuelbbk.md")
    with open(src, "w", encoding="utf-8") as f:
        f.write("\n".join(md))
    pandoc(src, os.path.join(OUT, "deuelbbk-rpandocdocx.docx"))


# =================================================================== B04
def francl_ebooklibepub():
    """Néo-Crampon Libre, EPUB 3 via EbookLib: one XHTML per book + nav; h1
    \\toc1; floated drop numerals replacing verse 1 (Psalms: h4 from \\cl);
    superscript verse numbers; h2 \\ms, h3 \\s by level, italic \\r; EPUB 3
    pop-up notes (* footnotes, † cross references) as asides after each
    chapter."""
    from ebooklib import epub
    books = load("francl_usfm.zip")
    bk = epub.EpubBook()
    bk.set_identifier("recon-francl-ebooklib")
    bk.set_title("Sainte Bible néo-Crampon Libre")
    bk.set_language("fr")
    bk.add_author("Fraternité de Tibériade")
    css = epub.EpubItem(uid="css", file_name="style/bible.css", media_type="text/css", content=(
        "body{font-family:serif} h1{text-align:center} h2{text-align:center;font-size:1.1em}"
        " h3.s1{font-weight:bold;font-size:1em} h3.s2{font-style:italic;font-weight:normal;font-size:1em}"
        " h3.s3{font-style:italic;font-weight:normal;font-size:0.9em} h4{text-align:center;font-variant:small-caps}"
        " p.r{font-style:italic;font-size:0.9em} p.d{font-style:italic} p.qa{font-weight:bold}"
        " span.dropnum{float:left;font-size:2.6em;line-height:0.9;font-weight:bold;margin-right:0.12em}"
        " sup.v{font-size:0.65em} p.q1{margin-left:1.5em;text-indent:-1em} p.q2{margin-left:2.5em;text-indent:-1em}"
        " .sc{font-variant:small-caps} aside{font-size:0.85em} a.noteref{text-decoration:none}").encode())
    bk.add_item(css)
    items = []
    n = 0
    for i, b in enumerate(books):
        title = b["toc1"] or b["title"]
        psalm = b["code"] == "PSA"
        body = [f"<h1>{esc(title)}</h1>"]
        notes = []
        chap, verse, pending = 0, "1", None

        def flush():
            for nid, mark, text in notes:
                body.append(f"<aside epub:type='footnote' id='{nid}'><p>{mark} {text}</p></aside>")
            notes.clear()

        for bl in b["blocks"]:
            if bl[0] == "chapter":
                flush()
                chap, verse = bl[1], "1"
                pending = None if psalm else chap
            elif bl[0] == "heading":
                m, t = bl[1], esc(bl[2])
                if m == "cl" and psalm:
                    body.append(f"<h4>{t}</h4>")
                elif m.startswith("ms"):
                    body.append(f"<h2>{t}</h2>")
                elif m in ("s", "s1", "s2", "s3"):
                    lvl = "s1" if m == "s" else m
                    body.append(f"<h3 class='{lvl}'>{t}</h3>")
                elif m == "r":
                    body.append(f"<p class='r'>{t}</p>")
                elif m == "d":
                    body.append(f"<p class='d'>{t}</p>")
                elif m in ("qa", "sp"):
                    body.append(f"<p class='qa'>{t}</p>")
            elif bl[0] == "para":
                out = []
                if pending is not None:
                    out.append(f"<span class='dropnum'>{pending}</span>")
                for kind, val in bl[2]:
                    if kind == "v":
                        verse = val
                        if pending is not None and val == "1":
                            pending = None
                            continue
                        out.append(f"<sup class='v'>{esc(val)}</sup>")
                    elif kind == "note":
                        n += 1
                        xref = val.startswith("x:")
                        mark = "†" if xref else "*"
                        text = note_html(note_parts(val[2:] if xref else val))
                        nid = f"n{n}"
                        out.append(f"<a class='noteref' epub:type='noteref' href='#{nid}'>{mark}</a>")
                        notes.append((nid, mark, text))
                    else:
                        out.append(run_html(kind, val))
                pending = None
                text = "".join(out).strip()
                if text:
                    body.append(f"<p class='{bl[1]}'>{text}</p>")
        flush()
        item = epub.EpubHtml(title=title, file_name=f"book{i + 1:02d}.xhtml", lang="fr")
        item.content = "<html><head></head><body>" + "\n".join(body) + "</body></html>"
        item.add_item(css)
        bk.add_item(item)
        items.append(item)
    bk.toc = items
    bk.add_item(epub.EpubNcx())
    bk.add_item(epub.EpubNav())
    bk.spine = ["nav"] + items
    epub.write_epub(os.path.join(OUT, "francl-rebooklibepub.epub"), bk)


# =================================================================== LibreOffice
def soffice(src, fmt, outdir, filt=None):
    target = f"{fmt}:{filt}" if filt else fmt
    subprocess.run(["soffice", "--headless", "--convert-to", target, "--outdir", outdir, src],
                   check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=7200)


# =================================================================== B05
def spavbl_lofodt():
    """Versión Biblia Libre, A5, flat ODT exported by LibreOffice: front
    matter; Heading 1 = \\mt1 on a new page; running head = the book name by
    chapter field, small caps with a rule; folio centred at the foot;
    Heading 2 Capítulo N / Salmo N; bold superscript verse numbers; real
    page-foot footnotes lettered a, b, c per page, each opening with a bold
    c:v."""
    books = load("spavbl_usfm.zip", {"FRT"} | set(OT) | set(NT))
    x = lambda t: esc(t, quote=False)  # noqa: E731

    def runs(pieces):
        out = []
        for kind, val in pieces:
            if kind in ("v", "note"):
                continue
            style = {"it": "Italic", "add": "Italic", "em": "Italic", "bd": "Bold", "nd": "SmallCaps", "sc": "SmallCaps"}.get(kind)
            out.append(f'<text:span text:style-name="{style}">{x(val)}</text:span>' if style else x(val))
        return "".join(out)

    body = []
    nid = 0
    for b in books:
        if b["code"] == "FRT":
            body.append(f'<text:p text:style-name="Title">{x(b["mt"][0] if b["mt"] else b["title"])}</text:p>')
            for bl in b["blocks"]:
                if bl[0] != "intro":
                    continue
                if bl[1] == "ib" or not bl[2]:
                    body.append('<text:p text:style-name="Front"/>')
                elif bl[1].startswith(("is", "iot")):
                    body.append(f'<text:p text:style-name="FrontHead">{runs(bl[2])}</text:p>')
                else:
                    body.append(f'<text:p text:style-name="Front">{runs(bl[2])}</text:p>')
            continue
        name = b["mt"][0] if b["mt"] else b["title"]
        psalm = b["code"] == "PSA"
        body.append(f'<text:h text:style-name="Heading_20_1" text:outline-level="1">{x(name)}</text:h>')
        chap, verse = 0, "1"
        for bl in b["blocks"]:
            if bl[0] == "chapter":
                chap, verse = bl[1], "1"
                body.append(f'<text:h text:style-name="Heading_20_2" text:outline-level="2">{"Salmo" if psalm else "Capítulo"} {chap}</text:h>')
            elif bl[0] == "heading":
                if bl[1] == "d":
                    body.append(f'<text:p text:style-name="Superscription">{x(bl[2])}</text:p>')
                elif bl[1] in ("qa", "sp"):
                    body.append(f'<text:p text:style-name="Acrostic">{x(bl[2])}</text:p>')
                elif bl[1] in ("s", "s1", "s2"):
                    body.append(f'<text:p text:style-name="Section">{x(bl[2])}</text:p>')
            elif bl[0] == "para":
                style = {"q1": "Poetry_1", "q": "Poetry_1", "q2": "Poetry_2", "q3": "Poetry_2"}.get(bl[1], "Bible_Text")
                out = []
                for kind, val in bl[2]:
                    if kind == "v":
                        verse = val
                        out.append(f'<text:span text:style-name="Verse">{x(val)}</text:span>')
                    elif kind == "note":
                        nid += 1
                        xref, ref, parts = split_note(val, chap, verse)
                        text = "".join(f'<text:span text:style-name="Italic">{x(t)}</text:span>' if it else x(t) for it, t in parts)
                        out.append(f'<text:note text:id="ftn{nid}" text:note-class="footnote"><text:note-citation>{letters(1)}</text:note-citation>'
                                   f'<text:note-body><text:p text:style-name="Footnote"><text:span text:style-name="Bold">{x(ref)}</text:span> {text}</text:p></text:note-body></text:note>')
                    else:
                        out.append(runs([(kind, val)]))
                text = "".join(out).strip()
                if text:
                    body.append(f'<text:p text:style-name="{style}">{text}</text:p>')
    fodt = f"""<?xml version="1.0" encoding="UTF-8"?>
<office:document xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0" xmlns:svg="urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:meta="urn:oasis:names:tc:opendocument:xmlns:meta:1.0" office:version="1.3" office:mimetype="application/vnd.oasis.opendocument.text">
<office:meta><dc:title>Versión Biblia Libre</dc:title><dc:language>es</dc:language></office:meta>
<office:font-face-decls><style:font-face style:name="Liberation Serif" svg:font-family="'Liberation Serif'" style:font-family-generic="roman"/></office:font-face-decls>
<office:styles>
 <style:default-style style:family="paragraph"><style:paragraph-properties fo:hyphenation-ladder-count="no-limit"/><style:text-properties style:font-name="Liberation Serif" fo:font-size="10pt" fo:language="es" fo:country="ES" fo:hyphenate="true"/></style:default-style>
 <style:style style:name="Standard" style:family="paragraph"/>
 <style:style style:name="Bible_Text" style:display-name="Bible Text" style:family="paragraph" style:parent-style-name="Standard"><style:paragraph-properties fo:text-align="justify" fo:text-indent="0.4cm" fo:margin-bottom="0.05cm"/></style:style>
 <style:style style:name="Poetry_1" style:display-name="Poetry 1" style:family="paragraph" style:parent-style-name="Standard"><style:paragraph-properties fo:margin-left="0.8cm" fo:text-indent="-0.5cm"/></style:style>
 <style:style style:name="Poetry_2" style:display-name="Poetry 2" style:family="paragraph" style:parent-style-name="Standard"><style:paragraph-properties fo:margin-left="1.3cm" fo:text-indent="-0.5cm"/></style:style>
 <style:style style:name="Superscription" style:family="paragraph" style:parent-style-name="Standard"><style:paragraph-properties fo:margin-bottom="0.1cm"/><style:text-properties fo:font-style="italic"/></style:style>
 <style:style style:name="Acrostic" style:family="paragraph" style:parent-style-name="Standard"><style:paragraph-properties fo:margin-top="0.15cm"/><style:text-properties fo:font-weight="bold"/></style:style>
 <style:style style:name="Section" style:family="paragraph" style:parent-style-name="Standard"><style:paragraph-properties fo:margin-top="0.2cm" fo:keep-with-next="always"/><style:text-properties fo:font-weight="bold" fo:font-style="italic"/></style:style>
 <style:style style:name="Front" style:family="paragraph" style:parent-style-name="Standard"><style:paragraph-properties fo:text-align="justify" fo:margin-bottom="0.1cm"/></style:style>
 <style:style style:name="FrontHead" style:family="paragraph" style:parent-style-name="Standard"><style:paragraph-properties fo:margin-top="0.3cm" fo:margin-bottom="0.15cm"/><style:text-properties fo:font-weight="bold" fo:font-size="11pt"/></style:style>
 <style:style style:name="Title" style:family="paragraph" style:parent-style-name="Standard"><style:paragraph-properties fo:text-align="center" fo:margin-top="3cm" fo:margin-bottom="1cm"/><style:text-properties fo:font-size="22pt" fo:font-weight="bold"/></style:style>
 <style:style style:name="Heading" style:family="paragraph" style:parent-style-name="Standard"><style:paragraph-properties fo:keep-with-next="always"/></style:style>
 <style:style style:name="Heading_20_1" style:display-name="Heading 1" style:family="paragraph" style:parent-style-name="Heading" style:default-outline-level="1"><style:paragraph-properties fo:text-align="center" fo:break-before="page" fo:margin-top="0.6cm" fo:margin-bottom="0.5cm"/><style:text-properties fo:font-size="18pt" fo:font-weight="bold"/></style:style>
 <style:style style:name="Heading_20_2" style:display-name="Heading 2" style:family="paragraph" style:parent-style-name="Heading" style:default-outline-level="2"><style:paragraph-properties fo:margin-top="0.3cm" fo:margin-bottom="0.1cm"/><style:text-properties fo:font-size="11pt" fo:font-weight="bold"/></style:style>
 <style:style style:name="Header" style:family="paragraph" style:parent-style-name="Standard"><style:paragraph-properties fo:text-align="center" fo:border-bottom="0.5pt solid #000000" fo:padding-bottom="0.05cm"/><style:text-properties fo:font-variant="small-caps" fo:font-size="9pt"/></style:style>
 <style:style style:name="Footer" style:family="paragraph" style:parent-style-name="Standard"><style:paragraph-properties fo:text-align="center"/><style:text-properties fo:font-size="9pt"/></style:style>
 <style:style style:name="Footnote" style:family="paragraph" style:parent-style-name="Standard"><style:paragraph-properties fo:margin-left="0.4cm" fo:text-indent="-0.4cm"/><style:text-properties fo:font-size="8pt"/></style:style>
 <style:style style:name="Verse" style:family="text"><style:text-properties fo:font-weight="bold" style:text-position="super 58%"/></style:style>
 <style:style style:name="Bold" style:family="text"><style:text-properties fo:font-weight="bold"/></style:style>
 <style:style style:name="Italic" style:family="text"><style:text-properties fo:font-style="italic"/></style:style>
 <style:style style:name="SmallCaps" style:family="text"><style:text-properties fo:font-variant="small-caps"/></style:style>
 <style:style style:name="Footnote_20_anchor" style:display-name="Footnote anchor" style:family="text"><style:text-properties style:text-position="super 58%"/></style:style>
 <style:style style:name="Footnote_20_Symbol" style:display-name="Footnote Symbol" style:family="text"/>
 <text:outline-style style:name="Outline"><text:outline-level-style text:level="1" style:num-format=""/><text:outline-level-style text:level="2" style:num-format=""/></text:outline-style>
 <text:notes-configuration text:note-class="footnote" text:citation-style-name="Footnote_20_Symbol" text:citation-body-style-name="Footnote_20_anchor" style:num-format="a" text:start-value="0" text:footnotes-position="page" text:start-numbering-at="page"/>
</office:styles>
<office:automatic-styles>
 <style:page-layout style:name="pm1"><style:page-layout-properties fo:page-width="14.8cm" fo:page-height="21cm" fo:margin-top="1.2cm" fo:margin-bottom="1.2cm" fo:margin-left="1.6cm" fo:margin-right="1.6cm"/>
  <style:header-style><style:header-footer-properties fo:min-height="0.6cm" fo:margin-bottom="0.3cm"/></style:header-style>
  <style:footer-style><style:header-footer-properties fo:min-height="0.5cm" fo:margin-top="0.3cm"/></style:footer-style></style:page-layout>
</office:automatic-styles>
<office:master-styles><style:master-page style:name="Standard" style:page-layout-name="pm1">
 <style:header><text:p text:style-name="Header"><text:chapter text:display="name" text:outline-level="1"/></text:p></style:header>
 <style:footer><text:p text:style-name="Footer"><text:page-number text:select-page="current"/></text:p></style:footer>
</style:master-page></office:master-styles>
<office:body><office:text>
{chr(10).join(body)}
</office:text></office:body></office:document>
"""
    work = tempfile.mkdtemp()
    src = os.path.join(work, "spavbl-rlofodt.fodt")
    with open(src, "w", encoding="utf-8") as f:
        f.write(fodt)
    soffice(src, "pdf", work, "writer_pdf_Export")
    shutil.copy(os.path.join(work, "spavbl-rlofodt.pdf"), OUT)


# =================================================================== python-docx helpers
def docx_field(paragraph, instr, cached="1", **font):
    """A complex field (PAGE, STYLEREF ...) with a cached result."""
    from docx.oxml import OxmlElement
    from docx.oxml.ns import qn

    def run_with(child):
        r = paragraph.add_run()
        for k, v in font.items():
            setattr(r.font, k, v)
        r._r.append(child)
        return r
    b = OxmlElement("w:fldChar")
    b.set(qn("w:fldCharType"), "begin")
    run_with(b)
    t = OxmlElement("w:instrText")
    t.set(qn("xml:space"), "preserve")
    t.text = f" {instr} "
    run_with(t)
    s = OxmlElement("w:fldChar")
    s.set(qn("w:fldCharType"), "separate")
    run_with(s)
    r = paragraph.add_run(cached)
    for k, v in font.items():
        setattr(r.font, k, v)
    e = OxmlElement("w:fldChar")
    e.set(qn("w:fldCharType"), "end")
    run_with(e)


# =================================================================== B06
PORTUGUESE_TITLES = {
    "MAT": "O Evangelho Segundo Mateus", "MRK": "O Evangelho Segundo Marcos", "LUK": "O Evangelho Segundo Lucas",
    "JHN": "O Evangelho Segundo João", "ACT": "Atos dos Apóstolos", "ROM": "Epístola de Paulo aos Romanos",
    "1CO": "Primeira Epístola de Paulo aos Coríntios", "2CO": "Segunda Epístola de Paulo aos Coríntios",
    "GAL": "Epístola de Paulo aos Gálatas", "EPH": "Epístola de Paulo aos Efésios",
    "PHP": "Epístola de Paulo aos Filipenses", "COL": "Epístola de Paulo aos Colossenses",
    "1TH": "Primeira Epístola de Paulo aos Tessalonicenses", "2TH": "Segunda Epístola de Paulo aos Tessalonicenses",
    "1TI": "Primeira Epístola de Paulo a Timóteo", "2TI": "Segunda Epístola de Paulo a Timóteo",
    "TIT": "Epístola de Paulo a Tito", "PHM": "Epístola de Paulo a Filemom", "HEB": "Epístola aos Hebreus",
    "JAS": "Epístola de Tiago", "1PE": "Primeira Epístola de Pedro", "2PE": "Segunda Epístola de Pedro",
    "1JN": "Primeira Epístola de João", "2JN": "Segunda Epístola de João", "3JN": "Terceira Epístola de João",
    "JUD": "Epístola de Judas", "REV": "Apocalipse de João",
}


def porblt_pydocx():
    """Bíblia Livre Para Todos NT, python-docx with direct formatting only:
    one section per book (header = book name in capitals at the right, PAGE
    footer); bold 18 pt centred full Portuguese titles; CAPÍTULO N; plain
    verse digits; superscript italic letter callers; notes per book under
    "Notas"."""
    from docx import Document
    from docx.enum.section import WD_SECTION
    from docx.enum.text import WD_ALIGN_PARAGRAPH
    from docx.shared import Pt
    books = load("porblt_usfm.zip", set(NT))
    doc = Document()
    doc.styles["Normal"].font.name = "Times New Roman"
    doc.styles["Normal"].font.size = Pt(11)
    p = doc.add_paragraph()
    p.alignment = WD_ALIGN_PARAGRAPH.CENTER
    p.paragraph_format.space_before = Pt(200)
    r = p.add_run("O NOVO TESTAMENTO")
    r.bold = True
    r.font.size = Pt(28)
    p = doc.add_paragraph()
    p.alignment = WD_ALIGN_PARAGRAPH.CENTER
    p.add_run("Bíblia Livre Para Todos").italic = True
    for b in books:
        sec = doc.add_section(WD_SECTION.NEW_PAGE)
        sec.header.is_linked_to_previous = False
        sec.footer.is_linked_to_previous = False
        hp = sec.header.paragraphs[0]
        hp.alignment = WD_ALIGN_PARAGRAPH.RIGHT
        hp.add_run((b["h"] or b["title"]).upper()).font.size = Pt(9)
        fp = sec.footer.paragraphs[0]
        fp.alignment = WD_ALIGN_PARAGRAPH.CENTER
        docx_field(fp, "PAGE", size=Pt(9))
        p = doc.add_paragraph()
        p.alignment = WD_ALIGN_PARAGRAPH.CENTER
        p.paragraph_format.space_after = Pt(12)
        r = p.add_run(PORTUGUESE_TITLES.get(b["code"], b["title"]))
        r.bold = True
        r.font.size = Pt(18)
        notes = []
        chap, verse, k = 0, "1", 0
        for bl in b["blocks"]:
            if bl[0] == "chapter":
                chap, verse, k = bl[1], "1", 0
                p = doc.add_paragraph()
                p.alignment = WD_ALIGN_PARAGRAPH.CENTER
                p.paragraph_format.space_before = Pt(10)
                p.add_run(f"CAPÍTULO {chap}").bold = True
            elif bl[0] == "para":
                if bl[1] == "b" or not bl[2]:
                    continue
                p = doc.add_paragraph()
                p.alignment = WD_ALIGN_PARAGRAPH.JUSTIFY
                if bl[1].startswith("pi"):
                    p.paragraph_format.left_indent = Pt(18)
                for kind, val in bl[2]:
                    if kind == "v":
                        verse = val
                        p.add_run(f"{val} ")
                    elif kind == "note":
                        k += 1
                        xref, ref, parts = split_note(val, chap, verse)
                        notes.append((letters(k), ref, parts))
                        r = p.add_run(letters(k))
                        r.font.superscript = True
                        r.italic = True
                    else:
                        r = p.add_run(val.lstrip() if p.runs and p.runs[-1].text.endswith(" ") else val)
                        if kind in ("it", "add", "em"):
                            r.italic = True
                        elif kind == "bd":
                            r.bold = True
        if notes:
            p = doc.add_paragraph()
            p.paragraph_format.space_before = Pt(14)
            p.add_run("Notas").bold = True
            for letter, ref, parts in notes:
                p = doc.add_paragraph()
                p.paragraph_format.space_after = Pt(2)
                r = p.add_run(letter)
                r.italic = True
                r.font.size = Pt(9)
                r = p.add_run(f" {ref} ")
                r.font.size = Pt(9)
                for it, t in parts:
                    r = p.add_run(t)
                    r.font.size = Pt(9)
                    r.italic = it
    doc.core_properties.title = "O Novo Testamento"
    doc.save(os.path.join(OUT, "porblt-rpydocx.docx"))


# =================================================================== B07 / C07 / B09 / C09 / C10 (text)
def kor_md_cv():
    """Korean 1910, Markdown: # title, block-quote source, ## \\h, one verse
    per line "C:V text" with a hard line break, no chapter headings."""
    books = load("kor_usfm.zip")
    out = ["# 성경전서", "", "> 출처: eBible.org, kor_usfm.zip — Korean Bible 1910 (한국어 성경), Public Domain", ""]
    for b in books:
        out += [f"## {b['h'] or b['title']}", ""]
        lines = []
        for c, v, text in verses(b):
            lines.append(f"{c}:{v} {text}  ")
        out += lines + [""]
    with open(os.path.join(OUT, "kor-rmdcv.md"), "w", encoding="utf-8") as f:
        f.write("\n".join(out))


def verses(b):
    """(chapter, verse, text) of a book, notes left out."""
    chap, cur = 0, None
    for bl in b["blocks"]:
        if bl[0] == "chapter":
            if cur:
                yield cur[0], cur[1], re.sub(r"\s+", " ", cur[2]).strip()
                cur = None
            chap = bl[1]
        elif bl[0] == "para":
            for kind, val in bl[2]:
                if kind == "v":
                    if cur:
                        yield cur[0], cur[1], re.sub(r"\s+", " ", cur[2]).strip()
                    cur = [chap, val, ""]
                elif kind != "note" and cur is not None:
                    cur[2] += val
            if cur:
                cur[2] += " "
    if cur:
        yield cur[0], cur[1], re.sub(r"\s+", " ", cur[2]).strip()


def kor_txtfull():
    """Korean 1910, plain text: one verse per line "Bookname C:V text", no headings."""
    books = load("kor_usfm.zip")
    lines = []
    for b in books:
        name = b["h"] or b["title"]
        lines += [f"{name} {c}:{v} {t}" for c, v, t in verses(b)]
    with open(os.path.join(OUT, "kor-rtxtfull.txt"), "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")


def engdra_txtbrk():
    """Douay-Rheims, plain text: full capital titles from the \\mt lines,
    Chapter N / Psalm N, prose wrapped at 72 columns with [n] markers,
    poetry one line each indented 4 spaces."""
    books = load("engDRA_usfm.zip")
    out = ["THE HOLY BIBLE", "Douay-Rheims Version, 1899 American Edition",
           "Translated from the Latin Vulgate", "", ""]
    wrap = textwrap.TextWrapper(width=72, break_long_words=False, break_on_hyphens=False)
    for b in books:
        title = " ".join(t for _, t in b["mts"]) or b["title"]
        out += [re.sub(r"\s+", " ", title).upper(), ""]
        psalm = b["code"] == "PSA"
        for bl in b["blocks"]:
            if bl[0] == "chapter":
                out += ["", f"{'Psalm' if psalm else 'Chapter'} {bl[1]}", ""]
            elif bl[0] == "para":
                text = ""
                for kind, val in bl[2]:
                    if kind == "v":
                        text += f" [{val}] "
                    elif kind != "note":
                        text += val
                text = re.sub(r"\s+", " ", text).strip()
                if not text:
                    continue
                if bl[1] in POETRY:
                    out.append("    " + text)
                else:
                    out += wrap.wrap(text) + [""]
        out += ["", ""]
    with open(os.path.join(OUT, "engdra-rtxtbrk.txt"), "w", encoding="utf-8") as f:
        f.write(re.sub(r"\n{4,}", "\n\n\n", "\n".join(out)).rstrip() + "\n")


def engwebster_txtnonum():
    """Webster 1833, plain text: \\toc1 titles in capitals, CHAPTER N, one
    verse per line without numbers."""
    books = load("engwebster_usfm.zip")
    out = ["THE HOLY BIBLE", "Noah Webster's Revision of the Authorized Version (1833)", "", ""]
    for b in books:
        out += [(b["toc1"] or b["title"]).upper(), ""]
        last = None
        for c, v, t in verses(b):
            if c != last:
                out += ["", f"CHAPTER {c}", ""]
                last = c
            out.append(t)
        out += ["", ""]
    with open(os.path.join(OUT, "engwebster-rtxtnonum.txt"), "w", encoding="utf-8") as f:
        f.write("\n".join(out).rstrip() + "\n")


def grctisch_mdbold():
    """Tischendorf 8th, Markdown: document title, the introduction (# title,
    ### section heads), # \\mt1 book headings, ## N chapters, **n** verses."""
    books = load("grc-tisch_usfm.zip", {"INT"} | set(NT))
    out = ["# Η ΚΑΙΝΗ ΔΙΑΘΗΚΗ — Tischendorf 8th edition", ""]
    for b in books:
        if b["code"] == "INT":
            out += [f"# {md_esc(' '.join(t for _, t in b['mts']))}", ""]
            for bl in b["blocks"]:
                if bl[0] != "intro" or not bl[2]:
                    continue
                text = "".join(md_run(k, v) for k, v in bl[2] if k not in ("v", "note")).strip()
                out += [f"### {text}" if bl[1].startswith("is") else text, ""]
            continue
        out += [f"# {md_esc(b['mt'][0] if b['mt'] else b['title'])}", ""]
        for bl in b["blocks"]:
            if bl[0] == "chapter":
                out += [f"## {bl[1]}", ""]
            elif bl[0] == "para":
                text = "".join(f" **{v}** " if k == "v" else md_run(k, v) for k, v in bl[2] if k != "note")
                text = re.sub(r"\s+", " ", text).strip()
                if text:
                    out += [text, ""]
    with open(os.path.join(OUT, "grctisch-rmdbold.md"), "w", encoding="utf-8") as f:
        f.write("\n".join(out))


# =================================================================== B08
def grcsbl_fpdf_mrk():
    """SBLGNT Mark, fpdf2 148x210 mm booklet: no title anywhere; folio
    centred at the top from page 2; 20 pt bold chapter numerals inline
    replacing verse 1; small superscript verses; apparatus after each
    chapter under a short rule in 7 pt, entries run together."""
    from fpdf import FPDF

    class Booklet(FPDF):
        def header(self):
            if self.page_no() > 1:
                self.set_font("FS", "", 8)
                self.set_y(8)
                self.cell(0, 5, str(self.page_no()), align="C")
                self.set_y(self.t_margin)

    books = load("grcsbl_usfm.zip", {"MRK"})
    pdf = Booklet(format=(148, 210))
    pdf.set_margins(16, 16, 16)
    pdf.set_auto_page_break(True, 16)
    for style, f in (("", ""), ("B", "Bold"), ("I", "Italic")):
        pdf.add_font("FS", style, FREESERIF.format(f))
    pdf.add_page()
    for b in books:
        chapters = []
        for bl in b["blocks"]:
            if bl[0] == "chapter":
                chapters.append([bl[1], [], []])
            elif bl[0] == "para" and chapters:
                chapters[-1][1].append(bl[2])
        for chap, paras, app in chapters:
            first = True
            with pdf.text_columns(text_align="J", ncols=1, line_height=1.15) as cols:
                for pieces in paras:
                    with cols.paragraph() as par:
                        if first:
                            pdf.set_font("FS", "B", 20)
                            par.write(f"{chap} ")
                        pdf.set_font("FS", "", 10.5)
                        for kind, val in pieces:
                            if kind == "v":
                                if first and val == "1":
                                    continue
                                pdf.set_font("FS", "", 10.5)
                                pdf.char_vpos = "SUP"
                                par.write(val)
                                pdf.char_vpos = "LINE"
                            elif kind == "note":
                                app.append(note_plain(val[2:] if val.startswith("x:") else val).strip())
                            else:
                                par.write(val if not first else val.lstrip())
                            first = False
                    first = False
            if app:
                pdf.ln(2)
                y = pdf.get_y()
                if y > pdf.h - 30:
                    pdf.add_page()
                    y = pdf.get_y()
                pdf.set_line_width(0.2)
                pdf.line(pdf.l_margin, y, pdf.l_margin + 25, y)
                pdf.ln(1.5)
                pdf.set_font("FS", "", 7)
                pdf.multi_cell(0, 3.1, "; ".join(a.rstrip(" ;") for a in app), align="J")
                pdf.ln(3)
    pdf.output(os.path.join(OUT, "grcsbl-rfpdf-mrk.pdf"))


# =================================================================== B11 / C04 (pandoc EPUB)
def englsv_pandocepub():
    """LSV, pandoc Markdown -> EPUB 3 with --split-level=2: h1 \\toc1 in its
    own file, every chapter (h2 "Genesis 1") its own file, superscript verse
    numbers, LSV's [ ] and || kept, TOC depth 1."""
    books = load("englsv_usfm.zip")
    md = ["---", "title: Literal Standard Version", "author: Covenant Press", "lang: en-US",
          "rights: Copyright © 2020 Covenant Press. CC BY-SA 4.0.", "---", ""]
    for b in books:
        md += [f"# {md_esc(b['toc1'] or b['title'])}", ""]
        name = b["h"] or b["title"]
        for bl in b["blocks"]:
            if bl[0] == "chapter":
                md += [f"## {md_esc(name)} {bl[1]}", ""]
            elif bl[0] == "para":
                text = "".join(f"^{md_esc(v)}^ " if k == "v" else md_run(k, v) for k, v in bl[2] if k != "note")
                text = re.sub(r"[ \t]+", " ", text).strip()
                if text:
                    md += [text, ""]
    work = tempfile.mkdtemp()
    src = os.path.join(work, "englsv.md")
    with open(src, "w", encoding="utf-8") as f:
        f.write("\n".join(md))
    pandoc(src, os.path.join(OUT, "englsv-rpandocepub.epub"), "-t", "epub3", "--split-level=2", "--toc-depth=1")


def frasbl_pandocepub():
    """Sainte Bible libre pour le monde, pandoc -> EPUB 3 with
    --split-level=1: h1 \\toc1, h2 Chapitre N / Psaume N, bold verse numbers,
    italic psalm titles, poetry as line blocks, footnotes and cross
    references ("Réf. :") as pandoc notes at the end of each book file."""
    books = load("frasbl_usfm.zip")
    md = ["---", "title: Sainte Bible libre pour le monde", "author: David Williams, Michael Paul Johnson",
          "lang: fr-FR", "rights: Domaine public", "---", ""]
    for b in books:
        md += [f"# {md_esc(b['toc1'] or b['title'])}", ""]
        psalm = b["code"] == "PSA"
        chap, verse = 0, "1"
        block = []

        def flush():
            if block:
                md.extend(block + [""])
                block.clear()

        for bl in b["blocks"]:
            if bl[0] == "chapter":
                flush()
                chap, verse = bl[1], "1"
                md += [f"## {'Psaume' if psalm else 'Chapitre'} {chap}", ""]
            elif bl[0] == "heading":
                flush()
                if bl[1] == "d":
                    md += [f"*{md_esc(bl[2])}*", ""]
                elif bl[1] in ("s", "s1", "s2", "ms", "ms1"):
                    md += [f"### {md_esc(bl[2])}", ""]
                elif bl[1] == "sp":
                    md += [f"*{md_esc(bl[2])}*", ""]
            elif bl[0] == "para":
                out = []
                for kind, val in bl[2]:
                    if kind == "v":
                        verse = val
                        out.append(f"**{md_esc(val)}** ")
                    elif kind == "note":
                        xref, ref, parts = split_note(val, chap, verse)
                        body = "".join(md_run("it" if it else "t", t) for it, t in parts).strip()
                        out.append(f"^[{'Réf. : ' if xref else ''}{md_esc(ref)} {body}]")
                    else:
                        out.append(md_run(kind, val))
                text = re.sub(r"[ \t]+", " ", "".join(out)).strip()
                if bl[1] == "b":
                    flush()
                    continue
                if not text:
                    continue
                if bl[1] in POETRY:
                    indent = "   " if bl[1] in ("q2", "q3") else ""
                    block.append(f"| {indent}{text}")
                else:
                    flush()
                    md += [text, ""]
        flush()
    work = tempfile.mkdtemp()
    src = os.path.join(work, "frasbl.md")
    with open(src, "w", encoding="utf-8") as f:
        f.write("\n".join(md))
    pandoc(src, os.path.join(OUT, "frasbl-rpandocepub.epub"), "-t", "epub3", "--split-level=1",
           "--reference-location=section")


# =================================================================== C01
def engkjvcpb_wpdfmargin():
    """KJV Cambridge Paragraph Bible, WeasyPrint 5.83x8.27 in, justified and
    hyphenated: title page and "The Translators to the Reader" with lower
    roman folios; stacked capital book titles; CHAPTER III / PSALM XXIII;
    verse numbers absolutely positioned in the left margin; running foot
    "BOOK c:v–c:v" outer and folio inner."""
    books = load("engkjvcpb_usfm.zip", {"FRT", "INT"} | set(OT) | set(NT))
    foot = "font: 7.5pt 'Liberation Serif'; letter-spacing: 0.04em;"
    css = f"""
    @page {{ size: 5.83in 8.27in; margin: 0.6in 0.55in 0.7in 0.55in; }}
    @page :left {{ @bottom-left {{ content: string(bk) " " string(cv, first) "–" string(cv, last); {foot} }}
                   @bottom-right {{ content: counter(page); {foot} }} }}
    @page :right {{ @bottom-right {{ content: string(bk) " " string(cv, first) "–" string(cv, last); {foot} }}
                    @bottom-left {{ content: counter(page); {foot} }} }}
    @page front {{ @bottom-left {{ content: none }} @bottom-right {{ content: none }}
                   @bottom-center {{ content: counter(page, lower-roman); {foot} }} }}
    @page front:first {{ @bottom-center {{ content: none }} }}
    @page :nth(1 of start) {{ counter-reset: page 1; }}
    html {{ font: 9.5pt/1.32 'Liberation Serif'; text-align: justify; hyphens: auto; }}
    .front {{ page: front; }}
    .titlepage {{ text-align: center; break-after: page; padding-top: 0.6in; }}
    .titlepage .mt1 {{ font-size: 17pt; margin: 6pt 0; }} .titlepage .mt2 {{ font-size: 11pt; margin: 5pt 0; }}
    .titlepage .mt3 {{ font-size: 8pt; margin: 3pt 0; }}
    .int h1 {{ text-align: center; font-weight: normal; margin: 0.2in 0 0.2in; }}
    .int h1 span {{ display: block; }} .int h1 .mt1 {{ font-size: 14pt; }} .int h1 .mt2 {{ font-size: 9pt; }}
    .int p {{ text-indent: 1em; margin: 0; }} .int h3 {{ font-size: 9.5pt; text-align: center; font-weight: normal; font-style: italic; }}
    section.book {{ break-before: page; }}
    section.start {{ page: start; }}
    .btitle {{ text-align: center; margin: 0.15in 0 0.2in; string-set: bk attr(data-bk); }}
    .btitle span {{ display: block; }} .btitle .mt1 {{ font-size: 15pt; letter-spacing: 0.06em; margin-top: 2pt; }}
    .btitle .mt2 {{ font-size: 9pt; letter-spacing: 0.05em; }} .btitle .mt3 {{ font-size: 7.5pt; }}
    .text {{ margin-left: 0.3in; position: relative; }}
    h2 {{ font-size: 8.5pt; font-weight: normal; letter-spacing: 0.12em; text-align: center; margin: 7pt 0 3pt; }}
    p {{ margin: 0; text-indent: 1em; }}
    p.m, p.nb, p.mi {{ text-indent: 0; }}
    p.q1, p.q, p.q2, p.q3 {{ text-align: left; margin-left: 1.5em; text-indent: -1em; }}
    p.q2 {{ margin-left: 2.3em; }}
    p.qc {{ text-align: center; text-indent: 0; }} p.b {{ height: 4pt; }}
    p.d {{ font-style: italic; text-indent: 0; margin-bottom: 2pt; }}
    .vn {{ position: absolute; left: -0.3in; width: 0.22in; text-align: right; text-indent: 0; font-size: 6.5pt;
           font-style: normal; font-weight: normal; string-set: cv attr(data-cv); }}
    .sc {{ font-variant: small-caps; }}
    """
    parts = [f"<html lang='en'><head><meta charset='utf-8'><title>The Cambridge Paragraph Bible</title><style>{css}</style></head><body>"]
    first_book = True
    for b in books:
        if b["code"] == "FRT":
            lines = "".join(f"<div class='{m if m in ('mt1', 'mt2') else 'mt3'}'>{esc(t)}</div>" for m, t in b["mts"])
            parts.append(f"<div class='front titlepage'>{lines}</div>")
            continue
        if b["code"] == "INT":
            lines = "".join(f"<span class='{m}'>{esc(t)}</span>" for m, t in b["mts"])
            parts.append(f"<div class='front int'><h1>{lines}</h1>{intro_html(b['blocks'])}</div>")
            continue
        name = (b["h"] or b["title"]).upper()
        psalm = b["code"] == "PSA"
        lines = "".join(f"<span class='{m if m in ('mt1', 'mt2') else 'mt3'}'>{esc(t)}</span>" for m, t in b["mts"])
        parts.append(f"<section class='book{' start' if first_book else ''}'><div class='btitle' data-bk='{esc(name)}'>{lines}</div><div class='text'>")
        first_book = False
        chap = 0
        for bl in b["blocks"]:
            if bl[0] == "chapter":
                chap = bl[1]
                parts.append(f"<h2>{'PSALM' if psalm else 'CHAPTER'} {roman(chap)}</h2>")
            elif bl[0] == "heading" and bl[1] == "d":
                parts.append(f"<p class='d'>{esc(bl[2])}</p>")
            elif bl[0] == "para":
                out = []
                for kind, val in bl[2]:
                    if kind == "v":
                        num = re.sub(r"[a-z]$", "", val)
                        out.append(f"<span class='vn' data-cv='{chap}:{esc(num)}'>{esc(num)}</span>")
                    elif kind != "note":
                        out.append(run_html(kind, val))
                text = "".join(out).strip()
                if text or bl[1] == "b":
                    parts.append(f"<p class='{bl[1]}'>{text}</p>")
        parts.append("</div></section>")
    parts.append("</body></html>")
    weasy("\n".join(parts), os.path.join(OUT, "engkjvcpb-rwpdfmargin.pdf"))


# =================================================================== C02
def engjps_docx2lo():
    """JPS 1917, python-docx with custom styles (Bible Book at outline level
    1, Chapter Number, Bible Text, Bible Poetry, Verse Number), 17x24 cm, two
    columns with a separator; STYLEREF running head, PAGE footer; exported to
    PDF by LibreOffice."""
    from docx import Document
    from docx.enum.section import WD_SECTION
    from docx.enum.style import WD_STYLE_TYPE
    from docx.enum.text import WD_ALIGN_PARAGRAPH
    from docx.oxml import OxmlElement
    from docx.oxml.ns import qn
    from docx.shared import Cm, Pt
    books = load("engjps_usfm.zip")
    doc = Document()
    doc.styles["Normal"].font.name = "Liberation Serif"
    doc.styles["Normal"].font.size = Pt(10)
    st = doc.styles
    book_st = st.add_style("Bible Book", WD_STYLE_TYPE.PARAGRAPH)
    book_st.base_style = st["Normal"]
    book_st.font.size = Pt(16)
    book_st.font.bold = True
    book_st.paragraph_format.alignment = WD_ALIGN_PARAGRAPH.CENTER
    book_st.paragraph_format.space_after = Pt(12)
    book_st.paragraph_format.keep_with_next = True
    lvl = OxmlElement("w:outlineLvl")
    lvl.set(qn("w:val"), "0")
    book_st.element.get_or_add_pPr().append(lvl)
    ch_st = st.add_style("Chapter Number", WD_STYLE_TYPE.PARAGRAPH)
    ch_st.base_style = st["Normal"]
    ch_st.font.bold = True
    ch_st.font.size = Pt(13)
    ch_st.paragraph_format.alignment = WD_ALIGN_PARAGRAPH.CENTER
    ch_st.paragraph_format.space_before = Pt(8)
    ch_st.paragraph_format.space_after = Pt(3)
    ch_st.paragraph_format.keep_with_next = True
    text_st = st.add_style("Bible Text", WD_STYLE_TYPE.PARAGRAPH)
    text_st.base_style = st["Normal"]
    text_st.paragraph_format.alignment = WD_ALIGN_PARAGRAPH.JUSTIFY
    text_st.paragraph_format.space_after = Pt(2)
    poet_st = st.add_style("Bible Poetry", WD_STYLE_TYPE.PARAGRAPH)
    poet_st.base_style = text_st
    poet_st.paragraph_format.alignment = WD_ALIGN_PARAGRAPH.LEFT
    poet_st.paragraph_format.left_indent = Cm(0.8)
    poet_st.paragraph_format.first_line_indent = Cm(-0.5)
    vn_st = st.add_style("Verse Number", WD_STYLE_TYPE.CHARACTER)
    vn_st.font.bold = True
    vn_st.font.size = Pt(7)

    def page_setup(sec, cols):
        sec.page_width, sec.page_height = Cm(17), Cm(24)
        sec.left_margin = sec.right_margin = Cm(1.8)
        sec.top_margin, sec.bottom_margin = Cm(2), Cm(1.8)
        sec.header_distance = sec.footer_distance = Cm(1)
        c = sec._sectPr.find(qn("w:cols"))
        if c is None:
            c = OxmlElement("w:cols")
            sec._sectPr.append(c)
        c.set(qn("w:num"), str(cols))
        c.set(qn("w:space"), "425")
        if cols > 1:
            c.set(qn("w:sep"), "1")

    page_setup(doc.sections[0], 1)
    p = doc.add_paragraph()
    p.alignment = WD_ALIGN_PARAGRAPH.CENTER
    p.paragraph_format.space_before = Pt(180)
    r = p.add_run("THE HOLY SCRIPTURES")
    r.bold = True
    r.font.size = Pt(24)
    p = doc.add_paragraph()
    p.alignment = WD_ALIGN_PARAGRAPH.CENTER
    p.add_run("ACCORDING TO THE MASORETIC TEXT").font.size = Pt(13)
    p = doc.add_paragraph()
    p.alignment = WD_ALIGN_PARAGRAPH.CENTER
    p.paragraph_format.space_before = Pt(40)
    p.add_run("Jewish Publication Society, 1917").italic = True
    for i, b in enumerate(books):
        sec = doc.add_section(WD_SECTION.NEW_PAGE)
        page_setup(sec, 2)
        name = (b["h"] or b["title"]).upper()
        if i == 0:
            sec.header.is_linked_to_previous = False
            sec.footer.is_linked_to_previous = False
            hp = sec.header.paragraphs[0]
            hp.alignment = WD_ALIGN_PARAGRAPH.CENTER
            docx_field(hp, 'STYLEREF "Bible Book"', name, size=Pt(9))
            fp = sec.footer.paragraphs[0]
            fp.alignment = WD_ALIGN_PARAGRAPH.CENTER
            docx_field(fp, "PAGE", "1", size=Pt(9))
        doc.add_paragraph(name, style="Bible Book")
        for bl in b["blocks"]:
            if bl[0] == "chapter":
                doc.add_paragraph(str(bl[1]), style="Chapter Number")
            elif bl[0] == "para":
                if not bl[2]:
                    continue
                p = doc.add_paragraph(style="Bible Poetry" if bl[1] in POETRY else "Bible Text")
                for kind, val in bl[2]:
                    if kind == "v":
                        p.add_run(val, style="Verse Number")
                        p.add_run(" ")
                    elif kind != "note":
                        r = p.add_run(val.lstrip() if p.runs and p.runs[-1].text == " " else val)
                        r.italic = kind in ("add", "it", "em")
    doc.core_properties.title = "The Holy Scriptures according to the Masoretic Text"
    work = tempfile.mkdtemp()
    src = os.path.join(work, "engjps-rdocx2lo.docx")
    doc.save(src)
    soffice(src, "pdf", work, "writer_pdf_Export")
    shutil.copy(os.path.join(work, "engjps-rdocx2lo.pdf"), OUT)


# =================================================================== C03
def deu1951_pydocxdrop():
    """Schlachter 1951, python-docx: book titles in the built-in Title style;
    each book a new odd-page section with header "<book> [tab] PAGE"; chapter
    numbers as real Word drop caps (framePr dropCap, 2 lines) with verse 1
    unnumbered; Psalms: bold centred "Psalm N" (verse 1 numbered);
    superscript verse numbers followed by a narrow no-break space."""
    from docx import Document
    from docx.enum.section import WD_SECTION
    from docx.enum.text import WD_ALIGN_PARAGRAPH
    from docx.oxml import OxmlElement
    from docx.oxml.ns import qn
    from docx.shared import Pt
    books = load("deu1951_usfm.zip")
    doc = Document()
    doc.styles["Normal"].font.name = "Liberation Serif"
    doc.styles["Normal"].font.size = Pt(11)
    p = doc.add_paragraph()
    p.alignment = WD_ALIGN_PARAGRAPH.CENTER
    p.paragraph_format.space_before = Pt(220)
    r = p.add_run("Die Heilige Schrift")
    r.font.size = Pt(30)
    p = doc.add_paragraph()
    p.alignment = WD_ALIGN_PARAGRAPH.CENTER
    p.add_run("übersetzt von Franz Eugen Schlachter (1951)").italic = True

    def drop_cap(number):
        p = doc.add_paragraph()
        ppr = p._p.get_or_add_pPr()
        fr = OxmlElement("w:framePr")
        for k, v in (("w:dropCap", "drop"), ("w:lines", "2"), ("w:wrap", "around"), ("w:vAnchor", "text"), ("w:hAnchor", "text")):
            fr.set(qn(k), v)
        ppr.insert(0, fr)
        sp = OxmlElement("w:spacing")
        sp.set(qn("w:after"), "0")
        sp.set(qn("w:line"), "540")
        sp.set(qn("w:lineRule"), "exact")
        ppr.append(sp)
        r = p.add_run(str(number))
        r.font.size = Pt(31)
        r.bold = True
        pos = OxmlElement("w:position")
        pos.set(qn("w:val"), "-4")
        rpr = r._r.get_or_add_rPr()
        rpr.insert(list(rpr).index(rpr.find(qn("w:sz"))), pos)

    for b in books:
        sec = doc.add_section(WD_SECTION.ODD_PAGE)
        sec.header.is_linked_to_previous = False
        hp = sec.header.paragraphs[0]
        name = b["h"] or b["title"]
        r = hp.add_run(f"{name}\t")
        r.font.size = Pt(9)
        docx_field(hp, "PAGE", size=Pt(9))
        # Right tab at the text edge.
        tabs = OxmlElement("w:tabs")
        tab = OxmlElement("w:tab")
        tab.set(qn("w:val"), "right")
        tab.set(qn("w:pos"), str(int((sec.page_width - sec.left_margin - sec.right_margin) / 635)))
        tabs.append(tab)
        hp._p.get_or_add_pPr().append(tabs)
        doc.add_paragraph(b["mt"][0] if b["mt"] else b["title"], style="Title")
        psalm = b["code"] == "PSA"
        pending = None
        for bl in b["blocks"]:
            if bl[0] == "chapter":
                if psalm:
                    p = doc.add_paragraph()
                    p.alignment = WD_ALIGN_PARAGRAPH.CENTER
                    p.paragraph_format.space_before = Pt(10)
                    p.add_run(f"Psalm {bl[1]}").bold = True
                else:
                    drop_cap(bl[1])
                    pending = bl[1]
            elif bl[0] == "para":
                if not bl[2]:
                    continue
                p = doc.add_paragraph()
                p.alignment = WD_ALIGN_PARAGRAPH.JUSTIFY
                for kind, val in bl[2]:
                    if kind == "v":
                        if pending is not None and val == "1":
                            pending = None
                            continue
                        r = p.add_run(val)
                        r.font.superscript = True
                        p.add_run(" ")
                    elif kind != "note":
                        p.add_run(val.lstrip() if not p.runs or p.runs[-1].text == " " else val)
                pending = None
    doc.core_properties.title = "Die Heilige Schrift"
    doc.save(os.path.join(OUT, "deu1951-rpydocxdrop.docx"))


# =================================================================== C05
def spabll_rlab2col():
    """Santa Biblia libre Latinoamericano, ReportLab A4 two frames with a
    centre rule; books run on (a third of a page needed); \\toc1 capital
    titles; 19 pt bold inline chapter numerals (also in Psalms); bold
    superscript verses; italic superscript note numbers, notes under NOTAS at
    each book's end; dictionary running heads (even: first reference top
    left, odd: last reference top right); folio bottom centre."""
    from reportlab.lib.enums import TA_CENTER, TA_JUSTIFY, TA_LEFT
    from reportlab.lib.pagesizes import A4
    from reportlab.lib.styles import ParagraphStyle
    from reportlab.lib.units import cm
    from reportlab.platypus import CondPageBreak, Frame, PageTemplate, Paragraph, Spacer
    from reportlab.platypus.doctemplate import BaseDocTemplate
    reportlab_fonts()
    W, H = A4
    books = load("spabll_usfm.zip")
    LM, RM, TM, BM, GAP = 1.6 * cm, 1.6 * cm, 2.0 * cm, 1.8 * cm, 0.8 * cm

    def draw(c, labels, page):
        c.saveState()
        c.setFont("Serif", 8.5)
        if labels:
            if page % 2 == 0:
                c.drawString(LM, H - 1.3 * cm, labels[0])
            else:
                c.drawRightString(W - RM, H - 1.3 * cm, labels[-1])
        c.drawCentredString(W / 2, 1.0 * cm, str(page))
        c.setLineWidth(0.4)
        c.line(W / 2, BM, W / 2, H - TM)
        c.restoreState()

    base = dict(fontName="Serif", fontSize=9, leading=11.2)
    st = {
        "p": ParagraphStyle("p", alignment=TA_JUSTIFY, firstLineIndent=10, **base),
        "m": ParagraphStyle("m", alignment=TA_JUSTIFY, **base),
        "q1": ParagraphStyle("q1", alignment=TA_LEFT, leftIndent=18, firstLineIndent=-10, **base),
        "q2": ParagraphStyle("q2", alignment=TA_LEFT, leftIndent=28, firstLineIndent=-10, **base),
        "title": ParagraphStyle("title", fontName="Serif-Bold", fontSize=12, leading=15, alignment=TA_CENTER, spaceBefore=10, spaceAfter=8),
        "s": ParagraphStyle("s", fontName="Serif-Bold", fontSize=9, leading=12, alignment=TA_CENTER, spaceBefore=5, spaceAfter=2),
        "d": ParagraphStyle("d", fontName="Serif-Italic", fontSize=8.5, leading=10.5, alignment=TA_LEFT, spaceAfter=1),
        "notes": ParagraphStyle("notes", fontName="Serif-Bold", fontSize=9, leading=12, alignment=TA_CENTER, spaceBefore=6, spaceAfter=2),
        "note": ParagraphStyle("note", fontName="Serif", fontSize=7.5, leading=9, alignment=TA_JUSTIFY),
    }
    story = []
    for b in books:
        name = (b["h"] or b["title"]).upper()
        story.append(CondPageBreak(H / 3))
        story.append(Paragraph(xesc((b["toc1"] or b["title"]).upper()), st["title"]))
        chap, verse, pending, notes = 0, "1", None, []
        for bl in b["blocks"]:
            if bl[0] == "chapter":
                chap, verse, pending = bl[1], "1", bl[1]
            elif bl[0] == "heading":
                if bl[1] == "d":
                    story.append(Paragraph(xesc(bl[2]), st["d"]))
                elif bl[1] in ("s", "s1", "s2") or bl[1].startswith("ms"):
                    story.append(Paragraph(small_caps(bl[2], 9), st["s"]))
                elif bl[1] == "sp":
                    story.append(Paragraph(f"<i>{xesc(bl[2])}</i>", st["d"]))
            elif bl[0] == "para":
                if bl[1] == "b":
                    story.append(Spacer(1, 3))
                    continue
                out = []
                big = pending is not None
                if pending is not None:
                    out.append(f"<onDraw name='verse' label='{xesc(name)} {pending}:1'/><font size='19'><b>{pending}</b></font>&nbsp;")
                for kind, val in bl[2]:
                    if kind == "v":
                        verse = val
                        if pending is not None and val == "1":
                            pending = None
                            continue
                        out.append(f"<onDraw name='verse' label='{xesc(name)} {chap}:{xesc(val)}'/><super><b>{xesc(val)}</b></super>")
                    elif kind == "note":
                        xref, ref, parts = split_note(val, chap, verse)
                        notes.append((ref, parts))
                        out.append(f"<super><i>{len(notes)}</i></super>")
                    else:
                        out.append(rl_run(kind, val, 9))
                pending = None
                text = "".join(out).strip()
                if not text:
                    continue
                m = bl[1]
                style = st["q1"] if m in ("q", "q1", "q3") else st["q2"] if m == "q2" \
                    else st["m"] if m in ("m", "mi", "nb") else st["p"]
                if big:  # the 19 pt numeral pushes the lines down
                    style = ParagraphStyle(style.name + "-c", parent=style, spaceBefore=3, spaceAfter=10, firstLineIndent=0)
                story.append(Paragraph(text, style))
        if notes:
            story.append(Paragraph("NOTAS", st["notes"]))
            for i, (ref, parts) in enumerate(notes, 1):
                body = "".join(f"<i>{xesc(t)}</i>" if it else xesc(t) for it, t in parts)
                story.append(Paragraph(f"<i>{i}</i> {xesc(ref)} {body}", st["note"]))
        story.append(Spacer(1, 8))
    path = os.path.join(OUT, "spabll-rrlab2col.pdf")
    doc = BaseDocTemplate(path, pagesize=A4, leftMargin=LM, rightMargin=RM, topMargin=TM, bottomMargin=BM,
                          title="Santa Biblia libre Latinoamericano")
    colw = (W - LM - RM - GAP) / 2
    frames = [Frame(LM, BM, colw, H - TM - BM, id="c1", leftPadding=0, rightPadding=0),
              Frame(LM + colw + GAP, BM, colw, H - TM - BM, id="c2", leftPadding=0, rightPadding=0)]
    doc.addPageTemplates([PageTemplate(id="two", frames=frames)])
    doc.build(story, canvasmaker=head_canvas(draw))


# =================================================================== C06
def poronbv_wpdfa5():
    """Nova Bíblia Viva, WeasyPrint A5: title page; book on a new page with
    centred h1 and no head; running head = italic book name top outer,
    folio bottom outer; spaced bold CAPÍTULO N / SALMO N; bold inline verse
    numbers; page-foot footnotes in body size with * † ‡ § callers per
    chapter, each opening with a bold c.v."""
    books = load("poronbv_usfm.zip")
    hd = "font: italic 9pt 'Liberation Serif';"
    fo = "font: 9pt 'Liberation Serif';"
    css = f"""
    @counter-style sym {{ system: symbolic; symbols: "*" "†" "‡" "§"; }}
    @page {{ size: A5; margin: 0.7in 0.55in 0.7in 0.55in;
      @footnote {{ border-top: 0.4pt solid #000; padding-top: 3pt; }} }}
    @page :left {{ @top-left {{ content: string(bk); {hd} }} @bottom-left {{ content: counter(page); {fo} }} }}
    @page :right {{ @top-right {{ content: string(bk); {hd} }} @bottom-right {{ content: counter(page); {fo} }} }}
    @page :nth(1 of book) {{ @top-left {{ content: none }} @top-right {{ content: none }} }}
    @page :first {{ @top-left {{ content: none }} @top-right {{ content: none }} @bottom-left {{ content: none }} @bottom-right {{ content: none }} }}
    html {{ font: 10pt/1.35 'Liberation Serif'; text-align: justify; hyphens: auto; }}
    .titlepage {{ text-align: center; padding-top: 2in; font-size: 26pt; }}
    .titlepage small {{ display: block; font-size: 12pt; margin-top: 0.3in; }}
    section.book {{ page: book; break-before: page; }}
    h1 {{ text-align: center; font-size: 18pt; margin: 0.3in 0 0.25in; string-set: bk attr(data-h); }}
    .ch {{ counter-reset: footnote; }}
    h2 {{ text-align: center; font-size: 10pt; letter-spacing: 0.25em; margin: 9pt 0 4pt; }}
    b.v {{ font-weight: bold; }}
    p {{ margin: 0 0 2pt; text-indent: 1em; }}
    p.m, p.nb {{ text-indent: 0; }}
    p.q1, p.q, p.q3 {{ margin-left: 1.4em; text-indent: -1em; text-align: left; }} p.q2 {{ margin-left: 2.4em; text-indent: -1em; text-align: left; }}
    p.li1 {{ margin-left: 1.2em; text-indent: -0.8em; }} p.li2 {{ margin-left: 2.2em; text-indent: -0.8em; }}
    p.lh, p.lf {{ text-indent: 0; }} p.b {{ height: 4pt; margin: 0; }}
    p.d {{ font-style: italic; text-indent: 0; }} p.qa, p.sp {{ font-weight: bold; text-indent: 0; margin-top: 3pt; }}
    h3.s {{ font-size: 10pt; text-align: center; font-style: italic; font-weight: normal; }}
    .sc {{ font-variant: small-caps; }}
    .fn {{ float: footnote; font-size: 10pt; line-height: 1.3; text-indent: 0; text-align: left; font-weight: normal; font-style: normal; }}
    .fn::footnote-call {{ content: counter(footnote, sym); vertical-align: super; font-size: 7pt; line-height: 0; }}
    .fn::footnote-marker {{ content: counter(footnote, sym) " "; }}
    """
    parts = [f"<html lang='pt'><head><meta charset='utf-8'><title>Nova Bíblia Viva</title><style>{css}</style></head><body>",
             "<div class='titlepage'>BÍBLIA SAGRADA<small>Nova Bíblia Viva</small></div>"]
    for b in books:
        name = b["h"] or b["title"]
        psalm = b["code"] == "PSA"
        parts.append(f"<section class='book'><h1 data-h='{esc(name)}'>{esc(b['mt'][0] if b['mt'] else b['title'])}</h1>")
        chap, verse, open_ch = 0, "1", False
        for bl in b["blocks"]:
            if bl[0] == "chapter":
                if open_ch:
                    parts.append("</div>")
                parts.append("<div class='ch'>")
                open_ch = True
                chap, verse = bl[1], "1"
                parts.append(f"<h2>{'SALMO' if psalm else 'CAPÍTULO'} {chap}</h2>")
            elif bl[0] == "heading":
                m, t = bl[1], esc(bl[2])
                if m == "d":
                    parts.append(f"<p class='d'>{t}</p>")
                elif m in ("qa", "sp"):
                    parts.append(f"<p class='qa'>{t}</p>")
                elif m in ("s", "s1", "s2"):
                    parts.append(f"<h3 class='s'>{t}</h3>")
            elif bl[0] == "para":
                out = []
                for kind, val in bl[2]:
                    if kind == "v":
                        verse = val
                        out.append(f"<b class='v'>{esc(val)}</b> ")
                    elif kind == "note":
                        xref, ref, nparts = split_note(val, chap, verse, sep=".")
                        out.append(f"<span class='fn'><b>{esc(ref)}</b> {note_html(nparts)}</span>")
                    else:
                        out.append(run_html(kind, val.lstrip() if out and out[-1].endswith("</b> ") else val))
                text = "".join(out).strip()
                if text or bl[1] == "b":
                    parts.append(f"<p class='{bl[1]}'>{text}</p>")
        if open_ch:
            parts.append("</div>")
        parts.append("</section>")
    parts.append("</body></html>")
    weasy("\n".join(parts), os.path.join(OUT, "poronbv-rwpdfa5.pdf"))


# =================================================================== C08
def hbo_wpdfrtl_rut():
    """Hebrew Ruth, WeasyPrint 140x210 mm right-to-left: title h1, "פרק א"
    chapter headings with Hebrew-letter numerals, small raised Hebrew-letter
    verse numbers, running head (book name) from page 2, Arabic folios, one
    Latin colophon line."""
    books = load("hbo_usfm.zip", {"RUT"})
    css = """
    @page { size: 140mm 210mm; margin: 20mm 16mm 18mm 16mm;
      @top-center { content: string(bk); font: 10pt 'Noto Serif Hebrew'; }
      @bottom-center { content: counter(page); font: 9pt 'Liberation Serif'; } }
    @page :first { @top-center { content: none } }
    html { font: 13pt/1.7 'Noto Serif Hebrew'; text-align: justify; }
    h1 { text-align: center; font-size: 24pt; margin: 10mm 0 8mm; string-set: bk content(text); }
    h2 { text-align: center; font-size: 13pt; margin: 6mm 0 2mm; }
    sup { font-size: 8pt; vertical-align: super; line-height: 0; margin-inline-start: 0.15em; }
    p { margin: 0 0 2mm; text-indent: 0; }
    p.colophon { direction: ltr; text-align: center; font: 7.5pt 'Liberation Serif'; margin-top: 10mm; }
    """
    parts = [f"<html dir='rtl' lang='he'><head><meta charset='utf-8'><title>רוּת</title><style>{css}</style></head><body>"]
    for b in books:
        parts.append(f"<h1>{esc(b['mt'][0] if b['mt'] else b['title'])}</h1>")
        for bl in b["blocks"]:
            if bl[0] == "chapter":
                parts.append(f"<h2>פרק {hebrew_numeral(bl[1])}</h2>")
            elif bl[0] == "para":
                out = []
                for kind, val in bl[2]:
                    if kind == "v":
                        n = int(re.match(r"\d+", val).group())
                        out.append(f"<sup>{hebrew_numeral(n)}</sup>&#x2009;")
                    elif kind != "note":
                        out.append(esc(val))
                parts.append(f"<p>{''.join(out).strip()}</p>")
    parts.append("<p class='colophon' lang='en'>Hebrew Masoretic Old Testament (hbo), eBible.org — Public Domain.</p>")
    parts.append("</body></html>")
    weasy("\n".join(parts), os.path.join(OUT, "hbo-rwpdfrtl-rut.pdf"))


# =================================================================== eBible originals
def download_ebible(name, url):
    path = os.path.join(OUT, name)
    req = urllib.request.Request(url, headers={"User-Agent": "abidingword-recon"})
    with urllib.request.urlopen(req, timeout=600) as r, open(path + ".part", "wb") as f:
        shutil.copyfileobj(r, f)
    os.replace(path + ".part", path)


JOBS = {
    # set B
    "engbsb-rwpdf2col": engbsb_wpdf2col,
    "engasv-rrlab1col": engasv_rlab1col,
    "deuelbbk-rpandocdocx": deuelbbk_pandocdocx,
    "francl-rebooklibepub": francl_ebooklibepub,
    "spavbl-rlofodt": spavbl_lofodt,
    "porblt-rpydocx": porblt_pydocx,
    "kor-rmdcv": kor_md_cv,
    "grcsbl-rfpdf-mrk": grcsbl_fpdf_mrk,
    "engdra-rtxtbrk": engdra_txtbrk,
    "engrv-ebiblebook": lambda: download_ebible("eng-rv_book.pdf", "https://ebible.org/pdf/eng-rv/eng-rv_book.pdf"),
    "englsv-rpandocepub": englsv_pandocepub,
    # set C
    "engkjvcpb-rwpdfmargin": engkjvcpb_wpdfmargin,
    "engjps-rdocx2lo": engjps_docx2lo,
    "deu1951-rpydocxdrop": deu1951_pydocxdrop,
    "frasbl-rpandocepub": frasbl_pandocepub,
    "spabll-rrlab2col": spabll_rlab2col,
    "poronbv-rwpdfa5": poronbv_wpdfa5,
    "kor-rtxtfull": kor_txtfull,
    "hbo-rwpdfrtl-rut": hbo_wpdfrtl_rut,
    "grctisch-rmdbold": grctisch_mdbold,
    "engwebster-rtxtnonum": engwebster_txtnonum,
    "engoebus-ebiblea4": lambda: download_ebible("engoebus_a4.pdf", "https://ebible.org/pdf/engoebus/engoebus_a4.pdf"),
}

if __name__ == "__main__":
    for key, job in JOBS.items():
        if ONLY and key not in ONLY:
            continue
        print("building", key, flush=True)
        job()
        print("built", key, flush=True)
