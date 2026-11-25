#!/usr/bin/env python3
"""The reviewer's R2 and integrity scenarios, through lab.py with the real
harness binary. Writes one JSON summary.

    scenarios.py <project> <binary> <work> <out.json> [--only=a,b]
"""
import hashlib, importlib.util, io, json, os, re, shutil, stat, subprocess, sys, zipfile

project, binary, work, out = sys.argv[1:5]
only = set(sys.argv[5].split("=", 1)[1].split(",")) if len(sys.argv) > 5 else set()
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import lab as L


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


results = {"binary_sha256": None, "scenarios": {}, "probes": {}}
lab = L.Lab(project, binary, work)
results["binary_sha256"] = lab.binary_sha
runs = {}


def go(name, docs, **kw):
    if only and name not in only:
        return None
    r, ctx = lab.scenario(name, docs, **kw)
    results["scenarios"][name] = r
    if ctx:
        runs[name] = ctx
    s = r.get("score", {})
    print(name, "|", r["outcome"], "|", s.get("verdict", ""), "|", (r.get("refusals") or r.get("runner") or s.get("failed_validity") or "")
          , file=sys.stderr)
    return r


# ------------------------------------------------------------------ R2
go("r2-clean", [doc("cl-rut", "scored", markdown("RUT"), ["RUT"], "RUT"),
                doc("cl-jon", "scored", markdown("JON"), ["JON"], "JON"),
                doc("cl-ctl", "control", PROSE)])
go("r2-zero-verified-prose", [doc("zp-a", "scored", PROSE, ["RUT"], "RUT"),
                              doc("zp-b", "scored", PROSE + b"More invented prose.\n", ["JON"], "JON")])
go("r2-empty-zip", [{**doc("ez", "scored", PROSE), "reference": L.zip_bytes([]), "sources": False}])
go("r2-prose-only-reference", [{**doc("po", "scored", PROSE), "reference": [("copr.htm", b"<p>licence only</p>")]}])
go("r2-unsupported-only-tobit", [invented_ref("tob", "\\id TOB invented\n\\c 1\n\\p\n\\v 1 Invented words of a book outside the 66.\n\\v 2 More.\n",
                                              data=b"# Tobit\n\n## 1\n\n1 Invented words of a book outside the 66. 2 More.\n")])
go("r2-unsupported-only-verse0-chapter-beyond", [invented_ref("v0", "\\id RUT invented\n\\c 1\n\\p\n\\v 0 Invented title verse.\n\\c 9\n\\p\n\\v 1 Invented chapter beyond Ruth.\n",
                                                              data=b"# Ruth\n\n## 1\n\n0 Invented title verse.\n\n## 9\n\n1 Invented chapter beyond Ruth.\n", books="RUT")])
go("r2-numbered-but-empty", [invented_ref("ne", "\\id RUT invented\n\\c 1\n\\p\n\\v 1\n\\v 2\n\\v 3\n", books="RUT")])
go("r2-declared-book-not-printed", [doc("db", "scored", markdown("RUT") + markdown("JON"), ["RUT"], "RUT,JON")])
go("r2-all-scored-refused-whole", [doc("rw-a", "scored", b"%PDF-1.4\n%junk, not a readable PDF\n", ["RUT"], "RUT", fmt="pdf"),
                                   doc("rw-b", "scored", b"%PDF-1.4\n%more junk\n", ["JON"], "JON", fmt="pdf")])
