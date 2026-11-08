"""Markdown tables for the report from out/results.json."""
import json, statistics as st
R = json.load(open('/root/a2-pdf/out/results.json'))
MAIN = ['current', 'pdfium', 'mupdf', 'poppler']
def pct(x): return '–' if x is None else f'{100*x:.1f}'
def c(e, k='coverage', o='in_order'):
    if not e: return 'n/m'
    if not e['opens'] and e.get(k) is None: return f"**FAIL**"
    return f"{pct(e.get(k))}/{pct(e.get(o))}"
def perf(e):
    if not e: return 'n/m'
    if e.get('peak_rss_mb') is None: return f"{e['seconds']:.1f}s\\* –"
    return f"{e['seconds']:.1f}s {e['peak_rss_mb']:.0f}MB"
out = []
P = out.append
P('| doc | lang | pages | verses | ' + ' | '.join(f'{m} cov/ord % · s · MB' for m in MAIN) + ' | pdfminer cov/ord % · s · MB | app verified |')
P('|' + '---|' * (6 + len(MAIN)))
for d in R['documents']:
    E = d['engines']; pages = next((E[m]['pages'] for m in ['mupdf', 'pdfium', 'poppler'] if E.get(m) and E[m].get('pages')), '?')
    cells = []
    for m in MAIN + ['pdfminer']:
        e = E.get(m)
        if not e: cells.append('n/m'); continue
        if not e['opens'] and e.get('coverage') is None: cells.append(f"**FAIL** · {e['seconds']:.1f}s"); continue
        cells.append(f"{c(e)} · {perf(e)}")
    app = d.get('app'); a = f"{app['verified']}/{app['expected']}" if app else 'refused (no report)'
    P(f"| {d['key']} | {d['lang']} | {pages} | {d['ref_verses']} | " + ' | '.join(cells) + f' | {a} |')
md_main = '\n'.join(out)

# letters-only (digits dropped) in-order and NFKC
out = []; P = out.append
P('| doc | ' + ' | '.join(f'{m} covL/ordL' for m in MAIN) + ' | ' + ' | '.join(f'{m} ord NFKC' for m in MAIN) + ' |')
P('|' + '---|' * (1 + 2 * len(MAIN)))
for d in R['documents']:
    if d['hebrew']: continue
    E = d['engines']
    P(f"| {d['key']} | " + ' | '.join(c(E.get(m), 'coverage_letters', 'in_order_letters') for m in MAIN) + ' | ' +
      ' | '.join(pct((E.get(m) or {}).get('in_order_nfkc')) for m in MAIN) + ' |')
md_letters = '\n'.join(out)

# summary
out = []; P = out.append
def vals(m, k):
    return [d['engines'][m].get(k) for d in R['documents'] if d['engines'].get(m) and d['engines'][m].get(k) is not None]
def clean(m, k):
    return [(d['engines'][m]['clean'][k], d['key']) for d in R['documents'] if d['engines'].get(m) and d['engines'][m].get('clean')]
def opened(m): return sum(1 for d in R['documents'] if d['engines'].get(m) and d['engines'][m]['opens'])
def n(m): return sum(1 for d in R['documents'] if d['engines'].get(m))
P('| metric | ' + ' | '.join(MAIN + ['pdfminer']) + ' |'); P('|' + '---|' * 6)
P('| documents run | ' + ' | '.join(str(n(m)) for m in MAIN + ['pdfminer']) + ' |')
P('| opened | ' + ' | '.join(f'{opened(m)}/{n(m)}' for m in MAIN + ['pdfminer']) + ' |')
for k, lab in [('coverage', 'mean coverage % (failures count 0)'), ('in_order', 'mean in-order % (failures count 0)'),
               ('in_order_letters', 'mean in-order % letters-only (non-Hebrew; failures 0)')]:
    row = []
    for m in MAIN + ['pdfminer']:
        xs = [ (d['engines'][m].get(k) or 0) for d in R['documents'] if d['engines'].get(m) and not (k == 'in_order_letters' and d['hebrew'])]
        row.append(pct(st.mean(xs)) if xs else '–')
    P(f'| {lab} | ' + ' | '.join(row) + ' |')
