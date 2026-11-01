"""Small USFM reader used by all set-F renderers.

parse_usfm(text) -> book dict:
  code, meta{h,toc1,toc2,toc3,id_rest}, mt[(level,text)], book_cl (label used for
  every chapter, if \\cl comes before the first \\c), blocks[...]

Block kinds:
  {'kind':'chapter','n':'3','cp':None|'ג','cl':None|'Psalm 3'}
  {'kind':'para','style':'p'|'q1'|'s1'|...,'items':[...]}
  {'kind':'table','rows':[[cellitems,...],...]}
Items inside a para:
  ('v', '12', display)                      verse start (display = \\vp or None)
  ('t', 'text', ('add','nd',...))           text run with character styles
  ('n', 'f'|'x'|'fe', caller, [(part, text), ...])   note
Character markers are reduced to their text (attributes after '|' dropped).
Nothing that is verse text is ever discarded; only \\rem, \\fig, \\cat, \\esb and
attributes are skipped.
"""
import re

PARA = set('''p m pi pi1 pi2 pi3 pi4 mi nb pc pr pm pmo pmc pmr cls ph ph1 ph2 ph3
q q1 q2 q3 q4 qc qr qm qm1 qm2 qm3 qa qd li li1 li2 li3 li4 lim lim1 lim2 lh lf b d sp
s s1 s2 s3 s4 ms ms1 ms2 ms3 mr sr r sd sd1 sd2 mt mt1 mt2 mt3 mt4 mte mte1 mte2
imt imt1 imt2 imt3 is is1 is2 is3 ip ipi im imi ipq imq ipr ib ili ili1 ili2 io io1 io2 io3
iot ie iex imte h toc1 toc2 toc3 toca1 toca2 toca3 id ide rem cl cp sts restore periph
usfm tr c v'''.split())
NOTES = {'f', 'fe', 'x', 'ef', 'ex'}
NOTE_PARTS = set('fr ft fk fq fqa fl fw fp fv fdc fm xo xk xq xt xta xop xot xnt xdc'.split())
SKIP_CHAR = {'fig', 'cat', 'jmp_attr'}
ATTR_CHAR = {'w', 'rb', 'jmp', 'fig', 'xt', 'ref'}
TOKEN = re.compile(r'\\(\+?)([a-zA-Z]+[0-9]*)(\*?)')


def _ws(s):
    return re.sub(r'\s+', ' ', s)


