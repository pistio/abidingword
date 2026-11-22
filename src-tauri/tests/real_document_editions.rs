//! Accuracy of Bibles read from documents, on a corpus of real editions
//! (v0.24; corpus and accepted-verse audit v0.24.2; evaluation rules E2 for
//! the frozen A1.5 recogniser). Not run by default.
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
//! never sees the USFM. Only afterwards is the publisher's text read, and
//! compared under the evaluation rules of candidate A1.5-E3
//! (`docs/audit/candidate-a1.5/evaluation-e3/EVALUATION-CANDIDATE.md`):
//!
//! * **the reference** is read by a neutral inventory (`eval_support/reference.rs`)
//!   whose labels and text come from the evaluation's own USFM reader
//!   (`eval_support/usfm_text.rs`), never from the app's importer or its
//!   USFM reader: every published verse is kept with its numbers as written
//!   and its exact text (notes removed without a space put in their place),
//!   what the 66-book canon cannot hold (verse 0, chapters above its counts,
//!   other books, unusual labels, a number covered twice) is counted and
//!   listed as unsupported, never dropped, and each verse's text digest is
//!   reported for the scorer to compare with the check's own reading;
//! * **every verse is classed** by the strict fidelity rule
//!   (`eval_support/fidelity.rs`): exact, canonically equivalent (NFC),
//!   layout (white-space runs only) or mismatch. Every mismatch is wrong
//!   text, whatever its kind (case, word boundary, punctuation, marks...);
//! * **every verified reference** of the document is looked up in the
//!   reference's complete inventory: one it does not have (a verse, a whole
//!   chapter, a book), or one it cannot validate (a number it gives twice),
//!   is listed.
//!
//! Each report states its rules (`evaluation_schema`, `fidelity_rules`,
//! `inventory_rules`); the scorer refuses any other.

// Shared with tests/evaluation_rules.rs and tests/evaluation_references.rs,
// which test every item.
#[allow(dead_code)]
#[path = "eval_support/fidelity.rs"]
mod fidelity;
#[allow(dead_code)]
#[path = "eval_support/reference.rs"]
mod reference;
#[allow(dead_code)]
#[path = "eval_support/usfm_text.rs"]
mod usfm_text;

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Instant;

use abiding_word_lib::scripture::canon::{self, NON_CANONICAL_USFM_CODES};
use abiding_word_lib::scripture::import::{self, ImportOrigin, Severity};
use fidelity::Class;

/// The report format the scorer requires (`scripts/evaluation/score-set.py`).
const EVALUATION_SCHEMA: &str = "aw-eval-3";

/// The Unicode version of this harness's normalisation (NFC), as
/// "major.minor.update"; the scorer states its own beside it.
fn unicode_version() -> String {
    let (a, b, c) = unicode_normalization::UNICODE_VERSION;
    format!("{a}.{b}.{c}")
}

fn folder() -> Option<PathBuf> {
    std::env::var_os("AW_DOCUMENT_EDITIONS").map(PathBuf::from)
}

/// Letters and digits only, in lower case: used only to check that a
/// verified verse's kept place in the document holds its opening words
/// (the location check), never to judge its text.
fn letters(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
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
    /// "withheld-b", "withheld-c", "withheld-d" (each independent until its one evaluation).
    role: String,
    /// The books the document holds (None: the whole edition).
    books: Option<Vec<String>>,
    /// From a sealed set's own manifest (`AW_CORPUS_MANIFEST`): its `books`
    /// column is the only book selection; a file name never is (E2, C3).
    sealed: bool,
}

/// The corpus manifests: the development and regression editions, and the
/// withheld sets (v0.24.3), each a TSV with the same first nine columns
/// (the withheld sets add the books each document holds, its checksum and
/// the program that made it).
fn corpus() -> Vec<Edition> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/corpus");
    let mut out = Vec::new();
    for (name, sealed) in [
        "editions.tsv",
        "withheld-b.tsv",
        "withheld-c.tsv",
        "withheld-d.tsv",
        "recon-d.tsv",
        "recon-bc.tsv",
        "withheld-e.tsv",
        "withheld-f.tsv",
        "withheld-g.tsv",
    ]
    .into_iter()
    .map(|name| (dir.join(name), false))
    // AW_CORPUS_MANIFEST=<file> (Phase A1.5): one more manifest, from
    // anywhere: a sealed set's own, kept outside the repository.
    .chain(std::env::var_os("AW_CORPUS_MANIFEST").map(|m| (PathBuf::from(m), true)))
    {
        let Ok(text) = std::fs::read_to_string(&name) else {
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
                sealed,
            });
        }
    }
    out
}

