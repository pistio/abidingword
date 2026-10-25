//! PDF: the text layer, page by page. Page numbers are the physical page
//! order (1 = first page). No OCR: a PDF without a usable text layer is
//! reported as needing OCR, never "imported" empty.

use std::panic::{catch_unwind, AssertUnwindSafe};

use lopdf::{decode_text_string, Document, Object};

use super::pdf_layout::{Collector, PageLayout, PageOrder};

use super::{
    paragraphs, split_long, ExtractionStatus, FoundMetadata, ImportedDocument, NewSegment,
    SourceImportAdapter,
};
use crate::error::{AppError, AppResult, UserMessage};

pub struct PdfImporter;

const MAX_SEGMENT: usize = 1_500;
/// Below this many letters per page on average, the PDF is treated as
/// scanned images without a text layer.
const MIN_LETTERS_PER_PAGE: usize = 25;

impl SourceImportAdapter for PdfImporter {
    fn format(&self) -> &'static str {
        "pdf"
    }

    fn accepts(&self, extension: &str) -> bool {
        extension == "pdf"
    }

    fn extract(&self, bytes: &[u8], file_name: &str) -> AppResult<ImportedDocument> {
        let PdfText {
            metadata,
            pages,
            scanned,
            ..
        } = read_pages(bytes, file_name)?;
        let page_count = pages.len() as u32;
        if scanned {
            return Ok(ImportedDocument {
                format: "pdf",
                metadata,
                segments: Vec::new(),
                extraction_status: ExtractionStatus::OcrRequired,
                page_count: Some(page_count),
                chapter_count: None,
            });
        }
        let segments = page_segments(&pages);
        Ok(ImportedDocument {
            format: "pdf",
            metadata,
            segments,
            extraction_status: ExtractionStatus::ExtractedText,
            page_count: Some(page_count),
            chapter_count: None,
        })
    }
}

/// A PDF's text layer, page by page, in reading order (lines kept; v0.24.1:
/// put in order from where the text is drawn, `pdf_layout`), and whether it is
/// too thin to be anything but scanned images.
pub struct PdfText {
    pub metadata: FoundMetadata,
    pub pages: Vec<String>,
    /// How each page's reading order was established (same length as `pages`).
    pub orders: Vec<PageOrder>,
    /// Each page laid out: its lines with their positions.
    pub layouts: Vec<PageLayout>,
    /// Pages whose text could not be read, or only in part (past the
    /// character limit); reported in a Bible import's review.
    pub failed_pages: Vec<u32>,
    pub scanned: bool,
}

/// Reads a PDF's metadata and text layer. Refuses an unreadable or
/// password-protected file with a coded message; never panics.
pub fn read_pages(bytes: &[u8], file_name: &str) -> AppResult<PdfText> {
    let unreadable = || {
        AppError::Import(
            UserMessage::new(
                "library.badPdf",
                format!("“{file_name}” could not be read as a PDF."),
            )
            .with("fileName", file_name),
        )
    };
    let document = Document::load_mem(bytes).map_err(|_| unreadable())?;
    if document.is_encrypted() {
        return Err(AppError::Import(
            UserMessage::new(
                "library.passwordProtected",
                format!(
                    "“{file_name}” is password-protected. Remove the protection and import it again."
                ),
            )
            .with("fileName", file_name),
        ));
    }
    let metadata = metadata(&document);
    let (layouts, failed_pages) = read_layouts(&document, file_name).ok_or_else(unreadable)?;
    let pages: Vec<String> = layouts.iter().map(PageLayout::text).collect();
    let orders = layouts.iter().map(|l| l.order).collect();
    let letters: usize = pages
        .iter()
        .map(|p| p.chars().filter(|c| c.is_alphanumeric()).count())
        .sum();
    let scanned = pages.is_empty() || letters < MIN_LETTERS_PER_PAGE * pages.len();
    Ok(PdfText {
        metadata,
        pages,
        orders,
        layouts,
        failed_pages,
        scanned,
    })
}

/// Every page laid out in reading order (`pdf_layout`), in page order. The
/// text extractor can panic or fail on unusual files; that must never take
/// the app down: the whole document is read at once, and if that fails each
/// page is read on its own, so one bad page leaves only that page empty
/// (listed in the second value). None when no page can be read.
fn read_layouts(document: &Document, file_name: &str) -> Option<(Vec<PageLayout>, Vec<u32>)> {
    let numbers: Vec<u32> = document.get_pages().keys().copied().collect();
    if numbers.is_empty() {
        return None;
    }
    let whole = catch_unwind(AssertUnwindSafe(|| {
        let mut collector = Collector::default();
        pdf_extract::output_doc(document, &mut collector).map(|_| collector)
    }));
    if let Ok(Ok(collector)) = whole {
        // Pages cut off at the character limit are listed as not read.
        let truncated = collector.truncated_from;
        let layouts = collector.layouts();
        if layouts.len() == numbers.len() {
            let failed: Vec<u32> = match truncated {
                Some(from) => {
                    log::warn!("pdf text of {file_name} cut off from page {from}");
                    (from..=layouts.len() as u32).collect()
                }
                None => Vec::new(),
            };
            return Some((layouts, failed));
        }
    }
    log::warn!("pdf text extraction failed for {file_name}; reading page by page");
    let mut layouts = Vec::with_capacity(numbers.len());
    let mut failed = Vec::new();
    for number in numbers {
        let page = catch_unwind(AssertUnwindSafe(|| {
            let mut collector = Collector::default();
            pdf_extract::output_doc_page(document, &mut collector, number).map(|_| collector)
        }));
        match page {
            Ok(Ok(collector)) => match collector.layouts().pop() {
                Some(mut layout) => {
                    layout.number = layouts.len() as u32 + 1;
                    layouts.push(layout);
                }
                None => {
                    failed.push(number);
                    layouts.push(PageLayout::empty(layouts.len() as u32 + 1));
                }
            },
            _ => {
                failed.push(number);
                layouts.push(PageLayout::empty(layouts.len() as u32 + 1));
            }
        }
    }
    (failed.len() < layouts.len()).then_some((layouts, failed))
}

/// The segments of a PDF's pages: paragraphs, with their page and their
/// place in the page's text.
pub fn page_segments(pages: &[String]) -> Vec<NewSegment> {
    let mut segments = Vec::new();
    for (index, page) in pages.iter().enumerate() {
        let number = index as u32 + 1;
        let mut paragraph = 0u32;
        for (text, start, end) in paragraphs(page) {
            for part in split_long(&text, MAX_SEGMENT) {
                segments.push(NewSegment {
                    text: part,
                    page_number: Some(number),
                    paragraph_index: Some(paragraph),
                    char_start: Some(start as u32),
                    char_end: Some(end as u32),
                    ..Default::default()
                });
                paragraph += 1;
            }
        }
    }
    segments
}

fn metadata(document: &Document) -> FoundMetadata {
    let info = document
        .trailer
        .get(b"Info")
        .ok()
        .and_then(|info| match info {
            Object::Reference(id) => document.get_dictionary(*id).ok(),
            Object::Dictionary(d) => Some(d),
            _ => None,
        });
    let field = |key: &[u8]| -> Option<String> {
        info.and_then(|d| d.get(key).ok())
            .and_then(|v| decode_text_string(v).ok())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    };
    let language = document
        .catalog()
        .ok()
        .and_then(|c| c.get(b"Lang").ok())
        .and_then(|v| decode_text_string(v).ok())
        .filter(|s| !s.is_empty());
    FoundMetadata {
        title: field(b"Title"),
        author: field(b"Author"),
        language,
        ..Default::default()
    }
}
