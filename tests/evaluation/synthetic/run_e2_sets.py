#!/usr/bin/env python3
"""Runs the SYNTHETIC E2 regression sets end to end, in a scratch Git
repository outside this one, with one version of the evaluation tools:

    run_e2_sets.py old <harness binary> <work folder> [set,set...]
    run_e2_sets.py new <harness binary> <work folder> [set,set...]

old  the A1.5 tools exactly as frozen (`git show 767e226:<path>`: check-set,
     seal-set, runner, scorer, the inventory and the corpus manifests)
new  the tools of this working tree (E2)

For each set: make_e2_sets.py writes the documents and the seal folder;
seal-set.py --write seals it (a refusal is recorded and the set stops
there); the seal is committed alone; the runner runs it once and commits its
own record; the scorer scores it against that record. The harness binary is
a local build, so the scorer is told its SHA-256: with the old tools by
--frozen-binary (A1.5 accepted that silently), with the new ones only in
--diagnostic mode, which labels the result as not an official assessment.

Writes <work>/<set>/: seal.log, run.log, score.json, score.md (or
seal-refused.txt), and copies of the run outputs. Prints one summary line
per set. Nothing here touches this repository.
"""
import hashlib
import json
import os
import shutil
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.normpath(os.path.join(HERE, "..", "..", ".."))
FROZEN_TOOLS_COMMIT = "767e22678eb7fccae53625df2c9bc435c4a6cd53"
OLD_FILES = ["scripts/evaluation/check-set.py", "scripts/evaluation/seal-set.py",
             "scripts/evaluation/run-withheld-set.sh", "scripts/evaluation/score-set.py",
             "docs/evaluation/disclosed-documents.sha256"]
NEW_FILES = OLD_FILES + ["scripts/evaluation/fidelity.py", "scripts/evaluation/reference_inventory.py",
                         "docs/evaluation/usfm-books.tsv", "docs/evaluation/protocol-e2.json"]


def sh(args, **kw):
    return subprocess.run(args, capture_output=True, text=True, **kw)


def git(repo, *args):
    r = sh(["git", "-C", repo, *args])
    if r.returncode:
        raise SystemExit(f"git {' '.join(args)}: {r.stderr}")
    return r.stdout.strip()


def copy_tools(mode, repo):
    files = OLD_FILES if mode == "old" else NEW_FILES
    corpus = sorted(f for f in os.listdir(os.path.join(ROOT, "docs", "corpus")) if f.endswith(".tsv"))
    for rel in files + [f"docs/corpus/{f}" for f in corpus]:
        dst = os.path.join(repo, rel)
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        if mode == "old":
            data = subprocess.run(["git", "-C", ROOT, "show", f"{FROZEN_TOOLS_COMMIT}:{rel}"],
                                  capture_output=True, check=True).stdout
        else:
            data = open(os.path.join(ROOT, rel), "rb").read()
        open(dst, "wb").write(data)
    os.chmod(os.path.join(repo, "scripts/evaluation/run-withheld-set.sh"), 0o755)


def rehearsal_protocol(repo, name, binary_sha, seal):
    """The official protocol with the local binary and the set's own declared
    composition: a rehearsal protocol, outside the repository's history, so
    the scorer can only use it in --diagnostic mode."""
    p = json.load(open(os.path.join(repo, "docs/evaluation/protocol-e2.json")))
    p["name"] = "SYNTHETIC rehearsal of " + p["name"]
    p["harness_binary_sha256"] = binary_sha
    lines = open(os.path.join(seal, "composition.tsv")).read().splitlines()[1:]
    p["required_composition"] = {k: int(v) for k, v in (line.split("\t") for line in lines)}
    path = os.path.join(os.path.dirname(repo), f"rehearsal-protocol-{name}.json")
    json.dump(p, open(path, "w"), indent=1)
    return path


