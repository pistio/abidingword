#!/usr/bin/env python3
"""Runs the reviewer's vectors through the scorer's fidelity.py and the
harness's fidelity.rs (via the review probe), and compares both with each
other and with the reviewer's expectation."""
import json, subprocess, sys, importlib.util, collections
REPO = "/home/user/abidingword"
PROBE = sys.argv[1]
spec = importlib.util.spec_from_file_location("fidelity", REPO + "/scripts/evaluation/fidelity.py")
fid = importlib.util.module_from_spec(spec); spec.loader.exec_module(fid)
vectors = json.load(open("vectors.json"))
rs = subprocess.run([PROBE, "fidelity"], input="\n".join(json.dumps(v, ensure_ascii=False) for v in vectors),
                    capture_output=True, text=True, check=True).stdout.splitlines()
rs = {r["id"]: r for r in map(json.loads, rs)}
rows, stats = [], collections.Counter()
for v in vectors:
    pc, pk = fid.judge(v["edition"], v["found"], v["matched"])
    r = rs[v["id"]]
    agree = (pc, pk) == (r["class"], r["kind"])
    faithful = pc in fid.FAITHFUL
    verdict_ok = (faithful and v["expect"] == "faithful") or (not faithful and v["expect"] == "wrong")
    stats["agree" if agree else "DISAGREE"] += 1
    stats["as_expected" if verdict_ok else "UNEXPECTED"] += 1
    if v["expect"] == "wrong" and faithful: stats["WRONG_TEXT_COUNTED_FAITHFUL"] += 1
    if v["expect"] == "faithful" and not faithful: stats["faithful_text_counted_wrong"] += 1
    rows.append(dict(id=v["id"], what=v["what"], expect=v["expect"], python=[pc, pk], rust=[r["class"], r["kind"]],
                     agree=agree, as_expected=verdict_ok))
json.dump(rows, open("results.json", "w"), ensure_ascii=False, indent=1)
print(dict(stats))
for r in rows:
    if not r["agree"] or not r["as_expected"]:
        print(r)
