"""G10: render the Five Scrolls and Jonah from Miqra according to the Masorah (MAM) as a
right-to-left LibreOffice flat ODF document written by hand, exported to PDF by
LibreOffice Writer (writer_pdf_Export).
The book data come from conv_mapm_osis.py (run again here; its USFM output must equal the
reference zip byte for byte), so the renderer can print the edition's own Hebrew-letter
chapter and verse numbers and its poetic line divisions.
Layout: 17 x 24 cm pages, right-to-left, Noto Serif Hebrew with full pointing and
cantillation; Hebrew book name as running head, page number in the footer; book titles
large and centred; chapter headings "פרק ג" ; verse numbers as Hebrew letters (א, ב ... יא)
in small bold type before each verse; prose in justified paragraphs, poetry (Song of
Songs, Lamentations and poetic passages) one line per stich with indented second halves;
the written form (ketiv) in parentheses before the read form, MAM's own convention.
Usage: python3 -I render_g10_mam_lo_pdf.py MAPM.xml REF.zip OUT.pdf WORKDIR
"""
import filecmp
import html
import os
import shutil
import subprocess
import sys
import zipfile

sys.path.insert(0, "/root/aw-set-g/tools")
import conv_mapm_osis as C  # noqa: E402

HEB_ORDER = ["Song", "Ruth", "Lam", "Eccl", "Esth", "Jonah"]
CODE = {"Song": "SNG", "Ruth": "RUT", "Lam": "LAM", "Eccl": "ECC", "Esth": "EST", "Jonah": "JON"}

HEAD = """<?xml version="1.0" encoding="UTF-8"?>
<office:document xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
 xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0"
 xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0"
 xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0"
 xmlns:svg="urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0"
 xmlns:dc="http://purl.org/dc/elements/1.1/"
 office:version="1.3" office:mimetype="application/vnd.oasis.opendocument.text">
<office:meta><dc:title>חמש מגילות ויונה — מקרא על פי המסורה</dc:title></office:meta>
<office:font-face-decls><style:font-face style:name="Noto Serif Hebrew" svg:font-family="'Noto Serif Hebrew'"/></office:font-face-decls>
<office:styles>
 <style:default-style style:family="paragraph">
  <style:paragraph-properties style:writing-mode="rl-tb" fo:text-align="start"/>
  <style:text-properties style:font-name-complex="Noto Serif Hebrew" style:font-size-complex="12.5pt" style:language-complex="he" style:country-complex="IL" style:font-name="Noto Serif Hebrew" fo:font-size="12pt"/>
 </style:default-style>
 <style:style style:name="Prose" style:family="paragraph"><style:paragraph-properties style:writing-mode="rl-tb" fo:text-align="justify" fo:margin-bottom="0.15cm" fo:line-height="135%"/></style:style>
 <style:style style:name="Poem1" style:family="paragraph"><style:paragraph-properties style:writing-mode="rl-tb" fo:text-align="start" fo:margin-right="0cm" fo:margin-bottom="0cm" fo:line-height="130%"/></style:style>
 <style:style style:name="Poem2" style:family="paragraph"><style:paragraph-properties style:writing-mode="rl-tb" fo:text-align="start" fo:margin-right="1.2cm" fo:margin-bottom="0cm" fo:line-height="130%"/></style:style>
 <style:style style:name="Poem3" style:family="paragraph"><style:paragraph-properties style:writing-mode="rl-tb" fo:text-align="start" fo:margin-right="2.2cm" fo:margin-bottom="0cm" fo:line-height="130%"/></style:style>
 <style:style style:name="Chapter" style:family="paragraph"><style:paragraph-properties style:writing-mode="rl-tb" fo:text-align="center" fo:margin-top="0.35cm" fo:margin-bottom="0.15cm" fo:keep-with-next="always"/>
  <style:text-properties style:font-size-complex="14pt" style:font-weight-complex="bold"/></style:style>
 <style:style style:name="Book" style:family="paragraph"><style:paragraph-properties style:writing-mode="rl-tb" fo:text-align="center" fo:break-before="page" fo:margin-top="0.6cm" fo:margin-bottom="0.5cm" fo:keep-with-next="always"/>
  <style:text-properties style:font-size-complex="24pt" style:font-weight-complex="bold"/></style:style>
 <style:style style:name="Front" style:family="paragraph"><style:paragraph-properties style:writing-mode="rl-tb" fo:text-align="center" fo:margin-bottom="0.3cm"/></style:style>
 <style:style style:name="FrontLatin" style:family="paragraph"><style:paragraph-properties style:writing-mode="lr-tb" fo:text-align="center" fo:margin-top="1cm"/><style:text-properties fo:font-size="8pt" style:font-name="Noto Serif Hebrew"/></style:style>
 <style:style style:name="Header" style:family="paragraph"><style:paragraph-properties style:writing-mode="rl-tb" fo:text-align="center"/><style:text-properties style:font-size-complex="9pt"/></style:style>
 <style:style style:name="Footer" style:family="paragraph"><style:paragraph-properties fo:text-align="center"/><style:text-properties fo:font-size="9pt"/></style:style>
 <style:style style:name="VN" style:family="text"><style:text-properties style:font-size-complex="8.5pt" style:font-weight-complex="bold" fo:color="#7a1f1f"/></style:style>
 <style:style style:name="Title" style:family="text"><style:text-properties style:font-size-complex="30pt" style:font-weight-complex="bold"/></style:style>
</office:styles>
<office:automatic-styles>
 <style:page-layout style:name="pm1"><style:page-layout-properties fo:page-width="17cm" fo:page-height="24cm" fo:margin-top="1.5cm" fo:margin-bottom="1.4cm" fo:margin-left="1.8cm" fo:margin-right="1.8cm" style:writing-mode="rl-tb"/>
  <style:header-style><style:header-footer-properties fo:min-height="0.6cm" fo:margin-bottom="0.3cm"/></style:header-style>
  <style:footer-style><style:header-footer-properties fo:min-height="0.5cm" fo:margin-top="0.3cm"/></style:footer-style></style:page-layout>
</office:automatic-styles>
<office:master-styles>
 <style:master-page style:name="Standard" style:page-layout-name="pm1">
  <style:header><text:p text:style-name="Header"><text:chapter text:display="name" text:outline-level="1"/></text:p></style:header>
  <style:footer><text:p text:style-name="Footer"><text:page-number text:select-page="current">1</text:page-number></text:p></style:footer>
 </style:master-page>
</office:master-styles>
<office:body><office:text>
"""


