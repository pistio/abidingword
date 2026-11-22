#!/usr/bin/env python3
"""Checks a withheld evaluation set before it is sealed and before it is run.

    check-set.py <seal folder> <documents folder> [--seal-commit=SHA]
                 [--inventory=FILE] [--protocol=FILE] [--report=FILE] [--verses-out=DIR]

Any other option is refused (exit 2), never ignored (E3).

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
                     class (scored | control | mixed | exploratory).
                     books (E2, C3): `all` or USFM codes (docs/evaluation/
                     usfm-books.tsv), never empty; `-` only for a document
                     with no reference. The harness never guesses books from
                     a file name for a sealed set.
  composition.tsv    (E2, C2) the declared totals: `measure<TAB>documents`,
                     one row each for scored, control, mixed, exploratory,
                     authentic and generated. They must be the manifest's,
                     and the protocol's required composition when it has one
  reference-sources.tsv
                     (E3) where each reference package comes from: key,
                     reference, publisher_archive (name or URL),
                     publisher_archive_sha256, member, member_sha256, role.
                     One `kept` row per member of the package (its SHA-256
                     must be the package member's), one `removed` row per
                     member of the publisher's archive left out of it
                     (keys.asc, *.css...). The publisher's archive hash is
                     recorded apart from the package's own (checksums.sha256)
  checksums.sha256   every file of the documents folder, `sha256sum` format,
                     names relative to that folder
  provenance.tsv     one row per key (columns in PROVENANCE below)
  allowed/<key>.txt  for each `mixed` document: the references it may
                     verify, one per line ("JHN 3:16", "PSA 23:1-6", "PSA 23")
  review-ledger.md   the ground-truth reviewer's signed pre-seal ledger
  access-records.md  who filled each role, and what each read
  SEAL.txt           "sealed <UTC>" and the SHA-256 of every other file here

The documents folder (outside Git) holds the documents and references only.

References (E2, C4 and C5): each must be a .zip of USFM books (.usfm, .sfm),
with nothing else but prose (.htm), or one .usfm/.sfm file; UTF-8; one \\id
per file. A verse map, OSIS, MorphGNT or any other member is refused. Each
scored or exploratory document's reference is read by the neutral inventory
(reference_inventory.py, with the check's own USFM text reader, E3) and the
report lists, before any run, its complete coverage: every scorable verse
key, every label in each unsupported class (verse 0, chapters above the
66-book counts, other books, unusual labels, duplicates) and every book
outside the declared scope, and the digest of every verse's text
(--verses-out writes the per-verse records). The scorer compares the
harness's own inventory and texts with this one, verse by verse.

E3 refuses, besides: a reference the text reader cannot read unambiguously
(an unclosed note or span, a mismatched end marker, an unknown marker...);
verse text with a code point unassigned in this check's Unicode version; and
(R2) a scored document whose reference holds no printed scorable verse in
its declared books (an empty or prose-only package, only unsupported or
empty verses), or a declared book the reference does not print. Each
reference's evidence class is reported before the seal.

Paths: every name in the manifest, checksums.sha256 and SEAL.txt must be a
plain relative path (no absolute path, no '..', '.' or empty part, no
backslash), and no file or folder in the seal or documents folder may be a
symbolic link. Names are checked before any file is opened, and each file
must resolve inside its folder.

What this enforces, and what it cannot: see the safeguards section of the
evaluation protocol.
"""
import hashlib
import importlib.util
import json
import os
import re
import subprocess
import sys
import zipfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO_DEFAULT = os.path.normpath(os.path.join(HERE, "..", ".."))
# Beside this script (a builder's flat copy of the tools), else in the repository.
PROTOCOL_DEFAULT = next((p for p in (os.path.join(HERE, "protocol-e3.json"),
                                     os.path.join(REPO_DEFAULT, "docs", "evaluation", "protocol-e3.json"))
                         if os.path.isfile(p)), os.path.join(REPO_DEFAULT, "docs", "evaluation", "protocol-e3.json"))
_spec = importlib.util.spec_from_file_location("reference_inventory", os.path.join(HERE, "reference_inventory.py"))
reference_inventory = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(reference_inventory)

COLUMNS = ["key", "files", "reference", "language", "script", "format", "layout",
           "role", "source and licence", "books", "sha256", "generator", "kind", "class"]
CLASSES = {"scored", "control", "mixed", "exploratory"}
KINDS = {"authentic", "generated"}
PROVENANCE = ["key", "publisher", "edition", "revision", "retrieved_from", "retrieved_utc",
              "licence", "document_sha256", "reference_source", "reference_method",
              "identity_checked_by"]
