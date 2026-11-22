#!/usr/bin/env python3
"""The check's own USFM verse-text reader, rules `aw-usfm-text-1`
(evaluation candidate A1.5-E3).

One of two readers of the reference's verse text, written apart:

* this one, in Python, used by check-set.py before the seal and before the
  run (a lexer, then a tree of paragraphs and inline spans, then a walk of
  that tree);
* the harness's, in Rust (`src-tauri/tests/eval_support/usfm_text.rs`, one
  streaming pass with a stack of open spans).

Neither shares code, tables or regular expressions with the other, or with
the app's USFM reader (`src-tauri/src/scripture/usfm.rs`), whose text the
independent E2 review found wrong in 528 verses of the bundled references
(a space inserted where a note was removed). Both follow the written rules
in `docs/evaluation/usfm-text-rules.md`, taken from the USFM 3 specification
(https://ubsicap.github.io/usfm/). The scorer compares the two readers'
per-verse text digests; any difference makes a run not a protocol run.

The text of a verse is exactly what the edition prints:

* notes (`\\f`, `\\fe`, `\\x`, `\\ef`, `\\ex`) are removed with their
  content, and nothing is put in their place: the white space the source has
  around them is kept, so "heel.\\f ...\\f*" + "”" reads "heel.”";
* character spans keep their text; `\\va`, `\\vp`, `\\ca`, `\\fig`, `\\rq`
  and `\\cat` do not (alternate and published numbers are recorded beside
  the verse); attribute lists (`|lemma="..."`) are not text;
* milestones (`\\qt-s |who="..."\\*`, `\\ts\\*`) are not text;
* USFM white space is U+0020, tab, CR and LF only: each run of it is one
  U+0020 and the ends are trimmed. Every other character, a no-break space
  included, is kept as it is; `~` is a no-break space (U+00A0) and `//` an
  optional line break (nothing);
* a paragraph or table-cell boundary inside a verse is one space.

Anything this reader cannot read unambiguously is a **problem**, never a
guess: a note, character span, sidebar or milestone left open; a closing
marker that is not the innermost open one's; a note inside a note; a verse
or chapter marker without a number; text after a chapter number; text in a
chapter that belongs to no verse; an attribute bar outside a span; a marker
that is not USFM 3; a backslash that starts no marker; a control
character in verse text. Problems never drop a verse: every `\\v` marker becomes a verse
(or a verse before any chapter), and a deliberately simple count of the
`\\v` markers in the source must agree (`verse_markers_unaccounted`).

    usfm_text.py <file.usfm | archive.zip> [MEMBER]   prints what it read, as JSON
"""
import hashlib
import json
import sys
import zipfile

RULES = "aw-usfm-text-1"

USFM_SPACE = " \t\n"  # after CR and CRLF are read as LF
NO_BREAK_SPACE = "\u00a0"

# Marker kinds, by the marker's name without its trailing level digits.
# Paragraphs whose text is verse text (when a verse is open).
BODY = frozenset("p m po pr cls pmo pm pmc pmr pi mi nb pc ph b q qr qc qm qd lh li lf lim tr".split())
# Paragraphs whose text is never verse text: identification, titles,
# headings, chapter labels and descriptions, introductions.
OTHER = frozenset(("usfm ide sts rem h toc toca mt mte ms mr s sr r d sp sd cl cp cd qa lit periph restore "
                   "imt is ip ipi im imi ipq imq ipr iq ib ili iot io iex imte ie").split())
# Table cells: a boundary inside a row (one space), never paired.
CELLS = frozenset("th thc thr tc tcc tcr".split())
NOTES = frozenset("f fe x ef ex".split())
# Parts of a note: they may close one another implicitly (USFM 3, syntax).
NOTE_PARTS = frozenset("fr fq fqa fk fl fw fp fv ft fdc fm efm xo xk xq xt xta xop xot xnt xdc".split())
# Character spans whose text is the edition's text.
KEPT = frozenset(("add bk dc k nd ord pn png addpn qt sig sls tl wj em bd it bdit no sc sup ndx rb pro "
                  "w wg wh wa jmp qs qac litl lik liv ior iqt").split())
