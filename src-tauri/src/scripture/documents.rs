//! Bible documents (v0.23): PDF, EPUB, Word (.docx), Markdown and plain-text
//! files added as Bibles.
//!
//! The file is read by the **library's own adapters** (`library::adapters`):
//! the same text extraction, page locations, chapter structure and safety
//! limits the Sources library uses. Nothing here parses a PDF, EPUB or DOCX
//! itself. What this module adds is the Bible's view of that text:
//!
//! * the **segments** kept for reading and searching the document as it is
//!   (paragraphs with their page or section, as the library keeps them);
//! * the **units** the verse recogniser reads (`scripture::structure`): the
//!   lines of a PDF page or text file, or the paragraphs and headings of an
//!   EPUB, DOCX or Markdown file, each with the segment it lies in, so every
//!   verse found can be traced back to its place in the original.
//!
//! A PDF without a text layer (scanned pages) is kept with no segments and
//! no units: it can be read page by page in the original, never searched or
//! divided into verses. No OCR is done and nothing leaves the computer.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::{AppError, AppResult, UserMessage};
use crate::library::adapters::{
    docx::DocxImporter,
    epub::EpubImporter,
    markdown::MarkdownImporter,
    pdf,
    pdf_layout::{LineRole, PageOrder},
    text::TextImporter,
    ExtractionStatus, FoundMetadata, NewSegment, SourceImportAdapter,
};
use crate::library::files::content_hash;

/// Largest document accepted (as the library: a long PDF Bible is well under this).
pub const MAX_FILE_BYTES: u64 = 200 * 1024 * 1024;
/// All the documents of one Bible together.
pub const MAX_TOTAL_BYTES: u64 = 400 * 1024 * 1024;
/// More documents than this are not one edition.
pub const MAX_FILES: usize = 100;

/// The file extensions read as Bible documents.
pub fn is_document_extension(extension: &str) -> bool {
    matches!(
        extension,
        "pdf" | "epub" | "docx" | "md" | "markdown" | "txt"
    )
}

/// Whether a Bible's source format is a document (v0.23).
pub fn is_document_format(format: &str) -> bool {
    matches!(format, "pdf" | "epub" | "docx" | "markdown" | "text")
}

/// One piece of text the verse recogniser reads, with where it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unit {
    pub text: String,
    /// A heading of the document's own structure (EPUB, DOCX, Markdown).
    /// PDF and plain-text lines never are: their headings are recognised
    /// by what they say, not by how they look.
    pub heading: bool,
    pub place: Place,
    /// On a PDF page whose reading order could not be established from the
    /// positions of its text (v0.24.1): verses read from it are not used.
    pub uncertain_order: bool,
    /// Notes at the foot of a PDF page, known from where and how they are set
    /// (smaller text below all of the page's running text; v0.24.1).
    pub note: bool,
    /// A PDF line set noticeably larger than the page's running text (a
    /// title or heading; v0.24.1).
    pub large: bool,
    /// A PDF line set in another typeface than the page's running text
    /// (bold, italic): with other evidence, a heading.
    pub styled: bool,
}

/// Where a unit is in the original document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Place {
    /// Which of the Bible's documents (0 = the first).
    pub document: u16,
    /// The page (PDF), 1 = the first.
    pub page: Option<u32>,
    /// The kept segment the unit lies in (its position in the document).
    pub segment: Option<u32>,
}

/// A document read and ready to be kept.
#[derive(Debug)]
pub struct DocumentFile {
    /// The file as chosen (copied on commit).
    pub path: PathBuf,
    pub file_name: String,
    /// "pdf", "epub", "docx", "markdown" or "text".
    pub format: &'static str,
    pub extension: String,
    pub size: u64,
    pub hash: String,
    pub metadata: FoundMetadata,
    pub extraction_status: ExtractionStatus,
    pub page_count: Option<u32>,
    pub segments: Vec<NewSegment>,
    pub units: Vec<Unit>,
    /// Footnotes and endnotes kept apart from the body (DOCX).
    pub notes: usize,
    /// How each PDF page's reading order was established (empty otherwise).
    pub page_orders: Vec<PageOrder>,
    /// PDF pages whose text could not be read, or only in part.
    pub failed_pages: Vec<u32>,
}

