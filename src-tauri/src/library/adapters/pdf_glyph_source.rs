//! Phase A2.1 prototype: a PDF's glyphs from a source other than
//! `pdf_extract`, laid out by the app's own layout code (`pdf_layout`).
//!
//! `PdfGlyphSource` is the shared extraction interface proposed in A2
//! (`docs/audit/a2/pdf-engines.md` §10a), reduced to what the layout code
//! needs. A source reads a PDF and gives each page's glyphs, in the order
//! drawn, to a `GlyphSink`. The existing `Collector` is one, so pages are
//! laid out as they end and their glyphs let go, as with `pdf_extract`.
//!
//! [`PdfiumHelper`] runs PDFium in a separate process (`tools/pdfium-glyphs`),
//! never inside the app: a crash, a hang or runaway memory in PDFium ends
//! that process only. The current backend (`pdf::read_pages`) is unchanged
//! and nothing in the application calls this module; it is reached only
//! through `documents::read_with` from tests and harnesses.

use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::error::{AppError, AppResult, UserMessage};
use crate::library::adapters::pdf_layout::{GlyphSink, SourceGlyph};

/// What a glyph source reports about a document besides its glyphs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GlyphReport {
    /// Which engine read it ("pdfium").
    pub engine: &'static str,
    pub pages: u32,
    /// Pages that could not be read (1-based).
    pub failed_pages: Vec<u32>,
    /// The first page cut off at the character limit.
    pub truncated_from: Option<u32>,
}

/// A reader of a PDF's glyphs (Phase A2.1).
pub trait PdfGlyphSource {
    /// Reads `bytes`, giving every page and glyph to `sink` in page order.
    /// Refuses an unreadable or password-protected file with the same coded
    /// messages as `pdf::read_pages` (`library.badPdf`,
    /// `library.passwordProtected`).
    fn read(
        &self,
        bytes: &[u8],
        file_name: &str,
        sink: &mut dyn GlyphSink,
    ) -> AppResult<GlyphReport>;
}

/// PDFium, in a helper process of its own.
#[derive(Debug, Clone)]
pub struct PdfiumHelper {
    /// The helper (`aw-pdfium-glyphs`).
    pub exe: PathBuf,
    /// The directory holding the pinned PDFium library.
    pub library: PathBuf,
    /// Longest the helper may run before it is stopped.
    pub timeout: Duration,
    /// Address-space cap for the helper (Unix; a Job Object on Windows).
    pub max_memory_mb: u64,
    /// The character limit, as the current backend's (`pdf_layout`).
    pub max_chars: usize,
}

fn bad(file_name: &str) -> AppError {
    AppError::Import(
        UserMessage::new(
            "library.badPdf",
            format!("“{file_name}” could not be read as a PDF."),
        )
        .with("fileName", file_name),
    )
}

