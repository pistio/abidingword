#!/usr/bin/env python3
"""The reviewer's own adversarial USFM vectors for the E3 text readers, each
with an expectation written by hand from the USFM 3 specification (not by
running either reader). Every text is invented for this review.

expect:
  {"ok": [[chapter, label, text], ...]}   read with no problem, exactly these verses
  {"refuse": True, "labels": [...], "keep": {"c:v": text}}
        at least one problem; every \\v still a verse (labels, in order); the
        verses in `keep` read exactly so (the problem must not swallow them)
  {"choice": "..."}                        a stated marker-table choice: output recorded, judged in the report

    make_vectors.py <out.json>
"""
import json, sys

H = "\\id RUT invented review text\n\\h Test\n\\mt1 Test\n"
V = []


def add(name, body, expect, group):
    V.append({"name": name, "group": group, "usfm": H + body, "expect": expect})


NB, NNB = "\u00a0", "\u202f"
# ---- notes and cross references (USFM 3)
add("note-before-closing-quote", "\\c 1\n\\p\n\\v 1 his heel.\\f + \\fr 1:1 \\ft Or offspring.\\f*\u201d\n\\v 2 Next.\n",
    {"ok": [["1", "1", "his heel.\u201d"], ["1", "2", "Next."]]}, "notes")
add("note-spaces-both-sides", "\\c 1\n\\p\n\\v 1 word \\f + \\ft x\\f* next\n",
    {"ok": [["1", "1", "word next"]]}, "notes")
add("note-inside-word", "\\c 1\n\\p\n\\v 1 wo\\f + \\ft x\\f*rd\n", {"ok": [["1", "1", "word"]]}, "notes")
add("note-space-before-only", "\\c 1\n\\p\n\\v 1 word \\f + \\ft x\\f*, next\n",
    {"ok": [["1", "1", "word , next"]]}, "notes")
add("xref-after-open-paren", "\\c 1\n\\p\n\\v 1 (\\x - \\xo 1:1 \\xt Gen 1:1\\x*c\u2019est dit)\n",
    {"ok": [["1", "1", "(c\u2019est dit)"]]}, "notes")
add("xref-full-parts", "\\c 1\n\\p\n\\v 1 gerahs\\x - \\xo 1:1: \\xk k \\xq q \\xt Exod 30:13; \\xta also \\xt Lev 27:25\\x*);\n",
    {"ok": [["1", "1", "gerahs);"]]}, "notes")
add("endnote", "\\c 1\n\\p\n\\v 1 alpha\\fe + \\fr 1:1 \\ft end note\\fe* beta\n", {"ok": [["1", "1", "alpha beta"]]}, "notes")
add("extended-notes", "\\c 1\n\\p\n\\v 1 alpha\\ef - \\fr 1:1 \\ft study\\ef* beta\\ex - \\xo 1:1 \\xt Ruth 1:2\\ex*.\n",
    {"ok": [["1", "1", "alpha beta."]]}, "notes")
add("note-parts-fq-fqa-fk-fv-fp", "\\c 1\n\\p\n\\v 1 a\\f + \\fr 1:1 \\fk key: \\fq quote \\fqa alt \\fv 2\\fv* \\fp para \\ft t\\f* b\n",
    {"ok": [["1", "1", "a b"]]}, "notes")
add("note-nested-plus-spans", "\\c 1\n\\p\n\\v 1 a\\f + \\ft see \\+it this\\+it* and \\+xt Ruth 1:1|link-href=\"RUT 1:1\"\\+xt*\\f* b\n",
    {"ok": [["1", "1", "a b"]]}, "notes")
add("note-with-bar", "\\c 1\n\\p\n\\v 1 a\\f + \\ft x|y\\f* b\n", {"ok": [["1", "1", "a b"]]}, "notes")
add("note-at-verse-start", "\\c 1\n\\p\n\\v 1 \\f + \\ft x\\f*Alpha beta.\n", {"ok": [["1", "1", "Alpha beta."]]}, "notes")
add("note-then-combining-mark", "\\c 1\n\\p\n\\v 1 cafe\\f + \\ft x\\f*\u0301 au lait\n",
    {"ok": [["1", "1", "cafe\u0301 au lait"]]}, "notes")
