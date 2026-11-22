#!/usr/bin/env python3
"""Real bundled references through a harness binary (evaluation candidate
A1.5-E3 evidence; R1 end to end).

For each selection (an edition's book, or a whole edition), a Markdown
document is made from the edition's own words and run through the given
`real_document_editions` binary exactly as `run-withheld-set.sh` runs a
document (structural pipeline, its own process). The reference given to
the harness is the edition's USFM member(s), byte for byte, from the archive
bundled in `resources/scripture/`, whose SHA-256 is checked against its
`SOURCE.md` first.

The document's words come from a THIRD reader: the independent E2
reviewer's extractor (`evaluation-e2-review/evidence/inventory/indep_usfm.py`),
not from either E3 reader, so that agreement is not circular. Every verified
verse with wrong text is then attributed: a **reference artefact** when the
document holds the third reader's text and the harness's reference text
differs from it; a **document reading** difference otherwise.

These are GENERATED documents (the editions' own words, notes, headings and
titles left out). No authentic published document is available here.

    run_real.py <binary> <work folder> <out.json> [--full] [--e3]

--e3: also compare the harness's per-verse text digests with the Python
check's (`scripts/evaluation/reference_inventory.py`), as the scorer does.
"""
import hashlib
import importlib.util
import io
import json
import os
import re
import shutil
import subprocess
import sys
import time
import zipfile

ROOT = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), *[".."] * 6))
INDEP = os.path.join(ROOT, "docs/audit/candidate-a1.5/evaluation-e2-review/evidence/inventory/indep_usfm.py")
SELECTIONS = [  # key, edition, books, language, script
    ("e3r-engbsb-gen", "engbsb", ["GEN"], "en", "Latin"),
    ("e3r-grcsbl-mat", "grcsbl", ["MAT"], "grc", "Greek"),
    ("e3r-engwebp-dan", "engwebp", ["DAN"], "en", "Latin"),
    ("e3r-fralsg-eph", "fraLSG", ["EPH"], "fr", "Latin"),
    ("e3r-kor1961-rut", "kor1961", ["RUT"], "ko", "Hangul"),
    ("e3r-deu1912-rut", "deu1912", ["RUT"], "de", "Latin"),
    ("e3r-grcbyz-3jn", "grcbyz", ["3JN"], "grc", "Greek"),
]
FULL = [
    ("e3r-engbsb-all", "engbsb", None, "en", "Latin"),
    ("e3r-grcsbl-nt", "grcsbl", None, "grc", "Greek"),
]
COLUMNS = ["key", "files", "reference", "language", "script", "format", "layout", "role",
           "source and licence", "books", "sha256", "generator", "kind", "class"]


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m


def archive(edition):
    folder = os.path.join(ROOT, "resources", "scripture", edition)
    path = next(os.path.join(folder, f) for f in os.listdir(folder) if f.endswith("_usfm.zip"))
    want = re.search(r"SHA-256 \| `([0-9a-f]{64})`", open(os.path.join(folder, "SOURCE.md"), encoding="utf-8").read())
    got = hashlib.sha256(open(path, "rb").read()).hexdigest()
    if not want or want.group(1) != got:
        sys.exit(f"{path}: SHA-256 {got} is not the one in SOURCE.md")
    return path, got


def book_code(text):
    m = re.search(r"\\id\s+(\S+)", text)
    return m.group(1).upper() if m else None


def title(text):
    for marker in ("h", "toc2", "mt1", "mt"):
        m = re.search(r"^\\" + marker + r" +(.+?)\s*$", text, re.M)
        if m:
            return m.group(1)
    return book_code(text)


