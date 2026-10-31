#!/usr/bin/env python3
"""Reconstructions of the disclosed set D layouts (v0.24.5).

Set D's own generators were kept outside Git and are not available; its
manifest (docs/corpus/withheld-d.tsv) and evaluation
(docs/audit/v0.24.4/withheld-d/) describe each layout. These scripts rebuild
documents *in those layouts* from the same eBible USFM, so the disclosed
failure classes can be reproduced and general fixes tested on them. They are
not the sealed set D files (different checksums, different program
versions): results on them are regression evidence for v0.24.5, never an
independent measure.

    python3 scripts/recon/build_recon_d.py <usfm-folder> <out-folder> [key ...]
"""

import html
import os
import re
import shutil
import subprocess
import sys
import tempfile
import zipfile

sys.path.insert(0, os.path.dirname(__file__))
from usfm_blocks import read_zip  # noqa: E402

SRC, OUT = sys.argv[1], sys.argv[2]
ONLY = set(sys.argv[3:])
os.makedirs(OUT, exist_ok=True)

esc = html.escape
POETRY = {"q", "q1", "q2", "q3", "qm1", "qm2", "qr", "qc"}


def spaced(text):
    return text.upper()


# ---------------------------------------------------------------- WeasyPrint
def weasy(html_text, out):
    from weasyprint import HTML
    HTML(string=html_text).write_pdf(out)


def deu_dropcap():
    """Luther 1912, 5.5x8.5 in, drop-cap chapter numbers, verse 1 unnumbered,
    running head = the book's \\toc1 title in capitals (not on its first page)."""
    books = read_zip(os.path.join(SRC, "deu1912_usfm.zip"))
    css = """
    @page { size: 5.5in 8.5in; margin: 0.8in 0.6in 0.7in 0.6in;
      @top-center { content: string(head, first-except); font: 8pt 'Liberation Serif'; letter-spacing: 0.08em; text-transform: uppercase; }
      @bottom-center { content: counter(page); font: 8pt 'Liberation Serif'; } }
    @page :first { @top-center { content: none } }
    body { font: 10pt/1.3 'Liberation Serif'; text-align: justify; hyphens: auto; }
    h1 { font-size: 16pt; text-align: center; page-break-before: always; margin: 0.4in 0 0.3in; string-set: head content(text); }
    .dc { float: left; font-size: 26pt; line-height: 1; font-weight: bold; margin: 0 4pt 0 0; }
    sup { font-size: 6.5pt; font-style: italic; vertical-align: super; line-height: 0; }
    p { margin: 0 0 3pt; text-indent: 0; }
    p.q1 { margin-left: 1.4em; text-indent: -1em; } p.q2 { margin-left: 2.4em; text-indent: -1em; }
    .titlepage { text-align: center; font-size: 28pt; margin-top: 2.5in; }
    """
    parts = [f"<html lang='de'><head><meta charset='utf-8'><style>{css}</style></head><body>",
             "<div class='titlepage'>Die Bibel</div>"]
    for b in books:
        title = b["toc1"] or b["title"]
        parts.append(f"<h1>{esc(title)}</h1>")
        head_set = False
        pending_cap = None
        for bl in b["blocks"]:
            if bl[0] == "chapter":
                pending_cap = bl[1]
            elif bl[0] == "para":
                cls = bl[1] if bl[1] in ("q1", "q2") else ""
                out = []
                if pending_cap is not None:
                    out.append(f"<span class='dc'>{pending_cap}</span>")
                for kind, val in bl[2]:
                    if kind == "v":
                        if pending_cap is not None and val == "1":
                            pending_cap = None
                            continue
                        out.append(f"<sup>{esc(val)}</sup>")
                    elif kind == "t":
                        out.append(esc(val))
                pending_cap = None
                parts.append(f"<p class='{cls}'>{''.join(out)}</p>")
    parts.append("</body></html>")
    weasy("\n".join(parts), os.path.join(OUT, "deu1912-rdropcap.pdf"))