impl PdfGlyphSource for PdfiumHelper {
    fn read(
        &self,
        bytes: &[u8],
        file_name: &str,
        sink: &mut dyn GlyphSink,
    ) -> AppResult<GlyphReport> {
        // The helper reads a file: the bytes are written to a private
        // temporary one, removed afterwards.
        let input = TempFile::write(bytes).ok_or_else(|| bad(file_name))?;
        let mut child = Command::new(&self.exe)
            .arg(&self.library)
            .arg(&input.0)
            .arg("--max-chars")
            .arg(self.max_chars.to_string())
            .arg("--max-memory-mb")
            .arg(self.max_memory_mb.to_string())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| bad(file_name))?;
        let stdout = child.stdout.take().ok_or_else(|| bad(file_name))?;
        let mut stderr = child.stderr.take().ok_or_else(|| bad(file_name))?;
        let child = Arc::new(Mutex::new(child));
        let done = Arc::new(AtomicBool::new(false));
        let timed_out = Arc::new(AtomicBool::new(false));
        // The watchdog: stops the helper at the deadline.
        let watchdog = {
            let (child, done, timed_out) = (child.clone(), done.clone(), timed_out.clone());
            let deadline = Instant::now() + self.timeout;
            std::thread::spawn(move || {
                while !done.load(Ordering::Relaxed) {
                    if Instant::now() >= deadline {
                        timed_out.store(true, Ordering::Relaxed);
                        if let Ok(mut c) = child.lock() {
                            let _ = c.kill();
                        }
                        return;
                    }
                    std::thread::sleep(Duration::from_millis(20));
                }
            })
        };
        let errors = std::thread::spawn(move || {
            let mut text = String::new();
            let _ = stderr.read_to_string(&mut text);
            text
        });
        let mut report = GlyphReport {
            engine: "pdfium",
            ..Default::default()
        };
        let parsed = parse(BufReader::with_capacity(1 << 20, stdout), sink, &mut report);
        done.store(true, Ordering::Relaxed);
        let _ = watchdog.join();
        let status = child.lock().ok().and_then(|mut c| c.wait().ok());
        let message = errors.join().unwrap_or_default();
        if timed_out.load(Ordering::Relaxed) {
            log::warn!(
                "pdfium helper stopped after {:?} for {file_name}",
                self.timeout
            );
            return Err(bad(file_name));
        }
        match status.and_then(|s| s.code()) {
            Some(0) if parsed => Ok(report),
            Some(3) => Err(AppError::Import(
                UserMessage::new(
                    "library.passwordProtected",
                    format!(
                        "“{file_name}” is password-protected. Remove the protection and import it again."
                    ),
                )
                .with("fileName", file_name),
            )),
            other => {
                log::warn!(
                    "pdfium helper failed for {file_name} ({other:?}): {}",
                    message.trim()
                );
                Err(bad(file_name))
            }
        }
    }
}

/// A private temporary copy of the PDF for the helper, removed when dropped.
struct TempFile(PathBuf);

