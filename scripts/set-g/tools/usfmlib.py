r"""Small shared helpers for the set-G reference builders and renderers.

Data model used by all converters ("book dict"):
  book = {"code": "GEN", "title": "...", "items": [...]}
  items are tuples in reading order:
    ("c", n)                 chapter start
    ("v", n, text, notes)    verse; text is plain verse text; notes is a list of
                             (kind, text) with kind "f" (footnote) or "x" (cross reference),
                             each attached at the end of the verse unless the text
                             contains the placeholder \u0001<k>\u0001 (k = index into notes)
    ("s", text)              section heading (\s1)
    ("d", text)              psalm title / descriptive heading (\d)
    ("cd", text)             chapter description / summary (\cd)
    ("p",)                   paragraph break (\p)
    ("q", level)             poetry line start (\q1, \q2)
    ("b",)                   blank line (\b)
    ("rem", text)            comment (\rem), never printed as verse text
"""
import io
import os
import re
import zipfile

BOOK_ORDER = [
    "GEN", "EXO", "LEV", "NUM", "DEU", "JOS", "JDG", "RUT", "1SA", "2SA", "1KI", "2KI",
    "1CH", "2CH", "EZR", "NEH", "EST", "JOB", "PSA", "PRO", "ECC", "SNG", "ISA", "JER",
    "LAM", "EZK", "DAN", "HOS", "JOL", "AMO", "OBA", "JON", "MIC", "NAM", "HAB", "ZEP",
    "HAG", "ZEC", "MAL", "MAT", "MRK", "LUK", "JHN", "ACT", "ROM", "1CO", "2CO", "GAL",
    "EPH", "PHP", "COL", "1TH", "2TH", "1TI", "2TI", "TIT", "PHM", "HEB", "JAS", "1PE",
    "2PE", "1JN", "2JN", "3JN", "JUD", "REV", "TOB", "JDT", "ESG", "WIS", "SIR", "BAR",
    "LJE", "S3Y", "SUS", "BEL", "1MA", "2MA", "3MA", "4MA", "1ES", "2ES", "MAN", "PS2",
    "DAG",
]

# Paratext-style file numbers
BOOK_NUM = {c: i + 1 for i, c in enumerate(BOOK_ORDER[:39])}
BOOK_NUM.update({c: 41 + i for i, c in enumerate(BOOK_ORDER[39:66])})
_EXTRA = {"TOB": 68, "JDT": 69, "ESG": 70, "WIS": 71, "SIR": 72, "BAR": 73, "LJE": 74,
          "S3Y": 75, "SUS": 76, "BEL": 77, "1MA": 78, "2MA": 79, "3MA": 80, "4MA": 81,
          "1ES": 82, "2ES": 83, "MAN": 84, "PS2": 85, "DAG": 87}
BOOK_NUM.update(_EXTRA)

# Zefania / common numbering 1..66 in Protestant order, 67.. deuterocanon (Zefania convention)
ZEF_NUM = {i + 1: c for i, c in enumerate(BOOK_ORDER[:66])}
ZEF_NUM.update({67: "JDT", 68: "WIS", 69: "TOB", 70: "SIR", 71: "BAR", 72: "1MA",
                73: "2MA", 74: "DAG", 75: "ESG", 76: "MAN"})


def clean_ws(s):
    return re.sub(r"\s+", " ", s).strip()


def esc(s):
    """USFM text must not contain backslashes."""
    return s.replace("\\", "/")


