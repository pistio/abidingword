"""Minimal flat-ODT (FODT) writer shared by the LibreOffice-based generators."""
import html

NS = ('xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" '
      'xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0" '
      'xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" '
      'xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0" '
      'xmlns:svg="urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0" '
      'xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0" '
      'xmlns:dc="http://purl.org/dc/elements/1.1/" '
      'xmlns:meta="urn:oasis:names:tc:opendocument:xmlns:meta:1.0"')


def E(t):
    return html.escape(t, quote=False)


def text_with_spaces(t):
    """Escape and keep runs of spaces (ODF collapses them otherwise)."""
    out = []
    for i, ch in enumerate(t):
        if ch == ' ' and i > 0 and t[i - 1] == ' ':
            out.append('<text:s/>')
        elif ch == '\t':
            out.append('<text:tab/>')
        else:
            out.append(E(ch))
    return ''.join(out)


def document(fonts, styles, auto_styles, master, body, title='', lang='en', country='US'):
    ff = ''.join('<style:font-face style:name="%s" svg:font-family="&apos;%s&apos;"/>' % (f, f) for f in fonts)
    return ('<?xml version="1.0" encoding="UTF-8"?>\n<office:document %s office:version="1.3" '
            'office:mimetype="application/vnd.oasis.opendocument.text">'
            '<office:meta><dc:title>%s</dc:title><dc:language>%s-%s</dc:language></office:meta>'
            '<office:font-face-decls>%s</office:font-face-decls>'
            '<office:styles>%s</office:styles><office:automatic-styles>%s</office:automatic-styles>'
            '<office:master-styles>%s</office:master-styles>'
            '<office:body><office:text>%s</office:text></office:body></office:document>\n') % (
        NS, E(title), lang, country, ff, styles, auto_styles, master, body)
