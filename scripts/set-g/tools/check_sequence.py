"""Conversion check: verify that every reference verse (USFM zip) occurs, in order,
in the linear text of a document.

The document text is produced separately (EPUB/DOCX XML text, pdftotext output,
the plain text itself ...). Comparison is on letters only (case-folded, Unicode
NFC, marks kept), so line breaks, hyphenation at line ends, spacing and
punctuation do not matter; word order and spelling do.

For each verse:
  exact   - its letter string occurs contiguously after the previous verse
  gapped  - all its words occur in order, interrupted by other material
            (running heads, page numbers, notes, verse numbers ...); the gap
            text is reported
  missing - not found
Usage: python3 -I check_sequence.py REF.zip DOC.txt OUT_PREFIX [--books A,B] [--strip-regex R]
       [--book-order A,B,...] [--numcheck arabic|cjk|hebrew] [--window N] [--rtl-lines] [--base (compare letters without combining marks)]
"""
import json
import re
import sys
import unicodedata

sys.path.insert(0, "/root/aw-set-g/tools")
import usfmlib as U  # noqa: E402


CJK = {"〇": 0, "一": 1, "二": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9}


def cjk_val(s):
    return str(int("".join(str(CJK[c]) for c in s)))


HEB = {c: v for c, v in zip("אבגדהוזחטיכלמנסעפצקרשת", [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 200, 300, 400])}
HEB.update({"ך": 20, "ם": 40, "ן": 50, "ף": 80, "ץ": 90})


def heb_val(s):
    return str(sum(HEB.get(c, 0) for c in s if c in HEB))


NUMPAT = {"arabic": r"\d+", "cjk": r"[〇一二三四五六七八九]+", "hebrew": r"[אבגדהוזחטיכלמנסעפצקרשתךםןףץ]{1,3}(?=[׳״\s.:]*$|[׳״])"}
NUMVAL = {"arabic": lambda s: str(int(s)), "cjk": cjk_val, "hebrew": heb_val}


BASE_ONLY = False


def letters(s, keep_map=False):
    s = unicodedata.normalize("NFC", s)
    if BASE_ONLY:
        s = "".join(c for c in unicodedata.normalize("NFD", s) if unicodedata.category(c)[0] != "M")
    out, idx = [], []
    for i, ch in enumerate(s):
        cat = unicodedata.category(ch)
        if cat[0] == "L" or cat[0] == "M":
            f = ch.casefold()
            out.append(f)
            idx.extend([i] * len(f))
    return ("".join(out), idx) if keep_map else "".join(out)


