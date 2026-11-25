#!/usr/bin/env python3
"""Regression: the same reviewer-designed sets through the E2 tools and binary
and through the E3 tools and binary (official path, simulated owner revision
in a scratch repository). Invented text only ("review invented text").

    regress.py <tools dir (holds scripts/ docs/)> <e2|e3> <binary> <work> <out.json>
"""
import json, os, random, sys

tools, version, binary, work, out = sys.argv[1:6]
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import lab as L

rng = random.Random(1125)
A = ["the weaver", "a ferryman", "the lamplighter", "two shepherds", "the scribe's daughter", "a cooper"]
B = ["mended the nets", "counted the jars", "carried water", "sang at the threshold", "swept the court"]
C = ["before dawn", "beside the cistern", "under the sycamore", "at the east gate", "when the wind rose"]


def invented(n):
    return [f"Then {rng.choice(A)} {rng.choice(B)} {rng.choice(C)}, and said, “It is verse {v}.”" for v in range(1, n + 1)]


def usfm(code, texts, raw=None):
    lines = [f"\\id {code} review invented text, not Scripture", "\\h Test", "\\mt1 Test", "\\c 1", "\\p"]
    lines += [f"\\v {v} {(raw or {}).get(v, t)}" for v, t in enumerate(texts, 1)]
    return "\n".join(lines) + "\n"


def md(title, texts):
    out, para = [f"# {title}\n", "## 1\n"], []
    for v, t in enumerate(texts, 1):
        para.append(f"{v} {t}")
        if len(para) == 5:
            out.append(" " + " ".join(para) + "\n"); para = []
    if para:
        out.append(" " + " ".join(para) + "\n")
    return "\n".join(out).encode("utf-8")


def d(key, cls, data, ref, books):
    return {"key": key, "cls": cls, "file": (f"{key}.md", data), "fmt": "md", "kind": "generated", "books": books,
            "reference": ref}


PROSE = b"# A note on lamps\n\nReview invented prose; nothing here is Scripture. The lamps were trimmed at dusk.\n"
lab = L.Lab(tools, binary, work, version=version)
res = {"version": version, "binary_sha256": lab.binary_sha, "scenarios": {}}

# a. Notes placed directly against punctuation, as eBible editions place them; the document prints the edition.
oba = invented(21)
raw = {v: t[:-1] + f"\\f + \\fr 1:{v} \\ft review note\\f*”" for v, t in enumerate(oba, 1) if v % 2}
raw.update({v: t.replace(", and said", f"\\x - \\xo 1:{v} \\xt Ruth 1:1\\x*, and said") for v, t in enumerate(oba, 1) if not v % 2})
res["scenarios"]["notes-against-punctuation"] = lab.scenario("notes", [d("rg-oba", "scored", md("Obadiah", oba), [("31-OBA.usfm", usfm("OBA", oba, raw))], "OBA")])[0]
# b. An unclosed cross reference before a later footnote.
jud = invented(25)
raw = {3: jud[2] + "\\x - \\xo 1:3 \\xt Ruth 1:1", 6: jud[5][:-1] + "\\f + \\ft review note\\f*”"}
res["scenarios"]["unclosed-xref"] = lab.scenario("unclosed", [d("rg-jud", "scored", md("Jude", jud), [("65-JUD.usfm", usfm("JUD", jud, raw))], "JUD")])[0]
# c. A USFM 3 milestone in the reference.
phm = invented(25)
raw = {4: jud[3].replace("said, ", "said, \\qt-s |who=\"review\"\\*") + "\\qt-e\\*"}
phm[3] = jud[3]
res["scenarios"]["milestone"] = lab.scenario("milestone", [d("rg-phm", "scored", md("Philemon", phm), [("57-PHM.usfm", usfm("PHM", phm, raw))], "PHM")])[0]
# d. Vacuous: a prose-only reference package and a Tobit-only reference.
res["scenarios"]["vacuous"] = lab.scenario("vacuous", [
    d("rg-v1", "scored", PROSE, [("copr.htm", b"<p>licence only</p>")], "all"),
    d("rg-v2", "scored", b"# Tobit\n\n## 1\n\n1 Review invented words. 2 More invented words.\n",
      [("69-TOB.usfm", "\\id TOB review invented\n\\c 1\n\\p\n\\v 1 Review invented words.\n\\v 2 More invented words.\n")], "all")])[0]
# e. Zero coverage with sound references.
hag = invented(15)
res["scenarios"]["zero-coverage"] = lab.scenario("zero", [d("rg-z", "scored", PROSE, [("37-HAG.usfm", usfm("HAG", hag))], "HAG")])[0]
json.dump(res, open(out, "w", encoding="utf-8"), ensure_ascii=False, indent=1)
for k, r in res["scenarios"].items():
    s = r.get("score", {})
    print(version, k, "|", r["outcome"], "|", s.get("verdict"), "|", (s.get("verdict_line") or s.get("accuracy_claim") or "")[:200], "|",
          r.get("refusals", "") or s.get("failed_validity", ""), "| criteria failed:", s.get("criteria_failed"),
          {k: v for k, v in (s.get("documents") or {}).items()})
