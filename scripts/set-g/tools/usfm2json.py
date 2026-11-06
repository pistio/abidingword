"""Dump a reference USFM zip as JSON (books with items) for the JavaScript renderer.
Verse note placeholders \\x01n\\x01 are kept; notes as [kind, text].
Usage: python3 -I usfm2json.py REF.zip OUT.json
"""
import json
import sys

sys.path.insert(0, "/root/aw-set-g/tools")
import usfmlib as U  # noqa: E402

books = U.read_usfm_zip(sys.argv[1])
json.dump([{"code": b["code"], "headers": b["headers"], "items": [list(i) for i in b["items"]]} for b in books],
          open(sys.argv[2], "w", encoding="utf-8"), ensure_ascii=False)
print(len(books), "books")
