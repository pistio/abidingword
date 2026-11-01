//! Scripture recognition v0.24.7: general rules for the failure classes set F
//! disclosed (Samuel/Kings naming in Hebrew-based and Septuagint Bibles, Latin
//! chapter labels and chapters inferred from verse counts, Markdown raw HTML,
//! letter-spaced running heads, the Septuagint evidence and contents pages).
//! Each test was written before its rule and failed at 324cfd1.
//!
//! Every text here is invented for the tests; books and verse counts are
//! real, because the recogniser checks chapters against them.

use abiding_word_lib::scripture::canon;
use abiding_word_lib::scripture::documents::{Place, Unit};
use abiding_word_lib::scripture::import::Severity;
use abiding_word_lib::scripture::structure::{self, Corrections, Recognition};

fn words(book: &str, chapter: u16, verse: u16) -> String {
    format!("Invented words of {book} c{chapter}v{verse} for testing.")
}

fn unit(text: &str, page: Option<u32>) -> Unit {
    Unit {
        text: text.to_string(),
        heading: false,
        place: Place {
            document: 0,
            page,
            segment: Some(page.unwrap_or(0)),
            offset: None,
        },
        uncertain_order: false,
        note: false,
        large: false,
        styled: false,
        edge: false,
        centred: false,
        column: 0,
        gap_before: false,
        short: false,
        indented: false,
    }
}

fn line(text: &str, page: u32) -> Unit {
    unit(text, Some(page))
}

fn recognise(units: &[Unit], language: &str) -> Recognition {
    structure::recognise(units, Some(language), &Corrections::default())
}

fn verse(r: &Recognition, code: &str, chapter: u16, verse: u16) -> Option<String> {
    let book = canon::by_code(code).unwrap();
    r.content
        .books
        .get(&book.ordinal)?
        .chapters
        .get(&chapter)?
        .verses
        .get(&verse)
        .cloned()
}

fn verified(r: &Recognition, code: &str, chapter: u16, v: u16) -> bool {
    let book = canon::by_code(code).unwrap();
    verse(r, code, chapter, v).is_some() && !r.unverified.contains_key(&(book.ordinal, chapter, v))
}

fn errors(r: &Recognition) -> Vec<String> {
    r.content
        .issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .map(|i| format!("{:?} {:?} {:?}", i.code, i.book, i.chapter))
        .collect()
}

/// A title, then `chapters` of a book as PDF lines, `per_page` verses a page.
fn book_pages(
    title: &str,
    code: &str,
    chapters: std::ops::RangeInclusive<u16>,
    first_page: u32,
    per_page: u16,
) -> Vec<Unit> {
    let book = canon::by_code(code).unwrap();
    let mut title_unit = line(title, first_page);
    title_unit.large = true;
    let mut units = vec![title_unit];
    let mut n = 0u16;
    for c in chapters {
        let mut heading = line(
            &format!("Chapter {c}"),
            first_page + u32::from(n / per_page),
        );
        heading.large = true;
        units.push(heading);
        for v in 1..=canon::verse_count(book, c).unwrap() {
            units.push(line(
                &format!("{v} {}", words(book.name, c, v)),
                first_page + u32::from(n / per_page),
            ));
            n += 1;
        }
    }
    units
}

// --- verse numbers set against their first word -------------------------------

fn titled_book(
    title: &str,
    code: &str,
    chapters: std::ops::RangeInclusive<u16>,
    page: u32,
) -> Vec<Unit> {
    book_pages(title, code, chapters, page, 40)
}

// --- 1. Samuel and Kings: never by an ambiguous name alone ------------------------

