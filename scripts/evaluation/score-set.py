#!/usr/bin/env python3
"""Scores one run of the harness on a sealed set, and assesses it
(Phase A1.5; evaluation rules E3).

    score-set.py <run folder> <seal folder> --run-record=COMMIT [--json=OUT] [--md=OUT]
    score-set.py <run folder> <seal folder> --run-record=COMMIT --diagnostic
                 [--protocol=FILE] [--frozen-binary=SHA256] [--inventory=FILE] [--json=OUT] [--md=OUT]

Any other option, or an option given twice, is refused (exit 2): an option
is never silently ignored (E3).

**Official assessment** takes no option that can change the protocol: the
protocol is the committed docs/evaluation/protocol-e3.json beside this
scorer (its frozen harness binary, ceilings, canonical inventory, required
composition, evidence floor, Unicode versions, rule versions and approved
tools), the scorer, its rule modules and the protocol must be committed and
unchanged, and every tool must be the one the protocol approves by SHA-256
(E3: a committed replacement is not an approved tool). --protocol,
--frozen-binary and --inventory are refused without --diagnostic. With
--diagnostic the score is labelled DIAGNOSTIC throughout: `mode` is
"diagnostic", the overrides are recorded, the verdict is always "not a
protocol run", and what the verdict would be under the overrides is
`diagnostic_verdict`, for rehearsals and tests only (E2, C1).

**Verdicts (E3, R2).** Safety and evidence are judged apart. The safety
verdict is the research gate (criteria 1-4). The evidence verdict is the
protocol's evidence floor: by default at least one verse verified with
faithful text in a completed scored document. The verdict is "pass" only
when both hold; "fail" when a criterion fails; "inconclusive" when the
criteria hold but the evidence is below the floor (a run that verifies
nothing can never pass); "not a protocol run" when the run is not one.
Every verdict line states the verified (faithful and mismatch), printed
scorable, unsupported, held, refused and missing (not found) counts.

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
   and checked again here (E2, E3):
   * the reference's verse texts as the harness read them (its own Rust
     reader) are compared, verse by verse, with the check's (the Python
     reader, recorded before the run): any difference in a label, a status,
     a line or a text digest makes the run not a protocol run (E3, R1);
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
import unicodedata

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

PROTOCOL = os.path.join(REPO, "docs", "evaluation", "protocol-e3.json")
EVALUATION_SCHEMA = "aw-eval-3"
usfm_text = reference_inventory.usfm_text
# The tools an official assessment rests on: committed, unchanged, and each
# the one the protocol approves by SHA-256 (`approved_tools`).
TOOL_FILES = ["scripts/evaluation/score-set.py", "scripts/evaluation/check-set.py",
              "scripts/evaluation/seal-set.py", "scripts/evaluation/run-withheld-set.sh",
              "scripts/evaluation/fidelity.py", "scripts/evaluation/reference_inventory.py",
              "scripts/evaluation/usfm_text.py", "docs/evaluation/usfm-books.tsv"]
OFFICIAL_FILES = TOOL_FILES + ["docs/evaluation/protocol-e3.json"]
# RUN.txt's record of the tools the runner used, by field.
RUN_TOOLS = {"runner": "scripts/evaluation/run-withheld-set.sh", "check_set": "scripts/evaluation/check-set.py",
             "reference_inventory": "scripts/evaluation/reference_inventory.py",
             "usfm_text": "scripts/evaluation/usfm_text.py"}
OVERRIDES = ("protocol", "frozen-binary", "inventory")
OPTIONS = ("run-record", "json", "md") + OVERRIDES
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


def unassigned(text):
    """Code points unassigned in this scorer's Unicode version."""
    return sorted({f"U+{ord(c):04X}" for c in text if unicodedata.category(c) == "Cn"})


