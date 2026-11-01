"""Independent crude strip of USFM verse text vs parser output (word sequence)."""
import sys, re, glob, os
sys.path.insert(0, '/root/aw-set-e/tools')
import usfm
def crude(path):
    s = open(path, encoding='utf-8-sig').read()
    s = re.sub(r'\\(f|fe|x|ef) .*?\\\1\*', ' ', s, flags=re.S)
    s = re.sub(r'\|[^\\]*?(?=\\\+?w\*)', '', s)
    out = {}; ch = None
    # chapter split
    for part in re.split(r'(\\c \S+)', s):
        m = re.match(r'\\c (\S+)', part)
        if m: ch = m.group(1); continue
        if ch is None: continue
        segs = re.split(r'\\v (\S+)', part)
        for i in range(1, len(segs), 2):
            t = segs[i+1]
            t = re.split(r'\\(?:s\d?|ms\d?|d|sp|r|mr|qa|cl|cp|b)\b[^\n]*', t)[0] if False else t
            t = re.sub(r'\n\\(s\d?|ms\d?|mr|r|d|sp|qa|cl|cp|is\d?|ip|mt\d?)( [^\n]*)?', ' ', t)
            t = re.sub(r'\\\+?[a-z]+\d*\*? ?', ' ', t)
            out[(ch, segs[i])] = t.split()
    return out
bad = 0; tot = 0
for d in sys.argv[1:]:
    for f in sorted(glob.glob(d + '/*.usfm')):
        b = usfm.parse_file(f)
        c = crude(f)
        for ch, v, t in usfm.verses(b):
            tot += 1
            if ''.join(t.split()) != ''.join(c.get((ch, v)) or ['<missing>']):
                bad += 1
                if bad < 400 and not os.path.basename(f).startswith("23-SNG"): print(os.path.basename(f), ch, v, '\n P:', t[:200], '\n C:', ' '.join(c.get((ch, v)) or [])[:200])
print('verses', tot, 'mismatch', bad)
