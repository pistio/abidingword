#!/usr/bin/env python3
"""Checks a withheld evaluation set before it is sealed and before it is run.

    check-set.py <seal folder> <documents folder> [--seal-commit=SHA]
                 [--inventory=FILE] [--report=FILE]

The builder runs it (through seal-set.py) before sealing; the runner
(`run-withheld-set.sh`) runs it with --seal-commit before reading anything.
Exit 0 only when every check passes; otherwise 2, and every failure is
listed. It reads only metadata, checksums and reference verse markers; it
never runs a recogniser. The scorer reuses its committed-seal check.

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

Paths: every name in the manifest, checksums.sha256 and SEAL.txt must be a
plain relative path (no absolute path, no '..', '.' or empty part, no
backslash), and no file or folder in the seal or documents folder may be a
symbolic link. Names are checked before any file is opened, and each file
must resolve inside its folder.

What this enforces, and what it cannot: see
docs/audit/candidate-a1.5/set-h-decision-package.md section 6.
"""
import hashlib
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
SUM_LINE = re.compile(r"^([0-9a-f]{64}) [ *](.+)$")
REF = re.compile(r"^([1-4A-Z][A-Z0-9]{2}) (\d+)(?::(\d+)(?:-(\d+))?)?$")

# The minimum of scored, authentic, Scripture-bearing evidence declared for
# Set H before any document is chosen (decision package section 3). Below it
# the set may still be run, but its result is reported with the stated
# qualification limit. Reported here and by the scorer; never an error.
# "reference_verses" counts verses of the references (printed verses in the
# run), not verified verses.
AUTHENTIC_MINIMUM = {"documents": 8, "reference_verses": 20000, "formats": 2, "languages": 4}


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def sha256_bytes(data):
    return hashlib.sha256(data).hexdigest()


# ---------------------------------------------------------------- paths

def path_problem(name):
    """Why `name` is not a plain relative path, or None."""
    if not name or name != name.strip():
        return "is empty or has surrounding spaces"
    if "\\" in name or any(ord(c) < 32 for c in name):
        return "contains a backslash or a control character"
    if name.startswith("/") or re.match(r"^[A-Za-z]:", name):
        return "is an absolute path"
    if any(part in ("", ".", "..") for part in name.split("/")):
        return "is not a plain relative path (it has an empty, '.' or '..' part)"
    return None


def inside(base, name, errors, label):
    """The path of `name` under `base`, or None with the reason recorded:
    `name` must be a plain relative path, pass through no symbolic link,
    and resolve inside `base`. Nothing is opened."""
    problem = path_problem(name)
    if problem:
        errors.append(f"{label}: '{name}' {problem}")
        return None
    current = base
    for part in name.split("/"):
        current = os.path.join(current, part)
        if os.path.islink(current):
            errors.append(f"{label}: '{name}' is or passes through a symbolic link")
            return None
    real_base = os.path.realpath(base)
    if os.path.commonpath([real_base, os.path.realpath(current)]) != real_base:
        errors.append(f"{label}: '{name}' resolves outside its folder")
        return None
    return current


def plain_files(base, errors, label):
    """Every regular file under `base`, as a relative name. Symbolic links
    (to files or folders) are refused and never followed."""
    found = []
    if os.path.islink(base):
        errors.append(f"{label}: the folder itself is a symbolic link")
        return found
    for root, dirs, files in os.walk(base, followlinks=False):
        for name in sorted(dirs + files):
            full = os.path.join(root, name)
            rel = os.path.relpath(full, base).replace(os.sep, "/")
            if os.path.islink(full):
                errors.append(f"{label}: '{rel}' is a symbolic link; links are not accepted")
            elif name in files and not os.path.isfile(full):
                errors.append(f"{label}: '{rel}' is not a regular file")
            elif name in files:
                found.append(rel)
    return sorted(found)


# ---------------------------------------------------------------- parsing

def parse_sums(text, label):
    sums = {}
    for n, line in enumerate(text.splitlines(), 1):
        if not line.strip():
            continue
        m = SUM_LINE.match(line)
        if not m:
            raise ValueError(f"{label} line {n} is not `sha256  name`")
        sums[m.group(2)] = m.group(1)
    return sums


def read_sums(path):
    return parse_sums(open(path, encoding="utf-8").read(), os.path.basename(path))


def manifest_rows_text(text, errors):
    lines = text.splitlines()
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


