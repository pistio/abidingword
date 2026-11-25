#!/usr/bin/env python3
"""Approved slots 1 and 4 are scored documents WITH deuterocanonical books.
How does E3's seal treat one, by how its `books` column is written?

    deutero.py <project> <binary> <work> <out.json>
"""
import json, os, sys
project, binary, work, out = sys.argv[1:5]
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import lab as L
import scenarios_lib as S

tob = "\\id TOB review invented text, not Scripture\n\\c 1\n\\p\n\\v 1 Review invented words of a book outside the 66.\n\\v 2 More invented words.\n"
rut_name, rut = S.member("RUT")
doc = S.markdown("RUT") + b"\n# Tobit\n\n## 1\n\n 1 Review invented words of a book outside the 66. 2 More invented words.\n"
res = {}
for name, books in (("deutero-books-listed", "RUT,TOB"), ("deutero-books-all", "all"), ("deutero-books-66-only", "RUT")):
    lab = L.Lab(project, binary, os.path.join(work, name))
    d = {"key": "dt", "cls": "scored", "file": ("dt.md", doc), "fmt": "md", "kind": "generated", "books": books,
         "reference": [(rut_name, rut), ("70-TOBreview.usfm", tob.encode())]}
    r, _ = lab.scenario(name, [d])
    res[name] = r
    s = r.get("score", {})
    print(name, "|", r["outcome"], "|", r.get("refusals") or s.get("verdict_line"), "|", (s.get("documents") or {}))
json.dump(res, open(out, "w", encoding="utf-8"), ensure_ascii=False, indent=1)
