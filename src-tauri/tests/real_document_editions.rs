//! Accuracy of Bibles read from documents, on a corpus of real editions
//! (v0.24; corpus and accepted-verse audit v0.24.2). Not run by default.
//!
//! The corpus is listed in `docs/corpus/editions.tsv`: each edition's files
//! (PDF, EPUB, Word, Markdown, plain text; one or more), its language and
//! layout, and the publisher's USFM of the same revision to compare with.
//! `scripts/fetch-real-editions.sh <folder>` downloads them and checks their
//! checksums; `scripts/build-derived-editions.py` writes the converted ones.
//! Then
//!
//! ```text
//! AW_DOCUMENT_EDITIONS=<folder> AW_AUDIT_OUT=<folder>/audit \
//!   cargo test --release --test real_document_editions -- --ignored --nocapture
//! ```
//!
//! `AW_EDITIONS=key,key` audits only those; editions marked `hidden` (kept
//! out of sight while the recogniser was being developed, for an
//! independent final evaluation) are audited only with
//! `AW_INCLUDE_HIDDEN=1`.
//!
//! Each document is read exactly as a person's file would be: the recogniser
//! never sees the USFM. Only afterwards is every verse compared with the
//! publisher's text of the same edition, and classified:
//!
//! * **exact** — the same characters (outer whitespace aside);
//! * **normalised** — equal once whitespace is collapsed and removed between
//!   punctuation (a thin space printed between two quotation marks);
//! * **different** — anything else: a genuine textual difference.
//!
//! Accepted (usable) verses and refused ones are counted apart: the accuracy
//! that matters is that of the verses the app would show.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Instant;

use abiding_word_lib::scripture::canon;
use abiding_word_lib::scripture::import::{self, ImportOrigin, Severity};

fn folder() -> Option<PathBuf> {
    std::env::var_os("AW_DOCUMENT_EDITIONS").map(PathBuf::from)
}

