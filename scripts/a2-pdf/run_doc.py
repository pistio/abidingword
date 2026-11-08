"""Run every engine on one document (each in its own process), measure wall time and peak RSS,
score the text.  usage: python3 -I run_doc.py <key> [engines,comma,separated]"""
import sys, os, json, time, subprocess, signal, threading, gzip, re
sys.path.insert(0, '/root/a2-pdf/scripts')
import score as S

OUT = '/root/a2-pdf/out'; TXT = OUT + '/text'; TMP = '/root/a2-pdf/tmp'
DUMP = '/tmp/claude-0/-home-user/3283e166-e2af-5804-b13d-4343144cf2ee/scratchpad/dump-ea5f944'
PY = ['python3', '-I', '/root/a2-pdf/scripts/extract.py']
TIMEOUT = {'current': 2400, 'pdfminer': 1800}
DEFAULT = ['current', 'pdfium', 'mupdf', 'poppler', 'mupdf:sort', 'poppler:raw']

def run(cmd, stdout=None, cwd=None, env=None, timeout=1200):
    """Start cmd, wait with os.wait4 to get this child's own peak RSS; returns dict."""
    t0 = time.perf_counter()
    p = subprocess.Popen(cmd, stdout=stdout or subprocess.DEVNULL, stderr=subprocess.PIPE, cwd=cwd, env=env)
    err_chunks = []
    th = threading.Thread(target=lambda: err_chunks.append(p.stderr.read())); th.start()
    killed = [False]
    def kill():
        killed[0] = True
        try: os.kill(p.pid, signal.SIGKILL)   # our own child, by PID
        except ProcessLookupError: pass
    tm = threading.Timer(timeout, kill); tm.start()
    _, status, ru = os.wait4(p.pid, 0)
    wall = time.perf_counter() - t0
    tm.cancel(); th.join(); p.returncode = os.waitstatus_to_exitcode(status)
    err = (err_chunks[0] if err_chunks else b'').decode('utf-8', 'replace')
    return {'rc': p.returncode, 'seconds': round(wall, 3), 'cpu_seconds': round(ru.ru_utime + ru.ru_stime, 3),
            'peak_rss_mb': round(ru.ru_maxrss / 1024, 1), 'timeout': killed[0], 'stderr_tail': err[-1500:]}

UNIT = re.compile(r'^\s*(\d+) p(\d+)\s+s(\d+)\s+(\S{8}) ?(.*)$')

