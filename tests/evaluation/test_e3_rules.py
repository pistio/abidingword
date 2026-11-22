"""The corrections of evaluation candidate A1.5-E3, each shown on SYNTHETIC
material (invented sentences, labelled as such) through the real checker,
runner and scorer with the fake harness of test_withheld_set_tools.py, and
on the 13 real references bundled in resources/scripture/.

R1  the reference's verse text is read by the evaluation's own USFM readers
    (Rust in the harness, Python in the check), never by the app's; notes
    are removed without a space put in their place; malformed references
    are refused at the seal; the scorer compares both readings verse by
    verse; no reading loses a verse silently.
R2  a scored document's reference must print a scorable verse (seal); the
    verdict is "inconclusive", never "pass", below the protocol's evidence
    floor (by default: one faithful verified verse); safety and evidence are
    judged apart; every verdict line carries its counts.
T3  options are never silently ignored; tools are pinned by SHA-256 in the
    protocol; the protocol's execution gate; the publisher's archive is
    recorded apart from the USFM-only package; Unicode versions.

    python3 -m unittest discover -s tests/evaluation -p 'test_*.py'
"""
import glob
import hashlib
import json
import os
import random
import re
import subprocess
import sys
import unittest
import warnings
import zipfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from test_e2_rules import E2Fixture, E2Set, OBA, e2_report, usfm, zipped  # noqa: E402
from test_withheld_set_tools import (CHECK, SCORE, SEAL, ROOT, Fixture, SetFixture, approved_tools,  # noqa: E402
                                     control, reference_inventory, report, sha)

usfm_text = reference_inventory.usfm_text
warnings.simplefilter("ignore", ResourceWarning)
VECTORS = os.path.join(ROOT, "tests", "evaluation", "usfm-text-vectors.json")
BUNDLED = os.path.join(ROOT, "tests", "evaluation", "bundled-references.expected.json")
REVIEW_528 = os.path.join(ROOT, "docs", "audit", "candidate-a1.5", "evaluation-e2-review", "evidence", "inventory",
                          "note-space-differences.json")


def codes(result):
    return sorted({p["code"] for p in result["problems"]})


# ---------------------------------------------------------------- R1: the readers

class Reader(unittest.TestCase):
    def test_every_vector(self):
        doc = json.load(open(VECTORS, encoding="utf-8"))
        self.assertEqual(doc["rules"], usfm_text.RULES)
        self.assertGreaterEqual(len(doc["vectors"]), 50)
        for v in doc["vectors"]:
            r = usfm_text.read_text(v["usfm"])
            self.assertEqual([[x["chapter"], x["label"], x["text"]] for x in r["verses"]], v["verses"], v["id"])
            self.assertEqual(codes(r), v["problems"], v["id"])
            self.assertEqual(r["verses_before_chapter"], v.get("before_chapter", 0), v["id"])
            if "chapters" in v:
                self.assertEqual([{k: c[k] for k in ("label", "alternate", "published")} for c in r["chapters"]],
                                 v["chapters"], v["id"])

    def test_the_marker_table_is_the_rules_documents(self):
        with open(os.path.join(ROOT, "docs", "evaluation", "usfm-text-rules.md"), encoding="utf-8") as f:
            block = f.read().split("```aw-usfm-text-1 markers\n")[1].split("```")[0]
        table = {role: set(markers.split()) for role, markers in (l.split(": ", 1) for l in block.splitlines())}
        self.assertEqual(table, {"body": usfm_text.BODY, "other": usfm_text.OTHER, "cell": usfm_text.CELLS,
                                 "note": usfm_text.NOTES, "note part": usfm_text.NOTE_PARTS,
                                 "kept": usfm_text.KEPT, "dropped": usfm_text.DROPPED})

    def test_a_note_is_removed_without_a_space(self):
        # The review's R1 artefact, in invented text: the app's reader reads "heel. ”".
        r = usfm_text.read_text("\\id GEN SYNTHETIC\n\\c 1\n\\v 1 strike the heel.\\f + \\ft SYNTHETIC\\f*” Then\n")
        self.assertEqual(r["verses"][0]["text"], "strike the heel.” Then")

    def test_text_is_never_normalised(self):
        nfd, nbsp = "Mu\u0308ller", "a\u00a0b"
        r = usfm_text.read_text(f"\\id GEN SYNTHETIC\n\\c 1\n\\v 1 {nfd} {nbsp}\n")
        self.assertEqual(r["verses"][0]["text"], f"{nfd} {nbsp}")

    def test_no_mutation_loses_a_verse_silently(self):
        """Neither reader may omit a verse without a problem: here the Python
        one, on thousands of mutations (the Rust one has its own test, and
        evidence/readers/fuzz_readers.py runs both on 50 000)."""
        seeds = [v["usfm"] for v in json.load(open(VECTORS, encoding="utf-8"))["vectors"]]
        pieces = ["\\f + ", "\\f*", "\\x - ", "\\x*", "\\wj ", "\\wj*", "\\+w ", "\\+w*", "|", "\\va ", "\\va*",
                  "\\fig ", "\\qt-s |who=\"x\"\\*", "\\qt-e\\*", "\\*", "\\", "\\v ", "\\v 9 ", "\\c ", "\\c 7\n",
                  "\\p\n", "\\esb\n", "\\esbe\n", "\\add ", "\\nd*", "\\zz ", "\n"]
        rng = random.Random(1122)
        third = re.compile(r"\\v(?=[ \t\n\\]|$)")
        refused = clean = 0
        for _ in range(3000):
            s = rng.choice(seeds)
            for _ in range(rng.randint(1, 4)):
                k = rng.randint(0, len(s))
                op = rng.random()
                if op < 0.4:
                    s = s[:k] + rng.choice(pieces) + s[k:]
                elif op < 0.7:
                    m = s.find("\\", k)
                    if m >= 0:
                        e = m + 1
                        while e < len(s) and (s[e].isalnum() or s[e] in "+*-"):
                            e += 1
                        s = s[:m] + s[e:]
                else:
                    s = s[:k]
            r = usfm_text.read_text(s)
            if r["problems"]:
                refused += 1
                continue
            clean += 1
            text = s.lstrip("\ufeff").replace("\r\n", "\n").replace("\r", "\n")
            self.assertEqual(len(r["verses"]) + r["verses_before_chapter"], len(third.findall(text)), repr(s))
        self.assertGreater(refused, 500)
        self.assertGreater(clean, 300)


