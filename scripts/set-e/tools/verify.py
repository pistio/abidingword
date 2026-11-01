"""Self-check: share of source verses whose text (whitespace removed) appears in the document's extracted text."""
import sys, re, zipfile, subprocess, html, unicodedata
sys.path.insert(0, '/root/aw-set-e/tools')
import usfm
doc, src = sys.argv[1], sys.argv[2]
codes = sys.argv[3].split(',') if len(sys.argv) > 3 and sys.argv[3] != 'all' else None
def norm(s):
    s = unicodedata.normalize('NFC', s)
    return re.sub(r'\s+|­', '', s)
if doc.endswith('.pdf'):
    t = subprocess.run(['pdftotext'] + ([] if 'heb' in doc else ['-raw']) + [doc, '-'], capture_output=True, text=True).stdout
    t = re.sub(r'-\n', '', t)  # undo hyphenation (may also join real hyphens)
elif doc.endswith(('.docx', '.epub')):
    z = zipfile.ZipFile(doc); parts = []
    for n in z.namelist():
        if n.endswith(('.xml', '.xhtml')) and ('word/document' in n or n.endswith('.xhtml')):
            x = z.read(n).decode('utf-8')
            x = re.sub(r'<sup[^>]*>.*?</sup>|<span class="vn">.*?</span>|<a class="fn".*?</a>', '', x)
            x = re.sub(r'<w:tab/>', ' ', x)
            x = re.sub(r'<[^>]+>', '', x); parts.append(html.unescape(x))
    t = '\n'.join(parts)
else:
    t = open(doc, encoding='utf-8').read()
    t = re.sub(r'(?m)\\$', '', t)
    t = re.sub(r'(?m)^> ?', '', t).replace('\u2003', '')
    t = re.sub(r'\\(.)', r'\1', t)
    t = re.sub(r'\[\^[^\]]+\]', '', t).replace('*', '')
T = norm(t)
books = usfm.load(src)
tot = ok = 0; miss = []
for bk in books.values():
    if codes and bk.code not in codes: continue
    for ch, v, txt in usfm.verses(bk):
        n = norm(txt)
        if not n: continue
        tot += 1
        if n in T: ok += 1
        elif len(miss) < 3: miss.append((bk.code, ch, v, txt[:80]))
print('%s: %d/%d verses found (%.2f%%)' % (doc.split('/')[-1], ok, tot, 100.0 * ok / max(tot, 1)), miss)
