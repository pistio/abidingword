#!/usr/bin/env python3
"""Builder's self-check: is every reference verse text present, in order, in the
document's extracted text? (Not the evaluation; only guards against a renderer
silently dropping text.) usage: verify.py DOC REF.zip"""
import sys, os, re, zipfile, unicodedata
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import usfm
from common import load_zip

SKIP = re.compile(r'[\s\-­‐‑​    ‎‏]+')


def norm(s):
    s = unicodedata.normalize('NFC', s)
    return SKIP.sub('', s)


MINSIZE = float(os.environ.get('MINSIZE', '0'))


def doc_text(path):
    ext = path.rsplit('.', 1)[1].lower()
    if ext == 'pdf':
        import pymupdf
        d = pymupdf.open(path)
        out = []
        for p in d:
            h = p.rect.height
            for b in p.get_text('dict')['blocks']:
                for ln in b.get('lines', []):
                    y = (ln['bbox'][1] + ln['bbox'][3]) / 2
                    if h * 0.08 < y < h * 0.935:
                        out.append(''.join(sp['text'] for sp in ln['spans'] if sp['size'] >= MINSIZE))
        return '\n'.join(out)
    if ext in ('txt', 'md'):
        t = open(path, encoding='utf-8-sig').read()
        if ext == 'md':
            t = re.sub(r'\[\^[^\]]+\]', '', t)
            t = re.sub(r'<[^>]+>', '', t)
            t = re.sub(r'(?m)^> ?', '', t)
            t = t.replace('*', '').replace('\\', '')
        return t
    if ext == 'docx':
        z = zipfile.ZipFile(path)
        x = z.read('word/document.xml').decode('utf-8')
        x = re.sub(r'<w:footnoteReference[^>]*/>', '', x)
        x = re.sub(r'</w:p>', '\n', x)
        x = re.sub(r'<w:tab/>', ' ', x)
        return re.sub(r'<[^>]+>', '', x).replace('&amp;', '&').replace('&lt;', '<').replace('&gt;', '>').replace('&quot;', '"').replace('&apos;', "'")
    if ext == 'epub':
        z = zipfile.ZipFile(path)
        import html
        parts = []
        names = [n for n in z.namelist() if n.endswith(('.xhtml', '.html', '.htm'))]
        # spine order
        opf = [n for n in z.namelist() if n.endswith('.opf')][0]
        o = z.read(opf).decode('utf-8')
        base = os.path.dirname(opf)
        man = dict(re.findall(r'<item[^>]*?id="([^"]+)"[^>]*?href="([^"]+)"', o))
        man.update({a: b for b, a in re.findall(r'<item[^>]*?href="([^"]+)"[^>]*?id="([^"]+)"', o)})
        for idref in re.findall(r'<itemref[^>]*idref="([^"]+)"', o):
            n = os.path.join(base, man[idref]) if base else man[idref]
            x = z.read(n).decode('utf-8')
            x = re.sub(r'<a[^>]*class="[^"]*(noteref|fn)[^"]*"[^>]*>.*?</a>', '', x, flags=re.S)
            x = re.sub(r'<(aside|section)[^>]*epub:type="(footnote|endnotes|footnotes)"[^>]*>.*?</\1>', '', x, flags=re.S)
            x = re.sub(r'<sup[^>]*class="[^"]*fnref[^"]*"[^>]*>.*?</sup>', '', x, flags=re.S)
            parts.append(html.unescape(re.sub(r'<[^>]+>', ' ', x)))
        return '\n'.join(parts)
    raise SystemExit('unknown ext')


def main():
    doc, ref = sys.argv[1:3]
    order = sys.argv[3].split(',') if len(sys.argv) > 3 else None
    text = norm(doc_text(doc))
    books = load_zip(ref, order)
    pos = 0
    miss = 0
    total = 0
    for b in books:
        for c, v, t in usfm.verses_of(b):
            total += 1
            n = norm(t)
            if not n:
                continue
            k = text.find(n, pos)
            if k < 0:
                k2 = text.find(n)
                miss += 1
                if miss <= 8:
                    print('MISSING', b['code'], c, v, t[:90], '(found elsewhere)' if k2 >= 0 else '')
                continue
            pos = k + len(n)
    print('%s: %d verses, %d not found in order' % (os.path.basename(doc), total, miss))


if __name__ == '__main__':
    main()
