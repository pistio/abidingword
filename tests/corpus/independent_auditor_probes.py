#!/usr/bin/env python3
"""Adversarial probes for scripts/corpus/audit-derived-completeness.py.

Written by an independent reviewer agent (Phase A1.5) that was given the
auditor's claim and told to break it. It built its own fixtures and did
not use the auditor's own tests. Kept as written, apart from where the
fixtures go and how the tool is found. 83 probes: each a (USFM, DOCX)
pair with the verdict an honest audit must give.

    python3 tests/corpus/independent_auditor_probes.py [tool] [tool options…]

Standard library only. All fixture words are invented, not Scripture.
"""
import json
import os
import subprocess
import sys
import unicodedata
import zipfile
from xml.sax.saxutils import escape

import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
FX = os.environ.get("AW_PROBE_DIR") or tempfile.mkdtemp(prefix="aw-auditor-probes-")
TOOL = os.path.join(HERE, "..", "..", "scripts", "corpus", "audit-derived-completeness.py")
EXTRA = ["--profile=build-derived-editions-usfm"]
NS = 'xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"'


# ---------- DOCX mini-DSL ----------
def T(text, **props):
    """A run. props: sup, b, i, vanish, color, sz, strike, rstyle, pos."""
    rpr = ""
    if props.get("rstyle"):
        rpr += f'<w:rStyle w:val="{props["rstyle"]}"/>'
    if props.get("b"):
        rpr += "<w:b/>"
    if props.get("i"):
        rpr += "<w:i/>"
    if props.get("strike"):
        rpr += "<w:strike/>"
    if props.get("vanish"):
        rpr += "<w:vanish/>"
    if props.get("color"):
        rpr += f'<w:color w:val="{props["color"]}"/>'
    if props.get("sz"):
        rpr += f'<w:sz w:val="{props["sz"]}"/>'
    if props.get("sup"):
        rpr += '<w:vertAlign w:val="superscript"/>'
    if rpr:
        rpr = f"<w:rPr>{rpr}</w:rPr>"
    return f'<w:r>{rpr}<w:t xml:space="preserve">{escape(text)}</w:t></w:r>'


def V(label):
    return T(label, sup=True)


def RAW(xml):
    return xml


def P(*runs, style=None, jc=None):
    ppr = ""
    if style:
        ppr += f'<w:pStyle w:val="{style}"/>'
    if jc:
        ppr += f'<w:jc w:val="{jc}"/>'
    if ppr:
        ppr = f"<w:pPr>{ppr}</w:pPr>"
    return f"<w:p>{ppr}{''.join(runs)}</w:p>"


def H1(t):
    return P(T(t), style="Heading1")


def H2(t):
    return P(T(t), style="Heading2")


STYLES = (
    f'<?xml version="1.0" encoding="UTF-8"?><w:styles {NS}>'
    '<w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/></w:style>'
    '<w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/></w:style>'
    '<w:style w:type="character" w:styleId="VerseNo"><w:name w:val="Verse No"/>'
    '<w:rPr><w:vertAlign w:val="superscript"/></w:rPr></w:style>'
    '<w:style w:type="character" w:styleId="Ghost"><w:name w:val="Ghost"/>'
    "<w:rPr><w:vanish/></w:rPr></w:style>"
    "</w:styles>"
)


def write_docx(path, paragraphs, footnotes=None):
    body = "".join(paragraphs)
    doc = f'<?xml version="1.0" encoding="UTF-8"?><w:document {NS}><w:body>{body}</w:body></w:document>'
    with zipfile.ZipFile(path, "w") as z:
        z.writestr("[Content_Types].xml", '<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"/>')
        z.writestr("word/document.xml", doc)
        z.writestr("word/styles.xml", STYLES)
        if footnotes:
            z.writestr("word/footnotes.xml", f'<?xml version="1.0"?><w:footnotes {NS}><w:footnote w:id="1">{footnotes}</w:footnote></w:footnotes>')


def write_zip(path, files):
    with zipfile.ZipFile(path, "w") as z:
        for name, content in files.items():
            z.writestr(name, content if isinstance(content, bytes) else content.encode("utf-8"))


