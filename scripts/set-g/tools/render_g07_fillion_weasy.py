"""G07: render the Fillion (1904) deuterocanonical/Greek-additions group from the reference
USFM as HTML+CSS and print it to PDF with WeasyPrint.
Layout: 6x9 in pages, single column of justified text, chapter numbers as large floated
drop numerals (no "chapitre" word), verse 1 unnumbered, other verse numbers as small
superscripts run into the paragraph, running heads with the book name (left pages) and
"Livre — chapitre" (right pages), page numbers in the outer corner, a contents page with
page references, a half-title for the group.
Usage: python3 -I render_g07_vigouroux_weasy.py REF.zip OUT.pdf WORKDIR
"""
import html
import os
import re
import sys

sys.path.insert(0, "/root/aw-set-g/tools")
import usfmlib as U  # noqa: E402
from weasyprint import HTML  # noqa: E402

NAMES = {"TOB": "Tobie", "JDT": "Judith", "EST": "Esther", "WIS": "La Sagesse",
         "SIR": "L’Ecclésiastique", "BAR": "Baruch", "DAN": "Daniel", "1MA": "I Machabées",
         "2MA": "II Machabées"}
FULL = {"TOB": "Livre de Tobie", "JDT": "Livre de Judith", "EST": "Livre d’Esther",
        "WIS": "Livre de la Sagesse", "SIR": "Livre de l’Ecclésiastique", "BAR": "Prophétie de Baruch",
        "DAN": "Prophétie de Daniel", "1MA": "Premier livre des Machabées",
        "2MA": "Second livre des Machabées"}
CSS = """
@page { size: 6in 9in; margin: 0.75in 0.6in 0.7in 0.7in;
  @top-left { content: string(bookname); font: italic 8.5pt 'DejaVu Serif'; }
  @top-right { content: string(chapref); font: 8.5pt 'DejaVu Serif'; }
  @bottom-left { content: none; } }
@page :left { @top-left { content: counter(page); font: bold 9pt 'DejaVu Serif'; }
              @top-right { content: string(bookname); font: italic 8.5pt 'DejaVu Serif'; } }
@page :right { @top-left { content: string(chapref); font: 8.5pt 'DejaVu Serif'; }
               @top-right { content: counter(page); font: bold 9pt 'DejaVu Serif'; } }
@page :first { @top-left { content: none; } @top-right { content: none; } }
@page front { @top-left { content: none; } @top-right { content: none; } }
body { font-family: 'DejaVu Serif'; font-size: 9.6pt; line-height: 1.32; hyphens: auto; }
.front { page: front; text-align: center; }
.front h1 { font-size: 22pt; margin-top: 2.2in; letter-spacing: 0.06em; }
.front p { font-size: 10pt; }
.toc { page: front; break-before: page; }
.toc h2 { text-align: center; font-size: 13pt; letter-spacing: 0.2em; }
.toc a { color: black; text-decoration: none; }
.toc li { list-style: none; margin: 0.25em 0; }
.toc a::after { content: leader('.') target-counter(attr(href), page); }
h2.book { break-before: page; string-set: bookname content(); text-align: center;
  font-size: 15pt; font-variant: small-caps; letter-spacing: 0.05em; margin: 0.4in 0 0.25in; }
p.chap { text-align: justify; margin: 0 0 0.45em; text-indent: 0; }
span.cn { float: left; font-size: 27pt; line-height: 0.9; font-weight: bold; margin: 0.04em 0.12em 0 0;
  string-set: chapref attr(data-ref); }
sup.v { font-size: 6.5pt; font-weight: bold; vertical-align: 0.45em; margin-right: 0.08em; }
"""


def main(ref, out_pdf, work):
    os.makedirs(work, exist_ok=True)
    books = U.read_usfm_zip(ref)
    h = ["<html lang='fr'><head><meta charset='utf-8'><title>La Sainte Bible — livres deutérocanoniques"
         "</title><style>" + CSS + "</style></head><body>"]
    h.append("<div class='front'><h1>LA SAINTE BIBLE</h1><p>Livres deutérocanoniques et parties grecques</p>"
             "<p>Traduction de L.-Cl. Fillion, prêtre de Saint-Sulpice</p>"
             "<p style='margin-top:2in;font-size:8pt'>Texte du domaine public (édition Zefania XML).<br>"
             "Mise en page : WeasyPrint.</p></div>")
    h.append("<div class='toc'><h2>TABLE</h2><ul>")
    for b in books:
        h.append(f"<li><a href='#b{b['code']}'>{html.escape(FULL[b['code']])}</a></li>")
    h.append("</ul></div>")
    for b in books:
        code = b["code"]
        h.append(f"<h2 class='book' id='b{code}'>{html.escape(FULL[code])}</h2>")
        open_p = False
        ch = 0
        for it in b["items"]:
            if it[0] == "c":
                if open_p:
                    h.append("</p>")
                ch = it[1]
                h.append(f"<p class='chap'><span class='cn' data-ref='{html.escape(NAMES[code])}, {ch}'>{ch}</span>")
                open_p = True
            elif it[0] == "v":
                txt = html.escape(re.sub(r"\x01\d+\x01", "", it[2]))
                if str(it[1]) == "1":
                    h.append(txt + " ")
                else:
                    h.append(f"<sup class='v'>{it[1]}</sup>{txt} ")
            elif it[0] == "s":
                raise SystemExit("unexpected heading")
        if open_p:
            h.append("</p>")
    h.append("</body></html>")
    src = os.path.join(work, "g07.html")
    open(src, "w", encoding="utf-8").write("".join(h))
    HTML(filename=src).write_pdf(out_pdf)
    print("written", out_pdf)


if __name__ == "__main__":
    main(*sys.argv[1:4])
