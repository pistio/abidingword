"""G08: render the Reina-Valera 1862 New Testament (Project Gutenberg #5879 text) as a
LibreOffice flat ODF text document written by hand, then saved as Word .docx by
LibreOffice Writer itself (soffice --convert-to docx, "MS Word 2007 XML" filter).
Layout: A5 pages, one column; each verse its own paragraph with a hanging indent and the
verse number in bold at the left; chapter headings "CAPITULO 5." centred in small capitals;
upper-case book titles centred; closing subscriptions of the epistles in italics; header
"EL NUEVO TESTAMENTO", page number field in the footer; the valera1909.com notice and
introduction (which the source asks to keep) on the first pages.
Usage: python3 -I render_g08_rv1862_lo.py REF.zip PG5879.txt OUT.docx WORKDIR
"""
import html
import os
import re
import shutil
import subprocess
import sys

sys.path.insert(0, "/root/aw-set-g/tools")
import usfmlib as U  # noqa: E402

HEAD = """<?xml version="1.0" encoding="UTF-8"?>
<office:document xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
 xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0"
 xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0"
 xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0"
 xmlns:dc="http://purl.org/dc/elements/1.1/"
 xmlns:meta="urn:oasis:names:tc:opendocument:xmlns:meta:1.0"
 office:version="1.3" office:mimetype="application/vnd.oasis.opendocument.text">
<office:meta><dc:title>El Nuevo Testamento (Reina-Valera 1862)</dc:title></office:meta>
<office:font-face-decls>
 <style:font-face style:name="Liberation Serif" svg:font-family="'Liberation Serif'" xmlns:svg="urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0"/>
</office:font-face-decls>
<office:styles>
 <style:default-style style:family="paragraph"><style:paragraph-properties fo:hyphenation-ladder-count="no-limit"/>
  <style:text-properties style:font-name="Liberation Serif" fo:font-size="9.5pt" fo:language="es" fo:country="ES" fo:hyphenate="true"/></style:default-style>
 <style:style style:name="Standard" style:family="paragraph"/>
 <style:style style:name="Verse" style:family="paragraph"><style:paragraph-properties fo:margin-left="0.55cm" fo:text-indent="-0.55cm" fo:margin-bottom="0.06cm" fo:text-align="justify"/></style:style>
 <style:style style:name="Chapter" style:family="paragraph"><style:paragraph-properties fo:text-align="center" fo:margin-top="0.3cm" fo:margin-bottom="0.15cm" fo:keep-with-next="always"/>
  <style:text-properties fo:font-variant="small-caps" fo:font-size="10pt" fo:letter-spacing="0.04cm"/></style:style>
 <style:style style:name="BookTitle" style:family="paragraph"><style:paragraph-properties fo:text-align="center" fo:margin-top="0.4cm" fo:margin-bottom="0.4cm" fo:break-before="page" fo:keep-with-next="always"/>
  <style:text-properties fo:font-size="12.5pt" fo:font-weight="bold"/></style:style>
 <style:style style:name="Colophon" style:family="paragraph"><style:paragraph-properties fo:text-align="center" fo:margin-top="0.25cm"/>
  <style:text-properties fo:font-style="italic" fo:font-size="8.5pt"/></style:style>
 <style:style style:name="Front" style:family="paragraph"><style:paragraph-properties fo:text-align="justify" fo:margin-bottom="0.12cm"/>
  <style:text-properties fo:font-size="8.5pt"/></style:style>
 <style:style style:name="FrontTitle" style:family="paragraph"><style:paragraph-properties fo:text-align="center" fo:margin-top="2cm" fo:margin-bottom="0.6cm"/>
  <style:text-properties fo:font-size="16pt" fo:font-weight="bold"/></style:style>
 <style:style style:name="Header" style:family="paragraph"><style:paragraph-properties fo:text-align="center"/><style:text-properties fo:font-size="7.5pt" fo:letter-spacing="0.05cm"/></style:style>
 <style:style style:name="Footer" style:family="paragraph"><style:paragraph-properties fo:text-align="center"/><style:text-properties fo:font-size="8pt"/></style:style>
 <style:style style:name="VN" style:family="text"><style:text-properties fo:font-weight="bold" fo:font-size="8pt"/></style:style>
</office:styles>
<office:automatic-styles>
 <style:page-layout style:name="pm1"><style:page-layout-properties fo:page-width="14.8cm" fo:page-height="21cm" fo:margin-top="1.2cm" fo:margin-bottom="1.1cm" fo:margin-left="1.6cm" fo:margin-right="1.4cm"/>
  <style:header-style><style:header-footer-properties fo:min-height="0.5cm" fo:margin-bottom="0.3cm"/></style:header-style>
  <style:footer-style><style:header-footer-properties fo:min-height="0.5cm" fo:margin-top="0.3cm"/></style:footer-style></style:page-layout>
</office:automatic-styles>
<office:master-styles>
 <style:master-page style:name="Standard" style:page-layout-name="pm1">
  <style:header><text:p text:style-name="Header">EL NUEVO TESTAMENTO</text:p></style:header>
  <style:footer><text:p text:style-name="Footer"><text:page-number text:select-page="current">1</text:page-number></text:p></style:footer>
 </style:master-page>
</office:master-styles>
<office:body><office:text>
"""


