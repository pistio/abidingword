#!/usr/bin/env python3
"""The reviewer's end-to-end laboratory for the E3 withheld-set tools.

A scratch Git repository OUTSIDE the project receives a byte-for-byte copy of
the E3 tools (scripts/evaluation, docs/evaluation, docs/corpus). Each scenario
builds a set (documents folder outside Git, seal folder inside), commits a
SIMULATED owner revision of protocol-e3.json in the scratch repository only
(required composition = the set's, gate open; the binary pin and approved
tools are left as committed unless a scenario says otherwise), seals with
seal-set.py --write, commits the seal alone, runs run-withheld-set.sh once
with the real harness binary, and scores with score-set.py officially.

Nothing here is an approval; nothing touches the project repository.
Documents use public-domain text from the bundled engwebp/engbsb references
(read by the reviewer's own extractor, rev_usfm.py) or invented prose.
"""
import hashlib, io, json, os, shutil, subprocess, sys, zipfile

HERE = os.path.dirname(os.path.abspath(__file__))
COLUMNS = ["key", "files", "reference", "language", "script", "format", "layout", "role",
           "source and licence", "books", "sha256", "generator", "kind", "class"]
PROVENANCE = ["key", "publisher", "edition", "revision", "retrieved_from", "retrieved_utc", "licence",
              "document_sha256", "reference_source", "reference_method", "identity_checked_by"]
SOURCES = ["key", "reference", "publisher_archive", "publisher_archive_sha256", "member", "member_sha256", "role"]


def sha(b):
    return hashlib.sha256(b).hexdigest()


def git(repo, *a, check=True):
    r = subprocess.run(["git", "-C", repo, *a], capture_output=True, text=True)
    if check and r.returncode:
        raise RuntimeError(f"git {a}: {r.stderr}")
    return r.stdout.strip()


def zip_bytes(members):
    buf = io.BytesIO()
    with zipfile.ZipFile(buf, "w", zipfile.ZIP_DEFLATED) as z:
        for name, data in members:
            z.writestr(zipfile.ZipInfo(name, date_time=(2026, 11, 25, 0, 0, 0)),
                       data if isinstance(data, bytes) else data.encode("utf-8"))
    return buf.getvalue()


