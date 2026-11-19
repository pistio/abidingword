#!/usr/bin/env python3
"""Checks a withheld evaluation set before it is sealed and before it is run.

    check-set.py <seal folder> <documents folder> [--seal-commit=SHA]
                 [--inventory=FILE] [--report=FILE]

The builder runs it before sealing (without --seal-commit); the runner
(`run-withheld-set.sh`) runs it with --seal-commit before reading anything.
Exit 0 only when every check passes; otherwise 2, and every failure is
listed. It reads only metadata, checksums and reference verse markers; it
never runs a recogniser.

The seal folder (committed to the repository, one commit that changes
nothing else) holds:

  manifest.tsv       14 columns: key, files, reference, language, script,
                     format, layout, role, source and licence, books, sha256,
                     generator, kind (authentic | generated),
                     class (scored | control | mixed | exploratory)
  checksums.sha256   every file of the documents folder, `sha256sum` format,
                     names relative to that folder
  provenance.tsv     one row per key (columns in PROVENANCE below)
  allowed/<key>.txt  for each `mixed` document: the references it may
                     verify, one per line ("JHN 3:16", "PSA 23:1-6", "PSA 23")
  review-ledger.md   the ground-truth reviewer's signed pre-seal ledger
  access-records.md  who filled each role, and what each read
  SEAL.txt           "sealed <UTC>" and the SHA-256 of every other file here

The documents folder (outside Git) holds the documents and references only.

What this enforces, and what it cannot: see
docs/audit/candidate-a1.5/set-h-decision-package.md section 6.
"""
import hashlib
import io
import json
import os
import re
import subprocess
import sys
import zipfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO_DEFAULT = os.path.normpath(os.path.join(HERE, "..", ".."))

COLUMNS = ["key", "files", "reference", "language", "script", "format", "layout",
           "role", "source and licence", "books", "sha256", "generator", "kind", "class"]
CLASSES = {"scored", "control", "mixed", "exploratory"}
KINDS = {"authentic", "generated"}
PROVENANCE = ["key", "publisher", "edition", "revision", "retrieved_from", "retrieved_utc",
              "licence", "document_sha256", "reference_source", "reference_method",
              "identity_checked_by"]
SEAL_FILES = ["manifest.tsv", "checksums.sha256", "provenance.tsv", "review-ledger.md",
              "access-records.md"]
LOCK = ".run-started"
KEY = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]*$")
HEX = re.compile(r"^[0-9a-f]{64}$")
REF = re.compile(r"^([1-4A-Z][A-Z0-9]{2}) (\d+)(?::(\d+)(?:-(\d+))?)?$")

# The minimum of scored, authentic, Scripture-bearing evidence declared for
# Set H before any document is chosen (decision package section 3). Below it
# the set may still be run, but its result is reported with the stated
# qualification limit. Reported here and by the scorer; never an error.
AUTHENTIC_MINIMUM = {"documents": 8, "reference_verses": 20000, "formats": 2, "languages": 4}


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def read_sums(path):
    sums = {}
    for n, line in enumerate(open(path, encoding="utf-8").read().splitlines(), 1):
        if not line.strip():
            continue
        m = re.match(r"^([0-9a-f]{64}) [ *](.+)$", line)
        if not m:
            raise ValueError(f"{os.path.basename(path)} line {n} is not `sha256  name`")
        sums[m.group(2)] = m.group(1)
    return sums


def manifest_rows(path, errors):
    lines = open(path, encoding="utf-8").read().splitlines()
    header = lines[0].split("\t") if lines else []
    if header != COLUMNS:
        errors.append(f"manifest header must be exactly: {' | '.join(COLUMNS)}")
        return []
    rows = []
    for n, line in enumerate(lines[1:], 2):
        if not line.strip() or line.startswith("#"):
            continue
        c = line.split("\t")
        if len(c) != len(COLUMNS):
            errors.append(f"manifest line {n}: {len(c)} columns, {len(COLUMNS)} required")
            continue
        rows.append(dict(zip(COLUMNS, [x.strip() for x in c])))
    return rows


def parse_allowed(path):
    """The references a mixed document may verify: a set of (book, chapter,
    verse) with verse None for a whole chapter. Raises on any line it does
    not understand."""
    allowed = set()
    for n, line in enumerate(open(path, encoding="utf-8").read().splitlines(), 1):
        line = line.split("#", 1)[0].strip()
        if not line:
            continue
        m = REF.match(line)
        if not m:
            raise ValueError(f"{os.path.basename(path)} line {n}: '{line}' is not 'BOOK C', 'BOOK C:V' or 'BOOK C:V-W'")
        book, chapter, first, last = m.group(1), int(m.group(2)), m.group(3), m.group(4)
        if first is None:
            allowed.add((book, chapter, None))
            continue
        first = int(first)
        last = int(last) if last else first
        if last < first:
            raise ValueError(f"{os.path.basename(path)} line {n}: range runs backwards")
        for v in range(first, last + 1):
            allowed.add((book, chapter, v))
    if not allowed:
        raise ValueError(f"{os.path.basename(path)} lists no reference")
    return allowed


