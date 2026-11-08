//! Phase A2.1 prototype: a PDF's characters, read by PDFium in a process of
//! their own, written as glyph records for Abiding Word's own layout code
//! (`src-tauri/src/library/adapters/pdf_layout.rs`).
//!
//! It does no layout, ordering or recognition: each character PDFium reads
//! from the page's content is written once, in PDFium's character order,
//! with the measures the layout code already works from (`Glyph` there).
//! Characters PDFium generates itself (spaces and line ends it infers) are
//! left out: the layout code decides where words and lines are. A ligature
//! PDFium expands ("ﬁ" read as "f", "i", at one origin) is written as one
//! glyph, as the current backend gives it.
//!
//! A glyph's advance is the font's own width for the character
//! (`FPDFFont_GetGlyphWidth`), as the current backend takes it from the
//! font's widths: the layout code places word spaces by the gaps between
//! advances. PDFium's loose box follows the glyph's outline and runs past
//! the advance of overhanging letters ("f"), which closed real word gaps
//! ("of the" read as "ofthe"), so it is used only where the font gives no
//! width. pdfium-render's public raw bindings are used for this, because
//! its wrappers do not give a character's font.
//!
//! ```text
//! aw-pdfium-glyphs <libpdfium directory> <file.pdf> [--max-chars N] [--max-memory-mb N]
//! ```
//!
//! Output (UTF-8, one record a line, tab-separated):
//!
//! ```text
//! AWG1                                  format version
//! D  <pages>                            the document opened
//! P  <n> <width> <height>               page n (1-based), its media box size
//! G  <x> <y> <end> <size> <w> <f> <hex> a glyph (see `glyph`)
//! F  <n> <reason>                       page n could not be read
//! T  <n>                                cut off at --max-chars from page n
//! E                                     end
//! ```
//!
//! Exit codes: 0 read; 2 not a readable PDF; 3 password-protected; 4 the
//! PDFium library could not be loaded; 5 bad arguments.

use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use pdfium_render::prelude::*;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let (Some(library), Some(file)) = (args.next(), args.next()) else {
        eprintln!("usage: aw-pdfium-glyphs <libpdfium directory> <file.pdf> [--max-chars N] [--max-memory-mb N]");
        return ExitCode::from(5);
    };
    let mut max_chars = usize::MAX;
    let mut max_memory_mb: Option<u64> = None;
    while let Some(flag) = args.next() {
        let value = args.next().and_then(|v| v.parse::<u64>().ok());
        match (flag.as_str(), value) {
            ("--max-chars", Some(v)) => max_chars = v as usize,
            ("--max-memory-mb", Some(v)) => max_memory_mb = Some(v),
            _ => {
                eprintln!("unknown or incomplete argument {flag}");
                return ExitCode::from(5);
            }
        }
    }
    if let Some(mb) = max_memory_mb {
        limit_memory(mb);
    }
    let bytes = match std::fs::read(&file) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("cannot read {file}: {e}");
            return ExitCode::from(2);
        }
    };
    let path = Pdfium::pdfium_platform_library_name_at_path(&PathBuf::from(&library));
    let bindings = match Pdfium::bind_to_library(&path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("cannot load PDFium from {}: {e}", path.display());
            return ExitCode::from(4);
        }
    };
    // The raw interface: a character's text object and font are needed for
    // the font's own advance, which pdfium-render's wrappers do not give.
    // SAFETY: every handle below comes from PDFium, is used only while its
    // document, page or text page is open, and is closed once.
    unsafe { read(&*bindings, &bytes, max_chars) }
}

const FPDF_ERR_PASSWORD: u64 = 4;

