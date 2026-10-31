"""A small USFM reader for the disclosed-layout reconstructions (v0.24.5).

Reads one eBible USFM zip into books of blocks a layout writer can set:

    ("title", text)                 \\mt1 (or \\h when there is none)
    ("chapter", n)                  \\c
    ("heading", marker, text)       \\s1 \\s2 \\ms \\r \\d \\sp \\mr
    ("para", marker, [pieces])      \\p \\m \\q1 \\q2 \\pi \\li1 ... with pieces
                                    ("v", "12") | ("t", text) | ("note", text)

Character styles are kept as plain text (\\add \\nd \\sc ...); \\f and \\x are
returned as ("note", text) pieces (cross references flagged with "x:").
Nothing here knows the recogniser: it only reads the publisher's USFM.
"""

import re
import zipfile

PARA = {"p", "m", "pi", "pi1", "pi2", "q", "q1", "q2", "q3", "li", "li1", "li2",
        "nb", "pc", "pm", "pmo", "mi", "qr", "qc", "b", "qm1", "qm2", "pmr"}
HEAD = {"s", "s1", "s2", "s3", "ms", "ms1", "ms2", "mr", "r", "d", "sp", "qa", "cl"}
SKIP = {"ide", "rem", "sts", "toc2", "toc3", "imt", "imt1", "imt2", "is", "is1",
        "ip", "ipr", "io", "io1", "io2", "iot", "ie", "mt2", "mt3", "ili", "ili1",
        "iex", "imte", "cp", "ca", "va", "periph", "usfm", "toca1", "toca2", "toca3"}

CHAR = re.compile(r"\\\+?(?:add|nd|sc|it|bd|bdit|em|wj|tl|qs|k|w|pn|ord|sig|sls|qt|bk|dc|no|png|addpn|fv|ior|rq|lit|ndx|wg|wh|wa|rb|jmp|sup)\*?")


def strip_chars(text):
    text = re.sub(r"\|[^\\]*?(?=\\\+?w\*)", "", text)  # \w word|lemma\w*
    text = CHAR.sub("", text)
    return re.sub(r"[ \t]+", " ", text)


def note_text(body):
    body = re.sub(r"^\s*[+\-\w]\s*", "", body, count=1)
    body = re.sub(r"\\\+?f(?:r|q|qa|k|t|l|w|p|v|dc|m)\*?|\\\+?x(?:o|k|q|t|ta|op|ot|nt|dc)\*?", " ", body)
    return re.sub(r"\s+", " ", strip_chars(body)).strip()


def split_notes(text):
    """Text with \\f..\\f* and \\x..\\x* as separate pieces."""
    out = []
    pos = 0
    for m in re.finditer(r"\\(f|fe|x) (.*?)\\\1\*", text, re.S):
        if m.start() > pos:
            out.append(("t", strip_chars(text[pos:m.start()])))
        kind = "x:" if m.group(1) == "x" else ""
        out.append(("note", kind + note_text(m.group(2))))
        pos = m.end()
    if pos < len(text):
        out.append(("t", strip_chars(text[pos:])))
    return [p for p in out if p[0] != "t" or p[1] != ""]


def read_book(usfm):
    book = {"code": None, "h": None, "toc1": None, "mt": [], "blocks": []}
    blocks = book["blocks"]
    para = None
    for raw in re.split(r"(?=\\(?:id|h|toc\d|mt\d?|c|v|p|m|pi\d?|q\d?|li\d?|nb|pc|pm|pmo|mi|qr|qc|b|s\d?|ms\d?|mr|r|d|sp|qa|cl|ide|rem|sts|imt\d?|is\d?|ip|ipr|io\d?|iot|ie|mt\d|ili\d?|iex|imte|cp|ca|va|periph|usfm|qm\d)[\s\\])", usfm):
        raw = raw.strip("\r\n")
        m = re.match(r"\\(\w+)\s?(.*)", raw, re.S)
        if not m:
            continue
        marker, rest = m.group(1), m.group(2).replace("\n", " ")
        if marker == "id":
            book["code"] = rest.split()[0]
        elif marker == "h":
            book["h"] = strip_chars(rest).strip()
        elif marker == "toc1":
            book["toc1"] = strip_chars(rest).strip()
        elif re.fullmatch(r"mt\d?", marker):
            if marker in ("mt", "mt1"):
                book["mt"].append(strip_chars(rest).strip())
        elif marker == "c":
            blocks.append(("chapter", int(rest.split()[0])))
            para = None
        elif marker == "v":
            num, _, text = rest.partition(" ")
            if para is None:
                para = ("para", "p", [])
                blocks.append(para)
            para[2].append(("v", num))
            para[2].extend(split_notes(text))
        elif marker in PARA:
            para = ("para", marker, [])
            blocks.append(para)
            if rest.strip():
                para[2].extend(split_notes(rest))
        elif marker in HEAD:
            text = re.sub(r"\s+", " ", "".join(p[1] for p in split_notes(rest) if p[0] == "t")).strip()
            if text:
                blocks.append(("heading", marker, text))
            para = None
        elif marker in SKIP:
            para = None if marker not in ("cp", "ca", "va") else para
    # Titles: \mt1 lines, or \h.
    title = " ".join(t for t in book["mt"] if t) or book["h"] or book["code"]
    book["title"] = title
    return book


def read_zip(path, only=None):
    books = []
    with zipfile.ZipFile(path) as z:
        names = sorted((n for n in z.namelist() if n.lower().endswith((".usfm", ".sfm"))),
                       key=lambda n: int(re.match(r"(\d+)", n.split("/")[-1]).group(1)) if re.match(r"\d+", n.split("/")[-1]) else 9999)
        for name in names:
            text = z.read(name).decode("utf-8-sig")
            b = read_book(text)
            if b["code"] in ("FRT", "INT", "GLO", "BAK", "OTH", "XXA", "XXB", "XXC", "CNC", "TDX", "NDX"):
                continue
            if only and b["code"] not in only:
                continue
            if not any(bl[0] == "chapter" for bl in b["blocks"]):
                continue
            books.append(b)
    return books


def para_text(pieces, notes=False):
    out = []
    for kind, val in pieces:
        if kind == "t":
            out.append(val)
        elif kind == "v":
            out.append(f" {val} ")
        elif kind == "note" and notes:
            out.append(f" [{val}] ")
    return re.sub(r"\s+", " ", "".join(out)).strip()
