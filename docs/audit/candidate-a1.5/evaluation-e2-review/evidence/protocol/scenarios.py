#!/usr/bin/env python3
"""Independent review of A1.5-E2: protocol and scoring integrity attacks.

    scenarios.py <pinned harness binary> <other-build harness binary> <work folder>

Every text is the reviewer's own invention. Writes <work>/results.json.
"""
import json
import os
import stat
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from lab import Lab, markdown, usfm, plain_book, zip_bytes, sha, git, summary, TITLES  # noqa: E402

PINNED, OTHER, WORK = (os.path.abspath(a) for a in sys.argv[1:4])
lab = Lab(WORK)
R = {}
STANDARD = {"scored": 4, "control": 1, "mixed": 1, "exploratory": 0, "authentic": 0, "generated": 6}


def scored(key, code, counts, seed, *, doc_book=None, ref_chapters=None, books="all", lang="en", extra_members=()):
    """A scored document of `code` with invented verses; its reference holds the same text unless
    `ref_chapters` (raw USFM chapters) is given."""
    ch = plain_book(code, counts, seed)
    doc = doc_book if doc_book is not None else [(TITLES[code], ch)]
    ref = ref_chapters if ref_chapters is not None else [(c, [(v, t) for v, t in vs]) for c, vs in ch]
    members = [(f"41-{code}.usfm", usfm(code, ref))] + list(extra_members)
    return {"key": key, "class": "scored", "books": books, "lang": lang, "md": markdown(doc), "ref_members": members}


def control(key):
    return {"key": key, "class": "control",
            "md": "# Minutes of the garden committee\n\nItem 1: the hedges were trimmed on 3 May. Item 2: 14 "
                  "volunteers came. Item 3: the shed roof needs 2 new panels.\n"}


def mixed(key):
    verses = plain_book("RUT", [3], 77)
    return {"key": key, "class": "mixed",
            "md": "# A short reflection\n\nThe author writes about patience and quotes a few lines.\n\n"
                  + markdown([("Ruth", verses)]),
            "allowed": ["RUT 1:1-3"]}


def standard_rest(prefix):
    return [control(f"{prefix}-ctl"), mixed(f"{prefix}-mix")]


def run_set(name, docs, *, binary=PINNED, want=None, env=None, protocol=None, scorer_args=(), composition=None):
    lab.build(name, docs, composition=composition)
    ok, commit, refusals = lab.seal(name, protocol=protocol)
    res = {"sealed": ok, "seal_refusals": refusals}
    if not ok:
        return res, None, None
    rc, record, out, log = lab.run(name, commit, binary, want=want, env=env)
    res.update(run_exit=rc, run_log_tail=log[-400:] if rc else None)
    if rc:
        return res, None, None
    rc, j, err = lab.score(name, out, record, *scorer_args)
    res.update(score_exit=rc, score_error=err if rc else None, official=summary(j))
    return res, out, record


def rehearsal_protocol(path, binary_sha=None, composition=STANDARD):
    p = json.load(open(lab.protocol))
    p["name"] = "REVIEW rehearsal of " + p["name"]
    if binary_sha:
        p["harness_binary_sha256"] = binary_sha
    p["required_composition"] = composition
    json.dump(p, open(path, "w"), indent=1)
    return path


# ===================================================== phase 1: the protocol as committed (composition null)
REH = rehearsal_protocol(os.path.join(WORK, "rehearsal-standard.json"))
clean = [scored("p1-oba", "OBA", [21], 1), scored("p1-phm", "PHM", [25], 2), scored("p1-2jn", "2JN", [13], 3),
         scored("p1-hag", "HAG", [15, 23], 4)] + standard_rest("p1")
