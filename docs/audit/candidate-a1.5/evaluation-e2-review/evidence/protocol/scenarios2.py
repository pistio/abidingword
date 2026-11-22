#!/usr/bin/env python3
"""Independent review of A1.5-E2: vacuous passes, an empty reference, and
what the scorer's own cross-checks can and cannot detect (fake harnesses).

    scenarios2.py <pinned harness binary> <work folder>
"""
import json
import os
import stat
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from lab import Lab, markdown, usfm, plain_book, sha, git, summary  # noqa: E402

PINNED, WORK = (os.path.abspath(a) for a in sys.argv[1:3])
lab = Lab(WORK)
R = {}
STANDARD = {"scored": 4, "control": 1, "mixed": 1, "exploratory": 0, "authentic": 0, "generated": 6}
p = json.load(open(lab.protocol))
p["required_composition"] = STANDARD
p["status"] = "REVIEW simulation of an owner-approved revision (scratch repository only)"
json.dump(p, open(lab.protocol, "w"), indent=1)
git(lab.repo, "commit", "-qam", "REVIEW: simulated protocol revision with a required composition")


def scored(key, code, counts, seed, books="all"):
    ch = plain_book(code, counts, seed)
    title = {"OBA": "Obadiah", "PHM": "Philemon", "2JN": "2 John", "HAG": "Haggai", "NAM": "Nahum"}[code]
    return {"key": key, "class": "scored", "books": books, "md": markdown([(title, ch)]),
            "ref_members": [(f"41-{code}.usfm", usfm(code, ch))]}


def rest(p):
    return [{"key": f"{p}-ctl", "class": "control", "md": "# Garden minutes\n\nThe hedges were trimmed.\n"},
            {"key": f"{p}-mix", "class": "mixed", "allowed": ["RUT 1:1-3"],
             "md": "# A reflection\n\nSome prose first.\n\n" + markdown([("Ruth", plain_book("RUT", [3], 5))])}]


def run_set(name, docs, binary=PINNED, protocol_env=None, diag_protocol=None):
    lab.build(name, docs)
    ok, commit, refusals = lab.seal(name)
    res = {"sealed": ok, "seal_refusals": refusals}
    if not ok:
        return res, None, None
    env = {"AW_PROTOCOL": protocol_env} if protocol_env else None
    rc, record, out, log = lab.run(name, commit, binary, env=env)
    res["run_exit"] = rc
    if rc:
        res["run_log"] = log
        return res, None, None
    rc, j, err = lab.score(name, out, record)
    res["official"] = summary(j)
    if diag_protocol:
        rc, j, err = lab.score(name, out, record, "--diagnostic", f"--protocol={diag_protocol}")
        res["diagnostic"] = summary(j)
    return res, out, record


# 1. Vacuous: four scored documents, none verifies anything scorable.
tob = plain_book("TOB", [10], 41)
vac = [{"key": "v-blank", "class": "scored", "books": "OBA", "md": " \n",
        "ref_members": [("41-OBA.usfm", usfm("OBA", plain_book("OBA", [21], 42)))]},
       {"key": "v-tobit", "class": "scored", "books": "all", "md": markdown([("Tobit", tob)]),
        "ref_members": [("41-TOB.usfm", usfm("TOB", tob))]},
       {"key": "v-prose", "class": "scored", "books": "all",
        "md": "# Notes\n\nThis document holds prose only; its reference prints a book.\n",
        "ref_members": [("41-NAM.usfm", usfm("NAM", plain_book("NAM", [15, 13, 19], 43)))]},
       {"key": "v-htm-only", "class": "scored", "books": "all", "md": "# Notes\n\nNothing numbered here at all.\n",
        "ref_members": [("copr.htm", "<p>REVIEW: a reference archive with prose only.</p>")]}] + rest("v")
res, _, _ = run_set("q-vacuous", vac)
R["Vacuous set (blank doc, Tobit only, prose only, a reference with no USFM)"] = res

# 2. An empty reference zip, against a document that prints Obadiah.
oba = plain_book("OBA", [21], 51)
res, out, _ = run_set("q-emptyref", [
    {"key": "e-oba", "class": "scored", "books": "all", "md": markdown([("Obadiah", oba)]), "ref_members": []},
    scored("e-phm", "PHM", [25], 52), scored("e-2jn", "2JN", [13], 53), scored("e-hag", "HAG", [15, 23], 54)]
    + rest("e"))
R["Empty reference zip (document prints Obadiah)"] = res

