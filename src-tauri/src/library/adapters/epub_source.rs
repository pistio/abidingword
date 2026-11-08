//! The structure an EPUB declares, kept with exact positions (Phase A1).
//!
//! This is a second, independent reading of a book's XHTML. It is not the
//! reading the library and the verse recogniser use (`epub::blocks`). It
//! keeps what that reading reduces away:
//!
//! * every character of text, with the exact byte range it comes from in
//!   its archive entry (an entity such as `&amp;` is one character from
//!   five bytes);
//! * every element, with its name, id, classes, `epub:type` and `role`
//!   values, its parent, and the character and byte ranges it covers;
//! * every link, with the element it actually leads to.
//!
//! From this, [`classify_links`] decides the role of each link by the
//! evidence of where it leads, not by what its visible text looks like. A
//! visible "[1]" is a note's caller only when the document shows that it
//! leads to a note. It is never assumed to be a verse number either.
//!
//! Nothing here changes text. Callers are reported with their positions;
//! the recogniser's reading marks them (`epub::read_structural`), and
//! [`crate::scripture::provenance`] traces each verse back to these
//! positions.

use std::collections::HashMap;
use std::io::Cursor;

use quick_xml::events::{BytesStart, Event};

use super::epub::{
    internal_link_target, local_name, normalise, parse_package, percent_decode, read_entry,
    tolerant_reader, unescape,
};

/// The block elements: a note's block, a verse's paragraph.
const BLOCKS: &[&str] = &[
    "p",
    "div",
    "li",
    "aside",
    "section",
    "blockquote",
    "dd",
    "dt",
    "td",
    "tr",
    "pre",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "body",
];

/// The `epub:type`, `role` and class values that declare an element holds
/// the document's notes.
const NOTE_CONTAINERS: &[&str] = &[
    "footnote",
    "footnotes",
    "endnote",
    "endnotes",
    "rearnote",
    "rearnotes",
    "doc-footnote",
    "doc-endnote",
    "doc-endnotes",
    "doc-footnotes",
];

/// The `epub:type`, `role` and class values that declare a link a note's
/// caller.
const NOTE_REFS: &[&str] = &["noteref", "doc-noteref", "footnote-ref", "footnoteref"];
/// Declared back-links from a note to its caller.
const BACK_LINKS: &[&str] = &["doc-backlink", "backlink", "footnote-back", "footnoteback"];

/// No element holds the character (`Entry::owners`).
pub const NO_ELEMENT: u32 = u32::MAX;

/// One element of an entry, as written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    /// Lower-case local name ("p", "a", "span").
    pub name: String,
    pub id: Option<String>,
    pub classes: Vec<String>,
    /// `epub:type` and `role` values, lower-case.
    pub semantics: Vec<String>,
    pub href: Option<String>,
    pub parent: Option<usize>,
    /// Characters of the entry's text it covers: `text_start..text_end`.
    pub text_start: usize,
    pub text_end: usize,
    /// Bytes of the entry it covers, from its start tag to its end tag.
    pub byte_start: usize,
    pub byte_end: usize,
    /// Inside `head`, `script` or `style`: its text is not the book's text.
    pub hidden: bool,
    /// Written as an empty-element tag (`<a id="x"/>`).
    pub empty_tag: bool,
}

impl Element {
    fn has(&self, values: &[&str]) -> bool {
        self.semantics
            .iter()
            .chain(self.classes.iter())
            .any(|v| values.contains(&v.to_ascii_lowercase().as_str()))
    }

    /// Whether the document declares this element a note's caller.
    pub fn declared_noteref(&self) -> bool {
        self.has(NOTE_REFS)
    }

    fn is_block(&self) -> bool {
        BLOCKS.contains(&self.name.as_str())
    }

    /// Whether the element is a block (`p`, `div`, `li` …).
    pub fn is_block_element(&self) -> bool {
        self.is_block()
    }

