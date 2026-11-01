#!/usr/bin/env python3
"""F13: Diodati 1885 (eBible ita1885) - the poetical books Giobbe-Cantico.
Generator: WeasyPrint. 140x210 mm page, single column; the edition's cross
references as real page footnotes (CSS float: footnote) called by letters that
restart on every page; the source's own chapter labels (\\cl 'SALMO I'),
section headings, psalm titles (\\d), Selah marks (\\qs) set off in italics, the
translators' added words (\\add) in italics, poetry lines with hanging indent;
running heads from string-set (book left, first chapter label right), page
numbers in the outer bottom corner, contents with target-counter."""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import weasyprint
from common import load_zip, clean_items, esc, DOCS, short_name, book_title, ws

REF = os.path.join(DOCS, 'F13-ita-diodati.usfm.zip')
OUT = os.path.join(DOCS, 'F13-ita-diodati.pdf')

CSS = """
@page { size: 140mm 210mm; margin: 17mm 13mm 16mm 15mm;
  @footnote { border-top: 0.4pt solid black; padding-top: 2pt; margin-top: 4pt; }
  @top-left { content: string(book); font-size: 7.5pt; font-style: italic; }
  @top-right { content: string(chap, first); font-size: 7.5pt; } }
@page :left { @bottom-left { content: counter(page); font-size: 8pt; } }
@page :right { @bottom-right { content: counter(page); font-size: 8pt; } }
@page :first { @top-left { content: none; } @top-right { content: none; } @bottom-right { content: none; } }
html { font-family: 'Liberation Serif', serif; font-size: 9.6pt; line-height: 1.32; hyphens: auto; }
h1 { font-size: 15pt; text-align: center; margin: 0 0 1pt 0; string-set: book attr(data-short); break-before: right; }
p.sub { text-align: center; font-size: 8pt; margin: 0 0 8pt 0; letter-spacing: 0.5pt; }
h2.cl { counter-reset: footnote; font-size: 9.5pt; text-align: center; letter-spacing: 1.2pt; margin: 9pt 0 2pt 0; string-set: chap content(); break-after: avoid; }
h3.s { font-size: 8.6pt; font-weight: bold; text-align: center; margin: 3pt 0 3pt 0; break-after: avoid; }
p.r { font-size: 7.6pt; font-style: italic; text-align: center; margin: 0 0 3pt 0; }
p.d { font-style: italic; font-size: 8.6pt; text-align: center; margin: 2pt 0 3pt 0; }
p.p { margin: 0; text-indent: 1em; text-align: justify; }
p.q1 { margin: 0 0 0 2.2em; text-indent: -1.4em; }
p.q2 { margin: 0 0 0 3.0em; text-indent: -1.4em; }
p.b { height: 4pt; margin: 0; }
b.vn { font-size: 7pt; }
i.add { font-style: italic; color: #333; }
span.qs { font-style: italic; font-size: 8.4pt; padding-left: 1.2em; }
span.fn { float: footnote; font-size: 7.1pt; line-height: 1.2; font-style: normal; font-weight: normal; }
span.fn::footnote-call { content: counter(footnote, lower-alpha); vertical-align: super; font-size: 5.8pt; line-height: 0; color: #444; }
span.fn::footnote-marker { content: counter(footnote, lower-alpha) ") "; font-weight: bold; }
div.title { text-align: center; padding-top: 55mm; }
div.toc { break-before: page; }
div.toc a { display: block; text-decoration: none; color: black; margin: 2pt 0; }
div.toc a::after { content: leader('.') target-counter(attr(href), page); }
"""


def render_items(items):
    out = []
    for it in clean_items(items):
        if it[0] == 'v':
            out.append('<b class="vn">%s</b> ' % it[1])
        elif it[0] == 't':
            t = esc(it[1])
            if 'qs' in it[2]:
                t = '<span class="qs">%s</span>' % t
            elif 'add' in it[2]:
                t = '<i class="add">%s</i>' % t
            elif 'nd' in it[2]:
                t = '<span style="font-variant:small-caps">%s</span>' % t
            out.append(t)
        elif it[0] == 'n':
            parts = it[3]
            ref = ''.join(t for k, t in parts if k in ('xo', 'fr'))
            body = ws(' '.join(t for k, t in parts if k not in ('xo', 'fr'))).strip()
            out.append('<span class="fn"><b>%s</b> %s</span>' % (esc(ref), esc(body)))
    return ''.join(out)


def main():
    books = load_zip(REF)
    h = ['<html lang="it"><head><meta charset="utf-8"><title>La Sacra Bibbia - Libri poetici (Diodati 1885)</title>'
         '<meta name="author" content="Giovanni Diodati"></head><body>']
    h.append('<div class="title"><p style="font-size:22pt;margin:0">LA SACRA BIBBIA</p>'
             '<p style="font-size:11pt">tradotta in lingua italiana da Giovanni Diodati</p>'
             '<p style="font-size:11pt;margin-top:14mm">I LIBRI POETICI</p>'
             '<p style="font-size:8pt;margin-top:40mm">Edizione riveduta del 1885. Testo di pubblico dominio (eBible.org).</p></div>')
    h.append('<div class="toc"><p style="text-align:center;font-weight:bold">INDICE</p>')
    for b in books:
        h.append('<a href="#b-%s">%s</a>' % (b['code'], esc(book_title(b))))
    h.append('</div>')
    for b in books:
        h.append('<h1 id="b-%s" data-short="%s">%s</h1>' % (b['code'], esc(short_name(b)), esc(book_title(b))))
        mt2 = [t for l, t in b['mt'] if l == 2]
        if mt2:
            h.append('<p class="sub">%s</p>' % esc(' '.join(mt2)))
        for bl in b['blocks']:
            if bl['kind'] == 'chapter':
                h.append('<h2 class="cl">%s</h2>' % esc(bl['cl'] or ('CAPO %s' % bl['n'])))
                continue
            if bl['kind'] != 'para':
                continue
            st = bl['style']
            body = render_items(bl['items'])
            if st == 'b':
                h.append('<p class="b"></p>')
                continue
            if not body.strip():
                continue
            if st in ('s1', 's2', 's', 'ms', 'ms1'):
                h.append('<h3 class="s">%s</h3>' % body)
            elif st == 'r':
                h.append('<p class="r">%s</p>' % body)
            elif st == 'd':
                h.append('<p class="d">%s</p>' % body)
            elif st in ('q1', 'q', 'q2', 'q3'):
                h.append('<p class="%s">%s</p>' % ('q2' if st in ('q2', 'q3') else 'q1', body))
            else:
                h.append('<p class="p">%s</p>' % body)
    h.append('</body></html>')
    pdf = weasyprint.HTML(string='\n'.join(h)).write_pdf(stylesheets=[weasyprint.CSS(string=CSS)])
    with open(OUT, 'wb') as f:
        f.write(pdf)
    print(OUT)


if __name__ == '__main__':
    main()