# 3. Fake harnesses: the pinned binary, then its report edited. Pinned in a rehearsal protocol, so
#    the scorer can only judge them in --diagnostic mode; the question is what its own checks find.
def fake(name, body):
    path = os.path.join(WORK, f"fake-{name}.py")
    open(path, "w").write("#!/usr/bin/env python3\nimport json, os, subprocess, sys\n"
                          f"_proc = subprocess.run([{PINNED!r}] + sys.argv[1:])\n"
                          "key = os.environ['AW_EDITIONS']; p = os.path.join(os.environ['AW_AUDIT_OUT'], key + '.json')\n"
                          "if os.path.exists(p):\n    j = json.load(open(p))\n" + body +
                          "    json.dump(j, open(p, 'w'), ensure_ascii=False, indent=1)\nsys.exit(_proc.returncode)\n")
    os.chmod(path, os.stat(path).st_mode | stat.S_IEXEC)
    proto = os.path.join(WORK, f"protocol-{name}.json")
    q = json.load(open(lab.protocol))
    q["harness_binary_sha256"] = sha(path)
    json.dump(q, open(proto, "w"), indent=1)
    return path, proto


def wrong_word_set(prefix):
    o = plain_book("OBA", [21], 61)
    doc = [(1, [(v, t.replace("lamp", "lamb") if v == 5 else t) for v, t in o[0][1]])]  # one real error
    return [{"key": f"{prefix}-oba", "class": "scored", "books": "all", "md": markdown([("Obadiah", doc)]),
             "ref_members": [("41-OBA.usfm", usfm("OBA", o))]},
            scored(f"{prefix}-phm", "PHM", [25], 62), scored(f"{prefix}-2jn", "2JN", [13], 63),
            scored(f"{prefix}-hag", "HAG", [15, 23], 64)] + rest(prefix)


omit = """    keep = []
    for r in j['fidelity_records']:
        if r['status'] == 'verified' and r['class'] == 'mismatch':
            for t in [j['totals']] + [b for b in j['books']]:
                if t['verified_mismatch']:
                    t['verified_mismatch'] -= 1; t['verified_exact'] += 1
                    k = r['kind']; t['verified_mismatch_kinds'][k] -= 1
                    if not t['verified_mismatch_kinds'][k]: del t['verified_mismatch_kinds'][k]
            continue
        keep.append(r)
    j['fidelity_records'] = keep
"""
path, proto = fake("omit", omit)
res, _, _ = run_set("q-real-error", wrong_word_set("w"), diag_protocol=None)
R["Control: the same kind of set with one real wrong word, pinned binary"] = res
res, _, _ = run_set("q-fake-omit", wrong_word_set("f"), binary=path, protocol_env=proto, diag_protocol=proto)
R["Fake harness that omits its one mismatch record and counts the verse exact"] = res

substitute = """    for r in j['fidelity_records']:
        if r['status'] == 'verified' and r['class'] == 'mismatch':
            r['edition'] = r['found']
"""
path, proto = fake("edition", substitute)
res, _, _ = run_set("q-fake-edition", wrong_word_set("g"), binary=path, protocol_env=proto, diag_protocol=proto)
R["Fake harness that writes the found text as the edition text (record kept)"] = res

drop_unsupported = """    inv = j['reference_inventory']
    c = inv['unsupported']['verse_zero']
    if c['references']:
        c['references'].pop(); c['printed'] -= 1; inv['printed_unsupported'] -= 1; inv['labels'] -= 1
"""
path, proto = fake("drop0", drop_unsupported)
o = plain_book("OBA", [21], 71)
ref0 = [(1, [("0", "A verse zero, invented.")] + o[0][1])]
res, _, _ = run_set("q-fake-drop0", [{"key": "d-oba", "class": "scored", "books": "all",
                                       "md": markdown([("Obadiah", o)]),
                                       "ref_members": [("41-OBA.usfm", usfm("OBA", ref0))]},
                                      scored("d-phm", "PHM", [25], 72), scored("d-2jn", "2JN", [13], 73),
                                      scored("d-hag", "HAG", [15, 23], 74)] + rest("d"),
                    binary=path, protocol_env=proto, diag_protocol=proto)
R["Fake harness that drops a verse-zero label from its inventory"] = res

hide_absent = """    absent = set(a['reference'] for a in j['verified_absent'])
    j['verified_absent'] = []
    j['verified_references'] = [r for r in j['verified_references'] if r not in absent]
"""
path, proto = fake("hideabsent", hide_absent)
jol = plain_book("JOL", [20, 32, 21], 81)
res, _, _ = run_set("q-fake-hideabsent", [{"key": "h-jol", "class": "scored", "books": "all",
                                            "md": markdown([("Joel", jol)]),
                                            "ref_members": [("41-JOL.usfm", usfm("JOL", jol[:2]))]},
                                           scored("h-phm", "PHM", [25], 82), scored("h-2jn", "2JN", [13], 83),
                                           scored("h-hag", "HAG", [15, 23], 84)] + rest("h"),
                    binary=path, protocol_env=proto, diag_protocol=proto)
R["Fake harness that hides a whole extra chapter (drops it from verified_references and verified_absent)"] = res

json.dump(R, open(os.path.join(WORK, "results.json"), "w"), ensure_ascii=False, indent=1)
for k, v in R.items():
    print("==", k)
    print("  ", json.dumps(v, ensure_ascii=False)[:900])
