#!/usr/bin/env python3
"""List oddities of a reference USFM zip: verse-number gaps and repeats, empty
verses, verse texts that contain digits or verse-like numbers, chapter counts.
usage: scan_ref.py REF.zip"""
import sys, os, re
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import usfm
from common import load_zip


def vnum(v):
    m = re.match(r'(\d+)', v)
    return int(m.group(1)) if m else None


def main():
    books = load_zip(sys.argv[1], order='file')
    tot = 0
    for b in books:
        vs = usfm.verses_of(b)
        tot += len(vs)
        chs = [bl['n'] for bl in b['blocks'] if bl['kind'] == 'chapter']
        issues = []
        prev = {}
        for c, v, t in vs:
            n = vnum(v)
            p = prev.get(c)
            if p is None:
                if n != 1:
                    issues.append('%s:%s first verse of chapter is %s' % (c, v, v))
            else:
                if n is None or n != p + 1:
                    issues.append('%s:%s follows %s' % (c, v, p))
            prev[c] = n if n is not None else (p or 0)
            if not v.isdigit():
                issues.append('%s:%s non-numeric verse label' % (c, v))
            if not t.strip():
                issues.append('%s:%s empty' % (c, v))
            elif re.search(r'\d', t):
                issues.append('%s:%s digits in text: %s' % (c, v, re.sub(r'\s+', ' ', t)[:70]))
        print('%s chapters=%d verses=%d' % (b['code'], len(chs), len(vs)))
        for i in issues[:40]:
            print('   ', i)
        if len(issues) > 40:
            print('    ... %d more' % (len(issues) - 40))
    print('TOTAL verses', tot)


if __name__ == '__main__':
    main()
