r"""Build the G02 reference (USFM) from the explicit verse markup of Project Gutenberg
#2100 (Bibeln 1917, Project Runeberg transcription, plain text).

The publisher's own structure is used: "CCC:VVV text" starts a verse, 8-space indented
lines continue it, a blank line closes a paragraph (\p), book titles come from the file's
own contents table, "[n]" at the start of a verse is a note anchor whose note line
"[n] ..." follows the passage (-> \f at the anchor). Verses printed as a bare number
(no text) are kept as empty \v (they are printed so in the file).
Usage: python3 -I conv_pg2100_txt.py 2100.txt OUT.zip REPORT.txt
"""
import re
import sys

sys.path.insert(0, "/root/aw-set-g/tools")
import usfmlib as U  # noqa: E402
from parse_pg2100_txt import T2C, TITLES  # noqa: E402


def main(src, out, report):
    lines = open(src, encoding="utf-8").read().replace("\r\n", "\n").split("\n")
    start = next(i for i, l in enumerate(lines) if l.startswith("*** START OF"))
    end = next(i for i, l in enumerate(lines) if l.startswith("*** END OF"))
    i0 = [i for i in range(start, end) if lines[i].strip() == TITLES[0]][1]
    books, cur, rep = [], None, []
    midpara = []
    pending_notes = {}  # anchor label -> (book, item index)
    lastch = None
    lastvs = 0
    for l in lines[i0:end]:
        s = l.strip()
        if s in T2C:
            cur = {"code": T2C[s], "items": [], "mt": [s],
                   "idtext": "Bibeln 1917 - Project Gutenberg #2100 (Project Runeberg)"}
            books.append(cur)
            lastch = None
            continue
        m = re.match(r"^(\d{3}):(\d{3})(?: (.*)|(--.*)|\s*)$", l)
        if m:
            ch, vs = int(m.group(1)), int(m.group(2))
            if lastch is not None and vs == 1 and lastvs > 1 and ch != lastch + 1:
                rep.append(f"{cur['code']} printed {m.group(1)}:{m.group(2)} after {lastch}:{lastvs}; "
                           f"taken as the start of chapter {lastch + 1}")
                ch = lastch + 1
            elif lastch is not None and ch != lastch and vs != 1 and vs == lastvs + 1:
                # printed chapter number disagrees with the running sequence (misprint in
                # the file): keep the verse in the running chapter and report it
                rep.append(f"{cur['code']} printed {m.group(1)}:{m.group(2)} inside chapter {lastch} "
                           f"(after {lastch}:{lastvs}); kept as {lastch}:{vs}")
                ch = lastch
            lastvs = vs
            txt = (m.group(3) if m.group(3) is not None else (m.group(4) or "")).strip()
            if ch != lastch:
                cur["items"].append(("c", ch))
                cur["items"].append(("p",))
                lastch = ch
            elif cur["items"][-1] == ("b",):
                cur["items"][-1] = ("p",)
            notes = []
            am = re.match(r"\[(\d+)\]\s*", txt)
            if am:
                notes.append(("f", "?"))
                txt = "\x010\x01 " + txt[am.end():]
                pending_notes[am.group(1)] = (cur, len(cur["items"]))
            cur["items"].append(("v", vs, txt, notes))
            if not txt:
                rep.append(f"{cur['code']} {ch}:{vs} printed without text")
            continue
        if (s and not s.startswith("[") and cur and len(cur["items"]) >= 2
                and cur["items"][-1] == ("b",) and cur["items"][-2][0] == "v"):
            # a verse continued in a new (unindented) paragraph after a blank line
            cur["items"].pop()
            v = cur["items"][-1]
            cur["items"][-1] = ("v", v[1], (v[2] + " " + s).strip(), v[3])
            midpara.append(f"{cur['code']} {lastch}:{v[1]}")
            continue
        if l.startswith("        ") and cur and cur["items"] and cur["items"][-1][0] == "v":
            v = cur["items"][-1]
            cur["items"][-1] = ("v", v[1], (v[2] + " " + s).strip(), v[3])
            continue
        if not s:
            if cur and cur["items"] and cur["items"][-1][0] == "v":
                cur["items"].append(("b",))
            continue
        nm = re.match(r"\[(\d+)\]\s*(.*)", s)
        if nm and nm.group(1) in pending_notes:
            b, idx = pending_notes.pop(nm.group(1))
            v = b["items"][idx]
            b["items"][idx] = ("v", v[1], v[2], [("f", nm.group(2))])
            continue
        rep.append(f"unparsed line in {cur and cur['code']}: {s}")
    for b in books:
        # verse numbers that break the running sequence while the next verse continues it
        # (e.g. 21, 23, 23, 24 or 20, 21, 21 at a chapter end) are misprints: corrected, reported
        ch = 0
        idxs = []
        groups = []
        for k, it in enumerate(b["items"]):
            if it[0] == "c":
                if idxs:
                    groups.append((ch, idxs))
                ch, idxs = it[1], []
            elif it[0] == "v":
                idxs.append(k)
        if idxs:
            groups.append((ch, idxs))
        for ch, idxs in groups:
            prev = 0
            for j, k in enumerate(idxs):
                it = b["items"][k]
                n_ = it[1]
                nxt = b["items"][idxs[j + 1]][1] if j + 1 < len(idxs) else None
                if n_ != prev + 1 and prev > 0 and (nxt is None or nxt == prev + 2):
                    rep.append(f"{b['code']} {ch}: printed verse number {n_} after {prev} (next {nxt}) taken as {prev + 1}")
                    b["items"][k] = ("v", prev + 1, it[2], it[3])
                    n_ = prev + 1
                prev = n_
    for b in books:
        # trailing paragraph-break markers are layout only
        b["items"] = [("p",) if it == ("b",) else it for it in b["items"]]
        while b["items"] and b["items"][-1] == ("p",):
            b["items"].pop()
        # collapse duplicate \p
        out_items = []
        for it in b["items"]:
            if it == ("p",) and out_items and out_items[-1] == ("p",):
                continue
            out_items.append(it)
        b["items"] = out_items
    if pending_notes:
        rep.append("unresolved note anchors: " + ",".join(pending_notes))
    order = {c: i for i, c in enumerate(U.BOOK_ORDER)}
    file_order = [b["code"] for b in books]
    books.sort(key=lambda b: order[b["code"]])
    U.write_usfm_zip(books, out, "sv1917pg")
    rep.append("verses continued in a new paragraph (paragraph break inside the verse, joined): " + " ".join(sorted(set(midpara))))
    rep.append("book order in the file: " + " ".join(file_order))
    open(report, "w", encoding="utf-8").write("\n".join(rep) + "\n")
    st = U.structure(books)
    print(len(books), "books", sum(sum(c.values()) for c in st.values()), "verses;", len(rep), "report lines")


if __name__ == "__main__":
    main(*sys.argv[1:4])
