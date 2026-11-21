#!/usr/bin/env python3
"""Scores one run of the harness on a sealed set, and assesses it
(Phase A1.5; evaluation rules E2).

    score-set.py <run folder> <seal folder> --run-record=COMMIT [--json=OUT] [--md=OUT]
    score-set.py <run folder> <seal folder> --run-record=COMMIT --diagnostic
                 [--protocol=FILE] [--frozen-binary=SHA256] [--inventory=FILE] [--json=OUT] [--md=OUT]

**Official assessment** takes no option that can change the protocol: the
protocol is the committed docs/evaluation/protocol-e2.json beside this
scorer (its frozen harness binary, ceilings, canonical inventory, required
composition and rule versions), and the scorer, its rule modules and the
protocol must be committed and unchanged. --protocol, --frozen-binary and
--inventory are refused without --diagnostic. With --diagnostic the score
is labelled DIAGNOSTIC throughout: `mode` is "diagnostic", the overrides
are recorded, the verdict is always "not a protocol run", and what the
verdict would be under the overrides is `diagnostic_verdict`, for
rehearsals and tests only (E2, C1).

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

1. **Raw observations**, per document, exactly as the harness wrote them,
   and checked again here (E2):
   * every usable verse whose text is not byte-exact is classed again by
     this scorer's own implementation of the strict fidelity rule
     (fidelity.py): exact, canonical (NFC), layout (white-space runs) or
     mismatch. Every verified mismatch is wrong text, whatever its kind;
     typography, case, word boundaries and marks are never excused;
   * every verified reference is looked up again in the reference's
     complete inventory as check-set.py read it before the run (CHECK.json):
     one the edition does not have, or cannot validate, fails criterion 2,
     whether it is a verse, a whole chapter or a book;
   * the reference's printed content is accounted for in full: scorable,
     unsupported by class (never scored, never dropped), outside the
     declared books, and printed = verified + held + refused + not found.
   Nothing is reclassified away or dropped. A document whose process failed
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
                  the edition does not have or cannot validate are counted;
   * control      no Scripture: every verified reference is a failure;
   * mixed        Scripture quoted in other text: every verified reference
                  outside the sealed allowed/<key>.txt is a failure;
   * exploratory  no trustworthy matching reference: reported raw, never
                  part of the gate or of any accuracy claim.
   Only completed scored documents make the accuracy figures. The run itself
   must be a protocol run (the protocol's binary, ceilings, inventory and
   composition; E2 reports; harness and scorer in agreement; sound
   references; complete and intact); otherwise the verdict is "not a
   protocol run", whatever the counts.
"""
import collections
import importlib.util
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.normpath(os.path.join(HERE, "..", ".."))


def _load(name, filename):
    spec = importlib.util.spec_from_file_location(name, os.path.join(HERE, filename))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


check_set = _load("check_set", "check-set.py")
fidelity = _load("fidelity", "fidelity.py")
reference_inventory = check_set.reference_inventory

PROTOCOL = os.path.join(REPO, "docs", "evaluation", "protocol-e2.json")
EVALUATION_SCHEMA = "aw-eval-2"
# The files an official assessment rests on: committed and unchanged.
OFFICIAL_FILES = ["scripts/evaluation/score-set.py", "scripts/evaluation/check-set.py",
                  "scripts/evaluation/fidelity.py", "scripts/evaluation/reference_inventory.py",
                  "docs/evaluation/usfm-books.tsv", "docs/evaluation/protocol-e2.json"]
OVERRIDES = ("protocol", "frozen-binary", "inventory")
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
    return problems, {"run_text": run_text, "check_text": committed["CHECK.json"].decode("utf-8"),
                      "seal": info, "run_record": record, "run_record_path": rel_run}


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




def ref_key(text):
    m = re.match(r"^(\S+) (\d+):(\d+)$", text)
    return (m.group(1), int(m.group(2)), int(m.group(3))) if m else None


