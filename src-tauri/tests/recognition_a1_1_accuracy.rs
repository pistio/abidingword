//! Phase A1.1: every letter of the original between one verse and the next
//! is accounted for, or the verse is not verified.
//!
//! The recogniser may set text apart from a verse (a heading, a
//! cross-reference). A verse stays verified only when the document itself
//! grounds that: the text stands in a heading element, a note, a floated
//! block, or an element whose class, measured in this document, never
//! carries verse text. An inline piece the recogniser reports removing is
//! kept as an excluded range of the original when it is exactly such an
//! element; otherwise the verse is refused. Nothing is ever removed by its
//! look alone.

use std::io::Write;
use std::path::PathBuf;

use abiding_word_lib::scripture::canon;
use abiding_word_lib::scripture::import::{self, ImportOrigin, StagedImport};
use abiding_word_lib::scripture::provenance::{ExcludedKind, TraceStatus};
use abiding_word_lib::scripture::structural::{self, GateCause, StructuralReport};

const RUTH: [u16; 4] = [22, 23, 18, 22];

fn epub(body: &str) -> Vec<u8> {
    let mut out = std::io::Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut out);
        let stored: zip::write::SimpleFileOptions = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file("mimetype", stored).unwrap();
        zip.write_all(b"application/epub+zip").unwrap();
        zip.start_file("META-INF/container.xml", stored).unwrap();
        zip.write_all(br#"<?xml version="1.0"?><container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#).unwrap();
        zip.start_file("OEBPS/content.opf", stored).unwrap();
        zip.write_all(br#"<?xml version="1.0"?><package xmlns="http://www.idpf.org/2007/opf" version="3.0"><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>Ruth</dc:title><dc:language>en</dc:language></metadata><manifest><item id="t" href="ruth.xhtml" media-type="application/xhtml+xml"/></manifest><spine><itemref idref="t"/></spine></package>"#).unwrap();
        zip.start_file("OEBPS/ruth.xhtml", stored).unwrap();
        write!(
            zip,
            r#"<?xml version="1.0" encoding="utf-8"?><html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><head><title>Ruth</title></head><body>{body}</body></html>"#
        )
        .unwrap();
        zip.finish().unwrap();
    }
    out.into_inner()
}

fn write(name: &str, bytes: &[u8]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("aw-a1-1-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}.epub"));
    std::fs::write(&path, bytes).unwrap();
    path
}

fn read(name: &str, body: &str) -> (StagedImport, StructuralReport) {
    let path = write(name, &epub(body));
    structural::stage_documents(std::slice::from_ref(&path), ImportOrigin::UserFiles).unwrap()
}

fn text(s: &StagedImport, c: u16, v: u16) -> Option<String> {
    let ruth = canon::by_code("RUT").unwrap().ordinal;
    s.content
        .books
        .get(&ruth)?
        .chapters
        .get(&c)?
        .verses
        .get(&v)
        .cloned()
}

fn verified(s: &StagedImport, c: u16, v: u16) -> bool {
    let ruth = canon::by_code("RUT").unwrap().ordinal;
    text(s, c, v).is_some()
        && !s
            .documents
            .as_ref()
            .unwrap()
            .unverified
            .contains_key(&(ruth, c, v))
        && !s.content.issues.iter().any(|i| {
            i.severity == import::Severity::Error && i.book == Some("RUT") && i.chapter == Some(c)
        })
}

/// Ruth with every verse in one paragraph class, `inside(c, v)` in the
/// middle of the verse and `extra(c, v)` right after its paragraph.
fn ruth(extra: impl Fn(u16, u16) -> String, inside: impl Fn(u16, u16) -> String) -> String {
    let mut body = String::from("<h1>The Book of Ruth</h1>");
    for (c, &n) in RUTH.iter().enumerate() {
        let c = c as u16 + 1;
        body.push_str(&format!("<h2>Chapter {c}</h2>"));
        for v in 1..=n {
            body.push_str(&format!(
                r#"<p class="p"><sup class="v">{v}</sup>Invented words of Ruth c{c}v{v}. {}They are here for testing.</p>{}"#,
                inside(c, v),
                extra(c, v)
            ));
        }
    }
    body
}

