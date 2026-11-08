"""Make damaged COPIES of three documents in /root/a2-pdf/damaged (originals untouched).
(i) xref: every in-use offset in the classic xref table shifted by +13 bytes (startxref left correct)
(ii) trunc: file cut at the start of the final 'xref' table: all objects kept, no xref/trailer/startxref
(iii) length: the /Length of one body page's content stream halved (padded with spaces, so all byte
     offsets stay valid)"""
import json, re, os, sys
import pymupdf
DST = '/root/a2-pdf/damaged'
docs = {d['key']: d for d in json.load(open('/root/a2-pdf/out/docs.json'))}
out = []
for key in ['G13', 'grcsbl-rfpdf-mrk', 'G09']:
    d = docs[key]; data = open(d['pdf'], 'rb').read()
    xr = data.rfind(b'\nxref') + 1
    # (i)
    m = re.compile(rb'xref\s+(.*?)trailer', re.S).search(data, xr)
    table = m.group(1)
    def shift(mm):
        off = int(mm.group(1)); return b'%010d %s n' % (off + 13, mm.group(2))
    t2 = re.sub(rb'(\d{10}) (\d{5}) n', shift, table)
    d1 = data[:m.start(1)] + t2 + data[m.end(1):]
    assert len(d1) == len(data)
    # (ii)
    d2 = data[:xr]
    # (iii)
    doc = pymupdf.open(d['pdf']); pno = min(3, doc.page_count - 1)
    cx = doc[pno].get_contents()[0]
    om = re.search(rb'(?<![0-9])%d 0 obj' % cx, data)
    lm = re.compile(rb'/Length\s+(\d+)(?!\s+\d+\s+R)').search(data, om.end(), om.end() + 400)
    if not lm:
        raise SystemExit(f'{key}: no direct /Length near object {cx}')
    L = int(lm.group(1)); new = str(L // 2).encode().ljust(len(lm.group(1)), b' ')
    d3 = data[:lm.start(1)] + new + data[lm.end(1):]
    assert len(d3) == len(data)
    for tag, blob, note in [('xref', d1, 'xref offsets +13'), ('trunc', d2, f'cut at byte {xr} of {len(data)}'),
                            ('length', d3, f'page {pno+1} content stream obj {cx}: /Length {L} -> {L//2}')]:
        p = os.path.join(DST, f'{key}~{tag}.pdf'); open(p, 'wb').write(blob)
        e = dict(d); e.update(key=f'{key}~{tag}', pdf=p, damage=note, original=key); out.append(e)
        print(e['key'], note)
json.dump(out, open('/root/a2-pdf/out/damaged_docs.json', 'w'), indent=1)