def observe_report(j, check_inventory):
    """The observations of a scored (or exploratory) document's report, and
    every reason the scorer does not trust them (`integrity`)."""
    integrity = []
    for field, want in (("evaluation_schema", EVALUATION_SCHEMA), ("fidelity_rules", fidelity.RULES),
                        ("inventory_rules", reference_inventory.RULES)):
        if j.get(field) != want:
            integrity.append(f"{field} is {j.get(field)!r}, not {want!r}: not a report of the E2 harness")
    if integrity:
        # Not an E2 report: nothing in it is read, nothing counted.
        empty = {k: 0 for k in SCORED_SUMS}
        empty.update(verified_references=[], verified_mismatch_kinds={}, unsupported={}, integrity=integrity)
        return empty, integrity
    t = j["totals"]
    inv = j["reference_inventory"]
    # 1. Fidelity: every non-exact usable verse classed again, independently.
    classes = {"verified": collections.Counter(), "held": collections.Counter()}
    kinds = collections.Counter()
    mismatches, disagreements = [], []
    for r in j["fidelity_records"]:
        c, k = fidelity.judge(r["edition"], r["found"], bool(r.get("matched")))
        if (c, k) != (r["class"], r.get("kind")):
            disagreements.append(f"{r['reference']}: harness {r['class']}/{r.get('kind')}, scorer {c}/{k}")
        classes[r["status"]][c] += 1
        if r["status"] == "verified" and c == "mismatch":
            kinds[k] += 1
            mismatches.append({"reference": r["reference"], "kind": k})
    v = classes["verified"]
    if v["canonical"] != t["verified_canonical"] or v["layout"] != t["verified_layout"] \
            or v["mismatch"] != t["verified_mismatch"]:
        disagreements.append("the harness's verified class totals are not its own records'")
    if classes["held"]["mismatch"] != t["held_mismatch"]:
        disagreements.append("the harness's held-back mismatch total is not its own records'")
    exact = t["verified"] - sum(v.values())
    if exact != t["verified_exact"] or exact < 0:
        disagreements.append("verified exact is not verified minus the non-exact records")
    if disagreements:
        integrity.append(f"fidelity: {len(disagreements)} disagreement(s) between harness and scorer")
    # 2. Accounting: nothing leaves the denominator unexplained.
    acc = t["expected"] == t["verified"] + t["held"] + t["refused"] + t["not_found"]
    unsupported = inv["unsupported"]
    if not acc or inv["printed_scorable"] != t["expected"] or not j["accounting"]["holds"]:
        integrity.append("accounting: printed scorable verses are not verified + held + refused + not found")
    # Every label read is in exactly one place (verses before any chapter
    # have no label and are counted on their own).
    placed = (inv["printed_scorable"] + inv["absent_in_edition"]
              + sum(c["printed"] + c["empty"] for c in unsupported.values())
              + inv["outside_declared_scope"]["printed"] + inv["outside_declared_scope"]["empty"])
    before = unsupported["verse_before_chapter"]["printed"] + inv["before_chapter_outside"]
    if placed != inv["labels"] + before:
        integrity.append(f"accounting: {inv['labels']} labels read, {placed - before} placed")
    verified_refs = len(j["verified_references"])
    if verified_refs != t["verified"] + len(j["verified_absent"]) + len(j["verified_unvalidatable"]) \
            + j["verified_empty_in_both"]:
        integrity.append("accounting: verified references are not verified + absent + unvalidatable + empty")
    if j["book_selection"]["rule"] == "file name":
        integrity.append("books chosen from a file name; a sealed set states them (C3)")
    if j["book_selection"].get("unknown_codes"):
        integrity.append(f"unknown book codes {j['book_selection']['unknown_codes']}")
    if inv["problems"]:
        integrity.append("reference problems: " + "; ".join(inv["problems"]))
    # 3. The reference read twice: the harness's inventory against the
    #    check's own (CHECK.json, committed before the run).
    absent_check = None
    if check_inventory is None:
        integrity.append("CHECK.json holds no inventory of this document's reference")
    else:
        diffs = reference_inventory.compare(check_inventory, inv)
        if diffs:
            integrity.append("reference inventories differ: " + "; ".join(diffs))
        # 4. Every verified reference, looked up again in the check's keys.
        keys = reference_inventory.expand(check_inventory["scorable_ranges"])
        absent_check = sorted(r for r in j["verified_references"] if ref_key(r) not in keys)
        harness = sorted({a["reference"] for a in j["verified_absent"] if a["reason"] != "empty_in_edition"}
                         | {a["reference"] for a in j["verified_unvalidatable"]})
        if absent_check != harness:
            integrity.append(f"verified references absent from the edition: harness {len(harness)}, "
                             f"scorer {len(absent_check)}")
    # Unsupported Scripture verified under another reference: a mismatch
    # whose words are those of a verse the recogniser cannot hold.
    unsupported_labels = set()
    for labels in (check_inventory or {}).get("unsupported_labels", {}).values():
        unsupported_labels.update(labels)
    misidentified = [f"{r['reference']} = {r['matched']}" for r in j["fidelity_records"]
                     if r["status"] == "verified" and r.get("matched") in unsupported_labels]
    absent = [a["reference"] for a in j["verified_absent"]]
    unvalidatable = [a["reference"] for a in j["verified_unvalidatable"]]
    faithful = t["verified_exact"] + t["verified_canonical"] + t["verified_layout"]
    printed_unsupported = inv["printed_unsupported"]
    observed = {
        "printed": t["expected"],
        "printed_unsupported": printed_unsupported,
        "unsupported": {k: c["printed"] for k, c in unsupported.items() if c["printed"] or c["empty"]},
        "unsupported_references": {k: c["references"] for k, c in unsupported.items() if c["references"]},
        "absent_in_edition": inv["absent_in_edition"],
        "outside_declared_books": inv["outside_declared_scope"]["references"],
        "verified": t["verified"],
        "verified_exact": t["verified_exact"],
        "verified_canonical": t["verified_canonical"],
        "verified_layout": t["verified_layout"],
        "verified_faithful": faithful,
        "verified_mismatch": len(mismatches),
        "verified_mismatch_kinds": dict(sorted(kinds.items())),
        "verified_mismatch_material": sum(n for k, n in kinds.items() if k in fidelity.MATERIAL),
        "verified_mismatch_references": mismatches,
        "verified_misidentified_unsupported": misidentified,
        "verified_absent": len(absent),
        "verified_absent_references": [f"{a['reference']} ({a['reason']})" for a in j["verified_absent"]],
        "verified_unvalidatable": len(unvalidatable),
        "verified_unvalidatable_references": [f"{a['reference']} ({a['reason']})"
                                              for a in j["verified_unvalidatable"]],
        "held_faithful": t["held_faithful"],
        "held_mismatch": t["held_mismatch"],
        "refused": t["refused"],
        "not_found": t["not_found"],
        "refused_chapters": len(j.get("refused_chapters", [])),
        "books_not_found": len(j.get("books_not_found", [])),
        "book_selection": j["book_selection"]["rule"],
        "importer_comparison": {k: j["importer_comparison"].get(k) for k in
                                ("agree", "only_importer", "only_inventory", "text_differs", "error")},
        "fidelity_disagreements": disagreements[:50],
        "integrity": integrity,
        "peak_memory_mb": j.get("read_peak_memory_mb"),
        "seconds": j.get("read_seconds"),
    }
    return observed, integrity


