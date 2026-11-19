"""Qualifies the withheld-set tools (Phase A1.5) on synthetic sets.

A fake harness stands in for the recogniser: for each document it writes
the report it is told to write, then exits, sleeps, aborts or is killed as
told. No Scripture, no corpus and no recogniser are needed, so every
refusal of the runner and every way a document's run can end are shown.

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
import unittest
import warnings
import zipfile

warnings.simplefilter("ignore", ResourceWarning)

ROOT = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
TOOLS = os.path.join(ROOT, "scripts", "evaluation")
RUNNER = os.path.join(TOOLS, "run-withheld-set.sh")
CHECK = os.path.join(TOOLS, "check-set.py")
SEAL = os.path.join(TOOLS, "seal-set.py")
SCORE = os.path.join(TOOLS, "score-set.py")

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


def report(printed, verified, wrong=(), typography=(), absent=(), clean=0, caught=0):
    differences = [{"reference": r, "verified": True, "kind": "changed"} for r in wrong]
    differences += [{"reference": r, "verified": True, "kind": "typography"} for r in typography]
    differences += [{"reference": "GEN 1:1", "verified": False, "kind": "changed"}] * caught
    return {"totals": {"expected": printed, "verified": verified, "unverified_clean": clean,
                       "unverified_caught": caught},
            "differences": differences, "verified_extra_verses": list(absent),
            "refused_chapters": [], "books_not_found": [], "read_peak_memory_mb": 40.0,
            "read_seconds": 0.1}


def control(references, refused=None):
    c = {"control": True, "detected": len(references), "verified": len(references),
         "verified_references": list(references), "peak_memory_mb": 20.0, "seconds": 0.1}
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
                 tamper=None):
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
        rows, prov, sums = [], [], {}
        for d in docs:
            key = d["key"]
            name = f"{key}.{d.get('format', 'epub')}"
            content = d.get("content", f"document {key}".encode())
            open(os.path.join(self.docs, name), "wb").write(content)
            sums[name] = sha(content)
            ref = "-"
            if d.get("reference", d["class"] in ("scored", "exploratory")):
                ref = f"{key}.usfm.zip"
                data = d.get("reference_bytes", usfm_zip(f"\\id MAT\n\\c 1\n\\v 1 {key}\n"))
                open(os.path.join(self.docs, ref), "wb").write(data)
                sums[ref] = sha(data)
            rows.append([key, name, ref, d.get("language", "en"), "Latin", d.get("format", "epub"), "-",
                         "set-x", "test", "all", sums[name], "-", d.get("kind", "generated"), d["class"]])
            prov.append([key, "pub", "ed", "rev", "https://example.invalid/x", "2026-11-19T00:00:00Z",
                         "public domain", sums[name], "src" if ref != "-" else "-",
                         "method" if ref != "-" else "-", "reviewer"])
            json.dump(d.get("behaviour", {}), open(os.path.join(self.behaviour, key + ".json"), "w"))
        self.write("manifest.tsv", "\t".join(HEADER) + "\n" + "".join("\t".join(r) + "\n" for r in rows))
        self.write("checksums.sha256", "".join(f"{h}  {n}\n" for n, h in sorted(sums.items())))
        self.write("provenance.tsv", "\t".join(PROV) + "\n" + "".join("\t".join(r) + "\n" for r in prov))
        self.write("review-ledger.md", "signed\n")
        self.write("access-records.md", "records\n")
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
        env = dict(os.environ, FAKE_BEHAVIOUR=self.behaviour, AW_DISCLOSED_INVENTORY=self.inventory)
        if timeout:
            env["AW_RUN_TIMEOUT_SECONDS"] = str(timeout)
        r = subprocess.run(["bash", RUNNER, self.seal, self.commit, self.docs, self.fake,
                            binsha or self.fake_sha, out], capture_output=True, text=True, env=env)
        if check:
            self.assertOk(r)
        return r, out

    @staticmethod
    def assertOk(r):
        if r.returncode != 0:
            raise AssertionError(r.stdout + r.stderr)

    def score(self, out):
        js = os.path.join(self.tmp, "score.json")
        r = subprocess.run(["python3", SCORE, out, self.seal, f"--json={js}",
                            f"--frozen-binary={self.fake_sha}", f"--inventory={self.inventory}"],
                           capture_output=True, text=True)
        return r, (json.load(open(js)) if r.returncode == 0 else None)


GOOD = [
    {"key": "S1", "class": "scored", "kind": "authentic", "format": "epub",
     "behaviour": {"write": {".json": report(100, 90, typography=["MAT 1:3"], clean=8, caught=2)}}},
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
        self.assertIn("is not the frozen", self.refused(self.make(), binsha="0" * 64))

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
        book = b"\\id MAT\n\\c 1\n\\v 1 S1\n"
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
        self.assertEqual(a["scored"]["verified_wrong"], 0)
        self.assertEqual(a["scored"]["verified_typography"], 1)
        self.assertIn("Exploratory documents are not part of this figure", a["accuracy_claim"])
        self.assertEqual(score["groups"]["format"], {"epub": {
            "documents": 1, "printed": 100, "verified": 90, "verified_correct": 90, "verified_wrong": 0,
            "verified_typography": 1, "verified_absent": 0, "refused_correct": 8, "caught": 2,
            "refused_chapters": 0}})
        e1 = next(d for d in score["documents"] if d["key"] == "E1")
        self.assertEqual(e1["observed"]["verified_wrong"], 40)  # raw observation kept
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
            "1_no_wrong_verified_text": False,
            "2_no_verified_reference_outside_the_edition": False,
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
        self.assertIn("not a run of this seal", r.stderr)


class Builder(Fixture):
    def test_the_builder_can_check_a_set_before_sealing(self):
        s = self.make()
        r = subprocess.run(["python3", CHECK, s.seal, s.docs, f"--inventory={s.inventory}"],
                           capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        report = json.loads(r.stdout)
        self.assertEqual(report["classes"], {"control": 2, "exploratory": 1, "mixed": 1, "scored": 2})
        self.assertEqual(report["authentic_scored_evidence"]["documents"], 1)
        self.assertEqual(report["authentic_scored_evidence"]["reference_verses"], 1)
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
                "observed": {"printed": printed, "verified": printed, "verified_correct": printed,
                             "verified_wrong": 0, "verified_absent": 0}}

    def test_the_declared_minimum_is_met_or_the_limit_is_stated(self):
        m = self.load()
        run = {"fields": {}}
        seal = {"manifest_sha256": None, "checksums_sha256": None}
        enough = [self.doc(i, ["epub", "pdf"][i % 2], ["en", "de", "fr", "es"][i % 4], 2500) for i in range(8)]
        a = m.assess(enough, run, seal, "x")
        self.assertTrue(a["authentic_evidence"]["meets_minimum"])
        self.assertIsNone(a["qualification_limit"])
        a = m.assess(enough[:7], run, seal, "x")
        self.assertFalse(a["authentic_evidence"]["meets_minimum"])
        self.assertIn("below the declared minimum", a["qualification_limit"])
        few_verses = [self.doc(i, ["epub", "pdf"][i % 2], ["en", "de", "fr", "es"][i % 4], 100) for i in range(8)]
        self.assertFalse(m.assess(few_verses, run, seal, "x")["authentic_evidence"]["meets_minimum"])


if __name__ == "__main__":
    unittest.main()
