> **Provenance of this report.** Measured on 2026-11-08 in an isolated work
> directory (`/root/a2-pdf`) by a separate worker that wrote nothing into the
> repository and built nothing. It is copied here unchanged apart from this note.
> Its scripts are in `scripts/a2-pdf/`, and its raw numbers, checksums and
> timings in `docs/audit/a2/data/`. Extracted texts and fetched crate metadata
> stay outside the repository (edition licences). Headline numbers were
> re-derived from `data/results.json` before committing. Every document here
> belongs to a **disclosed** set (B–G, the development corpus or its
> reconstructions). This is engineering measurement, not independent evaluation.
> No production code is changed by A2.

# PDF text-extraction engines for Abiding Word — measured comparison

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
  of 22 documents where the app verified < 50 % (or refused the file), the current extractor's text is
  complete and in order (≥ 85 % of verses found in order, or as good as any engine) in 16. Extraction is clearly at fault in
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
| pdfminer | pdfminer.six 20260107 (optional 4th reference) | `extract_text()` defaults; run on 40 of 57 documents |

**Text fidelity.** Reference = the USFM zip named in the manifest, restricted to the books the app's own
audit report expects for that document (the manifest's books column for G05). Verse text = text after
`\v` up to the next `\v`/`\c`, with `\f…\f*`, `\fe…\fe*`, `\x…\x*`, `\va`, `\vp`, `\fig`, `\rq` removed, `\w …|attr\w*`
attributes removed, every other marker removed, and heading paragraphs (`\s`, `\ms`, `\mr`, `\r`, `\d`, `\qa`,
`\sp`, `\cl`, titles, introductions) excluded. Both sides normalised: NFC, keep letters, digits and
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
  document) are pdfium 0.06 s / 20.2 MB, mupdf 0.537 s / 137.6 MB
  (PyMuPDF 2.0 loads a 36 MB libmupdf plus extras), pdfminer 0.096 s / 33.0 MB, pdftotext 0.006 s / 15.2 MB.
  MuPDF's ~140 MB floor is mostly the Python wrapper, not MuPDF itself;
* the current extractor's binary is a 420 MB test executable with full DWARF debug sections and appears to
  be an unoptimised (debug) build, and it runs the app's layout analysis as well as parsing. Its times are
  an upper bound for the release app; for reference the app's own audit JSON reports the *whole import*
  (extraction + recognition, release build) at e.g. 299 s for G03 and 187 s for kor-pdf (`app.seconds` in
  `results.json`). A like-for-like Rust timing of pdf-extract vs pdfium-render was not possible without
  building Rust code.

## 3. Checksums

All 57 PDFs listed in the manifests (withheld-b/c/d originals are not on disk and were skipped, as
instructed) were hashed before measuring: **57/57 match every recorded checksum** (column
sha256 of withheld-e/f/g.tsv, `docs/audit/v0.24.7/corpus-checksums.sha256`, and `/root/aw-set-g/SEAL.txt`
for set G; the sealed `withheld-g.tsv` itself also matches the seal: d0318e02…). Every reference zip that has
a recorded checksum matches too. Full list in `out/checksums.txt` / `out/docs.json`.

<details><summary>Per-file verification</summary>

| key | file | bytes | sha256 | checks | reference zip |
|---|---|---|---|---|---|
| engwebp-pdf | `/root/aw-corpus/engwebp_all.pdf` | 9,597,871 | `37be7fab3f124b95…` | corpus-checksums=OK | OK |
| engwebp-book | `/root/aw-corpus/engwebp_book.pdf` | 9,288,817 | `9c6795f3710a151d…` | corpus-checksums=OK | OK |
| kjv-pdf | `/root/aw-corpus/eng-kjv2006_a4.pdf` | 10,514,289 | `d8e0d814e2af79b9…` | corpus-checksums=OK | OK |
| deu1912-pdf | `/root/aw-corpus/deu1912_all.pdf` | 8,839,248 | `ccf956f61c027d82…` | corpus-checksums=OK | OK |
| deuelo-pdf | `/root/aw-corpus/deuelo_book.pdf` | 8,902,332 | `f1493825b515157b…` | corpus-checksums=OK | OK |
| fraLSG-pdf | `/root/aw-corpus/fraLSG_all.pdf` | 12,850,420 | `1c2d1169cc103152…` | corpus-checksums=OK | OK |
| fraLSG-luk | `/root/aw-corpus/fraLSG_LUK.pdf` | 575,962 | `adb0e401da85eb1c…` | corpus-checksums=OK | OK |
| fob-pdf | `/root/aw-corpus/fra_fob_all.pdf` | 9,745,404 | `f1e77d5e9d3deee8…` | corpus-checksums=OK | OK |
| rv1909-pdf | `/root/aw-corpus/spaRV1909_all.pdf` | 9,071,021 | `82914e1edc88e9ed…` | corpus-checksums=OK | OK |
| rv1909-lopdf | `/root/aw-corpus/spaRV1909_lo.pdf` | 7,635,572 | `1b82ac19c7941f8c…` | corpus-checksums=OK | OK |
| spabes-pdf | `/root/aw-corpus/spabes_a4.pdf` | 8,849,355 | `e32bf345414eddea…` | corpus-checksums=OK | OK |
| porbrbsl-pdf | `/root/aw-corpus/porbrbsl_all.pdf` | 12,137,019 | `820feb05b8e60325…` | corpus-checksums=OK | OK |
| porbr2018-pdf | `/root/aw-corpus/porbr2018_book.pdf` | 8,938,259 | `c73a54114d297ef4…` | corpus-checksums=OK | OK |
| kor-pdf | `/root/aw-corpus/kor_all.pdf` | 10,530,556 | `9cde68e514a3872f…` | corpus-checksums=OK | OK |
| grctr-pdf | `/root/aw-corpus/grctr_all.pdf` | 2,338,087 | `ee7907d7147a2868…` | corpus-checksums=OK | OK |
| hboWLC-pdf | `/root/aw-corpus/hboWLC_all.pdf` | 6,639,554 | `d00337ea8feffb57…` | corpus-checksums=OK | OK |
| latVUC-pdf | `/root/aw-corpus/latVUC_all.pdf` | 19,211,065 | `2b7049ed354fa197…` | corpus-checksums=OK | OK |
| deutkw-pdf | `/root/aw-corpus/deutkw_all.pdf` | 9,476,623 | `7576ecfaef5170ef…` | corpus-checksums=OK | OK |
| frajnd-pdf | `/root/aw-corpus/frajnd_a4.pdf` | 12,364,438 | `a3ed40032c5ccecc…` | corpus-checksums=OK | OK |
| spablm-pdf | `/root/aw-corpus/spablm_all.pdf` | 11,849,145 | `4ed61385a465218d…` | corpus-checksums=OK | OK |
| engylt-pdf | `/root/aw-corpus/engylt_book.pdf` | 8,696,318 | `b99369ff6d2aa171…` | corpus-checksums=OK | OK |
| grcbyz-pdf | `/root/aw-corpus/grcbyz_a4.pdf` | 2,571,781 | `fae2b849a836fe5d…` | corpus-checksums=OK | OK |
| kor1961-lopdf | `/root/aw-corpus/kor1961_usfm_lo.pdf` | 18,677,420 | `6cbd0e3ba247141d…` | corpus-checksums=OK | OK |
| deutkw-lopdf | `/root/aw-corpus/deutkw_usfm_lo.pdf` | 12,285,849 | `08344d51aca0b6c1…` | corpus-checksums=OK | OK |
| frajnd-lopdf | `/root/aw-corpus/frajnd_usfm_lo.pdf` | 12,856,007 | `be5de731c77848f4…` | corpus-checksums=OK | OK |
| engbsb-rwpdf2col | `/root/aw-recon-bc/engbsb-rwpdf2col.pdf` | 6,396,406 | `89f877f2921090f9…` | corpus-checksums=OK | OK |
| engasv-rrlab1col | `/root/aw-recon-bc/engasv-rrlab1col.pdf` | 4,019,371 | `72c01362317ea0f0…` | corpus-checksums=OK | OK |
| spavbl-rlofodt | `/root/aw-recon-bc/spavbl-rlofodt.pdf` | 23,052,531 | `e7dbf9e2822bff35…` | corpus-checksums=OK | OK |
| grcsbl-rfpdf-mrk | `/root/aw-recon-bc/grcsbl-rfpdf-mrk.pdf` | 168,340 | `fa3cb84131b34b35…` | corpus-checksums=OK | OK |
| engrv-ebiblebook | `/root/aw-recon-bc/eng-rv_book.pdf` | 10,596,535 | `ec7bc275d991d44a…` | corpus-checksums=OK | OK |
| engkjvcpb-rwpdfmargin | `/root/aw-recon-bc/engkjvcpb-rwpdfmargin.pdf` | 7,290,366 | `ea7da19de872e081…` | corpus-checksums=OK | OK |
| engjps-rdocx2lo | `/root/aw-recon-bc/engjps-rdocx2lo.pdf` | 12,788,718 | `c39a2cbf4010a898…` | corpus-checksums=OK | OK |
| spabll-rrlab2col | `/root/aw-recon-bc/spabll-rrlab2col.pdf` | 4,630,220 | `41f81b9fa7304acf…` | corpus-checksums=OK | OK |
| poronbv-rwpdfa5 | `/root/aw-recon-bc/poronbv-rwpdfa5.pdf` | 7,504,843 | `e31de1b121664aa8…` | corpus-checksums=OK | OK |
| hbo-rwpdfrtl-rut | `/root/aw-recon-bc/hbo-rwpdfrtl-rut.pdf` | 60,662 | `13997cf463f741ba…` | corpus-checksums=OK | OK |
| engoebus-ebiblea4 | `/root/aw-recon-bc/engoebus_a4.pdf` | 4,071,750 | `f45bfced0a75816d…` | corpus-checksums=OK | OK |
| engbrenton-rlo2col | `/root/aw-recon/engbrenton-rlo2col.pdf` | 17,714,800 | `dffb9a18645614ee…` | corpus-checksums=OK | OK |
| deu1912-rdropcap | `/root/aw-recon/deu1912-rdropcap.pdf` | 6,835,365 | `5a8deda28d742312…` | corpus-checksums=OK | OK |
| kor-r2col | `/root/aw-recon/kor-r2col.pdf` | 4,937,639 | `5c7bf54ea44687f9…` | corpus-checksums=OK | OK |
| engwebu-rl2col | `/root/aw-set-e/docs/engwebu-rl2col.pdf` | 5,696,571 | `fbdebaef155e3a1b…` | manifest=OK, corpus-checksums=OK | OK |
| engtnt-wp3col | `/root/aw-set-e/docs/engtnt-wp3col.pdf` | 1,729,982 | `718136ecbb7a7e1d…` | manifest=OK, corpus-checksums=OK | OK |
| deutkw-fpdfa5 | `/root/aw-set-e/docs/deutkw-fpdfa5.pdf` | 1,171,097 | `c05900a4ec24be13…` | manifest=OK, corpus-checksums=OK | OK |
| kor-lopdf | `/root/aw-set-e/docs/kor-lopdf.pdf` | 6,980,831 | `38852a2549e984cd…` | manifest=OK, corpus-checksums=OK | OK |
| heb-wprtl | `/root/aw-set-e/docs/heb-wprtl.pdf` | 1,034,851 | `e5cd8eebcd09f843…` | manifest=OK, corpus-checksums=OK | OK |
| F01 | `/root/aw-set-f/docs/F01-eng-nheb.pdf` | 2,530,256 | `61cb24f6ae39ed10…` | manifest=OK, corpus-checksums=OK | OK |
| F02 | `/root/aw-set-f/docs/F02-eng-leeser.pdf` | 14,941,664 | `b3153cdf6927b6c6…` | manifest=OK, corpus-checksums=OK | OK |
| F03 | `/root/aw-set-f/docs/F03-deu-menge.pdf` | 5,375,224 | `7c3c91c0aa32a051…` | manifest=OK, corpus-checksums=OK | OK |
| F06 | `/root/aw-set-f/docs/F06-por-nva.pdf` | 1,107,109 | `687950e463498612…` | manifest=OK, corpus-checksums=OK | OK |
| F07 | `/root/aw-set-f/docs/F07-kor-rv.pdf` | 10,498,624 | `2776d2e95ea14a17…` | manifest=OK, corpus-checksums=OK | OK |
| F13 | `/root/aw-set-f/docs/F13-ita-diodati.pdf` | 1,072,321 | `46c02167cc29e35c…` | manifest=OK, corpus-checksums=OK | OK |
| G03 | `/root/aw-set-g/docs/G03_en_bibletex_kjv.pdf` | 4,732,042 | `36940c681aeaf44e…` | manifest=OK, SEAL=OK | OK |
| G04 | `/root/aw-set-g/docs/G04_he_ebible_heblb_a4.pdf` | 2,069,567 | `7b22c964af775f37…` | manifest=OK, SEAL=OK | OK |
| G05 | `/root/aw-set-g/docs/G05_de_luther1545_auszug.pdf` | 2,002,848 | `b6ba51eaaef33e89…` | manifest=OK, SEAL=OK | OK |
| G07 | `/root/aw-set-g/docs/G07_fr_fillion_deuterocanoniques.pdf` | 1,245,166 | `3df8c19ae98ba46a…` | manifest=OK, SEAL=OK | OK |
| G09 | `/root/aw-set-g/docs/G09_ko_kaeyok1938_selection.pdf` | 416,768 | `733b6029bbf490c8…` | manifest=OK, SEAL=OK | OK |
| G10 | `/root/aw-set-g/docs/G10_he_mam_megillot_jonah.pdf` | 852,810 | `ee7e11750ed7ca22…` | manifest=OK, SEAL=OK | OK |
| G13 | `/root/aw-set-g/docs/G13_en_ulb_selection.pdf` | 224,105 | `4a7a9c63c6aa554f…` | manifest=OK, SEAL=OK | OK |

