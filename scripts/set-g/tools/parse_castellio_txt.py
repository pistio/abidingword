r"""Second encoding for check (c) of G14: the project's own plain-text files
(TXT/4 - Macronized, same commit) -> USFM zip.  "CAPUT <roman>" starts a chapter,
"N. text" starts verse N (also after a tab inside poetry), other lines continue a verse.
Usage: python3 -I parse_castellio_txt.py TXT_DIR OUT.zip
"""
import os
import re
import sys

sys.path.insert(0, "/root/aw-set-g/tools")
import usfmlib as U  # noqa: E402

FILES = [("Vetus Fœdus/genesis.txt", "GEN"), ("Novum Fœdus/matthaeus.txt", "MAT"),
         ("Novum Fœdus/marcus.txt", "MRK"), ("Novum Fœdus/lucas.txt", "LUK"),
         ("Novum Fœdus/johannes.txt", "JHN")]


def roman(s):
    vals = {"I": 1, "V": 5, "X": 10, "L": 50, "C": 100}
    tot = 0
    for i, c in enumerate(s):
        v = vals[c]
        tot += -v if i + 1 < len(s) and vals[s[i + 1]] > v else v
    return tot


def main(d, out):
    books = []
    for rel, code in FILES:
        b = {"code": code, "items": []}
        for line in open(os.path.join(d, rel), encoding="utf-8").read().split("\n"):
            s = line.strip()
            m = re.fullmatch(r"CAPUT ([IVXLC]+)", s)
            if m:
                b["items"].append(("c", roman(m.group(1))))
                continue
            if not s or s.startswith("LIBER:"):
                continue
            vm = re.match(r"(\d+)\.\s+(.*)", s)
            if vm:
                b["items"].append(("v", int(vm.group(1)), vm.group(2), []))
            elif b["items"] and b["items"][-1][0] == "v":
                v = b["items"][-1]
                b["items"][-1] = ("v", v[1], v[2] + " " + s, [])
        books.append(b)
    U.write_usfm_zip(books, out, "laCastellioTXT")
    print(sum(1 for b in books for i in b["items"] if i[0] == "v"), "verses")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