def manifest_rows(path, errors):
    return manifest_rows_text(open(path, encoding="utf-8").read(), errors)


def parse_allowed_text(text, label):
    """The references a mixed document may verify: a set of (book, chapter,
    verse) with verse None for a whole chapter. Raises on any line it does
    not understand."""
    allowed = set()
    for n, line in enumerate(text.splitlines(), 1):
        line = line.split("#", 1)[0].strip()
        if not line:
            continue
        m = REF.match(line)
        if not m:
            raise ValueError(f"{label} line {n}: '{line}' is not 'BOOK C', 'BOOK C:V' or 'BOOK C:V-W'")
        book, chapter, first, last = m.group(1), int(m.group(2)), m.group(3), m.group(4)
        if first is None:
            allowed.add((book, chapter, None))
            continue
        first = int(first)
        last = int(last) if last else first
        if last < first:
            raise ValueError(f"{label} line {n}: range runs backwards")
        for v in range(first, last + 1):
            allowed.add((book, chapter, v))
    if not allowed:
        raise ValueError(f"{label} lists no reference")
    return allowed


def parse_allowed(path):
    return parse_allowed_text(open(path, encoding="utf-8").read(), os.path.basename(path))


def parse_seal(text, errors):
    lines = text.splitlines()
    if not lines or not re.match(r"^sealed \d{4}-\d\d-\d\dT\d\d:\d\d:\d\dZ$", lines[0]):
        errors.append("SEAL.txt must start with 'sealed <UTC time>'")
    sealed = {}
    for line in lines[1:]:
        m = SUM_LINE.match(line)
        if not m:
            if line.strip():
                errors.append(f"SEAL.txt: '{line}' is not `sha256  name`")
            continue
        problem = path_problem(m.group(2))
        if problem:
            errors.append(f"SEAL.txt: '{m.group(2)}' {problem}")
            continue
        sealed[m.group(2)] = m.group(1)
    for rel in SEAL_FILES:
        if rel not in sealed:
            errors.append(f"SEAL.txt must list {rel}")
    return sealed


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


# ---------------------------------------------------------------- Git

def git(repo, *args):
    return subprocess.run(["git", "-C", repo, *args], capture_output=True, text=True)


def git_bytes(repo, *args):
    return subprocess.run(["git", "-C", repo, *args], capture_output=True)


def repo_of(folder):
    top = git(folder, "rev-parse", "--show-toplevel")
    return top.stdout.strip() if top.returncode == 0 else None


def repo_relative(repo, folder):
    return os.path.relpath(os.path.realpath(folder), os.path.realpath(repo)).replace(os.sep, "/")


def committed_seal(seal, commit):
    """The seal exactly as `commit` holds it, and every reason to distrust it:
    the commit must be in the checked-out history and change only the seal
    folder; its SEAL.txt must list every other file it holds, with matching
    SHA-256; the seal folder on disk must be byte for byte what it holds,
    with nothing added, removed or changed since (committed or not).
    Returns (errors, info); info["files"] maps each name to its bytes at
    the commit, so callers read the sealed metadata from the commit itself.
    Used before the run (check-set) and at assessment (score-set)."""
    errors = []
    repo = repo_of(seal)
    if not repo:
        return ["the seal folder is not in a Git repository"], None
    rel_seal = repo_relative(repo, seal)
    full = git(repo, "rev-parse", "--verify", "--quiet", commit + "^{commit}")
    if full.returncode != 0 or not full.stdout.strip():
        return [f"seal commit {commit} not found"], None
    c = full.stdout.strip()
    if git(repo, "merge-base", "--is-ancestor", c, "HEAD").returncode != 0:
        errors.append("the seal commit is not in the checked-out history")
    changed = git(repo, "diff-tree", "--no-commit-id", "--name-only", "-r", "--root", c).stdout.split()
    outside = [f for f in changed if not f.startswith(rel_seal + "/")]
    if not changed or outside:
        errors.append(f"the seal commit must change only {rel_seal}/ (it changes {', '.join(outside) or 'nothing'})")
    files = {}
    for line in git(repo, "ls-tree", "-r", "--full-tree", c, "--", rel_seal + "/").stdout.splitlines():
        meta, path = line.split("\t", 1)
        mode = meta.split()[0]
        name = path[len(rel_seal) + 1:]
        if mode == "120000":
            errors.append(f"seal commit: '{name}' is a symbolic link; links are not accepted")
            continue
        files[name] = git_bytes(repo, "cat-file", "blob", meta.split()[2]).stdout
    sealed = {}
    if "SEAL.txt" not in files:
        errors.append("SEAL.txt is not in the seal commit")
    else:
        sealed = parse_seal(files["SEAL.txt"].decode("utf-8", "replace"), errors)
        for name in sorted(set(files) - set(sealed) - {"SEAL.txt"}):
            errors.append(f"{name} is in the seal commit but not in its SEAL.txt")
        for name, want in sorted(sealed.items()):
            if name not in files:
                errors.append(f"SEAL.txt lists {name}, which the seal commit does not hold")
            elif sha256_bytes(files[name]) != want:
                errors.append(f"{name} in the seal commit differs from its sealed SHA-256")
    on_disk = plain_files(seal, errors, "seal folder")
    for name in sorted(set(on_disk) | set(files)):
        if name not in files:
            errors.append(f"{name} is in the seal folder but not in the seal commit")
        elif name not in on_disk:
            errors.append(f"{name} is in the seal commit but missing from the seal folder")
        elif open(os.path.join(seal, *name.split("/")), "rb").read() != files[name]:
            errors.append(f"{name} differs from the seal commit")
    if git(repo, "diff", "--quiet", c, "HEAD", "--", rel_seal).returncode != 0:
        errors.append("the seal folder changed after the seal commit")
    if git(repo, "status", "--porcelain", "--", rel_seal).stdout.strip():
        errors.append("the seal folder has uncommitted changes")
    return errors, {"repo": repo, "commit": c, "seal_rel": rel_seal, "files": files, "sealed": sealed}


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