def markdown(members, indep):
    out = []
    for text in members:
        out.append(f"# {title(text)}\n")
        chapter, para = None, []
        for c, v, t in indep.extract(text):
            if c != chapter:
                if para:
                    out.append(" " + " ".join(para) + "\n")
                    para = []
                out.append(f"## {c}\n")
                chapter = c
            if t.strip():
                para.append(f"{v} {t}")
            if len(para) == 5:
                out.append(" " + " ".join(para) + "\n")
                para = []
        if para:
            out.append(" " + " ".join(para) + "\n")
    return "\n".join(out)


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    flags = {a for a in sys.argv[1:] if a.startswith("--")}
    if len(args) != 3 or flags - {"--full", "--e3"}:
        sys.exit(__doc__)
    binary, work, out_json = (os.path.abspath(a) for a in args)
    indep = load("indep_usfm", INDEP)
    fidelity = load("fidelity", os.path.join(ROOT, "scripts/evaluation/fidelity.py"))
    if os.path.exists(work):
        shutil.rmtree(work)
    docs, out = os.path.join(work, "documents"), os.path.join(work, "out")
    os.makedirs(docs)
    rows, provenance, oracle = [], {}, {}
    for key, edition, books, lang, script in SELECTIONS + (FULL if "--full" in flags else []):
        path, archive_sha = archive(edition)
        with zipfile.ZipFile(path) as z:
            members = []
            for name in z.namelist():
                if not name.lower().endswith((".usfm", ".sfm")):
                    continue
                data = z.read(name)
                code = book_code(data.decode("utf-8"))
                if (books is None and code not in ("FRT", "INT", "OTH", "BAK", "GLO", "CNC", "TDX", "NDX")) \
                        or (books and code in books):
                    members.append((name, data))
        texts = [d.decode("utf-8") for _, d in members]
        for (name, _), text in zip(members, texts):
            for c, v, t in indep.extract(text):
                oracle[(key, book_code(text), c, v)] = t
        md = markdown(texts, indep).encode("utf-8")
        open(os.path.join(docs, key + ".md"), "wb").write(md)
        buf = io.BytesIO()
        with zipfile.ZipFile(buf, "w", zipfile.ZIP_DEFLATED) as z:
            for name, data in members:
                z.writestr(name, data)
        open(os.path.join(docs, key + "_usfm.zip"), "wb").write(buf.getvalue())
        provenance[key] = {"edition": edition, "archive_sha256": archive_sha,
                           "members": {n: hashlib.sha256(d).hexdigest() for n, d in members},
                           "document_sha256": hashlib.sha256(md).hexdigest()}
        rows.append([key, key + ".md", key + "_usfm.zip", lang, script, "md", "derived", "e3-real", "bundled",
                     ",".join(books) if books else "all", hashlib.sha256(md).hexdigest(), "run_real.py",
                     "generated", "scored"])
    manifest = os.path.join(work, "manifest.tsv")
    open(manifest, "w").write("\t".join(COLUMNS) + "\n" + "".join("\t".join(r) + "\n" for r in rows))
    results = {"binary_sha256": hashlib.sha256(open(binary, "rb").read()).hexdigest(), "documents": {}}
    if "--e3" in flags:
        check_set = load("check_set", os.path.join(ROOT, "scripts/evaluation/check-set.py"))
        score_set = load("score_set", os.path.join(ROOT, "scripts/evaluation/score-set.py"))
    for row in rows:
        key = row[0]
        env = dict(os.environ, AW_PIPELINE="structural", AW_CORPUS_MANIFEST=manifest, AW_EDITIONS=key,
                   AW_DOCUMENT_EDITIONS=docs, AW_AUDIT_OUT=out)
        started = time.time()
        r = subprocess.run(["bash", "-c", 'ulimit -v 4194304; exec "$0" --ignored --nocapture', binary],
                           env=env, capture_output=True, text=True, timeout=3600)
        report_path = os.path.join(out, key + ".json")
        d = {"exit": r.returncode, "seconds": round(time.time() - started, 1), **provenance[key]}
        if r.returncode or not os.path.exists(report_path):
            d["error"] = r.stderr[-2000:]
            results["documents"][key] = d
            continue
        j = json.load(open(report_path, encoding="utf-8"))
        t = j["totals"]
        d.update({k: t[k] for k in ("expected", "verified", "verified_exact", "verified_canonical",
                                    "verified_layout", "verified_mismatch", "held", "not_found")})
        d["evaluation_schema"] = j.get("evaluation_schema")
        d["importer_comparison"] = {k: j["importer_comparison"].get(k) for k in
                                    ("agree", "only_importer", "only_inventory", "text_differs")}
        causes, rows_out = {"reference_artefact": 0, "document_reading": 0}, []
        for rec in j["fidelity_records"]:
            if rec["status"] != "verified" or rec["class"] != "mismatch":
                continue
            book, cv = rec["reference"].split(" ")
            c, v = cv.split(":")
            third = oracle.get((key, book, c, v))
            if third is not None and fidelity.classify(third, rec["found"]) != "mismatch" \
                    and fidelity.classify(third, rec["edition"]) == "mismatch":
                cause = "reference_artefact"
            else:
                cause = "document_reading"
            causes[cause] += 1
            i = next((k for k in range(min(len(rec["found"]), len(rec["edition"])))
                      if rec["found"][k] != rec["edition"][k]), 0)
            rows_out.append({"reference": rec["reference"], "kind": rec["kind"], "cause": cause,
                             "document": rec["found"][max(0, i - 40):i + 30],
                             "harness_reference": rec["edition"][max(0, i - 40):i + 30]})
        d["verified_mismatch_causes"] = causes
        d["verified_mismatches"] = rows_out
        if "--e3" in flags:
            check = check_set.reference_inventory.read(os.path.join(docs, key + "_usfm.zip"),
                                                       None if row[9] == "all" else set(row[9].split(",")))
            d["check_problems"] = check["problems"]
            d["check_text_digest"] = check.get("text_digest")
            d["harness_text_digest"] = j["reference_inventory"].get("text_digest")
            observed, integrity = score_set.observe_report(j, check)
            d["scorer_integrity"] = integrity
            d["scorer_verified_mismatch"] = observed.get("verified_mismatch")
        results["documents"][key] = d
        print(key, d.get("verified"), "verified,", d.get("verified_mismatch"), "mismatch", causes,
              d.get("scorer_integrity", ""), f"{d['seconds']}s", file=sys.stderr)
    json.dump(results, open(out_json, "w", encoding="utf-8"), ensure_ascii=False, indent=1)


if __name__ == "__main__":
    main()