def main():
    mode, binary, work = sys.argv[1], os.path.abspath(sys.argv[2]), os.path.abspath(sys.argv[3])
    chosen = sys.argv[4].split(",") if len(sys.argv) > 4 else None
    assert mode in ("old", "new")
    if os.path.exists(work):
        raise SystemExit(f"{work} exists")
    repo, docs_root = os.path.join(work, "repo"), os.path.join(work, "documents")
    os.makedirs(repo)
    git(repo, "init", "-q")
    git(repo, "config", "user.email", "synthetic@example.invalid")
    git(repo, "config", "user.name", "synthetic e2")
    copy_tools(mode, repo)
    binary_sha = hashlib.sha256(open(binary, "rb").read()).hexdigest()
    git(repo, "add", "-A")
    git(repo, "commit", "-qm", f"tools ({mode})")
    make = [sys.executable, os.path.join(HERE, "make_e2_sets.py"), os.path.join(repo, "eval"), docs_root]
    r = sh(make + ([",".join(chosen)] if chosen else []))
    if r.returncode:
        raise SystemExit(r.stderr)
    names = [line.split()[0] for line in r.stdout.splitlines()]
    tools = os.path.join(repo, "scripts", "evaluation")
    summary = {}
    for name in names:
        out = os.path.join(work, name)
        os.makedirs(out)
        seal = os.path.join(repo, "eval", name, "seal")
        docs = os.path.join(docs_root, name)
        shutil.copy(os.path.join(repo, "eval", name, "expected.json"), out)
        os.remove(os.path.join(repo, "eval", name, "expected.json"))
        protocol = rehearsal_protocol(repo, name, binary_sha, seal) if mode == "new" else None
        extra = [f"--protocol={protocol}"] if protocol else []
        r = sh([sys.executable, os.path.join(tools, "seal-set.py"), seal, docs, "--write", *extra])
        open(os.path.join(out, "seal.log"), "w").write(r.stdout + r.stderr)
        if r.returncode:
            refusals = [l for l in r.stderr.splitlines() if l.startswith("refused")]
            open(os.path.join(out, "seal-refused.txt"), "w").write("\n".join(refusals) + "\n")
            summary[name] = {"sealed": False, "refusals": refusals}
            shutil.rmtree(os.path.join(repo, "eval", name))
            print(f"{name}: seal refused: {refusals[:3]}")
            continue
        git(repo, "add", "--", f"eval/{name}/seal")
        git(repo, "commit", "-qm", f"seal {name}")
        seal_commit = git(repo, "rev-parse", "HEAD")
        run_out = os.path.join(work, "outputs", name)
        env = dict(os.environ)
        if protocol:
            env["AW_PROTOCOL"] = protocol
        r = sh(["bash", os.path.join(tools, "run-withheld-set.sh"), seal, seal_commit, docs, binary,
                binary_sha, run_out], env=env)
        open(os.path.join(out, "run.log"), "w").write(r.stdout + r.stderr)
        if r.returncode:
            summary[name] = {"sealed": True, "ran": False}
            print(f"{name}: run refused: {r.stderr[-400:]}")
            continue
        record = git(repo, "rev-parse", "HEAD")
        score = [sys.executable, os.path.join(tools, "score-set.py"), run_out, seal, f"--run-record={record}",
                 f"--json={os.path.join(out, 'score.json')}", f"--md={os.path.join(out, 'score.md')}"]
        score += ([f"--frozen-binary={binary_sha}"] if mode == "old"
                  else ["--diagnostic", f"--protocol={protocol}"])
        r = sh(score)
        open(os.path.join(out, "score.log"), "w").write(r.stderr)
        if r.returncode:
            summary[name] = {"sealed": True, "ran": True, "scored": False, "error": r.stderr[-800:]}
            print(f"{name}: score refused: {r.stderr[-400:]}")
            continue
        shutil.copytree(run_out, os.path.join(out, "run-outputs"))
        a = json.load(open(os.path.join(out, "score.json")))["assessment"]
        s = a["scored"]
        summary[name] = {"sealed": True, "ran": True, "scored": True,
                         "verdict": a["verdict"], "diagnostic_verdict": a.get("diagnostic_verdict"),
                         "criteria": a["criteria"], "scored_totals": s}
        if mode == "old":
            print(f"{name}: verdict {a['verdict']}; printed {s.get('printed')} verified {s.get('verified')} "
                  f"correct {s.get('verified_correct')} wrong {s.get('verified_wrong')} "
                  f"absent {s.get('verified_absent')}")
        else:
            print(f"{name}: verdict {a['verdict']} (diagnostic {a.get('diagnostic_verdict')}); printed "
                  f"{s.get('printed')} + unsupported {s.get('printed_unsupported')}; verified {s.get('verified')}: "
                  f"faithful {s.get('verified_faithful')} mismatch {s.get('verified_mismatch')} "
                  f"{s.get('verified_mismatch_kinds')}; absent {s.get('verified_absent')} "
                  f"unvalidatable {s.get('verified_unvalidatable')}")
    json.dump({"mode": mode, "binary_sha256": binary_sha, "sets": summary},
              open(os.path.join(work, "summary.json"), "w"), ensure_ascii=False, indent=1)
    shutil.copytree(os.path.join(repo, "eval"), os.path.join(work, "sealed-sets"))
    open(os.path.join(work, "scratch-repo-log.txt"), "w").write(git(repo, "log", "--format=%H %s") + "\n")


if __name__ == "__main__":
    main()
