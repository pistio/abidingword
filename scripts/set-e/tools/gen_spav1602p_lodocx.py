"""spav1602p (Valera 1602 Purificada) Salmos, Proverbios, Eclesiastés, Cantares -> FODT -> .docx via LibreOffice.
A5 portrait, single column, 'SALMO N' / 'CAPÍTULO N' headings, Psalm titles in italic small type,
verse numbers hung in a left gutter (number, tab, hanging indent), running head = book via chapter field,
page number at bottom right, ALEF/BET acrostic headings."""
import sys, os, subprocess
sys.path.insert(0, '/root/aw-set-e/tools')
import usfm, fodt
from fodt import E, text_with_spaces

SRC, OUT, PROFILE = sys.argv[1], sys.argv[2], sys.argv[3]
F = 'Liberation Serif'
styles = f'''
<style:default-style style:family="paragraph"><style:text-properties style:font-name="{F}" fo:font-size="10pt" fo:language="es" fo:country="ES"/></style:default-style>
<style:style style:name="Standard" style:family="paragraph" style:class="text"/>
<style:style style:name="Versiculo" style:family="paragraph" style:parent-style-name="Standard">
 <style:paragraph-properties fo:margin-left="0.7cm" fo:text-indent="-0.7cm" fo:margin-bottom="0.05cm" fo:text-align="justify">
 <style:tab-stops><style:tab-stop style:position="0cm"/></style:tab-stops></style:paragraph-properties></style:style>
<style:style style:name="Heading_20_1" style:display-name="Heading 1" style:family="paragraph" style:default-outline-level="1" style:class="text">
 <style:paragraph-properties fo:text-align="center" fo:margin-top="1.5cm" fo:margin-bottom="0.6cm" fo:break-before="page"/>
 <style:text-properties fo:font-size="18pt" fo:font-weight="bold" fo:font-variant="small-caps"/></style:style>
<style:style style:name="Heading_20_2" style:display-name="Heading 2" style:family="paragraph" style:default-outline-level="2" style:class="text">
 <style:paragraph-properties fo:text-align="center" fo:margin-top="0.35cm" fo:margin-bottom="0.1cm" fo:keep-with-next="always"/>
 <style:text-properties fo:font-size="10.5pt" fo:font-weight="bold" fo:letter-spacing="0.05cm"/></style:style>
<style:style style:name="Heading_20_3" style:display-name="Heading 3" style:family="paragraph" style:default-outline-level="3" style:class="text">
 <style:paragraph-properties fo:text-align="start" fo:margin-top="0.2cm" fo:keep-with-next="always"/>
 <style:text-properties fo:font-size="9pt" fo:font-weight="bold"/></style:style>
<style:style style:name="Titulo_20_del_20_Salmo" style:display-name="Titulo del Salmo" style:family="paragraph">
 <style:paragraph-properties fo:text-align="center" fo:margin-bottom="0.1cm" fo:keep-with-next="always"/>
 <style:text-properties fo:font-size="8.5pt" fo:font-style="italic"/></style:style>
<style:style style:name="Header" style:family="paragraph" style:class="extra"><style:paragraph-properties fo:text-align="center"/>
 <style:text-properties fo:font-size="8pt" fo:font-variant="small-caps"/></style:style>
<style:style style:name="Footer" style:family="paragraph" style:class="extra"><style:paragraph-properties fo:text-align="end"/>
 <style:text-properties fo:font-size="8pt"/></style:style>
<style:style style:name="Portada" style:family="paragraph"><style:paragraph-properties fo:text-align="center" fo:margin-top="4cm"/>
 <style:text-properties fo:font-size="22pt" fo:font-weight="bold"/></style:style>
<style:style style:name="Aviso" style:family="paragraph"><style:paragraph-properties fo:text-align="justify" fo:margin-top="5cm"/>
 <style:text-properties fo:font-size="7.5pt"/></style:style>
<style:style style:name="Num" style:family="text"><style:text-properties fo:font-weight="bold" fo:font-size="8pt"/></style:style>
<style:style style:name="Add" style:family="text"><style:text-properties fo:font-style="italic"/></style:style>
'''
auto = '''<style:page-layout style:name="pm1"><style:page-layout-properties fo:page-width="14.8cm" fo:page-height="21cm"
 fo:margin-top="1.2cm" fo:margin-bottom="1.2cm" fo:margin-left="1.6cm" fo:margin-right="1.4cm"/>
 <style:header-style><style:header-footer-properties fo:min-height="0.5cm" fo:margin-bottom="0.3cm"/></style:header-style>
 <style:footer-style><style:header-footer-properties fo:min-height="0.5cm" fo:margin-top="0.3cm"/></style:footer-style></style:page-layout>'''
