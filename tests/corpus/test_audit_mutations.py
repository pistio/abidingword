#!/usr/bin/env python3
"""Disposable mutation qualifications for the USFM->DOCX completeness auditor.

Run in a checkout, without Actions or corpus access:
    python3 -m unittest discover -s tests/corpus -p 'test_audit_mutations.py' -v

All Bible-looking content here is invented test text, not Scripture. No
external package, real Bible document, source download or production DB used.
"""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
import zipfile
from xml.sax.saxutils import escape

ROOT = Path(__file__).resolve().parents[2]
AUDITOR = ROOT / "scripts" / "corpus" / "audit-derived-completeness.py"
W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"

def paragraph(style=None, verse=None, words=""):
    ppr = f'<w:pPr><w:pStyle w:val="{style}"/></w:pPr>' if style else ""
    number = (
        f'<w:r><w:rPr><w:vertAlign w:val="superscript"/></w:rPr>'
        f'<w:t>{escape(str(verse))}</w:t></w:r>'
    ) if verse is not None else ""
    content = f'<w:r><w:t xml:space="preserve">{escape(words)}</w:t></w:r>'
    return f"<w:p>{ppr}{number}{content}</w:p>"

def make_docx(path, *, book="Genesis", chapter="Chapter 1", verses=None):
    verses = verses if verses is not None else [
        ("1", "Alpha beta gamma delta"), ("2", "Epsilon zeta")
    ]
    body = [paragraph(style="Heading1", words=book),
            paragraph(style="Heading2", words=chapter)]
    body.extend(paragraph(verse=n, words=" " + words) for n, words in verses)
    styles = (
        f'<w:styles xmlns:w="{W}">'
        '<w:style w:styleId="Heading1"><w:name w:val="Heading 1"/></w:style>'
        '<w:style w:styleId="Heading2"><w:name w:val="Heading 2"/></w:style>'
        '</w:styles>'
    )
    document = f'<w:document xmlns:w="{W}"><w:body>{"".join(body)}</w:body></w:document>'
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("word/document.xml", document)
        z.writestr("word/styles.xml", styles)

def make_usfm(path, *, marker="mi"):
    # A new text-bearing paragraph marker should be included, not dropped.
    contents = (
        "\\id GEN\n\\h Genesis\n\\c 1\n\\p\n"
        "\\v 1 Alpha beta\n"
        + "\\" + marker + " gamma delta\n"
        + "\\v 2 Epsilon zeta\n"
    )
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("01GEN.usfm", contents)

class CompletenessMutationTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.base = Path(self.tmp.name)
        self.source = self.base / "source_usfm.zip"
        self.docx = self.base / "derived.docx"
        self.output = self.base / "audit.json"
        make_usfm(self.source)
        make_docx(self.docx)

    def audit(self, complete, expected_reference=None):
        proc = subprocess.run(
            [sys.executable, str(AUDITOR), str(self.source),
             str(self.docx), f"--json={self.output}"],
            capture_output=True, text=True, timeout=30, check=False
        )
        self.assertTrue(self.output.is_file(),
                        f"Audit did not emit report: {proc.stdout} {proc.stderr}")
        report = json.loads(self.output.read_text(encoding="utf-8"))
        self.assertEqual(proc.returncode == 0, complete,
                         f"Unexpected audit status: {proc.stdout} {proc.stderr}")
        self.assertEqual(report["complete"], complete)
        if expected_reference is not None:
            unresolved = json.dumps(
                [report.get(k) for k in
                 ("verses_incomplete", "verses_missing_from_document",
                  "verses_only_in_document")], ensure_ascii=False)
            self.assertIn(expected_reference, unresolved)
        return report

    def test_complete_two_verse_document(self):
        report = self.audit(True)
        self.assertEqual(report["verses_complete"], 2)

    def test_detect_missing_mi_continuation(self):
        make_docx(self.docx, verses=[("1", "Alpha beta"), ("2", "Epsilon zeta")])
        report = self.audit(False, "GEN 1:1")
        self.assertGreater(report["words_missing"], 0)

    def test_detect_unknown_text_bearing_paragraph(self):
        make_usfm(self.source, marker="zz")
        self.audit(True)
        make_docx(self.docx, verses=[("1", "Alpha beta"), ("2", "Epsilon zeta")])
        self.audit(False, "GEN 1:1")

    def test_detect_truncated_verse(self):
        make_docx(self.docx, verses=[("1", "Alpha beta gamma"), ("2", "Epsilon zeta")])
        self.audit(False, "GEN 1:1")

    def test_detect_added_words(self):
        make_docx(self.docx, verses=[("1", "Alpha beta gamma delta INSERTED"),
                                     ("2", "Epsilon zeta")])
        self.audit(False, "GEN 1:1")

    def test_detect_reordered_words(self):
        make_docx(self.docx, verses=[("1", "Alpha gamma beta delta"),
                                     ("2", "Epsilon zeta")])
        self.audit(False, "GEN 1:1")

    def test_detect_wrong_chapter(self):
        make_docx(self.docx, chapter="Chapter 2")
        self.audit(False, "GEN 1:1")

    def test_detect_wrong_verse_number(self):
        make_docx(self.docx, verses=[("3", "Alpha beta gamma delta"),
                                     ("2", "Epsilon zeta")])
        self.audit(False, "GEN 1:1")

    def test_detect_wrong_book_heading(self):
        make_docx(self.docx, book="Exodus")
        report = self.audit(False)
        self.assertTrue(report["verses_missing_from_document"])

    def test_detect_missing_entire_verse(self):
        make_docx(self.docx, verses=[("1", "Alpha beta gamma delta")])
        self.audit(False, "GEN 1:2")

    def test_detect_extra_verse(self):
        make_docx(self.docx, verses=[("1", "Alpha beta gamma delta"),
                                     ("2", "Epsilon zeta"), ("3", "Invented fixture words")])
        self.audit(False, "GEN 1:3")

if __name__ == "__main__":
    unittest.main()