# ---------- baseline ----------
HEAD = "\\id GEN\n\\h Zorbel\n\\mt1 Zorbel\n"
BASE = HEAD + "\\c 1\n\\p\n\\v 1 blim narf quux\n\\v 2 dorf melk sarn tib\n"
V1, V2 = "blim narf quux", "dorf melk sarn tib"


def doc(*body):
    return [H1("Zorbel"), H2("Chapter 1"), *body]


BASEDOC = doc(P(V("1"), T(V1 + " "), V("2"), T(V2)))

PROBES = []


def probe(name, usfm, paragraphs, expect, what, footnotes=None):
    """expect: 'complete' or 'incomplete' (what an honest audit must say)."""
    if isinstance(usfm, str):
        usfm = {"01GEN.usfm": usfm}
    PROBES.append((name, usfm, paragraphs, expect, what, footnotes))


C, I = "complete", "incomplete"

# ===== controls =====
probe("c01-baseline", BASE, BASEDOC, C, "nothing")
probe("c02-word-dropped", BASE, doc(P(V("1"), T("blim quux "), V("2"), T(V2))), I, "doc drops 'narf'")
probe("c03-word-altered", BASE, doc(P(V("1"), T("blim narv quux "), V("2"), T(V2))), I, "doc alters one letter")
probe("c04-words-swapped", BASE, doc(P(V("1"), T("narf blim quux "), V("2"), T(V2))), I, "doc swaps two words in a verse")
probe("c05-word-moved-next-verse", BASE, doc(P(V("1"), T("blim narf "), V("2"), T("quux " + V2))), I, "last word of v1 moved to v2")
probe("c06-verse-missing", BASE, doc(P(V("1"), T(V1))), I, "doc lacks v2")
probe("c07-notes-excluded",
      HEAD + "\\c 1\n\\p\n\\v 1 blim\\f + \\fr 1.1 \\ft grolp vex\\f* narf \\x - \\xo 1.1 \\xt Zed 2.3\\x*quux\n\\v 2 dorf melk sarn tib\n",
      BASEDOC, C, "source has \\f and \\x notes; doc omits them")
probe("c08-w-attributes",
      HEAD + "\\c 1\n\\p\n\\v 1 \\w blim|lemma=\"bl\" strong=\"H1\"\\w* \\w narf|x-occ=\"1\"\\w* quux\n\\v 2 dorf melk sarn tib\n",
      BASEDOC, C, "\\w word|attrs\\w*")
probe("c09-word-split-across-runs", BASE, doc(P(V("1"), T("bl"), T("im na", b=True), T("rf quux "), V("2"), T(V2))), C, "words split across runs")
probe("c10-nested-char", HEAD + "\\c 1\n\\p\n\\v 1 \\wj blim \\+nd narf\\+nd* quux\\wj*\n\\v 2 dorf melk sarn tib\n", BASEDOC, C, "nested \\+nd in \\wj")
probe("c11-poetry", HEAD + "\\c 1\n\\q1\n\\v 1 blim\n\\q2 narf quux\n\\b\n\\q1\n\\v 2 dorf melk\n\\q2 sarn tib\n",
      doc(P(V("1"), T("blim")), P(T("narf quux")), P(V("2"), T("dorf melk")), P(T("sarn tib"))), C, "\\q1 \\q2 \\b, doc one paragraph per line")
TBL = HEAD + "\\c 1\n\\p\n\\v 1 blim\n\\tr \\tc1 narf \\tc2 quux\n\\tr \\tc1 dorf \\tcr2 melk\n\\v 2 sarn tib\n"
probe("c12-table-kept", TBL,
      doc(P(V("1"), T("blim")),
          "<w:tbl><w:tr><w:tc>" + P(T("narf")) + "</w:tc><w:tc>" + P(T("quux")) + "</w:tc></w:tr>"
          "<w:tr><w:tc>" + P(T("dorf")) + "</w:tc><w:tc>" + P(T("melk")) + "</w:tc></w:tr></w:tbl>",
          P(V("2"), T("sarn tib"))), C, "\\tr \\tc table -> Word table")
