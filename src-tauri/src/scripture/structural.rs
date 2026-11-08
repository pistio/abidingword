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
    /// Letters of the original between this verse and the next that no verse
    /// holds and nothing in the document accounts for (a heading, a note, a
    /// class that never carries verse text): words may have been dropped.
    UnaccountedText,
}

impl GateCause {
    /// The existing unverified reason it is stored as.
    pub fn reason(self) -> &'static str {
        match self {
            GateCause::Untraced
            | GateCause::IdContradicts
            | GateCause::SwallowedVerseNumber
            | GateCause::UnaccountedText => "boundary",
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
    /// The verse's spans, as character ranges of the entry.
    spans: Vec<(usize, usize)>,
    /// Verified by the recogniser.
    verified: bool,
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
        let mut marks: Vec<EntryMarks> = book
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
        // Where verses that could not be traced begin: (entry, nonspace).
        let mut unplaced: Vec<(usize, usize)> = Vec::new();
        // Each verse's place in the original: (entry, from, to) nonspace.
        let mut verse_places: HashMap<(u8, u16, u16), (usize, usize, usize)> = HashMap::new();
        let mut removals: HashMap<(u8, u16, u16), Vec<Removal>> = HashMap::new();
        // Pass 0 learns which classes carry verse text; pass 1 traces again
        // with the removals the document grounds, and decides.
        for pass in 0..2 {
            let last = pass == 1;
            if last {
                let owners: Vec<Vec<u32>> = book.entries.iter().map(|e| e.owners()).collect();
                let apart = SetApart::measure(book, &source.roles, &marks, &owners, &placed);
                removals = grounded_removals(
                    &stage.apparatus,
                    &verse_places,
                    &marks,
                    &owners,
                    &apart,
                    book,
                );
                for list in removals.values() {
                    for r in list {
                        marks[r.entry].mark_removed(r.element);
                    }
                }
            }
            placed.clear();
            unplaced.clear();
            let mut cursor: Option<(usize, usize)> = None;
            for (key, text) in verses.iter().cloned() {
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
                    if !last && entry == entry_to {
                        verse_places.insert(key, (entry, from, to));
                    }
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
                        let found = found.filter(|t| {
                            removals_in_place(
                                &marks[entry],
                                t,
                                removals.get(&key).map(Vec::as_slice),
                            )
                        });
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
                        verified,
                        spans: record
                            .spans
                            .iter()
                            .map(|r| (r.char_start as usize, r.char_end as usize))
                            .collect(),
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
                } else if let Some(&(entry, (from, _))) = places.0 {
                    unplaced.push((entry, from));
                }
                if !last {
                    continue;
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
        }
        // Links of uncertain role, and caller-shaped words.
        placed.sort_by_key(|p| (p.entry, p.start));
        // Letters between a verse and the next that nothing accounts for.
        {
            let owners: Vec<Vec<u32>> = book.entries.iter().map(|e| e.owners()).collect();
            let apart = SetApart::measure(book, &source.roles, &marks, &owners, &placed);
            for key in unaccounted_text(book, &marks, &owners, &apart, &placed, &unplaced) {
                gated.entry(key).or_insert(GateCause::UnaccountedText);
            }
        }
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

/// What the document itself sets apart from its verse text (Phase A1.1).
///
/// Structure first (`Entry::set_apart`: headings, notes, navigation,
/// floated blocks). Then measured classes: a class is apart when at least
/// three elements carry it and no character of any traced verse the
/// recogniser verified stands in
/// an element of that class (the verse character's innermost element and
/// its ancestors up to the block). Nothing is decided from a class's name or
/// from how text looks.
struct SetApart<'a> {
    floated: &'a HashSet<String>,
    /// Blocks a note's caller leads to: (entry, block element). The reader
    /// reads them as notes (`epub::read_structural`).
    note_blocks: HashSet<(usize, usize)>,
    carrying: HashSet<&'a str>,
    count: HashMap<&'a str, usize>,
}

impl<'a> SetApart<'a> {
    fn measure(
        book: &'a SourceBook,
        roles: &LinkRoles,
        marks: &[EntryMarks],
        owners: &[Vec<u32>],
        placed: &[Placed],
    ) -> Self {
        let mut carrying: HashSet<&'a str> = HashSet::new();
        // Only verified verses measure what verse text is: a verse the
        // recogniser itself refused (for a reference read in, a possible
        // heading …) says nothing about where verse text stands.
        for p in placed.iter().filter(|p| p.verified) {
            let entry = &book.entries[p.entry];
            let mut last = epub_source::NO_ELEMENT;
            for &(from, to) in &p.spans {
                let end = to.min(entry.text.len());
                for (at, &owner) in owners[p.entry].iter().enumerate().take(end).skip(from) {
                    if owner == last
                        || owner == epub_source::NO_ELEMENT
                        || entry.text[at].is_whitespace()
                        || marks[p.entry].excluded_kind(at).is_some()
                    {
                        continue;
                    }
                    last = owner;
                    for e in entry.chain_to_block(owner as usize) {
                        for c in &entry.elements[e].classes {
                            carrying.insert(c.as_str());
                        }
                    }
                }
            }
        }
        let mut count: HashMap<&'a str, usize> = HashMap::new();
        for entry in &book.entries {
            for e in &entry.elements {
                for c in &e.classes {
                    *count.entry(c.as_str()).or_default() += 1;
                }
            }
        }
        let note_blocks = roles
            .links
            .iter()
            .filter(|l| l.role == LinkRole::NoteCaller)
            .filter_map(|l| l.target)
            .map(|(entry, element)| (entry, book.entries[entry].block_of(element)))
            .collect();
        SetApart {
            floated: &book.floated,
            note_blocks,
            carrying,
            count,
        }
    }

    /// Whether an element's classes (all of them) are apart by measurement.
    fn class_apart(&self, element: &epub_source::Element) -> bool {
        !element.classes.is_empty()
            && element.classes.iter().all(|c| {
                !self.carrying.contains(c.as_str())
                    && self.count.get(c.as_str()).copied().unwrap_or(0) >= 3
            })
    }

    /// Whether the character's innermost element is set apart, by the
    /// document's structure or by a measured class up to its block.
    fn holds(&self, book: &SourceBook, entry_index: usize, owner: u32) -> bool {
        if owner == epub_source::NO_ELEMENT {
            return false;
        }
        let entry = &book.entries[entry_index];
        let owner = owner as usize;
        entry.set_apart(owner, self.floated)
            || entry
                .ancestors_of(owner)
                .any(|a| self.note_blocks.contains(&(entry_index, a)))
            || entry
                .chain_to_block(owner)
                .into_iter()
                .any(|e| self.class_apart(&entry.elements[e]))
    }
}

/// A removal the recogniser reported from inside a verse's words that the
/// document grounds (Phase A1.1).
struct Removal {
    entry: usize,
    element: usize,
    /// Non-space characters of the verse's own words before it (after the
    /// earlier removals of the same verse are taken away).
    before: usize,
}

/// The pieces the recogniser reported setting apart from inside a verse
/// (`Apparatus::within_verse`), each found in the original: the one whole
/// element, in the verse's place, whose text is exactly the piece and that
/// the document sets apart (`SetApart::class_apart`). A piece with no such
/// element, or with more than one, is not grounded, and its verse keeps no
/// claim on it.
fn grounded_removals(
    apparatus: &[crate::scripture::structure::Apparatus],
    verse_places: &HashMap<(u8, u16, u16), (usize, usize, usize)>,
    marks: &[EntryMarks],
    owners: &[Vec<u32>],
    apart: &SetApart,
    book: &SourceBook,
) -> HashMap<(u8, u16, u16), Vec<Removal>> {
    // By verse, in the order they stood.
    type Pieces<'p> = HashMap<(u8, u16, u16), Vec<(u32, &'p str)>>;
    let mut pieces: Pieces = HashMap::new();
    for piece in apparatus {
        if let (Some(key), Some(at)) = (piece.verse, piece.within_verse) {
            pieces
                .entry(key)
                .or_default()
                .push((at, piece.text.as_str()));
        }
    }
    let mut out: HashMap<(u8, u16, u16), Vec<Removal>> = HashMap::new();
    for (key, mut list) in pieces {
        let Some(&(entry, from, to)) = verse_places.get(&key) else {
            continue;
        };
        list.sort_by_key(|(at, _)| *at);
        let m = &marks[entry];
        let e = &book.entries[entry];
        let end = to.min(m.nonspace_len());
        let mut taken = 0usize;
        let mut found = Vec::new();
        for (at, text) in list {
            let wanted: Vec<char> = text.chars().filter(|c| !c.is_whitespace()).collect();
            if wanted.is_empty() {
                continue;
            }
            let mut candidates = Vec::new();
            let mut k = from;
            while k + wanted.len() <= end {
                if (0..wanted.len()).all(|n| e.text[m.char_of(k + n)] == wanted[n]) {
                    let owner = owners[entry][m.char_of(k)];
                    if owner != epub_source::NO_ELEMENT {
                        let whole = e.chain_to_block(owner as usize).into_iter().find(|&i| {
                            let el = &e.elements[i];
                            m.nonspace_at(el.text_start) == k
                                && m.nonspace_at(el.text_end) == k + wanted.len()
                        });
                        if let Some(i) = whole {
                            if !e.elements[i].is_block_element()
                                && apart.class_apart(&e.elements[i])
                            {
                                candidates.push(i);
                            }
                        }
                    }
                }
                k += 1;
            }
            if candidates.len() != 1 {
                continue;
            }
            found.push(Removal {
                entry,
                element: candidates[0],
                before: (at as usize).saturating_sub(taken),
            });
            taken += wanted.len();
        }
        if !found.is_empty() {
            out.insert(key, found);
        }
    }
    out
}

/// Whether a traced verse's removed elements are exactly those the
/// recogniser reported, each after exactly the reported number of the
/// verse's characters. With no reported removal, no element may have been
/// skipped as removed.
fn removals_in_place(m: &EntryMarks, t: &provenance::Traced, reported: Option<&[Removal]>) -> bool {
    let reported = reported.unwrap_or(&[]);
    let mut seen: Vec<usize> = Vec::new();
    for &(at, kind) in &t.excluded {
        if kind != ExcludedKind::Apparatus {
            continue;
        }
        let Some(element) = m.removed[at] else {
            return false;
        };
        if seen.contains(&element) {
            continue;
        }
        let Some(r) = reported.iter().find(|r| r.element == element) else {
            return false;
        };
        let before = t.matched.iter().filter(|&&c| c < at).count();
        if before != r.before {
            return false;
        }
        seen.push(element);
    }
    seen.len() == reported.len()
}

/// Verses after which the original holds letters that no verse holds and
/// nothing in the document accounts for, before the next verse begins (or
/// the entry ends). Where a verse that could not be traced begins inside the
/// gap, only the part before its start is judged: that verse may hold the
/// rest.
fn unaccounted_text(
    book: &SourceBook,
    marks: &[EntryMarks],
    owners: &[Vec<u32>],
    apart: &SetApart,
    placed: &[Placed],
    unplaced: &[(usize, usize)],
) -> Vec<(u8, u16, u16)> {
    let mut out = Vec::new();
    for (i, p) in placed.iter().enumerate() {
        let entry = &book.entries[p.entry];
        let m = &marks[p.entry];
        let next = placed.get(i + 1).filter(|q| q.entry == p.entry);
        let (mut gap_end, gap_end_nonspace) = match next {
            Some(q) if q.start < p.end => continue,
            Some(q) => (q.chars.0, q.start),
            None => (entry.text.len(), m.nonspace_len()),
        };
        // A verse that could not be traced, beginning in the gap, may hold
        // what follows its start: the gap is judged only up to there.
        if let Some(&(_, at)) = unplaced
            .iter()
            .filter(|&&(e, at)| e == p.entry && at >= p.end && at < gap_end_nonspace)
            .min_by_key(|&&(_, at)| at)
        {
            gap_end = m.char_of(at);
        }
        let loose = (p.chars.1..gap_end).any(|at| {
            entry.text[at].is_alphabetic()
                && m.excluded_kind(at).is_none()
                && !apart.holds(book, p.entry, owners[p.entry][at])
        });
        if loose {
            out.push(p.key);
        }
    }
    out
}
