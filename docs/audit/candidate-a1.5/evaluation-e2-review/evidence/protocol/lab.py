#!/usr/bin/env python3
"""Independent review of A1.5-E2: a scratch evaluation laboratory.

A scratch Git repository outside the project holds a copy of the E2 tools
(byte-identical to the reviewed commit). Sets are built from the reviewer's
own INVENTED text (no Scripture, no withheld material), sealed with
seal-set.py --write, committed, run once with run-withheld-set.sh and scored
with score-set.py, officially and/or diagnostically.
"""
import hashlib
import io
import json
import os
import random
import shutil
import subprocess
import sys
import zipfile

REPO = "/home/user/abidingword"
TOOLS = ["scripts/evaluation/check-set.py", "scripts/evaluation/seal-set.py",
         "scripts/evaluation/run-withheld-set.sh", "scripts/evaluation/score-set.py",
         "scripts/evaluation/fidelity.py", "scripts/evaluation/reference_inventory.py",
         "docs/evaluation/protocol-e2.json", "docs/evaluation/usfm-books.tsv",
         "docs/evaluation/disclosed-documents.sha256"]
COLUMNS = ["key", "files", "reference", "language", "script", "format", "layout", "role",
           "source and licence", "books", "sha256", "generator", "kind", "class"]
PROVENANCE = ["key", "publisher", "edition", "revision", "retrieved_from", "retrieved_utc",
              "licence", "document_sha256", "reference_source", "reference_method", "identity_checked_by"]
TITLES = {"OBA": "Obadiah", "JOL": "Joel", "RUT": "Ruth", "PHM": "Philemon", "2JN": "2 John",
          "3JN": "3 John", "JUD": "Jude", "HAG": "Haggai", "HAB": "Habakkuk", "JON": "Jonah", "NAM": "Nahum"}

# ---- invented text (the reviewer's own vocabulary; not Scripture)
WHO = ["the lantern-maker", "a ferryman", "the old scribe", "the gardener", "two shepherds", "the beekeeper",
       "a traveller", "the stonemason", "the baker's daughter", "the night watch", "a cartwright", "the archivist"]
DID = ["mended the nets", "counted the jars", "carried water", "lit the evening fire", "swept the steps",
       "read aloud", "planted figs", "sharpened the sickles", "waited patiently", "folded the cloth"]
WHERE = ["beside the cistern", "on the upper road", "near the cedar grove", "at the ford", "inside the courtyard",
         "under the awning", "by the east wall", "along the terraces", "at the crossroads"]
WHEN = ["before sunrise", "at the ninth hour", "after the storm", "in the dry season", "while it was cool",
        "on the second day", "as the bells rang"]


def sentence(rng, n):
    return (f"{rng.choice(WHO).capitalize()} {rng.choice(DID)} {rng.choice(WHERE)} {rng.choice(WHEN)}, "
            f"and the {['first','second','third','fourth','fifth','sixth','seventh'][n % 7]} lamp stayed lit.")


def git(repo, *args, check=True):
    r = subprocess.run(["git", "-C", repo, *args], capture_output=True, text=True)
    if check and r.returncode:
        raise RuntimeError(f"git {args}: {r.stderr}")
    return r.stdout.strip()


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def zip_bytes(members):
    buf = io.BytesIO()
    with zipfile.ZipFile(buf, "w", zipfile.ZIP_DEFLATED) as z:
        for name, data in members:
            z.writestr(zipfile.ZipInfo(name, date_time=(2026, 11, 22, 0, 0, 0)),
                       data if isinstance(data, bytes) else data.encode("utf-8"))
    return buf.getvalue()


def markdown(books):
    """books: [(title, [(chapter, [(verse, text)])])] -> Markdown in the derived layout."""
    out = []
    for title, chapters in books:
        out.append(f"# {title}\n")
        for c, verses in chapters:
            out.append(f"## {c}\n")
            para = []
            for v, t in verses:
                para.append(f"{v} {t}")
                if len(para) == 5:
                    out.append(" " + " ".join(para) + "\n")
                    para = []
            if para:
                out.append(" " + " ".join(para) + "\n")
    return "\n".join(out)


