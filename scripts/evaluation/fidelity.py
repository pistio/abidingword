#!/usr/bin/env python3
"""Strict text fidelity, rules `aw-fidelity-2` (evaluation candidate A1.5-E2).

The scorer's own implementation of the rule the harness applies
(`src-tauri/tests/eval_support/fidelity.rs`). The two are written apart and
checked against the same golden vectors (`tests/evaluation/fidelity-vectors.json`);
the scorer re-classifies every non-exact verse the harness reports and
refuses to call a run a protocol run when the two disagree.

A verse found in a document (`found`) is compared with the edition's own text
(`edition`). Its class is the first that holds:

  exact       the same code points, leading and trailing white space aside;
  canonical   the same after Unicode NFC (canonical equivalence only);
  layout      the same after NFC and white-space-run normalisation: every
              maximal run of white space becomes one U+0020 and the ends are
              trimmed. A run is never removed and none is added, so word
              boundaries are preserved;
  mismatch    anything else.

Only exact, canonical and layout are faithful. Every mismatch counts against
the zero-wrong-text criterion, whatever its kind. The kind is a diagnostic
label, never an excuse (see KINDS).

White space is the Unicode White_Space property, listed explicitly (WS) so
that both implementations use the same 25 code points.
"""
import unicodedata

RULES = "aw-fidelity-2"

WS = frozenset(chr(c) for c in [0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x20, 0x85, 0xA0, 0x1680,
                                0x2000, 0x2001, 0x2002, 0x2003, 0x2004, 0x2005, 0x2006,
                                0x2007, 0x2008, 0x2009, 0x200A, 0x2028, 0x2029, 0x202F,
                                0x205F, 0x3000])

FAITHFUL = ("exact", "canonical", "layout")
CLASSES = FAITHFUL + ("mismatch",)

# Mismatch kinds. Material kinds: the letters and digits themselves differ.
# The others keep the same letters and digits but are still wrong text under
# the strict rule. The order they are tried in is `kind` below.
KINDS = (
    "case",                 # only letter case differs
    "word_boundary",        # words joined or split (white space placement between letters)
    "punctuation_spacing",  # only white space placement differs, beside punctuation
    "format_characters",    # only invisible format characters differ (soft hyphen, ZWJ...)
    "compatibility",        # only compatibility forms differ (ligature, superscript, full width)
    "marks",                # diacritics differ: accents, breathings, vowel points, cantillation
    "sign",                 # a note sign (*, dagger...) the edition does not have
    "punctuation",          # punctuation, symbols, quotation marks, brackets or their places
    "extra",                # material: the edition's words and more
    "missing",              # material: part of the edition's words only
    "wrong_reference",      # material: the words of another verse of the reference
    "changed",              # material: the words themselves differ
)
MATERIAL = ("extra", "missing", "wrong_reference", "changed")
NOTE_SIGNS = "*†‡§¶"


def trim(t):
    i, j = 0, len(t)
    while i < j and t[i] in WS:
        i += 1
    while j > i and t[j - 1] in WS:
        j -= 1
    return t[i:j]


def runs(t):
    """Each maximal run of white space as one U+0020, the ends trimmed."""
    out, gap = [], False
    for c in trim(t):
        if c in WS:
            gap = True
            continue
        if gap:
            out.append(" ")
            gap = False
        out.append(c)
    return "".join(out)


def nfc(t):
    return unicodedata.normalize("NFC", t)


def dense(t):
    return "".join(c for c in t if c not in WS)


def _keep(t, classes):
    return "".join(c for c in t if unicodedata.category(c)[0] in classes)


def base(t):
    """Letters and digits only, compatibility-decomposed, lower case, marks dropped."""
    return _keep(unicodedata.normalize("NFKD", t.lower()), "LN")


def marked(t):
    """Letters, digits and combining marks, compatibility-decomposed, lower case."""
    return _keep(unicodedata.normalize("NFKD", t.lower()), "LMN")


def cased(t):
    """Letters, digits and combining marks, compatibility-decomposed, case kept."""
    return _keep(unicodedata.normalize("NFKD", t), "LMN")


def words(t):
    """The white-space-separated words, each as `marked`, empty ones dropped."""
    return [w for w in (marked(x) for x in runs(t).split(" ")) if w]


def without_format(t):
    return "".join(c for c in t if c in WS or unicodedata.category(c) not in ("Cf", "Cc"))


def classify(edition, found):
    """exact | canonical | layout | mismatch."""
    if trim(found) == trim(edition):
        return "exact"
    if nfc(trim(found)) == nfc(trim(edition)):
        return "canonical"
    if runs(nfc(found)) == runs(nfc(edition)):
        return "layout"
    return "mismatch"


def kind(edition, found, matched=False):
    """The diagnostic kind of a mismatch. `matched`: the found text's letters
    are those of another verse of the reference (the harness looks that up
    in the whole reference inventory)."""
    e, f = nfc(trim(edition)), nfc(trim(found))
    be, bf = base(e), base(f)
    if be != bf:
        if be and be in bf:
            return "extra"
        if bf in be:
            return "missing"
        return "wrong_reference" if matched else "changed"
    if runs(e).lower() == runs(f).lower():
        return "case"
    if dense(e) == dense(f):
        return "word_boundary" if words(e) != words(f) else "punctuation_spacing"
    if runs(without_format(e)) == runs(without_format(f)):
        return "format_characters"
    if unicodedata.normalize("NFKC", runs(e)) == unicodedata.normalize("NFKC", runs(f)):
        return "compatibility"
    if marked(e) != marked(f):
        return "marks"
    if words(e) != words(f):
        return "word_boundary"
    if sum(f.count(s) for s in NOTE_SIGNS) > sum(e.count(s) for s in NOTE_SIGNS):
        return "sign"
    return "punctuation"


def judge(edition, found, matched=False):
    """(class, kind); kind is None unless the class is mismatch."""
    c = classify(edition, found)
    return c, (kind(edition, found, matched) if c == "mismatch" else None)
