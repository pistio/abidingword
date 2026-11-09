//! Scripture recognition v0.24.6: general rules for the failure classes set E
//! disclosed (Markdown footnotes, alphabetic chapter numerals, running heads
//! naming book and chapter, words read with the other column, cut verses),
//! and checks that ordinary Scripture text is not put in doubt by them.
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
        partly_styled: false,
        small: false,
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
    verse(r, code, chapter, v).is_some()
        && !r.unverified.contains_key(&(book.ordinal, chapter, v))
        // (A refused chapter is refused by an error on it; v0.24.7.)
        && !r.content.issues.iter().any(|i| {
            i.severity == Severity::Error && i.book == Some(book.code) && i.chapter == Some(chapter)
        })
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

// --- Markdown footnotes (through the reader) ------------------------------------

fn markdown_units(md: &str) -> Vec<Unit> {
    let doc = abiding_word_lib::scripture::documents::read_bytes(
        std::path::Path::new("t.md"),
        "t.md",
        "md",
        md.as_bytes(),
        md.len() as u64,
        0,
    )
    .unwrap();
    doc.units
}

#[test]
fn markdown_footnote_callers_and_definitions_are_never_verse_text() {
    let mut md = String::from("# Filemom\n\n## Filemom 1\n\n");
    for v in 1..=25u16 {
        let caller = if v == 6 { "[^phm-1-1]" } else { "" };
        md.push_str(&format!(
            "**{v}** Palavras inventadas do versículo {v}.{caller}\n\n"
        ));
    }
    md.push_str("[^phm-1-1]: **1:26** Alguns manuscritos acrescentam: palavras de uma variante.\n");
    let units = markdown_units(&md);
    let r = recognise(&units, "pt");
    let six = verse(&r, "PHM", 1, 6).unwrap();
    assert!(!six.contains("[^") && !six.contains("phm-1-1"), "{six}");
    assert!(verified(&r, "PHM", 1, 6), "{:?}", r.unverified);
    for v in 1..=25 {
        let text = verse(&r, "PHM", 1, v).unwrap_or_default();
        assert!(!text.contains("manuscritos"), "{v}: {text}");
    }
    assert_eq!(verse(&r, "PHM", 1, 26), None);
}

#[test]
fn a_footnote_caller_left_as_written_leaves_the_verse_unverified() {
    let mut units = book_pages("Philemon", "PHM", 1..=1, 1, 10);
    let at = units.iter().position(|u| u.text.starts_with("6 ")).unwrap();
    units[at].text = "6 Invented words of the sixth verse.[^phm-1-1]".into();
    let r = recognise(&units, "en");
    assert!(!verified(&r, "PHM", 1, 6));
    assert!(verified(&r, "PHM", 1, 7));
}

// --- alphabetic chapter numerals ---------------------------------------------------

#[test]
fn greek_alphabetic_chapter_headings_give_their_chapters() {
    // "ΚΕΦΑΛΑΙΟΝ Αʹ" … "ΚΕΦΑΛΑΙΟΝ ΙΔʹ": each chapter under its own heading,
    // the text a verse per line.
    let book = canon::by_code("REV").unwrap();
    let greek = |n: u16| {
        let u = ["", "Α", "Β", "Γ", "Δ", "Ε", "Ϛ", "Ζ", "Η", "Θ"];
        let t = ["", "Ι", "Κ"];
        format!("{}{}ʹ", t[(n / 10) as usize], u[(n % 10) as usize])
    };
    let mut units = vec![{
        let mut t = unit("ΑΠΟΚΑΛΥΨΙΣ ΙΩΑΝΝΟΥ", None);
        t.heading = true;
        t
    }];
    for c in 1..=14u16 {
        units.push(unit(&format!("ΚΕΦΑΛΑΙΟΝ {}", greek(c)), None));
        for v in 1..=canon::verse_count(book, c).unwrap() {
            units.push(unit(&format!("{v}  λογοι {c} {v} του στιχου"), None));
        }
    }
    let r = recognise(&units, "el");
    assert_eq!(
        verse(&r, "REV", 13, 1).as_deref(),
        Some("λογοι 13 1 του στιχου"),
        "{:?}",
        errors(&r)
    );
    assert_eq!(
        verse(&r, "REV", 14, 20).as_deref(),
        Some("λογοι 14 20 του στιχου")
    );
}

#[test]
fn a_chapter_label_whose_number_cannot_be_read_refuses_its_chapter() {
    // "ΚΕΦΑΛΑΙΟΝ ϠΩʹ" (no numeral): the chapter that follows is not placed
    // under a guessed number.
    let mut units = book_pages("Philemon", "PHM", 1..=1, 1, 30);
    units.insert(1, line("CHAPTER ONE", 1));
    units.retain(|u| u.text != "Chapter 1");
    let r = recognise(&units, "en");
    assert_eq!(verse(&r, "PHM", 1, 5), None, "{:?}", r.books);
}

