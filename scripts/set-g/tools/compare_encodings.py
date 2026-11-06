"""Check (c): verse-by-verse comparison of a reference USFM zip with a second, independent
encoding of the same edition (another USFM zip). Texts are compared after removing notes,
markup, and normalising whitespace, dash spacing and typographic quotes; an optional
--letters mode compares letters only (punctuation-insensitive).
Usage: python3 -I compare_encodings.py REF.zip OTHER.zip OUT_PREFIX [--letters|--base|--nocant] [--books A,B]
       [--strip-other REGEX]
"""
import json
import re
import sys
import unicodedata

sys.path.insert(0, "/root/aw-set-g/tools")
import usfmlib as U  # noqa: E402
from check_sequence import letters  # noqa: E402


def norm(s, mode):
    s = unicodedata.normalize("NFC", s)
    s = re.sub(r"<[^>]+>", "", s)
    if mode == "letters":
        return letters(s)
    if mode == "base":
        # letters without combining marks (e.g. Hebrew consonants only, Greek without accents)
        return "".join(c for c in unicodedata.normalize("NFD", letters(s)) if unicodedata.category(c)[0] != "M")
    if mode == "nocant":
        # Hebrew: drop cantillation marks (U+0591-U+05AF), meteg (U+05BD); keep vowel points
        s = re.sub("[\u0591-\u05af\u05bd]", "", s)
        return letters(s)
    s = s.replace("’", "'").replace("‘", "'").replace("“", '"').replace("”", '"')
    s = re.sub(r"\s*--\s*", "--", s)
    s = re.sub(r"\s*—\s*", "—", s)
    s = re.sub(r"\s+([,.;:!?»])", r"\1", s)
    return U.clean_ws(s)


def main():
    a, b, outp = sys.argv[1:4]
    mode = "text"
    books = None
    strip_other = None
    args = sys.argv[4:]
    i = 0
    while i < len(args):
        if args[i] == "--letters":
            mode = "letters"
            i += 1
        elif args[i] in ("--base", "--nocant"):
            mode = args[i][2:]
            i += 1
        elif args[i] == "--books":
            books = set(args[i + 1].split(","))
            i += 2
        elif args[i] == "--strip-other":
            strip_other = args[i + 1]
            i += 2
        else:
            raise SystemExit("bad arg")
    A = U.verse_map(U.read_usfm_zip(a))
    B = U.verse_map(U.read_usfm_zip(b))
    if books:
        A = {k: v for k, v in A.items() if k[0] in books}
        B = {k: v for k, v in B.items() if k[0] in books}
    if strip_other:
        B = {k: re.sub(strip_other, "", v) for k, v in B.items()}
    common = [k for k in A if k in B]
    onlyA = [k for k in A if k not in B]
    onlyB = [k for k in B if k not in A]
    diffs = [k for k in common if norm(A[k], mode) != norm(B[k], mode)]
    res = {"ref_verses": len(A), "other_verses": len(B), "common": len(common),
           "only_in_ref": len(onlyA), "only_in_other": len(onlyB), "text_differences": len(diffs),
           "agreement_rate_on_common": round(1 - len(diffs) / len(common), 6) if common else None,
           "mode": mode}
    json.dump(res, open(outp + ".json", "w", encoding="utf-8"), indent=1)
    with open(outp + ".diffs.tsv", "w", encoding="utf-8") as f:
        for k in onlyA:
            f.write(f"{k[0]} {k[1]}:{k[2]}\tonly_in_ref\t{A[k][:200]}\n")
        for k in onlyB:
            f.write(f"{k[0]} {k[1]}:{k[2]}\tonly_in_other\t{B[k][:200]}\n")
        for k in diffs:
            f.write(f"{k[0]} {k[1]}:{k[2]}\tdiff\tREF: {A[k]}\tOTHER: {B[k]}\n")
    print(json.dumps(res))


if __name__ == "__main__":
    main()