</details>

## 4. Comparison matrix

Cell = coverage % / in-order % (per book; Hebrew: points stripped) · wall seconds · peak RSS MB (clean pass).
`\*` = time from the batch run, RSS not measured cleanly. "app verified" = the app's own audit
(`totals.verified / totals.expected`).

| doc | lang | pages | verses | current cov/ord % · s · MB | pdfium cov/ord % · s · MB | mupdf cov/ord % · s · MB | poppler cov/ord % · s · MB | pdfminer cov/ord % · s · MB | app verified |
|---|---|---|---|---|---|---|---|---|---|
| engwebp-pdf | en | 1273 | 31098 | 98.2/94.7 · 111.0s 105MB | 98.2/94.7 · 5.5s 65MB | 98.2/94.7 · 4.6s 151MB | 69.5/53.6 · 5.6s 24MB | 93.9/65.6 · 144.7s\* – | 29828/31098 |
| engwebp-book | en | 1065 | 31098 | 98.6/97.6 · 110.5s 102MB | 98.6/97.6 · 6.3s 63MB | 98.6/97.6 · 4.7s 150MB | 96.9/91.4 · 5.5s 24MB | 93.9/72.4 · 140.4s\* – | 30112/31098 |
| kjv-pdf | en | 1561 | 31102 | 99.5/99.5 · 108.1s 280MB | 99.5/99.5 · 5.5s 70MB | 99.5/99.5 · 5.0s 160MB | 99.5/99.5 · 6.0s 35MB | 98.1/98.1 · 160.9s\* – | 30868/31102 |
| deu1912-pdf | de | 1344 | 31102 | 99.7/99.7 · 91.1s 225MB | 99.7/99.7 · 5.0s 60MB | 99.7/99.7 · 4.7s 145MB | 99.7/99.7 · 4.6s 34MB | 97.8/91.5 · 129.2s\* – | 30700/31102 |
| deuelo-pdf | de | 1187 | 31102 | 99.7/99.7 · 96.7s 266MB | 99.7/99.7 · 5.1s 61MB | 99.7/99.7 · 4.6s 145MB | 99.7/99.7 · 4.8s 34MB | 99.2/99.2 · 134.3s\* – | 30510/31102 |
| fraLSG-pdf | fr | 1755 | 31170 | 82.4/63.4 · 136.4s 359MB | 82.4/63.4 · 7.0s 79MB | 82.4/63.4 · 5.7s 164MB | 82.0/63.3 · 6.7s 41MB | 81.8/63.1 · 195.5s\* – | 29457/31170 |
| fraLSG-luk | fr | 133 | 1151 | 77.2/77.2 · 5.0s 37MB | 77.2/77.2 · 0.3s 37MB | 77.2/77.2 · 0.9s 140MB | 77.2/77.2 · 0.3s 37MB | 77.2/77.2 · 6.4s\* – | 996/1151 |
| fob-pdf | fr | 1348 | 31107 | 99.8/99.8 · 105.9s 299MB | 99.8/99.8 · 5.3s 63MB | 99.8/99.8 · 4.6s 147MB | 99.7/99.7 · 5.0s 37MB | 98.6/97.4 · 150.3s\* – | 30782/31107 |
| rv1909-pdf | es | 1004 | 31084 | 97.4/95.7 · 95.2s 233MB | 97.4/95.7 · 5.2s 58MB | 97.4/95.7 · 4.5s 150MB | 97.4/95.7 · 4.7s 37MB | 97.1/95.4 · 127.9s\* – | 30582/31084 |
| rv1909-lopdf | es | 881 | 31084 | 100.0/100.0 · 31.5s 294MB | 100.0/100.0 · 2.1s 52MB | 100.0/100.0 · 2.4s 161MB | 100.0/100.0 · 1.4s 37MB | 100.0/100.0 · 29.2s\* – | 30420/31084 |
| spabes-pdf | es | 1359 | 31102 | 99.8/99.8 · 90.0s 216MB | 99.8/99.8 · 4.6s 58MB | 99.8/99.8 · 4.1s 149MB | 99.7/98.0 · 4.8s 37MB | 98.1/95.9 · 101.2s\* – | 30240/31102 |
| porbrbsl-pdf | pt | 1898 | 31098 | 99.0/96.0 · 125.3s 322MB | 99.0/96.0 · 6.5s 77MB | 99.0/96.0 · 5.1s 157MB | 98.5/88.0 · 6.4s 37MB | 97.3/94.3 · 138.0s\* – | 30199/31098 |
| porbr2018-pdf | pt | 1094 | 31102 | 99.9/99.9 · 94.8s 236MB | 99.9/99.9 · 5.3s 61MB | 99.9/99.9 · 4.4s 146MB | 99.9/99.9 · 5.2s 37MB | 99.1/99.1 · 107.3s\* – | 29536/31102 |
| kor-pdf | ko | 796 | 30991 | 99.5/99.5 · 154.7s 85MB | 99.5/99.5 · 7.0s 51MB | 99.5/99.5 · 5.4s 141MB | 80.7/57.6 · 6.7s 25MB | 99.5/97.5 · 184.1s\* – | 30917/30991 |
| grctr-pdf | grc | 339 | 7957 | 98.3/90.3 · 22.7s 41MB | 98.3/90.3 · 2.0s 35MB | 98.3/90.3 · 1.6s 141MB | 66.0/44.5 · 1.2s 25MB | 95.4/81.1 · 29.5s\* – | 7640/7957 |
| hboWLC-pdf | hbo | 913 | 23213 | 65.9/65.8 · 63.4s 162MB | 0.0/0.0 · 5.1s 56MB | 26.3/25.7 · 3.3s 144MB | 4.8/4.1 · 3.8s 37MB | 0.0/0.0 · 739.4s\* – | 0/23192 |
| latVUC-pdf | la | 2147 | 31434 | 96.0/89.3 · 185.7s 497MB | 96.0/89.3 · 10.2s 118MB | 96.0/89.3 · 8.1s 190MB | 57.8/31.4 · 10.7s 51MB | n/m | 8527/31229 |
| deutkw-pdf | de | 1490 | 31157 | 99.6/99.1 · 99.7s 243MB | 99.6/99.1 · 5.3s 64MB | 99.6/99.1 · 4.4s 146MB | 99.5/99.0 · 5.1s 45MB | n/m | 29724/31136 |
| frajnd-pdf | fr | 1550 | 31167 | 84.9/69.5 · 129.6s 319MB | 84.9/69.5 · 6.5s 74MB | 84.9/69.5 · 5.8s 159MB | 84.8/69.5 · 6.7s 45MB | n/m | 29625/31146 |
| spablm-pdf | es | 1845 | 31098 | 99.1/96.1 · 125.4s 314MB | 99.1/96.1 · 5.7s 74MB | 99.1/96.1 · 5.5s 153MB | 98.8/95.7 · 6.1s 45MB | n/m | 29104/31098 |
| engylt-pdf | en | 1088 | 31102 | 99.9/99.9 · 92.0s 234MB | 99.9/99.9 · 4.9s 60MB | 99.9/99.9 · 4.5s 148MB | 99.9/99.9 · 4.5s 45MB | n/m | 31057/31102 |
| grcbyz-pdf | grc | 323 | 7958 | 98.7/98.7 · 26.5s 40MB | 98.7/98.7 · 2.1s 36MB | 98.7/98.7 · 1.8s 140MB | 76.0/51.0 · 1.4s 25MB | 97.8/96.2 · 30.9s\* – | 7921/7958 |
| kor1961-lopdf | ko | 1119 | 31081 | 100.0/98.8 · 128.7s 376MB | 100.0/100.0 · 2.0s 60MB | 100.0/100.0 · 2.8s 188MB | 100.0/100.0 · 13.0s 36MB | 99.9/99.9 · 44.7s\* – | 30808/31081 |
| deutkw-lopdf | de | 1126 | 31157 | 100.0/100.0 · 37.2s 399MB | 100.0/100.0 · 1.8s 61MB | 100.0/100.0 · 2.9s 186MB | 100.0/100.0 · 1.6s 45MB | n/m | 29821/31136 |
| frajnd-lopdf | fr | 1156 | 31167 | 100.0/100.0 · 38.3s 410MB | 100.0/100.0 · 2.0s 64MB | 100.0/100.0 · 2.8s 190MB | 99.9/99.9 · 1.7s 45MB | n/m | 26360/31146 |
| engbsb-rwpdf2col | en | 993 | 31086 | 89.1/65.6 · 57.6s 90MB | 89.1/65.6 · 3.4s 54MB | 89.1/65.6 · 3.0s 150MB | 80.0/42.2 · 3.0s 36MB | 86.5/48.8 · 62.5s\* – | 22363/31086 |
| engasv-rrlab1col | en | 1381 | 31086 | 97.1/91.4 · 38.9s 237MB | 97.1/91.3 · 1.9s 64MB | 97.1/91.3 · 1.8s 147MB | 97.1/91.4 · 1.6s 45MB | n/m | 27471/31086 |
| spavbl-rlofodt | es | 1691 | 31086 | 88.2/86.0 · 56.5s 573MB | 88.2/86.0 · 2.7s 72MB | 88.2/86.0 · 3.8s 219MB | 88.2/86.0 · 3.0s 50MB | n/m | 3453/31086 |
| grcsbl-rfpdf-mrk | grc | 37 | 673 | 96.9/96.9 · 0.9s 50MB | 96.9/96.9 · 0.1s 50MB | 96.9/96.9 · 0.8s 142MB | 96.6/96.6 · 0.1s 50MB | n/m | 0/673 |
| engrv-ebiblebook | en | 1337 | 31086 | 99.9/99.9 · 113.6s 286MB | 99.9/99.9 · 5.8s 68MB | 99.9/99.9 · 4.9s 152MB | 99.9/99.9 · 5.8s 50MB | n/m | 30408/31086 |
| engkjvcpb-rwpdfmargin | en | 1716 | 31102 | 36.3/28.3 · 62.2s 255MB | 96.2/84.8 · 3.3s 56MB | 96.2/84.8 · 3.1s 149MB | 37.9/29.2 · 3.0s 36MB | 96.1/84.8 · 66.5s\* – | 1/31102 |
| engjps-rdocx2lo | en | 949 | 23145 | 96.4/82.7 · 44.2s 203MB | 96.4/82.7 · 2.4s 50MB | 96.4/82.7 · 3.1s 179MB | 94.4/46.5 · 2.1s 50MB | n/m | 20867/23145 |
| spabll-rrlab2col | es | 742 | 31098 | 80.9/73.2 · 35.8s 80MB | 97.3/95.6 · 2.1s 70MB | 97.3/95.6 · 2.1s 150MB | 95.5/38.9 · 1.8s 36MB | 95.6/89.0 · 43.6s\* – | 0/31098 |
| poronbv-rwpdfa5 | pt | 2179 | 31105 | 96.5/96.4 · 73.8s 277MB | 96.5/96.4 · 3.5s 59MB | 96.5/96.4 · 3.4s 151MB | 96.5/96.4 · 3.3s 50MB | n/m | 29988/31105 |
| hbo-rwpdfrtl-rut | hbo | 8 | 85 | 95.3/95.3 · 0.3s 36MB | 0.0/0.0 · 0.1s 36MB | 95.3/95.3 · 0.7s 140MB | 2.4/2.4 · 0.0s 36MB | 0.0/0.0 · 4.9s\* – | 0/85 |
| engoebus-ebiblea4 | en | 719 | 13894 | 99.0/98.9 · 39.8s 118MB | 99.0/98.9 · 2.2s 50MB | 99.0/98.9 · 2.2s 139MB | 98.6/98.5 · 2.1s 50MB | n/m | 13558/13894 |
| engbrenton-rlo2col | en | 762 | 22970 | 87.0/78.5 · 51.2s 284MB | 87.4/79.0 · 2.6s 62MB | 87.4/79.0 · 3.2s 199MB | 66.3/32.8 · 2.4s 50MB | n/m | 14409/22358 |
| deu1912-rdropcap | de | 1641 | 31102 | 95.7/87.4 · 66.5s 242MB | 95.7/87.4 · 3.6s 54MB | 95.7/87.4 · 3.0s 146MB | 95.7/87.4 · 3.0s 36MB | 95.6/85.1 · 67.6s\* – | 27284/31102 |
| kor-r2col | ko | 1303 | 30991 | 99.0/98.8 · 69.3s 83MB | 99.0/98.9 · 15.5s 46MB | 99.0/98.9 · 2.4s 143MB | 98.0/28.5 · 9.5s 36MB | 98.5/80.7 · 35.8s\* – | 30723/30991 |
| engwebu-rl2col | en | 1415 | 31098 | 94.9/83.5 · 51.1s 89MB | 94.9/83.5 · 2.6s 77MB | 94.9/83.5 · 2.7s 151MB | 92.6/77.6 · 2.4s 36MB | 84.7/61.2 · 48.2s\* – | 7499/31098 |
| engtnt-wp3col | en | 133 | 7954 | 98.7/98.7 · 14.8s 36MB | 98.8/98.8 · 0.8s 36MB | 98.8/98.8 · 1.3s 142MB | 96.5/96.5 · 0.6s 36MB | 97.0/96.7 · 15.9s 55MB | 6040/7954 |
| deutkw-fpdfa5 | de | 389 | 5486 | 96.6/96.6 · 6.4s 63MB | 96.6/96.6 · 0.6s 50MB | 96.6/96.6 · 1.1s 140MB | 96.6/96.6 · 1.2s 50MB | n/m | 0/5465 |
| kor-lopdf | ko | 354 | 7920 | 95.4/87.0 · 33.7s 119MB | 97.3/89.3 · 0.7s 36MB | 97.3/89.3 · 1.4s 148MB | 19.6/12.5 · 2.8s 36MB | 30.7/23.9 · 16.8s 54MB | 65/7920 |
| heb-wprtl | he | 183 | 5852 | 5.5/1.9 · 8.1s 36MB | 0.0/0.0 · 0.5s 36MB | 97.4/80.5 · 1.0s 142MB | 6.1/2.0 · 0.6s 36MB | 0.0/0.0 · 12.8s\* – | 0/5852 |
| F01 | eng | 327 | 11328 | 97.1/97.1 · 22.4s 85MB | 97.4/97.4 · 0.9s 36MB | 97.4/97.4 · 1.5s 142MB | 95.6/95.2 · 1.1s 36MB | 83.2/72.9 · 14.6s\* – | 0/11328 |
| F02 | eng | 1134 | 23143 | 96.4/83.6 · 46.5s 237MB | 96.4/83.6 · 2.3s 55MB | 96.4/83.6 · 2.7s 184MB | 93.1/77.3 · 2.2s 50MB | n/m | 20487/23143 |
| F03 | deu | 1102 | 31163 | 98.0/94.2 · 44.4s 89MB | 97.4/93.1 · 2.5s 76MB | 97.4/93.1 · 2.2s 151MB | 90.4/79.7 · 2.0s 36MB | 82.9/69.2 · 41.9s 79MB | 10848/31142 |
| F06 | por | 372 | 7949 | 96.1/94.8 · 6.5s 68MB | 96.1/94.8 · 0.6s 36MB | 96.1/94.8 · 0.9s 141MB | 96.1/94.8 · 1.2s 36MB | 96.1/94.8 · 6.7s 43MB | 4721/7949 |
| F07 | kor | 1131 | 31084 | 99.0/97.8 · 73.1s 246MB | 99.0/97.8 · 6.0s 57MB | 99.0/97.8 · 3.0s 163MB | 98.1/96.4 · 2.8s 36MB | 98.5/90.9 · 43.1s\* – | 0/31084 |
| F13 | ita | 356 | 4784 | 69.4/63.3 · 11.7s 53MB | 69.4/63.3 · 0.7s 36MB | 69.4/63.3 · 2.4s 140MB | 69.4/63.3 · 0.7s 36MB | 69.2/63.1 · 10.9s\* – | 7/4784 |
| G03 | eng | 816 | 31011 | 99.1/93.1 · 34.3s 70MB | 99.1/93.1 · 2.6s 54MB | 99.1/93.1 · 2.4s 147MB | 67.9/48.8 · 2.0s 36MB | 94.1/73.4 · 32.2s\* – | 0/31011 |
| G04 | heb | 399 | 7727 | 99.5/99.5 · 20.5s 67MB | 0.0/0.0 · 1.6s 36MB | 10.5/8.8 · 1.7s 141MB | 99.6/99.6 · 1.2s 36MB | 0.0/0.0 · 25.9s\* – | 0/7508 |
| G05 | deu | 394 | 8930 | **FAIL** · 6.3s | 89.1/82.0 · 0.9s 36MB | 89.1/82.0 · 1.6s 142MB | 78.0/70.5 · 2.3s 36MB | 45.1/34.4 · 12.7s 44MB | refused (no report) |
| G07 | fra | 264 | 805 | 94.4/94.4 · 12.3s 54MB | 94.4/94.4 · 0.8s 36MB | 94.4/94.4 · 1.2s 140MB | 94.3/94.3 · 0.5s 36MB | 94.2/94.2 · 10.6s\* – | 115/601 |
| G09 | kor | 27 | 764 | 90.4/90.4 · 1.9s 36MB | 90.6/90.6 · 0.2s 36MB | 90.6/90.6 · 0.7s 140MB | 89.4/84.4 · 0.1s 36MB | 87.6/81.4 · 1.2s 36MB | 0/764 |
| G10 | hbo | 39 | 793 | 39.6/39.6 · 4.5s 36MB | 0.0/0.0 · 0.3s 36MB | 2.3/2.3 · 0.9s 140MB | 87.8/87.8 · 0.2s 36MB | 0.0/0.0 · 80.5s\* – | 0/793 |
| G13 | eng | 28 | 1288 | 48.6/48.6 · 1.3s 36MB | 98.1/98.1 · 0.1s 36MB | 98.1/98.1 · 0.8s 141MB | 83.9/82.5 · 0.1s 36MB | 95.3/93.1 · 1.5s 39MB | 0/1288 |