for k, lab in [('in_order', 'median in-order %')]:
    P(f'| {lab} | ' + ' | '.join(pct(st.median([(d["engines"][m].get(k) or 0) for d in R["documents"] if d["engines"].get(m)])) if n(m) else '–' for m in MAIN + ['pdfminer']) + ' |')
for th in (0.9, 0.5):
    P(f'| docs with in-order ≥ {int(th*100)}% | ' + ' | '.join(str(sum(1 for d in R['documents'] if d['engines'].get(m) and (d['engines'][m].get('in_order') or 0) >= th)) for m in MAIN + ['pdfminer']) + ' |')
P('| docs where engine is best or within 1 pt of best in-order | ' + ' | '.join(str(sum(1 for d in R['documents'] if d['engines'].get(m) and (d['engines'][m].get('in_order') or 0) >= max((d['engines'][x].get('in_order') or 0) for x in MAIN if d['engines'].get(x)) - 0.01)) for m in MAIN) + ' | – |')
def worst(m):
    xs = [(d['engines'][m].get('in_order') or 0, d['key']) for d in R['documents'] if d['engines'].get(m)]
    xs.sort(); return ', '.join(f'{k} {pct(v)}' for v, k in xs[:4])
P('| worst in-order | ' + ' | '.join(worst(m) for m in MAIN + ['pdfminer']) + ' |')
P('| docs with clean timing | ' + ' | '.join(str(len(clean(m, 'seconds'))) for m in MAIN + ['pdfminer']) + ' |')
P('| total wall s (clean-timed docs) | ' + ' | '.join(f"{sum(x for x, _ in clean(m,'seconds')):.0f}" if clean(m,'seconds') else '–' for m in MAIN + ['pdfminer']) + ' |')
P('| median wall s (clean) | ' + ' | '.join(f"{st.median([x for x, _ in clean(m,'seconds')]):.2f}" if clean(m,'seconds') else '–' for m in MAIN + ['pdfminer']) + ' |')
P('| max wall s (clean) | ' + ' | '.join((lambda xs: f"{max(xs)[0]:.1f} ({max(xs)[1]})")(clean(m,'seconds')) if clean(m,'seconds') else '–' for m in MAIN + ['pdfminer']) + ' |')
P('| median peak RSS MB (clean) | ' + ' | '.join(f"{st.median([x for x, _ in clean(m,'peak_rss_mb')]):.0f}" if clean(m,'peak_rss_mb') else '–' for m in MAIN + ['pdfminer']) + ' |')
P('| max peak RSS MB (clean) | ' + ' | '.join((lambda xs: f"{max(xs)[0]:.0f} ({max(xs)[1]})")(clean(m,'peak_rss_mb')) if clean(m,'peak_rss_mb') else '–' for m in MAIN + ['pdfminer']) + ' |')
P('| G03 (816 pages, large-document test) s / MB | ' + ' | '.join((lambda e: f"{e['seconds']:.1f} / {e['peak_rss_mb'] if e.get('peak_rss_mb') is not None else '–'}" if e else '–')(next((d['engines'].get(m) for d in R['documents'] if d['key']=='G03'), None)) for m in MAIN + ['pdfminer']) + ' |')
P('| failures | ' + ' | '.join(', '.join(f"{d['key']}" for d in R['documents'] if d['engines'].get(m) and not d['engines'][m]['opens']) or 'none' for m in MAIN + ['pdfminer']) + ' |')
md_summary = '\n'.join(out)

# RTL
out = []; P = out.append
P('| doc | engine | cov/ord pointed (NFC) | cov/ord points stripped | line-reversed (stripped) | word-order-reversed (stripped) |'); P('|---|---|---|---|---|---|')
for d in R['documents']:
    if not d['hebrew']: continue
    for m in MAIN + ['pdfminer', 'mupdf:sort', 'poppler:raw']:
        e = d['engines'].get(m)
        if not e: continue
        if not e['opens'] and e.get('coverage') is None: P(f"| {d['key']} | {m} | FAIL | | | |"); continue
        lr = e.get('stripped_line_reversed') or {}; wr = e.get('stripped_word_reversed') or {}
        P(f"| {d['key']} | {m} | {pct(e.get('coverage_pointed'))}/{pct(e.get('in_order_pointed'))} | {pct(e.get('coverage'))}/{pct(e.get('in_order'))} | {pct(lr.get('coverage'))}/{pct(lr.get('in_order'))} | {pct(wr.get('coverage'))}/{pct(wr.get('in_order'))} |")
