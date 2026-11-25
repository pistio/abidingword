#!/usr/bin/env python3
"""Generated documents (NOT authentic published files) made from the bundled
references' own words, read by the reviewer's extractor (rev_usfm.py: a
fourth reading, independent of both E3 readers and the E2 reviewer's).
Markdown: `# Book`, `## chapter`, paragraphs of five numbered verses.
References: USFM-only packages of the edition's own members, byte for byte
(copr.htm kept; keys.asc and *.css left out).

    make_docs.py <repo> <out folder>
"""
import hashlib, importlib.util, io, json, os, re, sys, zipfile

repo, out = sys.argv[1:3]
here = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("rev_usfm", os.path.join(here, "..", "r1", "rev_usfm.py"))
rev = importlib.util.module_from_spec(spec)
spec.loader.exec_module(rev)
T = dict(GEN="Genesis", EXO="Exodus", LEV="Leviticus", NUM="Numbers", DEU="Deuteronomy", JOS="Joshua", JDG="Judges",
         RUT="Ruth", **{"1SA": "1 Samuel", "2SA": "2 Samuel", "1KI": "1 Kings", "2KI": "2 Kings", "1CH": "1 Chronicles",
         "2CH": "2 Chronicles"}, EZR="Ezra", NEH="Nehemiah", EST="Esther", JOB="Job", PSA="Psalms", PRO="Proverbs",
         ECC="Ecclesiastes", SNG="Song of Solomon", ISA="Isaiah", JER="Jeremiah", LAM="Lamentations", EZK="Ezekiel",
         DAN="Daniel", HOS="Hosea", JOL="Joel", AMO="Amos", OBA="Obadiah", JON="Jonah", MIC="Micah", NAM="Nahum",
         HAB="Habakkuk", ZEP="Zephaniah", HAG="Haggai", ZEC="Zechariah", MAL="Malachi", MAT="Matthew", MRK="Mark",
         LUK="Luke", JHN="John", ACT="Acts", ROM="Romans", **{"1CO": "1 Corinthians", "2CO": "2 Corinthians"},
         GAL="Galatians", EPH="Ephesians", PHP="Philippians", COL="Colossians",
         **{"1TH": "1 Thessalonians", "2TH": "2 Thessalonians", "1TI": "1 Timothy", "2TI": "2 Timothy"},
         TIT="Titus", PHM="Philemon", HEB="Hebrews", JAS="James", **{"1PE": "1 Peter", "2PE": "2 Peter",
         "1JN": "1 John", "2JN": "2 John", "3JN": "3 John"}, JUD="Jude", REV="Revelation")
os.makedirs(out, exist_ok=True)
rows, texts = [], {}
for ed, lang, script, skip in (("engbsb", "en", "Latin", ()), ("grcsbl", "grc", "Greek", ())):
    z = zipfile.ZipFile(os.path.join(repo, "resources", "scripture", ed, f"{ed}_usfm.zip"))
    books, keep = {}, []
    for n in z.namelist():
        if n.lower().endswith(".usfm"):
            r = rev.read(z.read(n).decode("utf-8"))
            if r["book"] in T and r["book"] not in skip:
                books[r["book"]] = r
                keep.append(n)
            assert not r["trouble"], (n, r["trouble"])
        elif n.lower().endswith(".htm"):
            keep.append(n)
    md = []
    for code in T:
        if code not in books:
            continue
        md.append(f"# {T[code]}\n")
        chapter, para = None, []
        for c, v, t in books[code]["verses"]:
            texts[f"{ed} {code} {c}:{v}"] = t
            if c != chapter:
                if para:
                    md.append(" " + " ".join(para) + "\n"); para = []
                chapter = c
                md.append(f"## {c}\n")
            if t:
                para.append(f"{v} {t}")
                if len(para) == 5:
                    md.append(" " + " ".join(para) + "\n"); para = []
        if para:
            md.append(" " + " ".join(para) + "\n")
    name = f"rv-{ed}.md"
    data = "\n".join(md).encode("utf-8")
    open(os.path.join(out, name), "wb").write(data)
    buf = io.BytesIO()
    with zipfile.ZipFile(buf, "w", zipfile.ZIP_DEFLATED) as o:
        for n in keep:
            o.writestr(zipfile.ZipInfo(n, date_time=(2026, 11, 25, 0, 0, 0)), z.read(n))
    ref = f"rv-{ed}_usfm.zip"
    open(os.path.join(out, ref), "wb").write(buf.getvalue())
    rows.append([f"rv-{ed}", name, ref, lang, script, "md", "review generated, whole edition", "review",
                 f"{ed} bundled eBible USFM (public domain / CC BY 4.0)", "all", hashlib.sha256(data).hexdigest(),
                 "review make_docs.py", "generated", "scored"])
cols = ["key", "files", "reference", "language", "script", "format", "layout", "role", "source and licence",
        "books", "sha256", "generator", "kind", "class"]
open(os.path.join(out, "manifest.tsv"), "w").write("\t".join(cols) + "\n" + "".join("\t".join(r) + "\n" for r in rows))
json.dump(texts, open(os.path.join(out, "document-verse-texts.json"), "w", encoding="utf-8"), ensure_ascii=False)
for r in rows:
    print(r[0], r[1], r[10])
