//! The structure-aware reading of a Bible's documents (Phase A1).
//!
//! It sits beside the frozen reading (`documents::read`, `import::
//! stage_documents`), never in place of it, so that both can be run on the
//! same files and compared. For EPUBs it changes two things and adds one.
//!
//! 1. **Link roles by evidence.** A link is a note's caller when the book
//!    shows that it leads to a note (`epub_source::classify_links`), whatever
//!    its visible text. Callers are marked for the recogniser, not cut out
//!    of the text: the stored passage stays as published.
//! 2. **No caller removed by its look.** The frozen reading cuts short
//!    links of letters or signs out of the text by their label. This
//!    reading does not.
//! 3. **Exact provenance and a gate.** Every verse is traced to ordered
//!    spans of the original entry's bytes, with excluded callers reported
//!    (`provenance`). A verse the recogniser verified stays verified only if:
//!    * it is traced;
//!    * no id at its start names another verse;
//!    * no link next to it is of uncertain role;
//!    * no caller-shaped link was read into its words.
//!
//!    The gate only ever withdraws verification, using the reasons that
//!    already exist (`boundary`, `noteSign`), so the stored values and the
//!    interface are unchanged.
//!
//! Other formats are read exactly as before, and their verses get no
//! provenance claim.

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::{AppError, AppResult, UserMessage};
use crate::library::adapters::epub;
use crate::library::adapters::epub_source::{
    self, caller_shaped, id_reference, LinkRole, LinkRoles, SourceBook,
};
use crate::library::files::content_hash;
use crate::scripture::documents::{self, DocumentFile, MAX_FILE_BYTES};
use crate::scripture::import::{self, ImportOrigin, StagedImport};
use crate::scripture::provenance::{
    self, collapse, EntryMarks, ExcludedKind, FileIdentity, IdCheck, TraceStatus, VerseProvenance,
};

/// What the structure-aware reading keeps of one EPUB, for tracing.
pub struct StructuralSource {
    pub document: u16,
    pub file_sha256: String,
    pub book: SourceBook,
    pub roles: LinkRoles,
    /// For each spine entry read: (archive path, first segment, segment count).
    pub entry_segments: Vec<(String, usize, usize)>,
}

/// Reads one document. EPUBs are read structure-aware (with their source);
/// any other format exactly as `documents::read` reads it.
pub fn read(path: &Path, document: u16) -> AppResult<(DocumentFile, Option<StructuralSource>)> {
    let extension = path
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    if extension != "epub" {
        return Ok((documents::read(path, document)?, None));
    }
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "file".into());
    let size = fs::metadata(path)
        .map_err(|e| AppError::filesystem(path, e))?
        .len();
    if size > MAX_FILE_BYTES {
        return Ok((documents::read(path, document)?, None));
    }
    let bytes = fs::read(path).map_err(|e| AppError::filesystem(path, e))?;
    read_epub_bytes(path, &name, &bytes, document)
}

/// Reads an EPUB already in memory, structure-aware.
pub fn read_epub_bytes(
    path: &Path,
    name: &str,
    bytes: &[u8],
    document: u16,
) -> AppResult<(DocumentFile, Option<StructuralSource>)> {
    let Some(book) = epub_source::read(bytes) else {
        // Not readable for its structure: read as before, with no claims.
        return Ok((
            documents::read_bytes(path, name, "epub", bytes, bytes.len() as u64, document)?,
            None,
        ));
    };
    let roles = epub_source::classify_links(&book);
    let (mut doc, entry_segments) = epub::read_structural(bytes, name, &book, &roles)?;
    let links = epub::links(bytes, doc.segments.len());
    documents::clear_navigation(&links, &mut doc.segments);
    let (units, notes) = documents::structured_units(&doc.segments, document);
    if doc.segments.is_empty() {
        return Err(AppError::Import(
            UserMessage::new(
                "bibleImport.documentEmpty",
                format!("“{name}” has no text to read."),
            )
            .with("fileName", name),
        ));
    }
    let hash = content_hash(bytes);
    let file = DocumentFile {
        path: path.to_path_buf(),
        file_name: name.into(),
        format: doc.format,
        extension: "epub".into(),
        size: bytes.len() as u64,
        hash: hash.clone(),
        metadata: doc.metadata,
        extraction_status: doc.extraction_status,
        page_count: doc.page_count,
        segments: doc.segments,
        units,
        notes,
        page_orders: Vec::new(),
        failed_pages: Vec::new(),
    };
    Ok((
        file,
        Some(StructuralSource {
            document,
            file_sha256: hash,
            book,
            roles,
            entry_segments,
        }),
    ))
}

