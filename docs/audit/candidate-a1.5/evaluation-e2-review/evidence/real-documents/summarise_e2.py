#!/usr/bin/env python3
"""Summarises E2 harness reports and re-checks each with the scorer's own
observe_report() (fidelity re-classification, accounting, inventory compare
against the Python inventory computed as check-set.py would)."""
import json, os, sys, glob, importlib.util, collections
REPO = "/home/user/abidingword"
spec = importlib.util.spec_from_file_location("score", REPO + "/scripts/evaluation/score-set.py")
score = importlib.util.module_from_spec(spec); spec.loader.exec_module(score)
ri = score.reference_inventory
docs, out = sys.argv[1], sys.argv[2]
rows = json.load(open(os.path.join(docs, "documents.json")))
result = {}
for r in rows:
    p = os.path.join(out, r["key"] + ".json")
    if not os.path.exists(p):
        rej = os.path.join(out, r["key"] + ".rejected.md")
        result[r["key"]] = {"report": False, "rejected": os.path.exists(rej)}
        print(r["key"], "NO REPORT", "rejected" if os.path.exists(rej) else "")
        continue
    j = json.load(open(p))
    check_inv = ri.read(os.path.join(docs, r["reference"]), {r["book"]})
    obs, integrity = score.observe_report(j, check_inv)
    t = j["totals"]
    result[r["key"]] = {k: obs[k] for k in ("printed", "verified", "verified_exact", "verified_canonical", "verified_layout",
                                           "verified_mismatch", "verified_mismatch_kinds", "verified_absent",
                                           "verified_unvalidatable", "held_faithful", "held_mismatch", "refused", "not_found")}
    result[r["key"]]["integrity"] = integrity
    result[r["key"]]["mismatch_refs"] = [m["reference"] for m in obs["verified_mismatch_references"]]
    result[r["key"]]["verified_absent_refs"] = obs["verified_absent_references"]
    print(f"{r['key']:14s} printed {obs['printed']:5d} verified {obs['verified']:5d} exact {obs['verified_exact']:5d} "
          f"canon {obs['verified_canonical']} layout {obs['verified_layout']:4d} MISMATCH {obs['verified_mismatch']:3d} "
          f"{obs['verified_mismatch_kinds']} absent {obs['verified_absent']} held {obs['held_faithful']}/{obs['held_mismatch']} "
          f"refused {obs['refused']} notfound {obs['not_found']} integrity {len(integrity)}")
json.dump(result, open(os.path.join(out, "review-summary.json"), "w"), ensure_ascii=False, indent=1)