res, out, record = run_set("p1-clean", clean)
R["P1 official protocol as committed (required_composition null), clean set, pinned binary"] = res
# Overrides without --diagnostic; diagnostic alone; diagnostic with the rehearsal protocol.
for opt in (f"--protocol={REH}", "--frozen-binary=" + "0" * 64, f"--inventory={lab.protocol}"):
    rc, j, err = lab.score("p1-clean", out, record, opt)
    R[f"P1 override {opt.split('=')[0]} without --diagnostic"] = {"exit": rc, "stderr": err.strip()[-300:]}
rc, j, err = lab.score("p1-clean", out, record, "--diagnostic")
R["P1 --diagnostic with no override"] = summary(j)
rc, j, err = lab.score("p1-clean", out, record, "--diagnostic", f"--protocol={REH}")
R["P1 --diagnostic with a rehearsal protocol that sets the composition"] = summary(j)
rc, j, err = lab.score("p1-clean", out, record, "--Protocol=" + REH)
R["P1 unknown option spelling --Protocol= (ignored?)"] = {"exit": rc, "summary": summary(j)}

# ===================================================== phase 2: an owner-style protocol revision is committed
p = json.load(open(lab.protocol))
p["required_composition"] = STANDARD
p["status"] = "REVIEW simulation of an owner-approved revision (scratch repository only)"
json.dump(p, open(lab.protocol, "w"), indent=1)
git(lab.repo, "commit", "-qam", "REVIEW: simulated protocol revision with a required composition")
R["P2 simulated revised protocol sha256"] = sha(lab.protocol)

res, out2, record2 = run_set("p2-clean", [scored("p2-oba", "OBA", [21], 11), scored("p2-phm", "PHM", [25], 12),
                                          scored("p2-2jn", "2JN", [13], 13), scored("p2-hag", "HAG", [15, 23], 14)]
                             + standard_rest("p2"))
R["P2 clean set under the revised protocol (official path end to end)"] = res

# B3 and other nonexistent references (the reviewer's own variants)
jol = plain_book("JOL", [20, 32, 21], 21)
ruth = plain_book("RUT", [22, 23], 22)
oba = plain_book("OBA", [21], 23)
phm = plain_book("PHM", [25], 24)
extra = [
    {"key": "x-jol3", "class": "scored", "books": "all", "md": markdown([("Joel", jol)]),
     "ref_members": [("41-JOL.usfm", usfm("JOL", jol[:2]))]},                       # chapter 3 absent
    {"key": "x-rut-oba", "class": "scored", "books": "all", "md": markdown([("Ruth", ruth), ("Obadiah", oba)]),
     "ref_members": [("41-RUT.usfm", usfm("RUT", ruth))]},                          # a book absent
    {"key": "x-declared", "class": "scored", "books": "RUT", "md": markdown([("Ruth", ruth), ("Obadiah", oba)]),
     "ref_members": [("41-RUT.usfm", usfm("RUT", ruth)), ("42-OBA.usfm", usfm("OBA", oba))]},  # outside declared
    {"key": "x-phm25", "class": "scored", "books": "all", "md": markdown([("Philemon", phm)]),
     "ref_members": [("41-PHM.usfm", usfm("PHM", [(1, phm[0][1][:24])]))]},         # one verse absent
] + standard_rest("x")
res, _, _ = run_set("p2-extra", extra)
R["P2 extra chapter / book / outside declared books / extra verse"] = res

# Unsupported reference content of every class (the reviewer's own construction)
hab = plain_book("HAB", [17, 20, 19], 31)
hab_ref = [(1, [("0", "A superscription numbered zero, invented.")] + hab[0][1]),
           (2, [(v, t) for v, t in hab[1][1]] + [("7", "A second verse seven, invented."), ("12b", "Half verse.")]),
           (3, hab[2][1] + [("201", "Two hundred and one, invented.")]),
           ("4", [(1, "A fourth chapter Habakkuk does not have, invented.")]),
           ("0", [(1, "Chapter zero, invented.")]), ("2a", [(1, "Chapter two-a, invented.")])]