def observe_control(c):
    integrity = []
    if c.get("evaluation_schema") != EVALUATION_SCHEMA:
        integrity.append(f"evaluation_schema is {c.get('evaluation_schema')!r}: not a report of the E2 harness")
    return {
        "detected": c.get("detected", 0),
        "verified": c.get("verified", 0),
        "verified_references": c.get("verified_references", []),
        "integrity": integrity,
        "peak_memory_mb": c.get("peak_memory_mb"),
        "seconds": c.get("seconds"),
    }, integrity


def document(folder, sealed_files, row, run, check):
    key = row["key"]
    d = {k: row[k] for k in ("key", "files", "reference", "language", "script", "format", "kind", "class")}
    exit_code, wall = run["documents"].get(key, (None, None))
    d["exit"], d["wall_seconds"] = exit_code, wall
    report = os.path.join(folder, key + ".json")
    control = os.path.join(folder, key + ".control.json")
    rejected = os.path.join(folder, key + ".rejected.md")
    log_path = os.path.join(folder, key + ".log")
    log = open(log_path, encoding="utf-8", errors="replace").read()[-20000:] if os.path.exists(log_path) else ""
    observed, refusal, integrity = None, None, []
    if os.path.exists(report):
        observed, integrity = observe_report(json.load(open(report, encoding="utf-8")),
                                             check.get("reference_inventories", {}).get(key))
    elif os.path.exists(control):
        c = json.load(open(control, encoding="utf-8"))
        observed, integrity = observe_control(c)
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
        d["integrity"] = integrity
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


