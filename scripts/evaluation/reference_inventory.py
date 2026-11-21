#!/usr/bin/env python3
"""A neutral label inventory of a reference edition, rules
`aw-reference-inventory-1` (evaluation candidate A1.5-E2).

The check's own reading of a reference, written apart from the harness's
(`src-tauri/tests/eval_support/reference.rs`) and independent of the
recogniser's supported canon. It reads the `\\id`, `\\c` and `\\v` labels of
every USFM book file exactly as the app's USFM reader takes them (notes
removed first; the first word after each marker), and gives every verse
label a status:

  scorable                a book of the 66, a chapter within its count, a
                          verse 1..200 or a bridge N-M (stored on N), unique
  outside_declared_scope  a book the manifest's `books` column leaves out
  one of UNSUPPORTED      kept and counted, never dropped

It reads labels only, never verse text, so it counts labels (printed and
empty together). check-set.py records it in CHECK.json before the run: the
complete reference coverage and every unsupported class, declared before
any result. The scorer compares it with the harness's inventory (label
digest and counts per class); any difference makes the run not a protocol
run.

A reference must be a .zip of USFM books (.usfm, .sfm), optionally with
prose (.htm, .html, never read), or one .usfm/.sfm file. Anything else in it
(a verse map, OSIS, MorphGNT, metadata, any other file), text that is not
UTF-8, or a file with more than one \\id line is a problem, and check-set.py
refuses the set.

    reference_inventory.py <reference file> [BOOK,BOOK...]   prints the inventory
"""
import hashlib
import json
import os
import re
import sys
import unicodedata
import zipfile

RULES = "aw-reference-inventory-1"
HERE = os.path.dirname(os.path.abspath(__file__))
# Beside this script (a builder's flat copy of the tools), else in the repository.
BOOKS_TSV = next((p for p in (os.path.join(HERE, "usfm-books.tsv"),
                              os.path.normpath(os.path.join(HERE, "..", "..", "docs", "evaluation", "usfm-books.tsv")))
                  if os.path.isfile(p)), os.path.join(HERE, "usfm-books.tsv"))
MAX_VERSE = 200

UNSUPPORTED = ("verse_zero", "verse_label_unusual", "verse_duplicate", "verse_before_chapter",
               "chapter_zero", "chapter_label_unusual", "chapter_beyond_canon",
               "book_outside_canon", "book_code_unknown", "book_duplicate", "book_unidentified")

# Unicode White_Space, as the app's reader (Rust char::is_whitespace) uses it.
WS = "\t\n\x0b\x0c\r \x85\xa0\u1680\u2000\u2001\u2002\u2003\u2004\u2005\u2006\u2007\u2008\u2009\u200a\u2028\u2029\u202f\u205f\u3000"
_WSC = "[" + "".join("\\x%02x" % ord(c) if ord(c) < 0x100 else "\\u%04x" % ord(c) for c in WS) + "]"
NOTES = re.compile(r"(?s)\\(f|fe|ef|x|ex)" + _WSC + r".*?\\(f|fe|ef|x|ex)\*")
MARKER = re.compile(r"\\\+?([a-z]+[0-9]*)(?:(\*)|[ \t\r\n]?)")


def load_books(path=BOOKS_TSV):
    """{code: chapters (int) for the 66, None for every other known code}."""
    books = {}
    for line in open(path, encoding="utf-8"):
        line = line.rstrip("\n")
        if not line or line.startswith("#") or line.startswith("code\t"):
            continue
        code, group, chapters, _ = line.split("\t")
        books[code] = int(chapters) if group == "canon66" else None
    return books


def _first_word(text):
    text = text.lstrip(WS)
    for i, c in enumerate(text):
        if c in WS:
            return text[:i]
    return text


def _ascii_upper(s):
    return "".join(chr(ord(c) - 32) if "a" <= c <= "z" else c for c in s)


def as_usfm_defines(text):
    """`\\qa` as a heading, line by line, as the harness reads a reference."""
    lines = text.split("\n")
    if lines and lines[-1] == "":
        lines.pop()
    out = []
    for line in lines:
        if line.endswith("\r"):
            line = line[:-1]
        out.append("\\s1 " + line[4:] if line.startswith("\\qa ") else line)
    return "\n".join(out)