# Character spans whose content is not verse text.
DROPPED = frozenset("va vp ca fig rq cat".split())


def base_name(name):
    """The marker's name without its level digits: q2 -> q, toc1 -> toc."""
    end = len(name)
    while end > 0 and "0" <= name[end - 1] <= "9":
        end -= 1
    return name[:end] or name


# ---------------------------------------------------------------- lexing

class Tok:
    """One lexical item: text, a marker, a milestone, or an attribute bar."""
    __slots__ = ("kind", "name", "plus", "closing", "text", "line")

    def __init__(self, kind, line, name="", plus=False, closing=False, text=""):
        self.kind, self.line, self.name, self.plus, self.closing, self.text = kind, line, name, plus, closing, text

    def __repr__(self):
        return f"Tok({self.kind},{self.name!r},{self.text!r})"


def _name_char(c):
    return ("a" <= c <= "z") or ("A" <= c <= "Z") or ("0" <= c <= "9")


def lex(text, problems):
    """Tokens of `text` (newlines already LF). Text runs are sliced between
    backslashes and bars, so the work grows with the number of markers."""
    toks = []
    n = len(text)
    i = 0
    line = 1

    def emit_text(a, b):
        nonlocal line
        if a >= b:
            return
        # A bar is lexed on its own: only the parser knows whether it opens
        # an attribute list.
        for k, part in enumerate(text[a:b].split("|")):
            if k:
                toks.append(Tok("bar", line))
            if part:
                toks.append(Tok("text", line, text=part))
                line += part.count("\n")

    while i < n:
        j = text.find("\\", i)
        if j < 0:
            emit_text(i, n)
            break
        emit_text(i, j)
        i = j + 1
        if i < n and text[i] == "*":
            toks.append(Tok("mclose", line))  # the end of a milestone: \*
            i += 1
            continue
        plus = False
        if i < n and text[i] == "+":
            plus = True
            i += 1
        k = i
        while k < n and _name_char(text[k]):
            k += 1
        # A milestone's start or end (`-s`, `-e`), or a table cell's span (`-2`).
        if k + 1 < n and text[k] == "-" and k > i and (text[k + 1] in "se" or "0" <= text[k + 1] <= "9"):
            k += 2
            while k < n and "0" <= text[k] <= "9":
                k += 1
        name = text[i:k]
        if not name:
            problems.append(("backslash_not_marker", line, repr(text[j:j + 3])))
            i = k
            continue
        closing = k < n and text[k] == "*"
        if closing:
            k += 1
        tok = Tok("marker", line, name=name, plus=plus, closing=closing)
        i = k
        if not closing:
            # The one white space character that ends an opening marker is
            # part of the marker, not of the text.
            if i < n and text[i] in USFM_SPACE:
                if text[i] == "\n":
                    line += 1
                i += 1
            # A milestone: the marker, optionally a space and an attribute
            # list, then `\*`. Its `-s`/`-e` forms must be closed so.
            stop = text.find("\\", i)
            between = text[i:stop] if stop >= 0 else ""
            terminated = stop >= 0 and text.startswith("\\*", stop) and \
                (between == "" or between.lstrip(USFM_SPACE).startswith("|"))
            if terminated:
                tok.kind = "milestone"
                line += between.count("\n")
                i = stop + 2
            elif name.endswith(("-s", "-e")):
                problems.append(("milestone_unterminated", line, f"\\{name} has no \\* ending"))
                tok.kind = "milestone"
        toks.append(tok)
    return toks


# ---------------------------------------------------------------- the tree