impl DocumentFile {
    /// Characters of readable text.
    pub fn text_chars(&self) -> usize {
        self.segments.iter().map(|s| s.text.chars().count()).sum()
    }

    /// Whether the original is shown as it is laid out (PDF pages, the
    /// EPUB's and DOCX's formatting); Markdown and plain text are shown as
    /// their text.
    pub fn keeps_layout(&self) -> bool {
        matches!(self.format, "pdf" | "epub" | "docx")
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "file".into())
}

fn extension(path: &Path) -> String {
    path.extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default()
}

/// Reads one document. `document` is its place among the Bible's documents.
pub fn read(path: &Path, document: u16) -> AppResult<DocumentFile> {
    let name = file_name(path);
    let extension = extension(path);
    let size = fs::metadata(path)
        .map_err(|e| AppError::filesystem(path, e))?
        .len();
    if size > MAX_FILE_BYTES {
        return Err(AppError::Import(
            UserMessage::new(
                "library.tooLarge",
                format!("“{name}” is too large to import."),
            )
            .with("fileName", &name),
        ));
    }
    let bytes = fs::read(path).map_err(|e| AppError::filesystem(path, e))?;
    read_bytes(path, &name, &extension, &bytes, size, document)
}

/// Reads a document already in memory (tests read fixtures this way).
pub fn read_bytes(
    path: &Path,
    name: &str,
    extension: &str,
    bytes: &[u8],
    size: u64,
    document: u16,
) -> AppResult<DocumentFile> {
    let hash = content_hash(bytes);
    let mut file = match extension {
        "pdf" => {
            let text = pdf::read_pages(bytes, name)?;
            let page_count = Some(text.pages.len() as u32);
            if text.scanned {
                DocumentFile {
                    path: path.to_path_buf(),
                    file_name: name.into(),
                    format: "pdf",
                    extension: extension.into(),
                    size,
                    hash,
                    metadata: text.metadata,
                    extraction_status: ExtractionStatus::OcrRequired,
                    page_count,
                    segments: Vec::new(),
                    units: Vec::new(),
                    notes: 0,
                    page_orders: text.orders.clone(),
                    failed_pages: text.failed_pages.clone(),
                }
            } else {
                let segments = pdf::page_segments(&text.pages);
                let mut units = Vec::new();
                for (index, page) in text.pages.iter().enumerate() {
                    let number = index as u32 + 1;
                    let from = units.len();
                    line_units(page, Some(number), &segments, document, &mut units);
                    let uncertain = text.orders.get(index) == Some(&PageOrder::Uncertain);
                    // The page's lines and its units are in the same order
                    // (one unit per non-empty line of the page's text).
                    let layout = &text.layouts[index];
                    let body = layout.body_size();
                    let lines: Vec<(LineRole, f64, Option<&String>, bool)> = layout
                        .lines
                        .iter()
                        .filter(|l| !l.text.trim().is_empty())
                        .map(|l| (l.role, l.size, l.marked.as_ref(), l.styled))
                        .collect();
                    let aligned = lines.len() == units.len() - from;
                    // Larger text is a heading only where it is little of the
                    // page. When much of the page is larger than its most
                    // common size (a page dense with smaller notes, where the
                    // notes' size looks like the body's), which text is
                    // Scripture is not known: the page is uncertain.
                    let letters = |t: &str| t.chars().filter(|c| !c.is_whitespace()).count();
                    let text_letters: usize = layout
                        .lines
                        .iter()
                        .filter(|l| l.role == LineRole::Text)
                        .map(|l| letters(&l.text))
                        .sum();
                    let large_letters: usize = layout
                        .lines
                        .iter()
                        .filter(|l| l.role == LineRole::Text && l.size >= 1.15 * body)
                        .map(|l| letters(&l.text))
                        .sum();
                    // A page read only in part (cut off at the character limit)
                    // may end in the middle of a verse.
                    let uncertain = uncertain
                        || large_letters * 5 > text_letters
                        || text.failed_pages.contains(&number);
                    for (i, unit) in units[from..].iter_mut().enumerate() {
                        unit.uncertain_order = uncertain;
                        if aligned {
                            unit.note = lines[i].0 == LineRole::Note;
                            unit.large = lines[i].0 == LineRole::Text && lines[i].1 >= 1.15 * body;
                            unit.styled = lines[i].3;
                            // Raised letters marked for the recogniser.
                            if let Some(marked) = lines[i].2 {
                                unit.text = marked.trim().to_string();
                            }
                        }
                    }
                }
                DocumentFile {
                    path: path.to_path_buf(),
                    file_name: name.into(),
                    format: "pdf",
                    extension: extension.into(),
                    size,
                    hash,
                    metadata: text.metadata,
                    extraction_status: ExtractionStatus::ExtractedText,
                    page_count,
                    segments,
                    units,
                    notes: 0,
                    page_orders: text.orders,
                    failed_pages: text.failed_pages,
                }
            }
        }
        "txt" => {
            let doc = TextImporter.extract(bytes, name)?;
            let text = crate::library::adapters::text::decode_utf8(bytes, name)?;
            let mut units = Vec::new();
            line_units(&text, None, &doc.segments, document, &mut units);
            DocumentFile {
                path: path.to_path_buf(),
                file_name: name.into(),
                format: doc.format,
                extension: extension.into(),
                size,
                hash,
                metadata: doc.metadata,
                extraction_status: doc.extraction_status,
                page_count: None,
                segments: doc.segments,
                units,
                notes: 0,
                page_orders: Vec::new(),
                failed_pages: Vec::new(),
            }
        }
        "epub" | "docx" | "md" | "markdown" => {
            let adapter: &dyn SourceImportAdapter = match extension {
                "epub" => &EpubImporter,
                "docx" => &DocxImporter,
                _ => &MarkdownImporter,
            };
            let mut doc = adapter.extract(bytes, name)?;
            if extension == "epub" {
                take_out_note_links(bytes, &mut doc.segments);
            }
            let (units, notes) = structured_units(&doc.segments, document);
            DocumentFile {
                path: path.to_path_buf(),
                file_name: name.into(),
                format: doc.format,
                extension: extension.into(),
                size,
                hash,
                metadata: doc.metadata,
                extraction_status: doc.extraction_status,
                page_count: doc.page_count,
                segments: doc.segments,
                units,
                notes,
                page_orders: Vec::new(),
                failed_pages: Vec::new(),
            }
        }
        _ => {
            return Err(AppError::UnsupportedFormat(
                UserMessage::new(
                    "bibleImport.documentFormat",
                    format!("“{name}” is not a kind of document Abiding Word can read as a Bible."),
                )
                .with("fileName", name),
            ))
        }
    };
    if file.segments.is_empty() && file.extraction_status != ExtractionStatus::OcrRequired {
        // Nothing to read at all (an empty text file, an EPUB without text).
        return Err(AppError::Import(
            UserMessage::new(
                "bibleImport.documentEmpty",
                format!("No text was found in “{name}”."),
            )
            .with("fileName", name),
        ));
    }
    file.units.retain(|u| !u.text.trim().is_empty());
    Ok(file)
}