def scan(text):
    """(book code or None, [(chapter label, verse label)], verses before any
    chapter), as the app's USFM reader takes the labels."""
    cleaned = NOTES.sub(" ", text)
    state = {"book": None, "chapter": None, "verses": [], "outside": 0, "expect": None}

    def segment(seg):
        kind, state["expect"] = state["expect"], None
        if kind is None:
            return
        word = _first_word(seg)
        if kind == "id":
            state["book"] = _ascii_upper(word)
        elif kind == "c":
            state["chapter"] = word
        elif state["chapter"] is not None:
            state["verses"].append((state["chapter"], word))
        else:
            state["outside"] += 1

    pos = 0
    for m in MARKER.finditer(cleaned):
        segment(cleaned[pos:m.start()])
        if m.group(2) is None and m.group(1) in ("id", "c", "v"):
            state["expect"] = m.group(1)
        pos = m.end()
    segment(cleaned[pos:])
    return state["book"], state["verses"], state["outside"]


def verse_key(label):
    """The verse number the importer would store `label` under, or why not."""
    parts = re.split("[-\u2013]", label, maxsplit=1)

    def num(s):
        # As Rust's u16 parse: digits, an optional leading '+', at most 65535.
        return int(s) if re.fullmatch(r"\+?[0-9]+", s) and int(s) <= 65535 else None
    first = num(parts[0])
    if first == 0:
        return None, "verse_zero"
    if len(parts) == 1:
        return (first, None) if first is not None and first <= MAX_VERSE else (None, "verse_label_unusual")
    last = num(parts[1])
    if first is not None and first <= MAX_VERSE and last is not None and last > first:
        return first, None
    return None, "verse_label_unusual"


def chapter_number(label):
    return int(label) if re.fullmatch(r"\+?[0-9]+", label) and int(label) <= 65535 else None


def members(path, problems):
    """[(member name, bytes)] of the USFM book files, in archive order, and
    the member list with roles."""
    name = os.path.basename(path)
    low = name.lower()
    out, listed = [], []
    if low.endswith(".zip"):
        try:
            z = zipfile.ZipFile(path)
        except (zipfile.BadZipFile, OSError) as e:
            problems.append(f"{name}: not a readable zip ({e})")
            return out, listed
        with z:
            for info in z.infolist():
                if info.is_dir():
                    continue
                m, ml = info.filename, info.filename.lower()
                try:
                    data = z.read(info)
                except (zipfile.BadZipFile, OSError, ValueError) as e:
                    problems.append(f"{m}: unreadable ({e})")
                    continue
                if ml.endswith((".usfm", ".sfm")):
                    out.append((m, data))
                    listed.append({"name": m, "role": "usfm"})
                elif ml.endswith((".htm", ".html")):
                    listed.append({"name": m, "role": "prose"})
                else:
                    if ml.endswith("versemap.xml") or b"<verseMap" in data:
                        why = "a verse map, which would renumber the reference; never read"
                    else:
                        why = "not a USFM book (.usfm, .sfm) or prose (.htm); never read"
                    problems.append(f"{m}: {why}")
                    listed.append({"name": m, "role": "refused"})
    elif low.endswith((".usfm", ".sfm")):
        out.append((name, open(path, "rb").read()))
        listed.append({"name": name, "role": "usfm"})
    else:
        problems.append(f"{name}: not a USFM reference (a .zip of USFM books, or a .usfm or .sfm file)")
    return out, listed


