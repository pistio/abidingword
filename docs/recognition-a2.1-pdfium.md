# Scripture recognition, Phase A2.1: PDFium glyphs through the app's own layout

**Status:** isolated prototype for the owner's review. **The current PDF
backend (`pdf::read_pages`: pdf-extract and lopdf) is unchanged, still the
only one the application uses, and the fallback.** Nothing in the
application calls the new code; it is reached only through the test
harnesses. There is no migration, no change to `src-tauri/Cargo.lock`, no
integration with the Sources work, and no Windows production change.

All 57 PDFs are A2's disclosed, checksummed documents. These numbers are
regression evidence on known documents, not independent evaluation.

## 1. Commits (branch `claude/recognition-a2.1-pdfium-glyphs`, from A2 `c5d81e3`)

| SHA | What |
|---|---|
| `4e96dd0` | `PdfGlyphSource` / `GlyphSink`, `PdfiumHelper` (helper process, watchdog, memory cap, streaming), `tools/pdfium-glyphs`, harness switches, measurement script |
| `227a0a7` | Helper: the document's own spaces, line-final hyphens |
| `b795b5f` | Helper: inferred spaces left out, relabelled spaces measured as drawn |
| `a26af11` | Helper: right-to-left text and Hebrew marks as drawn |
| this commit | Isolation tests, clean timing, evidence (`docs/audit/a2.1/`), this report |

## 2. The interface

```rust
pub trait GlyphSink {            // pdf_layout.rs; the existing Collector is one
    fn begin_page(&mut self, number: u32, width: f64, height: f64);
    fn glyph(&mut self, glyph: SourceGlyph);   // text, x, y, end, size, width, upright, slanted
    fn end_page(&mut self);
}
pub trait PdfGlyphSource {       // pdf_glyph_source.rs
    fn read(&self, bytes: &[u8], file_name: &str, sink: &mut dyn GlyphSink)
        -> AppResult<GlyphReport>;               // pages, failed pages, truncation
}
```

* `SourceGlyph` is exactly what the layout code already keeps per glyph
  (`pdf_layout::Glyph`). A source therefore feeds the **same** layout,
  column, reading-order and recognition code as pdf-extract.
* `pdf::read_pages_with(bytes, name, &source)` gives a `PdfText` like
  `read_pages`.
* `documents::read_with` and `import::stage_documents_read` take a PDF
  reader; `read` and `stage_documents` delegate to them with the current
  backend, so their behaviour is unchanged.
* The current path through the refactored layout was re-run on all 57 PDFs:
  identical to v0.24.7 on the 56 with a baseline (G05 has none: the current
  backend cannot open it).

This is A2's proposed shared interface reduced to what the layout needs. It
should be agreed with Codex's Sources work **after** the current Windows
Source acceptance, before either side builds on it. No schema is involved.

## 3. Results on the 57 PDFs

Full tables, grouped as asked (G05, Hebrew, Korean, Greek, multi-column):
`docs/audit/a2.1/measurements.md`. Glyph-level findings:
`docs/audit/a2.1/glyph-fidelity.md`.