fn squash(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The same text with every whitespace removed: letters, digits and
/// punctuation in order.
fn dense(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Letters and digits only, in lower case: what is left when punctuation,
/// typography (quotes, dashes, apostrophes), spacing and case are set aside.
fn letters(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// What kind of difference a found verse has from the edition's own text
/// (only for verses that differ beyond spacing and letter case):
/// - `unicode`: canonically the same characters (Unicode NFC): the document
///   encodes them differently (a CJK compatibility ideograph, 靈 U+F9B3,
///   written as the unified one, U+9748), the words are the same;
/// - `sign`: the same letters, with a note's sign the edition does not have
///   left in the verse ("†ᾧ"): apparatus, counted with `extra`;
/// - `typography`: the same letters and digits; punctuation or quotes differ;
/// - `extra`: the edition's words and more (a heading, note, reference or
///   another verse read into it: apparatus contamination);
/// - `missing`: part of the edition's words only (the verse is cut short);
/// - `wrong_reference`: the words of another verse of the book (a verse
///   given the wrong number or chapter);
/// - `changed`: anything else (the words themselves differ).
fn difference_kind(want: &str, found: &str, book: &HashSet<String>) -> &'static str {
    use unicode_normalization::UnicodeNormalization;
    let nfc = |t: &str| {
        t.split_whitespace()
            .collect::<String>()
            .nfc()
            .collect::<String>()
    };
    let (w, f) = (letters(want), letters(found));
    if nfc(want) == nfc(found) {
        "unicode"
    } else if w == f {
        let signs = |t: &str| t.chars().filter(|c| "*†‡§¶".contains(*c)).count();
        if signs(found) > signs(want) {
            "sign"
        } else {
            "typography"
        }
    } else if !w.is_empty() && f.contains(&w) {
        "extra"
    } else if !f.is_empty() && w.contains(&f) {
        "missing"
    } else if book.contains(&f) {
        "wrong_reference"
    } else {
        "changed"
    }
}

/// One edition of the corpus (a row of `docs/corpus/editions.tsv`).
#[derive(Clone, serde::Serialize)]
struct Edition {
    key: String,
    files: Vec<String>,
    reference: Option<String>,
    language: String,
    script: String,
    format: String,
    layout: String,
    /// "impl" (development), "regression" (disclosed evaluation sets),
    /// "withheld-b", "withheld-c" (independent, evaluated once each).
    role: String,
    /// The books the document holds (None: the whole edition).
    books: Option<Vec<String>>,
}

/// The corpus manifests: the development and regression editions, and the
/// withheld sets (v0.24.3), each a TSV with the same first nine columns
/// (the withheld sets add the books each document holds, its checksum and
/// the program that made it).
fn corpus() -> Vec<Edition> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/corpus");
    let mut out = Vec::new();
    for name in ["editions.tsv", "withheld-b.tsv", "withheld-c.tsv"] {
        let Ok(text) = std::fs::read_to_string(dir.join(name)) else {
            continue;
        };
        for line in text.lines().skip(1) {
            let cols: Vec<&str> = line.split('\t').collect();
            if cols.len() < 9 || line.starts_with('#') {
                continue;
            }
            // key, files, reference, language, script, format, layout, role,
            // source [, books, sha256, generator]
            let books = cols
                .get(9)
                .map(|b| b.trim())
                .filter(|b| !b.is_empty() && *b != "all")
                .map(|b| b.split(',').map(|c| c.trim().to_string()).collect());
            out.push(Edition {
                key: cols[0].into(),
                files: cols[1].split(',').map(str::to_string).collect(),
                reference: (cols[2] != "-").then(|| cols[2].to_string()),
                language: cols[3].into(),
                script: cols[4].into(),
                format: cols[5].into(),
                layout: cols[6].into(),
                role: cols[7].into(),
                books,
            });
        }
    }
    out
}

#[derive(Default, Clone, serde::Serialize)]
struct BookAudit {
    code: String,
    /// Verses with text in the edition's own USFM: what the edition prints.
    expected: usize,
    /// Verses the USFM numbers but leaves empty (omitted in this edition).
    absent_in_edition: usize,
    detected: usize,
    usable: usize,
    unmapped: usize,
    chapters: usize,
    usable_chapters: usize,
    refused_reading_order: usize,
    refused_verse_numbers: usize,
    refused_other: usize,
    /// Among usable (accepted) verses.
    usable_exact: usize,
    usable_normalised: usize,
    /// Of the normalised: letters in another case only (small capitals
    /// printed as capitals: "LORD’S" for "LORD’s").
    usable_case_only: usize,
    usable_different: usize,
    /// Of the genuine differences, by kind (see `difference_kind`).
    usable_typography: usize,
    usable_extra_text: usize,
    usable_missing_text: usize,
    usable_wrong_reference: usize,
    usable_changed: usize,
    /// Among refused verses (never shown).
    refused_exact: usize,
    refused_normalised: usize,
    refused_different: usize,
    /// Usable verses the edition does not have at all (a wrong number or
    /// boundary): false positives.
    usable_extra: usize,
    /// Verses missing from a chapter that is otherwise usable.
    missing_in_usable_chapter: usize,
    /// Of the usable verses (structurally accepted), those whose text is
    /// verified (v0.24.3): what the app uses as the Bible's text.
    verified: usize,
    verified_exact: usize,
    verified_normalised: usize,
    verified_different: usize,
    verified_typography: usize,
    verified_extra_text: usize,
    verified_missing_text: usize,
    verified_wrong_reference: usize,
    verified_changed: usize,
    verified_extra: usize,
    /// Usable verses held back as unverified: the text was in fact the
    /// edition's (caution's cost) or was not (contamination caught).
    unverified_clean: usize,
    unverified_caught: usize,
    /// Verified verses whose kept place in the document holds their words.
    located: usize,
    location_checked: usize,
}

impl BookAudit {
    fn add(&mut self, b: &BookAudit) {
        self.expected += b.expected;
        self.absent_in_edition += b.absent_in_edition;
        self.detected += b.detected;
        self.usable += b.usable;
        self.unmapped += b.unmapped;
        self.chapters += b.chapters;
        self.usable_chapters += b.usable_chapters;
        self.refused_reading_order += b.refused_reading_order;
        self.refused_verse_numbers += b.refused_verse_numbers;
        self.refused_other += b.refused_other;
        self.usable_exact += b.usable_exact;
        self.usable_normalised += b.usable_normalised;
        self.usable_case_only += b.usable_case_only;
        self.usable_different += b.usable_different;
        self.usable_typography += b.usable_typography;
        self.usable_extra_text += b.usable_extra_text;
        self.usable_missing_text += b.usable_missing_text;
        self.usable_wrong_reference += b.usable_wrong_reference;
        self.usable_changed += b.usable_changed;
        self.refused_exact += b.refused_exact;
        self.refused_normalised += b.refused_normalised;
        self.refused_different += b.refused_different;
        self.usable_extra += b.usable_extra;
        self.missing_in_usable_chapter += b.missing_in_usable_chapter;
        self.verified += b.verified;
        self.verified_exact += b.verified_exact;
        self.verified_normalised += b.verified_normalised;
        self.verified_different += b.verified_different;
        self.verified_typography += b.verified_typography;
        self.verified_extra_text += b.verified_extra_text;
        self.verified_missing_text += b.verified_missing_text;
        self.verified_wrong_reference += b.verified_wrong_reference;
        self.verified_changed += b.verified_changed;
        self.verified_extra += b.verified_extra;
        self.unverified_clean += b.unverified_clean;
        self.unverified_caught += b.unverified_caught;
        self.located += b.located;
        self.location_checked += b.location_checked;
    }
}

#[derive(serde::Serialize)]
struct Difference {
    reference: String,
    usable: bool,
    verified: bool,
    unverified_reason: Option<&'static str>,
    kind: &'static str,
    page: Option<u32>,
    found: String,
    edition: String,
}

#[derive(serde::Serialize)]
struct EditionAudit {
    edition: Edition,
    reference: String,
    totals: BookAudit,
    books: Vec<BookAudit>,
    /// Books of the edition of which no verse was found.
    books_not_found: Vec<String>,
    issues: BTreeMap<String, usize>,
    refused_chapters: Vec<String>,
    differences: Vec<Difference>,
    extra_verses: Vec<String>,
    /// False positives among the verified verses.
    verified_extra_verses: Vec<String>,
    unverified_reasons: BTreeMap<String, usize>,
    apparatus: usize,
    pages_uncertain: usize,
    pages: usize,
    found_books: Vec<(String, Option<String>, Option<u32>)>,
    errors: Vec<String>,
    seconds: f64,
    peak_memory_mb: Option<f64>,
}

/// Why a chapter was refused, from the codes of the errors naming it.
fn reason(code: &str) -> &'static str {
    match code {
        "bibleImport.documentSequence" | "bibleImport.documentReadingOrder" => "order",
        "bibleImport.documentAmbiguous"
        | "bibleImport.documentOrder"
        | "bibleImport.verseTwice"
        | "bibleImport.documentUnnumbered"
        | "bibleImport.documentChapterShort"
        | "bibleImport.documentLongVerse" => "numbers",
        _ => "other",
    }
}

