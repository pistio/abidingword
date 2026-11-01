"""engwyc2017 Apocrypha -> hand-assembled EPUB 3 (zipfile). Bracketed verse numbers, CHAPTER n headings,
EPUB3 popup footnotes collected at the end of each book file, nav + NCX."""
import sys, html, zipfile, uuid, datetime
sys.path.insert(0, '/root/aw-set-e/tools')
import usfm

SRC, OUT = sys.argv[1], sys.argv[2]
CODES = 'TOB JDT ESG WIS SIR BAR LJE S3Y SUS BEL 1MA 2MA 1ES MAN'.split()
books = usfm.load(SRC)
seq = [books[c] for c in CODES if c in books]
E = lambda t: html.escape(t, quote=False)
UID = 'urn:uuid:' + str(uuid.uuid5(uuid.NAMESPACE_URL, 'aw-set-e/engwyc2017-epubapoc'))

CSS = """body{font-family:"Georgia",serif;line-height:1.45;margin:0 4%}
h1{font-variant:small-caps;text-align:center;font-size:1.7em;margin:1.5em 0 .3em;letter-spacing:.05em}
h2.chap{font-size:1em;text-align:center;font-weight:bold;letter-spacing:.2em;margin:1.6em 0 .6em;border-bottom:1px solid #aaa;padding-bottom:.2em}
p.vs{margin:0 0 .25em 0;text-indent:0;text-align:justify}
span.vn{color:#7a1f1f;font-weight:bold;font-size:.8em;margin-right:.25em}
span.alt{font-style:italic;color:#444}
em.gloss{font-style:italic}
a.fn{font-size:.7em;vertical-align:super;text-decoration:none;color:#336}
aside.fnote{font-size:.85em;margin:.2em 0}
section.notes{border-top:1px solid #999;margin-top:2em;padding-top:.5em}
p.intro{font-size:.92em;text-indent:1.2em;color:#222}
.title{text-align:center;margin-top:30%}
.copy{font-size:.8em}
"""


def page(title, body):
    return ('<?xml version="1.0" encoding="UTF-8"?>\n<!DOCTYPE html>\n'
            '<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" xml:lang="en" lang="en">\n'
            '<head><meta charset="UTF-8"/><title>%s</title><link rel="stylesheet" type="text/css" href="style.css"/></head>\n'
            '<body>\n%s\n</body></html>\n') % (E(title), body)


def inl(items, notes, bid):
    out = []
    for it in items:
        if it[0] == 't':
            t = E(it[1]); st = it[2]
            if 'add' in st:
                t = '<span class="alt">%s</span>' % t
            elif st & {'em', 'it'}:
                t = '<em class="gloss">%s</em>' % t
            elif 'bk' in st:
                t = '<cite>%s</cite>' % t
            out.append(t)
        elif it[0] == 'v':
            out.append('<span class="vn">[%s]</span> ' % E(it[1]))
        elif it[0] == 'n':
            ref, txt = usfm.note_text(it[3])
            n = len(notes) + 1
            nid = '%s-n%d' % (bid, n)
            notes.append((nid, ref, txt))
            out.append('<a class="fn" epub:type="noteref" href="#%s" id="r-%s">%d</a>' % (nid, nid, n))
    return ''.join(out)


files = []
files.append(('title.xhtml', 'Title', page('Title', '<div class="title"><h1>The Apocrypha</h1><p>Wycliffe’s Bible with Modern Spelling</p>'
                                         '<p><em>Tobit – Prayer of Manasses</em></p></div>')))
copy = ('<h2>Copyright</h2><p class="copy">Wycliffe’s Bible with Modern Spelling. English: Wycliffe’s Bible Modern Spelling (with Deuterocanon). '
        'Copyright © 2001,2010,2011,2012,2013,2015 Terry Noble. Translation by: Terry Noble.</p>'
        '<p class="copy">This translation is made available to you under the terms of the Creative Commons Attribution-Noncommercial-No Derivatives '
        'license 4.0. Source: eBible.org (engwyc2017).</p>'
        '<p class="copy">Readings in square brackets beginning “or” are the translator’s alternative renderings and are printed in italic.</p>')
