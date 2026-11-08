"""Phase A2.1: the comparison tables, from a21_measure.py's per-document
results (text fidelity, reading order), the real-document harness's reports
for each engine (verification) and a21_clean_timing.py's timings.

usage: python3 -I a21_tables.py <measure-dir> <current-reports> <pdfium-reports> <clean-timing.json> > tables.md
"""
import json, os, sys

GROUPS = [
    ('G05 (Luther 1545, Fraktur forms)', lambda d: d['key'] == 'G05'),
    ('Hebrew, right to left', lambda d: d['script'] == 'Hebrew'),
    ('Korean', lambda d: d['script'] == 'Hangul'),
    ('Greek', lambda d: d['script'] == 'Greek'),
    ('Multi-column', lambda d: any(w in d.get('layout', '').lower() for w in ('two columns', 'two css columns', 'three justified', 'two justified', 'two reportlab', 'two columns with'))),
]

def pct(v):
    return '—' if v is None else f'{100 * v:.1f} %'

def fid(m, eng, hebrew):
    e = (m or {}).get('engines', {}).get(eng)
    if not e or not e.get('ok') or 'score' not in e:
        return None, None
    s = e['score']['stripped' if hebrew and 'stripped' in e['score'] else 'nfc']
    return s.get('coverage'), s.get('in_order_share')

def rec(path):
    if not os.path.exists(path):
        return None
    j = json.load(open(path)); t = j['totals']
    wrong = sum(1 for x in j['differences'] if x.get('verified') and x.get('kind') not in ('typography', 'unicode'))
    return {'printed': t['expected'], 'verified': t['verified'], 'wrong': wrong, 'fp': len(j.get('verified_extra_verses', []))}

def main():
    mdir, cur, pdf, timing = sys.argv[1:5]
    docs = json.load(open('/root/a2-pdf/out/docs.json'))
    tm = json.load(open(timing)) if os.path.exists(timing) else {}
    rows = {}
    for d in docs:
        k = d['key']; mp = os.path.join(mdir, k + '.json')
        m = json.load(open(mp)) if os.path.exists(mp) else None
        heb = d['script'] == 'Hebrew'
        rows[k] = dict(d=d, cur_f=fid(m, 'current', heb), pdf_f=fid(m, 'pdfium-layout', heb),
                       cur_r=rec(os.path.join(cur, k + '.json')), pdf_r=rec(os.path.join(pdf, k + '.json')),
                       t=tm.get(k, {}))
    def line(k):
        r = rows[k]; cr, pr = r['cur_r'], r['pdf_r']; t = r['t']
        v = lambda x, f: '—' if not x else x[f]
        tt = lambda e: '—' if e not in t or t[e]['rc'] != 0 else f"{t[e]['seconds']:.1f} s / {t[e]['peak_rss_mb']:.0f} MB"
        return (f"| {k} | {pct(r['cur_f'][0])} / {pct(r['pdf_f'][0])} | {pct(r['cur_f'][1])} / {pct(r['pdf_f'][1])} "
                f"| {v(cr,'verified')} / {v(pr,'verified')} | {v(cr,'wrong')} / {v(pr,'wrong')} | {v(cr,'fp')} / {v(pr,'fp')} "
                f"| {tt('current')} | {tt('pdfium-layout')} | {tt('helper_alone')} |")
    head = ("| Document | Text coverage current / PDFium | In order current / PDFium | Verified current / PDFium "
            "| Verified wrong text current / PDFium | Verified not in edition current / PDFium | Current: time / peak | PDFium through layout: time / peak | Helper alone |\n"
            "|---|---|---|---|---|---|---|---|---|")
    print('# A2.1 measurements: current backend against PDFium glyphs through the app\'s layout\n')
    print('Same 57 checksummed PDFs as A2. Text coverage and order: share of the edition\'s verses found whole, and found in order, in the text the app\'s layout produced (A2\'s `score.py`; Hebrew with points and cantillation stripped, as in A2). Verification: the real-document harness on the same documents. Time and peak memory: release build, each reading in its own process (`clean_run.py`); for PDFium the peak is the larger of the app\'s and the helper\'s.\n')
    for title, f in GROUPS:
        ks = [d['key'] for d in docs if f(d)]
        print(f'## {title}\n'); print(head)
        for k in ks: print(line(k))
        print()
    print('## All 57\n'); print(head)
    tot = [0] * 6; ts = {'current': 0.0, 'pdfium-layout': 0.0, 'helper_alone': 0.0}
    for d in docs:
        k = d['key']; print(line(k)); r = rows[k]
        for i, (x, f) in enumerate([(r['cur_r'], 'verified'), (r['pdf_r'], 'verified'), (r['cur_r'], 'wrong'), (r['pdf_r'], 'wrong'), (r['cur_r'], 'fp'), (r['pdf_r'], 'fp')]):
            tot[i] += x[f] if x else 0
        for e in ts:
            if e in r['t'] and r['t'][e]['rc'] == 0: ts[e] += r['t'][e]['seconds']
    print(f"| **Total** | | | {tot[0]} / {tot[1]} | {tot[2]} / {tot[3]} | {tot[4]} / {tot[5]} | {ts['current']:.0f} s | {ts['pdfium-layout']:.0f} s | {ts['helper_alone']:.0f} s |")

if __name__ == '__main__':
    main()
