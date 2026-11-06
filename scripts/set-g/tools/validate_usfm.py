"""Structure summary and sanity check of a reference USFM zip: books, chapters, verses per
chapter; reports duplicate verses, non-increasing chapter/verse numbers, empty verses,
unknown markers. Usage: python3 -I validate_usfm.py REF.zip OUT.json
"""
import json
import re
import sys
import zipfile

sys.path.insert(0, "/root/aw-set-g/tools")
import usfmlib as U  # noqa: E402

KNOWN = set("id usfm ide h toc1 toc2 toc3 mt mt1 mt2 mt3 c v p m q q1 q2 q3 s s1 s2 d cd qa b rem f ft fr fqa fq fk x xt xo nb pi pi1 li li1 add nd w wj tl it bk sc qs pmo mi".split())


def main(ref, out):
    res = {"books": {}, "problems": []}
    with zipfile.ZipFile(ref) as z:
        for n in sorted(z.namelist()):
            if not n.endswith((".usfm", ".sfm")):
                continue
            t = z.read(n).decode("utf-8-sig")
            for mk in set(re.findall(r"\\\+?([a-z]+\d?)\*?", t)):
                if mk not in KNOWN:
                    res["problems"].append(f"{n}: marker \\{mk}")
            b = U.parse_usfm(t)
            if not t.startswith("\\id " + b["code"]):
                res["problems"].append(f"{n}: does not start with \\id")
            chs, seen, lastc, lastv = {}, set(), 0, 0
            empty = 0
            ch = 0
            for it in b["items"]:
                if it[0] == "c":
                    ch = it[1]
                    if ch <= lastc:
                        res["problems"].append(f"{b['code']}: chapter {ch} after {lastc}")
                    lastc, lastv = ch, 0
                    chs[ch] = 0
                elif it[0] == "v":
                    first = int(re.match(r"\d+", str(it[1])).group(0))
                    if (ch, it[1]) in seen:
                        res["problems"].append(f"{b['code']} {ch}:{it[1]} duplicate")
                    if first <= lastv:
                        res["problems"].append(f"{b['code']} {ch}:{it[1]} after {lastv}")
                    seen.add((ch, it[1]))
                    lastv = int(re.findall(r"\d+", str(it[1]))[-1])
                    chs[ch] = chs.get(ch, 0) + 1
                    if not re.sub(r"\x01\d+\x01", "", it[2]).strip():
                        empty += 1
            res["books"][b["code"]] = {"chapters": len(chs), "verses": sum(chs.values()), "empty_verses": empty,
                                       "verses_per_chapter": chs}
    res["total_books"] = len(res["books"])
    res["total_verses"] = sum(v["verses"] for v in res["books"].values())
    json.dump(res, open(out, "w", encoding="utf-8"), indent=1)
    print(res["total_books"], "books", res["total_verses"], "verses", len(res["problems"]), "problems", res["problems"][:5])


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