/// Counts for one book of the reference (or all of them, `ALL`). Every
/// scorable printed verse is in exactly one of `verified`, `held`,
/// `refused` and `not_found` (`accounting`); its text in exactly one
/// fidelity class.
#[derive(Default, Clone, serde::Serialize)]
struct BookAudit {
    code: String,
    /// Scorable verses with text in the reference: the scored denominator.
    expected: usize,
    /// Scorable verses the reference numbers but leaves empty.
    absent_in_edition: usize,
    /// Printed verses the document has a verse for.
    detected: usize,
    /// Of those, in chapters the recogniser accepts.
    usable: usize,
    /// Not found, or in a refused chapter.
    unmapped: usize,
    not_found: usize,
    /// Detected in a chapter the recogniser refused.
    refused: usize,
    chapters: usize,
    usable_chapters: usize,
    refused_reading_order: usize,
    refused_verse_numbers: usize,
    refused_other: usize,
    /// Verses missing from a chapter that is otherwise usable.
    missing_in_usable_chapter: usize,
    /// Usable verses whose text the app verifies: what it uses as Scripture.
    verified: usize,
    verified_exact: usize,
    verified_canonical: usize,
    verified_layout: usize,
    /// Verified verses whose text is wrong under the strict rule.
    verified_mismatch: usize,
    verified_mismatch_kinds: BTreeMap<String, usize>,
    /// Usable verses held back as unverified ("recognised but uncertain"):
    /// their text was in fact faithful (caution's cost) or not (caught).
    held: usize,
    held_faithful: usize,
    held_mismatch: usize,
    /// Verses in refused chapters, by their text.
    refused_faithful: usize,
    refused_mismatch: usize,
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
        self.not_found += b.not_found;
        self.refused += b.refused;
        self.chapters += b.chapters;
        self.usable_chapters += b.usable_chapters;
        self.refused_reading_order += b.refused_reading_order;
        self.refused_verse_numbers += b.refused_verse_numbers;
        self.refused_other += b.refused_other;
        self.missing_in_usable_chapter += b.missing_in_usable_chapter;
        self.verified += b.verified;
        self.verified_exact += b.verified_exact;
        self.verified_canonical += b.verified_canonical;
        self.verified_layout += b.verified_layout;
        self.verified_mismatch += b.verified_mismatch;
        for (k, n) in &b.verified_mismatch_kinds {
            *self.verified_mismatch_kinds.entry(k.clone()).or_default() += n;
        }
        self.held += b.held;
        self.held_faithful += b.held_faithful;
        self.held_mismatch += b.held_mismatch;
        self.refused_faithful += b.refused_faithful;
        self.refused_mismatch += b.refused_mismatch;
        self.located += b.located;
        self.location_checked += b.location_checked;
    }
}

/// A usable verse whose text is not byte-exact: both texts as they are, so
/// that the scorer can class it again on its own.
#[derive(serde::Serialize)]
struct FidelityRecord {
    reference: String,
    /// "verified" or "held".
    status: &'static str,
    unverified_reason: Option<&'static str>,
    class: &'static str,
    kind: Option<&'static str>,
    /// For a material mismatch: another verse of the reference with these
    /// words (any book, any status).
    matched: Option<String>,
    page: Option<u32>,
    found: String,
    edition: String,
}

/// A verified reference the reference edition does not have, or cannot
/// validate.
#[derive(serde::Serialize)]
struct AbsentReference {
    reference: String,
    reason: &'static str,
}

/// How the books the document is measured against were chosen.
#[derive(serde::Serialize)]
struct BookSelection {
    /// "manifest" (the books column), "all", or "file name" (only for the
    /// disclosed corpus's own manifests, never for a sealed set's).
    rule: &'static str,
    books: Option<Vec<String>>,
    /// Codes in the books column that are no USFM book identifier.
    unknown_codes: Vec<String>,
}

/// The app's own importer on the same reference, for comparison only: what
/// it would have kept that the inventory scores differently. Never used to
/// score.
#[derive(Default, serde::Serialize)]
struct ImporterComparison {
    error: Option<String>,
    agree: usize,
    only_importer: usize,
    only_inventory: usize,
    text_differs: usize,
    examples: Vec<String>,
    issue_codes: BTreeMap<String, usize>,
}

#[derive(serde::Serialize)]
struct Accounting {
    printed_scorable: usize,
    verified: usize,
    held: usize,
    refused: usize,
    not_found: usize,
    holds: bool,
}

