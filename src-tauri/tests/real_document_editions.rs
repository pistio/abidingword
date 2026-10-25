//! Accuracy of Bibles read from documents on real, typeset editions (v0.24).
//! Not run by default: set AW_DOCUMENT_EDITIONS to a folder holding eBible.org's
//! PDF of an edition whose USFM Abiding Word ships, e.g.
//! `engwebp_all.pdf` (https://ebible.org/pdf/engwebp/) and `deu1912_all.pdf`,
//! then `cargo test --test real_document_editions -- --ignored --nocapture`.
//!
//! Every verse recognised in the PDF is compared with the publisher's own
//! USFM text of the same edition (whitespace collapsed; nothing else is
//! normalised), so the report shows what partial use would store and whether
//! any of it differs from the edition.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use abiding_word_lib::scripture::import::{self, ImportOrigin};

fn folder() -> Option<PathBuf> {
    std::env::var_os("AW_DOCUMENT_EDITIONS").map(PathBuf::from)
}

fn squash(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
#[ignore]
fn real_document_editions_report() {
    let Some(dir) = folder() else {
        eprintln!("AW_DOCUMENT_EDITIONS not set; nothing to do");
        return;
    };
    let resources = Path::new(env!("CARGO_MANIFEST_DIR")).join("../resources/scripture");
    for (pdf, id) in [
        ("engwebp_all.pdf", "engwebp"),
        ("deu1912_all.pdf", "deu1912"),
    ] {
        let path = dir.join(pdf);
        if !path.exists() {
            eprintln!("{pdf}: missing, skipped");
            continue;
        }
        let reference = import::stage(
            &[resources.join(id).join(format!("{id}_usfm.zip"))],
            ImportOrigin::UserFiles,
        )
        .unwrap();
        let staged = import::stage_documents(&[path], ImportOrigin::UserFiles).unwrap();
        let content = &staged.content;
        let usable = content.usable_chapters();
        let mut found = 0usize;
        let mut usable_verses = 0usize;
        let mut exact = 0usize;
        let mut differ: Vec<String> = Vec::new();
        let mut usable_differ = 0usize;
        let mut usable_spacing = 0usize;
        for (ordinal, book) in &content.books {
            for (chapter, ch) in &book.chapters {
                let ok = usable
                    .as_ref()
                    .is_some_and(|u| u.contains(&(*ordinal, *chapter)));
                for (verse, text) in &ch.verses {
                    found += 1;
                    if ok {
                        usable_verses += 1;
                    }
                    let want = reference
                        .content
                        .books
                        .get(ordinal)
                        .and_then(|b| b.chapters.get(chapter))
                        .and_then(|c| c.verses.get(verse));
                    let dense = |t: &str| t.split_whitespace().collect::<String>();
                    match want {
                        Some(w) if squash(w) == squash(text) => exact += 1,
                        // Only spacing differs (a thin space printed between
                        // two quotation marks): the same words, in order.
                        Some(w) if dense(w) == dense(text) => {
                            exact += 1;
                            if ok {
                                usable_spacing += 1;
                            }
                        }
                        _ => {
                            if ok {
                                usable_differ += 1;
                            }
                            if ok && differ.len() < 14 {
                                differ.push(format!(
                                    "{ordinal}.{chapter}.{verse}{}: found «{}» edition «{}»",
                                    if ok { " [usable]" } else { "" },
                                    squash(text),
                                    want.map(|w| squash(w)).unwrap_or_default()
                                ));
                            }
                        }
                    }
                }
            }
        }
        let total: usize = reference
            .content
            .books
            .values()
            .flat_map(|b| b.chapters.values())
            .map(|c| c.verses.len())
            .sum();
        let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
        for issue in &content.issues {
            *kinds
                .entry(format!(
                    "{:?} {}",
                    issue.severity,
                    issue.code.as_deref().unwrap_or("-")
                ))
                .or_default() += 1;
        }
        eprintln!("== {pdf}");
        eprintln!(
            "edition verses {total}; recognised {found}; identical to the edition {exact}; usable (partial use) {usable_verses}; usable but different {usable_differ}; of the identical, spacing only differs in {usable_spacing}"
        );
        eprintln!(
            "usable chapters: {}",
            usable
                .as_ref()
                .map_or("none (an error names no book)".into(), |u| u
                    .len()
                    .to_string())
        );
        let per_book: Vec<String> = content
            .books
            .iter()
            .map(|(o, b)| {
                let n: usize = b.chapters.values().map(|c| c.verses.len()).sum();
                format!("{o}:{}ch/{n}v", b.chapters.len())
            })
            .collect();
        eprintln!("books: {}", per_book.join(" "));
        for issue in content
            .issues
            .iter()
            .filter(|i| format!("{:?}", i.severity) == "Error")
            .take(8)
        {
            eprintln!("  error: {}", issue.message);
        }
        for (k, n) in &kinds {
            eprintln!("  {n:>5} × {k}");
        }
        for d in &differ {
            eprintln!("  {d}");
        }
    }
}
