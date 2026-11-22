#!/usr/bin/env python3
"""End-to-end rehearsals of the withheld-set tools with the REAL harness
binary (evaluation candidate A1.5-E3): seal, run once, score officially.

Each scenario builds a set of SYNTHETIC documents (invented text, labelled
as such; never Scripture, never withheld material) in a scratch Git
repository outside the project, which holds a byte-for-byte copy of one
version of the tools (`--tools`, the E2 tools for the failing-before
evidence, this checkout for E3). The set is checked and sealed with
`seal-set.py --write`, the seal committed alone, run once with
`run-withheld-set.sh` and the real `real_document_editions` binary, and
scored with `score-set.py` in official mode.

An official verdict needs an owner-approved protocol. None exists, so each
scenario commits, **in the scratch repository only**, a simulated owner
revision of the protocol: the set's own composition, the binary's SHA-256
and (E3) permission to run, with the copied tools as the approved ones.
Nothing here is an approval of anything.

    rehearse.py --tools=DIR --version=e2|e3 --binary=FILE --work=DIR --out=FILE [--only=a,b]

Writes one JSON summary per scenario (verdict, validity, counts, refusals).
"""
import argparse
import hashlib
import io
import json
import os
import random
import shutil
import subprocess
import sys
import zipfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.normpath(os.path.join(HERE, "..", "..", ".."))
COLUMNS = ["key", "files", "reference", "language", "script", "format", "layout", "role",
           "source and licence", "books", "sha256", "generator", "kind", "class"]
PROVENANCE = ["key", "publisher", "edition", "revision", "retrieved_from", "retrieved_utc", "licence",
              "document_sha256", "reference_source", "reference_method", "identity_checked_by"]
# The tools an E3 protocol approves by SHA-256 (score-set.py's TOOL_FILES).
TOOL_FILES = ["scripts/evaluation/score-set.py", "scripts/evaluation/check-set.py",
              "scripts/evaluation/seal-set.py", "scripts/evaluation/run-withheld-set.sh",
              "scripts/evaluation/fidelity.py", "scripts/evaluation/reference_inventory.py",
              "scripts/evaluation/usfm_text.py", "docs/evaluation/usfm-books.tsv"]
TITLES = {"OBA": "Obadiah", "RUT": "Ruth", "PHM": "Philemon", "2JN": "2 John", "3JN": "3 John",
          "JUD": "Jude", "HAG": "Haggai", "JON": "Jonah", "NAM": "Nahum", "TOB": "Tobit"}

# ---------------------------------------------------------------- invented text

NOUNS = ["the potter", "a weaver", "the miller", "two fishers", "the lamp-trimmer", "a clerk", "the glazier",
         "the orchard keeper", "a carter", "the bell-ringer", "the cooper", "a tanner"]
VERBS = ["stacked the bowls", "wound the thread", "ground the barley", "mended a sail", "trimmed the wicks",
         "copied the ledger", "set the panes", "pruned the quinces", "loaded the wagon", "oiled the hinges"]
PLACES = ["behind the granary", "near the lower gate", "past the reed beds", "under the plane tree",
          "beside the kiln", "on the chalk path", "inside the wool shed", "at the market cross"]
TIMES = ["at first light", "before the rain", "when the tide turned", "late in the evening", "on the fourth day",
         "after the harvest", "while the kettle sang"]


def invented(rng, n):
    return (f"{rng.choice(NOUNS).capitalize()} {rng.choice(VERBS)} {rng.choice(PLACES)} {rng.choice(TIMES)}, "
            f"and the {['first', 'second', 'third', 'fourth', 'fifth', 'sixth'][n % 6]} bell rang.")


def book(code, counts, seed):
    """[(chapter, [(verse, text)])] of invented text."""
    rng = random.Random(seed)
    return [(c + 1, [(v + 1, invented(rng, v)) for v in range(n)]) for c, n in enumerate(counts)]