unsup_members = [
    ("41-HAB.usfm", usfm("HAB", hab_ref).replace("\\c 1\n", "\\v 9 Before any chapter.\n\\c 1\n", 1)),
    ("42-SIR.usfm", usfm("SIR", [(1, [(1, "Deuterocanonical book, invented."), (2, "More, invented.")])])),
    ("43-ZZZ.usfm", usfm("ZZZ", [(1, [(1, "Unknown code, invented.")])])),
    ("44-none.usfm", usfm("OBA", [(1, [(1, "No id line, invented.")])], id_line=False)),
    ("45-JON.usfm", usfm("JON", [(1, [(1, "Jonah file one, invented.")])])),
    ("46-JON.usfm", usfm("JON", [(1, [(1, "Jonah file two, invented.")])])),
]
unsup = [{"key": "u-hab", "class": "scored", "books": "all", "md": markdown([("Habakkuk", hab)]),
          "ref_members": unsup_members},
         scored("u-oba", "OBA", [21], 32), scored("u-phm", "PHM", [25], 33), scored("u-2jn", "2JN", [13], 34)] \
    + standard_rest("u")
res, outu, _ = run_set("p2-unsup", unsup)
R["P2 unsupported classes (verse 0, 12b, 201, duplicate, ch 4/0/2a, SIR, ZZZ, no id, JON twice, before chapter)"] = res
if outu:
    j = json.load(open(os.path.join(outu, "u-hab.json")))
    R["P2 u-hab inventory (harness)"] = {k: j["reference_inventory"][k] for k in
                                         ("labels", "printed_scorable", "printed_unsupported", "problems")}
    R["P2 u-hab unsupported by class"] = {k: v["references"] for k, v in j["reference_inventory"]["unsupported"].items()
                                          if v["references"] or v["printed"]}
    R["P2 u-hab verified_unvalidatable"] = j["verified_unvalidatable"]

# Vacuous pass: nothing scorable is verified, nothing goes wrong
tob = plain_book("TOB", [10], 41)
vac = [{"key": "v-empty", "class": "scored", "books": "OBA", "md": "",
        "ref_members": [("41-OBA.usfm", usfm("OBA", plain_book("OBA", [21], 42)))]},
       {"key": "v-tobit", "class": "scored", "books": "all", "md": markdown([("Tobit", tob)]),
        "ref_members": [("41-TOB.usfm", usfm("TOB", tob))]},
       {"key": "v-prose", "class": "scored", "books": "all",
        "md": "# Notes\n\nThis document holds prose only; its reference prints a book.\n",
        "ref_members": [("41-NAM.usfm", usfm("NAM", plain_book("NAM", [15, 13, 19], 43)))]},
       {"key": "v-htm-only", "class": "scored", "books": "all", "md": "# Notes\n\nNothing numbered here.\n",
        "ref_members": [("copr.htm", "<p>REVIEW: a reference archive with prose only.</p>")]}] + standard_rest("v")
res, _, _ = run_set("p2-vacuous", vac)
R["P2 vacuous set: empty doc, Tobit only, prose only, a reference with no USFM at all"] = res

