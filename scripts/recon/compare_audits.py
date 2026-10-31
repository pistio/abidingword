#!/usr/bin/env python3
"""Compares two audit folders written by the real_document_editions harness.

    python3 scripts/recon/compare_audits.py <before> <after> "<title>" [--wrong]

Prints a Markdown table, one row per edition: printed verses, detected and
text-verified before/after, verified verses the edition does not have
(verified false positives) and verified verses whose text differs beyond
whitespace or typography ("wrong"), before/after. With --wrong, lists every
wrong or false-positive verified verse after, with what differs.
"""

import difflib
import glob
import json
import os
import sys


def load(folder):
    out = {}
    for path in sorted(glob.glob(os.path.join(folder, "*.json"))):
        data = json.load(open(path))
        t = data["totals"]
        wrong = [d for d in data["differences"]
                 if d.get("verified") and d.get("kind") not in ("typography", "unicode")]
        out[os.path.basename(path)[:-5]] = {
            "printed": t["expected"],
            "detected": t["usable"],
            "verified": t["verified"],
            "fp_verified": data.get("verified_extra_verses", []),
            "fp_detected": len(data.get("extra_verses", [])),
            "wrong": wrong,
        }
    return out


def pct(n, d):
    return f"{100 * n / d:.1f} %" if d else "—"


def main():
    before, after, title = load(sys.argv[1]), load(sys.argv[2]), sys.argv[3]
    print(f"## {title}\n")
    print("| Edition | Printed | Detected before | Detected after | Verified before | Verified after | Δ verified | Verified FPs before / after | Verified wrong text before / after |")
    print("|---|---|---|---|---|---|---|---|---|")
    tot = [0] * 9
    for key in sorted(set(before) | set(after)):
        b, a = before.get(key), after.get(key)
        if not b or not a:
            print(f"| {key} | not read on one side | | | | | | | |")
            continue
        row = [a["printed"], b["detected"], a["detected"], b["verified"], a["verified"],
               len(b["fp_verified"]), len(a["fp_verified"]), len(b["wrong"]), len(a["wrong"])]
        tot = [x + y for x, y in zip(tot, row)]
        print(f"| {key} | {row[0]} | {row[1]} | {row[2]} ({pct(row[2], row[0])}) | {row[3]} | {row[4]} ({pct(row[4], row[0])}) | {row[4] - row[3]:+d} | {row[5]} / {row[6]} | {row[7]} / {row[8]} |")
    print(f"| **Total** | {tot[0]} | {tot[1]} | {tot[2]} ({pct(tot[2], tot[0])}) | {tot[3]} | {tot[4]} ({pct(tot[4], tot[0])}) | {tot[4] - tot[3]:+d} | {tot[5]} / {tot[6]} | {tot[7]} / {tot[8]} |")
    if "--wrong" in sys.argv:
        print("\n### Every verified verse with wrong text or not in the edition, after\n")
        any_row = False
        for key, a in sorted(after.items()):
            for ref in a["fp_verified"]:
                any_row = True
                print(f"* {key} {ref}: verified, but the edition has no such verse")
            for d in a["wrong"]:
                any_row = True
                sm = difflib.SequenceMatcher(None, d["edition"], d["found"])
                ops = [(op, d["edition"][i1:i2], d["found"][j1:j2])
                       for op, i1, i2, j1, j2 in sm.get_opcodes() if op != "equal"][:2]
                what = "; ".join(f"{op} «{e[:60]}» → «{f[:60]}»" for op, e, f in ops)
                print(f"* {key} {d['reference']} ({d['kind']}, page {d['page']}): {what}")
        if not any_row:
            print("None.")


main()
