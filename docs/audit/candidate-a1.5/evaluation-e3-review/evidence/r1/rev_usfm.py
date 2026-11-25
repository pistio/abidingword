#!/usr/bin/env python3
"""The independent E3 reviewer's own USFM verse-text extractor.

Written for this review from the USFM 3 specification
(https://ubsicap.github.io/usfm/), without reading or copying either E3
reader (scripts/evaluation/usfm_text.py, src-tauri/tests/eval_support/
usfm_text.rs), the E2 reviewer's extractor or the app's reader. Design: a
regular-expression tokenizer, then one pass that keeps a list of open spans
and a "suppressed" flag; notes are cut out by matching their own end marker.
Unlike the E3 readers it does not try to recover from malformed markup: it
records what it could not read in `trouble` and continues.

Spec choices (taken from the specification, stated independently):
* USFM white space: space, tab, line end. One white space character after an
  opening marker belongs to the marker. Runs of white space are one space in
  the verse text; the verse text is trimmed.
* Notes (\\f \\fe \\x \\ef \\ex ... their own end marker) are not verse text and
  leave nothing behind.
* Character spans keep their text; their attributes (after `|` up to the
  span's end marker) are not text. \\va \\vp \\ca \\fig \\rq \\cat are not text.
* Milestones (\\name-s / \\name-e, optional attributes, then \\*) and empty
  standalone milestones (\\ts\\*) are not text.
* `~` is U+00A0 (no-break space); `//` is a discretionary break: nothing.
* Paragraph and poetry markers continue the verse (one space). Titles,
  headings, labels, introductions and descriptive titles (\\d) are not verse
  text, unless a \\v is inside them.

    rev_usfm.py <file.usfm|archive.zip>   prints {member: {book, verses:[[c, v, text]], trouble}}
"""
import json
import re
import sys
import zipfile

TOKEN = re.compile(r"\\\*|\\(\+?)([A-Za-z0-9]+(?:-[se])?)(\*?)|\|")
NOTE = {"f", "fe", "x", "ef", "ex"}
DROP = {"va", "vp", "ca", "fig", "rq", "cat"}
VERSE_PARA = {"p", "m", "po", "pr", "cls", "pmo", "pm", "pmc", "pmr", "pi", "mi", "nb", "pc", "ph", "b",
              "q", "qr", "qc", "qm", "qd", "lh", "li", "lf", "lim", "tr", "th", "thr", "thc", "tc", "tcr", "tcc",
              "pb"}
NON_VERSE_PARA = {"id", "ide", "h", "toc", "toca", "rem", "sts", "usfm", "mt", "mte", "ms", "mr", "s", "sr", "r",
                  "d", "sp", "sd", "cl", "cd", "cp", "imt", "is", "ip", "ipi", "im", "imi", "ipq", "imq", "ipr",
                  "iq", "ib", "ili", "iot", "io", "iex", "imte", "ie", "lit", "qa", "periph", "restore"}
CHAR = {"add", "bk", "dc", "k", "nd", "ord", "pn", "png", "addpn", "qt", "sig", "sls", "tl", "wj", "em", "bd",
        "it", "bdit", "no", "sc", "sup", "ndx", "rb", "pro", "w", "wg", "wh", "wa", "jmp", "qs", "qac", "litl",
        "lik", "liv", "ior", "iqt", "ref"}
WS = " \t\n"


def stem(name):
    return name.rstrip("0123456789") or name


def collapse(s):
    s = s.replace("//", "").replace("~", "\u00a0")
    out = []
    for part in re.split(r"[ \t\n]+", s):
        if part:
            out.append(part)
    return " ".join(out)