/// The lines of a page (PDF) or a whole text file, each with the segment
/// whose paragraph it belongs to. `segments` carry byte offsets into the
/// same text (`char_start`/`char_end`, as the library's adapters record).
fn line_units(
    text: &str,
    page: Option<u32>,
    segments: &[NewSegment],
    document: u16,
    out: &mut Vec<Unit>,
) {
    // The segments of this page (or the file), by where their paragraph starts.
    let ranges: Vec<(u32, u32, u32)> = segments
        .iter()
        .enumerate()
        .filter(|(_, s)| s.page_number == page)
        .filter_map(|(i, s)| Some((s.char_start?, s.char_end?, i as u32)))
        .collect();
    let mut offset = 0u32;
    for line in text.split_inclusive('\n') {
        let start = offset;
        offset += line.len() as u32;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let segment = ranges
            .iter()
            .find(|(s, e, _)| start + 1 > *s && start < *e + 1)
            .or_else(|| ranges.iter().find(|(s, _, _)| *s >= start))
            .map(|(_, _, i)| *i);
        out.push(Unit {
            text: trimmed.to_string(),
            heading: false,
            place: Place {
                document,
                page,
                segment,
            },
            uncertain_order: false,
            note: false,
            large: false,
            styled: false,
        });
    }
}

