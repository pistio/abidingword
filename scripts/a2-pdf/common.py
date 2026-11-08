"""Shared helpers: USFM reference parsing and text normalisation."""
import re, unicodedata, zipfile, json, os

NON_VERSE_PARA = re.compile(r'^\\(s\d?|ms\d?|mr|r|d|sp|cl|cp|cd|qa|toc\d?|toca\d?|h\d?|mt\d?|mte\d?|imt\d?|ip|ipi|im|is\d?|io\d?|iot|ili\d?|ie|rem|id|ide|sr|sts|restore|usfm|periph|lit|ib|ipq|iq\d?|imq|iex)\b')
STRIP_SPANS = [re.compile(r'\\f\s.*?\\f\*', re.S), re.compile(r'\\fe\s.*?\\fe\*', re.S),
               re.compile(r'\\ef\s.*?\\ef\*', re.S),
               re.compile(r'\\x\s.*?\\x\*', re.S), re.compile(r'\\va\s.*?\\va\*', re.S),
               re.compile(r'\\vp\s.*?\\vp\*', re.S), re.compile(r'\\ca\s.*?\\ca\*', re.S),
               re.compile(r'\\fig\s?.*?\\fig\*', re.S), re.compile(r'\\rq\s.*?\\rq\*', re.S)]
ATTR = re.compile(r'\|[^\\]*?(?=\\[a-z+]+\d?\*)')   # \w word|strong="..."\w*
MARKER = re.compile(r'\\\+?[a-z]+\d*\*?')

def parse_usfm_zip(path, books=None):
    """Return list of (book, chapter, verse, raw_text) in reference order."""
    out = []
    with zipfile.ZipFile(path) as z:
        names = sorted(n for n in z.namelist() if n.lower().endswith(('.usfm', '.sfm')))
        for n in names:
            data = z.read(n).decode('utf-8-sig', 'replace')
            m = re.search(r'\\id\s+(\S+)', data)
            if not m: continue
            book = m.group(1).upper()
            if books is not None and book not in books: continue
            out.extend(parse_usfm_book(book, data))
    return out

def parse_usfm_book(book, data):
    for sp in STRIP_SPANS: data = sp.sub(' ', data)
    verses = []; chap = None; cur = None
    # split into paragraph-marker lines
    lines = re.split(r'\n(?=\\)', data.replace('\r', ''))
    for ln in lines:
        ln = ln.strip()
        if not ln: continue
        mc = re.match(r'\\c\s+(\S+)', ln)
        if mc:
            chap = mc.group(1); cur = None; continue
        if NON_VERSE_PARA.match(ln):
            continue
        # a line may contain several \v
        parts = re.split(r'\\v\s+(\S+)\s?', ln)
        head = parts[0]
        if cur is not None: cur[3].append(head)
        for i in range(1, len(parts), 2):
            cur = [book, chap, parts[i], [parts[i + 1]]]
            verses.append(cur)
    res = []
    for b, c, v, chunks in verses:
        t = ' '.join(chunks)
        t = ATTR.sub('', t)
        t = MARKER.sub(' ', t)
        res.append((b, c, v, re.sub(r'\s+', ' ', t).strip()))
    return res

def norm(s, form='NFC', strip_marks=False, digits=True):
    """Letters, digits (and combining marks unless strip_marks) only, case-folded.
    digits=False also drops digits (so stray verse numbers inside a verse do not matter)."""
    if not digits:
        s = unicodedata.normalize(form, s)
        s = ''.join(ch for ch in s if unicodedata.category(ch)[0] in 'LM')
        return unicodedata.normalize(form, s.casefold())
    if strip_marks:
        s = unicodedata.normalize('NFKD', s)
        s = ''.join(ch for ch in s if unicodedata.category(ch)[0] in 'LN')
        return unicodedata.normalize('NFC', s).casefold()
    s = unicodedata.normalize(form, s)
    s = ''.join(ch for ch in s if unicodedata.category(ch)[0] in 'LNM')
    return unicodedata.normalize(form, s.casefold())
