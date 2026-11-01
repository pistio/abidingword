"""kor (Korean Bible 1910) New Testament -> FODT -> PDF with LibreOffice (headless).
B5 page, book title full width, text in two balanced columns (section) with column rule, '제 N 장' chapter headings,
one verse per line with the verse number leading, running head = current book (chapter field), '- n -' footer."""
import sys, os, subprocess
sys.path.insert(0, '/root/aw-set-e/tools')
import usfm, fodt
from fodt import E, text_with_spaces

SRC, OUT, PROFILE = sys.argv[1], sys.argv[2], sys.argv[3]
F = 'Noto Serif CJK KR'
FS = 'Noto Sans CJK KR'
styles = f'''
<style:default-style style:family="paragraph"><style:paragraph-properties fo:hyphenation-ladder-count="no-limit"/>
 <style:text-properties style:font-name="{F}" style:font-name-asian="{F}" fo:font-size="9.5pt" style:font-size-asian="9.5pt"
  fo:language="ko" fo:country="KR" style:language-asian="ko" style:country-asian="KR"/></style:default-style>
<style:style style:name="Standard" style:family="paragraph" style:class="text"/>
<style:style style:name="Verse" style:family="paragraph" style:parent-style-name="Standard">
 <style:paragraph-properties fo:margin-left="0.45cm" fo:text-indent="-0.45cm" fo:margin-bottom="0.04cm" fo:line-height="135%" fo:text-align="justify"/></style:style>
<style:style style:name="Heading_20_1" style:display-name="Heading 1" style:family="paragraph" style:default-outline-level="1" style:class="text">
 <style:paragraph-properties fo:text-align="center" fo:margin-top="1.2cm" fo:margin-bottom="0.5cm" fo:break-before="page"/>
 <style:text-properties style:font-name="{FS}" style:font-name-asian="{FS}" fo:font-size="20pt" style:font-size-asian="20pt" fo:font-weight="bold" style:font-weight-asian="bold"/></style:style>
<style:style style:name="Heading_20_2" style:display-name="Heading 2" style:family="paragraph" style:default-outline-level="2" style:class="text">
 <style:paragraph-properties fo:text-align="center" fo:margin-top="0.3cm" fo:margin-bottom="0.15cm" fo:keep-with-next="always" fo:background-color="#e8e8e8"/>
 <style:text-properties style:font-name="{FS}" style:font-name-asian="{FS}" fo:font-size="10.5pt" style:font-size-asian="10.5pt" fo:font-weight="bold" style:font-weight-asian="bold"/></style:style>
<style:style style:name="Header" style:family="paragraph" style:class="extra"><style:paragraph-properties fo:text-align="end" fo:border-bottom="0.5pt solid #000000" fo:padding-bottom="0.05cm"/>
 <style:text-properties fo:font-size="8pt" style:font-size-asian="8pt"/></style:style>
<style:style style:name="Footer" style:family="paragraph" style:class="extra"><style:paragraph-properties fo:text-align="center"/>
 <style:text-properties fo:font-size="8pt" style:font-size-asian="8pt"/></style:style>
<style:style style:name="Title" style:family="paragraph"><style:paragraph-properties fo:text-align="center" fo:margin-top="7cm"/>
 <style:text-properties style:font-name="{FS}" style:font-name-asian="{FS}" fo:font-size="32pt" style:font-size-asian="32pt" fo:font-weight="bold" style:font-weight-asian="bold"/></style:style>
<style:style style:name="Sub" style:family="paragraph"><style:paragraph-properties fo:text-align="center" fo:margin-top="0.5cm"/>
 <style:text-properties fo:font-size="12pt" style:font-size-asian="12pt"/></style:style>
<style:style style:name="Small" style:family="paragraph"><style:paragraph-properties fo:text-align="center" fo:margin-top="9cm" fo:break-after="page"/>
 <style:text-properties fo:font-size="8pt" style:font-size-asian="8pt"/></style:style>
<style:style style:name="VN" style:family="text"><style:text-properties fo:font-weight="bold" style:font-weight-asian="bold" fo:color="#1f3c88" fo:font-size="8pt" style:font-size-asian="8pt"/></style:style>
'''
auto = '''
<style:page-layout style:name="pm1"><style:page-layout-properties fo:page-width="18.2cm" fo:page-height="25.7cm"
 fo:margin-top="1.2cm" fo:margin-bottom="1.2cm" fo:margin-left="1.8cm" fo:margin-right="1.5cm"/>
 <style:header-style><style:header-footer-properties fo:min-height="0.6cm" fo:margin-bottom="0.3cm"/></style:header-style>
 <style:footer-style><style:header-footer-properties fo:min-height="0.5cm" fo:margin-top="0.3cm"/></style:footer-style></style:page-layout>
<style:style style:name="Sect" style:family="section"><style:section-properties text:dont-balance-text-columns="false">
 <style:columns fo:column-count="2" fo:column-gap="0.7cm"><style:column-sep style:width="0.02cm" style:color="#555555" style:height="100%"/></style:columns>
 </style:section-properties></style:style>
'''
master = '''<style:master-page style:name="Standard" style:page-layout-name="pm1">
<style:header><text:p text:style-name="Header"><text:chapter text:display="name" text:outline-level="1"/></text:p></style:header>
<style:footer><text:p text:style-name="Footer">- <text:page-number text:select-page="current"/> -</text:p></style:footer>
</style:master-page>'''
books = usfm.load(SRC)
body = ['<text:p text:style-name="Title">신약전서</text:p><text:p text:style-name="Sub">한국어 성경 (1910)</text:p>',
        '<text:p text:style-name="Small">The Holy Bible in Korean, 1910 translation. Public Domain. 출처: eBible.org (kor)</text:p>']
for i, bk in enumerate(usfm.ordered(books, usfm.NT)):
    body.append('<text:h text:style-name="Heading_20_1" text:outline-level="1">%s</text:h>' % E(bk.long_name.strip()))
    body.append('<text:section text:style-name="Sect" text:name="Book%d">' % (i + 1))
    for b in bk.blocks:
        if b[0] == 'c':
            body.append('<text:h text:style-name="Heading_20_2" text:outline-level="2">제 %s 장</text:h>' % E(b[1]))
        elif b[0] == 'p':
            out = []
            for it in b[2]:
                if it[0] == 'v':
                    if out:
                        body.append('<text:p text:style-name="Verse">%s</text:p>' % ''.join(out)); out = []
                    out.append('<text:span text:style-name="VN">%s</text:span> ' % E(it[1]))
                elif it[0] == 't':
                    t = it[1].strip() if out and out[-1].endswith('</text:span> ') else it[1]
                    out.append(text_with_spaces(t))
            if out:
                body.append('<text:p text:style-name="Verse">%s</text:p>' % ''.join(out))
    body.append('</text:section>')
doc = fodt.document([F, FS], styles, auto, master, '\n'.join(body), title='신약전서 (한국어 성경 1910)', lang='ko', country='KR')
src = os.path.splitext(OUT)[0] + '.fodt'
open(src, 'w', encoding='utf-8').write(doc)
subprocess.run(['soffice', '-env:UserInstallation=file://' + PROFILE, '--headless', '--convert-to', 'pdf',
                '--outdir', os.path.dirname(OUT), src], check=True)
os.remove(src)
print('ok')