SCORED_SUMS = ["printed", "printed_unsupported", "verified", "verified_exact", "verified_canonical",
               "verified_layout", "verified_faithful", "verified_mismatch", "verified_mismatch_material",
               "verified_absent", "verified_unvalidatable", "held_faithful", "held_mismatch", "refused",
               "not_found", "refused_chapters"]


def scored_totals(docs):
    t = collections.Counter()
    kinds, unsupported = collections.Counter(), collections.Counter()
    for d in docs:
        t["documents"] += 1
        for k in SCORED_SUMS:
            t[k] += d["observed"].get(k, 0) or 0
        kinds.update(d["observed"].get("verified_mismatch_kinds", {}))
        unsupported.update(d["observed"].get("unsupported", {}))
    return dict(t, verified_mismatch_kinds=dict(sorted(kinds.items())),
                unsupported=dict(sorted(unsupported.items())))


def official_state():
    """Whether the files an official assessment rests on are committed and
    unchanged in this scorer's repository, and their identities."""
    files = {rel: check_set.sha256(os.path.join(REPO, rel)) if os.path.isfile(os.path.join(REPO, rel)) else None
             for rel in OFFICIAL_FILES}
    head = check_set.git(REPO, "rev-parse", "HEAD")
    status = check_set.git(REPO, "status", "--porcelain", "--", *OFFICIAL_FILES)
    tracked = check_set.git(REPO, "ls-files", "--error-unmatch", "--", *OFFICIAL_FILES)
    committed = (head.returncode == 0 and status.returncode == 0 and not status.stdout.strip()
                 and tracked.returncode == 0 and all(files.values()))
    return {"committed_and_unchanged": committed, "commit": head.stdout.strip() if head.returncode == 0 else None,
            "files": files}