# ---------------------------------------------------------------- the check

def check(seal, docs, seal_commit=None, inventory=None):
    errors, notes = [], []
    report = {"seal_folder": seal, "documents_folder": docs, "seal_commit": seal_commit}

    def stop():
        report.update(errors=errors, notes=notes, ok=False)
        return report

    # 0. One run only: a set already started is never checked again.
    if os.path.lexists(os.path.join(docs, LOCK)):
        errors.append(f"{LOCK} exists in the documents folder: this set has already been run")
        return stop()

    # 1. The seal: every file listed with its SHA-256, nothing unlisted, no
    #    link, no name outside the folder.
    on_seal = plain_files(seal, errors, "seal folder")
    if "SEAL.txt" not in on_seal:
        errors.append("SEAL.txt missing")
        return stop()
    sealed = parse_seal(open(os.path.join(seal, "SEAL.txt"), encoding="utf-8").read(), errors)
    for rel in sorted(set(on_seal) - set(sealed) - {"SEAL.txt"}):
        errors.append(f"{rel} is in the seal folder but not in SEAL.txt")
    for rel, want in sorted(sealed.items()):
        p = inside(seal, rel, errors, "SEAL.txt")
        if p is None:
            continue
        if not os.path.isfile(p):
            errors.append(f"SEAL.txt lists {rel}, which is missing")
        elif sha256(p) != want:
            errors.append(f"{rel} differs from its sealed SHA-256")
    if errors:
        return stop()
    report["manifest_sha256"] = sealed["manifest.tsv"]
    report["checksums_sha256"] = sealed["checksums.sha256"]

    # 2. The manifest: names, classes, kinds and references consistent.
    rows = manifest_rows(os.path.join(seal, "manifest.tsv"), errors)
    keys = [r["key"] for r in rows]
    for k in sorted({k for k in keys if keys.count(k) > 1}):
        errors.append(f"key {k} appears more than once")
    for r in rows:
        k = r["key"]
        if not KEY.match(k):
            errors.append(f"{k}: a key is letters, digits, '.', '_' or '-'")
            continue
        r["file_list"] = [f.strip() for f in r["files"].split(",") if f.strip()]
        for name in r["file_list"] + ([r["reference"]] if r["reference"] != "-" else []):
            problem = path_problem(name)
            if problem:
                errors.append(f"{k}: '{name}' {problem}")
        if r["class"] not in CLASSES:
            errors.append(f"{k}: class '{r['class']}' is not one of {sorted(CLASSES)}")
        if r["kind"] not in KINDS:
            errors.append(f"{k}: kind '{r['kind']}' is not one of {sorted(KINDS)}")
        if r["class"] == "scored" and r["reference"] == "-":
            errors.append(f"{k}: a scored document needs its exact matching reference")
        if r["class"] in ("control", "mixed") and r["reference"] != "-":
            errors.append(f"{k}: a {r['class']} document has no reference text ('-')")
        allowed = f"allowed/{k}.txt"
        if r["class"] == "mixed":
            if allowed not in sealed:
                errors.append(f"{k}: a mixed document needs allowed/{k}.txt in the seal")
            else:
                try:
                    parse_allowed(os.path.join(seal, "allowed", k + ".txt"))
                except ValueError as e:
                    errors.append(f"{k}: {e}")
        elif allowed in sealed:
            errors.append(f"{k}: allowed/{k}.txt exists but the document is not mixed")
    rows = [r for r in rows if "file_list" in r]
    report["classes"] = {c: sum(r["class"] == c for r in rows) for c in sorted(CLASSES)}

    # 3. Documents: every name a plain relative path inside the folder, every
    #    file listed, present and as checksummed; nothing else; no link.
    try:
        sums = read_sums(os.path.join(seal, "checksums.sha256"))
    except ValueError as e:
        errors.append(str(e))
        sums = {}
    on_disk = plain_files(docs, errors, "documents folder")
    for rel in sorted(set(on_disk) - set(sums)):
        errors.append(f"documents folder holds {rel}, which checksums.sha256 does not list")
    paths = {}
    for rel, want in sorted(sums.items()):
        p = inside(docs, rel, errors, "checksums.sha256")
        if p is None:
            continue
        if not os.path.isfile(p):
            errors.append(f"checksums.sha256 lists {rel}, which is missing")
        elif sha256(p) != want:
            errors.append(f"{rel} differs from its checksum")
        else:
            paths[rel] = p
    named = set()
    for r in rows:
        files = r["file_list"]
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
        want = ",".join(sums.get(f, "?") for f in r["file_list"])
        if p["document_sha256"] not in ("", "-") and p["document_sha256"] != want:
            errors.append(f"{r['key']}: provenance document_sha256 is not the checksummed file's")
    for k in sorted(set(prov) - set(keys)):
        errors.append(f"provenance.tsv has a row for {k}, which the manifest does not list")

    # 5. Not a disclosed document: byte-identical files and USFM books.
    inventory = inventory or os.path.join(REPO_DEFAULT, "docs", "evaluation", "disclosed-documents.sha256")
    if not os.path.isfile(inventory):
        errors.append(f"disclosed-document inventory {inventory} missing")
    else:
        known = {line.split()[0] for line in open(inventory, encoding="utf-8") if HEX.match(line[:64])}
        report["inventory_sha256"] = sha256(inventory)
        references = {r["reference"] for r in rows}
        for rel, h in sorted(sums.items()):
            if h in known:
                errors.append(f"{rel} is byte-identical to a disclosed document")
            if rel in paths and rel in references:
                for name, data in usfm_members(paths[rel]):
                    if sha256_bytes(data) in known:
                        errors.append(f"{rel}: its {name} is byte-identical to a disclosed USFM book")

    # 6. Not a disclosed key (the harness would also run that edition's files).
    repo = repo_of(seal) if seal_commit else None
    repo = repo or (REPO_DEFAULT if os.path.isdir(os.path.join(REPO_DEFAULT, "docs", "corpus")) else None)
    if repo:
        clash = sorted(set(keys) & disclosed_keys(repo))
        if clash:
            errors.append(f"keys already used by disclosed editions: {', '.join(clash)}")
    else:
        notes.append("no repository: disclosed keys not checked (the runner checks them)")

    # 7. The committed seal (runner only): the seal folder is exactly what one
    #    commit added, unchanged since; and no run record exists yet.
    if seal_commit:
        seal_errors, info = committed_seal(seal, seal_commit)
        errors.extend(seal_errors)
        if info:
            report["seal_commit"] = info["commit"]
            run_dir = f"{os.path.dirname(info['seal_rel'])}/run"
            if os.path.lexists(os.path.join(info["repo"], run_dir)) or \
                    git(info["repo"], "cat-file", "-e", f"HEAD:{run_dir}").returncode == 0:
                errors.append(f"a run record already exists at {run_dir}: this set has been run")

    # 8. Scored authentic evidence, against the declared minimum.
    auth = [r for r in rows if r["class"] == "scored" and r["kind"] == "authentic"]
    verses = 0
    for r in auth:
        if r["reference"] in paths:
            verses += sum(verse_markers(data) for _, data in usfm_members(paths[r["reference"]]))
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