def parse_usfm(text):
    book = {'code': None, 'meta': {}, 'mt': [], 'book_cl': None, 'blocks': []}
    pos = 0
    toks = []
    for m in TOKEN.finditer(text):
        if m.start() > pos:
            toks.append(('txt', text[pos:m.start()]))
        toks.append(('mk', m.group(2), m.group(1) == '+', m.group(3) == '*'))
        pos = m.end()
        # one whitespace character after an opening marker belongs to the marker
        if not m.group(3) and pos < len(text) and text[pos] in ' \t\r\n':
            pos += 1
    if pos < len(text):
        toks.append(('txt', text[pos:]))

    cur = None          # current para block
    stack = []          # char style stack
    note = None         # current note dict
    linemode = None     # ('meta', key) | ('mt', level) | ('cl',) | ('cp',) | ('skip',)
    linebuf = []
    last_chapter = None
    seen_chapter = False
    attr_cut = False    # inside an attribute (after '|') of the top char span
    vp_target = None
    vp_buf = None
    i = 0

    def flush_line():
        nonlocal linemode, linebuf
        if linemode is None:
            return
        s = _ws(''.join(linebuf)).strip()
        kind = linemode[0]
        if kind == 'meta':
            key = linemode[1]
            if key == 'id':
                parts = s.split(' ', 1)
                book['code'] = parts[0].strip()
                book['meta']['id_rest'] = parts[1] if len(parts) > 1 else ''
            else:
                book['meta'][key] = s
        elif kind == 'mt':
            book['mt'].append((linemode[1], s))
        elif kind == 'cl':
            if last_chapter is None:
                book['book_cl'] = s
            else:
                last_chapter['cl'] = s
        elif kind == 'cp':
            if last_chapter is not None:
                last_chapter['cp'] = s
        linemode = None
        linebuf = []

    def new_para(style):
        nonlocal cur, stack, attr_cut
        cur = {'kind': 'para', 'style': style, 'items': []}
        book['blocks'].append(cur)
        stack = []
        attr_cut = False

    def add_text(s):
        nonlocal attr_cut
        if not s:
            return
        if stack and stack[-1] in ATTR_CHAR and not attr_cut and '|' in s:
            s = s.split('|', 1)[0]
            attr_cut = True
        elif attr_cut:
            return
        if any(st in SKIP_CHAR for st in stack):
            return
        if cur is None:
            if not s.strip():
                return
            new_para('p')
        styles = tuple(stack)
        it = cur['items']
        if it and it[-1][0] == 't' and it[-1][2] == styles:
            it[-1] = ('t', it[-1][1] + s, styles)
        else:
            it.append(('t', s, styles))

    while i < len(toks):
        tk = toks[i]
        if tk[0] == 'txt':
            s = tk[1]
            if note is not None:
                if note['attr_cut']:
                    i += 1
                    continue
                if note['in_attr'] and '|' in s:
                    s = s.split('|', 1)[0]
                    note['attr_cut'] = True
                if note['caller'] is None:
                    st = s.lstrip()
                    if st:
                        note['caller'] = st[0]
                        s = st[1:]
                    else:
                        i += 1
                        continue
                note['parts'][-1][1].append(s)
            elif vp_buf is not None:
                vp_buf.append(s)
            elif linemode is not None:
                if linemode[0] in ('cl', 'cp', 'meta', 'mt', 'skip') and '\n' in s and linemode[0] != 'mt':
                    head, rest = s.split('\n', 1)
                    linebuf.append(head)
                    flush_line()
                    if rest.strip():
                        add_text(rest)
                else:
                    linebuf.append(s)
            else:
                add_text(s)
            i += 1
            continue
        _, name, plus, close = tk
        # notes
        if note is not None:
            if name in NOTES and close:
                for p in note['parts']:
                    pass
                parts = [(k, _ws(''.join(v)).strip()) for k, v in note['parts']]
                parts = [(k, v) for k, v in parts if v]
                if cur is None:
                    new_para('p')
                cur['items'].append(('n', note['kind'], note['caller'], parts))
                note = None
            elif name in NOTE_PARTS and not close:
                note['parts'].append([name, []])
                note['in_attr'] = False
                note['attr_cut'] = False
            elif name in ATTR_CHAR and not close:
                note['in_attr'] = True
                note['attr_cut'] = False
            elif close:
                note['in_attr'] = False
                note['attr_cut'] = False
            i += 1
            continue
        if vp_buf is not None:
            if name == 'vp' and close:
                disp = _ws(''.join(vp_buf)).strip()
                if vp_target is not None:
                    idx, item = vp_target
                    cur['items'][idx] = ('v', item[1], disp)
                vp_buf = None
            i += 1
            continue
        if name in NOTES and not close:
            flush_line()
            note = {'kind': name, 'caller': None, 'parts': [['ft', []]],
                    'in_attr': False, 'attr_cut': False}
            i += 1
            continue
        if name == 'c':
            flush_line()
            # chapter number is the next word
            n = None
            if i + 1 < len(toks) and toks[i + 1][0] == 'txt':
                s = toks[i + 1][1]
                m = re.match(r'\s*(\S+)', s)
                n = m.group(1)
                rest = s[m.end():]
                toks[i + 1] = ('txt', rest)
            last_chapter = {'kind': 'chapter', 'n': n, 'cp': None, 'cl': None}
            book['blocks'].append(last_chapter)
            seen_chapter = True
            cur = None
            stack = []
            i += 1
            continue
        if name == 'v':
            flush_line()
            n = None
            if i + 1 < len(toks) and toks[i + 1][0] == 'txt':
                s = toks[i + 1][1]
                m = re.match(r'\s*(\S+) ?', s)
                n = m.group(1)
                toks[i + 1] = ('txt', s[m.end():])
            if cur is None:
                new_para('p')
            stack = []
            attr_cut = False
            cur['items'].append(('v', n, None))
            vp_target = (len(cur['items']) - 1, cur['items'][-1])
            i += 1
            continue
        if name == 'vp' and not close:
            vp_buf = []
            i += 1
            continue
        if name in ('ca', 'va'):
            # alternate numbering: skip content until closing marker
            j = i + 1
            while j < len(toks) and not (toks[j][0] == 'mk' and toks[j][1] == name and toks[j][3]):
                j += 1
            i = j + 1
            continue
        if name in PARA and not plus:
            flush_line()
            if name in ('id', 'h', 'toc1', 'toc2', 'toc3', 'toca1', 'toca2', 'toca3', 'ide', 'usfm', 'sts'):
                linemode = ('meta', name)
                cur = None
            elif name in ('rem', 'restore', 'periph'):
                linemode = ('skip',)
                cur = None
            elif name.startswith('mt') and not seen_chapter and name != 'mte':
                lvl = int(name[2:] or 1)
                linemode = ('mt', lvl)
                cur = None
            elif name == 'cl':
                linemode = ('cl',)
                cur = None
            elif name == 'cp':
                linemode = ('cp',)
                cur = None
            elif name == 'tr':
                if not book['blocks'] or book['blocks'][-1]['kind'] != 'table':
                    book['blocks'].append({'kind': 'table', 'rows': []})
                row = {'kind': 'para', 'style': 'tr', 'items': []}
                book['blocks'][-1]['rows'].append(row)
                cur = row
                stack = []
            else:
                new_para(name)
            i += 1
            continue
        # character markers
        if name in ('tc1', 'tc2', 'tc3', 'tc4', 'th1', 'th2', 'th3', 'tcr1', 'tcr2', 'tcr3', 'thr1', 'thr2'):
            add_text(' \u241f ')
            i += 1
            continue
        if close:
            if name in stack:
                while stack and stack[-1] != name:
                    stack.pop()
                if stack:
                    stack.pop()
            attr_cut = False
        else:
            if not plus and name in stack:
                pass
            stack.append(name)
            attr_cut = False
        i += 1
    flush_line()
    return book


