"""sparvg (Reina Valera Gómez) whole Bible -> Word .docx via pandoc (HTML input).
Book = Heading 1, chapter = Heading 2 holding only the number, one verse per paragraph with
superscript verse number, Psalm titles italic, supplied words in italics, words of Christ in red (custom style span)."""
import sys, html, subprocess, os
sys.path.insert(0, '/root/aw-set-e/tools')
import usfm

SRC, OUT = sys.argv[1], sys.argv[2]
E = lambda t: html.escape(t, quote=False)
books = usfm.load(SRC)
h = ['<html lang="es"><head><meta charset="utf-8"><title>Santa Biblia Reina Valera Gómez</title></head><body>']
h.append('<p><strong>DERECHOS RESERVADOS</strong> Copyright 2004, 2010, 2023 By Dr. Humberto Gómez Caballero. '
         'Toda Iglesia u organización que desee imprimirla o reproducirla para su distribución gratuita tendrá la plena libertad '
         'de hacerlo sin necesidad de pagar regalías, siempre y cuando no cambie ninguna de las palabras escritas. '
         'Fuente: eBible.org (sparvg).</p>')
h.append('<h1>ANTIGUO TESTAMENTO</h1>')


def inl(items):
    out = []
    for it in items:
        if it[0] == 't':
            t = E(it[1]); st = it[2]
            if st & {'add', 'it', 'em'}:
                t = '<em>%s</em>' % t
            if 'wj' in st:
                t = '<span class="wj" style="color:#C00000">%s</span>' % t
            out.append(t)
        elif it[0] == 'v':
            out.append('<sup>%s</sup> ' % E(it[1]))
    return ''.join(out).strip()


for bk in usfm.ordered(books):
    if bk.code == 'MAT':
        h.append('<h1>NUEVO TESTAMENTO</h1>')
    h.append('<h1>%s</h1>' % E(bk.long_name.strip()))
    for b in bk.blocks:
        if b[0] == 'c':
            h.append('<h2>%s</h2>' % E(b[1]))
        elif b[0] == 'h':
            h.append('<p><em>%s</em></p>' % inl(b[2]))
        elif b[0] == 'p':
            h.append('<p>%s</p>' % inl(b[2]))
h.append('</body></html>')
tmp = OUT + '.src.html'
open(tmp, 'w', encoding='utf-8').write('\n'.join(h))
subprocess.run(['pandoc', '-f', 'html', '-t', 'docx', '--metadata', 'title=Santa Biblia Reina Valera Gómez',
                '--metadata', 'lang=es-MX', '--metadata', 'author=Dr. Humberto Gómez Caballero', '-o', OUT, tmp], check=True)
os.remove(tmp)
print('ok')