/// Resets the process's peak resident memory (Linux), so the next reading
/// of it covers one edition.
fn reset_peak() {
    let _ = std::fs::write("/proc/self/clear_refs", "5");
}

fn peak_mb() -> Option<f64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|l| l.starts_with("VmHWM:"))?;
    let kb: f64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kb / 1024.0)
}

/// The reference USFM read as the USFM standard defines it where the app's
/// importer does not (yet): an acrostic heading (`\qa BETH`) is a heading,
/// not words of the verse before it. The importer keeps the bundled Bibles'
/// text as it was (see docs/bible-recognition-v0.24.2.md); the comparison
/// must not count the heading a document rightly leaves out as a difference.
fn as_usfm_defines(path: &Path) -> PathBuf {
    use std::io::{Read, Write};
    let Ok(file) = std::fs::File::open(path) else {
        return path.to_path_buf();
    };
    let Ok(mut zip) = zip::ZipArchive::new(file) else {
        return path.to_path_buf();
    };
    let out = std::env::temp_dir().join(format!(
        "aw-reference-{}-{}",
        std::process::id(),
        path.file_name().unwrap().to_string_lossy()
    ));
    let mut writer = zip::ZipWriter::new(std::fs::File::create(&out).unwrap());
    let options: zip::write::SimpleFileOptions = zip::write::SimpleFileOptions::default();
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).unwrap();
        let name = entry.name().to_string();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        let lower = name.to_ascii_lowercase();
        if lower.ends_with(".usfm") || lower.ends_with(".sfm") {
            let text = String::from_utf8_lossy(&bytes)
                .lines()
                .map(|l| match l.strip_prefix("\\qa ") {
                    Some(rest) => format!("\\s1 {rest}"),
                    None => l.to_string(),
                })
                .collect::<Vec<_>>()
                .join("\n");
            bytes = text.into_bytes();
        }
        writer.start_file(name, options).unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap();
    out
}