probe("c13-table-cell-dropped", TBL,
      doc(P(V("1"), T("blim")),
          "<w:tbl><w:tr><w:tc>" + P(T("narf")) + "</w:tc><w:tc>" + P(T("quux")) + "</w:tc></w:tr>"
          "<w:tr><w:tc>" + P(T("dorf")) + "</w:tc></w:tr></w:tbl>",
          P(V("2"), T("sarn tib"))), I, "doc drops a table cell")
probe("c14-custom-z-paragraph-dropped", HEAD + "\\c 1\n\\p\n\\v 1 blim\n\\zodd narf quux\n\\v 2 dorf melk sarn tib\n",
      doc(P(V("1"), T("blim "), V("2"), T(V2))), I, "doc drops text under custom \\zodd paragraph")
HEB = "\u05d1\u05b0\u05bc\u05e8\u05b5\u05d0 \u05d6\u05b8\u05e8\u05b0\u05d1\u05b5\u05bc\u05dc \u05e0\u05b7\u05e8\u05b0\u05e3"
probe("c15-hebrew-pointed", HEAD + f"\\c 1\n\\p\n\\v 1 {HEB}\n", doc(P(V("1"), T(HEB))), C, "pointed Hebrew, identical")
probe("c16-hebrew-point-dropped", HEAD + f"\\c 1\n\\p\n\\v 1 {HEB}\n", doc(P(V("1"), T(HEB.replace("\u05bc", "", 1)))), I, "doc loses one dagesh")
KOR = "\ube14\ub9bc\uc774 \ub098\ub974\ud504\ub97c \ucfe0\uc6b1\uc2a4\ud588\ub2e4"
probe("c17-korean", HEAD + f"\\c 1\n\\p\n\\v 1 {KOR}\n", doc(P(V("1"), T(KOR))), C, "Korean, identical")
probe("c18-korean-syllable-dropped", HEAD + f"\\c 1\n\\p\n\\v 1 {KOR}\n", doc(P(V("1"), T(KOR.replace("\ub97c", "")))), I, "doc drops one Hangul syllable")
NFC = "bl\u00e9m n\u00e4rf"
probe("c19-nfd-vs-nfc", HEAD + f"\\c 1\n\\p\n\\v 1 {NFC}\n", doc(P(V("1"), T(unicodedata.normalize("NFD", NFC)))), I,
      "doc is NFD, source NFC (code points altered; strict tool should flag)")
probe("c20-bridges", HEAD + "\\c 1\n\\p\n\\v 1-2 blim narf\n\\v 3a quux\n\\v 3b dorf\n",
      doc(P(V("1-2"), T("blim narf "), V("3a"), T("quux "), V("3b"), T("dorf"))), C, "\\v 1-2, 3a, 3b")
probe("c21-va-vp-ca-cp", HEAD + "\\c 1\n\\ca 2\\ca*\n\\cp B\n\\p\n\\v 1 \\va 3\\va* \\vp 1b\\vp* blim narf quux\n\\v 2 dorf melk sarn tib\n",
      BASEDOC, C, "\\ca \\cp \\va \\vp present in source")
probe("c22-s-heading-inside-verse", HEAD + "\\c 1\n\\p\n\\v 1 blim\n\\s1 Grolp Vex\n\\p narf quux\n\\v 2 dorf melk sarn tib\n",
      doc(P(V("1"), T("blim")), P(T("Grolp Vex", b=True), jc="center"), P(T("narf quux "), V("2"), T(V2))), C, "\\s1 inside v1, doc has bold-centred heading")
probe("c23-tracked-deletion", BASE,
      doc(P(V("1"), T("blim "), '<w:del w:id="1" w:author="x"><w:r><w:delText>narf </w:delText></w:r></w:del>', T("quux "), V("2"), T(V2))), I, "'narf' is a tracked deletion (w:del)")