def book_to_usfm(book, header_lines=None):
    out = [f"\\id {book['code']} {esc(book.get('idtext', ''))}".rstrip()]
    out.append("\\usfm 3.0")
    if book.get("h"):
        out.append(f"\\h {esc(book['h'])}")
    if book.get("toc1"):
        out.append(f"\\toc1 {esc(book['toc1'])}")
    if book.get("toc2"):
        out.append(f"\\toc2 {esc(book['toc2'])}")
    for line in header_lines or []:
        out.append(line)
    if book.get("mt"):
        for i, t in enumerate(book["mt"]):
            out.append(f"\\mt{i + 1} {esc(t)}")
    for it in book["items"]:
        k = it[0]
        if k == "c":
            out.append(f"\\c {it[1]}")
        elif k == "v":
            n, text, notes = it[1], it[2], it[3]
            body = esc(text)
            used = set()

            def note_usfm(i):
                kind, nt = notes[i]
                if kind == "x":
                    return f"\\x - \\xt {esc(nt)}\\x*"
                return f"\\f + \\ft {esc(nt)}\\f*"

            def rep(m):
                i = int(m.group(1))
                used.add(i)
                return note_usfm(i)

            body = re.sub(r"\x01(\d+)\x01", rep, body)
            for i in range(len(notes)):
                if i not in used:
                    body += note_usfm(i)
            out.append(f"\\v {n} {body}".rstrip())
        elif k == "s":
            out.append(f"\\s1 {esc(it[1])}")
        elif k == "d":
            out.append(f"\\d {esc(it[1])}")
        elif k == "cd":
            out.append(f"\\cd {esc(it[1])}")
        elif k == "qa":
            out.append(f"\\qa {esc(it[1])}")
        elif k == "p":
            out.append("\\p")
        elif k == "q":
            out.append(f"\\q{it[1]}")
        elif k == "b":
            out.append("\\b")
        elif k == "rem":
            out.append(f"\\rem {esc(it[1])}")
        else:
            raise ValueError(it)
    return "\n".join(out) + "\n"


def write_usfm_zip(books, zip_path, suffix, extra_files=None):
    """Write one USFM file per book into a deterministic ZIP."""
    os.makedirs(os.path.dirname(zip_path), exist_ok=True)
    with zipfile.ZipFile(zip_path, "w", zipfile.ZIP_DEFLATED) as z:
        for b in books:
            name = f"{BOOK_NUM[b['code']]:02d}-{b['code']}{suffix}.usfm"
            zi = zipfile.ZipInfo(name, date_time=(2026, 11, 6, 0, 0, 0))
            zi.compress_type = zipfile.ZIP_DEFLATED
            z.writestr(zi, book_to_usfm(b).encode("utf-8"))
        for name, data in (extra_files or {}).items():
            zi = zipfile.ZipInfo(name, date_time=(2026, 11, 6, 0, 0, 0))
            zi.compress_type = zipfile.ZIP_DEFLATED
            z.writestr(zi, data.encode("utf-8") if isinstance(data, str) else data)


# ---------- minimal USFM reader (for renderers and checks) ----------

NOTE_RE = re.compile(r"\\(f|x|fe) (.*?)\\\1\*", re.S)


def strip_char_markup(body):
    """Remove USFM character markers and word attributes, keep the words."""
    body = re.sub(r"\|[^\\|]*?(?=\\\+?[a-z]+\d*\*)", "", body)   # \\w word|strong="..."\\w*
    body = re.sub(r"\\\+?[a-z]+\d*\*", "", body)                  # closing markers
    body = re.sub(r"\\\+?[a-z]+\d*(\s|$)", "", body)              # opening markers
    return body


def note_text(raw):
    """Plain text of a note body (drop caller and markers)."""
    raw = re.sub(r"^\S+\s*", "", raw.strip())  # caller (+, -, letter)
    raw = re.sub(r"\\fr [^\\]*", "", raw)  # the note's own reference (\\fr) is not note text
    raw = re.sub(r"\\[a-z]+\d*\*?\s?", "", raw)
    return clean_ws(raw)


