"""Small USFM reader used by the set-E generators.

Produces, per book:
  Book.code, Book.meta (h, toc1, toc2, toc3), Book.mt [(level, text)],
  Book.blocks: list of tuples
     ('c', number_str, label_or_None, published_number_or_None)
     ('h', marker, inlines)        headings: s1 s2 ms mr r d sp qa is ...
     ('p', marker, inlines)        paragraphs: p m q1 q2 pi li ip ... ; 'b' blank
  inlines: ('t', text, frozenset(styles)) | ('v', number) | ('n', kind, caller, [(marker, text)])
The text of the publisher is kept word for word; only USFM whitespace is
normalised (runs of whitespace become one space).
"""
import glob
import os
import re

PARA = {'p', 'm', 'mi', 'nb', 'pi', 'pi1', 'pi2', 'pi3', 'q', 'q1', 'q2', 'q3', 'q4', 'qr', 'qc', 'qm1', 'qm2',
        'li', 'li1', 'li2', 'li3', 'pc', 'pr', 'pm', 'pmo', 'cls', 'b', 'ip', 'ipi', 'im', 'imi', 'ipr', 'ipq', 'ili',
        'ili1', 'ili2', 'io', 'io1', 'io2', 'tr', 'pmr', 'ph', 'ph1'}
HEAD = {'s', 's1', 's2', 's3', 's4', 'ms', 'ms1', 'ms2', 'mr', 'sr', 'r', 'd', 'sp', 'qa', 'is', 'is1', 'is2',
        'imt', 'imt1', 'imt2', 'iot', 'mte', 'mte1', 'sd'}
META = {'h', 'toc1', 'toc2', 'toc3', 'ide', 'rem', 'sts', 'usfm'}
CHAR = {'add', 'nd', 'em', 'it', 'bd', 'bdit', 'k', 'bk', 'qs', 'sc', 'tl', 'pn', 'wj', 'qt', 'sig', 'ord', 'no',
        'ior', 'iqt', 'qac', 'png', 'addpn', 'dc', 'sls', 'rq', 'w', 'wg', 'wh', 'wa', 'rb', 'ref'}
NOTE = {'f', 'fe', 'x', 'ef'}

TOKEN = re.compile(r'\\(\+?)([a-z]+[0-9]*)(\*?)')


class Book:
    def __init__(self, code):
        self.code = code
        self.meta = {}
        self.mt = []
        self.blocks = []
        self.chapter_label = None  # \cl before first \c

    @property
    def name(self):
        return self.meta.get('toc2') or self.meta.get('h') or self.code

    @property
    def long_name(self):
        return self.meta.get('toc1') or self.name

    def chapters(self):
        return [b for b in self.blocks if b[0] == 'c']


def _clean_w(s):
    # \w word|attr\w*  -> word   (also \+w)
    return re.sub(r'\\(\+?)w ([^\\|]*?)(\|[^\\]*?)?\\\1w\*', lambda m: m.group(2), s)


def parse_file(path):
    return _parse(path)