def kor_2col():
    """Korean 1910, 6x9 in two columns with a rule; running head "Book c–Book c"
    (first and last chapter on the page); plain body-size verse digits."""
    books = read_zip(os.path.join(SRC, "kor_usfm.zip"))
    css = """
    @page { size: 6in 9in; margin: 0.75in 0.5in 0.7in 0.5in;
      @top-center { content: string(ch, first) "–" string(ch, last); font: 8pt 'Noto Serif CJK KR'; }
      @bottom-center { content: counter(page); font: 8pt 'Noto Serif CJK KR'; } }
    @page :first { @top-center { content: none } }
    body { font: 9pt/1.45 'Noto Serif CJK KR'; text-align: justify; }
    .book { column-count: 2; column-gap: 0.25in; column-rule: 0.4pt solid #444; }
    h1 { font-size: 15pt; text-align: center; page-break-before: always; margin: 0.2in 0 0.2in; }
    h2 { font-size: 10pt; text-align: center; font-weight: bold; margin: 6pt 0 3pt; string-set: ch attr(data-ch); }
    p { margin: 0 0 2pt; } p.q1 { margin-left: 1.2em; text-indent: -0.8em; } p.q2 { margin-left: 2em; text-indent: -0.8em; }
    .titlepage { text-align: center; font-size: 26pt; margin-top: 2.5in; }
    """
    parts = [f"<html lang='ko'><head><meta charset='utf-8'><style>{css}</style></head><body>",
             "<div class='titlepage'>성경전서</div>"]
    for b in books:
        name = b["h"] or b["title"]
        parts.append(f"<h1>{esc(name)}</h1><div class='book'>")
        psalm = b["code"] == "PSA"
        for bl in b["blocks"]:
            if bl[0] == "chapter":
                label = f"제 {bl[1]} {'편' if psalm else '장'}"
                parts.append(f"<h2 data-ch='{esc(name)} {bl[1]}'>{label}</h2>")
            elif bl[0] == "para":
                cls = bl[1] if bl[1] in ("q1", "q2") else ""
                out = []
                for kind, val in bl[2]:
                    if kind == "v":
                        out.append(f"{esc(val)} ")
                    elif kind == "t":
                        out.append(esc(val))
                parts.append(f"<p class='{cls}'>{''.join(out)}</p>")
        parts.append("</div>")
    parts.append("</body></html>")
    weasy("\n".join(parts), os.path.join(OUT, "kor-r2col.pdf"))


# ---------------------------------------------------------------- LibreOffice
def soffice(src, fmt, outdir, filt=None):
    target = f"{fmt}:{filt}" if filt else fmt
    subprocess.run(["soffice", "--headless", "--convert-to", target, "--outdir", outdir, src],
                   check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=3600)


def two_columns(odt):
    """Patch the ODT's page layouts to two columns with a thin rule."""
    tmp = odt + ".tmp"
    with zipfile.ZipFile(odt) as zin, zipfile.ZipFile(tmp, "w", zipfile.ZIP_DEFLATED) as zout:
        for item in zin.infolist():
            data = zin.read(item.filename)
            if item.filename == "styles.xml":
                s = data.decode("utf-8")
                cols = ('<style:columns fo:column-count="2" fo:column-gap="0.6cm">'
                        '<style:column-sep style:width="0.02cm" style:color="#000000" style:height="100%"/>'
                        '</style:columns>')
                s = re.sub(r"(<style:page-layout-properties[^>]*?)/>", r"\1>" + cols + "</style:page-layout-properties>", s)
                s = re.sub(r"(<style:page-layout-properties[^>]*?[^/])>(?!<style:columns)", r"\1>" + cols, s)
                data = s.encode("utf-8")
            if item.filename == "mimetype":
                zout.writestr(item, data, compress_type=zipfile.ZIP_STORED)
            else:
                zout.writestr(item, data)
    os.replace(tmp, odt)


