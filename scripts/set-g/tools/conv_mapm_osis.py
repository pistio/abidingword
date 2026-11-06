r"""Convert Miqra according to the Masorah (MAM, Seth (Avi) Kadish; OSIS by David Troidl,
openscriptures/morphhb MAPM/MAPM.xml, CC BY-SA 3.0) to USFM for selected books.

OSIS -> USFM:
  <chapter sID n="א">            -> \c (arabic number from osisID; Hebrew numeral kept for print)
  <verse sID .. eID>             -> \v (text between start and end milestones)
  <p>                            -> \p ; <lg>/<l level=n> -> \q1/\q2/\q3 ; <lb/> -> line break
  <table>/<row>/<cell>           -> text of the cells in order (printed as a table)
  <seg type="x-ketiv">K</seg> Q  -> "(K) Q" : the written form in parentheses before the
                                    read form, the MAM display convention
  <seg type="x-big|x-small|x-suspended|x-paseq|x-reversednun">  -> text kept
  <seg type="x-symbol">          -> editorial symbol, not printed, not in the reference
  <seg type="x-space">           -> space
  <title>                        -> \mt1 (Hebrew book name)
Usage: python3 -I conv_mapm_osis.py MAPM.xml OUT.zip REPORT BOOKS(osis ids, comma)
"""
import re
import sys

sys.path.insert(0, "/root/aw-set-g/tools")
import usfmlib as U  # noqa: E402

OSIS2CODE = {"Ruth": "RUT", "Jonah": "JON", "Eccl": "ECC", "Lam": "LAM", "Esth": "EST", "Song": "SNG"}


def main(src, out, report, books_arg):
    x = open(src, encoding="utf-8-sig").read()
    rep, books = [], []
    for osis in books_arg.split(","):
        m = re.search(r'<div type="book" osisID="' + osis + r'">(.*?)\n\t\t</div>', x, re.S)
        body = m.group(1)
        code = OSIS2CODE[osis]
        title = re.search(r"<title>(.*?)</title>", body).group(1)
        book = {"code": code, "mt": [title], "items": [], "heb": {},
                "idtext": "Miqra according to the Masorah (MAM), OSIS MAPM 2014-02-19"}
        toks = re.split(r"(<[^>]+>)", body)
        in_verse = None
        buf = []
        pending_ketiv = False
        seg_type = None
        for t in toks:
            if not t:
                continue
            if t.startswith("<"):
                tag = re.match(r"</?([a-zA-Z]+)", t).group(1)
                closing = t.startswith("</")
                if tag == "chapter" and "sID" in t:
                    cn = int(re.search(r'osisID="[^.]+\.(\d+)"', t).group(1))
                    book["items"].append(("c", cn))
                    book["heb"][("c", cn)] = re.search(r'n="([^"]*)"', t).group(1)
                elif tag == "verse" and "sID" in t:
                    vm = re.search(r'osisID="[^.]+\.(\d+)\.(\d+)"', t)
                    in_verse = int(vm.group(2))
                    book["heb"][("v", int(vm.group(1)), in_verse)] = re.search(r'n="([^"]*)"', t).group(1)
                    buf = []
                elif tag == "verse" and "eID" in t:
                    txt = U.clean_ws("".join(buf))
                    book["items"].append(("v", in_verse, txt, []))
                    in_verse = None
                elif (tag == "chapter" and "eID" in t) or (tag == "p" and closing):
                    pass
                elif tag == "p" and not closing:
                    book["items"].append(("p",))
                elif tag == "l" and not closing:
                    lvl = int(re.search(r'level="(\d)"', t).group(1))
                    if in_verse is not None:
                        buf.append(f" \x02q{lvl}\x02 ")
                    else:
                        book["items"].append(("q", lvl))
                elif tag == "lb":
                    if in_verse is not None:
                        buf.append(" \x02lb\x02 ")
                elif tag == "seg":
                    if closing:
                        if seg_type == "x-ketiv":
                            buf.append(")")
                        seg_type = None
                    else:
                        seg_type = re.search(r'type="([^"]*)"', t).group(1)
                        if seg_type == "x-ketiv":
                            buf.append("(")
                elif tag in ("table", "row", "cell", "lg", "title", "div", "note"):
                    if tag == "cell" and not closing and in_verse is not None:
                        buf.append(" ")
                    if tag == "note" and not closing:
                        rep.append(f"{code}: OSIS note present")
                else:
                    rep.append(f"{code}: unhandled tag {t[:40]}")
                continue
            # text
            if seg_type == "x-symbol":
                rep.append(f"{code} {in_verse}: editorial symbol {t!r} not printed")
                continue
            if seg_type == "x-space":
                buf.append(" ")
                continue
            if in_verse is not None:
                buf.append(t)
            elif t.strip() and not re.match(r"^[\s]*$", t):
                if not re.search(r"<title>", t):
                    pass
        # move line-structure markers out of verse text: keep plain text for the reference,
        # keep the line breaks for the renderer in a side table
        items = []
        lines = {}
        for it in book["items"]:
            if it[0] == "v":
                parts = re.split(r"\s*\x02(q\d|lb)\x02\s*", it[2])
                text_segments, marks = [parts[0]], []
                for k in range(1, len(parts), 2):
                    marks.append(parts[k])
                    text_segments.append(parts[k + 1])
                plain = U.clean_ws(" ".join(s for s in text_segments if s))
                plain = plain.replace("( ", "(").replace(" )", ")")
                items.append(("v", it[1], plain, []))
                lines[len(items) - 1] = list(zip([None] + marks, text_segments))
            elif it[0] == "c" and title and not any(i[0] == "c" for i in items):
                items.append(it)
            else:
                items.append(it)
        book["items"] = items
        book["lines"] = lines
        books.append(book)
    order = {c: i for i, c in enumerate(U.BOOK_ORDER)}
    books.sort(key=lambda b: order[b["code"]])
    U.write_usfm_zip(books, out, "heMAM")
    open(report, "w", encoding="utf-8").write("\n".join(rep) + "\n")
    st = U.structure(books)
    print(len(books), "books", sum(sum(c.values()) for c in st.values()), "verses;", len(rep), "report lines")
    return books


if __name__ == "__main__":
    main(*sys.argv[1:5])