def _parse(path):
    # simpler, explicit implementation built on top of a line pre-split
    raw = open(path, encoding='utf-8-sig').read()
    raw = _clean_w(raw)
    raw = re.sub(r'\|[a-zA-Z\-]+="[^"\\]*"(\s+[a-zA-Z\-]+="[^"\\]*")*(?=\\)', '', raw)
    # Put every line-level marker on its own line so heading/meta arguments end cleanly.
    linemk = sorted(PARA | HEAD | META | {'mt', 'mt1', 'mt2', 'mt3', 'mt4', 'c', 'cl', 'cp', 'id', 'periph'},
                    key=len, reverse=True)
    raw = re.sub(r'(?<!\n)\\(' + '|'.join(linemk) + r')(?=[\s])', lambda m: '\n\\' + m.group(1), raw)
    lines = raw.split('\n')
    book = None
    cur = None
    styles = []
    note = None
    last_c = None
    for line in lines:
        m = re.match(r'\\([a-z]+[0-9]*)(?:\s+|$)(.*)$', line)
        mk = m.group(1) if m else None
        body = m.group(2) if m else line
        if mk == 'id':
            book = Book(body.split()[0].upper())
            continue
        if book is None:
            continue
        if mk in META:
            if mk not in ('rem', 'ide', 'usfm', 'sts'):
                book.meta[mk] = body.strip()
            continue
        if mk and re.fullmatch(r'mt[0-9]?', mk):
            book.mt.append((int(mk[2:] or 1), body.strip()))
            continue
        if mk == 'c':
            styles.clear()
            parts = body.split(None, 1)
            last_c = ['c', parts[0], None, None]
            book.blocks.append(last_c)
            cur = None
            body = parts[1] if len(parts) > 1 else ''
            if not body.strip():
                continue
            mk = None
        elif mk == 'cl':
            if last_c is None:
                book.chapter_label = body.strip()
            else:
                last_c[2] = body.strip()
            continue
        elif mk == 'cp':
            if last_c is not None:
                last_c[3] = body.strip()
            continue
        if mk in HEAD:
            styles.clear()
            cur = []
            book.blocks.append(('h', mk, cur))
            cur, styles, note = _inline(body, cur, styles, note, book)
            cur = None
            continue
        if mk and (mk in PARA or re.fullmatch(r'(q|li|pi|ph)[0-9]?', mk)):
            styles.clear()
            cur = []
            book.blocks.append(('p', mk, cur))
            cur, styles, note = _inline(body, cur, styles, note, book)
            continue
        if mk and mk not in CHAR and mk not in NOTE and mk != 'v' and not mk.startswith('t'):
            # unknown line marker: new paragraph
            styles.clear()
            cur = []
            book.blocks.append(('p', mk, cur))
            cur, styles, note = _inline(body, cur, styles, note, book)
            continue
        cur, styles, note = _inline(line, cur, styles, note, book)
    for b in book.blocks:
        if b[0] == 'c':
            pass
    book.blocks = [tuple(b) if isinstance(b, list) else b for b in book.blocks]
    # drop empty paragraphs (except b)
    out = []
    for b in book.blocks:
        if b[0] in ('p', 'h'):
            _tidy(b[2])
            if not b[2] and b[1] != 'b':
                continue
        out.append(b)
    book.blocks = out
    return book


def _tidy(inl):
    # collapse whitespace; trim paragraph edges
    for j, it in enumerate(inl):
        if it[0] == 't':
            inl[j] = ('t', re.sub(r'\s+', ' ', it[1]), it[2])
    while inl and inl[0][0] == 't' and not inl[0][1].strip():
        inl.pop(0)
    if inl and inl[0][0] == 't':
        inl[0] = ('t', inl[0][1].lstrip(), inl[0][2])
    while inl and inl[-1][0] == 't' and not inl[-1][1].strip():
        inl.pop()
    if inl and inl[-1][0] == 't':
        inl[-1] = ('t', inl[-1][1].rstrip(), inl[-1][2])