def brenton_lo2col():
    """Brenton English Septuagint, US Letter, two columns (LibreOffice from
    HTML): a static running head (the edition's title) on every page, page
    number at the foot, page-foot footnotes numbered through the document,
    each opening with its bold c:v; superscript verse numbers with no space
    after them; book titles in capitals ("ESTHER (GREEK)")."""
    books = read_zip(os.path.join(SRC, "eng-Brenton_usfm.zip"))
    head = "THE SEPTUAGINT VERSION OF THE OLD TESTAMENT — ENGLISH TRANSLATION BY SIR L. C. L. BRENTON"
    parts = ["<html><head><meta charset='utf-8'><title>Brenton</title>",
             "<style>p{font-family:'Liberation Serif';font-size:10pt;margin:0 0 3pt;text-align:justify}"
             "h1{font-size:15pt;text-align:center;letter-spacing:2pt} h2{font-size:10pt;text-align:center;font-variant:small-caps}"
             "p.q1{margin-left:1.2em} p.q2{margin-left:2.2em}</style></head><body>",
             "<p align='center' style='font-size:24pt'>THE SEPTUAGINT</p>",
             f"<p>{esc(head)}</p>"]
    n = 0
    notes = []
    for b in books:
        title = b["title"].upper()
        if b["h"] and "(" in b["h"]:
            title = b["h"].upper()
        parts.append(f"<h1>{esc(title)}</h1>")
        psalm = b["code"] == "PSA"
        chap = 0
        for bl in b["blocks"]:
            if bl[0] == "chapter":
                chap = bl[1]
                parts.append(f"<h2>{'Psalm' if psalm else 'Chapter'} {bl[1]}</h2>")
            elif bl[0] == "heading":
                continue
            elif bl[0] == "para":
                cls = bl[1] if bl[1] in ("q1", "q2") else ""
                out = []
                verse = "1"
                for kind, val in bl[2]:
                    if kind == "v":
                        verse = val
                        out.append(f"<sup>{esc(val)}</sup>")
                    elif kind == "t":
                        out.append(esc(val))
                    elif kind == "note":
                        n += 1
                        body = val[2:] if val.startswith("x:") else val
                        if not re.match(r"\d+[:.]\d", body):
                            body = f"{chap}:{verse} {body}"
                        ref, _, rest = body.partition(" ")
                        notes.append(f"<div id='sdfootnote{n}'><p class='sdfootnote'><a class='sdfootnotesym' name='sdfootnote{n}sym' href='#sdfootnote{n}anc'>{n}</a><b>{esc(ref)}</b> {esc(rest)}</p></div>")
                        out.append(f"<a class='sdfootnoteanc' name='sdfootnote{n}anc' href='#sdfootnote{n}sym'><sup>{n}</sup></a>")
                parts.append(f"<p class='{cls}'>{''.join(out)}</p>")
    parts.append("".join(notes))
    parts.append("</body></html>")
    work = tempfile.mkdtemp()
    src = os.path.join(work, "engbrenton-rlo2col.html")
    with open(src, "w", encoding="utf-8") as f:
        f.write("\n".join(parts))
    soffice(src, "odt", work, "writer8")
    odt = os.path.join(work, "engbrenton-rlo2col.odt")
    two_columns(odt)
    add_header_footer(odt, head)
    soffice(odt, "pdf", work, "writer_pdf_Export")
    shutil.copy(os.path.join(work, "engbrenton-rlo2col.pdf"), OUT)


def add_header_footer(odt, head):
    """A static header (the title) and a page-number footer on the default page style."""
    tmp = odt + ".tmp"
    with zipfile.ZipFile(odt) as zin, zipfile.ZipFile(tmp, "w", zipfile.ZIP_DEFLATED) as zout:
        for item in zin.infolist():
            data = zin.read(item.filename)
            if item.filename == "styles.xml":
                s = data.decode("utf-8")
                hf = (f'<style:header><text:p text:style-name="Header">{esc(head)}</text:p></style:header>'
                      '<style:footer><text:p text:style-name="Footer"><text:page-number text:select-page="current"/></text:p></style:footer>')
                s = re.sub(r'(<style:master-page style:name="(?:Standard|HTML)"[^>]*?)/>', r"\1>" + hf + "</style:master-page>", s)
                s = re.sub(r'(<style:master-page style:name="(?:Standard|HTML)"[^>]*[^/]>)', r"\1" + hf, s)
                s = re.sub(r'(<style:page-layout-properties[^>]*>.*?</style:page-layout-properties>)(?!<style:header-style)',
                           r'\1<style:header-style><style:header-footer-properties fo:min-height="0.5cm" fo:margin-bottom="0.3cm"/></style:header-style>'
                           r'<style:footer-style><style:header-footer-properties fo:min-height="0.5cm" fo:margin-top="0.3cm"/></style:footer-style>', s, flags=re.S)
                data = s.encode("utf-8")
            if item.filename == "mimetype":
                zout.writestr(item, data, compress_type=zipfile.ZIP_STORED)
            else:
                zout.writestr(item, data)
    os.replace(tmp, odt)