def usfm(code, chapters, id_line=True, extra_head=""):
    """chapters: [(chapter label, [(verse label, raw USFM verse content)])]."""
    lines = [f"\\id {code} REVIEW invented text, not Scripture"] if id_line else []
    lines += [f"\\h {TITLES.get(code, code)}", f"\\mt1 {TITLES.get(code, code)}"] + ([extra_head] if extra_head else [])
    for c, verses in chapters:
        lines += [f"\\c {c}", "\\p"]
        for v, t in verses:
            lines.append(f"\\v {v} {t}" if t else f"\\v {v}")
    return "\n".join(lines) + "\n"


def plain_book(code, counts, seed):
    """Invented verses for chapters with `counts` verses: [(chapter, [(verse, text)])]."""
    rng = random.Random(seed)
    return [(c + 1, [(v + 1, sentence(rng, v)) for v in range(n)]) for c, n in enumerate(counts)]


class Lab:
    def __init__(self, work):
        self.work = os.path.abspath(work)
        if os.path.exists(self.work):
            shutil.rmtree(self.work)
        self.repo = os.path.join(self.work, "repo")
        self.docs_root = os.path.join(self.work, "documents")
        os.makedirs(self.repo)
        git(self.repo, "init", "-q")
        git(self.repo, "config", "user.email", "review@example.invalid")
        git(self.repo, "config", "user.name", "independent review")
        for rel in TOOLS + [f"docs/corpus/{f}" for f in sorted(os.listdir(f"{REPO}/docs/corpus")) if f.endswith(".tsv")]:
            dst = os.path.join(self.repo, rel)
            os.makedirs(os.path.dirname(dst), exist_ok=True)
            shutil.copy(os.path.join(REPO, rel), dst)
        git(self.repo, "add", "-A")
        git(self.repo, "commit", "-qm", "E2 tools, byte-identical to the reviewed commit")
        self.tools = os.path.join(self.repo, "scripts", "evaluation")
        self.protocol = os.path.join(self.repo, "docs", "evaluation", "protocol-e2.json")

    # -------------------------------------------------------------- sets
    def build(self, name, docs, composition=None, extra=None):
        """docs: dicts with key, class, kind, books, md (text or bytes), file (opt), ref_members (opt),
        allowed (opt), lang (opt)."""
        seal = os.path.join(self.repo, "eval", name, "seal")
        folder = os.path.join(self.docs_root, name)
        os.makedirs(seal)
        os.makedirs(folder)
        rows, prov, sums = [], [], []
        for d in docs:
            fname = d.get("file", f"{d['key']}.md")
            data = d["md"] if isinstance(d["md"], bytes) else d["md"].encode("utf-8")
            open(os.path.join(folder, fname), "wb").write(data)
            sums.append((sha(os.path.join(folder, fname)), fname))
            ref = "-"
            if d.get("ref_members") is not None:
                ref = d.get("ref_name", f"{d['key']}_usfm.zip")
                payload = d["ref_members"] if isinstance(d["ref_members"], bytes) else zip_bytes(d["ref_members"])
                open(os.path.join(folder, ref), "wb").write(payload)
                sums.append((sha(os.path.join(folder, ref)), ref))
            rows.append([d["key"], fname, ref, d.get("lang", "en"), d.get("script", "Latin"), "md", "review",
                         "review", "REVIEW invented text", d.get("books", "all" if ref != "-" else "-"),
                         sha(os.path.join(folder, fname)), "review lab.py", d.get("kind", "generated"), d["class"]])
            prov.append([d["key"], "REVIEW", "invented", "1", "generated locally", "2026-11-22T00:00:00Z",
                         "none (invented)", sha(os.path.join(folder, fname)),
                         "generated with the document" if ref != "-" else "-",
                         "same generator" if ref != "-" else "-", "reviewer"])
            if d.get("allowed"):
                os.makedirs(os.path.join(seal, "allowed"), exist_ok=True)
                open(os.path.join(seal, "allowed", f"{d['key']}.txt"), "w").write("\n".join(d["allowed"]) + "\n")
        w = lambda n, t: open(os.path.join(seal, n), "w", encoding="utf-8").write(t)
        w("manifest.tsv", "\t".join(COLUMNS) + "\n" + "".join("\t".join(r) + "\n" for r in rows))
        w("provenance.tsv", "\t".join(PROVENANCE) + "\n" + "".join("\t".join(r) + "\n" for r in prov))
        w("checksums.sha256", "".join(f"{h}  {n}\n" for h, n in sorted(sums, key=lambda x: x[1])))
        counts = {c: sum(r[13] == c for r in rows) for c in ("scored", "control", "mixed", "exploratory")}
        counts.update({k: sum(r[12] == k for r in rows) for k in ("authentic", "generated")})
        counts.update(composition or {})
        w("composition.tsv", "measure\tdocuments\n" + "".join(f"{k}\t{v}\n" for k, v in counts.items()))
        w("review-ledger.md", "REVIEW laboratory set: invented text. Not a withheld set.\n")
        w("access-records.md", "Built by the independent reviewer's lab.py. No withheld document read.\n")
        for n, t in (extra or {}).items():
            w(n, t)
        return seal, folder

    def seal(self, name, protocol=None, inventory=None):
        seal = os.path.join(self.repo, "eval", name, "seal")
        args = [sys.executable, os.path.join(self.tools, "seal-set.py"), seal,
                os.path.join(self.docs_root, name), "--write"]
        args += [f"--protocol={protocol}"] if protocol else []
        args += [f"--inventory={inventory}"] if inventory else []
        r = subprocess.run(args, capture_output=True, text=True)
        refusals = [l for l in r.stderr.splitlines() if l.startswith("refused")]
        if r.returncode == 0:
            git(self.repo, "add", "--", f"eval/{name}/seal")
            git(self.repo, "commit", "-qm", f"seal {name}")
            return True, git(self.repo, "rev-parse", "HEAD"), refusals
        return False, None, refusals

    def run(self, name, seal_commit, binary, want=None, env=None):
        out = os.path.join(self.work, "outputs", name)
        e = dict(os.environ, **(env or {}))
        r = subprocess.run(["bash", os.path.join(self.tools, "run-withheld-set.sh"),
                            os.path.join(self.repo, "eval", name, "seal"), seal_commit,
                            os.path.join(self.docs_root, name), binary, want or sha(binary), out],
                           capture_output=True, text=True, env=e)
        record = git(self.repo, "rev-parse", "HEAD") if r.returncode == 0 else None
        return r.returncode, record, out, (r.stdout + r.stderr)[-1500:]

    def score(self, name, out, record, *extra, scorer=None):
        js = os.path.join(self.work, f"score-{name}-{abs(hash(extra)) % 10**8}.json")
        if os.path.exists(js):
            os.remove(js)
        args = [sys.executable, scorer or os.path.join(self.tools, "score-set.py"), out,
                os.path.join(self.repo, "eval", name, "seal"), f"--run-record={record}", f"--json={js}", *extra]
        r = subprocess.run(args, capture_output=True, text=True)
        j = json.load(open(js)) if r.returncode == 0 and os.path.exists(js) else None
        return r.returncode, j, r.stderr[-1500:]


def summary(j):
    if j is None:
        return None
    a = j["assessment"]
    s = a["scored"]
    return {"mode": j["mode"], "verdict": a["verdict"], "diagnostic_verdict": a.get("diagnostic_verdict"),
            "failed_validity": [k for k, v in a["run_validity"].items() if not v],
            "criteria_failed": [k for k, v in a["criteria"].items() if not v],
            "integrity": a["integrity"],
            "scored": {k: s.get(k) for k in ("documents", "printed", "printed_unsupported", "verified",
                                             "verified_faithful", "verified_mismatch", "verified_mismatch_kinds",
                                             "verified_absent", "verified_unvalidatable", "coverage_percent",
                                             "coverage_percent_of_all_printed", "refused_whole", "unsupported")},
            "claim": a["accuracy_claim"]}