def observe_report(j, check_inventory, check_verses=None):
    """The observations of a scored (or exploratory) document's report, and
    every reason the scorer does not trust them (`integrity`).
    `check_verses`: the check's per-verse reference records (E3)."""
    integrity = []
    for field, want in (("evaluation_schema", EVALUATION_SCHEMA), ("fidelity_rules", fidelity.RULES),
                        ("inventory_rules", reference_inventory.RULES), ("reader_rules", usfm_text.RULES)):
        if j.get(field) != want:
            integrity.append(f"{field} is {j.get(field)!r}, not {want!r}: not a report of the E3 harness")
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
    # A text beyond this scorer's Unicode version cannot be classed with
    # certainty here: distrusted, never relied on (E3).
    beyond = [r["reference"] for r in j["fidelity_records"] if unassigned(r["found"] + r["edition"])]
    if beyond:
        integrity.append(f"fidelity: {len(beyond)} record(s) hold code points unassigned in the scorer's Unicode "
                         f"{unicodedata.unidata_version} ({', '.join(beyond[:5])})")
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
        # E3 (R1): labels, statuses, lines and every verse's text digest, as
        # the two independent readers read them.
        diffs = reference_inventory.compare(check_inventory, inv, check_verses)
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
        "held": t["held"],
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
        "unicode_version": j.get("unicode_version"),
        "integrity": integrity,
        "peak_memory_mb": j.get("read_peak_memory_mb"),
        "seconds": j.get("read_seconds"),
    }
    return observed, integrity


def observe_control(c):
    integrity = []
    if c.get("evaluation_schema") != EVALUATION_SCHEMA:
        integrity.append(f"evaluation_schema is {c.get('evaluation_schema')!r}: not a report of the E3 harness")
    return {
        "detected": c.get("detected", 0),
        "verified": c.get("verified", 0),
        "verified_references": c.get("verified_references", []),
        "integrity": integrity,
        "peak_memory_mb": c.get("peak_memory_mb"),
        "seconds": c.get("seconds"),
    }, integrity


def check_verses(folder, inv):
    """The check's per-verse records of a reference, from the run folder
    (covered by the committed OUTPUTS.sha256), and why they cannot be used:
    the file must be the one CHECK.json names, with its SHA-256, and its
    records must make CHECK.json's text digest."""
    if not inv:
        return None, None
    name = inv.get("verses_file")
    if not name:
        return None, "CHECK.json holds no per-verse records of this reference (not an E3 check)"
    problems = []
    path = check_set.inside(folder, name, problems, "CHECK.json")
    if path is None or not os.path.isfile(path):
        return None, f"the check's per-verse records {name} are missing" + (f" ({problems[0]})" if problems else "")
    if check_set.sha256(path) != inv.get("verses_file_sha256"):
        return None, f"the check's per-verse records {name} are not the ones CHECK.json names"
    verses = json.load(open(path, encoding="utf-8"))
    if reference_inventory.text_digest(verses) != inv.get("text_digest"):
        return None, f"the check's per-verse records {name} do not make its text digest"
    return verses, None


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
        inv = check.get("reference_inventories", {}).get(key)
        verses, why = check_verses(folder, inv)
        observed, integrity = observe_report(json.load(open(report, encoding="utf-8")), inv, verses)
        if why:
            integrity.append(why)
        if row["class"] == "scored" and not observed.get("printed"):
            integrity.append("a scored document whose reference holds no printed scorable verse "
                             "(the seal must refuse it)")
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
               "verified_absent", "verified_unvalidatable", "held", "held_faithful", "held_mismatch", "refused",
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


EVIDENCE_MEASURES = ("minimum_verified_faithful", "minimum_documents_with_verified_faithful",
                     "minimum_verified_faithful_share_of_printed_scorable", "maximum_scored_documents_refused_whole")


def evidence_floor(protocol, scored, completed_scored, refused_whole):
    """The protocol's evidence floor (aw-evidence-1) against a run: whether
    the evidence suffices for the verdict pass, each measure with its floor,
    its observed value and whether it is met, and every reason it does not
    suffice. A protocol without a readable floor gives no pass."""
    floor = protocol.get("evidence_floor")
    measures, reasons = {}, []
    base = floor.get("minimum_verified_faithful") if isinstance(floor, dict) else None
    if not isinstance(base, int) or isinstance(base, bool) or base < 1:
        return False, measures, ["the protocol defines no readable evidence floor (minimum_verified_faithful, "
                                 "a whole number of at least 1): no verdict can be pass"]
    unknown = sorted(set(floor) - set(EVIDENCE_MEASURES) - {"note"})
    if unknown:
        reasons.append(f"the evidence floor names measures this scorer does not know ({', '.join(unknown)}): "
                       "no verdict can be pass")
    faithful, printed = scored.get("verified_faithful", 0), scored.get("printed", 0)
    observed = {
        "minimum_verified_faithful": faithful,
        "minimum_documents_with_verified_faithful":
            sum(1 for d in completed_scored if (d["observed"].get("verified_faithful") or 0) > 0),
        "minimum_verified_faithful_share_of_printed_scorable": round(faithful / printed, 6) if printed else 0.0,
        "maximum_scored_documents_refused_whole": refused_whole,
    }
    for name in EVIDENCE_MEASURES:
        want = floor.get(name)
        if want is None:
            continue
        number = isinstance(want, (int, float)) and not isinstance(want, bool)
        got = observed[name]
        met = number and (got <= want if name.startswith("maximum") else got >= want)
        measures[name] = {"floor": want, "observed": got, "met": bool(met)}
        if not met:
            reasons.append(f"{name.replace('_', ' ')}: {got} against the floor {want}" if number
                           else f"{name.replace('_', ' ')}: the floor {want!r} is not a number")
    return not reasons, measures, reasons


