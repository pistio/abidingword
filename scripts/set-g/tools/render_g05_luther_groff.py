"""G05: render Luther 1545 (selection) from the reference USFM with GNU groff (-ms, gropdf).
Layout: A5, two columns, Bookman; running head with the book name, centred folio at the
foot; book titles in the old style; chapter headings "Das XII. Capitel." (Roman numerals);
superscript verse numbers (verse 1 included); captions as italic centred headings;
Luther's glosses and cross references as numbered footnotes at the foot of the column.
Usage: python3 -I render_g05_luther_groff.py REF.zip OUT.pdf WORKDIR
"""
import os
import re
import subprocess
import sys

sys.path.insert(0, "/root/aw-set-g/tools")
import usfmlib as U  # noqa: E402

TITLES = {"GEN": "Das Erste Buch Mose.", "RUT": "Das Buch Ruth.", "PSA": "Der Psalter.",
          "JDT": "Das Buch Judith.", "WIS": "Die Weisheit Salomonis.", "TOB": "Das Buch Tobias.",
          "SIR": "Das Buch Jesus Syrach.", "BAR": "Das Buch Baruch.",
          "1MA": "Das Erste Buch der Maccabeer.", "2MA": "Das Ander Buch der Maccabeer.",
          "MAN": "Das Gebet Manasse.", "ROM": "Die Epistel S. Pauli an die Römer."}
HEADS = {"GEN": "I. Mose", "RUT": "Ruth", "PSA": "Psalter", "JDT": "Judith", "WIS": "Weisheit",
         "TOB": "Tobias", "SIR": "Syrach", "BAR": "Baruch", "1MA": "I. Maccabeer",
         "2MA": "II. Maccabeer", "MAN": "Manasse", "ROM": "Römer"}
APO = ["JDT", "WIS", "TOB", "SIR", "BAR", "1MA", "2MA", "MAN"]


def roman(n):
    vals = [(1000, "M"), (900, "CM"), (500, "D"), (400, "CD"), (100, "C"), (90, "XC"), (50, "L"),
            (40, "XL"), (10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I")]
    out = ""
    for v, s in vals:
        while n >= v:
            out += s
            n -= v
    return out


def esc(s):
    s = s.replace("\\", "\\e")
    s = re.sub(r"^([.'])", r"\\&\1", s)
    return s


def verse_text(txt, notes):
    parts = re.split(r"\x01(\d+)\x01", txt)
    out = [esc(parts[0])]
    for k in range(1, len(parts), 2):
        kind, nt = notes[int(parts[k])]
        out.append("\\**\n.FS\n" + ("\\f[I]" if kind == "x" else "") + esc(nt) + ("\\f[]" if kind == "x" else "") + "\n.FE\n" + esc(parts[k + 1].lstrip()))
    return "".join(out)


def main(ref, out_pdf, work):
    os.makedirs(work, exist_ok=True)
    books = U.read_usfm_zip(ref)
    order = ["GEN", "RUT", "PSA", "ROM", "JDT", "WIS", "TOB", "SIR", "BAR", "1MA", "2MA", "MAN"]
    books.sort(key=lambda b: order.index(b["code"]))
    L = []
    L += [".nr PS 9", ".nr VS 11", ".nr PO 1.6c", ".nr LL 11.2c", ".nr HM 1.4c", ".nr FM 1.6c",
          ".nr PI 0", ".nr PD 0.2v", ".nr FPS 7", ".nr FVS 8", ".ds FAM BM", ".fam BM", ".nr GROWPS 3", ".nr PSINCR 2p",
          ".nr HY 1", ".hy 4", ".hla de", ".ds CF \\\\n[%]", ".ds LH", ".ds RH", ".ds CH"]
    # title page
    L += [".nr TITLEPAGE 1", ".sp 6", ".ce 99", ".ps 20", ".vs 26", "\\f[B]Biblia\\f[]", ".ps 11", ".vs 14",
          ".sp 1", "Das ist / die gantze Heilige Schrifft Deudsch.", ".sp 1", "D. Mart. Luth.", ".sp 1",
          "Auszug: Das Erste Buch Mose, Ruth, der Psalter, die Epistel an die Römer,",
          "und die Bücher, so man Apocrypha nennet.", ".sp 2", ".ps 8", ".vs 10",
          "Wittemberg 1545 (letzte Hand). Text nach der Digitalisierung von lutherbibel.net",
          "(Zefania XML). Gemeinfrei. Satz: GNU groff.", ".ce 0", ".bp"]
    # contents
    L += [".ps 9", ".vs 11", ".ce 1", "\\f[B]Register der Bücher\\f[]", ".sp 1", ".nf"]
    for b in books:
        L.append(esc(TITLES[b["code"]]))
    L += [".fi", ".bp", ".2C"]
    apo_started = False
    for b in books:
        code = b["code"]
        if code in APO and not apo_started:
            apo_started = True
            L += [".1C", ".bp", ".sp 4", ".ce 3", ".ps 14", "\\f[B]Apocrypha.\\f[]", ".ps 9",
                  "Das sind Bücher: so der heiligen Schrifft nicht gleich gehalten:",
                  "und doch nützlich und gut zu lesen sind.", ".sp 2", ".2C"]
        L += [".ds CH " + esc(HEADS[code]), ".sp 0.5", ".ne 6", ".ce 1", ".ps 13", ".vs 16",
              "\\f[B]" + esc(TITLES[code]) + "\\f[]", ".ps 9", ".vs 11", ".sp 0.3"]
        ch = 0
        pending_para = True
        for it in b["items"]:
            k = it[0]
            if k == "c":
                ch = it[1]
                if code != "MAN":
                    word = "Psalm" if code == "PSA" else "Capitel"
                    L += [".sp 0.4", ".ne 3", ".ce 1", "\\f[B]Das " + roman(ch) + ". " + word + ".\\f[]", ".sp 0.2"]
                pending_para = True
            elif k == "s":
                L += [".sp 0.2", ".ne 2", ".ce 1", "\\f[I]" + esc(it[1]) + "\\f[]"]
                pending_para = True
            elif k == "v":
                if pending_para:
                    L.append(".LP")
                    pending_para = False
                L.append("\\v'-0.3m'\\s-2\\f[B]" + str(it[1]) + "\\f[]\\s+2\\v'0.3m'\\|" + verse_text(it[2], it[3]))
            elif k == "p":
                pending_para = True
    src = os.path.join(work, "g05_luther.ms")
    with open(src, "w", encoding="utf-8") as f:
        f.write("\n".join(L) + "\n")
    with open(out_pdf, "wb") as f:
        subprocess.run(["groff", "-ms", "-Kutf8", "-Tpdf", "-P-pa5", "-dpaper=a5", src], stdout=f, check=True)
    print("written", out_pdf)


if __name__ == "__main__":
    main(*sys.argv[1:4])
