#!/usr/bin/env python3
"""A neutral inventory of a reference edition, labels and text, rules
`aw-reference-inventory-2` (evaluation candidate A1.5-E3).

The check's own reading of a reference, written apart from the harness's
(`src-tauri/tests/eval_support/reference.rs`) and independent of the
recogniser's supported canon. Labels **and verse text** are read by the
check's own USFM reader (`usfm_text.py`, rules `aw-usfm-text-1`), which
shares nothing with the harness's Rust reader or with the app's USFM reader.
(E2's version of this file copied the app reader's regular expressions and
read labels only, so it could not see the app reader's errors; the
independent E2 review, R1.)

Every verse label gets a status:

  scorable                a book of the 66, a chapter within its count, a
                          verse 1..200 or a bridge N-M (stored on N), and no
                          verse number covered twice in its chapter
  outside_declared_scope  a book the manifest's `books` column leaves out
  one of UNSUPPORTED      kept and counted, never dropped

A number covered twice (a verse given twice, or a bridge N-M beside a verse
or bridge that also covers one of N..M) is ambiguous: every label involved
is `verse_duplicate`, and a verified reference to any number they cover
cannot be validated. The numbers N+1..M of a sound bridge are not verses of
their own: a verified reference to one of them cannot be validated either
(`inside_bridge`).

Every verse is also recorded with its text's SHA-256 (`verses`, in reading
order: member, book, chapter, label, status, line of `\\v`, text SHA-256),
and `text_digest` is the SHA-256 of those records. check-set.py records the
inventory in CHECK.json before the run (the per-verse records in a file of
the run folder); the scorer compares them with the harness's inventory,
verse by verse, and any difference makes the run not a protocol run.

A reference must be a .zip of USFM books (.usfm, .sfm), optionally with
prose (.htm, .html, never read), or one .usfm/.sfm file. Anything else in it
(a verse map, OSIS, MorphGNT, metadata, any other file), text that is not
UTF-8, a USFM structure the reader cannot read unambiguously (`problems`),
or verse text with a code point unassigned in this scorer's Unicode version
(`unicode_problems`) makes check-set.py refuse the set.

    reference_inventory.py <reference file> [BOOK,BOOK...]   prints the inventory
"""
import hashlib
import importlib.util
import json
import os
import sys
import unicodedata
import zipfile

RULES = "aw-reference-inventory-2"
HERE = os.path.dirname(os.path.abspath(__file__))
# Beside this script (a builder's flat copy of the tools), else in the repository.
BOOKS_TSV = next((p for p in (os.path.join(HERE, "usfm-books.tsv"),
                              os.path.normpath(os.path.join(HERE, "..", "..", "docs", "evaluation", "usfm-books.tsv")))
                  if os.path.isfile(p)), os.path.join(HERE, "usfm-books.tsv"))
_spec = importlib.util.spec_from_file_location("usfm_text", os.path.join(HERE, "usfm_text.py"))
usfm_text = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(usfm_text)

MAX_VERSE = 200
UNICODE_VERSION = unicodedata.unidata_version

UNSUPPORTED = ("verse_zero", "verse_label_unusual", "verse_duplicate", "verse_before_chapter",
               "chapter_zero", "chapter_label_unusual", "chapter_beyond_canon",
               "book_outside_canon", "book_code_unknown", "book_duplicate", "book_unidentified")
# What a scored document's reference holds, before the seal (E3, R2).
EVIDENCE_CLASSES = ("scorable", "numbered_but_empty", "unsupported_only", "outside_declared_books_only",
                    "no_verses")


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


def _number(s):
    """As Rust's u16 parse: ASCII digits, an optional leading '+', at most 65535."""
    body = s[1:] if s.startswith("+") else s
    if body and all("0" <= c <= "9" for c in body):
        n = int(body)
        return n if n <= 65535 else None
    return None


def verse_span(label):
    """(first, last) verse numbers the label covers, or why it cannot be
    scored. A bridge N-M (hyphen or en dash) is stored on N."""
    # At the first hyphen or en dash (U+2013), as Rust's split_once does.
    cut = min((label.find(d) for d in ("-", "\u2013") if d in label), default=-1)
    a, b = (label[:cut], label[cut + 1:]) if cut >= 0 else (label, None)
    first = _number(a)
    if first == 0:
        return None, "verse_zero"
    if b is None:
        return ((first, first), None) if first is not None and first <= MAX_VERSE else (None, "verse_label_unusual")
    last = _number(b)
    if first is not None and first <= MAX_VERSE and last is not None and last > first:
        return (first, last), None
    return None, "verse_label_unusual"


def verse_key(label):
    """The verse number the importer would store `label` under, or why not."""
    span, why = verse_span(label)
    return (span[0], None) if span else (None, why)


def chapter_number(label):
    return _number(label)


