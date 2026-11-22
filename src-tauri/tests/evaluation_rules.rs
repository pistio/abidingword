//! The evaluation rules of candidate A1.5-E2, tested on SYNTHETIC material
//! only (invented sentences, labelled as such): the strict fidelity rule
//! (`eval_support/fidelity.rs`) on the golden vectors the scorer's Python
//! implementation must also reproduce, and the neutral reference inventory
//! (`eval_support/reference.rs`) on an invented USFM fixture whose label
//! digest the Python inventory must also reproduce. Runs with `cargo test`;
//! needs no corpus and no recogniser run.

#[path = "eval_support/fidelity.rs"]
mod fidelity;
#[path = "eval_support/reference.rs"]
mod reference;
#[allow(dead_code)]
#[path = "eval_support/usfm_text.rs"]
mod usfm_text;

use std::collections::BTreeSet;
use std::io::Write;
use std::path::{Path, PathBuf};

use abiding_word_lib::scripture::canon::{self, NON_CANONICAL_USFM_CODES};
use fidelity::Class;
use reference::Status;

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(rel)
}

#[test]
fn every_golden_vector_is_classed_as_specified() {
    let text = std::fs::read_to_string(repo("tests/evaluation/fidelity-vectors.json")).unwrap();
    let doc: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(doc["rules"], fidelity::RULES);
    let vectors = doc["vectors"].as_array().unwrap();
    assert!(vectors.len() >= 60, "the vectors are all there");
    let mut wrong = Vec::new();
    for v in vectors {
        let (edition, found) = (v["edition"].as_str().unwrap(), v["found"].as_str().unwrap());
        let matched = v["matched"].as_bool().unwrap_or(false);
        let (class, kind) = fidelity::judge(edition, found, matched);
        let want_kind = v["kind"].as_str();
        if class.as_str() != v["class"].as_str().unwrap() || kind != want_kind {
            wrong.push(format!(
                "{}: got {} {:?}, want {} {:?}",
                v["id"],
                class.as_str(),
                kind,
                v["class"],
                want_kind
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn removing_every_space_never_establishes_correctness() {
    let edition = "the shepherd rested by the gate";
    let found: String = edition.chars().filter(|c| *c != ' ').collect();
    assert_eq!(fidelity::classify(edition, &found), Class::Mismatch);
    // Collapsing runs keeps one space; it never removes one.
    assert_eq!(fidelity::runs("  a \u{a0}\t b  "), "a b");
    assert_eq!(
        fidelity::classify("set apart", "set a part"),
        Class::Mismatch
    );
}

#[test]
fn only_canonical_equivalence_is_accepted_beyond_exact_text() {
    // NFC-equal: accepted. NFKC-only (a ligature): a mismatch.
    assert_eq!(fidelity::classify("\u{e9}l", "e\u{301}l"), Class::Canonical);
    assert!(Class::Canonical.faithful() && Class::Layout.faithful());
    assert!(!Class::Mismatch.faithful());
    assert_eq!(fidelity::classify("field", "\u{fb01}eld"), Class::Mismatch);
    // A decomposed reference no longer hides a changed breathing.
    assert_eq!(
        fidelity::classify("\u{3b7}\u{313}\u{301}", "\u{3b7}\u{314}\u{301}"),
        Class::Mismatch
    );
}

#[test]
fn the_white_space_set_is_unicode_white_space() {
    for c in (0u32..=0x3000).filter_map(char::from_u32) {
        assert_eq!(fidelity::is_ws(c), c.is_whitespace(), "U+{:04X}", c as u32);
    }
}

fn fixture_zip(dir: &Path) -> PathBuf {
    let src = repo("tests/evaluation/inventory-fixture");
    let mut names: Vec<String> = std::fs::read_dir(&src)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    let path = dir.join("fixture_usfm.zip");
    let mut zip = zip::ZipWriter::new(std::fs::File::create(&path).unwrap());
    for name in names {
        zip.start_file(name.as_str(), zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(&std::fs::read(src.join(&name)).unwrap())
            .unwrap();
    }
    zip.finish().unwrap();
    path
}

fn expected() -> serde_json::Value {
    serde_json::from_str(
        &std::fs::read_to_string(repo("tests/evaluation/inventory-fixture.expected.json")).unwrap(),
    )
    .unwrap()
}

#[test]
fn the_inventory_keeps_and_classes_every_published_verse() {
    let dir = tempfile::tempdir().unwrap();
    let zip = fixture_zip(dir.path());
    let inv = reference::read(&zip, None);
    let report = inv.report();
    let want = expected();
    assert_eq!(report.label_digest, want["label_digest"].as_str().unwrap());
    assert_eq!(report.labels, want["labels"].as_u64().unwrap() as usize);
    let ranges: Vec<&str> = want["scorable_ranges"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r.as_str().unwrap())
        .collect();
    assert_eq!(report.scorable_ranges, ranges);
    assert!(report.problems.is_empty(), "{:?}", report.problems);
    for (class, n) in want["label_counts"].as_object().unwrap() {
        let n = n.as_u64().unwrap() as usize;
        let got = match class.as_str() {
            "scorable" => report.printed_scorable + report.absent_in_edition,
            "outside_declared_scope" => {
                report.outside_declared_scope.printed + report.outside_declared_scope.empty
            }
            c => report.unsupported[c].printed + report.unsupported[c].empty,
        };
        assert_eq!(got, n, "{class}");
    }
    // Every label is accounted for: nothing leaves the denominator unseen.
    let unsupported: usize = report
        .unsupported
        .values()
        .map(|c| c.printed + c.empty)
        .sum();
    assert_eq!(
        report.printed_scorable
            + report.absent_in_edition
            + unsupported
            + report.outside_declared_scope.printed
            + report.outside_declared_scope.empty,
        report.labels + inv.before_chapter
    );
    // Verse zero, a bridge, an empty verse, the duplicate, Joel 4, Tobit.
    let status = |label: &str| {
        inv.entries
            .iter()
            .filter(|e| e.label() == label)
            .map(|e| e.status.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(status("OBA 1:0"), [Status::Unsupported("verse_zero")]);
    assert_eq!(
        status("OBA 1:3-4"),
        [Status::Scorable {
            ordinal: 31,
            chapter: 1,
            verse: 3
        }]
    );
    assert_eq!(inv.text((31, 1, 5)), Some(""));
    assert_eq!(
        status("OBA 1:7"),
        [
            Status::Unsupported("verse_duplicate"),
            Status::Unsupported("verse_duplicate")
        ]
    );
    assert_eq!(
        inv.unscorable_keys.get(&(31, 1, 7)),
        Some(&"verse_duplicate")
    );
    assert_eq!(
        status("OBA 1:12a"),
        [Status::Unsupported("verse_label_unusual")]
    );
    assert_eq!(
        status("OBA 1:201"),
        [Status::Unsupported("verse_label_unusual")]
    );
    assert_eq!(
        status("JOL 4:1"),
        [Status::Unsupported("chapter_beyond_canon")]
    );
    assert_eq!(
        status("TOB 1:1"),
        [Status::Unsupported("book_outside_canon")]
    );
    assert_eq!(
        status("XYZ 1:1"),
        [Status::Unsupported("book_code_unknown")]
    );
    assert_eq!(status("? 1:1"), [Status::Unsupported("book_unidentified")]);
    assert_eq!(
        status("RUT 1:1"),
        [
            Status::Unsupported("book_duplicate"),
            Status::Unsupported("book_duplicate")
        ]
    );
    assert_eq!(
        status("JUD x:1"),
        [Status::Unsupported("chapter_label_unusual")]
    );
    assert_eq!(status("JUD 0:1"), [Status::Unsupported("chapter_zero")]);
    assert_eq!(report.unsupported["verse_before_chapter"].printed, 1);
    // Each verse keeps the file it came from.
    assert!(inv
        .entries
        .iter()
        .filter(|e| e.book == "TOB")
        .all(|e| e.file == "43-TOB.usfm"));
}

#[test]
fn a_declared_scope_lists_the_other_books_apart() {
    let dir = tempfile::tempdir().unwrap();
    let zip = fixture_zip(dir.path());
    let declared: BTreeSet<String> = ["OBA", "JOL", "TOB"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let inv = reference::read(&zip, Some(&declared));
    let report = inv.report();
    let want = &expected()["declared_OBA_JOL_TOB"];
    assert_eq!(
        report.outside_declared_scope.printed + report.outside_declared_scope.empty,
        want["outside_declared_scope"].as_u64().unwrap() as usize
    );
    assert_eq!(
        report.printed_scorable + report.absent_in_edition,
        want["scorable"].as_u64().unwrap() as usize
    );
    assert!(inv.outside_keys.contains(&(8, 1, 1)), "Ruth is outside");
}

#[test]
fn a_verse_map_or_another_file_is_never_read() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("joel_usfm.zip");
    let mut zip = zip::ZipWriter::new(std::fs::File::create(&path).unwrap());
    let opts = zip::write::SimpleFileOptions::default();
    zip.start_file("29-JOL.usfm", opts).unwrap();
    zip.write_all(
        b"\\id JOL SYNTHETIC\n\\c 3\n\\v 1 An invented verse.\n\\c 4\n\\v 1 Another invented verse.\n",
    )
    .unwrap();
    zip.start_file("VerseMap.xml", opts).unwrap();
    zip.write_all(br#"<verseMap><book osisID="Joel"><verse wlc="Joel.4.1" kjv="Joel.3.1" type="full"/></book></verseMap>"#)
        .unwrap();
    zip.start_file("words.txt", opts).unwrap();
    zip.write_all(b"not a USFM book").unwrap();
    zip.start_file("copr.htm", opts).unwrap();
    zip.write_all(b"<p>prose</p>").unwrap();
    zip.finish().unwrap();
    let inv = reference::read(&path, None);
    assert_eq!(inv.problems.len(), 2, "{:?}", inv.problems);
    assert!(inv.problems[0].contains("verse map"));
    // Not renumbered: Joel 4 stays Joel 4, unsupported.
    assert_eq!(inv.text((29, 3, 1)), Some("An invented verse."));
    assert!(
        inv.entries
            .iter()
            .any(|e| e.label() == "JOL 4:1"
                && e.status == Status::Unsupported("chapter_beyond_canon"))
    );
}

#[test]
fn text_that_is_not_utf8_is_a_problem() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("oba.usfm");
    std::fs::write(
        &path,
        b"\\id OBA SYNTHETIC\n\\c 1\n\\v 1 The caf\xe9 by the gate.\n",
    )
    .unwrap();
    let inv = reference::read(&path, None);
    assert_eq!(inv.problems.len(), 1);
    assert!(inv.problems[0].contains("not UTF-8"));
    assert!(!inv.members[0].utf8);
}

#[test]
fn the_book_table_matches_the_canon() {
    let text = std::fs::read_to_string(repo("docs/evaluation/usfm-books.tsv")).unwrap();
    let mut canon66 = Vec::new();
    let mut other = BTreeSet::new();
    for line in text.lines() {
        if line.is_empty() || line.starts_with('#') || line.starts_with("code\t") {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols[1] == "canon66" {
            canon66.push((cols[0].to_string(), cols[2].parse::<u16>().unwrap()));
        } else {
            other.insert(cols[0].to_string());
        }
    }
    let books: Vec<(String, u16)> = canon::BOOKS
        .iter()
        .map(|b| (b.code.to_string(), b.chapter_count))
        .collect();
    assert_eq!(canon66, books);
    let non: BTreeSet<String> = NON_CANONICAL_USFM_CODES
        .iter()
        .map(|c| c.to_string())
        .collect();
    assert_eq!(other, non);
}

/// E3: a number covered twice (a bridge beside a verse it covers, two
/// overlapping bridges) makes every label involved ambiguous; the numbers
/// inside a sound bridge cannot be validated alone. SYNTHETIC text.
#[test]
fn bridges_that_overlap_are_ambiguous_and_their_inside_numbers_unvalidatable() {
    let dir = std::env::temp_dir().join(format!("aw-e3-bridges-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("31-OBA.usfm");
    std::fs::write(
        &path,
        "\\id OBA SYNTHETIC\n\\c 1\n\\p\n\\v 1 a.\n\\v 2-3 b.\n\\v 4-5 c.\n\\v 5 d.\n\\v 6-8 e.\n\\v 8-9 f.\n\\v 10 g.\n",
    )
    .unwrap();
    let inv = reference::read(&path, None);
    let status = |label: &str| {
        inv.entries
            .iter()
            .find(|e| e.verse == label)
            .map(|e| e.status.clone())
            .unwrap()
    };
    assert!(matches!(status("2-3"), Status::Scorable { verse: 2, .. }));
    for label in ["4-5", "5", "6-8", "8-9"] {
        assert_eq!(
            status(label),
            Status::Unsupported("verse_duplicate"),
            "{label}"
        );
    }
    assert!(
        inv.bridge_keys.contains(&(31, 1, 3)),
        "3 is inside the bridge 2-3"
    );
    for n in 4..=9 {
        assert_eq!(
            inv.unscorable_keys.get(&(31, 1, n)),
            Some(&"verse_duplicate"),
            "{n}"
        );
    }
    let report = inv.report();
    assert_eq!(report.scorable_ranges, vec!["OBA 1:1-2", "OBA 1:10"]);
    assert_eq!(report.bridge_member_ranges, vec!["OBA 1:3"]);
    assert_eq!(report.unsupported["verse_duplicate"].printed, 4);
    std::fs::remove_dir_all(&dir).unwrap();
}
