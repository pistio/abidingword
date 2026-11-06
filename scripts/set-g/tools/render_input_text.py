"""Check helper: linear text of a renderer's input file (HTML, flat ODF, groff -ms source,
Typst source), so that the reference can be compared with exactly what was sent to the
typesetting program (used with check_sequence.py, in addition to the PDF text layer).
Usage: python3 -I render_input_text.py INPUT OUT.txt
"""
import html
import re
import sys


def main(src, out):
    t = open(src, encoding="utf-8").read()
    if src.endswith((".html", ".xhtml")):
        t = re.sub(r"<style.*?</style>", " ", t, flags=re.S)
        t = re.sub(r"</(p|h\d|li|div)>", "\n", t)
        t = html.unescape(re.sub(r"<[^>]+>", " ", t))
    elif src.endswith(".fodt"):
        t = t[t.index("<office:text>"):]
        t = re.sub(r"</text:(p|h)>", "\n", t)
        t = html.unescape(re.sub(r"<[^>]+>", " ", t))
    elif src.endswith(".ms"):
        lines = []
        for l in t.split("\n"):
            if l.startswith(".") and not l.startswith("\\&"):
                continue
            l = re.sub(r"\\v'[^']*'|\\s[-+]?\d|\\f\[[A-Z]*\]|\\\||\\\*\*|\\&", " ", l)
            lines.append(l.replace("\\e", "\\"))
        t = "\n".join(lines)
    elif src.endswith(".typ"):
        body = t[t.index("#show: rest => columns"):]
        strings = re.findall(r'#vn\("([^"]*)"\)|#footnote\("((?:[^"\\]|\\.)*)"\)|#"((?:[^"\\]|\\.)*)"|\n(==? [^\n]*)', body)
        parts = []
        for vn, fn, s, hd in strings:
            if vn:
                parts.append(" " + vn + " ")
            elif fn:
                parts.append(" [" + fn + "] ")
            elif hd:
                parts.append("\n" + hd + "\n")
            else:
                parts.append(s.replace('\\"', '"').replace("\\\\", "\\"))
        t = "".join(parts)
    open(out, "w", encoding="utf-8").write(t)
    print(len(t), "chars")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