def members(path, problems):
    """[(member name, bytes)] of the USFM book files, in archive order, and
    the member list with roles and SHA-256."""
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
                sha = hashlib.sha256(data).hexdigest()
                if ml.endswith((".usfm", ".sfm")):
                    out.append((m, data))
                    listed.append({"name": m, "role": "usfm", "sha256": sha})
                elif ml.endswith((".htm", ".html")):
                    listed.append({"name": m, "role": "prose", "sha256": sha})
                else:
                    if ml.endswith("versemap.xml") or b"<verseMap" in data:
                        why = "a verse map, which would renumber the reference; never read"
                    else:
                        why = "not a USFM book (.usfm, .sfm) or prose (.htm); never read"
                    problems.append(f"{m}: {why}")
                    listed.append({"name": m, "role": "refused", "sha256": sha})
    elif low.endswith((".usfm", ".sfm")):
        data = open(path, "rb").read()
        out.append((name, data))
        listed.append({"name": name, "role": "usfm", "sha256": hashlib.sha256(data).hexdigest()})
    else:
        problems.append(f"{name}: not a USFM reference (a .zip of USFM books, or a .usfm or .sfm file)")
    return out, listed


def verse_record(member, book, chapter, label, status, line, text):
    """One verse as both inventories record it."""
    return [member, book, chapter, label, status, line, usfm_text.text_sha256(text)]


def text_digest(records):
    h = hashlib.sha256()
    for r in records:
        h.update(("\t".join(str(x) for x in r) + "\n").encode("utf-8"))
    return h.hexdigest()


