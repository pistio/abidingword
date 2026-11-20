#!/usr/bin/env python3
"""The derivation script never drops text silently (Phase A1.5).

    python3 -m unittest discover -s tests/corpus -p 'test_derivation_*.py' -v

Invented words, standard library only. No converter is run: the layouts
are asked for no output format, which stops after the text is laid out.
"""
import importlib.util
import tempfile
import unittest
import zipfile
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[2] / "scripts" / "build-derived-editions.py"
spec = importlib.util.spec_from_file_location("build_derived_editions", SCRIPT)
build = importlib.util.module_from_spec(spec)
spec.loader.exec_module(build)


class UsfmLayout(unittest.TestCase):
    def lay_out(self, body):
        with tempfile.TemporaryDirectory() as tmp:
            with zipfile.ZipFile(Path(tmp) / "zzz_usfm.zip", "w") as z:
                z.writestr("01GEN.usfm", "\\id GEN\n\\h Zorbel\n\\c 1\n\\p\n\\v 1 blim narf\n" + body)
            build.from_usfm(tmp, "zzz", [], "Chapter {n}")

    def test_known_markers_are_laid_out(self):
        self.lay_out("\\mi quux dorf\n\\q1\n\\v 2 melk\n\\s1 Of the sarn\n\\sp Tib\n\\b\n")

    def test_an_unknown_marker_stops_the_build(self):
        with self.assertRaises(SystemExit) as stop:
            self.lay_out("\\zz quux dorf\n\\v 2 melk\n")
        self.assertIn("\\zz", str(stop.exception))

    def test_a_character_style_opening_a_line_is_text_not_a_stop(self):
        self.lay_out("\\wj quux dorf\\wj*\n\\v 2 melk\n")


class MarkdownLayout(unittest.TestCase):
    def lay_out(self, cls):
        page = ('<html lang="en"><body><div class="main">'
                f"<div class='{cls}'><span class=\"verse\" id=\"V1\">1&#160;</span>blim narf</div>"
                '</div><div class="copyright">x</div></body></html>')
        with tempfile.TemporaryDirectory() as tmp:
            with zipfile.ZipFile(Path(tmp) / "zzz_html.zip", "w") as z:
                z.writestr("index.htm", '<html lang="en"></html>')
                z.writestr("GEN.htm", "<h1><a href='index.htm'>Zorbel</a></h1>")
                z.writestr("GEN01.htm", page)
            import sys
            argv, sys.argv = sys.argv, ["x", tmp, "zzz", "md", "GEN"]
            try:
                build.main()
            finally:
                sys.argv = argv
            return (Path(tmp) / "zzz_GEN.md").read_text(encoding="utf-8")

    def test_an_indented_paragraph_is_written(self):
        # `mi` was left out before: the class of text the spablm DOCX lost.
        self.assertIn("1 blim narf", self.lay_out("mi"))

    def test_an_unknown_paragraph_class_stops_the_build(self):
        with self.assertRaises(SystemExit) as stop:
            self.lay_out("zz")
        self.assertIn("'zz'", str(stop.exception))


if __name__ == "__main__":
    unittest.main()