# The shared USFM reader: notes, unclosed spans, milestones (reviewer's own USFM)
def shared_docs():
    a = plain_book("OBA", [21], 51)
    a_doc = [(c, [(v, (t[:-1] + ".”" if v % 4 == 0 else t)) for v, t in vs]) for c, vs in a]
    a_ref = [(c, [(v, (t[:-1] + ".\\f + \\fr 1:%d \\ft An invented note.\\f*”" % v if v % 4 == 0 else t))
                  for v, t in vs]) for c, vs in a]
    b = plain_book("2JN", [13], 52)
    b_ref = [(1, [(v, t + (" \\x - \\xo 1:3 \\xt Invented 1:1" if v == 3 else "")
                   + (" \\f + \\ft invented note\\f*" if v == 6 else "")) for v, t in b[0][1]])]
    c = plain_book("3JN", [15], 53)
    c_ref = [(1, [(v, ("\\va 3\\va " if v == 3 else "") + t) for v, t in c[0][1]])]  # \va never closed
    d = plain_book("JUD", [25], 54)
    d_ref = [(1, [(v, ('\\qt-s |who="Invented"\\*' + t + "\\qt-e\\*" if v == 2 else t)) for v, t in d[0][1]])]
    return [
        {"key": "s-notespace", "class": "scored", "books": "all", "md": markdown([("Obadiah", a_doc)]),
         "ref_members": [("41-OBA.usfm", usfm("OBA", a_ref))]},
        {"key": "s-unclosed-x", "class": "scored", "books": "all", "md": markdown([("2 John", b)]),
         "ref_members": [("41-2JN.usfm", usfm("2JN", b_ref))]},
        {"key": "s-unclosed-va", "class": "scored", "books": "all", "md": markdown([("3 John", c)]),
         "ref_members": [("41-3JN.usfm", usfm("3JN", c_ref))]},
        {"key": "s-milestone", "class": "scored", "books": "all", "md": markdown([("Jude", d)]),
         "ref_members": [("41-JUD.usfm", usfm("JUD", d_ref))]},
    ]


res, outs, _ = run_set("p2-shared", shared_docs() + standard_rest("s"))
R["P2 shared USFM reader defects (faithful documents)"] = res
if outs:
    detail = {}
    for k in ("s-notespace", "s-unclosed-x", "s-unclosed-va", "s-milestone"):
        j = json.load(open(os.path.join(outs, k + ".json")))
        inv = j["reference_inventory"]
        detail[k] = {"labels": inv["labels"], "printed_scorable": inv["printed_scorable"],
                     "absent_in_edition": inv["absent_in_edition"], "scorable_ranges": inv["scorable_ranges"],
                     "verified_mismatch": [(r["reference"], r["kind"]) for r in j["fidelity_records"]
                                           if r["status"] == "verified" and r["class"] == "mismatch"],
                     "verified_absent": j["verified_absent"],
                     "example_edition_text": next((r["edition"][:140] for r in j["fidelity_records"]), None)}
    R["P2 shared reader detail"] = detail

# A file name like a one-book file, books=all (C3)
ruth2 = plain_book("RUT", [22], 61)
res, _, _ = run_set("p2-name", [{"key": "n-ruth", "class": "scored", "books": "all", "file": "review_gen.md",
                                 "md": markdown([("Ruth", ruth2)]),
                                 "ref_members": [("41-RUT.usfm", usfm("RUT", ruth2))]},
                                scored("n-oba", "OBA", [21], 62), scored("n-phm", "PHM", [25], 63),
                                scored("n-2jn", "2JN", [13], 64)] + standard_rest("n"))
R["P2 one-book file name (review_gen.md holding Ruth), books=all"] = res

# Another binary: the same source built at another path; the runner is given its own hash
res, _, _ = run_set("p2-otherbin", [scored("b-oba", "OBA", [21], 71), scored("b-phm", "PHM", [25], 72),
                                    scored("b-2jn", "2JN", [13], 73), scored("b-hag", "HAG", [15, 23], 74)]
                    + standard_rest("b"), binary=OTHER)
R["P2 clean set run with another build of the same source (sha passed to the runner)"] = res

# A run record made by hand: incomplete (no 'ended', one document line missing)
name = "p2-handmade"
lab.build(name, [scored("h-oba", "OBA", [21], 81), scored("h-phm", "PHM", [25], 82),
                 scored("h-2jn", "2JN", [13], 83), scored("h-hag", "HAG", [15, 23], 84)] + standard_rest("h"))