class BundledReferences(unittest.TestCase):
    """The 13 real eBible and Korean Bible Society references in the
    repository, read by the Python reader: no problem, every verse, the
    figures the Rust reader must also reproduce, and the review's 528."""

    @classmethod
    def setUpClass(cls):
        with open(BUNDLED, encoding="utf-8") as f:
            cls.expected = json.load(f)
        cls.read = {}
        for edition, want in cls.expected["editions"].items():
            path = os.path.join(ROOT, "resources", "scripture", edition, want["archive"])
            with open(path, "rb") as f:
                data = f.read()
            cls.read[edition] = {"sha256": hashlib.sha256(data).hexdigest(), "members": []}
            with zipfile.ZipFile(path) as z:
                for name in z.namelist():
                    if name.lower().endswith((".usfm", ".sfm")):
                        cls.read[edition]["members"].append((name, usfm_text.read_text(z.read(name).decode("utf-8"))))

    def test_all_thirteen_as_expected(self):
        self.assertEqual(len(self.expected["editions"]), 13)
        total = 0
        for edition, want in self.expected["editions"].items():
            got = self.read[edition]
            self.assertEqual(got["sha256"], want["archive_sha256"], edition)
            h = hashlib.sha256()
            verses = empty = before = 0
            for name, r in got["members"]:
                self.assertEqual(r["problems"], [], f"{edition} {name}")
                self.assertEqual(r["verse_markers"], len(r["verses"]) + r["verses_before_chapter"])
                before += r["verses_before_chapter"]
                for v in r["verses"]:
                    h.update(f"{name}\t{v['chapter']}\t{v['label']}\t{v['line']}\t"
                             f"{usfm_text.text_sha256(v['text'])}\n".encode("utf-8"))
                    verses += 1
                    empty += not v["text"]
            self.assertEqual((verses, empty, before, h.hexdigest()),
                             (want["verses"], want["numbered_but_empty"], want["verses_before_chapter"],
                              want["text_digest"]), edition)
            total += verses
        self.assertEqual(total, 311761)

    def test_the_review_528_read_as_the_edition_prints_them(self):
        with open(REVIEW_528, encoding="utf-8") as f:
            review = json.load(f)
        checked = 0
        for edition, found in review.items():
            texts = {}
            for _, r in self.read[edition]["members"]:
                for v in r["verses"]:
                    texts[f"{r['book']} {v['chapter']}:{v['label']}"] = v["text"]
            for row in found["rows"]:
                text = texts[row["reference"]]
                self.assertIn(row["independent_context"].strip(), text, row["reference"])
                self.assertNotIn(row["app_reader_context"].strip(), text, row["reference"])
                checked += 1
        self.assertEqual(checked, 528)

    def test_unicode_editions_are_kept_exactly(self):
        """English, German, Korean, Greek, French, Spanish and Portuguese
        text as published: the readers never normalise, and no verse holds a
        code point the scorer's Unicode version does not know."""
        for edition in self.expected["editions"]:
            for name, r in self.read[edition]["members"]:
                for v in r["verses"]:
                    self.assertFalse([c for c in v["text"] if __import__("unicodedata").category(c) == "Cn"],
                                     f"{edition} {name} {v['chapter']}:{v['label']}")
        # The four verses with a no-break space in their text keep it as published.
        found = {}
        for edition in ("engwebp", "porbr2018"):
            for _, r in self.read[edition]["members"]:
                for v in r["verses"]:
                    if "\u00a0" in v["text"]:
                        found[f"{edition} {r['book']} {v['chapter']}:{v['label']}"] = v["text"]
        self.assertEqual(sorted(found), ["engwebp ACT 20:35", "engwebp ACT 22:21", "engwebp JHN 5:11",
                                         "porbr2018 ZEC 7:13"])
        self.assertIn("walk.\u2019\u00a0\u201d", found["engwebp JHN 5:11"])


