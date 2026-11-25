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
        Ok(document_from_text(&read_pages(bytes, file_name)?))
    }
}

/// Library normalisation only; the shared layout/Scripture extractor is unchanged.
pub fn document_from_text(text: &PdfText) -> ImportedDocument {
    ImportedDocument {
        format: "pdf",
        metadata: text.metadata.clone(),
        segments: if text.scanned {
            Vec::new()
        } else {
            page_segments(&text.pages)
        },
        extraction_status: if text.scanned {
            ExtractionStatus::OcrRequired
        } else {
            ExtractionStatus::ExtractedText
        },
        page_count: Some(text.pages.len() as u32),
        chapter_count: None,
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

/// Explicit Source rebuild only: page boundaries provide progress and safe
/// cancellation without changing the import/Scripture recognition pipeline.
pub fn read_pages_with_progress(
    bytes: &[u8],
    mut progress: impl FnMut(usize, usize) -> AppResult<()>,
) -> AppResult<PdfText> {
    let invalid = || {
        AppError::Import(UserMessage::new(
            "library.badPdf",
            "This PDF could not be read. The original and saved metadata are unchanged.",
        ))
    };
    let document = Document::load_mem(bytes).map_err(|_| invalid())?;
    if document.is_encrypted() {
        return Err(AppError::Import(UserMessage::new(
            "library.passwordProtected",
            "Remove PDF password protection before analysing headings.",
        )));
    }
    let numbers: Vec<_> = document.get_pages().into_keys().collect();
    if numbers.is_empty() || numbers.len() > 3000 {
        return Err(AppError::InvalidInput(UserMessage::new(
            "library.structure.limit",
            "Heading analysis supports PDFs with 1–3000 pages.",
        )));
    }
    let mut layouts = Vec::with_capacity(numbers.len());
    let mut failed_pages = Vec::new();
    let mut chars = 0;
    for (index, number) in numbers.iter().enumerate() {
        progress(index, numbers.len())?;
        let result = catch_unwind(AssertUnwindSafe(|| {
            let mut collector = Collector::default();
            pdf_extract::output_doc_page(&document, &mut collector, *number).map(|_| collector)
        }));
        let layout = match result {
            Ok(Ok(collector)) if collector.truncated_from.is_none() => collector.layouts().pop(),
            _ => None,
        };
        let mut layout = layout.unwrap_or_else(|| {
            failed_pages.push(index as u32 + 1);
            PageLayout::empty(index as u32 + 1)
        });
        layout.number = index as u32 + 1;
        chars += layout.lines.iter().map(|l| l.text.len()).sum::<usize>();
        if chars > crate::library::pdf_structure::MAX_CHARS {
            return Err(AppError::InvalidInput(UserMessage::new(
                "library.structure.limit",
                "The PDF text exceeds the bounded heading-analysis limit.",
            )));
        }
        layouts.push(layout);
    }
    progress(numbers.len(), numbers.len())?;
    if failed_pages.len() == layouts.len() {
        return Err(invalid());
    }
    let pages: Vec<_> = layouts.iter().map(PageLayout::text).collect();
    let letters: usize = pages
        .iter()
        .map(|p| p.chars().filter(|c| c.is_alphanumeric()).count())
        .sum();
    Ok(PdfText {
        metadata: metadata(&document),
        scanned: letters < MIN_LETTERS_PER_PAGE * pages.len(),
        orders: layouts.iter().map(|l| l.order).collect(),
        layouts,
        pages,
        failed_pages,
    })
}

/// Reads a PDF's metadata and text layer. Refuses an unreadable or
/// password-protected file with a coded message; never panics.
pub fn read_pages(bytes: &[u8], file_name: &str) -> AppResult<PdfText> {
    // Loading, metadata and the page list run inside the guard too (Phase
    // A1.5, independent review finding 14: a panic there ended the import's
    // task and the interface waited for ever). A stack overflow (a document
    // that recurses without end) is not a panic and is not caught here.
    catch_unwind(AssertUnwindSafe(|| read_pages_unguarded(bytes, file_name))).unwrap_or_else(|_| {
        Err(AppError::Import(
            UserMessage::new(
                "library.badPdf",
                format!("“{file_name}” could not be read as a PDF."),
            )
            .with("fileName", file_name),
        ))
    })
}

fn read_pages_unguarded(bytes: &[u8], file_name: &str) -> AppResult<PdfText> {
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
    // A document whose drawing would recurse without end overflows the
    // extractor's stack, which no guard catches: refused before it is
    // drawn (Phase A1.5, independent review finding 3).
    if recursion_hazard(&document) {
        return Err(unreadable());
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

/// How deep forms may draw forms, and how much of the graph is walked,
/// before a document is taken to recurse without end. Real documents draw
/// a form inside a form rarely, and never dozens deep.
const MAX_FORM_DEPTH: usize = 32;
const MAX_FORM_VISITS: usize = 100_000;
const MAX_PARENTS: usize = 256;

/// Whether drawing the document could recurse without end in the text
/// extractor (pdf-extract 0.12): a page's `/Parent` chain that loops (its
/// inherited properties are looked up recursively), or a form XObject that
/// draws itself through any chain of forms (`Do` draws a form recursively,
/// with the form's own resources or else the caller's), or forms nested
/// past [`MAX_FORM_DEPTH`]. Every XObject a resource dictionary names
/// counts, drawn or not: the walk must not depend on parsing content.
fn recursion_hazard(document: &Document) -> bool {
    use std::collections::HashSet;
    fn deref<'a>(document: &'a Document, o: &'a Object) -> Option<&'a lopdf::Dictionary> {
        match document.dereference(o).ok()?.1 {
            Object::Dictionary(d) => Some(d),
            Object::Stream(s) => Some(&s.dict),
            _ => None,
        }
    }
    let mut safe: HashSet<(lopdf::ObjectId, usize)> = HashSet::new();
    let mut visits = 0usize;
    // (form, resources it would be drawn with) → whether drawing it may
    // recurse without end.
    fn walk<'a>(
        document: &'a Document,
        resources: &'a lopdf::Dictionary,
        stack: &mut Vec<lopdf::ObjectId>,
        safe: &mut HashSet<(lopdf::ObjectId, usize)>,
        visits: &mut usize,
    ) -> bool {
        if stack.len() > MAX_FORM_DEPTH {
            return true;
        }
        let Some(xobjects) = resources
            .get(b"XObject")
            .ok()
            .and_then(|o| deref(document, o))
        else {
            return false;
        };
        for (_, value) in xobjects.iter() {
            let Ok(id) = value.as_reference() else {
                continue;
            };
            *visits += 1;
            if *visits > MAX_FORM_VISITS || stack.contains(&id) {
                return true;
            }
            let Ok(Object::Stream(form)) = document.get_object(id) else {
                continue;
            };
            if form.dict.get(b"Subtype").and_then(Object::as_name).ok() != Some(b"Form".as_slice())
            {
                continue;
            }
            let own = form
                .dict
                .get(b"Resources")
                .ok()
                .and_then(|o| deref(document, o));
            let next = own.unwrap_or(resources);
            let key = (id, next as *const lopdf::Dictionary as usize);
            if safe.contains(&key) {
                continue;
            }
            stack.push(id);
            if walk(document, next, stack, safe, visits) {
                return true;
            }
            stack.pop();
            safe.insert(key);
        }
        false
    }
    for (_, page) in document.get_pages() {
        // The `/Parent` chain, and the resources the page inherits along it.
        let mut seen = HashSet::new();
        let mut id = page;
        let mut resources = None;
        loop {
            if !seen.insert(id) || seen.len() > MAX_PARENTS {
                return true;
            }
            let Ok(node) = document.get_dictionary(id) else {
                break;
            };
            if resources.is_none() {
                resources = node.get(b"Resources").ok().and_then(|o| deref(document, o));
            }
            match node.get(b"Parent").and_then(Object::as_reference) {
                Ok(parent) => id = parent,
                Err(_) => break,
            }
        }
        if let Some(resources) = resources {
            if walk(document, resources, &mut Vec::new(), &mut safe, &mut visits) {
                return true;
            }
        }
    }
    false
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

#[cfg(test)]
mod recursion_tests {
    use super::*;
    use lopdf::content::{Content, Operation};
    use lopdf::{dictionary, Stream};

    /// One page drawing form `X1`; the form draws `X1` again when
    /// `self_drawing`, else a second form that draws nothing.
    fn pdf(self_drawing: bool) -> Vec<u8> {
        let mut doc = Document::with_version("1.5");
        let pages_id = doc.new_object_id();
        let font_id = doc.add_object(dictionary! {
            "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica",
        });
        let text = Content {
            operations: vec![
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec!["F1".into(), 11.into()]),
                Operation::new("Td", vec![72.into(), 700.into()]),
                Operation::new("Tj", vec![Object::string_literal("Invented words")]),
                Operation::new("ET", vec![]),
            ],
        };
        let leaf = doc.add_object(Stream::new(
            dictionary! { "Type" => "XObject", "Subtype" => "Form",
            "BBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
            "Resources" => dictionary! { "Font" => dictionary! { "F1" => font_id } } },
            text.encode().unwrap(),
        ));
        let form_id = doc.new_object_id();
        let draws = Content {
            operations: vec![Operation::new("Do", vec!["X1".into()])],
        };
        let target = if self_drawing { form_id } else { leaf };
        doc.objects.insert(
            form_id,
            Object::Stream(Stream::new(
                dictionary! { "Type" => "XObject", "Subtype" => "Form",
                "BBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
                "Resources" => dictionary! { "XObject" => dictionary! { "X1" => target } } },
                draws.encode().unwrap(),
            )),
        );
        let content_id = doc.add_object(Stream::new(dictionary! {}, draws.encode().unwrap()));
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page", "Parent" => pages_id, "Contents" => content_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
            "Resources" => dictionary! { "XObject" => dictionary! { "X1" => form_id } },
        });
        doc.objects.insert(
            pages_id,
            Object::Dictionary(dictionary! {
                "Type" => "Pages", "Kids" => vec![Object::Reference(page_id)], "Count" => 1,
            }),
        );
        let catalog_id = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
        doc.trailer.set("Root", catalog_id);
        let mut bytes = Vec::new();
        doc.save_to(&mut bytes).unwrap();
        bytes
    }

    #[test]
    fn a_form_that_draws_itself_is_refused_before_it_is_drawn() {
        // Phase A1.5 (independent review finding 3): drawn, it recursed until
        // the stack overflowed and the whole app was gone.
        assert!(read_pages(&pdf(true), "loop.pdf").is_err());
        // The control: a form drawing another form once is read.
        let read = read_pages(&pdf(false), "nested.pdf").unwrap();
        assert!(read.pages[0].contains("Invented words"), "{:?}", read.pages);
    }
}
