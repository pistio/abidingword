#!/usr/bin/env python3
"""F01: New Heart English Bible - New Testament with Psalms and Proverbs.
Layout engine: MuPDF Story (PyMuPDF). Two columns on a 5.5x8.5in page, chapter
numerals as large figures that replace verse 1, superscript verse numbers,
running heads with verse ranges added in a second pass, contents in roman pages."""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import pymupdf as fitz
from common import load_zip, clean_items, esc, DOCS, WORK, NT, book_title, short_name

REF = os.path.join(DOCS, 'F01-eng-nheb.usfm.zip')
OUT = os.path.join(DOCS, 'F01-eng-nheb.pdf')
ORDER = NT + ['PSA', 'PRO']
W, H = 396, 612
M_IN, M_OUT, M_TOP, M_BOT, GUT = 36, 30, 46, 40, 14

CSS = """
@font-face {font-family: body; src: url(Caladea-Regular.ttf);}
@font-face {font-family: body; font-weight: bold; src: url(Caladea-Bold.ttf);}
@font-face {font-family: body; font-style: italic; src: url(Caladea-Italic.ttf);}
body {font-family: body; font-size: 8.6pt; line-height: 1.18;}
p {margin: 0; text-indent: 0.9em; text-align: justify;}
p.first {text-indent: 0;}
h1 {font-size: 15pt; text-align: center; margin: 4pt 0 2pt 0; font-weight: bold; letter-spacing: 1pt;}
h2 {font-size: 8pt; text-align: center; font-style: italic; font-weight: normal; margin: 0 0 6pt 0;}
span.cn {font-size: 19pt; font-weight: bold; line-height: 0.9;}
sup {font-size: 5.6pt; font-weight: bold; color: #7a1f1f;}
p.psalm {text-align: center; font-weight: bold; font-size: 9pt; margin-top: 5pt; text-indent: 0; letter-spacing: 0.5pt;}
"""


def build_html(books):
    h = ['<html><body>']
    for b in books:
        code = b['code']
        h.append('<h1 id="b-%s">%s</h1>' % (code, esc(short_name(b).upper())))
        h.append('<h2>%s</h2>' % esc(book_title(b)))
        chap = None
        for bl in b['blocks']:
            if bl['kind'] == 'chapter':
                chap = bl['n']
                pending = chap
                continue
            if bl['kind'] != 'para':
                continue
            items = clean_items(bl['items'])
            out = []
            first_para = False
            if code == 'PSA' and pending:
                h.append('<p class="psalm">PSALM %s</p>' % chap)
            for it in items:
                if it[0] == 'v':
                    vid = 'v-%s-%s-%s' % (code, chap, it[1])
                    if pending and it[1] == '1' and code != 'PSA':
                        out.append('<span class="cn" id="%s">%s</span>&#8202;' % (vid, chap))
                        first_para = True
                    elif pending and it[1] == '1':
                        out.append('<sup id="%s">1</sup>' % vid)
                        first_para = True
                    else:
                        out.append('<sup id="%s">%s</sup>' % (vid, it[1]))
                    pending = None
                elif it[0] == 't':
                    out.append(esc(it[1]))
            h.append('<p class="%s">%s</p>' % ('first' if first_para else '', ''.join(out)))
    h.append('</body></html>')
    return '\n'.join(h)


def layout(html, path, columns=2, front=False):
    arch = fitz.Archive('/usr/share/fonts/truetype/crosextra')
    story = fitz.Story(html=html, user_css=CSS, archive=arch)
    writer = fitz.DocumentWriter(path)
    positions = []
    page = 0
    more = 1
    while more:
        mb = fitz.Rect(0, 0, W, H)
        dev = writer.begin_page(mb)
        odd = page % 2 == 0
        left = M_IN if odd else M_OUT
        right = W - (M_OUT if odd else M_IN)
        if columns == 2:
            cw = (right - left - GUT) / 2
            rects = [fitz.Rect(left, M_TOP, left + cw, H - M_BOT), fitz.Rect(left + cw + GUT, M_TOP, right, H - M_BOT)]
        else:
            rects = [fitz.Rect(left, M_TOP, right, H - M_BOT)]
        for r in rects:
            if not more:
                break
            more, filled = story.place(r)
            def cb(pos):
                if pos.id:
                    positions.append((pos.page_num, pos.id, pos.rect))
            story.element_positions(cb, {'page_num': page})
            story.draw(dev)
        writer.end_page()
        page += 1
    writer.close()
    return positions, page