/// Why the gate withdrew a verse's verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GateCause {
    /// The verse could not be traced to spans of the original.
    Untraced,
    /// An id at the verse's start names another chapter and verse.
    IdContradicts,
    /// A link of uncertain role (`Ambiguous`, `Conflict`) stands in or next
    /// to the verse.
    UncertainLink,
    /// A caller-shaped link that leads to no note was read as the verse's
    /// words.
    CallerShapedWords,
    /// The verse's text holds an element of the document's own verse-number
    /// class: a verse number was read as words.
    SwallowedVerseNumber,
}

impl GateCause {
    /// The existing unverified reason it is stored as.
    pub fn reason(self) -> &'static str {
        match self {
            GateCause::Untraced | GateCause::IdContradicts | GateCause::SwallowedVerseNumber => {
                "boundary"
            }
            GateCause::UncertainLink | GateCause::CallerShapedWords => "noteSign",
        }
    }
}

/// What the structure-aware reading found and decided, for the review and
/// for audits.
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralReport {
    /// Every verse of an EPUB the recogniser read, with its provenance.
    pub provenance: BTreeMap<String, VerseProvenance>,
    /// Verses whose verification the gate withdrew, and why.
    pub gated: BTreeMap<String, GateCause>,
    /// Links by role: "noteCaller", "navigation", "conflict", "ambiguous".
    pub links: BTreeMap<String, usize>,
    /// Verses traced exactly / with other spacing / not traced.
    pub exact: usize,
    pub spacing_differs: usize,
    pub untraced: usize,
    pub id_agrees: usize,
    pub id_contradicts: usize,
    /// Verses of documents read without a structural source (other formats).
    pub without_source: usize,
}

fn key_name(key: (u8, u16, u16)) -> String {
    let code = crate::scripture::canon::by_ordinal(key.0)
        .map(|b| b.code)
        .unwrap_or("???");
    format!("{code} {}:{}", key.1, key.2)
}

fn role_name(role: LinkRole) -> &'static str {
    match role {
        LinkRole::NoteCaller => "noteCaller",
        LinkRole::Navigation => "navigation",
        LinkRole::Conflict => "conflict",
        LinkRole::Ambiguous => "ambiguous",
    }
}

/// Stages a Bible's documents with the structure-aware reading: the same
/// steps as `import::stage_documents`, then provenance and the gate.
pub fn stage_documents(
    paths: &[PathBuf],
    origin: ImportOrigin,
) -> AppResult<(StagedImport, StructuralReport)> {
    let sources: RefCell<HashMap<u16, StructuralSource>> = RefCell::new(HashMap::new());
    let reader = |path: &Path, document: u16| -> AppResult<DocumentFile> {
        let (file, source) = read(path, document)?;
        if let Some(source) = source {
            sources.borrow_mut().insert(document, source);
        }
        Ok(file)
    };
    let mut staged = import::stage_documents_read(paths, origin, &reader)?;
    let sources = sources.into_inner();
    let report = gate(&mut staged, &sources);
    Ok((staged, report))
}

/// One verse, as the gate sees it.
struct Placed {
    key: (u8, u16, u16),
    entry: usize,
    start: usize,
    end: usize,
    matched_links: Vec<(LinkRole, usize)>,
    conflict_inside: bool,
    /// Characters of the entry from the first span's start to the last
    /// span's end.
    chars: (usize, usize),
}