add("two-notes-adjacent", "\\c 1\n\\p\n\\v 1 a.\\f + \\ft x\\f*\\x - \\xt y\\x*\u201d b\n", {"ok": [["1", "1", "a.\u201d b"]]}, "notes")
# ---- closing markers, nested spans
add("nested-plus-span", "\\c 1\n\\p\n\\v 1 \\add added \\+nd Lord\\+nd*\\add* text\n",
    {"ok": [["1", "1", "added Lord text"]]}, "spans")
add("wj-kept", "\\c 1\n\\p\n\\v 1 He said, \\wj \u201cCome.\u201d\\wj*\n", {"ok": [["1", "1", "He said, \u201cCome.\u201d"]]}, "spans")
add("span-closed-by-other-marker", "\\c 1\n\\p\n\\v 1 \\add text\\nd*\n\\v 2 Two.\n",
    {"refuse": True, "labels": ["1", "2"], "keep": {"1:2": "Two."}}, "malformed")
add("span-unclosed-until-next-verse", "\\c 1\n\\p\n\\v 1 \\add text\n\\v 2 Two.\n\\v 3 Three.\n",
    {"refuse": True, "labels": ["1", "2", "3"], "keep": {"1:2": "Two.", "1:3": "Three."}}, "malformed")
add("span-nested-without-plus", "\\c 1\n\\p\n\\v 1 \\add a \\nd b\\nd*\\add*\n", {"refuse": True, "labels": ["1"]}, "malformed")
add("note-closed-by-other-type", "\\c 1\n\\p\n\\v 1 a\\f + \\ft x\\x* b\n\\v 2 Two.\n",
    {"refuse": True, "labels": ["1", "2"], "keep": {"1:2": "Two."}}, "malformed")
add("xref-unclosed-before-later-note", "\\c 1\n\\p\n\\v 1 a\\x - \\xt y b\n\\v 2 c\\f + \\ft z\\f* d\n\\v 3 Three.\n",
    {"refuse": True, "labels": ["1", "2", "3"], "keep": {"1:3": "Three."}}, "malformed")
add("verse-inside-note", "\\c 1\n\\p\n\\v 1 a\\f + \\ft x \\v 2 Two.\\f*\n\\v 3 Three.\n",
    {"refuse": True, "labels": ["1", "2", "3"], "keep": {"1:3": "Three."}}, "malformed")
add("note-inside-note", "\\c 1\n\\p\n\\v 1 a\\f + \\ft x\\x - \\xt y\\x*\\f* b\n", {"refuse": True, "labels": ["1"]}, "malformed")
add("wj-across-poetry-line", "\\c 1\n\\q1\n\\v 1 \\wj Come\n\\q2 and see.\\wj*\n\\v 2 Two.\n",
    {"refuse": True, "labels": ["1", "2"], "keep": {"1:2": "Two."}}, "malformed")
add("plus-span-unclosed-inside-note", "\\c 1\n\\p\n\\v 1 a\\f + \\ft \\+it x\\f* b\n\\v 2 Two.\n",
    {"refuse": True, "labels": ["1", "2"], "keep": {"1:2": "Two."}}, "malformed")
add("stray-end-markers", "\\c 1\n\\p\n\\v 1 a\\f* b\\add* c\n", {"refuse": True, "labels": ["1"]}, "malformed")
# ---- white space
add("nbsp-kept", f"\\c 1\n\\p\n\\v 1 a{NB}b{NNB}c\u2009d\u3000e\n", {"ok": [["1", "1", f"a{NB}b{NNB}c\u2009d\u3000e"]]}, "whitespace")
add("tilde-is-nbsp", "\\c 1\n\\p\n\\v 1 a~b\n", {"ok": [["1", "1", f"a{NB}b"]]}, "whitespace")
add("double-slash-nothing", "\\c 1\n\\p\n\\v 1 a//b c // d\n", {"ok": [["1", "1", "ab c d"]]}, "whitespace")
add("tabs-newlines-collapse", "\\c 1\n\\p\n\\v 1 a\t\tb\n\n\nc  \n", {"ok": [["1", "1", "a b c"]]}, "whitespace")
add("crlf", "\\c 1\r\n\\p\r\n\\v 1 a\r\nb\r\n\\v 2 c\r\n", {"ok": [["1", "1", "a b"], ["1", "2", "c"]]}, "whitespace")
add("marker-two-spaces", "\\c 1\n\\p\n\\v 1 the \\nd  Lord\\nd*  said\n", {"ok": [["1", "1", "the Lord said"]]}, "whitespace")
add("verse-number-newline", "\\c 1\n\\p\n\\v 1\nalpha\n", {"ok": [["1", "1", "alpha"]]}, "whitespace")
add("end-marker-takes-no-space", "\\c 1\n\\p\n\\v 1 \\nd Lord\\nd* said\n", {"ok": [["1", "1", "Lord said"]]}, "whitespace")
add("glued-span", "\\c 1\n\\p\n\\v 1 word\\nd Lord\\nd*s\n", {"ok": [["1", "1", "wordLords"]]}, "whitespace")
add("invisible-format-kept", "\\c 1\n\\p\n\\v 1 a\u200db\u200cc\u00add\u2060e\ufefff\n",
    {"ok": [["1", "1", "a\u200db\u200cc\u00add\u2060e\ufefff"]]}, "whitespace")
