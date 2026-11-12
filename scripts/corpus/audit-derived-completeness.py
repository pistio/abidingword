#!/usr/bin/env python3
"""Source-to-document completeness audit for a USFM-derived Word document.

    audit-derived-completeness.py <id>_usfm.zip <document.docx> [--json OUT]

Checks that every word of every verse of the edition's USFM is in the
derived DOCX, in order, under the same book, chapter and verse. It shares
no code with the generator (scripts/build-derived-editions.py): the USFM is
read marker by marker here, not line by line, and the DOCX is read from
word/document.xml.

What is NOT verse text, and so excluded on the USFM side (each counted in
the report, by marker, so that nothing is excluded silently):

* notes: \\f … \\f*, \\fe … \\fe*, \\x … \\x*;
* attributes of \\w word|attributes\\w* (the word is kept);
* alternative and published numbers: \\va, \\vp, \\ca, \\cp;
* headings, titles and introductions (HEADINGS below): their text is not a
  verse's, and a verse goes on after them.

Every other marker's text after a \\v is the verse's, whatever the marker:
a paragraph marker this script has never heard of counts as verse text, so
a generator that drops it is caught.

The only normalisation: white space. Words are compared exactly.
Exit status 0 only when every verse is complete.
"""
import collections
import difflib
import hashlib
import json
import re
import sys
import zipfile
import xml.etree.ElementTree as ET

ORDER = (
    "GEN EXO LEV NUM DEU JOS JDG RUT 1SA 2SA 1KI 2KI 1CH 2CH EZR NEH EST JOB PSA "
    "PRO ECC SNG ISA JER LAM EZK DAN HOS JOL AMO OBA JON MIC NAM HAB ZEP HAG ZEC "
    "MAL MAT MRK LUK JHN ACT ROM 1CO 2CO GAL EPH PHP COL 1TH 2TH 1TI 2TI TIT PHM "
    "HEB JAS 1PE 2PE 1JN 2JN 3JN JUD REV"
).split()

# Paragraph-level markers whose text is a heading, title, label or
# introduction, never a verse's own words (USFM 3 reference).
HEADINGS = set(
    "id ide usfm sts rem h toc1 toc2 toc3 toca1 toca2 toca3 "
    "mt mt1 mt2 mt3 mt4 mte mte1 mte2 ms ms1 ms2 ms3 mr s s1 s2 s3 s4 sr r rq d sp sd sd1 sd2 "
    "cl cd qa "
    "imt imt1 imt2 imt3 is is1 is2 ip ipi im imi ipq imq ipr iq iq1 iq2 iq3 ib ili ili1 ili2 "
    "iot io io1 io2 io3 io4 iex imte imte1 ie periph".split()
)
# Paragraph-level markers that hold verse text (listed for the report only:
# an unlisted marker is treated the same way and named "unlisted").
VERSE_PARAGRAPHS = set(
    "p m po pr cls pmo pm pmc pmr pi pi1 pi2 pi3 mi nb pc ph ph1 ph2 ph3 b "
    "q q1 q2 q3 q4 qr qc qm qm1 qm2 qm3 qd lh li li1 li2 li3 li4 lf lim lim1 lim2 "
    "tr lit".split()
)
NOTES = [("f", r"\\f\s.*?\\f\*"), ("fe", r"\\fe\s.*?\\fe\*"), ("x", r"\\x\s.*?\\x\*")]
NUMBERS = [("va", r"\\va\s.*?\\va\*"), ("vp", r"\\vp\s.*?\\vp\*"), ("ca", r"\\ca\s.*?\\ca\*")]


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def read_usfm(path):
    """{book: {(chapter, verse label): [(marker, word), …]}}, titles by
    book, and what was excluded, by marker."""
    z = zipfile.ZipFile(path)
    books, titles = {}, {}
    excluded = collections.Counter()
    verse_words_by_marker = collections.Counter()
    for name in sorted(z.namelist()):
        text = z.read(name).decode("utf-8-sig", errors="replace")
        m = re.search(r"\\id\s+([A-Z0-9]{3})", text)
        if not m or m.group(1) not in ORDER:
            continue
        code = m.group(1)
        for label, pattern in NOTES + NUMBERS:
            text, n = re.subn(pattern, " " if label in ("va", "vp", "ca") else "", text, flags=re.S)
            excluded["\\" + label + " (occurrences)"] += n
        text, n = re.subn(r"(\\\+?w\s[^|\\]*)\|[^\\]*(\\\+?w\*)", r"\1\2", text)
        excluded["\\w attributes (occurrences)"] += n
        verses = collections.OrderedDict()
        chapter, verse, paragraph, heading = None, None, None, False
        pieces = re.split(r"(\\\+?[a-z]+\d*\*?)", text)
        i = 0
        while i < len(pieces):
            piece = pieces[i]
            i += 1
            if not piece.startswith("\\"):
                # The white space right after an opening marker ends the
                # marker; it is not text ("\\w word|…\\w*:" reads "word:").
                opened = i >= 2 and pieces[i - 2].startswith("\\") and not pieces[i - 2].endswith("*")
                if opened and piece[:1].isspace():
                    piece = piece[1:]
                words = piece.split()
                if heading or verse is None:
                    if words:
                        excluded[f"\\{paragraph} (words, {'heading' if heading else 'before a verse'})"] += len(words)
                elif piece:
                    verses.setdefault((chapter, verse), []).append((paragraph, piece))
                continue
            marker = piece[1:]
            if marker.endswith("*") or marker.startswith("+"):
                continue  # a character style's end, or a nested one: its words stay
            if marker == "c":
                number, _, rest = pieces[i].lstrip().partition("\n") if i < len(pieces) else ("", "", "")
                chapter = number.split()[0] if number.split() else None
                pieces[i] = rest if i < len(pieces) else ""
                verse, paragraph, heading = None, "c", False
            elif marker == "cp":
                number, _, rest = pieces[i].lstrip().partition("\n")
                excluded["\\cp (occurrences)"] += 1
                pieces[i] = rest
            elif marker == "v":
                number, _, rest = pieces[i].lstrip().partition(" ") if i < len(pieces) else ("", "", "")
                if "\n" in number:
                    number, _, more = number.partition("\n")
                    rest = more + " " + rest
                verse = number.strip()
                verses.setdefault((chapter, verse), [])
                pieces[i] = rest
                heading = False
                if paragraph in HEADINGS or paragraph is None:
                    paragraph = "p"
            elif marker == "h":
                title, _, rest = pieces[i].partition("\n")
                titles[code] = title.strip()
                excluded["\\h (words, heading)"] += len(title.split())
                pieces[i] = rest
            elif marker in HEADINGS:
                paragraph, heading = marker, True
                if verse is not None:
                    verses.setdefault((chapter, verse), []).append((paragraph, " "))
            elif marker in VERSE_PARAGRAPHS or is_paragraph_marker(pieces, i - 1):
                paragraph, heading = marker, False
                if verse is not None:
                    verses.setdefault((chapter, verse), []).append((paragraph, " "))
            # else: a character style (\wj, \nd, \bk, \w, \sc …): its words
            # belong where they stand.
        # Each verse's words, each with the paragraph marker it begins under.
        worded = collections.OrderedDict()
        for key, chunks in verses.items():
            text = "".join(c for _, c in chunks)
            owner = [m for m, c in chunks for _ in c]
            tagged = [(owner[w.start()], w.group()) for w in re.finditer(r"\S+", text)]
            for marker, _ in tagged:
                kind = marker if marker in VERSE_PARAGRAPHS else f"{marker} (unlisted)"
                verse_words_by_marker["\\" + kind] += 1
            worded[key] = tagged
        books[code] = worded
    return books, titles, excluded, verse_words_by_marker


