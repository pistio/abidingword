"""Conversion check (b), word-for-word sample: for a deterministic sample of verses (first
and last verse of every book, first and last verse of up to 12 chapters per book, plus
evenly spaced verses up to at least MIN), test that the reference verse text (notes
removed, whitespace normalised, all punctuation kept) occurs verbatim in the document text
(whitespace normalised; optional regex removed first, e.g. note markers or soft hyphens).
Writes OUT.tsv with one line per sampled verse: ref, ok/FAIL, verse text.
Usage: python3 -I check_sample_verbatim.py REF.zip DOC.txt OUT.tsv [--min N] [--strip REGEX]
       [--rtl-lines] [--join-hyphens]
"""
import re
import sys
import unicodedata

sys.path.insert(0, "/root/aw-set-g/tools")
import usfmlib as U  # noqa: E402


def norm(s):
    s = unicodedata.normalize("NFC", s).replace(" ", " ").replace("­", "")
    return re.sub(r"\s+", " ", s).strip()


def main():
    ref, doc, out = sys.argv[1:4]
    args = sys.argv[4:]
    mn, strip, rtl, joinh = 40, None, False, False
    i = 0
    while i < len(args):
        if args[i] == "--min":
            mn = int(args[i + 1]); i += 2
        elif args[i] == "--strip":
            strip = args[i + 1]; i += 2
        elif args[i] == "--rtl-lines":
            rtl = True; i += 1
        elif args[i] == "--join-hyphens":
            joinh = True; i += 1
        else:
            raise SystemExit("bad arg")
    raw = open(doc, encoding="utf-8").read()
    if rtl:
        raw = "\n".join(" ".join(reversed(re.sub("[‪-‮‎‏]", "", l).split())) for l in raw.split("\n"))
    if joinh:
        raw = re.sub(r"[‐-](\n)", "", raw)
    if strip:
        raw = re.sub(strip, "", raw)
    D = norm(raw)
    books = U.read_usfm_zip(ref)
    allv = []
    sample = set()
    for b in books:
        chs = {}
        ch = 0
        bv = []
        for it in b["items"]:
            if it[0] == "c":
                ch = it[1]
            elif it[0] == "v":
                k = (b["code"], ch, it[1], norm(re.sub(r"\x01\d+\x01", "", it[2])))
                if not k[3]:
                    continue
                bv.append(k)
                chs.setdefault(ch, []).append(k)
        if not bv:
            continue
        allv += bv
        sample.add(bv[0])
        sample.add(bv[-1])
        keys = sorted(chs)
        step = max(1, len(keys) // 12)
        for c in keys[::step][:12]:
            sample.add(chs[c][0])
            sample.add(chs[c][-1])
    if len(sample) < mn:
        step = max(1, len(allv) // (mn - len(sample) + 1))
        for k in allv[::step]:
            sample.add(k)
    order = {k: i for i, k in enumerate(allv)}
    ok = 0
    with open(out, "w", encoding="utf-8") as f:
        for k in sorted(sample, key=lambda k: order[k]):
            hit = k[3] in D
            ok += hit
            f.write(f"{k[0]} {k[1]}:{k[2]}\t{'ok' if hit else 'FAIL'}\t{k[3]}\n")
    print(f"{ok}/{len(sample)} sampled verses verbatim")


if __name__ == "__main__":
    main()
