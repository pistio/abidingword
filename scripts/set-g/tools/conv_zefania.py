r"""Convert a Zefania XML Bible to USFM (one file per book, zipped).

Mapping:
  <BIBLEBOOK bnumber=N>   -> book (Zefania numbering 1-66 Protestant order, 67.. deuterocanon)
  <CHAPTER cnumber=N>     -> \c
  <CAPTION vref=N>        -> \s1 before verse N
  <VERS vnumber=N>        -> \v N (a number breaking the sequence when the next verse
                             continues it is a typing error: corrected and reported)
  <NOTE ...>              -> \f (study note); notes whose text is only references -> \x
  <STYLE ...>, <gr>       -> text kept, markup dropped
  <BR/>                   -> space
Usage: python3 -I conv_zefania.py SRC.xml OUT.zip SUFFIX [--books CODE,CODE] [--idtext TEXT] [--report R]
"""
import re
import sys
import xml.etree.ElementTree as ET

sys.path.insert(0, "/root/aw-set-g/tools")
import usfmlib as U  # noqa: E402

REFPAT = re.compile(r"^[\s\dA-Za-zÄÖÜäöüß\.,;:\-–]+$")


def gather(el, notes, rep, where):
    parts = [el.text or ""]
    for ch in el:
        tag = ch.tag
        if tag == "NOTE":
            t = U.clean_ws("".join(ch.itertext()))
            kind = "x" if (REFPAT.match(t) and re.search(r"\d", t) and len(t) < 200 and not re.search(r"[a-zäöü]{5,}\s+[a-zäöü]{4,}", t)) else "f"
            notes.append((kind, t))
            parts.append(f" \x01{len(notes) - 1}\x01 ")
        elif tag == "BR":
            parts.append(" ")
        elif tag in ("STYLE", "gr", "DIV", "sup", "SUP", "XREF"):
            parts.append(gather(ch, notes, rep, where))
        else:
            rep.append(f"{where}: unknown tag {tag}")
            parts.append(gather(ch, notes, rep, where))
        parts.append(ch.tail or "")
    return "".join(parts)


def convert(src, books_filter=None, idtext="", rep=None):
    rep = rep if rep is not None else []
    tree = ET.parse(src)
    root = tree.getroot()
    books = []
    for bb in root.iter("BIBLEBOOK"):
        n = int(bb.get("bnumber"))
        code = U.ZEF_NUM.get(n)
        if code is None:
            rep.append(f"unknown book number {n} {bb.get('bname')}")
            continue
        if books_filter and code not in books_filter:
            continue
        book = {"code": code, "items": [], "idtext": idtext, "zefname": bb.get("bname")}
        for chap in bb.findall("CHAPTER"):
            c = int(chap.get("cnumber"))
            book["items"].append(("c", c))
            book["items"].append(("p",))
            # verse numbers that break the running sequence while the next verse continues it
            # (typing errors in the source, e.g. 30, 3, 32) are corrected and reported
            vlist = [el for el in chap if el.tag == "VERS"]
            fixed = {}
            prev = 0
            for k, el in enumerate(vlist):
                vn = el.get("vnumber")
                n_ = int(vn) if vn.isdigit() else None
                nxt = vlist[k + 1].get("vnumber") if k + 1 < len(vlist) else None
                if n_ is not None and n_ != prev + 1 and prev > 0 and (nxt is None or (nxt.isdigit() and int(nxt) == prev + 2)):
                    fixed[id(el)] = prev + 1
                    rep.append(f"{code} {c}: source verse number {vn} after {prev} (next {nxt}) taken as {prev + 1}")
                    n_ = prev + 1
                prev = n_ if n_ is not None else prev
            for el in chap:
                if el.tag == "CAPTION":
                    book["items"].append(("s", U.clean_ws("".join(el.itertext()))))
                elif el.tag == "VERS":
                    notes = []
                    raw = gather(el, notes, rep, f"{code} {c}:{el.get('vnumber')}")
                    txt = U.clean_ws(raw)
                    txt = re.sub(r"\s*\x01(\d+)\x01\s*", lambda m: f"\x01{m.group(1)}\x01 ", txt).strip()
                    txt = re.sub(r"\x01(\d+)\x01 ([,.;:!?)])", lambda m: "\x01" + m.group(1) + "\x01" + m.group(2), txt)
                    v = el.get("vnumber")
                    v = fixed.get(id(el), int(v) if v.isdigit() else v)
                    book["items"].append(("v", v, txt, notes))
                elif el.tag in ("PROLOG", "REMARK"):
                    rep.append(f"{code} {c}: {el.tag} skipped: {''.join(el.itertext())[:80]}")
                else:
                    rep.append(f"{code} {c}: unexpected element {el.tag}")
        books.append(book)
    order = {c: i for i, c in enumerate(U.BOOK_ORDER)}
    books.sort(key=lambda b: order[b["code"]])
    return books


def main():
    src, out, suffix = sys.argv[1:4]
    args = sys.argv[4:]
    bf, idtext, report = None, "", None
    i = 0
    while i < len(args):
        if args[i] == "--books":
            bf = args[i + 1].split(",")
        elif args[i] == "--idtext":
            idtext = args[i + 1]
        elif args[i] == "--report":
            report = args[i + 1]
        i += 2
    rep = []
    books = convert(src, bf, idtext, rep)
    if bf:
        order = {c: i for i, c in enumerate(bf)}
        missing = [c for c in bf if c not in {b["code"] for b in books}]
        if missing:
            rep.append("requested books missing: " + ",".join(missing))
    U.write_usfm_zip(books, out, suffix)
    if report:
        open(report, "w", encoding="utf-8").write("\n".join(rep) + "\n")
    st = U.structure(books)
    print(len(books), "books", sum(sum(c.values()) for c in st.values()), "verses;", len(rep), "report lines")


if __name__ == "__main__":
    main()
