r"""Convert Project Gutenberg #62383 (A Biblia Sagrada, Almeida, Lisboa 1911 printing)
HTML (pg62383-images.html, from pg62383-h.zip) to USFM.

Structure used (publisher's own markup):
  <h2 id="GEN">           book start
  <h3><i>...</i></h3>     chapter summary  -> \\cd
  <div class="sidenote">  marginal chronology -> \\rem (not verse text)
  <p class="center">      psalm title  -> \\d
  <p id="Gen1-2">N text   verse; first verse of a chapter has <span class="first">C</span>
  <a class="fnanchor" href="#Footnote_N">  note anchor -> \\x (reference lists) or \\f
Usage: python3 -I conv_pt_gutenberg62383.py <html> <out.zip> <report.txt>
"""
import html as H
import re
import sys

sys.path.insert(0, "/root/aw-set-g/tools")
import usfmlib as U  # noqa: E402

BOOKS = ['GEN', 'EXO', 'LEV', 'NUM', 'DEU', 'JOS', 'JUI', 'RUTH', 'ISAM', 'IISAM', 'IREIS',
         'IIREIS', 'ICHR', 'IICHR', 'ESD', 'NEH', 'EST', 'JOB', 'PSA', 'PRO', 'ECC', 'CAN',
         'ISA', 'JER', 'LAM', 'EZE', 'DAN', 'OSE', 'JOEL', 'AMOS', 'OBA', 'JON', 'MIQ', 'NAH',
         'HAB', 'SOF', 'AGG', 'ZAC', 'MAL', 'MAT', 'MAR', 'LUC', 'JOAO', 'ACT', 'ROM', 'ICOR',
         'IICOR', 'GAL', 'EPH', 'IPHI', 'COL', 'ITHE', 'IITHE', 'ITIM', 'IITIM', 'TITO',
         'IIPHI', 'HEB', 'THI', 'IPED', 'IIPED', 'IJOAO', 'IIJOAO', 'IIIJOAO', 'JUD', 'APO']
HEBLET = set('Aleph Beth Gimel Daleth He Vau Zain Heth Teth Jod Caph Lamed Mem Nun Samech Ain Pe Tsade Koph Coph Resch Res Schin Sin Tau'.split())
CODE = dict(zip(BOOKS, U.BOOK_ORDER[:66]))


def text_of(frag):
    frag = re.sub(r'<span class="pagenum">.*?</span>', ' ', frag, flags=re.S)
    frag = re.sub(r'<br\s*/?>', ' ', frag)
    frag = re.sub(r'<[^>]+>', '', frag)
    return U.clean_ws(H.unescape(frag))


