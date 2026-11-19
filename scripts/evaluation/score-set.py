#!/usr/bin/env python3
"""Scores one run of the harness on a sealed set, and assesses it (Phase A1.5).

    score-set.py <run folder> <seal folder> [--json=OUT] [--md=OUT]
                 [--frozen-binary=SHA256] [--inventory=FILE]

Writes nothing into the run folder. Refuses (exit 2) to score a run whose
outputs differ from its OUTPUTS.sha256, or whose RUN.txt is not the run of
this seal.

Three layers, kept apart:

1. **Raw observations**, per document, exactly as the harness wrote them.
   Nothing is reclassified or dropped. A document whose process failed
   keeps whatever report it left under `diagnostic`, outside every total.
2. **Execution**, per document, from RUN.txt and the outputs:
   * completed   exit 0 and a report;
   * refused     exit 0 and the app's clean refusal of the whole document;
   * failure     anything else: timeout (124), memory (the reader's
                 allocation failed under the ceiling), aborted (134), killed
                 (137), crashed (other signal), panicked (101), no output
                 (exit 0 and nothing written), not run (no RUN.txt line).
3. **Assessment** against the protocol's criteria, by the manifest's class:
   * scored       an exact matching reference: wrong text and references
                  the edition does not have are counted;
   * control      no Scripture: every verified reference is a failure;
   * mixed        Scripture quoted in other text: every verified reference
                  outside the sealed allowed/<key>.txt is a failure;
   * exploratory  no trustworthy matching reference: reported raw, never
                  part of the gate or of any accuracy claim.
   Only completed scored documents make the accuracy figures. The run itself
   must be a protocol run (frozen binary, protocol ceilings, the canonical
   disclosed-document inventory, complete and intact); otherwise the verdict
   is "not a protocol run", whatever the counts.
"""
import collections
import importlib.util
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.normpath(os.path.join(HERE, "..", ".."))
_spec = importlib.util.spec_from_file_location("check_set", os.path.join(HERE, "check-set.py"))
check_set = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(check_set)

FROZEN_BINARY = "920f7e28e6c03f0c2fd7832bb78ce7c2077d6767541f0b86bb02f2fec1ed3a8e"
PROTOCOL_LIMITS = {"timeout_seconds": 1800, "memory_kb": 4194304}
INVENTORY = os.path.join(REPO, "docs", "evaluation", "disclosed-documents.sha256")
SUM_LINE = re.compile(r"^([0-9a-f]{64}) [ *](.+)$")


def verify_outputs(folder):
    """The run folder exactly as OUTPUTS.sha256 says: every file present and
    unchanged, nothing added."""
    problems = []
    path = os.path.join(folder, "OUTPUTS.sha256")
    if not os.path.exists(path):
        return ["OUTPUTS.sha256 missing"]
    listed = {}
    for line in open(path, encoding="utf-8").read().splitlines():
        m = SUM_LINE.match(line)
        if m:
            listed[os.path.normpath(m.group(2))] = m.group(1)
    present = set()
    for root, _, files in os.walk(folder):
        for f in files:
            rel = os.path.normpath(os.path.relpath(os.path.join(root, f), folder))
            if rel != "OUTPUTS.sha256":
                present.add(rel)
    for rel in sorted(present - set(listed)):
        problems.append(f"{rel} was added after the run")
    for rel, want in sorted(listed.items()):
        p = os.path.join(folder, rel)
        if not os.path.exists(p):
            problems.append(f"{rel} is missing")
        elif check_set.sha256(p) != want:
            problems.append(f"{rel} changed after the run")
    return problems


def read_run(folder):
    run = {"documents": {}, "fields": {}}
    path = os.path.join(folder, "RUN.txt")
    if not os.path.exists(path):
        return run
    for line in open(path, encoding="utf-8").read().splitlines():
        m = re.match(r"^(\S+) exit (\d+) seconds (\d+)$", line)
        if m:
            run["documents"][m.group(1)] = (int(m.group(2)), int(m.group(3)))
            continue
        m = re.match(r"^limits timeout_seconds (\d+) memory_kb (\d+)$", line)
        if m:
            run["fields"]["limits"] = {"timeout_seconds": int(m.group(1)), "memory_kb": int(m.group(2))}
            continue
        parts = line.split(" ", 1)
        if len(parts) == 2:
            run["fields"][parts[0]] = parts[1]
    return run


def failure_cause(code, log):
    if code == 124:
        return "timeout"
    if "memory allocation of" in log or "Cannot allocate memory" in log or "out of memory" in log.lower():
        return "memory"
    if code == 134:
        return "aborted"
    if code == 137:
        return "killed"
    if code == 101:
        return "panicked"
    if code > 128:
        return f"crashed (signal {code - 128})"
    return f"exit {code}"