# ---------------------------------------------------------------- R1: the seal and the scorer

UNCLOSED_XREF = zipped([("63-2JN.usfm", "\\id 2JN SYNTHETIC\n\\c 1\n\\p\n\\v 1 a.\\x - \\xo 1:1 \\xt Inv\n"
                                         "\\v 2 b.\n\\v 3 c.\\f + \\ft SYNTHETIC\\f*\n")])
UNCLOSED_VA = zipped([("37-HAG.usfm", "\\id HAG SYNTHETIC\n\\c 1\n\\p\n\\v 1 \\va 2\\va a.\n\\v 2 b.\n")])
MILESTONE_BAD = zipped([("57-PHM.usfm", "\\id PHM SYNTHETIC\n\\c 1\n\\p\n\\v 1 \\qt-s |who=\"x\" a.\\qt-e\\*\n")])
NOTE_QUOTE = zipped([("08-RUT.usfm", usfm("RUT", [(1, range(1, 4))]).replace(
    "\\v 2 Invented verse 1:2 of RUT.", "\\v 2 “Invented verse 1:2 of RUT.\\f + \\ft SYNTHETIC\\f*”"))])


class SealRefusesMalformedReferences(E2Fixture):
    def refused(self, reference, books):
        s = E2Set(self.tmp, reference, lambda inv: e2_report(inv), books=books)
        r = subprocess.run(["python3", CHECK, s.seal, s.docs], capture_output=True, text=True)
        self.assertEqual(r.returncode, 2, r.stdout[-2000:])
        return r.stderr

    def test_an_unclosed_cross_reference(self):
        self.assertIn("note_unclosed", self.refused(UNCLOSED_XREF, "2JN"))

    def test_an_unclosed_alternate_number(self):
        self.assertIn("dropped_unclosed", self.refused(UNCLOSED_VA, "HAG"))

    def test_an_unterminated_milestone(self):
        self.assertIn("milestone_unterminated", self.refused(MILESTONE_BAD, "PHM"))

    def test_a_note_before_a_quotation_mark_is_sound_and_read_exactly(self):
        inv = reference_inventory.read(self.write(NOTE_QUOTE), {"RUT"})
        self.assertEqual(inv["problems"], [])
        r = usfm_text.read_text(zipfile.ZipFile(self.write(NOTE_QUOTE)).read("08-RUT.usfm").decode())
        self.assertEqual(r["verses"][1]["text"], "“Invented verse 1:2 of RUT.”")

    def write(self, data):
        path = os.path.join(self.tmp, "ref.zip")
        open(path, "wb").write(data)
        return path