class Lab:
    def __init__(self, project, binary, work, version="e3"):
        self.project, self.binary, self.work = project, os.path.abspath(binary), os.path.abspath(work)
        self.version = version
        if os.path.exists(self.work):
            shutil.rmtree(self.work)
        self.repo = os.path.join(self.work, "repo")
        os.makedirs(self.repo)
        git(self.repo, "init", "-q")
        git(self.repo, "config", "user.email", "review-lab@example.invalid")
        git(self.repo, "config", "user.name", "E3 review lab (scratch)")
        for rel in ("scripts/evaluation", "docs/evaluation", "docs/corpus"):
            shutil.copytree(os.path.join(project, rel), os.path.join(self.repo, rel))
        git(self.repo, "add", "-A")
        git(self.repo, "commit", "-qm", "E3 tools, byte for byte (scratch)")
        self.binary_sha = sha(open(self.binary, "rb").read())
        self.protocol = os.path.join(self.repo, "docs", "evaluation", f"protocol-{version}.json")

    def tool(self, name):
        return os.path.join(self.repo, "scripts", "evaluation", name)

    def revise(self, composition, gate=True, edit=None, message="SIMULATED owner revision (scratch only)"):
        p = json.load(open(self.protocol, encoding="utf-8"))
        p["required_composition"] = composition
        if self.version == "e2":
            p["harness_binary_sha256"] = self.binary_sha  # E2 has no gate and no approved tools
        else:
            p["withheld_execution_permitted"] = gate
        p["status"] = "SIMULATED owner revision, review scratch repository only"
        if edit:
            edit(p)
        json.dump(p, open(self.protocol, "w", encoding="utf-8"), indent=1, ensure_ascii=False)
        git(self.repo, "add", "-A")
        if git(self.repo, "status", "--porcelain"):
            git(self.repo, "commit", "-qm", message)

    def build(self, name, docs):
        """docs: [{key, cls, kind, file: (name, bytes), fmt, reference: [(member, bytes)] | bytes | None,
        books, archive: (name, sha) , removed: [(member, sha)], lang, script}]"""
        seal = os.path.join(self.repo, "sets", name, "seal")
        folder = os.path.join(self.work, "documents", name)
        os.makedirs(seal)
        os.makedirs(folder)
        rows, prov, sums, sources = [], [], {}, []
        for d in docs:
            fname, data = d["file"]
            open(os.path.join(folder, fname), "wb").write(data)
            sums[fname] = sha(data)
            ref = "-"
            if d.get("reference") is not None:
                ref = d.get("ref_name", f"{d['key']}_usfm.zip")
                package = d["reference"] if isinstance(d["reference"], bytes) else zip_bytes(d["reference"])
                open(os.path.join(folder, ref), "wb").write(package)
                sums[ref] = sha(package)
                arch, arch_sha = d.get("archive", ("review lab package (invented or extracted)", sha(package)))
                if d.get("sources", True):
                    with zipfile.ZipFile(io.BytesIO(package)) as z:
                        for m in z.namelist():
                            sources.append([d["key"], ref, arch, arch_sha, m, sha(z.read(m)), "kept"])
                    for m, h in d.get("removed", []):
                        sources.append([d["key"], ref, arch, arch_sha, m, h, "removed"])
                    if d.get("tamper"):
                        sources[:] = d["tamper"](sources)
            rows.append([d["key"], fname, ref, d.get("lang", "en"), d.get("script", "Latin"), d.get("fmt", "md"),
                         "review generated", "review", "public domain / invented", d.get("books", "all") if ref != "-" else "-",
                         sums[fname], "review lab.py", d.get("kind", "generated"), d["cls"]])
            prov.append([d["key"], "review lab", "lab", "1", "https://example.invalid/review", "2026-11-25T00:00:00Z",
                         "public domain / invented", sums[fname], "bundled eBible / invented" if ref != "-" else "-",
                         "review extractor" if ref != "-" else "-", "E3 reviewer (lab)"])
            if d.get("allowed"):
                os.makedirs(os.path.join(seal, "allowed"), exist_ok=True)
                open(os.path.join(seal, "allowed", d["key"] + ".txt"), "w").write("\n".join(d["allowed"]) + "\n")

        def w(rel, text):
            open(os.path.join(seal, rel), "w", encoding="utf-8").write(text)
        w("manifest.tsv", "\t".join(COLUMNS) + "\n" + "".join("\t".join(r) + "\n" for r in rows))
        w("checksums.sha256", "".join(f"{h}  {n}\n" for n, h in sorted(sums.items())))
        w("provenance.tsv", "\t".join(PROVENANCE) + "\n" + "".join("\t".join(r) + "\n" for r in prov))
        w("review-ledger.md", "Review lab: invented or public-domain documents; no withheld material.\n")
        w("access-records.md", "Review lab: scratch repository only.\n")
        comp = {c: sum(r[13] == c for r in rows) for c in ("scored", "control", "mixed", "exploratory")}
        comp.update({k: sum(r[12] == k for r in rows) for k in ("authentic", "generated")})
        w("composition.tsv", "measure\tdocuments\n" + "".join(f"{k}\t{v}\n" for k, v in comp.items()))
        if self.version != "e2":
            w("reference-sources.tsv", "\t".join(SOURCES) + "\n" + "".join("\t".join(r) + "\n" for r in sources))
        return seal, folder, comp

    def seal(self, seal, folder):
        r = subprocess.run(["python3", "-I", self.tool("seal-set.py"), seal, folder, "--write"], capture_output=True, text=True)
        refusals = [l for l in r.stderr.splitlines() if l.startswith("refused")]
        if r.returncode:
            return None, refusals or r.stderr.strip().splitlines()[-5:]
        git(self.repo, "add", "--", os.path.relpath(seal, self.repo))
        git(self.repo, "commit", "-qm", f"seal {os.path.relpath(seal, self.repo)}")
        return git(self.repo, "rev-parse", "HEAD"), []

    def run(self, seal, commit, folder, name, binary=None, want=None, env=None):
        out = os.path.join(self.work, "out", name)
        b = binary or self.binary
        r = subprocess.run(["bash", self.tool("run-withheld-set.sh"), seal, commit, folder, b,
                            want or sha(open(b, "rb").read()), out], capture_output=True, text=True,
                           env=dict(os.environ, **(env or {})))
        return r.returncode, out, r.stderr.strip().splitlines()[-4:], git(self.repo, "rev-parse", "HEAD")

    def score(self, out, seal, record, name, extra=(), scorer=None):
        js = os.path.join(self.work, f"{name}.score.json")
        md = os.path.join(self.work, f"{name}.score.md")
        r = subprocess.run(["python3", "-I", scorer or self.tool("score-set.py"), out, seal, f"--run-record={record}",
                            f"--json={js}", f"--md={md}", *extra], capture_output=True, text=True)
        res = {"exit": r.returncode}
        if r.returncode or not os.path.exists(js):
            res["stderr"] = r.stderr.strip().splitlines()[-4:]
            return res
        s = json.load(open(js, encoding="utf-8"))
        a = s["assessment"]
        res.update(verdict=a["verdict"], verdict_line=a.get("verdict_line"), safety=a.get("safety_verdict"),
                   evidence=a.get("evidence_verdict"), shortfall=(a.get("evidence_floor") or {}).get("shortfall"),
                   failed_validity=sorted(k for k, v in a["run_validity"].items() if not v),
                   criteria_failed=sorted(k for k, v in a["criteria"].items() if not v),
                   integrity=a.get("integrity"), counts=a.get("counts"), qualification_limit=a.get("qualification_limit"),
                   accuracy_claim=a.get("accuracy_claim"), diagnostic=a.get("diagnostic_verdict"),
                   diagnostic_line=a.get("diagnostic_verdict_line"),
                   protocol_sha256=s.get("anchors", {}).get("protocol_sha256"))
        res["documents"] = {d["key"]: {"execution": d["execution"], **{k: (d.get("observed") or {}).get(k) for k in
                            ("printed", "verified", "verified_faithful", "verified_mismatch", "verified_absent",
                             "verified_unvalidatable", "printed_unsupported")}} for d in s["documents"]}
        return res

    def scenario(self, name, docs, protocol_edit=None, gate=True):
        seal, folder, comp = self.build(name, docs)
        self.revise(comp, gate=gate, edit=protocol_edit)
        res = {"scenario": name, "composition": comp}
        commit, refusals = self.seal(seal, folder)
        if commit is None:
            res.update(outcome="refused at the seal", refusals=refusals)
            return res, None
        code, out, err, record = self.run(seal, commit, folder, name)
        if code:
            res.update(outcome="refused by the runner", runner_exit=code, runner=err)
            return res, None
        res.update(outcome="scored", score=self.score(out, seal, record, name))
        return res, (out, seal, record, folder, commit)
