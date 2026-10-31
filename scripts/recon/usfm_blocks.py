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

read_zip(..., rich=True) (used by build_recon_bc.py; the default is
unchanged) additionally:

  * keeps character styles as their own pieces: ("add", text), ("nd", text),
    ("sc", text), ("it", text), ("bd", text), ("em", text), ("qs", text) ...
    next to plain ("t", text) pieces (see plain() / STYLED);
  * keeps the \\fqa / \\fq quotations of notes between NOTE_IT_ON and
    NOTE_IT_OFF (see note_parts(), note_plain(), note_ref());
  * records every title line in book["mts"] as (marker, text) in source order;
  * keeps the introduction / front matter: ("intro", marker, pieces) blocks
    for \\ip \\im \\is1 \\iot \\ib ..., and FRT / INT books when they are named
    in `only`;
  * splits \\lh \\lf \\lim \\im \\ib ... into paragraphs and drops \\vp ...\\vp*.
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


SPLIT = r"(?=\\(?:id|h|toc\d|mt\d?|c|v|p|m|pi\d?|q\d?|li\d?|nb|pc|pm|pmo|mi|qr|qc|b|s\d?|ms\d?|mr|r|d|sp|qa|cl|ide|rem|sts|imt\d?|is\d?|ip|ipr|io\d?|iot|ie|mt\d|ili\d?|iex|imte|cp|ca|va|periph|usfm|qm\d)[\s\\])"
RICH_SPLIT = SPLIT[:-len(")[\\s\\\\])")] + "|im|imi|imq|ipi|ipq|ib|lh|lf|lim\\d?|sr|tr|toca\\d)[\\s\\\\])"
RICH_PARA = PARA | {"lh", "lf", "lim", "lim1", "lim2", "li3", "q4", "pi3", "ph1", "tr"}
INTRO = {"ip", "im", "imi", "imq", "ipi", "ipq", "ipr", "is", "is1", "is2", "iot", "io", "io1", "io2",
         "ib", "iex", "ili", "ili1", "imt", "imt1", "imt2", "ie"}

# Character styles kept by rich=True, in order of precedence when nested.
STYLED = ("nd", "sc", "add", "it", "em", "bdit", "bd", "tl", "bk", "qs", "sup", "rq", "sls", "wh", "no")
NOTE_IT_ON, NOTE_IT_OFF = "\x0e", "\x0f"


def styled_runs(text):
    """Text with character styles as (style, text) runs ("t" for plain)."""
    text = re.sub(r"\\vp .*?\\vp\*", "", text)
    text = re.sub(r"\|[^\\]*?(?=\\\+?\w+\*)", "", text)  # \w word|lemma\w*
    stack, out = [], []
    for tok in re.split(r"(\\\+?[a-z]+\d?(?:\*|[ \t]?))", text):
        if not tok:
            continue
        if tok.startswith("\\"):
            tok = tok.rstrip()
            name = tok.lstrip("\\+").rstrip("*")
            if tok.endswith("*"):
                if name in stack:
                    del stack[len(stack) - 1 - stack[::-1].index(name)]
            else:
                stack.append(name)
            continue
        kind = next((k for k in STYLED if k in stack), "t")
        tok = re.sub(r"[ \t]+", " ", tok)
        if out and out[-1][0] == kind:
            out[-1] = (kind, re.sub(r"[ \t]+", " ", out[-1][1] + tok))
        else:
            out.append((kind, tok))
    return out


def rich_note_text(body):
    body = re.sub(r"^\s*[+\-\w]\s*", "", body, count=1)
    body = re.sub(r"\\\+?f(?:qa|q)\s", NOTE_IT_ON, body)
    body = re.sub(r"\\\+?f(?:qa|q)\*", NOTE_IT_OFF, body)
    # Close a quotation at the next note marker.
    parts = re.split(r"(\\\+?(?:f(?:r|k|t|l|w|p|v|dc|m)|x(?:o|k|q|t|ta|op|ot|nt|dc))\*?)", body)
    out, open_it = [], False
    for part in parts:
        if re.fullmatch(r"\\\+?\w+\*?", part):
            if open_it:
                out.append(NOTE_IT_OFF)
                open_it = False
            out.append(" ")
            continue
        for ch in re.findall(r"[\x0e\x0f]", part):
            open_it = ch == NOTE_IT_ON
        out.append(part)
    if open_it:
        out.append(NOTE_IT_OFF)
    text = "".join(out)
    text = "".join(t for _, t in styled_runs(text))
    text = re.sub(r"\s+", " ", text).strip()
    text = re.sub(NOTE_IT_ON + r"\s*" + NOTE_IT_OFF, "", text)
    return text


