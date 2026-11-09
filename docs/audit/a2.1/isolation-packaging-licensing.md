# A2.1: isolation, Windows packaging, licences, offline operation

## Isolation: implemented and tested

PDFium never runs inside the app. `PdfiumHelper`
(`src-tauri/src/library/adapters/pdf_glyph_source.rs`) works like this:

1. It writes the document's bytes to a private temporary file, removed when
   reading ends.
2. It starts `aw-pdfium-glyphs` with the pinned library's directory, that
   file, a character limit (the current backend's, 40 000 000) and a memory
   cap.
3. It reads the helper's record stream as it comes, page by page, into the
   layout code's `GlyphSink`. A page is laid out when it ends and its
   glyphs are let go.
4. A watchdog thread stops the helper at the deadline. Unless the stream
   ended with `E` and the helper exited with 0, the document is refused
   with the existing coded messages (`library.badPdf`,
   `library.passwordProtected`).

| Failure | How it is contained | Evidence |
|---|---|---|
| Hang | Watchdog kills the helper at the deadline; the reading is refused. | Unit test `a_helper_that_hangs_is_stopped_at_the_deadline` (stopped at 300 ms). |
| Crash part-way | The stream does not end with `E`; nothing partial is used. | Unit test `a_helper_that_dies_part_way_gives_no_document` (SIGSEGV after one glyph). |
| Runaway memory | The helper caps its own address space (`RLIMIT_AS`) before loading PDFium; allocation past it aborts the helper only. | G05 read with a 16 MB cap: aborted (exit 134), refused. With a 64 MB cap it reads normally. Whole kjv-pdf (1 561 pages): helper peak 40 MB. |
| Password | Exit code 3 → `library.passwordProtected`. | Unit test. |
| Unreadable | Exit code 2 → `library.badPdf`. | — |

**Not implemented: the memory cap on Windows.** There the cap needs a Job
Object (`CreateJobObject`, `JOB_OBJECT_LIMIT_PROCESS_MEMORY`, assigned
before the helper's main thread runs). That is the parent's job, through
the `windows` crate. It was not written: it cannot be tested here, and it
touches the app's dependency graph. Until then a Windows helper runs
without a memory cap; the watchdog and the stream checks still apply.

## Windows packaging: cross-built, not run

* The helper cross-builds for `x86_64-pc-windows-gnu` (MinGW, 23 s): a
  714 KB console executable importing only system DLLs (`kernel32`,
  `ntdll`, `msvcrt`, `bcryptprimitives`, `api-ms-win-core-synch`). It loads
  `pdfium.dll` (7.8 MB, SHA-256 `71a3e63e…`, pinned in `PDFIUM.lock`) at
  run time from a directory it is given.
* For the installer, three files would ship beside the app:
  * `aw-pdfium-glyphs.exe`, built with the MSVC toolchain in CI like the
    app;
  * `pdfium.dll`, checked against `PDFIUM.lock`;
  * the licence notices.
  Both executables need code signing like the app. A console helper must
  start without a window (`CREATE_NO_WINDOW`), which `PdfiumHelper` does
  not set yet.
* **Not run on Windows**: this session has no Windows machine or emulator.
  The Windows build is evidence that it compiles and links, nothing more.
  Windows acceptance is the owner's (or Codex's), on the production
  profile, after review.

## Licences

All permissive, none copyleft (`tools/pdfium-glyphs/LICENSE-NOTICE.md`):

* PDFium: BSD-3-Clause;
* pdfium-binaries packaging: MIT;
* pdfium-render and its Rust dependencies: MIT and/or Apache-2.0;
* the 18 third-party notices in the PDFium archive: abseil, FreeType (with
  its credit line), HarfBuzz, ICU, Little CMS, libjpeg-turbo, OpenJPEG,
  libpng, zlib and others.

Shipping requires copying those notices into the installed files and an
"About / licences" page. MuPDF (AGPL or commercial) and Poppler (GPL) were
not used. PDFium's CVE history and current security advisories could not be
checked from this environment; the owner should check them before any
decision to ship.

## Offline operation

The helper opens no network connection and sends no telemetry. It reads
only the file it is given. PDFium is a prebuilt binary pinned by SHA-256;
nothing is fetched at run time. Building the helper fetches its crates from
crates.io once, as any Rust build does; `--locked` builds from
`tools/pdfium-glyphs/Cargo.lock`. The app's `src-tauri/Cargo.lock` is
unchanged.