master = '''<style:master-page style:name="Standard" style:page-layout-name="pm1">
<style:header><text:p text:style-name="Header"><text:chapter text:display="name" text:outline-level="1"/></text:p></style:header>
<style:footer><text:p text:style-name="Footer"><text:page-number text:select-page="current"/></text:p></style:footer></style:master-page>'''
books = usfm.load(SRC)
body = ['<text:p text:style-name="Portada">Los Libros Poéticos</text:p>',
        '<text:p text:style-name="Standard">Salmos · Proverbios · Eclesiastés · Cantar de los Cantares</text:p>',
        '<text:p text:style-name="Standard">Santa Biblia Valera 1602 Purificada</text:p>',
        '<text:p text:style-name="Aviso">© 2007, 2019, 2024 Iglesia Bautista Bíblica de la Gracia. Todos los derechos reservados conforme a la ley. '
        'Se puede imprimir sin alterar su contenido, esto además prohibe añadir, aumentar, quitar o disminuir letras, palabras, signos de '
        'puntuación o cualesquiera de los caracteres contenidos en esta obra. Prohibida su reproducción con fines de lucro. Fuente: eBible.org (spav1602p).</text:p>']


def inl(items):
    out = []
    for it in items:
        if it[0] == 't':
            t = text_with_spaces(it[1])
            if it[2] & {'add', 'it', 'em'}:
                t = '<text:span text:style-name="Add">%s</text:span>' % t
            out.append(t)
    return ''.join(out).strip()


for bk in usfm.ordered(books, ['PSA', 'PRO', 'ECC', 'SNG']):
    body.append('<text:h text:style-name="Heading_20_1" text:outline-level="1">%s</text:h>' % E(bk.long_name.strip()))
    for b in bk.blocks:
        if b[0] == 'c':
            lab = ('SALMO %s' if bk.code == 'PSA' else 'CAPÍTULO %s') % b[1]
            body.append('<text:h text:style-name="Heading_20_2" text:outline-level="2">%s</text:h>' % lab)
        elif b[0] == 'h':
            if b[1] == 'd':
                body.append('<text:p text:style-name="Titulo_20_del_20_Salmo">%s</text:p>' % inl(b[2]))
            else:
                body.append('<text:h text:style-name="Heading_20_3" text:outline-level="3">%s</text:h>' % inl(b[2]))
        elif b[0] == 'p':
            cur_v = None; buf = []
            segs = []
            for it in b[2]:
                if it[0] == 'v':
                    if cur_v is not None or buf:
                        segs.append((cur_v, buf))
                    cur_v, buf = it[1], []
                else:
                    buf.append(it)
            segs.append((cur_v, buf))
            for v, items in segs:
                txt = inl(items)
                if v is None and not txt:
                    continue
                num = '<text:span text:style-name="Num">%s</text:span><text:tab/>' % E(v) if v else '<text:tab/>'
                body.append('<text:p text:style-name="Versiculo">%s%s</text:p>' % (num, txt))
doc = fodt.document([F], styles, auto, master, '\n'.join(body), title='Los Libros Poéticos (Valera 1602 Purificada)', lang='es', country='ES')
src = os.path.splitext(OUT)[0] + '.fodt'
open(src, 'w', encoding='utf-8').write(doc)
subprocess.run(['soffice', '-env:UserInstallation=file://' + PROFILE, '--headless', '--convert-to', 'docx:MS Word 2007 XML',
                '--outdir', os.path.dirname(OUT), src], check=True)
os.remove(src)
print('ok')