// --- running heads ------------------------------------------------------------------

#[test]
fn a_head_naming_book_and_chapter_is_page_furniture() {
    // "Matthew Chapter 5" over each page, and the same as the page's first
    // line without the edge flag; never in a verse.
    let mut units = book_pages("Philemon", "PHM", 1..=1, 1, 5);
    for page in 2..=5u32 {
        let at = units
            .iter()
            .position(|u| u.place.page == Some(page))
            .unwrap();
        let mut head = line(&format!("Philemon Chapter 1 {}", page + 40), page);
        head.edge = page % 2 == 0;
        units[at - 1].text = format!(
            "{} and",
            units[at - 1].text.trim_end_matches(" for testing.")
        );
        units.insert(at, head);
        units[at + 1].text = format!("so on {}", units[at + 1].text);
    }
    let r = recognise(&units, "en");
    for v in 1..=25 {
        let text = verse(&r, "PHM", 1, v).unwrap_or_default();
        assert!(!text.contains("Chapter"), "{v}: {text}");
    }
}

#[test]
fn a_psalm_s_own_heading_at_a_page_s_top_stays_a_heading() {
    // "Psalm 2" opening a page, followed by its verse 1: the chapter's own
    // title, not a running head.
    let mut units = book_pages("Psalms", "PSA", 1..=2, 1, 6);
    for u in units.iter_mut() {
        if u.text == "Chapter 2" {
            u.text = "Psalm 2".into();
            u.large = false;
        }
    }
    let r = recognise(&units, "en");
    let psa = canon::by_code("PSA").unwrap();
    assert_eq!(
        verse(&r, "PSA", 2, 1),
        Some(words(psa.name, 2, 1)),
        "{:?}",
        errors(&r)
    );
}

// --- words read with the other column -----------------------------------------------

#[test]
fn selah_in_the_middle_of_a_verse_leaves_it_unverified() {
    let mut units = book_pages("Philemon", "PHM", 1..=1, 1, 10);
    let at = units.iter().position(|u| u.text.starts_with("6 ")).unwrap();
    units[at].text = "6 When their rulers are thrown down, Selah they will listen.".into();
    let r = recognise(&units, "en");
    assert!(!verified(&r, "PHM", 1, 6));
    // At the verse's end it is the verse's own word.
    let at = units.iter().position(|u| u.text.starts_with("6 ")).unwrap();
    units[at].text = "6 Invented words of the verse. Selah".into();
    let r = recognise(&units, "en");
    assert!(verified(&r, "PHM", 1, 6), "{:?}", r.unverified);
}

// --- cut verses -------------------------------------------------------------------

#[test]
fn a_body_line_set_apart_by_its_look_that_ends_a_sentence_leaves_the_verse_before_unverified() {
    // "⁵ Her feet go down to death." / "Her steps lead straight to Sheol."
    // (centred, in another face): as likely the verse's second line as a
    // heading.
    let mut units = book_pages("Philemon", "PHM", 1..=1, 1, 10);
    let at = units.iter().position(|u| u.text.starts_with("5 ")).unwrap();
    units[at].text = "5 Her feet go down to death.".into();
    let mut line2 = line("Her steps lead straight to Sheol.", 1);
    line2.styled = true;
    line2.centred = true;
    line2.short = true;
    line2.gap_before = true;
    units.insert(at + 1, line2);
    let r = recognise(&units, "en");
    assert!(
        !verified(&r, "PHM", 1, 5) || verse(&r, "PHM", 1, 5).unwrap().contains("Sheol"),
        "{:?}",
        verse(&r, "PHM", 1, 5)
    );
}

// --- ordinary Scripture text stays verified ---------------------------------------

#[test]
fn genuine_numbers_punctuation_and_short_lines_stay_verified() {
    // Numbers in the text (not the verse's own or next number against a
    // word), quotations, dashes, a short closing line: all verified.
    let mut units = book_pages("Philemon", "PHM", 1..=1, 1, 10);
    let texts = [
        (
            4,
            "4 He lived 130 years, and begot sons; his days were 930 years.",
        ),
        (7, "7 “Who is this?” they asked — and he answered: “I am.”"),
        (
            8,
            "8 Of the tribe of Judah were sealed 12,000; of Reuben 12,000.",
        ),
        (9, "9 Grace be with you all. Amen."),
        (
            10,
            "10 The book of Moses says in chapter 3 how the bush burned.",
        ),
    ];
    for (v, t) in texts {
        let at = units
            .iter()
            .position(|u| u.text.starts_with(&format!("{v} ")))
            .unwrap();
        units[at].text = t.into();
    }
    let r = recognise(&units, "en");
    for (v, _) in texts {
        assert!(
            verified(&r, "PHM", 1, v),
            "{v}: {:?} {:?}",
            verse(&r, "PHM", 1, v),
            r.unverified.get(&(57, 1, v))
        );
    }
}