probe("c24-text-moved-to-footnote", BASE, doc(P(V("1"), T("blim quux "), V("2"), T(V2))), I, "'narf' only in word/footnotes.xml", footnotes=P(T("narf")))
probe("c25-tab-and-br", HEAD + "\\c 1\n\\q1\n\\v 1 blim\n\\q2 narf quux\n",
      doc(P(V("1"), "<w:r><w:t>blim</w:t><w:br/></w:r>", "<w:r><w:tab/><w:t>narf</w:t></w:r>", T(" quux"))), C, "w:br / w:tab between words")
probe("c26-wrong-chapter", HEAD + "\\c 1\n\\p\n\\v 1 blim narf\n\\c 2\n\\p\n\\v 1 quux dorf\n",
      [H1("Zorbel"), H2("Chapter 1"), P(V("1"), T("quux dorf")), H2("Chapter 2"), P(V("1"), T("blim narf"))], I, "chapter 1 and 2 texts exchanged")
probe("c27-wrong-book", {"01GEN.usfm": BASE, "02EXO.usfm": "\\id EXO\n\\h Vandrel\n\\c 1\n\\p\n\\v 1 plix omba\n"},
      [H1("Zorbel"), H2("1"), P(V("1"), T("plix omba")), H1("Vandrel"), H2("1"), P(V("1"), T(V1 + " "), V("2"), T(V2))], I, "books' texts exchanged")
probe("c28-verse-text-before-first-heading", BASE, [P(T("narf")), *doc(P(V("1"), T("blim quux "), V("2"), T(V2)))], I, "'narf' moved before the first heading")
probe("c29-unterminated-note-no-closer", HEAD + "\\c 1\n\\p\n\\v 1 blim narf quux \\f + \\ft grolp vex\n\\v 2 dorf melk sarn tib\n",
      BASEDOC, I, "malformed source: \\f never closed (loud failure is right)")
probe("c30-d-before-v", HEAD + "\\c 1\n\\d\n\\v 1 blim narf quux\n\\q1\n\\v 2 dorf melk sarn tib\n",
      doc(P(V("1"), T(V1, i=True)), P(V("2"), T(V2))), C, "\\d holding \\v 1 (verse text kept, italic with number)")
probe("c31-d-before-v-dropped", HEAD + "\\c 1\n\\d\n\\v 1 blim narf quux\n\\q1\n\\v 2 dorf melk sarn tib\n",
      doc(P(V("1")), P(V("2"), T(V2))), I, "doc drops the text of \\d \\v 1")

# ===== suspected escapes: tool should say incomplete =====
probe("e01-vacuous-empty-zip", {}, [P(T("nothing here"))], I, "USFM zip has no files; doc has no books")
probe("e02-vacuous-readme-only", {"README.txt": "grolp vex"}, [], I, "zip has only a README; doc empty")
TOB = "\\id TOB\n\\h Quendle\n\\c 1\n\\p\n\\v 1 frab jost wimmel\n"
probe("e03-non66-book-ignored", {"01GEN.usfm": BASE, "70TOB.usfm": TOB}, BASEDOC, I, "source has TOB (outside the 66); doc lacks it entirely")
probe("e04-only-non66-book", {"70TOB.usfm": TOB}, [], I, "source is only TOB; doc empty")
probe("e05-lowercase-id", {"01GEN.usfm": BASE, "02exo.usfm": "\\id exo\n\\h Vandrel\n\\c 1\n\\p\n\\v 1 plix omba\n"}, BASEDOC, I, "second book has '\\id exo' (lower case); doc lacks it")
probe("e06-utf16-file", {"01GEN.usfm": BASE, "02EXO.usfm": ("\\id EXO\n\\h Vandrel\n\\c 1\n\\p\n\\v 1 plix omba\n").encode("utf-16")}, BASEDOC, I, "second book file is UTF-16; doc lacks it")
probe("e07-duplicate-book-id", {"01GEN-a.usfm": HEAD + "\\c 1\n\\p\n\\v 1 frab jost\n\\v 2 wimmel\n\\v 3 ploon\n", "01GEN-b.usfm": BASE}, BASEDOC, I,
      "two files both \\id GEN with different verses; doc has only the second file's text")
