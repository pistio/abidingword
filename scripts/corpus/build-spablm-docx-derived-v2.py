#!/usr/bin/env python3
"""Builds `spablm-docx-derived-v2`: the corrected derivative of the Spanish
Free Bible's USFM as a Word document, beside the original `spablm-docx`
(v1), which stays in the corpus exactly as it was sealed.

    build-spablm-docx-derived-v2.py <corpus folder> <output folder>

<corpus folder> holds `spablm_usfm.zip` (checked against its SHA-256 below).
<output folder> receives:

    spablm_usfm_lo.v2.docx                     the document
    spablm-docx-derived-v2.provenance.json     how it was made, and its checksums

v1 was written by scripts/build-derived-editions.py as it stood before
commit ccaad81; that revision left out the text of `\\mi` paragraphs. v2 is
written by the same script as it stands in this checkout (ccaad81 or
later), with the same command.

Reproducible: LibreOffice writes the same parts every time, but stamps the
container's entries with the time of day. This script rewrites the
container with fixed entry dates, the original entry order and no
compression (so that no compression library's version matters). The
document's identity is therefore two checksums:

* `parts_sha256`: over every part's own SHA-256 and name. It depends on
  the source, the generator and the LibreOffice version only.
* `sha256`: of the file this script writes.

The build fails if either differs from the values pinned below, so a
different LibreOffice cannot silently give a different "v2".
"""
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import zipfile

KEY = "spablm-docx-derived-v2"
FILE = "spablm_usfm_lo.v2.docx"
SOURCE = "spablm_usfm.zip"
SOURCE_SHA256 = "69ff87d19976bfc2306cb11ffeffd9fe0b4163eac9bd09da7f687b53a4de90f5"
CHAPTER_LABEL = "Capítulo {n}"
V1_FILE = "spablm_usfm_lo.docx"
V1_SHA256 = "2ad1094b299e2e19f4275ff90dea15cf8934717b7be83df3aab0ec8ed9b7d28d"
# Pinned results (LibreOffice 24.2.7.2, Linux x86-64).
PARTS_SHA256 = "ed5c8ad1a7f55e79ad8994dba25e6976255508f09fbfbcc538a302776ab8a097"
SHA256 = "7e5c90804f203d780a116af9969c9b2bffbc82d2a9056d5b30f0c0fc80aa4925"

HERE = os.path.dirname(os.path.abspath(__file__))
GENERATOR = os.path.join(HERE, "..", "build-derived-editions.py")


def sha256_of(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def parts_of(path):
    z = zipfile.ZipFile(path)
    return [(i.filename, hashlib.sha256(z.read(i)).hexdigest()) for i in z.infolist()]


def parts_sha256(parts):
    lines = "".join(f"{digest}  {name}\n" for name, digest in sorted(parts))
    return hashlib.sha256(lines.encode()).hexdigest()


def canonical(src, dst):
    """The same parts in the same order, entry dates fixed, stored."""
    z = zipfile.ZipFile(src)
    with zipfile.ZipFile(dst, "w", zipfile.ZIP_STORED) as out:
        for item in z.infolist():
            info = zipfile.ZipInfo(item.filename, date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_STORED
            info.create_system = 0
            info.external_attr = 0
            out.writestr(info, z.read(item))


def git(*args):
    try:
        return subprocess.run(["git", "-C", HERE, *args], check=True, capture_output=True,
                              text=True).stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        return None


def main():
    corpus, out = sys.argv[1], sys.argv[2]
    source = os.path.join(corpus, SOURCE)
    got = sha256_of(source)
    if got != SOURCE_SHA256:
        sys.exit(f"{SOURCE}: SHA-256 {got}, expected {SOURCE_SHA256}")
    os.makedirs(out, exist_ok=True)
    office = subprocess.run(["soffice", "--version"], capture_output=True, text=True).stdout.strip()
    with tempfile.TemporaryDirectory() as tmp:
        shutil.copy(source, os.path.join(tmp, SOURCE))
        command = [sys.executable, GENERATOR, tmp, "spablm", "docx", f"--usfm={CHAPTER_LABEL}"]
        subprocess.run(command, check=True)
        raw = os.path.join(tmp, "spablm_usfm_lo.docx")
        parts = parts_of(raw)
        target = os.path.join(out, FILE)
        canonical(raw, target)
    assert parts_of(target) == parts
    record = {
        "key": KEY,
        "file": FILE,
        "sha256": sha256_of(target),
        "parts_sha256": parts_sha256(parts),
        "parts": [{"name": n, "sha256": d} for n, d in parts],
        "container": "ZIP, entries stored, dates 1980-01-01, original order",
        "source": {"file": SOURCE, "sha256": SOURCE_SHA256,
                   "origin": "https://ebible.org/Scriptures/spablm_usfm.zip (as held in the corpus; "
                             "docs/audit/v0.24.7/corpus-checksums.sha256)"},
        "generator": {
            "script": "scripts/build-derived-editions.py",
            "script_sha256": sha256_of(GENERATOR),
            "script_git_blob": git("hash-object", GENERATOR),
            "script_last_commit": git("log", "-1", "--format=%H", "--", GENERATOR),
            "command": f'scripts/build-derived-editions.py <folder> spablm docx --usfm="{CHAPTER_LABEL}"',
            "normalised_by": "scripts/corpus/build-spablm-docx-derived-v2.py",
        },
        "tools": {"libreoffice": office, "python": sys.version.split()[0]},
        "supersedes_nothing": f"v1 ({V1_FILE}, SHA-256 {V1_SHA256}) stays in the corpus unchanged",
    }
    with open(os.path.join(out, f"{KEY}.provenance.json"), "w", encoding="utf-8") as f:
        json.dump(record, f, ensure_ascii=False, indent=1)
        f.write("\n")
    print(json.dumps({k: record[k] for k in ("key", "file", "sha256", "parts_sha256")}, indent=1))
    for name, pinned, value in (("parts_sha256", PARTS_SHA256, record["parts_sha256"]),
                                ("sha256", SHA256, record["sha256"])):
        if pinned and pinned != value:
            sys.exit(f"{name} is {value}, pinned {pinned}: not the pinned v2 ({office})")
    if not (PARTS_SHA256 and SHA256):
        print("not pinned yet")


if __name__ == "__main__":
    main()
