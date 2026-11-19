#!/usr/bin/env python3
"""Builds the two exclusion files a withheld set's builder and runner use.

    disclosed-inventory.py <out folder> <corpus folder>...

* disclosed-documents.sha256: the SHA-256 of every file in the given corpus
  folders (documents, references, build intermediates), of every USFM book
  inside a zip among them, and every SHA-256 the repository's manifests
  record (docs/corpus/*.tsv). One hash per line, no names: a byte-identical
  file is a disclosed one. A re-saved or re-zipped file is not caught here;
  that is what the edition list is for.
* disclosed-editions.tsv: the edition identity of every disclosed document,
  from the manifests' reference and source columns, one row per reference
  text: language, the reference's name, and the source and licence as
  recorded. No layout, converter, result or rule: nothing that describes how
  the recogniser was developed or how it fared.
"""
import glob
import hashlib
import os
import re
import sys
import zipfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.normpath(os.path.join(HERE, "..", ".."))
HEX = re.compile(r"^[0-9a-f]{64}$")


def sha(data):
    return hashlib.sha256(data).hexdigest()


def file_sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def clean_source(text):
    text = re.sub(r"\[(real file|rendered|eBible|other)\]\s*", "", text)
    text = re.sub(r"\s*;?\s*scripts/\S+", "", text)
    text = re.sub(r"\b(render|conv|build)_\w+\.py\b", "", text)
    # Remarks about how earlier sets were made are not identity.
    text = re.sub(r"\s*(REUSE )?NOTE:.*$", "", text)
    text = re.sub(r"\s*\(layout[^)]*\)", "", text)
    return re.sub(r"\s+", " ", text).strip(" ;")


def main():
    out, folders = sys.argv[1], sys.argv[2:]
    hashes, counts = set(), {"files": 0, "usfm_books": 0, "recorded": 0}
    for folder in folders:
        for root, _, files in os.walk(folder, followlinks=True):
            for f in files:
                p = os.path.join(root, f)
                if not os.path.isfile(p):
                    continue
                hashes.add(file_sha(p))
                counts["files"] += 1
                if zipfile.is_zipfile(p):
                    try:
                        with zipfile.ZipFile(p) as z:
                            for info in z.infolist():
                                if info.filename.lower().endswith((".usfm", ".sfm")):
                                    hashes.add(sha(z.read(info)))
                                    counts["usfm_books"] += 1
                    except (zipfile.BadZipFile, OSError):
                        pass
    editions = {}
    for path in sorted(glob.glob(os.path.join(REPO, "docs", "corpus", "*.tsv"))):
        lines = open(path, encoding="utf-8").read().splitlines()
        header = lines[0].split("\t") if lines else []
        for line in lines[1:]:
            if not line.strip() or line.startswith("#"):
                continue
            c = line.split("\t")
            row = dict(zip(header, c))
            for col in ("sha256",):
                if HEX.match(row.get(col, "").strip()):
                    hashes.add(row[col].strip())
                    counts["recorded"] += 1
            ref = row.get("reference", "").strip()
            if ref and ref != "-" and len(c) >= 9:
                name = re.sub(r"(\.usfm)?\.zip$", "", ref)
                name = re.sub(r"_usfm$", "", name)
                source = clean_source(c[8])
                old = editions.get(name)
                if not old or len(source) > len(old[1]):
                    editions[name] = (c[3].strip(), source)
    os.makedirs(out, exist_ok=True)
    with open(os.path.join(out, "disclosed-documents.sha256"), "w", encoding="utf-8") as f:
        f.write("".join(f"{h}\n" for h in sorted(hashes)))
    with open(os.path.join(out, "disclosed-editions.tsv"), "w", encoding="utf-8") as f:
        f.write("reference text\tlanguage\tsource and licence, as recorded\n")
        for name, (lang, source) in sorted(editions.items(), key=lambda x: x[0].lower()):
            f.write(f"{name}\t{lang}\t{source}\n")
    print(f"{len(hashes)} hashes ({counts}); {len(editions)} reference texts")


if __name__ == "__main__":
    main()
