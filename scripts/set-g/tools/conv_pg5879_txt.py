r"""Convert Project Gutenberg #5879 (Reina-Valera New Testament 1862, SPCC edition, plain
text from valera1909.com) to USFM.

The file's own structure:
  title block (upper-case lines beginning "EL SANTO EVANGELIO", "LOS HECHOS ...",
  "LA ... EPÍSTOLA ...", "EL APOCALIPSIS ...")  -> new book (\mt); single-chapter books
                                                    have no chapter heading (implicit \c 1)
  "CAPITULO n."                                 -> \c n
  a line beginning with a number (after a blank line, or continuing the count)
                                                -> a new verse (each verse is a paragraph)
  other lines                                   -> continue the current verse
  "FIN DEL NUEVO TESTAMENTO."                   -> end
Verse numbering: two passes. In a chapter whose last printed number equals the number of
verse paragraphs, verses are numbered 1..n in order and every printed number that differs
is reported (misprints in the file). Otherwise the printed numbers are used and the chapter
is reported for review. A first verse printed without a number is verse 1 (reported).
Brackets [..] (supplied words) are printed in the file and kept verbatim.
Usage: python3 -I conv_pg5879_txt.py 5879.txt OUT.zip REPORT
"""
import re
import sys

sys.path.insert(0, "/root/aw-set-g/tools")
import usfmlib as U  # noqa: E402

NT = U.BOOK_ORDER[39:66]
TITLE_START = re.compile(r"^(EL SANTO EVANGELIO|LOS HECHOS DE LOS APÓSTOLES\.|LA (PRIMERA |SEGUNDA |TERCERA )?EPÍSTOLA\^?( .*)?|EL APOCALIPSIS.*)$")
UPPER = re.compile(r"^[A-ZÁÉÍÓÚÑÜ][A-ZÁÉÍÓÚÑÜ .,\-^]*$")


