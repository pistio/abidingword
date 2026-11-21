"""The corrections of evaluation candidate A1.5-E2, each shown on SYNTHETIC
material (invented sentences, labelled as such) through the real checker,
runner and scorer, with the fake harness of test_withheld_set_tools.py.

F1  strict text fidelity: case, word boundaries, punctuation and marks are
    wrong text; the scorer classes every record again and distrusts a
    harness that disagrees.
B1  verse zero, B2 other content the 66-book canon cannot hold: declared
    before the run, reported in the score, never dropped; a harness whose
    inventory disagrees with the check's is not trusted.
B3  a verified verse, chapter or book the edition does not have is found by
    the scorer's own full-set lookup, whatever the harness reports.
C1  official scoring takes no override; a diagnostic score says so.
C2  declared composition, checked against the manifest and the protocol.
C3  books are stated, never taken from a file name.
C4  a verse map or other file in a reference archive is refused.
C5  a reference that is not UTF-8 is refused; a decomposed one is noted.

    python3 -m unittest discover -s tests/evaluation -p 'test_*.py'
"""
import io
import json
import os
import subprocess
import sys
import unittest
import zipfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from test_withheld_set_tools import (CHECK, SCORE, Fixture, SetFixture, control, reference_usfm, report,
                                     reference_inventory, sha)

ROOT = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))


def load(name, path):
    import importlib.util
    spec = importlib.util.spec_from_file_location(name, path)
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m


fidelity = load("fidelity", os.path.join(ROOT, "scripts", "evaluation", "fidelity.py"))


def zipped(members):
    buf = io.BytesIO()
    with zipfile.ZipFile(buf, "w") as z:
        for name, data in members:
            z.writestr(name, data.encode("utf-8") if isinstance(data, str) else data)
    return buf.getvalue()


def usfm(book, chapters):
    """chapters: [(chapter label, [verse labels])], invented text."""
    lines = [f"\\id {book} SYNTHETIC test reference, invented text"]
    for c, verses in chapters:
        lines.append(f"\\c {c}")
        lines += [f"\\v {v} Invented verse {c}:{v} of {book}." for v in verses]
    return "\n".join(lines) + "\n"


def e2_report(inv, records=(), absent=(), unvalidatable=(), extra_verified=(), drop=None):
    """An E2 report verifying every scorable verse of the reference the
    Python inventory `inv` read, with the given non-exact `records`
    (reference, edition, found, class, kind), and verified references the
    edition lacks (`absent`, `unvalidatable`, as (reference, reason)).
    `extra_verified`: verified references listed but not reported absent
    (what the A1.5 harness did with a chapter the reference lacks)."""
    keys = sorted(reference_inventory.expand(inv["scorable_ranges"]))
    labels = [f"{b} {c}:{v}" for b, c, v in keys]
    recs = [{"reference": r, "status": "verified", "unverified_reason": None, "class": c, "kind": k,
             "matched": None, "page": None, "edition": e, "found": f} for r, e, f, c, k in records]
    count = lambda c: sum(1 for r in recs if r["class"] == c)
    j = report(0, 0)
    j.pop("_reference_usfm")
    j["totals"] = {"expected": len(keys), "verified": len(keys),
                   "verified_exact": len(keys) - len(recs), "verified_canonical": count("canonical"),
                   "verified_layout": count("layout"), "verified_mismatch": count("mismatch"),
                   "verified_mismatch_kinds": {}, "held": 0, "held_faithful": 0, "held_mismatch": 0,
                   "refused": 0, "not_found": 0}
    j["fidelity_records"] = recs
    j["verified_references"] = labels + [r for r, _ in absent] + [r for r, _ in unvalidatable] + list(extra_verified)
    j["verified_absent"] = [{"reference": r, "reason": why} for r, why in absent]
    j["verified_unvalidatable"] = [{"reference": r, "reason": why} for r, why in unvalidatable]
    from test_withheld_set_tools import harness_inventory
    j["reference_inventory"] = harness_inventory(inv)
    if drop:
        drop(j)
    return j


