"""G09: render the 1938 Korean Revised Bible selection (셩경 개역) from the reference USFM
with Typst (Python package "typst").
Layout: B5 pages, two columns, Noto Sans CJK KR; title page and contents; running head
with the book name; page number in the footer; book titles as large centred headings;
chapter headings in the edition's own style "뎨삼쟝" (archaic Sino-Korean ordinals);
verse numbers in Chinese numerals as in the 1938 printing (一, 二 ... 一〇, 一一 ...),
small and raised; the edition's marginal notes as numbered footnotes; the printed
paragraph sign 〇 kept in the text.
Usage: python3 -I render_g09_kaeyok_typst.py REF.zip OUT.pdf WORKDIR
"""
import os
import re
import sys

sys.path.insert(0, "/root/aw-set-g/tools")
import usfmlib as U  # noqa: E402
import typst  # noqa: E402

NAMES = {"RUT": "룻긔", "JON": "요나", "MIC": "미가", "PHP": "빌닙보셔", "COL": "골노새셔",
         "1TH": "데살노니가 젼셔", "JAS": "야고보셔", "1JN": "요한一셔", "JUD": "유다셔"}
DIG = "〇一二三四五六七八九"
SINO = ["", "일", "이", "삼", "사", "오", "륙", "칠", "팔", "구", "십"]


def cjk(n):
    return "".join(DIG[int(c)] for c in str(n))


def ordinal(n):
    if n <= 10:
        return "뎨" + SINO[n] + "쟝"
    t, u = divmod(n, 10)
    return "뎨" + ("" if t == 1 else SINO[t]) + "십" + SINO[u] + "쟝"


def tstr(s):
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"') + '"'


def main(ref, out_pdf, work):
    os.makedirs(work, exist_ok=True)
    books = U.read_usfm_zip(ref)
    T = []
    T.append('''#set document(title: "셩경 개역 발췌")
#set page(paper: "iso-b5", margin: (top: 2.2cm, bottom: 2cm, inside: 2cm, outside: 1.6cm),
  header: context {
    let hs = query(selector(heading.where(level: 1)).before(here()))
    if hs.len() > 0 and counter(page).get().first() > 2 {
      set text(size: 8pt)
      align(center, hs.last().body)
      line(length: 100%, stroke: 0.4pt)
    }
  },
  footer: context { if counter(page).get().first() > 2 { align(center, text(size: 8pt)[— #counter(page).display() —]) } })
#set text(font: "Noto Sans CJK KR", size: 9.5pt, lang: "ko")
#set par(justify: true, leading: 0.72em, first-line-indent: 0em)
#show heading.where(level: 1): it => { colbreak(weak: true); align(center, block(above: 0.6em, below: 1em, text(size: 16pt, weight: "bold", it.body))) }
#show heading.where(level: 2): it => align(center, block(above: 0.9em, below: 0.5em, text(size: 10.5pt, weight: "bold", it.body)))
#let vn(n) = box(text(size: 6.5pt, weight: "bold", baseline: -0.35em, n)) + h(0.15em)
#set footnote.entry(separator: line(length: 30%, stroke: 0.4pt))
#show footnote.entry: set text(size: 7.5pt)
#align(center + horizon)[
  #text(size: 26pt, weight: "bold")[셩경 개역]
  #v(1em)
  #text(size: 11pt)[구약 룻긔·요나·미가 · 신약 서신 발췌]
  #v(3em)
  #text(size: 8pt)[京城 聖書公會 昭和十三年(1938) 發行本에 의함 · 위키문헌 전사본 (퍼블릭 도메인)]
]
#pagebreak()
#heading(level: 2, outlined: false)[목록]
#outline(title: none, depth: 1)
#pagebreak()
#show: rest => columns(2, gutter: 0.6cm, rest)
''')
    for b in books:
        T.append(f"= {NAMES[b['code']]}\n")
        for it in b["items"]:
            if it[0] == "c":
                T.append(f"\n== {ordinal(it[1])}\n\n")
            elif it[0] == "v":
                parts = re.split(r"\x01(\d+)\x01", it[2])
                seg = [f'#vn({tstr(cjk(it[1]))})#{tstr(parts[0])}']
                for k in range(1, len(parts), 2):
                    nt = it[3][int(parts[k])][1]
                    seg.append(f'#footnote({tstr(nt)})#{tstr(parts[k + 1].lstrip())}')
                T.append("".join(seg) + " ")
            elif it[0] == "p":
                T.append("\n\n")
        T.append("\n\n")
    src = os.path.join(work, "g09.typ")
    open(src, "w", encoding="utf-8").write("".join(T))
    typst.compile(src, output=out_pdf, root=work)
    print("written", out_pdf)


if __name__ == "__main__":
    main(*sys.argv[1:4])