probe("e08-merged-words", BASE, doc(P(V("1"), T("blimnarfquux "), V("2"), T("dorfmelksarntib"))), I, "doc lost every space inside the verses")
probe("e09-word-boundary-shifted", BASE, doc(P(V("1"), T("bli mnar fquux "), V("2"), T(V2))), I, "doc moves the spaces: 'bli mnar fquux'")
probe("e10-korean-spacing-lost", HEAD + f"\\c 1\n\\p\n\\v 1 {KOR}\n", doc(P(V("1"), T(KOR.replace(" ", "")))), I, "Korean with all word spaces removed")
probe("e11-verses-reordered", BASE, doc(P(V("2"), T(V2 + " "), V("1"), T(V1))), I, "doc prints v2 before v1")
probe("e12-verse-text-interleaved", BASE, doc(P(V("1"), T("blim "), V("2"), T("dorf melk "), V("1"), T("narf quux "), V("2"), T("sarn tib"))), I,
      "half of v1 printed after half of v2 under a repeated number")
probe("e13-chapters-reordered", HEAD + "\\c 1\n\\p\n\\v 1 blim narf\n\\c 2\n\\p\n\\v 1 quux dorf\n",
      [H1("Zorbel"), H2("Chapter 2"), P(V("1"), T("quux dorf")), H2("Chapter 1"), P(V("1"), T("blim narf"))], I, "doc prints chapter 2 before chapter 1")
probe("e14-books-reordered", {"01GEN.usfm": BASE, "02EXO.usfm": "\\id EXO\n\\h Vandrel\n\\c 1\n\\p\n\\v 1 plix omba\n"},
      [H1("Vandrel"), H2("1"), P(V("1"), T("plix omba")), H1("Zorbel"), H2("1"), P(V("1"), T(V1 + " "), V("2"), T(V2))], I, "doc prints EXO before GEN")
probe("e15-hidden-vanish", BASE, doc(P(V("1"), T("blim "), T("narf ", vanish=True), T("quux "), V("2"), T(V2, vanish=True))), I, "'narf' and all of v2 are hidden text (w:vanish)")
probe("e16-hidden-via-char-style", BASE, doc(P(V("1"), T("blim "), T("narf ", rstyle="Ghost"), T("quux "), V("2"), T(V2))), I, "'narf' hidden by a character style with w:vanish")
probe("e17-white-1pt-text", BASE, doc(P(V("1"), T("blim "), T("narf ", color="FFFFFF", sz="2"), T("quux "), V("2"), T(V2))), I, "'narf' is white, 1pt")
probe("e18-tracked-move-away", BASE,
      doc(P('<w:moveTo w:id="2" w:author="x"><w:r><w:t>narf</w:t></w:r></w:moveTo>'),
          P(V("1"), T("blim "), '<w:moveFrom w:id="1" w:author="x"><w:r><w:t xml:space="preserve">narf </w:t></w:r></w:moveFrom>', T("quux "), V("2"), T(V2))), I,
      "'narf' is tracked-moved out of v1 (w:moveFrom) to before the first verse number")
probe("e19-rq-swallows-rest", HEAD + "\\c 1\n\\p\n\\v 1 blim \\rq Zed 4.2\\rq* narf quux\n\\v 2 dorf melk sarn tib\n",
      doc(P(V("1"), T("blim "), V("2"), T(V2))), I, "inline \\rq …\\rq* in v1; doc drops the verse words after it")
probe("e20-unterminated-note-swallows-verses",
      HEAD + "\\c 1\n\\p\n\\v 1 blim \\f + \\ft grolp vex\n\\v 2 dorf melk sarn tib\n\\v 3 frab jost\n\\v 4 wimmel \\f + \\ft ploon\\f* quendle\n",
      doc(P(V("1"), T("blim quendle"))), I, "\\f in v1 never closed, next \\f* is in v4; doc lacks v2, v3 and most of v4")
