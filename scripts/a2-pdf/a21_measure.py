"""Phase A2.1: the current PDF backend against PDFium glyphs through the app's
own layout code, on A2's 57 disclosed PDFs (same inputs, checksums verified).

For each document and each path, in its own process:
  * text fidelity and reading order: the app's unit dump scored by A2's
    score.py against the edition's reference (coverage, in-order share);
  * recognition: the real-document harness (verified verses, verified verses
    with wrong text, verified verses not in the edition), run separately;
  * wall time and peak RSS of the dump process (os.wait4: for pdfium-layout
    the larger of the app's and the helper's peaks, not their sum), and the
    helper alone on the same file (its own time and peak).

usage: python3 -I a21_measure.py <dump-binary> <helper> <libdir> <out-dir> [keys]
"""
import sys, os, json, gzip
sys.path.insert(0, '/root/a2-pdf/scripts')
import score as S
import run_doc as R

def main():
    dump, helper, libdir, out = sys.argv[1:5]
    only = set(sys.argv[5].split(',')) if len(sys.argv) > 5 else None
    docs = json.load(open('/root/a2-pdf/out/docs.json'))
    os.makedirs(out, exist_ok=True)
    for d in docs:
        key = d['key']
        if only and key not in only:
            continue
        books = None
        if d.get('audit_json') and os.path.exists(d['audit_json']):
            a = json.load(open(d['audit_json'])); books = [b['code'] for b in a['books'] if b.get('expected', 0) > 0]
        verses = S.reference(d['ref'], books, key)
        hebrew = d['script'].lower().startswith('hebr') or d['lang'] in ('he', 'heb', 'hbo')
        res = {'key': key, 'sha256': d['sha256'], 'ref_verses': len(verses), 'engines': {}}
        for eng in ('current', 'pdfium-layout'):
            env = dict(os.environ, AW_DUMP_FILE=d['pdf'], AW_DUMP_UNITS='0-10000000', RUST_BACKTRACE='0')
            if eng == 'pdfium-layout':
                env.update(AW_PDF_ENGINE='pdfium', AW_PDFIUM_HELPER=helper, AW_PDFIUM_LIB=libdir)
            tmp = os.path.join(out, f'{key}.{eng}.dump')
            with open(tmp, 'wb') as f:
                info = R.run([dump, '--ignored', '--nocapture'], stdout=f, cwd='/home/user/abidingword/src-tauri',
                             env=env, timeout=2400)
            pages = {}
            text = open(tmp, encoding='utf-8', errors='replace').read()
            for line in text.splitlines():
                m = R.UNIT.match(line)
                if m:
                    pages.setdefault(int(m.group(2)), []).append(m.group(5))
            ok = info['rc'] == 0 and 'test result: ok' in text[-3000:]
            lines = [l for k in sorted(pages) for l in pages[k]]
            r = {'run': info, 'ok': ok, 'pages_with_text': len(pages)}
            if lines:
                r['score'] = S.score(lines, verses, rtl=hebrew, hebrew=hebrew)
            if not ok:
                r['error'] = (info['stderr_tail'] + text[-1500:])[-800:]
            with gzip.open(os.path.join(out, f'{key}.{eng}.dump.gz'), 'wb') as g:
                g.write(text.encode('utf-8'))
            os.remove(tmp)
            res['engines'][eng] = r
            sc = r.get('score', {}).get('nfc', {})
            print(key, eng, 'ok' if ok else 'FAIL', info['seconds'], 's', info['peak_rss_mb'], 'MB',
                  'cov', sc.get('coverage'), 'ord', sc.get('in_order_share'), flush=True)
        h = R.run([helper, libdir, d['pdf'], '--max-chars', '40000000', '--max-memory-mb', '4096'], timeout=1800)
        res['helper_alone'] = {k: h[k] for k in ('rc', 'seconds', 'cpu_seconds', 'peak_rss_mb', 'timeout')}
        json.dump(res, open(os.path.join(out, key + '.json'), 'w'), ensure_ascii=False, indent=1)

if __name__ == '__main__':
    main()