### Summary rows

| metric | current | pdfium | mupdf | poppler | pdfminer |
|---|---|---|---|---|---|
| documents run | 57 | 57 | 57 | 57 | 40 |
| opened | 56/57 | 57/57 | 57/57 | 57/57 | 40/40 |
| mean coverage % (failures count 0) | 88.9 | 87.4 | 91.4 | 84.0 | 78.9 |
| mean in-order % (failures count 0) | 85.5 | 84.0 | 87.7 | 74.6 | 71.5 |
| mean in-order % letters-only (non-Hebrew; failures 0) | 90.6 | 92.4 | 92.4 | 80.1 | 82.9 |
| median in-order % | 95.3 | 95.6 | 95.6 | 87.8 | 83.1 |
| docs with in-order ≥ 90% | 37 | 37 | 38 | 27 | 17 |
| docs with in-order ≥ 50% | 52 | 52 | 54 | 44 | 32 |
| docs where engine is best or within 1 pt of best in-order | 49 | 51 | 53 | 29 | – |
| worst in-order | G05 0.0, heb-wprtl 1.9, engkjvcpb-rwpdfmargin 28.3, G10 39.6 | G04 0.0, G10 0.0, hbo-rwpdfrtl-rut 0.0, hboWLC-pdf 0.0 | G10 2.3, G04 8.8, hboWLC-pdf 25.7, F13 63.3 | heb-wprtl 2.0, hbo-rwpdfrtl-rut 2.4, hboWLC-pdf 4.1, kor-lopdf 12.5 | G04 0.0, G10 0.0, hbo-rwpdfrtl-rut 0.0, hboWLC-pdf 0.0 |
| docs with clean timing | 57 | 57 | 57 | 57 | 7 |
| total wall s (clean-timed docs) | 3416 | 192 | 171 | 189 | 97 |
| median wall s (clean) | 51.13 | 2.56 | 2.83 | 2.43 | 12.70 |
| max wall s (clean) | 185.7 (latVUC-pdf) | 15.5 (kor-r2col) | 8.1 (latVUC-pdf) | 13.0 (kor1961-lopdf) | 41.9 (F03) |
| median peak RSS MB (clean) | 162 | 56 | 147 | 36 | 44 |
| max peak RSS MB (clean) | 573 (spavbl-rlofodt) | 118 (latVUC-pdf) | 219 (spavbl-rlofodt) | 51 (latVUC-pdf) | 79 (F03) |
| G03 (816 pages, large-document test) s / MB | 34.3 / 70.0 | 2.6 / 53.7 | 2.4 / 146.8 | 2.0 / 35.5 | 32.2 / – |
| failures | G05 | none | none | none | none |

