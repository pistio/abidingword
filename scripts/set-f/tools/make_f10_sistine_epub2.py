#!/usr/bin/env python3
"""F10: Vulgata Sistina (1590) - the Prophets (Isaias - Malachias, with Baruch).
Generator: this script writes an EPUB 2.0.1 by hand (zipfile: mimetype stored
first, OPF 2.0, NCX with nested book/chapter navPoints, XHTML 1.1 files).
Every book its own file; chapters headed 'CAPUT I.' (roman); one verse per
paragraph with its number floated into the left margin; Vulgate chapter
division of Daniel (14 chapters) as the source has it."""
import os, sys, zipfile, uuid
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import load_zip, clean_items, esc, DOCS, short_name, book_title, roman

REF = os.path.join(DOCS, 'F10-lat-sistine.usfm.zip')
OUT = os.path.join(DOCS, 'F10-lat-sistine.epub')
BOOKID = 'urn:uuid:' + str(uuid.uuid5(uuid.NAMESPACE_URL, 'set-f/vulgata-sistina/prophetae'))

CSS = """body { font-family: serif; margin: 0 1em 0 3.2em; line-height: 1.4; }
h1 { text-align: center; font-size: 1.4em; letter-spacing: 0.1em; margin: 1.5em 0 0.2em -2.2em; }
p.inc { text-align: center; font-style: italic; margin: 0 0 1.2em -2.2em; }
h2 { text-align: center; font-size: 1em; letter-spacing: 0.2em; margin: 1.4em 0 0.6em -2.2em; font-weight: normal; }
p.v { margin: 0 0 0.2em 0; text-indent: 0; text-align: justify; }
span.vn { float: left; width: 1.8em; margin-left: -2.2em; text-align: right; font-size: 0.75em; color: #900; padding-top: 0.15em; }
"""

XHTML = '''<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.1//EN" "http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd">
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="la">
<head><title>%s</title><link rel="stylesheet" type="text/css" href="vulgata.css"/></head>
<body>
%s
</body></html>'''


def main():
    books = load_zip(REF, order='file')
    files = []
    nav = []
    title = XHTML % ('Prophetae', '<h1 style="margin-top:30%%">BIBLIA SACRA</h1><p class="inc">VVLGATAE EDITIONIS</p>'
                     '<p class="inc">SIXTI V. PONT. MAX. IVSSV RECOGNITA</p><p class="inc">ROMAE 1590</p>'
                     '<h2>TOMVS: PROPHETAE</h2><p class="inc">Textus publici iuris (scrollmapper/bible_databases, VulgSistine).</p>')
    files.append(('titulus.xhtml', title))
    for i, b in enumerate(books):
        fn = 'liber%02d.xhtml' % (i + 1)
        body = ['<h1 id="top">%s</h1>' % esc(book_title(b).upper()), '<p class="inc">%s</p>' % esc(short_name(b))]
        chaps = []
        chap = None
        for bl in b['blocks']:
            if bl['kind'] == 'chapter':
                chap = bl['n']
                body.append('<h2 id="c%s">CAPUT %s.</h2>' % (chap, roman(int(chap))))
                chaps.append(chap)
                continue
            if bl['kind'] != 'para':
                continue
            cur = None
            for it in clean_items(bl['items']):
                if it[0] == 'v':
                    if cur is not None:
                        body.append(cur + '</p>')
                    cur = '<p class="v"><span class="vn">%s</span>' % it[1]
                elif it[0] == 't':
                    if cur is None:
                        cur = '<p class="v">'
                    cur += esc(it[1])
            if cur is not None:
                body.append(cur + '</p>')
        files.append((fn, XHTML % (esc(short_name(b)), '\n'.join(body))))
        nav.append((fn, short_name(b), chaps))
    opf = ['<?xml version="1.0" encoding="utf-8"?>',
           '<package xmlns="http://www.idpf.org/2007/opf" version="2.0" unique-identifier="bookid">',
           '<metadata xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:opf="http://www.idpf.org/2007/opf">',
           '<dc:title>Biblia Sacra Vulgatae Editionis (Sixtina 1590): Prophetae</dc:title>',
           '<dc:language>la</dc:language><dc:identifier id="bookid">%s</dc:identifier>' % BOOKID,
           '<dc:rights>Public domain</dc:rights><dc:creator opf:role="aut">Hieronymus</dc:creator>',
           '<dc:date>1590</dc:date><meta name="generator" content="set-F make_f10_sistine_epub2.py"/>',
           '</metadata><manifest>',
           '<item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>',
           '<item id="css" href="vulgata.css" media-type="text/css"/>']
    for k, (fn, _) in enumerate(files):
        opf.append('<item id="f%d" href="%s" media-type="application/xhtml+xml"/>' % (k, fn))
    opf.append('</manifest><spine toc="ncx">')
    for k in range(len(files)):
        opf.append('<itemref idref="f%d"/>' % k)
    opf.append('</spine><guide><reference type="title-page" title="Titulus" href="titulus.xhtml"/></guide></package>')
    ncx = ['<?xml version="1.0" encoding="utf-8"?>',
           '<!DOCTYPE ncx PUBLIC "-//NISO//DTD ncx 2005-1//EN" "http://www.daisy.org/z3986/2005/ncx-2005-1.dtd">',
           '<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1"><head>',
           '<meta name="dtb:uid" content="%s"/><meta name="dtb:depth" content="2"/>' % BOOKID,
           '<meta name="dtb:totalPageCount" content="0"/><meta name="dtb:maxPageNumber" content="0"/></head>',
           '<docTitle><text>Prophetae</text></docTitle><navMap>']
    po = 1
    ncx.append('<navPoint id="np%d" playOrder="%d"><navLabel><text>Titulus</text></navLabel><content src="titulus.xhtml"/></navPoint>' % (po, po))
    for fn, name, chaps in nav:
        po += 1
        ncx.append('<navPoint id="np%d" playOrder="%d"><navLabel><text>%s</text></navLabel><content src="%s"/>' % (po, po, esc(name), fn))
        for c in chaps:
            po += 1
            ncx.append('<navPoint id="np%d" playOrder="%d"><navLabel><text>Caput %s</text></navLabel><content src="%s#c%s"/></navPoint>'
                       % (po, po, roman(int(c)), fn, c))
        ncx.append('</navPoint>')
    ncx.append('</navMap></ncx>')
    container = ('<?xml version="1.0"?><container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">'
                 '<rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles></container>')
    dt = (2026, 11, 1, 0, 0, 0)
    with zipfile.ZipFile(OUT, 'w') as z:
        z.writestr(zipfile.ZipInfo('mimetype', dt), 'application/epub+zip', compress_type=zipfile.ZIP_STORED)
        def put(name, data):
            zi = zipfile.ZipInfo(name, dt)
            zi.compress_type = zipfile.ZIP_DEFLATED
            z.writestr(zi, data.encode('utf-8'))
        put('META-INF/container.xml', container)
        put('OEBPS/content.opf', '\n'.join(opf))
        put('OEBPS/toc.ncx', '\n'.join(ncx))
        put('OEBPS/vulgata.css', CSS)
        for fn, data in files:
            put('OEBPS/' + fn, data)
    print(OUT)


if __name__ == '__main__':
    main()
