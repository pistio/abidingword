#!/usr/bin/env python3
"""Reviewer's independent USFM verse-text extractor, written for the
independent review of evaluation candidate A1.5-E2 from the USFM 3
specification. It shares no code and no regular expression with the app's
reader (src-tauri/src/scripture/usfm.rs) or with the check's label reader
(scripts/evaluation/reference_inventory.py): it is a character tokenizer
with an explicit stack of hidden spans.

Verse text = the content of each \\v, across paragraph and poetry markers,
excluding: notes (\\f \\fe \\x \\ef \\ex, closed by their own name), \\fig,
\\va \\vp \\ca \\rq \\cat, milestones (\\name-s ... \\*, \\name-e\\*), sidebars
(\\esb ... \\esbe), headings and other non-verse paragraphs, \\d and \\qa
(headings), and word attributes after '|'. '~' is a no-break space and '//'
an optional break (removed). White space (Unicode White_Space) is collapsed
to one U+0020 and trimmed.

    indep_usfm.py <file.usfm>        prints JSON [[chapter, label, text], ...]
"""
import json
import re
import sys

WS = set("\t\n\x0b\x0c\r \x85\xa0\u1680\u2000\u2001\u2002\u2003\u2004\u2005\u2006"
         "\u2007\u2008\u2009\u200a\u2028\u2029\u202f\u205f\u3000")
NOTES = {"f", "fe", "x", "ef", "ex"}
HIDDEN_CHAR = {"fig", "va", "vp", "ca", "rq", "cat"}
ATTRIBUTED = {"w", "rb", "jmp", "xt", "ref", "fig"}
TEXT_PARA = {"p", "m", "po", "pr", "cls", "pmo", "pm", "pmc", "pmr", "pi", "mi", "nb", "pc", "ph", "b",
             "q", "qr", "qc", "qm", "qd", "lh", "li", "lf", "lim", "tr", "tc", "th", "tcr", "thr"}
HIDDEN_PARA = {"ide", "h", "toc", "toca", "mt", "mte", "ms", "mr", "s", "sr", "r", "sp", "sd", "cl", "cp",
               "cd", "rem", "sts", "restore", "periph", "lit", "usfm", "d", "qa"}
MARKER = re.compile(r"\\(\+?)([A-Za-z][A-Za-z0-9]*(?:-[se])?)(\*?)")


def stem(name):
    return name.rstrip("0123456789")


def tokens(src):
    i, n = 0, len(src)
    while i < n:
        if src[i] == "\\":
            m = MARKER.match(src, i)
            if m:
                j = m.end()
                closing = m.group(3) == "*"
                if not closing and j < n and src[j] in " \t\r\n":
                    j += 1  # an opening marker owns one following white space
                yield "marker", m.group(2), closing
                i = j
                continue
            if src.startswith("\\*", i):
                yield "close-milestone", None, True
                i += 2
                continue
            yield "text", "\\", False
            i += 1
            continue
        j = src.find("\\", i)
        j = n if j < 0 else j
        yield "text", src[i:j], False
        i = j


def collapse(t):
    out, gap = [], False
    for c in t:
        if c in WS:
            gap = True
            continue
        if gap and out:
            out.append(" ")
        gap = False
        out.append(c)
    return "".join(out)


def extract(src):
    verses = []            # [chapter, label, [parts]]
    chapter = None
    current = None
    hidden = []            # stack of hiding spans (notes, hidden characters, milestones, sidebars)
    chars = []             # open character markers (for attributes)
    visible = False        # inside verse-text paragraphs
    expect = None
    for kind, value, closing in tokens(src):
        if kind == "close-milestone":
            if hidden and hidden[-1] == "#milestone":
                hidden.pop()
            continue
        if kind == "marker":
            name = value
            s = stem(name)
            if name.endswith("-s") or name.endswith("-e"):
                hidden.append("#milestone")
                continue
            if s in NOTES:
                if closing:
                    if s in hidden:
                        while hidden.pop() != s:
                            pass
                elif not any(h in NOTES for h in hidden):
                    hidden.append(s)
                continue
            if any(h in NOTES for h in hidden):
                continue  # everything inside a note belongs to the note
            if s in HIDDEN_CHAR:
                if closing:
                    if s in hidden:
                        while hidden.pop() != s:
                            pass
                else:
                    hidden.append(s)
                continue
            if name == "esb":
                hidden.append("#esb")
                continue
            if name == "esbe":
                if "#esb" in hidden:
                    while hidden.pop() != "#esb":
                        pass
                continue
            if closing:
                if chars and chars[-1] == s:
                    chars.pop()
                continue
            if name in ("id", "c"):
                current = None
                visible = False
                expect = name
                continue
            if name == "v":
                expect = "v"
                continue
            if s in TEXT_PARA:
                visible = current is not None
                if current is not None:
                    current[2].append(" ")
                continue
            if s in HIDDEN_PARA or (s.startswith("i") and s != "it"):
                visible = False
                continue
            # a character marker (known or not): its content is text
            chars.append(s)
            continue
        text = value
        if hidden:
            # a verse number can still be read inside a hidden character span
            # only if the span is not a note; notes never hold verse markers here
            if expect == "v" and not any(h in NOTES for h in hidden) and False:
                pass
            continue
        if expect:
            stripped = text.lstrip("".join(WS))
            word = ""
            for k, c in enumerate(stripped):
                if c in WS:
                    word, text = stripped[:k], stripped[k:]
                    break
            else:
                word, text = stripped, ""
            if expect == "c":
                chapter = word
            elif expect == "v":
                if chapter is not None:
                    current = [chapter, word, []]
                    verses.append(current)
                    visible = True
            expect = None
            if current is None:
                continue
        if not visible or current is None:
            continue
        if chars and chars[-1] in ATTRIBUTED and "|" in text:
            text = text.split("|", 1)[0]
        text = text.replace("~", "\u00a0").replace("//", "")
        current[2].append(text)
    return [[c, v, collapse("".join(parts))] for c, v, parts in verses]


if __name__ == "__main__":
    print(json.dumps(extract(open(sys.argv[1], encoding="utf-8").read()), ensure_ascii=False))
