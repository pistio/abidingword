"""Qualifies the withheld-set tools (Phase A1.5; evaluation rules E2) on
synthetic sets.

A fake harness stands in for the recogniser: for each document it writes
the report it is told to write, then exits, sleeps, aborts or is killed as
told. No Scripture, no corpus and no recogniser are needed, so every
refusal of the runner and every way a document's run can end are shown.
Every reference and report here is invented (SYNTHETIC).

E2 (candidate A1.5-E2): each set has a declared composition and a test
protocol; the scorer is run in --diagnostic mode with that protocol, and
`score()` checks that the official verdict is then always "not a protocol
run" before it hands the tests the diagnostic verdict.

E3 (candidate A1.5-E3): the fake reports are in the E3 format (`aw-eval-3`:
the reference inventory carries every verse's text digest, read here by the
check's own reader, so the fake harness agrees with the check by
construction); every seal holds reference-sources.tsv; the test protocol
approves the tools in use by SHA-256, states both Unicode versions, permits
the run and sets the default evidence floor.

    python3 -m unittest discover -s tests/evaluation -p 'test_*.py'
"""
import hashlib
import importlib.util
import io
import json
import os
import shutil
import subprocess
import tempfile
import unicodedata
import unittest
import warnings
import zipfile
from collections import Counter

warnings.simplefilter("ignore", ResourceWarning)

ROOT = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
# AW_EVAL_TOOLS: another copy of the tools (used once, to show which E2
# tests fail on the A1.5 tools); the default is this repository's.
TOOLS = os.environ.get("AW_EVAL_TOOLS", os.path.join(ROOT, "scripts", "evaluation"))
RUNNER = os.path.join(TOOLS, "run-withheld-set.sh")
CHECK = os.path.join(TOOLS, "check-set.py")
SEAL = os.path.join(TOOLS, "seal-set.py")
SCORE = os.path.join(TOOLS, "score-set.py")
_spec = importlib.util.spec_from_file_location(
    "reference_inventory", os.path.join(ROOT, "scripts", "evaluation", "reference_inventory.py"))
reference_inventory = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(reference_inventory)
PROTOCOL_LIMITS = {"timeout_seconds": 1800, "memory_kb": 4194304}
# The tools an official assessment rests on (score-set.py's TOOL_FILES).
TOOL_FILES = ["scripts/evaluation/score-set.py", "scripts/evaluation/check-set.py",
              "scripts/evaluation/seal-set.py", "scripts/evaluation/run-withheld-set.sh",
              "scripts/evaluation/fidelity.py", "scripts/evaluation/reference_inventory.py",
              "scripts/evaluation/usfm_text.py", "docs/evaluation/usfm-books.tsv"]
HARNESS_UNICODE = "17.0.0"
SOURCES = ["key", "reference", "publisher_archive", "publisher_archive_sha256", "member", "member_sha256", "role"]


def approved_tools():
    """The SHA-256 of every tool in this checkout: what a test protocol approves."""
    return {rel: hashlib.sha256(open(os.path.join(ROOT, rel), "rb").read()).hexdigest() for rel in TOOL_FILES}


def test_protocol(fake_sha, inventory, composition, **changes):
    """A test protocol: the fake harness, the protocol ceilings, the test
    inventory, a required composition, and (E3) the tools in use approved,
    both Unicode versions, the run permitted, the default evidence floor."""
    p = {"name": "test protocol", "harness_binary_sha256": fake_sha, "limits": PROTOCOL_LIMITS,
         "inventory": inventory, "inventory_sha256": hashlib.sha256(open(inventory, "rb").read()).hexdigest(),
         "required_composition": composition, "evidence_rules": "aw-evidence-1",
         "evidence_floor": {"minimum_verified_faithful": 1},
         "unicode": {"harness_normalisation": HARNESS_UNICODE, "scorer_python": unicodedata.unidata_version},
         "approved_tools": approved_tools(), "withheld_execution_permitted": True}
    p.update(changes)
    return p

FAKE = r'''#!/usr/bin/env python3
import json, os, signal, sys, time
key = os.environ["AW_EDITIONS"]
out = os.environ["AW_AUDIT_OUT"]
spec = json.load(open(os.path.join(os.environ["FAKE_BEHAVIOUR"], key + ".json")))
os.makedirs(out, exist_ok=True)
for suffix, content in spec.get("write", {}).items():
    with open(os.path.join(out, key + suffix), "w") as f:
        f.write(content if isinstance(content, str) else json.dumps(content))
if spec.get("stderr"):
    sys.stderr.write(spec["stderr"]); sys.stderr.flush()
if spec.get("sleep"):
    time.sleep(spec["sleep"])
if spec.get("abort"):
    os.abort()
if spec.get("kill"):
    os.kill(os.getpid(), signal.SIGKILL)
sys.exit(spec.get("exit", 0))
'''

HEADER = ["key", "files", "reference", "language", "script", "format", "layout", "role",
          "source and licence", "books", "sha256", "generator", "kind", "class"]
PROV = ["key", "publisher", "edition", "revision", "retrieved_from", "retrieved_utc", "licence",
        "document_sha256", "reference_source", "reference_method", "identity_checked_by"]


def sha(data):
    return hashlib.sha256(data).hexdigest()


# Invented texts for fidelity records: (edition, found).
CHANGED = ("The weaver sang at midday.", "The potter worked at dawn.")
TYPOGRAPHY = ("It's the elders' doing.", "Its the elders' doing.")
CANONICAL = ("\u00e9l que espera", "e\u0301l que espera")


def record(reference, status, edition_found, cls, kind):
    return {"reference": reference, "status": status, "unverified_reason": None if status == "verified" else "x",
            "class": cls, "kind": kind, "matched": None, "page": None,
            "edition": edition_found[0], "found": edition_found[1]}


def reference_usfm(keys, book="MAT"):
    """An invented reference holding exactly `keys` (book, chapter, verse)."""
    lines, chapter = [f"\\id {book} SYNTHETIC test reference, invented text"], None
    for b, c, v in keys:
        if c != chapter:
            lines.append(f"\\c {c}")
            chapter = c
        lines.append(f"\\v {v} Invented verse {c}:{v}.")
    return "\n".join(lines) + "\n"


