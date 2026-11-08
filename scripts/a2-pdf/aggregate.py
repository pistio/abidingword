"""Collect per-document results into out/results.json and print markdown tables for the report."""
import json, os, glob, statistics as st
OUT = '/root/a2-pdf/out'
docs = json.load(open(OUT + '/docs.json'))
damaged = json.load(open(OUT + '/damaged_docs.json'))
base = json.load(open(OUT + '/baseline.json'))
ENG = ['current', 'pdfium', 'mupdf', 'poppler', 'pdfminer']
MAIN = ['current', 'pdfium', 'mupdf', 'poppler']

def best_variant(sc, hebrew):
    """the normalisation variant used for the headline numbers"""
    return sc.get('stripped') if hebrew else sc.get('nfc')

def cell(r, hebrew):
    if r is None: return None
    m = r['meta']; ok = bool(m.get('ok'))
    sc = r.get('score') or {}
    v = best_variant(sc, hebrew) if sc else None
    lt = sc.get('letters') if sc else None
    pages = m.get('page_count', m.get('max_page'))
    out = {'opens': ok, 'error': m.get('error'), 'pages': pages, 'pages_with_text': m.get('pages_with_text'),
           'seconds': r['run']['seconds'], 'cpu_seconds': r['run']['cpu_seconds'], 'peak_rss_mb': r['run']['peak_rss_mb'],
           'timeout': r['run']['timeout']}
    if v:
        out.update(coverage=v['coverage'], in_order_global=v['in_order_share'], in_order=v['in_order_book_share'])
    if lt and not hebrew:
        out.update(coverage_letters=lt['coverage'], in_order_letters=lt['in_order_book_share'])
    if sc.get('nfkc'): out['coverage_nfkc'] = sc['nfkc']['coverage']; out['in_order_nfkc'] = sc['nfkc']['in_order_book_share']
    if hebrew and sc:
        out['coverage_pointed'] = sc['nfc']['coverage']; out['in_order_pointed'] = sc['nfc']['in_order_book_share']
        for k in ('stripped_line_reversed', 'stripped_word_reversed'):
            if k in sc: out[k] = {'coverage': sc[k]['coverage'], 'in_order': sc[k]['in_order_book_share']}
    if 'nfc' in sc and 'per_book' in sc['nfc']:
        out['per_book_nfc'] = sc['nfc']['per_book']
    return out

results = {'baseline_null_process': base, 'documents': [], 'damaged': []}
for d in docs:
    rp = f"{OUT}/results/{d['key']}.json"
    if not os.path.exists(rp): continue
    r = json.load(open(rp)); heb = r['hebrew']
    e = {'key': d['key'], 'role': d['role'], 'lang': d['lang'], 'script': d['script'], 'pdf': d['pdf'], 'sha256': d['sha256'],
         'checksum_verified': d['verified'], 'checks': d['checks'], 'ref_verses': r['ref_verses'], 'books': r['books'], 'hebrew': heb,
         'engines': {x: cell(r['engines'].get(x), heb) for x in r['engines']}}
    if d.get('audit_json'):
        a = json.load(open(d['audit_json'])); t = a['totals']
        e['app'] = {'expected': t['expected'], 'verified': t['verified'], 'detected': t['detected'],
                    'refused_reading_order': t['refused_reading_order'], 'seconds': a.get('seconds'), 'audit_json': d['audit_json']}
    sp = f"{OUT}/struct/{d['key']}.json"
    if os.path.exists(sp): e['structure'] = json.load(open(sp))
    results['documents'].append(e)
tc = json.load(open(OUT + '/timing_clean.json')) if os.path.exists(OUT + '/timing_clean.json') else {}
tm = json.load(open(OUT + '/timing_pdfminer.json')) if os.path.exists(OUT + '/timing_pdfminer.json') else {}
for e in results['documents']:
    for m, x in e['engines'].items():
        if not x: continue
        c = (tc.get(e['key']) or {}).get(m) if m != 'pdfminer' else tm.get(e['key'])
        if c: x['clean'] = {'seconds': c['seconds'], 'cpu_seconds': c['cpu_seconds'], 'peak_rss_mb': c['peak_rss_mb'], 'rc': c['rc']}
        x['batch_seconds'] = x['seconds']; x['batch_peak_rss_mb_inflated'] = x.pop('peak_rss_mb')
        x['seconds'] = c['seconds'] if c else x['seconds']
        x['peak_rss_mb'] = c['peak_rss_mb'] if c else None
        x['timing_source'] = 'clean' if c else 'batch (RSS not reliable)'
for d in damaged:
    rp = f"{OUT}/results/{d['key']}.json"
    if not os.path.exists(rp): continue
    r = json.load(open(rp))
    results['damaged'].append({'key': d['key'], 'damage': d['damage'], 'original': d['original'],
                               'engines': {x: cell(r['engines'][x], r['hebrew']) for x in r['engines']}})
for e in results['documents']:
    for x in e['engines'].values():
        if x: x.pop('per_book_nfc', None) if False else None
json.dump(results, open(OUT + '/results.json', 'w'), indent=1, ensure_ascii=False)
print('docs', len(results['documents']), 'damaged', len(results['damaged']))