Mean figures count a failure as 0. "Docs with clean timing" shows how many documents the clean pass
covered for each engine.

## 5. Sensitivity: letters-only and NFKC

covL/ordL = coverage/in-order with digits dropped as well (so misplaced verse numbers no longer matter);
"ord NFKC" = in-order with NFKC instead of NFC (ligatures such as ﬁ folded). Non-Hebrew documents.

<details><summary>Table</summary>

| doc | current covL/ordL | pdfium covL/ordL | mupdf covL/ordL | poppler covL/ordL | current ord NFKC | pdfium ord NFKC | mupdf ord NFKC | poppler ord NFKC |
|---|---|---|---|---|---|---|---|---|
| engwebp-pdf | 98.2/94.7 | 98.2/94.7 | 98.2/94.7 | 70.3/54.2 | 94.7 | 94.7 | 94.7 | 53.6 |
| engwebp-book | 98.6/97.6 | 98.6/97.6 | 98.6/97.6 | 97.0/89.8 | 97.6 | 97.6 | 97.6 | 91.4 |
| kjv-pdf | 99.5/99.5 | 99.5/99.5 | 99.5/99.5 | 99.5/99.5 | 99.5 | 99.5 | 99.5 | 99.5 |
| deu1912-pdf | 99.7/99.7 | 99.7/99.7 | 99.7/99.7 | 99.7/99.7 | 99.7 | 99.7 | 99.7 | 99.7 |
| deuelo-pdf | 99.7/99.7 | 99.7/99.7 | 99.7/99.7 | 99.7/99.7 | 99.7 | 99.7 | 99.7 | 99.7 |
| fraLSG-pdf | 82.4/63.4 | 82.4/63.4 | 82.4/63.4 | 82.1/63.3 | 63.4 | 63.4 | 63.4 | 63.3 |
| fraLSG-luk | 77.2/77.2 | 77.2/77.2 | 77.2/77.2 | 77.2/77.2 | 77.2 | 77.2 | 77.2 | 77.2 |
| fob-pdf | 99.8/99.8 | 99.8/99.8 | 99.8/99.8 | 99.7/99.7 | 99.8 | 99.8 | 99.8 | 99.7 |
| rv1909-pdf | 97.4/95.7 | 97.4/95.7 | 97.4/95.7 | 97.4/95.7 | 95.7 | 95.7 | 95.7 | 95.7 |
| rv1909-lopdf | 100.0/100.0 | 100.0/100.0 | 100.0/100.0 | 100.0/100.0 | 100.0 | 100.0 | 100.0 | 100.0 |
| spabes-pdf | 99.8/99.8 | 99.8/99.8 | 99.8/99.8 | 99.7/98.0 | 99.8 | 99.8 | 99.8 | 98.0 |
| porbrbsl-pdf | 99.0/96.0 | 99.0/96.0 | 99.0/96.0 | 98.7/93.4 | 96.0 | 96.0 | 96.0 | 88.0 |
| porbr2018-pdf | 99.9/99.9 | 99.9/99.9 | 99.9/99.9 | 99.9/99.9 | 99.9 | 99.9 | 99.9 | 99.9 |
| kor-pdf | 99.5/99.5 | 99.5/99.5 | 99.5/99.5 | 81.5/54.7 | 99.5 | 99.5 | 99.5 | 57.6 |
| grctr-pdf | 98.3/90.3 | 98.3/90.3 | 98.3/90.3 | 66.1/44.6 | 90.3 | 90.3 | 90.3 | 44.5 |
| latVUC-pdf | 96.0/89.3 | 96.0/89.3 | 96.0/89.3 | 58.5/31.6 | 89.3 | 89.3 | 89.3 | 31.4 |
| deutkw-pdf | 99.6/99.1 | 99.6/99.1 | 99.6/99.1 | 99.5/99.0 | 99.1 | 99.1 | 99.1 | 99.0 |
| frajnd-pdf | 84.9/69.5 | 84.9/69.5 | 84.9/69.5 | 84.9/69.4 | 69.5 | 69.5 | 69.5 | 69.5 |
| spablm-pdf | 99.1/96.1 | 99.1/96.1 | 99.1/96.1 | 99.0/96.0 | 96.1 | 96.1 | 96.1 | 95.7 |
| engylt-pdf | 99.9/99.9 | 99.9/99.9 | 99.9/99.9 | 99.9/99.9 | 99.9 | 99.9 | 99.9 | 99.9 |
| grcbyz-pdf | 98.7/98.7 | 98.7/98.7 | 98.7/98.7 | 76.2/51.1 | 98.7 | 98.7 | 98.7 | 51.0 |
| kor1961-lopdf | 100.0/100.0 | 100.0/100.0 | 100.0/100.0 | 100.0/100.0 | 98.8 | 100.0 | 100.0 | 100.0 |
| deutkw-lopdf | 100.0/100.0 | 100.0/100.0 | 100.0/100.0 | 100.0/100.0 | 100.0 | 100.0 | 100.0 | 100.0 |
| frajnd-lopdf | 100.0/100.0 | 100.0/100.0 | 100.0/100.0 | 100.0/100.0 | 100.0 | 100.0 | 100.0 | 99.9 |
| engbsb-rwpdf2col | 89.1/65.7 | 89.1/65.7 | 89.1/65.7 | 80.5/42.3 | 65.6 | 65.6 | 65.6 | 42.2 |
| engasv-rrlab1col | 97.1/91.4 | 97.1/91.3 | 97.1/91.3 | 97.1/91.4 | 91.4 | 91.3 | 91.3 | 91.4 |
| spavbl-rlofodt | 88.2/86.7 | 88.2/86.7 | 88.2/86.7 | 88.2/86.7 | 86.0 | 86.0 | 86.0 | 86.0 |
| grcsbl-rfpdf-mrk | 100.0/100.0 | 100.0/100.0 | 100.0/100.0 | 100.0/100.0 | 96.9 | 96.9 | 96.9 | 96.6 |
| engrv-ebiblebook | 99.9/99.9 | 99.9/99.9 | 99.9/99.9 | 99.9/99.9 | 99.9 | 99.9 | 99.9 | 99.9 |
| engkjvcpb-rwpdfmargin | 96.2/84.9 | 96.2/84.8 | 96.2/84.8 | 96.2/84.8 | 28.3 | 84.8 | 84.8 | 29.2 |
| engjps-rdocx2lo | 96.5/76.0 | 96.5/76.0 | 96.5/76.0 | 94.5/44.0 | 82.7 | 82.7 | 82.7 | 46.5 |
| spabll-rrlab2col | 99.0/99.0 | 98.8/98.8 | 98.8/98.8 | 97.3/39.7 | 73.2 | 95.6 | 95.6 | 38.9 |
| poronbv-rwpdfa5 | 96.5/96.4 | 96.5/96.5 | 96.5/96.5 | 96.5/96.4 | 96.4 | 96.4 | 96.4 | 96.4 |
| engoebus-ebiblea4 | 99.0/89.0 | 99.0/89.0 | 99.0/89.0 | 98.6/88.8 | 98.9 | 98.9 | 98.9 | 98.5 |
| engbrenton-rlo2col | 96.5/91.2 | 96.5/91.2 | 96.5/91.2 | 73.3/37.1 | 78.5 | 79.0 | 79.0 | 32.8 |
| deu1912-rdropcap | 95.7/86.8 | 95.9/87.0 | 95.9/87.0 | 95.7/86.8 | 87.4 | 87.4 | 87.4 | 87.4 |
| kor-r2col | 99.0/98.8 | 99.0/98.9 | 99.0/98.9 | 98.0/26.8 | 98.8 | 98.9 | 98.9 | 28.5 |
| engwebu-rl2col | 94.9/83.5 | 94.9/83.5 | 94.9/83.5 | 92.8/77.8 | 83.5 | 83.5 | 83.5 | 77.6 |
| engtnt-wp3col | 98.7/98.7 | 98.8/98.8 | 98.8/98.8 | 97.6/97.6 | 98.7 | 98.8 | 98.8 | 96.5 |
| deutkw-fpdfa5 | 96.6/96.6 | 96.6/96.6 | 96.6/96.6 | 96.6/96.6 | 96.6 | 96.6 | 96.6 | 96.6 |
| kor-lopdf | 95.5/87.0 | 97.3/89.3 | 97.3/89.3 | 20.5/12.1 | 87.0 | 89.3 | 89.3 | 12.5 |
| F01 | 97.1/97.1 | 97.4/97.4 | 97.4/97.4 | 97.3/96.8 | 97.1 | 97.4 | 97.4 | 95.2 |
| F02 | 96.4/76.9 | 96.4/76.9 | 96.4/76.9 | 96.3/76.6 | 83.6 | 83.6 | 83.6 | 77.3 |
| F03 | 98.0/94.2 | 97.4/93.1 | 97.4/93.1 | 97.4/92.7 | 94.2 | 93.1 | 93.1 | 79.7 |
| F06 | 96.1/94.9 | 96.1/94.9 | 96.1/94.9 | 96.1/94.9 | 94.8 | 94.8 | 94.8 | 94.8 |
| F07 | 99.0/97.8 | 99.0/97.8 | 99.0/97.8 | 99.0/97.7 | 97.8 | 97.8 | 97.8 | 96.4 |
| F13 | 69.4/63.3 | 69.4/63.3 | 69.4/63.3 | 69.4/63.3 | 63.3 | 63.3 | 63.3 | 63.3 |
| G03 | 100.0/100.0 | 100.0/100.0 | 100.0/100.0 | 70.6/54.9 | 93.1 | 93.1 | 93.1 | 48.8 |
| G05 | **FAIL** | 93.9/93.9 | 93.9/93.9 | 82.3/80.8 | – | 82.0 | 82.0 | 70.5 |
| G07 | 94.4/94.4 | 94.4/94.4 | 94.4/94.4 | 94.3/94.3 | 94.4 | 94.4 | 94.4 | 94.3 |
| G09 | 97.6/97.6 | 97.8/97.8 | 97.8/97.8 | 97.0/91.8 | 90.4 | 90.6 | 90.6 | 84.4 |
| G13 | 97.9/97.9 | 98.1/98.1 | 98.1/98.1 | 96.7/95.3 | 48.6 | 98.1 | 98.1 | 82.5 |