def usfm(code, chapters, raw=None, head=""):
    """A USFM book of invented text; `raw` maps (chapter, verse) to raw USFM
    to print instead of the plain text (notes, spans, milestones)."""
    lines = [f"\\id {code} SYNTHETIC rehearsal reference, invented text, not Scripture",
             f"\\h {TITLES.get(code, code)}", f"\\mt1 {TITLES.get(code, code)}"] + ([head] if head else [])
    for c, verses in chapters:
        lines += [f"\\c {c}", "\\p"]
        for v, t in verses:
            lines.append(f"\\v {v} {(raw or {}).get((c, v), t)}")
    return "\n".join(lines) + "\n"


def markdown(books):
    """The derived Markdown layout: `# title`, `## chapter`, paragraphs of
    numbered verses."""
    out = []
    for title, chapters in books:
        out.append(f"# {title}\n")
        for c, verses in chapters:
            out.append(f"## {c}\n")
            for k in range(0, len(verses), 5):
                out.append(" " + " ".join(f"{v} {t}" for v, t in verses[k:k + 5]) + "\n")
    return "\n".join(out)


def zip_bytes(members):
    buf = io.BytesIO()
    with zipfile.ZipFile(buf, "w", zipfile.ZIP_DEFLATED) as z:
        for name, data in members:
            z.writestr(zipfile.ZipInfo(name, date_time=(2026, 11, 22, 0, 0, 0)),
                       data if isinstance(data, bytes) else data.encode("utf-8"))
    return buf.getvalue()


def sha(data):
    return hashlib.sha256(data).hexdigest()


def sha_file(path):
    return sha(open(path, "rb").read())


# ---------------------------------------------------------------- scenarios

def doc(key, cls, text, reference=None, books="all", kind="generated", allowed=None):
    return {"key": key, "class": cls, "kind": kind, "text": text, "reference": reference, "books": books,
            "allowed": allowed}


