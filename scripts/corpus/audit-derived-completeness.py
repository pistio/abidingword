#!/usr/bin/env python3
"""Source-to-document completeness audit for a USFM-derived Word document.

    audit-derived-completeness.py <id>_usfm.zip <document.docx>
        [--json=OUT] [--profile=NAME] [--books=66]

Exit status 0 and "complete": true only when, book by book, the document
holds the source's chapters, verse numbers and words as ONE ordered
stream equal to the source's: nothing missing, added, moved, merged,
split or re-ordered, under the same chapter and verse.

It shares no code with the generator (scripts/build-derived-editions.py).
The USFM is read marker by marker; the DOCX from word/document.xml.

Version 2 (Phase A1.5), after an independent adversarial review of
version 1 found 33 ways a damaged document passed. The rule now: whatever
this script does not positively understand FAILS the audit.

What is compared
----------------
Per book, a stream of tokens: each chapter, each verse number, each word.
A word is tagged T (a verse's or paragraph's text) or H (a heading). The
two streams must be equal, token for token, and

* a source H word must stand in a document paragraph set as a heading
  (wholly bold and centred, or wholly italic, without a verse number);
* a source T word may stand in any paragraph of the document.

White space is the only normalisation. Words are compared exactly.

What is not compared, and is counted by marker in the report
------------------------------------------------------------
* identification that is never printed: \\id \\ide \\usfm \\sts \\rem \\toc* \\toca*;
  \\h is the book's heading and must equal the document's;
* notes and figures: \\f \\fe \\ef \\x \\ex \\fig (each must be closed, and must
  not run over a verse or chapter marker);
* inline cross-references \\rq … \\rq*;
* alternative and published numbers: \\va \\vp \\ca (closed), \\cp;
* attributes after "|" inside a character marker, and milestones;
* with --profile, the markers that generator is DECLARED not to print.
  The default profile omits nothing. Every omitted word is counted, and
  those inside a verse are listed by verse.
* with --books=66, source books outside the 66-book list, DECLARED not
  rendered and listed with their verse counts. By default every book in
  the source is audited.

What fails outright
-------------------
No book or no verse in the source; a file that is not UTF-8; a USFM-named
file without one valid \\id; a repeated book; an unclosed or overrunning
note; \\v or \\c without a number; words on a \\c line; hidden, struck,
white or tiny text; tracked changes; text boxes, alternate content,
symbols, special hyphens, fields; a raised run that is not a verse
number; a chapter heading without exactly one number.
"""
import collections
import difflib
import hashlib
import json
import re
import sys
import zipfile
import xml.etree.ElementTree as ET

VERSION = 2

ORDER = (
    "GEN EXO LEV NUM DEU JOS JDG RUT 1SA 2SA 1KI 2KI 1CH 2CH EZR NEH EST JOB PSA "
    "PRO ECC SNG ISA JER LAM EZK DAN HOS JOL AMO OBA JON MIC NAM HAB ZEP HAG ZEC "
    "MAL MAT MRK LUK JHN ACT ROM 1CO 2CO GAL EPH PHP COL 1TH 2TH 1TI 2TI TIT PHM "
    "HEB JAS 1PE 2PE 1JN 2JN 3JN JUD REV"
).split()

# Never printed.
IDENTIFICATION = set("id ide usfm sts rem toc1 toc2 toc3 toca1 toca2 toca3".split())
# Titles and introductions: a book's front, before its first chapter.
FRONT = set(
    "mt mt1 mt2 mt3 mt4 mte mte1 mte2 imt imt1 imt2 imt3 is is1 is2 ip ipi im imi ipq imq ipr "
    "iq iq1 iq2 iq3 ib ili ili1 ili2 iot io io1 io2 io3 io4 iex imte imte1 ie periph".split()
)
# Headings inside a book's chapters.
HEADINGS = set("ms ms1 ms2 ms3 mr s s1 s2 s3 s4 sr r d sp sd sd1 sd2 cl cd qa".split())
# Paragraphs of text (listed for the report only: any other paragraph
# marker is text too, and named "unlisted").
TEXT_PARAGRAPHS = set(
    "p m po pr cls pmo pm pmc pmr pi pi1 pi2 pi3 mi nb pc ph ph1 ph2 ph3 b "
    "q q1 q2 q3 q4 qr qc qm qm1 qm2 qm3 qd lh li li1 li2 li3 li4 lf lim lim1 lim2 "
    "tr lit".split()
)
CLOSED_EXCLUSIONS = ["f", "fe", "ef", "x", "ex", "fig", "rq", "va", "vp", "ca"]

