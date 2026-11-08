"""Build the document list (PDF rows of the corpus manifests) and verify checksums."""
import csv, hashlib, json, os, sys
CORPUS = '/home/user/abidingword/docs/corpus'
LOC = {'withheld-g': '/root/aw-set-g/docs', 'withheld-f': '/root/aw-set-f/docs',
       'withheld-e': '/root/aw-set-e/docs', 'recon-d': '/root/aw-recon',
       'recon-bc': '/root/aw-recon-bc', 'impl': '/root/aw-corpus', 'regression': '/root/aw-corpus'}
AUD = {'withheld-g': '/root/audit/withheld-g.ORIGINAL', 'withheld-f': '/root/audit/v7-f',
       'withheld-e': '/root/audit/v7-e', 'recon-d': '/root/audit/v7-recon',
       'recon-bc': '/root/audit/v7-bc', 'impl': '/root/audit/v7-dev', 'regression': '/root/audit/v7-dev'}
def sha(p):
    h = hashlib.sha256()
    with open(p, 'rb') as f:
        for b in iter(lambda: f.read(1 << 20), b''): h.update(b)
    return h.hexdigest()
audit_sums = {}
for line in open('/home/user/abidingword/docs/audit/v0.24.7/corpus-checksums.sha256'):
    s, p = line.split(None, 1); audit_sums['/root/' + p.strip()] = s
seal = {}
for line in open('/root/aw-set-g/SEAL.txt'):
    parts = line.split()
    if len(parts) == 2 and len(parts[0]) == 64: seal['/root/aw-set-g/' + parts[1]] = parts[0]
docs = []
for fn in sorted(os.listdir(CORPUS)):
    if not fn.endswith('.tsv') or fn == 'sources.tsv': continue
    with open(os.path.join(CORPUS, fn)) as f:
        for r in csv.DictReader(f, delimiter='\t'):
            if r['format'] != 'pdf': continue
            role = r['role']
            if role not in LOC: continue
            d = LOC[role]
            pdf = os.path.join(d, r['files']); ref = os.path.join(d, r['reference'])
            exp = []
            if r.get('sha256') and len(r['sha256']) == 64: exp.append(('manifest', r['sha256']))
            if pdf in audit_sums: exp.append(('corpus-checksums', audit_sums[pdf]))
            if pdf in seal: exp.append(('SEAL', seal[pdf]))
            res = {'key': r['key'], 'tsv': fn, 'role': role, 'lang': r['language'], 'script': r['script'],
                   'layout': r['layout'][:160], 'books_col': r.get('books'), 'pdf': pdf, 'ref': ref, 'exists': os.path.exists(pdf),
                   'ref_exists': os.path.exists(ref)}
            if res['exists']:
                got = sha(pdf); res['sha256'] = got; res['size'] = os.path.getsize(pdf)
                res['checks'] = [{'source': s, 'expected': e, 'match': e == got} for s, e in exp]
                res['verified'] = bool(exp) and all(c['match'] for c in res['checks'])
            if res['ref_exists']:
                rs = sha(ref); rexp = audit_sums.get(ref) or seal.get(ref)
                res['ref_check'] = None if rexp is None else (rexp == rs)
            a = os.path.join(AUD[role], r['key'] + '.json')
            res['audit_json'] = a if os.path.exists(a) else None
            docs.append(res)
json.dump(docs, open('/root/a2-pdf/out/docs.json', 'w'), indent=1)
for d in docs:
    print(d['key'], d['role'], d['exists'], d.get('verified'), [c['source'] for c in d.get('checks', [])], d.get('ref_check'), d.get('size'), bool(d['audit_json']))
print(len(docs))
