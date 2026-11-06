r"""Convert the Nestle 1904 Greek New Testament XML (biblicalhumanities/Nestle1904 xml/,
markup CC BY-SA 4.0 by Jonathan Robie; base text by Diego Renato dos Santos, public domain)
to USFM.

  <div type="book"><title type="main">   -> \mt1
  <chapter> <title>n</title>              -> \c n
  <p>                                     -> \p
  <milestone unit="verse" id="B.c.v"/>    -> \v v
  <w>word</w>                             -> word (space separated)
  <pc>.</pc>                              -> punctuation attached to the preceding word
  <milestone unit="tc" display="X"/>      -> text-critical sign X printed in the text
                                             (opening signs attach to the next word,
                                             closing signs to the previous word)
  <q>/<lg>/<l>                            -> Old Testament quotation / poetry: \q1 lines
                                             (layout; the words stay in the verse)
Usage: python3 -I conv_nestle1904_xml.py XMLDIR OUT.zip REPORT [BOOKS]
"""
import glob
import os
import re
import sys
import html

sys.path.insert(0, "/root/aw-set-g/tools")
import usfmlib as U  # noqa: E402

OSIS = {"Matt": "MAT", "Mark": "MRK", "Luke": "LUK", "John": "JHN", "Acts": "ACT", "Rom": "ROM",
        "1Cor": "1CO", "2Cor": "2CO", "Gal": "GAL", "Eph": "EPH", "Phil": "PHP", "Col": "COL",
        "1Thess": "1TH", "2Thess": "2TH", "1Tim": "1TI", "2Tim": "2TI", "Titus": "TIT",
        "Phlm": "PHM", "Heb": "HEB", "Jas": "JAS", "1Pet": "1PE", "2Pet": "2PE", "1John": "1JN",
        "2John": "2JN", "3John": "3JN", "Jude": "JUD", "Rev": "REV"}
OPEN = {"<", "[", "[["}
CLOSE = {">", "]", "]]"}


def convert(xmldir, books_filter=None, rep=None):
    rep = rep if rep is not None else []
    books = []
    for f in sorted(glob.glob(os.path.join(xmldir, "[0-9][0-9]-*.xml"))):
        x = open(f, encoding="utf-8").read()
        osis = re.search(r'osisID="([^"]+)"', x).group(1)
        code = OSIS[osis]
        if books_filter and code not in books_filter:
            continue
        title = html.unescape(re.search(r'<title type="main">(.*?)</title>', x).group(1))
        book = {"code": code, "mt": [title], "items": [], "poetry": {},
                "idtext": "Nestle 1904 Greek New Testament (biblicalhumanities.org XML)"}
        toks = re.findall(r"<[^>]+>|[^<]+", x)
        verse = None  # [num, words]
        attach_next = False
        in_l = False
        cur_ch = 0

        def flush():
            nonlocal verse
            if verse:
                book["items"].append(("v", verse[0], U.clean_ws(verse[1]), []))
                verse = None

        i = 0
        expect_title = False
        while i < len(toks):
            t = toks[i]
            if t.startswith("<chapter"):
                flush()
                cur_ch = int(re.search(r'osisId="[^.]+\.(\d+)"', t).group(1))
                book["items"].append(("c", cur_ch))
                expect_title = True
            elif t.startswith("<title") and expect_title:
                # chapter number title: skip its text
                i += 2
                expect_title = False
            elif t.startswith("<p") and not t.startswith("<pc"):
                nxt = next((u for u in toks[i + 1:] if u.strip()), "")
                if verse is None or nxt.startswith('<milestone unit="verse"'):
                    flush()
                    book["items"].append(("p",))
                else:
                    # paragraph break inside a verse: layout only
                    book["poetry"].setdefault((cur_ch, verse[0]), []).append(("p", len(verse[1])))
            elif t.startswith("<l") and not t.startswith("<lg"):
                # a poetry line inside a verse: remember the position for the renderer
                if verse:
                    book["poetry"].setdefault((cur_ch, verse[0]), []).append(len(verse[1]))
                in_l = True
            elif t.startswith('<milestone unit="verse"'):
                flush()
                v = int(re.search(r'id="[^.]+\.\d+\.(\d+)"', t).group(1))
                verse = [v, ""]
            elif t.startswith('<milestone unit="tc"'):
                d = html.unescape(re.search(r'display="([^"]*)"', t).group(1))
                if verse is None:
                    rep.append(f"{code} {cur_ch}: tc sign outside verse")
                elif d in OPEN:
                    verse[1] += " " + d
                    attach_next = True
                elif d in CLOSE:
                    verse[1] += d
                else:
                    rep.append(f"{code} {cur_ch}: unknown tc display {d!r}")
            elif t.startswith("<w"):
                word = html.unescape(toks[i + 1])
                i += 2
                if verse is None:
                    rep.append(f"{code} {cur_ch}: word outside verse {word}")
                else:
                    verse[1] += ("" if attach_next else " ") + word
                attach_next = False
            elif t.startswith("<pc"):
                p = html.unescape(toks[i + 1])
                i += 2
                if verse is not None:
                    verse[1] += p
                else:
                    # punctuation after a verse milestone boundary belongs to the previous verse
                    for k in range(len(book["items"]) - 1, -1, -1):
                        if book["items"][k][0] == "v":
                            it = book["items"][k]
                            book["items"][k] = ("v", it[1], it[2] + p, it[3])
                            break
            i += 1
        flush()
        books.append(book)
    return books


def main(xmldir, out, report, books_arg=None):
    rep = []
    bf = books_arg.split(",") if books_arg else None
    books = convert(xmldir, bf, rep)
    U.write_usfm_zip(books, out, "grcN1904")
    open(report, "w", encoding="utf-8").write("\n".join(rep) + "\n")
    st = U.structure(books)
    print(len(books), "books", sum(sum(c.values()) for c in st.values()), "verses;", len(rep), "report lines")


if __name__ == "__main__":
    main(*sys.argv[1:])
