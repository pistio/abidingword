//! Phase A1: structure-aware EPUB reading and exact source spans.
//!
//! Every book here is invented for the tests (Ruth's real verse counts, so
//! the recogniser accepts the chapters). Each test builds a small EPUB and
//! stages it twice when the comparison matters:
//! * with the frozen reading (`import::stage_documents`);
//! * with the structure-aware reading (`structural::stage_documents`).

use std::io::Write;
use std::path::PathBuf;

use abiding_word_lib::library::adapters::epub_source;
use abiding_word_lib::scripture::canon;
use abiding_word_lib::scripture::import::{self, ImportOrigin, StagedImport};
use abiding_word_lib::scripture::provenance::{self, EntryMarks, ExcludedKind, TraceStatus};
use abiding_word_lib::scripture::structural::{self, GateCause, StructuralReport};

const RUTH: [u16; 4] = [22, 23, 18, 22];

fn words(chapter: u16, verse: u16) -> String {
    format!("Invented words of Ruth c{chapter}v{verse} are here for testing.")
}

/// A minimal EPUB 3 with the given spine documents (name, body markup).
fn epub(files: &[(&str, String)]) -> Vec<u8> {
    epub_styled(files, "")
}

/// [`epub`] with a stylesheet every document links.
fn epub_styled(files: &[(&str, String)], css: &str) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut out));
        let stored: zip::write::SimpleFileOptions = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file("mimetype", stored).unwrap();
        zip.write_all(b"application/epub+zip").unwrap();
        zip.start_file("META-INF/container.xml", stored).unwrap();
        zip.write_all(br#"<?xml version="1.0"?><container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#).unwrap();
        let mut manifest = String::new();
        let mut spine = String::new();
        for (i, (name, _)) in files.iter().enumerate() {
            manifest.push_str(&format!(
                r#"<item id="d{i}" href="{name}" media-type="application/xhtml+xml"/>"#
            ));
            spine.push_str(&format!(r#"<itemref idref="d{i}"/>"#));
        }
        if !css.is_empty() {
            manifest.push_str(r#"<item id="css" href="style.css" media-type="text/css"/>"#);
            zip.start_file("OEBPS/style.css", stored).unwrap();
            zip.write_all(css.as_bytes()).unwrap();
        }
        zip.start_file("OEBPS/content.opf", stored).unwrap();
        zip.write_all(format!(r#"<?xml version="1.0"?><package xmlns="http://www.idpf.org/2007/opf" version="3.0"><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>Invented Ruth</dc:title><dc:language>en</dc:language></metadata><manifest>{manifest}</manifest><spine>{spine}</spine></package>"#).as_bytes()).unwrap();
        for (name, body) in files {
            zip.start_file(format!("OEBPS/{name}"), stored).unwrap();
            zip.write_all(format!(r#"<?xml version="1.0" encoding="utf-8"?><html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><head><title>t</title><link rel="stylesheet" href="style.css"/></head><body>{body}</body></html>"#).as_bytes()).unwrap();
        }
        zip.finish().unwrap();
    }
    out
}

/// An entry of an EPUB as stored in the archive, checked against the hash
/// the source model recorded for it: the bytes every byte range points into.
fn stored_entry(epub: &[u8], entry: &epub_source::Entry) -> String {
    use std::io::Read;
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(epub)).unwrap();
    let mut stored = String::new();
    archive
        .by_name(&entry.path)
        .unwrap()
        .read_to_string(&mut stored)
        .unwrap();
    assert_eq!(
        abiding_word_lib::library::files::content_hash(stored.as_bytes()),
        entry.sha256,
        "the stored entry is the one the source model read"
    );
    assert_eq!(stored.len(), entry.source_len);
    stored
}

fn write(name: &str, bytes: &[u8]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("aw-a1-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}.epub"));
    std::fs::write(&path, bytes).unwrap();
    path
}

fn baseline(path: &PathBuf) -> StagedImport {
    import::stage_documents(std::slice::from_ref(path), ImportOrigin::UserFiles).unwrap()
}

fn structure_aware(path: &PathBuf) -> (StagedImport, StructuralReport) {
    structural::stage_documents(std::slice::from_ref(path), ImportOrigin::UserFiles).unwrap()
}

fn text(s: &StagedImport, chapter: u16, verse: u16) -> Option<String> {
    let ruth = canon::by_code("RUT").unwrap().ordinal;
    s.content
        .books
        .get(&ruth)?
        .chapters
        .get(&chapter)?
        .verses
        .get(&verse)
        .cloned()
}

fn verified(s: &StagedImport, chapter: u16, verse: u16) -> bool {
    let ruth = canon::by_code("RUT").unwrap().ordinal;
    let stage = s.documents.as_ref().unwrap();
    text(s, chapter, verse).is_some()
        && !stage.unverified.contains_key(&(ruth, chapter, verse))
        && !s.content.issues.iter().any(|i| {
            i.severity == import::Severity::Error
                && i.book == Some("RUT")
                && i.chapter == Some(chapter)
        })
}

/// Why a verse is not verified (for failure messages).
fn why(s: &StagedImport, chapter: u16, verse: u16) -> String {
    let ruth = canon::by_code("RUT").unwrap().ordinal;
    let stage = s.documents.as_ref().unwrap();
    let reason = stage.unverified.get(&(ruth, chapter, verse));
    let issues: Vec<_> = s
        .content
        .issues
        .iter()
        .filter(|i| i.chapter == Some(chapter) || i.chapter.is_none())
        .map(|i| (i.severity, i.code.clone()))
        .collect();
    format!(
        "text {:?}, unverified {reason:?}, issues {issues:?}",
        text(s, chapter, verse)
    )
}

/// Every verified verse's text, by (chapter, verse).
fn verified_texts(s: &StagedImport) -> Vec<((u16, u16), String)> {
    let mut out = Vec::new();
    for (c, &n) in RUTH.iter().enumerate() {
        let c = c as u16 + 1;
        for v in 1..=n + 3 {
            if verified(s, c, v) {
                out.push(((c, v), text(s, c, v).unwrap()));
            }
        }
    }
    out
}

/// Ruth as one spine document: a title, chapter headings, one paragraph per
/// verse led by its number; `verse` writes each verse's paragraph content.
fn ruth(verse: impl Fn(u16, u16) -> String, after: &str) -> String {
    let mut body = String::from("<h1>The Book of Ruth</h1>");
    for (c, &n) in RUTH.iter().enumerate() {
        let c = c as u16 + 1;
        body.push_str(&format!("<h2>Chapter {c}</h2>"));
        for v in 1..=n {
            body.push_str(&format!(
                r#"<p id="Ruth{c}-{v}"><span class="verse">{v}</span> {}</p>"#,
                verse(c, v)
            ));
        }
    }
    body.push_str(after);
    body
}

/// A caller in the Project Gutenberg manner: an anchor, then a link whose
/// visible text is a bracketed label, leading to a note that links back.
fn gutenberg_caller(n: u32, label: &str) -> String {
    format!(r##"<a id="FNanchor_{n}"></a><a class="fnanchor" href="#Footnote_{n}">[{label}]</a>"##)
}

fn gutenberg_note(n: u32, label: &str, class: &str) -> String {
    format!(
        r##"<div class="{class}"><p><a id="Footnote_{n}"></a><a href="#FNanchor_{n}"><span class="label">[{label}]</span></a> Note {n}: an invented explanation.</p></div>"##
    )
}

/// Ruth with a numeric caller in every third verse, labelled with the next
/// verse's number (the label of a note is no verse number), and the notes
/// at the end in a block of the given class.
fn ruth_with_numeric_callers(class: &str) -> Vec<u8> {
    let mut notes = String::new();
    let mut n = 0u32;
    let mut callers = std::collections::BTreeMap::new();
    for (c, &count) in RUTH.iter().enumerate() {
        for v in (1..=count).step_by(3) {
            n += 1;
            callers.insert((c as u16 + 1, v), n);
        }
    }
    for (&(_, v), &n) in &callers {
        notes.push_str(&gutenberg_note(n, &(v + 1).to_string(), class));
    }
    let body = ruth(
        |c, v| match callers.get(&(c, v)) {
            Some(&n) => {
                let w = words(c, v);
                let (a, b) = w.split_at(w.find(" are").unwrap());
                format!(
                    "{a} {} {}",
                    gutenberg_caller(n, &(v + 1).to_string()),
                    b.trim()
                )
            }
            None => words(c, v),
        },
        &notes,
    );
    epub(&[("ruth.xhtml", body)])
}

#[test]
fn numeric_note_links_are_callers_by_where_they_lead_never_verse_numbers() {
    // No semantic markup at all: the notes' class says nothing. Only the
    // links say what they are: each caller leads to a note that links back.
    let path = write("numeric", &ruth_with_numeric_callers("fnote"));
    let (s, report) = structure_aware(&path);
    for (c, &n) in RUTH.iter().enumerate() {
        let c = c as u16 + 1;
        for v in 1..=n {
            assert!(
                verified(&s, c, v),
                "{c}:{v} {} {:?}",
                why(&s, c, v),
                report.gated
            );
            assert_eq!(text(&s, c, v).unwrap(), words(c, v), "{c}:{v}");
        }
    }
    assert!(report.gated.is_empty(), "{:?}", report.gated);
    assert!(
        report.links.get("noteCaller").copied().unwrap_or(0) >= 30,
        "{:?}",
        report.links
    );
}

#[test]
fn the_frozen_reading_reads_numeric_callers_as_verse_numbers() {
    // The failure class itself, at the frozen reading: "[2]" in verse 1 is
    // read as verse 2's number and verse 1 is cut there. (Its own checks
    // leave the cut verse unverified in this small book; in set G's real
    // EPUB such verses were verified.)
    let path = write("numeric-baseline", &ruth_with_numeric_callers("fnote"));
    let s = baseline(&path);
    assert_eq!(
        text(&s, 1, 1).as_deref(),
        Some("Invented words of Ruth c1v1")
    );
}

#[test]
fn letter_and_sign_callers_leading_into_declared_endnotes_are_callers() {
    let labels = ["A", "OR", "a", "*", "ii"];
    let mut notes = String::from(r#"<section epub:type="endnotes" role="doc-endnotes"><ol>"#);
    for (i, l) in labels.iter().enumerate() {
        notes.push_str(&format!(
            r#"<li id="en{i}" epub:type="endnote"><p>{l} An invented endnote.</p></li>"#
        ));
    }
    notes.push_str("</ol></section>");
    let body = ruth(
        |c, v| {
            let w = words(c, v);
            if c == 2 && v <= labels.len() as u16 {
                let i = v as usize - 1;
                let (a, b) = w.split_at(w.find(" here").unwrap());
                format!(
                    r#"{a}<a href="notes.xhtml#en{i}">[{}]</a> {}"#,
                    labels[i],
                    b.trim()
                )
            } else {
                w
            }
        },
        "",
    );
    let path = write(
        "letters",
        &epub(&[("ruth.xhtml", body), ("notes.xhtml", notes)]),
    );
    let (s, report) = structure_aware(&path);
    for v in 1..=23 {
        assert!(
            verified(&s, 2, v),
            "2:{v} {} {:?}",
            why(&s, 2, v),
            report.gated
        );
        assert_eq!(text(&s, 2, v).unwrap(), words(2, v), "2:{v}");
    }
}

#[test]
fn bracketed_words_and_words_linked_to_a_glossary_stay_scripture() {
    // "[and]" is a translator's supplied word, not a link; "Boaz" leads to a
    // glossary entry, not a note. Neither is ever taken out.
    let glossary =
        r#"<dl><dt id="boaz">Boaz</dt><dd>An invented glossary entry.</dd></dl>"#.to_string();
    let body = ruth(
        |c, v| {
            match (c, v) {
            (2, 1) => r#"Invented words [and] more words of <a href="glossary.xhtml#boaz">Boaz</a> for testing."#.to_string(),
            _ => words(c, v),
        }
        },
        "",
    );
    let path = write(
        "glossary",
        &epub(&[("ruth.xhtml", body), ("glossary.xhtml", glossary)]),
    );
    let (s, report) = structure_aware(&path);
    assert!(
        verified(&s, 2, 1),
        "{:?} {:?}",
        text(&s, 2, 1),
        report.gated
    );
    assert_eq!(
        text(&s, 2, 1).unwrap(),
        "Invented words [and] more words of Boaz for testing."
    );
}

#[test]
fn verse_numbers_that_are_links_to_the_contents_are_still_verse_numbers() {
    // A contents page as books have one: a list of links into the text.
    let contents =
        r#"<nav epub:type="toc"><ol><li id="c1"><a href="ruth.xhtml">Ruth</a></li></ol></nav>"#
            .to_string();
    let mut body = String::from("<h1>The Book of Ruth</h1>");
    for (c, &n) in RUTH.iter().enumerate() {
        let c = c as u16 + 1;
        body.push_str(&format!("<h2>Chapter {c}</h2>"));
        for v in 1..=n {
            body.push_str(&format!(
                r#"<p><a href="contents.xhtml#c1">{v}</a> {}</p>"#,
                words(c, v)
            ));
        }
    }
    let path = write(
        "linked-numbers",
        &epub(&[("contents.xhtml", contents), ("ruth.xhtml", body)]),
    );
    let (s, report) = structure_aware(&path);
    for v in 1..=22 {
        assert!(
            verified(&s, 1, v),
            "1:{v} {:?} {:?} {:?} {:?}",
            text(&s, 1, v),
            report.gated,
            s.documents
                .as_ref()
                .unwrap()
                .unverified
                .iter()
                .take(3)
                .collect::<Vec<_>>(),
            s.content
                .issues
                .iter()
                .map(|i| (&i.code, i.chapter))
                .collect::<Vec<_>>()
        );
        assert_eq!(text(&s, 1, v).unwrap(), words(1, v));
    }
}

#[test]
fn a_declared_caller_that_leads_to_a_verse_is_a_conflict_and_nothing_next_to_it_is_verified() {
    let body = ruth(
        |c, v| match (c, v) {
            (1, 3) => {
                let w = words(c, v);
                let (a, b) = w.split_at(w.find(" here").unwrap());
                format!(
                    r##"{a}<a epub:type="noteref" href="#Ruth1-5">[a]</a> {}"##,
                    b.trim()
                )
            }
            _ => words(c, v),
        },
        "",
    );
    let path = write("conflict", &epub(&[("ruth.xhtml", body)]));
    let (s, report) = structure_aware(&path);
    assert!(!verified(&s, 1, 3), "{:?}", text(&s, 1, 3));
    assert_eq!(
        report.gated.get("RUT 1:3"),
        Some(&GateCause::UncertainLink),
        "{:?}",
        report.gated
    );
    for v in [1, 2, 4, 5, 6] {
        assert!(verified(&s, 1, v), "1:{v} {:?}", report.gated);
    }
    // The frozen reading trusts the declaration and verifies it.
    assert!(verified(&baseline(&path), 1, 3));
}

#[test]
fn a_caller_shaped_link_leading_nowhere_is_ambiguous_and_never_makes_a_verse() {
    // "[3]" in verse 2 leads to no element: it might be a caller or verse 3's
    // number. Neither verse 2 nor 3 is verified, and no verse holds "[3]".
    let body = ruth(
        |c, v| match (c, v) {
            (1, 2) => {
                let w = words(c, v);
                let (a, b) = w.split_at(w.find(" here").unwrap());
                format!(r##"{a} <a href="#nowhere">[3]</a> {}"##, b.trim())
            }
            _ => words(c, v),
        },
        "",
    );
    let path = write("ambiguous", &epub(&[("ruth.xhtml", body)]));
    let (s, report) = structure_aware(&path);
    assert!(
        !verified(&s, 1, 2),
        "{:?} {:?}",
        text(&s, 1, 2),
        report.gated
    );
    for ((c, v), t) in verified_texts(&s) {
        assert!(!t.contains("[3]"), "{c}:{v} {t}");
        assert_eq!(t, words(c, v), "{c}:{v}");
    }
    // (The recogniser may refuse the whole chapter: that is its own
    // decision, and safe.) The other chapters are untouched.
    assert!(!verified(&s, 1, 3), "{}", why(&s, 1, 3));
    for v in 1..=23 {
        assert!(verified(&s, 2, v), "2:{v} {}", why(&s, 2, v));
    }
}

#[test]
fn notes_in_several_layers_never_enter_a_verse() {
    // A verse's caller leads to a note in the notes section; that note has
    // its own caller, to a second note, which links back to it.
    let notes = r##"<section class="notes"><div class="fnote"><p><a id="n1"></a><a href="#r1">[1]</a> First-level note with its own caller <a id="r2"></a><a href="#n2">[2]</a> inside.</p></div><div class="fnote"><p><a id="n2"></a><a href="#r2">[2]</a> Second-level note.</p></div></section>"##;
    let body = ruth(
        |c, v| match (c, v) {
            (3, 7) => {
                let w = words(c, v);
                let (a, b) = w.split_at(w.find(" here").unwrap());
                format!(r##"{a}<a id="r1"></a><a href="#n1">[1]</a> {}"##, b.trim())
            }
            _ => words(c, v),
        },
        notes,
    );
    let path = write("layers", &epub(&[("ruth.xhtml", body)]));
    let (s, report) = structure_aware(&path);
    for v in 1..=18 {
        assert!(
            verified(&s, 3, v),
            "3:{v} {} {:?}",
            why(&s, 3, v),
            report.gated
        );
        assert_eq!(text(&s, 3, v).unwrap(), words(3, v));
    }
    for (_, t) in verified_texts(&s) {
        assert!(!t.contains("note"), "{t}");
    }
}

#[test]
fn a_verse_around_a_caller_is_two_exact_spans_and_one_excluded_range() {
    let body = ruth(
        |c, v| match (c, v) {
            (1, 1) => format!(
                "Text before note {} text after note.",
                gutenberg_caller(1, "1")
            ),
            _ => words(c, v),
        },
        &gutenberg_note(1, "1", "fnote"),
    );
    let path = write("spans", &epub(&[("ruth.xhtml", body)]));
    let bytes = std::fs::read(&path).unwrap();
    let (s, report) = structure_aware(&path);
    assert!(verified(&s, 1, 1), "{:?}", report.gated);
    assert_eq!(text(&s, 1, 1).unwrap(), "Text before note text after note.");
    let p = &report.provenance["RUT 1:1"];
    assert_eq!(p.status, TraceStatus::Exact);
    assert_eq!(p.file.entry, "OEBPS/ruth.xhtml");
    assert_eq!(
        p.file.file_sha256,
        abiding_word_lib::library::files::content_hash(&bytes)
    );
    assert_eq!(p.spans.len(), 2, "{:?}", p.spans);
    let book = epub_source::read(&bytes).unwrap();
    let entry = &book.entries[book.by_path["OEBPS/ruth.xhtml"]];
    assert_eq!(p.file.entry_sha256, entry.sha256);
    let stored = stored_entry(&bytes, entry);
    assert_eq!(
        provenance::original_bytes(&stored, &p.spans[0]),
        "Text before note"
    );
    assert_eq!(
        provenance::original_bytes(&stored, &p.spans[1]),
        "text after note."
    );
    assert_eq!(p.excluded.len(), 1);
    let x = &p.excluded[0];
    assert_eq!(x.kind, ExcludedKind::NoteCaller);
    assert_eq!(x.text, "[1]");
    let (a, b) = x.element_bytes.unwrap();
    assert_eq!(
        &stored[a as usize..b as usize],
        r##"<a class="fnanchor" href="#Footnote_1">[1]</a>"##
    );
    // The caller lies between the two spans, in the original's order.
    assert!(p.spans[0].byte_end <= a && b <= p.spans[1].byte_start);
    assert_eq!(p.quote, "Text before note text after note.");
}

#[test]
fn entities_and_inline_markup_keep_exact_byte_ranges() {
    let body = ruth(
        |c, v| match (c, v) {
            (1, 1) => "Naomi &amp; Ruth <i>went</i> on&#8212;together.".to_string(),
            _ => words(c, v),
        },
        "",
    );
    let path = write("entities", &epub(&[("ruth.xhtml", body)]));
    let (s, report) = structure_aware(&path);
    let p = &report.provenance["RUT 1:1"];
    assert_eq!(text(&s, 1, 1).unwrap(), "Naomi & Ruth went on—together.");
    assert_eq!(p.status, TraceStatus::Exact, "{p:?}");
    assert_eq!(p.quote, "Naomi & Ruth went on—together.");
    let bytes = std::fs::read(&path).unwrap();
    let book = epub_source::read(&bytes).unwrap();
    let entry = &book.entries[0];
    let stored = stored_entry(&bytes, entry);
    let original: String = p
        .spans
        .iter()
        .map(|s| provenance::original_bytes(&stored, s))
        .collect();
    assert_eq!(original, "Naomi &amp; Ruth <i>went</i> on&#8212;together.");
}

#[test]
fn every_verse_of_a_book_is_traced_exactly_and_its_spans_give_its_text() {
    let path = write("all", &ruth_with_numeric_callers("fnote"));
    let (s, report) = structure_aware(&path);
    let total: u16 = RUTH.iter().sum();
    assert_eq!(report.exact, usize::from(total), "{:?}", report.untraced);
    assert_eq!(report.untraced, 0);
    for (name, p) in &report.provenance {
        assert_eq!(p.quote, p.text, "{name}");
        assert!(
            p.spans.windows(2).all(|w| w[0].byte_end <= w[1].byte_start),
            "{name}"
        );
        assert!(
            p.excluded
                .iter()
                .all(|x| x.kind == ExcludedKind::NoteCaller),
            "{name}"
        );
    }
    // Ids such as "Ruth1-4" stand at each verse's start and agree with it.
    assert_eq!(report.id_agrees, usize::from(total));
    assert_eq!(report.id_contradicts, 0);
    assert!(!s.content.books.is_empty());
}

#[test]
fn an_id_at_a_verse_s_start_naming_another_verse_withdraws_verification() {
    let mut body = String::from("<h1>The Book of Ruth</h1>");
    for (c, &n) in RUTH.iter().enumerate() {
        let c = c as u16 + 1;
        body.push_str(&format!("<h2>Chapter {c}</h2>"));
        for v in 1..=n {
            let id = if (c, v) == (1, 8) { 9 } else { v };
            body.push_str(&format!(
                r#"<p id="Ruth{c}-{id}"><span class="verse">{v}</span> {}</p>"#,
                words(c, v)
            ));
        }
    }
    let path = write("ids", &epub(&[("ruth.xhtml", body)]));
    let (s, report) = structure_aware(&path);
    assert!(!verified(&s, 1, 8));
    assert_eq!(report.gated.get("RUT 1:8"), Some(&GateCause::IdContradicts));
    assert!(verified(&s, 1, 7) && verified(&s, 1, 9));
}

#[test]
fn an_id_whose_digits_run_the_book_into_the_chapter_is_no_evidence() {
    // Some generators write "R110_1" for chapter 10 verse 1 of a book whose
    // code begins with a digit: the run "110" cannot be split with
    // certainty, so it must neither agree nor contradict.
    let mut body = String::from("<h1>The Book of Ruth</h1>");
    for (c, &n) in RUTH.iter().enumerate() {
        let c = c as u16 + 1;
        body.push_str(&format!("<h2>Chapter {c}</h2>"));
        for v in 1..=n {
            body.push_str(&format!(
                r#"<p><span class="verse" id="R1{c}_{v}">{v}</span> {}</p>"#,
                words(c, v)
            ));
        }
    }
    let path = write("prefixed-ids", &epub(&[("ruth.xhtml", body)]));
    let (s, report) = structure_aware(&path);
    assert_eq!(report.id_contradicts, 0);
    assert!(report.gated.is_empty(), "{:?}", report.gated);
    for (c, &n) in RUTH.iter().enumerate() {
        for v in 1..=n {
            assert!(verified(&s, c as u16 + 1, v), "Ruth {}:{v}", c + 1);
        }
    }
}

#[test]
fn a_verse_number_read_as_words_is_never_verified() {
    // An omitted verse printed as "[[ <span class='v'>22</span>]]" with a
    // note caller after it: the number must not become words of 1:21.
    let mut body = String::from("<h1>The Book of Ruth</h1>");
    for (c, &n) in RUTH.iter().enumerate() {
        let c = c as u16 + 1;
        body.push_str(&format!("<h2>Chapter {c}</h2><p class='p'>"));
        for v in 1..=n {
            if (c, v) == (1, 22) {
                body.push_str(
                    "[[ <span class='v'>22</span>]]<sup><a id='r1' href='#fn1'>1</a></sup> ",
                );
            } else {
                body.push_str(&format!("<span class='v'>{v}</span>{} ", words(c, v)));
            }
        }
        body.push_str("</p>");
    }
    body.push_str(
        "<div class='footnotes'><p id='fn1'><a href='#r1'>1</a> Some copies lack this verse.</p></div>",
    );
    let path = write("omitted-verse", &epub(&[("ruth.xhtml", body)]));
    let (s, report) = structure_aware(&path);
    if let Some(t) = text(&s, 1, 21) {
        assert!(
            !t.contains("22") || !verified(&s, 1, 21),
            "1:21 verified as {t:?}"
        );
    }
    assert!(!verified(&s, 1, 22));
    assert!(
        verified(&s, 1, 20) && verified(&s, 2, 1),
        "{:?}",
        report.gated
    );
}

#[test]
fn a_note_s_own_back_link_never_withdraws_the_verses_around_it() {
    // Pandoc's end-of-section notes stand between chapters; each note opens
    // with a numbered back-link to its caller. The back-link belongs to the
    // apparatus and is no uncertainty in the text.
    let mut body = String::from("<h1>The Book of Ruth</h1>");
    let mut note = 0;
    for (c, &n) in RUTH.iter().enumerate() {
        let c = c as u16 + 1;
        body.push_str(&format!("<section><h2>Chapter {c}</h2><p>"));
        for v in 1..=n {
            body.push_str(&format!("<strong>{v}</strong> {}", words(c, v)));
            // The caller ends its paragraph, as the note's back-link begins
            // its own: only the declared note container tells them apart.
            if v == n {
                note += 1;
                body.push_str(&format!(
                    r##"<a href="#fn{note}" class="footnote-ref" id="fnref{note}" epub:type="noteref">{note}</a>"##
                ));
            }
            if v < n {
                body.push(' ');
            }
        }
        body.push_str(&format!(
            r##"</p><section class="footnotes" epub:type="footnotes"><aside epub:type="footnote" id="fn{note}"><p><a href="#fnref{note}" class="footnote-back" role="doc-backlink">{note}</a>. {c}:{n} A note on the verse.</p></aside></section></section>"##
        ));
    }
    let path = write("back-links", &epub(&[("ruth.xhtml", body)]));
    let (s, report) = structure_aware(&path);
    assert!(report.gated.is_empty(), "{:?}", report.gated);
    for (c, &n) in RUTH.iter().enumerate() {
        let c = c as u16 + 1;
        assert!(verified(&s, c, 1) && verified(&s, c, n), "Ruth {c}");
    }
}

#[test]
fn tracing_never_bridges_text_it_cannot_name() {
    let entry = epub_source::read_entry_structure(
        "x.xhtml".into(),
        r##"<p>alpha beta gamma</p><p>delta <a href="#n">[1]</a> epsilon</p><p id="n">note</p>"##
            .into(),
    );
    let plain = EntryMarks::new(&entry, &[]);
    let all = plain.nonspace_len();
    assert!(provenance::trace(&plain, "alpha gamma", 0, all).is_none());
    assert!(provenance::trace(&plain, "alpha beta gamma", 0, all).is_some());
    // The caller is skipped only when the link is known to be one.
    assert!(provenance::trace(&plain, "delta epsilon", 0, all).is_none());
    let link = entry
        .elements
        .iter()
        .position(|e| e.href.is_some())
        .unwrap();
    let marked = EntryMarks::new(&entry, &[(epub_source::LinkRole::NoteCaller, link)]);
    let t = provenance::trace(&marked, "delta epsilon", 0, all).expect("traced");
    assert_eq!(provenance::excluded_ranges(&marked, &t)[0].text, "[1]");
}

#[test]
fn caller_shapes_are_short_labels_never_longer_words() {
    // Only links are judged by this. Up to three letters may be a note's
    // label ("a", "OR", "ii") as well as a word ("and"): a link of that shape
    // that leads nowhere, or that is read into a verse, is never verified.
    for yes in ["[1]", "12", "a", "[OR]", "*", "(ii)", "†", "and"] {
        assert!(epub_source::caller_shaped(yes), "{yes}");
    }
    for no in ["Boaz", "", "12345", "words"] {
        assert!(!epub_source::caller_shaped(no), "{no}");
    }
}

#[test]
fn a_block_the_stylesheet_floats_beside_the_text_is_never_part_of_a_verse() {
    // A marginal note (a chronology) in its own block between two verses,
    // after a verse ending with a colon, floated beside the text by the
    // book's stylesheet. The chapter's numeral is floated too, but inline:
    // it stays the chapter's number.
    let css = ".margin { float: right; width: 20%; font-size: smaller }\n.first { float: left; font-size: 200% }";
    let mut body = String::from("<h1>The Book of Ruth</h1>");
    for (c, &n) in RUTH.iter().enumerate() {
        let c = c as u16 + 1;
        for v in 1..=n {
            let number = if v == 1 {
                format!(r#"<span class="first">{c}</span>"#)
            } else {
                v.to_string()
            };
            let w = if (c, v) == (2, 4) {
                words(c, v).replace("testing.", "testing, saying:")
            } else {
                words(c, v)
            };
            body.push_str(&format!("<p>{number} {w}</p>"));
            if (c, v) == (2, 4) {
                body.push_str(r#"<div class="margin">Before Christ 1312</div>"#);
            }
        }
    }
    let path = write("floated", &epub_styled(&[("ruth.xhtml", body)], css));
    let (s, report) = structure_aware(&path);
    assert!(verified(&s, 2, 4), "{} {:?}", why(&s, 2, 4), report.gated);
    assert_eq!(
        text(&s, 2, 4).unwrap(),
        "Invented words of Ruth c2v4 are here for testing, saying:"
    );
    for v in 1..=23 {
        assert!(verified(&s, 2, v), "2:{v} {}", why(&s, 2, v));
    }
}
