"""Phase A1.1 tables from the release-build harness runs.

usage: python3 -I scripts/a1.1/tables.py <frozen-dir-pattern> <structural-dir-pattern> <a1-dir-pattern> <out-dir>
       (patterns contain {s} for the set: g f e recon bc dev)

Writes regression-tables.md, performance.md and removed-ranges.md to out-dir.
Every removed range is checked against the stored file, byte for byte.
"""
import collections, glob, html, json, os, re, sys, zipfile

SETS = ['g', 'f', 'e', 'recon', 'bc', 'dev']
DOCS = {'g': '/root/aw-set-g/docs', 'f': '/root/aw-set-f/docs', 'e': '/root/aw-set-e/docs',
        'recon': '/root/aw-recon', 'bc': '/root/aw-recon-bc', 'dev': '/root/aw-corpus'}

def totals(path):
    j = json.load(open(path)); t = j['totals']
    wrong = [d for d in j['differences'] if d.get('verified') and d.get('kind') not in ('typography', 'unicode')]
    return dict(printed=t['expected'], verified=t['verified'], wrong=len(wrong), fp=len(j.get('verified_extra_verses', [])),
                seconds=j.get('seconds'), mb=j.get('peak_memory_mb'))

def epub_of(s, key):
    # The harness's edition list names each edition's file; find the EPUB by the key's stem.
    cands = [p for p in glob.glob(os.path.join(DOCS[s], '*.epub'))]
    stem = key.replace('-epub', '').replace('-rebooklibepub', '').replace('-rpandocepub', '').replace('-pandocepub', '').replace('-repub2', '').replace('-epubapoc', '')
    best = [p for p in cands if os.path.basename(p).startswith(key) or os.path.basename(p).startswith(stem)]
    return best[0] if best else None