# What a generator is declared not to print. Named, so that a report says
# which declaration it was audited under.
PROFILES = {
    "strict": {"front": set(), "headings": set()},
    # scripts/build-derived-editions.py --usfm: prints \h, \c, \s \s1 \s2, \d
    # and verse paragraphs; nothing else.
    "build-derived-editions-usfm": {
        "front": set(FRONT),
        "headings": HEADINGS - {"s", "s1", "s2", "d"},
    },
}


class Failure(Exception):
    pass


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


# ---------------------------------------------------------------- source

def read_usfm(path, profile, only66, report):
    """[(code, title, tokens)] in the order of ORDER then of the archive;
    tokens: ("C", n) | ("V", (chapter, label)) | ("W", word, tag, marker)."""
    z = zipfile.ZipFile(path)
    found = {}
    order = []
    for name in z.namelist():
        if name.endswith("/"):
            continue
        raw = z.read(name)
        usfm_named = re.search(r"\.(usfm|sfm|usx?)$", name, re.I) is not None
        try:
            text = raw.decode("utf-8-sig")
        except UnicodeDecodeError:
            if usfm_named or b"\\id" in raw or b"\\\x00i\x00d" in raw:
                raise Failure(f"{name}: not UTF-8")
            report["source_files_ignored"].append(name)
            continue
        ids = re.findall(r"\\id\b[ \t]*(\S*)", text)
        if not ids:
            if usfm_named or re.search(r"\\[cv]\b", text):
                raise Failure(f"{name}: no \\id")
            report["source_files_ignored"].append(name)
            continue
        if len(ids) != 1:
            raise Failure(f"{name}: {len(ids)} \\id markers in one file")
        code = ids[0]
        if not re.fullmatch(r"[A-Z0-9]{3}", code):
            raise Failure(f"{name}: \\id {code!r} is not a book code")
        if code in found or code in report["source_books_declared_not_rendered"]:
            raise Failure(f"{name}: book {code} a second time")
        if only66 and code not in ORDER:
            verses = len(re.findall(r"\\v\s", text))
            report["source_books_declared_not_rendered"][code] = verses
            continue
        found[code] = (name, text)
        order.append(code)
    first = {c: n for n, c in enumerate(order)}
    order.sort(key=lambda c: (ORDER.index(c) if c in ORDER else len(ORDER), first[c]))
    books = []
    for code in order:
        name, text = found[code]
        title, tokens = read_book(code, name, text, profile, report)
        books.append((code, title, tokens))
    if not books:
        raise Failure("the source has no book")
    return books


def strip_closed(text, name, excluded, where):
    """Removes every \\name … \\name*; fails if one is unclosed or runs over
    a verse, a chapter or another opener."""
    opener = re.compile(r"\\" + name + r"(?![a-z0-9*-])")
    closer = "\\" + name + "*"
    out, at, n = [], 0, 0
    while True:
        m = opener.search(text, at)
        if not m:
            break
        end = text.find(closer, m.end())
        if end < 0:
            raise Failure(f"{where}: \\{name} is never closed")
        inside = text[m.end():end]
        if re.search(r"\\[vc]\b", inside) or opener.search(inside):
            raise Failure(f"{where}: a \\{name} … \\{name}* runs over a marker it cannot hold")
        out.append(text[at:m.start()])
        # A number or reference standing between words leaves a space;
        # a note leaves nothing (the words around it keep their own).
        out.append(" " if name in ("va", "vp", "ca", "rq") else "")
        at = end + len(closer)
        n += 1
    out.append(text[at:])
    if closer in "".join(out):
        raise Failure(f"{where}: a \\{name}* closes nothing")
    excluded[f"\\{name} (occurrences)"] += n
    return "".join(out)