def observe_report(j):
    t = j["totals"]
    wrong = [x for x in j["differences"]
             if x.get("verified") and x.get("kind") not in ("typography", "unicode")]
    typo = [x for x in j["differences"]
            if x.get("verified") and x.get("kind") in ("typography", "unicode")]
    return {
        "printed": t["expected"],
        "verified": t["verified"],
        "verified_wrong": len(wrong),
        "verified_wrong_references": [str(x["reference"]) for x in wrong],
        "verified_typography": len(typo),
        "verified_absent": len(j.get("verified_extra_verses", [])),
        "verified_absent_references": [str(x) for x in j.get("verified_extra_verses", [])],
        "verified_correct": t["verified"] - len(wrong),
        "refused_correct": t.get("unverified_clean", 0),
        "caught": t.get("unverified_caught", 0),
        "refused_chapters": len(j.get("refused_chapters", [])),
        "books_not_found": len(j.get("books_not_found", [])),
        "peak_memory_mb": j.get("read_peak_memory_mb"),
        "seconds": j.get("read_seconds"),
    }


def observe_control(c):
    return {
        "detected": c.get("detected", 0),
        "verified": c.get("verified", 0),
        "verified_references": c.get("verified_references", []),
        "peak_memory_mb": c.get("peak_memory_mb"),
        "seconds": c.get("seconds"),
    }


def ref_key(text):
    m = re.match(r"^(\S+) (\d+):(\d+)$", text)
    return (m.group(1), int(m.group(2)), int(m.group(3))) if m else None


def document(folder, seal, row, run):
    key = row["key"]
    d = {k: row[k] for k in ("key", "files", "reference", "language", "script", "format", "kind", "class")}
    exit_code, wall = run["documents"].get(key, (None, None))
    d["exit"], d["wall_seconds"] = exit_code, wall
    report = os.path.join(folder, key + ".json")
    control = os.path.join(folder, key + ".control.json")
    rejected = os.path.join(folder, key + ".rejected.md")
    log_path = os.path.join(folder, key + ".log")
    log = open(log_path, encoding="utf-8", errors="replace").read()[-20000:] if os.path.exists(log_path) else ""
    observed, refusal = None, None
    if os.path.exists(report):
        observed = observe_report(json.load(open(report, encoding="utf-8")))
    elif os.path.exists(control):
        c = json.load(open(control, encoding="utf-8"))
        observed = observe_control(c)
        refusal = c.get("refused_document")
    elif os.path.exists(rejected):
        refusal = open(rejected, encoding="utf-8").read().rsplit("The app refused the document: ", 1)[-1].strip()

    if exit_code is None:
        d["execution"], d["cause"] = "failure", "not run"
    elif exit_code != 0:
        d["execution"], d["cause"] = "failure", failure_cause(exit_code, log)
    elif refusal is not None:
        d["execution"], d["refusal"] = "refused", refusal
    elif observed is not None:
        d["execution"] = "completed"
    else:
        d["execution"], d["cause"] = "failure", "no output"

    if d["execution"] == "completed":
        d["observed"] = observed
    elif d["execution"] == "failure" and observed is not None:
        d["diagnostic"] = observed  # kept, never counted
    if d["execution"] == "completed" and row["class"] == "mixed":
        allowed = check_set.parse_allowed(os.path.join(seal, "allowed", key + ".txt"))
        inside, outside = [], []
        for r in observed["verified_references"]:
            k = ref_key(r)
            ok = k is not None and (k in allowed or (k[0], k[1], None) in allowed)
            (inside if ok else outside).append(r)
        d["allowed_verified"], d["outside_allowed"] = inside, outside
    return d


SCORED_SUMS = ["printed", "verified", "verified_correct", "verified_wrong", "verified_typography",
               "verified_absent", "refused_correct", "caught", "refused_chapters"]


def scored_totals(docs):
    t = collections.Counter()
    for d in docs:
        t["documents"] += 1
        for k in SCORED_SUMS:
            t[k] += d["observed"].get(k, 0) or 0
    return dict(t)


