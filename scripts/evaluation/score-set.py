#!/usr/bin/env python3
"""Scores one run of the harness on a set of documents (Phase A1.5).

    score-set.py <run folder> <manifest.tsv> [--json=OUT] [--md=OUT]

Reads every <key>.json (a document with a reference) and <key>.control.json
(a control) the harness wrote, and the run's RUN.txt when there is one.
Writes nothing into the run folder.

Measures, per document and grouped by format, language, script and kind
(the manifest's 13th column, "authentic" or "generated", when present):

* verified_wrong     verified verses whose text differs from the reference
                     (typography and Unicode normalisation aside: those are
                     listed separately and are not wrong words);
* verified_absent    verified references the edition does not have;
* verified_correct   verified, text equal;
* refused_correct    not verified, text equal (conservative refusals);
* caught             not verified, text different;
* control_verified   for a control (reference "-"): every verified
                     reference, each a failure unless the builder's
                     ground truth lists it;
* no_result          a document with no report: refused at reading, crashed,
                     timed out or killed (from RUN.txt and the log).

Nothing is reclassified here. Adjudication is a separate, later ledger.
"""
import collections
import json
import os
import re
import sys


def manifest(path):
    rows = []
    with open(path, encoding="utf-8") as f:
        lines = f.read().splitlines()
    for line in lines[1:]:
        if not line.strip() or line.startswith("#"):
            continue
        c = line.split("\t")
        if len(c) < 9:
            continue
        rows.append({
            "key": c[0], "files": c[1], "reference": c[2], "language": c[3],
            "script": c[4], "format": c[5], "role": c[7],
            "kind": c[12].strip() if len(c) > 12 and c[12].strip() else "unstated",
        })
    return rows


def run_status(folder):
    status = {}
    path = os.path.join(folder, "RUN.txt")
    if os.path.exists(path):
        for line in open(path, encoding="utf-8"):
            m = re.match(r"(\S+) exit (\d+) seconds (\d+)", line)
            if m:
                status[m.group(1)] = (int(m.group(2)), int(m.group(3)))
    return status


def score(folder, rows):
    status = run_status(folder)
    docs = []
    for r in rows:
        d = dict(r)
        exit_code, seconds = status.get(r["key"], (None, None))
        d["exit"], d["wall_seconds"] = exit_code, seconds
        report = os.path.join(folder, r["key"] + ".json")
        control = os.path.join(folder, r["key"] + ".control.json")
        if r["reference"] == "-":
            if os.path.exists(control):
                c = json.load(open(control, encoding="utf-8"))
                d.update(result="control", control_verified=c.get("verified", 0),
                         control_references=c.get("verified_references", []),
                         peak_memory_mb=c.get("peak_memory_mb"), seconds=c.get("seconds"))
            else:
                d.update(result="no_result")
        elif os.path.exists(report):
            j = json.load(open(report, encoding="utf-8"))
            t = j["totals"]
            wrong = [x for x in j["differences"]
                     if x.get("verified") and x.get("kind") not in ("typography", "unicode")]
            typo = [x for x in j["differences"]
                    if x.get("verified") and x.get("kind") in ("typography", "unicode")]
            d.update(
                result="scored",
                printed=t["expected"],
                verified=t["verified"],
                verified_wrong=len(wrong),
                verified_wrong_references=[str(x["reference"]) for x in wrong],
                verified_typography=len(typo),
                verified_absent=len(j.get("verified_extra_verses", [])),
                verified_absent_references=j.get("verified_extra_verses", []),
                verified_correct=t["verified"] - len(wrong),
                refused_correct=t.get("unverified_clean", 0),
                caught=t.get("unverified_caught", 0),
                refused_chapters=len(j.get("refused_chapters", [])),
                books_not_found=len(j.get("books_not_found", [])),
                peak_memory_mb=j.get("read_peak_memory_mb"),
                seconds=j.get("read_seconds"),
            )
        else:
            rejected = os.path.exists(os.path.join(folder, r["key"] + ".rejected.md"))
            d.update(result="refused_document" if rejected else "no_result")
        docs.append(d)
    return docs


SUMS = ["printed", "verified_correct", "verified_wrong", "verified_absent",
        "refused_correct", "caught", "control_verified"]


def totals(docs):
    t = collections.Counter()
    for d in docs:
        for k in SUMS:
            t[k] += d.get(k, 0) or 0
        t["documents"] += 1
        t[d["result"]] += 1
    mem = [d["peak_memory_mb"] for d in docs if d.get("peak_memory_mb")]
    t["peak_memory_mb_max"] = round(max(mem)) if mem else 0
    return dict(t)


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    opts = dict(a[2:].split("=", 1) for a in sys.argv[1:] if a.startswith("--") and "=" in a)
    folder, man = args
    docs = score(folder, manifest(man))
    groups = {}
    for key in ("format", "language", "script", "kind"):
        g = collections.defaultdict(list)
        for d in docs:
            g[d[key]].append(d)
        groups[key] = {k: totals(v) for k, v in sorted(g.items())}
    out = {"documents": docs, "totals": totals(docs), "groups": groups}
    if opts.get("json"):
        json.dump(out, open(opts["json"], "w", encoding="utf-8"), ensure_ascii=False, indent=1)
    t = out["totals"]
    lines = [
        "| Measure | Value |", "|---|---:|",
        f"| Documents | {t['documents']} |",
        f"| **Verified, wrong text** | **{t['verified_wrong']}** |",
        f"| **Verified, not in the edition** | **{t['verified_absent']}** |",
        f"| **Verified in controls** | **{t['control_verified']}** |",
        f"| Verified, correct | {t['verified_correct']} |",
        f"| Refused, correct | {t['refused_correct']} |",
        f"| Caught | {t['caught']} |",
        f"| No result (crash, timeout, killed) | {t.get('no_result', 0)} |",
        f"| Refused whole document | {t.get('refused_document', 0)} |",
        f"| Highest reading peak, MB | {t['peak_memory_mb_max']} |",
    ]
    for key, g in groups.items():
        lines += ["", f"| {key.capitalize()} | Docs | Correct | Wrong | Absent | Control verified | Refused | Caught | No result |",
                  "|---|---:|---:|---:|---:|---:|---:|---:|---:|"]
        for k, v in g.items():
            lines.append(f"| {k} | {v['documents']} | {v['verified_correct']} | {v['verified_wrong']} | "
                         f"{v['verified_absent']} | {v['control_verified']} | {v['refused_correct']} | "
                         f"{v['caught']} | {v.get('no_result', 0)} |")
    lines += ["", "| Document | Result | Correct | Wrong | Absent | Refused | Caught | Peak MB | Exit |",
              "|---|---|---:|---:|---:|---:|---:|---:|---|"]
    for d in docs:
        lines.append(f"| `{d['key']}` | {d['result']} | {d.get('verified_correct', '')} | "
                     f"{d.get('verified_wrong', d.get('control_verified', ''))} | {d.get('verified_absent', '')} | "
                     f"{d.get('refused_correct', '')} | {d.get('caught', '')} | "
                     f"{round(d['peak_memory_mb']) if d.get('peak_memory_mb') else ''} | {d.get('exit')} |")
    text = "\n".join(lines) + "\n"
    if opts.get("md"):
        open(opts["md"], "w", encoding="utf-8").write(text)
    print(text)


if __name__ == "__main__":
    main()