| Measure | Current backend | PDFium through the layout |
|---|---|---|
| Documents opened | 56 | **57** (G05 opens) |
| Text coverage (A2's scorer): same / better / worse than current | — | 54 / **2** / 0, plus G05 |
| Verified verses (sum) | 896 041 | **900 392** (+4 351) |
| Verified verses with wrong text | 9 | 9 (the same 9 verses, pre-existing) |
| Verified verses the edition does not have | 0 | 0 |
| Processing time, release build, the 56 PDFs both open (whole reading, all units) | 305 s | 318 s (+4 %); the helper alone 234 s |
| Peak memory, largest document (spavbl-rlofodt) / median | 571 MB / 194 MB | 550 MB / 151 MB for the app; the helper at most 64 MB (median 27 MB) at the same time |

**Where PDFium is better**

* F01: 0 → 8 541 verified.
* spavbl-rlofodt: 3 453 → 4 020.
* frajnd-pdf: 29 625 → 29 804.
* hboWLC-pdf: text coverage 65.9 % → 99.4 %, in order 99.4 %.
* G10 (Hebrew, pointed): 39.6 % → 73.4 %.
* G05: unreadable → 89.1 % of its text found, though no book is recognised
  (1545 German forms).

**Where it is worse**

* F03: 10 848 → 6 107 verified.
* engwebu-rl2col: 7 499 → 7 307.
* frajnd-lopdf: −4.

F03 and engwebu are justified with word spacing (`Tw`), which pdf-extract
adds to a space's advance and PDFium does not expose (§5). Their text is
found as completely; only verification suffers.

**Equal on every other document**, including all Korean, all Greek and the
multi-column eBible editions.

No document gains a wrong verified verse through PDFium.

## 4. What it took

A first measurement fed PDFium's text page to the layout as it comes.
kjv-pdf halved and F02 verified 8 verses with wrong text. Seven
differences between PDFium's text page and the glyphs the layout was built
for were found by comparing the two engines glyph by glyph, and each was
corrected in the helper (`glyph-fidelity.md`):

* advances from the font, not the loose box;
* expanded ligatures;
* the document's own spaces PDFium marks generated;
* spaces it infers inside letter-spaced headings;
* relabelled spaces' matrix;
* line-final hyphens;
* right-to-left order and Hebrew marks.

On the sampled pages of F02 and engbsb every glyph both engines read now
ends within 0.1 pt of pdf-extract's.

## 5. Unresolved risks

1. **Word spacing (`Tw`).** It is not available from PDFium per character,
   and a `TJ`-justified line looks the same. Documents justified with `Tw`
   lose verification (F03 −4 741). Fixing it needs either the text state
   read from the content stream, or a PDFium interface giving each
   character's drawn advance.
2. **PDFium's text page is built for search and copy, not drawing.** Every
   correction above restores drawing facts from its output by rules
   measured on these documents. Unseen producers may show differences the
   rules do not cover. A sealed blind PDF set would be needed before any
   claim.
3. **Hebrew is not glyph-identical** to pdf-extract (86 % of pdf-extract's
   glyphs matched on hboWLC). It is better through the layout, but not
   understood glyph for glyph.
4. **Windows.**
   * Cross-built but never run.
   * No Job Object memory cap yet.
   * No `CREATE_NO_WINDOW`.
   * Code signing and MSVC builds are not set up.
5. **Security.** PDFium's CVE history and advisories were not checked
   from here. A helper process limits the damage, it does not prevent it.
6. **Operational cost.** There are two more binaries to ship, sign and
   update (8 MB). The PDF is copied to a temporary file.

## 6. Recommendation

Do not switch backends. Keep pdf-extract as the backend and the fallback.

The prototype supports a **per-document fallback**, behind an
off-by-default setting, once reviewed with Codex: use PDFium glyphs only
when the current backend cannot open a PDF (G05) or reads no Scripture
from it (F01, F07). In this corpus that gains 8 542 verified verses and
G05's text, and loses nothing, because documents the current path reads
are untouched.

A wholesale switch would need:

* word spacing solved (F03, engwebu);
* the Windows Job Object, window and signing work;
* a sealed blind PDF set showing no new wrong verses.

## 7. How it was measured

* **Text fidelity and reading order:** `scripts/a2-pdf/a21_measure.py`. The
  app's unit dump for each engine is scored with A2's `score.py` against the
  edition's own text (Hebrew with points stripped, as in A2).
* **Verification:** the real-document harness, `AW_PDF_ENGINE=pdfium` for
  PDFium, on each document's corpus, from a release build.
* **Time and memory:** `scripts/a2-pdf/a21_clean_timing.py`. Each reading
  is started from a fresh, small process (`clean_run.py`), because a child
  forked from the scoring process inherits its resident size and its peak
  is inflated. The peak for PDFium is the larger of the app's and the
  helper's peaks (`wait4`), not their sum.
* **Glyph comparisons:** a scratch tool printing pdf-extract's glyphs as
  the `Collector` computes them (not committed). The helper's own output
  was compared with it.
* **Machine:** 4 cores, Linux, PDFium chromium/8138 (`PDFIUM.lock`).
