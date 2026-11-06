r"""Build the G03 reference from the publisher's own TeX sources of vermiculus/bible
(BibleTeX project, KJV text from Project Gutenberg; bible.pdf is the real file).

Rules taken from bible.cls / bible.tex:
  \InputBook{name = ..., toc-name = ...}{dir/file}  (uncommented lines of bible.tex, in order)
  \Chapter[name = X]  -> \c n (counter) and \s1 X (printed centred, bold)
  \Verse              -> \v n (counter)
  \LORD               -> LORD (printed in small capitals)
  \Father{..} etc. (speakers declared with \NewSpeaker) -> “..” (\enquote, csquotes);
  starred form -> ‘..’ (\enquote*); the speaker colour is layout only.
Usage: python3 -I conv_vermiculus_kjv_tex.py REPO_DIR OUT.zip REPORT
"""
import os
import re
import sys

sys.path.insert(0, "/root/aw-set-g/tools")
import usfmlib as U  # noqa: E402

FILE2CODE = {
    "genesis": "GEN", "exodus": "EXO", "leviticus": "LEV", "numbers": "NUM", "dueteronomy": "DEU",
    "joshua": "JOS", "judges": "JDG", "ruth": "RUT", "samuel1": "1SA", "samuel2": "2SA",
    "kings1": "1KI", "kings2": "2KI", "chronicles1": "1CH", "chronicles2": "2CH", "ezra": "EZR",
    "nehemiah": "NEH", "esther": "EST", "job": "JOB", "psalms": "PSA", "proverbs": "PRO",
    "ecclesiastes": "ECC", "songofsolomon": "SNG", "isaiah": "ISA", "jeremiah": "JER",
    "lamentations": "LAM", "ezekiel": "EZK", "daniel": "DAN", "hosea": "HOS", "joel": "JOL",
    "amos": "AMO", "obadiah": "OBA", "jonah": "JON", "micah": "MIC", "nahum": "NAM",
    "habakkuk": "HAB", "zephaniah": "ZEP", "haggai": "HAG", "zechariah": "ZEC", "malachi": "MAL",
    "matthew": "MAT", "mark": "MRK", "luke": "LUK", "john": "JHN", "acts": "ACT", "romans": "ROM",
    "corinthians1": "1CO", "corinthians2": "2CO", "galatians": "GAL", "ephesians": "EPH",
    "philippians": "PHP", "colossians": "COL", "thessalonians1": "1TH", "thessalonians2": "2TH",
    "timothy1": "1TI", "timothy2": "2TI", "titus": "TIT", "philemon": "PHM", "hebrews": "HEB",
    "james": "JAS", "peter1": "1PE", "peter2": "2PE", "john1": "1JN", "john2": "2JN",
    "john3": "3JN", "jude": "JUD", "revelations": "REV"}


def strip_comments(t):
    return "\n".join(re.sub(r"(?<!\\)%.*$", "", l) for l in t.split("\n"))


def resolve_speakers(s, speakers):
    """Replace \\Sp{..} by \u201c..\u201d and \\Sp*{..} by \u2018..\u2019, with brace matching
    (a speech may run over several verses and contain nested speeches)."""
    pat = re.compile(r"\\(" + "|".join(speakers) + r")(\*?)\{")
    while True:
        m = pat.search(s)
        if not m:
            return s
        depth, i = 1, m.end()
        while depth:
            if s[i] == "{":
                depth += 1
            elif s[i] == "}":
                depth -= 1
            i += 1
        inner = s[m.end():i - 1]
        oq, cq = ("\u2018", "\u2019") if m.group(2) else ("\u201c", "\u201d")
        s = s[:m.start()] + oq + inner + cq + s[i:]


def clean(s, speakers, rep, where):
    s = s.replace("\\LORD", "LORD")
    s = s.replace("~", " ")
    if "\\" in s or "{" in s or "}" in s:
        rep.append(f"{where}: leftover TeX: {s[:120]}")
    return U.clean_ws(s)


def main(repo, out, report):
    main_tex = strip_comments(open(os.path.join(repo, "bible.tex"), encoding="utf-8").read())
    rep = []
    books = []
    for m in re.finditer(r"\\InputBook\{([^}]*)\}\{([^}]*)\}", main_tex):
        keys = dict((k.strip(), v.strip()) for k, v in (kv.split("=", 1) for kv in m.group(1).split(",") if "=" in kv))
        path = m.group(2)
        code = FILE2CODE[os.path.basename(path)]
        src = strip_comments(open(os.path.join(repo, "kjv", path + ".tex"), encoding="utf-8").read())
        speakers = re.findall(r"\\NewSpeaker\{\\(\w+)\}", main_tex + src)
        src = re.sub(r"\\NewSpeaker\{[^}]*\}\{[^}]*\}\{[^}]*\}", "", src)
        if speakers:
            src = resolve_speakers(src, speakers)
        book = {"code": code, "items": [], "mt": [keys.get("name", "").replace("~", " ")],
                "toc2": keys.get("toc-name", keys.get("name", "")).replace("~", " "),
                "idtext": "King James Version - vermiculus/bible (BibleTeX project) bible.pdf"}
        ch = 0
        parts = re.split(r"(\\Chapter(?:\[[^\]]*\])?|\\Verse\b)", src)
        vs = 0
        pre = parts[0].strip()
        if pre:
            rep.append(f"{code}: text before first chapter: {pre[:80]}")
        for i in range(1, len(parts), 2):
            tok, body = parts[i], parts[i + 1]
            if tok.startswith("\\Chapter"):
                ch += 1
                vs = 0
                book["items"].append(("c", ch))
                nm = re.match(r"\\Chapter\[\s*name\s*=\s*(.*)\]", tok)
                if nm:
                    book["items"].append(("s", nm.group(1).strip()))
                book["items"].append(("p",))
                if body.strip():
                    rep.append(f"{code} {ch}: text between chapter and first verse: {body.strip()[:80]}")
            else:
                vs += 1
                if "\n\n" in body.strip():
                    pass
                book["items"].append(("v", vs, clean(body, speakers, rep, f"{code} {ch}:{vs}"), []))
        books.append(book)
    U.write_usfm_zip(books, out, "kjvBibleTeX")
    included = [b["code"] for b in books]
    rep.append("books in bible.tex (document order): " + " ".join(included))
    rep.append("book files in kjv/ not included by bible.tex: " + " ".join(
        sorted(set(FILE2CODE.values()) - set(included))))
    open(report, "w", encoding="utf-8").write("\n".join(rep) + "\n")
    st = U.structure(books)
    print(len(books), "books", sum(sum(c.values()) for c in st.values()), "verses;", len(rep), "report lines")


if __name__ == "__main__":
    main(*sys.argv[1:4])
