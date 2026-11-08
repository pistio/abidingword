"""Clean memory/time measurement of pdfminer.six on a few documents (its batch RSS figures were inflated
by the parent process; see report)."""
import sys, os, json
sys.path.insert(0, '/root/a2-pdf/scripts')
from run_doc import run, PY
OUT = '/root/a2-pdf/out'
docs = {d['key']: d for d in json.load(open(OUT + '/docs.json'))}
res = {}
for k in sys.argv[1:]:
    with open(docs[k]['pdf'], 'rb') as f: f.read()
    o = f'/root/a2-pdf/tmp/tp.{k}'
    r = run(PY + ['pdfminer', 'text', docs[k]['pdf'], o], timeout=1800); r.pop('stderr_tail', None)
    for p in (o, o + '.json'):
        if os.path.exists(p): os.remove(p)
    res[k] = r; print(k, r['seconds'], r['peak_rss_mb'], flush=True)
json.dump(res, open(OUT + '/timing_pdfminer.json', 'w'), indent=1)
