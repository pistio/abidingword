#!/usr/bin/env python3
"""Convert a scrollmapper/bible_databases JSON file (formats/json/<T>.json) to
reference USFM, one file per book, written into a ZIP.

usage: scroll2usfm.py SRC.json LANG BOOKS OUT.zip LOG.txt [--pilcrow] [--title "..."]
  BOOKS: all | ot | nt | comma separated USFM codes (output keeps source order)
  --cp-greek : add \\cp with the Greek alphabetic chapter numeral the document prints
  --pilcrow : a verse text that begins with the pilcrow sign starts a new \\p
              paragraph and the sign itself is not kept (it is a paragraph mark,
              not a word); otherwise every chapter is one \\p paragraph.

Verse text is copied unchanged except: leading/trailing white space removed and
runs of white space collapsed to one space. Verses whose source text is empty
are left out (the documents do not print them) and are listed in LOG.
Book titles come from booknames.NAMES[LANG] (the source has none).
"""
import json, sys, re, zipfile, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from booknames import SCROLL_CODES, NAMES, OT, NT


def greek_numeral(n):
    """Greek alphabetic numeral with keraia, modern form (6 = ΣΤ)."""
    units = ['', 'Α', 'Β', 'Γ', 'Δ', 'Ε', 'ΣΤ', 'Ζ', 'Η', 'Θ']
    tens = ['', 'Ι', 'Κ', 'Λ', 'Μ', 'Ν', 'Ξ', 'Ο', 'Π', 'Ϟ']
    hund = ['', 'Ρ', 'Σ', 'Τ', 'Υ', 'Φ', 'Χ', 'Ψ', 'Ω', 'Ϡ']
    return hund[n // 100 % 10] + tens[n // 10 % 10] + units[n % 10] + '\u0374'


def main():
    a = sys.argv[1:]
    src, lang, books, out, logp = a[:5]
    pilcrow = '--pilcrow' in a
    greek_cp = '--cp-greek' in a
    d = json.load(open(src, encoding='utf-8'))
    if books == 'all':
        want = None
    elif books == 'ot':
        want = set(OT)
    elif books == 'nt':
        want = set(NT)
    else:
        want = set(books.split(','))
    log = []
    files = []
    idx = 0
    for b in d['books']:
        code = SCROLL_CODES[b['name']]
        if want is not None and code not in want:
            continue
        idx += 1
        short, full = NAMES[lang][code]
        L = ['\\id %s %s' % (code, d['translation'].split(':')[0]), '\\usfm 3.0', '\\h ' + short,
             '\\toc1 ' + full, '\\toc2 ' + short, '\\toc3 ' + short, '\\mt1 ' + full]
        for c in b['chapters']:
            L.append('\\c %d' % c['chapter'])
            if greek_cp:
                L.append('\\cp ' + greek_numeral(c['chapter']))
            L.append('\\p')
            first = True
            for v in c['verses']:
                t = re.sub(r'\s+', ' ', v['text']).strip()
                if not t:
                    log.append('%s %d:%d empty in source - omitted' % (code, c['chapter'], v['verse']))
                    continue
                if pilcrow:
                    if t.startswith('¶'):
                        t = t[1:].strip()
                        if not first:
                            L.append('\\p')
                    if '¶' in t:
                        log.append('%s %d:%d pilcrow inside verse kept' % (code, c['chapter'], v['verse']))
                L.append('\\v %d %s' % (v['verse'], t))
                first = False
        files.append(('%02d-%s.usfm' % (idx, code), '\n'.join(L) + '\n'))
    with zipfile.ZipFile(out, 'w', zipfile.ZIP_DEFLATED) as z:
        for name, txt in files:
            zi = zipfile.ZipInfo(name, date_time=(2026, 11, 1, 0, 0, 0))
            zi.compress_type = zipfile.ZIP_DEFLATED
            z.writestr(zi, txt.encode('utf-8'))
    with open(logp, 'w', encoding='utf-8') as f:
        f.write('source %s\nbooks %d\n' % (os.path.basename(src), len(files)))
        f.write('\n'.join(log) + '\n')
    print(out, len(files), 'books;', len(log), 'log lines')


if __name__ == '__main__':
    main()