/// The paragraphs of an EPUB, DOCX or Markdown document, with its headings
/// put back between them: the adapters keep headings as each paragraph's
/// section path ("Genesis › Chapter 1"), so a heading starts wherever that
/// path changes. Footnotes and endnotes (DOCX) are left out and counted.
/// An EPUB's note references (v0.24.2): a link inside the book whose text is
/// a letter or a sign ("a", "†", "*"), leading to another passage, is a
/// reference to a note — the book's own markup says so. Its text is taken
/// out of the passage (for reading verses only; the library keeps the text
/// as published) and the passage it leads to is a note. A link of digits
/// is left alone: it may be a verse's number.
fn take_out_note_links(bytes: &[u8], segments: &mut [NewSegment]) {
    use crate::library::adapters::epub;
    let links = epub::links(bytes, segments.len());
    // A passage that is nothing but links to other places (a list of the
    // book's chapters, "1 Samuel" leading to the book's index) is the
    // book's navigation, not its text.
    let mut linked: std::collections::BTreeMap<usize, usize> = Default::default();
    for link in &links {
        if let Ok(at) = usize::try_from(link.position) {
            *linked.entry(at).or_default() += link.end.saturating_sub(link.start);
        }
    }
    for (at, covered) in linked {
        if let Some(segment) = segments.get_mut(at) {
            let length: usize = segment.text.chars().map(char::len_utf16).sum();
            let spaces = segment.text.chars().filter(|c| c.is_whitespace()).count();
            if covered + spaces >= length && length > 0 {
                segment.text.clear();
            }
        }
    }
    let mut cuts: std::collections::BTreeMap<usize, Vec<(usize, usize)>> = Default::default();
    let mut notes: std::collections::BTreeSet<usize> = Default::default();
    for link in &links {
        let (Ok(at), Ok(target)) = (usize::try_from(link.position), usize::try_from(link.target))
        else {
            continue;
        };
        if at == target || at >= segments.len() || target >= segments.len() {
            continue;
        }
        let text = &segments[at].text;
        // UTF-16 offsets, as the links count them, to byte offsets.
        let byte = |utf16: usize| -> Option<usize> {
            let mut units = 0usize;
            for (i, c) in text.char_indices() {
                if units >= utf16 {
                    return Some(i);
                }
                units += c.len_utf16();
            }
            (units >= utf16).then_some(text.len())
        };
        let (Some(start), Some(end)) = (byte(link.start), byte(link.end)) else {
            continue;
        };
        let label = text[start..end].trim();
        let short = !label.is_empty() && label.chars().count() <= 3;
        let no_digits = !label.chars().any(|c| c.is_ascii_digit());
        if short && no_digits && !label.chars().all(char::is_whitespace) {
            cuts.entry(at).or_default().push((start, end));
            // The passage it leads to is a note when it begins with the
            // same label ("a 1:1 …", "† 1:1 …"); a link to a verse never
            // makes the verse a note.
            if segments[target].text.trim_start().starts_with(label) {
                notes.insert(target);
            }
        }
    }
    for (at, mut ranges) in cuts {
        ranges.sort_unstable();
        let text = &segments[at].text;
        let mut out = String::with_capacity(text.len());
        let mut from = 0;
        for (start, end) in ranges {
            if start < from {
                continue;
            }
            out.push_str(&text[from..start]);
            from = end;
        }
        out.push_str(&text[from..]);
        segments[at].text = out;
    }
    for target in notes {
        if let Some(segment) = segments.get_mut(target) {
            segment.note = true;
        }
    }
}

