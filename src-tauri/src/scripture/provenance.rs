//! Exact provenance of recognised verses (Phase A1).
//!
//! A verse read from a document is not always one piece of it. A note's
//! caller can stand in the middle of a verse:
//!
//! ```text
//! E havendo Deus acabado … que tinha feito, <a href="#fn1">[1]</a> descançou …
//! ```
//!
//! The verse is then **two spans** of the original, with the caller's bytes
//! between them **excluded** (kept and reported, never silently dropped).
//! [`trace`] finds those spans for one verse's text. It works on the
//! entry's characters with their exact byte ranges (`epub_source::Entry`),
//! and only ever skips characters it can name: white space, a note's caller
//! the document links to a note, or a note sign. Anything else between two
//! words means the verse cannot be traced, and the verse is not verified.
//!
//! The verse text is the spans' text joined by one rule ([`quote`]):
//! white space inside a span becomes one space, and two spans are joined by
//! one space when the original had white space between them, by nothing
//! otherwise. No other character is ever placed between spans.

use serde::Serialize;

use crate::library::adapters::epub_source::{Entry, LinkRole};

/// The note signs the recogniser takes out of a verse when a note explains
/// them (`structure::NOTE_SIGNS`).
const NOTE_SIGNS: [char; 5] = ['†', '‡', '§', '¶', '*'];

/// Which file, and which part of it, a verse comes from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileIdentity {
    /// Which of the Bible's documents (0 = the first).
    pub document: u16,
    /// SHA-256 of the whole file (as `DocumentFile::hash`).
    pub file_sha256: String,
    /// The archive entry ("OEBPS/text/ch01.xhtml").
    pub entry: String,
    /// SHA-256 of that entry's bytes.
    pub entry_sha256: String,
}

/// A run of the original: characters of the entry's text and the bytes of
/// the entry they come from (end exclusive).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceRange {
    pub char_start: u32,
    pub char_end: u32,
    pub byte_start: u32,
    pub byte_end: u32,
}

/// Why a piece of the original inside a verse is not part of its text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ExcludedKind {
    /// A link that leads to a note (`LinkRole::NoteCaller`).
    NoteCaller,
    /// A link the document declares a caller but that leads to no note
    /// (`LinkRole::Conflict`): excluded, and the verse is not verified.
    ConflictingCaller,
    /// A note sign (†, ‡, §, ¶, *) the recogniser took out.
    NoteSign,
}

/// Original text inside a verse that is not the verse's text, kept with its
/// place.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExcludedRange {
    pub kind: ExcludedKind,
    /// The visible characters excluded.
    pub range: SourceRange,
    /// The whole element (`<a …>[1]</a>`), for a link.
    pub element_bytes: Option<(u32, u32)>,
    /// The visible text excluded ("[1]").
    pub text: String,
}

/// Whether, and how, a verse was traced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TraceStatus {
    /// Every character of the verse is in the spans, and [`quote`] gives the
    /// verse's text exactly.
    Exact,
    /// Every non-space character is in the spans in order, but the verse's
    /// spacing differs from the spans' (a space more or less): its letters
    /// are the document's, its spacing the recogniser's.
    SpacingDiffers,
    /// The verse could not be found as spans of the original.
    Untraced,
}

/// A verse's id in the document ("Gen8-1") compared with the verse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum IdCheck {
    /// No id at the verse's start encodes a chapter and verse.
    None,
    Agrees,
    /// The id at the verse's start names another chapter and verse.
    Contradicts {
        chapter: u16,
        verse: u16,
    },
}

/// Where one verse comes from, exactly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerseProvenance {
    /// (book ordinal, chapter, verse).
    pub reference: (u8, u16, u16),
    pub file: FileIdentity,
    /// The verse's text, as the recogniser gave it.
    pub text: String,
    /// The original's runs that make the verse, in order.
    pub spans: Vec<SourceRange>,
    /// Original text between the spans that is not the verse's.
    pub excluded: Vec<ExcludedRange>,
    /// The spans' text joined by [`quote`]'s rule.
    pub quote: String,
    pub status: TraceStatus,
    pub id_check: IdCheck,
    /// Why the verse was accepted or refused, in words (extraction, links,
    /// tracing, verification).
    pub reasons: Vec<String>,
}