class Span:
    """An inline container: a note, a character span or a sidebar."""
    __slots__ = ("kind", "name", "plus", "line", "children", "attributes", "closed", "verse_context")

    def __init__(self, kind, name, plus, line):
        self.kind, self.name, self.plus, self.line = kind, name, plus, line
        self.children, self.attributes, self.closed, self.verse_context = [], None, False, False


class Para:
    __slots__ = ("name", "line", "children")

    def __init__(self, name, line):
        self.name, self.line, self.children = name, line, []


def parse(toks, problems):
    """[Para]: every paragraph-level marker starts one; `\\c` and `\\id` are
    paragraphs of their own. Inline items are text, verse markers, spans
    and milestones, nested as the markers say."""
    paras = [Para("", 0)]  # whatever precedes the first marker
    stack = []  # open spans of the current paragraph, innermost last
    milestones = []  # open -s milestones of the book: (base, line)

    def container():
        return stack[-1].children if stack else paras[-1].children

    def close_all(at, why):
        while stack:
            s = stack.pop()
            if s.kind == "note_part":
                continue
            problems.append((f"{s.kind}_unclosed", s.line, f"\\{s.name} opened on line {s.line} is still open at {why} (line {at})"))

    def in_note():
        return any(s.kind == "note" for s in stack)

    def in_sidebar():
        return any(s.kind == "sidebar" for s in stack)

    for t in toks:
        if t.kind == "text":
            top = stack[-1] if stack else None
            if top is not None and top.attributes is not None:
                top.attributes.append(t.text)
            else:
                container().append(t.text)
            continue
        if t.kind == "bar":
            top = stack[-1] if stack else None
            if top is not None and top.kind == "char" and top.attributes is None:
                top.attributes = []
            elif top is not None and top.kind in ("note", "note_part", "dropped"):
                container().append("|")  # never read: inside removed content
            elif top is not None and top.attributes is not None:
                top.attributes.append("|")
            else:
                container().append(("bar", t.line))
            continue
        # A marker that adds nothing to the tree (an end marker, a stray or
        # unknown one) still stands between a \\v, \\c or \\id and the text
        # after it: a placeholder keeps that so for the walk.
        before = (len(paras), len(container()), id(container()))
        try:
            if t.kind == "mclose":
                problems.append(("milestone_close_stray", t.line, "\\* that ends no milestone"))
                continue
            name, base = t.name, base_name(t.name.split("-")[0])
            if t.kind == "milestone":
                if name.endswith("-s"):
                    milestones.append((name[:-2], t.line))
                elif name.endswith("-e"):
                    want = name[:-2]
                    match = [m for m in milestones if m[0] == want]
                    if match:
                        milestones.remove(match[-1])
                    else:
                        problems.append(("milestone_unpaired", t.line, f"\\{name} ends no \\{want}-s"))
                container().append(("milestone", t.line))
                continue
            # An attribute list ends only at its span's own closing marker.
            top = stack[-1] if stack else None
            if top is not None and top.attributes is not None and not (t.closing and t.name == top.name):
                problems.append(("attributes_interrupted", t.line, f"\\{name} inside the attribute list of \\{top.name}"))
            if name == "esbe" and not t.closing:
                if stack and stack[-1].kind == "sidebar":
                    stack.pop().closed = True
                else:
                    problems.append(("sidebar_close_stray", t.line, "\\esbe without \\esb"))
                continue
            if in_sidebar():
                if name in ("v", "c", "id") and not t.closing and not t.plus:
                    close_all(t.line, f"\\{name}")  # the sidebar was never closed; the marker is read
                elif name == "esb" and not t.closing:
                    problems.append(("sidebar_nested", t.line, "\\esb inside a sidebar"))
                    continue
                else:
                    continue  # sidebar content: never verse text
            if name == "esb" and not t.closing:
                close_all(t.line, "\\esb")
                s = Span("sidebar", name, False, t.line)
                container().append(s)
                stack.append(s)
                continue
            if name in ("c", "id") and not t.closing and not t.plus:
                close_all(t.line, f"\\{name}")
                paras.append(Para(name, t.line))
                continue
            if name == "v" and not t.closing and not t.plus:
                if stack:
                    close_all(t.line, "\\v")
                container().append(("verse", t.line))
                continue
            if not t.closing and (base in BODY or base in OTHER) and not t.plus:
                close_all(t.line, f"\\{name}")
                paras.append(Para(name, t.line))
                continue
            if not t.closing and (base in CELLS or name == "pb"):
                if stack:
                    close_all(t.line, f"\\{name}")
                container().append(("boundary", t.line))
                continue
            if base in NOTES:
                if not t.closing:
                    if in_note():
                        problems.append(("note_nested", t.line, f"\\{name} inside an open note"))
                    s = Span("note", name, t.plus, t.line)
                    container().append(s)
                    stack.append(s)
                    continue
                # Closing a note: its own parts may still be open (implicitly closed).
                while stack and stack[-1].kind == "note_part":
                    stack.pop()
                if stack and stack[-1].kind == "note" and stack[-1].name == name:
                    stack.pop().closed = True
                elif in_note():
                    problems.append(("note_close_mismatch", t.line, f"\\{name}* closes \\{[s for s in stack if s.kind == 'note'][-1].name}"))
                else:
                    problems.append(("note_close_stray", t.line, f"\\{name}* with no note open"))
                continue
            if base in NOTE_PARTS:
                if in_note():
                    if t.closing:
                        if stack and stack[-1].kind == "note_part" and stack[-1].name == name:
                            stack.pop()
                        continue
                    if stack and stack[-1].kind == "note_part" and not t.plus:
                        stack.pop()  # one part closes the one before it
                    s = Span("note_part", name, t.plus, t.line)
                    container().append(s)
                    stack.append(s)
                else:
                    container().append(("note_part_outside", t.line, name))
                continue
            if base in KEPT or base in DROPPED:
                kind = "char" if base in KEPT else "dropped"
                if in_note():
                    # Inside a note everything is removed; only nesting by `+`
                    # is followed so the note's own end is found.
                    if t.closing:
                        if stack and stack[-1].name == name and stack[-1].kind in ("char", "dropped"):
                            stack.pop()
                        continue
                    if t.plus:
                        s = Span(kind, name, True, t.line)
                        container().append(s)
                        stack.append(s)
                    continue
                if t.closing:
                    if stack and stack[-1].kind in ("char", "dropped") and stack[-1].name == name \
                            and stack[-1].plus == t.plus:
                        stack.pop().closed = True
                    elif any(s.kind in ("char", "dropped") for s in stack):
                        problems.append(("span_close_mismatch", t.line,
                                         f"\\{'+' if t.plus else ''}{name}* closes \\{stack[-1].name}"))
                    else:
                        problems.append(("span_close_stray", t.line, f"\\{name}* with no span open"))
                    continue
                if any(s.kind in ("char", "dropped") for s in stack) and not t.plus:
                    problems.append(("span_nested_without_plus", t.line, f"\\{name} inside \\{stack[-1].name}"))
                s = Span(kind, name, t.plus, t.line)
                container().append(s)
                stack.append(s)
                continue
            if base.startswith("z"):
                problems.append(("marker_private", t.line, f"\\{name}: a \\z marker whose text this reader cannot place"))
                continue
            problems.append(("marker_unknown", t.line, f"\\{'+' if t.plus else ''}{name}{'*' if t.closing else ''}"))
        finally:
            if (len(paras), len(container()), id(container())) == before:
                container().append(("mark", t.line))
    close_all(10 ** 9, "the end of the file")
    for m, line in milestones:
        problems.append(("milestone_unpaired", line, f"\\{m}-s has no \\{m}-e"))
    return paras