impl TempFile {
    fn write(bytes: &[u8]) -> Option<Self> {
        use std::sync::atomic::AtomicU64;
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let name = format!(
            "aw-pdf-glyphs-{}-{}.pdf",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let path = std::env::temp_dir().join(name);
        std::fs::write(&path, bytes).ok()?;
        Some(TempFile(path))
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Reads the helper's records into `sink`. False when the output is not a
/// whole, well-formed record stream (the helper stopped part-way).
fn parse(input: impl BufRead, sink: &mut dyn GlyphSink, report: &mut GlyphReport) -> bool {
    let mut open = false;
    let mut header = false;
    let mut ended = false;
    for line in input.lines() {
        let Ok(line) = line else {
            return false;
        };
        let mut f = line.split('\t');
        match f.next() {
            Some("AWG1") => header = true,
            Some("D") => {
                report.pages = f.next().and_then(|v| v.parse().ok()).unwrap_or(0);
            }
            Some("P") => {
                if open {
                    sink.end_page();
                }
                let number = f.next().and_then(|v| v.parse().ok()).unwrap_or(0);
                let width = f.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
                let height = f.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
                sink.begin_page(number, width, height);
                open = true;
            }
            Some("G") if open => {
                let mut n = || f.next().and_then(|v| v.parse::<f32>().ok());
                let (Some(x), Some(y), Some(end), Some(size), Some(width)) =
                    (n(), n(), n(), n(), n())
                else {
                    continue;
                };
                let flags: u8 = f.next().and_then(|v| v.parse().ok()).unwrap_or(0);
                let text: String = f
                    .next()
                    .unwrap_or("")
                    .split('.')
                    .filter_map(|h| u32::from_str_radix(h, 16).ok())
                    .filter_map(char::from_u32)
                    .collect();
                if text.is_empty() {
                    continue;
                }
                sink.glyph(SourceGlyph {
                    text,
                    x,
                    y,
                    end,
                    size,
                    width: width.clamp(0.0, 60000.0) as u16,
                    upright: flags & 1 != 0,
                    slanted: flags & 2 != 0,
                });
            }
            Some("F") => {
                if let Some(n) = f.next().and_then(|v| v.parse().ok()) {
                    report.failed_pages.push(n);
                }
            }
            Some("T") => {
                report.truncated_from = f.next().and_then(|v| v.parse().ok());
            }
            Some("E") => ended = true,
            _ => {}
        }
    }
    if open {
        sink.end_page();
    }
    header && ended
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Pages(Vec<(u32, Vec<SourceGlyph>)>);
    impl GlyphSink for Pages {
        fn begin_page(&mut self, number: u32, _w: f64, _h: f64) {
            self.0.push((number, Vec::new()));
        }
        fn glyph(&mut self, g: SourceGlyph) {
            self.0.last_mut().unwrap().1.push(g);
        }
        fn end_page(&mut self) {}
    }

    #[test]
    fn the_record_stream_is_read_page_by_page_and_must_end() {
        let stream = "AWG1\nD\t2\nP\t1\t595\t842\nG\t72\t100\t78.5\t12\t542\t1\t48\n\
                      G\t78.5\t100\t84\t12\t458\t1\t5d0.5b8\nP\t2\t595\t842\nF\t2\tbad page\nE\n";
        let mut pages = Pages::default();
        let mut report = GlyphReport::default();
        assert!(parse(stream.as_bytes(), &mut pages, &mut report));
        assert_eq!(report.pages, 2);
        assert_eq!(report.failed_pages, vec![2]);
        assert_eq!(pages.0.len(), 2);
        assert_eq!(pages.0[0].1[0].text, "H");
        // A Hebrew letter with its point, as one glyph's text.
        assert_eq!(pages.0[0].1[1].text, "\u{5d0}\u{5b8}");
        assert!(pages.0[0].1[0].upright);
        let mut cut = Pages::default();
        assert!(!parse(
            "AWG1\nD\t1\nP\t1\t1\t1\n".as_bytes(),
            &mut cut,
            &mut GlyphReport::default()
        ));
    }

    /// A stand-in helper: a shell script (Unix), for the parent's handling
    /// of a helper that hangs, dies part-way or refuses a file.
    #[cfg(unix)]
    fn script(name: &str, body: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let path = std::env::temp_dir().join(format!("aw-glyphs-{}-{name}.sh", std::process::id()));
        std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    #[cfg(unix)]
    fn helper(exe: PathBuf, timeout: Duration) -> PdfiumHelper {
        PdfiumHelper {
            exe,
            library: PathBuf::from("/nonexistent"),
            timeout,
            max_memory_mb: 64,
            max_chars: 1000,
        }
    }

    #[cfg(unix)]
    fn code(result: AppResult<GlyphReport>) -> Option<&'static str> {
        match result {
            Err(AppError::Import(m)) => m.code,
            other => panic!("expected a refusal, got {other:?}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn a_helper_that_hangs_is_stopped_at_the_deadline() {
        let exe = script("hang", "printf 'AWG1\\nD\\t1\\n'; exec sleep 30");
        let started = Instant::now();
        let result = helper(exe, Duration::from_millis(300)).read(
            b"%PDF-1.4",
            "slow.pdf",
            &mut Pages::default(),
        );
        assert_eq!(code(result), Some("library.badPdf"));
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "{:?}",
            started.elapsed()
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_helper_that_dies_part_way_gives_no_document() {
        let exe = script(
            "crash",
            "printf 'AWG1\\nD\\t2\\nP\\t1\\t595\\t842\\nG\\t72\\t100\\t78\\t12\\t500\\t1\\t48\\n'; kill -SEGV $$",
        );
        let result = helper(exe, Duration::from_secs(20)).read(
            b"%PDF-1.4",
            "broken.pdf",
            &mut Pages::default(),
        );
        assert_eq!(code(result), Some("library.badPdf"));
    }

    #[cfg(unix)]
    #[test]
    fn a_password_protected_file_is_refused_with_its_own_message() {
        let exe = script("password", "echo password-protected >&2; exit 3");
        let result = helper(exe, Duration::from_secs(20)).read(
            b"%PDF-1.4",
            "locked.pdf",
            &mut Pages::default(),
        );
        assert_eq!(code(result), Some("library.passwordProtected"));
    }
}
