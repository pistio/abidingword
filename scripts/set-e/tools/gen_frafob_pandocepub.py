"""fra_fob (Ostervald) New Testament + Psaumes -> EPUB via pandoc (HTML input).
Book = level-1 heading (one EPUB chapter file per book), 'Chapitre N' / 'Psaume N' level-2 headings,
superscript verse numbers run into the paragraph."""
import sys, html, subprocess, os
sys.path.insert(0, '/root/aw-set-e/tools')
import usfm

SRC, OUT = sys.argv[1], sys.argv[2]
E = lambda t: html.escape(t, quote=False)
books = usfm.load(SRC)
seq = usfm.ordered(books, usfm.NT + ['PSA'])
h = ['<html lang="fr"><head><meta charset="utf-8"><title>Le Nouveau Testament et les Psaumes</title></head><body>']
for bk in seq:
    h.append('<h1>%s</h1>' % E(bk.long_name.strip()))
    if bk.code == 'PSA':
        h.append('<p><em>Le livre des Psaumes, placé à la suite du Nouveau Testament.</em></p>')
    for b in bk.blocks:
        if b[0] == 'c':
            h.append('<h2>%s %s</h2>' % ('Psaume' if bk.code == 'PSA' else 'Chapitre', E(b[1])))
        elif b[0] == 'p':
            out = []
            for it in b[2]:
                if it[0] == 'v':
                    out.append('<sup>%s</sup> ' % E(it[1]))
                elif it[0] == 't':
                    out.append(E(it[1]))
            h.append('<p>%s</p>' % ''.join(out).strip())
h.append('</body></html>')
tmp = OUT + '.src.html'
open(tmp, 'w', encoding='utf-8').write('\n'.join(h))
meta = OUT + '.meta.yaml'
open(meta, 'w', encoding='utf-8').write(
    '---\ntitle: Le Nouveau Testament et les Psaumes\nsubtitle: Version d\'Ostervald\ncreator:\n- role: translator\n  text: J.-F. Ostervald\n'
    'lang: fr-FR\nrights: "La Sainte Bible, Ostervald. Domaine public (Public Domain). Source : eBible.org (fra_fob)."\n'
    'date: 2026-10-31\n...\n')
css = OUT + '.css'
open(css, 'w').write('body{font-family:serif;line-height:1.5} h1{text-align:center;font-variant:small-caps} '
                     'h2{font-size:1.05em;text-align:left;margin-top:1.4em;color:#552} p{text-indent:1.2em;margin:0;text-align:justify} '
                     'sup{font-size:.62em;color:#666}\n')
subprocess.run(['pandoc', '-f', 'html', '-t', 'epub3', '--metadata-file', meta, '--css', css, '--toc', '--toc-depth=1',
                '--split-level=1', '-o', OUT, tmp], check=True)
for f in (tmp, meta, css):
    os.remove(f)
print('ok')
