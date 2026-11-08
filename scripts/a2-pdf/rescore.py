"""Recompute the scores of one document from the saved engine texts (out/text/<engine>/<key>.txt.gz)."""
import sys, os, json, gzip, time
sys.path.insert(0, '/root/a2-pdf/scripts')
import score as S
OUT = '/root/a2-pdf/out'
key = sys.argv[1]
rp = f'{OUT}/results/{key}.json'
res = json.load(open(rp))
verses = S.reference(None, None, key.split('~')[0])  # cached
for eng, r in res['engines'].items():
    t = f"{OUT}/text/{eng.replace(':', '_')}/{key}.txt.gz"
    if not os.path.exists(t): continue
    txt = gzip.open(t, 'rt', encoding='utf-8').read()
    lines = txt.replace('\f', '\n').split('\n')
    if r['meta'].get('ok') or txt.strip():
        r['score'] = S.score(lines, verses, rtl=res['hebrew'], hebrew=res['hebrew'])
json.dump(res, open(rp, 'w'), ensure_ascii=False, indent=1)
print(key, {e: round(r['score']['letters']['in_order_book_share'], 3) for e, r in res['engines'].items() if 'score' in r})
