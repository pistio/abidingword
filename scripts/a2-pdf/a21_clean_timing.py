"""Phase A2.1: wall time, CPU time and peak memory of each reading of A2's
57 PDFs, each in its own process started by clean_run.py (not inflated by the
measuring process): the current backend and PDFium glyphs through the app's
layout (the app's unit dump, all units), and the helper alone. For
pdfium-layout the peak is the larger of the app's and the helper's (wait4
counts the waited-for helper), not their sum.

usage: python3 -I a21_clean_timing.py <dump-binary> <helper> <libdir> <out.json>
"""
import json, os, subprocess, sys

def main():
    dump, helper, libdir, out = sys.argv[1:5]
    here = os.path.dirname(os.path.abspath(__file__))
    docs = json.load(open('/root/a2-pdf/out/docs.json'))
    res = {}
    for d in docs:
        key = d['key']; r = {}
        for eng in ('current', 'pdfium-layout'):
            env = dict(os.environ, AW_DUMP_FILE=d['pdf'], AW_DUMP_UNITS='0-10000000', RUST_BACKTRACE='0',
                       AW_CLEAN_CWD='/home/user/abidingword/src-tauri')
            if eng == 'pdfium-layout':
                env.update(AW_PDF_ENGINE='pdfium', AW_PDFIUM_HELPER=helper, AW_PDFIUM_LIB=libdir)
            p = subprocess.run([sys.executable, '-I', os.path.join(here, 'clean_run.py'), '2400', dump,
                                '--ignored', '--nocapture'], env=env, capture_output=True, text=True)
            r[eng] = json.loads(p.stdout.strip().splitlines()[-1])
        p = subprocess.run([sys.executable, '-I', os.path.join(here, 'clean_run.py'), '1800', helper, libdir,
                            d['pdf'], '--max-chars', '40000000', '--max-memory-mb', '4096'],
                           capture_output=True, text=True)
        r['helper_alone'] = json.loads(p.stdout.strip().splitlines()[-1])
        res[key] = r
        print(key, {e: (v['rc'], v['seconds'], v['peak_rss_mb']) for e, v in r.items()}, flush=True)
    json.dump(res, open(out, 'w'), indent=1)

if __name__ == '__main__':
    main()