SEAL_FILES = ["manifest.tsv", "checksums.sha256", "provenance.tsv", "review-ledger.md",
              "access-records.md", "composition.tsv", "reference-sources.tsv"]
SOURCES = ["key", "reference", "publisher_archive", "publisher_archive_sha256", "member", "member_sha256", "role"]
OPTIONS = ("seal-commit", "inventory", "protocol", "report", "verses-out")
COMPOSITION = ["scored", "control", "mixed", "exploratory", "authentic", "generated"]
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


def parse_composition(text, errors):
    """{measure: documents} from composition.tsv; every measure once."""
    lines = [l for l in text.splitlines() if l.strip() and not l.startswith("#")]
    if not lines or lines[0].split("\t") != ["measure", "documents"]:
        errors.append("composition.tsv header must be exactly: measure | documents")
        return None
    out = {}
    for line in lines[1:]:
        c = line.split("\t")
        if len(c) != 2 or c[0].strip() not in COMPOSITION or not c[1].strip().isdigit():
            errors.append(f"composition.tsv: '{line}' is not '<measure><TAB><number>' with a known measure")
            continue
        if c[0].strip() in out:
            errors.append(f"composition.tsv: {c[0].strip()} twice")
        out[c[0].strip()] = int(c[1])
    for m in COMPOSITION:
        if m not in out:
            errors.append(f"composition.tsv must declare {m}")
    return out


def manifest_composition(rows):
    out = {c: sum(r["class"] == c for r in rows) for c in ("scored", "control", "mixed", "exploratory")}
    out.update({k: sum(r["kind"] == k for r in rows) for k in ("authentic", "generated")})
    return out


def load_protocol(path, errors):
    try:
        return json.load(open(path, encoding="utf-8"))
    except (OSError, ValueError) as e:
        errors.append(f"protocol {path} unreadable: {e}")
        return None


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


def check_sources(seal, rows, paths, errors):
    """reference-sources.tsv against the sealed packages: every member of a
    package listed once as `kept` with its SHA-256, every `removed` member
    absent from it, the publisher's archive hash given. Returns
    {key: {"package_sha256", "publisher_archive", "publisher_archive_sha256", "kept", "removed"}}."""
    lines = open(os.path.join(seal, "reference-sources.tsv"), encoding="utf-8").read().splitlines()
    if not lines or lines[0].split("\t") != SOURCES:
        errors.append(f"reference-sources.tsv header must be exactly: {' | '.join(SOURCES)}")
        return {}
    by_key = {}
    for n, line in enumerate(lines[1:], 2):
        if not line.strip() or line.startswith("#"):
            continue
        c = [x.strip() for x in line.split("\t")]
        if len(c) != len(SOURCES):
            errors.append(f"reference-sources.tsv line {n}: {len(c)} columns, {len(SOURCES)} required")
            continue
        row = dict(zip(SOURCES, c))
        if row["role"] not in ("kept", "removed"):
            errors.append(f"reference-sources.tsv line {n}: role '{row['role']}' is not kept or removed")
        for h in ("publisher_archive_sha256", "member_sha256"):
            if not HEX.match(row[h]):
                errors.append(f"reference-sources.tsv line {n}: {h} is not a SHA-256")
        by_key.setdefault(row["key"], []).append(row)
    out = {}
    for r in rows:
        key, ref = r["key"], r["reference"]
        listed = by_key.pop(key, [])
        if ref == "-":
            if listed:
                errors.append(f"{key}: reference-sources.tsv lists a reference, but the document has none")
            continue
        if not listed:
            errors.append(f"{key}: reference-sources.tsv has no row for its reference {ref}")
            continue
        if {x["reference"] for x in listed} != {ref}:
            errors.append(f"{key}: reference-sources.tsv names another reference than the manifest's {ref}")
        archives = {(x["publisher_archive"], x["publisher_archive_sha256"]) for x in listed}
        if len(archives) != 1:
            errors.append(f"{key}: reference-sources.tsv gives more than one publisher archive")
        kept = {}
        for x in listed:
            if x["role"] == "kept":
                if x["member"] in kept:
                    errors.append(f"{key}: reference-sources.tsv lists {x['member']} twice")
                kept[x["member"]] = x["member_sha256"]
        removed = {x["member"]: x["member_sha256"] for x in listed if x["role"] == "removed"}
        actual = {}
        if ref in paths:
            if zipfile.is_zipfile(paths[ref]):
                with zipfile.ZipFile(paths[ref]) as z:
                    for info in z.infolist():
                        if not info.is_dir():
                            actual[info.filename] = sha256_bytes(z.read(info))
            else:
                actual[os.path.basename(ref)] = sha256(paths[ref])
        for m in sorted(set(actual) | set(kept)):
            if m not in kept:
                errors.append(f"{key}: package member {m} is not listed as kept in reference-sources.tsv")
            elif m not in actual:
                errors.append(f"{key}: reference-sources.tsv keeps {m}, which the package does not hold")
            elif actual[m] != kept[m]:
                errors.append(f"{key}: package member {m} is not the one reference-sources.tsv records")
        for m in sorted(removed):
            if m in actual:
                errors.append(f"{key}: {m} is listed as removed but is in the package")
        archive, archive_sha = next(iter(archives)) if archives else ("", "")
        out[key] = {"package_sha256": sha256(paths[ref]) if ref in paths else None,
                    "publisher_archive": archive, "publisher_archive_sha256": archive_sha,
                    "kept": len(kept), "removed": sorted(removed)}
    for key in sorted(by_key):
        errors.append(f"reference-sources.tsv has rows for {key}, which the manifest does not list")
    return out


