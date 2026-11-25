#!/usr/bin/env python3
"""Integrity probes against the E3 tools with the real binary (lab.py).

    probes.py <project> <binary> <other-path binary> <work> <out.json>
"""
import hashlib, json, os, shutil, subprocess, sys

project, binary, other, work, out = sys.argv[1:6]
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import lab as L
import scenarios_lib as S

R = {}
lab = L.Lab(project, binary, work)


def run_scenario(name, docs, **kw):
    r, ctx = lab.scenario(name, docs, **kw)
    return r, ctx


clean, cctx = run_scenario("clean", [S.doc("pc-rut", "scored", S.markdown("RUT"), ["RUT"], "RUT")])
wrong, wctx = run_scenario("wrong", [S.doc("pw-rut", "scored", S.markdown("RUT").replace(b"Naomi", b"Naomy", 1), ["RUT"], "RUT")])
R["baseline"] = {"clean": clean["score"]["verdict"], "wrong": wrong["score"]["verdict"]}


def cli(argv):
    r = subprocess.run(["python3", "-I", *argv], capture_output=True, text=True)
    return {"exit": r.returncode, "stderr": r.stderr.strip().splitlines()[:1]}


# 1. Unknown and repeated options, every tool.
out_c, seal_c, rec_c, folder_c, commit_c = cctx
sc, ck, sl = lab.tool("score-set.py"), lab.tool("check-set.py"), lab.tool("seal-set.py")
R["options"] = {
    "score --Protocol=x": cli([sc, out_c, seal_c, f"--run-record={rec_c}", "--Protocol=/x.json"]),
    "score --json twice": cli([sc, out_c, seal_c, f"--run-record={rec_c}", "--json=/tmp/a.json", "--json=/tmp/b.json"]),
    "score --run-record twice": cli([sc, out_c, seal_c, f"--run-record={rec_c}", f"--run-record={rec_c}"]),
    "score --diagnostic=yes": cli([sc, out_c, seal_c, f"--run-record={rec_c}", "--diagnostic=yes"]),
    "score --protocol without --diagnostic": cli([sc, out_c, seal_c, f"--run-record={rec_c}", f"--protocol={lab.protocol}"]),
    "score --json (no value)": cli([sc, out_c, seal_c, f"--run-record={rec_c}", "--json"]),
    "score extra positional": cli([sc, out_c, seal_c, "extra", f"--run-record={rec_c}"]),
    "score bare --": cli([sc, out_c, seal_c, f"--run-record={rec_c}", "--"]),
    "check --Inventory=x": cli([ck, seal_c, folder_c, "--Inventory=/x"]),
    "check --report twice": cli([ck, seal_c, folder_c, "--report=/tmp/a", "--report=/tmp/b"]),
    "seal --write twice": cli([sl, seal_c, folder_c, "--write", "--write"]),
    "seal --writ": cli([sl, seal_c, folder_c, "--writ"]),
    "seal --protocol=x --protocol=y": cli([sl, seal_c, folder_c, "--protocol=/x", "--protocol=/y"]),
}
# 2. Outputs changed after the run; run record recommitted.
p = os.path.join(out_c, "pc-rut.json")
saved = open(p, "rb").read()
open(p, "ab").write(b" ")
R["output_changed_after_run"] = lab.score(out_c, seal_c, rec_c, "p-outputs")
open(p, "wb").write(saved)
# 3. Scorer under another Python (Unicode 15.0): must not be a protocol run.
R["scorer_python312"] = lab.score(out_c, seal_c, rec_c, "p-py312", scorer=None) if not shutil.which("python3.12") else None
if shutil.which("python3.12"):
    js = os.path.join(lab.work, "p-py312.json")
    r = subprocess.run(["python3.12", "-I", sc, out_c, seal_c, f"--run-record={rec_c}", f"--json={js}"], capture_output=True, text=True)
    a = json.load(open(js))["assessment"] if os.path.exists(js) else {}
    R["scorer_python312"] = {"exit": r.returncode, "verdict": a.get("verdict"),
                             "failed_validity": sorted(k for k, v in a.get("run_validity", {}).items() if not v)}
