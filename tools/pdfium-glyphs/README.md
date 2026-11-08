# aw-pdfium-glyphs (Phase A2.1 prototype)

Reads a PDF's characters with PDFium **in a process of its own** and writes
them as glyph records for Abiding Word's existing layout code
(`src-tauri/src/library/adapters/pdf_layout.rs`, through
`pdf_glyph_source::PdfiumHelper`).

* It is not part of the application build: it has its own `Cargo.toml`,
  workspace and `Cargo.lock`, and `src-tauri/Cargo.lock` is unchanged.
* It binds at run time to the pinned PDFium build in `PDFIUM.lock`. Verify
  the archive's SHA-256 before use.
* It works offline: no network access, no telemetry, nothing but the file it
  is given.
* The application's current PDF backend (`pdf::read_pages`, pdf-extract +
  lopdf) is unchanged and is what the app uses.

```bash
cargo build --release --locked
target/release/aw-pdfium-glyphs <dir with libpdfium.so or pdfium.dll> file.pdf --max-chars 40000000 --max-memory-mb 4096
```

How it measures a glyph (origin, the font's own advance, size, ligatures)
is described at the top of `src/main.rs`.

Exit codes: 0 read; 2 not a readable PDF; 3 password-protected; 4 PDFium
could not be loaded; 5 bad arguments. The output format is described at the
top of `src/main.rs`. Licences: `LICENSE-NOTICE.md`.
