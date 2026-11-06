"""Second encoding for check (c) of G11: Diego Renato dos Santos' Nestle 1904 base text
(biblicalhumanities/Nestle1904 xhtml/*-ap.xhtml) -> USFM zip.
<h1> book, <h2> chapter number, <span class="T1">n</span> verse number (class name may
vary: any span whose whole content is a number), text up to the next verse number.
Usage: python3 -I parse_nestle_xhtml.py XHTMLDIR OUT.zip
"""
import glob
import html
import os
import re
import sys

sys.path.insert(0, "/root/aw-set-g/tools")
import usfmlib as U  # noqa: E402

FN = {"Matthew": "MAT", "Mark": "MRK", "Luke": "LUK", "John": "JHN", "Acts": "ACT", "Romans": "ROM",
      "1-Corinthians": "1CO", "2-Corinthians": "2CO", "Galatians": "GAL", "Ephesians": "EPH",
      "Philippians": "PHP", "Colossians": "COL", "1-Thessalonians": "1TH", "2-Thessalonians": "2TH",
      "1-Timothy": "1TI", "2-Timothy": "2TI", "Titus": "TIT", "Philemon": "PHM", "Hebrews": "HEB",
      "James": "JAS", "1-Peter": "1PE", "2-Peter": "2PE", "1-John": "1JN", "2-John": "2JN",
      "3-John": "3JN", "Jude": "JUD", "Revelation": "REV"}


def main(d, out):
    books = []
    for f in glob.glob(os.path.join(d, "*-ap.xhtml")):
        code = FN[os.path.basename(f)[:-len("-ap.xhtml")]]
        x = open(f, encoding="utf-8").read()
        x = x[x.find("<body"):]
        x = re.sub(r"<h2[^>]*>(?:<a[^>]*></a>)?\s*(\d+)\s*</h2>", lambda m: f"\x02C{m.group(1)}\x02", x)
        x = re.sub(r"<span class=\s*\"T\d+\">\s*(\d+)\s*</span>", lambda m: f"\x02V{m.group(1)}\x02", x)
        x = re.sub(r"<h1.*?</h1>", " ", x, flags=re.S)
        x = re.sub(r"<[^>]+>", " ", x)
        x = html.unescape(x)
        b = {"code": code, "items": []}
        for tok in re.split(r"(\x02[CV]\d+\x02)", x):
            m = re.match(r"\x02([CV])(\d+)\x02", tok)
            if m and m.group(1) == "C":
                b["items"].append(("c", int(m.group(2))))
            elif m:
                b["items"].append(("v", int(m.group(2)), "", []))
            elif b["items"] and b["items"][-1][0] == "v":
                v = b["items"][-1]
                b["items"][-1] = ("v", v[1], U.clean_ws(v[2] + " " + tok), [])
        books.append(b)
    order = {c: i for i, c in enumerate(U.BOOK_ORDER)}
    books.sort(key=lambda b: order[b["code"]])
    U.write_usfm_zip(books, out, "grcN1904xhtml")
    print(len(books), "books")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