def spaonbv_lodocx():
    """Spanish ONBV poetical books, Word by LibreOffice from HTML: bold
    superscript verse numbers set against the first word, speaker labels in
    small capitals with a colon, headings as Heading 1-4."""
    books = read_zip(os.path.join(SRC, "spaonbv_usfm.zip"), {"JOB", "PSA", "PRO", "ECC", "SNG"})
    parts = ["<html><head><meta charset='utf-8'><title>Poéticos</title></head><body>",
             "<p align='center' style='font-size:24pt'>Los Libros Poéticos</p>"]
    for b in books:
        parts.append(f"<h1 style='page-break-before:always'>{esc(b['title'])}</h1>")
        psalm = b["code"] == "PSA"
        for bl in b["blocks"]:
            if bl[0] == "chapter":
                parts.append(f"<h2>{'Salmo' if psalm else 'Capítulo'} {bl[1]}</h2>")
            elif bl[0] == "heading":
                if bl[1] == "sp":
                    parts.append(f"<p style='font-variant:small-caps'>{esc(bl[2])}:</p>")
                elif bl[1].startswith("ms"):
                    parts.append(f"<h4 align='center'>{esc(bl[2])}</h4>")
                elif bl[1] in ("s", "s1", "s2"):
                    parts.append(f"<h3><i>{esc(bl[2])}</i></h3>")
                elif bl[1] == "d":
                    parts.append(f"<p><i>{esc(bl[2])}</i></p>")
            elif bl[0] == "para":
                style = {"q1": "margin-left:1.2cm", "q2": "margin-left:2.2cm"}.get(bl[1], "")
                out = []
                for kind, val in bl[2]:
                    if kind == "v":
                        out.append(f"<b><sup>{esc(val)}</sup></b>")
                    elif kind == "t":
                        out.append(esc(val.lstrip()) if out and out[-1].endswith("</sup></b>") else esc(val))
                parts.append(f"<p style='{style}'>{''.join(out)}</p>")
    parts.append("</body></html>")
    work = tempfile.mkdtemp()
    src = os.path.join(work, "spaonbv-rlodocx-poet.html")
    with open(src, "w", encoding="utf-8") as f:
        f.write("\n".join(parts))
    soffice(src, "docx", work, "MS Word 2007 XML")
    shutil.copy(os.path.join(work, "spaonbv-rlodocx-poet.docx"), OUT)


# ---------------------------------------------------------------- EPUB 2
def spapddpt_epub2():
    """Spanish PDDPT, EPUB 2 written directly: bold small inline verse-number
    spans with a CSS gap and no space, notes at each book's end."""
    books = read_zip(os.path.join(SRC, "spapddpt_usfm.zip"))
    path = os.path.join(OUT, "spapddpt-repub2.epub")
    css = ("body{font-family:serif} .v{font-weight:bold;font-size:0.8em;margin-right:0.15em}"
           " p.q1{margin-left:1.5em;text-indent:-1em} p.q2{margin-left:2.5em;text-indent:-1em} h3{font-style:italic}")
    files = []
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr(zipfile.ZipInfo("mimetype"), "application/epub+zip", compress_type=zipfile.ZIP_STORED)
        z.writestr("META-INF/container.xml", "<?xml version='1.0'?><container version='1.0' xmlns='urn:oasis:names:tc:opendocument:xmlns:container'><rootfiles><rootfile full-path='OEBPS/content.opf' media-type='application/oebps-package+xml'/></rootfiles></container>")
        z.writestr("OEBPS/style.css", css)
        for i, b in enumerate(books):
            name = f"b{i:02d}.xhtml"
            files.append((name, b["toc1"] or b["title"]))
            body = [f"<h1>{esc(b['toc1'] or b['title'])}</h1>"]
            notes = []
            psalm = b["code"] == "PSA"
            chap = 0
            for bl in b["blocks"]:
                if bl[0] == "chapter":
                    chap = bl[1]
                    body.append(f"<h2>{'Salmo' if psalm else 'Capítulo'} {bl[1]}</h2>")
                elif bl[0] == "heading" and bl[1] in ("s", "s1", "s2"):
                    body.append(f"<h3>{esc(bl[2])}</h3>")
                elif bl[0] == "para":
                    cls = bl[1] if bl[1] in ("q1", "q2") else "p"
                    out = []
                    verse = "1"
                    for kind, val in bl[2]:
                        if kind == "v":
                            verse = val
                            out.append(f"<span class='v'>{esc(val)}</span>")
                        elif kind == "t":
                            out.append(esc(val.lstrip()) if out and out[-1].endswith("</span>") else esc(val))
                        elif kind == "note" and not val.startswith("x:"):
                            k = len(notes) + 1
                            notes.append(f"<p id='fn{k}'><a href='#r{k}'>{k}</a> <b>{chap}:{verse}</b> {esc(val)}</p>")
                            out.append(f"<sup><a id='r{k}' href='#fn{k}'>{k}</a></sup>")
                    body.append(f"<p class='{cls}'>{''.join(out)}</p>")
            if notes:
                body.append("<h2>Notas</h2>" + "".join(notes))
            z.writestr("OEBPS/" + name, "<?xml version='1.0' encoding='utf-8'?><!DOCTYPE html PUBLIC '-//W3C//DTD XHTML 1.1//EN' 'http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd'><html xmlns='http://www.w3.org/1999/xhtml' xml:lang='es'><head><title>x</title><link rel='stylesheet' href='style.css' type='text/css'/></head><body>" + "\n".join(body) + "</body></html>")
        manifest = "".join(f"<item id='i{i}' href='{n}' media-type='application/xhtml+xml'/>" for i, (n, _) in enumerate(files))
        spine = "".join(f"<itemref idref='i{i}'/>" for i in range(len(files)))
        z.writestr("OEBPS/content.opf", "<?xml version='1.0'?><package xmlns='http://www.idpf.org/2007/opf' version='2.0' unique-identifier='id'><metadata xmlns:dc='http://purl.org/dc/elements/1.1/'><dc:title>La Biblia</dc:title><dc:language>es</dc:language><dc:identifier id='id'>recon-spapddpt</dc:identifier></metadata><manifest><item id='css' href='style.css' media-type='text/css'/><item id='ncx' href='toc.ncx' media-type='application/x-dtbncx+xml'/>" + manifest + "</manifest><spine toc='ncx'>" + spine + "</spine></package>")
        nav = "".join(f"<navPoint id='n{i}' playOrder='{i+1}'><navLabel><text>{esc(t)}</text></navLabel><content src='{n}'/></navPoint>" for i, (n, t) in enumerate(files))
        z.writestr("OEBPS/toc.ncx", "<?xml version='1.0'?><ncx xmlns='http://www.daisy.org/z3986/2005/ncx/' version='2005-1'><head/><docTitle><text>La Biblia</text></docTitle><navMap>" + nav + "</navMap></ncx>")


