r"""Convert the LaTeX sources of "Biblia Sacra ex Sebastiani Castellionis Interpretatione"
(github.com/njand/biblia-castellionis, commit 0f93476 of 2023-04-02, CC BY-SA 4.0) to USFM.

Macros (defined in LaTeX/main.tex of that commit):
  \Liber[long title]{short}  -> book (\mt1 long title, \h short)
  \Caput                     -> \c n (counter)
  \Versus                    -> \v n (counter; verse 1 unnumbered in print)
  \emph{x}                   -> x (italics)
  verse/patverse* environments, \indentpattern{..}, \\ -> poetry layout only
  ``..'' / `..'  -> “..” / ‘..’ ; -- -> – ; --- -> — ; ~ -> no-break space
  % comments (incl. commented \summary and \footnote) are not printed
Usage: python3 -I conv_castellio_latex.py LATEX_DIR OUT.zip REPORT
"""
import os
import re
import sys

sys.path.insert(0, "/root/aw-set-g/tools")
import usfmlib as U  # noqa: E402

FILES = [("vetus foedus/genesis.tex", "GEN"), ("novum foedus/matthaeus.tex", "MAT"),
         ("novum foedus/marcus.tex", "MRK"), ("novum foedus/lucas.tex", "LUK"),
         ("novum foedus/johannes.tex", "JHN")]


def strip_comments(t):
    return "\n".join(re.sub(r"(?<!\\)%.*$", "", l) for l in t.split("\n"))


def remove_emph(s):
    """\\emph{..} -> .. with brace matching (an italic quotation may span verses)."""
    while True:
        m = re.search(r"\\emph\{", s)
        if not m:
            return s
        depth, i = 1, m.end()
        while depth:
            if s[i] == "{":
                depth += 1
            elif s[i] == "}":
                depth -= 1
            i += 1
        s = s[:m.start()] + s[m.end():i - 1] + s[i:]


def latex_text(s, rep, where):
    s = re.sub(r"\\begin\{[a-z*]+\}|\\end\{[a-z*]+\}", " ", s)
    s = re.sub(r"\\indentpattern\{[^}]*\}", " ", s)
    s = s.replace("\\\\", " ")
    s = s.replace("\\-", "")
    s = re.sub(r"\\(medskip|smallskip|bigskip|noindent)\b", " ", s)
    s = s.replace("``", "\u201c").replace("''", "\u201d").replace("`", "\u2018").replace("'", "\u2019")
    s = s.replace("{}", "")
    s = s.replace("---", "\u2014").replace("--", "\u2013").replace("~", "\u00a0")
    if re.search(r"[\\{}]", s):
        rep.append(f"{where}: leftover LaTeX: {s[:100]}")
    return U.clean_ws(s)


def main(latex_dir, out, report):
    rep, books = [], []
    for rel, code in FILES:
        src = strip_comments(open(os.path.join(latex_dir, "biblia", rel), encoding="utf-8").read())
        lm = re.search(r"\\Liber\[(.*?)\]\{(.*?)\}", src, re.S)
        long_title = U.clean_ws(lm.group(1).replace("\\\\", " "))
        book = {"code": code, "mt": [long_title], "h": lm.group(2),
                "idtext": "Biblia Sacra ex Sebastiani Castellionis interpretatione (njand/biblia-castellionis 0f93476)",
                "items": []}
        body = remove_emph(src[lm.end():])
        parts = re.split(r"(\\Caput\b|\\Versus\b)", body)
        ch = vs = 0
        if parts[0].strip():
            rep.append(f"{code}: text before first chapter: {parts[0].strip()[:60]}")
        for i in range(1, len(parts), 2):
            tok, txt = parts[i], parts[i + 1]
            if tok == "\\Caput":
                ch += 1
                vs = 0
                book["items"].append(("c", ch))
                book["items"].append(("p",))
                if latex_text(txt, rep, f"{code} {ch}"):
                    rep.append(f"{code} {ch}: text between \\Caput and first \\Versus")
            else:
                vs += 1
                book["items"].append(("v", vs, latex_text(txt, rep, f"{code} {ch}:{vs}"), []))
        books.append(book)
    U.write_usfm_zip(books, out, "laCastellio")
    open(report, "w", encoding="utf-8").write("\n".join(rep) + "\n")
    st = U.structure(books)
    print(len(books), "books", sum(sum(c.values()) for c in st.values()), "verses;", len(rep), "report lines")


if __name__ == "__main__":
    main(*sys.argv[1:4])