def _inline(s, cur, styles, note, book):
    s = s + ' '
    pos = 0
    def ensure():
        nonlocal cur
        if cur is None:
            cur = []
            book.blocks.append(('p', 'm', cur))
    def add(t):
        nonlocal cur, note
        if not t:
            return
        if note is not None:
            parts = note[2]
            if parts and parts[-1][0] == note[3]:
                parts[-1] = (note[3], parts[-1][1] + t)
            else:
                parts.append((note[3], t))
            return
        if cur is None and not t.strip():
            return
        ensure()
        st = frozenset(styles)
        if cur and cur[-1][0] == 't' and cur[-1][2] == st:
            cur[-1] = ('t', cur[-1][1] + t, st)
        else:
            cur.append(('t', t, st))
    for m in TOKEN.finditer(s):
        if m.start() < pos:
            continue
        add(s[pos:m.start()])
        plus, mk, star = m.group(1), m.group(2), m.group(3)
        pos = m.end()
        if not star and s[pos:pos + 1] in (' ', '\t'):
            pos += 1
        if mk in NOTE:
            if star:
                if note is not None:
                    ensure()
                    kind, caller, parts, _ = note
                    cur.append(('n', kind, caller, [(a, re.sub(r'\s+', ' ', b).strip()) for a, b in parts if b.strip()]))
                    note = None
            else:
                mm = re.match(r'(\S+)\s?', s[pos:])
                caller = mm.group(1) if mm else '+'
                pos += mm.end() if mm else 0
                note = (mk, caller, [], 'ft' if mk.startswith('f') or mk == 'ef' else 'xt')
            continue
        if note is not None:
            if not star and not plus and mk not in CHAR:
                note = (note[0], note[1], note[2], mk)
            continue
        if mk == 'v':
            mm = re.match(r'(\S+)\s?', s[pos:])
            ensure()
            cur.append(('v', mm.group(1)))
            pos += mm.end()
            continue
        if mk in CHAR or plus:
            if star:
                for j in range(len(styles) - 1, -1, -1):
                    if styles[j] == mk:
                        del styles[j]
                        break
            else:
                styles.append(mk)
            continue
        if mk.startswith('tc') or mk.startswith('th'):
            if not star:
                add('\t')
            continue
        # unexpected marker inside a line: ignore marker
    add(s[pos:])
    return cur, styles, note


def load(dirpath, codes=None):
    books = {}
    for f in sorted(glob.glob(os.path.join(dirpath, '*.usfm')) + glob.glob(os.path.join(dirpath, '*.SFM'))):
        b = parse_file(f)
        books[b.code] = b
    return books


CANON = ('FRT GEN EXO LEV NUM DEU JOS JDG RUT 1SA 2SA 1KI 2KI 1CH 2CH EZR NEH EST JOB PSA PRO ECC SNG ISA JER LAM EZK '
         'DAN HOS JOL AMO OBA JON MIC NAM HAB ZEP HAG ZEC MAL TOB JDT ESG WIS SIR BAR LJE S3Y SUS BEL 1MA 2MA 1ES MAN '
         'PS2 3MA 2ES 4MA DAG MAT MRK LUK JHN ACT ROM 1CO 2CO GAL EPH PHP COL 1TH 2TH 1TI 2TI TIT PHM HEB JAS 1PE 2PE '
         '1JN 2JN 3JN JUD REV GLO').split()
NT = 'MAT MRK LUK JHN ACT ROM 1CO 2CO GAL EPH PHP COL 1TH 2TH 1TI 2TI TIT PHM HEB JAS 1PE 2PE 1JN 2JN 3JN JUD REV'.split()


def ordered(books, codes=None):
    order = codes or CANON
    return [books[c] for c in order if c in books]


def plain(inlines, notes=False):
    out = []
    for it in inlines:
        if it[0] == 't':
            out.append(it[1])
        elif it[0] == 'v':
            out.append(' ')
    return re.sub(r'\s+', ' ', ''.join(out)).strip()


def note_text(parts, with_ref=False):
    """Note body. Leading \\fr / \\xo reference is returned separately."""
    ref = ''
    body = []
    for mk, t in parts:
        if mk in ('fr', 'xo'):
            ref = t.strip()
        else:
            body.append(t)
    txt = re.sub(r'\s+', ' ', ' '.join(x.strip() for x in body)).strip()
    return ref, txt


def verses(book):
    """Yield (chapter, verse, text) — used only for self-checks."""
    ch = None
    v = None
    buf = []
    for b in book.blocks:
        if b[0] == 'c':
            if v:
                yield ch, v, ' '.join(buf)
            ch, v, buf = b[1], None, []
        elif b[0] == 'p':
            for it in b[2]:
                if it[0] == 'v':
                    if v:
                        yield ch, v, ' '.join(buf)
                    v, buf = it[1], []
                elif it[0] == 't' and v:
                    buf.append(it[1])
    if v:
        yield ch, v, ' '.join(buf)