def assess(docs, run, seal_report, frozen_binary, inventory=INVENTORY):
    f = run["fields"]
    validity = {
        "outputs_intact": True,  # checked before scoring; a changed run is refused
        "run_complete": "ended" in f,
        "every_document_has_a_record": all(d["exit"] is not None for d in docs),
        "seal_matches": f.get("manifest") == seal_report["manifest_sha256"]
                        and f.get("checksums") == seal_report["checksums_sha256"],
        "frozen_binary": f.get("binary") == frozen_binary,
        "protocol_limits": f.get("limits") == PROTOCOL_LIMITS,
        "canonical_inventory": os.path.exists(inventory) and f.get("inventory") == check_set.sha256(inventory),
    }
    by_class = collections.defaultdict(list)
    for d in docs:
        by_class[d["class"]].append(d)

    def done(c):
        return [d for d in by_class[c] if d["execution"] == "completed"]

    scored = scored_totals(done("scored"))
    failures = [d for d in docs if d["execution"] == "failure"]
    control_verified = sum(d["observed"]["verified"] for d in done("control"))
    mixed_outside = sum(len(d["outside_allowed"]) for d in done("mixed"))
    criteria = {
        "1_no_wrong_verified_text": scored.get("verified_wrong", 0) == 0,
        "2_no_verified_reference_outside_the_edition": scored.get("verified_absent", 0) == 0,
        "3_no_verified_reference_in_controls_or_outside_allowed": control_verified == 0 and mixed_outside == 0,
        "4_every_document_completed_or_refused_cleanly": not failures,
    }
    valid = all(validity.values())
    verdict = ("pass" if all(criteria.values()) else "fail") if valid else "not a protocol run"

    auth = [d for d in done("scored") if d["kind"] == "authentic"]
    minimum = check_set.AUTHENTIC_MINIMUM
    evidence = {
        "documents": len(auth),
        "printed_verses": sum(d["observed"]["printed"] for d in auth),
        "verified_correct": sum(d["observed"]["verified_correct"] for d in auth),
        "formats": sorted({d["format"] for d in auth}),
        "languages": sorted({d["language"] for d in auth}),
        "minimum": minimum,
    }
    evidence["meets_minimum"] = (
        evidence["documents"] >= minimum["documents"]
        and evidence["printed_verses"] >= minimum["reference_verses"]
        and len(evidence["formats"]) >= minimum["formats"]
        and len(evidence["languages"]) >= minimum["languages"])
    claim = (f"{scored.get('verified_wrong', 0)} verified verses with wrong text and "
             f"{scored.get('verified_absent', 0)} verified references outside the edition, among "
             f"{scored.get('verified', 0)} verified verses of {scored.get('documents', 0)} completed scored "
             f"documents ({len(auth)} authentic). Exploratory documents are not part of this figure.")
    limit = None
    if not evidence["meets_minimum"]:
        limit = (f"Scored authentic evidence ({evidence['documents']} documents, {evidence['printed_verses']} "
                 f"printed verses, formats {evidence['formats'] or 'none'}, languages {evidence['languages'] or 'none'}) "
                 f"is below the declared minimum ({minimum['documents']} documents, {minimum['reference_verses']} "
                 f"verses, {minimum['formats']} formats, {minimum['languages']} languages). The result qualifies "
                 f"the candidate for the generated documents and for the authentic documents listed only; it makes "
                 f"no claim for authentic published files in general.")
    return {
        "verdict": verdict,
        "run_validity": validity,
        "criteria": criteria,
        "accuracy_claim": claim,
        "authentic_evidence": evidence,
        "qualification_limit": limit,
        "execution": dict(collections.Counter(d["execution"] for d in docs)),
        "failure_causes": dict(collections.Counter(d["cause"] for d in failures)),
        "scored": scored,
        "control": {"documents": len(by_class["control"]), "completed": len(done("control")),
                    "verified": control_verified},
        "mixed": {"documents": len(by_class["mixed"]), "completed": len(done("mixed")),
                  "verified_allowed": sum(len(d["allowed_verified"]) for d in done("mixed")),
                  "verified_outside_allowed": mixed_outside},
        "exploratory": {"documents": len(by_class["exploratory"]), "completed": len(done("exploratory")),
                        "note": "raw observations only; no matching reference, so nothing is judged"},
    }


