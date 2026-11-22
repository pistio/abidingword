#!/usr/bin/env python3
"""Verse text of the shared app reader (as the E2 inventory stores it) against
the reviewer's independent extractor, on the real bundled references."""
import json, subprocess, sys, glob, os, zipfile, collections, importlib.util
REPO = "/home/user/abidingword"; PROBE = sys.argv[1]
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import indep_usfm
spec = importlib.util.spec_from_file_location("fid", REPO + "/scripts/evaluation/fidelity.py")
fid = importlib.util.module_from_spec(spec); spec.loader.exec_module(fid)
summary = {}
for z in sorted(glob.glob(REPO + "/resources/scripture/*/*_usfm.zip")):
    name = os.path.basename(z)
    rs = json.loads(subprocess.run([PROBE, "inventory", z], capture_output=True, text=True, check=True).stdout)
    app = collections.defaultdict(list)
    for e in rs["entries"]:
        app[e["file"]].append(e)
    st = collections.Counter(); examples = []
    with zipfile.ZipFile(z) as zf:
        for member, entries in app.items():
            mine = indep_usfm.extract(zf.read(member).decode("utf-8"))
            labels_app = [(e["label"].split(" ")[1]) for e in entries]
            labels_mine = [f"{c}:{v}" for c, v, _ in mine]
            if labels_app != labels_mine:
                st["files_label_sequence_differs"] += 1
                examples.append(f"{member}: labels differ ({len(labels_app)} vs {len(labels_mine)})")
                continue
            for e, (c, v, t) in zip(entries, mine):
                st["verses"] += 1
                if e["text"] == t:
                    st["identical"] += 1
                else:
                    cls = fid.classify(t, e["text"])
                    st["differ_" + cls] += 1
                    if cls == "mismatch" and len(examples) < 12:
                        examples.append(f"{e['label']}: app «{e['text'][:160]}» | independent «{t[:160]}»")
    summary[name] = {"counts": dict(st), "examples": examples}
    print(name, dict(st)); [print("   ", x) for x in examples[:6]]
json.dump(summary, open("text-crosscheck.json", "w"), ensure_ascii=False, indent=1)
