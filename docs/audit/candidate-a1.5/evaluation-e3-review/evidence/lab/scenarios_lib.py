#!/usr/bin/env python3
"""Materials shared by scenarios.py and probes.py: public-domain verse text
from the bundled engwebp reference (read by the reviewer's extractor) and
document/reference builders."""
import hashlib, importlib.util, os, sys, zipfile
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import lab as L
project = os.environ.get("AW_REVIEW_PROJECT", os.path.normpath(os.path.join(HERE, "..", "..", "..", "..", "..", "..")))


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m


rev = load("rev_usfm", os.path.join(HERE, "..", "r1", "rev_usfm.py"))
WEBP = os.path.join(project, "resources", "scripture", "engwebp", "engwebp_usfm.zip")
WEBP_SHA = hashlib.sha256(open(WEBP, "rb").read()).hexdigest()
zw = zipfile.ZipFile(WEBP)
MEMBERS = {n: zw.read(n) for n in zw.namelist()}
DISCLOSED = {"00-FRTengwebp.usfm", "106-GLOengwebp.usfm"}
TITLES = {"RUT": "Ruth", "JON": "Jonah", "OBA": "Obadiah", "PHM": "Philemon", "JUD": "Jude", "3JN": "3 John",
          "HAG": "Haggai", "NAM": "Nahum"}


def member(code):
    for n, b in MEMBERS.items():
        if n.endswith(f"{code}engwebp.usfm"):
            return n, b
    raise KeyError(code)


def removed_of(kept):
    return [(n, L.sha(b)) for n, b in MEMBERS.items() if n not in kept]


def verses(code):
    n, b = member(code)
    return rev.read(b.decode("utf-8"))["verses"]


def markdown(code, vs=None, title=None):
    vs = vs if vs is not None else verses(code)
    out, chapter, para = [f"# {title or TITLES.get(code, code)}\n"], None, []
    for c, v, t in vs:
        if c != chapter:
            if para:
                out.append(" " + " ".join(para) + "\n"); para = []
            chapter = c
            out.append(f"## {c}\n")
        if t:
            para.append(f"{v} {t}")
            if len(para) == 5:
                out.append(" " + " ".join(para) + "\n"); para = []
    if para:
        out.append(" " + " ".join(para) + "\n")
    return "\n".join(out).encode("utf-8")


PROSE = ("# Notes on a pottery workshop\n\nInvented prose for the review: the kiln was fired twice, the glaze "
         "held, and the apprentices swept the yard before the rain. Nothing here is Scripture.\n\n").encode()


def doc(key, cls, data, ref_codes=None, books="all", fmt="md", kind="generated", **kw):
    d = {"key": key, "cls": cls, "file": (f"{key}.{fmt}", data), "fmt": fmt, "kind": kind, "books": books}
    if ref_codes is not None:
        kept = dict(member(c) for c in ref_codes)
        d["reference"] = list(kept.items())
        d["archive"] = ("https://ebible.org/Scriptures/engwebp_usfm.zip (bundled copy)", WEBP_SHA)
        d["removed"] = removed_of(kept)
    d.update(kw)
    return d


def invented_ref(key, usfm_text, cls="scored", data=None, books="all", name="X.usfm"):
    return {"key": key, "cls": cls, "file": (f"{key}.md", data or PROSE), "fmt": "md", "kind": "generated",
            "books": books, "reference": [(name, usfm_text.encode("utf-8"))]}


