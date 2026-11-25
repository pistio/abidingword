#!/usr/bin/env python3
"""Reads the pinned harness's reports on the generated whole-edition documents
and checks them three ways:

 1. the scorer's own observe_report(), with the check's inventory and
    per-verse records computed here by reference_inventory.read() (so the
    scorer's verse-by-verse comparison of the Rust and Python readings runs);
 2. every verified mismatch attributed: the edition text in the record against
    the document's own words (from make_docs.py, the reviewer's extractor):
    equal -> the recogniser's reading of the document; different -> the
    reference reader;
 3. the reviewer's extractor's text for every reference verse against the
    harness's per-verse text digests (a third reading of all verses).

    assess_real.py <repo> <docs folder> <out.json>
"""
import hashlib, importlib.util, json, os, sys

repo, folder, out = sys.argv[1:4]
spec = importlib.util.spec_from_file_location("score_set", os.path.join(repo, "scripts", "evaluation", "score-set.py"))
ss = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ss)
ri = ss.reference_inventory
doc_text = json.load(open(os.path.join(folder, "document-verse-texts.json"), encoding="utf-8"))
res = {}
for key, ed in (("rv-engbsb", "engbsb"), ("rv-grcsbl", "grcsbl")):
    j = json.load(open(os.path.join(folder, "out", f"{key}.json"), encoding="utf-8"))
    inv = ri.read(os.path.join(folder, f"{key}_usfm.zip"))
    verses = inv.pop("verses")
    observed, integrity = ss.observe_report(j, inv, verses)
    attributed = []
    for r in j["fidelity_records"]:
        if r["status"] != "verified" or r["class"] != "mismatch":
            continue
        b, cv = r["reference"].split(" ")
        mine = doc_text.get(f"{ed} {b} {cv}")
        attributed.append({"reference": r["reference"], "kind": r.get("kind"),
                           "cause": "document reading (recogniser)" if mine == r["edition"] else "reference reader",
                           "edition": r["edition"], "found": r["found"]})
    # third reading: the reviewer's extractor text digest for every verse vs the harness per-verse records
    third_same = third_diff = 0
    for rec in j["reference_inventory"]["verses"]:
        member, book, chapter, label, status, line, digest = rec
        t = doc_text.get(f"{ed} {book} {chapter}:{label}")
        if t is not None and hashlib.sha256(t.encode("utf-8")).hexdigest() == digest:
            third_same += 1
        else:
            third_diff += 1
    t = j["totals"]
    res[key] = {"printed_scorable": t["expected"], "verified": t["verified"], "verified_exact": t["verified_exact"],
                "verified_canonical": t["verified_canonical"], "verified_layout": t["verified_layout"],
                "verified_mismatch": t["verified_mismatch"], "held": t["held"], "refused": t["refused"],
                "not_found": t["not_found"], "scorer_integrity_issues": integrity,
                "scorer_mismatches": observed["verified_mismatch"], "mismatches": attributed,
                "reference_verses_third_reading_same": third_same, "reference_verses_third_reading_differ": third_diff,
                "importer_comparison": observed["importer_comparison"], "unicode_version": j.get("unicode_version"),
                "reference_problems": j["reference_inventory"]["problems"]}
json.dump(res, open(out, "w", encoding="utf-8"), ensure_ascii=False, indent=1)
for k, v in res.items():
    print(k, {x: v[x] for x in v if x not in ("mismatches",)})
    for m in v["mismatches"]:
        print("   ", m)
