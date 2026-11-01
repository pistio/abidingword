#!/usr/bin/env python3
"""F12: Russian Synodal Bible (eBible russyn), complete, as EPUB 3.
Generator: pandoc (HTML input written here -> epub3, split at level-1 headings,
two-level contents). Books as level-1 headings, 'Глава N' (Psalms 'Псалом N')
as level-2 headings, verse numbers as superscripts, poetry (\\q1) lines kept as
separate lines inside a line block, psalm titles (\\d) in italics."""
import os, sys, subprocess
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import load_zip, clean_items, esc, DOCS, WORK, short_name, book_title, NT

REF = os.path.join(DOCS, 'F12-rus-synodal.usfm.zip')
OUT = os.path.join(DOCS, 'F12-rus-synodal.epub')
CSS = """body { font-family: serif; }
h1 { text-align: center; }
h2 { font-size: 1em; text-align: center; letter-spacing: 0.1em; margin-top: 1.5em; }
sup { font-size: 0.65em; color: #7a1f1f; font-weight: bold; }
p.d { font-style: italic; text-align: center; }
div.poem p { margin: 0 0 0 2em; text-indent: -1em; }
"""


def inline(items):
    s = []
    for it in clean_items(items):
        if it[0] == 'v':
            s.append('<sup>%s</sup> ' % it[1])
        elif it[0] == 't':
            s.append(esc(it[1]))
    return ''.join(s)


def main():
    books = load_zip(REF)
    h = ['<!DOCTYPE html><html lang="ru"><head><meta charset="utf-8"><title>Библия</title></head><body>']
    for b in books:
        if b['code'] == 'GEN':
            pass
        h.append('<h1>%s</h1>' % esc(book_title(b)))
        poem = False
        for bl in b['blocks']:
            if bl['kind'] == 'chapter':
                if poem:
                    h.append('</div>'); poem = False
                h.append('<h2>%s %s</h2>' % ('Псалом' if b['code'] == 'PSA' else 'Глава', bl['n']))
                continue
            if bl['kind'] != 'para':
                continue
            st = bl['style']
            t = inline(bl['items'])
            if not t.strip():
                continue
            if st.startswith('q'):
                if not poem:
                    h.append('<div class="poem">'); poem = True
                h.append('<p>%s</p>' % t)
                continue
            if poem:
                h.append('</div>'); poem = False
            if st == 'd':
                h.append('<p class="d">%s</p>' % t)
            else:
                h.append('<p>%s</p>' % t)
        if poem:
            h.append('</div>')
    h.append('</body></html>')
    src = os.path.join(WORK, 'F12-rus-synodal.html')
    with open(src, 'w', encoding='utf-8') as f:
        f.write('\n'.join(h))
    css = os.path.join(WORK, 'f12.css')
    with open(css, 'w') as f:
        f.write(CSS)
    subprocess.run(['pandoc', '-f', 'html', '-t', 'epub3', '--split-level=1', '--toc', '--toc-depth=1',
                    '--css', css, '--metadata', 'title=Библия. Книги Священного Писания Ветхого и Нового Завета',
                    '--metadata', 'lang=ru', '--metadata', 'rights=Public domain (Синодальный перевод)',
                    '--metadata', 'subtitle=Синодальный перевод', '-o', OUT, src], check=True)
    print(OUT)


if __name__ == '__main__':
    main()
