//! A neutral inventory of a reference edition, rules
//! `aw-reference-inventory-1` (evaluation candidate A1.5-E2).
//!
//! The A1.5 harness read the reference through the app's importer, which
//! keeps only what the 66-book canon can hold and drops the rest with an
//! error the harness discarded: verse 0, chapters above the canon's counts,
//! books outside the 66, unusual labels, a second verse with the same
//! number. A verse map in the archive renumbered it. None of that was ever
//! counted or reported.
//!
//! This inventory reads every verse of every USFM book file exactly as
//! published (`usfm::parse`: the same text the importer stores, with its
//! numbers as written) and gives each one a status:
//!
//! * `scorable` — a book of the 66, a chapter within its count, a verse
//!   1..=200 or a bridge `N-M` (stored on N, as the importer does), unique;
//! * one of the unsupported classes (`UNSUPPORTED`): kept, counted and
//!   listed, never dropped;
//! * outside the declared scope (the manifest's `books`), listed apart.
//!
//! A verse map, or any member that is not a USFM book or prose (`.htm`), is
//! never read and is reported as a problem; so is text that is not UTF-8.
//! The labels in reading order are hashed (`label_digest`) so the scorer can
//! compare them with the check's own Python reading of the same file
//! (`scripts/evaluation/reference_inventory.py`).

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use abiding_word_lib::scripture::canon::{self, NON_CANONICAL_USFM_CODES};
use abiding_word_lib::scripture::reference::MAX_VERSE;
use abiding_word_lib::scripture::usfm;
use sha2::{Digest, Sha256};

pub const RULES: &str = "aw-reference-inventory-1";

/// Every reason a published verse cannot be scored, in report order.
pub const UNSUPPORTED: [&str; 11] = [
    "verse_zero",
    "verse_label_unusual",
    "verse_duplicate",
    "verse_before_chapter",
    "chapter_zero",
    "chapter_label_unusual",
    "chapter_beyond_canon",
    "book_outside_canon",
    "book_code_unknown",
    "book_duplicate",
    "book_unidentified",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Scorable {
        ordinal: u8,
        chapter: u16,
        verse: u16,
    },
    Unsupported(&'static str),
    OutsideScope,
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub file: String,
    pub book: String,
    pub chapter: String,
    pub verse: String,
    pub text: String,
    pub status: Status,
}

impl Entry {
    pub fn label(&self) -> String {
        format!(
            "{} {}:{}",
            if self.book.is_empty() {
                "?"
            } else {
                &self.book
            },
            self.chapter,
            self.verse
        )
    }

    pub fn printed(&self) -> bool {
        !self.text.trim().is_empty()
    }
}

#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct ClassCount {
    pub printed: usize,
    pub empty: usize,
    pub references: Vec<String>,
}

#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct Member {
    pub name: String,
    /// "usfm", "prose" (read by nobody), or "refused".
    pub role: &'static str,
    pub book: Option<String>,
    pub verses: usize,
    pub verses_before_chapter: usize,
    pub utf8: bool,
    pub nfc: bool,
}

#[derive(Debug, Default)]
pub struct Inventory {
    pub entries: Vec<Entry>,
    /// Scorable keys (printed or empty) and the entry each is.
    pub scorable: BTreeMap<(u8, u16, u16), usize>,
    /// Keys in the canon's space that cannot be scored (a duplicate number,
    /// a book given twice), with why.
    pub unscorable_keys: BTreeMap<(u8, u16, u16), &'static str>,
    /// Keys of canon books the reference holds outside the declared scope.
    pub outside_keys: BTreeSet<(u8, u16, u16)>,
    pub members: Vec<Member>,
    pub problems: Vec<String>,
    pub before_chapter: usize,
    /// Verses before any chapter in a book outside the declared scope.
    pub before_chapter_outside: usize,
    pub label_digest: String,
}

