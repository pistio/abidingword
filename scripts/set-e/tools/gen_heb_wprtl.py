"""heb (Hebrew Bible, eBible 'heb') Torah -> right-to-left two-column PDF via WeasyPrint.
Hebrew-letter chapter and verse numbers (gematria, 15=טו 16=טז), 'פרק' headings, running heads with the book
name (string-set), Hebrew page numbering at the bottom centre, title page and Hebrew table of contents."""
import sys, html
sys.path.insert(0, '/root/aw-set-e/tools')
import usfm
from weasyprint import HTML, CSS

SRC, OUT = sys.argv[1], sys.argv[2]
E = lambda t: html.escape(t, quote=False)
books = usfm.load(SRC)
ONES = ['', 'א', 'ב', 'ג', 'ד', 'ה', 'ו', 'ז', 'ח', 'ט']
TENS = ['', 'י', 'כ', 'ל', 'מ', 'נ', 'ס', 'ע', 'פ', 'צ']
HUND = ['', 'ק', 'ר', 'ש', 'ת']


def heb(n):
    s = ''
    while n >= 400:
        s += 'ת'; n -= 400
    s += HUND[n // 100]; n %= 100
    if n == 15:
        s += 'טו'
    elif n == 16:
        s += 'טז'
    else:
        s += TENS[n // 10] + ONES[n % 10]
    return s


def heb_label(n):
    s = heb(n)
    return s[:-1] + '״' + s[-1] if len(s) > 1 else s + '׳'


CSSTXT = """
@page { size: 170mm 240mm; margin: 18mm 15mm 16mm 15mm;
  @top-center { content: string(bk); font: 9pt "Noto Serif Hebrew"; direction: rtl; }
  @bottom-center { content: counter(page, hebrew); font: 9pt "Noto Serif Hebrew"; } }
@page title { @top-center { content: none } @bottom-center { content: none } }
html { direction: rtl; }
body { font: 11pt/1.55 "Noto Serif Hebrew", serif; text-align: justify; }
.title { page: title; text-align: center; padding-top: 55mm; break-after: page; }
.title h1 { font-size: 34pt; margin: 0 }
.title p { font-size: 12pt }
.toc { break-after: page; } .toc h2 { text-align: center } .toc li { list-style: none; font-size: 13pt; margin: 2mm 0 }
.toc a::after { content: leader('.') target-counter(attr(href), page, hebrew); }
.book { columns: 2; column-gap: 7mm; column-rule: 0.5pt solid #777; break-before: page; }
.book h1 { column-span: all; text-align: center; font-size: 22pt; margin: 0 0 5mm; string-set: bk content(); }
h2 { font-size: 12pt; font-weight: bold; margin: 3mm 0 1mm; break-after: avoid; text-align: right; }
p { margin: 0 0 1.5mm; text-indent: 0; }
span.v { font-weight: bold; font-size: 8pt; color: #5a3a1a; }
"""
CODES = ['GEN', 'EXO', 'LEV', 'NUM', 'DEU']
seq = [books[c] for c in CODES]
h = ['<html lang="he" dir="rtl"><head><meta charset="utf-8"><title>תורה — חמשה חומשי תורה</title></head><body>',
     '<div class="title"><h1>חמשה חומשי תורה</h1><p>בראשית · שמות · ויקרא · במדבר · דברים</p>'
     '<p dir="ltr" style="font-size:8pt;margin-top:70mm;font-family:Liberation Serif">The Holy Bible in Modern Hebrew. Public Domain. '
     'Source: eBible.org (heb).</p></div>',
     '<div class="toc"><h2>תוכן העניינים</h2><ul>']
for bk in seq:
    h.append('<li><a href="#%s">%s</a></li>' % (bk.code, E(bk.long_name.strip())))
h.append('</ul></div>')
for bk in seq:
    h.append('<section class="book"><h1 id="%s">%s</h1>' % (bk.code, E(bk.long_name.strip())))
    for b in bk.blocks:
        if b[0] == 'c':
            h.append('<h2>פרק %s</h2>' % heb_label(int(b[1])))
        elif b[0] == 'p':
            out = []
            for it in b[2]:
                if it[0] == 'v':
                    try:
                        lab = heb(int(it[1]))
                    except ValueError:
                        lab = it[1]
                    out.append('<span class="v">%s</span> ' % lab)
                elif it[0] == 't':
                    out.append(E(it[1]))
            h.append('<p>%s</p>' % ''.join(out).strip())
    h.append('</section>')
h.append('</body></html>')
HTML(string='\n'.join(h)).write_pdf(OUT, stylesheets=[CSS(string=CSSTXT)])
print('ok')