ok, commit, _ = lab.seal(name)
out_h = os.path.join(WORK, "outputs", name)
os.makedirs(out_h)
seal_h = os.path.join(lab.repo, "eval", name, "seal")
chk = subprocess.run([sys.executable, os.path.join(lab.tools, "check-set.py"), seal_h,
                      os.path.join(lab.docs_root, name), f"--seal-commit={commit}",
                      f"--report={out_h}/CHECK.json"], capture_output=True, text=True)
keys = [l.split("\t")[0] for l in open(os.path.join(seal_h, "manifest.tsv")).read().splitlines()[1:]]
lines = ["started 2026-11-22T00:00:00Z", f"seal_commit {commit}", f"manifest {sha(seal_h + '/manifest.tsv')}",
         f"checksums {sha(seal_h + '/checksums.sha256')}", f"binary {sha(PINNED)}", "runner -", "check_set -",
         f"inventory {sha(lab.repo + '/docs/evaluation/disclosed-documents.sha256')}",
         f"protocol {sha(lab.protocol)}", "limits timeout_seconds 1800 memory_kb 4194304", "host review"]
for k in keys[:-1]:  # the last document is left out, and no 'ended' line
    env = dict(os.environ, AW_PIPELINE="structural", AW_CORPUS_MANIFEST=seal_h + "/manifest.tsv", AW_EDITIONS=k,
               AW_DOCUMENT_EDITIONS=os.path.join(lab.docs_root, name), AW_AUDIT_OUT=out_h)
    r = subprocess.run([PINNED, "--ignored", "--nocapture"], env=env, capture_output=True, text=True)
    open(os.path.join(out_h, k + ".log"), "w").write(r.stdout + r.stderr)
    lines.append(f"{k} exit {r.returncode} seconds 1")
open(os.path.join(out_h, "RUN.txt"), "w").write("\n".join(lines) + "\n")
subprocess.run(f"cd {out_h} && find . -type f ! -name OUTPUTS.sha256 -print0 | sort -z | xargs -0 sha256sum > OUTPUTS.sha256",
               shell=True, check=True)
rec_dir = os.path.join(lab.repo, "eval", name, "run")
os.makedirs(rec_dir)
for f in ("RUN.txt", "OUTPUTS.sha256", "CHECK.json"):
    open(os.path.join(rec_dir, f), "wb").write(open(os.path.join(out_h, f), "rb").read())
git(lab.repo, "add", "--", f"eval/{name}/run")
git(lab.repo, "commit", "-qm", "hand-made run record")
rc, j, err = lab.score(name, out_h, git(lab.repo, "rev-parse", "HEAD"))
R["P2 hand-made incomplete run record (no 'ended', a document missing)"] = {"exit": rc, "official": summary(j)}

# A run record changed in a later commit (on p2-clean)
rec = os.path.join(lab.repo, "eval", "p2-clean", "run", "RUN.txt")
open(rec, "a").write("note added later\n")
git(lab.repo, "commit", "-qam", "REVIEW: change the p2-clean run record after the run")
rc, j, err = lab.score("p2-clean", out2, record2)
R["P2 run record changed in a later commit"] = {"exit": rc, "stderr": err.strip()[-300:]}

# Seal-time refusals (the reviewer's own malformed sets)
def refusal(name, docs, composition=None):
    lab.build(name, docs, composition=composition)
    ok, _, refusals = lab.seal(name)
    return {"sealed": ok, "refusals": refusals}


base = lambda p: [scored(f"{p}-a", "OBA", [21], 91), scored(f"{p}-b", "PHM", [25], 92),
                  scored(f"{p}-c", "2JN", [13], 93)] + standard_rest(p)
oba9 = plain_book("OBA", [21], 99)
R["Seal: composition declared differently from the manifest"] = refusal(
    "r-comp1", base("rc1") + [scored("rc1-d", "HAG", [15, 23], 94)], composition={"scored": 5})
R["Seal: composition consistent with manifest but not the protocol's"] = refusal(
    "r-comp2", base("rc2") + [scored("rc2-d", "HAG", [15, 23], 94), scored("rc2-e", "NAM", [15, 13, 19], 95)])
