#!/usr/bin/env python3
"""F08: Salkinson-Ginsburg Hebrew New Testament (eBible hebsg), as Word .docx.
Generator: a flat ODF text document written here, saved as .docx by LibreOffice
Writer (headless, filter 'MS Word 2007 XML'). Right-to-left paragraphs in two
columns, Hebrew-letter chapter labels and verse numbers exactly as the source's
\\cl / \\vp give them, book titles from \\mt1 as Heading 1 (with the short name
in the running head through a chapter field), page numbers in the footer, and
the edition's English introduction (left-to-right) in front."""
import os, sys, zipfile, subprocess, shutil
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import usfm
from common import load_zip, clean_items, esc, DOCS, ROOT, WORK, short_name, book_title

REF = os.path.join(DOCS, 'F08-heb-salkinson.usfm.zip')
SRC = os.path.join(ROOT, 'src', 'eb', 'hebsg_usfm.zip')
OUT = os.path.join(DOCS, 'F08-heb-salkinson.docx')

HEAD = r'''<?xml version="1.0" encoding="UTF-8"?>
<office:document xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
 xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0"
 xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0" xmlns:svg="urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0"
 xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:meta="urn:oasis:names:tc:opendocument:xmlns:meta:1.0"
 office:version="1.3" office:mimetype="application/vnd.oasis.opendocument.text">
<office:meta><dc:title>הברית החדשה - תרגום זלקינסון-גינצבורג</dc:title><meta:generator>set-F make_f08_salkinson_lodocx.py</meta:generator></office:meta>
<office:font-face-decls>
 <style:font-face style:name="Noto Serif Hebrew" svg:font-family="'Noto Serif Hebrew'" style:font-family-generic="roman"/>
 <style:font-face style:name="Noto Serif" svg:font-family="'Noto Serif'" style:font-family-generic="roman"/>
</office:font-face-decls>
<office:styles>
 <style:default-style style:family="paragraph">
  <style:paragraph-properties style:writing-mode="rl-tb" fo:text-align="justify"/>
  <style:text-properties style:font-name="Noto Serif" fo:font-size="10pt" fo:language="en" fo:country="US"
    style:font-name-complex="Noto Serif Hebrew" style:font-size-complex="11pt" style:language-complex="he" style:country-complex="IL"/></style:default-style>
 <style:style style:name="Standard" style:family="paragraph"/>
 <style:style style:name="Verse" style:family="paragraph" style:parent-style-name="Standard">
  <style:paragraph-properties style:writing-mode="rl-tb" fo:text-align="justify" fo:margin-bottom="0.03in" fo:line-height="140%"/></style:style>
 <style:style style:name="Heading_20_1" style:display-name="Heading 1" style:family="paragraph" style:default-outline-level="1">
  <style:paragraph-properties style:writing-mode="rl-tb" fo:text-align="center" fo:margin-bottom="0.1in" fo:break-before="page"/>
  <style:text-properties fo:font-size="15pt" fo:font-weight="bold" style:font-size-complex="16pt" style:font-weight-complex="bold"/></style:style>
 <style:style style:name="Heading_20_2" style:display-name="Heading 2" style:family="paragraph" style:default-outline-level="2">
  <style:paragraph-properties style:writing-mode="rl-tb" fo:text-align="center" fo:margin-top="0.1in" fo:margin-bottom="0.04in" fo:keep-with-next="always"/>
  <style:text-properties fo:font-size="11pt" fo:font-weight="bold" style:font-size-complex="12pt" style:font-weight-complex="bold"/></style:style>
 <style:style style:name="Intro" style:family="paragraph"><style:paragraph-properties style:writing-mode="lr-tb" fo:text-align="justify" fo:margin-bottom="0.06in"/></style:style>
 <style:style style:name="IntroHead" style:family="paragraph"><style:paragraph-properties style:writing-mode="lr-tb" fo:margin-top="0.12in" fo:margin-bottom="0.04in"/>
  <style:text-properties fo:font-weight="bold" fo:font-size="11pt"/></style:style>
 <style:style style:name="Title" style:family="paragraph"><style:paragraph-properties fo:text-align="center" fo:margin-top="0.15in"/>
  <style:text-properties fo:font-size="14pt" style:font-size-complex="26pt"/></style:style>
 <style:style style:name="Header" style:family="paragraph"><style:paragraph-properties style:writing-mode="rl-tb" fo:text-align="center"/>
  <style:text-properties fo:font-size="8pt" style:font-size-complex="9pt"/></style:style>
 <style:style style:name="Footer" style:family="paragraph"><style:paragraph-properties fo:text-align="center"/><style:text-properties fo:font-size="8pt"/></style:style>
 <style:style style:name="VN" style:family="text"><style:text-properties fo:color="#6b1d1d" fo:font-weight="bold" style:font-weight-complex="bold" style:font-size-complex="8pt"/></style:style>
</office:styles>
<office:automatic-styles>
 <style:page-layout style:name="pm1"><style:page-layout-properties fo:page-width="6.3in" fo:page-height="9.25in" fo:margin-top="0.6in" fo:margin-bottom="0.55in" fo:margin-left="0.6in" fo:margin-right="0.6in" style:writing-mode="rl-tb"/>
  <style:header-style><style:header-footer-properties fo:min-height="0in" fo:margin-bottom="0.1in"/></style:header-style>
  <style:footer-style><style:header-footer-properties fo:min-height="0in" fo:margin-top="0.1in"/></style:footer-style></style:page-layout>
 <style:style style:name="Sect2" style:family="section"><style:section-properties style:writing-mode="rl-tb"><style:columns fo:column-count="2" fo:column-gap="0.25in"><style:column-sep style:width="0.4pt" style:color="#000000" style:height="100%"/></style:columns></style:section-properties></style:style>
</office:automatic-styles>
<office:master-styles>
 <style:master-page style:name="Standard" style:page-layout-name="pm1">
  <style:header><text:p text:style-name="Header"><text:chapter text:display="name" text:outline-level="1"/></text:p></style:header>
  <style:footer><text:p text:style-name="Footer"><text:page-number text:select-page="current"/></text:p></style:footer>
 </style:master-page>
</office:master-styles>
<office:body><office:text>
'''


