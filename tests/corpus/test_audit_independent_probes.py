#!/usr/bin/env python3
"""The independent reviewer's 83 probes as a test, and the cases added
after its review (Phase A1.5).

    python3 -m unittest discover -s tests/corpus -p 'test_audit_*.py' -v

Version 1 of the auditor let 33 of the probes' damaged documents pass.
The expectations below are the reviewer's own verdicts. Five probes do
not get the reviewer's verdict from version 2, each for a stated reason;
they are listed here so that any change to them is seen.
"""
import contextlib
import io
import json
import subprocess
import sys
import tempfile
import unittest
import warnings
import zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import independent_auditor_probes as probes  # noqa: E402

AUDITOR = HERE.parents[1] / "scripts" / "corpus" / "audit-derived-completeness.py"

# Refused although the document is complete: the auditor fails closed on
# what it does not positively understand.
FAILS_CLOSED = {
    "f05-note-callers-in-doc": "a raised run that is not a verse number",
    "f06-nobreak-hyphen": "a hyphen written as an element, not a character",
    "f11-chapter-label-with-book-number": "a chapter heading with two numbers",
    "f12-alternate-content": "alternate content (two copies of a word)",
}
# Passes although words of the source are not in the document: \sp (a
# speaker's name) is a heading the generator profile DECLARES it does not
# print. The words are counted and listed by verse in the report. Under
# the default profile ("strict") the same document fails (tested below).
DECLARED_OMISSION = {"e29-sp-inside-verse-dropped"}