#[test]
fn a_hebrew_based_greek_bible_names_samuel_and_kings_apart() {
    // "ΣΑΜΟΥΗΛ Α΄" … "ΒΑΣΙΛΕΩΝ Α΄": with Samuel named, the first book of
    // Kings is 1 Kings, never 1 Samuel.
    let mut units = titled_book("ΣΑΜΟΥΗΛ Α΄", "1SA", 1..=1, 1);
    units.extend(titled_book("ΣΑΜΟΥΗΛ Β΄", "2SA", 1..=1, 3));
    units.extend(titled_book("ΒΑΣΙΛΕΩΝ Α΄", "1KI", 1..=1, 5));
    let r = recognise(&units, "el");
    assert!(verse(&r, "1SA", 1, 28).is_some(), "{:?}", r.books);
    assert!(verse(&r, "1KI", 1, 53).is_some(), "{:?}", r.books);
    // Never 1 Kings' words under 1 Samuel.
    let kings = canon::by_code("1KI").unwrap();
    assert_ne!(verse(&r, "1SA", 1, 1), Some(words(kings.name, 1, 1)));
}

#[test]
fn a_kings_title_with_no_evidence_for_either_numbering_names_no_book() {
    // "ΒΑΣΙΛΕΩΝ Α΄" alone (no Samuel, no third or fourth Kings): 1 Samuel
    // in the Septuagint's numbering, 1 Kings in the Hebrew Bible's — not
    // known, so neither.
    let units = titled_book("ΒΑΣΙΛΕΩΝ Α΄", "1KI", 1..=1, 1);
    let r = recognise(&units, "el");
    assert_eq!(verse(&r, "1SA", 1, 1), None, "{:?}", r.books);
    assert_eq!(verse(&r, "1KI", 1, 1), None, "{:?}", r.books);
}

// --- 2. Latin chapter labels; no chapter from verse counts alone ------------------

#[test]
fn latin_chapter_labels_give_their_chapters() {
    // "CAPUT I." … "CAPUT XIV.": read as chapters, whatever the verse counts.
    let mut units = titled_book("PROPHETIA EZECHIELIS", "EZK", 1..=4, 1);
    // (Another numbering: chapter 3 ends two verses early.)
    let ezk = canon::by_code("EZK").unwrap();
    units.retain(|u| {
        !(u.text.starts_with("26 ") || u.text.starts_with("27 ")) || !u.text.contains("c3v")
    });
    let _ = ezk;
    for u in units.iter_mut() {
        if let Some(n) = u.text.strip_prefix("Chapter ") {
            let roman = ["", "I", "II", "III", "IV"][n.parse::<usize>().unwrap()];
            u.text = format!("CAPUT {roman}.");
            u.large = false;
        }
    }
    let r = recognise(&units, "la");
    let ezk = canon::by_code("EZK").unwrap();
    assert_eq!(
        verse(&r, "EZK", 4, 1),
        Some(words(ezk.name, 4, 1)),
        "{:?}",
        errors(&r)
    );
}

#[test]
fn a_chapter_is_never_taken_from_verse_numbers_starting_again_alone() {
    // No chapter label at all between two chapters, only a verse "1" after a
    // chapter that has its standard count: which chapter follows is not
    // shown, so it is not placed under the next number by inference.
    let mut units = titled_book("Ruth", "RUT", 1..=2, 1);
    units.retain(|u| u.text != "Chapter 2");
    let r = recognise(&units, "en");
    let ruth = canon::by_code("RUT").unwrap();
    assert_eq!(verse(&r, "RUT", 1, 22), Some(words(ruth.name, 1, 22)));
    assert!(
        verse(&r, "RUT", 2, 1).is_none() || !verified(&r, "RUT", 2, 1),
        "{:?}",
        verse(&r, "RUT", 2, 1)
    );
}

// --- 3. Markdown raw HTML -------------------------------------------------------

fn markdown_units(md: &str) -> Vec<Unit> {
    abiding_word_lib::scripture::documents::read_bytes(
        std::path::Path::new("t.md"),
        "t.md",
        "md",
        md.as_bytes(),
        md.len() as u64,
        0,
    )
    .unwrap()
    .units
}

