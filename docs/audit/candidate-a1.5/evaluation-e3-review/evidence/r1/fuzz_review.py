#!/usr/bin/env python3
"""The reviewer's own fuzzing of the E3 readers (a generator written for this
review, not the author's mutation fuzzer). Random USFM books are built from a
grammar of well-formed and malformed fragments (notes, spans, milestones,
attributes, numbering, white space, scripts). Each is read by:

  * the E3 Python reader (scripts/evaluation/usfm_text.py),
  * the E3 Rust reader (the reviewer's probe, usfm_text.rs included read-only),
  * the reviewer's extractor (rev_usfm.py), as an independent oracle.

Reported: Rust/Python differences; inputs E3 reads cleanly whose verse count
is not the source's \\v count (silent loss); inputs E3 reads cleanly where the
reviewer's extractor reads other text, or finds trouble (a possible missed
refusal); each with an example.

    fuzz_review.py <repo> <probe> <count> <seed> <out.json>
"""
import importlib.util, json, os, random, re, subprocess, sys

repo, probe, count, seed, out = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4]), sys.argv[5]
here = os.path.dirname(os.path.abspath(__file__))


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m


py = load("usfm_text", os.path.join(repo, "scripts", "evaluation", "usfm_text.py"))
rev = load("rev_usfm", os.path.join(here, "rev_usfm.py"))
rng = random.Random(seed)
WORDS = ["word", "heel.", "\u201cGo\u201d", "(c\u2019est", "Gro\u0308\u00dfe", "\ud0dc\ucd08", "\u1f00\u03c1\u03c7\u1fc7",
         "\u05d1\u05bc\u05b0\u05e8\u05b5\u05d0\u05e9\u05b4\u0591\u05d9\u05ea\u200f", "\u0645\u06cc\u200c\u062e\u0648\u0627\u0647\u0645",
         "\u592a\u521d", "a\u00a0b", "x~y", "p//q", ",", ";", "\u201d", ")", "\u2014", "Lord\u2019s", "\U0001D400"]
WS = [" ", " ", " ", "\n", "  ", "\t", " \n"]


def words(n):
    return "".join(rng.choice(WORDS) + rng.choice(WS) for _ in range(n))


def note(ok=True):
    kind = rng.choice(["f", "x", "fe", "ef", "ex"])
    caller = rng.choice(["+", "-", "a"])
    parts = rng.choice([f"\\fr 1:{rng.randint(1,9)} \\ft {words(2)}", f"\\xo 1:1 \\xt Ruth 1:{rng.randint(1,9)}",
                        f"\\fq {words(1)}\\fqa {words(1)}", f"\\ft see \\+it {words(1)}\\+it*", f"\\ft a|b"])
    end = f"\\{kind}*" if ok else rng.choice(["", f"\\{rng.choice(['f', 'x', 'fe'])}*", "\\v 9 "])
    return f"\\{kind} {caller} {parts}{end}"


def span(ok=True, depth=0):
    name = rng.choice(["add", "nd", "wj", "w", "k", "qs", "it", "bd", "tl", "sc"])
    inner = words(rng.randint(1, 3))
    if name == "w" and rng.random() < 0.7:
        inner = inner.rstrip() + rng.choice(['|lemma="x" strong="H1"', "|grace", '|x-occurrence="1"'])
    if depth < 2 and rng.random() < 0.3:
        inner += span(rng.random() < 0.9, depth + 1).replace(f"\\", "\\+", 1).replace("\\", "\\+", 0) if False else ""
        sub = rng.choice(["nd", "add", "w"])
        inner += f"\\+{sub} {words(1).rstrip()}\\+{sub}*" if ok else f"\\{sub} {words(1)}"
    end = f"\\{name}*" if ok else rng.choice(["", "\\nd*" if name != "nd" else "\\add*"])
    return f"\\{name} {inner}{end}"


