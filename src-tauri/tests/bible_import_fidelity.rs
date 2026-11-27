//! Product fidelity regressions; no withheld collection or frozen code is changed.

#[allow(dead_code)]
#[path = "support/usfm_fidelity_reference.rs"]
mod independent_reference;

use std::io::Read;
use std::path::Path;

use abiding_word_lib::scripture::canon;
use abiding_word_lib::scripture::documents;
use abiding_word_lib::scripture::import::{self, ImportOrigin, Severity};
use abiding_word_lib::scripture::structure::{self, Corrections};

#[test]
fn malformed_markup_refuses_the_book_without_losing_source_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("RUT.usfm");
    let bytes = b"\\id RUT\n\\c 1\n\\v 1 first\\x + \\xt broken\n\\v 2 second\n";
    std::fs::write(&path, bytes).unwrap();
    let staged = import::stage(std::slice::from_ref(&path), ImportOrigin::UserFiles).unwrap();
    assert_eq!(staged.content.usable_verse_count(), 0);
    assert!(staged.content.issues.iter().any(|issue| {
        issue.severity == Severity::Error
            && issue.code.as_deref() == Some("bibleImport.malformedUsfm")
    }));
    assert_eq!(std::fs::read(path).unwrap(), bytes);
}

#[test]
fn unusable_superscription_is_visible_without_refusing_the_verses() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("PSA.usfm");
    let bytes = b"\\id PSA\n\\c 1\n\\d A\x01 title\x03\n\\v 1 Invented words.\n";
    std::fs::write(&path, bytes).unwrap();
    let staged = import::stage(std::slice::from_ref(&path), ImportOrigin::UserFiles).unwrap();
    assert_eq!(staged.content.usable_verse_count(), 1);
    assert!(staged.content.issues.iter().any(|issue| {
        issue.severity == Severity::Warning
            && issue.code.as_deref() == Some("bibleImport.superscriptionUnusable")
    }));
    assert_eq!(std::fs::read(path).unwrap(), bytes);
}

#[test]
fn sbl_acts_missing_29_cannot_verify_combined_28_30_31_as_verse_28() {
    // Reproduce the disclosed E2/E3 case from bundled, attributed source data.
    // SBL Greek New Testament © SBL and Logos, CC BY 4.0; this is a generated
    // Markdown layout, not an authentic published Markdown edition or Set H.
    let archive =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../resources/scripture/grcsbl/grcsbl_usfm.zip");
    let bytes_before = std::fs::read(&archive).unwrap();
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&bytes_before)).unwrap();
    let mut reference = None;
    for index in 0..zip.len() {
        let mut entry = zip.by_index(index).unwrap();
        if !entry.name().ends_with(".usfm") {
            continue;
        }
        let mut source = String::new();
        entry.read_to_string(&mut source).unwrap();
        let book = independent_reference::read(&source);
        if book.code.as_deref() == Some("ACT") {
            assert!(book.problems.is_empty());
            reference = Some(book);
            break;
        }
    }
    let reference = reference.expect("bundled SBL Acts");
    assert!(!reference
        .verses
        .iter()
        .any(|verse| verse.chapter == "28" && verse.label == "29"));
    let mut text = String::from("# Acts\n\n");
    let mut chapter = String::new();
    let mut paragraph = Vec::new();
    for verse in &reference.verses {
        if chapter != verse.chapter {
            if !paragraph.is_empty() {
                text.push_str(&format!(" {}\n\n", paragraph.join(" ")));
                paragraph.clear();
            }
            chapter = verse.chapter.clone();
            text.push_str(&format!("## {chapter}\n\n"));
        }
        paragraph.push(format!("{} {}", verse.label, verse.text));
        if paragraph.len() == 5 {
            text.push_str(&format!(" {}\n\n", paragraph.join(" ")));
            paragraph.clear();
        }
    }
    if !paragraph.is_empty() {
        text.push_str(&format!(" {}\n\n", paragraph.join(" ")));
    }
    let file = documents::read_bytes(
        Path::new("sbl-acts.md"),
        "sbl-acts.md",
        "md",
        text.as_bytes(),
        text.len() as u64,
        0,
    )
    .unwrap();
    let result = structure::recognise(&file.units, None, &Corrections::default());
    let book = canon::by_code("ACT").unwrap();
    let key = (book.ordinal, 28, 28);
    assert!(
        result.origins.contains_key(&key),
        "the source place must remain inspectable"
    );
    assert_eq!(
        result.unverified.get(&key),
        Some(&"boundary"),
        "the combined text was promoted to verified Scripture"
    );
    assert!(!result.unverified.contains_key(&(book.ordinal, 1, 1)));
    assert_eq!(
        result.content.books[&book.ordinal].chapters[&1].verses[&1],
        reference.verses[0].text
    );
    assert_eq!(std::fs::read(archive).unwrap(), bytes_before);
}