add("bom-at-start", "\ufeff" + "\\c 1\n\\p\n\\v 1 a\n", {"ok": [["1", "1", "a"]]}, "whitespace")
add("paragraph-boundary-one-space", "\\c 1\n\\q1\n\\v 1 line one,\n\\q2 line two.\n\\b\n\\m and more\n",
    {"ok": [["1", "1", "line one, line two. and more"]]}, "whitespace")
# (The reviewer first expected the header row before verse 1 to read cleanly; it is text outside any
#  verse, which the rules refuse. Kept as a refusal; the in-verse table is the clean case.)
add("table-header-before-first-verse", "\\c 1\n\\tr \\th1 Tribe \\thr2 Number\n\\tr \\tc1 \\v 1 Judah \\tcr2 74,600\n",
    {"refuse": True, "labels": ["1"], "keep": {"1:1": "Judah 74,600"}}, "malformed")
add("table-cells", "\\c 1\n\\p\n\\v 1 The tribes:\n\\tr \\th1 Tribe \\thr2 Number\n\\tr \\tc1 Judah \\tcr2 74,600\n\\p\n\\v 2 b\n",
    {"ok": [["1", "1", "The tribes: Tribe Number Judah 74,600"], ["1", "2", "b"]]}, "whitespace")
# More markers in milestone form (\\name\\*): the reviewer expects refusal of every non-milestone marker so written.
add("poetry-as-milestone", "\\c 1\n\\q1\n\\v 1 line one,\\q2\\*line two\n", {"refuse": True}, "malformed")
add("note-as-milestone", "\\c 1\n\\p\n\\v 1 a\\f\\*b\n", {"refuse": True}, "malformed")
add("char-as-milestone", "\\c 1\n\\p\n\\v 1 a \\nd\\* b\n", {"refuse": True}, "malformed")
add("chapter-as-milestone-merges-without-duplicates", "\\c 1\n\\p\n\\v 1 a\n\\v 2 b\n\\c |n=\"2\"\\*\n\\p\n\\v 3 c\n",
    {"refuse": True}, "malformed")
# ---- milestones, attributes
add("quote-milestone", "\\c 1\n\\p\n\\v 1 He said, \\qt-s |who=\"A\"\\*\u201cGo.\u201d\\qt-e\\*\n",
    {"ok": [["1", "1", "He said, \u201cGo.\u201d"]]}, "milestones")
add("ts-standalone", "\\c 1\n\\p\n\\v 1 a\\ts\\* b\n", {"ok": [["1", "1", "a b"]]}, "milestones")
add("alignment-milestones", "\\c 1\n\\p\n\\v 1 \\zaln-s |x-strong=\"H1\" x-occurrence=\"1\"\\*\\w In|x-occurrence=\"1\"\\w*\\zaln-e\\* \\zaln-s |x-strong=\"H2\"\\*\\w the|x-occurrence=\"1\"\\w*\\zaln-e\\* beginning\n",
    {"ok": [["1", "1", "In the beginning"]]}, "milestones")
add("milestone-unterminated", "\\c 1\n\\p\n\\v 1 a \\qt-s |who=\"A\" b\n\\v 2 Two.\n",
    {"refuse": True, "labels": ["1", "2"], "keep": {"1:2": "Two."}}, "malformed")