#[test]
fn verse_text_in_a_markdown_html_block_is_never_silently_lost() {
    // "<div align=\"center\">Ἀμήν.</div>" closing the book's last verse: its
    // words are the verse's; either kept in it or the verse is not verified.
    let mut md = String::from("# Philemon\n\n## Philemon 1\n\n");
    for v in 1..=25u16 {
        md.push_str(&format!("<sup>{v}</sup>Invented words of verse {v}.\n\n"));
    }
    md.push_str("<div align=\"center\">Amen.</div>\n\n<script>alert('x')</script>\n");
    let r = recognise(&markdown_units(&md), "en");
    let last = verse(&r, "PHM", 1, 25).unwrap_or_default();
    assert!(
        last.ends_with("Amen.") || !verified(&r, "PHM", 1, 25),
        "{last} {:?}",
        r.unverified.get(&(57, 1, 25))
    );
    for v in 1..=25 {
        assert!(!verse(&r, "PHM", 1, v).unwrap_or_default().contains("alert"));
    }
}

// --- 4. Letter-spaced running heads ------------------------------------------------

#[test]
fn a_letter_spaced_running_head_is_furniture_even_with_a_figure() {
    // "1 S A M U E L" and "O B A D I A H" over the pages: never a verse
    // number, never verse text.
    // (Ten verses a page: a real book has many pages.)
    let mut units = book_pages("1 SAMUEL", "1SA", 1..=2, 1, 10);
    let mut page = 2u32;
    let mut i = 0;
    while i < units.len() {
        if units[i].place.page == Some(page) {
            let mut head = line("1 S A M U E L", page);
            head.edge = true;
            units[i - 1].text = format!(
                "{} and",
                units[i - 1].text.trim_end_matches(" for testing.")
            );
            units.insert(i, head);
            units[i + 1].text = format!("so on {}", units[i + 1].text);
            page += 1;
        }
        i += 1;
    }
    let r = recognise(&units, "en");
    let book = canon::by_code("1SA").unwrap();
    for (c, n) in [(1u16, 28u16), (2, 36)] {
        for v in 1..=n {
            let text = verse(&r, "1SA", c, v).unwrap_or_default();
            assert!(!text.contains("S A M"), "{c}:{v} {text}");
        }
    }
    assert!(
        verse(&r, "1SA", 2, 1).is_some_and(|t| t.contains(&words(book.name, 2, 1)[..20])),
        "{:?}",
        errors(&r)
    );
}

// --- 5. Septuagint evidence --------------------------------------------------------

#[test]
fn greek_book_names_alone_do_not_make_a_hebrew_based_bible_a_septuagint() {
    // A Greek Bible that names Samuel and has none of the Septuagint's own
    // books: its Psalms keep their numbering and its Esther stays Esther.
    let mut units = titled_book("ΓΕΝΕΣΙΣ", "GEN", 1..=1, 1);
    units.extend(titled_book("ΕΞΟΔΟΣ", "EXO", 1..=1, 2));
    units.extend(titled_book("ΛΕΥΙΤΙΚΟΝ", "LEV", 1..=1, 2));
    units.extend(titled_book("ΣΑΜΟΥΗΛ Α΄", "1SA", 1..=1, 3));
    units.extend(titled_book("ΕΣΘΗΡ", "EST", 1..=1, 5));
    units.extend(titled_book("ΨΑΛΜΟΙ", "PSA", 22..=22, 7).into_iter().skip(1));
    let r = recognise(&units, "el");
    let est = canon::by_code("EST").unwrap();
    assert_eq!(
        verse(&r, "EST", 1, 1),
        Some(words(est.name, 1, 1)),
        "{:?}",
        r.books
    );
}

// --- coverage: a contents page is no list of titles -------------------------------

#[test]
fn a_contents_page_of_titles_with_page_numbers_names_no_book() {
    // "Matthew 1" / "Mark 32" / "Luke 52" … on a contents page, then the
    // books themselves: the contents never begin the books.
    let mut units = Vec::new();
    for (i, name) in [
        "Matthew", "Mark", "Luke", "John", "Acts", "Romans", "Philemon",
    ]
    .iter()
    .enumerate()
    {
        units.push(line(&format!("{name} {}", 1 + i * 30), 1));
    }
    units.extend(titled_book("Matthew", "MAT", 1..=1, 3));
    let r = recognise(&units, "en");
    assert!(
        verse(&r, "MAT", 1, 25).is_some() && verified(&r, "MAT", 1, 25),
        "{:?} {:?}",
        r.books,
        errors(&r)
    );
}
