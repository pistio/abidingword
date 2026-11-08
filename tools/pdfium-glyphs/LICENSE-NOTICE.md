# Licences of the A2.1 PDFium prototype

This helper is a prototype for the owner's review. It is not distributed. If
it were shipped with the Windows app, these notices would have to ship with
it (an "About / licences" page and the installed files).

| Component | Licence | What shipping requires |
|---|---|---|
| `aw-pdfium-glyphs` (this helper) | the application's own terms | — |
| `pdfium-render` 0.9.5 (Rust binding) | MIT OR Apache-2.0 | copyright and licence notice |
| `libloading`, `bitflags`, `once_cell`, `log`, … (its Rust dependencies, `Cargo.lock`) | MIT and/or Apache-2.0 | notices |
| PDFium 159.0.8138.0 | BSD-3-Clause ("The PDFium Authors") | notice; no endorsement |
| pdfium-binaries packaging (Benoit Blanchon) | MIT | notice |
| Bundled in the PDFium build: abseil (Apache-2.0), agg 2.3, dragonbox (Apache-2.0 with LLVM exception / Boost), fast_float, FreeType (FTL), HarfBuzz (Old MIT), highway, ICU (Unicode v3), Little CMS (MIT), libjpeg-turbo (IJG / BSD), OpenJPEG (BSD-2), libpng, llvm-libc, simdutf, zlib | permissive | every notice in the archive's `licenses/` folder (18 files); FreeType's FTL also asks for a credit line in the documentation ("Portions of this software are copyright © The FreeType Project (www.freetype.org)") |

None of these is copyleft. MuPDF (AGPL) and Poppler (GPL) were deliberately
not used (A2, `docs/audit/a2/pdf-engines.md` §10).
