//! Accuracy of Bibles read from documents on real, typeset editions (v0.24,
//! audit v0.24.1). Not run by default: set `AW_DOCUMENT_EDITIONS` to a folder
//! holding eBible.org PDFs of editions whose USFM Abiding Word ships
//! (`scripts/fetch-real-editions.sh` downloads and checks them), then
//!
//! ```text
//! cargo test --release --test real_document_editions -- --ignored --nocapture
//! ```
//!
//! Set `AW_AUDIT_OUT` to a folder to also write `<edition>.json` (machine-
//! readable) and `<edition>.md` (human-readable) audits there.
//!
//! The PDF is read exactly as a person's file would be: the recogniser never
//! sees the USFM. Only afterwards is every verse recognised compared with the
//! publisher's own USFM text of the same edition, and classified:
//!
//! * **exact** — the same characters (outer whitespace aside);
//! * **normalised** — equal once whitespace is collapsed and removed between
//!   punctuation (a thin space printed between two quotation marks) — the
//!   same letters, digits and punctuation in the same order;
//! * **different** — anything else: a genuine textual difference.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

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

#[derive(Default, Clone, serde::Serialize)]
struct BookAudit {
    code: String,
    expected: usize,
    detected: usize,
    usable: usize,
    unmapped: usize,
    chapters: usize,
    usable_chapters: usize,
    refused_reading_order: usize,
    refused_verse_numbers: usize,
    refused_other: usize,
    exact: usize,
    normalised: usize,
    different: usize,
    usable_different: usize,
}

#[derive(serde::Serialize)]
struct Difference {
    reference: String,
    usable: bool,
    page: Option<u32>,
    found: String,
    edition: String,
}