def main():
    fz, st, a1, out = sys.argv[1:5]
    os.makedirs(out, exist_ok=True)
    rows = []
    for s in SETS:
        for p in sorted(glob.glob(os.path.join(st.format(s=s), 'structural', '*.json'))):
            k = os.path.basename(p)[:-5]
            f, n = os.path.join(fz.format(s=s), k + '.json'), os.path.join(st.format(s=s), k + '.json')
            a = os.path.join(a1.format(s=s), k + '.json')
            if not os.path.exists(a) and s == 'g':
                a = os.path.join(a1.format(s='gall'), k + '.json')
            if not (os.path.exists(f) and os.path.exists(n)):
                continue
            rep = json.load(open(p))
            if not rep['verses']:
                continue  # not read by the EPUB reader (frozen path in both settings)
            rows.append((s, k, totals(f), totals(a) if os.path.exists(a) else None, totals(n), rep))
    # Regression tables
    with open(os.path.join(out, 'regression-tables.md'), 'w') as o:
        o.write('# A1.1 regression: frozen reading, A1 and A1.1 on every disclosed EPUB-family edition\n\n')
        o.write('Release build, one process per edition and reading. *Wrong*: verified verses whose text differs from the edition beyond typography. *Not in edition*: verified verses the edition does not have. A1 is `205a453`; A1.1 is `47a4309`. Editions read without the EPUB reader are identical in both readings (`README.md`).\n\n')
        o.write('| Set | Edition | Printed | Verified frozen | Verified A1 | Verified A1.1 | Wrong frozen / A1 / A1.1 | Not in edition A1.1 | Exact / spacing / untraced (A1.1) | Withdrawn by cause (A1.1) |\n|---|---|---|---|---|---|---|---|---|---|\n')
        T = collections.Counter()
        for s, k, f, a, n, rep in rows:
            g = collections.Counter(rep['gated'].values())
            o.write(f"| {s} | {k} | {n['printed']} | {f['verified']} | {a['verified'] if a else '—'} | {n['verified']} | {f['wrong']} / {a['wrong'] if a else '—'} / {n['wrong']} | {n['fp']} | {rep['exact']} / {rep['spacingDiffers']} / {rep['untraced']} | {', '.join(f'{c} {v}' for c, v in sorted(g.items())) or '—'} |\n")
            T['printed'] += n['printed']; T['fz'] += f['verified']; T['a1'] += a['verified'] if a else 0; T['n'] += n['verified']
            T['wf'] += f['wrong']; T['wa'] += a['wrong'] if a else 0; T['wn'] += n['wrong']; T['fp'] += n['fp']
            T['exact'] += rep['exact']; T['sp'] += rep['spacingDiffers']; T['un'] += rep['untraced']
        o.write(f"| **Total** | {len(rows)} editions | {T['printed']} | {T['fz']} | {T['a1']} | {T['n']} | {T['wf']} / {T['wa']} / {T['wn']} | {T['fp']} | {T['exact']} / {T['sp']} / {T['un']} | |\n")
    # Performance
    with open(os.path.join(out, 'performance.md'), 'w') as o:
        o.write('# A1.1 performance: frozen against structure-aware reading, optimised build\n\n')
        o.write('`cargo test --release` (opt-level 3, no debug information), the real-document harness, **one process per edition and reading**, frozen then structural for each edition in turn on an otherwise idle 4-core machine. Seconds are the whole harness run for the edition (reading, recognition, comparison with the edition, report). Peak MB is that process\'s own peak resident memory (`peak_memory_mb`), so it is per edition.\n\n')
        o.write('| Set | Edition | Frozen s | Structural s | Δ | Frozen peak MB | Structural peak MB | Δ MB |\n|---|---|---|---|---|---|---|---|\n')
        tf = ts = 0.0; mf = ms = 0
        for s, k, f, a, n, rep in rows:
            o.write(f"| {s} | {k} | {f['seconds']:.1f} | {n['seconds']:.1f} | {100 * (n['seconds'] - f['seconds']) / f['seconds']:+.0f} % | {f['mb']:.0f} | {n['mb']:.0f} | {n['mb'] - f['mb']:+.0f} |\n")
            tf += f['seconds']; ts += n['seconds']; mf = max(mf, f['mb']); ms = max(ms, n['mb'])
        o.write(f"| | **Total** | {tf:.0f} | {ts:.0f} | {100 * (ts - tf) / tf:+.0f} % | max {mf:.0f} | max {ms:.0f} | |\n")
    # Removed ranges, checked against the stored file
    checked = bad = 0; lines = []
    for s, k, f, a, n, rep in rows:
        ep = None
        for key, r in sorted(rep['detailed'].items()):
            rem = [x for x in r['excluded'] if x['kind'] == 'apparatus']
            if not rem:
                continue
            ep = ep or epub_of(s, k)
            z = zipfile.ZipFile(ep); b = z.read(r['file']['entry'])
            for x in rem:
                rng = x['range']; raw = b[rng['byteStart']:rng['byteEnd']].decode('utf-8', 'replace')
                el = x.get('elementBytes') or x.get('element_bytes')
                elem = b[el[0]:el[1]].decode('utf-8', 'replace') if el else ''
                ok = html.unescape(re.sub('<[^>]+>', '', raw)) == x['text'] and elem.startswith('<') and x['text'].strip() in html.unescape(re.sub('<[^>]+>', '', elem))
                checked += 1; bad += 0 if ok else 1
                lines.append(f"| {k} | {key} | {x['text'].strip()} | {rng['charStart']}–{rng['charEnd']} | {rng['byteStart']}–{rng['byteEnd']} | {el[0]}–{el[1]} | {'yes' if ok else 'NO'} |")
    with open(os.path.join(out, 'removed-ranges.md'), 'w') as o:
        o.write('# A1.1: every reference removed from a verse, as a range of the original\n\n')
        o.write(f'Each row is an excluded `apparatus` range of a verified verse: the text the recogniser took out, its character range in the entry\'s visible text, its byte range in the stored entry and the bytes of the whole element. *Exact*: the stored bytes of the range, with tags removed and entities decoded, are the text, and the element\'s bytes are one whole element holding it. **{checked} ranges checked, {bad} not exact.**\n\n')
        o.write('| Edition | Verse | Removed text | Characters | Bytes | Element bytes | Exact |\n|---|---|---|---|---|---|---|\n')
        o.write('\n'.join(lines) + '\n')
    print('rows', len(rows), 'removed ranges', checked, 'not exact', bad)

if __name__ == '__main__':
    main()
