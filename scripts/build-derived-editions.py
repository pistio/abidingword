#!/usr/bin/env python3
"""Builds the derived test documents of the real-edition corpus (v0.24.2).

eBible.org publishes each edition as HTML (one page per chapter). This script
puts an edition's chapters together, in canonical order, as one HTML document
with the edition's own book titles, chapter labels, verse numbers and notes —
the words are the publisher's, untouched — and then has LibreOffice write it
as a Word document and/or a PDF. Those files have a layout produced by a
different program from eBible's own PDFs (paragraphs with verse numbers
inside them, Writer's page layout), so the recogniser is tested on more than
one producer. They are labelled "converted" in the corpus manifest.

A Markdown variant is written by this script itself: "# Book", "## Chapter n"
and paragraphs with the verse numbers inline.

    build-derived-editions.py <folder> <id> docx,pdf,md [BOOK,BOOK…]

<folder> must hold <id>_html.zip (fetched by scripts/fetch-real-editions.sh).
The outputs are <id>_lo.docx, <id>_lo.pdf and <id>.md (or with a _<BOOKS>
suffix when only some books are wanted). Nothing here is committed to Git.
"""
import html
import os
import re
import subprocess
import sys
import tempfile
import zipfile

ORDER = (
    "GEN EXO LEV NUM DEU JOS JDG RUT 1SA 2SA 1KI 2KI 1CH 2CH EZR NEH EST JOB PSA "
    "PRO ECC SNG ISA JER LAM EZK DAN HOS JOL AMO OBA JON MIC NAM HAB ZEP HAG ZEC "
    "MAL MAT MRK LUK JHN ACT ROM 1CO 2CO GAL EPH PHP COL 1TH 2TH 1TI 2TI TIT PHM "
    "HEB JAS 1PE 2PE 1JN 2JN 3JN JUD REV"
).split()


def book_title(z, code):
    try:
        page = z.read(f"{code}.htm").decode("utf-8-sig")
    except KeyError:
        return None
    titles = re.findall(r"<h1><a href='index.htm'>(.*?)</a></h1>", page)
    return html.unescape(titles[0]) if titles else code


def chapter_body(page):
    """The chapter's text: from the main division to the notes, with the
    navigation lists and copyright taken out."""
    start = page.find('<div class="main">')
    end = page.find('<div class="copyright">')
    body = page[start:end] if start >= 0 else ""
    body = re.sub(r"<ul class='tnav'>.*?</ul>", "", body, flags=re.S)
    # A note's text repeated in a pop-up inside its caller, hidden on screen
    # by eBible's style sheet: not part of the page as read (the note itself
    # is printed at the chapter's end).
    body = re.sub(r'<span class="popup">.*?</span>', "", body, flags=re.S)
    return body


def chapters(z, code):
    names = sorted(
        (n for n in z.namelist() if re.fullmatch(rf"{code}\d+\.htm", n)),
        key=lambda n: int(n[len(code):-4]),
    )
    for n in names:
        yield int(n[len(code):-4]), z.read(n).decode("utf-8-sig")


def plain(fragment):
    text = re.sub(r"<[^>]+>", "", fragment)
    return re.sub(r"\s+", " ", html.unescape(text)).strip()


def main():
    folder, ident, formats = sys.argv[1], sys.argv[2], sys.argv[3].split(",")
    books = sys.argv[4].split(",") if len(sys.argv) > 4 else ORDER
    suffix = "" if books is ORDER else "_" + "-".join(books)
    z = zipfile.ZipFile(os.path.join(folder, f"{ident}_html.zip"))
    lang = re.search(r'<html lang="([^"]+)"', z.read("index.htm").decode("utf-8-sig"))
    lang = lang.group(1) if lang else "en"
    parts = [
        f'<!DOCTYPE html><html lang="{lang}"><head><meta charset="UTF-8"/>'
        "<style>.chapterlabel{font-size:150%;font-weight:bold;margin-top:1em}"
        ".verse{vertical-align:super;font-size:70%}"
        "h1{page-break-before:always;text-align:center}</style></head><body>"
    ]
    md = []
    for code in books:
        title = book_title(z, code)
        if title is None:
            continue
        parts.append(f"<h1>{html.escape(title)}</h1>")
        md.append(f"# {title}\n")
        for number, page in chapters(z, code):
            body = chapter_body(page)
            parts.append(body)
            md.append(f"## {number}\n")
            for p in re.findall(r"<div class='(?:p|q\d?|m|pi\d?|nb)'>(.*?)</div>", body, re.S):
                # Verse numbers as plain numbers in the paragraph.
                p = re.sub(r'<span class="verse" id="V\d+">(\d+)&#160;</span>', r" \1 ", p)
                text = plain(p)
                if text:
                    md.append(text + "\n")
    parts.append("</body></html>")
    with tempfile.TemporaryDirectory() as tmp:
        src = os.path.join(tmp, f"{ident}{suffix}.html")
        with open(src, "w", encoding="utf-8") as f:
            f.write("".join(parts))
        for fmt in formats:
            if fmt == "md":
                with open(os.path.join(folder, f"{ident}{suffix}.md"), "w", encoding="utf-8") as f:
                    f.write("\n".join(md))
                continue
            filt = {"docx": "docx:MS Word 2007 XML", "pdf": "pdf:writer_pdf_Export"}[fmt]
            subprocess.run(
                ["soffice", "--headless", "--infilter=HTML (StarWriter)", "--convert-to", filt, "--outdir", tmp, src],
                check=True,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                timeout=3600,
            )
            os.replace(
                os.path.join(tmp, f"{ident}{suffix}.{fmt}"),
                os.path.join(folder, f"{ident}{suffix}_lo.{fmt}"),
            )


if __name__ == "__main__":
    main()