</details>

Findings: for 47 of the 57 documents the current extractor, PDFium and MuPDF are within 2 points of each
other on every variant — on ordinary single- and two-column PDFs the choice of parser does not change the
text. The differences are concentrated in four groups: Hebrew (§ 6), G05 (cannot open), kor-lopdf (current
87.0 vs 89.3) and the three verse-number-placement files (G13 48.6 vs 98.1 with digits; 97.9 vs 98.1
letters-only). NFKC changes nothing measurable on this corpus.

Engine variants (cov/ord):

<details><summary>MuPDF sort=True and pdftotext -raw</summary>

| doc | mupdf (stream order) | mupdf sort=True | poppler (default layout analysis) | poppler -raw |
|---|---|---|---|---|
| engwebp-pdf | 98.2/94.7 | 3.6/2.6 | 69.5/53.6 | 98.2/94.7 |
| engwebp-book | 98.6/97.6 | 3.6/3.0 | 96.9/91.4 | 98.6/97.6 |
| kjv-pdf | 99.5/99.5 | 99.5/99.5 | 99.5/99.5 | 99.5/99.5 |
| deu1912-pdf | 99.7/99.7 | 99.7/99.7 | 99.7/99.7 | 99.7/99.7 |
| deuelo-pdf | 99.7/99.7 | 99.7/99.7 | 99.7/99.7 | 99.7/99.7 |
| fraLSG-pdf | 82.4/63.4 | 82.4/63.4 | 82.0/63.3 | 82.4/63.4 |
| fraLSG-luk | 77.2/77.2 | 77.2/77.2 | 77.2/77.2 | 77.2/77.2 |
| fob-pdf | 99.8/99.8 | 99.8/99.8 | 99.7/99.7 | 99.8/99.8 |
| rv1909-pdf | 97.4/95.7 | 97.4/95.7 | 97.4/95.7 | 97.4/95.7 |
| rv1909-lopdf | 100.0/100.0 | 100.0/100.0 | 100.0/100.0 | 100.0/100.0 |
| spabes-pdf | 99.8/99.8 | 99.8/99.8 | 99.7/98.0 | 99.8/99.8 |
| porbrbsl-pdf | 99.0/96.0 | 99.0/96.0 | 98.5/88.0 | 99.0/96.0 |
| porbr2018-pdf | 99.9/99.9 | 99.9/99.9 | 99.9/99.9 | 99.9/99.9 |
| kor-pdf | 99.5/99.5 | 9.5/6.0 | 80.7/57.6 | 99.5/99.5 |
| grctr-pdf | 98.3/90.3 | 3.6/2.9 | 66.0/44.5 | 98.3/90.3 |
| hboWLC-pdf | 26.3/25.7 | 0.0/0.0 | 4.8/4.1 | 0.0/0.0 |
| latVUC-pdf | 96.0/89.3 | 6.6/3.3 | 57.8/31.4 | 96.0/89.3 |
| deutkw-pdf | 99.6/99.1 | 99.6/99.1 | 99.5/99.0 | 99.6/99.1 |
| frajnd-pdf | 84.9/69.5 | 84.9/69.5 | 84.8/69.5 | 84.9/69.5 |
| spablm-pdf | 99.1/96.1 | 99.1/96.1 | 98.8/95.7 | 99.1/96.1 |
| engylt-pdf | 99.9/99.9 | 99.9/99.9 | 99.9/99.9 | 99.9/99.9 |
| grcbyz-pdf | 98.7/98.7 | 4.7/4.3 | 76.0/51.0 | 98.7/98.7 |
| kor1961-lopdf | 100.0/100.0 | 100.0/100.0 | 100.0/100.0 | 100.0/100.0 |
| deutkw-lopdf | 100.0/100.0 | 100.0/100.0 | 100.0/100.0 | 100.0/100.0 |
| frajnd-lopdf | 100.0/100.0 | 100.0/100.0 | 99.9/99.9 | 100.0/100.0 |
| engbsb-rwpdf2col | 89.1/65.6 | 7.4/3.3 | 80.0/42.2 | 89.1/65.6 |
| engasv-rrlab1col | 97.1/91.3 | 97.1/91.4 | 97.1/91.4 | 97.1/91.3 |
| spavbl-rlofodt | 88.2/86.0 | 88.2/86.0 | 88.2/86.0 | 88.2/86.0 |
| grcsbl-rfpdf-mrk | 96.9/96.9 | 96.9/96.9 | 96.6/96.6 | 96.9/96.9 |
| engrv-ebiblebook | 99.9/99.9 | 99.9/99.9 | 99.9/99.9 | 99.9/99.9 |
| engkjvcpb-rwpdfmargin | 96.2/84.8 | 36.2/28.2 | 37.9/29.2 | 96.2/84.8 |
| engjps-rdocx2lo | 96.4/82.7 | 1.9/1.3 | 94.4/46.5 | 96.4/82.7 |
| spabll-rrlab2col | 97.3/95.6 | 6.0/3.8 | 95.5/38.9 | 97.3/95.6 |
| poronbv-rwpdfa5 | 96.5/96.4 | 96.5/96.4 | 96.5/96.4 | 96.5/96.4 |
| hbo-rwpdfrtl-rut | 95.3/95.3 | 0.0/0.0 | 2.4/2.4 | 0.0/0.0 |
| engoebus-ebiblea4 | 99.0/98.9 | 99.0/98.9 | 98.6/98.5 | 99.0/98.9 |
| engbrenton-rlo2col | 87.4/79.0 | 5.1/2.4 | 66.3/32.8 | 87.4/79.0 |
| deu1912-rdropcap | 95.7/87.4 | 90.5/73.5 | 95.7/87.4 | 95.7/87.4 |
| kor-r2col | 99.0/98.9 | 4.8/3.0 | 98.0/28.5 | 99.0/98.9 |
| engwebu-rl2col | 94.9/83.5 | 5.3/4.1 | 92.6/77.6 | 94.9/83.5 |
| engtnt-wp3col | 98.8/98.8 | 0.5/0.5 | 96.5/96.5 | 98.8/98.8 |
| deutkw-fpdfa5 | 96.6/96.6 | 96.6/96.6 | 96.6/96.6 | 96.6/96.6 |
| kor-lopdf | 97.3/89.3 | 6.2/5.2 | 19.6/12.5 | 97.3/89.3 |
| heb-wprtl | 97.4/80.5 | 0.0/0.0 | 6.1/2.0 | 0.0/0.0 |
| F01 | 97.4/97.4 | 0.4/0.4 | 95.6/95.2 | 97.4/97.4 |
| F02 | 96.4/83.6 | 2.1/1.3 | 93.1/77.3 | 96.4/83.6 |
| F03 | 97.4/93.1 | 3.2/2.6 | 90.4/79.7 | 97.4/93.1 |
| F06 | 96.1/94.8 | 96.1/94.8 | 96.1/94.8 | 96.1/94.8 |
| F07 | 99.0/97.8 | 4.0/3.3 | 98.1/96.4 | 99.0/97.8 |
| F13 | 69.4/63.3 | 69.4/63.3 | 69.4/63.3 | 69.4/63.3 |
| G03 | 99.1/93.1 | 4.2/2.9 | 67.9/48.8 | 99.1/93.1 |
| G04 | 10.5/8.8 | 0.0/0.0 | 99.6/99.6 | 0.0/0.0 |
| G05 | 89.1/82.0 | 0.8/0.6 | 78.0/70.5 | 89.1/82.0 |
| G07 | 94.4/94.4 | 91.2/91.2 | 94.3/94.3 | 94.4/94.4 |
| G09 | 90.6/90.6 | 9.4/9.3 | 89.4/84.4 | 90.6/90.6 |
| G10 | 2.3/2.3 | 0.0/0.0 | 87.8/87.8 | 0.0/0.0 |
| G13 | 98.1/98.1 | 2.6/2.3 | 83.9/82.5 | 98.1/98.1 |