# ---------------------------------------------------------------- the check

def parse_options(argv, allowed, flags=()):
    """(positional arguments, {option: value}, {flag}) of a command line, or
    an error naming every option that is not one of `allowed` (`--name=VALUE`)
    or `flags` (`--name`): an option is never silently ignored (E3)."""
    args, opts, given, bad = [], {}, set(), []
    for a in argv:
        if not a.startswith("--"):
            args.append(a)
        elif "=" in a and a[2:].split("=", 1)[0] in allowed:
            k, v = a[2:].split("=", 1)
            if k in opts:
                bad.append(f"{a} (given twice)")
            opts[k] = v
        elif "=" not in a and a[2:] in flags:
            given.add(a[2:])
        else:
            bad.append(a)
    return args, opts, given, bad


def check(seal, docs, seal_commit=None, inventory=None, protocol=None, verses_out=None):
    errors, notes = [], []
    report = {"seal_folder": seal, "documents_folder": docs, "seal_commit": seal_commit}
    protocol = protocol or PROTOCOL_DEFAULT
    report["tools"] = {name: sha256(os.path.join(HERE, name)) for name in
                       ("check-set.py", "reference_inventory.py", "usfm_text.py")}
    report["tools"]["usfm-books.tsv"] = sha256(reference_inventory.BOOKS_TSV)
    report["protocol"] = protocol
    report["protocol_sha256"] = sha256(protocol) if os.path.isfile(protocol) else None

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

    # 2b. Books (C3): stated, never guessed from a file name.
    known_books = reference_inventory.load_books()
    for r in rows:
        cell = r["books"]
        if cell == "-":
            if r["reference"] != "-":
                errors.append(f"{r['key']}: books is '-' but the document has a reference; write 'all' or the books")
            r["declared"] = None
            continue
        if not cell:
            errors.append(f"{r['key']}: books is empty; write 'all' or the USFM codes of the books it holds")
            continue
        if cell == "all":
            r["declared"] = None
            continue
        codes = [c.strip() for c in cell.split(",")]
        bad = [c for c in codes if c not in known_books]
        if bad:
            errors.append(f"{r['key']}: books {', '.join(bad)} are not USFM book codes (docs/evaluation/usfm-books.tsv)")
        r["declared"] = set(codes)

    # 2c. Composition (C2): declared before the seal, the manifest's own, and
    #     the protocol's when it requires one.
    report["composition"] = manifest_composition(rows)
    if "composition.tsv" in sealed:
        declared = parse_composition(open(os.path.join(seal, "composition.tsv"), encoding="utf-8").read(), errors)
        report["declared_composition"] = declared
        if declared is not None and declared != report["composition"]:
            errors.append(f"composition.tsv declares {declared}, but the manifest holds {report['composition']}")
        proto = load_protocol(protocol, errors)
        required = (proto or {}).get("required_composition")
        report["required_composition"] = required
        if required is not None and declared != required:
            errors.append(f"the declared composition {declared} is not the protocol's required {required}")
        if proto is not None and required is None:
            notes.append("the protocol requires no composition yet: no run under it is a protocol run")
        if proto is not None and proto.get("withheld_execution_permitted") is not True:
            notes.append("the protocol does not permit a withheld run yet (owner decisions pending): "
                         "the runner refuses to run under it")

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

    # 5b. References (C4, C5, B1, B2; E3 R1, R2): sound USFM read by the
    #     check's own text reader, their complete inventory with every
    #     unsupported class and every verse's text digest declared before any
    #     run, and evidence to score.
    report["reference_inventories"] = {}
    report["reference_evidence"] = {}
    for r in rows:
        if r["reference"] == "-" or r["reference"] not in paths or "declared" not in r:
            continue
        inv = reference_inventory.read(paths[r["reference"]], r["declared"], known_books)
        for problem in inv["problems"]:
            errors.append(f"{r['key']}: reference {problem}")
        for problem in inv["unicode_problems"]:
            errors.append(f"{r['key']}: reference {problem}")
        verses = inv.pop("verses")
        if verses_out:
            os.makedirs(verses_out, exist_ok=True)
            out = os.path.join(verses_out, f"{r['key']}.verses.json")
            with open(out, "w", encoding="utf-8") as f:
                json.dump(verses, f, ensure_ascii=False)
            inv["verses_file"] = f"{os.path.basename(os.path.normpath(verses_out))}/{r['key']}.verses.json"
            inv["verses_file_sha256"] = sha256(out)
        inv["verses_count"] = len(verses)
        report["reference_inventories"][r["key"]] = inv
        evidence = {"class": r["class"], "evidence_class": inv["evidence_class"],
                    "printed_scorable": inv["printed_scorable"], "absent_in_edition": inv["absent_in_edition"],
                    "printed_unsupported": inv["printed_unsupported"],
                    "printed_outside_declared_books": inv["printed_counts"]["outside_declared_scope"],
                    "printed_scorable_by_book": inv["printed_scorable_by_book"], "declared_books_without_scorable": {}}
        # Every declared book must be printed, scorably, in the reference.
        for code in sorted(r["declared"] or ()):
            if evidence["printed_scorable_by_book"].get(code):
                continue
            if known_books.get(code) is None:
                why = "outside the 66-book canon: its verses can only be unsupported"
            elif any(lbl.startswith(code + " ") for lbls in inv["unsupported_labels"].values() for lbl in lbls):
                why = "only unsupported verses in the reference"
            else:
                why = "no printed verse in the reference"
            evidence["declared_books_without_scorable"][code] = why
        report["reference_evidence"][r["key"]] = evidence
        if r["class"] == "scored":
            if inv["evidence_class"] != "scorable":
                errors.append(f"{r['key']}: a scored document's reference holds no printed scorable verse "
                              f"({inv['evidence_class']}: {inv['printed_scorable']} scorable, "
                              f"{inv['absent_in_edition']} numbered but empty, {inv['printed_unsupported']} unsupported); "
                              f"nothing in it could be verified. Use a reference that prints the document's books, "
                              f"or declare the document exploratory")
            for code, why in evidence["declared_books_without_scorable"].items():
                errors.append(f"{r['key']}: declared book {code}: {why}; a scored document's books must be "
                              f"printed in its reference")
        elif inv["evidence_class"] != "scorable":
            notes.append(f"{r['key']} ({r['class']}): its reference holds no printed scorable verse "
                         f"({inv['evidence_class']})")
        unsupported = {k: len(v) for k, v in inv["unsupported_labels"].items() if v}
        if inv["verses_before_chapter"]:
            unsupported["verse_before_chapter"] = inv["verses_before_chapter"]
        if unsupported:
            notes.append(f"{r['key']}: reference content the recogniser cannot hold, counted as unsupported: "
                         + ", ".join(f"{k} {n}" for k, n in unsupported.items()))
        if not all(m.get("nfc", True) for m in inv["members"] if m["role"] == "usfm"):
            notes.append(f"{r['key']}: the reference is not in Unicode NFC (compared under canonical equivalence)")

    # 5c. Reference provenance (E3): the publisher's archive recorded apart
    #     from the USFM-only package sealed here, member by member.
    report["reference_sources"] = check_sources(seal, rows, paths, errors) if "reference-sources.tsv" in sealed else {}

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
    args, opts, _, bad = parse_options(sys.argv[1:], OPTIONS)
    if bad:
        print(f"refused: unknown or repeated option(s) {', '.join(bad)}; options are "
              f"{', '.join('--' + o + '=...' for o in OPTIONS)}", file=sys.stderr)
        sys.exit(2)
    if len(args) != 2:
        sys.exit(__doc__)
    report = check(args[0], args[1], opts.get("seal-commit"), opts.get("inventory"), opts.get("protocol"),
                   opts.get("verses-out"))
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