HEADING_PREFIX = ('s', 'ms', 'mr', 'sr', 'r', 'd', 'sp', 'qa', 'cl', 'mt', 'imt', 'is', 'ip', 'im', 'io', 'iot', 'ib', 'ie', 'ili', 'iex', 'ipr', 'ipq', 'imq', 'ipi', 'imi', 'sd', 'mte', 'imte')


def is_heading(style):
    base = style.rstrip('0123456789')
    return base in HEADING_PREFIX


def verses_of(book):
    """Yield (chapter, verse, plain_text) - for checks."""
    ch = None
    out = []
    curv = None
    buf = []
    for b in book['blocks']:
        if b['kind'] == 'chapter':
            if curv:
                out.append((ch, curv, _ws(''.join(buf)).strip()))
            curv = None
            buf = []
            ch = b['n']
            continue
        rows = b['rows'] if b['kind'] == 'table' else [b]
        for r in rows:
            heading = is_heading(r['style']) and not any(it[0] == 'v' for it in r['items'])
            for it in r['items']:
                if heading:
                    continue
                if it[0] == 'v':
                    if curv:
                        out.append((ch, curv, _ws(''.join(buf)).strip()))
                    curv = it[1]
                    buf = []
                elif it[0] == 't' and curv:
                    buf.append(it[1])
            if curv:
                buf.append(' ')
    if curv:
        out.append((ch, curv, _ws(''.join(buf)).strip()))
    return out


def load(path):
    with open(path, encoding='utf-8-sig') as f:
        return parse_usfm(f.read())