fn audit(dir: &Path, edition: &Edition) -> Option<EditionAudit> {
    let files: Vec<PathBuf> = edition.files.iter().map(|f| dir.join(f)).collect();
    if let Some(missing) = files.iter().find(|f| !f.exists()) {
        eprintln!("{}: {} missing, skipped", edition.key, missing.display());
        return None;
    }
    let reference_file = dir.join(edition.reference.as_deref()?);
    if !reference_file.exists() {
        eprintln!("{}: reference missing, skipped", edition.key);
        return None;
    }
    reset_peak();
    let started = Instant::now();
    let staged = import::stage_documents(&files, ImportOrigin::UserFiles).unwrap();
    let seconds = started.elapsed().as_secs_f64();
    let peak_memory_mb = peak_mb();
    // Only now is the publisher's text read, for the comparison.
    let reference_file = as_usfm_defines(&reference_file);
    let reference = import::stage(
        std::slice::from_ref(&reference_file),
        ImportOrigin::UserFiles,
    )
    .unwrap();
    let content = &staged.content;
    let usable = content.usable_chapters().unwrap_or_default();
    let origins = staged.document_origins();
    let stage = staged.documents.as_ref().expect("a document stage");
    let mut unverified_reasons: BTreeMap<String, usize> = BTreeMap::new();
    for reason in stage.unverified.values() {
        *unverified_reasons.entry((*reason).to_string()).or_default() += 1;
    }
    // Whether a verse's kept place (its first to last paragraph) holds its
    // words: the original location the app shows for it.
    let located = |key: (u8, u16, u16), text: &str| -> Option<bool> {
        let origin = stage.origins.get(&key)?;
        let file = stage.files.get(usize::from(origin.start.document))?;
        let first = origin.start.segment? as usize;
        let last = if origin.end.document == origin.start.document {
            origin.end.segment.map_or(first, |s| s as usize)
        } else {
            file.segments.len().saturating_sub(1)
        };
        let held: String = file
            .segments
            .get(first..=last.min(file.segments.len().saturating_sub(1)).max(first))?
            .iter()
            .map(|s| letters(&s.text))
            .collect();
        let start: String = letters(text).chars().take(24).collect();
        Some(start.is_empty() || held.contains(&start))
    };

    let mut refused: BTreeMap<(String, u16), &'static str> = BTreeMap::new();
    let mut issues: BTreeMap<String, usize> = BTreeMap::new();
    for issue in &content.issues {
        *issues
            .entry(format!(
                "{:?} {}",
                issue.severity,
                issue.code.as_deref().unwrap_or("-")
            ))
            .or_default() += 1;
        if issue.severity == Severity::Error {
            if let (Some(book), Some(chapter)) = (issue.book, issue.chapter) {
                let r = reason(issue.code.as_deref().unwrap_or(""));
                let entry = refused.entry((book.to_string(), chapter)).or_insert(r);
                if *entry == "other" {
                    *entry = r;
                }
            }
        }
    }

    let mut books: Vec<BookAudit> = Vec::new();
    let mut differences = Vec::new();
    let mut extra_verses = Vec::new();
    let mut verified_extra_verses = Vec::new();
    let mut books_not_found = Vec::new();
    let mut seen_books: BTreeSet<u8> = BTreeSet::new();
    // A file of one book ("fraLSG_LUK.pdf") is measured against that book;
    // a document holding some books (the manifest's `books`), against those.
    let only_books: Option<BTreeSet<u8>> = match &edition.books {
        Some(codes) => Some(
            codes
                .iter()
                .filter_map(|c| canon::by_code(c))
                .map(|b| b.ordinal)
                .collect(),
        ),
        None => edition
            .files
            .first()
            .and_then(|f| f.rsplit_once('_'))
            .and_then(|(_, rest)| rest.split('.').next())
            .and_then(canon::by_code)
            .map(|b| BTreeSet::from([b.ordinal])),
    };
    for (ordinal, rbook) in &reference.content.books {
        if only_books.as_ref().is_some_and(|o| !o.contains(ordinal)) {
            continue;
        }
        seen_books.insert(*ordinal);
        let book = canon::by_ordinal(*ordinal).unwrap();
        let mut a = BookAudit {
            code: book.code.to_string(),
            ..Default::default()
        };
        let found = content.books.get(ordinal);
        // Every verse of the edition's book, by its letters: a found verse
        // with another verse's words has the wrong reference.
        let book_verses: HashSet<String> = rbook
            .chapters
            .values()
            .flat_map(|c| c.verses.values())
            .map(|t| letters(t))
            .filter(|t| !t.is_empty())
            .collect();
        for (chapter, rch) in &rbook.chapters {
            let fch = found.and_then(|b| b.chapters.get(chapter));
            if fch.is_some() {
                a.chapters += 1;
            }
            let ok = usable.contains(&(*ordinal, *chapter));
            if ok {
                a.usable_chapters += 1;
            } else if let Some(r) = refused.get(&(book.code.to_string(), *chapter)) {
                match *r {
                    "order" => a.refused_reading_order += 1,
                    "numbers" => a.refused_verse_numbers += 1,
                    _ => a.refused_other += 1,
                }
            }
            for (verse, want) in &rch.verses {
                if want.trim().is_empty() {
                    a.absent_in_edition += 1;
                    continue;
                }
                a.expected += 1;
                let Some(text) = fch.and_then(|c| c.verses.get(verse)) else {
                    a.unmapped += 1;
                    if ok {
                        a.missing_in_usable_chapter += 1;
                    }
                    continue;
                };
                a.detected += 1;
                if ok {
                    a.usable += 1;
                } else {
                    a.unmapped += 1;
                }
                let case_only = dense(want) != dense(text)
                    && dense(want).to_lowercase() == dense(text).to_lowercase();
                let class = if want.trim() == text.trim() {
                    0
                } else if squash(want) == squash(text) || dense(want) == dense(text) || case_only {
                    1
                } else {
                    2
                };
                if ok && case_only {
                    a.usable_case_only += 1;
                }
                match (ok, class) {
                    (true, 0) => a.usable_exact += 1,
                    (true, 1) => a.usable_normalised += 1,
                    (true, _) => a.usable_different += 1,
                    (false, 0) => a.refused_exact += 1,
                    (false, 1) => a.refused_normalised += 1,
                    (false, _) => a.refused_different += 1,
                }
                let kind = difference_kind(want, text, &book_verses);
                let key = (*ordinal, *chapter, *verse);
                let unverified = stage.unverified.get(&key).copied();
                let verified = ok && unverified.is_none();
                let contaminated = class == 2 && !matches!(kind, "typography" | "unicode");
                if ok && unverified.is_some() {
                    if contaminated {
                        a.unverified_caught += 1;
                    } else {
                        a.unverified_clean += 1;
                    }
                }
                if verified {
                    a.verified += 1;
                    match class {
                        0 => a.verified_exact += 1,
                        1 => a.verified_normalised += 1,
                        _ => {
                            a.verified_different += 1;
                            match kind {
                                "typography" | "unicode" => a.verified_typography += 1,
                                "extra" | "sign" => a.verified_extra_text += 1,
                                "missing" => a.verified_missing_text += 1,
                                "wrong_reference" => a.verified_wrong_reference += 1,
                                _ => a.verified_changed += 1,
                            }
                        }
                    }
                    if let Some(found_here) = located(key, text) {
                        a.location_checked += 1;
                        if found_here {
                            a.located += 1;
                        }
                    }
                }
                if ok && class == 2 {
                    match kind {
                        "typography" | "unicode" => a.usable_typography += 1,
                        "extra" | "sign" => a.usable_extra_text += 1,
                        "missing" => a.usable_missing_text += 1,
                        "wrong_reference" => a.usable_wrong_reference += 1,
                        _ => a.usable_changed += 1,
                    }
                }
                if class == 2 && (ok || differences.len() < 60) {
                    differences.push(Difference {
                        reference: format!("{} {chapter}:{verse}", book.code),
                        usable: ok,
                        verified,
                        unverified_reason: unverified,
                        kind,
                        page: origins
                            .get(&(*ordinal, *chapter, *verse))
                            .copied()
                            .flatten(),
                        found: squash(text),
                        edition: squash(want),
                    });
                }
            }
            // Accepted verses the edition does not have.
            if let (true, Some(fch)) = (ok, fch) {
                for (verse, text) in &fch.verses {
                    let present = rch.verses.get(verse).is_some_and(|w| !w.trim().is_empty());
                    if !present && !text.trim().is_empty() {
                        a.usable_extra += 1;
                        extra_verses.push(format!("{} {chapter}:{verse}", book.code));
                        if !stage.unverified.contains_key(&(*ordinal, *chapter, *verse)) {
                            a.verified_extra += 1;
                            verified_extra_verses.push(format!("{} {chapter}:{verse}", book.code));
                        }
                    }
                }
            }
        }
        if a.detected == 0 && a.expected > 0 {
            books_not_found.push(book.code.to_string());
        }
        books.push(a);
    }
    // Accepted chapters of books the edition does not contain at all.
    for (ordinal, fbook) in &content.books {
        if seen_books.contains(ordinal) {
            continue;
        }
        let code = canon::by_ordinal(*ordinal).map_or("?", |b| b.code);
        for (chapter, ch) in &fbook.chapters {
            if usable.contains(&(*ordinal, *chapter)) {
                for verse in ch.verses.keys() {
                    extra_verses.push(format!("{code} {chapter}:{verse}"));
                    if !stage.unverified.contains_key(&(*ordinal, *chapter, *verse)) {
                        verified_extra_verses.push(format!("{code} {chapter}:{verse}"));
                    }
                }
            }
        }
    }
    let mut totals = BookAudit {
        code: "ALL".into(),
        ..Default::default()
    };
    for b in &books {
        totals.add(b);
    }
    totals.usable_extra = extra_verses.len();
    totals.verified_extra = verified_extra_verses.len();
    let refused_chapters = refused
        .iter()
        .map(|((b, c), r)| format!("{b} {c} ({r})"))
        .collect();
    let (pages, pages_uncertain) = staged.document_page_orders();
    let found_books = staged
        .documents
        .as_ref()
        .map(|d| {
            d.books
                .iter()
                .map(|b| (b.heading.clone(), b.book.map(str::to_string), b.place.page))
                .collect()
        })
        .unwrap_or_default();
    let errors = content
        .issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .map(|i| i.message.clone())
        .take(200)
        .collect();
    extra_verses.truncate(200);
    Some(EditionAudit {
        edition: edition.clone(),
        reference: reference_file
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned(),
        totals,
        books,
        books_not_found,
        issues,
        refused_chapters,
        differences,
        extra_verses,
        verified_extra_verses,
        unverified_reasons,
        apparatus: stage.apparatus.len(),
        pages_uncertain,
        pages,
        found_books,
        errors,
        seconds,
        peak_memory_mb,
    })
}

