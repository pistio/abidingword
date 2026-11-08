# A2.1: what PDFium's characters needed before the layout code could use them

The layout code (`pdf_layout.rs`) was written for pdf-extract's glyphs: an
origin, an advance (the font's width for the glyph), a size and the font
width. It finds words from the gaps between advances, and lines and columns
from where the glyphs stand. PDFium's text page answers a different
question: it gives text for searching and copying. It infers spaces and line
ends itself, and keeps some characters it infers apart from what is drawn.

Every difference below was found by comparing the two engines **glyph by
glyph**:

* `pxglyphs`, a scratch tool, not committed, prints the glyphs pdf-extract
  gives the `Collector`, computed with the `Collector`'s own formulas;
* the helper's record stream;
* glyphs matched by character and origin (rounded to 0.1 pt).

The comparison used every seventh page of a document. Each fix is in
`tools/pdfium-glyphs/src/main.rs` and is described at its top.

| # | Found on | What PDFium does | Effect through the layout | What the helper does now |
|---|---|---|---|---|
| 1 | kjv-pdf ("top **ofthe** hill") | Its loose box follows the glyph's outline and runs past the advance of overhanging letters ("f" 0.439 em instead of 0.369 em). | Word gaps closed: kjv-pdf verified 30 868 → 15 016, and whole books lost. | The advance is the font's own width for the character (`FPDFFont_GetGlyphWidth`), as pdf-extract takes it. The loose box is used only when the font gives no width. |
| 2 | kjv-pdf ("**fi rst**") | It expands a ligature into its letters, all at one origin. | False gaps inside words. | Letters at one origin are one glyph, with their advances together. Asking the font for U+FB00–FB04 was tried and rejected: 401 lines got worse. |
| 3 | F02 page 298 | It marks some of the document's **own** spaces "generated" (130 of 512 on that page); they keep their text object. | Skipping every generated character dropped them. F02 verified **8 verses with wrong text**, the other column's verse numbers run into them. | A generated character is left out only when no text object stands behind it. |
| 4 | F02 ("C H A P T E R") | It also infers spaces between the letters of a letter-spaced heading; these carry a text object too, but stand at the very origin of the next letter. | Joined to the next letter (" E"): chapter headings were lost, F02 verified 1 751. | A generated character at the next character's origin is left out. |
| 5 | engbsb-rwpdf2col | A space it marks generated comes with an identity matrix (scale 1 instead of 0.75). | Spaces 0.79 pt too long; 2 verses verified with wrong text. | Such a space takes the matrix of the character drawn before it in the same text object. |
| 6 | F02 ("Beth-") | It reports a hyphen that ends a line as U+0002. | The hyphen was lost. | Written as "-" when a text object stands behind it. |
| 7 | hboWLC-pdf, G10 (Hebrew) | It gives right-to-left text in reading order (its own bidi reordering). Where the font maps one glyph to a letter and its point, it splits them at one origin and gives the point an advance. It adds spaces at a mark's origin without marking them generated. | Scrambled Hebrew: hboWLC-pdf text coverage 2.2 % (current path 65.9 %). Rule 2's join had attached points to the wrong letter. | A text object running right to left is put back in drawing order, a letter before its marks. A letter and the marks at its origin are one glyph, with the letter's advance; a mark alone has no advance. A space at the previous character's origin, or a generated one the font gives no width, is left out. The ligature join is limited to ASCII letters. |

After these fixes, **every matched glyph ends within 0.1 pt of
pdf-extract's** on the sampled pages of F02 (457 583 glyphs) and engbsb
(609 090), with the same glyph count for engbsb.

Hebrew is not glyph-identical: on hboWLC-pdf's sampled pages 283 890 of
pdf-extract's 331 310 glyphs match, and PDFium gives 368 408. Through the
layout, though, its Hebrew is better than the current path's: text
coverage is 99.4 % against 65.9 % (`measurements.md`).

## What is not reproduced

**Word spacing.** pdf-extract adds the text state's word spacing (`Tw`) to
a space's advance; it does not add the offsets of a `TJ` array. A line
justified either way looks the same from the glyphs' positions. PDFium
exposes neither `Tw` nor the `TJ` offsets per character. Measuring the gap
after spaces from positions was tried; it made engbsb (justified by `TJ`)
worse, so it was removed.

Documents justified with `Tw` therefore have shorter spaces through PDFium
than through pdf-extract:

* F03: 13.5 % of glyph ends differ, all spaces; −4 741 verified verses;
* engwebu-rl2col: all spaces; −192.

Reading `Tw` from the content stream, or a PDFium interface that gives
each character's advance as drawn, would be needed.

**Line-final spaces.** PDFium replaces a space that ends a line with its own
line break, so that space is gone (F02: 67 on page 298). The comparison
showed it does not change F02's layout.
