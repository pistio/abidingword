"""engtnt (Tyndale NT 1534) -> A4 three-column PDF via WeasyPrint (HTML/CSS paged media).
Drop-cap chapter numerals, bold inline verse numbers, running heads via string-set, page numbers bottom outside."""
import sys, html
sys.path.insert(0, '/root/aw-set-e/tools')
import usfm
from weasyprint import HTML

SRC, OUT = sys.argv[1], sys.argv[2]
E = lambda t: html.escape(t, quote=False)
books = usfm.load(SRC)
CSS = """
@page { size: A4; margin: 20mm 14mm 18mm 14mm;
  @top-left { content: string(bookname); font: italic 8pt "Liberation Serif"; }
  @top-right { content: "Chapter " string(chapno); font: 8pt "Liberation Serif"; }
  @bottom-right { content: counter(page); font: 8pt "Liberation Serif"; } }
@page :left { @bottom-right { content: none } @bottom-left { content: counter(page); font: 8pt "Liberation Serif"; } }
@page front { @top-left { content: none } @top-right { content: none } @bottom-right { content: none } @bottom-left { content: none } }
body { font: 8.6pt/1.28 "Liberation Serif", serif; text-align: justify; hyphens: auto; }
.front { page: front; text-align: center; padding-top: 70mm; break-after: page; }
.front h1 { font-size: 30pt; letter-spacing: 1pt; margin: 0 0 6mm }
.front p { font-size: 12pt; margin: 2mm }
.book { columns: 3; column-gap: 5mm; column-rule: 0.4pt solid #888; break-before: page; }
.book h1 { column-span: all; text-align: center; font: bold 15pt "Liberation Serif"; text-transform: uppercase;
  letter-spacing: 1.5pt; border-bottom: 0.6pt solid #000; padding-bottom: 2mm; margin: 0 0 4mm; string-set: bookname attr(data-h); }
p { margin: 0; text-indent: 1em; }
p.cs { text-indent: 0; }
span.chap { float: left; font: bold 25pt/0.9 "Liberation Serif"; margin: 1pt 3pt 0 0; string-set: chapno content(); }
b.v { font-size: 7pt; }
"""
parts = ['<html><head><meta charset="utf-8"><title>The New Testament (Tyndale 1534)</title>'
         '<meta name="author" content="William Tyndale"><meta name="generator" content="WeasyPrint"></head><body>',
         '<div class="front"><h1>The Newe Testament</h1><p>Translated by William Tyndale, 1534</p>'
         '<p style="font-size:9pt;margin-top:60mm">The Tyndale New Testament (1534). Public Domain. Text from eBible.org (engtnt).</p></div>']
for bk in usfm.ordered(books, usfm.NT):
    h = bk.meta.get('h', bk.name).strip()
    parts.append('<section class="book"><h1 data-h="%s">%s</h1>' % (E(h), E(bk.long_name)))
    pend = None
    for b in bk.blocks:
        if b[0] == 'c':
            pend = b[1]
            continue
        if b[0] != 'p':
            continue
        out = []
        first_v = True
        for it in b[2]:
            if it[0] == 'v':
                if pend and first_v and it[1] == '1':
                    first_v = False
                    continue  # verse 1 number is replaced by the chapter numeral
                first_v = False
                out.append('<b class="v">%s</b> ' % E(it[1]))
            elif it[0] == 't':
                out.append(E(it[1]))
        if pend:
            parts.append('<p class="cs"><span class="chap">%s</span>%s</p>' % (E(pend), ''.join(out)))
            pend = None
        else:
            parts.append('<p>%s</p>' % ''.join(out))
    parts.append('</section>')
parts.append('</body></html>')
HTML(string='\n'.join(parts)).write_pdf(OUT, stylesheets=[__import__('weasyprint').CSS(string=CSS)])
print('ok')
