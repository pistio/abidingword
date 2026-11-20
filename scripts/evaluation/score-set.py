#!/usr/bin/env python3
"""Scores one run of the harness on a sealed set, and assesses it (Phase A1.5).

    score-set.py <run folder> <seal folder> --run-record=COMMIT
                 [--json=OUT] [--md=OUT] [--frozen-binary=SHA256] [--inventory=FILE]

Writes nothing into the run folder. Before reading any output it checks
two anchors, and refuses (exit 2) when either is missing or inconsistent:

* **The run record**: COMMIT must be the one commit, in the checked-out
  history, that added RUN.txt, OUTPUTS.sha256 and CHECK.json to
  <seal folder>/../run/ (the runner commits them when the run ends), and
  no later commit or uncommitted change may touch them. The run folder's
  three files must be byte for byte the committed ones, and every output
  must match the committed OUTPUTS.sha256, with nothing added.
* **The seal**: the seal commit RUN.txt names must be an ancestor of the
  run record and pass check-set.py's committed-seal check (it changes only
  the seal folder; its SEAL.txt lists every file it holds; the seal folder
  on disk is byte for byte that commit, unchanged since). The manifest and
  every allowed/<key>.txt are read from the seal commit, never from disk,
  and RUN.txt's manifest and checksums hashes must be the sealed ones.

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


def verify_outputs(folder, listing):
    """The run folder exactly as the committed OUTPUTS.sha256 (`listing`)
    says: every file present and unchanged, nothing added, no link, no name
    outside the folder."""
    problems = []
    listed = {}
    for line in listing.splitlines():
        m = SUM_LINE.match(line)
        if not m:
            if line.strip():
                problems.append(f"OUTPUTS.sha256: '{line}' is not `sha256  name`")
            continue
        name = m.group(2)[2:] if m.group(2).startswith("./") else m.group(2)
        listed[name] = m.group(1)
    present = set(check_set.plain_files(folder, problems, "run folder")) - {"OUTPUTS.sha256"}
    for rel in sorted(present - set(listed)):
        problems.append(f"{rel} was added after the run")
    for rel, want in sorted(listed.items()):
        p = check_set.inside(folder, rel, problems, "OUTPUTS.sha256")
        if p is None:
            continue
        if not os.path.isfile(p):
            problems.append(f"{rel} is missing")
        elif check_set.sha256(p) != want:
            problems.append(f"{rel} changed after the run")
    return problems


ANCHORS = ("RUN.txt", "OUTPUTS.sha256", "CHECK.json")


def anchors(folder, seal, run_record):
    """Checks the run record and the seal (see the module's notes). Returns
    (problems, info): info holds the committed RUN.txt text, the seal's
    files at its commit and the commits themselves."""
    problems = []
    repo = check_set.repo_of(seal)
    if not repo:
        return ["the seal folder is not in a Git repository"], None
    git = lambda *a: check_set.git(repo, *a)
    rel_seal = check_set.repo_relative(repo, seal)
    rel_run = f"{os.path.dirname(rel_seal)}/run"
    if not run_record:
        return ["--run-record=<commit> is required: the commit that recorded this run"], None
    full = git("rev-parse", "--verify", "--quiet", run_record + "^{commit}")
    if full.returncode != 0 or not full.stdout.strip():
        return [f"run record commit {run_record} not found"], None
    record = full.stdout.strip()
    if git("merge-base", "--is-ancestor", record, "HEAD").returncode != 0:
        problems.append("the run record commit is not in the checked-out history")
    paths = [f"{rel_run}/{a}" for a in ANCHORS]
    touching = git("log", "--format=%H", "HEAD", "--", *paths).stdout.split()
    if not touching:
        return problems + [f"no committed run record at {rel_run}/"], None
    if touching != [record]:
        if record not in touching:
            problems.append(f"{run_record} is not the commit that recorded this run ({touching[-1][:12]} is)")
        if len(touching) > 1:
            problems.append("the run record was changed after it was committed")
    if git("status", "--porcelain", "--", *paths).stdout.strip():
        problems.append("the run record has uncommitted changes")
    committed = {}
    for a in ANCHORS:
        blob = check_set.git_bytes(repo, "show", f"{record}:{rel_run}/{a}")
        if blob.returncode != 0:
            problems.append(f"the run record commit does not hold {rel_run}/{a}")
            continue
        committed[a] = blob.stdout
        local = os.path.join(folder, a)
        if os.path.islink(local) or not os.path.isfile(local):
            problems.append(f"{a} is missing from the run folder")
        elif open(local, "rb").read() != blob.stdout:
            problems.append(f"{a} in the run folder differs from the committed run record")
    if problems:
        return problems, None
    run_text = committed["RUN.txt"].decode("utf-8")
    fields = dict(line.split(" ", 1) for line in run_text.splitlines() if " " in line)
    seal_commit = fields.get("seal_commit", "").strip()
    if not seal_commit:
        return ["the committed RUN.txt names no seal commit"], None
    seal_problems, info = check_set.committed_seal(seal, seal_commit)
    problems += seal_problems
    if info is None:
        return problems, None
    if git("merge-base", "--is-ancestor", info["commit"], record).returncode != 0:
        problems.append("the seal commit is not an ancestor of the run record")
    files = info["files"]
    if fields.get("manifest") != check_set.sha256_bytes(files.get("manifest.tsv", b"")):
        problems.append("RUN.txt's manifest is not the sealed one")
    if fields.get("checksums") != check_set.sha256_bytes(files.get("checksums.sha256", b"")):
        problems.append("RUN.txt's checksums are not the sealed ones")
    problems += verify_outputs(folder, committed["OUTPUTS.sha256"].decode("utf-8"))
    return problems, {"run_text": run_text, "seal": info, "run_record": record,
                      "run_record_path": rel_run}


def read_run(text):
    run = {"documents": {}, "fields": {}}
    for line in text.splitlines():
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


def document(folder, sealed_files, row, run):
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
        allowed = check_set.parse_allowed_text(
            sealed_files[f"allowed/{key}.txt"].decode("utf-8"), f"allowed/{key}.txt")
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
        "anchored_and_intact": True,  # checked before scoring; anything else is refused
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
        "verified": sum(d["observed"]["verified"] for d in auth),
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
    printed = scored.get("printed", 0)
    coverage = round(100.0 * scored.get("verified_correct", 0) / printed, 1) if printed else None
    evidence["coverage_percent"] = (round(100.0 * evidence["verified_correct"] / evidence["printed_verses"], 1)
                                    if evidence["printed_verses"] else None)
    claim = (f"{scored.get('verified_wrong', 0)} verified verses with wrong text and "
             f"{scored.get('verified_absent', 0)} verified references outside the edition, among "
             f"{scored.get('verified', 0)} verified verses of {scored.get('documents', 0)} completed scored "
             f"documents ({len(auth)} authentic), which print {printed} verses: coverage "
             f"{coverage if coverage is not None else '-'} % (verified correct / printed; no pass mark). "
             f"Exploratory documents are not part of this figure.")
    # A clean refusal is safe, never coverage: say which scored documents
    # gave nothing, and why, beside the figure.
    not_read = [d for d in by_class["scored"] if d["execution"] != "completed"]
    refused = [d["key"] for d in not_read if d["execution"] == "refused"]
    failed = [d["key"] for d in not_read if d["execution"] == "failure"]
    if not_read:
        claim += (f" Not in the coverage figure, and not recognition coverage: {len(refused)} scored "
                  f"document(s) refused whole{' (' + ', '.join(refused) + ')' if refused else ''} and "
                  f"{len(failed)} that failed to run{' (' + ', '.join(failed) + ')' if failed else ''}, "
                  f"of {len(by_class['scored'])} scored documents; no verse from them counts as verified.")
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
        "scored": dict(scored, coverage_percent=coverage, documents_in_manifest=len(by_class["scored"]),
                       refused_whole=refused, failed_to_run=failed),
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

    an = out["anchors"]
    L = [f"## Verdict: **{a['verdict']}**", "",
         f"Run record `{an['run_record_commit']}` ({an['run_record_path']}), seal `{an['seal_commit']}`, "
         f"scorer `{an['scorer_sha256'][:16]}…`.", "", a["accuracy_claim"], ""]
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
    problems, info = anchors(folder, seal, opts.get("run-record"))
    if problems:
        for p in problems:
            print("refused:", p, file=sys.stderr)
        sys.exit(2)
    files = info["seal"]["files"]
    run = read_run(info["run_text"])
    seal_report = {"manifest_sha256": check_set.sha256_bytes(files["manifest.tsv"]),
                   "checksums_sha256": check_set.sha256_bytes(files["checksums.sha256"])}
    errors = []
    rows = check_set.manifest_rows_text(files["manifest.tsv"].decode("utf-8"), errors)
    for r in rows:
        if r["class"] == "mixed" and f"allowed/{r['key']}.txt" not in files:
            errors.append(f"{r['key']}: the seal commit holds no allowed/{r['key']}.txt")
    if errors:
        print("refused:", "; ".join(errors), file=sys.stderr)
        sys.exit(2)
    docs = [document(folder, files, r, run) for r in rows]
    groups = {}
    for key in ("format", "language", "script", "kind"):
        g = collections.defaultdict(list)
        for d in docs:
            if d["class"] == "scored" and d["execution"] == "completed":
                g[d[key]].append(d)
        groups[key] = {k: scored_totals(v) for k, v in sorted(g.items())}
    out = {
        "anchors": {"run_record_commit": info["run_record"], "run_record_path": info["run_record_path"],
                    "seal_commit": info["seal"]["commit"], "seal_path": info["seal"]["seal_rel"],
                    "scorer_sha256": check_set.sha256(os.path.abspath(__file__)),
                    "check_set_sha256": check_set.sha256(os.path.join(HERE, "check-set.py"))},
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