def engine_text(eng, d):
    """Returns (run_info, pages: list[list[str]] of lines, meta)"""
    pdf = d['pdf']; key = d['key']; base = eng.split(':')[0]
    os.makedirs(TMP, exist_ok=True)
    tmp = os.path.join(TMP, f'{key}.{eng.replace(":", "_")}.txt')
    meta = {}
    if base == 'current':
        env = dict(os.environ, AW_DUMP_FILE=pdf, AW_DUMP_UNITS='0-10000000', RUST_BACKTRACE='0')
        with open(tmp, 'wb') as f:
            info = run([DUMP, '--ignored', '--nocapture'], stdout=f, cwd='/home/user/abidingword/src-tauri', env=env,
                       timeout=TIMEOUT['current'])
        pages = {}; units = 0
        with open(tmp, encoding='utf-8', errors='replace') as f:
            for line in f:
                m = UNIT.match(line.rstrip('\n'))
                if m:
                    units += 1; pages.setdefault(int(m.group(2)), []).append(m.group(5))
        ok = info['rc'] == 0 and 'test result: ok' in open(tmp, errors='replace').read()[-3000:]
        if not ok:
            m = re.search(r"panicked at ([^\n]+)\n([^\n]+(?:\n  left: [^\n]+\n right: [^\n]+)?)", info['stderr_tail'] + open(tmp, errors='replace').read()[-5000:])
            meta['error'] = (m.group(1) + ' ' + m.group(2)) if m else info['stderr_tail'][-300:]
        meta.update(ok=ok, units=units, pages_with_text=len(pages), max_page=max(pages) if pages else 0)
        plist = [pages[k] for k in sorted(pages)]
    elif base == 'poppler':
        cmd = ['pdftotext', '-enc', 'UTF-8'] + (['-raw'] if eng.endswith(':raw') else []) + [pdf, tmp]
        info = run(cmd)
        ok = info['rc'] == 0 and os.path.exists(tmp)
        plist = []
        if os.path.exists(tmp):
            txt = open(tmp, encoding='utf-8', errors='replace').read()
            pp = txt.split('\f')
            if pp and pp[-1] == '': pp = pp[:-1]
            plist = [p.split('\n') for p in pp]
        meta.update(ok=ok, page_count=len(plist), pages_with_text=sum(1 for p in plist if ''.join(p).strip()))
        if not ok or info['stderr_tail']: meta['error' if not ok else 'warnings'] = info['stderr_tail'][-500:]
    else:
        info = run(PY + [eng, 'text', pdf, tmp], timeout=TIMEOUT.get(base, 1200))
        try: meta = json.load(open(tmp + '.json'))
        except Exception: meta = {'ok': False, 'error': 'no output; ' + info['stderr_tail'][-300:]}
        if info['timeout']: meta = {'ok': False, 'error': 'timeout'}
        plist = []
        if meta.get('ok'):
            plist = [p.split('\n') for p in open(tmp, encoding='utf-8', errors='replace').read().split('\f')]
        if os.path.exists(tmp + '.json'): os.remove(tmp + '.json')
    # keep a compressed copy of the text
    os.makedirs(os.path.join(TXT, eng.replace(':', '_')), exist_ok=True)
    with gzip.open(os.path.join(TXT, eng.replace(':', '_'), key + '.txt.gz'), 'wt', encoding='utf-8') as f:
        f.write('\f'.join('\n'.join(p) for p in plist))
    if base == 'current' and os.path.exists(tmp):
        with open(tmp, 'rb') as a, gzip.open(os.path.join(TXT, 'current', key + '.dump.gz'), 'wb') as b: b.write(a.read())
    if os.path.exists(tmp): os.remove(tmp)
    return info, plist, meta

def main():
    key = sys.argv[1]
    engines = sys.argv[2].split(',') if len(sys.argv) > 2 else DEFAULT
    docs = {d['key']: d for d in json.load(open(OUT + '/docs.json'))}
    if os.path.exists(OUT + '/damaged_docs.json'):
        docs.update({d['key']: d for d in json.load(open(OUT + '/damaged_docs.json'))})
    d = docs[key]
    books = None
    if d.get('audit_json'):
        a = json.load(open(d['audit_json'])); books = [b['code'] for b in a['books'] if b.get('expected', 0) > 0]
    elif d.get('books_col') and d['books_col'] != 'all':
        books = d['books_col'].split(',')
    verses = S.reference(d['ref'], books, key.split('~')[0])
    hebrew = d['script'].lower().startswith('hebr') or d['lang'] in ('he', 'heb', 'hbo')
    rtl = hebrew
    rp = os.path.join(OUT, 'results', key + '.json')
    os.makedirs(os.path.dirname(rp), exist_ok=True)
    res = json.load(open(rp)) if os.path.exists(rp) else {'key': key, 'engines': {}}
    res.update(ref_verses=len(verses), books=books, hebrew=hebrew)
    with open(d['pdf'], 'rb') as f: f.read()   # warm the page cache so no engine pays the disk read
    for eng in engines:
        info, plist, meta = engine_text(eng, d)
        lines = [l for p in plist for l in p]
        r = {'run': info, 'meta': meta}
        if meta.get('ok') or lines:
            t0 = time.time(); r['score'] = S.score(lines, verses, rtl=rtl, hebrew=hebrew); r['score_seconds'] = round(time.time() - t0, 1)
        res['engines'][eng] = r
        sc = r.get('score', {}).get('nfc', {})
        print(key, eng, 'ok' if meta.get('ok') else 'FAIL', info['seconds'], 's', info['peak_rss_mb'], 'MB',
              'cov', sc.get('coverage'), 'ord', sc.get('in_order_share'), flush=True)
        json.dump(res, open(rp, 'w'), ensure_ascii=False, indent=1)

if __name__ == '__main__':
    main()
