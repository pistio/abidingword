#!/usr/bin/env python3
"""F02: Leeser Tanakh (eBible englee) - the Twenty-Four Books in Jewish order.
Generator: a flat ODF text document (.fodt) written here, turned into PDF by
LibreOffice Writer (headless). One verse per line with a hanging indent and the
number followed by a full stop, chapters headed in roman numerals, book name in
the running head via an ODF chapter field, page number centred in the footer."""
import os, sys, subprocess, shutil
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import load_zip, clean_items, esc, DOCS, WORK, TANAKH, roman, short_name, book_title

REF = os.path.join(DOCS, 'F02-eng-leeser.usfm.zip')
OUT = os.path.join(DOCS, 'F02-eng-leeser.pdf')
DIV = {'GEN': 'THE LAW', 'JOS': 'THE PROPHETS', 'PSA': 'THE HOLY WRITINGS'}

HEAD = r'''<?xml version="1.0" encoding="UTF-8"?>
<office:document xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
 xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0"
 xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0" xmlns:svg="urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0"
 xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:meta="urn:oasis:names:tc:opendocument:xmlns:meta:1.0"
 office:version="1.3" office:mimetype="application/vnd.oasis.opendocument.text">
<office:meta><dc:title>The Twenty-Four Books of the Holy Scriptures (Leeser)</dc:title><meta:generator>set-F make_f02_leeser_lo.py</meta:generator></office:meta>
<office:font-face-decls>
 <style:font-face style:name="Liberation Serif" svg:font-family="'Liberation Serif'" style:font-family-generic="roman"/>
</office:font-face-decls>
<office:styles>
 <style:default-style style:family="paragraph"><style:paragraph-properties fo:hyphenation-ladder-count="no-limit"/>
  <style:text-properties style:font-name="Liberation Serif" fo:font-size="10pt" fo:language="en" fo:country="US" fo:hyphenate="true"/></style:default-style>
 <style:style style:name="Standard" style:family="paragraph"/>
 <style:style style:name="Verse" style:family="paragraph"><style:paragraph-properties fo:margin-left="0.32in" fo:text-indent="-0.32in" fo:text-align="justify" fo:margin-top="0in" fo:margin-bottom="0.02in"/></style:style>
 <style:style style:name="Heading_20_1" style:display-name="Heading 1" style:family="paragraph" style:default-outline-level="1">
  <style:paragraph-properties fo:text-align="center" fo:margin-top="0.1in" fo:margin-bottom="0.04in" fo:break-before="page"/>
  <style:text-properties fo:font-size="17pt" fo:font-weight="bold" fo:letter-spacing="0.03in"/></style:style>
 <style:style style:name="BookSub" style:family="paragraph"><style:paragraph-properties fo:text-align="center" fo:margin-bottom="0.16in"/>
  <style:text-properties fo:font-size="10pt" fo:font-style="italic"/></style:style>
 <style:style style:name="Division" style:family="paragraph"><style:paragraph-properties fo:text-align="center" fo:margin-top="2.6in" fo:break-before="page"/>
  <style:text-properties fo:font-size="22pt" fo:font-weight="bold" fo:letter-spacing="0.06in"/></style:style>
 <style:style style:name="Chapter" style:family="paragraph"><style:paragraph-properties fo:text-align="center" fo:margin-top="0.12in" fo:margin-bottom="0.05in" fo:keep-with-next="always"/>
  <style:text-properties fo:font-size="10pt" fo:font-variant="small-caps" fo:letter-spacing="0.02in"/></style:style>
 <style:style style:name="Header" style:family="paragraph"><style:paragraph-properties fo:text-align="center" fo:border-bottom="0.5pt solid #000000" fo:padding-bottom="0.02in"/>
  <style:text-properties fo:font-size="8.5pt" fo:letter-spacing="0.03in"/></style:style>
 <style:style style:name="Footer" style:family="paragraph"><style:paragraph-properties fo:text-align="center"/><style:text-properties fo:font-size="8.5pt"/></style:style>
 <style:style style:name="Title" style:family="paragraph"><style:paragraph-properties fo:text-align="center" fo:margin-top="0.2in"/><style:text-properties fo:font-size="12pt"/></style:style>
 <style:style style:name="Num" style:family="text"><style:text-properties fo:font-weight="bold"/></style:style>
</office:styles>
<office:automatic-styles>
 <style:page-layout style:name="pm1"><style:page-layout-properties fo:page-width="6.5in" fo:page-height="9.5in" fo:margin-top="0.55in" fo:margin-bottom="0.5in" fo:margin-left="0.75in" fo:margin-right="0.7in"><style:columns fo:column-count="2" fo:column-gap="0.22in"><style:column-sep style:width="0.4pt" style:color="#000000" style:height="100%"/></style:columns></style:page-layout-properties>
  <style:header-style><style:header-footer-properties fo:min-height="0in" fo:margin-bottom="0.12in"/></style:header-style>
  <style:footer-style><style:header-footer-properties fo:min-height="0in" fo:margin-top="0.1in"/></style:footer-style></style:page-layout>
 <style:style style:name="Sect2" style:family="section"><style:section-properties><style:columns fo:column-count="2" fo:column-gap="0.22in"><style:column-sep style:width="0.4pt" style:color="#000000" style:height="100%"/></style:columns></style:section-properties></style:style>
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
    books = load_zip(REF, TANAKH)
    x = [HEAD]
    x.append('<text:p text:style-name="Title"><text:span text:style-name="Num">THE TWENTY-FOUR BOOKS</text:span></text:p>')
    x.append('<text:p text:style-name="Title">OF THE</text:p><text:p text:style-name="Title"><text:span text:style-name="Num">HOLY SCRIPTURES:</text:span></text:p>')
    x.append('<text:p text:style-name="Title">CAREFULLY TRANSLATED ACCORDING TO THE MASSORETIC TEXT,</text:p>')
    x.append('<text:p text:style-name="Title">BY ISAAC LEESER.</text:p>')
    x.append('<text:p text:style-name="Title">(Public domain text from eBible.org)</text:p>')
    x.append('<text:p text:style-name="Chapter">ORDER OF THE BOOKS</text:p>')
    for b in books:
        x.append('<text:p text:style-name="BookSub">%s</text:p>' % esc(short_name(b)))
    for b in books:
        if b['code'] in DIV:
            x.append('<text:p text:style-name="Division">%s</text:p>' % DIV[b['code']])
        x.append('<text:h text:style-name="Heading_20_1" text:outline-level="1">%s</text:h>' % esc(short_name(b).upper()))
        if b['mt'] and book_title(b).upper() != short_name(b).upper():
            x.append('<text:p text:style-name="BookSub">%s</text:p>' % esc(book_title(b)))
        for bl in b['blocks']:
            if bl['kind'] == 'chapter':
                n = int(bl['n'])
                label = ('PSALM %s.' if b['code'] == 'PSA' else 'CHAPTER %s.') % roman(n)
                x.append('<text:p text:style-name="Chapter">%s</text:p>' % label)
                continue
            if bl['kind'] != 'para':
                continue
            items = clean_items(bl['items'])
            cur = None
            for it in items:
                if it[0] == 'v':
                    if cur is not None:
                        x.append(cur + '</text:p>')
                    cur = '<text:p text:style-name="Verse"><text:span text:style-name="Num">%s.</text:span><text:tab/>' % it[1]
                elif it[0] == 't':
                    if cur is None:
                        cur = '<text:p text:style-name="Verse">'
                    cur += esc(it[1])
            if cur is not None:
                x.append(cur + '</text:p>')
    x.append('</office:text></office:body></office:document>')
    fodt = os.path.join(WORK, 'F02-eng-leeser.fodt')
    with open(fodt, 'w', encoding='utf-8') as f:
        f.write('\n'.join(x))
    od = os.path.join(WORK, 'f02out')
    os.makedirs(od, exist_ok=True)
    subprocess.run(['soffice', '--headless', '-env:UserInstallation=file:///root/aw-set-f/work/lo_profile',
                    '--convert-to', 'pdf:writer_pdf_Export', '--outdir', od, fodt], check=True)
    shutil.copy(os.path.join(od, 'F02-eng-leeser.pdf'), OUT)
    print(OUT)


if __name__ == '__main__':
    main()