def x(s):
    return html.escape(s, quote=False)


def main(mapm, ref, out_pdf, work):
    os.makedirs(work, exist_ok=True)
    check_zip = os.path.join(work, "reconverted.zip")
    books = C.main(mapm, check_zip, os.path.join(work, "reconvert_report.txt"), ",".join(HEB_ORDER))
    za, zb = zipfile.ZipFile(check_zip), zipfile.ZipFile(ref)
    for n in zb.namelist():
        if za.read(n) != zb.read(n):
            raise SystemExit("reference differs from re-conversion: " + n)
    bymap = {b["code"]: b for b in books}
    out = [HEAD,
           '<text:p text:style-name="Front"><text:span text:style-name="Title">חמש מגילות</text:span></text:p>',
           '<text:p text:style-name="Front">שיר השירים · רות · איכה · קהלת · אסתר · ויונה</text:p>',
           '<text:p text:style-name="Front">מקרא על פי המסורה</text:p>',
           '<text:p text:style-name="FrontLatin">Miqra according to the Masorah, ed. Seth (Avi) Kadish; OSIS (MAPM) by David Troidl, '
           'openscriptures/morphhb. Licence: CC BY-SA 3.0. Layout: LibreOffice Writer.</text:p>']
    for osis in HEB_ORDER:
        b = bymap[CODE[osis]]
        out.append(f'<text:h text:style-name="Book" text:outline-level="1">{x(b["mt"][0])}</text:h>')
        ch = 0
        prose_open = False
        poetic = any(b["lines"].get(i) and any(m for m, _ in b["lines"][i]) for i in b["lines"])
        for idx, it in enumerate(b["items"]):
            if it[0] == "c":
                if prose_open:
                    out.append("</text:p>")
                    prose_open = False
                ch = it[1]
                out.append(f'<text:p text:style-name="Chapter">פרק {x(b["heb"][("c", ch)])}</text:p>')
            elif it[0] == "p":
                if prose_open:
                    out.append("</text:p>")
                    prose_open = False
            elif it[0] == "v":
                vn = b["heb"][("v", ch, it[1])]
                lines = b["lines"].get(idx, [(None, it[2])])
                marks = [m for m, _ in lines if m]
                if marks and any(m.startswith("q") for m in marks):
                    if prose_open:
                        out.append("</text:p>")
                        prose_open = False
                    first = True
                    for m, seg in lines:
                        seg = seg.strip().replace("( ", "(").replace(" )", ")")
                        if not seg:
                            continue
                        lvl = int(m[1]) if (m and m.startswith("q")) else 1
                        num = f'<text:span text:style-name="VN">{x(vn)}</text:span> ' if first else ""
                        out.append(f'<text:p text:style-name="Poem{min(lvl, 3)}">{num}{x(seg)}</text:p>')
                        first = False
                else:
                    if not prose_open:
                        out.append('<text:p text:style-name="Prose">')
                        prose_open = True
                    out.append(f'<text:span text:style-name="VN">{x(vn)}</text:span> {x(it[2])} ')
        if prose_open:
            out.append("</text:p>")
    out.append("</office:text></office:body></office:document>\n")
    fodt = os.path.join(work, "G10_mam.fodt")
    open(fodt, "w", encoding="utf-8").write("".join(out))
    prof = os.path.abspath(os.path.join(work, "lo_profile"))
    subprocess.run(["soffice", f"-env:UserInstallation=file://{prof}", "--headless", "--convert-to",
                    "pdf:writer_pdf_Export", "--outdir", os.path.abspath(work), os.path.abspath(fodt)], check=True,
                   stdin=subprocess.DEVNULL, stdout=open(os.path.join(work, "soffice.log"), "w"),
                   stderr=subprocess.STDOUT, timeout=900)
    shutil.copyfile(os.path.join(work, "G10_mam.pdf"), out_pdf)
    print("written", out_pdf)


if __name__ == "__main__":
    main(*sys.argv[1:5])
