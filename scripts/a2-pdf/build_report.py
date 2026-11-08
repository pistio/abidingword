"""Assemble /root/a2-pdf/report.md from out/tables.json, out/results.json and the narrative below."""
import json, statistics as st
OUT = '/root/a2-pdf/out'
T = json.load(open(OUT + '/tables.json'))
R = json.load(open(OUT + '/results.json'))
docs = json.load(open(OUT + '/docs.json'))
base = json.load(open(OUT + '/baseline.json'))
nver = sum(1 for d in docs if d.get('verified'))
nchk = len(docs)
agg = T['struct_agg']
ATTR = json.load(open(OUT + '/attribution.json'))
n_attr = len(ATTR)
n_rec = sum(1 for a in ATTR if a['class'].startswith('RECOGNITION'))
n_ext = sum(1 for a in ATTR if a['class'].startswith('EXTRACTION'))
n_doc = sum(1 for a in ATTR if a['class'].startswith('DOCUMENT'))
rec_keys = ', '.join(a['key'] for a in ATTR if a['class'].startswith('RECOGNITION'))
ext_keys = ', '.join(a['key'] for a in ATTR if a['class'].startswith('EXTRACTION'))

def pct(x): return '–' if x is None else f'{100*x:.1f}'

chk_lines = []
for d in docs:
    src = ', '.join(f"{c['source']}={'OK' if c['match'] else 'MISMATCH'}" for c in d['checks'])
    chk_lines.append(f"| {d['key']} | `{d['pdf']}` | {d['size']:,} | `{d['sha256'][:16]}…` | {src} | {'OK' if d.get('ref_check') else ('n/a' if d.get('ref_check') is None else 'MISMATCH')} |")
chk = '\n'.join(['| key | file | bytes | sha256 | checks | reference zip |', '|---|---|---|---|---|---|'] + chk_lines)