# ---------------------------------------------------------------- the walk

def _verse_text(pieces):
    s = "".join(pieces).replace("//", "").replace("~", NO_BREAK_SPACE)
    out, gap = [], False
    for c in s:
        if c in USFM_SPACE:
            gap = True
            continue
        if gap and out:
            out.append(" ")
        gap = False
        out.append(c)
    return "".join(out)


def _first_word(s):
    s = s.lstrip(USFM_SPACE)
    for k, c in enumerate(s):
        if c in USFM_SPACE:
            return s[:k], s[k:]
    return s, ""


def walk(paras, problems):
    book = None
    ids = 0
    chapters, verses = [], []
    state = {"chapter": None, "verse": None, "target": False, "before_chapter": 0}

    def finish():
        v = state["verse"]
        if v is not None:
            v["text"] = _verse_text(v.pop("pieces"))
            verses.append(v)
        state["verse"] = None

    def take_label(items, k, what, line):
        """The number after `\\v` or `\\c`: the first word of the next text."""
        if k < len(items) and isinstance(items[k], str):
            word, rest = _first_word(items[k])
            if word:
                return word, rest
        problems.append((f"{what}_number_missing", line, f"\\{what} without a number"))
        return "", None

    def inline(items, para_name, in_chapter):
        k = 0
        while k < len(items):
            it = items[k]
            k += 1
            if isinstance(it, str):
                if state["target"]:
                    state["verse"]["pieces"].append(it)
                elif in_chapter and base_name(para_name) in BODY and it.strip(USFM_SPACE):
                    problems.append(("text_outside_verse", None, f"{state['chapter']}: «{it.strip()[:40]}»"))
                continue
            if isinstance(it, tuple):
                kind, line = it[0], it[1]
                if kind == "verse":
                    finish()
                    label, rest = take_label(items, k, "verse", line)
                    if rest is not None:
                        k += 1
                    if state["chapter"] is None:
                        state["before_chapter"] += 1
                        state["target"] = False
                        continue
                    state["verse"] = {"chapter": state["chapter"]["label"], "label": label, "line": line,
                                      "pieces": [rest or ""], "alternate": {}}
                    state["target"] = True
                elif kind == "boundary":
                    if state["target"]:
                        state["verse"]["pieces"].append(" ")
                elif kind == "bar":
                    if state["target"]:
                        problems.append(("bar_outside_span", line, "| in verse text outside a character span"))
                elif kind == "note_part_outside":
                    if state["target"]:
                        problems.append(("note_part_outside_note", line, f"\\{it[2]} in verse text outside a note"))
                continue
            # A span.
            if it.kind in ("note", "note_part", "sidebar"):
                continue
            if it.kind == "dropped":
                if it.name in ("va", "vp", "ca"):
                    value = _verse_text(c for c in it.children if isinstance(c, str))
                    if it.name == "ca" and state["chapter"] is not None and not state["target"]:
                        state["chapter"]["alternate"] = value
                    elif state["target"]:
                        state["verse"]["alternate"][it.name] = value
                continue
            inline(it.children, para_name, in_chapter)

    for p in paras:
        if p.name == "id":
            ids += 1
            finish()
            state["target"] = False
            first = p.children[0] if p.children and isinstance(p.children[0], str) else ""
            code, _ = _first_word(first)
            if book is None:
                book = "".join(chr(ord(c) - 32) if "a" <= c <= "z" else c for c in code) or None
            inline(p.children[1:] if first else p.children, "id", state["chapter"] is not None)
            continue
        if p.name == "c":
            finish()
            state["target"] = False
            items = list(p.children)
            label, rest = "", ""
            if items and isinstance(items[0], str):
                label, rest = _first_word(items[0])
                items = items[1:]
            if not label:
                problems.append(("chapter_number_missing", p.line, "\\c without a number"))
            chapter = {"label": label, "line": p.line, "alternate": None, "published": None}
            chapters.append(chapter)
            state["chapter"] = chapter
            # Before the chapter's first verse marker: white space, an
            # alternate chapter number, milestones; nothing else.
            pre, k = [rest], 0
            while k < len(items) and not (isinstance(items[k], tuple) and items[k][0] == "verse"):
                x = items[k]
                k += 1
                if isinstance(x, str):
                    pre.append(x)
                elif isinstance(x, Span) and x.kind == "dropped" and x.name == "ca":
                    chapter["alternate"] = _verse_text(c for c in x.children if isinstance(c, str))
                elif isinstance(x, Span) and x.kind in ("char", "dropped"):
                    pre.append("\\" + x.name)
            if "".join(pre).strip(USFM_SPACE):
                problems.append(("text_after_chapter_number", p.line, f"\\c {label} is followed by text"))
            inline(items[k:], "c", True)
            continue
        b = base_name(p.name)
        if b == "cp" and state["chapter"] is not None:
            first = p.children[0] if p.children and isinstance(p.children[0], str) else ""
            state["chapter"]["published"] = _first_word(first)[0]
        in_chapter = state["chapter"] is not None
        if b in BODY:
            if state["verse"] is not None:
                state["verse"]["pieces"].append(" ")
                state["target"] = True
            else:
                state["target"] = False
        else:
            state["target"] = False  # a heading, title or introduction
        inline(p.children, p.name, in_chapter)
    finish()
    return book, ids, chapters, verses, state["before_chapter"]