# 4. A modified scorer committed (criterion 1 forced), on the failing run.
out_w, seal_w, rec_w, folder_w, commit_w = wctx
text = open(sc, encoding="utf-8").read()
forced = text.replace('"1_no_verified_text_mismatch": scored.get("verified_mismatch", 0) == 0,',
                      '"1_no_verified_text_mismatch": True,  # MODIFIED (review probe)')
assert forced != text
open(sc, "w", encoding="utf-8").write(forced)
L.git(lab.repo, "commit", "-qam", "review probe: a modified scorer, committed")
R["modified_scorer_committed"] = lab.score(out_w, seal_w, rec_w, "p-modscorer")
# 5. ... and the protocol's approved_tools rewritten to the modified scorer, committed with it.
lab.revise(json.load(open(lab.protocol))["required_composition"],
           edit=lambda p: p["approved_tools"].__setitem__("scripts/evaluation/score-set.py", L.sha(open(sc, "rb").read())),
           message="review probe: the protocol rewritten to approve the modified scorer")
R["modified_scorer_and_protocol_committed"] = lab.score(out_w, seal_w, rec_w, "p-modboth")
R["modified_scorer_and_protocol_committed"]["note"] = ("the run was made under the earlier protocol file, so "
                                                        "run_under_this_protocol is expected to fail too")
open(sc, "w", encoding="utf-8").write(text)
L.git(lab.repo, "commit", "-qam", "review probe: scorer restored")
lab.revise(json.load(open(lab.protocol))["required_composition"],
           edit=lambda p: p["approved_tools"].__setitem__("scripts/evaluation/score-set.py", L.sha(open(sc, "rb").read())),
           message="review probe: protocol restored")
# 6. Modified scorer AND protocol committed BEFORE the run: a whole consistent substitute toolset.
forced_pass = text.replace('"1_no_verified_text_mismatch": scored.get("verified_mismatch", 0) == 0,',
                           '"1_no_verified_text_mismatch": True,  # MODIFIED (review probe)')
open(sc, "w", encoding="utf-8").write(forced_pass)
L.git(lab.repo, "commit", "-qam", "review probe: substitute scorer")
lab.revise(None, edit=lambda p: p["approved_tools"].__setitem__("scripts/evaluation/score-set.py", L.sha(open(sc, "rb").read())),
           message="review probe: substitute protocol approving it")
r6, ctx6 = run_scenario("substitute", [S.doc("ps-rut", "scored", S.markdown("RUT").replace(b"Naomi", b"Naomy", 1), ["RUT"], "RUT")])
R["substitute_scorer_and_protocol_before_run"] = r6
R["substitute_scorer_and_protocol_before_run"]["committed_protocol_sha256"] = L.sha(open(os.path.join(project, "docs/evaluation/protocol-e3.json"), "rb").read())
open(sc, "w", encoding="utf-8").write(text)
L.git(lab.repo, "commit", "-qam", "review probe: scorer restored")
lab.revise(None, edit=lambda p: p["approved_tools"].__setitem__("scripts/evaluation/score-set.py", L.sha(open(sc, "rb").read())),
           message="review probe: protocol restored")
# 7. A modified text reader committed before the run (approved list unchanged).
ut = lab.tool("usfm_text.py")
ut_text = open(ut, encoding="utf-8").read()
open(ut, "w", encoding="utf-8").write(ut_text.replace('RULES = "aw-usfm-text-1"', 'RULES = "aw-usfm-text-1"  # modified copy'))
L.git(lab.repo, "commit", "-qam", "review probe: modified usfm_text.py")
r7, _ = run_scenario("modreader", [S.doc("pm-rut", "scored", S.markdown("RUT"), ["RUT"], "RUT")])
R["modified_reader_committed"] = r7
open(ut, "w", encoding="utf-8").write(ut_text)
L.git(lab.repo, "commit", "-qam", "review probe: reader restored")
# 8. The gate: committed protocol shut, the runner given AW_PROTOCOL pointing at an open copy.
lab.revise(None, gate=False, message="review probe: gate shut")
seal8, folder8, comp8 = lab.build("gatebypass", [S.doc("pg-rut", "scored", S.markdown("RUT"), ["RUT"], "RUT")])
lab.revise(comp8, gate=False, message="review probe: composition, gate still shut")
c8, refusals8 = lab.seal(seal8, folder8)
open_copy = os.path.join(lab.work, "open-protocol.json")
pj = json.load(open(lab.protocol))
pj["withheld_execution_permitted"] = True
json.dump(pj, open(open_copy, "w"), indent=1)
code_a, _, err_a, _ = lab.run(seal8, c8, folder8, "gate-a")
code_b, out_b, err_b, rec_b = lab.run(seal8, c8, folder8, "gate-b", env={"AW_PROTOCOL": open_copy})
R["gate"] = {"committed_shut": {"runner_exit": code_a, "stderr": err_a},
             "AW_PROTOCOL_open_copy": {"runner_exit": code_b, "stderr": err_b,
                                       "documents_were_run": os.path.exists(os.path.join(out_b, "pg-rut.json")),
                                       "score": lab.score(out_b, seal8, rec_b, "p-gate") if code_b == 0 else None}}
