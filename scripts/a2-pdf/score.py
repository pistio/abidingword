"""Text-fidelity scoring of one engine text against the reference verses."""
import sys, json, os, re, gzip, bisect
sys.path.insert(0, '/root/a2-pdf/scripts')
import common

CACHE = '/root/a2-pdf/out/refcache'

def reference(ref_zip, books, key):
    os.makedirs(CACHE, exist_ok=True)
    cp = os.path.join(CACHE, key + '.json.gz')
    if os.path.exists(cp):
        with gzip.open(cp, 'rt') as f: return json.load(f)
    vs = common.parse_usfm_zip(ref_zip, set(books) if books else None)
    vs = [v for v in vs if v[3].strip()]
    with gzip.open(cp, 'wt') as f: json.dump(vs, f, ensure_ascii=False)
    return vs

def occurrences(full, nvs):
    """start positions of every (distinct) verse string in full, via one Aho-Corasick pass."""
    import ahocorasick
    A = ahocorasick.Automaton()
    for nv in set(nvs):
        if nv: A.add_word(nv, nv)
    occ = {}
    if len(A) == 0: return occ
    A.make_automaton()
    for end, nv in A.iter(full):
        l = occ.setdefault(nv, [])
        if len(l) < 5000: l.append(end - len(nv) + 1)
    return occ

def measure(full, nvs, books_of):
    """coverage; global greedy in-order (spec: each verse searched from the end of the previous match);
    per-book greedy in-order (pointer restarts at 0 for each book, so a different book order in the
    document does not count against the engine)."""
    occ = occurrences(full, nvs)
    ptr = 0; cov = 0; ordered = 0; per = {}; bptr = {}; ordered_book = 0
    for nv, b in zip(nvs, books_of):
        if not nv: continue
        pb = per.setdefault(b, [0, 0, 0, 0])
        pb[0] += 1
        o = occ.get(nv)
        if not o: continue
        cov += 1; pb[1] += 1
        i = bisect.bisect_left(o, ptr)
        if i < len(o):
            ordered += 1; ptr = o[i] + len(nv); pb[2] += 1
        q = bptr.get(b, 0); j = bisect.bisect_left(o, q)
        if j < len(o):
            ordered_book += 1; bptr[b] = o[j] + len(nv); pb[3] += 1
    n = sum(1 for v in nvs if v)
    return {'verses': n, 'covered': cov, 'in_order': ordered, 'in_order_book': ordered_book,
            'coverage': cov / n if n else None, 'in_order_share': ordered / n if n else None,
            'in_order_book_share': ordered_book / n if n else None,
            'per_book': per}

def score(lines, verses, rtl=False, hebrew=False, nfkc=True):
    books_of = [v[0] for v in verses]
    res = {}
    full = ''.join(common.norm(l) for l in lines)
    res['norm_chars'] = len(full)
    res['nfc'] = measure(full, [common.norm(v[3]) for v in verses], books_of)
    fulll = ''.join(common.norm(l, digits=False) for l in lines)
    res['letters'] = measure(fulll, [common.norm(v[3], digits=False) for v in verses], books_of)
    if nfkc:
        fullk = ''.join(common.norm(l, 'NFKC') for l in lines)
        res['nfkc'] = measure(fullk, [common.norm(v[3], 'NFKC') for v in verses], books_of)
        for k in ('per_book',): res['nfkc'].pop(k)
    if hebrew:
        fulls = ''.join(common.norm(l, strip_marks=True) for l in lines)
        nvs = [common.norm(v[3], strip_marks=True) for v in verses]
        res['stripped'] = measure(fulls, nvs, books_of); res['stripped'].pop('per_book')
        if rtl:
            # visual-order check: reverse each line's normalised text
            fullr = ''.join(common.norm(l, strip_marks=True)[::-1] for l in lines)
            res['stripped_line_reversed'] = measure(fullr, nvs, books_of); res['stripped_line_reversed'].pop('per_book')
            # words in visual (left-to-right) order but each word's letters logical: reverse word order per line
            fullw = ''.join(common.norm(' '.join(l.split()[::-1]), strip_marks=True) for l in lines)
            res['stripped_word_reversed'] = measure(fullw, nvs, books_of); res['stripped_word_reversed'].pop('per_book')
    if rtl and not hebrew:
        fullr = ''.join(common.norm(l)[::-1] for l in lines)
        res['line_reversed'] = measure(fullr, [common.norm(v[3]) for v in verses], books_of)
        res['line_reversed'].pop('per_book')
    return res