/// Traces every verse read from an EPUB, and withdraws the verification of
/// those that fail the checks (see the module's notes).
pub fn gate(
    staged: &mut StagedImport,
    sources: &HashMap<u16, StructuralSource>,
) -> StructuralReport {
    let mut report = StructuralReport::default();
    for source in sources.values() {
        for (role, n) in source.roles.counts() {
            *report.links.entry(role_name(role).into()).or_default() += n;
        }
    }
    let Some(stage) = staged.documents.as_mut() else {
        return report;
    };
    let content = &staged.content;
    // The verses, by document, in the order they stand.
    type Verses = Vec<((u8, u16, u16), String)>;
    let mut by_document: BTreeMap<u16, Verses> = BTreeMap::new();
    for (&book, b) in &content.books {
        for (&chapter, c) in &b.chapters {
            for (&verse, text) in &c.verses {
                let key = (book, chapter, verse);
                let Some(origin) = stage.origins.get(&key) else {
                    continue;
                };
                by_document
                    .entry(origin.start.document)
                    .or_default()
                    .push((key, text.clone()));
            }
        }
    }
    let mut gated: BTreeMap<(u8, u16, u16), GateCause> = BTreeMap::new();
    for (document, mut verses) in by_document {
        let Some(source) = sources.get(&document) else {
            report.without_source += verses.len();
            continue;
        };
        let Some(file) = stage.files.get(usize::from(document)) else {
            continue;
        };
        verses.sort_by_key(|(key, _)| {
            let o = &stage.origins[key];
            (o.start.segment, *key)
        });
        // Marks and segment places, per entry.
        let book = &source.book;
        let mut links_of: Vec<Vec<(LinkRole, usize)>> = vec![Vec::new(); book.entries.len()];
        for l in &source.roles.links {
            links_of[l.entry].push((l.role, l.element));
        }
        let marks: Vec<EntryMarks> = book
            .entries
            .iter()
            .zip(links_of.iter())
            .map(|(e, l)| EntryMarks::new(e, l))
            .collect();
        // Segment → (entry, nonspace range).
        let mut segment_place: HashMap<usize, (usize, (usize, usize))> = HashMap::new();
        for (path, first, count) in &source.entry_segments {
            let Some(&entry) = book.by_path.get(path) else {
                continue;
            };
            let texts: Vec<&str> = file
                .segments
                .get(*first..first + count)
                .unwrap_or(&[])
                .iter()
                .map(|s| s.text.as_str())
                .collect();
            for (i, place) in provenance::locate_segments(&marks[entry], &texts)
                .into_iter()
                .enumerate()
            {
                if let Some(range) = place {
                    segment_place.insert(first + i, (entry, range));
                }
            }
        }
        let mut placed: Vec<Placed> = Vec::new();
        let mut cursor: Option<(usize, usize)> = None;
        for (key, text) in verses {
            let origin = stage.origins[&key];
            let verified = !stage.unverified.contains_key(&key);
            let from_segment = origin.start.segment.map(|s| s as usize);
            let to_segment = if origin.end.document == origin.start.document {
                origin.end.segment.map(|s| s as usize)
            } else {
                None
            };
            let places = (
                from_segment.and_then(|s| segment_place.get(&s)),
                to_segment.and_then(|s| segment_place.get(&s)),
            );
            let mut reasons = Vec::new();
            let mut entry_path = String::new();
            let mut entry_sha = String::new();
            let mut traced_out = None;
            if let (Some(&(entry, (from, _))), Some(&(entry_to, (_, to)))) = places {
                entry_path = book.entries[entry].path.clone();
                entry_sha = book.entries[entry].sha256.clone();
                if entry != entry_to {
                    reasons.push("its text crosses two documents of the book".to_string());
                } else {
                    let start = match cursor {
                        Some((e, at)) if e == entry && at > from && at < to => at,
                        _ => from,
                    };
                    let found = provenance::trace(&marks[entry], &text, start, to)
                        .or_else(|| provenance::trace(&marks[entry], &text, from, to));
                    if let Some(t) = found {
                        cursor = Some((entry, t.end));
                        traced_out = Some((entry, t));
                    } else {
                        reasons.push(
                            "its words are not found, in order, in the original text of its place (only white space, note callers and note signs may stand between them)"
                                .to_string(),
                        );
                    }
                }
            } else {
                reasons.push("its place in the original could not be found".to_string());
            }
            let mut record = VerseProvenance {
                reference: key,
                file: FileIdentity {
                    document,
                    file_sha256: source.file_sha256.clone(),
                    entry: entry_path,
                    entry_sha256: entry_sha,
                },
                text: text.clone(),
                spans: Vec::new(),
                excluded: Vec::new(),
                quote: String::new(),
                status: TraceStatus::Untraced,
                id_check: IdCheck::None,
                reasons,
            };
            if let Some((entry, t)) = traced_out {
                let m = &marks[entry];
                record.spans = provenance::spans(m, &t);
                record.excluded = provenance::excluded_ranges(m, &t);
                record.quote = provenance::quote(m, &record.spans);
                record.status = if record.quote == collapse(&text) {
                    TraceStatus::Exact
                } else {
                    TraceStatus::SpacingDiffers
                };
                record.id_check = id_check(m, &record, key);
                let matched_links: Vec<(LinkRole, usize)> =
                    t.matched.iter().filter_map(|&at| m.link[at]).collect();
                let conflict_inside = record
                    .excluded
                    .iter()
                    .any(|x| x.kind == ExcludedKind::ConflictingCaller);
                placed.push(Placed {
                    key,
                    entry,
                    start: m.nonspace_at(record.spans[0].char_start as usize),
                    end: t.end,
                    matched_links,
                    conflict_inside,
                    chars: (
                        record.spans[0].char_start as usize,
                        record.spans[record.spans.len() - 1].char_end as usize,
                    ),
                });
                record.reasons.push(format!(
                    "traced to {} span(s) of {}, {} excluded range(s)",
                    record.spans.len(),
                    record.file.entry,
                    record.excluded.len()
                ));
            }
            match record.status {
                TraceStatus::Exact => report.exact += 1,
                TraceStatus::SpacingDiffers => report.spacing_differs += 1,
                TraceStatus::Untraced => report.untraced += 1,
            }
            match record.id_check {
                IdCheck::Agrees => report.id_agrees += 1,
                IdCheck::Contradicts { .. } => report.id_contradicts += 1,
                IdCheck::None => {}
            }
            if verified {
                let cause = match (record.status, record.id_check) {
                    (TraceStatus::Untraced, _) => Some(GateCause::Untraced),
                    (_, IdCheck::Contradicts { .. }) => Some(GateCause::IdContradicts),
                    _ => None,
                };
                if let Some(cause) = cause {
                    gated.insert(key, cause);
                }
            } else if let Some(reason) = stage.unverified.get(&key) {
                record
                    .reasons
                    .push(format!("not verified by the recogniser ({reason})"));
            }
            report.provenance.insert(key_name(key), record);
        }
        // Links of uncertain role, and caller-shaped words.
        placed.sort_by_key(|p| (p.entry, p.start));
        // Verse numbers read as words.
        for key in swallowed_verse_numbers(book, &placed) {
            gated.entry(key).or_insert(GateCause::SwallowedVerseNumber);
        }
        for p in &placed {
            if p.conflict_inside {
                gated.entry(p.key).or_insert(GateCause::UncertainLink);
            }
            let shaped = p.matched_links.iter().any(|&(role, element)| {
                role == LinkRole::Navigation && {
                    let e = &book.entries[p.entry].elements[element];
                    caller_shaped(&book.entries[p.entry].slice(e.text_start, e.text_end))
                }
            });
            if shaped {
                gated.entry(p.key).or_insert(GateCause::CallerShapedWords);
            }
        }
        for l in &source.roles.links {
            if !matches!(l.role, LinkRole::Ambiguous | LinkRole::Conflict) {
                continue;
            }
            let m = &marks[l.entry];
            let at = m.nonspace_at(book.entries[l.entry].elements[l.element].text_start);
            let same: Vec<&Placed> = placed.iter().filter(|p| p.entry == l.entry).collect();
            // The verse it stands in, or the two it stands between.
            let inside: Vec<&&Placed> = same
                .iter()
                .filter(|p| p.start <= at && at < p.end)
                .collect();
            if !inside.is_empty() {
                for p in inside {
                    gated.entry(p.key).or_insert(GateCause::UncertainLink);
                }
                continue;
            }
            if let Some(before) = same.iter().rev().find(|p| p.end <= at) {
                gated.entry(before.key).or_insert(GateCause::UncertainLink);
            }
            if let Some(after) = same.iter().find(|p| p.start > at) {
                gated.entry(after.key).or_insert(GateCause::UncertainLink);
            }
        }
    }
    for (key, cause) in gated {
        if stage.unverified.contains_key(&key) {
            continue;
        }
        stage.unverified.insert(key, cause.reason());
        let name = key_name(key);
        if let Some(record) = report.provenance.get_mut(&name) {
            record.reasons.push(format!(
                "verification withdrawn by the structural check: {cause:?} (stored as \"{}\")",
                cause.reason()
            ));
        }
        report.gated.insert(name, cause);
    }
    for record in report.provenance.values_mut() {
        if !report.gated.contains_key(&key_name(record.reference))
            && !stage.unverified.contains_key(&record.reference)
        {
            record.reasons.push("verified".to_string());
        }
    }
    report
}