class ScorerComparesBothReadings(E2Fixture):
    def test_a_harness_reading_another_text_is_not_trusted(self):
        """The fake harness reads RUT 1:2 as the app's reader did ("RUT. ”"):
        its own report is consistent, but its text digest is not the check's."""
        def build(inv):
            j = e2_report(inv)
            hv = j["reference_inventory"]["verses"]
            k = next(i for i, v in enumerate(hv) if v[3] == "2")
            hv[k] = hv[k][:6] + [usfm_text.text_sha256("“Invented verse 1:2 of RUT. ”")]
            j["reference_inventory"]["text_digest"] = reference_inventory.text_digest(hv)
            return j
        _, _, score = self.e2(NOTE_QUOTE, build, books="RUT")
        a = score["assessment"]
        self.assertEqual(a["verdict"], "not a protocol run")
        why = "; ".join(a["integrity"]["S1"])
        self.assertIn("verse text digests differ", why)
        self.assertIn("1 verse(s) read differently", why)
        self.assertIn("RUT 1:2", why)

    def test_agreeing_readings_pass(self):
        _, _, score = self.e2(NOTE_QUOTE, lambda inv: e2_report(inv), books="RUT")
        self.assertEqual(score["assessment"]["verdict"], "pass", score["assessment"]["integrity"])

    def test_altered_check_records_are_not_used(self):
        s = E2Set(self.tmp, NOTE_QUOTE, lambda inv: e2_report(inv), books="RUT")
        _, out = s.run(check=True)
        path = os.path.join(out, "check-verses", "S1.verses.json")
        self.assertTrue(os.path.exists(path), os.listdir(out))
        verses = json.load(open(path))
        verses[0][6] = "0" * 64
        json.dump(verses, open(path, "w"))
        r, _ = s.score(out)  # refused: the committed OUTPUTS.sha256 covers the check's records
        self.assertEqual(r.returncode, 2)
        self.assertIn("check-verses/S1.verses.json changed after the run", r.stderr)


BRIDGES = zipped([("31-OBA.usfm", "\\id OBA SYNTHETIC\n\\c 1\n\\p\n\\v 1 a.\n\\v 2-3 b.\n\\v 4-5 c.\n"
                                   "\\v 5 d.\n\\v 6 e.\n")])


class Labels(E2Fixture):
    """Missing, duplicated and unusual labels (E3 adds bridges)."""

    def test_a_bridge_beside_a_verse_it_covers_is_ambiguous(self):
        path = os.path.join(self.tmp, "b.zip")
        open(path, "wb").write(BRIDGES)
        inv = reference_inventory.read(path, {"OBA"})
        self.assertEqual(inv["unsupported_labels"]["verse_duplicate"], ["OBA 1:4-5", "OBA 1:5"])
        self.assertEqual(inv["scorable_ranges"], ["OBA 1:1-2", "OBA 1:6"])
        self.assertEqual(inv["bridge_member_ranges"], ["OBA 1:3"])
        self.assertEqual(inv["unscorable_keys"], {"OBA 1:4": "verse_duplicate", "OBA 1:5": "verse_duplicate"})

    def test_a_verified_number_inside_a_bridge_cannot_be_validated(self):
        _, _, score = self.e2(BRIDGES, lambda inv: e2_report(inv, unvalidatable=[("OBA 1:3", "inside_bridge"),
                                                                                ("OBA 1:5", "verse_duplicate")]),
                              books="OBA")
        a = score["assessment"]
        self.assertEqual(a["verdict"], "fail", a["integrity"])
        self.assertTrue(a["run_validity"]["reports_trusted"], a["integrity"])
        self.assertFalse(a["criteria"]["2_no_verified_reference_outside_or_unvalidatable"])
        self.assertEqual(a["scored"]["verified_unvalidatable"], 2)

    def test_a_harness_that_hides_a_bridge_number_is_not_trusted(self):
        """It verifies OBA 1:3 and does not report it: the scorer's own
        lookup in the check's keys finds it."""
        def build(inv):
            j = e2_report(inv)
            j["verified_references"].append("OBA 1:3")
            j["verified_empty_in_both"] = 1  # keeps its own accounting whole
            return j
        _, _, score = self.e2(BRIDGES, build, books="OBA")
        a = score["assessment"]
        self.assertEqual(a["verdict"], "not a protocol run")
        self.assertIn("verified references absent from the edition", "; ".join(a["integrity"]["S1"]))

    def test_a_gap_in_the_numbering_is_absent_not_lost(self):
        gap = zipped([("31-OBA.usfm", "\\id OBA SYNTHETIC\n\\c 1\n\\p\n\\v 1 a.\n\\v 2 b.\n\\v 4 d.\n")])
        _, _, score = self.e2(gap, lambda inv: e2_report(inv, absent=[("OBA 1:3", "verse_not_in_chapter")]),
                              books="OBA")
        a = score["assessment"]
        self.assertEqual(a["verdict"], "fail")
        self.assertTrue(a["run_validity"]["reports_trusted"], a["integrity"])
        self.assertEqual(a["scored"]["verified_absent"], 1)