class IndependentProbes(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        warnings.simplefilter("ignore", ResourceWarning)
        with contextlib.redirect_stdout(io.StringIO()):
            cls.rows = {r["probe"]: r for r in probes.run()}

    def test_there_are_83_probes(self):
        self.assertEqual(len(self.rows), 83)

    def test_no_damaged_document_passes(self):
        escapes = {n for n, r in self.rows.items() if r["cls"] == "ESCAPE"}
        self.assertEqual(escapes, DECLARED_OMISSION)

    def test_complete_documents_pass_but_for_the_listed_refusals(self):
        alarms = {n for n, r in self.rows.items() if r["cls"] == "FALSE ALARM"}
        self.assertEqual(alarms, set(FAILS_CLOSED))

    def test_nothing_crashes(self):
        self.assertEqual([n for n, r in self.rows.items() if r["cls"].startswith("CRASH")], [])

    def test_the_declared_omission_fails_under_the_strict_profile(self):
        fx = Path(probes.FX)
        out = fx / "strict.json"
        r = subprocess.run(
            [sys.executable, str(AUDITOR), str(fx / "e29-sp-inside-verse-dropped_usfm.zip"),
             str(fx / "e29-sp-inside-verse-dropped.docx"), f"--json={out}"],
            capture_output=True, text=True)
        self.assertEqual(r.returncode, 1)
        self.assertFalse(json.loads(out.read_text(encoding="utf-8"))["complete"])


W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"


class AfterTheReview(unittest.TestCase):
    """Cases the reviewer's probes did not hold."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.base = Path(self.tmp.name)

    def audit(self, usfm, paragraphs, *options):
        source, docx, out = self.base / "s_usfm.zip", self.base / "d.docx", self.base / "a.json"
        with zipfile.ZipFile(source, "w") as z:
            z.writestr("01GEN.usfm", usfm)
        styles = (f'<w:styles xmlns:w="{W}">'
                  '<w:style w:styleId="H1"><w:name w:val="heading 1"/></w:style>'
                  '<w:style w:styleId="H2"><w:name w:val="heading 2"/></w:style></w:styles>')
        body = "".join(paragraphs)
        with zipfile.ZipFile(docx, "w") as z:
            z.writestr("word/document.xml", f'<w:document xmlns:w="{W}"><w:body>{body}</w:body></w:document>')
            z.writestr("word/styles.xml", styles)
        r = subprocess.run([sys.executable, str(AUDITOR), str(source), str(docx), f"--json={out}", *options],
                           capture_output=True, text=True)
        report = json.loads(out.read_text(encoding="utf-8"))
        self.assertEqual(r.returncode == 0, report["complete"])
        return report

    @staticmethod
    def p(*runs, style=None):
        ppr = f'<w:pPr><w:pStyle w:val="{style}"/></w:pPr>' if style else ""
        return f"<w:p>{ppr}{''.join(runs)}</w:p>"

    @staticmethod
    def t(text):
        return f'<w:r><w:t xml:space="preserve">{text}</w:t></w:r>'

    @staticmethod
    def v(n):
        return f'<w:r><w:rPr><w:vertAlign w:val="superscript"/></w:rPr><w:t>{n}</w:t></w:r>'

    def doc(self, *body):
        return [self.p(self.t("Zorbel"), style="H1"), self.p(self.t("Chapter 1"), style="H2"), *body]

    def test_a_word_running_across_a_character_marker_is_one_word(self):
        # "\w blim|x\w*," is "blim,": version 2's first draft read "blim" and ",".
        usfm = '\\id GEN\n\\h Zorbel\n\\c 1\n\\p\n\\v 1 \\w blim|strong="H1"\\w*, \\w narf|strong="H2"\\w*.\n'
        report = self.audit(usfm, self.doc(self.p(self.v(1), self.t("blim, narf."))))
        self.assertTrue(report["complete"], report)
        report = self.audit(usfm, self.doc(self.p(self.v(1), self.t("blim , narf ."))))
        self.assertFalse(report["complete"])

    def test_control_characters_of_the_source_are_counted_not_compared(self):
        usfm = "\\id GEN\n\\h Zorbel\n\\c 1\n\\p\n\\v 1 blim\x01 narf\x03quux\n"
        report = self.audit(usfm, self.doc(self.p(self.v(1), self.t("blim narfquux"))))
        self.assertTrue(report["complete"], report)
        self.assertEqual(report["source_excluded"]["control characters (not printable)"], 2)

    def test_a_title_must_be_printed_unless_the_profile_declares_it_is_not(self):
        usfm = "\\id GEN\n\\h Zorbel\n\\mt1 The Scroll of Zorbel\n\\c 1\n\\p\n\\v 1 blim narf\n"
        document = self.doc(self.p(self.v(1), self.t("blim narf")))
        self.assertFalse(self.audit(usfm, document)["complete"])
        report = self.audit(usfm, document, "--profile=build-derived-editions-usfm")
        self.assertTrue(report["complete"], report)
        self.assertEqual(report["source_excluded"]["\\mt1 (words)"], 4)

    def test_a_book_outside_the_66_is_audited_unless_declared_not_rendered(self):
        usfm = "\\id TOB\n\\h Tobel\n\\c 1\n\\p\n\\v 1 frab jost\n"
        report = self.audit(usfm, [])
        self.assertFalse(report["complete"])
        report = self.audit(usfm, [], "--books=66")
        self.assertFalse(report["complete"])  # and then the source has no book left
        self.assertEqual(report["source_books_declared_not_rendered"], {"TOB": 1})

    def test_a_heading_of_the_source_set_as_plain_text_fails(self):
        usfm = "\\id GEN\n\\h Zorbel\n\\c 1\n\\p\n\\v 1 blim narf\n\\s1 Of the quux\n\\p\n\\v 2 dorf\n"
        heading = ('<w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:rPr><w:b/></w:rPr>'
                   '<w:t>Of the quux</w:t></w:r></w:p>')
        good = self.doc(self.p(self.v(1), self.t("blim narf")), heading, self.p(self.v(2), self.t("dorf")))
        self.assertTrue(self.audit(usfm, good)["complete"])
        bad = self.doc(self.p(self.v(1), self.t("blim narf Of the quux ")), self.p(self.v(2), self.t("dorf")))
        report = self.audit(usfm, bad)
        self.assertFalse(report["complete"])
        self.assertEqual(report["heading_words_set_as_text"], 3)


if __name__ == "__main__":
    unittest.main()