fn pct(a: usize, b: usize) -> String {
    if b == 0 {
        "—".into()
    } else {
        format!("{:.1} %", 100.0 * a as f64 / b as f64)
    }
}

fn markdown(a: &EditionAudit) -> String {
    let mut s = String::new();
    let t = &a.totals;
    let e = &a.edition;
    let _ = writeln!(s, "# {} — {}\n", e.key, e.files.join(", "));
    let _ = writeln!(
        s,
        "Language {} ({}), format {}, layout: {}. Compared with {} (same revision).\n",
        e.language, e.script, e.format, e.layout, a.reference
    );
    let _ = writeln!(
        s,
        "Pages: {} ({} with an uncertain reading order). Read in {:.1} s, peak memory {}.\n",
        a.pages,
        a.pages_uncertain,
        a.seconds,
        a.peak_memory_mb
            .map_or("—".into(), |m| format!("{m:.0} MB"))
    );
    let _ = writeln!(s, "| | Verses | Share |\n|---|---|---|");
    let _ = writeln!(
        s,
        "| Printed in the edition (its USFM) | {} | |",
        t.expected
    );
    let _ = writeln!(
        s,
        "| Numbered but empty in the edition | {} | |",
        t.absent_in_edition
    );
    let _ = writeln!(
        s,
        "| Detected | {} | {} |",
        t.detected,
        pct(t.detected, t.expected)
    );
    let _ = writeln!(
        s,
        "| **Usable (accepted)** | **{}** | **{}** |",
        t.usable,
        pct(t.usable, t.expected)
    );
    let _ = writeln!(
        s,
        "| Unmapped or refused | {} | {} |",
        t.unmapped,
        pct(t.unmapped, t.expected)
    );
    let _ = writeln!(
        s,
        "| Usable: exact | {} | {} |",
        t.usable_exact,
        pct(t.usable_exact, t.usable)
    );
    let _ = writeln!(
        s,
        "| Usable: whitespace or typography only | {} | {} |",
        t.usable_normalised,
        pct(t.usable_normalised, t.usable)
    );
    let _ = writeln!(
        s,
        "| … of which letter case only (small capitals) | {} | |",
        t.usable_case_only
    );
    let _ = writeln!(
        s,
        "| **Usable: genuine differences** | **{}** | |",
        t.usable_different
    );
    let _ = writeln!(
        s,
        "| … punctuation, typography or Unicode encoding only | {} | |",
        t.usable_typography
    );
    let _ = writeln!(
        s,
        "| … extra text read into the verse (heading, note, reference, a note's sign) | {} | |",
        t.usable_extra_text
    );
    let _ = writeln!(s, "| … verse cut short | {} | |", t.usable_missing_text);
    let _ = writeln!(
        s,
        "| … another verse's words (wrong reference) | {} | |",
        t.usable_wrong_reference
    );
    let _ = writeln!(s, "| … words changed | {} | |", t.usable_changed);
    let _ = writeln!(
        s,
        "| **Usable verses the edition does not have (false positives)** | **{}** | |",
        t.usable_extra
    );
    let _ = writeln!(
        s,
        "| Missing from usable chapters | {} | |",
        t.missing_in_usable_chapter
    );
    let _ = writeln!(
        s,
        "| **Text-verified (used as the Bible's text)** | **{}** | **{}** |",
        t.verified,
        pct(t.verified, t.expected)
    );
    let _ = writeln!(
        s,
        "| Verified: exact / whitespace or case only / different | {} / {} / {} | |",
        t.verified_exact, t.verified_normalised, t.verified_different
    );
    let _ = writeln!(
        s,
        "| Verified differences: typography / extra (apparatus) / cut / wrong reference / changed | {} / {} / {} / {} / {} | |",
        t.verified_typography,
        t.verified_extra_text,
        t.verified_missing_text,
        t.verified_wrong_reference,
        t.verified_changed
    );
    let _ = writeln!(
        s,
        "| **Verified verses the edition does not have** | **{}** | |",
        t.verified_extra
    );
    let _ = writeln!(
        s,
        "| Held back as unverified: text was clean / contamination caught | {} / {} | |",
        t.unverified_clean, t.unverified_caught
    );
    let _ = writeln!(
        s,
        "| Original location holds the verse (verified verses checked) | {} of {} | {} |",
        t.located,
        t.location_checked,
        pct(t.located, t.location_checked)
    );
    let _ = writeln!(
        s,
        "| Pieces of apparatus kept with their places | {} | |",
        a.apparatus
    );
    if !a.unverified_reasons.is_empty() {
        let reasons: Vec<String> = a
            .unverified_reasons
            .iter()
            .map(|(r, n)| format!("{r} {n}"))
            .collect();
        let _ = writeln!(s, "\nUnverified by reason: {}.", reasons.join(", "));
    }
    let _ = writeln!(
        s,
        "| Refused verses: exact / whitespace / different | {} / {} / {} | |",
        t.refused_exact, t.refused_normalised, t.refused_different
    );
    let _ = writeln!(
        s,
        "\nChapters: {} found, {} usable; refused for reading order {}, for verse numbers {}, other {}.\n",
        t.chapters, t.usable_chapters, t.refused_reading_order, t.refused_verse_numbers, t.refused_other
    );
    if !a.books_not_found.is_empty() {
        let _ = writeln!(s, "Books not found: {}.\n", a.books_not_found.join(", "));
    }
    let _ = writeln!(s, "## Per book\n");
    let _ = writeln!(
        s,
        "| Book | Printed | Usable | Refused chapters (order / numbers / other) | Usable exact / ws / different | False positives | Missing in usable |\n|---|---|---|---|---|---|---|"
    );
    for b in &a.books {
        let _ = writeln!(
            s,
            "| {} | {} | {} ({}) | {} / {} / {} | {} / {} / {} | {} | {} |",
            b.code,
            b.expected,
            b.usable,
            pct(b.usable, b.expected),
            b.refused_reading_order,
            b.refused_verse_numbers,
            b.refused_other,
            b.usable_exact,
            b.usable_normalised,
            b.usable_different,
            b.usable_extra,
            b.missing_in_usable_chapter
        );
    }
    let _ = writeln!(s, "\n## Issues\n");
    for (k, n) in &a.issues {
        let _ = writeln!(s, "- {n} × {k}");
    }
    let _ = writeln!(
        s,
        "\n## Refused chapters\n\n{}",
        a.refused_chapters.join(", ")
    );
    let _ = writeln!(s, "\n## Book headings found\n");
    for (heading, book, page) in &a.found_books {
        let _ = writeln!(
            s,
            "- {heading} → {} (page {})",
            book.as_deref().unwrap_or("not identified"),
            page.map_or("?".into(), |p| p.to_string())
        );
    }
    let _ = writeln!(s, "\n## Errors (first 200)\n");
    for e in &a.errors {
        let _ = writeln!(s, "- {e}");
    }
    if !a.extra_verses.is_empty() {
        let _ = writeln!(s, "\n## False positives\n\n{}", a.extra_verses.join(", "));
    }
    if !a.verified_extra_verses.is_empty() {
        let _ = writeln!(
            s,
            "\n## False positives among verified verses\n\n{}",
            a.verified_extra_verses.join(", ")
        );
    }
    let _ = writeln!(s, "\n## Differences\n");
    for d in &a.differences {
        let _ = writeln!(
            s,
            "- {}{}{} [{}] (page {}): found «{}» — edition «{}»",
            d.reference,
            if d.verified {
                " **verified**"
            } else if d.usable {
                " usable"
            } else {
                ""
            },
            d.unverified_reason
                .map(|r| format!(" (unverified: {r})"))
                .unwrap_or_default(),
            d.kind,
            d.page.map_or("?".into(), |p| p.to_string()),
            d.found,
            d.edition
        );
    }
    s
}