fn structured_units(segments: &[NewSegment], document: u16) -> (Vec<Unit>, usize) {
    let mut out = Vec::new();
    let mut notes = 0usize;
    let mut previous: Vec<&str> = Vec::new();
    for (index, segment) in segments.iter().enumerate() {
        let place = Place {
            document,
            page: segment.page_number,
            segment: Some(index as u32),
        };
        if segment.note {
            // Kept as a note (never verse text): its sign and verse show
            // which signs in the verses are references to it.
            notes += 1;
            out.push(Unit {
                text: segment.text.clone(),
                heading: false,
                place,
                uncertain_order: false,
                note: true,
                large: false,
                styled: false,
            });
            continue;
        }
        let path: Vec<&str> = segment
            .section_path
            .as_deref()
            .map(|p| p.split(" › ").collect())
            .unwrap_or_default();
        // The first level where the path differs: every heading from there on
        // was passed since the paragraph before.
        let same = previous
            .iter()
            .zip(path.iter())
            .take_while(|(a, b)| a == b)
            .count();
        let changed = same < path.len() || path.len() < previous.len();
        if changed {
            for heading in &path[same..] {
                out.push(Unit {
                    text: (*heading).to_string(),
                    heading: true,
                    place,
                    uncertain_order: false,
                    note: false,
                    large: false,
                    styled: false,
                });
            }
        }
        previous = path;
        out.push(Unit {
            text: segment.text.clone(),
            heading: false,
            place,
            uncertain_order: false,
            note: false,
            large: false,
            styled: segment.styled,
        });
    }
    (out, notes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_named(name: &str, bytes: &[u8]) -> DocumentFile {
        let extension = name.rsplit('.').next().unwrap();
        read_bytes(
            Path::new(name),
            name,
            extension,
            bytes,
            bytes.len() as u64,
            0,
        )
        .unwrap()
    }

    #[test]
    fn text_lines_keep_their_paragraph() {
        let file = read_named(
            "bible.txt",
            "Genesis\n\nChapter 1\n\n1 In the beginning.\n2 And the earth\nwas without form.\n"
                .as_bytes(),
        );
        let texts: Vec<&str> = file.units.iter().map(|u| u.text.as_str()).collect();
        assert_eq!(
            texts,
            [
                "Genesis",
                "Chapter 1",
                "1 In the beginning.",
                "2 And the earth",
                "was without form."
            ]
        );
        // Lines of the same paragraph point at the same kept segment.
        assert_eq!(file.units[3].place.segment, file.units[4].place.segment);
        assert_eq!(file.units[0].place.segment, Some(0));
        assert_eq!(file.units[2].place.segment, Some(2));
        assert!(file.units.iter().all(|u| !u.heading));
    }

    #[test]
    fn markdown_headings_come_back_between_paragraphs() {
        let file = read_named(
            "bible.md",
            "# Genesis\n\n## Chapter 1\n\n1 In the beginning.\n\n## Chapter 2\n\n1 Thus the heavens.\n"
                .as_bytes(),
        );
        let got: Vec<(&str, bool)> = file
            .units
            .iter()
            .map(|u| (u.text.as_str(), u.heading))
            .collect();
        assert_eq!(
            got,
            [
                ("Genesis", true),
                ("Chapter 1", true),
                ("1 In the beginning.", false),
                ("Chapter 2", true),
                ("1 Thus the heavens.", false),
            ]
        );
    }

    #[test]
    fn an_empty_text_file_is_refused() {
        let error = read_bytes(Path::new("x.txt"), "x.txt", "txt", b"\n\n", 2, 0).unwrap_err();
        assert!(format!("{error:?}").contains("documentEmpty"));
    }

    #[test]
    fn other_files_are_refused() {
        assert!(read_bytes(Path::new("x.rtf"), "x.rtf", "rtf", b"{\\rtf1}", 7, 0).is_err());
    }
}