lab.revise(comp8, gate=True, message="review probe: gate open again")
# 9. Another binary (the same source built at another path) given to the runner.
seal9, folder9, comp9 = lab.build("otherbin", [S.doc("po-rut", "scored", S.markdown("RUT"), ["RUT"], "RUT")])
lab.revise(comp9)
c9, _ = lab.seal(seal9, folder9)
code9, _, err9, _ = lab.run(seal9, c9, folder9, "other-a", binary=other)
code9b, _, err9b, _ = lab.run(seal9, c9, folder9, "other-b", binary=other, want=lab.binary_sha)
pin_other = os.path.join(lab.work, "pin-other.json")
pj = json.load(open(lab.protocol))
pj["harness_binary_sha256"] = L.sha(open(other, "rb").read())
json.dump(pj, open(pin_other, "w"), indent=1)
code9c, out9c, err9c, rec9c = lab.run(seal9, c9, folder9, "other-c", binary=other, env={"AW_PROTOCOL": pin_other})
R["other_binary"] = {"own_hash": {"exit": code9, "stderr": err9}, "pinned_hash_claimed": {"exit": code9b, "stderr": err9b},
                     "AW_PROTOCOL_pinning_it": {"exit": code9c, "stderr": err9c,
                                                "score": lab.score(out9c, seal9, rec9c, "p-other") if code9c == 0 else None}}
# 10. A fake harness: the real binary, then one verse text digest of its reference inventory changed.
fake = os.path.join(lab.work, "fake-harness.sh")
open(fake, "w").write(f"""#!/usr/bin/env bash
"{binary}" "$@"; status=$?
for f in "$AW_AUDIT_OUT"/*.json; do
  case "$f" in *.control.json|*/CHECK.json) continue;; esac
  python3 -I -c '
import json,sys
p=sys.argv[1]; j=json.load(open(p))
inv=j.get("reference_inventory") or {{}}
v=inv.get("verses")
if v:
    v[3][6]="0"*64
    json.dump(j,open(p,"w"))
' "$f"
done
exit $status
""")
os.chmod(fake, 0o755)
seal10, folder10, comp10 = lab.build("fake", [S.doc("pf-rut", "scored", S.markdown("RUT"), ["RUT"], "RUT")])
lab.revise(comp10, edit=lambda p: p.__setitem__("harness_binary_sha256", L.sha(open(fake, "rb").read())),
           message="review probe: SIMULATED revision pinning a fake harness")
c10, _ = lab.seal(seal10, folder10)
code10, out10, err10, rec10 = lab.run(seal10, c10, folder10, "fake", binary=fake)
R["fake_harness_changed_verse_digest"] = {"runner_exit": code10, "stderr": err10,
                                          "score": lab.score(out10, seal10, rec10, "p-fake") if code10 == 0 else None}
lab.revise(comp10, edit=lambda p: p.__setitem__("harness_binary_sha256", lab.binary_sha), message="pin restored")
# 11. A Unicode 16 code point inside a document's verse (the reference without it).
doc11 = S.markdown("RUT").replace(b"Naomi", "Naomi\U00016D40".encode(), 1)
r11, _ = run_scenario("unicode16-doc", [S.doc("pu-rut", "scored", doc11, ["RUT"], "RUT")])
R["unicode16_in_document"] = r11
json.dump(R, open(out, "w", encoding="utf-8"), ensure_ascii=False, indent=1)
print(json.dumps(R, ensure_ascii=False, indent=1)[:200])
