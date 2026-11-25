#!/usr/bin/env python3
"""The 13 bundled references: the reviewer's extractor (rev_usfm.py) against
the E3 Python reader (scripts/evaluation/usfm_text.py), and optionally the E3
Rust reader (a JSON dump produced by the reviewer's probe crate).

    compare_bundled.py <repo> <out.json> [rust-dump.json]
"""
import glob, hashlib, importlib.util, json, os, sys, zipfile

repo, out = sys.argv[1], sys.argv[2]
rust = json.load(open(sys.argv[3], encoding="utf-8")) if len(sys.argv) > 3 else None
here = os.path.dirname(os.path.abspath(__file__))


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m


rev = load("rev_usfm", os.path.join(here, "rev_usfm.py"))
py = load("usfm_text", os.path.join(repo, "scripts", "evaluation", "usfm_text.py"))
result = {"editions": {}, "differences": []}
for z in sorted(glob.glob(os.path.join(repo, "resources", "scripture", "*", "*_usfm.zip"))):
    ed = os.path.basename(os.path.dirname(z))
    st = {"zip_sha256": hashlib.sha256(open(z, "rb").read()).hexdigest(), "verses": 0, "same_text": 0,
          "differ": 0, "label_mismatch": 0, "e3_problems": 0, "reviewer_trouble": 0, "rust_same": 0, "rust_differ": 0}
    with zipfile.ZipFile(z) as zz:
        for nm in zz.namelist():
            if not nm.lower().endswith((".usfm", ".sfm")):
                continue
            src = zz.read(nm).decode("utf-8")
            a = rev.read(src)
            b = py.read_text(src)
            st["e3_problems"] += len(b["problems"])
            st["reviewer_trouble"] += len(a["trouble"])
            if a["trouble"]:
                result["differences"].append({"edition": ed, "member": nm, "reviewer_trouble": a["trouble"][:5]})
            bv = [[v["chapter"], v["label"], v["text"]] for v in b["verses"]]
            if [x[:2] for x in a["verses"]] != [x[:2] for x in bv]:
                st["label_mismatch"] += 1
                result["differences"].append({"edition": ed, "member": nm, "labels": "differ",
                                              "reviewer": len(a["verses"]), "e3": len(bv)})
                continue
            rv = None
            if rust is not None:
                rv = rust.get(f"{ed}/{nm}")
            for i, (x, y) in enumerate(zip(a["verses"], bv)):
                st["verses"] += 1
                if x[2] == y[2]:
                    st["same_text"] += 1
                else:
                    st["differ"] += 1
                    result["differences"].append({"edition": ed, "book": b["book"], "ref": f"{y[0]}:{y[1]}",
                                                  "reviewer": x[2], "e3_python": y[2]})
                if rv is not None:
                    if rv[i] == [y[0], y[1], y[2]]:
                        st["rust_same"] += 1
                    else:
                        st["rust_differ"] += 1
                        result["differences"].append({"edition": ed, "book": b["book"], "ref": f"{y[0]}:{y[1]}",
                                                      "e3_rust": rv[i], "e3_python": y})
    result["editions"][ed] = st
tot = {k: sum(s[k] for s in result["editions"].values()) for k in
       ("verses", "same_text", "differ", "label_mismatch", "e3_problems", "reviewer_trouble", "rust_same", "rust_differ")}
result["totals"] = tot
json.dump(result, open(out, "w", encoding="utf-8"), ensure_ascii=False, indent=1)
print(json.dumps(tot))
for d in result["differences"][:20]:
    print(json.dumps(d, ensure_ascii=False)[:400])
