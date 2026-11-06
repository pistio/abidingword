"""Linear text of a document, for the conversion checks only (never used to build a
reference). EPUB: spine order, tags removed. DOCX: word/document.xml text (+ footnotes
appended at the end). PDF: pdftotext -raw (poppler, content-stream order). MD/TXT: as is.
Usage: python3 -I doc_text.py DOC OUT.txt
"""
import html
import re
import subprocess
import sys
import zipfile
import posixpath


def epub_text(p):
    z = zipfile.ZipFile(p)
    cont = z.read("META-INF/container.xml").decode()
    opf = re.search(r'full-path="([^"]+)"', cont).group(1)
    o = z.read(opf).decode("utf-8")
    base = posixpath.dirname(opf)
    man = {m.group(1): m.group(2) for m in re.finditer(r'<item\b[^>]*?id="([^"]+)"[^>]*?href="([^"]+)"', o)}
    man.update({m.group(2): m.group(1) for m in re.finditer(r'<item\b[^>]*?href="([^"]+)"[^>]*?id="([^"]+)"', o)})
    spine = re.findall(r'<itemref\b[^>]*idref="([^"]+)"', o)
    out = []
    for sid in spine:
        href = man[sid]
        path = posixpath.normpath(posixpath.join(base, href)) if base else href
        x = z.read(path).decode("utf-8")
        x = re.sub(r"<head.*?</head>", " ", x, flags=re.S)
        x = re.sub(r"<(br|/p|/div|/h\d|/li|/tr)\b[^>]*>", "\n", x)
        x = re.sub(r"<[^>]+>", " ", x)
        out.append(html.unescape(x))
    return "\n".join(out)


def docx_text(p):
    z = zipfile.ZipFile(p)
    out = []
    for part in ["word/document.xml", "word/footnotes.xml", "word/endnotes.xml"]:
        if part not in z.namelist():
            continue
        x = z.read(part).decode("utf-8")
        x = re.sub(r"<w:tab/>", "\t", x)
        x = re.sub(r"<w:br[^>]*/>", "\n", x)
        x = re.sub(r"</w:p>", "\n", x)
        x = re.sub(r"<w:t[^>]*>(.*?)</w:t>", lambda m: "\x02" + m.group(1) + "\x03", x, flags=re.S)
        x = re.sub(r"<[^>]+>", "", x)
        x = x.replace("\x02", "").replace("\x03", "")
        out.append(html.unescape(x))
    return "\n".join(out)


def main(doc, out):
    if doc.endswith(".epub"):
        t = epub_text(doc)
    elif doc.endswith(".docx"):
        t = docx_text(doc)
    elif doc.endswith(".pdf"):
        t = subprocess.run(["pdftotext", "-raw", "-enc", "UTF-8", doc, "-"], capture_output=True, check=True).stdout.decode("utf-8")
    else:
        t = open(doc, encoding="utf-8-sig").read()
    open(out, "w", encoding="utf-8").write(t)
    print(len(t), "chars")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