# ---------------------------------------------------------------- reading

def simple_verse_marker_count(text):
    """`\\v` markers counted the plainest way, apart from the lexer: a
    backslash, `v`, then anything that cannot continue a marker name."""
    count, start = 0, 0
    while True:
        k = text.find("\\v", start)
        if k < 0:
            return count
        nxt = text[k + 2:k + 3]
        if not (nxt and (_name_char(nxt) or nxt in "*-")):
            count += 1
        start = k + 2


def read_text(source):
    """What the edition prints, read from one USFM book file's text."""
    problems = []
    bom = source.startswith("\ufeff")
    if bom:
        source = source[1:]
    text = source.replace("\r\n", "\n").replace("\r", "\n")
    toks = lex(text, problems)
    paras = parse(toks, problems)
    book, ids, chapters, verses, before = walk(paras, problems)
    # A control character in verse text (titles and notes are not read).
    for v in verses:
        bad = [c for c in v["text"] if ord(c) < 0x20 or 0x7f <= ord(c) < 0xa0]
        if bad:
            problems.append(("control_character", v["line"],
                             f"{v['chapter']}:{v['label']} holds U+{ord(bad[0]):04X}"))
    markers = simple_verse_marker_count(text)
    if markers != len(verses) + before:
        problems.append(("verse_markers_unaccounted", None,
                         f"{markers} \\v markers in the source, {len(verses)} verses and {before} before any chapter read"))
    if ids > 1:
        problems.append(("id_repeated", None, f"{ids} \\id lines; one book per file"))
    return {
        "rules": RULES,
        "book": book,
        "id_lines": ids,
        "bom": bom,
        "verses": [{"chapter": v["chapter"], "label": v["label"], "text": v["text"], "line": v["line"],
                    "alternate": v["alternate"]} for v in verses],
        "verses_before_chapter": before,
        "verse_markers": markers,
        "chapters": chapters,
        "problems": [{"code": c, "line": line, "detail": d} for c, line, d in problems],
    }


def text_sha256(text):
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def main():
    if len(sys.argv) not in (2, 3):
        sys.exit(__doc__)
    path = sys.argv[1]
    if path.lower().endswith(".zip"):
        with zipfile.ZipFile(path) as z:
            names = [sys.argv[2]] if len(sys.argv) == 3 else \
                [n for n in z.namelist() if n.lower().endswith((".usfm", ".sfm"))]
            out = {n: read_text(z.read(n).decode("utf-8")) for n in names}
    else:
        out = {path: read_text(open(path, encoding="utf-8").read())}
    print(json.dumps(out, ensure_ascii=False, indent=1))


if __name__ == "__main__":
    main()