#[derive(serde::Serialize)]
struct EditionAudit {
    evaluation_schema: &'static str,
    fidelity_rules: &'static str,
    inventory_rules: &'static str,
    reader_rules: &'static str,
    unicode_version: String,
    edition: Edition,
    reference: String,
    book_selection: BookSelection,
    reference_inventory: reference::InventoryReport,
    importer_comparison: ImporterComparison,
    accounting: Accounting,
    totals: BookAudit,
    books: Vec<BookAudit>,
    /// Books of the edition of which no verse was found.
    books_not_found: Vec<String>,
    issues: BTreeMap<String, usize>,
    refused_chapters: Vec<String>,
    /// Every usable verse whose text is not byte-exact.
    fidelity_records: Vec<FidelityRecord>,
    /// Every verified verse of the document, in its own numbering.
    verified_references: Vec<String>,
    /// Verified references the edition does not have, with why.
    verified_absent: Vec<AbsentReference>,
    /// Verified references the edition has only ambiguously (a number covered
    /// twice, a book given twice, a number inside a bridge): they cannot be
    /// validated.
    verified_unvalidatable: Vec<AbsentReference>,
    /// Verified verses numbered but empty both here and in the edition.
    verified_empty_in_both: usize,
    /// Usable but unverified verses the edition does not have (diagnostic).
    usable_absent: Vec<String>,
    unverified_reasons: BTreeMap<String, usize>,
    location_misses: Vec<String>,
    apparatus: usize,
    pages_uncertain: usize,
    pages: usize,
    found_books: Vec<(String, Option<String>, Option<u32>)>,
    errors: Vec<String>,
    seconds: f64,
    peak_memory_mb: Option<f64>,
    /// The reading alone (Phase A1.2): time and peak resident memory when
    /// `stage_documents` returns, before this harness writes any report.
    /// `seconds` and `peak_memory_mb` also cover writing the structural
    /// provenance report, which holds the whole report as JSON at once.
    read_seconds: f64,
    read_peak_memory_mb: Option<f64>,
    /// Where the document's printed numbering departs from the reference
    /// (Phase A1.5, `scripture::numbering`), per book.
    numbering: Vec<abiding_word_lib::scripture::numbering::BookNumbering>,
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

/// The structure-aware reading's report for an edition (Phase A1), kept
/// compact: the counts; every gated verse; the full provenance of every
/// verse not traced exactly and of a sample; and for every verse its status,
/// spans, excluded ranges and id check.
fn write_structural(
    edition: &Edition,
    report: &abiding_word_lib::scripture::structural::StructuralReport,
) {
    use serde_json::{json, Value};
    let Some(out) = std::env::var_os("AW_AUDIT_OUT").map(PathBuf::from) else {
        return;
    };
    let out = out.join("structural");
    std::fs::create_dir_all(&out).unwrap();
    let full = serde_json::to_value(report).unwrap();
    let provenance = full["provenance"].as_object().cloned().unwrap_or_default();
    let mut verses = serde_json::Map::new();
    let mut detailed = serde_json::Map::new();
    for (n, (key, p)) in provenance.iter().enumerate() {
        let status = p["status"].as_str().unwrap_or("");
        verses.insert(
            key.clone(),
            json!([
                status,
                p["spans"].as_array().map_or(0, Vec::len),
                p["excluded"].as_array().map_or(0, Vec::len),
                p["idCheck"].clone()
            ]),
        );
        // Every verse something was removed from is kept whole: its
        // removed ranges are the evidence for verifying it (Phase A1.1).
        let removed = p["excluded"]
            .as_array()
            .is_some_and(|x| x.iter().any(|r| r["kind"] == "apparatus"));
        if status != "exact" || n < 25 || removed || report.gated.contains_key(key) {
            detailed.insert(key.clone(), p.clone());
        }
    }
    let mut summary = full.clone();
    if let Value::Object(m) = &mut summary {
        m.remove("provenance");
        m.insert("verses".into(), Value::Object(verses));
        m.insert("detailed".into(), Value::Object(detailed));
    }
    std::fs::write(
        out.join(format!("{}.json", edition.key)),
        serde_json::to_string(&summary).unwrap(),
    )
    .unwrap();
}

/// What the recogniser made of a document that has no reference text (a
/// control: a document with no Scripture, or one whose quotations are
/// judged by hand). References only, never the text.
fn control(dir: &Path, edition: &Edition) -> Option<serde_json::Value> {
    let files: Vec<PathBuf> = edition.files.iter().map(|f| dir.join(f)).collect();
    if let Some(missing) = files.iter().find(|f| !f.exists()) {
        eprintln!("{}: {} missing, skipped", edition.key, missing.display());
        return None;
    }
    reset_peak();
    let started = Instant::now();
    let structural = std::env::var("AW_PIPELINE").is_ok_and(|p| p == "structural");
    let staged = if structural {
        abiding_word_lib::scripture::structural::stage_documents(&files, ImportOrigin::UserFiles)
            .map(|(staged, _)| staged)
    } else {
        import::stage_documents(&files, ImportOrigin::UserFiles)
    };
    let seconds = started.elapsed().as_secs_f64();
    let peak = peak_mb();
    let base = serde_json::json!({
        "evaluation_schema": EVALUATION_SCHEMA,
        "key": edition.key, "control": true, "format": edition.format,
        "language": edition.language, "script": edition.script,
        "seconds": seconds, "peak_memory_mb": peak,
    });
    let mut out = base.as_object().cloned().unwrap();
    match staged {
        Err(e) => {
            out.insert("refused_document".into(), format!("{e:?}").into());
            out.insert("detected".into(), 0.into());
            out.insert("verified".into(), 0.into());
        }
        Ok(staged) => {
            let usable = staged.content.usable_chapters().unwrap_or_default();
            let stage = staged.documents.as_ref().expect("a document stage");
            let (mut detected, mut verified) = (0usize, Vec::new());
            for (ordinal, book) in &staged.content.books {
                let code = canon::by_ordinal(*ordinal).map_or("?", |b| b.code);
                for (chapter, c) in &book.chapters {
                    for verse in c.verses.keys() {
                        detected += 1;
                        if usable.contains(&(*ordinal, *chapter))
                            && !stage.unverified.contains_key(&(*ordinal, *chapter, *verse))
                        {
                            verified.push(format!("{code} {chapter}:{verse}"));
                        }
                    }
                }
            }
            out.insert("detected".into(), detected.into());
            out.insert("verified".into(), verified.len().into());
            out.insert("verified_references".into(), verified.into());
            out.insert(
                "errors".into(),
                staged
                    .content
                    .issues
                    .iter()
                    .filter(|i| i.severity == Severity::Error)
                    .count()
                    .into(),
            );
        }
    }
    eprintln!(
        "{}: control, detected {}, verified {}",
        edition.key, out["detected"], out["verified"]
    );
    Some(serde_json::Value::Object(out))
}

/// The books a document is measured against (E2, C3). A sealed set's
/// manifest is the only source: `all`, or the codes listed. Only the
/// disclosed corpus's own manifests keep the A1.5 rule of a one-book file
/// name ("fraLSG_LUK.pdf"), and the report says when it was used.
fn book_selection(edition: &Edition) -> BookSelection {
    let known = |c: &str| canon::by_code(c).is_some() || NON_CANONICAL_USFM_CODES.contains(&c);
    if let Some(codes) = &edition.books {
        let books: Vec<String> = codes.iter().map(|c| c.to_ascii_uppercase()).collect();
        let unknown_codes = books.iter().filter(|c| !known(c)).cloned().collect();
        return BookSelection {
            rule: "manifest",
            books: Some(books),
            unknown_codes,
        };
    }
    if !edition.sealed {
        if let Some(book) = edition
            .files
            .first()
            .and_then(|f| f.rsplit_once('_'))
            .and_then(|(_, rest)| rest.split('.').next())
            .and_then(canon::by_code)
        {
            return BookSelection {
                rule: "file name",
                books: Some(vec![book.code.to_string()]),
                unknown_codes: Vec::new(),
            };
        }
    }
    BookSelection {
        rule: "all",
        books: None,
        unknown_codes: Vec::new(),
    }
}

/// The app's importer on the same reference, compared with the inventory.
fn importer_comparison(path: &Path, inventory: &reference::Inventory) -> ImporterComparison {
    let mut out = ImporterComparison::default();
    let staged = match import::stage(&[as_usfm_defines(path)], ImportOrigin::UserFiles) {
        Ok(staged) => staged,
        Err(e) => {
            out.error = Some(format!("{e:?}"));
            return out;
        }
    };
    for issue in &staged.content.issues {
        *out.issue_codes
            .entry(issue.code.clone().unwrap_or_else(|| "-".into()))
            .or_default() += 1;
    }
    let mut seen = BTreeSet::new();
    for (ordinal, book) in &staged.content.books {
        let code = canon::by_ordinal(*ordinal).map_or("?", |b| b.code);
        for (chapter, c) in &book.chapters {
            for (verse, text) in &c.verses {
                let key = (*ordinal, *chapter, *verse);
                seen.insert(key);
                match inventory.text(key) {
                    Some(t) if t == text => out.agree += 1,
                    Some(_) => {
                        out.text_differs += 1;
                        if out.examples.len() < 20 {
                            out.examples
                                .push(format!("{code} {chapter}:{verse}: text differs"));
                        }
                    }
                    None => {
                        out.only_importer += 1;
                        if out.examples.len() < 20 {
                            out.examples
                                .push(format!("{code} {chapter}:{verse}: only the importer"));
                        }
                    }
                }
            }
        }
    }
    for key in inventory.scorable.keys() {
        if !seen.contains(key) {
            out.only_inventory += 1;
            if out.examples.len() < 20 {
                let code = canon::by_ordinal(key.0).map_or("?", |b| b.code);
                out.examples
                    .push(format!("{code} {}:{}: only the inventory", key.1, key.2));
            }
        }
    }
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
    // A document the app refuses to read at all (a PDF whose text cannot be
    // extracted, say) is a rejected format: recorded, never a crash of the
    // report (v0.24.4).
    // AW_PIPELINE=structural (Phase A1): the structure-aware reading
    // (`scripture::structural`), with its provenance report written next to
    // the edition's report. Unset: the frozen reading, exactly as before.
    let structural = std::env::var("AW_PIPELINE").is_ok_and(|p| p == "structural");
    let read_done = std::cell::Cell::new((0.0f64, None));
    let staged = if structural {
        abiding_word_lib::scripture::structural::stage_documents(&files, ImportOrigin::UserFiles)
            .map(|(staged, report)| {
                read_done.set((started.elapsed().as_secs_f64(), peak_mb()));
                write_structural(edition, &report);
                staged
            })
    } else {
        let staged = import::stage_documents(&files, ImportOrigin::UserFiles);
        read_done.set((started.elapsed().as_secs_f64(), peak_mb()));
        staged
    };
    let (read_seconds, read_peak_memory_mb) = read_done.get();
    let staged = match staged {
        Ok(staged) => staged,
        Err(error) => {
            let message = format!("{error:?}");
            eprintln!("{}: not read: {message}", edition.key);
            if let Some(out) = std::env::var_os("AW_AUDIT_OUT") {
                let out = PathBuf::from(out);
                std::fs::create_dir_all(&out).unwrap();
                std::fs::write(
                    out.join(format!("{}.rejected.md", edition.key)),
                    format!(
                        "# {} — not read\n\n{}\n\nThe app refused the document: {message}\n",
                        edition.key, edition.layout
                    ),
                )
                .unwrap();
            }
            return None;
        }
    };
    let seconds = started.elapsed().as_secs_f64();
    let peak_memory_mb = peak_mb();
    // Only now is the publisher's text read, for the comparison: by the
    // neutral inventory, never through the importer (which is run beside it
    // for comparison only).
    let selection = book_selection(edition);
    let declared: Option<BTreeSet<String>> = selection
        .books
        .as_ref()
        .map(|b| b.iter().cloned().collect());
    let inventory = reference::read(&reference_file, declared.as_ref());
    let importer = importer_comparison(&reference_file, &inventory);
    // Every verse of the reference by its letters (any book, any status): a
    // found verse with another verse's words has the wrong reference.
    let mut by_letters: HashMap<String, String> = HashMap::new();
    for e in &inventory.entries {
        let b = fidelity::base(&fidelity::nfc(&e.text));
        if !b.is_empty() {
            by_letters.entry(b).or_insert_with(|| e.label());
        }
    }

    let content = &staged.content;
    let numbering = abiding_word_lib::scripture::numbering::assess(content);
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

    // The reference side: every scorable verse, by book.
    let mut by_book: BTreeMap<u8, BookAudit> = BTreeMap::new();
    let mut chapters_seen: BTreeSet<(u8, u16)> = BTreeSet::new();
    let mut records = Vec::new();
    let mut location_misses: Vec<String> = Vec::new();
    for (&key, &index) in &inventory.scorable {
        let (ordinal, chapter, verse) = key;
        let book = canon::by_ordinal(ordinal).expect("a canon book");
        let a = by_book.entry(ordinal).or_insert_with(|| BookAudit {
            code: book.code.to_string(),
            ..Default::default()
        });
        let fch = content
            .books
            .get(&ordinal)
            .and_then(|b| b.chapters.get(&chapter));
        let ok = usable.contains(&(ordinal, chapter));
        if chapters_seen.insert((ordinal, chapter)) {
            if fch.is_some() {
                a.chapters += 1;
            }
            if ok {
                a.usable_chapters += 1;
            } else if let Some(r) = refused.get(&(book.code.to_string(), chapter)) {
                match *r {
                    "order" => a.refused_reading_order += 1,
                    "numbers" => a.refused_verse_numbers += 1,
                    _ => a.refused_other += 1,
                }
            }
        }
        let want = inventory.entries[index].text.as_str();
        if want.trim().is_empty() {
            a.absent_in_edition += 1;
            continue;
        }
        a.expected += 1;
        let Some(text) = fch.and_then(|c| c.verses.get(&verse)) else {
            a.not_found += 1;
            a.unmapped += 1;
            if ok {
                a.missing_in_usable_chapter += 1;
            }
            continue;
        };
        a.detected += 1;
        let class = fidelity::classify(want, text);
        if !ok {
            a.refused += 1;
            a.unmapped += 1;
            if class.faithful() {
                a.refused_faithful += 1;
            } else {
                a.refused_mismatch += 1;
            }
            continue;
        }
        a.usable += 1;
        let unverified = stage.unverified.get(&key).copied();
        let label = format!("{} {chapter}:{verse}", book.code);
        let (kind, matched) = if class == Class::Mismatch {
            let other = by_letters
                .get(&fidelity::base(&fidelity::nfc(text)))
                .filter(|l| **l != label)
                .cloned();
            (Some(fidelity::kind(want, text, other.is_some())), other)
        } else {
            (None, None)
        };
        if unverified.is_some() {
            a.held += 1;
            if class.faithful() {
                a.held_faithful += 1;
            } else {
                a.held_mismatch += 1;
            }
        } else {
            a.verified += 1;
            match class {
                Class::Exact => a.verified_exact += 1,
                Class::Canonical => a.verified_canonical += 1,
                Class::Layout => a.verified_layout += 1,
                Class::Mismatch => {
                    a.verified_mismatch += 1;
                    *a.verified_mismatch_kinds
                        .entry(kind.unwrap_or("changed").to_string())
                        .or_default() += 1;
                }
            }
            if let Some(found_here) = located(key, text) {
                a.location_checked += 1;
                if found_here {
                    a.located += 1;
                } else if location_misses.len() < 20 {
                    location_misses.push(format!(
                        "{label} (page {:?}, segments {:?}–{:?}): «{}»",
                        stage.origins.get(&key).and_then(|o| o.start.page),
                        stage.origins.get(&key).and_then(|o| o.start.segment),
                        stage.origins.get(&key).and_then(|o| o.end.segment),
                        text.chars().take(60).collect::<String>()
                    ));
                }
            }
        }
        if class != Class::Exact {
            records.push(FidelityRecord {
                reference: label,
                status: if unverified.is_some() {
                    "held"
                } else {
                    "verified"
                },
                unverified_reason: unverified,
                class: class.as_str(),
                kind,
                matched,
                page: origins.get(&key).copied().flatten(),
                found: text.clone(),
                edition: want.to_string(),
            });
        }
    }

    // The document side, in full: every verified reference is looked up in
    // the reference's complete inventory.
    let mut verified_references = Vec::new();
    let mut verified_absent = Vec::new();
    let mut verified_unvalidatable = Vec::new();
    let mut usable_absent = Vec::new();
    let mut verified_empty_in_both = 0usize;
    let reference_books: BTreeSet<u8> = inventory.scorable.keys().map(|k| k.0).collect();
    let reference_chapters: BTreeSet<(u8, u16)> =
        inventory.scorable.keys().map(|k| (k.0, k.1)).collect();
    for (ordinal, fbook) in &content.books {
        let code = canon::by_ordinal(*ordinal).map_or("?", |b| b.code);
        for (chapter, ch) in &fbook.chapters {
            if !usable.contains(&(*ordinal, *chapter)) {
                continue;
            }
            for (verse, text) in &ch.verses {
                let key = (*ordinal, *chapter, *verse);
                let verified = !stage.unverified.contains_key(&key);
                let label = format!("{code} {chapter}:{verse}");
                if verified {
                    verified_references.push(label.clone());
                }
                let why: Option<(&'static str, bool)> = match inventory.scorable.get(&key) {
                    Some(&i) => {
                        // Numbered but empty in the edition, given text here.
                        let empty = inventory.entries[i].text.trim().is_empty();
                        if empty && text.trim().is_empty() && verified {
                            verified_empty_in_both += 1;
                        }
                        (empty && !text.trim().is_empty()).then_some(("empty_in_edition", false))
                    }
                    None => Some(if let Some(why) = inventory.unscorable_keys.get(&key) {
                        (*why, true)
                    } else if inventory.bridge_keys.contains(&key) {
                        ("inside_bridge", true)
                    } else if inventory.outside_keys.contains(&key) {
                        ("outside_declared_books", false)
                    } else if !reference_books.contains(ordinal) {
                        ("book_not_in_reference", false)
                    } else if !reference_chapters.contains(&(*ordinal, *chapter)) {
                        ("chapter_not_in_book", false)
                    } else {
                        ("verse_not_in_chapter", false)
                    }),
                };
                let Some((reason, unvalidatable)) = why else {
                    continue;
                };
                if !verified {
                    usable_absent.push(format!("{label} ({reason})"));
                } else if unvalidatable {
                    verified_unvalidatable.push(AbsentReference {
                        reference: label,
                        reason,
                    });
                } else {
                    verified_absent.push(AbsentReference {
                        reference: label,
                        reason,
                    });
                }
            }
        }
    }

    let books: Vec<BookAudit> = by_book.into_values().collect();
    let mut totals = BookAudit {
        code: "ALL".into(),
        ..Default::default()
    };
    for b in &books {
        totals.add(b);
    }
    let books_not_found = books
        .iter()
        .filter(|b| b.detected == 0 && b.expected > 0)
        .map(|b| b.code.clone())
        .collect();
    let accounting = Accounting {
        printed_scorable: totals.expected,
        verified: totals.verified,
        held: totals.held,
        refused: totals.refused,
        not_found: totals.not_found,
        holds: totals.expected == totals.verified + totals.held + totals.refused + totals.not_found,
    };
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
    usable_absent.truncate(200);
    Some(EditionAudit {
        evaluation_schema: EVALUATION_SCHEMA,
        fidelity_rules: fidelity::RULES,
        inventory_rules: reference::RULES,
        reader_rules: usfm_text::RULES,
        unicode_version: unicode_version(),
        edition: edition.clone(),
        reference: reference_file
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned(),
        book_selection: selection,
        reference_inventory: inventory.report(),
        importer_comparison: importer,
        accounting,
        totals,
        books,
        books_not_found,
        issues,
        refused_chapters,
        fidelity_records: records,
        verified_references,
        verified_absent,
        verified_unvalidatable,
        verified_empty_in_both,
        usable_absent,
        unverified_reasons,
        location_misses,
        apparatus: stage.apparatus.len(),
        pages_uncertain,
        pages,
        found_books,
        errors,
        seconds,
        peak_memory_mb,
        read_seconds,
        read_peak_memory_mb,
        numbering,
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
    let inv = &a.reference_inventory;
    let _ = writeln!(s, "# {} — {}\n", e.key, e.files.join(", "));
    let _ = writeln!(
        s,
        "Language {} ({}), format {}, layout: {}. Compared with {} (same revision), rules {} / {} / {} / {} (Unicode {}).\n",
        e.language,
        e.script,
        e.format,
        e.layout,
        a.reference,
        a.evaluation_schema,
        a.fidelity_rules,
        a.inventory_rules,
        a.reader_rules,
        a.unicode_version
    );
    let _ = writeln!(
        s,
        "Books measured: {} ({}).{}\n",
        a.book_selection
            .books
            .as_ref()
            .map_or("all".into(), |b| b.join(", ")),
        a.book_selection.rule,
        if a.book_selection.unknown_codes.is_empty() {
            String::new()
        } else {
            format!(
                " **Unknown book codes:** {}.",
                a.book_selection.unknown_codes.join(", ")
            )
        }
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
    if !inv.problems.is_empty() {
        let _ = writeln!(
            s,
            "**Reference problems (not a sound USFM reference):** {}\n",
            inv.problems.join("; ")
        );
    }
    let _ = writeln!(s, "## The reference, in full\n");
    let _ = writeln!(s, "| | Verses |\n|---|---:|");
    let _ = writeln!(s, "| Labels read | {} |", inv.labels);
    let _ = writeln!(
        s,
        "| **Printed, scorable** | **{}** |",
        inv.printed_scorable
    );
    let _ = writeln!(
        s,
        "| Numbered but empty (absent in the edition) | {} |",
        inv.absent_in_edition
    );
    let _ = writeln!(
        s,
        "| **Printed, unsupported (counted, never scored)** | **{}** |",
        inv.printed_unsupported
    );
    for (class, c) in &inv.unsupported {
        if c.printed + c.empty > 0 {
            let _ = writeln!(
                s,
                "| … {class}: {} printed, {} empty | {} |",
                c.printed,
                c.empty,
                c.references
                    .iter()
                    .take(12)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
                    + if c.references.len() > 12 { ", …" } else { "" }
            );
        }
    }
    let _ = writeln!(
        s,
        "| Outside the declared books: {} printed, {} empty | {} |",
        inv.outside_declared_scope.printed,
        inv.outside_declared_scope.empty,
        inv.outside_declared_scope.references.join(", ")
    );
    let _ = writeln!(s, "\n## The document against it\n");
    let _ = writeln!(s, "| | Verses | Share of printed |\n|---|---:|---:|");
    let _ = writeln!(s, "| Printed, scorable | {} | |", t.expected);
    let _ = writeln!(
        s,
        "| **Verified: exact / canonical / layout** | **{} / {} / {}** | {} |",
        t.verified_exact,
        t.verified_canonical,
        t.verified_layout,
        pct(
            t.verified_exact + t.verified_canonical + t.verified_layout,
            t.expected
        )
    );
    let kinds: Vec<String> = t
        .verified_mismatch_kinds
        .iter()
        .map(|(k, n)| format!("{k} {n}"))
        .collect();
    let _ = writeln!(
        s,
        "| **Verified with wrong text (mismatch)** | **{}** | {} |",
        t.verified_mismatch,
        kinds.join(", ")
    );
    let _ = writeln!(
        s,
        "| Recognised but uncertain (held back): faithful / mismatch | {} / {} | |",
        t.held_faithful, t.held_mismatch
    );
    let _ = writeln!(
        s,
        "| In refused chapters: faithful / mismatch | {} / {} | |",
        t.refused_faithful, t.refused_mismatch
    );
    let _ = writeln!(s, "| Not found in the document | {} | |", t.not_found);
    let _ = writeln!(
        s,
        "| Accounting (printed = verified + held + refused + not found) | {} | |",
        if a.accounting.holds {
            "holds"
        } else {
            "**BROKEN**"
        }
    );
    let _ = writeln!(
        s,
        "| **Verified references the edition does not have** | **{}** | |",
        a.verified_absent.len()
    );
    let _ = writeln!(
        s,
        "| **Verified references the edition cannot validate** | **{}** | |",
        a.verified_unvalidatable.len()
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
        "\nChapters: {} found, {} usable; refused for reading order {}, for verse numbers {}, other {}.\n",
        t.chapters, t.usable_chapters, t.refused_reading_order, t.refused_verse_numbers, t.refused_other
    );
    if !a.books_not_found.is_empty() {
        let _ = writeln!(s, "Books not found: {}.\n", a.books_not_found.join(", "));
    }
    let ic = &a.importer_comparison;
    let _ = writeln!(
        s,
        "The app's importer on the same reference (comparison only): {} agree, {} only in the importer, {} only in the inventory, {} with other text{}.\n",
        ic.agree,
        ic.only_importer,
        ic.only_inventory,
        ic.text_differs,
        ic.error
            .as_ref()
            .map(|e| format!("; it refused the reference: {e}"))
            .unwrap_or_default()
    );
    let _ = writeln!(s, "## Per book\n");
    let _ = writeln!(
        s,
        "| Book | Printed | Verified faithful | Verified mismatch | Held | Refused | Not found |\n|---|---:|---:|---:|---:|---:|---:|"
    );
    for b in &a.books {
        let _ = writeln!(
            s,
            "| {} | {} | {} | {} | {} | {} | {} |",
            b.code,
            b.expected,
            b.verified_exact + b.verified_canonical + b.verified_layout,
            b.verified_mismatch,
            b.held,
            b.refused,
            b.not_found
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
    if !a.verified_absent.is_empty() || !a.verified_unvalidatable.is_empty() {
        let _ = writeln!(
            s,
            "\n## Verified references the edition does not have or cannot validate\n"
        );
        for r in a.verified_absent.iter().chain(&a.verified_unvalidatable) {
            let _ = writeln!(s, "- {} ({})", r.reference, r.reason);
        }
    }
    if !a.usable_absent.is_empty() {
        let _ = writeln!(
            s,
            "\n## Usable, unverified verses the edition does not have\n\n{}",
            a.usable_absent.join(", ")
        );
    }
    if !a.location_misses.is_empty() {
        let _ = writeln!(
            s,
            "\n## Verified verses whose kept place does not hold them (first 20)\n"
        );
        for m in &a.location_misses {
            let _ = writeln!(s, "- {m}");
        }
    }
    let _ = writeln!(s, "\n## Usable verses whose text is not exact\n");
    for d in &a.fidelity_records {
        let _ = writeln!(
            s,
            "- {} {}{} [{}{}] (page {}): found «{}» — edition «{}»",
            d.reference,
            if d.status == "verified" {
                "**verified**"
            } else {
                "held"
            },
            d.unverified_reason
                .map(|r| format!(" (unverified: {r})"))
                .unwrap_or_default(),
            d.class,
            d.kind.map(|k| format!(": {k}")).unwrap_or_default(),
            d.page.map_or("?".into(), |p| p.to_string()),
            fidelity::runs(&d.found),
            fidelity::runs(&d.edition)
        );
    }
    s
}

fn summary_row(a: &EditionAudit) -> String {
    let t = &a.totals;
    let e = &a.edition;
    format!(
        "| {} | {} | {} | {} | {} | {} | {} | {} / {} / {} | {} | {} / {} | {} / {} | {} | {:.0} s | {} |",
        e.key,
        e.language,
        e.format,
        e.layout,
        t.expected,
        a.reference_inventory.printed_unsupported,
        t.verified,
        t.verified_exact,
        t.verified_canonical,
        t.verified_layout,
        t.verified_mismatch,
        a.verified_absent.len(),
        a.verified_unvalidatable.len(),
        t.held_faithful,
        t.held_mismatch,
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
        // A document with no reference text ("-" in the manifest): a
        // control. Nothing in it is compared; what was verified is counted.
        if edition.reference.is_none() {
            if let (Some(c), Some(out)) = (control(&dir, &edition), &out) {
                std::fs::create_dir_all(out).unwrap();
                std::fs::write(
                    out.join(format!("{}.control.json", edition.key)),
                    serde_json::to_string_pretty(&c).unwrap(),
                )
                .unwrap();
            }
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
        "| Edition | Lang | Format | Layout | Printed (scorable) | Printed (unsupported) | Verified | Exact / canonical / layout | Verified mismatch | Not in the edition / unvalidatable | Held: faithful / mismatch | Location | Time | Peak memory |\n|---|---|---|---|---|---|---|---|---|---|---|---|---|---|\n{}\n",
        rows.join("\n")
    );
    eprintln!("{table}");
    if let Some(out) = &out {
        std::fs::write(out.join("summary.md"), table).unwrap();
    }
}
