#!/usr/bin/env python3
"""The 13 bundled references read three ways (evaluation candidate A1.5-E3
evidence, R1): the harness's Rust reader (a dump written by the
`evaluation_references` test with AW_EVAL_TEXT_DUMP), the check's Python
reader, and, as a third reading written by someone else, the independent E2
reviewer's extractor (`evaluation-e2-review/evidence/inventory/indep_usfm.py`).
Then the 528 verses the review found the app's reader spacing wrongly, as
the E3 readers read them.

    crosscheck_bundled.py <rust dump folder> <out.json>
"""
import glob
import hashlib
import importlib.util
import json
import os
import sys
import zipfile

ROOT = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), *[".."] * 6))
REVIEW = os.path.join(ROOT, "docs/audit/candidate-a1.5/evaluation-e2-review/evidence/inventory")


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m


def main():
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    dump, out_json = sys.argv[1:]
    usfm_text = load("usfm_text", os.path.join(ROOT, "scripts/evaluation/usfm_text.py"))
    indep = load("indep_usfm", os.path.join(REVIEW, "indep_usfm.py"))
    review = json.load(open(os.path.join(REVIEW, "note-space-differences.json"), encoding="utf-8"))
    out = {"editions": {}, "review_528": {}}
    for path in sorted(glob.glob(os.path.join(ROOT, "resources/scripture/*/*_usfm.zip"))):
        edition = os.path.basename(os.path.dirname(path))
        rust = json.load(open(os.path.join(dump, f"{edition}.rust.json"), encoding="utf-8"))
        c = {"archive_sha256": hashlib.sha256(open(path, "rb").read()).hexdigest(), "verses": 0,
             "rust_equals_python": 0, "python_equals_review_extractor": 0, "python_problems": 0,
             "rust_label_sequence_differs": 0, "review_extractor_label_sequence_differs": 0,
             "differences_from_review_extractor": []}
        texts = {}
        with zipfile.ZipFile(path) as z:
            for name in z.namelist():
                if not name.lower().endswith((".usfm", ".sfm")):
                    continue
                source = z.read(name).decode("utf-8")
                p = usfm_text.read_text(source)
                c["python_problems"] += len(p["problems"])
                mine = [[v["chapter"], v["label"], v["line"], usfm_text.text_sha256(v["text"]), v["alternate"]]
                        for v in p["verses"]]
                if mine != rust[name]["verses"]:
                    c["rust_label_sequence_differs"] += 1
                c["rust_equals_python"] += sum(a == b for a, b in zip(mine, rust[name]["verses"]))
                third = indep.extract(source)
                if [(v["chapter"], v["label"]) for v in p["verses"]] != [(a, b) for a, b, _ in third]:
                    c["review_extractor_label_sequence_differs"] += 1
                for v, (_, _, t) in zip(p["verses"], third):
                    c["verses"] += 1
                    texts[f"{p['book']} {v['chapter']}:{v['label']}"] = v["text"]
                    if v["text"] == t:
                        c["python_equals_review_extractor"] += 1
                    else:
                        i = next(k for k in range(min(len(t), len(v["text"]) + 1)) if v["text"][k:k + 1] != t[k:k + 1])
                        c["differences_from_review_extractor"].append({
                            "reference": f"{p['book']} {v['chapter']}:{v['label']}",
                            "e3": v["text"][max(0, i - 30):i + 10], "review_extractor": t[max(0, i - 30):i + 10],
                            "code_points": [f"U+{ord(x):04X}" for x in v["text"][i:i + 1]]})
        out["editions"][edition] = c
        if edition in review:
            rows = review[edition]["rows"]
            resolved = sum(1 for r in rows if r["independent_context"].strip() in texts[r["reference"]]
                           and r["app_reader_context"].strip() not in texts[r["reference"]])
            out["review_528"][edition] = {"rows": len(rows), "kinds_under_e2": review[edition]["kinds"],
                                          "read_as_the_edition_prints_them": resolved}
        print(edition, {k: v for k, v in c.items() if k != "differences_from_review_extractor"},
              len(c["differences_from_review_extractor"]), file=sys.stderr)
    out["totals"] = {k: sum(e[k] for e in out["editions"].values()) for k in
                     ("verses", "rust_equals_python", "python_equals_review_extractor", "python_problems")}
    out["totals"]["review_528_resolved"] = sum(e["read_as_the_edition_prints_them"] for e in out["review_528"].values())
    json.dump(out, open(out_json, "w", encoding="utf-8"), ensure_ascii=False, indent=1)
    print(out["totals"], file=sys.stderr)


if __name__ == "__main__":
    main()
