#!/usr/bin/env python3
"""Both E3 readers on thousands of mutated USFM texts (evaluation candidate
A1.5-E3 evidence: neither reader can silently omit a verse, and they agree).

Each input is a seed (the shared vectors, and chapters of real bundled
references) with random mutations: a marker or its end removed, duplicated
or moved, a backslash, bar, asterisk or newline inserted or deleted, the
text truncated. Both readers read every input; for each one:

1. **No silent loss.** A third count of `\\v` markers, made here with a
   regular expression of its own, must equal the verses a reader read plus
   its verses before any chapter, unless that reader reports a problem.
2. **Agreement.** The Rust and Python readers give the same verses
   (chapter, label, text, line), the same verses before any chapter and the
   same set of problem codes.

    fuzz_readers.py <cargo test binary of evaluation_references> <out.json> [count]
"""
import glob
import json
import os
import random
import re
import subprocess
import sys
import tempfile
import zipfile

ROOT = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), *[".."] * 6))
sys.path.insert(0, os.path.join(ROOT, "scripts", "evaluation"))
import usfm_text  # noqa: E402

THIRD = re.compile(r"\\v(?![A-Za-z0-9*\-])")
PIECES = ["\\f + ", "\\f*", "\\x - ", "\\x*", "\\ft ", "\\wj ", "\\wj*", "\\+w ", "\\+w*", "\\w ", "\\w*", "|",
          "\\va ", "\\va*", "\\fig ", "\\fig*", "\\qt-s |who=\"x\"\\*", "\\qt-e\\*", "\\*", "\\", "\\v ", "\\v 9 ",
          "\\c ", "\\c 7\n", "\\p\n", "\\q1 ", "\\s1 Heading\n", "\\esb\n", "\\esbe\n", "~", "//", "\n", "\t",
          "\\add ", "\\add*", "\\nd*", "\\zz ", "\\qqq ", "\\id XXX\n", "\u00a0", "\u0001"]


def seeds():
    out = [v["usfm"] for v in json.load(open(os.path.join(ROOT, "tests/evaluation/usfm-text-vectors.json")))["vectors"]]
    for path in sorted(glob.glob(os.path.join(ROOT, "resources/scripture/*/*_usfm.zip"))):
        with zipfile.ZipFile(path) as z:
            for name in z.namelist()[:6]:
                if name.lower().endswith((".usfm", ".sfm")):
                    text = z.read(name).decode("utf-8")
                    out.append(text[:6000])  # the head of a real book: \id, titles, notes, chapter 1
    return out


def mutate(rng, text):
    for _ in range(rng.randint(1, 4)):
        op = rng.random()
        if op < 0.35 or not text:
            k = rng.randint(0, len(text))
            text = text[:k] + rng.choice(PIECES) + text[k:]
        elif op < 0.65:
            markers = [m.start() for m in re.finditer(r"\\", text)]
            if markers:
                k = rng.choice(markers)
                end = k + 1
                while end < len(text) and (text[end].isalnum() or text[end] in "+*-"):
                    end += 1
                text = text[:k] + text[end:]
        elif op < 0.85:
            a, b = sorted(rng.randint(0, len(text)) for _ in range(2))
            text = text[:a] + text[b:b + (b - a)] + text[a:]
        else:
            text = text[:rng.randint(0, len(text))]
    return text


def main():
    if len(sys.argv) not in (3, 4):
        sys.exit(__doc__)
    binary, out_json = sys.argv[1], sys.argv[2]
    count = int(sys.argv[3]) if len(sys.argv) == 4 else 5000
    rng = random.Random(20261122)
    base = seeds()
    inputs = base + [mutate(rng, rng.choice(base)) for _ in range(count)]
    with tempfile.TemporaryDirectory() as tmp:
        src, dst = os.path.join(tmp, "in.json"), os.path.join(tmp, "out.json")
        json.dump(inputs, open(src, "w", encoding="utf-8"), ensure_ascii=False)
        subprocess.run([binary, "--ignored", "--exact", "read_texts_for_cross_checking"], check=True,
                       env=dict(os.environ, AW_EVAL_TEXT_INPUT=src, AW_EVAL_TEXT_OUTPUT=dst),
                       capture_output=True)
        rust = json.load(open(dst, encoding="utf-8"))
    summary = {"inputs": len(inputs), "seeds": len(base), "mutated": count, "agree": 0, "disagree": 0,
               "with_problems": 0, "clean": 0, "silent_loss_rust": 0, "silent_loss_python": 0, "examples": []}
    for text, r in zip(inputs, rust):
        p = usfm_text.read_text(text)
        py = {"book": p["book"], "verses": [[v["chapter"], v["label"], v["text"], v["line"]] for v in p["verses"]],
              "before_chapter": p["verses_before_chapter"], "verse_markers": p["verse_markers"],
              "problems": sorted({x["code"] for x in p["problems"]})}
        # \v markers as this script counts them, in the text as both readers normalise it.
        third = len(THIRD.findall(text.lstrip("\ufeff").replace("\r\n", "\n").replace("\r", "\n")))
        for name, x in (("rust", r), ("python", py)):
            if not x["problems"] and len(x["verses"]) + x["before_chapter"] != third:
                summary[f"silent_loss_{name}"] += 1
        summary["with_problems" if py["problems"] else "clean"] += 1
        if py == r:
            summary["agree"] += 1
        else:
            summary["disagree"] += 1
            if len(summary["examples"]) < 10:
                summary["examples"].append({"input": text[:400], "rust": r, "python": py})
    json.dump(summary, open(out_json, "w", encoding="utf-8"), ensure_ascii=False, indent=1)
    print({k: v for k, v in summary.items() if k != "examples"})


if __name__ == "__main__":
    main()