def read(path, declared=None, books=None):
    """The inventory of `path`. `declared`: the manifest's books (upper-case
    codes), or None for all."""
    books = books if books is not None else load_books()
    problems, unicode_problems = [], []
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
        book = usfm_text.read_text(text)
        for p in book["problems"]:
            where = f" (line {p['line']})" if p["line"] else ""
            problems.append(f"{member}: {p['code']}{where}: {p['detail']}")
        code = book["book"]
        info.update(book=code, verses=len(book["verses"]), verses_before_chapter=book["verses_before_chapter"])
        if code is not None and books.get(code) is not None:
            files_of[code] = files_of.get(code, 0) + 1
        parsed.append((member, code, book))
    entries = []
    before = before_outside = 0
    for member, code, book in parsed:
        c = code or ""
        for v in book["verses"]:
            chapter, label, text = v["chapter"], v["label"], v["text"]
            digest.update(f"{member}\t{c}\t{chapter}\t{label}\n".encode("utf-8"))
            span = None
            if not c:
                status = "book_unidentified"
            elif declared is not None and c not in declared:
                status = "outside_declared_scope"
            elif books.get(c) is not None:
                if files_of.get(c, 0) > 1:
                    status = "book_duplicate"
                else:
                    n = chapter_number(chapter)
                    if n is None:
                        status = "chapter_label_unusual"
                    elif n == 0:
                        status = "chapter_zero"
                    elif n > books[c]:
                        status = "chapter_beyond_canon"
                    else:
                        span, why = verse_span(label)
                        status = "scorable" if why is None else why
                        if span:
                            span = (c, n, span[0], span[1])
            elif c in books:
                status = "book_outside_canon"
            else:
                status = "book_code_unknown"
            bad = sorted({f"U+{ord(ch):04X}" for ch in text if unicodedata.category(ch) == "Cn"})
            if bad:
                unicode_problems.append(f"{member} {c or '?'} {chapter}:{label}: code point(s) {', '.join(bad)} "
                                        f"unassigned in Unicode {UNICODE_VERSION}")
            entries.append({"member": member, "book": c, "chapter": chapter, "label": label, "line": v["line"],
                            "text": text, "status": status, "span": span, "alternate": v["alternate"]})
        if book["verses_before_chapter"]:
            outside = book["verses_before_chapter"]
            digest.update(f"{member}\t#before\t{outside}\n".encode("utf-8"))
            if not c or declared is None or c in declared:
                before += outside
            else:
                before_outside += outside
    # A verse number covered twice in a chapter (a duplicate, or a bridge
    # overlapping another label): every label involved is ambiguous.
    cover = {}
    for e in entries:
        if e["status"] == "scorable":
            b, n, first, last = e["span"]
            for k in range(first, last + 1):
                cover[(b, n, k)] = cover.get((b, n, k), 0) + 1
    unscorable, bridged = {}, {}
    for e in entries:
        if e["status"] == "scorable":
            b, n, first, last = e["span"]
            if any(cover[(b, n, k)] > 1 for k in range(first, last + 1)):
                e["status"] = "verse_duplicate"
                for k in range(first, last + 1):
                    unscorable[(b, n, k)] = "verse_duplicate"
            else:
                for k in range(first + 1, last + 1):
                    bridged[(b, n, k)] = "inside_bridge"
        elif e["status"] == "book_duplicate":
            n = chapter_number(e["chapter"])
            span, _ = verse_span(e["label"])
            if n is not None and span:
                for k in range(span[0], span[1] + 1):
                    unscorable[(e["book"], n, k)] = "book_duplicate"
    counts = {k: 0 for k in ("scorable", "outside_declared_scope") + UNSUPPORTED}
    printed = {k: 0 for k in counts}
    for e in entries:
        counts[e["status"]] += 1
        if e["text"].strip(usfm_text.USFM_SPACE):
            printed[e["status"]] += 1
    counts["verse_before_chapter"] += before
    printed["verse_before_chapter"] += before
    counts["outside_declared_scope"] += before_outside
    printed["outside_declared_scope"] += before_outside
    unsupported = {k: [f"{e['book'] or '?'} {e['chapter']}:{e['label']}" for e in entries if e["status"] == k]
                   for k in UNSUPPORTED}
    order = {c: i for i, c in enumerate(books)}
    keys = sorted(((e["book"], int(e["chapter"]), e["span"][2]) for e in entries if e["status"] == "scorable"),
                  key=lambda k: (order[k[0]], k[1], k[2]))
    records = [verse_record(e["member"], e["book"], e["chapter"], e["label"], e["status"], e["line"], e["text"])
               for e in entries]
    printed_unsupported = sum(printed[k] for k in UNSUPPORTED)
    if printed["scorable"]:
        evidence = "scorable"
    elif counts["scorable"]:
        evidence = "numbered_but_empty"
    elif printed_unsupported:
        evidence = "unsupported_only"
    elif printed["outside_declared_scope"]:
        evidence = "outside_declared_books_only"
    else:
        evidence = "no_verses"
    return {
        "rules": RULES,
        "reader_rules": usfm_text.RULES,
        "unicode_version": UNICODE_VERSION,
        "members": listed,
        "problems": problems,
        "unicode_problems": unicode_problems,
        "label_digest": digest.hexdigest(),
        "text_digest": text_digest(records),
        "labels": len(entries),
        "label_counts": counts,
        "printed_counts": printed,
        "printed_scorable": printed["scorable"],
        "absent_in_edition": counts["scorable"] - printed["scorable"],
        "printed_unsupported": printed_unsupported,
        "evidence_class": evidence,
        "printed_scorable_by_book": {b: sum(1 for e in entries if e["status"] == "scorable" and e["book"] == b
                                            and e["text"].strip(usfm_text.USFM_SPACE))
                                     for b in sorted({e["book"] for e in entries if e["status"] == "scorable"},
                                                     key=lambda b: order[b])},
        "unsupported_labels": unsupported,
        "verses_before_chapter": before,
        "scorable_ranges": ranges(keys),
        "unscorable_keys": {f"{b} {n}:{k}": why for (b, n, k), why in sorted(unscorable.items())},
        "bridge_member_ranges": ranges(sorted(bridged, key=lambda k: (order.get(k[0], 999), k[1], k[2]))),
        "outside_declared_books": sorted({e["book"] for e in entries if e["status"] == "outside_declared_scope"}),
        "verses": records,
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


def compare(python, harness, check_verses=None):
    """Differences between this inventory and the harness's report of the
    same reference (its `reference_inventory`); [] when they agree.
    `check_verses`: this inventory's per-verse records (when they are kept
    apart from `python`, as in CHECK.json)."""
    diffs = []
    if harness.get("rules") != RULES:
        diffs.append(f"harness inventory rules {harness.get('rules')!r}, not {RULES!r}")
    if harness.get("reader_rules") != usfm_text.RULES:
        diffs.append(f"harness reader rules {harness.get('reader_rules')!r}, not {usfm_text.RULES!r}")
    if python["label_digest"] != harness.get("label_digest"):
        diffs.append("label digests differ")
    if python["text_digest"] != harness.get("text_digest"):
        diffs.append("verse text digests differ")
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
    if python["printed_scorable"] != harness.get("printed_scorable"):
        diffs.append(f"printed scorable: check {python['printed_scorable']}, harness {harness.get('printed_scorable')}")
    if python["scorable_ranges"] != harness.get("scorable_ranges"):
        diffs.append("scorable verse keys differ")
    if python.get("bridge_member_ranges", []) != harness.get("bridge_member_ranges", []):
        diffs.append("verse numbers inside bridges differ")
    if len(python["problems"]) != len(harness.get("problems", [])):
        diffs.append(f"reference problems: check {len(python['problems'])}, harness {len(harness.get('problems', []))}")
    mine = check_verses if check_verses is not None else python.get("verses")
    theirs = harness.get("verses")
    if mine is None or theirs is None:
        diffs.append("per-verse text records missing")
    else:
        differing = [f"{a[1]} {a[2]}:{a[3]}" for a, b in zip(mine, theirs) if list(a) != list(b)]
        if len(mine) != len(theirs):
            diffs.append(f"per-verse records: check {len(mine)}, harness {len(theirs)}")
        if differing:
            diffs.append(f"{len(differing)} verse(s) read differently (label, status, line or text): "
                         + ", ".join(differing[:10]) + (", ..." if len(differing) > 10 else ""))
    return diffs


def main():
    if len(sys.argv) not in (2, 3):
        sys.exit(__doc__)
    declared = set(sys.argv[2].split(",")) if len(sys.argv) == 3 else None
    print(json.dumps(read(sys.argv[1], declared), ensure_ascii=False, indent=1))


if __name__ == "__main__":
    main()
