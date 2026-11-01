#!/usr/bin/env python3
"""F09: The Text-Critical Greek New Testament (Byzantine Text), as Markdown.
Generator: this script (plain Python, CommonMark + footnote extension syntax).
Front matter and introduction (with its sigla table) from the edition's FRT/INT
files; '# BOOK' headings, '## N' chapter headings, '### ...' section headings
(English), verse numbers as <sup>N</sup>, textual-variant notes as Markdown
footnotes [^n] collected at the end of every chapter, OT quotations (\\q1) as
block-quote lines, the final 'Ἀμήν' paragraphs centred with <div align>."""
import os, sys, zipfile, re
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import usfm
from common import load_zip, clean_items, DOCS, ROOT, short_name, note_text, ws

REF = os.path.join(DOCS, 'F09-grc-tcgnt.usfm.zip')
SRC = os.path.join(ROOT, 'src', 'eb', 'grctcgnt_usfm.zip')
OUT = os.path.join(DOCS, 'F09-grc-tcgnt.md')


def mdesc(s):
    s = s.replace('\\', '\\\\')
    for ch in '*_[]<>`':
        s = s.replace(ch, '\\' + ch)
    return s


class Notes:
    def __init__(self):
        self.n = 0
        self.pending = []

    def add(self, parts):
        self.n += 1
        key = str(self.n)
        ref = ''
        for k, t in parts:
            if k in ('fr', 'xo'):
                ref = t
        body = note_text(parts)
        self.pending.append('[^%s]: %s%s' % (key, (ref + ' ') if ref else '', mdesc(body)))
        return '[^%s]' % key

    def flush(self, out):
        if self.pending:
            out.append('')
            out.extend(self.pending)
            out.append('')
            self.pending = []


def inline(items, notes):
    s = []
    for it in clean_items(items):
        if it[0] == 'v':
            s.append('<sup>%s</sup>' % it[1])
        elif it[0] == 't':
            t = mdesc(it[1])
            st = it[2]
            if ('it' in st or 'bdit' in st) and t.strip():
                lead = t[:len(t) - len(t.lstrip())]
                trail = t[len(t.rstrip()):]
                t = lead + '*' + t.strip() + '*' + trail
            if 'bd' in st and t.strip():
                lead = t[:len(t) - len(t.lstrip())]
                trail = t[len(t.rstrip()):]
                t = lead + '**' + t.strip() + '**' + trail
            if 'sup' in st:
                t = '<sup>%s</sup>' % t
            s.append(t)
        elif it[0] == 'n':
            s.append(notes.add(it[3]))
    out = ''.join(s)
    # a line must not start like a list item or heading
    if re.match(r'^(\d+[.)]\s|[#>+\-]\s)', out):
        out = '\\' + out
    return out


def front(out, notes):
    with zipfile.ZipFile(SRC) as z:
        for fn in ('00-FRTgrctcgnt.usfm', '01-INTgrctcgnt.usfm'):
            b = usfm.parse_usfm(z.read(fn).decode('utf-8-sig'))
            for lvl, t in b['mt']:
                out.append('%s %s' % ('#' if lvl == 1 else '##', mdesc(t)))
                out.append('')
            for bl in b['blocks']:
                if bl['kind'] == 'table':
                    out.append('| Siglum | Edition |')
                    out.append('|---|---|')
                    for r in bl['rows']:
                        cells = inline(r['items'], notes).split('\u241f')
                        cells = [c.strip() for c in cells if c.strip()]
                        out.append('| ' + ' | '.join(cells) + ' |')
                    out.append('')
                    continue
                if bl['kind'] != 'para':
                    continue
                if bl['style'] == 'ib':
                    continue
                t = inline(bl['items'], notes)
                if not t.strip():
                    continue
                if bl['style'].startswith('is'):
                    out.append('### ' + t)
                else:
                    out.append(t)
                out.append('')
    notes.flush(out)
    out.append('---')
    out.append('')


def main():
    books = load_zip(REF)
    out = ['<!-- The Text-Critical Greek New Testament: Byzantine Text. Public domain (eBible.org grctcgnt). -->', '']
    notes = Notes()
    front(out, notes)
    for b in books:
        title = ' '.join(t for l, t in sorted(b['mt'], key=lambda x: -x[0]))
        out.append('# ' + mdesc(ws(title).strip() or short_name(b)))
        out.append('')
        quote = False
        for bl in b['blocks']:
            if bl['kind'] == 'chapter':
                notes.flush(out)
                out.append('## %s' % bl['n'])
                out.append('')
                continue
            if bl['kind'] != 'para':
                continue
            st = bl['style']
            t = inline(bl['items'], notes)
            if st == 'b':
                if quote:
                    out.append('')
                    quote = False
                continue
            if not t.strip():
                continue
            if st.startswith('s') and st[1:].isdigit() or st in ('ms', 'ms1', 'mr', 'r', 'd'):
                if quote:
                    out.append('')
                    quote = False
                out.append('### ' + t)
                out.append('')
            elif st.startswith('q'):
                out.append('> ' + t + '  ')
                quote = True
            elif st == 'pc':
                if quote:
                    out.append('')
                    quote = False
                out.append('<div align="center">%s</div>' % t)
                out.append('')
            else:
                if quote:
                    out.append('')
                    quote = False
                out.append(t)
                out.append('')
        if quote:
            out.append('')
        notes.flush(out)
        out.append('---')
        out.append('')
    with open(OUT, 'w', encoding='utf-8', newline='\n') as f:
        f.write('\n'.join(out))
    print(OUT)


if __name__ == '__main__':
    main()
