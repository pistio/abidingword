r"""Convert ko.wikisource proofread pages of "셩경 개역" (Korean Revised Bible, Seoul:
British and Foreign Bible Society (聖書公會), printed 1 Sept 1938) to USFM.

Source markup (page wikitext, namespace 페이지:셩경 개역.pdf/N):
  {{더크게|NAME}}         book title -> new book
  == 뎨일쟝 ==            chapter heading (archaic Sino-Korean ordinal; number taken from {{절}})
  {{절|C|V(|장빈칸=f)}}   verse V of chapter C, numbers in Chinese numerals (一〇 = 10)
  {{u|..}} {{du|..}} {{물결밑줄|..}} {{복원|..}} {{작게|..}}  -> text kept (underlines,
                         small type and restorations are typography only)
  <ref>..</ref>          marginal/foot note -> \f
  {{upe}} at a page end    word continues on the next page: pages joined without a space
  {{여백|..}} {{nop}} <noinclude>..</noinclude>  -> layout only, dropped
  〇                     paragraph mark printed in the text -> kept as printed
Usage: python3 -I conv_wikisource_kaeyok1938.py PAGES_DIR OUT.zip REPORT
"""
import glob
import os
import re
import sys

sys.path.insert(0, "/root/aw-set-g/tools")
import usfmlib as U  # noqa: E402

TITLE2CODE = {"룻긔": "RUT", "요나": "JON", "미가": "MIC", "빌닙보셔": "PHP", "골노새셔": "COL",
              "데살노니가 젼셔": "1TH", "야고보셔": "JAS", "요한一셔": "1JN", "유다셔": "JUD"}
CJK = {"〇": 0, "一": 1, "二": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9}


def cjk(s):
    return int("".join(str(CJK[c]) for c in s.strip()))


def strip_templates(t):
    t = re.sub(r"\{\{(?:u|du|물결밑줄|복원|작게)\|([^{}]*)\}\}", r"\1", t)
    t = re.sub(r"\{\{여백\|[^{}]*\}\}", " ", t)
    t = re.sub(r"\{\{(?:upe|nop)\}\}", "", t)
    t = re.sub(r"\{\{왼쪽 여백/[se][^{}]*\}\}", "", t)
    return t


def main(pages_dir, out, report):
    files = sorted(glob.glob(os.path.join(pages_dir, "p*.txt")), key=lambda p: int(re.search(r"p(\d+)", p).group(1)))
    rep, books, quality = [], [], {}
    cur, keep = None, False
    stream = []
    for f in files:
        n = int(re.search(r"p(\d+)", f).group(1))
        t = open(f, encoding="utf-8").read()
        q = re.search(r'pagequality level="(\d)"', t)
        quality[n] = q.group(1) if q else "?"
        t = re.sub(r"<noinclude>.*?</noinclude>", "", t, flags=re.S)
        if re.search(r"<html|<script", t):
            rep.append(f"page {n}: not wikitext")
            continue
        stream.append((n, t))
    # tokenise across pages (a verse may continue on the next page)
    full = ""
    page_at = []
    for n, t in stream:
        page_at.append((len(full), n))
        t = t.strip()
        # {{upe}} at a page end marks a word continued on the next page: join without a space
        full += t[:-len("{{upe}}")] if t.endswith("{{upe}}") else t + "\n"
    full = strip_templates(full)
    toks = re.split(r'(\{\{더크게\|[^{}]*\}\}|\{\{절\|[^{}]*\}\}|^==[^=\n]+==\s*$|<ref(?: name="[^"]*")?>.*?</ref>|<ref name="[^"]*"\s*/>)', full, flags=re.M | re.S)
    named = dict(re.findall(r'<ref name="([^"]*)">(.*?)</ref>', full, flags=re.S))
    cur_v = None
    for tok in toks:
        if not tok:
            continue
        m = re.match(r"\{\{더크게\|([^{}]*)\}\}", tok)
        if m:
            name = m.group(1).strip()
            code = TITLE2CODE.get(name)
            if code is None:
                rep.append(f"book outside the selection skipped: {name}")
                keep = False
                cur = None
                cur_v = None
                continue
            cur = {"code": code, "items": [], "mt": [name],
                   "idtext": "셩경 개역 (1938), ko.wikisource transcription"}
            books.append(cur)
            keep = True
            cur_v = None
            continue
        if not keep:
            continue
        m = re.match(r"\{\{절\|([^|{}]*)\|([^|{}]*)(\|[^{}]*)?\}\}", tok)
        if m:
            c, v = cjk(m.group(1)), cjk(m.group(2))
            chs = [it[1] for it in cur["items"] if it[0] == "c"]
            if not chs or chs[-1] != c:
                cur["items"].append(("c", c))
                cur["items"].append(("p",))
            cur["items"].append(("v", v, "", []))
            cur_v = len(cur["items"]) - 1
            continue
        if re.match(r"^==[^=\n]+==", tok):
            continue
        m = re.match(r'<ref(?: name="[^"]*")?>(.*?)</ref>', tok, re.S)
        mn = re.match(r'<ref name="([^"]*)"\s*/>', tok)
        if mn:
            m = re.match(r"(.*)", named.get(mn.group(1), "?"), re.S)
        if m:
            if cur_v is None:
                rep.append(f"note outside a verse: {m.group(1)}")
                continue
            it = cur["items"][cur_v]
            notes = it[3] + [("f", U.clean_ws(m.group(1)))]
            cur["items"][cur_v] = ("v", it[1], it[2] + f" \x01{len(notes) - 1}\x01 ", notes)
            continue
        txt = tok
        if cur_v is None:
            if txt.strip():
                rep.append(f"{cur['code']}: text before first verse: {txt.strip()[:60]}")
            continue
        it = cur["items"][cur_v]
        cur["items"][cur_v] = ("v", it[1], it[2] + txt, it[3])
    for b in books:
        items = []
        for it in b["items"]:
            if it[0] == "v":
                t = U.clean_ws(it[2].replace("\n", " "))
                t = re.sub(r"\s*\x01(\d+)\x01\s*", lambda m: "\x01" + m.group(1) + "\x01 ", t).strip()
                # a word broken across a page boundary in the transcription is joined by the
                # page break only; keep text as transcribed
                items.append(("v", it[1], t, it[3]))
                if "{{" in t or "<" in t or "[[" in t:
                    rep.append(f"{b['code']} v{it[1]}: leftover markup: {t[:80]}")
            else:
                items.append(it)
        b["items"] = items
    order = {c: i for i, c in enumerate(U.BOOK_ORDER)}
    books.sort(key=lambda b: order[b["code"]])
    U.write_usfm_zip(books, out, "ko1938")
    rep.append("page quality levels (1 = not proofread, 3 = proofread): " +
               " ".join(f"{n}:{q}" for n, q in sorted(quality.items())))
    open(report, "w", encoding="utf-8").write("\n".join(rep) + "\n")
    st = U.structure(books)
    print(len(books), "books", sum(sum(c.values()) for c in st.values()), "verses;", len(rep), "report lines")


if __name__ == "__main__":
    main(*sys.argv[1:4])