probe("e21-unterminated-va-swallows-verses",
      HEAD + "\\c 1\n\\p\n\\v 1 \\va 2 blim narf\n\\v 2 dorf melk sarn tib\n\\v 3 \\va 4\\va* frab\n",
      doc(P(V("1"), T("frab"))), I, "\\va in v1 never closed; doc lacks v2 and v3")
probe("e22-text-between-c-and-v", HEAD + "\\c 1\n\\p\n\\v 1 blim narf quux\n\\c 2\n\\p frab jost wimmel\n\\v 1 dorf melk sarn tib\n",
      [H1("Zorbel"), H2("Chapter 1"), P(V("1"), T(V1)), H2("Chapter 2"), P(V("1"), T(V2))], I, "running text between \\c 2 and its first \\v; doc drops it")
probe("e23-text-on-c-line", HEAD + "\\c 1 frab jost wimmel\n\\p\n\\v 1 blim narf quux\n\\v 2 dorf melk sarn tib\n", BASEDOC, I,
      "words on the \\c line after the number; doc drops them (and the report does not count them)")
probe("e24-duplicate-h1-first-discarded", BASE,
      [H1("Zorbel"), H2("Chapter 1"), P(V("1"), T("ALTERED frab jost "), V("2"), T("wimmel")), *BASEDOC], I,
      "doc has the book twice; the first copy is altered")
probe("e25-alien-italic-paragraph-in-verse", BASE, doc(P(V("1"), T(V1)), P(T("frab jost wimmel ploon", i=True)), P(V("2"), T(V2))), I,
      "doc inserts an all-italic paragraph of foreign words between v1 and v2")
probe("e26-alien-bold-centred-in-verse", BASE, doc(P(V("1"), T(V1)), P(T("frab jost wimmel", b=True), jc="center"), P(V("2"), T(V2))), I,
      "doc inserts a bold-centred paragraph of foreign words")
probe("e27-alien-text-before-first-verse", BASE, doc(P(T("frab jost wimmel ploon")), P(V("1"), T(V1 + " "), V("2"), T(V2))), I,
      "doc inserts a plain paragraph of foreign words between chapter heading and v1")
probe("e28-d-inside-verse-dropped", HEAD + "\\c 1\n\\q1\n\\v 1 blim\n\\d narf quux\n\\q1\n\\v 2 dorf melk sarn tib\n",
      doc(P(V("1"), T("blim")), P(V("2"), T(V2))), I, "\\d after \\v 1 (words are inside v1 in the source); doc drops them")
probe("e29-sp-inside-verse-dropped", HEAD + "\\c 1\n\\q1\n\\v 1 blim\n\\sp narf quux\n\\q1\n\\v 2 dorf melk sarn tib\n",
      doc(P(V("1"), T("blim")), P(V("2"), T(V2))), I, "\\sp inside v1; doc drops its words")
probe("e30-ip-inside-verse-dropped", HEAD + "\\c 1\n\\p\n\\v 1 blim\n\\ip narf quux\n\\v 2 dorf melk sarn tib\n",
      doc(P(V("1"), T("blim "), V("2"), T(V2))), I, "an introduction marker (\\ip) after \\v 1; doc drops its words")
probe("e31-textbox-italic", BASE,
      doc(P(V("1"), T("blim "),
            "<w:r><w:pict><w:txbxContent>" + P(T("narf", i=True)) + "</w:txbxContent></w:pict></w:r>",
            T(" quux "), V("2"), T(V2))), I, "'narf' moved into a floating text box (italic)")
probe("e32-struck-through", BASE, doc(P(V("1"), T("blim "), T("narf ", strike=True), T("quux "), V("2"), T(V2))), I, "'narf' struck through (debatable: still visible)")
probe("e33-book-no-verses-malformed-v", {"01GEN.usfm": HEAD + "\\c 1\n\\p\n\\v1 blim narf quux\n\\v2 dorf melk\n"}, [H1("Zorbel")], I,
      "source writes \\v1 \\v2 (no space): no verse is recognised; doc has only the book heading")