def markdown(out):
    a = out["assessment"]

    def yes(b):
        return "yes" if b else "**no**"

    L = [f"## Verdict: **{a['verdict']}**", "", a["accuracy_claim"], ""]
    if a["qualification_limit"]:
        L += ["**Qualification limit:** " + a["qualification_limit"], ""]
    L += ["| Run validity | |", "|---|---|"] + [f"| {k} | {yes(v)} |" for k, v in a["run_validity"].items()]
    L += ["", "| Criterion | Met |", "|---|---|"] + [f"| {k} | {yes(v)} |" for k, v in a["criteria"].items()]
    L += ["", "| Execution | Documents |", "|---|---:|"] + [f"| {k} | {v} |" for k, v in sorted(a["execution"].items())]
    L += [f"| failure: {k} | {v} |" for k, v in sorted(a["failure_causes"].items())]
    s = a["scored"]
    L += ["", "| Scored (completed, exact reference) | Value |", "|---|---:|"] + \
         [f"| {k} | {s.get(k, 0)} |" for k in ["documents"] + SCORED_SUMS]
    m = a["mixed"]
    L += ["", "| Other classes | Documents | Completed | Verified | Of which failures |", "|---|---:|---:|---:|---:|",
          f"| control | {a['control']['documents']} | {a['control']['completed']} | {a['control']['verified']} | {a['control']['verified']} |",
          f"| mixed | {m['documents']} | {m['completed']} | {m['verified_allowed'] + m['verified_outside_allowed']} | {m['verified_outside_allowed']} |",
          f"| exploratory | {a['exploratory']['documents']} | {a['exploratory']['completed']} | raw, per document | not judged |"]
    for key, g in out["groups"].items():
        L += ["", f"| Scored by {key} | Docs | Printed | Correct | Wrong | Absent | Refused | Caught |",
              "|---|---:|---:|---:|---:|---:|---:|---:|"]
        for k, v in g.items():
            L.append(f"| {k} | {v['documents']} | {v.get('printed', 0)} | {v.get('verified_correct', 0)} | "
                     f"{v.get('verified_wrong', 0)} | {v.get('verified_absent', 0)} | {v.get('refused_correct', 0)} | {v.get('caught', 0)} |")
    L += ["", "| Document | Class | Kind | Execution | Exit | Printed | Verified | Wrong | Absent | Outside allowed | Peak MB |",
          "|---|---|---|---|---:|---:|---:|---:|---:|---:|---:|"]
    for d in out["documents"]:
        o = d.get("observed") or {}
        ex = d["execution"] + (f" ({d['cause']})" if d.get("cause") else "") + (" [diagnostic kept]" if d.get("diagnostic") else "")
        judged = d["class"] == "scored"
        L.append(f"| `{d['key']}` | {d['class']} | {d['kind']} | {ex} | {d['exit']} | {o.get('printed', '')} | "
                 f"{o.get('verified', '')} | {o.get('verified_wrong', '') if judged else ''} | "
                 f"{o.get('verified_absent', '') if judged else ''} | "
                 f"{len(d['outside_allowed']) if 'outside_allowed' in d else ''} | "
                 f"{round(o['peak_memory_mb']) if o.get('peak_memory_mb') else ''} |")
    return "\n".join(L) + "\n"


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    opts = dict(a[2:].split("=", 1) for a in sys.argv[1:] if a.startswith("--") and "=" in a)
    if len(args) != 2:
        sys.exit(__doc__)
    folder, seal = args
    problems = verify_outputs(folder)
    if problems:
        for p in problems:
            print("refused:", p, file=sys.stderr)
        sys.exit(2)
    run = read_run(folder)
    sealed = {}
    for line in open(os.path.join(seal, "SEAL.txt"), encoding="utf-8").read().splitlines()[1:]:
        m = SUM_LINE.match(line)
        if m:
            sealed[m.group(2)] = m.group(1)
    seal_report = {"manifest_sha256": sealed.get("manifest.tsv"), "checksums_sha256": sealed.get("checksums.sha256")}
    if check_set.sha256(os.path.join(seal, "manifest.tsv")) != seal_report["manifest_sha256"]:
        print("refused: the seal folder's manifest is not the sealed one", file=sys.stderr)
        sys.exit(2)
    if run["fields"].get("manifest") != seal_report["manifest_sha256"]:
        print("refused: RUN.txt is not a run of this seal's manifest", file=sys.stderr)
        sys.exit(2)
    errors = []
    rows = check_set.manifest_rows(os.path.join(seal, "manifest.tsv"), errors)
    if errors:
        print("refused:", "; ".join(errors), file=sys.stderr)
        sys.exit(2)
    docs = [document(folder, seal, r, run) for r in rows]
    groups = {}
    for key in ("format", "language", "script", "kind"):
        g = collections.defaultdict(list)
        for d in docs:
            if d["class"] == "scored" and d["execution"] == "completed":
                g[d[key]].append(d)
        groups[key] = {k: scored_totals(v) for k, v in sorted(g.items())}
    out = {
        "run": run["fields"],
        "assessment": assess(docs, run, seal_report, opts.get("frozen-binary", FROZEN_BINARY),
                             opts.get("inventory", INVENTORY)),
        "groups": groups,
        "documents": docs,
    }
    if opts.get("json"):
        json.dump(out, open(opts["json"], "w", encoding="utf-8"), ensure_ascii=False, indent=1)
    text = markdown(out)
    if opts.get("md"):
        open(opts["md"], "w", encoding="utf-8").write(text)
    print(text)


if __name__ == "__main__":
    main()