rep = f"""# PDF text-extraction engines for Abiding Word — measured comparison

Date: 2026-11-08. Work directory `/root/a2-pdf` (scripts in `scripts/`, raw numbers in `out/results.json`).
Nothing under `/home/user/abidingword` was changed; the app was not built.

## 1. Short answer

* **Primary engine: PDFium** (via the `pdfium-render` crate, binding at run time to a prebuilt `pdfium.dll`
  from bblanchon/pdfium-binaries). On this corpus its text is as complete and as well ordered as the
  current extractor on every Latin, Greek and Korean document, it opened every file (including G05, which
  the current extractor cannot open), it survived corrupted xref offsets and a wrong `/Length`, it was
  a median 18× faster (3.7–63×) than the current extractor's test binary, at a median 56 MB peak RSS vs 162 MB, and it exposes every geometry/structure
  item the app might want. Its licence (BSD-3-Clause plus permissive third-party licences) allows
  closed-source redistribution with notices.
* **MuPDF** scored best overall (it is the only engine that reads the truncated files and heb-wprtl), but
  it is **AGPL-3.0** (or a paid Artifex commercial licence) — not usable in a closed-source app without
  buying a licence. **Poppler** is **GPL**; its default layout mode also scored worst on columns.
* **Keep the current pdf-extract/lopdf path as a fallback, but for a narrower reason than "it reads more"**:
  on these 57 PDFs there is no file that the current extractor opens and PDFium does not. What the current
  path has that raw PDFium text lacks is the app's own layout code (`pdf_layout.rs`), which is what gets
  G04/hbo-rwpdfrtl-rut/hboWLC (Hebrew) right. The sensible design is PDFium as the *glyph source* for the
  existing layout code (PDFium gives per-character Unicode, box, origin, font name and size), with the old
  path kept behind a switch until that combination has been measured — **this combination was not
  measured here** (it needs Rust work in the app).
* Most low-verification results in the app's audits are **recognition** problems, not extraction problems:
  of {n_attr} documents where the app verified < 50 % (or refused the file), the current extractor's text is
  complete and in order (≥ 85 % of verses found in order, or as good as any engine) in {n_rec}. Extraction is clearly at fault in
  G05 (cannot open), heb-wprtl and G10 (Hebrew), and in three files where the app's layout code moves
  inline/margin verse numbers out of their place in the line (G13, spabll-rrlab2col, engkjvcpb-rwpdfmargin).

## 2. What was measured and how

**Engines** (each run in its own process, one at a time):

| name here | engine and version | call |
|---|---|---|
| current | the app's extractor: pdf-extract 0.12.1 on lopdf 0.42 plus the app's layout code, via the frozen test binary `dump-ea5f944` (prints every unit in the app's reading order) | `AW_DUMP_FILE=… AW_DUMP_UNITS=0-10000000 dump-ea5f944 --ignored --nocapture` |
| pdfium | PDFium 153.0.7999.0 via pypdfium2 5.13.0 | `FPDFText_GetText` per page (`get_text_range()`) |
| mupdf | MuPDF 1.28.5 via PyMuPDF 2.0 | `page.get_text("text")` (content-stream order, default flags); `sort=True` also run as a variant |
| poppler | Poppler 24.02.0 `pdftotext` | default (layout-analysis reading order); `-raw` also run as a variant |
| pdfminer | pdfminer.six 20260107 (optional 4th reference) | `extract_text()` defaults; run on {sum(1 for d in R['documents'] if d['engines'].get('pdfminer'))} of 57 documents |

**Text fidelity.** Reference = the USFM zip named in the manifest, restricted to the books the app's own
audit report expects for that document (the manifest's books column for G05). Verse text = text after
`\\v` up to the next `\\v`/`\\c`, with `\\f…\\f*`, `\\fe…\\fe*`, `\\x…\\x*`, `\\va`, `\\vp`, `\\fig`, `\\rq` removed, `\\w …|attr\\w*`
attributes removed, every other marker removed, and heading paragraphs (`\\s`, `\\ms`, `\\mr`, `\\r`, `\\d`, `\\qa`,
`\\sp`, `\\cl`, titles, introductions) excluded. Both sides normalised: NFC, keep letters, digits and
combining marks (Unicode categories L, N, M), case-fold. The whole engine output is joined into one string.

* **coverage** = share of reference verses whose normalised text occurs as a contiguous substring;
* **in-order** = greedy monotonic search (each verse searched from the end of the previous match).
  The headline figure restarts the pointer at the beginning of each *book*, because the documents' book
  order often differs from the reference zip's order (G03, G05, F01: the global variant gives ~3 % for every
  engine on G03 although every engine reads it in order). The global variant is in `results.json`
  (`in_order_global`).
* **Hebrew**: the headline uses the points/cantillation-stripped variant (NFKD, drop marks, NFC) because
  several Hebrew PDFs omit cantillation the reference has, so the pointed comparison is 0 for every engine
  (G10, hbo-rwpdfrtl-rut); the pointed figures are shown in § 6. RTL logical-order checks: the same search
  on each output line reversed character-by-character (visual order) and on each line with its *word order*
  reversed.
* **letters-only** variant (digits dropped too) is reported separately (§ 5): with the specified
  letters+digits normalisation a verse number that an engine puts in the wrong place *inside* a verse
  breaks that verse, which is a real ordering defect but a different one from losing words.
* Implementation: one Aho–Corasick pass per variant (pyahocorasick) finds every occurrence of every verse;
  the greedy search then runs over the sorted occurrence lists (equivalent to repeated `str.find`).

**Known limits of the metric (they affect all engines equally):** lettered footnote callers printed inside
verses (fraLSG-pdf, frajnd-pdf, F13, engbsb, engwebu) and footnote/gloss text inserted in the middle of a
verse at a page or column break (G05) count as misses because the match must be contiguous. That is why
fraLSG-pdf and F13 stop at ~82 % / ~69 % coverage for *every* engine while the app verifies 94.5 % of
fraLSG-pdf. Very short verses can match spuriously (tiny upward bias).

**Performance.** Wall time and peak RSS of the engine's own process, from `os.wait4()` on that child
(per-child `ru_maxrss`), file pre-read into the page cache. Numbers come from a separate *clean timing
pass* (`scripts/timing.py`): text extraction only, nothing else of mine running, one run per document.
Another job (not mine) kept one of the 4 cores busy throughout. Important caveats:
* the batch run's RSS figures (in `results.json` as `batch_peak_rss_mb_inflated`) are **not usable**: the
  child's `ru_maxrss` includes the parent's resident pages at fork time, and the batch parent grew to
  ~200 MB; the clean pass uses a small parent, so values at or below **~25–45 MB are the measurement floor**
  (the timing parent grew from ~25 to ~45 MB during the pass; poppler's `pdftotext` usually sits at the floor);
* PDFium/MuPDF/pdfminer were driven from Python: null-process baselines (start interpreter + import, no
  document) are pdfium {base['pdfium']['seconds']} s / {base['pdfium']['peak_rss_mb']} MB, mupdf {base['mupdf']['seconds']} s / {base['mupdf']['peak_rss_mb']} MB
  (PyMuPDF 2.0 loads a 36 MB libmupdf plus extras), pdfminer {base['pdfminer']['seconds']} s / {base['pdfminer']['peak_rss_mb']} MB, pdftotext {base['poppler']['seconds']} s / {base['poppler']['peak_rss_mb']} MB.
  MuPDF's ~140 MB floor is mostly the Python wrapper, not MuPDF itself;
* the current extractor's binary is a 420 MB test executable with full DWARF debug sections and appears to
  be an unoptimised (debug) build, and it runs the app's layout analysis as well as parsing. Its times are
  an upper bound for the release app; for reference the app's own audit JSON reports the *whole import*
  (extraction + recognition, release build) at e.g. 299 s for G03 and 187 s for kor-pdf (`app.seconds` in
  `results.json`). A like-for-like Rust timing of pdf-extract vs pdfium-render was not possible without
  building Rust code.

## 3. Checksums

All {nchk} PDFs listed in the manifests (withheld-b/c/d originals are not on disk and were skipped, as
instructed) were hashed before measuring: **{nver}/{nchk} match every recorded checksum** (column
sha256 of withheld-e/f/g.tsv, `docs/audit/v0.24.7/corpus-checksums.sha256`, and `/root/aw-set-g/SEAL.txt`
for set G; the sealed `withheld-g.tsv` itself also matches the seal: d0318e02…). Every reference zip that has
a recorded checksum matches too. Full list in `out/checksums.txt` / `out/docs.json`.

<details><summary>Per-file verification</summary>

{chk}

</details>

## 4. Comparison matrix

Cell = coverage % / in-order % (per book; Hebrew: points stripped) · wall seconds · peak RSS MB (clean pass).
`\\*` = time from the batch run, RSS not measured cleanly. "app verified" = the app's own audit
(`totals.verified / totals.expected`).

{T['main']}

### Summary rows

{T['summary']}

Mean figures count a failure as 0. "Docs with clean timing" shows how many documents the clean pass
covered for each engine.

## 5. Sensitivity: letters-only and NFKC

covL/ordL = coverage/in-order with digits dropped as well (so misplaced verse numbers no longer matter);
"ord NFKC" = in-order with NFKC instead of NFC (ligatures such as ﬁ folded). Non-Hebrew documents.

<details><summary>Table</summary>

{T['letters']}

</details>

Findings: for 47 of the 57 documents the current extractor, PDFium and MuPDF are within 2 points of each
other on every variant — on ordinary single- and two-column PDFs the choice of parser does not change the
text. The differences are concentrated in four groups: Hebrew (§ 6), G05 (cannot open), kor-lopdf (current
87.0 vs 89.3) and the three verse-number-placement files (G13 48.6 vs 98.1 with digits; 97.9 vs 98.1
letters-only). NFKC changes nothing measurable on this corpus.

Engine variants (cov/ord):

<details><summary>MuPDF sort=True and pdftotext -raw</summary>

{T['variants']}

</details>

`pdftotext -raw` (content-stream order) matches PDFium/MuPDF almost exactly (mean in-order 84.0 % vs
74.6 % for Poppler's default layout analysis); MuPDF `sort=True` is much worse (mean 48.8 %) because it
interleaves columns. Poppler's default mode is the one that collapses on two-column Korean (kor-r2col 28.5 %,
kor-lopdf 12.5 %) and Greek (grctr 44.5 %, grcbyz 51.0 %) and on margin numbers.

## 6. Right-to-left, Korean and Greek

**Hebrew** (cov/ord %; headline = points stripped):

{T['rtl']}

* No engine is right on all five Hebrew files. The current extractor (pdf-extract glyphs + the app's layout
  code) is best or joint best on G04 (99.5), hbo-rwpdfrtl-rut (95.3) and hboWLC-pdf (65.8), and fails on
  heb-wprtl (1.9) and G10 (39.6).
* **PDFium** `FPDFText_GetText` returns RTL text in content-stream (visual) order. On G04 and heb-wprtl each
  word is logical but the words of a line run left-to-right: reversing word order per line gives 99.0 % and
  80.6 % — fixable with a per-line bidi pass. On HarfBuzz/WeasyPrint-shaped Hebrew (hbo-rwpdfrtl-rut,
  hboWLC) it splits points from their letters and inserts spaces inside words; its raw text API is unusable
  there, and only glyph-box-based reordering (what `pdf_layout.rs` does) could recover it.
* **MuPDF** gets heb-wprtl (97.4 / 80.5) and hbo-rwpdfrtl-rut, but on G04 reverses the order of phrases
  between punctuation (8.8 %).
* **Poppler** (default mode) does full bidi reordering (G04 99.6) and is the only engine that honours the
  `/ActualText` spans LibreOffice wrote into G10 (38 content streams carry ActualText; Poppler 87.8 % vs ≤ 39.6 %
  for everyone else); it fails on the WeasyPrint files.
* With points kept, G10 and hbo-rwpdfrtl-rut score 0 for every engine: the PDFs omit cantillation that the
  reference carries; this is a reference/document difference, not an extraction one.

**Korean** — kor-pdf, kor-r2col, kor-lopdf, kor1961-lopdf, F07, G09: current, PDFium and MuPDF agree within
0–2 points (kor-pdf 99.5; kor-r2col 98.8–98.9; F07 97.8; kor1961-lopdf 98.8 vs 100, the difference being digits
only; G09 90.4/90.6, 97.6/97.8 letters-only; kor-lopdf 87.0 vs 89.3). Hangul is extracted cleanly by all;
the only Korean failures are Poppler's default layout mode on two columns and pdfminer on kor-lopdf (23.9).

**Greek** — grctr-pdf 90.3, grcbyz-pdf 98.7, grcsbl-rfpdf-mrk 96.9 (100 letters-only) for current, PDFium and
MuPDF alike; polytonic accents survive NFC in all three. Poppler default: 44.5 / 51.0.

**Large document** — G03 (816 pages, XeLaTeX, two columns, drop numerals): every engine except Poppler-default
finds 99.1 % of verses, 100 % in order letters-only (93.1 % with digits, because the drop numerals sit inside
verse 1). Times in the summary row "G03".

## 7. Geometry and structure available through each API

Capability (from the API, and exercised here where marked *measured*):

| item | current (pdf-extract 0.12 + lopdf 0.42) | PDFium (pdfium-render / FPDF_*) | MuPDF | Poppler | pdfminer.six |
|---|---|---|---|---|---|
| character position/box | yes — `OutputDev::output_character` gives the text matrix, width and size per glyph (the app uses this) | yes, *measured* (`FPDFText_GetCharBox`, `GetCharOrigin`; every sampled char had a box) | yes, *measured* (`rawdict` chars with bbox) | yes, *measured* word/line/block boxes (`pdftotext -bbox-layout`); glyph boxes via poppler-glib `get_text_layout` (not measured) | yes (LTChar bbox) |
| font name per span | **no** — the font object passed to `output_character` is a private trait without a public name accessor; would need lopdf look-ups | yes, *measured* (`FPDFText_GetFontInfo`) | yes, *measured* | yes, *measured* (`pdftohtml -xml` fontspecs); poppler-glib `get_text_attributes` | yes |
| font size per span | yes (`font_size` argument) | yes, *measured* (`FPDFText_GetFontSize`) | yes, *measured* | yes, *measured* (xml) | yes |
| outline / bookmarks | lopdf `get_toc()` (not exercised) | yes, *measured* | yes, *measured* | yes, *measured* (xml outline) | yes |
| link annotations | lopdf `get_page_annotations()` (not exercised) | yes, *measured* (`FPDFLink_Enumerate`) | yes, *measured* | yes (glib link mapping; CLI only shows `<a>` per text span, which over-counts) | yes (annots) |
| page labels | lopdf raw catalog walk only (no helper) | yes, *measured* (`FPDF_GetPageLabel`) | yes, *measured* | poppler-glib `poppler_page_get_label` (not measured; no CLI) | yes |
| tagged structure | lopdf raw dictionaries only | yes, *measured* (`FPDF_StructTree_*`, incl. `GetActualText`/`GetAltText`) | yes (`StructTreeRoot` *measured* via catalog) | yes, *measured* (`pdfinfo -struct`) | catalog only |
| ActualText used in text extraction | no | not observed on G10 | not observed on G10 | **yes** (G10) | not observed |

Measured counts per document (all three engines agree exactly on outline counts for {agg['agree_outline']}/{agg['n']} documents,
on link counts for {agg['agree_links']}/{agg['n']} and on page-label counts for {agg['agree_labels']}/{agg['n']}): {agg['outline']} documents have an
outline, {agg['links']} have link annotations, {agg['labels']} have page labels, {agg['tagged']} are tagged (StructTreeRoot), {agg['actualtext']} carry
ActualText in content streams (G09, G10, engbrenton-rlo2col, kor-lopdf).

<details><summary>Per-document structure counts</summary>

{T['struct']}

</details>

## 8. Damaged copies (malformed but readable)

Copies in `/root/a2-pdf/damaged/` of G13 (ReportLab), grcsbl-rfpdf-mrk (fpdf2) and G09 (Korean, tagged):
(i) every in-use offset of the classic xref table shifted by 13 bytes (startxref left correct);
(ii) file cut at the start of the final `xref` keyword — all objects present, no xref/trailer/startxref;
(iii) the `/Length` of page 4's content stream halved (space-padded so that no other offset moves).
"cov" = coverage % (headline normalisation); "orig" = the same engine on the intact file.

{T['damaged']}

* **xref corruption**: PDFium, MuPDF and Poppler rebuild the cross-reference and read 100 % of what they read
  from the original; **the current extractor refuses all three** (`lopdf` `Document::load_mem` fails →
  `library.badPdf`); pdfminer opens them but silently returns 0 pages.
* **missing xref/trailer**: only **MuPDF** repairs the file (full text). PDFium, Poppler, pdfminer and the
  current extractor refuse.
* **wrong /Length**: all engines open; PDFium and Poppler keep the full text, MuPDF loses ≤ 0.4 points, the
  **current extractor drops the whole damaged page** (27/28, 36/37, 26/27 pages with text; G09 coverage
  90.4 → 87.0), pdfminer loses a little.

## 9. Extraction vs recognition — documents where the app verified < 50 %

Rule: EXTRACTION = current cannot open, or its in-order share is ≥ 15 points below the best other engine;
"verse-number placement only" = the gap disappears when digits are ignored (words complete and in order,
numbers emitted out of position by the app's layout code); RECOGNITION = current text ≥ 85 % in order, or
current at least as good as every other engine; DOCUMENT = no engine reaches 50 %.

{T['attr']}

Reading of the table:

* **Recognition ({n_rec} of {n_attr}: {rec_keys})**: G03 (0 verified of 31,011, text 99.1 % / 93.1 % in order), F07 (0 of 31,084; the
  app refused 1,187 chapters for "reading order" although the text is 97.8 % in order), F01, G04, G07, G09,
  deutkw-fpdfa5, kor-lopdf, grcsbl-rfpdf-mrk, hbo-rwpdfrtl-rut, latVUC-pdf, spavbl-rlofodt, F03, and — with the
  text layer limiting every engine to 63–84 % — F13, engwebu-rl2col and hboWLC-pdf. A different PDF engine will
  not raise these; the recogniser has to.
* **Extraction ({n_ext}: {ext_keys})**: G05 (cannot open), heb-wprtl (1.9 % vs MuPDF 80.5 %), G10 (39.6 % vs Poppler 87.8 %),
  and the number-placement group G13, spabll-rrlab2col, engkjvcpb-rwpdfmargin. In the latter the PDF parser is
  not the culprit — PDFium's/MuPDF's raw stream order keeps numbers where they are printed (98.1 % on G13) —
  the app's line/role layout emits superscript or margin verse numbers as separate units before (or apart
  from) the line they belong to. Switching engine fixes this only if the layout code changes too.
* **Document ({n_doc})**: no low-verified document is unreadable for every engine.

## 10. Integration, licensing, packaging, security

What was verified in this session: crates.io metadata (versions, dates, licences, downloads) for
pdfium-render, mupdf/mupdf-sys, poppler-rs, poppler, pdf-extract, lopdf; the READMEs of pdfium-render,
pdfium-binaries and mupdf-rs and mupdf-sys's `build.rs` (raw.githubusercontent.com); MuPDF's COPYING and README
(AGPL-3.0 text, "commercial licensing … artifex.com"); the actual Windows PDFium package (downloaded and
unpacked: `pdfium-win-x64.tgz`, chromium/8138); the local Debian copyright and changelog files of Poppler
24.02. Not reachable: GitHub API, mupdf.com/artifex.com, gitlab.freedesktop.org, NVD/OSV/CVE databases,
RustSec — statements about CVE history beyond the Poppler changelog are general knowledge, not verified here.

### Rust integration options

| option | crate (latest, date) | how it links | Windows build/packaging | notes |
|---|---|---|---|---|
| **PDFium** | `pdfium-render` 0.9.5 (2026-11-01), MIT OR Apache-2.0, 90 releases, 3.76 M downloads | binds at **run time** to `pdfium.dll` (`Pdfium::bind_to_library`), or `static` feature to link a static PDFium you build yourself | prebuilt `pdfium.dll` from bblanchon/pdfium-binaries: **7.8 MB** (x64, chromium/8138 = PDFium 159.0.8138.0, built `is_debug=false`, V8/XFA off; tgz 4.0 MB) + licence folder; no C/C++ toolchain needed, MSVC-friendly; ship the DLL as a Tauri bundle resource next to the exe and load it by path — fully offline | PDFium is not thread-safe: pdfium-render serialises calls behind a mutex (README); use one worker thread or a helper process. Pin the crate's `pdfium_XXXX` feature to the DLL's build. Static linking means building PDFium with depot_tools/gn (heavy) |
| **MuPDF** | `mupdf` 0.8.0 (2026-06-22), **AGPL-3.0** | `mupdf-sys` compiles the vendored MuPDF C sources in `build.rs` (MSBuild on MSVC, bindgen ⇒ needs LLVM/libclang) | build time is long; default features pull JS, OCR (Tesseract), HTML/EPUB, fonts — must be trimmed (`default-features = false`); library size: 36 MB `libmupdf.so` in the PyMuPDF wheel (Linux, with fonts); Windows size not measured | licence blocks closed-source use unless an Artifex commercial licence is bought |
| **Poppler** | `poppler-rs` 0.26.0 (2026-02-21; MIT bindings to poppler-glib); `poppler` 0.6.0 (GPL-2.0) | links poppler-glib → libpoppler (C++) | needs glib/gobject, **cairo**, freetype, fontconfig, lcms2, libjpeg, openjpeg, libpng, libtiff (+ optionally NSS, curl); on Debian libpoppler alone has ~55 shared-library dependencies (ldd), poppler-glib 77; on Windows this means MSYS2/vcpkg/gvsbuild DLL bundles (not tried) | GPL; the heaviest packaging of the four |
| **keep pdf-extract/lopdf** | `pdf-extract` 0.12.1 (2026-09-16), MIT; requires `lopdf ^0.42` while lopdf is at **0.47.0** (2026-11-04), MIT | pure Rust, already integrated | nothing to ship | fails G05 (panic on a `/Widths` length mismatch), no xref repair, loses pages with a wrong `/Length`, no font names; slower (§ 4) |

### Licences and what they mean for a closed-source binary

* **PDFium**: BSD-3-Clause ("Copyright 2014 The PDFium Authors"); bundled third-party code in the prebuilt DLL
  under permissive licences — Apache-2.0 (abseil, dragonbox, llvm-libc, with LLVM exception), FreeType
  Licence (FTL), ICU licence, lcms2, libjpeg-turbo (IJG/BSD), OpenJPEG (BSD-2), libpng, zlib, simdutf, fast_float,
  highway, AGG 2.3 (the files are in the package's `licenses/` folder; ICU's text mentions GPL-3 only for an
  Autoconf `configure` script exception, which does not affect the binary). **Redistribution in a closed-source
  app is allowed**; ship the licence texts (e.g. an "open-source notices" file in the installer).
  pdfium-render itself is MIT/Apache-2.0; the bblanchon build scripts are MIT.
* **MuPDF**: AGPL-3.0-or-later, or Artifex commercial licence. Distributing the app with MuPDF under the AGPL
  requires offering the complete corresponding source of the *whole* app under the AGPL. **Not compatible with a
  closed-source app without a commercial licence** (price not checked — artifex.com unreachable).
* **Poppler**: GPL-2.0-or-later / GPL-3.0 (Debian copyright: "the poppler core library is GPL v2 or GPL v3").
  Linking it into the app makes the app subject to the GPL. **Not compatible with closed-source distribution.**
  Shipping `pdftotext.exe` as a separate program is sometimes argued to be "mere aggregation", but it still
  carries GPL source-offer obligations and is a legal judgement for the owner, not an engineering one.
* **pdf-extract / lopdf**: MIT — no constraint beyond keeping the notices.

### Security and maintenance

* PDF parsers are an attack surface and these documents are untrusted downloads. PDFium, MuPDF and Poppler are
  C/C++; each has a long history of memory-safety CVEs (heap overflows, use-after-free). PDFium is part of
  Chromium and is continuously fuzzed (ClusterFuzz/OSS-Fuzz) with fixes landing in Chromium's release train;
  bblanchon republishes builds about weekly (chromium/8138 dated 2026-11-04). Poppler releases monthly; the
  Ubuntu/Debian package installed here (24.02) lists 19 distinct CVE IDs fixed by distribution patches (1 in
  2022, 1 in 2023, 3 in 2024, 7 in 2025, 7 in 2026). MuPDF has several releases a year (1.28.5 here). (CVE
  databases themselves were not reachable.)
* pdf-extract/lopdf are Rust: memory-safe in safe code, but they *panic* on unexpected input (G05) — the app
  already wraps extraction in `catch_unwind` — and are far less fuzzed; resource-exhaustion bugs are possible.
* Recommendation for any C/C++ engine: run extraction **out of process** (a small helper executable bundled
  with the Tauri app that loads `pdfium.dll`, reads one file, returns glyphs/text over stdin/stdout), with a
  Windows Job Object memory/CPU-time limit and a wall-clock timeout, ideally at low integrity / restricted
  token or in an AppContainer. This also contains PDFium's lack of thread safety and any crash. Update the DLL
  with app releases (pin and record the PDFium build number).

## 10a. Integration complexity, fallback strategy and a shared extraction interface

**Integration complexity (estimate, not measured).**

| path | Rust work | build / CI | packaging |
|---|---|---|---|
| PDFium as glyph source for `pdf_layout.rs` | moderate: a new adapter that walks `PdfPageText` characters (unicode, loose/tight box, origin, font size, font name, render mode) and produces the same `Glyph`/`Span` input the current `Collector` builds from `output_character`; outline/labels/links via pdfium-render | no C++ toolchain (dynamic binding); CI only downloads a pinned pdfium-binaries tarball and checks its sha256 | +7.8 MB DLL (+~4 MB compressed) as a Tauri resource; licence notices |
| helper process around it | small: a sidecar exe (same crate, `--extract <file>` mode) speaking JSON/bincode over stdout; Job Object limits on Windows | none extra | one more exe in the bundle |
| MuPDF (`mupdf` crate) | similar adapter | heavy: compiles MuPDF C on every clean build, needs LLVM for bindgen and MSBuild | larger binary; AGPL or commercial licence |
| Poppler (`poppler-rs`) | similar adapter | heavy: glib/cairo/poppler DLL stack (MSYS2/vcpkg/gvsbuild) | dozens of DLLs; GPL |
| stay on pdf-extract | small fixes upstream/vendor: tolerate `/Widths` length mismatch (G05), xref reconstruction, per-stream length recovery | none | none |

**Fallback strategy.**
1. Try PDFium in the helper process (timeout, memory cap).
2. If PDFium refuses the file, crashes, times out, or yields almost no letters (the app's existing
   `MIN_LETTERS_PER_PAGE` "scanned" test), run the current pdf-extract/lopdf path in-process (it is pure Rust and
   already wrapped in `catch_unwind`).
3. Keep whichever result has more letters per page; record which engine produced the text in the import report
   (so the audits can keep separating extraction failures from recognition failures).
4. Optional, licence permitting: none of MuPDF/Poppler as a third fallback in a closed-source build. On this
   corpus the only file the fallback chain would still lose is a PDF with no xref/trailer (only MuPDF repairs that).

**Shared extraction interface (proposal).** One trait that both engines implement, so the layout code and the
recogniser never see engine types:

```rust
pub struct PdfGlyph {{ pub text: String, pub x0: f32, pub y0: f32, pub x1: f32, pub y1: f32,
                      pub origin: (f32, f32), pub size: f32, pub font: Option<String>,
                      pub rotation_deg: f32, pub from_actual_text: bool }}
pub struct PdfPageGlyphs {{ pub number: u32, pub label: Option<String>,
                           pub width: f32, pub height: f32, pub glyphs: Vec<PdfGlyph> }}
pub struct PdfOutlineItem {{ pub title: String, pub page: Option<u32>, pub depth: u8 }}
pub struct PdfDocGlyphs {{ pub metadata: FoundMetadata, pub pages: Vec<PdfPageGlyphs>,
                          pub failed_pages: Vec<u32>, pub outline: Vec<PdfOutlineItem>,
                          pub engine: &'static str, pub repaired: bool }}
pub trait PdfGlyphSource {{
    fn read(&self, bytes: &[u8], file_name: &str) -> AppResult<PdfDocGlyphs>; // badPdf / passwordProtected as today
}}
```

`pdf_layout.rs` would consume `PdfPageGlyphs` (it already works from per-glyph position, width and size), and
`read_pages()` would become "try PdfiumSource, else PdfExtractSource". The corpus measurement used here
(`scripts/run_doc.py` + `score.py`) can then be rerun on the app's own units from each source to decide when the
fallback can be dropped.

## 11. Recommendation

1. **Adopt PDFium (pdfium-render + prebuilt pdfium.dll, ~7.8 MB) as the primary PDF engine**, out of process.
   Evidence: opens 57/57 (current 56/57); same text as the current extractor on all non-Hebrew documents
   (mean in-order letters-only 92.4 % vs 90.6 %); survives xref corruption and wrong stream lengths where the
   current extractor refuses or drops pages; a median 18× faster in wall time (clean pass, § 4; G03 816 pages: 2.6 s vs 34.3 s) at lower memory
   (median 56 MB vs 162 MB; max 118 MB vs 573 MB); exposes boxes, font names/sizes, outline, links, page labels and structure tree; licence
   compatible with closed-source distribution.
2. **Feed PDFium's per-character data into the existing `pdf_layout.rs`** instead of using PDFium's own text
   order. This keeps the app's column/reading-order/line-role logic (which is what reads G04, hbo-rwpdfrtl-rut
   and hboWLC correctly) and avoids PDFium's raw RTL output. Fix the layout code's handling of inline and margin
   verse numbers (G13, spabll-rrlab2col, engkjvcpb-rwpdfmargin) — that is independent of the engine.
3. **Keep pdf-extract/lopdf as a fallback** behind a switch during the transition (and for A/B comparison),
   to be removed once the PDFium+layout path has been measured on this corpus to be at least as good. On the
   evidence here it adds no files PDFium cannot open; its remaining value is the proven layout pipeline.
4. **Do not adopt MuPDF or Poppler** unless the owner buys an Artifex commercial licence (MuPDF would then be a
   strong alternative: best mean scores and the only engine to repair a missing xref/trailer) or accepts GPL
   obligations (Poppler: worth knowing that it is the only engine honouring ActualText, which rescues G10).
5. Most low-verification documents are recognition problems (§ 9); an engine change alone will not move G03,
   F07, F01, G04, G09 etc.

**Uncertainties.** (a) PDFium glyphs through the app's layout code were not measured (requires Rust changes);
the RTL behaviour of that combination is the main open risk. (b) The current extractor's timings come from what
appears to be a debug test binary that also runs layout analysis; release-build parsing alone is faster than
shown. (c) RSS of Python-driven engines includes interpreter/wrapper overhead (baselines above). (d) The
substring metric penalises footnote callers inside verses equally for all engines. (e) Damaged-file tests cover
3 documents × 3 damage types only. (f) CVE histories and MuPDF commercial terms could not be checked online.

## 12. Files

* `report.md` (this file); `out/results.json` (all raw numbers: per document × engine: opens, error, pages,
  coverage/in-order for every normalisation variant, per-book counts, batch and clean timings; damaged files;
  structure counts; app audit totals); `out/attribution.json`; `out/timing_clean.json`; `out/docs.json` and
  `out/checksums.txt` (checksum verification); `out/struct/*.json`; `out/results/*.json` (per document);
  `out/text/<engine>/<key>.txt.gz` (each engine's extracted text, the current extractor's raw unit dump as
  `out/text/current/<key>.dump.gz`); `out/meta/` (crate metadata and READMEs fetched).
* Scripts (`scripts/`): `manifest.py` (document list + checksums), `common.py` (USFM parsing, normalisation),
  `score.py`, `extract.py` (per-engine worker), `run_doc.py` (runs engines, scores), `batch.sh`, `struct_doc.py`,
  `damage.py`, `timing.py`, `timing_pdfminer.py`, `rescore.py`, `aggregate.py`, `tables.py`, `build_report.py`.
"""
open('/root/a2-pdf/report.md', 'w').write(rep)
print(len(rep))