/// An id that encodes a chapter and verse, set where the verse begins
/// (nothing but white space, digits, punctuation or excluded text between
/// it and the verse's first word), compared with the verse.
fn id_check(marks: &EntryMarks, record: &VerseProvenance, key: (u8, u16, u16)) -> IdCheck {
    let entry = marks.entry;
    let Some(first) = record.spans.first() else {
        return IdCheck::None;
    };
    let first = first.char_start as usize;
    let mut best: Option<(usize, (u16, u16))> = None;
    for e in &entry.elements {
        let Some(id) = e.id.as_deref() else {
            continue;
        };
        if e.text_start > first {
            continue;
        }
        let Some(reference) = id_reference(id) else {
            continue;
        };
        let clear = (e.text_start..first).all(|i| {
            let c = entry.text[i];
            c.is_whitespace()
                || c.is_ascii_digit()
                || c.is_ascii_punctuation()
                || marks.link[i].is_some_and(|(r, _)| matches!(r, LinkRole::NoteCaller))
        });
        if clear && best.is_none_or(|(at, _)| e.text_start >= at) {
            best = Some((e.text_start, reference));
        }
    }
    match best {
        None => IdCheck::None,
        Some((_, (chapter, verse))) if (chapter, verse) == (key.1, key.2) => IdCheck::Agrees,
        // A longer run of digits ending in the verse's own chapter cannot be
        // split with certainty: some generators put a digit of the book's
        // code in front of the chapter ("R110_1" for 1 Chronicles 10:1).
        // That is no evidence either way.
        Some((_, (chapter, verse)))
            if verse == key.2 && digits_end_with(chapter, key.1) && chapter != key.1 =>
        {
            IdCheck::None
        }
        Some((_, (chapter, verse))) => IdCheck::Contradicts { chapter, verse },
    }
}