def scenarios():
    """{name: (what it probes, [documents])}. Every text is invented."""
    s = {}
    # A clean set: the official path can pass.
    oba, jud = book("OBA", [21], 11), book("JUD", [25], 12)
    s["clean"] = ("faithful documents; the official path works end to end", [
        doc("cl-oba", "scored", markdown([("Obadiah", oba)]), [("31-OBA.usfm", usfm("OBA", oba))], "OBA"),
        doc("cl-jud", "scored", markdown([("Jude", jud)]), [("65-JUD.usfm", usfm("JUD", jud))], "JUD"),
        doc("cl-ctl", "control", "# Notes on kiln firing\n\nSYNTHETIC control: no Scripture. The kiln was "
            "fired twice and the glaze held.\n"),
    ])
    # R1: a note placed against punctuation, as eBible editions place them.
    rut = book("RUT", [22, 23], 21)
    raw, printed = {}, []
    for c, verses in rut:
        shown = []
        for v, t in verses:
            if v % 4 == 1:
                raw[(c, v)] = t[:-1] + ".\\f + \\fr %d:%d \\ft SYNTHETIC note.\\f*" % (c, v)
            elif v % 4 == 2:
                raw[(c, v)] = "“" + t[:-1] + ".\\x - \\xo %d:%d \\xt Invented 1:1\\x*”" % (c, v)
                t = "“" + t + "”"  # the edition prints the quotation marks; the document too
            elif v % 4 == 3:
                raw[(c, v)] = t.replace(",", "\\f + \\ft SYNTHETIC.\\f*,", 1)
            shown.append((v, t))
        printed.append((c, shown))
    s["notes"] = ("R1: notes against punctuation; faithful documents (the edition's printed text)", [
        doc("nt-rut", "scored", markdown([("Ruth", printed)]), [("08-RUT.usfm", usfm("RUT", rut, raw))], "RUT"),
    ])
    # R1: structures the app's reader mis-reads (the review's p2-shared cases).
    jn2 = book("2JN", [13], 31)
    raw = {(1, 3): jn2[0][1][2][1] + "\\x - \\xo 1:3 \\xt Invented",  # an unclosed cross-reference
           (1, 7): jn2[0][1][6][1][:-1] + "\\f + \\ft SYNTHETIC.\\f*."}
    hag = book("HAG", [15, 23], 32)
    raw_hag = {(1, 4): "\\va 5\\va " + hag[0][1][3][1]}  # an unclosed alternate number
    phm = book("PHM", [25], 33)
    raw_phm = {(1, 5): "\\qt-s |who=\"SYNTHETIC speaker\"\\*" + phm[0][1][4][1] + "\\qt-e\\*"}
    s["malformed"] = ("R1: unclosed cross-reference and alternate number; a USFM 3 milestone", [
        doc("mf-2jn", "scored", markdown([("2 John", jn2)]), [("63-2JN.usfm", usfm("2JN", jn2, raw))], "2JN"),
        doc("mf-hag", "scored", markdown([("Haggai", hag)]), [("37-HAG.usfm", usfm("HAG", hag, raw_hag))], "HAG"),
        doc("mf-phm", "scored", markdown([("Philemon", phm)]), [("57-PHM.usfm", usfm("PHM", phm, raw_phm))], "PHM"),
    ])
    s["milestone"] = ("R1: a USFM 3 milestone only (well formed)", [
        doc("ms-phm", "scored", markdown([("Philemon", phm)]), [("57-PHM.usfm", usfm("PHM", phm, raw_phm))], "PHM"),
    ])
    # R2: the review's vacuous set, in invented text.
    tob = book("TOB", [10], 41)
    s["vacuous"] = ("R2: nothing verifiable (blank, deuterocanon only, prose, reference without USFM)", [
        doc("vq-blank", "scored", "<!-- SYNTHETIC blank document -->\n",
            [("31-OBA.usfm", usfm("OBA", oba))], "OBA"),
        doc("vq-tob", "scored", markdown([("Tobit", tob)]), [("68-TOB.usfm", usfm("TOB", tob))], "TOB"),
        doc("vq-prose", "scored", "# A letter\n\nSYNTHETIC prose with no verse numbers at all.\n",
            [("65-JUD.usfm", usfm("JUD", jud))], "JUD"),
        doc("vq-nousfm", "scored", "# A notice\n\nSYNTHETIC prose, a second letter with no verses.\n",
            [("copr.htm", "<p>SYNTHETIC licence page</p>")], "all"),
    ])
    # R2: zero coverage with sound references (sealable under every version).
    s["zero"] = ("R2: zero coverage, sound references", [
        doc("zc-blank", "scored", "<!-- SYNTHETIC blank document, zero coverage -->\n",
            [("31-OBA.usfm", usfm("OBA", oba))], "OBA"),
        doc("zc-prose", "scored", "# A letter\n\nSYNTHETIC prose with no verse numbers at all.\n",
            [("65-JUD.usfm", usfm("JUD", jud))], "JUD"),
    ])
    # A real wrong word: the gate fails.
    bad = [(c, [(v, t.replace("bell", "drum") if (c, v) == (1, 4) else t) for v, t in vs]) for c, vs in oba]
    s["error"] = ("a verified verse with a changed word", [
        doc("er-oba", "scored", markdown([("Obadiah", bad)]), [("31-OBA.usfm", usfm("OBA", oba))], "OBA"),
    ])
    return s


# ---------------------------------------------------------------- the laboratory

def git(repo, *args, check=True):
    r = subprocess.run(["git", "-C", repo, *args], capture_output=True, text=True)
    if check and r.returncode:
        raise RuntimeError(f"git {' '.join(args)}: {r.stderr}")
    return r.stdout.strip()