def read(path, declared=None, books=None):
    """The inventory of `path`. `declared`: the manifest's books (upper-case
    codes), or None for all."""
    books = books if books is not None else load_books()
    problems = []
    files, listed = members(path, problems)
    digest = hashlib.sha256()
    parsed, files_of = [], {}
    for (member, data), info in zip(files, [m for m in listed if m["role"] == "usfm"]):
        try:
            text = data.decode("utf-8")
            info["utf8"] = True
        except UnicodeDecodeError:
            text = data.decode("utf-8", "replace")
            info["utf8"] = False
            problems.append(f"{member}: not UTF-8 (read with replacement characters)")
        info["nfc"] = unicodedata.is_normalized("NFC", text)
        ids = text.count("\\id ") + text.count("\\id\t")
        if ids > 1:
            problems.append(f"{member}: {ids} \\id lines; one book per file")
        code, verses, outside = scan(as_usfm_defines(text))
        info.update(book=code, verses=len(verses), verses_before_chapter=outside)
        if code is not None and books.get(code) is not None:
            files_of[code] = files_of.get(code, 0) + 1
        parsed.append((member, code, verses, outside))
    entries = []
    before = before_outside = 0
    for member, code, verses, outside in parsed:
        c = code or ""
        for chapter, label in verses:
            digest.update(f"{member}\t{c}\t{chapter}\t{label}\n".encode("utf-8"))
            if not c:
                status, key = "book_unidentified", None
            elif declared is not None and c not in declared:
                status, key = "outside_declared_scope", None
            elif books.get(c) is not None:
                if files_of.get(c, 0) > 1:
                    status, key = "book_duplicate", None
                else:
                    n = chapter_number(chapter)
                    if n is None:
                        status, key = "chapter_label_unusual", None
                    elif n == 0:
                        status, key = "chapter_zero", None
                    elif n > books[c]:
                        status, key = "chapter_beyond_canon", None
                    else:
                        v, why = verse_key(label)
                        status, key = ("scorable", (c, n, v)) if why is None else (why, None)
            elif c in books:
                status, key = "book_outside_canon", None
            else:
                status, key = "book_code_unknown", None
            entries.append({"label": f"{c or '?'} {chapter}:{label}", "status": status, "key": key})
        if outside:
            digest.update(f"{member}\t#before\t{outside}\n".encode("utf-8"))
            if not c or declared is None or c in declared:
                before += outside
            else:
                before_outside += outside
    seen = {}
    for e in entries:
        if e["status"] == "scorable":
            seen[e["key"]] = seen.get(e["key"], 0) + 1
    for e in entries:
        if e["status"] == "scorable" and seen[e["key"]] > 1:
            e["status"] = "verse_duplicate"
    counts = {k: 0 for k in ("scorable", "outside_declared_scope") + UNSUPPORTED}
    for e in entries:
        counts[e["status"]] += 1
    counts["verse_before_chapter"] += before
    counts["outside_declared_scope"] += before_outside
    unsupported = {k: [e["label"] for e in entries if e["status"] == k] for k in UNSUPPORTED}
    order = {c: i for i, c in enumerate(books)}
    keys = sorted((e["key"] for e in entries if e["status"] == "scorable"),
                  key=lambda k: (order[k[0]], k[1], k[2]))
    return {
        "rules": RULES,
        "members": listed,
        "problems": problems,
        "label_digest": digest.hexdigest(),
        "labels": len(entries),
        "label_counts": counts,
        "unsupported_labels": unsupported,
        "verses_before_chapter": before,
        "scorable_ranges": ranges(keys),
        "outside_declared_books": sorted({e["label"].split(" ")[0] for e in entries
                                          if e["status"] == "outside_declared_scope"}),
    }


def ranges(keys):
    """Sorted (book, chapter, verse) keys as compact strings: "OBA 1:1-4"."""
    out, run = [], None
    for b, c, v in keys:
        if run and run[0] == b and run[1] == c and v == run[3] + 1:
            run[3] = v
            continue
        if run:
            out.append(f"{run[0]} {run[1]}:{run[2]}" + (f"-{run[3]}" if run[3] != run[2] else ""))
        run = [b, c, v, v]
    if run:
        out.append(f"{run[0]} {run[1]}:{run[2]}" + (f"-{run[3]}" if run[3] != run[2] else ""))
    return out


def expand(range_strings):
    """The keys of `ranges` output, as a set of (book, chapter, verse)."""
    keys = set()
    for r in range_strings:
        book, rest = r.split(" ")
        chapter, verses = rest.split(":")
        first, _, last = verses.partition("-")
        for v in range(int(first), int(last or first) + 1):
            keys.add((book, int(chapter), v))
    return keys


def compare(python, harness):
    """Differences between this inventory and the harness's report of the
    same reference (its `reference_inventory`); [] when they agree."""
    diffs = []
    if python["label_digest"] != harness.get("label_digest"):
        diffs.append("label digests differ")
    if python["labels"] != harness.get("labels"):
        diffs.append(f"labels: check {python['labels']}, harness {harness.get('labels')}")
    pc = python["label_counts"]
    hu = harness.get("unsupported", {})
    pairs = [("scorable", harness.get("printed_scorable", 0) + harness.get("absent_in_edition", 0)),
             ("outside_declared_scope", sum(harness.get("outside_declared_scope", {}).get(k, 0)
                                            for k in ("printed", "empty")))]
    pairs += [(k, hu.get(k, {}).get("printed", 0) + hu.get(k, {}).get("empty", 0)) for k in UNSUPPORTED]
    for k, h in pairs:
        if pc.get(k, 0) != h:
            diffs.append(f"{k}: check {pc.get(k, 0)}, harness {h}")
    if python["scorable_ranges"] != harness.get("scorable_ranges"):
        diffs.append("scorable verse keys differ")
    if len(python["problems"]) != len(harness.get("problems", [])):
        diffs.append(f"reference problems: check {len(python['problems'])}, harness {len(harness.get('problems', []))}")
    return diffs


def main():
    if len(sys.argv) not in (2, 3):
        sys.exit(__doc__)
    declared = set(sys.argv[2].split(",")) if len(sys.argv) == 3 else None
    print(json.dumps(read(sys.argv[1], declared), ensure_ascii=False, indent=1))


if __name__ == "__main__":
    main()