# ---------------------------------------------------------------- text
def grcbrent_txt():
    """Greek Septuagint, one verse per line "BOOKNAME c:v text" with the
    uppercase \\h names; psalm titles that are not verses on lines of their own."""
    books = read_zip(os.path.join(SRC, "grcbrent_usfm.zip"))
    lines = []
    for b in books:
        name = (b["h"] or b["title"]).upper()
        chap = 0
        cur = None
        for bl in b["blocks"]:
            if bl[0] == "chapter":
                chap = bl[1]
            elif bl[0] == "heading" and bl[1] == "d":
                if cur:
                    lines.append(cur)
                    cur = None
                lines.append(bl[2])
            elif bl[0] == "para":
                for kind, val in bl[2]:
                    if kind == "v":
                        if cur:
                            lines.append(cur)
                        cur = f"{name} {chap}:{val} "
                    elif kind == "t" and cur is not None:
                        cur += val
        if cur:
            lines.append(cur)
        lines.append("")
    with open(os.path.join(OUT, "grcbrent-rtxtref.txt"), "w", encoding="utf-8") as f:
        f.write("\n".join(re.sub(r"\s+", " ", l).strip() for l in lines) + "\n")


def portft_md():
    """Portuguese TFT NT, Markdown: "**c:v** text" per line, the edition's
    \\s1 lines (passage references and "TEMA: ..." theme lines) in italics."""
    books = read_zip(os.path.join(SRC, "portft_usfm.zip"))
    out = ["# A Bíblia Sagrada, Tradução para Tradutores", "", "*Domínio público.*", ""]
    for b in books:
        out += [f"## {b['toc1'] or b['title']}", ""]
        chap = 0
        cur = None
        for bl in b["blocks"]:
            if bl[0] == "chapter":
                chap = bl[1]
            elif bl[0] == "heading" and bl[1] in ("s", "s1", "s2", "r"):
                if cur:
                    out += [cur.strip(), ""]
                    cur = None
                out += [f"*{bl[2]}*", ""]
            elif bl[0] == "para":
                for kind, val in bl[2]:
                    if kind == "v":
                        if cur:
                            out += [cur.strip(), ""]
                        cur = f"**{chap}:{val}** "
                    elif kind == "t" and cur is not None:
                        cur += val
        if cur:
            out += [cur.strip(), ""]
    with open(os.path.join(OUT, "portft-rmdcvbold.md"), "w", encoding="utf-8") as f:
        f.write("\n".join(re.sub(r"[ \t]+", " ", l) for l in out))


JOBS = {
    "grcbrent-rtxtref": grcbrent_txt,
    "portft-rmdcvbold": portft_md,
    "spapddpt-repub2": spapddpt_epub2,
    "spaonbv-rlodocx-poet": spaonbv_lodocx,
    "engbrenton-rlo2col": brenton_lo2col,
    "deu1912-rdropcap": deu_dropcap,
    "kor-r2col": kor_2col,
}

for key, job in JOBS.items():
    if ONLY and key not in ONLY:
        continue
    print("building", key, flush=True)
    job()
    print("built", key, flush=True)
