"""G11: write the Nestle 1904 Greek New Testament (all books except Mark) as a hand-built
Markdown file (no converter program; this script writes the Markdown text itself).
The book data come from conv_nestle1904_xml.convert(); the USFM written from them must
equal the reference zip byte for byte.
Conventions: YAML front matter; a linked contents list; "# ΚΑΤΑ ΜΑΘΘΑΙΟΝ" book headings
(the edition's Greek titles); chapter headings "## Κεφάλαιον 5"; verse numbers as bold
numerals "**12**" run into the paragraph (verse 1 too); paragraphs as in the edition;
text-critical signs printed as in the edition; a horizontal rule between books.
Usage: python3 -I render_g11_nestle_md.py XMLDIR REF.zip OUT.md WORKDIR
"""
import os
import re
import sys
import zipfile

sys.path.insert(0, "/root/aw-set-g/tools")
import usfmlib as U  # noqa: E402
import conv_nestle1904_xml as C  # noqa: E402

BOOKS = [c for c in U.BOOK_ORDER[39:66] if c != "MRK"]


def slug(s):
    return re.sub(r"\s+", "-", s.strip().lower())


def md_escape(s):
    # keep the text; escape characters Markdown would treat as markup
    s = s.replace("\\", "\\\\").replace("*", "\\*").replace("_", "\\_")
    # [ ] < > are printed literally: in CommonMark they are not markup here (no link
    # target follows a bracket, no ASCII tag name follows <)
    return s


def main(xmldir, ref, out_md, work):
    os.makedirs(work, exist_ok=True)
    books = C.convert(xmldir, set(BOOKS), [])
    tmp = os.path.join(work, "reconverted.zip")
    U.write_usfm_zip(books, tmp, "grcN1904")
    za, zb = zipfile.ZipFile(tmp), zipfile.ZipFile(ref)
    if sorted(za.namelist()) != sorted(zb.namelist()) or any(za.read(n) != zb.read(n) for n in zb.namelist()):
        raise SystemExit("reference differs from re-conversion")
    L = ["---", "title: Ἡ Καινὴ Διαθήκη (Nestle 1904) — ἐκλογή", "language: grc",
         "source: Eberhard Nestle, Novum Testamentum Graece, British and Foreign Bible Society 1904",
         "text: base text by Diego Renato dos Santos (public domain); XML markup biblicalhumanities.org (CC BY-SA 4.0)",
         "---", "", "# Ἡ ΚΑΙΝΗ ΔΙΑΘΗΚΗ", "", "*Nestle 1904 — πάντα τὰ βιβλία πλὴν τοῦ κατὰ Μᾶρκον.*", "",
         "## Περιεχόμενα", ""]
    for b in books:
        L.append(f"- [{b['mt'][0]}](#{slug(b['mt'][0])})")
    L.append("")
    for b in books:
        L += ["---", "", f"# {b['mt'][0]}", ""]
        para = []
        ch = 0

        def flush():
            if para:
                L.append(" ".join(para))
                L.append("")
                para.clear()

        for it in b["items"]:
            if it[0] == "c":
                flush()
                ch = it[1]
                L += [f"## Κεφάλαιον {ch}", ""]
            elif it[0] == "p":
                flush()
            elif it[0] == "v":
                para.append(f"**{it[1]}** {md_escape(it[2])}")
        flush()
    open(out_md, "w", encoding="utf-8").write("\n".join(L) + "\n")
    print("written", out_md)


if __name__ == "__main__":
    main(*sys.argv[1:5])