probe("e34-two-books-in-one-file-second-missing",
      {"all.usfm": BASE + "\\id EXO\n\\h Vandrel\n\\c 2\n\\p\n\\v 1 plix omba\n"},
      [H1("Vandrel"), H2("Chapter 1"), P(V("1"), T(V1 + " "), V("2"), T(V2)), H2("2"), P(V("1"), T("plix omba"))], I,
      "one file holds GEN then EXO; doc files everything under the last \\h")
probe("e35-textbox-plain", BASE,
      doc(P(V("1"), T("blim "),
            "<w:r><w:pict><w:txbxContent>" + P(T("narf")) + "</w:txbxContent></w:pict></w:r>",
            T(" quux "), V("2"), T(V2))), I, "'narf' moved into a floating text box (plain)")

# ===== suspected false alarms: tool should say complete =====
probe("f01-verse-number-split-runs", HEAD + "\\c 1\n\\p\n\\v 10 blim narf quux\n", doc(P(V("1"), V("0"), T(V1))), C, "superscript '10' written as two runs '1','0'")
probe("f02-superscript-by-char-style", BASE, doc(P(T("1", rstyle="VerseNo"), T(V1 + " "), T("2", rstyle="VerseNo"), T(V2))), C, "verse numbers raised by a character style, not direct formatting")
probe("f03-milestone-qt", HEAD + "\\c 1\n\\p\n\\v 1 \\qt-s |who=\"Vandrel\"\\*blim narf quux\\qt-e\\*\n\\v 2 dorf melk sarn tib\n", BASEDOC, C, "\\qt-s |who=…\\* … \\qt-e\\* milestones")
probe("f04-milestone-zaln", HEAD + "\\c 1\n\\p\n\\v 1 \\zaln-s |x-strong=\"H1\" x-content=\"zz\"\\*\\w blim|x-occurrence=\"1\"\\w*\\zaln-e\\* narf quux\n\\v 2 dorf melk sarn tib\n", BASEDOC, C, "\\zaln-s …\\* alignment milestones")
probe("f05-note-callers-in-doc", BASE, doc(P(V("1"), T("blim"), T("a", sup=True), T(" narf quux "), V("2"), T(V2))), C, "doc keeps a superscript note caller 'a' in v1")
probe("f06-nobreak-hyphen", HEAD + "\\c 1\n\\p\n\\v 1 blim-narf quux\n", doc(P(V("1"), "<w:r><w:t>blim</w:t><w:noBreakHyphen/><w:t>narf</w:t></w:r>", T(" quux"))), C, "hyphen written as w:noBreakHyphen")
probe("f07-fig", HEAD + "\\c 1\n\\p\n\\v 1 blim narf quux \\fig Grolp vex|src=\"a.jpg\" size=\"col\"\\fig*\n\\v 2 dorf melk sarn tib\n", BASEDOC, C, "\\fig caption|attrs\\fig* in source (not verse text)")
probe("f08-ef-extended-note", HEAD + "\\c 1\n\\p\n\\v 1 blim narf quux\\ef - \\fr 1.1 \\ft grolp vex\\ef*\n\\v 2 dorf melk sarn tib\n", BASEDOC, C, "extended note \\ef …\\ef*")
probe("f09-two-books-same-h", {"01GEN.usfm": BASE, "02EXO.usfm": "\\id EXO\n\\h Zorbel\n\\c 1\n\\p\n\\v 1 plix omba\n"},
      [*BASEDOC, H1("Zorbel"), H2("1"), P(V("1"), T("plix omba"))], C, "two books share the same \\h; doc has both, complete")
probe("f10-tab-after-verse-number", HEAD + "\\c 1\n\\p\n\\v 1\tblim narf quux\n\\v 2 dorf melk sarn tib\n", BASEDOC, C, "tab instead of space after \\v 1")
probe("f11-chapter-label-with-book-number", "\\id 1SA\n\\h 1 Zorbel\n\\c 2\n\\p\n\\v 1 blim narf quux\n",
      [H1("1 Zorbel"), H2("1 Zorbel 2"), P(V("1"), T(V1))], C, "chapter heading '1 Zorbel 2' (book name starts with a number)")