def main():
    books = load_zip(REF, ORDER)
    names = {b['code']: short_name(b) for b in books}
    html = build_html(books)
    body = os.path.join(WORK, 'f01_body.pdf')
    pos, npages = layout(html, body)
    # first and last verse on every page, and first page of each book
    first, last, bstart = {}, {}, {}
    for page, eid, rect in pos:
        if eid.startswith('v-'):
            _, code, c, v = eid.split('-', 3)
            first.setdefault(page, (code, c, v))
            last[page] = (code, c, v)
        elif eid.startswith('b-'):
            bstart.setdefault(eid[2:], page)
    doc = fitz.open(body)
    for p in range(doc.page_count):
        page = doc[p]
        odd = p % 2 == 0
        num = str(p + 1)
        if p in first:
            a, b = first[p], last[p]
            if a[0] == b[0]:
                if a[1] == b[1]:
                    head = '%s %s:%s-%s' % (names[a[0]], a[1], a[2], b[2]) if a[2] != b[2] else '%s %s:%s' % (names[a[0]], a[1], a[2])
                else:
                    head = '%s %s:%s-%s:%s' % (names[a[0]], a[1], a[2], b[1], b[2])
            else:
                head = '%s %s:%s - %s %s:%s' % (names[a[0]], a[1], a[2], names[b[0]], b[1], b[2])
            head = head.upper()
            fs = 7.5
            tw = fitz.get_text_length(head, fontname='tiro', fontsize=fs)
            if odd:
                page.insert_text((W - M_OUT - tw, 30), head, fontname='tiro', fontsize=fs)
                page.insert_text((M_IN, 30), num, fontname='tibo', fontsize=8)
            else:
                page.insert_text((M_OUT, 30), head, fontname='tiro', fontsize=fs)
                nw = fitz.get_text_length(num, fontname='tibo', fontsize=8)
                page.insert_text((W - M_IN - nw, 30), num, fontname='tibo', fontsize=8)
            page.draw_line((M_IN if odd else M_OUT, 34), (W - (M_OUT if odd else M_IN), 34), width=0.4)
        else:
            nw = fitz.get_text_length(num, fontname='tiro', fontsize=8)
            page.insert_text(((W - nw) / 2, H - 20), num, fontname='tiro', fontsize=8)
    # front matter: title page and contents (roman page numbers)
    toc = ['<html><body>',
           '<p style="text-align:center;font-size:24pt;font-weight:bold;margin-top:120pt;text-indent:0">THE HOLY BIBLE</p>',
           '<p style="text-align:center;font-size:11pt;text-indent:0;margin-top:8pt">New Heart English Bible</p>',
           '<p style="text-align:center;font-size:10pt;text-indent:0;margin-top:30pt;font-style:italic">The New Testament<br/>with Psalms and Proverbs</p>',
           '<p style="text-align:center;font-size:7pt;text-indent:0;margin-top:150pt">The New Heart English Bible is in the public domain.</p>',
           '<p style="page-break-before:always;text-align:center;font-size:13pt;font-weight:bold;text-indent:0;margin-bottom:10pt">CONTENTS</p>',
           '<table style="width:100%;font-size:9pt">']
    for b in books:
        toc.append('<tr><td>%s</td><td style="text-align:right">%d</td></tr>' % (esc(names[b['code']]), bstart.get(b['code'], 0) + 1))
    toc.append('</table></body></html>')
    front = os.path.join(WORK, 'f01_front.pdf')
    story = fitz.Story(html='\n'.join(toc), user_css=CSS, archive=fitz.Archive('/usr/share/fonts/truetype/crosextra'))
    writer = fitz.DocumentWriter(front)
    more = 1
    while more:
        dev = writer.begin_page(fitz.Rect(0, 0, W, H))
        more, _ = story.place(fitz.Rect(50, 50, W - 50, H - 50))
        story.draw(dev)
        writer.end_page()
    writer.close()
    fd = fitz.open(front)
    for i in range(1, fd.page_count):
        r = ['i', 'ii', 'iii', 'iv', 'v', 'vi', 'vii', 'viii'][i]
        fd[i].insert_text((W / 2 - 4, H - 20), r, fontname='tiro', fontsize=8)
    fd.insert_pdf(doc)
    fd.set_metadata({'title': 'The Holy Bible: New Heart English Bible (NT, Psalms, Proverbs)', 'author': 'New Heart English Bible',
                     'creator': 'set-F make_f01_nheb_mupdf.py', 'producer': 'PyMuPDF %s / MuPDF Story' % fitz.VersionBind})
    toc_entries = [[1, names[b['code']], bstart[b['code']] + 1 + fd.page_count - doc.page_count] for b in books]
    fd.set_toc(toc_entries)
    fd.save(OUT, garbage=3, deflate=True)
    print(OUT, fd.page_count, 'pages')


if __name__ == '__main__':
    main()