def read_book(code, name, text, profile, report):
    excluded = report["source_excluded"]
    for marker in CLOSED_EXCLUSIONS:
        text = strip_closed(text, marker, excluded, f"{name} ({code})")
    # Milestones: \qt-s |attributes\*  and  \qt-e\*
    text, n = re.subn(r"\\[a-z0-9]+-[se]\b[^\\]*\\\*", "", text)
    excluded["milestones (occurrences)"] += n
    if "\\*" in text:
        raise Failure(f"{name} ({code}): a milestone end \\* without a milestone")
    # Attributes of a character marker: word|attributes\w*
    text, n = re.subn(r"\|[^\\\n]*(?=\\\+?[a-z0-9]+\*)", "", text)
    excluded["attributes (occurrences)"] += n
    pieces = re.split(r"(\\\+?[a-z]+\d*\*?)", text)
    markers = [p[1:] for p in pieces[1::2]]
    closers = collections.Counter(m[:-1].lstrip("+") for m in markers if m.endswith("*"))
    tokens = []
    title = None
    chapter = verse = None
    paragraph, tag = None, None  # tag: "T", "H", or None (not compared)
    i = 0
    # Text gathers here until the next structural marker: a word may run
    # across a character marker ("\w word|…\w*," is "word,").
    pending = []

    def flush():
        text = "".join(pending)
        pending.clear()
        controls = sum(1 for c in text if ord(c) < 32 and not c.isspace())
        if controls:
            excluded["control characters (not printable)"] += controls
            text = "".join(c for c in text if ord(c) >= 32 or c.isspace())
        words = text.split()
        if not words:
            return
        if tag is None:
            excluded[f"\\{paragraph} (words)"] += len(words)
            if verse is not None:
                report["omitted_heading_words_inside_verses"].setdefault(
                    f"{code} {chapter}:{verse}", []).append(f"\\{paragraph}: {' '.join(words)}")
            return
        if chapter is None and tag == "T":
            raise Failure(f"{name} ({code}): text before the first chapter under \\{paragraph}")
        kind = paragraph if (tag == "H" or paragraph in TEXT_PARAGRAPHS) else f"{paragraph} (unlisted)"
        report["source_words_by_marker"][f"\\{kind}"] += len(words)
        if tag == "T" and verse is None:
            report["source_text_words_before_a_verse"] += len(words)
        tokens.extend(("W", w, tag, paragraph) for w in words)

    def take_number(what):
        """The number after \\c or \\v: the next run of non-space."""
        nonlocal i
        rest = pieces[i] if i < len(pieces) else ""
        m = re.match(r"[ \t]+(\S+)", rest)
        if not m:
            raise Failure(f"{name} ({code}): \\{what} without a number near {rest[:30]!r}")
        pieces[i] = rest[m.end():]
        return m.group(1)

    while i < len(pieces):
        piece = pieces[i]
        i += 1
        if not piece.startswith("\\"):
            # The white space right after an opening marker ends the marker.
            opened = i >= 2 and pieces[i - 2].startswith("\\") and not pieces[i - 2].endswith("*")
            if opened and piece[:1].isspace():
                piece = piece[1:]
            pending.append(piece)
            continue
        marker = piece[1:]
        if marker.endswith("*") or marker.startswith("+"):
            continue  # a character style's end, or a nested one: its words stay
        if closers[marker] > 0 and marker not in ("c", "v", "h", "cp"):
            continue  # a character style (\wj, \nd, \bk …): its words stay where they stand
        flush()
        pending.append(" ")
        if re.fullmatch(r"[cv]\d+", marker):
            raise Failure(f"{name} ({code}): \\{marker}: a number run into its marker")
        if marker == "c":
            chapter = take_number("c")
            if pieces[i].split("\n", 1)[0].strip():
                raise Failure(f"{name} ({code}): words on the \\c {chapter} line")
            verse, paragraph, tag = None, "c", "T"
            tokens.append(("C", chapter))
        elif marker == "cp":
            line, _, rest = pieces[i].partition("\n")
            excluded["\\cp (occurrences)"] += 1
            pieces[i] = "\n" + rest
        elif marker == "v":
            if chapter is None:
                raise Failure(f"{name} ({code}): \\v before any \\c")
            verse = take_number("v")
            tokens.append(("V", (chapter, verse)))
            report["verses_in_source"] += 1
            if tag != "T":
                paragraph, tag = "p", "T"
        elif marker == "h":
            line, _, rest = pieces[i].partition("\n")
            if title is not None:
                raise Failure(f"{name} ({code}): a second \\h")
            title = " ".join(line.split())
            pieces[i] = "\n" + rest
        elif marker in IDENTIFICATION:
            line, _, rest = pieces[i].partition("\n")
            excluded[f"\\{marker} (words, never printed)"] += len(line.split())
            pieces[i] = "\n" + rest
        elif marker in FRONT:
            if chapter is not None:
                # An introduction marker inside the chapters is not a
                # front: its words are text.
                paragraph, tag = marker, "T"
            else:
                paragraph = marker
                tag = None if marker in profile["front"] else "H"
        elif marker in HEADINGS:
            paragraph = marker
            tag = None if marker in profile["headings"] else "H"
        elif marker in TEXT_PARAGRAPHS:
            paragraph, tag = marker, "T"
        else:
            # Unknown, never closed: a paragraph marker, and its words text.
            paragraph, tag = marker, "T"
    flush()
    if title is None:
        raise Failure(f"{name} ({code}): no \\h (the book's heading)")
    if not any(t[0] == "V" for t in tokens):
        raise Failure(f"{name} ({code}): no verse")
    return title, tokens


