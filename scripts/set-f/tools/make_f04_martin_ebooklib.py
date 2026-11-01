#!/usr/bin/env python3
"""F04: Bible David Martin (1744), complete, as EPUB 3.
Generator: EbookLib. One XHTML file per book; at the head of every book a grid
of chapter links; chapters headed 'Chapitre N'; one verse per paragraph with the
number in bold in front; nested navigation (Ancien / Nouveau Testament > books)."""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from ebooklib import epub
from common import load_zip, clean_items, esc, DOCS, short_name, book_title, NT

REF = os.path.join(DOCS, 'F04-fra-martin.usfm.zip')
OUT = os.path.join(DOCS, 'F04-fra-martin.epub')

CSS = """
body { font-family: Georgia, serif; line-height: 1.45; margin: 0 4%; }
h1 { text-align: center; font-size: 1.5em; margin: 1.2em 0 0.2em 0; font-variant: small-caps; }
p.sub { text-align: center; font-style: italic; margin: 0 0 1em 0; }
div.grid { text-align: center; margin: 0.6em 0 1.2em 0; font-size: 0.9em; }
div.grid a { display: inline-block; min-width: 2.2em; margin: 0.15em; padding: 0.2em 0; border: 1px solid #999; text-decoration: none; }
h2 { font-size: 1.05em; text-align: center; margin: 1.4em 0 0.5em 0; letter-spacing: 0.08em; }
p.v { margin: 0 0 0.25em 0; text-indent: 0; }
b.vn { color: #8b1a1a; margin-right: 0.3em; }
"""


def main():
    books = load_zip(REF)
    bk = epub.EpubBook()
    bk.set_identifier('set-f-martin-1744')
    bk.set_title('La Sainte Bible — version de David Martin (1744)')
    bk.set_language('fr')
    bk.add_author('David Martin')
    bk.add_metadata('DC', 'rights', 'Domaine public')
    css = epub.EpubItem(uid='css', file_name='style/bible.css', media_type='text/css', content=CSS)
    bk.add_item(css)
    title = epub.EpubHtml(title='Titre', file_name='titre.xhtml', lang='fr')
    title.content = ('<html><body><h1>LA SAINTE BIBLE</h1><p class="sub">qui contient le Vieux et le Nouveau Testament</p>'
                     '<p class="sub">revue sur les originaux par David Martin, pasteur de l\'Église wallonne d\'Utrecht</p>'
                     '<p class="sub">1744</p></body></html>')
    title.add_item(css)
    bk.add_item(title)
    ot_items, nt_items = [], []
    spine = ['nav', title]
    for i, b in enumerate(books):
        code = b['code']
        fn = 'livre%02d_%s.xhtml' % (i + 1, code.lower())
        it = epub.EpubHtml(title=short_name(b), file_name=fn, lang='fr')
        h = ['<html><body>', '<h1 id="top">%s</h1>' % esc(book_title(b))]
        chaps = [bl['n'] for bl in b['blocks'] if bl['kind'] == 'chapter']
        if len(chaps) > 1:
            h.append('<div class="grid">' + ' '.join('<a href="#c%s">%s</a>' % (c, c) for c in chaps) + '</div>')
        chap = None
        for bl in b['blocks']:
            if bl['kind'] == 'chapter':
                chap = bl['n']
                h.append('<h2 id="c%s">Chapitre %s</h2>' % (chap, chap))
                continue
            if bl['kind'] != 'para':
                continue
            cur = None
            for x in clean_items(bl['items']):
                if x[0] == 'v':
                    if cur is not None:
                        h.append(cur + '</p>')
                    cur = '<p class="v" id="c%sv%s"><b class="vn">%s</b>' % (chap, x[1], x[1])
                elif x[0] == 't':
                    if cur is None:
                        cur = '<p class="v">'
                    cur += esc(x[1])
            if cur is not None:
                h.append(cur + '</p>')
        h.append('</body></html>')
        it.content = '\n'.join(h)
        it.add_item(css)
        bk.add_item(it)
        spine.append(it)
        (nt_items if code in NT else ot_items).append(it)
    bk.toc = [epub.Link('titre.xhtml', 'Titre', 'titre'),
              (epub.Section('Le Vieux Testament'), ot_items),
              (epub.Section('Le Nouveau Testament'), nt_items)]
    bk.add_item(epub.EpubNcx())
    bk.add_item(epub.EpubNav())
    bk.spine = spine
    epub.write_epub(OUT, bk, {})
    print(OUT)


if __name__ == '__main__':
    main()