unsafe fn read(pdf: &dyn PdfiumLibraryBindings, bytes: &[u8], max_chars: usize) -> ExitCode {
    pdf.FPDF_InitLibrary();
    let document = pdf.FPDF_LoadMemDocument64(bytes, None);
    if document.is_null() {
        // `c_ulong` is 32 bits on Windows, 64 elsewhere.
        #[allow(clippy::unnecessary_cast)]
        let code = pdf.FPDF_GetLastError() as u64;
        pdf.FPDF_DestroyLibrary();
        if code == FPDF_ERR_PASSWORD {
            eprintln!("password-protected");
            return ExitCode::from(3);
        }
        eprintln!("not readable (PDFium error {code})");
        return ExitCode::from(2);
    }
    let stdout = std::io::stdout();
    let mut out = BufWriter::with_capacity(1 << 20, stdout.lock());
    let count = pdf.FPDF_GetPageCount(document).max(0);
    let _ = writeln!(out, "AWG1");
    let _ = writeln!(out, "D\t{count}");
    let mut chars = 0usize;
    let mut truncated = false;
    for index in 0..count {
        let number = index as u32 + 1;
        let page = pdf.FPDF_LoadPage(document, index);
        if page.is_null() {
            let _ = writeln!(out, "F\t{number}\tpage not readable");
            continue;
        }
        let (mut l, mut b, mut r, mut t) = (0f32, 0f32, 0f32, 0f32);
        let (left, top, width, height) =
            if pdf.is_true(pdf.FPDFPage_GetMediaBox(page, &mut l, &mut b, &mut r, &mut t)) {
                (l.min(r), t.max(b), (r - l).abs(), (t - b).abs())
            } else {
                let (w, h) = (pdf.FPDF_GetPageWidthF(page), pdf.FPDF_GetPageHeightF(page));
                (0.0, h, w, h)
            };
        let _ = writeln!(out, "P\t{number}\t{width}\t{height}");
        if truncated {
            pdf.FPDF_ClosePage(page);
            continue;
        }
        let text = pdf.FPDFText_LoadPage(page);
        if text.is_null() {
            let _ = writeln!(out, "F\t{number}\ttext not readable");
            pdf.FPDF_ClosePage(page);
            continue;
        }
        let n = pdf.FPDFText_CountChars(text).max(0);
        let mut pending: Option<Glyph> = None;
        let mut k = 0;
        while k < n {
            let i = k;
            k += 1;
            if pdf.FPDFText_IsGenerated(text, i) == 1 {
                continue;
            }
            let mut code = pdf.FPDFText_GetUnicode(text, i);
            // A pair of UTF-16 surrogates (where wchar_t is 16 bits).
            if (0xD800..0xDC00).contains(&code) && k < n {
                let low = pdf.FPDFText_GetUnicode(text, k);
                if (0xDC00..0xE000).contains(&low) {
                    code = 0x10000 + ((code - 0xD800) << 10) + (low - 0xDC00);
                    k += 1;
                }
            }
            let Some(c) = char::from_u32(code) else {
                continue;
            };
            if c == '\0' || c == '\u{fffe}' || c == '\u{ffff}' {
                continue;
            }
            if chars >= max_chars {
                let _ = writeln!(out, "T\t{number}");
                truncated = true;
                break;
            }
            chars += 1;
            if let Some(g) = glyph(pdf, text, i, c, left, top) {
                match pending.as_mut() {
                    // A ligature PDFium has expanded ("ﬁ" as "f", "i"): its
                    // letters share one origin. They are one glyph, whose
                    // advance is theirs together.
                    Some(p) if p.x == g.x && p.y == g.y => p.join(pdf, &g),
                    _ => {
                        if let Some(p) = pending.replace(g) {
                            let _ = out.write_all(p.record().as_bytes());
                        }
                    }
                }
            }
        }
        if let Some(p) = pending.take() {
            let _ = out.write_all(p.record().as_bytes());
        }
        pdf.FPDFText_ClosePage(text);
        pdf.FPDF_ClosePage(page);
    }
    let _ = writeln!(out, "E");
    let _ = out.flush();
    pdf.FPDF_CloseDocument(document);
    pdf.FPDF_DestroyLibrary();
    ExitCode::SUCCESS
}

/// One glyph as `pdf_layout::Glyph` holds it:
///
/// * `x`, `y`: its origin, from the media box's left edge and down from
///   its top edge (the baseline);
/// * `end`: where its advance ends: the font's own width for the character
///   (`FPDFFont_GetGlyphWidth`) at its size, scaled by the character's
///   matrix, as the current backend takes it from the font's widths (the
///   right edge of PDFium's loose box, which follows the glyph's outline,
///   only where the font gives no width);
/// * `size`: the font size times the matrix's scale;
/// * `w`: the font's width for the character in thousandths of the size;
/// * `f`: 1 upright, 2 slanted (a sheared upright face), as the existing
///   collector decides them from the text matrix;
/// * the text as hexadecimal code points joined by `.`.
struct Glyph {
    text: String,
    x: f32,
    y: f32,
    end: f32,
    size: f32,
    w: u32,
    flags: u8,
    /// The right edge of PDFium's loose box (the glyph's outline and
    /// advance), from the media box's left edge.
    loose_right: f32,
}

/// The font's own width for a character, in thousandths of an em.
unsafe fn font_width(pdf: &dyn PdfiumLibraryBindings, face: FPDF_FONT, c: char) -> f32 {
    let mut thousandths = 0f32;
    if !face.is_null() {
        let _ = pdf.FPDFFont_GetGlyphWidth(face, c as u32, 1000.0, &mut thousandths);
    }
    if thousandths.is_finite() && thousandths > 0.0 {
        thousandths
    } else {
        0.0
    }
}

