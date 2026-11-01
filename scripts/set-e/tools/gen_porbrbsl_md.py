"""porbrbsl (Bíblia Portuguesa Mundial) Salmos, Provérbios + Novo Testamento -> Markdown (Pandoc/GitHub flavour).
YAML front matter, '# Livro' headings, '## Livro N' chapter headings, bold verse numbers inline, poetry as
blockquote lines with hard breaks and indentation, footnotes and cross references as Markdown footnotes
[^c-n] defined after each chapter."""
import sys, re
sys.path.insert(0, '/root/aw-set-e/tools')
import usfm

SRC, OUT = sys.argv[1], sys.argv[2]
books = usfm.load(SRC)


def esc(t):
    return re.sub(r'([\\`*_\[\]<>|])', r'\\\1', t)


lines = ['---', 'title: "Bíblia Portuguesa Mundial — Salmos, Provérbios e Novo Testamento"', 'lang: pt-BR',
         'rights: "Public Domain. Este é um rascunho de tradução da Bíblia Sagrada e ainda em revisão. Fonte: eBible.org (porbrbsl)."',
         '---', '', '# Sumário', '']
seq = usfm.ordered(books, ['PSA', 'PRO'] + usfm.NT)
for bk in seq:
    lines.append('- [%s](#%s)' % (bk.long_name.strip(), bk.code.lower()))
lines.append('')
for bk in seq:
    name = bk.meta.get('h', bk.name).strip()
    lines += ['', '# %s {#%s}' % (esc(bk.long_name.strip()), bk.code.lower()), '']
    for lvl, t in bk.mt[1:]:
        pass
    notes = []
    ch = None

    def flush():
        if notes:
            lines.append('')
            for nid, ref, txt, kind in notes:
                lines.append('[^%s]: %s%s%s' % (nid, '**%s** ' % esc(ref) if ref else '', 'Ref.: ' if kind == 'x' else '', esc(txt)))
            notes.clear()
            lines.append('')

    def inl(items):
        out = []
        for it in items:
            if it[0] == 't':
                t = esc(it[1])
                if it[2] & {'add', 'em', 'it', 'qs', 'bk'} and t.strip():
                    lead = t[:len(t) - len(t.lstrip())]; trail = t[len(t.rstrip()):]
                    t = '%s*%s*%s' % (lead, t.strip(), trail)
                out.append(t)
            elif it[0] == 'v':
                out.append('**%s** ' % it[1])
            elif it[0] == 'n':
                ref, txt = usfm.note_text(it[3])
                nid = '%s-%s-%d' % (bk.code.lower(), ch or '0', len(notes) + 1)
                notes.append((nid, ref, txt, 'x' if it[1] == 'x' else 'f'))
                out.append('[^%s]' % nid)
        return re.sub(r' {2,}', ' ', ''.join(out)).strip()

    prev_poetry = False
    for b in bk.blocks:
        if b[0] == 'c':
            flush()
            ch = b[1]
            lab = b[2] or '%s %s' % (name, ch)
            lines += ['', '## %s' % esc(lab), '']
            prev_poetry = False
            continue
        if b[0] == 'h':
            t = inl(b[2])
            if b[1] == 'd':
                lines += ['', '*%s*' % t.replace('*', ''), ''] if t else []
            elif b[1] == 'sp':
                lines += ['', '**%s:**' % t, '']
            else:
                lines += ['', '### %s' % t, '']
            prev_poetry = False
            continue
        if b[1] == 'b':
            lines.append(''); prev_poetry = False; continue
        t = inl(b[2])
        if b[1].startswith('q'):
            ind = {'q1': '', 'q': '', 'q2': ' ', 'q3': '  '}.get(b[1], '')
            if not prev_poetry:
                lines.append('')
            lines.append('> %s%s\\' % (ind, t))
            prev_poetry = True
        else:
            if prev_poetry:
                if lines[-1].endswith('\\'):
                    lines[-1] = lines[-1][:-1]
                lines.append('')
            lines += [t, '']
            prev_poetry = False
    if lines and lines[-1].endswith('\\'):
        lines[-1] = lines[-1][:-1]
    flush()
txt = '\n'.join(lines)
# a trailing hard-break backslash before a blank line is not wanted
txt = re.sub(r'\\\n(?=\n|[^>])', '\n', txt)
open(OUT, 'w', encoding='utf-8').write(txt + '\n')
print('ok')
