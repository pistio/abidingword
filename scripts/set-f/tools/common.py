"""Helpers shared by the set-F renderers."""
import os, re, sys, zipfile, html
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import usfm
from booknames import OT, NT, DC

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DOCS = os.path.join(ROOT, 'docs')
WORK = os.path.join(ROOT, 'work')

CANON = OT + ['TOB', 'JDT', 'ESG', 'WIS', 'SIR', 'BAR', '1MA', '2MA', 'DAG', 'MAN'] + NT
TANAKH = ('GEN EXO LEV NUM DEU JOS JDG 1SA 2SA 1KI 2KI ISA JER EZK HOS JOL AMO OBA JON MIC NAM HAB ZEP HAG '
          'ZEC MAL PSA PRO JOB SNG RUT LAM ECC EST DAN EZR NEH 1CH 2CH').split()


def load_zip(path, order=None):
    books = []
    with zipfile.ZipFile(path) as z:
        for n in z.namelist():
            if n.endswith('.usfm'):
                books.append(usfm.parse_usfm(z.read(n).decode('utf-8-sig')))
    if order == 'file':
        return books
    order = order or CANON
    pos = {c: i for i, c in enumerate(order)}
    books.sort(key=lambda b: pos.get(b['code'], 999))
    return books


def ws(s):
    return re.sub(r'\s+', ' ', s)


def clean_items(items):
    """Collapse white space; strip at the edges of the paragraph. Returns a new list."""
    out = []
    for it in items:
        if it[0] == 't':
            s = ws(it[1])
            if out and out[-1][0] == 't' and out[-1][2] == it[2]:
                out[-1] = ('t', out[-1][1] + s, it[2])
            else:
                out.append(('t', s, it[2]))
        else:
            out.append(it)
    # strip leading space
    for k in range(len(out)):
        if out[k][0] == 't':
            out[k] = ('t', out[k][1].lstrip(), out[k][2])
            if out[k][1]:
                break
        elif out[k][0] == 'v':
            continue
        else:
            break
    for k in range(len(out) - 1, -1, -1):
        if out[k][0] == 't':
            out[k] = ('t', out[k][1].rstrip(), out[k][2])
            if out[k][1]:
                break
        else:
            break
    # a verse number never keeps a space before the text that follows it: drop
    # leading spaces directly after verse markers; double spaces between runs
    res = []
    for k, it in enumerate(out):
        if it[0] == 't' and res and res[-1][0] == 'v':
            it = ('t', it[1].lstrip(), it[2])
        if it[0] == 't' and res and res[-1][0] == 't' and res[-1][1].endswith(' ') and it[1].startswith(' '):
            it = ('t', it[1].lstrip(), it[2])
        if it[0] == 't' and not it[1]:
            continue
        res.append(it)
    return res


def note_text(parts, keep=('ft', 'fq', 'fqa', 'fk', 'xt', 'xta', 'fl', 'fp', 'fv', 'xk', 'xq')):
    return ws(' '.join(t for k, t in parts if k in keep)).strip()


def note_ref(parts):
    for k, t in parts:
        if k in ('fr', 'xo'):
            return t.strip()
    return ''


def para_text(items):
    return ''.join(it[1] for it in items if it[0] == 't')


def roman(n, upper=True):
    vals = [(1000, 'M'), (900, 'CM'), (500, 'D'), (400, 'CD'), (100, 'C'), (90, 'XC'), (50, 'L'), (40, 'XL'),
            (10, 'X'), (9, 'IX'), (5, 'V'), (4, 'IV'), (1, 'I')]
    s = ''
    for v, r in vals:
        while n >= v:
            s += r
            n -= v
    return s if upper else s.lower()


def esc(s):
    return html.escape(s, quote=False)


def book_title(b):
    if b['mt']:
        return ' '.join(t for l, t in sorted(b['mt'], key=lambda x: x[0], reverse=False) if t) if False else \
            ' '.join(t for l, t in b['mt'])
    return b['meta'].get('toc1') or b['meta'].get('h') or b['code']


def short_name(b):
    return b['meta'].get('h') or b['meta'].get('toc2') or b['code']


HEADING = usfm.is_heading