# Valid references, no recognisable Scripture: Ruth's words run together as prose, no numbers, no headings.
ruth_prose = ("Invented framing. " + " ".join(t for _, _, t in verses("RUT")[:20])).encode("utf-8")
go("r2-no-recognised-scripture", [doc("nr", "scored", ruth_prose, ["RUT"], "RUT")])
# One faithful verse among a very large collection: 66 books printed, one verse in the document.
big = [n for n in MEMBERS if n.endswith(".usfm") and n not in DISCLOSED]
one_verse = [x for x in verses("RUT") if x[0] == "1" and x[1] == "16"]
onedoc = PROSE + markdown("RUT", one_verse) + PROSE
go("r2-one-faithful-verse-large", [{**doc("one", "scored", onedoc), "reference": [(n, MEMBERS[n]) for n in big],
                                    "archive": ("https://ebible.org/Scriptures/engwebp_usfm.zip (bundled copy)", WEBP_SHA),
                                    "removed": removed_of(set(big))},
                                   doc("one-b", "scored", PROSE, ["JON"], "JON")])
# The smallest unit the frozen recogniser verifies is a whole chapter (partial chapters are refused:
# 3, 5, 8 and 12 of Ruth 1's 22 verses verified nothing; all 22 verified 22). Psalm 117 has two verses:
# two faithful verses against a whole Bible (and a second scored document verifying nothing).
psa117 = [x for x in verses("PSA") if x[0] == "117"]
go("r2-two-verses-in-whole-bible", [{**doc("ps", "scored", PROSE + markdown("PSA", psa117, "Psalms") + PROSE),
                                     "reference": [(n, MEMBERS[n]) for n in big],
                                     "archive": ("https://ebible.org/Scriptures/engwebp_usfm.zip (bundled copy)", WEBP_SHA),
                                     "removed": removed_of(set(big))},
                                    doc("ps-b", "scored", PROSE, ["JON"], "JON")])
# Only unsupported content in the document, scorable verses in the reference.
go("r2-document-only-unsupported", [invented_ref("ou", "\\id RUT invented\n\\c 1\n\\p\n\\v 0 Invented verse zero words here.\n\\v 1 Invented first verse.\n",
                                                 data=b"# Ruth\n\n## 1\n\n0 Invented verse zero words here.\n", books="RUT")])
# A faithful scored document plus exploratory/unsupported material: exploratory must not count.
go("r2-exploratory-not-counted", [doc("ex-a", "scored", PROSE, ["RUT"], "RUT"),
                                  doc("ex-b", "exploratory", markdown("JON"), ["JON"], "JON")])
# A real wrong word: fail, never inconclusive.
wrong = markdown("RUT").replace(b"Naomi", b"Naomy", 1)
go("r2-wrong-text", [doc("wt", "scored", wrong, ["RUT"], "RUT")])

# ------------------------------------------------------------------ integrity
go("gate-shut", [doc("gs", "scored", markdown("OBA"), ["OBA"], "OBA")], gate=False)
go("reference-raw-publisher-archive", [{**doc("ra", "scored", markdown("RUT")), "reference": open(WEBP, "rb").read(),
                                       "ref_name": "engwebp_usfm.zip", "sources": True, "removed": []}])
go("sources-member-hash-wrong", [{**doc("sw", "scored", markdown("RUT"), ["RUT"], "RUT"),
                                  "tamper": lambda rows: [r[:5] + ["0" * 64] + r[6:] if r[6] == "kept" else r for r in rows]}])
go("sources-removed-member-present", [{**doc("sr", "scored", markdown("RUT"), ["RUT"], "RUT"),
                                       "tamper": lambda rows: rows + [[rows[0][0], rows[0][1], rows[0][2], rows[0][3], rows[0][4], rows[0][5], "removed"]]}])
go("sources-row-missing", [{**doc("sm", "scored", markdown("RUT"), ["RUT"], "RUT"),
                            "tamper": lambda rows: [r for r in rows if r[6] != "kept"]}])
go("unicode16-in-reference", [invented_ref("u16", "\\id RUT invented\n\\c 1\n\\p\n\\v 1 Invented \U00016D40 text.\n\\v 2 Plain.\n",
                                           data=b"# Ruth\n\n## 1\n\n1 Invented text. 2 Plain.\n", books="RUT")])
json.dump(results, open(out, "w", encoding="utf-8"), ensure_ascii=False, indent=1)