files.append(('copyright.xhtml', 'Copyright', page('Copyright', copy)))
nav_items = []
for i, bk in enumerate(seq):
    bid = 'b%02d' % (i + 1)
    notes = []
    parts = ['<section epub:type="chapter" id="%s">' % bid, '<h1>%s</h1>' % E(bk.long_name.title() if bk.long_name.isupper() else bk.long_name)]
    chapnav = []
    for b in bk.blocks:
        if b[0] == 'c':
            lab = b[2] or ('CHAPTER %s' % b[1])
            cid = '%s-c%s' % (bid, b[1])
            parts.append('<h2 class="chap" id="%s">%s</h2>' % (cid, E(lab)))
            chapnav.append((cid, lab))
        elif b[0] == 'h':
            parts.append('<h3>%s</h3>' % inl(b[2], notes, bid))
        elif b[0] == 'p':
            cls = 'intro' if b[1].startswith('i') else 'vs'
            parts.append('<p class="%s">%s</p>' % (cls, inl(b[2], notes, bid)))
    if notes:
        parts.append('<section class="notes" epub:type="footnotes"><h3>Notes</h3>')
        for nid, ref, txt in notes:
            parts.append('<aside class="fnote" epub:type="footnote" id="%s"><p><a href="#r-%s">%s</a> %s</p></aside>'
                         % (nid, nid, E(ref), E(txt)))
        parts.append('</section>')
    parts.append('</section>')
    fn = '%s.xhtml' % bid
    files.append((fn, bk.name, page(bk.name, '\n'.join(parts))))
    nav_items.append((fn, bk.name, chapnav))

nav = ['<nav epub:type="toc" id="toc"><h1>Contents</h1><ol>', '<li><a href="copyright.xhtml">Copyright</a></li>']
for fn, name, chs in nav_items:
    nav.append('<li><a href="%s">%s</a><ol>%s</ol></li>' % (fn, E(name), ''.join(
        '<li><a href="%s#%s">%s</a></li>' % (fn, cid, E(lab)) for cid, lab in chs)))
nav.append('</ol></nav>')
navx = page('Contents', '\n'.join(nav))
ncx = ['<?xml version="1.0" encoding="UTF-8"?>\n<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1"><head>'
       '<meta name="dtb:uid" content="%s"/></head><docTitle><text>The Apocrypha</text></docTitle><navMap>' % UID]
for k, (fn, name, chs) in enumerate(nav_items):
    ncx.append('<navPoint id="np%d" playOrder="%d"><navLabel><text>%s</text></navLabel><content src="%s"/></navPoint>' % (k + 1, k + 1, E(name), fn))
ncx.append('</navMap></ncx>')
mod = datetime.datetime(2026, 10, 31, tzinfo=datetime.timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ')
opf = ['<?xml version="1.0" encoding="UTF-8"?>\n<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="uid">'
       '<metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:identifier id="uid">%s</dc:identifier>'
       '<dc:title>The Apocrypha – Wycliffe’s Bible with Modern Spelling</dc:title><dc:language>en</dc:language>'
       '<dc:creator>Terry Noble</dc:creator><dc:rights>CC BY-NC-ND 4.0</dc:rights><meta property="dcterms:modified">%s</meta></metadata><manifest>'
       '<item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>'
       '<item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>'
       '<item id="css" href="style.css" media-type="text/css"/>' % (UID, mod)]
for fn, t, _ in files:
    opf.append('<item id="%s" href="%s" media-type="application/xhtml+xml"/>' % (fn.split('.')[0], fn))
opf.append('</manifest><spine toc="ncx">')
for fn, t, _ in files:
    opf.append('<itemref idref="%s"/>' % fn.split('.')[0])
opf.append('</spine></package>')

with zipfile.ZipFile(OUT, 'w') as z:
    zi = zipfile.ZipInfo('mimetype', (2026, 10, 31, 0, 0, 0)); zi.compress_type = zipfile.ZIP_STORED
    z.writestr(zi, 'application/epub+zip')
    def w(name, data):
        zi = zipfile.ZipInfo(name, (2026, 10, 31, 0, 0, 0)); zi.compress_type = zipfile.ZIP_DEFLATED
        z.writestr(zi, data)
    w('META-INF/container.xml', '<?xml version="1.0"?><container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">'
      '<rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles></container>')
    w('OEBPS/content.opf', ''.join(opf))
    w('OEBPS/nav.xhtml', navx)
    w('OEBPS/toc.ncx', ''.join(ncx))
    w('OEBPS/style.css', CSS)
    for fn, t, data in files:
        w('OEBPS/' + fn, data)
print('ok', [b.code for b in seq])