def assess(docs, run, seal_report, protocol, protocol_path, inventory, official, check):
    f = run["fields"]
    completed = [d for d in docs if d["execution"] == "completed"]
    integrity = {d["key"]: d["integrity"] for d in completed if d.get("integrity")}
    required = protocol.get("required_composition")
    validity = {
        "anchored_and_intact": True,  # checked before scoring; anything else is refused
        "run_complete": "ended" in f,
        "every_document_has_a_record": all(d["exit"] is not None for d in docs),
        "seal_matches": f.get("manifest") == seal_report["manifest_sha256"]
                        and f.get("checksums") == seal_report["checksums_sha256"],
        "frozen_binary": f.get("binary") == protocol.get("harness_binary_sha256"),
        "protocol_limits": f.get("limits") == protocol.get("limits"),
        "canonical_inventory": os.path.exists(inventory) and f.get("inventory") == check_set.sha256(inventory)
                               and f.get("inventory") == protocol.get("inventory_sha256"),
        "run_under_this_protocol": f.get("protocol") == check_set.sha256(protocol_path),
        "composition_declared_and_matching": seal_report.get("declared_composition") is not None
                                             and seal_report["declared_composition"] == seal_report["composition"],
        "composition_required_by_protocol_and_met": required is not None
                                                    and seal_report.get("declared_composition") == required,
        "check_report_is_e2": check.get("reference_inventories") is not None,
        "reports_trusted": not integrity,
        "scorer_committed": official["committed_and_unchanged"],
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
        "1_no_verified_text_mismatch": scored.get("verified_mismatch", 0) == 0,
        "2_no_verified_reference_outside_or_unvalidatable": scored.get("verified_absent", 0) == 0
                                                             and scored.get("verified_unvalidatable", 0) == 0,
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
        "verified_faithful": sum(d["observed"]["verified_faithful"] for d in auth),
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
    unsupported = scored.get("printed_unsupported", 0)
    faithful = scored.get("verified_faithful", 0)
    coverage = round(100.0 * faithful / printed, 1) if printed else None
    coverage_all = round(100.0 * faithful / (printed + unsupported), 1) if printed + unsupported else None
    evidence["coverage_percent"] = (round(100.0 * evidence["verified_faithful"] / evidence["printed_verses"], 1)
                                    if evidence["printed_verses"] else None)
    kinds = scored.get("verified_mismatch_kinds", {})
    claim = (f"{scored.get('verified_mismatch', 0)} verified verses with wrong text under the strict fidelity rule"
             + (f" ({', '.join(f'{k} {n}' for k, n in kinds.items())})" if kinds else "")
             + f", {scored.get('verified_absent', 0)} verified references outside the edition and "
             f"{scored.get('verified_unvalidatable', 0)} the edition cannot validate, among "
             f"{scored.get('verified', 0)} verified verses of {scored.get('documents', 0)} completed scored "
             f"documents ({len(auth)} authentic). They print {printed} scorable verses and {unsupported} verses "
             f"the recogniser cannot hold (unsupported, by class: "
             f"{', '.join(f'{k} {n}' for k, n in scored.get('unsupported', {}).items()) or 'none'}). "
             f"Faithful coverage (exact, canonical or layout, verified / printed; no pass mark): "
             f"{coverage if coverage is not None else '-'} % of scorable verses, "
             f"{coverage_all if coverage_all is not None else '-'} % of all printed verses. "
             f"Exploratory documents are not part of this figure.")
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
        "integrity": integrity,
        "criteria": criteria,
        "accuracy_claim": claim,
        "authentic_evidence": evidence,
        "qualification_limit": limit,
        "execution": dict(collections.Counter(d["execution"] for d in docs)),
        "failure_causes": dict(collections.Counter(d["cause"] for d in failures)),
        "scored": dict(scored, coverage_percent=coverage, coverage_percent_of_all_printed=coverage_all,
                       documents_in_manifest=len(by_class["scored"]), refused_whole=refused, failed_to_run=failed),
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
    L = []
    if out["mode"] == "diagnostic":
        L += ["# DIAGNOSTIC SCORE: not an official assessment", "",
              f"Overrides: {', '.join(f'{k}={v}' for k, v in out['overrides'].items()) or 'none'}. "
              f"Under them the verdict would be **{a['diagnostic_verdict']}**.", ""]
    L += [f"## Verdict: **{a['verdict']}**", "",
          f"Run record `{an['run_record_commit']}` ({an['run_record_path']}), seal `{an['seal_commit']}`, "
          f"protocol `{an['protocol_sha256'][:16]}…`, scorer `{an['scorer_sha256'][:16]}…` "
          f"(rules {EVALUATION_SCHEMA}, {fidelity.RULES}, {reference_inventory.RULES}).", "",
          a["accuracy_claim"], ""]
    if a["qualification_limit"]:
        L += ["**Qualification limit:** " + a["qualification_limit"], ""]
    L += ["| Run validity | |", "|---|---|"] + [f"| {k} | {yes(v)} |" for k, v in a["run_validity"].items()]
    if a["integrity"]:
        L += ["", "**Reports the scorer does not trust:**", ""]
        L += [f"- `{k}`: {'; '.join(v)}" for k, v in a["integrity"].items()]
    L += ["", "| Criterion | Met |", "|---|---|"] + [f"| {k} | {yes(v)} |" for k, v in a["criteria"].items()]
    L += ["", "| Execution | Documents |", "|---|---:|"] + [f"| {k} | {v} |" for k, v in sorted(a["execution"].items())]
    L += [f"| failure: {k} | {v} |" for k, v in sorted(a["failure_causes"].items())]
    s = a["scored"]
    L += ["", "| Scored (completed, exact reference) | Value |", "|---|---:|"] + \
         [f"| {k} | {s.get(k, 0)} |" for k in ["documents"] + SCORED_SUMS]
    L += [f"| verified mismatch, {k} | {n} |" for k, n in s.get("verified_mismatch_kinds", {}).items()]
    L += [f"| printed unsupported, {k} | {n} |" for k, n in s.get("unsupported", {}).items()]
    m = a["mixed"]
    L += ["", "| Other classes | Documents | Completed | Verified | Of which failures |", "|---|---:|---:|---:|---:|",
          f"| control | {a['control']['documents']} | {a['control']['completed']} | {a['control']['verified']} | {a['control']['verified']} |",
          f"| mixed | {m['documents']} | {m['completed']} | {m['verified_allowed'] + m['verified_outside_allowed']} | {m['verified_outside_allowed']} |",
          f"| exploratory | {a['exploratory']['documents']} | {a['exploratory']['completed']} | raw, per document | not judged |"]
    for key, g in out["groups"].items():
        L += ["", f"| Scored by {key} | Docs | Printed | Unsupported | Faithful | Mismatch | Absent | Unvalidatable | Held faithful | Held mismatch |",
              "|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|"]
        for k, v in g.items():
            L.append(f"| {k} | {v['documents']} | {v.get('printed', 0)} | {v.get('printed_unsupported', 0)} | "
                     f"{v.get('verified_faithful', 0)} | {v.get('verified_mismatch', 0)} | {v.get('verified_absent', 0)} | "
                     f"{v.get('verified_unvalidatable', 0)} | {v.get('held_faithful', 0)} | {v.get('held_mismatch', 0)} |")
    L += ["", "| Document | Class | Kind | Execution | Exit | Printed | Unsupported | Verified | Faithful | Mismatch | Absent | Unvalidatable | Outside allowed | Peak MB |",
          "|---|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|"]
    for d in out["documents"]:
        o = d.get("observed") or {}
        ex = d["execution"] + (f" ({d['cause']})" if d.get("cause") else "") + (" [diagnostic kept]" if d.get("diagnostic") else "")
        judged = d["class"] == "scored"
        L.append(f"| `{d['key']}` | {d['class']} | {d['kind']} | {ex} | {d['exit']} | {o.get('printed', '')} | "
                 f"{o.get('printed_unsupported', '')} | {o.get('verified', '')} | {o.get('verified_faithful', '')} | "
                 f"{o.get('verified_mismatch', '') if judged else ''} | "
                 f"{o.get('verified_absent', '') if judged else ''} | "
                 f"{o.get('verified_unvalidatable', '') if judged else ''} | "
                 f"{len(d['outside_allowed']) if 'outside_allowed' in d else ''} | "
                 f"{round(o['peak_memory_mb']) if o.get('peak_memory_mb') else ''} |")
    wrong = [(d["key"], x) for d in out["documents"] if d["class"] == "scored"
             for x in (d.get("observed") or {}).get("verified_mismatch_references", [])]
    if wrong:
        L += ["", "### Verified verses with wrong text", ""]
        L += [f"- `{k}` {x['reference']}: {x['kind']}" for k, x in wrong]
    misidentified = [(d["key"], x) for d in out["documents"] if d["class"] == "scored"
                     for x in (d.get("observed") or {}).get("verified_misidentified_unsupported", [])]
    if misidentified:
        L += ["", "### Unsupported Scripture verified as another reference", ""]
        L += [f"- `{k}` {x}" for k, x in misidentified]
    absent = [(d["key"], x) for d in out["documents"] if d["class"] == "scored"
              for x in (d.get("observed") or {}).get("verified_absent_references", [])
              + (d.get("observed") or {}).get("verified_unvalidatable_references", [])]
    if absent:
        L += ["", "### Verified references the edition does not have or cannot validate", ""]
        L += [f"- `{k}` {x}" for k, x in absent]
    unsupported = [(d["key"], c, refs) for d in out["documents"] if d["class"] == "scored"
                   for c, refs in (d.get("observed") or {}).get("unsupported_references", {}).items()]
    if unsupported:
        L += ["", "### Reference content the recogniser cannot hold (counted, never scored)", ""]
        L += [f"- `{k}` {c}: {', '.join(refs[:20])}{', …' if len(refs) > 20 else ''} ({len(refs)})"
              for k, c, refs in unsupported]
    return "\n".join(L) + "\n"


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    opts = dict(a[2:].split("=", 1) for a in sys.argv[1:] if a.startswith("--") and "=" in a)
    flags = {a[2:] for a in sys.argv[1:] if a.startswith("--") and "=" not in a}
    if len(args) != 2 or flags - {"diagnostic"}:
        sys.exit(__doc__)
    diagnostic = "diagnostic" in flags
    overrides = {k: opts[k] for k in OVERRIDES if k in opts}
    if overrides and not diagnostic:
        print(f"refused: {', '.join('--' + k for k in overrides)} change(s) what a protocol run is; "
              f"an official assessment takes none. Use --diagnostic for a rehearsal or test score.",
              file=sys.stderr)
        sys.exit(2)
    folder, seal = args
    problems, info = anchors(folder, seal, opts.get("run-record"))
    if problems:
        for p in problems:
            print("refused:", p, file=sys.stderr)
        sys.exit(2)
    protocol_path = os.path.abspath(overrides.get("protocol", PROTOCOL))
    errors = []
    protocol = check_set.load_protocol(protocol_path, errors) or {}
    if "frozen-binary" in overrides:
        protocol["harness_binary_sha256"] = overrides["frozen-binary"]
    inventory = os.path.abspath(overrides.get("inventory", os.path.join(REPO, protocol.get("inventory", ""))))
    if "inventory" in overrides:
        protocol["inventory_sha256"] = check_set.sha256(inventory) if os.path.isfile(inventory) else None
    files = info["seal"]["files"]
    run = read_run(info["run_text"])
    seal_report = {"manifest_sha256": check_set.sha256_bytes(files["manifest.tsv"]),
                   "checksums_sha256": check_set.sha256_bytes(files["checksums.sha256"])}
    rows = check_set.manifest_rows_text(files["manifest.tsv"].decode("utf-8"), errors)
    for r in rows:
        if r["class"] == "mixed" and f"allowed/{r['key']}.txt" not in files:
            errors.append(f"{r['key']}: the seal commit holds no allowed/{r['key']}.txt")
    seal_report["composition"] = check_set.manifest_composition(rows)
    seal_report["declared_composition"] = (check_set.parse_composition(files["composition.tsv"].decode("utf-8"), [])
                                           if "composition.tsv" in files else None)
    if errors:
        print("refused:", "; ".join(errors), file=sys.stderr)
        sys.exit(2)
    try:
        check = json.loads(info["check_text"])
    except ValueError:
        check = {}
    docs = [document(folder, files, r, run, check) for r in rows]
    groups = {}
    for key in ("format", "language", "script", "kind"):
        g = collections.defaultdict(list)
        for d in docs:
            if d["class"] == "scored" and d["execution"] == "completed":
                g[d[key]].append(d)
        groups[key] = {k: scored_totals(v) for k, v in sorted(g.items())}
    official = official_state()
    assessment = assess(docs, run, seal_report, protocol, protocol_path, inventory, official, check)
    if diagnostic:
        # What the verdict would be under the overrides. Whether this scorer
        # is itself committed is an official-assessment condition only.
        validity = {k: v for k, v in assessment["run_validity"].items() if k != "scorer_committed"}
        criteria = assessment["criteria"]
        assessment["diagnostic_verdict"] = (("pass" if all(criteria.values()) else "fail")
                                            if all(validity.values()) else "not a protocol run")
        assessment["diagnostic_ignores"] = ["scorer_committed"]
        assessment["verdict"] = "not a protocol run"
    out = {
        "mode": "diagnostic" if diagnostic else "official",
        "overrides": overrides,
        "anchors": {"run_record_commit": info["run_record"], "run_record_path": info["run_record_path"],
                    "seal_commit": info["seal"]["commit"], "seal_path": info["seal"]["seal_rel"],
                    "protocol": protocol_path, "protocol_sha256": check_set.sha256(protocol_path),
                    "scorer_sha256": check_set.sha256(os.path.abspath(__file__)),
                    "check_set_sha256": check_set.sha256(os.path.join(HERE, "check-set.py")),
                    "fidelity_sha256": check_set.sha256(os.path.join(HERE, "fidelity.py")),
                    "reference_inventory_sha256": check_set.sha256(os.path.join(HERE, "reference_inventory.py")),
                    "tools_commit": official["commit"],
                    "tools_committed_and_unchanged": official["committed_and_unchanged"]},
        "rules": {"evaluation_schema": EVALUATION_SCHEMA, "fidelity": fidelity.RULES,
                  "reference_inventory": reference_inventory.RULES},
        "run": run["fields"],
        "assessment": assessment,
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