</details>

`pdftotext -raw` (content-stream order) matches PDFium/MuPDF almost exactly (mean in-order 84.0 % vs
74.6 % for Poppler's default layout analysis); MuPDF `sort=True` is much worse (mean 48.8 %) because it
interleaves columns. Poppler's default mode is the one that collapses on two-column Korean (kor-r2col 28.5 %,
kor-lopdf 12.5 %) and Greek (grctr 44.5 %, grcbyz 51.0 %) and on margin numbers.

## 6. Right-to-left, Korean and Greek

**Hebrew** (cov/ord %; headline = points stripped):

| doc | engine | cov/ord pointed (NFC) | cov/ord points stripped | line-reversed (stripped) | word-order-reversed (stripped) |
|---|---|---|---|---|---|
| hboWLC-pdf | current | 0.0/0.0 | 65.9/65.8 | 0.0/0.0 | 0.0/0.0 |
| hboWLC-pdf | pdfium | 0.0/0.0 | 0.0/0.0 | 1.9/1.9 | 1.6/1.6 |
| hboWLC-pdf | mupdf | 0.4/0.4 | 26.3/25.7 | 0.0/0.0 | 0.0/0.0 |
| hboWLC-pdf | poppler | 0.0/0.0 | 4.8/4.1 | 0.0/0.0 | 0.0/0.0 |
| hboWLC-pdf | pdfminer | 0.0/0.0 | 0.0/0.0 | 0.1/0.1 | 0.0/0.0 |
| hboWLC-pdf | mupdf:sort | 0.0/0.0 | 0.0/0.0 | 0.0/0.0 | 0.4/0.4 |
| hboWLC-pdf | poppler:raw | 0.0/0.0 | 0.0/0.0 | 0.0/0.0 | 0.0/0.0 |
| hbo-rwpdfrtl-rut | current | 0.0/0.0 | 95.3/95.3 | 0.0/0.0 | 0.0/0.0 |
| hbo-rwpdfrtl-rut | pdfium | 0.0/0.0 | 0.0/0.0 | 0.0/0.0 | 0.0/0.0 |
| hbo-rwpdfrtl-rut | mupdf | 70.6/70.6 | 95.3/95.3 | 0.0/0.0 | 0.0/0.0 |
| hbo-rwpdfrtl-rut | poppler | 0.0/0.0 | 2.4/2.4 | 0.0/0.0 | 0.0/0.0 |
| hbo-rwpdfrtl-rut | pdfminer | 0.0/0.0 | 0.0/0.0 | 0.0/0.0 | 0.0/0.0 |
| hbo-rwpdfrtl-rut | mupdf:sort | 0.0/0.0 | 0.0/0.0 | 0.0/0.0 | 71.8/71.8 |
| hbo-rwpdfrtl-rut | poppler:raw | 0.0/0.0 | 0.0/0.0 | 0.0/0.0 | 0.0/0.0 |
| heb-wprtl | current | 5.5/1.9 | 5.5/1.9 | 0.0/0.0 | 0.0/0.0 |
| heb-wprtl | pdfium | 0.0/0.0 | 0.0/0.0 | 0.0/0.0 | 97.5/80.6 |
| heb-wprtl | mupdf | 97.4/80.5 | 97.4/80.5 | 0.0/0.0 | 0.1/0.1 |
| heb-wprtl | poppler | 6.1/2.0 | 6.1/2.0 | 0.0/0.0 | 0.0/0.0 |
| heb-wprtl | pdfminer | 0.0/0.0 | 0.0/0.0 | 5.1/1.6 | 0.0/0.0 |
| heb-wprtl | mupdf:sort | 0.0/0.0 | 0.0/0.0 | 0.0/0.0 | 5.7/1.9 |
| heb-wprtl | poppler:raw | 0.0/0.0 | 0.0/0.0 | 0.0/0.0 | 97.5/80.6 |
| G04 | current | 99.1/99.1 | 99.5/99.5 | 0.0/0.0 | 0.0/0.0 |
| G04 | pdfium | 0.0/0.0 | 0.0/0.0 | 0.0/0.0 | 99.0/99.0 |
| G04 | mupdf | 10.5/8.7 | 10.5/8.8 | 0.0/0.0 | 0.0/0.0 |
| G04 | poppler | 99.1/99.1 | 99.6/99.6 | 0.0/0.0 | 0.0/0.0 |
| G04 | pdfminer | 0.0/0.0 | 0.0/0.0 | 99.2/99.2 | 0.0/0.0 |
| G04 | mupdf:sort | 0.0/0.0 | 0.0/0.0 | 0.0/0.0 | 99.0/99.0 |
| G04 | poppler:raw | 0.0/0.0 | 0.0/0.0 | 0.0/0.0 | 98.9/95.9 |
| G10 | current | 0.0/0.0 | 39.6/39.6 | 0.0/0.0 | 0.0/0.0 |
| G10 | pdfium | 0.0/0.0 | 0.0/0.0 | 0.1/0.1 | 0.0/0.0 |
| G10 | mupdf | 0.0/0.0 | 2.3/2.3 | 0.0/0.0 | 0.0/0.0 |
| G10 | poppler | 0.0/0.0 | 87.8/87.8 | 0.0/0.0 | 0.0/0.0 |
| G10 | pdfminer | 0.0/0.0 | 0.0/0.0 | 0.0/0.0 | 0.0/0.0 |
| G10 | mupdf:sort | 0.0/0.0 | 0.0/0.0 | 0.0/0.0 | 0.0/0.0 |
| G10 | poppler:raw | 0.0/0.0 | 0.0/0.0 | 0.0/0.0 | 0.1/0.1 |

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

Measured counts per document (all three engines agree exactly on outline counts for 57/57 documents,
on link counts for 57/57 and on page-label counts for 57/57): 48 documents have an
outline, 32 have link annotations, 23 have page labels, 12 are tagged (StructTreeRoot), 4 carry
ActualText in content streams (G09, G10, engbrenton-rlo2col, kor-lopdf).

<details><summary>Per-document structure counts</summary>

