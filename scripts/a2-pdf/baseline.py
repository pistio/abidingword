"""Interpreter+module import baseline (null run) per engine, measured like the real runs."""
import sys, json; sys.path.insert(0, '/root/a2-pdf/scripts')
from run_doc import run, PY, DUMP
res = {}
for e in ['pdfium', 'mupdf', 'pdfminer']:
    runs = [run(PY + [e, 'null', 'x', '/root/a2-pdf/tmp/null.json']) for _ in range(3)]
    res[e] = {'seconds': min(r['seconds'] for r in runs), 'peak_rss_mb': max(r['peak_rss_mb'] for r in runs)}
r = [run(['pdftotext', '-v']) for _ in range(3)]
res['poppler'] = {'seconds': min(x['seconds'] for x in r), 'peak_rss_mb': max(x['peak_rss_mb'] for x in r)}
r = [run([DUMP, '--list'], cwd='/home/user/abidingword/src-tauri') for _ in range(3)]
res['current'] = {'seconds': min(x['seconds'] for x in r), 'peak_rss_mb': max(x['peak_rss_mb'] for x in r), 'note': 'test binary --list'}
json.dump(res, open('/root/a2-pdf/out/baseline.json', 'w'), indent=1); print(res)