# -------------------------------------------------------------- document

W = "{http://schemas.openxmlformats.org/wordprocessingml/2006/main}"
UNSUPPORTED = {
    "ins": "tracked insertion", "del": "tracked deletion", "moveFrom": "tracked move",
    "moveTo": "tracked move", "txbxContent": "text box", "sym": "symbol character",
    "noBreakHyphen": "non-breaking hyphen element", "softHyphen": "soft hyphen element",
    "fldSimple": "field", "instrText": "field", "delText": "deleted text",
    "object": "embedded object", "ruby": "ruby text",
}


def on(rpr, tag):
    e = rpr.find(W + tag) if rpr is not None else None
    return e is not None and e.get(W + "val", "true") not in ("false", "0", "none")


def read_docx(path, report):
    """[(title, tokens)] in document order; tokens as in the source, with
    the word's paragraph kind ("H" or "T") as its tag."""
    z = zipfile.ZipFile(path)
    try:
        body = z.read("word/document.xml")
    except KeyError:
        raise Failure("the document has no word/document.xml")
    if b"AlternateContent" in body:
        report["document_unsupported"]["alternate content"] += body.count(b"AlternateContent")
    root = ET.fromstring(body)
    heading_of, style_rpr, based_on = {}, {}, {}
    if "word/styles.xml" in z.namelist():
        for s in ET.fromstring(z.read("word/styles.xml")).iter(W + "style"):
            sid = s.get(W + "styleId")
            name = s.find(W + "name")
            if name is not None and re.fullmatch(r"(?i)heading [12]", name.get(W + "val", "")):
                heading_of[sid] = int(name.get(W + "val")[-1])
            style_rpr[sid] = s.find(W + "rPr")
            b = s.find(W + "basedOn")
            based_on[sid] = b.get(W + "val") if b is not None else None

    def chain(sid):
        seen = set()
        while sid and sid not in seen:
            seen.add(sid)
            yield style_rpr.get(sid)
            sid = based_on.get(sid)

    for tag, why in UNSUPPORTED.items():
        n = sum(1 for _ in root.iter(W + tag))
        if n:
            report["document_unsupported"][why] += n

    def properties(r, paragraph_style):
        """The run's own properties, then its character style's, then the
        paragraph style's: the first that says so decides."""
        own = r.find(W + "rPr")
        rstyle = None
        if own is not None and own.find(W + "rStyle") is not None:
            rstyle = own.find(W + "rStyle").get(W + "val")
        return [own] + list(chain(rstyle)) + list(chain(paragraph_style))

    def raised(props):
        for p in props:
            if p is None:
                continue
            a, pos = p.find(W + "vertAlign"), p.find(W + "position")
            if a is not None:
                return a.get(W + "val") == "superscript"
            if pos is not None:
                return int(pos.get(W + "val", "0")) > 0
        return False

    def invisible(props):
        for p in props:
            if p is None:
                continue
            for tag in ("vanish", "webHidden", "specVanish"):
                if on(p, tag):
                    return "hidden text"
            for tag in ("strike", "dstrike"):
                if on(p, tag):
                    return "struck-through text"
            c = p.find(W + "color")
            if c is not None and c.get(W + "val", "").upper() == "FFFFFF":
                return "white text"
            sz = p.find(W + "sz")
            if sz is not None and sz.get(W + "val", "").isdigit() and int(sz.get(W + "val")) < 8:
                return "text under 4 pt"
        return None

    def styled(props, tag):
        for p in props:
            if p is not None and p.find(W + tag) is not None:
                return on(p, tag)
        return False

    books = []
    tokens = None
    chapter = None
    for p in root.iter(W + "p"):
        ppr = p.find(W + "pPr")
        style = None
        if ppr is not None and ppr.find(W + "pStyle") is not None:
            style = ppr.find(W + "pStyle").get(W + "val")
        runs = []
        for r in p.iter(W + "r"):
            text = ""
            for child in r:
                if child.tag == W + "t":
                    text += child.text or ""
                elif child.tag in (W + "tab", W + "br", W + "cr"):
                    text += " "
            if not text:
                continue
            props = properties(r, style)
            why = invisible(props)
            if why and text.strip():
                report["document_unsupported"][why] += 1
            runs.append((text, raised(props), styled(props, "b"), styled(props, "i")))
        whole = "".join(t for t, *_ in runs).strip()
        if not whole:
            continue
        level = heading_of.get(style)
        if level == 1:
            tokens = []
            books.append((" ".join(whole.split()), tokens))
            chapter = None
            continue
        if tokens is None:
            report["document_words_before_the_first_book"] += len(whole.split())
            continue
        if level == 2:
            numbers = re.findall(r"\d+", whole)
            if len(numbers) != 1:
                raise Failure(f"chapter heading {whole!r}: not exactly one number")
            chapter = numbers[0]
            tokens.append(("C", chapter))
            continue
        # Runs raised one after another are one number ("1" "0" = 10).
        merged = []
        for run in runs:
            if merged and merged[-1][1] and run[1]:
                merged[-1] = (merged[-1][0] + run[0], True, False, False)
            else:
                merged.append(run)
        lettered = [r for r in merged if r[0].strip()]
        numbered = any(r[1] for r in lettered)
        jc = ppr.find(W + "jc") if ppr is not None else None
        centred = jc is not None and jc.get(W + "val") == "center"
        heading = not numbered and (
            (all(r[2] for r in lettered) and centred) or all(r[3] for r in lettered))
        tag = "H" if heading else "T"
        pending = ""
        for text, is_raised, _, _ in merged + [("", True, False, False)]:
            if not is_raised:
                pending += text
                continue
            tokens.extend(("W", w, tag, None) for w in pending.split())
            pending = ""
            label = text.strip()
            if label:
                if not re.fullmatch(r"\d+[a-z]?([-–,]\d+[a-z]?)*", label):
                    report["document_unsupported"]["raised run that is not a verse number"] += 1
                if chapter is None:
                    raise Failure(f"verse number {label!r} before any chapter heading")
                tokens.append(("V", (chapter, label)))
    return books