    fn is_heading(&self) -> bool {
        matches!(self.name.as_str(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6")
    }
}

/// One spine document of the book, with its text and structure.
#[derive(Debug, Clone)]
pub struct Entry {
    /// The archive path ("OEBPS/text/ch01.xhtml").
    pub path: String,
    /// SHA-256 of the entry's bytes, as stored in the archive.
    pub sha256: String,
    /// The entry's length in bytes, as stored. (The bytes themselves are
    /// not kept, Phase A1.2: every byte range points into the stored entry,
    /// identified by `sha256`.)
    pub source_len: usize,
    /// The visible text in document order, entities decoded, white space as
    /// written. Text inside `head`, `script` and `style` is not included.
    pub text: Vec<char>,
    /// For each character of `text`, the bytes of `source` it comes from
    /// (`byte_range`), kept as a start and a length (Phase A1.2).
    bytes: ByteRanges,
    pub elements: Vec<Element>,
    /// Element by id (the first element with that id).
    pub ids: HashMap<String, usize>,
}

/// A value for every character of an entry, kept as runs of equal values
/// (Phase A1.2): a link's mark or a text node's innermost element is the
/// same over many characters. `get` gives exactly what the dense table
/// it was made from holds at that position.
#[derive(Debug, Clone, Default)]
pub struct Runs {
    starts: Vec<u32>,
    values: Vec<u32>,
    len: usize,
}

impl Runs {
    pub fn from_dense(dense: &[u32]) -> Self {
        let mut runs = Runs {
            len: dense.len(),
            ..Default::default()
        };
        for (i, &v) in dense.iter().enumerate() {
            if runs.values.last() != Some(&v) {
                runs.starts.push(i as u32);
                runs.values.push(v);
            }
        }
        runs.starts.shrink_to_fit();
        runs.values.shrink_to_fit();
        runs
    }

    /// The value at position `at` (`at` < the table's length).
    pub fn get(&self, at: usize) -> u32 {
        debug_assert!(at < self.len);
        let run = self.starts.partition_point(|&s| s as usize <= at);
        self.values[run - 1]
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// Byte ranges of an entry's characters: a `u32` start and a `u8` length
/// each, and the rare range longer than 255 bytes (a long `<br …/>` tag)
/// apart. Five bytes a character instead of eight.
#[derive(Debug, Clone, Default)]
pub struct ByteRanges {
    starts: Vec<u32>,
    lengths: Vec<u8>,
    long: HashMap<u32, u32>,
}

impl ByteRanges {
    fn push(&mut self, start: u32, end: u32) {
        let index = self.starts.len() as u32;
        self.starts.push(start);
        let length = end.saturating_sub(start);
        if length > u32::from(u8::MAX) {
            self.lengths.push(u8::MAX);
            self.long.insert(index, end);
        } else {
            self.lengths.push(length as u8);
        }
    }

    fn get(&self, i: usize) -> (u32, u32) {
        let start = self.starts[i];
        let length = self.lengths[i];
        if length == u8::MAX {
            if let Some(&end) = self.long.get(&(i as u32)) {
                return (start, end);
            }
        }
        (start, start + u32::from(length))
    }

    fn len(&self) -> usize {
        self.starts.len()
    }

    fn shrink_to_fit(&mut self) {
        self.starts.shrink_to_fit();
        self.lengths.shrink_to_fit();
        self.long.shrink_to_fit();
    }
}

impl Entry {
    /// The bytes of `source` the character `i` of `text` comes from.
    pub fn byte_range(&self, i: usize) -> (u32, u32) {
        self.bytes.get(i)
    }

    /// The number of characters with byte ranges (as `text`).
    pub fn byte_ranges(&self) -> usize {
        self.bytes.len()
    }
    /// The decoded text of a range of characters.
    pub fn slice(&self, from: usize, to: usize) -> String {
        self.text[from.min(self.text.len())..to.min(self.text.len())]
            .iter()
            .collect()
    }

    /// For each character of `text`, the innermost element that holds it
    /// (`NO_ELEMENT` when none does).
    pub fn owners(&self) -> Runs {
        let mut out = vec![NO_ELEMENT; self.text.len()];
        // Elements are in document order, parents before their children, so
        // a child's range overwrites its parent's.
        for (i, e) in self.elements.iter().enumerate() {
            let end = e.text_end.min(out.len());
            if e.text_start < end {
                out[e.text_start..end].fill(i as u32);
            }
        }
        Runs::from_dense(&out)
    }

    /// `element` and its ancestors up to and including the nearest block.
    pub fn chain_to_block(&self, element: usize) -> Vec<usize> {
        let mut out = Vec::new();
        for e in self.ancestors(element) {
            out.push(e);
            if self.elements[e].is_block() {
                break;
            }
        }
        out
    }

    /// Whether the document's own structure sets `element` apart from the
    /// text: it is, or stands inside, a heading (`h1`–`h6`), a declared
    /// note container, navigation (`nav`), an `aside`, hidden content, or a
    /// block whose class the stylesheet floats beside the text.
    pub fn set_apart(&self, element: usize, floated: &std::collections::HashSet<String>) -> bool {
        self.ancestors(element).any(|a| {
            let e = &self.elements[a];
            e.hidden
                || e.is_heading()
                || e.has(NOTE_CONTAINERS)
                || matches!(e.name.as_str(), "nav" | "aside")
                || (e.is_block() && e.classes.iter().any(|c| floated.contains(c)))
        })
    }

    /// `element` and its ancestors, nearest first.
    pub fn ancestors_of(&self, element: usize) -> impl Iterator<Item = usize> + '_ {
        self.ancestors(element)
    }

    fn ancestors(&self, element: usize) -> impl Iterator<Item = usize> + '_ {
        std::iter::successors(Some(element), move |e| self.elements[*e].parent)
    }

    /// The nearest block element that holds `element` (itself if a block).
    pub fn block_of(&self, element: usize) -> usize {
        self.ancestors(element)
            .find(|e| self.elements[*e].is_block())
            .unwrap_or(element)
    }

    fn in_note_container(&self, element: usize) -> bool {
        self.ancestors(element)
            .any(|e| self.elements[e].has(NOTE_CONTAINERS))
    }
}

/// A book's spine documents, read for their structure.
#[derive(Debug, Clone, Default)]
pub struct SourceBook {
    pub entries: Vec<Entry>,
    /// Entry index by archive path.
    pub by_path: HashMap<String, usize>,
    /// Classes the book's stylesheets float beside the text or position
    /// out of its flow (`epub::floated_classes`): apparatus by layout.
    pub floated: std::collections::HashSet<String>,
}

/// Reads every spine document of an EPUB for its structure. None when the
/// file is not an EPUB the importer could read either.
pub fn read(bytes: &[u8]) -> Option<SourceBook> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).ok()?;
    let container = read_entry(&mut archive, "META-INF/container.xml")?;
    let opf_path = super::epub::attribute_of(&container, "rootfile", "full-path")?;
    let opf = read_entry(&mut archive, &opf_path)?;
    let base = opf_path
        .rsplit_once('/')
        .map(|(dir, _)| format!("{dir}/"))
        .unwrap_or_default();
    let package = parse_package(&opf);
    let mut book = SourceBook::default();
    for item in package.manifest.values() {
        if item.media_type == "text/css" {
            let path = normalise(&format!("{base}{}", item.href));
            if let Some(css) = read_entry(&mut archive, &path) {
                book.floated.extend(super::epub::floated_classes(&css));
            }
        }
    }
    for idref in &package.spine {
        let Some(item) = package.manifest.get(idref) else {
            continue;
        };
        let path = normalise(&format!("{base}{}", item.href));
        if book.by_path.contains_key(&path) {
            continue;
        }
        let Some(source) = read_entry(&mut archive, &path) else {
            continue;
        };
        let entry = read_entry_structure(path.clone(), source);
        book.by_path.insert(path, book.entries.len());
        book.entries.push(entry);
    }
    Some(book)
}

