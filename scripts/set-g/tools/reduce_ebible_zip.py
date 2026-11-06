"""Reduce an eBible.org USFM zip to the books a document holds: the selected book files
and copr.htm are copied byte for byte (no other change).
Usage: python3 -I reduce_ebible_zip.py SRC.zip OUT.zip BOOK,BOOK,...
"""
import re
import sys
import zipfile


def main(src, out, books):
    want = set(books.split(","))
    with zipfile.ZipFile(src) as zi, zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as zo:
        got = set()
        for info in zi.infolist():
            m = re.match(r"\d+-([0-9A-Z]{3})", info.filename)
            if (m and m.group(1) in want) or info.filename == "copr.htm":
                zo.writestr(info, zi.read(info.filename))
                if m:
                    got.add(m.group(1))
        missing = want - got
        if missing:
            raise SystemExit("missing books: " + ",".join(sorted(missing)))
    print("copied", len(got), "books")


if __name__ == "__main__":
    main(*sys.argv[1:4])