def is_paragraph_marker(pieces, at):
    """A marker this script does not list, standing at the start of a line:
    a paragraph marker. (Character styles this script does not list either
    stand inside a line.)"""
    before = pieces[at - 1] if at > 0 else "\n"
    return before == "" and at <= 1 or before.rstrip(" \t").endswith("\n")


W = "{http://schemas.openxmlformats.org/wordprocessingml/2006/main}"


def read_docx(path, chapter_number):
    """{book title: {(chapter, verse label): [word, …]}} in document order,
    from word/document.xml: Heading 1 = book, Heading 2 = chapter label,
    a raised run (superscript or raised position) = a verse number; a paragraph wholly bold
    and centred, or wholly italic, is a heading as the generator writes
    them (\\s, \\d)."""
    z = zipfile.ZipFile(path)
    root = ET.fromstring(z.read("word/document.xml"))
    styles = ET.fromstring(z.read("word/styles.xml"))
    heading_of = {}
    for s in styles.iter(W + "style"):
        name = s.find(W + "name")
        if name is not None and re.fullmatch(r"(?i)heading [12]", name.get(W + "val", "")):
            heading_of[s.get(W + "styleId")] = int(name.get(W + "val")[-1])
    books = collections.OrderedDict()
    headings = collections.Counter()
    book = chapter = verse = None
    for p in root.iter(W + "p"):
        ppr = p.find(W + "pPr")
        style = ppr.find(W + "pStyle").get(W + "val") if ppr is not None and ppr.find(W + "pStyle") is not None else None
        runs = []
        for r in p.iter(W + "r"):
            text = "".join(t.text or "" for t in r.iter(W + "t"))
            if r.find(W + "tab") is not None or r.find(W + "br") is not None:
                text += " "
            rpr = r.find(W + "rPr")
            def on(tag):
                e = rpr.find(W + tag) if rpr is not None else None
                return e is not None and e.get(W + "val", "true") not in ("false", "0")
            # Raised: Word's superscript, or LibreOffice's raised position.
            align = rpr.find(W + "vertAlign") if rpr is not None else None
            position = rpr.find(W + "position") if rpr is not None else None
            raised = (align is not None and align.get(W + "val") == "superscript") or \
                (position is not None and int(position.get(W + "val", "0")) > 0)
            if text:
                runs.append((text, raised, on("b"), on("i")))
        whole = "".join(t for t, *_ in runs).strip()
        if not whole:
            continue
        level = heading_of.get(style)
        if level == 1:
            book = whole
            books[book] = collections.OrderedDict()
            chapter = verse = None
            continue
        if level == 2:
            chapter, verse = chapter_number(whole), None
            continue
        centred = ppr is not None and ppr.find(W + "jc") is not None and ppr.find(W + "jc").get(W + "val") == "center"
        lettered = [r for r in runs if r[0].strip()]
        if lettered and all(r[2] for r in lettered) and centred:
            headings["bold centred paragraph (\\s)"] += len(whole.split())
            continue
        if lettered and all(r[3] for r in lettered) and not any(r[1] for r in lettered):
            headings["italic paragraph (\\d)"] += len(whole.split())
            continue
        pending = ""
        for text, raised, _, _ in runs + [("", True, False, False)]:
            if raised:
                if pending.split() and book is not None:
                    if verse is None:
                        headings["text before a verse number"] += len(pending.split())
                    else:
                        books[book].setdefault((chapter, verse), []).extend(pending.split())
                pending = ""
                if text.strip():
                    verse = text.strip()
                    if book is not None:
                        books[book].setdefault((chapter, verse), [])
            else:
                pending += text
    return books, headings


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    out = next((a.split("=", 1)[1] for a in sys.argv[1:] if a.startswith("--json=")), None)
    usfm_path, docx_path = args
    source, titles, excluded, by_marker = read_usfm(usfm_path)

    def chapter_number(label):
        numbers = re.findall(r"\d+", label)
        return numbers[0] if numbers else label

    document, document_headings = read_docx(docx_path, chapter_number)
    title_of = {code: titles.get(code, code) for code in source}
    report = {
        "source_usfm": {"file": usfm_path.split("/")[-1], "sha256": sha256(usfm_path)},
        "document": {"file": docx_path.split("/")[-1], "sha256": sha256(docx_path)},
        "books_in_source": len(source),
        "books_in_document": len(document),
        "verses_in_source": 0,
        "verses_complete": 0,
        "verses_missing_from_document": [],
        "verses_only_in_document": [],
        "verses_incomplete": [],
        "words_in_source_verses": 0,
        "words_missing": 0,
        "words_missing_by_source_marker": {},
        "source_verse_words_by_paragraph_marker": dict(sorted(by_marker.items())),
        "source_excluded": dict(sorted(excluded.items())),
        "document_headings_excluded_words": dict(document_headings),
    }
    missing_by_marker = collections.Counter()
    document_titles = list(document)
    for code in ORDER:
        if code not in source:
            continue
        got = document.get(title_of[code])
        if got is None:
            report["verses_missing_from_document"].append(f"{code} (book: no heading “{title_of[code]}”)")
            report["verses_in_source"] += len(source[code])
            continue
        for key, tagged in source[code].items():
            report["verses_in_source"] += 1
            want = [w for _, w in tagged]
            report["words_in_source_verses"] += len(want)
            ref = f"{code} {key[0]}:{key[1]}"
            if key not in got:
                report["verses_missing_from_document"].append(ref)
                report["words_missing"] += len(want)
                continue
            have = got[key]
            if want == have:
                report["verses_complete"] += 1
                continue
            if "".join(want) == "".join(have):
                # The same letters, spaced differently where two pieces meet.
                report["verses_complete"] += 1
                report.setdefault("verses_complete_but_spaced_differently", []).append(ref)
                continue
            matcher = difflib.SequenceMatcher(a=want, b=have, autojunk=False)
            lost, added = [], []
            for op, a1, a2, b1, b2 in matcher.get_opcodes():
                if op in ("delete", "replace"):
                    lost.append(" ".join(want[a1:a2]))
                    for marker, _ in tagged[a1:a2]:
                        missing_by_marker["\\" + str(marker)] += 1
                        report["words_missing"] += 1
                if op in ("insert", "replace"):
                    added.append(" ".join(have[b1:b2]))
            report["verses_incomplete"].append(
                {"reference": ref, "missing": lost, "not_in_source": added,
                 "source": " ".join(want), "document": " ".join(have)})
        for key in got:
            if key not in source[code]:
                report["verses_only_in_document"].append(f"{code} {key[0]}:{key[1]}")
    for title in document_titles:
        if title not in title_of.values():
            report["verses_only_in_document"].append(f"(book heading “{title}”)")
    report["words_missing_by_source_marker"] = dict(missing_by_marker)
    complete = (not report["verses_incomplete"] and not report["verses_missing_from_document"]
                and not report["verses_only_in_document"])
    report["complete"] = complete
    if out:
        with open(out, "w", encoding="utf-8") as f:
            json.dump(report, f, ensure_ascii=False, indent=1)
    summary = {k: (v if not isinstance(v, list) else len(v)) for k, v in report.items()
               if k not in ("source_verse_words_by_paragraph_marker", "source_excluded")}
    print(json.dumps(summary, ensure_ascii=False, indent=1))
    sys.exit(0 if complete else 1)


if __name__ == "__main__":
    main()