def main(src, out, report):
    lines = open(src, encoding="utf-8").read().replace("\r\n", "\n").split("\n")
    start = next(i for i, l in enumerate(lines) if l.startswith("*** START OF"))
    end = next(i for i, l in enumerate(lines) if l.startswith("*** END OF"))
    rep = []
    books = []  # each: {code, title[], chapters: [[n, [entries]]]}, entry = [printed or None, text]
    cur = None
    in_title = False
    prev_blank = True
    for ln in range(start + 1, end):
        s = lines[ln].strip()
        if s == "FIN DEL NUEVO TESTAMENTO.":
            break
        if not s:
            prev_blank = True
            continue
        if TITLE_START.match(s) and prev_blank:
            cur = {"code": NT[len(books)], "title": [s], "chapters": []}
            books.append(cur)
            in_title = True
            prev_blank = False
            continue
        if cur is None:
            prev_blank = False
            continue
        cm = re.fullmatch(r"CAPITULO (\d+)\.", s)
        if cm:
            cur["chapters"].append([int(cm.group(1)), []])
            in_title = False
            prev_blank = True
            continue
        if in_title and UPPER.match(s):
            cur["title"].append(s)
            prev_blank = False
            continue
        if in_title:
            in_title = False
            if not cur["chapters"]:
                cur["chapters"].append([1, []])  # single-chapter book
        chap = cur["chapters"][-1][1]
        vm = re.match(r"^(\d+)(?:'(\d+))? (.*)$", s)
        last_pn = None
        for e in reversed(chap):
            if e[0] is not None:
                last_pn = e[0]
                break
        if vm:
            pn = int(vm.group(1) + (vm.group(2) or ""))
            printed = vm.group(1) + ("'" + vm.group(2) if vm.group(2) else "")
            if prev_blank or (last_pn is not None and pn == last_pn + 1) or not chap:
                chap.append([pn, vm.group(3).strip(), printed])
                prev_blank = False
                continue
        if not chap:
            chap.append([None, s, None])
            prev_blank = False
            continue
        if prev_blank:
            chap.append(["PARA", s, None])
            prev_blank = False
            continue
        chap[-1][1] = (chap[-1][1] + " " + s).strip()
        prev_blank = False
    for b in books:
        for ci, (cn, entries) in enumerate(b["chapters"]):
            new = []
            for k, e in enumerate(entries):
                if e[0] == "PARA":
                    is_last = (ci == len(b["chapters"]) - 1) and all(x[0] == "PARA" for x in entries[k:])
                    if is_last:
                        b.setdefault("colophon", []).append(e[1])
                        rep.append(f"{b['code']} {cn}: closing subscription (colophon, not a verse): {e[1][:70]}")
                        continue
                    rep.append(f"{b['code']} {cn}: unnumbered paragraph joined to the preceding verse: {e[1][:60]}")
                    new[-1][1] = (new[-1][1] + " " + e[1]).strip()
                    continue
                new.append(e)
            b["chapters"][ci][1] = new
            # continuation lines after a PARA belong to it (already merged into e[1])
    # a verse number typed with the letter l for the digit 1 inside the preceding verse
    # (e.g. "2l Y un ángel" in Rev 18:20): split there, report it
    for b in books:
        for cn, entries in b["chapters"]:
            k = 0
            while k < len(entries):
                e = entries[k]
                nxt = entries[k + 1][0] if k + 1 < len(entries) else None
                if isinstance(e[0], int):
                    m = re.search(r"\s(\d?)l\s(?=[A-ZÁÉÍÓÚÑ])", e[1])
                    if m and int(m.group(1) + "1") == e[0] + 1 and (nxt is None or nxt == e[0] + 2):
                        rep.append(f"{b['code']} {cn}:{e[0] + 1} printed as {m.group(1) + 'l'!r} inside verse {e[0]}; split there")
                        entries.insert(k + 1, [e[0] + 1, e[1][m.end():].strip(), m.group(1) + "l"])
                        e[1] = e[1][:m.start()].strip()
                k += 1
    out_books = []
    for b in books:
        book = {"code": b["code"], "mt": [" ".join(b["title"])], "items": [],
                "idtext": "Reina-Valera Nuevo Testamento 1862 - Project Gutenberg #5879"}
        prevc = 0
        for cn, entries in b["chapters"]:
            if cn != prevc + 1:
                rep.append(f"{b['code']}: chapter {cn} follows {prevc}")
            prevc = cn
            book["items"].append(("c", cn))
            n = len(entries)
            printed_nums = [e[0] for e in entries]
            last = next((p for p in reversed(printed_nums) if p is not None), None)
            seq_mismatch = sum(1 for i, e in enumerate(entries) if e[0] is not None and e[0] != i + 1)
            if last == n or seq_mismatch <= 1:
                for i, e in enumerate(entries):
                    v = i + 1
                    if e[0] is None:
                        rep.append(f"{b['code']} {cn}:{v} printed without a verse number")
                    elif e[0] != v:
                        rep.append(f"{b['code']} {cn}:{v} printed as {e[2]!r}")
                    book["items"].append(("p",))
                    book["items"].append(("v", v, e[1], []))
            else:
                rep.append(f"{b['code']} {cn}: REVIEW {n} verse paragraphs, last printed number {last}; printed sequence {printed_nums}")
                lastv = 0
                for e in entries:
                    v = e[0] if (e[0] is not None and e[0] > lastv) else lastv + 1
                    if e[0] != v:
                        rep.append(f"{b['code']} {cn}:{v} printed as {e[2]!r}")
                    book["items"].append(("p",))
                    book["items"].append(("v", v, e[1], []))
                    lastv = v
        for col in b.get("colophon", []):
            book["items"].append(("s", col))
        out_books.append(book)
    U.write_usfm_zip(out_books, out, "esRV1862")
    open(report, "w", encoding="utf-8").write("\n".join(rep) + "\n")
    st = U.structure(out_books)
    print(len(out_books), "books", sum(sum(c.values()) for c in st.values()), "verses;", len(rep), "report lines")


if __name__ == "__main__":
    main(*sys.argv[1:4])