def x(s):
    return html.escape(s, quote=False)


def main(ref, pgtxt, out_docx, work):
    os.makedirs(work, exist_ok=True)
    books = U.read_usfm_zip(ref)
    t = open(pgtxt, encoding="utf-8").read().replace("\r\n", "\n")
    notice = t[t.index("[Empieza Aviso e Introducción]"):t.index("[End Notice and Introduction]") + len("[End Notice and Introduction]")]
    body = [HEAD, '<text:p text:style-name="FrontTitle">EL NUEVO TESTAMENTO DE NUESTRO SEÑOR JESU-CRISTO</text:p>',
            '<text:p text:style-name="Chapter">Antigua versión de Cipriano de Valera, revisión de 1862</text:p>']
    for para in re.split(r"\n\s*\n", notice):
        p = U.clean_ws(para)
        if p:
            body.append(f'<text:p text:style-name="Front">{x(p)}</text:p>')
    for b in books:
        body.append(f'<text:p text:style-name="BookTitle">{x(b["headers"].get("mt1", b["headers"].get("mt", "")))}</text:p>')
        nchap = sum(1 for it in b["items"] if it[0] == "c")
        for it in b["items"]:
            if it[0] == "c":
                if nchap > 1:
                    body.append(f'<text:p text:style-name="Chapter">CAPITULO {it[1]}.</text:p>')
            elif it[0] == "v":
                txt = re.sub(r"\x01\d+\x01", "", it[2])
                body.append(f'<text:p text:style-name="Verse"><text:span text:style-name="VN">{it[1]}</text:span><text:tab/>{x(txt)}</text:p>')
            elif it[0] == "s":
                body.append(f'<text:p text:style-name="Colophon">{x(it[1])}</text:p>')
    body.append("</office:text></office:body></office:document>\n")
    fodt = os.path.join(work, "G08_rv1862.fodt")
    open(fodt, "w", encoding="utf-8").write("\n".join(body))
    prof = os.path.abspath(os.path.join(work, "lo_profile"))
    subprocess.run(["soffice", f"-env:UserInstallation=file://{prof}", "--headless", "--convert-to",
                    "docx:MS Word 2007 XML", "--outdir", work, fodt], check=True, stdin=subprocess.DEVNULL,
                   stdout=open(os.path.join(work, "soffice.log"), "w"), stderr=subprocess.STDOUT, timeout=600)
    shutil.copyfile(os.path.join(work, "G08_rv1862.docx"), out_docx)
    print("written", out_docx)


if __name__ == "__main__":
    main(*sys.argv[1:5])