# ---------------------------------------------------------------- R2: evidence

TOBIT = zipped([("68-TOB.usfm", usfm("TOB", [(1, range(1, 11))]))])
EMPTY_ZIP = zipped([])
PROSE_ONLY = zipped([("copr.htm", "<p>SYNTHETIC licence page</p>")])
NUMBERED_EMPTY = zipped([("31-OBA.usfm", "\\id OBA SYNTHETIC\n\\c 1\n\\p\n\\v 1\n\\v 2\n")])


class SealRefusesReferencesWithoutEvidence(E2Fixture):
    def check(self, reference, books="all", cls="scored"):
        s = E2Set(self.tmp, reference, lambda inv: e2_report(inv), books=books)
        if cls != "scored":
            manifest = os.path.join(s.seal, "manifest.tsv")
            text = open(manifest).read().replace("\tscored\n", f"\t{cls}\n")
            open(manifest, "w").write(text)
            comp = os.path.join(s.seal, "composition.tsv")
            text = open(comp).read().replace("scored\t1", "scored\t0").replace(f"{cls}\t0", f"{cls}\t1")
            open(comp, "w").write(text)
            s.reseal()
        r = subprocess.run(["python3", CHECK, s.seal, s.docs], capture_output=True, text=True)
        return r, json.loads(r.stdout)

    def test_an_empty_package(self):
        r, rep = self.check(EMPTY_ZIP)
        self.assertEqual(r.returncode, 2)
        self.assertEqual(rep["reference_evidence"]["S1"]["evidence_class"], "no_verses")
        self.assertIn("holds no printed scorable verse (no_verses", r.stderr)

    def test_a_package_with_prose_only(self):
        r, rep = self.check(PROSE_ONLY)
        self.assertEqual(r.returncode, 2)
        self.assertEqual(rep["reference_evidence"]["S1"]["evidence_class"], "no_verses")

    def test_a_deuterocanonical_reference_only(self):
        r, rep = self.check(TOBIT, books="TOB")
        self.assertEqual(r.returncode, 2)
        self.assertEqual(rep["reference_evidence"]["S1"]["evidence_class"], "unsupported_only")
        self.assertIn("declared book TOB: outside the 66-book canon", r.stderr)

    def test_verses_numbered_but_empty_only(self):
        r, rep = self.check(NUMBERED_EMPTY, books="OBA")
        self.assertEqual(r.returncode, 2)
        self.assertEqual(rep["reference_evidence"]["S1"]["evidence_class"], "numbered_but_empty")

    def test_a_declared_book_the_reference_does_not_print(self):
        r, _ = self.check(OBA, books="OBA,RUT")
        self.assertEqual(r.returncode, 2)
        self.assertIn("declared book RUT: no printed verse in the reference", r.stderr)

    def test_an_exploratory_document_may_hold_unsupported_content_only(self):
        r, rep = self.check(TOBIT, books="TOB", cls="exploratory")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(rep["reference_evidence"]["S1"]["evidence_class"], "unsupported_only")
        self.assertIn("its reference holds no printed scorable verse (unsupported_only)", r.stderr)


def zero_set(tmp, **kw):
    """Two scored documents with sound references that verify nothing."""
    return SetFixture(tmp, [
        {"key": "Z1", "class": "scored", "kind": "authentic", "behaviour": {"write": {".json": report(25, 0)}}},
        {"key": "Z2", "class": "scored", "format": "pdf",
         "behaviour": {"write": {".rejected.md": "# Z2 — not read\n\nThe app refused the document: badPdf\n"}}},
        {"key": "C1", "class": "control", "format": "docx", "behaviour": {"write": {".control.json": control([])}}},
    ], allowed={}, **kw)