def fragment():
    r = rng.random()
    if r < 0.30:
        return words(rng.randint(1, 4))
    if r < 0.42:
        return note(rng.random() < 0.85)
    if r < 0.55:
        return span(rng.random() < 0.85)
    if r < 0.60:
        return rng.choice(['\\qt-s |who="A"\\*', "\\qt-e\\*", "\\ts\\*", '\\zaln-s |x-strong="H1"\\*', "\\zaln-e\\*",
                           '\\qt-s |who="A"', "\\*"])
    if r < 0.66:
        return rng.choice(["\n\\q1 ", "\n\\q2 ", "\n\\p\n", "\n\\m ", "\n\\b\n", "\n\\pi1 ", "\n\\li1 ", "\n\\nb "])
    if r < 0.70:
        return rng.choice(["\n\\s1 Heading here\n", "\n\\d Title words\n", "\n\\r (Ruth 1:1)\n", "\n\\ms Part\n",
                           "\n\\sp Speaker\n", "\n\\qa Aleph\n"])
    if r < 0.74:
        return rng.choice(["\\va 4\\va* ", "\\vp 1a\\vp* ", "\\va 4 ", "\\rq Isa 1:1\\rq*", '\\fig cap|src="a.jpg"\\fig*',
                           "\\fig cap"])
    if r < 0.78:
        return rng.choice(["|", "\\", "\\foo ", "\\zx ", "\\esb ", "\\esbe ", "\u0007", "\\+v 3 ", "\\v ", "\\c ",
                           "\\p\\*", "\\c \\* ", "\\v\\* ", "\\tr \\tc1 cell \\tc2 cell"])
    return f"\n\\v {rng.choice(['1', '2', '3', '2-3', '0', '01', '4a', str(rng.randint(1, 30))])} "


def book():
    body = ["\\id RUT fuzz, invented\n\\h Fuzz\n"]
    for c in range(1, rng.randint(1, 3) + 1):
        body.append(rng.choice([f"\\c {c}\n\\p\n", f"\\c {c}\n\\ca {c + 1}\\ca*\n\\p\n", f"\\c {c}\n\\s1 Head\n\\p\n"]))
        for v in range(1, rng.randint(1, 6) + 1):
            body.append(f"\\v {v} ")
            for _ in range(rng.randint(0, 5)):
                body.append(fragment())
            body.append(rng.choice(["\n", " ", ""]))
    return "".join(body)


inputs = [book() for _ in range(count)]
res = {"count": count, "seed": seed, "rust_python_identical": 0, "e3_clean": 0, "e3_refused": 0,
       "e3_clean_silent_loss": 0, "e3_clean_reviewer_same": 0, "e3_clean_reviewer_differs": 0,
       "e3_clean_reviewer_trouble": 0, "examples": {"rust_python": [], "silent_loss": [], "reviewer_differs": [],
                                                    "reviewer_trouble": []}}
B = 2000
for k in range(0, count, B):
    batch = inputs[k:k + B]
    rs = json.loads(subprocess.run([probe], input=json.dumps([{"name": str(k + i), "text": t} for i, t in enumerate(batch)]),
                                   capture_output=True, text=True, check=True).stdout)
    for i, t in enumerate(batch):
        p = py.read_text(t)
        r = rs[str(k + i)]
        pv = [[v["chapter"], v["label"], v["text"], v["line"]] for v in p["verses"]]
        rv = [x[:4] for x in r["verses"]]
        pp, rp = sorted(x["code"] for x in p["problems"]), sorted(r["problems"])
        if pv == rv and pp == rp:
            res["rust_python_identical"] += 1
        elif len(res["examples"]["rust_python"]) < 10:
            res["examples"]["rust_python"].append({"usfm": t, "python": [pv, pp], "rust": [rv, rp]})
        if pp:
            res["e3_refused"] += 1
            continue
        res["e3_clean"] += 1
        raw = len(re.findall(r"\\v(?![A-Za-z0-9*\-])", t))
        if raw != len(pv) + p["verses_before_chapter"]:
            res["e3_clean_silent_loss"] += 1
            if len(res["examples"]["silent_loss"]) < 10:
                res["examples"]["silent_loss"].append({"usfm": t, "python": pv})
        a = rev.read(t)
        if a["trouble"]:
            res["e3_clean_reviewer_trouble"] += 1
            kinds = res.setdefault("reviewer_trouble_kinds", {})
            for kind in sorted({re.sub(r"\\[a-z0-9]+ written", "\\<marker> written", x.split(":")[0]) for x in a["trouble"]}):
                kinds[kind] = kinds.get(kind, 0) + 1
            if len(res["examples"]["reviewer_trouble"]) < 25:
                res["examples"]["reviewer_trouble"].append({"usfm": t, "trouble": a["trouble"], "e3": [x[:3] for x in pv]})
        elif [x[:3] for x in pv] == a["verses"]:
            res["e3_clean_reviewer_same"] += 1
        else:
            res["e3_clean_reviewer_differs"] += 1
            if len(res["examples"]["reviewer_differs"]) < 25:
                res["examples"]["reviewer_differs"].append({"usfm": t, "reviewer": a["verses"], "e3": [x[:3] for x in pv]})
json.dump(res, open(out, "w", encoding="utf-8"), ensure_ascii=False, indent=1)
print(json.dumps({k: v for k, v in res.items() if k != "examples"}, indent=1))