def main(src, out, report):
    h = open(src, encoding='utf-8').read()
    # footnotes
    fn = {}
    for m in re.finditer(r'<div class="footnote">\s*<p><a id="Footnote_(\d+)"></a>(.*?)</div>', h, re.S):
        body = re.sub(r'<span class="label">\[\d+\]</span>', '', m.group(2))
        fn[m.group(1)] = text_of(body)
    en = {}
    for m in re.finditer(r'<p><a id="Endnote_(\d+)"></a>(.*?)</p>', h, re.S):
        body = re.sub(r'<span class="label">\[[A-Z]+\]</span>', '', m.group(2))
        en[m.group(1)] = text_of(body)
    # body: split at h2 books
    parts = re.split(r'(<h2 id="[A-Z]+">)', h)
    books = []
    rep = []
    cur = None
    for part in parts:
        mm = re.match(r'<h2 id="([A-Z]+)">', part)
        if mm:
            if mm.group(1) in CODE:
                cur = {"code": CODE[mm.group(1)], "items": [], "src": mm.group(1)}
                books.append(cur)
            else:
                cur = None
            continue
        if cur is None:
            continue
        # title
        t = re.match(r'(.*?)</h2>', part, re.S)
        if t and not cur.get("mt"):
            cur["mt"] = [text_of(x) for x in re.split(r'<br\s*/?>', t.group(1)) if text_of(x)]
            cur["h"] = text_of(t.group(1).split('<br')[-1]) if False else None
            part = part[t.end():]
        # cut off footnote blocks for this chapter area
        part = re.sub(r'<div class="footnotes">.*?(?=<div class="chapter">|$)', '', part, flags=re.S)
        part = re.sub(r'<div class="footnote">.*?</div>', '', part, flags=re.S)
        toks = re.finditer(r'<h3[^>]*>(.*?)</h3>|<div class="sidenote">(.*?)</div>|<p([^>]*)>(.*?)</p>', part, re.S)
        last_ch = 0
        for tk in toks:
            if tk.group(1) is not None:
                cur["items"].append(("cd", text_of(tk.group(1))))
                continue
            if tk.group(2) is not None:
                cur["items"].append(("rem", "margin: " + text_of(tk.group(2))))
                continue
            attrs, body = tk.group(3), tk.group(4)
            idm = re.search(r'id="([A-Za-z]+?)(\d+)-(\d+)"', attrs)
            if not idm:
                txt = text_of(body)
                if not txt:
                    continue
                if 'class="center"' in attrs and cur['code'] == 'PSA' and txt.rstrip('.') in HEBLET:
                    cur["items"].append(("qa", txt))
                elif 'class="center"' in attrs:
                    cur["items"].append(("d", txt))
                else:
                    rep.append(f"{cur['code']} after ch{last_ch}: non-verse <p{attrs}>: {txt[:80]}")
                    cur["items"].append(("rem", "unnumbered paragraph: " + txt))
                continue
            ch, vs = int(idm.group(2)), int(idm.group(3))
            notes = []

            def note_rep(m):
                n = fn.get(m.group(1), '')
                kind = 'x' if re.fullmatch(r"[\w\s\.,;:\-–’'eé]*\d[\w\s\.,;:\-–’'eé]*", n) and len(n) < 400 else 'f'
                notes.append((kind, n))
                return f" \x01{len(notes) - 1}\x01 "

            def en_rep(m):
                notes.append(('f', en.get(m.group(1), '')))
                if m.group(1) not in en:
                    rep.append(f"{cur['code']} {ch}:{vs} missing endnote {m.group(1)}")
                return f" \x01{len(notes) - 1}\x01 "

            b2 = re.sub(r'<a id="FNanchor_\d+"></a><a class="fnanchor pginternal" href="#Footnote_(\d+)">\[\d+\]</a>', note_rep, body)
            b2 = re.sub(r'<a id="ENanchor_\d+"></a><a class="fnanchor pginternal" href="#Endnote_(\d+)">\[[A-Z]+\]</a>', en_rep, b2)
            acro = None
            am = re.match(r'\s*<span class="smcap">([A-Z][a-z]+)\.</span>\s*((?: \x01\d+\x01 \s*)*)(?=<span class="first">|\d)', b2)
            if am and cur['code'] in ('LAM', 'PSA'):
                acro = am.group(1) + '.'
                b2 = b2[am.end():]
                if am.group(2).strip():
                    # a note anchor printed between the acrostic letter and the verse number:
                    # keep it at the start of the verse text
                    b2 = re.sub(r'^(\s*(?:<span class="first">\d+</span>|\d+))', lambda m: m.group(1) + am.group(2), b2, count=1)
            fm = re.match(r'\s*<span class="first">(\d+)</span>', b2)
            if fm:
                if int(fm.group(1)) != ch:
                    rep.append(f"{cur['code']} {ch}:{vs} chapter span {fm.group(1)} != id")
                b2 = b2[fm.end():]
                num_ok = vs == 1
            else:
                nm = re.match(r'\s*(\d+)\s', text_of(re.sub(r'\x01\d+\x01', '', b2)) + ' ')
                num_ok = bool(nm) and int(nm.group(1)) == vs
                b2 = re.sub(r'^\s*(<[^>]+>\s*)*' + str(vs) + r'(?=\s)', '', b2, count=1) if num_ok else b2
            if not num_ok:
                rep.append(f"{cur['code']} {ch}:{vs} printed number mismatch: {text_of(body)[:60]}")
            txt = text_of(b2)
            txt = re.sub(r'\s*\x01(\d+)\x01\s*', lambda m: f"\x01{m.group(1)}\x01 ", txt)
            txt = re.sub(r'\x01(\d+)\x01 ([,.;:!?)])', lambda m: '\x01' + m.group(1) + '\x01' + m.group(2), txt).strip()
            if ch != last_ch:
                # move the preceding cd/rem/d of this chapter after \c
                items = cur["items"]
                moved = []
                while items and items[-1][0] in ("cd", "rem", "d", "qa"):
                    moved.insert(0, items.pop())
                items.append(("c", ch))
                items.extend(moved)
                last_ch = ch
            # \p before each verse paragraph (each verse is its own paragraph in the source)
            cur["items"].append(("p",))
            if acro:
                cur["items"].append(("qa", acro))
            cur["items"].append(("v", vs, txt, notes))
    for b in books:
        b["idtext"] = "Biblia Sagrada (Almeida), Lisboa 1911 - Project Gutenberg #62383"
        b["h"] = None
        b.pop("src", None)
    U.write_usfm_zip(books, out, "ptARC1911")
    with open(report, 'w', encoding='utf-8') as f:
        f.write("\n".join(rep) + "\n")
    st = U.structure(books)
    print(len(books), "books", sum(sum(c.values()) for c in st.values()), "verses;", len(rep), "report lines")


if __name__ == '__main__':
    main(*sys.argv[1:4])