add("milestone-unpaired", "\\c 1\n\\p\n\\v 1 a \\qt-s\\* b\n", {"refuse": True, "labels": ["1"]}, "malformed")
add("w-attributes", "\\c 1\n\\p\n\\v 1 \\w gracious|lemma=\"grace\" strong=\"G5485\"\\w* and \\w kind|kind\\w*\n",
    {"ok": [["1", "1", "gracious and kind"]]}, "milestones")
add("ruby", "\\c 1\n\\p\n\\v 1 \\rb \u795e|\u304b\u307f\\rb*\u306f\n", {"ok": [["1", "1", "\u795e\u306f"]]}, "milestones")
add("jmp", "\\c 1\n\\p\n\\v 1 see \\jmp here|link-href=\"#x\"\\jmp*.\n", {"ok": [["1", "1", "see here."]]}, "milestones")
add("fig-dropped", "\\c 1\n\\p\n\\v 1 a \\fig A field|src=\"f.jpg\" size=\"col\" ref=\"1:1\"\\fig* b\n", {"ok": [["1", "1", "a b"]]}, "milestones")
add("fig-usfm2-pipes", "\\c 1\n\\p\n\\v 1 a \\fig A field|f.jpg|col|||A field|1:1\\fig* b\n", {"ok": [["1", "1", "a b"]]}, "milestones")
add("attributes-interrupted", "\\c 1\n\\p\n\\v 1 \\w a|lemma=\"x\\nd y\\nd*\"\\w*\n", {"refuse": True, "labels": ["1"]}, "malformed")
add("bar-outside-span", "\\c 1\n\\p\n\\v 1 a | b\n", {"refuse": True, "labels": ["1"]}, "malformed")
# ---- bridges, alternate and published numbers
add("bridge", "\\c 1\n\\p\n\\v 1-2 a\n\\v 3 b\n", {"ok": [["1", "1-2", "a"], ["1", "3", "b"]]}, "numbering")
add("alternate-verse", "\\c 1\n\\p\n\\v 3 \\va 4\\va* text\n", {"ok": [["1", "3", "text"]]}, "numbering")
add("published-verse", "\\c 1\n\\p\n\\v 5 \\vp 5a\\vp* text\n", {"ok": [["1", "5", "text"]]}, "numbering")
add("alternate-chapter", "\\c 2\n\\ca 3\\ca*\n\\p\n\\v 1 a\n", {"ok": [["2", "1", "a"]]}, "numbering")
add("published-chapter", "\\c 2\n\\cp B\n\\p\n\\v 1 a\n", {"ok": [["2", "1", "a"]]}, "numbering")
add("chapter-label", "\\c 1\n\\cl Psalm 1\n\\d For the leader.\n\\q1\n\\v 1 Blessed\n", {"ok": [["1", "1", "Blessed"]]}, "numbering")
add("va-unclosed", "\\c 1\n\\p\n\\v 1 \\va 2 a\n\\v 2 Two.\n\\c 2\n\\p\n\\v 1 Next chapter.\n",
    {"refuse": True, "labels": ["1", "2", "1"], "keep": {"1:2": "Two.", "2:1": "Next chapter."}}, "malformed")
add("rq-dropped", "\\c 1\n\\p\n\\v 1 as written \\rq Isa 1:1\\rq*\n", {"ok": [["1", "1", "as written"]]}, "numbering")
add("label-as-written", "\\c 1\n\\p\n\\v 01 a\n\\v 2a b\n\\v 3,4 c\n", {"ok": [["1", "01", "a"], ["1", "2a", "b"], ["1", "3,4", "c"]]}, "numbering")
add("duplicate-verse-kept", "\\c 1\n\\p\n\\v 2 a\n\\v 2 b\n", {"ok": [["1", "2", "a"], ["1", "2", "b"]]}, "numbering")
add("title-numbered-as-verse", "\\c 3\n\\d \\v 1 A psalm of David.\n\\q1\n\\v 2 Lord, how many.\n",
    {"ok": [["3", "1", "A psalm of David."], ["3", "2", "Lord, how many."]]}, "numbering")