#[derive(Debug, Default, serde::Serialize)]
pub struct InventoryReport {
    pub rules: &'static str,
    pub members: Vec<Member>,
    /// Reasons the reference is not a sound USFM reference (a verse map, a
    /// member of another kind, text that is not UTF-8). Any one of them
    /// makes the run not a protocol run.
    pub problems: Vec<String>,
    pub label_digest: String,
    /// Every verse label read, in any status.
    pub labels: usize,
    /// Scorable verses with text: the scored denominator.
    pub printed_scorable: usize,
    /// Scorable verses numbered but empty in the edition.
    pub absent_in_edition: usize,
    /// Every unsupported class (all of `UNSUPPORTED`, zeros included).
    pub unsupported: BTreeMap<&'static str, ClassCount>,
    pub printed_unsupported: usize,
    pub outside_declared_scope: ClassCount,
    /// Scorable printed verses per book.
    pub scorable_by_book: BTreeMap<String, usize>,
    /// Every scorable key (printed or empty), as compact ranges in canon
    /// order ("OBA 1:1-4"): compared with the check's own reading.
    pub scorable_ranges: Vec<String>,
    /// Verses before any chapter in a book outside the declared scope
    /// (counted in `outside_declared_scope.printed`; they have no label).
    pub before_chapter_outside: usize,
}

/// The verse key the importer would store a label under, or why not.
fn verse_key(label: &str) -> Result<u16, &'static str> {
    let (first, last) = match label.split_once(['-', '–']) {
        Some((a, b)) => (a.parse::<u16>().ok(), Some(b.parse::<u16>().ok())),
        None => (label.parse::<u16>().ok(), None),
    };
    match (first, last) {
        (Some(0), _) => Err("verse_zero"),
        (Some(v), None) if v <= MAX_VERSE => Ok(v),
        (Some(v), Some(Some(w))) if v <= MAX_VERSE && w > v => Ok(v),
        _ => Err("verse_label_unusual"),
    }
}

fn usfm_code_known(code: &str) -> bool {
    canon::by_code(code).is_some() || NON_CANONICAL_USFM_CODES.contains(&code)
}