/// Ruth set as Crampon's EPUB sets it: no chapter headings, a chapter's
/// number as a drop capital on its first verse, every verse in one
/// paragraph class; `after(c, v)` is placed right after a verse's paragraph.
fn ruth_dropnum(after: impl Fn(u16, u16) -> String) -> String {
    let mut body = String::from("<h1>Le livre de Ruth</h1>");
    for (c, &n) in RUTH.iter().enumerate() {
        let c = c as u16 + 1;
        for v in 1..=n {
            let number = if v == 1 {
                format!(r#"<span class="dropnum">{c}</span>"#)
            } else {
                format!(r#"<sup class="v">{v}</sup>"#)
            };
            body.push_str(&format!(
                r#"<p class="p">{number}Invented words of Ruth c{c}v{v}. They are here for testing.</p>{}"#,
                after(c, v)
            ));
        }
    }
    body
}

#[test]
fn a_line_in_the_verse_paragraph_style_that_no_verse_holds_withdraws_the_verse_before_it() {
    // Crampon's "PAROLES D'AGUR." before a chapter: a paragraph of the very
    // class the verses are set in. Nothing in the document says it is a
    // heading; the edition's own source has it inside the verse before.
    let (s, report) = read(
        "title-in-verse-style",
        &ruth_dropnum(|c, v| {
            if (c, v) == (2, 23) {
                r#"<p class="p">WORDS OF THE ELDERS.</p>"#.into()
            } else {
                String::new()
            }
        }),
    );
    let t = text(&s, 2, 23).expect("2:23 read");
    assert!(
        t.contains("ELDERS") || !verified(&s, 2, 23),
        "2:23 verified without the line that follows it: {t:?}"
    );
    if !t.contains("ELDERS") {
        assert_eq!(
            report.gated.get("RUT 2:23"),
            Some(&GateCause::UnaccountedText)
        );
    }
    assert!(
        verified(&s, 2, 22) && verified(&s, 3, 1),
        "{:?}",
        report.gated
    );
}

#[test]
fn the_same_book_without_the_line_is_verified_throughout() {
    // The control: the reading itself verifies this layout.
    let (s, report) = read("dropnum-control", &ruth_dropnum(|_, _| String::new()));
    assert!(report.gated.is_empty(), "{:?}", report.gated);
    for (c, &n) in RUTH.iter().enumerate() {
        for v in 1..=n {
            assert!(verified(&s, c as u16 + 1, v), "Ruth {}:{v}", c + 1);
        }
    }
}

#[test]
fn headings_in_heading_elements_and_their_own_style_are_accounted_for() {
    let (s, report) = read(
        "own-heading-styles",
        &ruth(
            |c, v| match (c, v) {
                (1, 5) => "<h3>The widows return</h3>".into(),
                (2, 7) => r#"<p class="s">BOAZ MEETS RUTH</p>"#.into(),
                (3, 4) => r#"<p class="s">AT THE THRESHING FLOOR</p>"#.into(),
                (4, 1) => r#"<p class="s">THE REDEEMER</p>"#.into(),
                _ => String::new(),
            },
            |_, _| String::new(),
        ),
    );
    assert!(report.gated.is_empty(), "{:?}", report.gated);
    for (c, v) in [(1, 5), (2, 7), (3, 4), (4, 1)] {
        assert!(verified(&s, c, v), "Ruth {c}:{v}");
    }
}

/// An inline cross-reference element inside some verses: `<span class="rq">`.
fn with_references(class_in_text: bool) -> String {
    ruth(
        |_, _| String::new(),
        move |c, v| {
            if v == 3 {
                format!(r#"<span class="rq">Genesis {c}:{v} </span>"#)
            } else if class_in_text && (c, v) == (4, 10) {
                // The same class around words of a verse.
                r#"<span class="rq">Behold </span>"#.into()
            } else {
                String::new()
            }
        },
    )
}

#[test]
fn an_inline_reference_the_recogniser_removes_is_an_excluded_range_of_the_original() {
    let (s, report) = read("inline-references", &with_references(false));
    for c in 1..=4u16 {
        let key = format!("RUT {c}:3");
        let record = &report.provenance[&key];
        let t = text(&s, c, 3).unwrap();
        assert!(
            !t.contains("Genesis"),
            "{key}: the recogniser kept the reference as words: {t:?}"
        );
        assert_eq!(
            record.status,
            TraceStatus::Exact,
            "{key}: {:?}",
            record.reasons
        );
        let removed: Vec<_> = record
            .excluded
            .iter()
            .filter(|x| x.kind == ExcludedKind::Apparatus)
            .collect();
        assert_eq!(removed.len(), 1, "{key}: {:?}", record.excluded);
        assert_eq!(removed[0].text.trim(), format!("Genesis {c}:3"));
        assert!(removed[0].element_bytes.is_some());
        assert!(verified(&s, c, 3), "{key}: {:?}", record.reasons);
    }
}

#[test]
fn a_removed_reference_whose_class_also_carries_verse_words_is_not_grounded() {
    let (s, report) = read("inline-references-mixed", &with_references(true));
    for c in 1..=4u16 {
        let t = text(&s, c, 3).unwrap();
        if !t.contains("Genesis") {
            assert!(
                !verified(&s, c, 3),
                "RUT {c}:3 verified though its removed text has a class that holds verse words: {:?}",
                report.provenance[&format!("RUT {c}:3")].reasons
            );
        }
    }
}
