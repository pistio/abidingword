#!/usr/bin/env python3
"""Runs the reviewer's probe (the E3 Rust reader, included read-only) on every
USFM member of the 13 bundled references and writes {edition/member: [[c, v, text], ...]}.

    rust_dump.py <repo> <probe binary> <out.json>
"""
import glob, json, os, subprocess, sys, zipfile

repo, probe, out = sys.argv[1:4]
dump = {}
for z in sorted(glob.glob(os.path.join(repo, "resources", "scripture", "*", "*_usfm.zip"))):
    ed = os.path.basename(os.path.dirname(z))
    with zipfile.ZipFile(z) as zz:
        inputs = [{"name": f"{ed}/{nm}", "text": zz.read(nm).decode("utf-8")}
                  for nm in zz.namelist() if nm.lower().endswith((".usfm", ".sfm"))]
    r = subprocess.run([probe], input=json.dumps(inputs), capture_output=True, text=True, check=True)
    for name, b in json.loads(r.stdout).items():
        if b["problems"]:
            print("problems", name, b["problems"][:5])
        dump[name] = [v[:3] for v in b["verses"]]
json.dump(dump, open(out, "w", encoding="utf-8"), ensure_ascii=False)
print(len(dump), "members;", sum(len(v) for v in dump.values()), "verses")
