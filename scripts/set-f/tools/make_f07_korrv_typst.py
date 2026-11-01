#!/usr/bin/env python3
"""F07: 개역성경 (Korean Revised Version, CrossWire KorRV via scrollmapper), complete.
Generator: Typst (python 'typst' package). Two columns on a 128x188 mm page,
one verse per line with the number in a fixed box (hanging indent), book title
centred over both columns, chapters headed '제 N 장' (Psalms '제 N 편'),
running head with the current book name (outer) and page number, a 목차
(contents) page built by Typst's outline()."""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import typst
from common import load_zip, clean_items, DOCS, WORK, short_name, NT

REF = os.path.join(DOCS, 'F07-kor-rv.usfm.zip')
OUT = os.path.join(DOCS, 'F07-kor-rv.pdf')


def s(x):
    return '"' + x.replace('\\', '\\\\').replace('"', '\\"') + '"'


PRE = r'''
#set document(title: "성경전서 개역", author: "대한성서공회 개역 (1961)")
#set text(font: ("Noto Serif CJK KR", "Noto Sans CJK KR"), size: 8pt, lang: "ko")
#set par(justify: true, leading: 0.5em, spacing: 0.32em)
#set page(width: 128mm, height: 188mm, margin: (top: 15mm, bottom: 12mm, inside: 13mm, outside: 10mm),
  header: context {
    let p = here().page()
    let hs = query(heading.where(level: 1).before(here()))
    let after = query(heading.where(level: 1).after(here())).filter(h => h.location().page() == p)
    let name = if after.len() > 0 { after.first().body } else if hs.len() > 0 { hs.last().body } else { none }
    if p > 3 and name != none {
      set text(size: 7pt)
      if calc.odd(p) { align(right)[#name #h(1fr) #str(p)] } else { align(left)[#str(p) #h(1fr) #name] }
      v(-0.6em)
      line(length: 100%, stroke: 0.4pt)
    }
  },
  footer: none)
#show heading.where(level: 1): it => {
  set align(center)
  set text(size: 15pt, weight: "bold")
  block(above: 0.4em, below: 1em, it.body)
}
#show heading.where(level: 2): it => {
  set align(center)
  set text(size: 8.6pt, weight: "bold", fill: rgb("#7a1f1f"))
  block(above: 0.9em, below: 0.5em, sticky: true, it.body)
}
#let vs(n, t) = par(hanging-indent: 1.7em, first-line-indent: (amount: 0em, all: true))[#box(width: 1.7em)[#text(size: 6.5pt, weight: "bold")[#n]]#t]
#align(center + horizon)[#text(size: 26pt, weight: "bold")[성 경 전 서] \ #v(1em) #text(size: 12pt)[개 역] \ #v(2em) #text(size: 8pt)[구약 · 신약]]
#pagebreak()
#outline(title: [목 차], depth: 1, indent: 0em)
#pagebreak()
#set page(columns: 2)
#set columns(gutter: 5mm)
'''


def main():
    books = load_zip(REF)
    out = [PRE]
    for b in books:
        code = b['code']
        name = short_name(b)
        if code == 'GEN':
            out.append('#place(top + center, scope: "parent", float: true)[#text(size: 11pt)[구 약 전 서]]')
        if code == 'MAT':
            out.append('#pagebreak(weak: true)')
            out.append('#place(top + center, scope: "parent", float: true)[#text(size: 11pt)[신 약 전 서]]')
        out.append('#pagebreak(weak: true)')
        out.append('#place(top + center, scope: "parent", float: true, heading(level: 1, %s))' % s(name))
        for bl in b['blocks']:
            if bl['kind'] == 'chapter':
                out.append('#heading(level: 2, outlined: false, %s)' % s(('제 %s 편' if code == 'PSA' else '제 %s 장') % bl['n']))
                continue
            if bl['kind'] != 'para':
                continue
            cur = None
            for it in clean_items(bl['items']):
                if it[0] == 'v':
                    if cur is not None:
                        out.append('#vs(%s, %s)' % (s(cur[0]), s(cur[1].strip())))
                    cur = [it[1], '']
                elif it[0] == 't' and cur is not None:
                    cur[1] += it[1]
            if cur is not None:
                out.append('#vs(%s, %s)' % (s(cur[0]), s(cur[1].strip())))
    src = os.path.join(WORK, 'F07-kor-rv.typ')
    with open(src, 'w', encoding='utf-8') as f:
        f.write('\n'.join(out))
    typst.compile(src, output=OUT, font_paths=['/usr/share/fonts/opentype/noto'])
    print(OUT)


if __name__ == '__main__':
    main()