def rich_pieces(text):
    out = []
    pos = 0
    for m in re.finditer(r"\\(f|fe|x) (.*?)\\\1\*", text, re.S):
        if m.start() > pos:
            out.extend(styled_runs(text[pos:m.start()]))
        kind = "x:" if m.group(1) == "x" else ""
        out.append(("note", kind + rich_note_text(m.group(2))))
        pos = m.end()
    if pos < len(text):
        out.extend(styled_runs(text[pos:]))
    return [p for p in out if p[0] == "note" or p[1] != ""]


def plain(pieces):
    """The words of a piece list (styled runs included, verses and notes not)."""
    return "".join(v for k, v in pieces if k not in ("v", "note"))


def note_parts(text):
    """A rich note's text as [(italic, text), ...]."""
    out, it = [], False
    for tok in re.split(r"([\x0e\x0f])", text):
        if tok == NOTE_IT_ON:
            it = True
        elif tok == NOTE_IT_OFF:
            it = False
        elif tok:
            out.append((it, tok))
    return out


def note_plain(text):
    return re.sub(r"[\x0e\x0f]", "", text)


def note_ref(text):
    """(c:v reference, rest) when a note opens with its reference (\\fr), else ("", text)."""
    m = re.match(r"(\d+[:.,]\d+[a-z]?(?:[\-–,.]\d+[a-z]?)*)[:.]?\s+(.*)", text, re.S)
    return (m.group(1), m.group(2)) if m else ("", text)


def read_book(usfm, rich=False):
    book = {"code": None, "h": None, "toc1": None, "mt": [], "mts": [], "blocks": []}
    blocks = book["blocks"]
    para = None
    pieces = rich_pieces if rich else split_notes
    for raw in re.split(RICH_SPLIT if rich else SPLIT, usfm):
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
            if strip_chars(rest).strip():
                book["mts"].append((marker, re.sub(r"\s+", " ", strip_chars(rest)).strip()))
        elif marker == "c":
            blocks.append(("chapter", int(rest.split()[0])))
            para = None
        elif marker == "v":
            num, _, text = rest.partition(" ")
            if para is None:
                para = ("para", "p", [])
                blocks.append(para)
            para[2].append(("v", num))
            para[2].extend(pieces(text))
        elif marker in (RICH_PARA if rich else PARA):
            para = ("para", marker, [])
            blocks.append(para)
            if rest.strip():
                para[2].extend(pieces(rest))
        elif rich and marker in INTRO:
            blocks.append(("intro", marker, pieces(rest) if rest.strip() else []))
            para = None
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


PERIPHERAL = ("FRT", "INT", "GLO", "BAK", "OTH", "XXA", "XXB", "XXC", "CNC", "TDX", "NDX")


def read_zip(path, only=None, rich=False):
    books = []
    with zipfile.ZipFile(path) as z:
        names = sorted((n for n in z.namelist() if n.lower().endswith((".usfm", ".sfm"))),
                       key=lambda n: int(re.match(r"(\d+)", n.split("/")[-1]).group(1)) if re.match(r"\d+", n.split("/")[-1]) else 9999)
        for name in names:
            text = z.read(name).decode("utf-8-sig")
            b = read_book(text, rich)
            wanted_periph = rich and only and b["code"] in only
            if b["code"] in PERIPHERAL and not wanted_periph:
                continue
            if only and b["code"] not in only:
                continue
            if not wanted_periph and not any(bl[0] == "chapter" for bl in b["blocks"]):
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
