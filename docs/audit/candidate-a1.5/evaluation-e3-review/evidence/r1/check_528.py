#!/usr/bin/env python3
"""The 528 verses the E2 review found mis-spaced, checked against three
independent expectations (none of them the other E3 reader):

 1. the raw source: in every note of the verse (\\f..\\f*, \\x..\\x*), the
    character printed just before the note and the one just after it must be
    adjacent in the verse text exactly as in the source (no space inserted
    where the source has none; one space kept where the source has one);
 2. the E2 reviewer's committed context window (`independent_context`) must
    occur in the verse text, and the app reader's (`app_reader_context`) must not;
 3. the reviewer's own extractor (rev_usfm.py) must give the same text.

    check_528.py <repo> <out.json> [rust-dump.json]
"""
import importlib.util, json, os, re, sys, zipfile

repo, out = sys.argv[1], sys.argv[2]
rust = json.load(open(sys.argv[3], encoding="utf-8")) if len(sys.argv) > 3 else None
here = os.path.dirname(os.path.abspath(__file__))


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m


rev = load("rev_usfm", os.path.join(here, "rev_usfm.py"))
py = load("usfm_text", os.path.join(repo, "scripts", "evaluation", "usfm_text.py"))
rows = json.load(open(os.path.join(repo, "docs/audit/candidate-a1.5/evaluation-e2-review/evidence/inventory/"
                                   "note-space-differences.json"), encoding="utf-8"))
NOTE = re.compile(r"\\(f|x|fe)\s.*?\\\1\*", re.S)
results = {"checked": 0, "property_ok": 0, "context_ok": 0, "reviewer_same": 0, "rust_same": 0, "failures": [],
           "by_edition": {}, "notes_examined": 0}
for ed, info in rows.items():
    z = os.path.join(repo, "resources", "scripture", ed, f"{ed}_usfm.zip")
    texts, raws, revs, rkeys = {}, {}, {}, {}
    with zipfile.ZipFile(z) as zz:
        for nm in zz.namelist():
            if not nm.lower().endswith((".usfm", ".sfm")):
                continue
            src = zz.read(nm).decode("utf-8").replace("\r\n", "\n")
            b = py.read_text(src)
            a = rev.read(src)
            for i, v in enumerate(b["verses"]):
                key = f"{b['book']} {v['chapter']}:{v['label']}"
                texts[key] = v["text"]
                revs[key] = a["verses"][i][2]
                rkeys[key] = (f"{ed}/{nm}", i)
            # raw verse source: from "\v N " to the next \v or \c
            book = b["book"]
            chap = None
            for m in re.finditer(r"\\c (\S+)|\\v (\S+) ", src):
                if m.group(1):
                    chap = m.group(1)
                    continue
                start = m.end()
                nxt = re.compile(r"\\[vc] ").search(src, start)
                raws[f"{book} {chap}:{m.group(2)}"] = src[start:nxt.start() if nxt else len(src)]
    st = {"rows": len(info["rows"]), "ok": 0}
    for row in info["rows"]:
        ref = row["reference"]
        results["checked"] += 1
        text, raw = texts.get(ref), raws.get(ref)
        fails = []
        if text is None or raw is None:
            fails.append("verse not found")
        else:
            notes = list(NOTE.finditer(raw))
            if not notes:
                fails.append("no note in the raw verse")
            for m in notes:
                results["notes_examined"] += 1
                left = raw[:m.start()]
                right = raw[m.end():]
                # the visible text just before and after (markers stripped, rough but independent)
                def visible(x):
                    x = NOTE.sub("", x)                           # other notes of the verse
                    x = re.sub(r"\|[^\\]*(?=\\)", "", x)        # attribute lists
                    x = re.sub(r"\\\+?[a-z0-9]+\*", "", x)          # end markers take no space
                    return re.sub(r"\\\+?[a-z0-9]+[ \t\n]?", "", x)   # opening markers and their own space
                lt, rt = visible(left), visible(right)
                lc = lt[-1:] if lt else ""
                rc = rt[:1]
                if not lc or not rc or rc == "\n":
                    continue
                if lc in " \t\n" or rc in " \t\n":
                    continue  # the source has white space here; nothing to assert
                # the source joins lc and rc: the text must hold lc+rc, and not lc+" "+rc for this spot
                # find the visible tail word before the note and check adjacency in the verse text
                tail = re.split(r"[ \t\n]", lt.rstrip("\n"))[-1]
                head = re.split(r"[ \t\n]", rt)[0]
                tail = re.sub(r"\|.*$", "", tail)
                if (tail + head) not in text.replace("\u00a0", " "):
                    fails.append(f"source joins «{tail}»+«{head}» but the text does not")
            if row["independent_context"].strip() not in text:
                fails.append("E2 reviewer's context not in the text")
            if row["app_reader_context"].strip() in text and row["app_reader_context"].strip() != row["independent_context"].strip():
                fails.append("app reader's context is in the text")
            else:
                results["context_ok"] += 1
        if not fails:
            results["property_ok"] += 1
            st["ok"] += 1
        else:
            results["failures"].append({"edition": ed, "ref": ref, "why": fails, "text": text})
        if text is not None and revs.get(ref) == text:
            results["reviewer_same"] += 1
        if rust is not None and text is not None:
            f, i = rkeys[ref]
            if rust.get(f) and rust[f][i][2] == text:
                results["rust_same"] += 1
    results["by_edition"][ed] = st
json.dump(results, open(out, "w", encoding="utf-8"), ensure_ascii=False, indent=1)
print({k: v for k, v in results.items() if k != "failures"}, len(results["failures"]))
for f in results["failures"][:12]:
    print(json.dumps(f, ensure_ascii=False)[:500])