fn summary_row(a: &EditionAudit) -> String {
    let t = &a.totals;
    let e = &a.edition;
    format!(
        "| {} | {} | {} | {} | {} | {} ({}) | {} ({}) | {} | {} | {} | {} / {} / {} / {} / {} | {} / {} | {} / {} | {} | {:.0} s | {} |",
        e.key,
        e.language,
        e.format,
        e.layout,
        t.expected,
        t.usable,
        pct(t.usable, t.expected),
        t.verified,
        pct(t.verified, t.expected),
        t.verified_exact,
        t.verified_normalised,
        t.verified_different,
        t.verified_typography,
        t.verified_extra_text,
        t.verified_missing_text,
        t.verified_wrong_reference,
        t.verified_changed,
        t.usable_extra,
        t.verified_extra,
        t.unverified_clean,
        t.unverified_caught,
        pct(t.located, t.location_checked),
        a.seconds,
        a.peak_memory_mb
            .map_or("—".into(), |m| format!("{m:.0} MB"))
    )
}

#[test]
#[ignore]
fn real_document_editions_report() {
    let Some(dir) = folder() else {
        eprintln!("AW_DOCUMENT_EDITIONS not set; nothing to do");
        return;
    };
    let out = std::env::var_os("AW_AUDIT_OUT").map(PathBuf::from);
    let only: Option<BTreeSet<String>> = std::env::var("AW_EDITIONS")
        .ok()
        .map(|v| v.split(',').map(str::to_string).collect());
    // Which sets: AW_ROLES=impl,regression (the default), or withheld-b,
    // withheld-c (each evaluated once). AW_INCLUDE_HIDDEN=1 (v0.24.2) adds
    // the regression set, once the hidden one.
    let roles: BTreeSet<String> = std::env::var("AW_ROLES")
        .unwrap_or_else(|_| "impl,regression".into())
        .split(',')
        .map(|r| r.trim().to_string())
        .collect();
    let mut rows = Vec::new();
    for edition in corpus() {
        if only.as_ref().is_some_and(|o| !o.contains(&edition.key)) {
            continue;
        }
        if !roles.contains(&edition.role) && only.is_none() {
            continue;
        }
        let Some(a) = audit(&dir, &edition) else {
            continue;
        };
        let md = markdown(&a);
        eprintln!("{}", summary_row(&a));
        rows.push(summary_row(&a));
        if let Some(out) = &out {
            std::fs::create_dir_all(out).unwrap();
            std::fs::write(out.join(format!("{}.md", edition.key)), &md).unwrap();
            std::fs::write(
                out.join(format!("{}.json", edition.key)),
                serde_json::to_string_pretty(&a).unwrap(),
            )
            .unwrap();
        }
    }
    let table = format!(
        "| Edition | Lang | Format | Layout | Printed | Detected (structure accepted) | Text-verified | Verified exact | WS / case only | Verified different | Typography / extra / cut / wrong ref. / changed (verified) | False pos. detected / verified | Unverified: clean / caught | Location | Time | Peak memory |\n|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|\n{}\n",
        rows.join("\n")
    );
    eprintln!("{table}");
    if let Some(out) = &out {
        std::fs::write(out.join("summary.md"), table).unwrap();
    }
}