class Evidence(Fixture):
    def scored(self, s):
        _, out = s.run(check=True)
        r, score = s.score(out)
        self.assertEqual(r.returncode, 0, r.stderr)
        return score["assessment"]

    def test_a_zero_coverage_run_is_inconclusive_never_pass(self):
        a = self.scored(zero_set(self.tmp))
        self.assertEqual(a["verdict"], "inconclusive")
        self.assertEqual(a["safety_verdict"], "pass")
        self.assertEqual(a["evidence_verdict"], "insufficient")
        self.assertTrue(all(a["criteria"].values()))
        self.assertEqual(a["evidence_floor"]["measures"]["minimum_verified_faithful"],
                         {"floor": 1, "observed": 0, "met": False})
        c = a["counts"]
        self.assertEqual((c["verified"], c["printed_scorable"], c["missing"], c["refused_whole"]), (0, 25, 25, 1))
        self.assertEqual(c["documents_verifying_nothing"], ["Z1"])
        self.assertTrue(a["verdict_line"].startswith("INCONCLUSIVE (safety pass; evidence insufficient): verified 0"))
        for words in ("of 25 printed scorable", "printed unsupported", "held back", "refused chapters",
                      "25 missing", "1 refused whole"):
            self.assertIn(words, a["verdict_line"])
        self.assertIn("below the protocol's evidence floor", a["accuracy_claim"])

    def test_wrong_text_fails_whatever_the_evidence(self):
        s = SetFixture(self.tmp, [{"key": "W1", "class": "scored", "kind": "authentic",
                                   "behaviour": {"write": {".json": report(10, 1, wrong=["MAT 1:1"])}}}], allowed={})
        a = self.scored(s)
        self.assertEqual((a["verdict"], a["safety_verdict"], a["evidence_verdict"]), ("fail", "fail", "insufficient"))

    def test_one_faithful_verse_meets_the_default_floor(self):
        s = SetFixture(self.tmp, [{"key": "P1", "class": "scored", "kind": "authentic",
                                   "behaviour": {"write": {".json": report(10, 1)}}}], allowed={})
        a = self.scored(s)
        self.assertEqual((a["verdict"], a["evidence_verdict"]), ("pass", "sufficient"))

    def test_a_protocol_without_an_evidence_floor_gives_no_pass(self):
        for floor in (None, {}, {"minimum_verified_faithful": 0}, {"minimum_verified_faithful": "1"},
                      {"minimum_verified_faithful": True}):
            with self.subTest(floor=floor):
                tmp = os.path.join(self.tmp, f"f{abs(hash(json.dumps(floor))) % 10**8}")
                s = SetFixture(tmp, [{"key": "P1", "class": "scored", "kind": "authentic",
                                      "behaviour": {"write": {".json": report(10, 9)}}}], allowed={},
                               protocol_changes={"evidence_floor": floor})
                a = self.scored(s)
                self.assertEqual(a["verdict"], "inconclusive")
                self.assertIn("no readable evidence floor", a["evidence_floor"]["shortfall"][0])

    def test_the_owner_floor_slots(self):
        cases = [({"minimum_documents_with_verified_faithful": 2}, "inconclusive"),
                 ({"minimum_documents_with_verified_faithful": 1}, "pass"),
                 ({"minimum_verified_faithful_share_of_printed_scorable": 0.95}, "inconclusive"),
                 ({"minimum_verified_faithful_share_of_printed_scorable": 0.9}, "pass"),
                 ({"maximum_scored_documents_refused_whole": 0}, "inconclusive"),
                 ({"maximum_scored_documents_refused_whole": 1}, "pass"),
                 ({"maximum_scored_documents_refused_whole": "none"}, "inconclusive"),
                 ({"a_measure_nobody_defined": 3}, "inconclusive")]
        for n, (extra, want) in enumerate(cases):
            with self.subTest(extra=extra):
                s = SetFixture(os.path.join(self.tmp, str(n)), [
                    {"key": "P1", "class": "scored", "kind": "authentic", "behaviour": {"write": {".json": report(10, 9)}}},
                    {"key": "P2", "class": "scored", "format": "pdf",
                     "behaviour": {"write": {".rejected.md": "# P2 — not read\n\nThe app refused the document: x\n"}}},
                ], allowed={}, protocol_changes={"evidence_floor": dict({"minimum_verified_faithful": 1}, **extra)})
                self.assertEqual(self.scored(s)["verdict"], want)

    def test_counts_beside_every_result(self):
        s = zero_set(self.tmp)
        _, out = s.run(check=True)
        md = os.path.join(self.tmp, "score.md")
        r = subprocess.run(["python3", SCORE, out, s.seal, f"--run-record={s.record}", "--diagnostic",
                            f"--protocol={s.protocol}", f"--inventory={s.inventory}", f"--md={md}"],
                           capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        text = open(md).read()
        self.assertIn("Under them the verdict would be **inconclusive**: INCONCLUSIVE (safety pass; "
                      "evidence insufficient): verified 0", text)
        self.assertIn("| Document | Class | Kind | Execution | Exit | Printed | Unsupported | Verified | Faithful "
                      "| Mismatch | Absent | Unvalidatable | Held | Refused | Missing |", text)
        self.assertIn("| minimum_verified_faithful | 1 | 0 | **no** |", text)


# ---------------------------------------------------------------- T3: protocol integrity

class Options(Fixture):
    def test_unknown_and_repeated_options_are_refused(self):
        s = self.make()
        _, out = s.run(check=True)
        # A misspelt option, an unknown flag, and options given twice (the
        # last one: --run-record, given again below).
        for extra in (["--Protocol=x.json"], ["--protocl=x"], ["--diagnostics"], ["--json=a", "--json=b"],
                      ["--run-record=HEAD"]):
            with self.subTest(extra=extra):
                args = ["python3", SCORE, out, s.seal, *extra, f"--run-record={s.record}"]
                r = subprocess.run(args, capture_output=True, text=True)
                self.assertEqual(r.returncode, 2, extra)
                self.assertIn("unknown or repeated option", r.stderr)
        for tool, extra in ((CHECK, "--seal_commit=x"), (SEAL, "--Write"), (SEAL, "--protocol")):
            r = subprocess.run(["python3", tool, s.seal, s.docs, extra], capture_output=True, text=True)
            self.assertEqual(r.returncode, 2, (tool, extra))
            self.assertIn("unknown or repeated option", r.stderr)


class ToolIdentity(Fixture):
    def test_an_unapproved_tool_is_not_a_protocol_run(self):
        tools = approved_tools()
        tools["scripts/evaluation/score-set.py"] = "0" * 64
        s = self.make(protocol_changes={"approved_tools": tools})
        _, out = s.run(check=True)
        _, score = s.score(out)
        a = score["assessment"]
        self.assertEqual(a["verdict"], "not a protocol run")
        self.assertFalse(a["run_validity"]["tools_are_the_approved_ones"])
        self.assertFalse(a["tools"]["scripts/evaluation/score-set.py"]["match"])

    def test_a_runner_other_than_the_approved_one_is_not_a_protocol_run(self):
        tools = approved_tools()
        tools["scripts/evaluation/run-withheld-set.sh"] = "1" * 64
        s = self.make(protocol_changes={"approved_tools": tools})
        _, out = s.run(check=True)
        _, score = s.score(out)
        self.assertFalse(score["assessment"]["run_validity"]["run_used_the_approved_tools"])
        self.assertEqual(score["assessment"]["verdict"], "not a protocol run")

    def test_a_protocol_approving_no_tools(self):
        s = self.make(protocol_changes={"approved_tools": None})
        _, out = s.run(check=True)
        _, score = s.score(out)
        self.assertFalse(score["assessment"]["run_validity"]["tools_are_the_approved_ones"])

    def test_the_official_score_names_the_protocol_for_the_owner_to_check(self):
        s = self.make()
        _, out = s.run(check=True)
        r, score = s.score(out, official=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(score["assessment"]["verdict"], "not a protocol run")
        self.assertIn("check with sha256sum that docs/evaluation/protocol-e3.json is", score["owner_check"])


class ExecutionGate(Fixture):
    def test_the_runner_refuses_a_protocol_that_does_not_permit_a_run(self):
        s = self.make(protocol_changes={"withheld_execution_permitted": False})
        self.assertIn("does not permit a withheld run", self.refused(s))

    def test_the_committed_protocol_permits_no_run_yet(self):
        p = json.load(open(os.path.join(ROOT, "docs", "evaluation", "protocol-e3.json"), encoding="utf-8"))
        self.assertIs(p["withheld_execution_permitted"], False)
        self.assertIsNone(p["required_composition"])
        self.assertEqual(p["evidence_floor"]["minimum_verified_faithful"], 1)
        for slot in ("minimum_documents_with_verified_faithful", "minimum_verified_faithful_share_of_printed_scorable",
                     "maximum_scored_documents_refused_whole"):
            self.assertIsNone(p["evidence_floor"][slot])
        self.assertEqual(p["unicode"]["scorer_python"], __import__("unicodedata").unidata_version)

    def test_a_score_under_a_protocol_that_permits_no_run_is_not_a_protocol_run(self):
        s = self.make()
        _, out = s.run(check=True)
        p = json.load(open(s.protocol))
        p["withheld_execution_permitted"] = False
        json.dump(p, open(s.protocol, "w"))
        _, score = s.score(out)
        self.assertFalse(score["assessment"]["run_validity"]["protocol_permits_withheld_execution"])


class ReferenceSources(Fixture):
    def tampered(self, change):
        def tamper(fx):
            path = os.path.join(fx.seal, "reference-sources.tsv")
            text = open(path).read()
            open(path, "w").write(change(text))
        return self.make(tamper=tamper)

    def test_the_package_and_the_publisher_archive_are_recorded_apart(self):
        s = self.make()
        r = subprocess.run(["python3", CHECK, s.seal, s.docs], capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        rec = json.loads(r.stdout)["reference_sources"]["S1"]
        self.assertEqual(rec["kept"], 1)
        self.assertRegex(rec["publisher_archive_sha256"], "^[0-9a-f]{64}$")

    def test_a_member_not_recorded(self):
        s = self.tampered(lambda t: "\n".join(l for l in t.splitlines() if not l.startswith("S1\t")) + "\n")
        self.assertIn("S1: reference-sources.tsv has no row for its reference", self.refused(s))

    def test_a_member_that_is_not_the_recorded_one(self):
        s = self.tampered(lambda t: re.sub(r"(S1\t[^\n]*\t)[0-9a-f]{64}(\tkept)", lambda m: m.group(1) + "0" * 64
                                           + m.group(2), t))
        self.assertIn("is not the one reference-sources.tsv records", self.refused(s))

    def test_a_removed_member_still_in_the_package(self):
        s = self.tampered(lambda t: t + "S1\tS1.usfm.zip\tSYNTHETIC package\t" + "a" * 64
                          + "\t41-MATxx.usfm\t" + "b" * 64 + "\tremoved\n")
        self.assertIn("is listed as removed but is in the package", self.refused(s))

    def test_a_seal_without_the_record(self):
        s = self.make(tamper=lambda fx: os.remove(os.path.join(fx.seal, "reference-sources.tsv")))
        self.assertIn("SEAL.txt must list reference-sources.tsv", self.refused(s))


class UnicodeVersions(E2Fixture):
    def test_a_reference_beyond_the_scorers_unicode_is_refused(self):
        ref = zipped([("31-OBA.usfm", "\\id OBA SYNTHETIC\n\\c 1\n\\p\n\\v 1 a \U00016D67 b.\n")])
        s = E2Set(self.tmp, ref, lambda inv: e2_report(inv), books="OBA")
        r = subprocess.run(["python3", CHECK, s.seal, s.docs], capture_output=True, text=True)
        self.assertEqual(r.returncode, 2)
        self.assertIn("U+16D67 unassigned in Unicode", r.stderr)

    def test_a_record_beyond_the_scorers_unicode_is_distrusted(self):
        rec = [("OBA 1:1", "Invented verse 1:1 of OBA.", "Invented verse 1:1 of OBA \U00016D67.", "mismatch", "extra")]
        _, _, score = self.e2(OBA, lambda inv: e2_report(inv, rec))
        a = score["assessment"]
        self.assertEqual(a["verdict"], "not a protocol run")
        self.assertIn("unassigned in the scorer's Unicode", "; ".join(a["integrity"]["S1"]))

    def test_a_report_of_another_unicode_version_is_not_a_protocol_run(self):
        def build(inv):
            j = e2_report(inv)
            j["unicode_version"] = "16.0.0"
            return j
        _, _, score = self.e2(OBA, build)
        self.assertFalse(score["assessment"]["run_validity"]["unicode_versions_as_protocol"])


if __name__ == "__main__":
    unittest.main()
