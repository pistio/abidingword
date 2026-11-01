"""grcmt (Robinson-Pierpont Byzantine/Majority Text 2018) -> plain UTF-8 text.
Preface (with its tab-separated code tables) first, then each book under a ruled, centred title,
chapter headings 'ΚΕΦΑΛΑΙΟΝ Α΄' with Greek alphabetic numerals, one verse per line 'n  text' wrapped at
76 columns with hanging indent; CRLF line ends (as produced by many Windows tools)."""
import sys, textwrap
sys.path.insert(0, '/root/aw-set-e/tools')
import usfm

SRC, OUT = sys.argv[1], sys.argv[2]
books = usfm.load(SRC)
U = ['', 'Α', 'Β', 'Γ', 'Δ', 'Ε', 'Ϛ', 'Ζ', 'Η', 'Θ']
T = ['', 'Ι', 'Κ', 'Λ', 'Μ', 'Ν', 'Ξ', 'Ο', 'Π', 'Ϟ']
H = ['', 'Ρ', 'Σ', 'Τ', 'Υ', 'Φ', 'Χ', 'Ψ', 'Ω', 'Ϡ']


def greek_num(n):
    return H[n // 100] + T[(n // 10) % 10] + U[n % 10] + 'ʹ'


W = 76
out = []
out.append('Η ΚΑΙΝΗ ΔΙΑΘΗΚΗ'.center(W).rstrip())
out.append('The Greek New Testament according to the Byzantine Text form'.center(W).rstrip())
out.append('Robinson-Pierpont 2018 edition'.center(W).rstrip())
out.append('')
out.append('Public Domain. Translation by: Maurice A. Robinson and William G. Pierpont.')
out.append('Source: eBible.org (grcmt).')
out.append('')
out.append('=' * W)
fr = books.get('FRT')
if fr:
    out.append((fr.mt[0][1] if fr.mt else 'Preface').upper())
    out.append('')
    for b in fr.blocks:
        t = usfm.plain(b[2]) if b[1] != 'tr' else '\t'.join(x.strip() for it in b[2] if it[0] == 't' for x in it[1].split('\t') if x.strip())
        if b[0] == 'h':
            out += ['', t, '-' * min(len(t), W), '']
        elif b[1] == 'tr':
            out.append('    ' + t)
        elif b[1] == 'pc':
            out.append(t.center(W).rstrip())
        else:
            out += textwrap.wrap(t, W) + ['']
    out.append('')
for bk in usfm.ordered(books, usfm.NT):
    out += ['', '=' * W, bk.long_name.strip().center(W).rstrip(), '=' * W, '']
    for b in bk.blocks:
        if b[0] == 'c':
            out += ['', 'ΚΕΦΑΛΑΙΟΝ ' + greek_num(int(b[1])), '']
        elif b[0] == 'p':
            cur = None; buf = []
            segs = []
            for it in b[2]:
                if it[0] == 'v':
                    if cur is not None or buf:
                        segs.append((cur, ''.join(buf)))
                    cur, buf = it[1], []
                elif it[0] == 't':
                    buf.append(it[1])
            segs.append((cur, ''.join(buf)))
            for v, t in segs:
                t = ' '.join(t.split())
                if v is None and not t:
                    continue
                lab = (v or '').rjust(3) + '  '
                w = textwrap.wrap(t, W - len(lab)) or ['']
                out.append(lab + w[0])
                out += [' ' * len(lab) + x for x in w[1:]]
open(OUT, 'w', encoding='utf-8', newline='\r\n').write('\n'.join(out) + '\n')
print('ok')