class E2Set(SetFixture):
    """A set of one scored document whose reference and report are built
    together: `build(inv)` returns the report for the reference read."""

    def __init__(self, tmp, reference_bytes, build, books="all", **kw):
        path = os.path.join(tmp, "probe-ref.zip")
        os.makedirs(tmp, exist_ok=True)
        open(path, "wb").write(reference_bytes)
        declared = None if books == "all" else set(books.split(","))
        inv = reference_inventory.read(path, declared)
        os.remove(path)
        super().__init__(tmp, [{"key": "S1", "class": "scored", "kind": "authentic", "books": books,
                                "reference_bytes": reference_bytes,
                                "behaviour": {"write": {".json": build(inv)}}}], allowed={}, **kw)


class E2Fixture(Fixture):
    def e2(self, reference_bytes, build, **kw):
        s = E2Set(self.tmp, reference_bytes, build, **kw)
        r, out = s.run(check=True)
        r, score = s.score(out)
        self.assertEqual(r.returncode, 0, r.stderr)
        return s, out, score


OBA = zipped([("31-OBA.usfm", usfm("OBA", [(1, range(1, 22))]))])


class StrictFidelity(E2Fixture):
    def test_the_golden_vectors(self):
        vectors = json.load(open(os.path.join(ROOT, "tests", "evaluation", "fidelity-vectors.json")))
        self.assertEqual(vectors["rules"], fidelity.RULES)
        for v in vectors["vectors"]:
            got = fidelity.judge(v["edition"], v["found"], v.get("matched", False))
            self.assertEqual(got, (v["class"], v["kind"]), v["id"])

    def test_case_word_boundaries_punctuation_and_marks_are_wrong_text(self):
        records = [("OBA 1:1", "She shall be set apart.", "She shall be set a part.", "mismatch", "word_boundary"),
                   ("OBA 1:2", "The LORD is my keeper.", "The Lord is my keeper.", "mismatch", "case"),
                   ("OBA 1:3", "It's the elders' doing.", "Its the elders' doing.", "mismatch", "punctuation"),
                   ("OBA 1:4", "él que espera", "el que espera", "mismatch", "marks")]
        _, _, score = self.e2(OBA, lambda inv: e2_report(inv, records))
        a = score["assessment"]
        self.assertEqual(a["verdict"], "fail")
        self.assertFalse(a["criteria"]["1_no_verified_text_mismatch"])
        self.assertEqual(a["scored"]["verified_mismatch"], 4)
        self.assertEqual(a["scored"]["verified_mismatch_kinds"],
                         {"case": 1, "marks": 1, "punctuation": 1, "word_boundary": 1})
        self.assertEqual(a["scored"]["verified_faithful"], 17)
        self.assertTrue(a["run_validity"]["reports_trusted"], a["integrity"])

    def test_canonical_and_layout_text_is_faithful(self):
        records = [("OBA 1:1", "él que espera", "él que espera", "canonical", None),
                   ("OBA 1:2", "The shepherd rested.", "The shepherd  rested.", "layout", None)]
        _, _, score = self.e2(OBA, lambda inv: e2_report(inv, records))
        a = score["assessment"]
        self.assertEqual(a["verdict"], "pass", a)
        self.assertEqual((a["scored"]["verified_canonical"], a["scored"]["verified_layout"]), (1, 1))
        self.assertEqual(a["scored"]["verified_faithful"], 21)

    def test_the_scorer_classes_every_record_again(self):
        # The harness calls a word split "layout"; the scorer does not agree.
        records = [("OBA 1:1", "She shall be set apart.", "She shall be set a part.", "layout", None)]
        _, _, score = self.e2(OBA, lambda inv: e2_report(inv, records))
        a = score["assessment"]
        self.assertEqual(a["verdict"], "not a protocol run")
        self.assertFalse(a["run_validity"]["reports_trusted"])
        self.assertIn("disagreement(s) between harness and scorer", "; ".join(a["integrity"]["S1"]))
        self.assertEqual(a["scored"]["verified_mismatch"], 1)  # the scorer's own count
        self.assertFalse(a["criteria"]["1_no_verified_text_mismatch"])

    def test_a_report_of_the_a15_harness_is_not_trusted(self):
        old = {"totals": {"expected": 21, "verified": 21}, "differences": [], "verified_extra_verses": []}
        _, _, score = self.e2(OBA, lambda inv: old)
        a = score["assessment"]
        self.assertEqual(a["verdict"], "not a protocol run")
        self.assertIn("not a report of the E2 harness", "; ".join(a["integrity"]["S1"]))