def main():
    books = load_zip(REF)
    x = [HEAD]
    x.append('<text:p text:style-name="Title">הַבְּרִית הַחֲדָשָׁה</text:p>')
    x.append('<text:p text:style-name="Title">תרגום זלקינסון–גינצבורג</text:p>')
    x.append('<text:p text:style-name="Intro">The Salkinson–Ginsburg Hebrew New Testament. Public domain (eBible.org).</text:p>')
    with zipfile.ZipFile(SRC) as z:
        intro = usfm.parse_usfm(z.read('01-INThebsg.usfm').decode('utf-8-sig'))
    x.append('<text:p text:style-name="IntroHead">%s</text:p>' % esc(book_title(intro)))
    for bl in intro['blocks']:
        if bl['kind'] != 'para':
            continue
        t = esc(''.join(i[1] for i in clean_items(bl['items']) if i[0] == 't'))
        x.append('<text:p text:style-name="%s">%s</text:p>' % ('IntroHead' if bl['style'].startswith('is') else 'Intro', t))
    for b in books:
        x.append('<text:h text:style-name="Heading_20_1" text:outline-level="1">%s</text:h>' % esc(short_name(b)))
        x.append('<text:p text:style-name="Title">%s</text:p>' % esc(book_title(b)))
        x.append('<text:section text:style-name="Sect2" text:name="S%s">' % b['code'])
        for bl in b['blocks']:
            if bl['kind'] == 'chapter':
                x.append('<text:h text:style-name="Heading_20_2" text:outline-level="2">%s</text:h>'
                         % esc(bl['cl'] or ('פרק ' + (bl['cp'] or bl['n']))))
                continue
            if bl['kind'] != 'para':
                continue
            out = []
            for it in clean_items(bl['items']):
                if it[0] == 'v':
                    out.append('<text:span text:style-name="VN">%s</text:span> ' % esc(it[2] or it[1]))
                elif it[0] == 't':
                    out.append(esc(it[1]))
            x.append('<text:p text:style-name="Verse">%s</text:p>' % ''.join(out))
        x.append('</text:section>')
    x.append('</office:text></office:body></office:document>')
    fodt = os.path.join(WORK, 'F08-heb-salkinson.fodt')
    with open(fodt, 'w', encoding='utf-8') as f:
        f.write('\n'.join(x))
    od = os.path.join(WORK, 'f08out')
    os.makedirs(od, exist_ok=True)
    subprocess.run(['soffice', '--headless', '-env:UserInstallation=file:///root/aw-set-f/work/lo_profile',
                    '--convert-to', 'docx:MS Word 2007 XML', '--outdir', od, fodt], check=True)
    shutil.copy(os.path.join(od, 'F08-heb-salkinson.docx'), OUT)
    print(OUT)


if __name__ == '__main__':
    main()
