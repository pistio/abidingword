#!/usr/bin/env python3
"""Copy the USFM files of the listed books, byte for byte, from an eBible.org
<id>_usfm.zip into a reference ZIP. usage: reduce_ebible.py SRC.zip OUT.zip CODES|nt|ot|all"""
import sys, zipfile, re, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from booknames import OT, NT
src, out, sel = sys.argv[1:4]
want = set(OT + NT) if sel == 'all' else set(OT) if sel == 'ot' else set(NT) if sel == 'nt' else set(sel.split(','))
n = 0
with zipfile.ZipFile(src) as zi, zipfile.ZipFile(out, 'w', zipfile.ZIP_DEFLATED) as zo:
    for info in zi.infolist():
        m = re.match(r'\d+-([0-9A-Z]{3})', info.filename)
        if info.filename.endswith('.usfm') and m and m.group(1) in want:
            zo.writestr(info, zi.read(info.filename), compress_type=zipfile.ZIP_DEFLATED)
            n += 1
print(out, n, 'books')
