#!/usr/bin/env python3
"""Both E2 reference inventories on the real bundled eBible USFM references;
compared by the scorer's own compare(). Also counts the USFM features the
shared reader handles specially."""
import json, subprocess, sys, glob, os, re, zipfile, importlib.util, hashlib, collections
REPO = "/home/user/abidingword"; PROBE = sys.argv[1]
spec = importlib.util.spec_from_file_location("ri", REPO + "/scripts/evaluation/reference_inventory.py")
ri = importlib.util.module_from_spec(spec); spec.loader.exec_module(ri)
out = {}
for z in sorted(glob.glob(REPO + "/resources/scripture/*/*_usfm.zip")):
    name = os.path.basename(z)
    py = ri.read(z, None)
    rs = json.loads(subprocess.run([PROBE, "inventory", z], capture_output=True, text=True, check=True).stdout)
    diffs = ri.compare(py, rs["report"])
    rep = rs["report"]
    feats = collections.Counter()
    with zipfile.ZipFile(z) as zf:
        for i in zf.infolist():
            if not i.filename.lower().endswith((".usfm", ".sfm")): continue
            t = zf.read(i).decode("utf-8", "replace")
            for k, pat in {"tilde_nbsp": r"~", "optional_break //": r"//", "\\va": r"\\va ", "\\vp": r"\\vp ",
                           "\\ca": r"\\ca ", "\\cp": r"\\cp ", "\\fig": r"\\fig ", "\\qa": r"\\qa ",
                           "\\d": r"\\d\b", "milestone \\*": r"\\\*", "\\f": r"\\f ", "\\x": r"\\x ",
                           "\\fe": r"\\fe ", "\\esb": r"\\esb", "\\rq": r"\\rq ", "\\w": r"\\w ",
                           "uppercase marker": r"\\[A-Z][a-z]*\b", "\\v without space": r"\\v[0-9]",
                           "BOM": "\ufeff", "CR": "\r"}.items():
                feats[k] += len(re.findall(pat, t))
    texts = [e["text"] for e in rs["entries"]]
    out[name] = {
        "zip_sha256": hashlib.sha256(open(z, "rb").read()).hexdigest(),
        "agree": not diffs, "differences": diffs,
        "labels": rep["labels"], "printed_scorable": rep["printed_scorable"],
        "absent_in_edition": rep["absent_in_edition"], "printed_unsupported": rep["printed_unsupported"],
        "unsupported": {k: v["printed"] + v["empty"] for k, v in rep["unsupported"].items() if v["printed"] + v["empty"]},
        "unsupported_examples": {k: v["references"][:6] for k, v in rep["unsupported"].items() if v["references"]},
        "problems": rep["problems"], "members_not_nfc": sum(1 for m in rep["members"] if m.get("role") == "usfm" and not m.get("nfc")),
        "parsed_text_with_tilde": sum("~" in t for t in texts), "parsed_text_with_//": sum("//" in t for t in texts),
        "parsed_text_with_backslash": sum("\\" in t for t in texts), "parsed_text_with_pipe": sum("|" in t for t in texts),
        "usfm_features": {k: v for k, v in feats.items() if v},
    }
    print(name, "AGREE" if not diffs else "DIFFER", rep["labels"], rep["printed_scorable"], rep["absent_in_edition"],
          rep["printed_unsupported"], out[name]["unsupported"], diffs[:3])
json.dump(out, open("real-inventories.json", "w"), ensure_ascii=False, indent=1)