def parse_usfm(text):
    """Return dict: code, headers{}, items list (same model as above; verse notes
    are kept as (kind, text) with placeholders \x01i\x01 at their positions)."""
    book = {"code": None, "headers": {}, "items": []}
    lines = text.replace("\r\n", "\n").split("\n")
    # join continuation lines into marker-led lines
    toks = re.split(r"(?=\\(?:id|usfm|ide|h|toc\d|mt\d?|ms\d?|mr|c|v|s\d?|r|d|cd|p|m|pi\d?|q\d?|qc|b|rem|nb|li\d?|is\d?|ip|imt\d?|iot|io\d?|ie|sp|cl|cp|pmo|mi|qr|qa|sr|nb|pc|qm\d?|cls|tr|tc\d|th\d|periph|lit|d|ib|im|imq)\b)",
                    "\n".join(lines))
    for tok in toks:
        tok = tok.strip()
        if not tok:
            continue
        m = re.match(r"\\([a-z]+\d*)\s*(.*)", tok, re.S)
        if not m:
            continue
        mk, rest = m.group(1), m.group(2)
        if mk == "id":
            book["code"] = rest.split()[0]
        elif mk in ("h", "toc1", "toc2", "toc3", "mt", "mt1", "mt2", "mt3", "ide", "usfm"):
            book["headers"].setdefault(mk, clean_ws(rest))
        elif mk == "c":
            book["items"].append(("c", int(rest.split()[0])))
        elif mk == "v":
            mm = re.match(r"(\S+)\s*(.*)", rest, re.S)
            num, body = mm.group(1), mm.group(2)
            notes = []

            def rep(m2):
                notes.append(("x" if m2.group(1) == "x" else "f", note_text(m2.group(2))))
                return f"\x01{len(notes) - 1}\x01"

            body = NOTE_RE.sub(rep, body)
            body = strip_char_markup(body)
            book["items"].append(("v", num, clean_ws(body), notes))
        elif mk in ("s", "s1", "s2", "s3", "ms", "ms1", "ms2", "mr", "sr", "r"):
            body = NOTE_RE.sub("", rest)
            body = re.sub(r"\\\+?[a-z]+\d*\*?\s?", "", body)
            book["items"].append(("s" if mk != "r" else "r", clean_ws(body)))
        elif mk == "d":
            book["items"].append(("d", clean_ws(re.sub(r"\\\+?[a-z]+\d*\*?\s?", "", NOTE_RE.sub("", rest)))))
        elif mk == "cd":
            book["items"].append(("cd", clean_ws(rest)))
        elif mk == "qa":
            book["items"].append(("qa", clean_ws(rest)))
        elif mk in ("p", "m", "pi", "pi1", "pi2", "nb", "pmo", "mi", "li", "li1", "li2", "pc", "cls"):
            book["items"].append(("p",))
            if rest.strip():
                # text after a paragraph marker belongs to the preceding verse
                for i in range(len(book["items"]) - 1, -1, -1):
                    if book["items"][i][0] == "v":
                        v = book["items"][i]
                        book["items"][i] = ("v", v[1], clean_ws(v[2] + " " + rest), v[3])
                        break
        elif mk.startswith("q"):
            lvl = int(mk[1:]) if mk[1:].isdigit() else 1
            book["items"].append(("q", lvl))
            if rest.strip():
                for i in range(len(book["items"]) - 1, -1, -1):
                    if book["items"][i][0] == "v":
                        v = book["items"][i]
                        body = re.sub(r"\\\+?[a-z]+\d*\*?\s?", "", NOTE_RE.sub("", rest))
                        book["items"][i] = ("v", v[1], clean_ws(v[2] + " " + body), v[3])
                        break
        elif mk == "b":
            book["items"].append(("b",))
        elif mk == "rem":
            book["items"].append(("rem", clean_ws(rest)))
        else:
            book["items"].append(("other", mk, clean_ws(rest)))
    return book


def read_usfm_zip(path):
    books = []
    with zipfile.ZipFile(path) as z:
        for n in sorted(z.namelist()):
            if n.lower().endswith((".usfm", ".sfm")):
                books.append(parse_usfm(z.read(n).decode("utf-8-sig")))
    order = {c: i for i, c in enumerate(BOOK_ORDER)}
    books.sort(key=lambda b: order.get(b["code"], 999))
    return books


def verse_map(books):
    """{(book, chapter, verse): text} from parsed books (notes removed)."""
    out = {}
    for b in books:
        ch = 0
        for it in b["items"]:
            if it[0] == "c":
                ch = it[1]
            elif it[0] == "v":
                out[(b["code"], ch, it[1])] = clean_ws(re.sub(r"\x01\d+\x01", "", it[2]))
    return out


def structure(books):
    """{book: {chapter: verse_count}}"""
    st = {}
    for b in books:
        ch = 0
        for it in b["items"]:
            if it[0] == "c":
                ch = it[1]
                st.setdefault(b["code"], {})[ch] = 0
            elif it[0] == "v":
                st.setdefault(b["code"], {}).setdefault(ch, 0)
                st[b["code"]][ch] += 1
    return st
