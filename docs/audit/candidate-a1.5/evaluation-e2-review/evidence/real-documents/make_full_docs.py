#!/usr/bin/env python3
"""Whole-edition Markdown documents from real bundled references (reviewer's
independent extraction), for the A1.5-vs-E2 comparison. English book titles."""
import hashlib, json, os, re, sys, zipfile
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "inventory"))
import indep_usfm
REPO = "/home/user/abidingword"
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
ORDER = list(T)
out = sys.argv[1]; os.makedirs(out, exist_ok=True)
rows = []
for ed, lang, script in (("engbsb", "en", "Latin"), ("grcsbl", "grc", "Greek"), ("kor1961", "ko", "Hangul")):
    z = f"{REPO}/resources/scripture/{ed}/{ed}_usfm.zip"
    zf = zipfile.ZipFile(z)
    books = {}
    for i in zf.infolist():
        if i.filename.lower().endswith((".usfm", ".sfm")):
            t = zf.read(i).decode("utf-8")
            m = re.search(r"\\id\s+(\S+)", t)
            if m and m.group(1).upper() in T:
                books[m.group(1).upper()] = t
    md = []
    for code in ORDER:
        if code not in books:
            continue
        md.append(f"# {T[code]}\n")
        chapter, para = None, []
        for c, v, t in indep_usfm.extract(books[code]):
            if c != chapter:
                if para: md.append(" " + " ".join(para) + "\n"); para = []
                chapter = c; md.append(f"## {c}\n")
            if t:
                para.append(f"{v} {t}")
                if len(para) == 5: md.append(" " + " ".join(para) + "\n"); para = []
        if para: md.append(" " + " ".join(para) + "\n")
    name = f"full-{ed}.md"
    open(os.path.join(out, name), "w", encoding="utf-8").write("\n".join(md))
    ref = f"full-{ed}_usfm.zip"
    open(os.path.join(out, ref), "wb").write(open(z, "rb").read())   # the bundled reference, byte for byte
    rows.append([f"full-{ed}", name, ref, lang, script, "md", "review generated, whole edition", "review",
                 f"{ed} bundled eBible USFM", "all", hashlib.sha256(open(os.path.join(out, name), "rb").read()).hexdigest(),
                 "review make_full_docs.py", "generated", "scored"])
cols = ["key", "files", "reference", "language", "script", "format", "layout", "role", "source and licence",
        "books", "sha256", "generator", "kind", "class"]
open(os.path.join(out, "manifest.tsv"), "w").write("\t".join(cols) + "\n" + "".join("\t".join(r) + "\n" for r in rows))
json.dump([dict(key=r[0], file=r[1], reference=r[2], book=None) for r in rows], open(os.path.join(out, "documents.json"), "w"))
print("\n".join(r[0] for r in rows))
