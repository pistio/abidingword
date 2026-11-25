#!/usr/bin/env python3
"""The generated whole Berean Standard Bible through the OFFICIAL path in the
review lab (seal, run once, official score under a simulated owner revision),
less 2 John, whose USFM member is a disclosed book (check-set refuses it).

    real_official.py <project> <binary> <work> <out.json>
"""
import io, json, os, re, sys, zipfile

project, binary, work, out = sys.argv[1:5]
here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(here, "..", "lab"))
import lab as L

md = open("/tmp/claude-0/realdocs/rv-engbsb.md", encoding="utf-8").read() if os.path.exists("/tmp/claude-0/realdocs/rv-engbsb.md") else None
src = os.path.join(project, "resources", "scripture", "engbsb", "engbsb_usfm.zip")
z = zipfile.ZipFile(src)
# the document without 2 John: drop its "# 2 John" section
parts = re.split(r"(?m)^(?=# )", md)
doc = "".join(p for p in parts if not p.startswith("# 2 John\n")).encode("utf-8")
kept = [(n, z.read(n)) for n in z.namelist() if n.lower().endswith((".usfm", ".htm")) and "2JN" not in n]
removed = [(n, L.sha(z.read(n))) for n in z.namelist() if n not in dict(kept)]
lab = L.Lab(project, binary, work)
d = {"key": "bsb-whole", "cls": "scored", "file": ("bsb-whole.md", doc), "fmt": "md", "kind": "generated", "books": "all",
     "reference": kept, "archive": ("https://ebible.org/Scriptures/engbsb_usfm.zip (bundled copy)", L.sha(open(src, "rb").read())),
     "removed": removed}
r, ctx = lab.scenario("real-bsb-official", [d])
json.dump(r, open(out, "w", encoding="utf-8"), ensure_ascii=False, indent=1)
s = r.get("score", {})
print(r["outcome"], s.get("verdict"), s.get("failed_validity"), s.get("integrity"))
print(s.get("verdict_line"))