def tools_state(protocol, official, fields):
    """Whether the tools in use, and those RUN.txt says the runner used, are
    the ones the protocol approves by SHA-256 (E3)."""
    approved = protocol.get("approved_tools")
    if not isinstance(approved, dict):
        return False, False, {"approved_tools": "none in the protocol"}
    detail = {rel: {"in_use": official["files"].get(rel), "approved": approved.get(rel),
                    "match": approved.get(rel) is not None and official["files"].get(rel) == approved.get(rel)}
              for rel in TOOL_FILES}
    in_use = all(d["match"] for d in detail.values())
    run_used = all(fields.get(field) is not None and fields.get(field) == approved.get(rel)
                   for field, rel in RUN_TOOLS.items())
    return in_use, run_used, detail


def assess(docs, run, seal_report, protocol, protocol_path, inventory, official, check):
    f = run["fields"]
    completed = [d for d in docs if d["execution"] == "completed"]
    integrity = {d["key"]: d["integrity"] for d in completed if d.get("integrity")}
    required = protocol.get("required_composition")
    tools_in_use, tools_run, tools_detail = tools_state(protocol, official, f)
    unicode = protocol.get("unicode") if isinstance(protocol.get("unicode"), dict) else {}
    report_unicode = sorted({str(d["observed"].get("unicode_version")) for d in completed
                             if d.get("observed") and "unicode_version" in d["observed"]})
    inventories = check.get("reference_inventories")
    validity = {
        "anchored_and_intact": True,  # checked before scoring; anything else is refused
        "run_complete": "ended" in f,
        "every_document_has_a_record": all(d["exit"] is not None for d in docs),
        "seal_matches": f.get("manifest") == seal_report["manifest_sha256"]
                        and f.get("checksums") == seal_report["checksums_sha256"],
        "frozen_binary": f.get("binary") is not None and f.get("binary") == protocol.get("harness_binary_sha256"),
        "protocol_limits": f.get("limits") == protocol.get("limits"),
        "canonical_inventory": os.path.exists(inventory) and f.get("inventory") == check_set.sha256(inventory)
                               and f.get("inventory") == protocol.get("inventory_sha256"),
        "run_under_this_protocol": f.get("protocol") == check_set.sha256(protocol_path),
        "protocol_permits_withheld_execution": protocol.get("withheld_execution_permitted") is True,
        "composition_declared_and_matching": seal_report.get("declared_composition") is not None
                                             and seal_report["declared_composition"] == seal_report["composition"],
        "composition_required_by_protocol_and_met": required is not None
                                                    and seal_report.get("declared_composition") == required,
        "check_report_is_e3": isinstance(inventories, dict)
                              and all(i.get("rules") == reference_inventory.RULES for i in inventories.values()),
        "unicode_versions_as_protocol": unicodedata.unidata_version == unicode.get("scorer_python")
                                        and all(u == unicode.get("harness_normalisation") for u in report_unicode),
        "tools_are_the_approved_ones": tools_in_use,
        "run_used_the_approved_tools": tools_run,
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
    not_read = [d for d in by_class["scored"] if d["execution"] != "completed"]
    refused = [d["key"] for d in not_read if d["execution"] == "refused"]
    failed = [d["key"] for d in not_read if d["execution"] == "failure"]
    safety = "pass" if all(criteria.values()) else "fail"
    sufficient, measures, shortfall = evidence_floor(protocol, scored, done("scored"), len(refused))
    evidence_verdict = "sufficient" if sufficient else "insufficient"

    def verdict_of(valid):
        if not valid:
            return "not a protocol run"
        if safety == "fail":
            return "fail"
        return "pass" if sufficient else "inconclusive"
    verdict = verdict_of(all(validity.values()))

    printed = scored.get("printed", 0)
    unsupported = scored.get("printed_unsupported", 0)
    faithful = scored.get("verified_faithful", 0)
    counts = {
        "scored_documents": len(by_class["scored"]),
        "completed": len(done("scored")),
        "refused_whole": len(refused),
        "failed_to_run": len(failed),
        "printed_scorable": printed,
        "printed_unsupported": unsupported,
        "verified": scored.get("verified", 0),
        "verified_faithful": faithful,
        "verified_mismatch": scored.get("verified_mismatch", 0),
        "verified_absent_or_unvalidatable": scored.get("verified_absent", 0) + scored.get("verified_unvalidatable", 0),
        "held": scored.get("held_faithful", 0) + scored.get("held_mismatch", 0),
        "refused_verses": scored.get("refused", 0),
        "missing": scored.get("not_found", 0),
        "documents_verifying_nothing": sorted(d["key"] for d in done("scored") if not d["observed"].get("verified")),
    }

    def line(v):
        return (f"{v.upper()} (safety {safety}; evidence {evidence_verdict}): "
                f"verified {counts['verified']} ({counts['verified_faithful']} faithful, "
                f"{counts['verified_mismatch']} wrong text) of {counts['printed_scorable']} printed scorable verses; "
                f"{counts['verified_absent_or_unvalidatable']} verified references the edition lacks or cannot validate; "
                f"{counts['printed_unsupported']} printed unsupported (not scored); {counts['held']} held back; "
                f"{counts['refused_verses']} in refused chapters; {counts['missing']} missing (not found); "
                f"scored documents {counts['completed']} completed, {counts['refused_whole']} refused whole, "
                f"{counts['failed_to_run']} failed to run, of {counts['scored_documents']}")

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
    if not_read:
        claim += (f" Not in the coverage figure, and not recognition coverage: {len(refused)} scored "
                  f"document(s) refused whole{' (' + ', '.join(refused) + ')' if refused else ''} and "
                  f"{len(failed)} that failed to run{' (' + ', '.join(failed) + ')' if failed else ''}, "
                  f"of {len(by_class['scored'])} scored documents; no verse from them counts as verified.")
    if not sufficient:
        claim += (" The evidence is below the protocol's evidence floor (" + "; ".join(shortfall) + "): "
                  "the run cannot show that the recogniser works, whatever its safety.")
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
        "verdict_line": line(verdict),
        "safety_verdict": safety,
        "evidence_verdict": evidence_verdict,
        "evidence_floor": {"rules": protocol.get("evidence_rules"), "measures": measures, "shortfall": shortfall},
        "counts": counts,
        "run_validity": validity,
        "integrity": integrity,
        "criteria": criteria,
        "accuracy_claim": claim,
        "authentic_evidence": evidence,
        "qualification_limit": limit,
        "tools": tools_detail,
        "unicode": {"scorer": unicodedata.unidata_version, "harness_reports": report_unicode,
                    "protocol": unicode},
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
        "_verdict_of": verdict_of,
        "_line": line,
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
              f"Under them the verdict would be **{a['diagnostic_verdict']}**: {a['diagnostic_verdict_line']}.", ""]
    L += [f"## Verdict: **{a['verdict']}**", "",
          f"**{a['verdict_line']}.**", "",
          f"Safety (criteria 1-4): **{a['safety_verdict']}**. Evidence (the protocol's floor, "
          f"{a['evidence_floor']['rules']}): **{a['evidence_verdict']}**"
          + (f" ({'; '.join(a['evidence_floor']['shortfall'])})" if a['evidence_floor']['shortfall'] else "") + ".", "",
          f"Run record `{an['run_record_commit']}` ({an['run_record_path']}), seal `{an['seal_commit']}`, "
          f"protocol `{an['protocol_sha256']}`, scorer `{an['scorer_sha256'][:16]}…` "
          f"(rules {EVALUATION_SCHEMA}, {fidelity.RULES}, {reference_inventory.RULES}, {usfm_text.RULES}). "
          f"{out['owner_check']}", "",
          a["accuracy_claim"], ""]
    if a["qualification_limit"]:
        L += ["**Qualification limit:** " + a["qualification_limit"], ""]
    L += ["| Run validity | |", "|---|---|"] + [f"| {k} | {yes(v)} |" for k, v in a["run_validity"].items()]
    if a["integrity"]:
        L += ["", "**Reports the scorer does not trust:**", ""]
        L += [f"- `{k}`: {'; '.join(v)}" for k, v in a["integrity"].items()]
    L += ["", "| Criterion (safety) | Met |", "|---|---|"] + [f"| {k} | {yes(v)} |" for k, v in a["criteria"].items()]
    L += ["", "| Evidence floor | Floor | Observed | Met |", "|---|---:|---:|---|"] + \
         [f"| {k} | {m['floor']} | {m['observed']} | {yes(m['met'])} |" for k, m in a["evidence_floor"]["measures"].items()]
    c = a["counts"]
    L += ["", "| Counts (completed scored documents) | |", "|---|---:|"] + \
         [f"| {k} | {', '.join(v) or 'none' if isinstance(v, list) else v} |" for k, v in c.items()]
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
        L += ["", f"| Scored by {key} | Docs | Printed | Unsupported | Verified | Faithful | Mismatch | Absent | Unvalidatable | Held faithful | Held mismatch | Refused | Missing |",
              "|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|"]
        for k, v in g.items():
            L.append(f"| {k} | {v['documents']} | {v.get('printed', 0)} | {v.get('printed_unsupported', 0)} | "
                     f"{v.get('verified', 0)} | {v.get('verified_faithful', 0)} | {v.get('verified_mismatch', 0)} | "
                     f"{v.get('verified_absent', 0)} | {v.get('verified_unvalidatable', 0)} | {v.get('held_faithful', 0)} | "
                     f"{v.get('held_mismatch', 0)} | {v.get('refused', 0)} | {v.get('not_found', 0)} |")
    L += ["", "| Document | Class | Kind | Execution | Exit | Printed | Unsupported | Verified | Faithful | Mismatch | Absent | Unvalidatable | Held | Refused | Missing | Outside allowed | Peak MB |",
          "|---|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|"]
    for d in out["documents"]:
        o = d.get("observed") or {}
        ex = d["execution"] + (f" ({d['cause']})" if d.get("cause") else "") + (" [diagnostic kept]" if d.get("diagnostic") else "")
        judged = d["class"] == "scored"
        L.append(f"| `{d['key']}` | {d['class']} | {d['kind']} | {ex} | {d['exit']} | {o.get('printed', '')} | "
                 f"{o.get('printed_unsupported', '')} | {o.get('verified', '')} | {o.get('verified_faithful', '')} | "
                 f"{o.get('verified_mismatch', '') if judged else ''} | "
                 f"{o.get('verified_absent', '') if judged else ''} | "
                 f"{o.get('verified_unvalidatable', '') if judged else ''} | "
                 f"{o.get('held', '')} | {o.get('refused', '')} | {o.get('not_found', '')} | "
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
    args, opts, flags, bad = check_set.parse_options(sys.argv[1:], OPTIONS, ("diagnostic",))
    if bad:
        print(f"refused: unknown or repeated option(s) {', '.join(bad)}; options are "
              f"{', '.join('--' + o + '=...' for o in OPTIONS)} and --diagnostic", file=sys.stderr)
        sys.exit(2)
    if len(args) != 2:
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
    verdict_of, line = assessment.pop("_verdict_of"), assessment.pop("_line")
    if diagnostic:
        # What the verdict would be under the overrides. Whether this scorer
        # is itself committed is an official-assessment condition only.
        validity = {k: v for k, v in assessment["run_validity"].items() if k != "scorer_committed"}
        assessment["diagnostic_verdict"] = verdict_of(all(validity.values()))
        assessment["diagnostic_verdict_line"] = line(assessment["diagnostic_verdict"])
        assessment["diagnostic_ignores"] = ["scorer_committed"]
        assessment["verdict"] = "not a protocol run"
        assessment["verdict_line"] = line("not a protocol run")
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
                    "usfm_text_sha256": check_set.sha256(os.path.join(HERE, "usfm_text.py")),
                    "tools_commit": official["commit"],
                    "tools_committed_and_unchanged": official["committed_and_unchanged"],
                    "tools_are_the_approved_ones": assessment["run_validity"]["tools_are_the_approved_ones"]},
        "owner_check": (f"Before relying on this score, check with sha256sum that {os.path.relpath(protocol_path, REPO)} "
                        f"is {check_set.sha256(protocol_path)} and that this is the protocol you approved; its "
                        f"approved_tools then name every tool by SHA-256."),
        "rules": {"evaluation_schema": EVALUATION_SCHEMA, "fidelity": fidelity.RULES,
                  "reference_inventory": reference_inventory.RULES, "usfm_text": usfm_text.RULES,
                  "evidence": protocol.get("evidence_rules")},
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