/// Whether the decimal digits of `long` end with those of `short`.
fn digits_end_with(long: u16, short: u16) -> bool {
    long.to_string().ends_with(&short.to_string())
}

/// A number as verse numbers are printed: digits, perhaps a range or list
/// ("12", "1-11", "3,4"); returns its leading value.
fn verse_number_text(text: &str) -> Option<u16> {
    let t = text.trim();
    if t.is_empty()
        || t.chars().count() > 9
        || !t
            .chars()
            .all(|c| c.is_ascii_digit() || matches!(c, '-' | '\u{2010}'..='\u{2013}' | ','))
    {
        return None;
    }
    let lead: String = t.chars().take_while(|c| c.is_ascii_digit()).collect();
    lead.parse().ok()
}

/// Verses whose traced text holds an element of the document's own
/// verse-number class.
///
/// A class is the document's verse-number class only by measurement: at
/// least ten of the verses read stand directly after an element of that
/// class whose text is their own number, and those make up at least half of
/// that class's number-shaped elements. Nothing is assumed from the class's
/// name.
fn swallowed_verse_numbers(book: &SourceBook, placed: &[Placed]) -> Vec<(u8, u16, u16)> {
    // Number-shaped elements with classes, per entry, in text order.
    let numbers: Vec<Vec<(usize, usize, usize, u16)>> = book
        .entries
        .iter()
        .map(|entry| {
            let mut v: Vec<(usize, usize, usize, u16)> = entry
                .elements
                .iter()
                .enumerate()
                .filter(|(_, e)| !e.classes.is_empty() && e.text_end > e.text_start)
                .filter_map(|(i, e)| {
                    verse_number_text(&entry.slice(e.text_start, e.text_end))
                        .map(|n| (e.text_start, e.text_end, i, n))
                })
                .collect();
            v.sort_unstable();
            v
        })
        .collect();
    let mut starts: HashMap<&str, usize> = HashMap::new();
    let mut shaped: HashMap<&str, usize> = HashMap::new();
    for (entry, list) in book.entries.iter().zip(&numbers) {
        for &(_, _, i, _) in list {
            for c in &entry.elements[i].classes {
                *shaped.entry(c.as_str()).or_default() += 1;
            }
        }
    }
    for p in placed {
        let entry = &book.entries[p.entry];
        let list = &numbers[p.entry];
        let before = list.partition_point(|&(_, end, _, _)| end <= p.chars.0);
        // The last number element ending at or before the verse's text.
        let Some(&(_, end, i, n)) = list[..before].iter().max_by_key(|x| x.1) else {
            continue;
        };
        let gap_clear = (end..p.chars.0).all(|at| entry.text[at].is_whitespace());
        if gap_clear && n == p.key.2 {
            for c in &entry.elements[i].classes {
                *starts.entry(c.as_str()).or_default() += 1;
            }
        }
    }
    let marker: HashSet<&str> = starts
        .iter()
        .filter(|(c, &n)| n >= 10 && n * 2 >= shaped.get(*c).copied().unwrap_or(0))
        .map(|(c, _)| *c)
        .collect();
    if marker.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for p in placed {
        let entry = &book.entries[p.entry];
        let list = &numbers[p.entry];
        let from = list.partition_point(|&(start, _, _, _)| start < p.chars.0);
        let inside = list[from..]
            .iter()
            .take_while(|&&(start, _, _, _)| start < p.chars.1)
            .any(|&(_, end, i, _)| {
                end <= p.chars.1
                    && entry.elements[i]
                        .classes
                        .iter()
                        .any(|c| marker.contains(c.as_str()))
            });
        if inside {
            out.push(p.key);
        }
    }
    out
}