md_rtl = '\n'.join(out)

# variants
out = []; P = out.append
P('| doc | mupdf (stream order) | mupdf sort=True | poppler (default layout analysis) | poppler -raw |'); P('|---|---|---|---|---|')
for d in R['documents']:
    E = d['engines']
    P(f"| {d['key']} | {c(E.get('mupdf'))} | {c(E.get('mupdf:sort'))} | {c(E.get('poppler'))} | {c(E.get('poppler:raw'))} |")
md_var = '\n'.join(out)

# damaged
out = []; P = out.append
P('| damaged copy | damage | ' + ' | '.join(MAIN + ['pdfminer']) + ' |'); P('|' + '---|' * 7)
orig = {d['key']: d for d in R['documents']}
for d in R['damaged']:
    cells = []
    for m in MAIN + ['pdfminer']:
        e = d['engines'].get(m)
        if not e: cells.append('n/m'); continue
        o = orig.get(d['original'], {}).get('engines', {}).get(m)
        if not e['opens'] and not e.get('coverage'):
            err = e.get('error') or ''
            if 'library.badPdf' in err: err = 'library.badPdf (lopdf Document::load_mem failed)'
            cells.append('**FAIL**: ' + err[:75].replace('|', '/').replace('\n', ' ')); continue
        if not e.get('pages'):
            cells.append('**silent failure**: opens, 0 pages, no text'); continue
        ref = f" (orig {pct(o.get('coverage'))})" if o and o.get('coverage') is not None else ''
        cells.append(f"opens, {e.get('pages_with_text')}/{e.get('pages')} pages text, cov {pct(e.get('coverage'))}{ref}")
    P(f"| {d['key']} | {d['damage']} | " + ' | '.join(cells) + ' |')
md_dam = '\n'.join(out)
json.dump({'main': md_main, 'letters': md_letters, 'summary': md_summary, 'rtl': md_rtl, 'variants': md_var, 'damaged': md_dam},
          open('/root/a2-pdf/out/tables.json', 'w'), ensure_ascii=False, indent=0)
print(md_summary); print(); print(md_dam); print(); print(md_rtl)

# attribution
out = []; P = out.append
P('| doc | app verified | current in-order (spec / letters-only) | best other engine in-order | class | note |'); P('|---|---|---|---|---|---|')
attr = []
for d in R['documents']:
    app = d.get('app')
    share = app['verified'] / app['expected'] if app and app['expected'] else 0.0
    if app and share >= 0.5: continue
    E = d['engines']; cur = E['current']
    others = {m: (E[m].get('in_order') or 0) for m in ['pdfium', 'mupdf', 'poppler', 'pdfminer'] if E.get(m)}
    bm = max(others, key=others.get); best = others[bm]
    c0 = cur.get('in_order'); cL = cur.get('in_order_letters')
    note = ''
    bestL = max([(E[m].get('in_order_letters') or 0) for m in ['pdfium', 'mupdf', 'poppler', 'pdfminer'] if E.get(m)] or [0])
    if not cur['opens']:
        cls = 'EXTRACTION (cannot open)' if best >= 0.5 else 'DOCUMENT'
        note = 'pdf-extract 0.12.1 panics (lib.rs:584 assert_eq 256 vs 255: font /Widths length vs FirstChar..LastChar) on every page; app reports library.badPdf'
    elif c0 >= 0.85:
        cls = 'RECOGNITION'
    elif best < 0.5 and c0 < 0.5:
        cls = 'DOCUMENT (no engine gets it)'
    elif c0 >= best - 0.02:
        cls = 'RECOGNITION (partly document: text layer limits every engine)'
        note = f'current is the best or equal engine at {pct(c0)}% in order; the app verified far less'
    elif cL is not None and cL >= bestL - 0.02 and cL >= 0.8:
        cls = 'EXTRACTION (verse-number placement only)'
        note = f'words complete and in order (letters-only {pct(cL)}% vs best {pct(bestL)}%); verse numbers emitted out of their inline position'
    else:
        cls = 'EXTRACTION'
    if d['hebrew']:
        wr = {m: (E[m].get('stripped_word_reversed') or {}).get('in_order') or 0 for m in ['pdfium', 'mupdf', 'poppler'] if E.get(m)}
        wm = max(wr, key=wr.get)
        if wr[wm] > best + 0.1: note += f' {wm} reaches {pct(wr[wm])}% after reversing word order per line (needs bidi reordering)'
    a = f"{app['verified']}/{app['expected']} ({pct(share)}%)" if app else 'refused'
    P(f"| {d['key']} | {a} | {pct(c0)} / {pct(cL)} | {bm} {pct(best)} | {cls} | {note.strip()} |")
    attr.append({'key': d['key'], 'app_verified_share': share, 'current_in_order': c0, 'current_in_order_letters': cL,
                 'best_engine': bm, 'best_in_order': best, 'class': cls, 'note': note.strip()})