probe("f12-alternate-content", BASE,
      doc(P(V("1"), T("blim "),
            '<mc:AlternateContent xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006"><mc:Choice Requires="x">' + T("narf ") + "</mc:Choice><mc:Fallback>" + T("narf ") + "</mc:Fallback></mc:AlternateContent>",
            T("quux "), V("2"), T(V2))), C, "a word inside mc:AlternateContent (Choice + Fallback copies)")
probe("f13-centred-bold-verse-paragraph", HEAD + "\\c 1\n\\pc\n\\v 1 blim narf quux\n", doc(P(T("1", sup=True, b=True), T(V1, b=True), jc="center")), C, "\\pc inscription rendered bold and centred, with its verse number")
probe("f14-char-marker-at-line-start-after-heading", HEAD + "\\c 1\n\\p\n\\v 1 blim narf quux\n\\s1 Grolp of\n\\nd Vex\\nd* ploon\n\\p\n\\v 2 dorf melk sarn tib\n",
      doc(P(V("1"), T(V1)), P(T("Grolp of Vex ploon", b=True), jc="center"), P(V("2"), T(V2))), C, "heading wrapped onto a second line starting with \\nd")
probe("f15-italic-continuation-paragraph", HEAD + "\\c 1\n\\q1\n\\v 1 blim\n\\qc \\it narf quux\\it*\n",
      doc(P(V("1"), T("blim")), P(T("narf quux", i=True))), C, "a verse's second line is wholly italic in source and doc")
probe("f16-crlf", BASE.replace("\n", "\r\n"), BASEDOC, C, "CRLF line endings")
probe("f17-bom-nbsp", "\ufeff" + HEAD + "\\c 1\n\\p\n\\v 1 blim\u00a0narf quux\n\\v 2 dorf melk sarn tib\n", BASEDOC, C, "BOM; NBSP in source, plain space in doc")


def run():
    os.makedirs(FX, exist_ok=True)
    rows = []
    for name, usfm, paragraphs, expect, what, footnotes in PROBES:
        zp, dp, jp = (os.path.join(FX, name + ext) for ext in ("_usfm.zip", ".docx", ".json"))
        write_zip(zp, usfm)
        write_docx(dp, paragraphs, footnotes)
        if os.path.exists(jp):
            os.remove(jp)
        r = subprocess.run([sys.executable, TOOL, zp, dp, "--json=" + jp] + EXTRA, capture_output=True, text=True)
        if r.returncode == 0:
            actual = "complete"
        elif os.path.exists(jp):
            actual = "incomplete"
        else:
            actual = "CRASH"
        detail = ""
        if os.path.exists(jp):
            rep = json.load(open(jp, encoding="utf-8"))
            detail = "src_verses=%s ok=%s missing=%d only_doc=%d incompl=%d spaced=%d" % (
                rep["verses_in_source"], rep["verses_complete"], len(rep["verses_missing_from_document"]),
                len(rep["verses_only_in_document"]), len(rep["verses_incomplete"]),
                len(rep.get("verses_complete_but_spaced_differently", [])))
        else:
            detail = (r.stderr.strip().splitlines() or [""])[-1]
        if actual == expect:
            cls = "OK"
        elif actual == "complete":
            cls = "ESCAPE"
        elif actual == "CRASH":
            cls = "CRASH(" + ("fails closed" if expect == I else "false alarm") + ")"
        else:
            cls = "FALSE ALARM"
        rows.append(dict(probe=name, mutated=what, expected=expect, actual=actual, cls=cls, detail=detail))
        print(f"{cls:12} {name:44} exp={expect:10} act={actual:10} {detail}")
    json.dump(rows, open(os.path.join(FX, "results.json"), "w"), ensure_ascii=False, indent=1)
    count = {}
    for r in rows:
        count[r["cls"]] = count.get(r["cls"], 0) + 1
    print("\n", len(rows), "probes:", count)
    return rows


if __name__ == "__main__":
    if len(sys.argv) > 1:
        TOOL, EXTRA = sys.argv[1], sys.argv[2:]
    run()
