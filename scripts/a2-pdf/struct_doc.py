"""Geometry/structure availability per engine for one document (counts).
usage: python3 -I struct_doc.py <key>"""
import sys, os, json, re, subprocess
sys.path.insert(0, '/root/a2-pdf/scripts')
from run_doc import run, PY
OUT = '/root/a2-pdf/out'; TMP = '/root/a2-pdf/tmp'
key = sys.argv[1]
d = {x['key']: x for x in json.load(open(OUT + '/docs.json'))}[key]
pdf = d['pdf']; res = {}
for e in ['pdfium', 'mupdf'] + (['pdfminer'] if '--pdfminer' in sys.argv else []):
    o = f'{TMP}/{key}.{e}.struct.json'
    info = run(PY + [e, 'struct', pdf, o], timeout=1800)
    try: res[e] = json.load(open(o)); os.remove(o)
    except Exception: res[e] = {'ok': False, 'error': info['stderr_tail'][-300:]}
# poppler: pdfinfo, pdftotext -bbox-layout (word boxes), pdftohtml -xml (fonts per span, links, outline)
p = {}
inf = subprocess.run(['pdfinfo', pdf], capture_output=True, text=True, errors='replace')
for line in inf.stdout.splitlines():
    k, _, v = line.partition(':')
    if k in ('Tagged', 'Pages', 'Producer', 'Creator', 'PDF version', 'Page size'): p[k] = v.strip()
p['pdfinfo_stderr'] = inf.stderr[-300:]
st = subprocess.run(['pdfinfo', '-struct', pdf], capture_output=True, text=True, errors='replace')
p['struct_lines'] = len([l for l in st.stdout.splitlines() if l.startswith(' ')])
bb = f'{TMP}/{key}.bbox.html'
subprocess.run(['pdftotext', '-bbox-layout', '-enc', 'UTF-8', pdf, bb], capture_output=True)
if os.path.exists(bb):
    s = open(bb, encoding='utf-8', errors='replace').read()
    p['bbox_words'] = s.count('<word '); p['bbox_lines'] = s.count('<line '); p['bbox_blocks'] = s.count('<block ')
    os.remove(bb)
xml = f'{TMP}/{key}.p2h'
subprocess.run(['pdftohtml', '-xml', '-i', '-q', '-nodrm', pdf, xml], capture_output=True)
if os.path.exists(xml + '.xml'):
    s = open(xml + '.xml', encoding='utf-8', errors='replace').read()
    p['xml_fontspecs'] = s.count('<fontspec '); p['xml_text_spans'] = s.count('<text ')
    p['xml_links'] = len(re.findall(r'<a href=', s)); p['xml_outline_items'] = s.count('<item')
    os.remove(xml + '.xml')
for f in os.listdir(TMP):
    if f.startswith(key + '.p2h'): os.remove(os.path.join(TMP, f))
res['poppler'] = p
os.makedirs(OUT + '/struct', exist_ok=True)
json.dump(res, open(f'{OUT}/struct/{key}.json', 'w'), indent=1, ensure_ascii=False)
print(key, json.dumps({k: {kk: vv for kk, vv in v.items() if kk not in ('trace',)} for k, v in res.items()})[:1500])