ODD = zipped([("02-EXO.usfm", usfm("EXO", [(16, ["0"] + [str(v) for v in range(1, 37)])])),
              ("29-JOL.usfm", usfm("JOL", [(1, range(1, 21)), (4, range(1, 22))])),
              ("68-TOB.usfm", usfm("TOB", [(1, range(1, 23))])),
              ("64-3JN.usfm", usfm("3JN", [(1, ["1", "2", "12a", "201", "7", "7"])]))])


class ReferenceCompleteness(E2Fixture):
    def test_unsupported_content_is_declared_before_the_run(self):
        s = E2Set(self.tmp, ODD, lambda inv: e2_report(inv))
        r = subprocess.run(["python3", CHECK, s.seal, s.docs, f"--inventory={s.inventory}",
                            f"--protocol={s.protocol}"], capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        inv = json.loads(r.stdout)["reference_inventories"]["S1"]
        u = inv["unsupported_labels"]
        self.assertEqual(u["verse_zero"], ["EXO 16:0"])
        self.assertEqual(len(u["chapter_beyond_canon"]), 21)
        self.assertEqual(len(u["book_outside_canon"]), 22)
        self.assertEqual(u["verse_label_unusual"], ["3JN 1:12a", "3JN 1:201"])
        self.assertEqual(u["verse_duplicate"], ["3JN 1:7", "3JN 1:7"])
        self.assertIn("reference content the recogniser cannot hold", r.stderr)

    def test_unsupported_content_is_in_the_score_and_its_denominator(self):
        _, _, score = self.e2(ODD, lambda inv: e2_report(inv))
        a = score["assessment"]
        self.assertEqual(a["verdict"], "pass", a["integrity"])
        self.assertEqual(a["scored"]["printed"], 36 + 20 + 2)
        self.assertEqual(a["scored"]["printed_unsupported"], 1 + 21 + 22 + 2 + 2)
        self.assertEqual(a["scored"]["unsupported"], {"book_outside_canon": 22, "chapter_beyond_canon": 21,
                                                       "verse_duplicate": 2, "verse_label_unusual": 2,
                                                       "verse_zero": 1})
        self.assertIn("48 verses the recogniser cannot hold", a["accuracy_claim"])
        self.assertLess(a["scored"]["coverage_percent_of_all_printed"], a["scored"]["coverage_percent"])

    def test_a_harness_that_drops_verse_zero_is_not_trusted(self):
        def drop(j):
            j["reference_inventory"]["unsupported"]["verse_zero"] = {"printed": 0, "empty": 0, "references": []}
            j["reference_inventory"]["labels"] -= 1
        _, _, score = self.e2(ODD, lambda inv: e2_report(inv, drop=drop))
        a = score["assessment"]
        self.assertEqual(a["verdict"], "not a protocol run")
        self.assertIn("reference inventories differ", "; ".join(a["integrity"]["S1"]))

    def test_unsupported_scripture_verified_as_another_verse_is_named(self):
        # The document verifies JOL 1:1 with the words of the reference's
        # JOL 4:1, a chapter the recogniser cannot hold.
        text = "Invented verse 4:1 of JOL."
        def misread(j):
            j["fidelity_records"][0]["matched"] = "JOL 4:1"
        records = [("JOL 1:1", "Invented verse 1:1 of JOL.", text, "mismatch", "wrong_reference")]
        _, _, score = self.e2(ODD, lambda inv: e2_report(inv, records, drop=misread))
        doc = score["documents"][0]["observed"]
        self.assertEqual(doc["verified_misidentified_unsupported"], ["JOL 1:1 = JOL 4:1"])
        self.assertEqual(doc["verified_mismatch_kinds"], {"wrong_reference": 1})
        self.assertEqual(score["assessment"]["verdict"], "fail")

    def test_accounting_that_loses_verses_is_not_trusted(self):
        def lose(j):
            j["totals"]["expected"] += 5  # printed verses that are nowhere
        _, _, score = self.e2(OBA, lambda inv: e2_report(inv, drop=lose))
        self.assertIn("accounting", "; ".join(score["assessment"]["integrity"]["S1"]))
        self.assertEqual(score["assessment"]["verdict"], "not a protocol run")


MAL3 = zipped([("39-MAL.usfm", usfm("MAL", [(1, range(1, 15)), (2, range(1, 18)), (3, range(1, 25))]))])
MAL4 = [f"MAL 4:{v}" for v in range(1, 7)]


class NonexistentReferences(E2Fixture):
    def test_a_whole_extra_chapter_verified_fails_criterion_2(self):
        absent = [(r, "chapter_not_in_book") for r in MAL4]
        _, _, score = self.e2(MAL3, lambda inv: e2_report(inv, absent=absent))
        a = score["assessment"]
        self.assertEqual(a["verdict"], "fail")
        self.assertFalse(a["criteria"]["2_no_verified_reference_outside_or_unvalidatable"])
        self.assertEqual(a["scored"]["verified_absent"], 6)
        self.assertTrue(a["run_validity"]["reports_trusted"], a["integrity"])

    def test_the_a15_blind_spot_is_found_by_the_scorer(self):
        # As the A1.5 harness did: MAL 4 verified, reported nowhere.
        _, _, score = self.e2(MAL3, lambda inv: e2_report(inv, extra_verified=MAL4))
        a = score["assessment"]
        self.assertEqual(a["verdict"], "not a protocol run")
        self.assertIn("verified references absent from the edition: harness 0, scorer 6",
                      "; ".join(a["integrity"]["S1"]))

    def test_a_book_the_reference_lacks(self):
        absent = [(f"RUT 1:{v}", "book_not_in_reference") for v in range(1, 4)]
        _, _, score = self.e2(OBA, lambda inv: e2_report(inv, absent=absent))
        self.assertEqual(score["assessment"]["scored"]["verified_absent"], 3)
        self.assertEqual(score["assessment"]["verdict"], "fail")

    def test_a_number_the_edition_gives_twice_cannot_be_validated(self):
        ref = zipped([("31-OBA.usfm", usfm("OBA", [(1, ["1", "2", "3", "3", "4"])]))])
        _, _, score = self.e2(ref, lambda inv: e2_report(inv, unvalidatable=[("OBA 1:3", "verse_duplicate")]))
        a = score["assessment"]
        self.assertEqual(a["scored"]["verified_unvalidatable"], 1)
        self.assertFalse(a["criteria"]["2_no_verified_reference_outside_or_unvalidatable"])
        self.assertTrue(a["run_validity"]["reports_trusted"], a["integrity"])


class Safeguards(E2Fixture):
    def good(self, **kw):
        return E2Set(self.tmp, OBA, lambda inv: e2_report(inv), **kw)

    # C1
    def test_official_scoring_refuses_every_override(self):
        s = self.good()
        _, out = s.run(check=True)
        for option in (f"--frozen-binary={s.fake_sha}", f"--inventory={s.inventory}", f"--protocol={s.protocol}"):
            r = subprocess.run(["python3", SCORE, out, s.seal, f"--run-record={s.record}", option],
                               capture_output=True, text=True)
            self.assertEqual(r.returncode, 2, option)
            self.assertIn("an official assessment takes none", r.stderr)

    def test_a_diagnostic_score_says_so(self):
        s = self.good()
        _, out = s.run(check=True)
        md = os.path.join(self.tmp, "score.md")
        r = subprocess.run(["python3", SCORE, out, s.seal, f"--run-record={s.record}", "--diagnostic",
                            f"--protocol={s.protocol}", f"--inventory={s.inventory}", f"--md={md}"],
                           capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertTrue(open(md).read().startswith("# DIAGNOSTIC SCORE: not an official assessment"))
        _, score = s.score(out)
        self.assertEqual(score["mode"], "diagnostic")
        self.assertEqual(score["assessment"]["official_verdict"], "not a protocol run")
        self.assertEqual(sorted(score["overrides"]), ["inventory", "protocol"])

    def test_an_official_score_of_a_rehearsal_binary_is_not_a_protocol_run(self):
        s = self.good()
        _, out = s.run(check=True)
        r, score = s.score(out, official=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        a = score["assessment"]
        self.assertEqual(score["mode"], "official")
        self.assertEqual(a["verdict"], "not a protocol run")
        self.assertFalse(a["run_validity"]["frozen_binary"])
        self.assertFalse(a["run_validity"]["run_under_this_protocol"])

    # C2
    def test_a_composition_that_is_not_the_manifests_is_refused(self):
        s = self.good(composition={"scored": 2})
        self.assertIn("composition.tsv declares", self.refused(s))

    def test_a_composition_the_protocol_does_not_require_is_refused(self):
        s = self.good(required_composition={"scored": 23, "control": 3, "mixed": 2, "exploratory": 0,
                                            "authentic": 16, "generated": 12})
        self.assertIn("is not the protocol's required", self.refused(s))

    def test_a_protocol_without_a_required_composition_runs_nothing_as_protocol(self):
        s = self.good(required_composition=None)
        _, out = s.run(check=True)
        _, score = s.score(out)
        a = score["assessment"]
        self.assertFalse(a["run_validity"]["composition_required_by_protocol_and_met"])
        self.assertEqual(a["verdict"], "not a protocol run")

    def test_a_seal_without_a_composition_is_refused(self):
        def remove(fixture):
            os.remove(os.path.join(fixture.seal, "composition.tsv"))
        self.assertIn("SEAL.txt must list composition.tsv", self.refused(self.good(tamper=remove)))

    # C3
    def test_books_must_be_stated(self):
        s = E2Set(self.tmp, OBA, lambda inv: e2_report(inv), books="")
        self.assertIn("books is empty", self.refused(s))

    def test_book_codes_must_be_usfm_codes(self):
        s = E2Set(self.tmp, OBA, lambda inv: e2_report(inv), books="OBA,OBADIAH")
        self.assertIn("OBADIAH are not USFM book codes", self.refused(s))

    def test_a_report_that_chose_books_by_file_name_is_not_trusted(self):
        def by_name(j):
            j["book_selection"] = {"rule": "file name", "books": ["GEN"], "unknown_codes": []}
        _, _, score = self.e2(OBA, lambda inv: e2_report(inv, drop=by_name))
        self.assertIn("books chosen from a file name", "; ".join(score["assessment"]["integrity"]["S1"]))

    # C4
    def test_a_reference_with_a_verse_map_is_refused(self):
        ref = zipped([("31-OBA.usfm", usfm("OBA", [(1, range(1, 22))])),
                      ("VerseMap.xml", "<verseMap><book osisID=\"Obad\"></book></verseMap>")])
        s = E2Set(self.tmp, ref, lambda inv: e2_report(inv))
        self.assertIn("a verse map, which would renumber the reference", self.refused(s))

    def test_a_reference_with_another_file_is_refused(self):
        ref = zipped([("31-OBA.usfm", usfm("OBA", [(1, range(1, 22))])), ("words.txt", "not USFM"),
                      ("copr.htm", "<p>prose is allowed</p>")])
        err = self.refused(E2Set(self.tmp, ref, lambda inv: e2_report(inv)))
        self.assertIn("words.txt: not a USFM book", err)
        self.assertNotIn("copr.htm", err)

    # C5
    def test_a_reference_that_is_not_utf8_is_refused(self):
        ref = zipped([("31-OBA.usfm", usfm("OBA", [(1, range(1, 22))]).encode() + b"\\v 22 caf\xe9\n")])
        s = E2Set(self.tmp, ref, lambda inv: e2_report(inv))
        self.assertIn("not UTF-8", self.refused(s))

    def test_a_decomposed_reference_is_noted_and_compared_canonically(self):
        import unicodedata
        text = unicodedata.normalize("NFD", usfm("OBA", [(1, range(1, 22))]).replace("Invented", "Inventé"))
        s = E2Set(self.tmp, zipped([("31-OBA.usfm", text)]), lambda inv: e2_report(inv))
        r = subprocess.run(["python3", CHECK, s.seal, s.docs, f"--inventory={s.inventory}",
                            f"--protocol={s.protocol}"], capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("not in Unicode NFC", r.stderr)


if __name__ == "__main__":
    unittest.main()