# --------------------------------------------------------------- compare

def references(code, tokens):
    """The reference each token stands under."""
    out, chapter, verse = [], None, None
    for t in tokens:
        if t[0] == "C":
            chapter, verse = t[1], None
        elif t[0] == "V":
            chapter, verse = t[1]
        out.append(f"{code} {chapter}:{verse}" if verse is not None
                   else f"{code} {chapter} (before a verse)")
    return out


def compare(code, src, doc, report):
    """Compares one book's two streams; fills the report's lists."""
    refs = references(code, src)
    a, b = [t[:2] for t in src], [t[:2] for t in doc]
    damaged = collections.OrderedDict()
    missing_here = set()

    def entry(ref):
        return damaged.setdefault(ref, {"reference": ref, "missing": [], "not_in_source": []})

    if a != b:
        for op, a1, a2, b1, b2 in difflib.SequenceMatcher(a=a, b=b, autojunk=False).get_opcodes():
            if op == "equal":
                continue
            lost, added = src[a1:a2], doc[b1:b2]
            for t in lost:
                if t[0] == "V":
                    ref = f"{code} {t[1][0]}:{t[1][1]}"
                    report["verses_missing_from_document"].append(ref)
                    missing_here.add(ref)
                elif t[0] == "C":
                    report["chapters_missing_from_document"].append(f"{code} {t[1]}")
                else:
                    report["words_missing"] += 1
                    report["words_missing_by_source_marker"][f"\\{t[3]}"] += 1
            for t in added:
                if t[0] == "V":
                    report["verses_only_in_document"].append(f"{code} {t[1][0]}:{t[1][1]}")
                elif t[0] == "C":
                    report["chapters_only_in_document"].append(f"{code} {t[1]}")
                else:
                    report["words_not_in_source"] += 1
            # Words lost belong to the verse they stand in; words added, to
            # the verse they follow.
            if a1 < a2:
                ref = refs[a1]
            elif a1 > 0:
                ref = refs[a1 - 1]
            else:
                ref = f"{code} (the book's beginning)"
            e = entry(ref)
            words_lost = " ".join(t[1] for t in lost if t[0] == "W")
            words_added = " ".join(t[1] for t in added if t[0] == "W")
            if words_lost:
                e["missing"].append(words_lost)
            if words_added:
                e["not_in_source"].append(words_added)
            if any(t[0] != "W" for t in lost + added):
                e.setdefault("numbering", []).append(
                    {"source": [list(t[:2]) for t in lost if t[0] != "W"],
                     "document": [list(t[:2]) for t in added if t[0] != "W"]})
    else:
        # Equal streams: a heading of the source must be set as one.
        for s, d, ref in zip(src, doc, refs):
            if s[0] == "W" and s[2] == "H" and d[2] != "H":
                entry(ref).setdefault("heading_set_as_text", []).append(s[1])
                report["heading_words_set_as_text"] += 1
    for t in src:
        if t[0] == "V":
            ref = f"{code} {t[1][0]}:{t[1][1]}"
            if ref not in damaged and ref not in missing_here:
                report["verses_complete"] += 1
    report["verses_incomplete"].extend(damaged.values())


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    options = dict(a[2:].split("=", 1) for a in sys.argv[1:] if a.startswith("--") and "=" in a)
    out = options.get("json")
    profile_name = options.get("profile", "strict")
    only66 = options.get("books") == "66"
    usfm_path, docx_path = args
    counter = collections.Counter
    report = {
        "auditor_version": VERSION,
        "profile": profile_name,
        "books_option": "66" if only66 else "all",
        "source_usfm": {"file": usfm_path.split("/")[-1], "sha256": sha256(usfm_path)},
        "document": {"file": docx_path.split("/")[-1], "sha256": sha256(docx_path)},
        "complete": False,
        "failure": None,
        "books_in_source": 0,
        "books_in_document": 0,
        "verses_in_source": 0,
        "verses_complete": 0,
        "verses_incomplete": [],
        "verses_missing_from_document": [],
        "verses_only_in_document": [],
        "chapters_missing_from_document": [],
        "chapters_only_in_document": [],
        "books_missing_from_document": [],
        "books_only_in_document": [],
        "words_missing": 0,
        "words_not_in_source": 0,
        "heading_words_set_as_text": 0,
        "words_missing_by_source_marker": counter(),
        "source_words_by_marker": counter(),
        "source_text_words_before_a_verse": 0,
        "source_excluded": counter(),
        "source_files_ignored": [],
        "source_books_declared_not_rendered": {},
        "omitted_heading_words_inside_verses": {},
        "document_unsupported": counter(),
        "document_words_before_the_first_book": 0,
    }
    try:
        if profile_name not in PROFILES:
            raise Failure(f"unknown profile {profile_name!r}")
        if options.get("books") not in (None, "66", "all"):
            raise Failure(f"unknown --books value {options['books']!r}")
        source = read_usfm(usfm_path, PROFILES[profile_name], only66, report)
        document = read_docx(docx_path, report)
        report["books_in_source"], report["books_in_document"] = len(source), len(document)
        # Books are paired in order: the nth heading with the nth book.
        for n in range(max(len(source), len(document))):
            s = source[n] if n < len(source) else None
            d = document[n] if n < len(document) else None
            if s is None:
                report["books_only_in_document"].append(d[0])
                continue
            code, title, tokens = s
            if d is None or d[0] != title:
                report["books_missing_from_document"].append(
                    f"{code}: heading {title!r} expected in place {n + 1}, found {d[0] if d else None!r}")
                report["verses_missing_from_document"].extend(
                    f"{code} {t[1][0]}:{t[1][1]}" for t in tokens if t[0] == "V")
                if d is not None:
                    report["books_only_in_document"].append(d[0])
                continue
            compare(code, tokens, d[1], report)
        report["complete"] = not (
            report["verses_incomplete"] or report["verses_missing_from_document"]
            or report["verses_only_in_document"] or report["chapters_missing_from_document"]
            or report["chapters_only_in_document"] or report["books_missing_from_document"]
            or report["books_only_in_document"] or report["document_unsupported"]
            or report["document_words_before_the_first_book"] or report["heading_words_set_as_text"]
            or report["verses_in_source"] == 0)
    except Failure as e:
        report["failure"] = str(e)
        report["complete"] = False
    for k, v in list(report.items()):
        if isinstance(v, collections.Counter):
            report[k] = dict(sorted(v.items()))
    if out:
        with open(out, "w", encoding="utf-8") as f:
            json.dump(report, f, ensure_ascii=False, indent=1)
    brief = {k: (v if not isinstance(v, list) else len(v)) for k, v in report.items()
             if k not in ("source_words_by_marker", "source_excluded", "omitted_heading_words_inside_verses")}
    print(json.dumps(brief, ensure_ascii=False, indent=1))
    sys.exit(0 if report["complete"] else 1)


if __name__ == "__main__":
    main()