| doc | outline entries (pdfium/mupdf/poppler-xml) | link annots (pdfium/mupdf; poppler <a> spans) | pages with label (pdfium/mupdf) | tagged: pages with struct tree (pdfium) / StructTreeRoot (mupdf) / Tagged (pdfinfo) | ActualText (streams) | fonts seen (pdfium sample/mupdf spans/poppler fontspecs) | char boxes ok (pdfium sample) / mupdf chars / poppler words |
|---|---|---|---|---|---|---|---|
| engwebp-pdf | 68/68/68 | 1641/1641; 2251 | 1273/1273 | 0/False/no | 0 | 7/9/26 | 52202/4111745/821959 |
| engwebp-book | 68/68/68 | 1641/1641; 2300 | 1065/1065 | 0/False/no | 0 | 8/9/19 | 43627/4107466/820713 |
| kjv-pdf | 66/66/66 | 7029/7029; 7916 | 1561/1561 | 0/False/no | 0 | 5/7/20 | 63943/4492622/893028 |
| deu1912-pdf | 66/66/66 | 68/68; 74 | 1344/1344 | 0/False/no | 0 | 3/3/10 | 55094/4103406/740550 |
| deuelo-pdf | 66/66/66 | 70/70; 76 | 1187/1187 | 0/False/no | 0 | 3/4/8 | 48618/4335012/764786 |
| fraLSG-pdf | 66/66/66 | 9819/9819; 10270 | 1755/1755 | 0/False/no | 0 | 5/7/32 | 71909/4826596/899598 |
| fraLSG-luk | 1/1/1 | 547/547; 680 | 133/133 | 0/False/no | 0 | 5/5/21 | 5477/169726/32507 |
| fob-pdf | 66/66/66 | 68/68; 90 | 1348/1348 | 0/False/no | 0 | 3/3/11 | 55252/4243498/783698 |
| rv1909-pdf | 66/66/66 | 68/68; 88 | 1004/1004 | 0/False/no | 0 | 5/5/14 | 41150/3886178/744445 |
| rv1909-lopdf | 66/66/66 | 0/0; 0 | 0/0 | 881/True/yes | 0 | 2/2/3 | 36135/3899023/736132 |
| spabes-pdf | 66/66/66 | 69/69; 90 | 1359/1359 | 0/False/no | 0 | 4/5/14 | 55717/3767406/715890 |
| porbrbsl-pdf | 81/81/81 | 2398/2398; 3151 | 1898/1898 | 0/False/no | 0 | 8/8/31 | 77804/5063102/955322 |
| porbr2018-pdf | 66/66/66 | 1362/1362; 1388 | 1094/1094 | 0/False/no | 0 | 6/7/16 | 44816/3924469/753273 |
| kor-pdf | 66/66/66 | 68/68; 75 | 796/796 | 0/False/no | 0 | 4/4/13 | 32611/1445433/501928 |
| grctr-pdf | 28/28/28 | 373/373; 586 | 339/339 | 0/False/no | 0 | 7/7/21 | 13855/845294/155117 |
| hboWLC-pdf | 39/39/39 | 41/41; 42 | 913/913 | 0/False/no | 0 | 6/6/16 | 37385/2844335/1528466 |
| latVUC-pdf | 0/0/0 | 13777/13777; 17438 | 0/0 | 0/False/no | 0 | 7/7/18 | 87930/9091260/1432587 |
| deutkw-pdf | 66/66/66 | 70/70; 76 | 1490/1490 | 0/False/no | 0 | 5/6/15 | 61053/4395171/765347 |
| frajnd-pdf | 66/66/66 | 6409/6409; 7468 | 1550/1550 | 0/False/no | 0 | 6/7/16 | 63488/4577616/890314 |
| spablm-pdf | 82/82/82 | 1533/1533; 1875 | 1845/1845 | 0/False/no | 0 | 7/9/31 | 75602/4959000/947107 |
| engylt-pdf | 66/66/66 | 69/69; 69 | 1088/1088 | 0/False/no | 0 | 3/3/7 | 44561/4220874/835121 |
| grcbyz-pdf | 32/32/32 | 57/57; 71 | 323/323 | 0/False/no | 0 | 10/10/31 | 13204/899896/165356 |
| kor1961-lopdf | 1255/1255/1255 | 0/0; 0 | 0/0 | 1119/True/yes | 0 | 5/5/23 | 45972/1907160/504591 |
| deutkw-lopdf | 1255/1255/1255 | 0/0; 0 | 0/0 | 1126/True/yes | 0 | 2/2/5 | 46171/4390298/754789 |
| frajnd-lopdf | 1255/1255/1255 | 0/0; 0 | 0/0 | 1156/True/yes | 0 | 3/4/6 | 47365/4293170/824773 |
| engbsb-rwpdf2col | 3318/3318/3318 | 0/0; 0 | 0/0 | 0/False/no | 0 | 5/5/17 | 40707/4251216/845686 |
| engasv-rrlab1col | 0/0/0 | 0/0; 0 | 0/0 | 0/False/no | 0 | 3/3/15 | 56620/4253994/829380 |
| spavbl-rlofodt | 1255/1255/1255 | 8984/8984; 11910 | 0/0 | 1691/True/yes | 0 | 3/3/12 | 69366/4555069/846207 |
| grcsbl-rfpdf-mrk | 0/0/0 | 0/0; 0 | 0/0 | 0/False/no | 0 | 2/2/4 | 1518/114846/21377 |
| engrv-ebiblebook | 80/80/80 | 1435/1435; 1440 | 1337/1337 | 0/False/no | 0 | 5/6/16 | 54782/5057127/998132 |
| engkjvcpb-rwpdfmargin | 1190/1190/1190 | 0/0; 0 | 0/0 | 0/False/no | 0 | 3/3/12 | 70367/4258086/856473 |
| engjps-rdocx2lo | 39/39/39 | 948/948; 948 | 0/0 | 949/True/yes | 0 | 3/3/8 | 38937/3299570/640674 |
| spabll-rrlab2col | 0/0/0 | 0/0; 0 | 0/0 | 0/False/no | 0 | 3/3/10 | 30409/4977098/960002 |
| poronbv-rwpdfa5 | 1256/1256/1256 | 0/0; 0 | 0/0 | 0/False/no | 0 | 5/5/12 | 89348/4531872/859688 |
| hbo-rwpdfrtl-rut | 5/5/5 | 0/0; 0 | 0/0 | 0/False/no | 0 | 4/4/8 | 330/12200/7070 |
| engoebus-ebiblea4 | 45/45/45 | 111/111; 124 | 719/719 | 0/False/no | 0 | 5/6/23 | 29456/1711640/340150 |
| engbrenton-rlo2col | 1170/1170/1170 | 5492/5492; 9913 | 0/0 | 762/True/yes | 37 | 3/3/9 | 31227/4199802/822391 |
| deu1912-rdropcap | 66/66/66 | 0/0; 0 | 0/0 | 0/False/no | 0 | 3/3/6 | 67280/4112808/751468 |
| kor-r2col | 1254/1254/1254 | 0/0; 0 | 0/0 | 0/False/no | 0 | 2/2/5 | 53472/1884947/543277 |
| engwebu-rl2col | 0/0/0 | 0/0; 0 | 0/0 | 0/False/no | 0 | 3/4/26 | 57937/5081577/1006878 |
| engtnt-wp3col | 28/28/28 | 0/0; 0 | 0/0 | 0/False/no | 0 | 3/3/9 | 5464/958314/190767 |
| deutkw-fpdfa5 | 0/0/0 | 0/0; 0 | 0/0 | 0/False/no | 0 | 3/3/13 | 15957/916792/157708 |
| kor-lopdf | 286/286/286 | 0/0; 0 | 0/0 | 354/True/yes | 1 | 3/3/13 | 14588/448472/121358 |
| heb-wprtl | 194/194/194 | 15/15; 242 | 0/0 | 0/False/no | 0 | 4/4/11 | 7502/398479/88351 |
| F01 | 29/29/29 | 0/0; 0 | 0/0 | 0/False/no | 0 | 5/6/16 | 13406/1218172/245326 |
| F02 | 39/39/39 | 0/0; 0 | 0/0 | 1134/True/yes | 0 | 3/3/8 | 46505/3438753/664489 |
| F03 | 0/0/0 | 0/0; 0 | 0/0 | 0/False/no | 0 | 3/3/10 | 45134/5314961/915666 |
| F06 | 27/27/27 | 0/0; 0 | 0/0 | 0/False/no | 0 | 3/3/10 | 15234/955105/173541 |
| F07 | 66/66/66 | 66/66; 66 | 0/0 | 1131/True/yes | 0 | 2/2/8 | 46316/1840876/552287 |
| F13 | 462/462/462 | 15/15; 180 | 0/0 | 0/False/no | 0 | 3/3/15 | 14606/593252/116301 |
| G03 | 1184/1184/1184 | 0/0; 0 | 0/0 | 0/False/no | 0 | 6/8/13 | 33405/4102210/823364 |
| G04 | 27/27/27 | 238/238; 295 | 399/399 | 0/False/no | 0 | 4/5/16 | 16326/735250/147186 |
| G05 | 0/0/0 | 0/0; 0 | 0/0 | 0/False/no | 0 | 3/3/13 | 16152/1080576/219987 |
| G07 | 11/11/11 | 27/27; 542 | 0/0 | 0/False/no | 0 | 3/4/12 | 10831/705157/132009 |
| G09 | 48/48/48 | 135/135; 198 | 0/0 | 27/True/yes | 25 | 2/2/10 | 1145/49121/13892 |
| G10 | 6/6/6 | 0/0; 0 | 0/0 | 39/True/yes | 38 | 5/5/10 | 1601/120560/25578 |
| G13 | 0/0/0 | 0/0; 0 | 0/0 | 0/False/no | 0 | 3/3/12 | 1148/156434/31443 |

</details>

## 8. Damaged copies (malformed but readable)

Copies in `/root/a2-pdf/damaged/` of G13 (ReportLab), grcsbl-rfpdf-mrk (fpdf2) and G09 (Korean, tagged):
(i) every in-use offset of the classic xref table shifted by 13 bytes (startxref left correct);
(ii) file cut at the start of the final `xref` keyword — all objects present, no xref/trailer/startxref;
(iii) the `/Length` of page 4's content stream halved (space-padded so that no other offset moves).
"cov" = coverage % (headline normalisation); "orig" = the same engine on the intact file.

