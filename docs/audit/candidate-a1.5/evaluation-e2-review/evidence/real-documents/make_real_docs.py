#!/usr/bin/env python3
"""Independent review of A1.5-E2: generated documents from REAL references.

For each selection (edition, book) this writes, from the bundled eBible USFM
(resources/scripture/<id>/<id>_usfm.zip, identity checked against the SHA-256
in its SOURCE.md), using the reviewer's independent USFM extractor
(../inventory/indep_usfm.py):

  <key>.md            "# Title", "## n", paragraphs with inline verse numbers
  <key>_lo.docx/.pdf  the same as HTML converted by LibreOffice (optional)
  <key>_ref.zip       a reference holding only that book's USFM member, byte
                      for byte as published

and a 14-column manifest (the sealed-set format) for the harness.
The words are the edition's own; notes, headings and titles are left out.
"""
import hashlib, html, json, os, re, subprocess, sys, tempfile, zipfile
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "inventory"))
import indep_usfm

REPO = "/home/user/abidingword"
TITLES = {"GEN": "Genesis", "RUT": "Ruth", "DAN": "Daniel", "MAT": "Matthew", "EPH": "Ephesians",
          "JHN": "John", "3JN": "3 John", "PHP": "Philippians", "JUD": "Jude", "MRK": "Mark", "OBA": "Obadiah"}
LANG = {"engbsb": "en", "engwebp": "en", "grcsbl": "grc", "fraLSG": "fr", "kor1961": "ko", "deu1912": "de",
        "spaRV1909": "es", "grcbyz": "grc", "porbr2018": "pt", "deu1951": "de"}
SCRIPT = {"grc": "Greek", "ko": "Hangul"}


def source_sha(ed):
    return re.search(r"`([0-9a-f]{64})`", open(f"{REPO}/resources/scripture/{ed}/SOURCE.md").read()).group(1)


def build(out, key, ed, book, formats):
    z = f"{REPO}/resources/scripture/{ed}/{ed}_usfm.zip"
    zsha = hashlib.sha256(open(z, "rb").read()).hexdigest()
    assert zsha == source_sha(ed), f"{ed}: zip is not the one SOURCE.md records"
    zf = zipfile.ZipFile(z)
    member = next(i.filename for i in zf.infolist() if i.filename.lower().endswith((".usfm", ".sfm"))
                  and re.search(r"\\id\s+" + book + r"\b", zf.read(i).decode("utf-8")[:200]))
    data = zf.read(member)
    ref = os.path.join(out, f"{key}_ref.zip")
    with zipfile.ZipFile(ref, "w", zipfile.ZIP_DEFLATED) as w:
        info = zipfile.ZipInfo(member, date_time=(2026, 11, 22, 0, 0, 0))
        w.writestr(info, data)
    verses = indep_usfm.extract(data.decode("utf-8"))
    title = TITLES[book]
    md, h = [f"# {title}\n"], [f'<!DOCTYPE html><html lang="{LANG[ed]}"><head><meta charset="UTF-8"/></head><body>',
                               f"<h1>{html.escape(title)}</h1>"]
    chapter, para = None, []

    def flush():
        if para:
            md.append(" " + " ".join(f"{n} {t}" for n, t in para) + "\n")
            h.append("<p>" + " ".join(f"<sup>{html.escape(n)}</sup>{html.escape(t)}" for n, t in para) + "</p>")
            para.clear()
    for c, v, t in verses:
        if c != chapter:
            flush()
            chapter = c
            md.append(f"## {c}\n")
            h.append(f"<h2>{html.escape(c)}</h2>")
        if t:
            para.append((v, t))
            if len(para) == 5:
                flush()
    flush()
    h.append("</body></html>")
    files = []
    with open(os.path.join(out, f"{key}.md"), "w", encoding="utf-8") as f:
        f.write("\n".join(md))
    files.append(("md", f"{key}.md"))
    with tempfile.TemporaryDirectory() as tmp:
        src = os.path.join(tmp, f"{key}_lo.html")
        open(src, "w", encoding="utf-8").write("".join(h))
        for fmt in formats:
            filt = {"docx": "docx:MS Word 2007 XML", "pdf": "pdf:writer_pdf_Export"}[fmt]
            subprocess.run(["soffice", "--headless", f"-env:UserInstallation=file://{tmp}/lo",
                            "--infilter=HTML (StarWriter)", "--convert-to", filt, "--outdir", tmp, src],
                           check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=1800)
            os.replace(os.path.join(tmp, f"{key}_lo.{fmt}"), os.path.join(out, f"{key}_lo.{fmt}"))
            files.append((fmt, f"{key}_lo.{fmt}"))
    return [dict(key=f"{key}-{fmt}", file=name, reference=f"{key}_ref.zip", language=LANG[ed],
                 script=SCRIPT.get(LANG[ed], "Latin"), format=fmt, book=book, edition=ed, member=member,
                 member_sha256=hashlib.sha256(data).hexdigest(), zip_sha256=zsha, verses=len(verses))
            for fmt, name in files]


def main():
    out = os.path.abspath(sys.argv[1])
    os.makedirs(out, exist_ok=True)
    selections = [("bsb-gen", "engbsb", "GEN", ["docx", "pdf"]), ("webp-dan", "engwebp", "DAN", ["pdf"]),
                  ("sbl-mat", "grcsbl", "MAT", ["pdf"]), ("lsg-eph", "fraLSG", "EPH", ["docx"]),
                  ("kor-rut", "kor1961", "RUT", ["pdf"]), ("de-rut", "deu1912", "RUT", []),
                  ("es-rut", "spaRV1909", "RUT", ["docx"]), ("byz-3jn", "grcbyz", "3JN", [])]
    rows = []
    for key, ed, book, formats in selections:
        rows += build(out, key, ed, book, formats)
    cols = ["key", "files", "reference", "language", "script", "format", "layout", "role", "source and licence",
            "books", "sha256", "generator", "kind", "class"]
    with open(os.path.join(out, "manifest.tsv"), "w", encoding="utf-8") as f:
        f.write("\t".join(cols) + "\n")
        for r in rows:
            sha = hashlib.sha256(open(os.path.join(out, r["file"]), "rb").read()).hexdigest()
            r["sha256"] = sha
            f.write("\t".join([r["key"], r["file"], r["reference"], r["language"], r["script"], r["format"],
                               "review generated", "review", f"{r['edition']} (bundled eBible USFM)", r["book"],
                               sha, "review make_real_docs.py", "generated", "scored"]) + "\n")
    json.dump(rows, open(os.path.join(out, "documents.json"), "w"), indent=1)
    for r in rows:
        print(r["key"], r["file"], r["member"], r["verses"])


if __name__ == "__main__":
    main()