class Lab:
    def __init__(self, tools, version, binary, work):
        self.tools, self.version, self.binary = os.path.abspath(tools), version, os.path.abspath(binary)
        self.work = os.path.abspath(work)
        if os.path.exists(self.work):
            shutil.rmtree(self.work)
        self.repo = os.path.join(self.work, "repo")
        os.makedirs(self.repo)
        git(self.repo, "init", "-q")
        git(self.repo, "config", "user.email", "rehearsal@example.invalid")
        git(self.repo, "config", "user.name", "rehearsal (scratch repository)")
        for rel in ("scripts/evaluation", "docs/evaluation", "docs/corpus"):
            shutil.copytree(os.path.join(self.tools, rel), os.path.join(self.repo, rel))
        git(self.repo, "add", "-A")
        git(self.repo, "commit", "-qm", f"tools of {version}, byte for byte")
        self.protocol_rel = f"docs/evaluation/protocol-{version}.json"
        self.binary_sha = sha_file(self.binary)

    def tool(self, name):
        return os.path.join(self.repo, "scripts", "evaluation", name)

    def revise_protocol(self, composition):
        """The simulated owner revision, committed in the scratch repository."""
        path = os.path.join(self.repo, self.protocol_rel)
        p = json.load(open(path, encoding="utf-8"))
        p["required_composition"] = composition
        p["harness_binary_sha256"] = self.binary_sha
        p["status"] = "SIMULATED owner revision, scratch repository only"
        if self.version != "e2":
            p["withheld_execution_permitted"] = True
            p["approved_tools"] = {rel: sha_file(os.path.join(self.repo, rel)) for rel in TOOL_FILES}
        json.dump(p, open(path, "w", encoding="utf-8"), indent=1)
        git(self.repo, "add", "-A")
        git(self.repo, "commit", "-qm", "SIMULATED owner revision of the protocol (scratch only)")

    def build(self, name, docs):
        seal = os.path.join(self.repo, "sets", name, "seal")
        folder = os.path.join(self.work, "documents", name)
        os.makedirs(seal)
        os.makedirs(folder)
        rows, prov, sums, sources = [], [], {}, []
        for d in docs:
            data = d["text"].encode("utf-8")
            fname = f"{d['key']}.md"
            open(os.path.join(folder, fname), "wb").write(data)
            sums[fname] = sha(data)
            ref = "-"
            if d["reference"] is not None:
                ref = f"{d['key']}_usfm.zip"
                package = zip_bytes(d["reference"])
                open(os.path.join(folder, ref), "wb").write(package)
                sums[ref] = sha(package)
                for member, content in d["reference"]:
                    content = content if isinstance(content, bytes) else content.encode("utf-8")
                    sources.append([d["key"], ref, "SYNTHETIC invented package", sha(package), member, sha(content),
                                    "kept"])
            rows.append([d["key"], fname, ref, "en", "Latin", "md", "derived", "rehearsal", "SYNTHETIC, invented",
                         d["books"] if ref != "-" else "-", sums[fname], "rehearse.py", d["kind"], d["class"]])
            prov.append([d["key"], "SYNTHETIC", "invented", "1", "https://example.invalid/rehearsal",
                         "2026-11-22T00:00:00Z", "invented text", sums[fname], "invented" if ref != "-" else "-",
                         "generated with the document" if ref != "-" else "-", "rehearsal"])
            if d["allowed"]:
                os.makedirs(os.path.join(seal, "allowed"), exist_ok=True)
                open(os.path.join(seal, "allowed", d["key"] + ".txt"), "w").write("\n".join(d["allowed"]) + "\n")

        def w(rel, text):
            open(os.path.join(seal, rel), "w", encoding="utf-8").write(text)
        w("manifest.tsv", "\t".join(COLUMNS) + "\n" + "".join("\t".join(r) + "\n" for r in rows))
        w("checksums.sha256", "".join(f"{h}  {n}\n" for n, h in sorted(sums.items())))
        w("provenance.tsv", "\t".join(PROVENANCE) + "\n" + "".join("\t".join(r) + "\n" for r in prov))
        w("review-ledger.md", "SYNTHETIC rehearsal: no reviewer; invented documents.\n")
        w("access-records.md", "SYNTHETIC rehearsal: scratch repository only.\n")
        comp = {c: sum(r[13] == c for r in rows) for c in ("scored", "control", "mixed", "exploratory")}
        comp.update({k: sum(r[12] == k for r in rows) for k in ("authentic", "generated")})
        w("composition.tsv", "measure\tdocuments\n" + "".join(f"{k}\t{v}\n" for k, v in comp.items()))
        if self.version != "e2":
            w("reference-sources.tsv", "key\treference\tpublisher_archive\tpublisher_archive_sha256\tmember\t"
              "member_sha256\trole\n" + "".join("\t".join(r) + "\n" for r in sources))
        return seal, folder, comp

    def scenario(self, name, docs):
        out = {"scenario": name}
        seal, folder, comp = self.build(name, docs)
        self.revise_protocol(comp)
        r = subprocess.run(["python3", self.tool("seal-set.py"), seal, folder, "--write"], capture_output=True, text=True)
        out["seal_exit"] = r.returncode
        out["seal_refusals"] = [l[len("refused: "):] for l in r.stderr.splitlines() if l.startswith("refused: ")]
        out["seal_notes"] = [l[len("note: "):] for l in r.stderr.splitlines() if l.startswith("note: ")]
        if r.returncode:
            out["outcome"] = "refused at the seal"
            return out
        git(self.repo, "add", "--", os.path.relpath(seal, self.repo))
        git(self.repo, "commit", "-qm", f"seal {name}")
        seal_commit = git(self.repo, "rev-parse", "HEAD")
        run_out = os.path.join(self.work, "out", name)
        r = subprocess.run(["bash", self.tool("run-withheld-set.sh"), seal, seal_commit, folder, self.binary,
                            self.binary_sha, run_out], capture_output=True, text=True)
        out["run_exit"] = r.returncode
        if r.returncode:
            out["outcome"] = "refused by the runner"
            out["run_refusals"] = r.stderr.strip().splitlines()[-5:]
            return out
        record = git(self.repo, "rev-parse", "HEAD")
        js = os.path.join(self.work, f"{name}.score.json")
        r = subprocess.run(["python3", self.tool("score-set.py"), run_out, seal, f"--run-record={record}",
                            f"--json={js}"], capture_output=True, text=True)
        out["score_exit"] = r.returncode
        if r.returncode:
            out["outcome"] = "refused by the scorer"
            out["score_refusals"] = r.stderr.strip().splitlines()[-5:]
            return out
        score = json.load(open(js, encoding="utf-8"))
        a = score["assessment"]
        out["outcome"] = "scored"
        out["verdict"] = a["verdict"]
        for k in ("safety_verdict", "evidence_verdict", "verdict_line"):
            if k in a:
                out[k] = a[k]
        out["failed_validity"] = sorted(k for k, v in a["run_validity"].items() if not v)
        out["criteria_failed"] = sorted(k for k, v in a["criteria"].items() if not v)
        out["integrity"] = a["integrity"]
        s = a["scored"]
        out["scored"] = {k: s.get(k) for k in ("documents", "printed", "printed_unsupported", "verified",
                                               "verified_faithful", "verified_mismatch", "verified_absent",
                                               "verified_unvalidatable", "not_found", "coverage_percent")}
        out["documents"] = {d["key"]: {"execution": d["execution"],
                                       **{k: (d.get("observed") or {}).get(k) for k in
                                          ("printed", "printed_unsupported", "verified", "verified_faithful",
                                           "verified_mismatch", "verified_mismatch_kinds")},
                                       "scorable_ranges": None} for d in score["documents"]}
        # What the check declared about each reference before the run.
        check = json.load(open(os.path.join(run_out, "CHECK.json"), encoding="utf-8"))
        for key, inv in check.get("reference_inventories", {}).items():
            out["documents"][key]["scorable_ranges"] = inv.get("scorable_ranges")
        out["score_json"] = js
        self.last = (run_out, seal, record)
        self.runs = getattr(self, "runs", {})
        self.runs[name] = (run_out, seal, record)
        out["anchors"] = {k: score["anchors"].get(k) for k in ("scorer_sha256", "protocol_sha256")}
        return out

    def option_probe(self):
        """Official scoring of the last scored run with a misspelt option
        (`--Protocol=`, capital P): it must be refused, never ignored."""
        if not getattr(self, "last", None):
            return {"skipped": "no scored run"}
        run_out, seal, record = self.last
        r = subprocess.run(["python3", self.tool("score-set.py"), run_out, seal, f"--run-record={record}",
                            "--Protocol=/nonexistent/other-protocol.json"], capture_output=True, text=True)
        verdict = next((l for l in r.stdout.splitlines() if l.startswith("## Verdict")), None)
        return {"exit": r.returncode, "verdict_line": verdict, "stderr": r.stderr.strip().splitlines()[:2]}