/// What every character of an entry is, for tracing: part of a link with a
/// role, or plain.
pub struct EntryMarks<'a> {
    pub entry: &'a Entry,
    /// For each character: the role and element of the link it is in, if any.
    pub link: Vec<Option<(LinkRole, usize)>>,
    /// Positions of the non-white-space characters, in order.
    nonspace: Vec<usize>,
}

impl<'a> EntryMarks<'a> {
    pub fn new(entry: &'a Entry, links: &[(LinkRole, usize)]) -> Self {
        let mut link = vec![None; entry.text.len()];
        for &(role, element) in links {
            let e = &entry.elements[element];
            for slot in link
                .iter_mut()
                .take(e.text_end.min(entry.text.len()))
                .skip(e.text_start)
            {
                *slot = Some((role, element));
            }
        }
        let nonspace = entry
            .text
            .iter()
            .enumerate()
            .filter(|(_, c)| !c.is_whitespace())
            .map(|(i, _)| i)
            .collect();
        Self {
            entry,
            link,
            nonspace,
        }
    }

    /// Index in `nonspace` of the first non-space character at or after the
    /// character `at`.
    pub fn nonspace_at(&self, at: usize) -> usize {
        self.nonspace.partition_point(|&i| i < at)
    }

    pub fn nonspace_len(&self) -> usize {
        self.nonspace.len()
    }

    pub fn char_of(&self, k: usize) -> usize {
        self.nonspace[k]
    }

    fn excluded_kind(&self, at: usize) -> Option<ExcludedKind> {
        match self.link[at] {
            Some((LinkRole::NoteCaller, _)) => Some(ExcludedKind::NoteCaller),
            Some((LinkRole::Conflict, _)) => Some(ExcludedKind::ConflictingCaller),
            _ => None,
        }
    }
}

/// Where a segment's text lies in its entry: the non-space characters of
/// each segment found in order (a segment is the importer's reading of a
/// block, white space collapsed). Returns, for each segment, the range of
/// `nonspace` indices it covers, or None when it is not found (then nothing
/// in it can be traced).
pub fn locate_segments(marks: &EntryMarks, segments: &[&str]) -> Vec<Option<(usize, usize)>> {
    let mut out = Vec::with_capacity(segments.len());
    let stream: Vec<char> = marks
        .nonspace
        .iter()
        .map(|&i| marks.entry.text[i])
        .collect();
    let mut cursor = 0usize;
    for segment in segments {
        let wanted: Vec<char> = segment.chars().filter(|c| !c.is_whitespace()).collect();
        if wanted.is_empty() {
            out.push(None);
            continue;
        }
        let found = find(&stream, &wanted, cursor);
        match found {
            Some(at) => {
                out.push(Some((at, at + wanted.len())));
                cursor = at + wanted.len();
            }
            None => out.push(None),
        }
    }
    out
}