impl Glyph {
    /// Joins the next expanded letter of a ligature (its advance: the
    /// letters' together, never past PDFium's box for the ligature).
    unsafe fn join(&mut self, _pdf: &dyn PdfiumLibraryBindings, next: &Glyph) {
        self.text.push_str(&next.text);
        // The letters' advances together are a little wider than the
        // ligature's own ("ﬀ" is narrower than "f" + "f"); PDFium's box for
        // the expanded letters is the ligature glyph's, so the nearer edge
        // is taken.
        let together = self.end + (next.end - next.x).max(0.0);
        let boxed = self.loose_right.max(next.loose_right);
        self.end = if boxed > self.x {
            together.min(boxed)
        } else {
            together
        };
        self.w = if self.size > 0.0 {
            ((((self.end - self.x) / self.size) * 1000.0).round()).clamp(0.0, 60000.0) as u32
        } else {
            (self.w + next.w).min(60000)
        };
    }

    fn record(&self) -> String {
        let hex: Vec<String> = self
            .text
            .chars()
            .map(|c| format!("{:x}", c as u32))
            .collect();
        format!(
            "G\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            self.x,
            self.y,
            self.end,
            self.size,
            self.w,
            self.flags,
            hex.join(".")
        )
    }
}

unsafe fn glyph(
    pdf: &dyn PdfiumLibraryBindings,
    text: FPDF_TEXTPAGE,
    i: i32,
    c: char,
    left: f32,
    top: f32,
) -> Option<Glyph> {
    let (mut ox, mut oy) = (0f64, 0f64);
    if !pdf.is_true(pdf.FPDFText_GetCharOrigin(text, i, &mut ox, &mut oy)) {
        return None;
    }
    let mut m = FS_MATRIX {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };
    let _ = pdf.FPDFText_GetMatrix(text, i, &mut m);
    let (a, b, cc, d) = (m.a, m.b, m.c, m.d);
    let font = pdf.FPDFText_GetFontSize(text, i) as f32;
    let slanted = cc.abs() > 1e-3 && cc.abs() <= 0.5 * d.abs();
    let upright = b.abs() <= 1e-3 && (cc.abs() <= 1e-3 || slanted) && a > 0.0 && d > 0.0;
    let scale = if upright {
        (a * d).abs().sqrt()
    } else {
        (a * d - b * cc).abs().sqrt()
    };
    let size = font * scale;
    let size = if size.is_finite() && size > 0.0 {
        size
    } else {
        0.0
    };
    let x = ox as f32 - left;
    let y = top - oy as f32;
    // The font's own width for the character, in thousandths of an em.
    let object = pdf.FPDFText_GetTextObject(text, i);
    let face = if object.is_null() {
        std::ptr::null_mut()
    } else {
        pdf.FPDFTextObj_GetFont(object)
    };
    let thousandths = font_width(pdf, face, c);
    let mut r = FS_RECTF {
        left: 0.0,
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
    };
    let loose_right =
        if pdf.is_true(pdf.FPDFText_GetLooseCharBox(text, i, &mut r)) && r.right.is_finite() {
            (r.right - left).max(x)
        } else {
            x
        };
    let end = if thousandths.is_finite() && thousandths > 0.0 {
        x + thousandths / 1000.0 * font * a
    } else {
        loose_right
    };
    let w = if thousandths.is_finite() && thousandths > 0.0 {
        thousandths.round().clamp(0.0, 60000.0) as u32
    } else if size > 0.0 {
        (((end - x) / size) * 1000.0).round().clamp(0.0, 60000.0) as u32
    } else {
        0
    };
    let flags = u8::from(upright && size > 0.0 && x.is_finite() && y.is_finite())
        | (u8::from(slanted) << 1);
    Some(Glyph {
        text: c.to_string(),
        x,
        y,
        end,
        size,
        w,
        flags,
        loose_right,
    })
}

/// Caps the address space of this process: a damaged or hostile file
/// cannot take the machine's memory. (Unix; on Windows the parent puts the
/// process in a Job Object with a memory limit instead.)
#[cfg(unix)]
fn limit_memory(mb: u64) {
    let bytes = mb.saturating_mul(1024 * 1024);
    let limit = libc::rlimit {
        rlim_cur: bytes as libc::rlim_t,
        rlim_max: bytes as libc::rlim_t,
    };
    // SAFETY: setrlimit with a valid struct; failure leaves the limit unset.
    unsafe {
        libc::setrlimit(libc::RLIMIT_AS, &limit);
    }
}

#[cfg(not(unix))]
fn limit_memory(_mb: u64) {}
