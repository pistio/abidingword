"""G14: render Castellio's Latin Bible (Genesis 1-27, Matthew 1-13, Mark 1-5, Luke 1-13,
John 1-4, as transcribed by the project at commit 0f93476) as an EPUB 3 built with the
Python library EbookLib (XHTML written by this script).
Layout: one XHTML file per book with the project's long title; chapter headings
"CAPUT XII" (Roman numerals); verse numbers in square brackets "[12]" in small grey type
(verse 1 included); each chapter one paragraph; a navigation document and NCX; cover-less;
licence page (CC BY-SA 4.0, attribution to the project).
Usage: python3 -I render_g14_castellio_ebooklib.py REF.zip OUT.epub
"""
import html
import re
import sys

sys.path.insert(0, "/root/aw-set-g/tools")
import usfmlib as U  # noqa: E402
from ebooklib import epub  # noqa: E402


def roman(n):
    vals = [(50, "L"), (40, "XL"), (10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I")]
    out = ""
    for v, s in vals:
        while n >= v:
            out += s
            n -= v
    return out


CSS = """body { font-family: serif; }
h1 { text-align: center; font-size: 1.4em; margin: 1.2em 0; }
h2 { text-align: center; font-size: 0.95em; letter-spacing: 0.15em; margin: 1.2em 0 0.5em; }
p { text-align: justify; text-indent: 0; line-height: 1.5; }
span.vn { font-size: 0.7em; color: #777777; }
"""


def main(ref, out):
    books = U.read_usfm_zip(ref)
    bk = epub.EpubBook()
    bk.set_identifier("aw-set-g-castellio-0f93476")
    bk.set_title("Biblia Sacra ex Sebastiani Castellionis interpretatione (excerpta)")
    bk.set_language("la")
    bk.add_author("Sebastiānus Castelliō")
    bk.add_metadata("DC", "rights", "CC BY-SA 4.0 — transcription by the biblia-castellionis project (github.com/njand/biblia-castellionis)")
    css = epub.EpubItem(uid="css", file_name="style/main.css", media_type="text/css", content=CSS)
    bk.add_item(css)
    lic = epub.EpubHtml(title="Dē hāc ēditiōne", file_name="licentia.xhtml", lang="la")
    lic.content = ("<h1>Biblia Sacra</h1><p>ex Sebastiānī Castelliōnis interpretātiōne. Transcription with modern "
                   "orthography and macrons by the biblia-castellionis project (github.com/njand/biblia-castellionis, "
                   "commit 0f93476, 2 April 2023), licensed under Creative Commons Attribution-ShareAlike 4.0. "
                   "This EPUB is an excerpt in a new layout.</p>")
    lic.add_item(css)
    bk.add_item(lic)
    chapters = [lic]
    for b in books:
        title = b["headers"].get("mt1") or b["headers"].get("mt")
        c = epub.EpubHtml(title=b["headers"].get("h", title), file_name=f"{b['code'].lower()}.xhtml", lang="la")
        h = [f"<h1>{html.escape(title)}</h1>"]
        open_p = False
        for it in b["items"]:
            if it[0] == "c":
                if open_p:
                    h.append("</p>")
                h.append(f"<h2>CAPUT {roman(it[1])}</h2><p>")
                open_p = True
            elif it[0] == "v":
                txt = html.escape(re.sub(r"\x01\d+\x01", "", it[2]))
                h.append(f"<span class=\"vn\">[{it[1]}]</span> {txt} ")
        if open_p:
            h.append("</p>")
        c.content = "".join(h)
        c.add_item(css)
        bk.add_item(c)
        chapters.append(c)
    bk.toc = tuple(chapters)
    bk.add_item(epub.EpubNcx())
    bk.add_item(epub.EpubNav())
    bk.spine = ["nav"] + chapters
    epub.write_epub(out, bk)
    print("written", out)


if __name__ == "__main__":
    main(*sys.argv[1:3])