def usfm_members(path):
    """(name, bytes) of every USFM file inside a zip, or of the file itself."""
    if zipfile.is_zipfile(path):
        with zipfile.ZipFile(path) as z:
            for info in z.infolist():
                if info.filename.lower().endswith((".usfm", ".sfm")):
                    yield info.filename, z.read(info)
    elif path.lower().endswith((".usfm", ".sfm")):
        yield os.path.basename(path), open(path, "rb").read()


def verse_markers(data):
    return len(re.findall(rb"\\v\s+\d", data))


def git(repo, *args):
    return subprocess.run(["git", "-C", repo, *args], capture_output=True, text=True)


def disclosed_keys(repo):
    keys = set()
    corpus = os.path.join(repo, "docs", "corpus")
    if os.path.isdir(corpus):
        for name in os.listdir(corpus):
            if name.endswith(".tsv"):
                for line in open(os.path.join(corpus, name), encoding="utf-8").read().splitlines()[1:]:
                    if line.strip() and not line.startswith("#"):
                        keys.add(line.split("\t")[0].strip())
    return keys


def check(seal, docs, seal_commit=None, inventory=None):
    errors, notes = [], []
    report = {"seal_folder": seal, "documents_folder": docs, "seal_commit": seal_commit}

    # 0. One run only: a set already started is never checked again.
    if os.path.exists(os.path.join(docs, LOCK)):
        errors.append(f"{LOCK} exists in the documents folder: this set has already been run")
        report.update(errors=errors, ok=False)
        return report

    # 1. The seal: every file listed with its SHA-256, nothing unlisted.
    seal_txt = os.path.join(seal, "SEAL.txt")
    if not os.path.exists(seal_txt):
        errors.append("SEAL.txt missing")
        report.update(errors=errors, ok=False)
        return report
    seal_lines = open(seal_txt, encoding="utf-8").read().splitlines()
    if not seal_lines or not re.match(r"^sealed \d{4}-\d\d-\d\dT\d\d:\d\d:\d\dZ$", seal_lines[0]):
        errors.append("SEAL.txt must start with 'sealed <UTC time>'")
    sealed = {}
    for line in seal_lines[1:]:
        m = re.match(r"^([0-9a-f]{64}) [ *](.+)$", line)
        if m:
            sealed[m.group(2)] = m.group(1)
        elif line.strip():
            errors.append(f"SEAL.txt: '{line}' is not `sha256  name`")
    present = []
    for root, _, files in os.walk(seal):
        for f in files:
            rel = os.path.relpath(os.path.join(root, f), seal)
            if rel != "SEAL.txt":
                present.append(rel)
    for rel in sorted(set(present) - set(sealed)):
        errors.append(f"{rel} is in the seal folder but not in SEAL.txt")
    for rel, want in sorted(sealed.items()):
        p = os.path.join(seal, rel)
        if not os.path.exists(p):
            errors.append(f"SEAL.txt lists {rel}, which is missing")
        elif sha256(p) != want:
            errors.append(f"{rel} differs from its sealed SHA-256")
    for rel in SEAL_FILES:
        if rel not in sealed:
            errors.append(f"SEAL.txt must list {rel}")
    if errors:
        report.update(errors=errors, ok=False)
        return report
    report["manifest_sha256"] = sealed["manifest.tsv"]
    report["checksums_sha256"] = sealed["checksums.sha256"]

    # 2. The manifest: classes, kinds and references consistent.
    rows = manifest_rows(os.path.join(seal, "manifest.tsv"), errors)
    keys = [r["key"] for r in rows]
    for k in sorted({k for k in keys if keys.count(k) > 1}):
        errors.append(f"key {k} appears more than once")
    for r in rows:
        k = r["key"]
        if not KEY.match(k):
            errors.append(f"{k}: a key is letters, digits, '.', '_' or '-'")
        if r["class"] not in CLASSES:
            errors.append(f"{k}: class '{r['class']}' is not one of {sorted(CLASSES)}")
        if r["kind"] not in KINDS:
            errors.append(f"{k}: kind '{r['kind']}' is not one of {sorted(KINDS)}")
        if r["class"] == "scored" and r["reference"] == "-":
            errors.append(f"{k}: a scored document needs its exact matching reference")
        if r["class"] in ("control", "mixed") and r["reference"] != "-":
            errors.append(f"{k}: a {r['class']} document has no reference text ('-')")
        allowed = os.path.join(seal, "allowed", k + ".txt")
        if r["class"] == "mixed":
            if not os.path.exists(allowed):
                errors.append(f"{k}: a mixed document needs allowed/{k}.txt in the seal")
            else:
                try:
                    parse_allowed(allowed)
                except ValueError as e:
                    errors.append(f"{k}: {e}")
        elif os.path.exists(allowed):
            errors.append(f"{k}: allowed/{k}.txt exists but the document is not mixed")
    report["classes"] = {c: sum(r["class"] == c for r in rows) for c in sorted(CLASSES)}

    # 3. Documents: every file listed, present, and as checksummed; nothing else.
    try:
        sums = read_sums(os.path.join(seal, "checksums.sha256"))
    except ValueError as e:
        errors.append(str(e))
        sums = {}
    on_disk = []
    for root, _, files in os.walk(docs, followlinks=True):
        for f in files:
            on_disk.append(os.path.relpath(os.path.join(root, f), docs))
    for rel in sorted(set(on_disk) - set(sums)):
        errors.append(f"documents folder holds {rel}, which checksums.sha256 does not list")
    for rel, want in sorted(sums.items()):
        p = os.path.join(docs, rel)
        if not os.path.exists(p):
            errors.append(f"checksums.sha256 lists {rel}, which is missing")
        elif sha256(p) != want:
            errors.append(f"{rel} differs from its checksum")
    named = set()
    for r in rows:
        files = [f.strip() for f in r["files"].split(",") if f.strip()]
        named.update(files)
        if r["reference"] != "-":
            named.add(r["reference"])
        for f in files + ([r["reference"]] if r["reference"] != "-" else []):
            if f not in sums:
                errors.append(f"{r['key']}: {f} is not in checksums.sha256")
        if HEX.match(r["sha256"]) and len(files) == 1 and sums.get(files[0]) not in (None, r["sha256"]):
            errors.append(f"{r['key']}: the manifest's sha256 column disagrees with checksums.sha256")
    for rel in sorted(set(sums) - named):
        errors.append(f"{rel} is checksummed but no manifest row names it")

    # 4. Provenance: one complete row per key, its hashes the checksummed ones.
    prov = {}
    lines = open(os.path.join(seal, "provenance.tsv"), encoding="utf-8").read().splitlines()
    if not lines or lines[0].split("\t") != PROVENANCE:
        errors.append(f"provenance.tsv header must be exactly: {' | '.join(PROVENANCE)}")
    else:
        for line in lines[1:]:
            if line.strip():
                c = line.split("\t")
                if len(c) != len(PROVENANCE):
                    errors.append(f"provenance.tsv: a row has {len(c)} columns")
                    continue
                prov[c[0]] = dict(zip(PROVENANCE, [x.strip() for x in c]))
    for r in rows:
        p = prov.get(r["key"])
        if not p:
            errors.append(f"{r['key']}: no provenance row")
            continue
        needs = ["publisher", "edition", "revision", "retrieved_from", "retrieved_utc", "licence",
                 "document_sha256", "identity_checked_by"]
        if r["reference"] != "-":
            needs += ["reference_source", "reference_method"]
        for field in needs:
            if p[field] in ("", "-"):
                errors.append(f"{r['key']}: provenance '{field}' is empty")
        files = [f.strip() for f in r["files"].split(",") if f.strip()]
        want = ",".join(sums.get(f, "?") for f in files)
        if p["document_sha256"] not in ("", "-") and p["document_sha256"] != want:
            errors.append(f"{r['key']}: provenance document_sha256 is not the checksummed file's")
    for k in sorted(set(prov) - set(keys)):
        errors.append(f"provenance.tsv has a row for {k}, which the manifest does not list")

    # 5. Not a disclosed document: byte-identical files and USFM books.
    inventory = inventory or os.path.join(REPO_DEFAULT, "docs", "evaluation", "disclosed-documents.sha256")
    if not os.path.exists(inventory):
        errors.append(f"disclosed-document inventory {inventory} missing")
    else:
        known = {line.split()[0] for line in open(inventory, encoding="utf-8") if HEX.match(line[:64])}
        report["inventory_sha256"] = sha256(inventory)
        for rel, h in sorted(sums.items()):
            if h in known:
                errors.append(f"{rel} is byte-identical to a disclosed document")
            p = os.path.join(docs, rel)
            if os.path.exists(p) and rel in {r["reference"] for r in rows}:
                for name, data in usfm_members(p):
                    if hashlib.sha256(data).hexdigest() in known:
                        errors.append(f"{rel}: its {name} is byte-identical to a disclosed USFM book")

    # 6. Not a disclosed key (the harness would also run that edition's files).
    repo = None
    if seal_commit:
        top = git(seal, "rev-parse", "--show-toplevel")
        repo = top.stdout.strip() if top.returncode == 0 else None
    repo = repo or (REPO_DEFAULT if os.path.isdir(os.path.join(REPO_DEFAULT, "docs", "corpus")) else None)
    if repo:
        clash = sorted(set(keys) & disclosed_keys(repo))
        if clash:
            errors.append(f"keys already used by disclosed editions: {', '.join(clash)}")
    else:
        notes.append("no repository: disclosed keys not checked (the runner checks them)")

    # 7. The committed seal (runner only): the seal folder is exactly what one
    #    commit added, unchanged since, and no run record exists.
    if seal_commit:
        if not repo:
            errors.append("--seal-commit given but the seal folder is not in a Git repository")
        else:
            rel_seal = os.path.relpath(os.path.realpath(seal), os.path.realpath(repo))
            full = git(repo, "rev-parse", "--verify", seal_commit + "^{commit}")
            if full.returncode != 0:
                errors.append(f"seal commit {seal_commit} not found")
            else:
                c = full.stdout.strip()
                report["seal_commit"] = c
                if git(repo, "merge-base", "--is-ancestor", c, "HEAD").returncode != 0:
                    errors.append("the seal commit is not in the checked-out history")
                changed = git(repo, "diff-tree", "--no-commit-id", "--name-only", "-r", "--root", c).stdout.split()
                outside = [f for f in changed if not f.startswith(rel_seal + "/")]
                if not changed or outside:
                    errors.append(f"the seal commit must change only {rel_seal}/ (it changes {', '.join(outside) or 'nothing'})")
                for rel in ["SEAL.txt"] + sorted(sealed):
                    blob = subprocess.run(["git", "-C", repo, "show", f"{c}:{rel_seal}/{rel}"], capture_output=True)
                    if blob.returncode != 0:
                        errors.append(f"{rel} is not in the seal commit")
                    elif hashlib.sha256(blob.stdout).digest() != hashlib.sha256(open(os.path.join(seal, rel), "rb").read()).digest():
                        errors.append(f"{rel} differs from the seal commit")
                if git(repo, "diff", "--quiet", c, "HEAD", "--", rel_seal).returncode != 0:
                    errors.append("the seal folder changed after the seal commit")
                if git(repo, "status", "--porcelain", "--", rel_seal).stdout.strip():
                    errors.append("the seal folder has uncommitted changes")
                run_dir = os.path.join(os.path.dirname(rel_seal), "run")
                if os.path.exists(os.path.join(repo, run_dir)) or git(repo, "cat-file", "-e", f"HEAD:{run_dir}").returncode == 0:
                    errors.append(f"a run record already exists at {run_dir}: this set has been run")

    # 8. Scored authentic evidence, against the declared minimum.
    auth = [r for r in rows if r["class"] == "scored" and r["kind"] == "authentic"]
    verses = 0
    for r in auth:
        p = os.path.join(docs, r["reference"])
        if os.path.exists(p):
            verses += sum(verse_markers(data) for _, data in usfm_members(p))
    evidence = {
        "documents": len(auth),
        "reference_verses": verses,
        "formats": sorted({r["format"] for r in auth}),
        "languages": sorted({r["language"] for r in auth}),
    }
    evidence["minimum"] = AUTHENTIC_MINIMUM
    evidence["meets_minimum"] = (
        evidence["documents"] >= AUTHENTIC_MINIMUM["documents"]
        and verses >= AUTHENTIC_MINIMUM["reference_verses"]
        and len(evidence["formats"]) >= AUTHENTIC_MINIMUM["formats"]
        and len(evidence["languages"]) >= AUTHENTIC_MINIMUM["languages"]
    )
    report["authentic_scored_evidence"] = evidence
    if not evidence["meets_minimum"]:
        notes.append("scored authentic evidence is below the declared minimum: the result will carry the stated qualification limit")

    report.update(errors=errors, notes=notes, ok=not errors)
    return report


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    opts = dict(a[2:].split("=", 1) for a in sys.argv[1:] if a.startswith("--") and "=" in a)
    if len(args) != 2:
        sys.exit(__doc__)
    report = check(args[0], args[1], opts.get("seal-commit"), opts.get("inventory"))
    text = json.dumps(report, ensure_ascii=False, indent=1)
    if opts.get("report"):
        open(opts["report"], "w", encoding="utf-8").write(text + "\n")
    for e in report["errors"]:
        print("refused:", e, file=sys.stderr)
    for n in report.get("notes", []):
        print("note:", n, file=sys.stderr)
    print(text)
    sys.exit(0 if report["ok"] else 2)


if __name__ == "__main__":
    main()
