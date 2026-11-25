#!/usr/bin/env python3
"""Runs the reviewer's vectors through both E3 readers (Python: the module;
Rust: the reviewer's probe, which includes usfm_text.rs read-only) and judges
each reading against the reviewer's hand-written expectation.

    run_vectors.py <repo> <probe binary> <vectors.json> <out.json>
"""
import importlib.util, json, os, subprocess, sys

repo, probe, vec_path, out = sys.argv[1:5]
spec = importlib.util.spec_from_file_location("usfm_text", os.path.join(repo, "scripts", "evaluation", "usfm_text.py"))
py = importlib.util.module_from_spec(spec)
spec.loader.exec_module(py)
vectors = json.load(open(vec_path, encoding="utf-8"))
rs = json.loads(subprocess.run([probe], input=json.dumps([{"name": v["name"], "text": v["usfm"]} for v in vectors]),
                               capture_output=True, text=True, check=True).stdout)


def judge(expect, book_code, verses, problems, before):
    """[] if the reading meets the expectation, else the reasons."""
    why = []
    labels = [v[1] for v in verses]
    if "ok" in expect:
        if problems:
            why.append(f"refused ({', '.join(problems)}) but the reviewer expects a clean reading")
        if [v[:3] for v in verses] != expect["ok"]:
            why.append(f"read {[v[:3] for v in verses]!r}, expected {expect['ok']!r}")
        if "before" in expect and before != expect["before"]:
            why.append(f"{before} verses before a chapter, expected {expect['before']}")
    elif expect.get("refuse"):
        if not problems:
            why.append("NOT refused: read without any problem " + repr([v[:3] for v in verses]))
        if "labels" in expect and labels != expect["labels"]:
            why.append(f"labels {labels}, expected every \\v kept: {expect['labels']}")
        for cv, text in expect.get("keep", {}).items():
            c, v = cv.split(":")
            got = [x[2] for x in verses if x[0] == c and x[1] == v]
            if text not in got:
                why.append(f"{cv} read {got!r}, expected {text!r} (a problem swallowed it)")
    return why


results, summary = [], {"vectors": len(vectors), "python_meets": 0, "rust_meets": 0, "readers_identical": 0,
                        "choices": 0}
for v in vectors:
    p = py.read_text(v["usfm"])
    pv = [[x["chapter"], x["label"], x["text"]] for x in p["verses"]]
    pp = [x["code"] for x in p["problems"]]
    r = rs[v["name"]]
    rv = [x[:3] for x in r["verses"]]
    rp = r["problems"]
    same = pv == rv and sorted(pp) == sorted(rp)
    summary["readers_identical"] += same
    entry = {"name": v["name"], "group": v["group"], "expect": v["expect"], "python": {"verses": pv, "problems": pp},
             "rust": {"verses": rv, "problems": rp}, "readers_identical": same, "book": [p["book"], r["book"]]}
    if "choice" in v["expect"]:
        summary["choices"] += 1
        entry["verdict"] = "choice (judged in the report)"
    else:
        wp = judge(v["expect"], p["book"], pv, pp, p["verses_before_chapter"])
        wr = judge(v["expect"], r["book"], rv, rp, r["before"])
        summary["python_meets"] += not wp
        summary["rust_meets"] += not wr
        entry["python_unmet"], entry["rust_unmet"] = wp, wr
        entry["verdict"] = "meets" if not wp and not wr else "UNMET"
    results.append(entry)
summary["unmet"] = [e["name"] for e in results if e["verdict"] == "UNMET"]
summary["readers_differ"] = [e["name"] for e in results if not e["readers_identical"]]
json.dump({"summary": summary, "results": results}, open(out, "w", encoding="utf-8"), ensure_ascii=False, indent=1)
print(json.dumps(summary, ensure_ascii=False, indent=1))
for e in results:
    if e["verdict"] != "meets" or not e["readers_identical"]:
        print("--", e["name"], e["verdict"], "| py:", e["python"], "| rs:", e["rust"], "|", e.get("python_unmet"), e.get("rust_unmet"))