md_attr = '\n'.join(out)
t = json.load(open('/root/a2-pdf/out/tables.json')); t['attr'] = md_attr
json.dump(t, open('/root/a2-pdf/out/tables.json', 'w'), ensure_ascii=False, indent=0)
json.dump(attr, open('/root/a2-pdf/out/attribution.json', 'w'), indent=1)
print(md_attr)

# structure counts per document (measured)
out = []; P = out.append
P('| doc | outline entries (pdfium/mupdf/poppler-xml) | link annots (pdfium/mupdf; poppler <a> spans) | pages with label (pdfium/mupdf) | tagged: pages with struct tree (pdfium) / StructTreeRoot (mupdf) / Tagged (pdfinfo) | ActualText (streams) | fonts seen (pdfium sample/mupdf spans/poppler fontspecs) | char boxes ok (pdfium sample) / mupdf chars / poppler words |')
P('|---|---|---|---|---|---|---|---|')
agg = {'outline': 0, 'links': 0, 'labels': 0, 'tagged': 0, 'actualtext': 0, 'n': 0, 'agree_outline': 0, 'agree_links': 0, 'agree_labels': 0}
for d in R['documents']:
    s = d.get('structure')
    if not s: continue
    a, b, c = s.get('pdfium', {}), s.get('mupdf', {}), s.get('poppler', {})
    agg['n'] += 1
    agg['outline'] += (b.get('outline_entries') or 0) > 0; agg['links'] += (b.get('link_annots') or 0) > 0
    agg['labels'] += (b.get('pages_with_label') or 0) > 0; agg['tagged'] += bool(b.get('struct_tree_root'))
    agg['actualtext'] += (b.get('actualtext_streams') or 0) > 0
    agg['agree_outline'] += a.get('outline_entries') == b.get('outline_entries') == c.get('xml_outline_items')
    agg['agree_links'] += a.get('link_annots') == b.get('link_annots')
    agg['agree_labels'] += a.get('pages_with_label') == b.get('pages_with_label')
    P(f"| {d['key']} | {a.get('outline_entries')}/{b.get('outline_entries')}/{c.get('xml_outline_items')} | {a.get('link_annots')}/{b.get('link_annots')}; {c.get('xml_links')} | {a.get('pages_with_label')}/{b.get('pages_with_label')} | {a.get('pages_with_struct_tree')}/{b.get('struct_tree_root')}/{c.get('Tagged')} | {b.get('actualtext_streams')} | {a.get('distinct_fonts_sampled')}/{b.get('distinct_fonts')}/{c.get('xml_fontspecs')} | {a.get('char_boxes_sampled_ok')}/{b.get('chars')}/{c.get('bbox_words')} |")
md_struct = '\n'.join(out)
t = json.load(open('/root/a2-pdf/out/tables.json')); t['struct'] = md_struct; t['struct_agg'] = agg
json.dump(t, open('/root/a2-pdf/out/tables.json', 'w'), ensure_ascii=False, indent=0)
print(agg)