/// The reference's USFM text as the USFM standard defines it where the app's
/// importer does not: an acrostic heading (`\qa`) is a heading (unchanged
/// from the A1.5 harness's `as_usfm_defines`).
fn as_usfm_defines(text: &str) -> String {
    text.lines()
        .map(|l| match l.strip_prefix("\\qa ") {
            Some(rest) => format!("\\s1 {rest}"),
            None => l.to_string(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Reads `path` (a zip of USFM books, or one `.usfm`/`.sfm` file).
/// `declared`: the manifest's books (upper-case USFM codes), or None for all.
pub fn read(path: &Path, declared: Option<&BTreeSet<String>>) -> Inventory {
    use std::io::Read;
    let mut inv = Inventory::default();
    let mut texts: Vec<(String, String)> = Vec::new();
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let lower = name.to_ascii_lowercase();
    let mut take = |inv: &mut Inventory, member: String, bytes: Vec<u8>| {
        let utf8 = std::str::from_utf8(&bytes).is_ok();
        if !utf8 {
            inv.problems.push(format!(
                "{member}: not UTF-8 (read with replacement characters)"
            ));
        }
        let text = String::from_utf8_lossy(&bytes).into_owned();
        inv.members.push(Member {
            name: member.clone(),
            role: "usfm",
            utf8,
            nfc: unicode_normalization::is_nfc(&text),
            ..Default::default()
        });
        texts.push((member, text));
    };
    if lower.ends_with(".zip") {
        let archive = std::fs::File::open(path)
            .map_err(|e| e.to_string())
            .and_then(|f| zip::ZipArchive::new(f).map_err(|e| e.to_string()));
        match archive {
            Err(e) => inv
                .problems
                .push(format!("{name}: not a readable zip ({e})")),
            Ok(mut zip) => {
                for i in 0..zip.len() {
                    let Ok(mut entry) = zip.by_index(i) else {
                        inv.problems.push(format!("{name}: member {i} unreadable"));
                        continue;
                    };
                    if entry.is_dir() {
                        continue;
                    }
                    let member = entry.name().to_string();
                    let low = member.to_ascii_lowercase();
                    let mut bytes = Vec::new();
                    if entry.read_to_end(&mut bytes).is_err() {
                        inv.problems.push(format!("{member}: unreadable"));
                        continue;
                    }
                    if low.ends_with(".usfm") || low.ends_with(".sfm") {
                        take(&mut inv, member, bytes);
                    } else if low.ends_with(".htm") || low.ends_with(".html") {
                        inv.members.push(Member {
                            name: member,
                            role: "prose",
                            ..Default::default()
                        });
                    } else {
                        let text = String::from_utf8_lossy(&bytes);
                        let why = if low.ends_with("versemap.xml") || text.contains("<verseMap") {
                            "a verse map, which would renumber the reference; never read"
                        } else {
                            "not a USFM book (.usfm, .sfm) or prose (.htm); never read"
                        };
                        inv.problems.push(format!("{member}: {why}"));
                        inv.members.push(Member {
                            name: member,
                            role: "refused",
                            ..Default::default()
                        });
                    }
                }
            }
        }
    } else if lower.ends_with(".usfm") || lower.ends_with(".sfm") {
        match std::fs::read(path) {
            Ok(bytes) => take(&mut inv, name.clone(), bytes),
            Err(e) => inv.problems.push(format!("{name}: unreadable ({e})")),
        }
    } else {
        inv.problems.push(format!(
            "{name}: not a USFM reference (a .zip of USFM books, or a .usfm or .sfm file)"
        ));
    }

    // Read every book file; note which canon book each file claims.
    let mut digest = Sha256::new();
    let mut files_of: BTreeMap<u8, usize> = BTreeMap::new();
    let mut parsed = Vec::new();
    for (index, (member, text)) in texts.iter().enumerate() {
        let ids = text.matches("\\id ").count() + text.matches("\\id\t").count();
        if ids > 1 {
            inv.problems
                .push(format!("{member}: {ids} \\id lines; one book per file"));
        }
        let doc = usfm::parse(&as_usfm_defines(text));
        if let Some(book) = doc.book_code.as_deref().and_then(canon::by_code) {
            *files_of.entry(book.ordinal).or_default() += 1;
        }
        let m = inv
            .members
            .iter_mut()
            .filter(|m| m.role == "usfm")
            .nth(index)
            .expect("member");
        m.book = doc.book_code.clone();
        m.verses = doc.verses.len();
        m.verses_before_chapter = doc.verses_outside_chapter;
        parsed.push((member.clone(), doc));
    }
    for (member, doc) in parsed {
        let code = doc.book_code.clone().unwrap_or_default();
        let book = canon::by_code(&code);
        for v in doc.verses {
            digest.update(format!("{member}\t{code}\t{}\t{}\n", v.chapter, v.label).as_bytes());
            let in_scope = declared.is_none_or(|d| d.contains(&code));
            let status = if code.is_empty() {
                Status::Unsupported("book_unidentified")
            } else if !in_scope {
                Status::OutsideScope
            } else if let Some(book) = book {
                if files_of.get(&book.ordinal).copied().unwrap_or(0) > 1 {
                    Status::Unsupported("book_duplicate")
                } else {
                    match v.chapter.parse::<u16>() {
                        Err(_) => Status::Unsupported("chapter_label_unusual"),
                        Ok(0) => Status::Unsupported("chapter_zero"),
                        Ok(c) if c > book.chapter_count => {
                            Status::Unsupported("chapter_beyond_canon")
                        }
                        Ok(c) => match verse_key(&v.label) {
                            Ok(n) => Status::Scorable {
                                ordinal: book.ordinal,
                                chapter: c,
                                verse: n,
                            },
                            Err(why) => Status::Unsupported(why),
                        },
                    }
                }
            } else if usfm_code_known(&code) {
                Status::Unsupported("book_outside_canon")
            } else {
                Status::Unsupported("book_code_unknown")
            };
            inv.entries.push(Entry {
                file: member.clone(),
                book: code.clone(),
                chapter: v.chapter,
                verse: v.label,
                text: v.text,
                status,
            });
        }
        if doc.verses_outside_chapter > 0 {
            digest
                .update(format!("{member}\t#before\t{}\n", doc.verses_outside_chapter).as_bytes());
            if code.is_empty() || declared.is_none_or(|d| d.contains(&code)) {
                inv.before_chapter += doc.verses_outside_chapter;
            } else {
                inv.before_chapter_outside += doc.verses_outside_chapter;
            }
        }
    }
    inv.label_digest = format!("{:x}", digest.finalize());

    // A number given twice: every occurrence is unscorable.
    let mut seen: HashMap<(u8, u16, u16), usize> = HashMap::new();
    for e in &inv.entries {
        if let Status::Scorable {
            ordinal,
            chapter,
            verse,
        } = e.status
        {
            *seen.entry((ordinal, chapter, verse)).or_default() += 1;
        }
    }
    for (i, e) in inv.entries.iter_mut().enumerate() {
        match e.status {
            Status::Scorable {
                ordinal,
                chapter,
                verse,
            } => {
                let key = (ordinal, chapter, verse);
                if seen[&key] > 1 {
                    e.status = Status::Unsupported("verse_duplicate");
                    inv.unscorable_keys.insert(key, "verse_duplicate");
                } else {
                    inv.scorable.insert(key, i);
                }
            }
            Status::Unsupported("book_duplicate") => {
                if let (Some(book), Ok(c), Ok(v)) = (
                    canon::by_code(&e.book),
                    e.chapter.parse::<u16>(),
                    verse_key(&e.verse),
                ) {
                    inv.unscorable_keys
                        .insert((book.ordinal, c, v), "book_duplicate");
                }
            }
            Status::OutsideScope => {
                if let (Some(book), Ok(c), Ok(v)) = (
                    canon::by_code(&e.book),
                    e.chapter.parse::<u16>(),
                    verse_key(&e.verse),
                ) {
                    inv.outside_keys.insert((book.ordinal, c, v));
                }
            }
            _ => {}
        }
    }
    inv
}

impl Inventory {
    pub fn text(&self, key: (u8, u16, u16)) -> Option<&str> {
        self.scorable
            .get(&key)
            .map(|i| self.entries[*i].text.as_str())
    }

    pub fn report(&self) -> InventoryReport {
        let mut r = InventoryReport {
            rules: RULES,
            members: self.members.clone(),
            problems: self.problems.clone(),
            label_digest: self.label_digest.clone(),
            labels: self.entries.len(),
            ..Default::default()
        };
        for class in UNSUPPORTED {
            r.unsupported.insert(class, ClassCount::default());
        }
        let mut run: Option<(u8, u16, u16, u16)> = None;
        let code = |o: u8| canon::by_ordinal(o).map_or("?", |b| b.code);
        let flush = |run: (u8, u16, u16, u16), out: &mut Vec<String>| {
            let (o, c, first, last) = run;
            out.push(if first == last {
                format!("{} {c}:{first}", code(o))
            } else {
                format!("{} {c}:{first}-{last}", code(o))
            });
        };
        for &(o, c, v) in self.scorable.keys() {
            run = match run {
                Some((ro, rc, first, last)) if ro == o && rc == c && v == last + 1 => {
                    Some((ro, rc, first, v))
                }
                Some(previous) => {
                    flush(previous, &mut r.scorable_ranges);
                    Some((o, c, v, v))
                }
                None => Some((o, c, v, v)),
            };
        }
        if let Some(previous) = run {
            flush(previous, &mut r.scorable_ranges);
        }
        for e in &self.entries {
            match &e.status {
                Status::Scorable { .. } => {
                    if e.printed() {
                        r.printed_scorable += 1;
                        *r.scorable_by_book.entry(e.book.clone()).or_default() += 1;
                    } else {
                        r.absent_in_edition += 1;
                    }
                }
                Status::Unsupported(class) => {
                    let c = r.unsupported.get_mut(class).expect("class");
                    if e.printed() {
                        c.printed += 1;
                        r.printed_unsupported += 1;
                    } else {
                        c.empty += 1;
                    }
                    c.references.push(e.label());
                }
                Status::OutsideScope => {
                    let c = &mut r.outside_declared_scope;
                    if e.printed() {
                        c.printed += 1;
                    } else {
                        c.empty += 1;
                    }
                    if !c.references.contains(&e.book) {
                        c.references.push(e.book.clone());
                    }
                }
            }
        }
        // Verses before any chapter cannot be labelled: counted as printed.
        let before = r
            .unsupported
            .get_mut("verse_before_chapter")
            .expect("class");
        before.printed += self.before_chapter;
        r.printed_unsupported += self.before_chapter;
        r.outside_declared_scope.printed += self.before_chapter_outside;
        r.before_chapter_outside = self.before_chapter_outside;
        r
    }
}