# ---- malformed structure (each must be refused, with every verse kept)
add("verse-number-missing", "\\c 1\n\\p\n\\v \\add a\\add*\n\\v 2 b\n", {"refuse": True, "keep": {"1:2": "b"}}, "malformed")
add("chapter-number-missing", "\\c\n\\p\n\\v 1 a\n", {"refuse": True}, "malformed")
add("unknown-marker", "\\c 1\n\\p\n\\v 1 a \\foo b\\foo* c\n", {"refuse": True, "labels": ["1"]}, "malformed")
add("private-marker", "\\c 1\n\\p\n\\v 1 a \\zfoo b\\zfoo* c\n", {"refuse": True, "labels": ["1"]}, "malformed")
add("lone-backslash", "\\c 1\n\\p\n\\v 1 a \\ b\n", {"refuse": True, "labels": ["1"]}, "malformed")
add("text-before-first-verse", "\\c 1\n\\p Some words.\n\\v 1 a\n", {"refuse": True, "labels": ["1"]}, "malformed")
add("two-id-lines", "\\c 1\n\\p\n\\v 1 a\n\\id RUT again\n\\c 1\n\\p\n\\v 1 b\n", {"refuse": True}, "malformed")
add("control-character", "\\c 1\n\\p\n\\v 1 a\u0007b\n", {"refuse": True, "labels": ["1"]}, "malformed")
add("verse-close-stray-milestone-end", "\\c 1\n\\p\n\\v 3\\* a\n", {"refuse": True}, "malformed")
add("sidebar-unclosed", "\\c 1\n\\p\n\\v 1 a\n\\esb\n\\p sidebar text\n\\v 2 b\n", {"refuse": True, "keep": {"1:2": "b"}}, "malformed")
add("sidebar-closed", "\\c 1\n\\p\n\\v 1 a\n\\esb\n\\ms Box\n\\p sidebar text\n\\esbe\n\\p\n\\v 2 b\n",
    {"ok": [["1", "1", "a"], ["1", "2", "b"]]}, "structure")
add("verse-as-milestone", "\\c 1\n\\p\n\\v 1 a\n\\v\\* 2 b\n\\v 3 c\n", {"refuse": True}, "malformed")
add("plus-v", "\\c 1\n\\p\n\\v 1 a \\+v 2 b\n", {"refuse": True}, "malformed")
add("v-without-space-number", "\\c 1\n\\p\n\\v1 a\n\\v 2 b\n", {"refuse": True, "keep": {"1:2": "b"}}, "malformed")
# structural and paragraph markers written in milestone form: the reviewer expects refusal
add("chapter-as-milestone", "\\c 1\n\\p\n\\v 1 a\n\\v 2 b\n\\c \\* 2\n\\p\n\\v 1 c\n\\v 2 d\n",
    {"refuse": True}, "malformed")
add("paragraph-as-milestone", "\\c 1\n\\p\n\\v 1 a\\p\\*b\n", {"refuse": True}, "malformed")
add("id-as-milestone", "\\id\\* XYZ\n\\c 1\n\\p\n\\v 1 a\n", {"refuse": True}, "malformed")
# ---- Unicode (kept code point for code point; no normalisation)
add("german-nfd", "\\c 1\n\\p\n\\v 1 Gro\u0308\u00dfe Freude, u\u0308ber alles.\n", {"ok": [["1", "1", "Gro\u0308\u00dfe Freude, u\u0308ber alles."]]}, "unicode")
add("korean-syllables-and-jamo", "\\c 1\n\\p\n\\v 1 \ud0dc\ucd08\uc5d0 \u1112\u1161\u11ab\u1102\u1173\u11af\n", {"ok": [["1", "1", "\ud0dc\ucd08\uc5d0 \u1112\u1161\u11ab\u1102\u1173\u11af"]]}, "unicode")
add("greek-polytonic-nfd", "\\c 1\n\\p\n\\v 1 \u0395\u0313\u03bd \u1f00\u03c1\u03c7\u1fc7 \u1f26\u03bd \u1f41 \u03bb\u03bf\u0301\u03b3\u03bf\u03c2\u0387\n",
    {"ok": [["1", "1", "\u0395\u0313\u03bd \u1f00\u03c1\u03c7\u1fc7 \u1f26\u03bd \u1f41 \u03bb\u03bf\u0301\u03b3\u03bf\u03c2\u0387"]]}, "unicode")