fn find(hay: &[char], needle: &[char], from: usize) -> Option<usize> {
    if needle.is_empty() || hay.len() < needle.len() {
        return None;
    }
    let first = needle[0];
    let last = hay.len() - needle.len();
    let mut i = from;
    while i <= last {
        if hay[i] == first && hay[i..i + needle.len()] == *needle {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// A verse traced: the characters of the entry that make it, and those
/// skipped as excluded, in order.
pub struct Traced {
    pub matched: Vec<usize>,
    pub excluded: Vec<(usize, ExcludedKind)>,
    /// Index in `nonspace` just after the verse.
    pub end: usize,
}

/// Finds a verse's text in an entry, between the `nonspace` indices `from`
/// and `to`. Characters of a note's caller and note signs are skipped (and
/// reported); a caller is never part of a verse. Any other character that
/// is not the verse's next character ends the attempt there. The first
/// start that traces the whole verse is taken.
pub fn trace(marks: &EntryMarks, text: &str, from: usize, to: usize) -> Option<Traced> {
    let wanted: Vec<char> = text.chars().filter(|c| !c.is_whitespace()).collect();
    if wanted.is_empty() {
        return None;
    }
    let to = to.min(marks.nonspace.len());
    let mut start = from;
    while start < to {
        let at = marks.nonspace[start];
        if marks.entry.text[at] == wanted[0] && marks.excluded_kind(at).is_none() {
            if let Some(t) = attempt(marks, &wanted, start, to) {
                return Some(t);
            }
        }
        start += 1;
    }
    None
}

fn attempt(marks: &EntryMarks, wanted: &[char], start: usize, to: usize) -> Option<Traced> {
    let mut matched = Vec::with_capacity(wanted.len());
    let mut excluded = Vec::new();
    let mut k = start;
    let mut i = 0usize;
    while i < wanted.len() {
        if k >= to {
            return None;
        }
        let at = marks.nonspace[k];
        let c = marks.entry.text[at];
        if let Some(kind) = marks.excluded_kind(at) {
            excluded.push((at, kind));
        } else if c == wanted[i] {
            matched.push(at);
            i += 1;
        } else if NOTE_SIGNS.contains(&c) {
            excluded.push((at, ExcludedKind::NoteSign));
        } else {
            return None;
        }
        k += 1;
    }
    Some(Traced {
        matched,
        excluded,
        end: k,
    })
}

/// The spans of a traced verse: runs of matched characters with nothing but
/// white space between them.
pub fn spans(marks: &EntryMarks, traced: &Traced) -> Vec<SourceRange> {
    let entry = marks.entry;
    let mut out: Vec<SourceRange> = Vec::new();
    let mut run: Option<(usize, usize)> = None;
    for &at in &traced.matched {
        run = match run {
            Some((s, e)) if entry.text[e + 1..at].iter().all(|c| c.is_whitespace()) => {
                Some((s, at))
            }
            Some((s, e)) => {
                out.push(range(entry, s, e + 1));
                Some((at, at))
            }
            None => Some((at, at)),
        };
    }
    if let Some((s, e)) = run {
        out.push(range(entry, s, e + 1));
    }
    out
}

/// The excluded ranges of a traced verse, one per link (or sign), with the
/// link's whole element.
pub fn excluded_ranges(marks: &EntryMarks, traced: &Traced) -> Vec<ExcludedRange> {
    let entry = marks.entry;
    let mut out: Vec<ExcludedRange> = Vec::new();
    let mut i = 0usize;
    while i < traced.excluded.len() {
        let (at, kind) = traced.excluded[i];
        let element = marks.link[at].map(|(_, e)| e);
        let mut j = i + 1;
        while j < traced.excluded.len()
            && traced.excluded[j].1 == kind
            && element.is_some()
            && marks.link[traced.excluded[j].0].map(|(_, e)| e) == element
        {
            j += 1;
        }
        let end = traced.excluded[j - 1].0;
        let (char_start, char_end) = match element {
            Some(e) => (
                entry.elements[e].text_start.min(at),
                entry.elements[e].text_end.max(end + 1),
            ),
            None => (at, end + 1),
        };
        out.push(ExcludedRange {
            kind,
            range: range(entry, char_start, char_end),
            element_bytes: element.map(|e| {
                (
                    entry.elements[e].byte_start as u32,
                    entry.elements[e].byte_end as u32,
                )
            }),
            text: entry.slice(char_start, char_end),
        });
        i = j;
    }
    out
}

fn range(entry: &Entry, start: usize, end: usize) -> SourceRange {
    let end = end.max(start + 1).min(entry.text.len());
    SourceRange {
        char_start: start as u32,
        char_end: end as u32,
        byte_start: entry.bytes[start].0,
        byte_end: entry.bytes[end - 1].1,
    }
}

/// The text of spans joined by the one rule: white space inside a span is
/// one space; between two spans, one space where the original has white
/// space outside excluded text, nothing where it has none.
pub fn quote(marks: &EntryMarks, spans: &[SourceRange]) -> String {
    let entry = marks.entry;
    let mut out = String::new();
    for (n, s) in spans.iter().enumerate() {
        if n > 0 {
            let before = spans[n - 1].char_end as usize;
            let gap_space = (before..s.char_start as usize)
                .any(|i| entry.text[i].is_whitespace() && marks.excluded_kind(i).is_none());
            if gap_space {
                out.push(' ');
            }
        }
        out.push_str(&collapse(
            &entry.slice(s.char_start as usize, s.char_end as usize),
        ));
    }
    out
}

/// White space runs as one space, none at either end.
pub fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Reads back a span's exact original bytes from the entry (for audits:
/// the bytes, not the decoded text).
pub fn original_bytes<'b>(entry: &'b Entry, span: &SourceRange) -> &'b str {
    entry
        .source
        .get(span.byte_start as usize..span.byte_end as usize)
        .unwrap_or("")
}