def modified_scorer_probe(lab, name="error"):
    """The review's last finding: a scorer modified to pass everything and
    committed to the checkout. Scored officially on a failing run, it must
    not give an official pass (E3: the protocol approves the tools by SHA-256)."""
    if name not in getattr(lab, "runs", {}):
        return {"skipped": f"no scored run of {name}"}
    run_out, seal, record = lab.runs[name]
    scorer = lab.tool("score-set.py")
    text = open(scorer, encoding="utf-8").read()
    forced = text.replace('"1_no_verified_text_mismatch": scored.get("verified_mismatch", 0) == 0,',
                          '"1_no_verified_text_mismatch": True,  # MODIFIED (rehearsal)')
    if forced == text:
        return {"skipped": "criterion line not found"}
    open(scorer, "w", encoding="utf-8").write(forced)
    git(lab.repo, "commit", "-qam", "a modified scorer, committed (rehearsal)")
    js = os.path.join(lab.work, "modified-scorer.score.json")
    r = subprocess.run(["python3", scorer, run_out, seal, f"--run-record={record}", f"--json={js}"],
                       capture_output=True, text=True)
    out = {"scenario": name, "exit": r.returncode}
    if r.returncode == 0:
        a = json.load(open(js, encoding="utf-8"))["assessment"]
        out.update(verdict=a["verdict"], failed_validity=sorted(k for k, v in a["run_validity"].items() if not v),
                   scorer_sha256=json.load(open(js, encoding="utf-8"))["anchors"]["scorer_sha256"])
    else:
        out["stderr"] = r.stderr.strip().splitlines()[-3:]
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--tools", required=True)
    ap.add_argument("--version", required=True, choices=["e2", "e3"])
    ap.add_argument("--binary", required=True)
    ap.add_argument("--work", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--only", default="")
    a = ap.parse_args()
    lab = Lab(a.tools, a.version, a.binary, a.work)
    results = {"version": a.version, "binary_sha256": lab.binary_sha,
               "tools": git(lab.repo, "rev-parse", "HEAD"), "scenarios": {}}
    only = set(filter(None, a.only.split(",")))
    for name, (what, docs) in scenarios().items():
        if only and name not in only:
            continue
        r = lab.scenario(name, docs)
        r["probes"] = what
        results["scenarios"][name] = r
        print(name, r.get("outcome"), r.get("verdict", ""), r.get("seal_refusals", "") or "", file=sys.stderr)
    results["unknown_option"] = lab.option_probe()
    results["modified_committed_scorer"] = modified_scorer_probe(lab)
    json.dump(results, open(a.out, "w", encoding="utf-8"), ensure_ascii=False, indent=1)


if __name__ == "__main__":
    main()
