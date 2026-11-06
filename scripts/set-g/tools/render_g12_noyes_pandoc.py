"""G12: render George R. Noyes' translation (eBible engnoy; Job-Song of Songs and the four
Gospels) as EPUB 3 with pandoc. This script writes an HTML source with its own layout
choices; pandoc (3.1.3) converts it (-f html -t epub3 --toc --split-level=1).
Layout: one EPUB chapter per book; book title as <h1>; chapter headings "Chapter 4"
(Psalms: "Psalm 23") as <h2>; superscript verse numbers (verse 1 unnumbered, the chapter
heading stands for it); paragraphs as in the USFM (\\p); small stylesheet with indented
paragraphs; title page and copyright statement.
Usage: python3 -I render_g12_noyes_pandoc.py REF.zip OUT.epub WORKDIR
"""
import html
import os
import re
import subprocess
import sys

sys.path.insert(0, "/root/aw-set-g/tools")
import usfmlib as U  # noqa: E402

CSS = """body { font-family: serif; line-height: 1.45; }
h1 { text-align: center; font-variant: small-caps; margin: 1.5em 0 1em; }
h2 { font-size: 1em; text-align: center; letter-spacing: 0.1em; text-transform: uppercase; margin: 1.4em 0 0.4em; }
p { text-indent: 1.2em; margin: 0; text-align: justify; }
sup.v { font-size: 0.65em; color: #6b4b2a; padding-right: 0.15em; }
"""


def main(ref, out_epub, work):
    os.makedirs(work, exist_ok=True)
    books = U.read_usfm_zip(ref)
    h = ["<!DOCTYPE html><html lang='en'><head><meta charset='utf-8'><title>The Poetical Books and the Gospels</title></head><body>"]
    for b in books:
        title = b["headers"].get("toc1") or b["headers"].get("mt1") or b["headers"].get("h")
        h.append(f"<h1>{html.escape(title)}</h1>")
        ch = 0
        open_p = False
        for it in b["items"]:
            if it[0] == "c":
                if open_p:
                    h.append("</p>")
                    open_p = False
                ch = it[1]
                word = "Psalm" if b["code"] == "PSA" else "Chapter"
                h.append(f"<h2>{word} {ch}</h2>")
            elif it[0] == "p":
                if open_p:
                    h.append("</p>")
                h.append("<p>")
                open_p = True
            elif it[0] == "v":
                if not open_p:
                    h.append("<p>")
                    open_p = True
                txt = html.escape(re.sub(r"\x01\d+\x01", "", it[2]))
                if str(it[1]) == "1":
                    h.append(txt + " ")
                else:
                    h.append(f"<sup class='v'>{it[1]}</sup>{txt} ")
        if open_p:
            h.append("</p>")
    h.append("</body></html>")
    src = os.path.join(work, "g12.html")
    open(src, "w", encoding="utf-8").write("".join(h))
    css = os.path.join(work, "g12.css")
    open(css, "w", encoding="utf-8").write(CSS)
    meta = os.path.join(work, "meta.yaml")
    open(meta, "w", encoding="utf-8").write(
        "---\ntitle: 'The Poetical Books and the Four Gospels'\nsubtitle: 'Translated by George R. Noyes'\n"
        "creator:\n- role: translator\n  text: George R. Noyes\nlang: en\n"
        "rights: 'Public domain. Text: eBible.org engnoy (The Holy Bible, translated by George Noyes, 1869).'\n---\n")
    subprocess.run(["pandoc", "-f", "html", "-t", "epub3", "--toc", "--split-level=1", "--css", css,
                    "--metadata-file", meta, "-o", out_epub, src], check=True)
    print("written", out_epub)


if __name__ == "__main__":
    main(*sys.argv[1:4])