R["Seal: verse map member"] = refusal("r-map", base("rm") + [{
    "key": "rm-d", "class": "scored", "books": "all", "md": markdown([("Obadiah", oba9)]),
    "ref_members": [("41-OBA.usfm", usfm("OBA", oba9)), ("OBA.VERSEMAP.XML", "<verseMap/>")]}])
R["Seal: verse map hidden in a .usfm-named member"] = refusal("r-map2", base("rm2") + [{
    "key": "rm2-d", "class": "scored", "books": "all", "md": markdown([("Obadiah", oba9)]),
    "ref_members": [("41-OBA.usfm", usfm("OBA", oba9)), ("99-map.usfm", "<verseMap><book/></verseMap>")]}])
R["Seal: non-UTF-8 reference"] = refusal("r-latin1", base("rl") + [{
    "key": "rl-d", "class": "scored", "books": "all", "md": markdown([("Obadiah", oba9)]),
    "ref_members": [("41-OBA.usfm", usfm("OBA", oba9).encode("utf-8").replace(b"lantern", b"lant\xe9rn"))]}])
R["Seal: two \\id lines in one file"] = refusal("r-twoid", base("ri") + [{
    "key": "ri-d", "class": "scored", "books": "all", "md": markdown([("Obadiah", oba9)]),
    "ref_members": [("41-OBA.usfm", usfm("OBA", oba9) + usfm("RUT", [(1, [(1, "Second id.")])]))]}])
R["Seal: books column lower case"] = refusal("r-lower", base("rw") + [dict(scored("rw-d", "OBA", [21], 96), books="oba")])
R["Seal: books column empty"] = refusal("r-emptybooks", base("re") + [dict(scored("re-d", "OBA", [21], 96), books="")])
R["Seal: scored document without a reference"] = refusal("r-noref", base("rn") + [{
    "key": "rn-d", "class": "scored", "books": "-", "md": markdown([("Obadiah", oba9)])}])
R["Seal: reference is a plain .txt"] = refusal("r-txt", base("rt") + [{
    "key": "rt-d", "class": "scored", "books": "all", "md": markdown([("Obadiah", oba9)]),
    "ref_members": b"\\id OBA\n\\c 1\n\\v 1 text\n", "ref_name": "rt-d_ref.txt"}])
R["Seal: reference zip with no member at all"] = refusal("r-emptyzip", base("rz") + [{
    "key": "rz-d", "class": "scored", "books": "all", "md": markdown([("Obadiah", oba9)]),
    "ref_members": []}])

# A committed but modified scorer: criteria forced true (scored on the failing p2-extra set)
scorer = os.path.join(lab.tools, "score-set.py")
src = open(scorer).read()
mod = src.replace('valid = all(validity.values())', 'valid = all(validity.values())\n    criteria = {k: True for k in criteria}')
assert mod != src
open(scorer, "w").write(mod)
git(lab.repo, "commit", "-qam", "REVIEW: a modified scorer, committed")
out_x = os.path.join(WORK, "outputs", "p2-extra")
rec_x = git(lab.repo, "log", "--format=%H", "-1", "--", "eval/p2-extra/run/RUN.txt")
rc, j, err = lab.score("p2-extra", out_x, rec_x)
R["P2 failing set scored by a committed, modified scorer"] = {"exit": rc, "official": summary(j),
                                                              "scorer_sha256": j and j["anchors"]["scorer_sha256"],
                                                              "tools_committed_and_unchanged": j and j["anchors"]["tools_committed_and_unchanged"]}
open(scorer, "w").write(src)
git(lab.repo, "commit", "-qam", "REVIEW: scorer restored")

json.dump(R, open(os.path.join(WORK, "results.json"), "w"), ensure_ascii=False, indent=1)
for k, v in R.items():
    print("==", k)
    print("  ", json.dumps(v, ensure_ascii=False)[:700])