| damaged copy | damage | current | pdfium | mupdf | poppler | pdfminer |
|---|---|---|---|---|---|---|
| G13~xref | xref offsets +13 | **FAIL**: library.badPdf (lopdf Document::load_mem failed) | opens, 28/28 pages text, cov 98.1 (orig 98.1) | opens, 28/28 pages text, cov 98.1 (orig 98.1) | opens, 28/28 pages text, cov 83.9 (orig 83.9) | **silent failure**: opens, 0 pages, no text |
| G13~trunc | cut at byte 222408 of 224105 | **FAIL**: library.badPdf (lopdf Document::load_mem failed) | **FAIL**: PdfiumError: Failed to load document (PDFium: Data format error). | opens, 28/28 pages text, cov 98.1 (orig 98.1) | **FAIL**: Syntax Error: Couldn't find trailer dictionary Syntax Error: Couldn't find  | **FAIL**: PDFSyntaxError: No /Root object! - Is this really a PDF? |
| G13~length | page 4 content stream obj 49: /Length 5275 -> 2637 | opens, 27/28 pages text, cov 47.0 (orig 48.6) | opens, 28/28 pages text, cov 98.1 (orig 98.1) | opens, 28/28 pages text, cov 98.1 (orig 98.1) | opens, 28/28 pages text, cov 83.9 (orig 83.9) | opens, 28/28 pages text, cov 93.8 (orig 95.3) |
| grcsbl-rfpdf-mrk~xref | xref offsets +13 | **FAIL**: library.badPdf (lopdf Document::load_mem failed) | opens, 37/37 pages text, cov 96.9 (orig 96.9) | opens, 37/37 pages text, cov 96.9 (orig 96.9) | opens, 37/37 pages text, cov 96.6 (orig 96.6) | **silent failure**: opens, 0 pages, no text |
| grcsbl-rfpdf-mrk~trunc | cut at byte 165461 of 168340 | **FAIL**: library.badPdf (lopdf Document::load_mem failed) | **FAIL**: PdfiumError: Failed to load document (PDFium: Data format error). | opens, 37/37 pages text, cov 96.9 (orig 96.9) | **FAIL**: Syntax Error: Couldn't find trailer dictionary Syntax Error: Couldn't find  | **FAIL**: PDFSyntaxError: No /Root object! - Is this really a PDF? |
| grcsbl-rfpdf-mrk~length | page 4 content stream obj 10: /Length 3559 -> 1779 | opens, 36/37 pages text, cov 95.2 (orig 96.9) | opens, 37/37 pages text, cov 96.9 (orig 96.9) | opens, 37/37 pages text, cov 96.9 (orig 96.9) | opens, 37/37 pages text, cov 96.6 (orig 96.6) | opens, 37/37 pages text, cov 89.6 |
| G09~xref | xref offsets +13 | **FAIL**: library.badPdf (lopdf Document::load_mem failed) | opens, 27/27 pages text, cov 90.6 (orig 90.6) | opens, 27/27 pages text, cov 90.6 (orig 90.6) | opens, 27/27 pages text, cov 89.4 (orig 89.4) | **silent failure**: opens, 0 pages, no text |
| G09~trunc | cut at byte 395686 of 416768 | **FAIL**: library.badPdf (lopdf Document::load_mem failed) | **FAIL**: PdfiumError: Failed to load document (PDFium: Data format error). | opens, 27/27 pages text, cov 90.6 (orig 90.6) | **FAIL**: Syntax Error: Couldn't find trailer dictionary Syntax Error: Couldn't find  | **FAIL**: PDFSyntaxError: No /Root object! - Is this really a PDF? |
| G09~length | page 4 content stream obj 1019: /Length 5942 -> 2971 | opens, 26/27 pages text, cov 87.0 (orig 90.4) | opens, 27/27 pages text, cov 90.6 (orig 90.6) | opens, 27/27 pages text, cov 90.2 (orig 90.6) | opens, 27/27 pages text, cov 89.4 (orig 89.4) | opens, 27/27 pages text, cov 85.6 (orig 87.6) |

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

| doc | app verified | current in-order (spec / letters-only) | best other engine in-order | class | note |
|---|---|---|---|---|---|
| hboWLC-pdf | 0/23192 (0.0%) | 65.8 / – | mupdf 25.7 | RECOGNITION (partly document: text layer limits every engine) | current is the best or equal engine at 65.8% in order; the app verified far less |
| latVUC-pdf | 8527/31229 (27.3%) | 89.3 / 89.3 | pdfium 89.3 | RECOGNITION |  |
| spavbl-rlofodt | 3453/31086 (11.1%) | 86.0 / 86.7 | pdfium 86.0 | RECOGNITION |  |
| grcsbl-rfpdf-mrk | 0/673 (0.0%) | 96.9 / 100.0 | pdfium 96.9 | RECOGNITION |  |
| engkjvcpb-rwpdfmargin | 1/31102 (0.0%) | 28.3 / 84.9 | pdfium 84.8 | EXTRACTION (verse-number placement only) | words complete and in order (letters-only 84.9% vs best 84.8%); verse numbers emitted out of their inline position |
| spabll-rrlab2col | 0/31098 (0.0%) | 73.2 / 99.0 | pdfium 95.6 | EXTRACTION (verse-number placement only) | words complete and in order (letters-only 99.0% vs best 98.8%); verse numbers emitted out of their inline position |
| hbo-rwpdfrtl-rut | 0/85 (0.0%) | 95.3 / – | mupdf 95.3 | RECOGNITION |  |
| engwebu-rl2col | 7499/31098 (24.1%) | 83.5 / 83.5 | pdfium 83.5 | RECOGNITION (partly document: text layer limits every engine) | current is the best or equal engine at 83.5% in order; the app verified far less |
| deutkw-fpdfa5 | 0/5465 (0.0%) | 96.6 / 96.6 | pdfium 96.6 | RECOGNITION |  |
| kor-lopdf | 65/7920 (0.8%) | 87.0 / 87.0 | pdfium 89.3 | RECOGNITION |  |
| heb-wprtl | 0/5852 (0.0%) | 1.9 / – | mupdf 80.5 | EXTRACTION |  |
| F01 | 0/11328 (0.0%) | 97.1 / 97.1 | pdfium 97.4 | RECOGNITION |  |
| F03 | 10848/31142 (34.8%) | 94.2 / 94.2 | pdfium 93.1 | RECOGNITION |  |
| F07 | 0/31084 (0.0%) | 97.8 / 97.8 | pdfium 97.8 | RECOGNITION |  |
| F13 | 7/4784 (0.1%) | 63.3 / 63.3 | pdfium 63.3 | RECOGNITION (partly document: text layer limits every engine) | current is the best or equal engine at 63.3% in order; the app verified far less |
| G03 | 0/31011 (0.0%) | 93.1 / 100.0 | pdfium 93.1 | RECOGNITION |  |
| G04 | 0/7508 (0.0%) | 99.5 / – | poppler 99.6 | RECOGNITION |  |
| G05 | refused | – / – | pdfium 82.0 | EXTRACTION (cannot open) | pdf-extract 0.12.1 panics (lib.rs:584 assert_eq 256 vs 255: font /Widths length vs FirstChar..LastChar) on every page; app reports library.badPdf |
| G07 | 115/601 (19.1%) | 94.4 / 94.4 | pdfium 94.4 | RECOGNITION |  |
| G09 | 0/764 (0.0%) | 90.4 / 97.6 | pdfium 90.6 | RECOGNITION |  |
| G10 | 0/793 (0.0%) | 39.6 / – | poppler 87.8 | EXTRACTION |  |
| G13 | 0/1288 (0.0%) | 48.6 / 97.9 | pdfium 98.1 | EXTRACTION (verse-number placement only) | words complete and in order (letters-only 97.9% vs best 98.1%); verse numbers emitted out of their inline position |

Reading of the table:

* **Recognition (16 of 22: hboWLC-pdf, latVUC-pdf, spavbl-rlofodt, grcsbl-rfpdf-mrk, hbo-rwpdfrtl-rut, engwebu-rl2col, deutkw-fpdfa5, kor-lopdf, F01, F03, F07, F13, G03, G04, G07, G09)**: G03 (0 verified of 31,011, text 99.1 % / 93.1 % in order), F07 (0 of 31,084; the
  app refused 1,187 chapters for "reading order" although the text is 97.8 % in order), F01, G04, G07, G09,
  deutkw-fpdfa5, kor-lopdf, grcsbl-rfpdf-mrk, hbo-rwpdfrtl-rut, latVUC-pdf, spavbl-rlofodt, F03, and — with the
  text layer limiting every engine to 63–84 % — F13, engwebu-rl2col and hboWLC-pdf. A different PDF engine will
  not raise these; the recogniser has to.
* **Extraction (6: engkjvcpb-rwpdfmargin, spabll-rrlab2col, heb-wprtl, G05, G10, G13)**: G05 (cannot open), heb-wprtl (1.9 % vs MuPDF 80.5 %), G10 (39.6 % vs Poppler 87.8 %),
  and the number-placement group G13, spabll-rrlab2col, engkjvcpb-rwpdfmargin. In the latter the PDF parser is
  not the culprit — PDFium's/MuPDF's raw stream order keeps numbers where they are printed (98.1 % on G13) —
  the app's line/role layout emits superscript or margin verse numbers as separate units before (or apart
  from) the line they belong to. Switching engine fixes this only if the layout code changes too.
* **Document (0)**: no low-verified document is unreadable for every engine.

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
pub struct PdfGlyph { pub text: String, pub x0: f32, pub y0: f32, pub x1: f32, pub y1: f32,
                      pub origin: (f32, f32), pub size: f32, pub font: Option<String>,
                      pub rotation_deg: f32, pub from_actual_text: bool }
pub struct PdfPageGlyphs { pub number: u32, pub label: Option<String>,
                           pub width: f32, pub height: f32, pub glyphs: Vec<PdfGlyph> }
pub struct PdfOutlineItem { pub title: String, pub page: Option<u32>, pub depth: u8 }
pub struct PdfDocGlyphs { pub metadata: FoundMetadata, pub pages: Vec<PdfPageGlyphs>,
                          pub failed_pages: Vec<u32>, pub outline: Vec<PdfOutlineItem>,
                          pub engine: &'static str, pub repaired: bool }
pub trait PdfGlyphSource {
    fn read(&self, bytes: &[u8], file_name: &str) -> AppResult<PdfDocGlyphs>; // badPdf / passwordProtected as today
}
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