def report(printed, verified, wrong=(), typography=(), canonical=(), absent=(), clean=0, caught=0,
           book="MAT"):
    """An E2 report of a document whose reference prints `printed` verses
    (MAT 1:1, 1:2, ... a hundred to a chapter) of which the first `verified`
    are verified. `wrong` are verified with changed words, `typography` with
    only an apostrophe dropped (wrong text under E2), `canonical` with only
    canonically equivalent text (faithful). `absent`: verified references the
    reference does not have."""
    keys = [(book, 1 + i // 100, 1 + i % 100) for i in range(printed)]
    label = lambda k: f"{k[0]} {k[1]}:{k[2]}"
    held = [label(k) for k in keys[verified:verified + clean + caught]]
    records = [record(r, "verified", CHANGED, "mismatch", "changed") for r in wrong]
    records += [record(r, "verified", TYPOGRAPHY, "mismatch", "punctuation") for r in typography]
    records += [record(r, "verified", CANONICAL, "canonical", None) for r in canonical]
    records += [record(r, "held", CHANGED, "mismatch", "changed") for r in held[clean:]]
    kinds = Counter(["changed"] * len(wrong) + ["punctuation"] * len(typography))
    totals = {"expected": printed, "verified": verified,
              "verified_exact": verified - len(wrong) - len(typography) - len(canonical),
              "verified_canonical": len(canonical), "verified_layout": 0,
              "verified_mismatch": len(wrong) + len(typography), "verified_mismatch_kinds": dict(kinds),
              "held": clean + caught, "held_faithful": clean, "held_mismatch": caught, "refused": 0,
              "not_found": printed - verified - clean - caught}
    return {"evaluation_schema": "aw-eval-3", "fidelity_rules": "aw-fidelity-2",
            "inventory_rules": "aw-reference-inventory-2", "reader_rules": "aw-usfm-text-1",
            "unicode_version": HARNESS_UNICODE, "totals": totals, "fidelity_records": records,
            "verified_references": [label(k) for k in keys[:verified]] + list(absent),
            "verified_absent": [{"reference": r, "reason": "verse_not_in_chapter"} for r in absent],
            "verified_unvalidatable": [], "verified_empty_in_both": 0,
            "book_selection": {"rule": "all", "books": None, "unknown_codes": []},
            "importer_comparison": {"agree": printed, "only_importer": 0, "only_inventory": 0,
                                    "text_differs": 0, "error": None},
            "accounting": {"holds": True}, "reference_inventory": "AUTO",
            "refused_chapters": [], "books_not_found": [], "read_peak_memory_mb": 40.0,
            "read_seconds": 0.1, "_reference_usfm": reference_usfm(keys, book)}


def harness_inventory(inv):
    """The harness's inventory report, as the E3 harness writes it, for a
    reference the Python inventory read: the same labels, statuses and
    per-verse text digests (a fake harness that agrees with the check)."""
    counts, printed, labels = inv["label_counts"], inv["printed_counts"], inv["unsupported_labels"]
    unsupported = {k: {"printed": printed[k], "empty": counts[k] - printed[k], "references": labels[k]}
                   for k in reference_inventory.UNSUPPORTED}
    return {"rules": reference_inventory.RULES, "reader_rules": reference_inventory.usfm_text.RULES,
            "members": inv["members"], "problems": inv["problems"],
            "label_digest": inv["label_digest"], "text_digest": inv["text_digest"], "labels": inv["labels"],
            "printed_scorable": inv["printed_scorable"], "absent_in_edition": inv["absent_in_edition"],
            "unsupported": unsupported, "printed_unsupported": inv["printed_unsupported"],
            "outside_declared_scope": {"printed": printed["outside_declared_scope"],
                                       "empty": counts["outside_declared_scope"] - printed["outside_declared_scope"],
                                       "references": inv["outside_declared_books"]},
            "scorable_by_book": {}, "scorable_ranges": inv["scorable_ranges"],
            "bridge_member_ranges": inv["bridge_member_ranges"], "before_chapter_outside": 0,
            "verses": [list(v) for v in inv["verses"]]}


def control(references, refused=None):
    c = {"evaluation_schema": "aw-eval-3", "control": True, "detected": len(references),
         "verified": len(references), "verified_references": list(references), "peak_memory_mb": 20.0,
         "seconds": 0.1}
    if refused:
        c.update(refused_document=refused, verified=0, verified_references=[])
    return c


def usfm_zip(text):
    buf = io.BytesIO()
    with zipfile.ZipFile(buf, "w") as z:
        z.writestr("41-MATxx.usfm", text)
    return buf.getvalue()


class SetFixture:
    """A repository with a committed seal, a documents folder, a fake
    harness and an inventory of 'disclosed' hashes."""

    def __init__(self, tmp, docs, allowed=None, disclosed=(), disclosed_keys=("ZZ-disclosed",),
                 tamper=None, composition=None, required_composition="same", protocol_changes=None):
        self.tmp = tmp
        self.repo = os.path.join(tmp, "repo")
        self.seal = os.path.join(self.repo, "eval", "set-x", "seal")
        self.docs = os.path.join(tmp, "docs")
        self.behaviour = os.path.join(tmp, "behaviour")
        for d in (self.seal, self.docs, self.behaviour, os.path.join(self.repo, "docs", "corpus")):
            os.makedirs(d)
        self.git("init", "-q")
        self.git("config", "user.email", "t@example.invalid")
        self.git("config", "user.name", "t")
        with open(os.path.join(self.repo, "docs", "corpus", "editions.tsv"), "w") as f:
            f.write("key\tfiles\n" + "".join(f"{k}\tx\n" for k in disclosed_keys))
        with open(os.path.join(self.repo, "README"), "w") as f:
            f.write("repo\n")
        self.git("add", "-A")
        self.git("commit", "-qm", "base")
        rows, prov, sums, sources = [], [], {}, []
        for d in docs:
            key = d["key"]
            name = f"{key}.{d.get('format', 'epub')}"
            content = d.get("content", f"document {key}".encode())
            open(os.path.join(self.docs, name), "wb").write(content)
            sums[name] = sha(content)
            ref = "-"
            behaviour = json.loads(json.dumps(d.get("behaviour", {})))
            fake = behaviour.get("write", {}).get(".json")
            usfm = fake.pop("_reference_usfm", None) if isinstance(fake, dict) else None
            if d.get("reference", d["class"] in ("scored", "exploratory")):
                ref = f"{key}.usfm.zip"
                data = d.get("reference_bytes") or usfm_zip(usfm or f"\\id MAT\n\\c 1\n\\v 1 {key}\n")
                open(os.path.join(self.docs, ref), "wb").write(data)
                sums[ref] = sha(data)
                with zipfile.ZipFile(io.BytesIO(data)) as z:
                    for info in z.infolist():
                        sources.append([key, ref, "SYNTHETIC package", sha(data), info.filename,
                                        sha(z.read(info)), "kept"])
                if isinstance(fake, dict) and fake.get("reference_inventory") == "AUTO":
                    declared = None if d.get("books", "all") == "all" else set(d["books"].split(","))
                    fake["reference_inventory"] = harness_inventory(
                        reference_inventory.read(os.path.join(self.docs, ref), declared))
            rows.append([key, name, ref, d.get("language", "en"), "Latin", d.get("format", "epub"), "-",
                         "set-x", "test", d.get("books", "all"), sums[name], "-", d.get("kind", "generated"),
                         d["class"]])
            prov.append([key, "pub", "ed", "rev", "https://example.invalid/x", "2026-11-19T00:00:00Z",
                         "public domain", sums[name], "src" if ref != "-" else "-",
                         "method" if ref != "-" else "-", "reviewer"])
            json.dump(behaviour, open(os.path.join(self.behaviour, key + ".json"), "w"))
        self.write("manifest.tsv", "\t".join(HEADER) + "\n" + "".join("\t".join(r) + "\n" for r in rows))
        self.write("checksums.sha256", "".join(f"{h}  {n}\n" for n, h in sorted(sums.items())))
        self.write("provenance.tsv", "\t".join(PROV) + "\n" + "".join("\t".join(r) + "\n" for r in prov))
        self.write("review-ledger.md", "signed\n")
        self.write("access-records.md", "records\n")
        counts = {c: sum(r[13] == c for r in rows) for c in ("scored", "control", "mixed", "exploratory")}
        counts.update({k: sum(r[12] == k for r in rows) for k in ("authentic", "generated")})
        self.declared = dict(counts, **(composition or {}))
        self.write("composition.tsv", "measure\tdocuments\n" + "".join(f"{k}\t{v}\n" for k, v in self.declared.items()))
        self.write("reference-sources.tsv", "\t".join(SOURCES) + "\n" + "".join("\t".join(r) + "\n" for r in sources))
        for key, lines in (allowed or {}).items():
            os.makedirs(os.path.join(self.seal, "allowed"), exist_ok=True)
            self.write(f"allowed/{key}.txt", "\n".join(lines) + "\n")
        if tamper:
            tamper(self)
        self.reseal()
        self.git("add", "-A")
        self.git("commit", "-qm", "seal")
        self.commit = self.git("rev-parse", "HEAD").strip()
        self.inventory = os.path.join(tmp, "inventory.sha256")
        open(self.inventory, "w").write("".join(f"{h}\n" for h in [sha(b"something else")] + list(disclosed)))
        self.fake = os.path.join(tmp, "fake-harness")
        open(self.fake, "w").write(FAKE)
        os.chmod(self.fake, 0o755)
        self.fake_sha = sha(open(self.fake, "rb").read())
        # The test protocol: the fake harness, the protocol ceilings, the test
        # inventory, and (unless told otherwise) the declared composition.
        self.protocol = os.path.join(tmp, "protocol.json")
        json.dump(test_protocol(self.fake_sha, self.inventory,
                                self.declared if required_composition == "same" else required_composition,
                                **(protocol_changes or {})),
                  open(self.protocol, "w"))

    def git(self, *args):
        return subprocess.run(["git", "-C", self.repo, *args], check=True, capture_output=True, text=True).stdout

    def write(self, rel, text):
        p = os.path.join(self.seal, rel)
        os.makedirs(os.path.dirname(p), exist_ok=True)
        open(p, "w").write(text)

    def reseal(self):
        lines = ["sealed 2026-11-19T00:00:00Z"]
        for root, _, files in os.walk(self.seal):
            for f in sorted(files):
                rel = os.path.relpath(os.path.join(root, f), self.seal)
                if rel != "SEAL.txt":
                    lines.append(f"{sha(open(os.path.join(root, f), 'rb').read())}  {rel}")
        self.write("SEAL.txt", "\n".join(lines) + "\n")

    def run(self, out=None, binsha=None, timeout=None, check=False):
        out = out or os.path.join(self.tmp, "out")
        env = dict(os.environ, FAKE_BEHAVIOUR=self.behaviour, AW_DISCLOSED_INVENTORY=self.inventory,
                   AW_PROTOCOL=self.protocol)
        if timeout:
            env["AW_RUN_TIMEOUT_SECONDS"] = str(timeout)
        r = subprocess.run(["bash", RUNNER, self.seal, self.commit, self.docs, self.fake,
                            binsha or self.fake_sha, out], capture_output=True, text=True, env=env)
        if check:
            self.assertOk(r)
        if r.returncode == 0:
            self.record = self.git("rev-parse", "HEAD").strip()  # the runner's record commit
        return r, out

    @staticmethod
    def assertOk(r):
        if r.returncode != 0:
            raise AssertionError(r.stdout + r.stderr)

    def score(self, out, run_record=None, official=False):
        """Scores in --diagnostic mode with the test protocol (the fake
        harness is not the protocol's binary). The official verdict of a
        diagnostic score is always "not a protocol run"; that is checked
        here, and the tests then read the diagnostic verdict as `verdict`."""
        js = os.path.join(self.tmp, "score.json")
        if os.path.exists(js):
            os.remove(js)
        record = [] if run_record == "" else [f"--run-record={run_record or self.record}"]
        mode = [] if official else ["--diagnostic", f"--protocol={self.protocol}", f"--inventory={self.inventory}"]
        r = subprocess.run(["python3", SCORE, out, self.seal, f"--json={js}", *record, *mode],
                           capture_output=True, text=True)
        score = json.load(open(js)) if r.returncode == 0 else None
        if score is not None and not official:
            a = score["assessment"]
            assert score["mode"] == "diagnostic" and a["verdict"] == "not a protocol run", score["mode"]
            a["official_verdict"], a["verdict"] = a["verdict"], a["diagnostic_verdict"]
            a["official_verdict_line"], a["verdict_line"] = a["verdict_line"], a["diagnostic_verdict_line"]
        return r, score

    def rehash_outputs(self, out):
        """What someone would do to make altered outputs look consistent."""
        lines = []
        for root, _, files in os.walk(out):
            for f in sorted(files):
                if f != "OUTPUTS.sha256":
                    p = os.path.join(root, f)
                    lines.append(f"{sha(open(p, 'rb').read())}  ./{os.path.relpath(p, out)}")
        open(os.path.join(out, "OUTPUTS.sha256"), "w").write("\n".join(sorted(lines)) + "\n")


GOOD = [
    {"key": "S1", "class": "scored", "kind": "authentic", "format": "epub",
     "behaviour": {"write": {".json": report(100, 90, canonical=["MAT 1:3"], clean=8, caught=2)}}},
    {"key": "S2", "class": "scored", "format": "pdf",
     "behaviour": {"write": {".rejected.md": "# S2 — not read\n\nThe app refused the document: badPdf\n"}}},
    {"key": "C1", "class": "control", "format": "docx", "behaviour": {"write": {".control.json": control([])}}},
    {"key": "C2", "class": "control", "format": "pdf",
     "behaviour": {"write": {".control.json": control([], refused="not a PDF it can read")}}},
    {"key": "M1", "class": "mixed", "format": "epub",
     "behaviour": {"write": {".control.json": control(["JHN 3:16", "PSA 23:4"])}}},
    {"key": "E1", "class": "exploratory", "format": "epub",
     "behaviour": {"write": {".json": report(100, 95, wrong=["MAT 1:%d" % i for i in range(1, 41)])}}},
]
ALLOWED = {"M1": ["JHN 3:16", "PSA 23"]}


class Fixture(unittest.TestCase):
    def setUp(self):
        warnings.simplefilter("ignore", ResourceWarning)
        self.tmp = tempfile.mkdtemp(prefix="aw-eval-")

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def make(self, docs=GOOD, allowed=ALLOWED, **kw):
        return SetFixture(self.tmp, docs, allowed=allowed, **kw)

    def refused(self, s, **kw):
        r, out = s.run(**kw)
        self.assertEqual(r.returncode, 2, r.stdout + r.stderr)
        self.assertFalse(os.path.exists(os.path.join(s.docs, ".run-started")), "set marked as run")
        self.assertFalse(os.path.exists(out) and os.listdir(out) and not kw.get("out"), "outputs written")
        return r.stderr


class RunnerRefuses(Fixture):
    """Every refusal happens before any document is read."""

    def test_an_existing_output_folder(self):
        s = self.make()
        os.makedirs(os.path.join(self.tmp, "out"))
        self.assertIn("exists", self.refused(s, out=os.path.join(self.tmp, "out")))

    def test_another_binary(self):
        # E3: an expected SHA-256 other than the protocol's own pin is refused first.
        self.assertIn("is not the protocol's harness binary", self.refused(self.make(), binsha="0" * 64))

    def test_another_binary_file_under_the_pinned_hash(self):
        s = self.make()
        other = os.path.join(self.tmp, "other-harness")
        open(other, "w").write(FAKE + "\n# another build\n")
        os.chmod(other, 0o755)
        s.fake = other  # the expected SHA-256 stays the protocol's pin
        self.assertIn("is not the frozen", self.refused(s))

    def test_a_seal_changed_after_its_commit(self):
        s = self.make()
        with open(os.path.join(s.seal, "manifest.tsv"), "a") as f:
            f.write("# changed\n")
        self.assertIn("differs from its sealed SHA-256", self.refused(s))

    def test_a_seal_changed_and_resealed_after_its_commit(self):
        s = self.make()
        with open(os.path.join(s.seal, "manifest.tsv"), "a") as f:
            f.write("# changed\n")
        s.reseal()
        err = self.refused(s)
        self.assertIn("differs from the seal commit", err)
        self.assertIn("uncommitted changes", err)

    def test_a_seal_changed_in_a_later_commit(self):
        s = self.make()
        with open(os.path.join(s.seal, "manifest.tsv"), "a") as f:
            f.write("# changed later\n")
        s.reseal()
        s.git("add", "-A")
        s.git("commit", "-qm", "later change")
        self.assertIn("the seal folder changed after the seal commit", self.refused(s))

    def test_a_seal_commit_that_also_changes_something_else(self):
        s = self.make()
        s.git("reset", "-q", "--soft", "HEAD~1")
        with open(os.path.join(s.repo, "README"), "a") as f:
            f.write("more\n")
        s.git("add", "-A")
        s.git("commit", "-qm", "seal and more")
        s.commit = s.git("rev-parse", "HEAD").strip()
        self.assertIn("must change only", self.refused(s))

    def test_a_document_that_differs_from_its_checksum(self):
        s = self.make()
        with open(os.path.join(s.docs, "S1.epub"), "ab") as f:
            f.write(b"x")
        self.assertIn("differs from its checksum", self.refused(s))

    def test_an_unlisted_file_among_the_documents(self):
        s = self.make()
        open(os.path.join(s.docs, "extra.pdf"), "w").write("x")
        self.assertIn("does not list", self.refused(s))

    def test_a_document_byte_identical_to_a_disclosed_one(self):
        s = self.make(disclosed=[sha(b"document S1")])
        self.assertIn("byte-identical to a disclosed document", self.refused(s))

    def test_a_reference_holding_a_disclosed_usfm_book(self):
        # S1's reference (E2: generated from its report, 100 invented verses).
        book = reference_usfm([("MAT", 1 + i // 100, 1 + i % 100) for i in range(100)]).encode()
        s = self.make(disclosed=[sha(book)])
        self.assertIn("byte-identical to a disclosed USFM book", self.refused(s))

    def test_a_key_a_disclosed_edition_uses(self):
        s = self.make(disclosed_keys=("S1",))
        self.assertIn("keys already used by disclosed editions: S1", self.refused(s))

    def test_a_set_with_a_run_record_in_the_repository(self):
        s = self.make()
        run = os.path.join(s.repo, "eval", "set-x", "run")
        os.makedirs(run)
        open(os.path.join(run, "RUN.txt"), "w").write("earlier run\n")
        s.git("add", "-A")
        s.git("commit", "-qm", "run record")
        self.assertIn("a run record already exists", self.refused(s))

    def test_a_second_run_into_another_folder(self):
        s = self.make()
        s.run(check=True)
        r, _ = s.run(out=os.path.join(self.tmp, "out-again"))
        self.assertEqual(r.returncode, 2)
        self.assertIn("already been run", r.stderr)
        self.assertFalse(os.path.exists(os.path.join(self.tmp, "out-again")))

    def test_classes_that_contradict_their_references(self):
        docs = [{"key": "A", "class": "scored", "reference": False},
                {"key": "B", "class": "control", "reference": True},
                {"key": "C", "class": "mixed"},
                {"key": "D", "class": "guess"}]
        err = self.refused(self.make(docs=docs, allowed={}))
        self.assertIn("A: a scored document needs its exact matching reference", err)
        self.assertIn("B: a control document has no reference text", err)
        self.assertIn("C: a mixed document needs allowed/C.txt", err)
        self.assertIn("D: class 'guess'", err)

    def test_an_allowed_list_it_cannot_read(self):
        err = self.refused(self.make(allowed={"M1": ["John three sixteen"]}))
        self.assertIn("is not 'BOOK C'", err)

    def test_incomplete_provenance(self):
        def blank_licence(s):
            p = os.path.join(s.seal, "provenance.tsv")
            lines = open(p).read().splitlines()
            cells = lines[1].split("\t")
            cells[6] = ""
            lines[1] = "\t".join(cells)
            open(p, "w").write("\n".join(lines) + "\n")
        self.assertIn("provenance 'licence' is empty", self.refused(self.make(tamper=blank_licence)))


class Execution(Fixture):
    """How each document's run ended, and what that does to the score."""

    def test_a_clean_run_passes_and_counts_only_scored_documents(self):
        s = self.make()
        r, out = s.run(check=True)
        r, score = s.score(out)
        self.assertEqual(r.returncode, 0, r.stderr)
        a = score["assessment"]
        self.assertEqual(a["verdict"], "pass")
        self.assertEqual(a["execution"], {"completed": 4, "refused": 2})
        # Only S1 counts: E1's 40 differing verses are not judged.
        self.assertEqual(a["scored"]["documents"], 1)
        self.assertEqual(a["scored"]["verified"], 90)
        self.assertEqual(a["scored"]["verified_mismatch"], 0)
        self.assertEqual(a["scored"]["verified_canonical"], 1)  # E2: faithful, listed apart from exact
        self.assertIn("Exploratory documents are not part of this figure", a["accuracy_claim"])
        self.assertIn("not recognition coverage: 1 scored document(s) refused whole (S2)", a["accuracy_claim"])
        self.assertEqual(a["scored"]["refused_whole"], ["S2"])
        self.assertEqual(a["scored"]["documents_in_manifest"], 2)
        self.assertEqual(score["groups"]["format"], {"epub": {
            "documents": 1, "printed": 100, "printed_unsupported": 0, "verified": 90, "verified_exact": 89,
            "verified_canonical": 1, "verified_layout": 0, "verified_faithful": 90, "verified_mismatch": 0,
            "verified_mismatch_material": 0, "verified_absent": 0, "verified_unvalidatable": 0,
            "held": 10, "held_faithful": 8, "held_mismatch": 2, "refused": 0, "not_found": 0, "refused_chapters": 0,
            "verified_mismatch_kinds": {}, "unsupported": {}}})
        e1 = next(d for d in score["documents"] if d["key"] == "E1")
        self.assertEqual(e1["observed"]["verified_mismatch"], 40)  # raw observation kept
        m1 = next(d for d in score["documents"] if d["key"] == "M1")
        self.assertEqual(m1["outside_allowed"], [])
        self.assertEqual(sorted(m1["allowed_verified"]), ["JHN 3:16", "PSA 23:4"])
        # One authentic scored document is below the declared minimum.
        self.assertIsNotNone(a["qualification_limit"])

    def test_failures_are_never_completions(self):
        docs = [
            {"key": "W1", "class": "scored",
             "behaviour": {"write": {".json": report(10, 10, wrong=["MAT 1:2"], absent=["MAT 1:99"])}}},
            {"key": "X1", "class": "control", "behaviour": {"write": {".control.json": control(["GEN 1:1"])}}},
            {"key": "M2", "class": "mixed", "behaviour": {"write": {".control.json": control(["JHN 3:16", "JHN 3:17"])}}},
            {"key": "F1", "class": "scored",
             "behaviour": {"write": {".json": report(500, 500)}, "abort": True,
                           "stderr": "memory allocation of 1048576 bytes failed\n"}},
            {"key": "F2", "class": "scored", "behaviour": {"write": {".json": report(7, 7)}, "kill": True}},
            {"key": "F3", "class": "scored", "behaviour": {"exit": 101}},
            {"key": "F4", "class": "scored", "behaviour": {}},
            {"key": "F5", "class": "control", "behaviour": {"abort": True}},
        ]
        s = self.make(docs=docs, allowed={"M2": ["JHN 3:16"]})
        r, out = s.run(check=True)
        r, score = s.score(out)
        self.assertEqual(r.returncode, 0, r.stderr)
        a = score["assessment"]
        self.assertEqual(a["verdict"], "fail")
        self.assertEqual(a["criteria"], {
            "1_no_verified_text_mismatch": False,
            "2_no_verified_reference_outside_or_unvalidatable": False,
            "3_no_verified_reference_in_controls_or_outside_allowed": False,
            "4_every_document_completed_or_refused_cleanly": False})
        by = {d["key"]: d for d in score["documents"]}
        self.assertEqual((by["F1"]["execution"], by["F1"]["cause"]), ("failure", "memory"))
        self.assertEqual((by["F2"]["execution"], by["F2"]["cause"]), ("failure", "killed"))
        self.assertEqual((by["F3"]["execution"], by["F3"]["cause"]), ("failure", "panicked"))
        self.assertEqual((by["F4"]["execution"], by["F4"]["cause"]), ("failure", "no output"))
        self.assertEqual((by["F5"]["execution"], by["F5"]["cause"]), ("failure", "aborted"))
        # The reports F1 and F2 wrote before dying are kept, never counted.
        self.assertEqual(by["F1"]["diagnostic"]["verified"], 500)
        self.assertEqual(by["F2"]["diagnostic"]["verified"], 7)
        self.assertNotIn("observed", by["F1"])
        self.assertEqual(a["scored"]["documents"], 1)
        self.assertEqual(a["scored"]["verified"], 10)
        self.assertEqual(a["mixed"]["verified_outside_allowed"], 1)
        self.assertEqual(a["control"]["verified"], 1)

    def test_a_timeout_is_a_failure_and_not_a_protocol_run(self):
        docs = [{"key": "T1", "class": "scored",
                 "behaviour": {"write": {".json": report(3, 3)}, "sleep": 30}},
                {"key": "S1", "class": "scored", "behaviour": {"write": {".json": report(3, 3)}}}]
        s = self.make(docs=docs, allowed={})
        r, out = s.run(timeout=2, check=True)
        r, score = s.score(out)
        by = {d["key"]: d for d in score["documents"]}
        self.assertEqual((by["T1"]["execution"], by["T1"]["cause"]), ("failure", "timeout"))
        self.assertEqual(by["T1"]["diagnostic"]["verified"], 3)
        a = score["assessment"]
        self.assertFalse(a["run_validity"]["protocol_limits"])
        self.assertEqual(a["verdict"], "not a protocol run")


class ScorerRefuses(Fixture):
    def test_an_output_changed_after_the_run(self):
        s = self.make()
        _, out = s.run(check=True)
        with open(os.path.join(out, "S1.json"), "a") as f:
            f.write(" ")
        r, _ = s.score(out)
        self.assertEqual(r.returncode, 2)
        self.assertIn("S1.json changed after the run", r.stderr)

    def test_an_output_added_after_the_run(self):
        s = self.make()
        _, out = s.run(check=True)
        open(os.path.join(out, "S9.json"), "w").write("{}")
        r, _ = s.score(out)
        self.assertEqual(r.returncode, 2)
        self.assertIn("S9.json was added after the run", r.stderr)

    def test_a_run_of_another_seal(self):
        s = self.make()
        _, out = s.run(check=True)
        with open(os.path.join(s.seal, "manifest.tsv"), "a") as f:
            f.write("# another\n")
        s.reseal()
        r, _ = s.score(out)
        self.assertEqual(r.returncode, 2)
        self.assertIn("manifest.tsv differs from the seal commit", r.stderr)

    def test_a_link_among_the_outputs(self):
        s = self.make()
        _, out = s.run(check=True)
        target = os.path.join(self.tmp, "elsewhere.json")
        os.rename(os.path.join(out, "S1.json"), target)
        os.symlink(target, os.path.join(out, "S1.json"))
        r, _ = s.score(out)
        self.assertEqual(r.returncode, 2)
        self.assertIn("'S1.json' is a symbolic link", r.stderr)


MIXED = [{"key": "M1", "class": "mixed", "format": "epub",
          "behaviour": {"write": {".control.json": control(["JHN 3:16", "JHN 3:17"])}}}]
WRONG = [{"key": "S1", "class": "scored", "kind": "authentic", "format": "epub",
          "behaviour": {"write": {".json": report(10, 10, wrong=["MAT 1:2"])}}}]
PANIC = [{"key": "S1", "class": "scored", "kind": "authentic", "format": "epub",
          "behaviour": {"write": {".json": report(10, 10)}, "exit": 101}}]


class ScoringAnchors(Fixture):
    """The independent review's three alterations (each turned a failing run
    into a pass), and the other ways an anchor can be missing or wrong. The
    runner commits the run record; the scorer must be given that commit."""

    def assertRefused(self, s, out, message, **kw):
        r, _ = s.score(out, **kw)
        self.assertEqual(r.returncode, 2, r.stdout + r.stderr)
        self.assertIn(message, r.stderr)

    def test_an_allowed_list_widened_after_the_run(self):
        s = self.make(docs=MIXED, allowed={"M1": ["JHN 3:16"]})
        _, out = s.run(check=True)
        r, score = s.score(out)
        self.assertEqual(score["assessment"]["verdict"], "fail")
        self.assertEqual(score["assessment"]["mixed"]["verified_outside_allowed"], 1)
        s.write("allowed/M1.txt", "JHN 3:16-17\n")
        self.assertRefused(s, out, "allowed/M1.txt differs from the seal commit")
        s.reseal()  # SEAL.txt recomputed to match: still not the sealed one
        self.assertRefused(s, out, "SEAL.txt differs from the seal commit")
        s.git("add", "-A")
        s.git("commit", "-qm", "widen the allowed list after the run")
        self.assertRefused(s, out, "the seal folder changed after the seal commit")

    def test_a_report_changed_and_the_outputs_rehashed(self):
        s = self.make(docs=WRONG, allowed={})
        _, out = s.run(check=True)
        r, score = s.score(out)
        self.assertEqual(score["assessment"]["verdict"], "fail")
        p = os.path.join(out, "S1.json")
        value = json.load(open(p))
        value["fidelity_records"] = []
        json.dump(value, open(p, "w"))
        s.rehash_outputs(out)
        self.assertRefused(s, out, "OUTPUTS.sha256 in the run folder differs from the committed run record")
        self.assertEqual(s.git("status", "--porcelain").strip(), "")  # the record itself untouched

    def test_an_exit_code_changed_and_the_outputs_rehashed(self):
        s = self.make(docs=PANIC, allowed={})
        _, out = s.run(check=True)
        r, score = s.score(out)
        self.assertEqual(score["documents"][0]["execution"], "failure")
        self.assertEqual(score["assessment"]["verdict"], "fail")
        p = os.path.join(out, "RUN.txt")
        text = open(p).read()
        self.assertIn("S1 exit 101 seconds ", text)
        open(p, "w").write(text.replace("S1 exit 101 seconds ", "S1 exit 0 seconds "))
        s.rehash_outputs(out)
        self.assertRefused(s, out, "RUN.txt in the run folder differs from the committed run record")

    def test_a_run_record_changed_in_a_later_commit(self):
        s = self.make(docs=PANIC, allowed={})
        _, out = s.run(check=True)
        record = s.record
        p = os.path.join(out, "RUN.txt")
        text = open(p).read().replace("S1 exit 101 seconds ", "S1 exit 0 seconds ")
        open(p, "w").write(text)
        s.rehash_outputs(out)
        for name in ("RUN.txt", "OUTPUTS.sha256"):
            shutil.copyfile(os.path.join(out, name), os.path.join(s.repo, "eval", "set-x", "run", name))
        s.git("add", "-A")
        s.git("commit", "-qm", "a second run record")
        self.assertRefused(s, out, "the run record was changed after it was committed", run_record=record)
        self.assertRefused(s, out, "the run record was changed after it was committed",
                           run_record=s.git("rev-parse", "HEAD").strip())

    def test_a_run_record_with_uncommitted_changes(self):
        s = self.make(docs=WRONG, allowed={})
        _, out = s.run(check=True)
        with open(os.path.join(s.repo, "eval", "set-x", "run", "CHECK.json"), "a") as f:
            f.write(" ")
        self.assertRefused(s, out, "the run record has uncommitted changes")

    def test_no_committed_run_record(self):
        s = self.make(docs=WRONG, allowed={})
        _, out = s.run(check=True)
        s.git("reset", "-q", "--hard", "HEAD~1")
        self.assertRefused(s, out, "no committed run record", run_record=s.commit)

    def test_another_commit_named_as_the_run_record(self):
        s = self.make(docs=WRONG, allowed={})
        _, out = s.run(check=True)
        self.assertRefused(s, out, "is not the commit that recorded this run", run_record=s.commit)

    def test_no_run_record_named(self):
        s = self.make(docs=WRONG, allowed={})
        _, out = s.run(check=True)
        self.assertRefused(s, out, "--run-record=<commit> is required", run_record="")

    def test_the_runner_commits_the_record_alone(self):
        s = self.make(docs=WRONG, allowed={})
        _, out = s.run(check=True)
        changed = s.git("diff-tree", "--no-commit-id", "--name-only", "-r", s.record).split()
        self.assertEqual(sorted(changed), ["eval/set-x/run/CHECK.json", "eval/set-x/run/OUTPUTS.sha256",
                                           "eval/set-x/run/RUN.txt"])
        for name in ("RUN.txt", "OUTPUTS.sha256", "CHECK.json"):
            self.assertEqual(open(os.path.join(out, name), "rb").read(),
                             open(os.path.join(s.repo, "eval", "set-x", "run", name), "rb").read())


class PathContainment(Fixture):
    """Every name must be a plain relative path inside its folder, and no
    link is accepted: refused before any document is read (the independent
    review's three escapes, and the same rule for the seal)."""

    def escape(self, mode):
        def alter_input(fixture):
            declared = os.path.join(fixture.docs, "C1.docx")
            outside = os.path.join(fixture.tmp, "outside")
            os.makedirs(outside, exist_ok=True)
            target = os.path.join(outside, "C1.docx")
            os.rename(declared, target)
            if mode == "link":
                os.symlink(target, declared)
            elif mode == "inner link":
                os.rename(target, os.path.join(fixture.docs, "real.docx"))
                os.symlink(os.path.join(fixture.docs, "real.docx"), declared)
            else:
                replacement = "../outside/C1.docx" if mode == "parent" else target
                for name in ("manifest.tsv", "checksums.sha256"):
                    p = os.path.join(fixture.seal, name)
                    text = open(p).read().replace("C1.docx", replacement)
                    open(p, "w").write(text)
        s = SetFixture(self.tmp, [{"key": "C1", "class": "control", "format": "docx",
                                   "content": b"Invented, non-Scripture path probe.\n",
                                   "behaviour": {"write": {".control.json": control([])}}}],
                       tamper=alter_input)
        r, out = s.run()
        self.assertEqual(r.returncode, 2, r.stdout + r.stderr)
        self.assertFalse(os.path.exists(os.path.join(out, "C1.control.json")), "the harness ran")
        self.assertFalse(os.path.exists(os.path.join(s.docs, ".run-started")))
        return r.stderr

    def test_a_path_that_climbs_out_of_the_documents_folder(self):
        self.assertIn("'../outside/C1.docx' is not a plain relative path", self.escape("parent"))

    def test_an_absolute_path(self):
        self.assertIn("is an absolute path", self.escape("absolute"))

    def test_a_link_out_of_the_documents_folder(self):
        self.assertIn("'C1.docx' is a symbolic link", self.escape("link"))

    def test_a_link_that_stays_inside_the_documents_folder(self):
        self.assertIn("'C1.docx' is a symbolic link", self.escape("inner link"))

    def test_a_seal_entry_that_climbs_out(self):
        s = self.make()
        open(os.path.join(self.tmp, "outside.txt"), "w").write("x\n")
        with open(os.path.join(s.seal, "SEAL.txt"), "a") as f:
            f.write(f"{sha(b'x' + bytes([10]))}  ../../../outside.txt\n")
        s.git("add", "-A")
        s.git("commit", "-q", "--amend", "--no-edit")
        s.commit = s.git("rev-parse", "HEAD").strip()
        self.assertIn("'../../../outside.txt' is not a plain relative path", self.refused(s))

    def test_a_link_in_the_seal_folder(self):
        def link(fixture):
            open(os.path.join(fixture.tmp, "notes.md"), "w").write("outside\n")
            os.symlink(os.path.join(fixture.tmp, "notes.md"), os.path.join(fixture.seal, "notes.md"))
        err = self.refused(self.make(tamper=link))
        self.assertIn("'notes.md' is a symbolic link", err)


class Builder(Fixture):
    def test_the_builder_can_check_a_set_before_sealing(self):
        s = self.make()
        r = subprocess.run(["python3", CHECK, s.seal, s.docs, f"--inventory={s.inventory}"],
                           capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        report = json.loads(r.stdout)
        self.assertEqual(report["classes"], {"control": 2, "exploratory": 1, "mixed": 1, "scored": 2})
        self.assertEqual(report["authentic_scored_evidence"]["documents"], 1)
        self.assertEqual(report["authentic_scored_evidence"]["reference_verses"], 100)
        self.assertFalse(report["authentic_scored_evidence"]["meets_minimum"])


class Sealing(Fixture):
    """seal-set.py: the builder's check before the seal, and the seal."""

    def unsealed(self, ledger=True):
        s = self.make()
        os.remove(os.path.join(s.seal, "SEAL.txt"))
        if not ledger:
            os.remove(os.path.join(s.seal, "review-ledger.md"))
        return s

    def seal_set(self, s, *extra):
        return subprocess.run(["python3", SEAL, s.seal, s.docs, f"--inventory={s.inventory}", *extra],
                              capture_output=True, text=True)

    def test_the_builder_can_check_before_the_reviewer_signs(self):
        s = self.unsealed(ledger=False)
        r = self.seal_set(s)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("review-ledger.md not written yet", r.stderr)
        self.assertFalse(os.path.exists(os.path.join(s.seal, "SEAL.txt")))

    def test_the_seal_waits_for_the_reviewers_ledger(self):
        s = self.unsealed(ledger=False)
        r = self.seal_set(s, "--write")
        self.assertEqual(r.returncode, 2)
        self.assertIn("review-ledger.md is missing or empty", r.stderr)
        self.assertFalse(os.path.exists(os.path.join(s.seal, "SEAL.txt")))

    def test_the_seal_written_is_one_the_checks_accept(self):
        s = self.unsealed()
        r = self.seal_set(s, "--write")
        self.assertEqual(r.returncode, 0, r.stderr)
        r = subprocess.run(["python3", CHECK, s.seal, s.docs, f"--inventory={s.inventory}"],
                           capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)

    def test_a_sealed_folder_is_never_sealed_again(self):
        s = self.make()
        r = self.seal_set(s, "--write")
        self.assertEqual(r.returncode, 2)
        self.assertIn("never sealed again", r.stderr)

    def test_a_link_is_never_sealed(self):
        s = self.unsealed()
        open(os.path.join(self.tmp, "notes.md"), "w").write("outside\n")
        os.symlink(os.path.join(self.tmp, "notes.md"), os.path.join(s.seal, "notes.md"))
        r = self.seal_set(s, "--write")
        self.assertEqual(r.returncode, 2)
        self.assertIn("'notes.md' is a symbolic link", r.stderr)
        self.assertFalse(os.path.exists(os.path.join(s.seal, "SEAL.txt")))

    def test_nothing_is_sealed_that_the_checks_refuse(self):
        s = self.unsealed()
        with open(os.path.join(s.docs, "S1.epub"), "ab") as f:
            f.write(b"x")
        r = self.seal_set(s, "--write")
        self.assertEqual(r.returncode, 2)
        self.assertIn("differs from its checksum", r.stderr)
        self.assertFalse(os.path.exists(os.path.join(s.seal, "SEAL.txt")))


class AuthenticMinimum(unittest.TestCase):
    def load(self):
        spec = importlib.util.spec_from_file_location("score_set", SCORE)
        m = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(m)
        return m

    def doc(self, i, fmt, lang, printed):
        return {"key": f"A{i}", "class": "scored", "kind": "authentic", "format": fmt, "language": lang,
                "execution": "completed", "exit": 0,
                "observed": {"printed": printed, "verified": printed, "verified_faithful": printed,
                             "verified_mismatch": 0, "verified_absent": 0}}

    def assess(self, m, docs):
        seal = {"manifest_sha256": None, "checksums_sha256": None, "composition": {}, "declared_composition": {}}
        return m.assess(docs, {"fields": {}}, seal, {}, SCORE, "x", {"committed_and_unchanged": False}, {})

    def test_the_declared_minimum_is_met_or_the_limit_is_stated(self):
        m = self.load()
        enough = [self.doc(i, ["epub", "pdf"][i % 2], ["en", "de", "fr", "es"][i % 4], 2500) for i in range(8)]
        a = self.assess(m, enough)
        self.assertTrue(a["authentic_evidence"]["meets_minimum"])
        self.assertIsNone(a["qualification_limit"])
        a = self.assess(m, enough[:7])
        self.assertFalse(a["authentic_evidence"]["meets_minimum"])
        self.assertIn("below the declared minimum", a["qualification_limit"])
        few_verses = [self.doc(i, ["epub", "pdf"][i % 2], ["en", "de", "fr", "es"][i % 4], 100) for i in range(8)]
        self.assertFalse(self.assess(m, few_verses)["authentic_evidence"]["meets_minimum"])


if __name__ == "__main__":
    unittest.main()