/// Reads one XHTML document's text and elements with their positions.
pub fn read_entry_structure(path: String, source: String) -> Entry {
    use sha2::{Digest, Sha256};
    let sha256 = Sha256::digest(source.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let mut text = Vec::new();
    let mut bytes = ByteRanges::default();
    let mut elements: Vec<Element> = Vec::new();
    let mut ids = HashMap::new();
    let mut stack: Vec<usize> = Vec::new();
    // A byte-order mark is not markup: the reader is given what follows it,
    // and every position is counted from the entry's first byte, so the
    // byte ranges index the file as stored (Phase A1.1).
    let skip = if source.starts_with('\u{feff}') {
        '\u{feff}'.len_utf8()
    } else {
        0
    };
    let mut reader = tolerant_reader(&source[skip..]);
    loop {
        let at = reader.buffer_position() as usize + skip;
        let event = reader.read_event();
        let end = reader.buffer_position() as usize + skip;
        let hidden = stack.last().is_some_and(|e| elements[*e].hidden);
        match event {
            Ok(Event::Start(e)) => {
                let index = elements.len();
                let element = element_of(&e, stack.last().copied(), text.len(), at, hidden);
                if let Some(id) = &element.id {
                    ids.entry(id.clone()).or_insert(index);
                }
                elements.push(element);
                stack.push(index);
            }
            Ok(Event::Empty(e)) => {
                let index = elements.len();
                let mut element = element_of(&e, stack.last().copied(), text.len(), at, hidden);
                element.byte_end = end;
                element.text_end = text.len();
                element.empty_tag = true;
                if let Some(id) = &element.id {
                    ids.entry(id.clone()).or_insert(index);
                }
                let is_br = element.name == "br";
                elements.push(element);
                if is_br && !hidden {
                    // A line break is white space between words.
                    text.push('\n');
                    bytes.push(at as u32, end as u32);
                }
            }
            Ok(Event::End(e)) => {
                let name = local_name(e.name().as_ref()).to_ascii_lowercase();
                // End tags need not match (tolerant reading): close up to the
                // nearest open element of this name, as a browser does.
                if let Some(pos) = stack.iter().rposition(|i| elements[*i].name == name) {
                    while stack.len() > pos {
                        let index = stack.pop().expect("non-empty");
                        elements[index].text_end = text.len();
                        elements[index].byte_end = end;
                    }
                }
            }
            Ok(Event::Text(t)) if !hidden => {
                // Text comes without its character references (those arrive
                // as `GeneralRef`), so each character is its own bytes.
                let raw: &str = &t;
                let mut offset = at;
                for c in raw.chars() {
                    let len = c.len_utf8();
                    text.push(c);
                    bytes.push(offset as u32, (offset + len) as u32);
                    offset += len;
                }
            }
            Ok(Event::CData(t)) if !hidden => {
                let raw: &str = &t;
                // `<![CDATA[` is nine bytes.
                let mut offset = at + 9;
                for c in raw.chars() {
                    let len = c.len_utf8();
                    text.push(c);
                    bytes.push(offset as u32, (offset + len) as u32);
                    offset += len;
                }
            }
            Ok(Event::GeneralRef(r)) if !hidden => {
                let name: &str = &r;
                for c in unescape(&format!("&{name};")).chars() {
                    text.push(c);
                    bytes.push(at as u32, end as u32);
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    // Elements never closed run to the end.
    for index in stack {
        elements[index].text_end = text.len();
        elements[index].byte_end = source.len();
    }
    // A whole book's entries are held at once: no growth slack (Phase A1.2).
    text.shrink_to_fit();
    bytes.shrink_to_fit();
    elements.shrink_to_fit();
    for e in &mut elements {
        e.classes.shrink_to_fit();
        e.semantics.shrink_to_fit();
    }
    ids.shrink_to_fit();
    Entry {
        path,
        sha256,
        source_len: source.len(),
        text,
        bytes,
        elements,
        ids,
    }
}

fn element_of(
    e: &BytesStart,
    parent: Option<usize>,
    text_at: usize,
    byte_at: usize,
    parent_hidden: bool,
) -> Element {
    let name = local_name(e.name().as_ref()).to_ascii_lowercase();
    let mut id = None;
    let mut classes = Vec::new();
    let mut semantics = Vec::new();
    let mut href = None;
    for a in e.attributes().with_checks(false).flatten() {
        let key = local_name(a.key.as_ref()).to_ascii_lowercase();
        let value = unescape(&a.value);
        match key.as_str() {
            "id" => id = Some(percent_decode(&value)),
            "name" if name == "a" && id.is_none() => id = Some(percent_decode(&value)),
            "class" => classes.extend(value.split_whitespace().map(str::to_string)),
            "type" | "role" => {
                semantics.extend(value.split_whitespace().map(|v| v.to_ascii_lowercase()))
            }
            "href" if name == "a" => href = Some(value),
            _ => {}
        }
    }
    let hidden = parent_hidden || matches!(name.as_str(), "head" | "script" | "style");
    Element {
        name,
        id,
        classes,
        semantics,
        href,
        parent,
        text_start: text_at,
        text_end: text_at,
        byte_start: byte_at,
        byte_end: byte_at,
        hidden,
        empty_tag: false,
    }
}

/// What a link is, by where it leads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LinkRole {
    /// It leads to a note: its visible text is the note's caller, not the
    /// verse's words.
    NoteCaller,
    /// It leads somewhere that is not a note (a heading, a contents page, a
    /// glossary entry, another verse): its visible text is kept as written.
    Navigation,
    /// The document declares it a note's caller, but it does not lead to a
    /// note (no such target, or a target that is no note). Its text is
    /// treated as a caller, and nothing next to it is verified.
    Conflict,
    /// It looks like a caller ("[3]", "a", "*"), but where it leads cannot be
    /// found. It might be a caller or a verse number: nothing next to it is
    /// verified, and its text is kept as written.
    Ambiguous,
}

/// The evidence a link's role was decided on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LinkEvidence {
    /// Its target is inside an element the document declares holds notes
    /// (`epub:type="footnote"`, `role="doc-endnotes"`, `class="footnote"` …).
    TargetInNotes,
    /// The block it leads to links back to it (the usual footnote pattern:
    /// caller → note, note → caller).
    LinksBack,
    /// It and a link in its target's block lead to each other, but nothing
    /// shows which of the two is the note: the role is not known.
    PairUndirected,
    /// It leads to a heading.
    TargetIsHeading,
    /// It leads to an element that is neither a note nor a heading.
    TargetIsText,
    /// Its target is not in the book (no such entry or id).
    NoTarget,
    /// It leads outside the book (a web address, mail).
    External,
    /// It has no destination at all (`href=""`).
    Empty,
    /// It stands inside an element the document declares holds notes, or
    /// is declared a note's back-link (`role="doc-backlink"`): it belongs
    /// to the apparatus, whatever it leads to.
    InsideNotes,
}

/// One link with its role, its evidence and its place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifiedLink {
    pub entry: usize,
    /// The link's element in its entry.
    pub element: usize,
    /// Its position among the entry's links (`<a href>` elements, in
    /// document order), as the importer's reading counts them.
    pub ordinal: usize,
    pub role: LinkRole,
    pub evidence: LinkEvidence,
    pub declared: bool,
    /// Its visible text, trimmed.
    pub label: String,
    /// Where it leads: (entry, element), when found.
    pub target: Option<(usize, usize)>,
}

/// Every link of the book, with its role.
#[derive(Debug, Clone, Default)]
pub struct LinkRoles {
    pub links: Vec<ClassifiedLink>,
    /// Link index by (entry path, ordinal).
    by_place: HashMap<(String, usize), usize>,
}

impl LinkRoles {
    /// The role of the `ordinal`-th link of the entry at `path`.
    pub fn role(&self, path: &str, ordinal: usize) -> Option<LinkRole> {
        self.by_place
            .get(&(path.to_string(), ordinal))
            .map(|i| self.links[*i].role)
    }

    /// The ids, in the entry at `path`, of the elements that note callers
    /// lead to: the blocks that hold them are the book's notes.
    pub fn note_target_ids(
        &self,
        book: &SourceBook,
        path: &str,
    ) -> std::collections::HashSet<String> {
        let Some(&entry) = book.by_path.get(path) else {
            return Default::default();
        };
        self.links
            .iter()
            .filter(|l| l.role == LinkRole::NoteCaller)
            .filter_map(|l| l.target)
            .filter(|(e, _)| *e == entry)
            .filter_map(|(e, element)| book.entries[e].elements[element].id.clone())
            .collect()
    }

    pub fn counts(&self) -> HashMap<LinkRole, usize> {
        let mut out = HashMap::new();
        for l in &self.links {
            *out.entry(l.role).or_insert(0) += 1;
        }
        out
    }
}

/// Whether a link's visible text has the shape of a note's caller or a
/// verse number: digits, one to three letters or signs, optionally in
/// brackets. Words are never caller-shaped.
pub fn caller_shaped(label: &str) -> bool {
    let inner = label
        .trim()
        .trim_start_matches(['[', '(', '{', '<'])
        .trim_end_matches([']', ')', '}', '>'])
        .trim();
    let n = inner.chars().count();
    if n == 0 {
        return false;
    }
    let digits = inner.chars().all(|c| c.is_ascii_digit());
    let letters = inner.chars().all(char::is_alphabetic);
    let signs = inner
        .chars()
        .all(|c| !c.is_alphanumeric() && !c.is_whitespace());
    (digits && n <= 4) || (letters && n <= 3) || (signs && n <= 3)
}

/// Decides the role of every link of the book by where it leads.
pub fn classify_links(book: &SourceBook) -> LinkRoles {
    let mut roles = LinkRoles::default();
    for (entry_index, entry) in book.entries.iter().enumerate() {
        let dir = entry.path.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
        let mut ordinal = 0usize;
        for (element_index, element) in entry.elements.iter().enumerate() {
            let Some(href) = element.href.as_deref() else {
                continue;
            };
            if element.name != "a" {
                continue;
            }
            // The importer's reading counts every `<a href>` start tag, never
            // an empty-element one (`<a href="#x"/>`), in document order.
            if element.empty_tag {
                continue;
            }
            let this = ordinal;
            ordinal += 1;
            let label: String = entry
                .slice(element.text_start, element.text_end)
                .trim()
                .to_string();
            let declared = element.declared_noteref();
            let (mut evidence, target) = evidence_for(book, entry_index, element_index, dir, href);
            let in_apparatus = element.has(BACK_LINKS)
                || entry
                    .ancestors(element_index)
                    .any(|a| entry.elements[a].has(NOTE_CONTAINERS));
            if in_apparatus
                && !matches!(
                    evidence,
                    LinkEvidence::TargetInNotes | LinkEvidence::LinksBack
                )
            {
                evidence = LinkEvidence::InsideNotes;
            }
            let role = match evidence {
                LinkEvidence::TargetInNotes | LinkEvidence::LinksBack => LinkRole::NoteCaller,
                LinkEvidence::InsideNotes => LinkRole::Navigation,
                _ if declared => LinkRole::Conflict,
                LinkEvidence::NoTarget | LinkEvidence::Empty | LinkEvidence::PairUndirected
                    if caller_shaped(&label) =>
                {
                    LinkRole::Ambiguous
                }
                _ => LinkRole::Navigation,
            };
            roles
                .by_place
                .insert((entry.path.clone(), this), roles.links.len());
            roles.links.push(ClassifiedLink {
                entry: entry_index,
                element: element_index,
                ordinal: this,
                role,
                evidence,
                declared,
                label,
                target,
            });
        }
    }
    roles
}

fn evidence_for(
    book: &SourceBook,
    entry_index: usize,
    element_index: usize,
    dir: &str,
    href: &str,
) -> (LinkEvidence, Option<(usize, usize)>) {
    let entry = &book.entries[entry_index];
    let href = href.trim();
    if href.is_empty() {
        return (LinkEvidence::Empty, None);
    }
    let lower = href.to_ascii_lowercase();
    if lower.contains("://") || lower.starts_with("mailto:") || lower.starts_with("javascript:") {
        return (LinkEvidence::External, None);
    }
    let Some((path, fragment)) = internal_link_target(dir, &entry.path, href) else {
        return (LinkEvidence::External, None);
    };
    let Some(&target_entry) = book.by_path.get(&path) else {
        return (LinkEvidence::NoTarget, None);
    };
    let t = &book.entries[target_entry];
    let target = match fragment.as_deref() {
        Some(f) => match t.ids.get(f) {
            Some(&e) => e,
            None => return (LinkEvidence::NoTarget, None),
        },
        // A whole document: a chapter, a contents page, never one note.
        None => return (LinkEvidence::TargetIsText, None),
    };
    let found = Some((target_entry, target));
    let block = t.block_of(target);
    if t.elements[block].is_heading() || t.elements[target].is_heading() {
        return (LinkEvidence::TargetIsHeading, found);
    }
    if t.in_note_container(target) {
        return (LinkEvidence::TargetInNotes, found);
    }
    if let Some(back) = links_back(book, (entry_index, element_index), (target_entry, block)) {
        // A pair of links leading to each other: which side is the note? A
        // note's block begins with its link back (its label), or failing
        // that ends with it; a caller stands inside running text.
        let caller_block = entry.block_of(element_index);
        let note_begins = at_edge(t, block, back, Edge::Start);
        let caller_begins = at_edge(entry, caller_block, element_index, Edge::Start);
        let note_ends = at_edge(t, block, back, Edge::End);
        let caller_ends = at_edge(entry, caller_block, element_index, Edge::End);
        if note_begins && !caller_begins {
            return (LinkEvidence::LinksBack, found);
        }
        if !note_begins && !caller_begins && note_ends && !caller_ends {
            return (LinkEvidence::LinksBack, found);
        }
        if note_begins || note_ends {
            return (LinkEvidence::PairUndirected, found);
        }
    }
    (LinkEvidence::TargetIsText, found)
}

#[derive(Clone, Copy)]
enum Edge {
    Start,
    End,
}

/// Whether a link stands at the start (or end) of its block: nothing but
/// white space (or, at the end, punctuation) between them.
fn at_edge(entry: &Entry, block: usize, link: usize, edge: Edge) -> bool {
    let b = &entry.elements[block];
    let l = &entry.elements[link];
    match edge {
        Edge::Start => entry
            .slice(b.text_start, l.text_start)
            .chars()
            .all(char::is_whitespace),
        Edge::End => entry
            .slice(l.text_end, b.text_end)
            .chars()
            .all(|c| c.is_whitespace() || c.is_ascii_punctuation()),
    }
}

/// The link inside the target block that leads back to the caller: to the
/// caller element itself, to an element that holds it, or to an anchor set
/// where the caller stands (nothing but white space between them).
fn links_back(book: &SourceBook, caller: (usize, usize), block: (usize, usize)) -> Option<usize> {
    let (ce, ci) = caller;
    let caller_entry = &book.entries[ce];
    let c = &caller_entry.elements[ci];
    let (te, ti) = block;
    let t = &book.entries[te];
    let b = &t.elements[ti];
    // The block must not hold the caller itself (a link to its own paragraph).
    if te == ce && b.text_start <= c.text_start && c.text_end <= b.text_end {
        return None;
    }
    let dir = t.path.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
    for (i, e) in t.elements.iter().enumerate() {
        if e.text_start < b.text_start || e.text_end > b.text_end || i == ti {
            continue;
        }
        if !t.ancestors(i).any(|a| a == ti) {
            continue;
        }
        let Some(href) = e.href.as_deref() else {
            continue;
        };
        let Some((path, Some(fragment))) = internal_link_target(dir, &t.path, href.trim()) else {
            continue;
        };
        if book.by_path.get(&path) != Some(&ce) {
            continue;
        }
        let Some(&back) = caller_entry.ids.get(&fragment) else {
            continue;
        };
        if back == ci || caller_entry.ancestors(ci).any(|a| a == back) {
            return Some(i);
        }
        let anchor = &caller_entry.elements[back];
        // An anchor just before the caller ("<a id=r1/><a href=#n1>[1]</a>"),
        // or one that holds it.
        let between = if anchor.text_end <= c.text_start {
            caller_entry.slice(anchor.text_end, c.text_start)
        } else if anchor.text_start <= c.text_start && c.text_end <= anchor.text_end {
            String::new()
        } else {
            continue;
        };
        if between.chars().all(char::is_whitespace) && anchor.text_end <= c.text_end {
            return Some(i);
        }
    }
    None
}

/// A chapter and verse an element's id encodes ("Gen8-1", "v8_1", "c8v1",
/// "RUT.1.4"), when it encodes two numbers that way. Only used to
/// corroborate or contradict a verse's start, never to make one.
pub fn id_reference(id: &str) -> Option<(u16, u16)> {
    use std::sync::OnceLock;
    static FORM: OnceLock<regex::Regex> = OnceLock::new();
    let form = FORM.get_or_init(|| {
        regex::Regex::new(
            r"^(?:[A-Za-z]{1,12}[._-]?|[A-Za-z]{1,12}[._-]?\d?[A-Za-z]{1,12}[._-]?)?[cC]?(\d{1,3})(?:[._:-]|[vV])(\d{1,3})$",
        )
        .expect("valid pattern")
    });
    let c = form.captures(id)?;
    let chapter = c.get(1)?.as_str().parse().ok()?;
    let verse = c.get(2)?.as_str().parse().ok()?;
    (chapter > 0 && verse > 0).then_some((chapter, verse))
}

/// Notes of the book's own declared or linked structure, for reports.
pub fn note_count(book: &SourceBook) -> usize {
    book.entries
        .iter()
        .map(|e| {
            e.elements
                .iter()
                .enumerate()
                .filter(|(i, el)| {
                    el.has(NOTE_CONTAINERS)
                        && !e
                            .ancestors(*i)
                            .skip(1)
                            .any(|a| e.elements[a].has(NOTE_CONTAINERS))
                })
                .count()
        })
        .sum()
}

#[cfg(test)]
mod compact_tests {
    use super::*;

    #[test]
    fn runs_give_exactly_the_dense_table() {
        // Runs of every length, a value repeated after another, the last
        // position, and the empty table.
        let mut dense = Vec::new();
        let mut seed = 7u32;
        for _ in 0..5000 {
            seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            let value = if seed % 5 == 0 {
                NO_ELEMENT
            } else {
                (seed >> 8) % 7
            };
            for _ in 0..(seed >> 16) % 9 {
                dense.push(value);
            }
        }
        let runs = Runs::from_dense(&dense);
        assert_eq!(runs.len(), dense.len());
        for (at, &v) in dense.iter().enumerate() {
            assert_eq!(runs.get(at), v, "position {at}");
        }
        assert!(Runs::from_dense(&[]).is_empty());
    }

    #[test]
    fn byte_ranges_keep_every_start_and_end_including_long_ones() {
        let ranges = [
            (0u32, 1u32),
            (1, 9),
            (9, 9 + 255),
            (264, 264 + 256),
            (520, 4000),
            (4000, 4003),
        ];
        let mut b = ByteRanges::default();
        for &(s, e) in &ranges {
            b.push(s, e);
        }
        assert_eq!(b.len(), ranges.len());
        for (i, &r) in ranges.iter().enumerate() {
            assert_eq!(b.get(i), r, "range {i}");
        }
    }
}