add("hebrew-cantillation-rlm-maqaf", "\\c 1\n\\p\n\\v 1 \u05d1\u05bc\u05b0\u05e8\u05b5\u05d0\u05e9\u05b4\u0591\u05d9\u05ea \u05d1\u05bc\u05b8\u05e8\u05b8\u05a3\u05d0\u05be\u05d0\u05b1\u05dc\u05b9\u05d4\u05b4\u0591\u05d9\u05dd\u200f\u05c3\n",
    {"ok": [["1", "1", "\u05d1\u05bc\u05b0\u05e8\u05b5\u05d0\u05e9\u05b4\u0591\u05d9\u05ea \u05d1\u05bc\u05b8\u05e8\u05b8\u05a3\u05d0\u05be\u05d0\u05b1\u05dc\u05b9\u05d4\u05b4\u0591\u05d9\u05dd\u200f\u05c3"]]}, "unicode")
add("arabic-persian-zwnj", "\\c 1\n\\p\n\\v 1 \u0645\u06cc\u200c\u062e\u0648\u0627\u0647\u0645 \u0641\u06cc \u0627\u0644\u0628\u064e\u062f\u0621\u0650\n",
    {"ok": [["1", "1", "\u0645\u06cc\u200c\u062e\u0648\u0627\u0647\u0645 \u0641\u06cc \u0627\u0644\u0628\u064e\u062f\u0621\u0650"]]}, "unicode")
add("other-scripts", "\\c 1\n\\p\n\\v 1 \u0710\u0712\u0718\u0722 \u1260\u1218\u1300\u1218\u122a\u12EB \u0906\u0926\u093f \u592a\u521d\u6709\u9053 \u0e43\u0e19\u0e1b\u0e10\u0e21\u0e01\u0e32\u0e25\n",
    {"ok": [["1", "1", "\u0710\u0712\u0718\u0722 \u1260\u1218\u1300\u1218\u122a\u12EB \u0906\u0926\u093f \u592a\u521d\u6709\u9053 \u0e43\u0e19\u0e1b\u0e10\u0e21\u0e01\u0e32\u0e25"]]}, "unicode")
add("astral-and-compat", "\\c 1\n\\p\n\\v 1 \U0001D400 \ufb01 \u2126 \u212b \U00010330\n", {"ok": [["1", "1", "\U0001D400 \ufb01 \u2126 \u212b \U00010330"]]}, "unicode")
add("unicode-line-separators", "\\c 1\n\\p\n\\v 1 a\u2028b\u2029c\n", {"ok": [["1", "1", "a\u2028b\u2029c"]]}, "unicode")
add("c1-control-nel", "\\c 1\n\\p\n\\v 1 a\u0085b\n", {"refuse": True, "labels": ["1"]}, "unicode")
# ---- identity and accounting
add("book-code-lower-case", "\\c 1\n\\p\n\\v 1 a\n", {"ok": [["1", "1", "a"]]}, "identity")
add("verses-before-chapter", "\\p\n\\v 1 stray\n\\c 1\n\\p\n\\v 1 a\n", {"ok": [["1", "1", "a"]], "before": 1}, "identity")
add("chapter-repeated", "\\c 1\n\\p\n\\v 1 a\n\\c 1\n\\p\n\\v 1 b\n", {"ok": [["1", "1", "a"], ["1", "1", "b"]]}, "identity")
add("heading-mid-verse", "\\c 1\n\\p\n\\v 1 a\n\\s Heading\n\\p\nb\n\\v 2 c\n", {"ok": [["1", "1", "a b"], ["1", "2", "c"]]}, "identity")
add("verse-label-nbsp-glued", f"\\c 1\n\\p\n\\v 1{NB}a b\n", {"choice": "a no-break space after the number joins it to the label"}, "identity")
add("psalm-title-after-verse-number", "\\c 3\n\\v 1 \\d A psalm of David.\n\\q1 Lord, how many.\n",
    {"choice": "a \\d title after \\v: is the title verse text?"}, "identity")
for v in V:
    if v["name"] == "book-code-lower-case":
        v["usfm"] = v["usfm"].replace("\\id RUT", "\\id rut")
json.dump(V, open(sys.argv[1], "w", encoding="utf-8"), ensure_ascii=False, indent=1)
print(len(V), "vectors")