#[derive(serde::Serialize)]
struct EditionAudit {
    file: String,
    edition: String,
    reference: String,
    totals: BookAudit,
    books: Vec<BookAudit>,
    issues: BTreeMap<String, usize>,
    refused_chapters: Vec<String>,
    differences: Vec<Difference>,
    pages_uncertain: usize,
    pages: usize,
    /// Book headings found: (heading, book, page).
    found_books: Vec<(String, Option<String>, Option<u32>)>,
    /// Errors naming no chapter, and the first errors naming one.
    errors: Vec<String>,
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

fn audit(pdf: &Path, id: &str) -> EditionAudit {
    let resources = Path::new(env!("CARGO_MANIFEST_DIR")).join("../resources/scripture");
    // The USFM to compare with: the same revision as the PDF when one is
    // given (`AW_REFERENCE_USFM/<id>_usfm.zip`, from eBible.org on the day the
    // PDF was downloaded), else the copy shipped with Abiding Word.
    let same_revision = std::env::var_os("AW_REFERENCE_USFM")
        .map(|d| PathBuf::from(d).join(format!("{id}_usfm.zip")))
        .filter(|p| p.exists());
    let reference_file = same_revision
        .clone()
        .unwrap_or_else(|| resources.join(id).join(format!("{id}_usfm.zip")));
    let reference = import::stage(
        std::slice::from_ref(&reference_file),
        ImportOrigin::UserFiles,
    )
    .unwrap();
    let staged = import::stage_documents(&[pdf.to_path_buf()], ImportOrigin::UserFiles).unwrap();
    let content = &staged.content;
    let usable = content.usable_chapters().unwrap_or_default();
    let origins = staged.document_origins();

    // Errors by chapter, for the refusal reasons.
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
    for (ordinal, rbook) in &reference.content.books {
        let book = canon::by_ordinal(*ordinal).unwrap();
        let mut a = BookAudit {
            code: book.code.to_string(),
            ..Default::default()
        };
        let found = content.books.get(ordinal);
        for (chapter, rch) in &rbook.chapters {
            a.expected += rch.verses.len();
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
                let Some(text) = fch.and_then(|c| c.verses.get(verse)) else {
                    a.unmapped += 1;
                    continue;
                };
                a.detected += 1;
                if ok {
                    a.usable += 1;
                } else {
                    a.unmapped += 1;
                }
                if want.trim() == text.trim() {
                    a.exact += 1;
                } else if squash(want) == squash(text) || dense(want) == dense(text) {
                    a.normalised += 1;
                } else {
                    a.different += 1;
                    if ok {
                        a.usable_different += 1;
                    }
                    if ok || differences.len() < 60 {
                        differences.push(Difference {
                            reference: format!("{} {chapter}:{verse}", book.code),
                            usable: ok,
                            page: origins
                                .get(&(*ordinal, *chapter, *verse))
                                .copied()
                                .flatten(),
                            found: squash(text),
                            edition: squash(want),
                        });
                    }
                }
            }
        }
        books.push(a);
    }
    let mut totals = BookAudit {
        code: "ALL".into(),
        ..Default::default()
    };
    for b in &books {
        totals.expected += b.expected;
        totals.detected += b.detected;
        totals.usable += b.usable;
        totals.unmapped += b.unmapped;
        totals.chapters += b.chapters;
        totals.usable_chapters += b.usable_chapters;
        totals.refused_reading_order += b.refused_reading_order;
        totals.refused_verse_numbers += b.refused_verse_numbers;
        totals.refused_other += b.refused_other;
        totals.exact += b.exact;
        totals.normalised += b.normalised;
        totals.different += b.different;
        totals.usable_different += b.usable_different;
    }
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
    EditionAudit {
        file: pdf.file_name().unwrap().to_string_lossy().into_owned(),
        edition: id.into(),
        reference: if same_revision.is_some() {
            format!(
                "{} (same revision as the PDF)",
                reference_file.file_name().unwrap().to_string_lossy()
            )
        } else {
            format!(
                "{} (bundled with Abiding Word)",
                reference_file.file_name().unwrap().to_string_lossy()
            )
        },
        totals,
        books,
        issues,
        refused_chapters,
        differences,
        pages_uncertain,
        pages,
        found_books,
        errors,
    }
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
    let _ = writeln!(s, "# {} ({})\n", a.file, a.edition);
    let _ = writeln!(s, "Compared with: {}.\n", a.reference);
    let _ = writeln!(
        s,
        "Pages: {} ({} with an uncertain reading order).\n",
        a.pages, a.pages_uncertain
    );
    let _ = writeln!(s, "| | Verses | Share |\n|---|---|---|");
    let _ = writeln!(s, "| Expected (edition's USFM) | {} | |", t.expected);
    let _ = writeln!(
        s,
        "| Detected | {} | {} |",
        t.detected,
        pct(t.detected, t.expected)
    );
    let _ = writeln!(
        s,
        "| Usable (partial use) | {} | {} |",
        t.usable,
        pct(t.usable, t.expected)
    );
    let _ = writeln!(
        s,
        "| Unmapped | {} | {} |",
        t.unmapped,
        pct(t.unmapped, t.expected)
    );
    let _ = writeln!(s, "| Exact | {} | |", t.exact);
    let _ = writeln!(s, "| Normalised only (whitespace) | {} | |", t.normalised);
    let _ = writeln!(
        s,
        "| Genuine differences (all detected) | {} | |",
        t.different
    );
    let _ = writeln!(
        s,
        "| Genuine differences in usable verses | {} | |",
        t.usable_different
    );
    let _ = writeln!(
        s,
        "\nChapters: {} found, {} usable; refused for reading order {}, for verse numbers {}, other {}.\n",
        t.chapters, t.usable_chapters, t.refused_reading_order, t.refused_verse_numbers, t.refused_other
    );
    let _ = writeln!(s, "## Per book\n");
    let _ = writeln!(
        s,
        "| Book | Expected | Detected | Usable | Unmapped | Refused (order / numbers / other) | Exact | Normalised | Different (usable) |\n|---|---|---|---|---|---|---|---|---|"
    );
    for b in &a.books {
        let _ = writeln!(
            s,
            "| {} | {} | {} | {} ({}) | {} | {} / {} / {} | {} | {} | {} ({}) |",
            b.code,
            b.expected,
            b.detected,
            b.usable,
            pct(b.usable, b.expected),
            b.unmapped,
            b.refused_reading_order,
            b.refused_verse_numbers,
            b.refused_other,
            b.exact,
            b.normalised,
            b.different,
            b.usable_different
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
    let _ = writeln!(s, "\n## Errors\n");
    for e in &a.errors {
        let _ = writeln!(s, "- {e}");
    }
    let _ = writeln!(s, "\n## Differences\n");
    for d in &a.differences {
        let _ = writeln!(
            s,
            "- {}{} (page {}): found «{}» — edition «{}»",
            d.reference,
            if d.usable { " **usable**" } else { "" },
            d.page.map_or("?".into(), |p| p.to_string()),
            d.found,
            d.edition
        );
    }
    s
}

#[test]
#[ignore]
fn real_document_editions_report() {
    let Some(dir) = folder() else {
        eprintln!("AW_DOCUMENT_EDITIONS not set; nothing to do");
        return;
    };
    let out = std::env::var_os("AW_AUDIT_OUT").map(PathBuf::from);
    for (pdf, id) in [
        ("engwebp_all.pdf", "engwebp"),
        ("deu1912_all.pdf", "deu1912"),
        ("engwebp_book.pdf", "engwebp"),
        ("fraLSG_all.pdf", "fraLSG"),
    ] {
        let path = dir.join(pdf);
        if !path.exists() {
            eprintln!("{pdf}: missing, skipped");
            continue;
        }
        let resources = Path::new(env!("CARGO_MANIFEST_DIR")).join("../resources/scripture");
        if !resources.join(id).exists() {
            eprintln!("{pdf}: no bundled USFM for {id}, skipped");
            continue;
        }
        let a = audit(&path, id);
        let md = markdown(&a);
        eprintln!("{md}");
        if let Some(out) = &out {
            std::fs::create_dir_all(out).unwrap();
            let stem = pdf.trim_end_matches(".pdf");
            std::fs::write(out.join(format!("{stem}.md")), &md).unwrap();
            std::fs::write(
                out.join(format!("{stem}.json")),
                serde_json::to_string_pretty(&a).unwrap(),
            )
            .unwrap();
        }
    }
}