def read(text):
    if text.startswith("\ufeff"):
        text = text[1:]
    text = text.replace("\r\n", "\n").replace("\r", "\n")
    trouble = []
    book = None
    chapter = None
    verses = []          # [chapter, label, [pieces]]
    cur = None
    in_verse_para = True  # whether the current paragraph carries verse text
    spans = []            # open character / dropped spans: [name, plus, attrs]
    pos = 0
    want = None           # "id", "c", "v": the next word is a code / number
    n = len(text)

    def emit(s):
        nonlocal want, book, chapter, cur
        if not s:
            return
        if want:
            t = s.lstrip(WS)
            m = re.match(r"[^ \t\n]*", t)
            word = m.group(0)
            rest = t[len(word):]
            w, want = want, None
            if w == "id":
                book = word.upper() if book is None else book
                suppress_line[0] = True
                return
            if w == "c":
                chapter = word
                cur = None
                s = rest
                if s.strip(WS):
                    trouble.append(f"text after \\c {word}")
                return
            if w == "v":
                if chapter is None:
                    trouble.append(f"verse {word} before a chapter")
                    cur = None
                    return
                cur = [chapter, word, []]
                verses.append(cur)
                s = rest
        if suppress_line[0]:
            return
        if any(sp[0] in DROP for sp in spans) or any(sp[2] for sp in spans):
            return
        if cur is not None and in_verse_para:
            cur[2].append(s)

    suppress_line = [False]
    while pos < n:
        m = TOKEN.search(text, pos)
        if not m:
            emit(text[pos:])
            break
        emit(text[pos:m.start()])
        tok = m.group(0)
        pos = m.end()
        if tok == "|":
            if spans and not spans[-1][2]:
                spans[-1][2] = True       # attributes until the span's end
            elif not spans:
                trouble.append("bar outside a span")
                if cur is not None and in_verse_para:
                    cur[2].append("|")
            continue
        if tok == "\\*":
            trouble.append("stray \\*")
            continue
        plus, name, end = m.group(1), m.group(2), m.group(3)
        # milestones: -s/-e names, and any marker followed by optional attributes and \*
        if not end:
            k = pos + 1 if pos < n and text[pos] in WS else pos
            nxt = text.find("\\", k)
            seg = text[k:nxt] if nxt >= 0 else None
            if seg is not None and text.startswith("\\*", nxt) and (seg == "" or seg.lstrip(WS).startswith("|")):
                if name in ("v", "c", "id") or stem(name) in VERSE_PARA | NON_VERSE_PARA | NOTE:
                    trouble.append(f"\\{name} written as a milestone")
                pos = nxt + 2
                continue
            if name.endswith(("-s", "-e")):
                trouble.append(f"milestone \\{name} without \\*")
                continue
        if not end and pos < n and text[pos] in WS:
            pos += 1  # the marker's own white space
        base = stem(name)
        if name in ("v", "c", "id") and not end and not plus:
            if spans:
                trouble.append(f"spans open at \\{name}: {[s[0] for s in spans]}")
                spans.clear()
            suppress_line[0] = False
            if name == "v":
                if chapter is not None and not in_verse_para and cur is None:
                    pass
                in_verse_para = True if cur is not None or True else in_verse_para
            else:
                in_verse_para = False if name == "id" else True
            want = name
            continue
        if base in NOTE and not end:
            close = "\\" + name + "*"
            j = text.find(close, pos)
            inner_open = re.search(r"\\(f|fe|x|ef|ex) ", text[pos:j] if j >= 0 else text[pos:])
            if j < 0:
                trouble.append(f"\\{name} never closed")
                pos = n
                continue
            if inner_open:
                trouble.append(f"note inside \\{name}")
            for stop in ("\\v ", "\\c ", "\\p\n", "\\p "):
                if stop in text[pos:j]:
                    trouble.append(f"\\{name} spans a {stop.strip()} marker")
            pos = j + len(close)
            continue
        if base in NOTE and end:
            trouble.append(f"stray \\{name}*")
            continue
        if not end and not plus and (base in VERSE_PARA or base in NON_VERSE_PARA):
            if spans:
                trouble.append(f"spans open at \\{name}")
                spans.clear()
            if base in NON_VERSE_PARA:
                in_verse_para = False
                if name == "id":
                    want = "id"
            else:
                in_verse_para = True
                if cur is not None:
                    cur[2].append(" ")
            continue
        if base in CHAR or base in DROP:
            if end:
                if spans and spans[-1][0] == name:
                    spans.pop()
                else:
                    trouble.append(f"\\{name}* does not close the innermost span")
                continue
            if spans and not plus:
                trouble.append(f"\\{name} opened inside a span without +")
            spans.append([name, plus, False])
            continue
        trouble.append(f"unknown marker \\{plus}{name}{end}")
    if spans:
        trouble.append(f"spans open at the end: {[s[0] for s in spans]}")
    out = [[c, v, collapse("".join(p))] for c, v, p in verses]
    count = len(re.findall(r"\\v(?![A-Za-z0-9*\-])", text))
    if count != len(verses) + sum(1 for t in trouble if t.startswith("verse ") and "before a chapter" in t):
        trouble.append(f"{count} \\v markers, {len(verses)} verses")
    return {"book": book, "verses": out, "trouble": trouble}


def main():
    path = sys.argv[1]
    if path.lower().endswith(".zip"):
        with zipfile.ZipFile(path) as z:
            res = {nm: read(z.read(nm).decode("utf-8")) for nm in z.namelist() if nm.lower().endswith((".usfm", ".sfm"))}
    else:
        res = {path: read(open(path, encoding="utf-8").read())}
    print(json.dumps(res, ensure_ascii=False))


if __name__ == "__main__":
    main()