def main():
    ref, doc, outp = sys.argv[1:4]
    args = sys.argv[4:]
    books_filter = None
    strip = None
    window = 6000
    numcheck = None
    book_order = None
    rtl = False
    i = 0
    while i < len(args):
        if args[i] == "--books":
            books_filter = set(args[i + 1].split(","))
            i += 2
        elif args[i] == "--strip-regex":
            strip = args[i + 1]
            i += 2
        elif args[i] == "--base":
            global BASE_ONLY
            BASE_ONLY = True
            i += 1
        elif args[i] == "--rtl-lines":
            rtl = True
            i += 1
        elif args[i] == "--book-order":
            book_order = args[i + 1].split(",")
            i += 2
        elif args[i] == "--numcheck":
            numcheck = args[i + 1]  # "arabic" or a python expression name in NUMERALS
            i += 2
        elif args[i] == "--window":
            window = int(args[i + 1])
            i += 2
        else:
            raise SystemExit("bad arg " + args[i])
    books = U.read_usfm_zip(ref)
    if books_filter:
        books = [b for b in books if b["code"] in books_filter]
    if book_order:
        bo = {c: i for i, c in enumerate(book_order)}
        books.sort(key=lambda b: bo[b["code"]])
    raw = open(doc, encoding="utf-8").read()
    if rtl:
        # pdftotext gives right-to-left lines in visual order (with embedding marks):
        # restore logical order by reversing the words of each line
        raw = "\n".join(" ".join(reversed(re.sub("[\u202a-\u202e\u200e\u200f]", "", l).split()))
                         for l in raw.split("\n"))
    if strip:
        raw = re.sub(strip, " ", raw)
    dl, dmap = letters(raw, keep_map=True)
    pos = 0
    res = {"exact": 0, "gapped": 0, "missing": 0, "empty": 0}
    details = []
    for b in books:
        ch = 0
        for it in b["items"]:
            if it[0] == "c":
                ch = it[1]
                continue
            if it[0] != "v":
                continue
            txt = U.clean_ws(re.sub(r"\x01\d+\x01", " ", it[2]))
            key = f"{b['code']} {ch}:{it[1]}"
            vl = letters(txt)
            if not vl:
                res["empty"] += 1
                details.append((key, "empty", ""))
                continue
            j = dl.find(vl, pos, pos + window + len(vl))
            if j < 0 and len(vl) >= 25:
                # not near the previous verse: look further on (reported as a jump)
                j = dl.find(vl, pos)
                if j >= 0:
                    details.append((key, "jump", f"found {j - pos} letters after the previous verse"))
            if j >= 0:
                res["exact"] += 1
                if numcheck:
                    before = raw[max(0, dmap[j] - 60):dmap[j]]
                    nums = re.findall(NUMPAT[numcheck], before)
                    got = nums[-1] if nums else None
                    want = str(it[1])
                    val = NUMVAL[numcheck](got) if got else None
                    if val != want and not (want == "1" and (val in (None, str(ch)))):
                        res.setdefault("number_mismatch", 0)
                        res["number_mismatch"] += 1
                        details.append((key, "number", f"printed {got!r} before verse; context {before[-40:]!r}"))
                    else:
                        res.setdefault("number_ok", 0)
                        res["number_ok"] += 1
                pos = j + len(vl)
                continue
            # gapped: words in order
            words = [letters(w) for w in txt.split()]
            words = [w for w in words if w]
            cur = pos
            gaps = []
            ok = True
            start = None
            for wi, w in enumerate(words):
                lim = cur + (window if wi == 0 else 1500) + len(w)
                k = dl.find(w, cur, lim)
                if k < 0:
                    # tolerate a word split by a gap (hyphenated across a page break)
                    found = False
                    for cut in range(2, len(w) - 1):
                        k1 = dl.find(w[:cut], cur, lim)
                        if k1 >= 0:
                            k2 = dl.find(w[cut:], k1 + cut, k1 + cut + 1500)
                            if k2 >= 0:
                                if wi > 0 and k1 != cur:
                                    gaps.append(raw[dmap[cur - 1] + 1:dmap[k1]].strip()[:120])
                                gaps.append(raw[dmap[k1 + cut - 1] + 1:dmap[k2]].strip()[:120])
                                if start is None:
                                    start = k1
                                cur = k2 + len(w) - cut
                                found = True
                                break
                    if not found:
                        ok = False
                        break
                    continue
                if start is None:
                    start = k
                if wi > 0 and k != cur:
                    gaps.append(raw[dmap[cur - 1] + 1:dmap[k]].strip()[:120])
                cur = k + len(w)
            if ok:
                res["gapped"] += 1
                pos = cur
                details.append((key, "gapped", " | ".join(g for g in gaps if g)))
            else:
                res["missing"] += 1
                details.append((key, "missing", txt[:160]))
    total = res["exact"] + res["gapped"] + res["missing"] + res["empty"]
    res["total"] = total
    res["found_rate"] = round((res["exact"] + res["gapped"]) / total, 6) if total else 0
    with open(outp + ".json", "w", encoding="utf-8") as f:
        json.dump(res, f, ensure_ascii=False, indent=1)
    with open(outp + ".details.tsv", "w", encoding="utf-8") as f:
        for d in details:
            f.write("\t".join(d) + "\n")
    print(json.dumps(res, ensure_ascii=False))


if __name__ == "__main__":
    main()
