#!/usr/bin/env python3
"""The independent review's three scorer alterations, step for step, through
the anchored interface (the runner commits the run record; the scorer is
given that commit). Synthetic only: the committed fake harness.

Before each alteration the run must score (verdict 'fail'); after it, the
scorer must refuse (exit 2). Exit 0 if all three are refused, 1 if any
alteration is scored, 2 on anything unexpected.
"""
import importlib.util
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

sys.dont_write_bytecode = True
REPO = Path(sys.argv[1]).resolve()
ROOT = Path(sys.argv[2]) if len(sys.argv) > 2 else None
spec = importlib.util.spec_from_file_location("set_tools_tests", REPO / "tests/evaluation/test_withheld_set_tools.py")
t = importlib.util.module_from_spec(spec)
spec.loader.exec_module(t)
work = Path(tempfile.mkdtemp(prefix="anchored-", dir=ROOT))
revision = subprocess.run(["git", "-C", str(REPO), "rev-parse", "HEAD"], text=True, capture_output=True).stdout.strip()


def attempt(fixture, folder):
    r, data = fixture.score(folder)
    return {"exit": r.returncode, "verdict": data["assessment"]["verdict"] if data else None,
            "refusals": [l for l in r.stderr.splitlines() if l.startswith("refused:")]}


results = []
case = work / "mixed_allowlist"
case.mkdir()
f = t.SetFixture(str(case), [{"key": "M1", "class": "mixed", "format": "epub",
    "behaviour": {"write": {".control.json": t.control(["JHN 3:16", "JHN 3:17"])}}}], allowed={"M1": ["JHN 3:16"]})
_, folder = f.run(check=True)
before = attempt(f, folder)
Path(f.seal, "allowed/M1.txt").write_text("JHN 3:16-17\n")
after = attempt(f, folder)
f.reseal()
after_reseal = attempt(f, folder)
results.append({"case": "changed_sealed_allowlist", "run_record": f.record, "before": before,
                "after": after, "after_resealing_mutable_seal_txt": after_reseal})

for mode in ["report", "run_exit"]:
    case = work / mode
    case.mkdir()
    behaviour = {"write": {".json": t.report(10, 10, wrong=["MAT 1:2"] if mode == "report" else [])}}
    if mode == "run_exit":
        behaviour["exit"] = 101
    f = t.SetFixture(str(case), [{"key": "S1", "class": "scored", "kind": "authentic", "format": "epub",
                                  "behaviour": behaviour}], allowed={})
    _, folder = f.run(check=True)
    record = Path(f.seal).parent / "run"
    pinned = {n: t.sha((record / n).read_bytes()) for n in ["RUN.txt", "OUTPUTS.sha256", "CHECK.json"]}
    before = attempt(f, folder)
    if mode == "report":
        p = Path(folder) / "S1.json"
        v = json.loads(p.read_text())
        v["differences"] = []
        p.write_text(json.dumps(v))
    else:
        p = Path(folder) / "RUN.txt"
        p.write_text(p.read_text().replace("S1 exit 101 seconds ", "S1 exit 0 seconds "))
    f.rehash_outputs(folder)
    after = attempt(f, folder)
    results.append({"case": f"changed_external_{mode}_and_rehashed_outputs", "run_record": f.record,
                    "before": before, "after": after,
                    "committed_record_unchanged": all(t.sha((record / n).read_bytes()) == d for n, d in pinned.items()),
                    "tracked_git_worktree_clean": f.git("status", "--porcelain").strip() == ""})

summary = {"candidate_head": revision, "synthetic_only": True, "work_folder": str(work), "probes": results}
print(json.dumps(summary, indent=2))
ok = all(r["before"]["exit"] == 0 and r["before"]["verdict"] == "fail" and r["after"]["exit"] == 2 for r in results)
ok = ok and results[0]["after_resealing_mutable_seal_txt"]["exit"] == 2
scored = any(r["after"]["exit"] == 0 for r in results)
sys.exit(0 if ok else (1 if scored else 2))
