"""Clean timing pass: text extraction only (no scoring, nothing else of ours running), each engine in its
own process, file pre-read into the page cache. usage: python3 -I timing.py [keys...]"""
import sys, os, json
sys.path.insert(0, '/root/a2-pdf/scripts')
from run_doc import run, PY, DUMP
OUT = '/root/a2-pdf/out'; TMP = '/root/a2-pdf/tmp'
docs = json.load(open(OUT + '/docs.json'))
keys = sys.argv[1:] or [d['key'] for d in docs]
tp = OUT + '/timing_clean.json'
res = json.load(open(tp)) if os.path.exists(tp) else {}
for d in docs:
    if d['key'] not in keys or d['key'] in res: continue
    with open(d['pdf'], 'rb') as f: f.read()
    r = {}; o = f"{TMP}/t.{d['key']}"
    env = dict(os.environ, AW_DUMP_FILE=d['pdf'], AW_DUMP_UNITS='0-10000000', RUST_BACKTRACE='0')
    with open(o, 'wb') as f:
        r['current'] = run([DUMP, '--ignored', '--nocapture'], stdout=f, cwd='/home/user/abidingword/src-tauri', env=env, timeout=2400)
    r['pdfium'] = run(PY + ['pdfium', 'text', d['pdf'], o], timeout=1200)
    r['mupdf'] = run(PY + ['mupdf', 'text', d['pdf'], o], timeout=1200)
    r['poppler'] = run(['pdftotext', '-enc', 'UTF-8', d['pdf'], o], timeout=1200)
    for x in r.values(): x.pop('stderr_tail', None)
    for p in (o, o + '.json'):
        if os.path.exists(p): os.remove(p)
    res[d['key']] = r
    json.dump(res, open(tp, 'w'), indent=1)
    print(d['key'], {k: (v['seconds'], v['peak_rss_mb']) for k, v in r.items()}, flush=True)
