//! Scripture recognition v0.24.5: the general rules for the failure classes
//! set D disclosed (running heads of other forms, verse numbers set against
//! their first word, notes numbered through a document, theme lines, cut
//! and merged verses, and the Septuagint's own books and numbering).
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

fn usable(r: &Recognition, code: &str, chapter: u16) -> bool {
    let book = canon::by_code(code).unwrap();
    r.content.books.contains_key(&book.ordinal)
        && !r.content.issues.iter().any(|i| {
            i.severity == Severity::Error
                && i.book.as_deref() == Some(book.code)
                && i.chapter == Some(chapter)
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

#[test]
fn a_verse_number_set_against_its_first_word_leaves_the_verse_before_unverified() {
    // "… y Abram 23que de todo …": the "23" was not read as a verse number,
    // so verse 22 holds verse 23's words. Found, never verified; and the
    // verses around keep their own words.
    let mut units = book_pages("Philemon", "PHM", 1..=1, 1, 10);
    let at = units
        .iter()
        .position(|u| u.text.starts_with("22 "))
        .unwrap();
    units[at].text = "22 Invented words of the twenty-second, 23que de todo lo tuyo.".into();
    units.retain(|u| !u.text.starts_with("23 "));
    let r = recognise(&units, "en");
    assert!(verse(&r, "PHM", 1, 22).is_some(), "{:?}", errors(&r));
    assert!(!verified(&r, "PHM", 1, 22));
    assert!(verified(&r, "PHM", 1, 21));
    assert!(verified(&r, "PHM", 1, 24));
    // Behind an opening quotation mark, and in a script without capitals.
    for glued in [
        "22 Words of the verse. 23»La experiencia",
        "22 말씀이라 23`우리가",
    ] {
        let mut units = book_pages("Philemon", "PHM", 1..=1, 1, 10);
        let at = units
            .iter()
            .position(|u| u.text.starts_with("22 "))
            .unwrap();
        units[at].text = glued.into();
        units.retain(|u| !u.text.starts_with("23 "));
        let r = recognise(&units, "en");
        assert!(!verified(&r, "PHM", 1, 22), "{glued}");
    }
}

#[test]
fn a_verse_s_own_number_standing_in_it_again_puts_the_verse_before_in_doubt() {
    // "24 … de 25 años para arriba …" read "25" as verse 25's start; the
    // true "25Y se retirarán" stands inside what was read as verse 25.
    let mut units = book_pages("Philemon", "PHM", 1..=1, 1, 10);
    let at = units
        .iter()
        .position(|u| u.text.starts_with("24 "))
        .unwrap();
    units[at].text = "24 Invented words from the age of 25 years upward to serve,".into();
    let at = units
        .iter()
        .position(|u| u.text.starts_with("25 "))
        .unwrap();
    units[at].text = "25y se retirarán del servicio.".into();
    let r = recognise(&units, "en");
    assert!(!verified(&r, "PHM", 1, 24), "{:?}", r.unverified);
    assert!(!verified(&r, "PHM", 1, 25));
    assert!(verified(&r, "PHM", 1, 23));
}

#[test]
fn a_number_opening_a_line_out_of_the_count_leaves_the_open_verse_unverified() {
    // "36 아다마와, 라마와, …": a line from another column opening with a
    // verse number that is not the next.
    let mut units = book_pages("Philemon", "PHM", 1..=1, 1, 10);
    let at = units.iter().position(|u| u.text.starts_with("5 ")).unwrap();
    units.insert(at, line("19 words of another column read here,", 1));
    let r = recognise(&units, "en");
    assert!(!verified(&r, "PHM", 1, 4), "{:?}", r.unverified);
    assert!(verified(&r, "PHM", 1, 6));
}

// --- running heads and page furniture -----------------------------------------

#[test]
fn a_long_work_title_repeated_over_the_pages_is_page_furniture() {
    // The edition's title over every page, longer than any head before.
    let mut units = Vec::new();
    for page in 1..=6u32 {
        let mut head = line(
            "THE SEPTUAGINT VERSION OF THE OLD TESTAMENT — ENGLISH TRANSLATION BY SIR",
            page,
        );
        head.large = true;
        units.push(head);
        if page == 1 {
            let mut title = line("PHILEMON", 1);
            title.large = true;
            units.push(title);
        }
        for v in (page as u16 - 1) * 5 + 1..=(page as u16 * 5).min(25) {
            units.push(line(
                &format!("{v} Invented words of the verse {v}, and"),
                page,
            ));
            units.push(line("so on to its end.", page));
        }
    }
    let r = recognise(&units, "en");
    for v in 1..=25 {
        let text = verse(&r, "PHM", 1, v).unwrap_or_default();
        assert!(!text.contains("SEPTUAGINT"), "{v}: {text}");
    }
}

#[test]
fn a_book_s_long_title_at_a_later_page_s_edge_is_a_running_head() {
    // "DAS 1. BUCH MOSE (GENESIS)" over the pages after the title's; and a
    // short book's head over the one page after its title.
    let mut units = book_pages("Der Brief des Paulus an Philemon", "PHM", 1..=1, 1, 10);
    for page in 2..=3u32 {
        let at = units
            .iter()
            .position(|u| u.place.page == Some(page))
            .unwrap();
        // Mid-sentence, as a page break falls.
        units[at - 1].text = format!("{} und", units[at - 1].text.trim_end_matches('.'));
        let mut head = line("DER BRIEF DES PAULUS AN PHILEMON", page);
        head.edge = true;
        units.insert(at, head);
        units[at + 1].text = format!("weiter {}", units[at + 1].text);
    }
    let r = recognise(&units, "de");
    for v in 1..=25 {
        let text = verse(&r, "PHM", 1, v).unwrap_or_default();
        assert!(!text.contains("PHILEMON"), "{v}: {text} {:?}", errors(&r));
    }
}

#[test]
fn a_korean_head_naming_the_page_s_first_and_last_chapter_is_furniture() {
    let mut units = book_pages("빌레몬서", "PHM", 1..=1, 1, 8);
    for page in 2..=4u32 {
        let at = units
            .iter()
            .position(|u| u.place.page == Some(page))
            .unwrap();
        let mut head = line("빌레몬서 1–빌레몬서 1", page);
        head.edge = true;
        units.insert(at, head);
    }
    let r = recognise(&units, "ko");
    for v in 1..=25 {
        let text = verse(&r, "PHM", 1, v).unwrap_or_default();
        assert!(!text.contains("빌레몬서"), "{v}: {text}");
    }
}

#[test]
fn a_page_number_read_between_the_columns_is_no_chapter() {
    // The footer's number, set larger than the text, read after the left
    // column: it runs with the pages, so it is the page's number.
    let mut units = book_pages("Philemon", "PHM", 1..=1, 1, 5);
    for page in 1..=5u32 {
        let lines: Vec<usize> = (0..units.len())
            .filter(|&i| units[i].place.page == Some(page))
            .collect();
        let middle = lines[lines.len() / 2];
        let mut number = line(&format!("{}", page + 10), page);
        number.large = true;
        units.insert(middle, number);
    }
    let r = recognise(&units, "en");
    assert!(usable(&r, "PHM", 1), "{:?}", errors(&r));
    for v in 1..=25 {
        assert_eq!(verse(&r, "PHM", 1, v), Some(words("Philemon", 1, v)));
    }
}

// --- notes ------------------------------------------------------------------------

#[test]
fn a_note_whose_number_runs_into_its_reference_is_a_note() {
    // "1993:2 a Gr. fire of flame.": note 199, on 3:2, in the body's type at
    // the foot of the column, between two lines of a verse.
    let mut units = book_pages("Philemon", "PHM", 1..=1, 1, 10);
    let at = units.iter().position(|u| u.text.starts_with("8 ")).unwrap();
    units[at].text = "8 Invented words of the verse that goes".into();
    let page = units[at].place.page.unwrap();
    units.insert(at + 1, line("1993:2 a Gr. fire of flame.", page));
    units.insert(at + 2, line("3711:52 b Or, headship, according to", page));
    let mut next = line("on to its end.", page);
    next.column = 1;
    units.insert(at + 3, next);
    let r = recognise(&units, "en");
    let text = verse(&r, "PHM", 1, 8).unwrap();
    assert!(
        !text.contains("flame") && !text.contains("headship"),
        "{text}"
    );
}

// --- study apparatus -------------------------------------------------------------

#[test]
fn a_theme_line_set_apart_between_verse_lines_is_never_verified_verse_text() {
    // A verse per paragraph ("1:1 …"), with the edition's theme lines in
    // italics between them ("*TEMA: …*"): set apart, never verse text.
    let mut units = vec![{
        let mut t = unit("Filemom", None);
        t.heading = true;
        t
    }];
    for v in 1..=25u16 {
        units.push(unit(&format!("1:{v} {}", words("Filemom", 1, v)), None));
        if v == 3 || v == 11 {
            let mut theme = unit(
                "TEMA: Sempre dou graças a Deus por vocês porque ouço falar do seu amor.",
                None,
            );
            theme.styled = true;
            units.push(theme);
        }
    }
    let r = recognise(&units, "pt");
    for v in 1..=25 {
        let text = verse(&r, "PHM", 1, v).unwrap_or_default();
        assert!(
            !text.contains("TEMA") || !verified(&r, "PHM", 1, v),
            "{v}: {text} {:?}",
            errors(&r)
        );
    }
    assert!(verified(&r, "PHM", 1, 5));
}

// --- the Septuagint ------------------------------------------------------------

fn esther_and_daniel(front: &[&str], esther: &str, daniel: &str) -> Vec<Unit> {
    // (The title page's lines set large, as titles are.)
    let mut units: Vec<Unit> = front
        .iter()
        .map(|t| {
            let mut title = line(t, 1);
            title.large = true;
            title
        })
        .collect();
    units.extend(book_pages(esther, "EST", 1..=2, 2, 20));
    units.extend(book_pages(daniel, "DAN", 1..=1, 10, 20));
    units
}

#[test]
fn a_septuagint_s_esther_and_daniel_are_the_greek_books_never_the_hebrew() {
    // The document says it is a Septuagint: its Esther and Daniel are the
    // Greek books (ESG, DAG), named and left out.
    let units = esther_and_daniel(
        &["THE SEPTUAGINT VERSION OF THE OLD TESTAMENT"],
        "ESTHER",
        "DANIEL",
    );
    let r = recognise(&units, "en");
    assert_eq!(verse(&r, "EST", 1, 1), None, "{:?}", r.books);
    assert_eq!(verse(&r, "DAN", 1, 1), None);
    let outside: Vec<_> = r.books.iter().filter_map(|b| b.outside).collect();
    assert_eq!(outside, vec!["ESG", "DAG"]);
    // A title that says it is the Greek text, without any declaration.
    let units = esther_and_daniel(&[], "ESTHER (GREEK)", "DANIEL");
    let r = recognise(&units, "en");
    assert_eq!(verse(&r, "EST", 1, 1), None);
    assert!(verse(&r, "DAN", 1, 1).is_some());
    // A preface's wrapped line of prose about the Septuagint, and a
    // subtitle under one book's title, say nothing of the whole document.
    let mut units = esther_and_daniel(&[], "ESTHER", "DANIEL");
    units.insert(0, line("Esther and Daniel from the Greek Septuagint", 1));
    let r = recognise(&units, "en");
    assert_eq!(verse(&r, "EST", 1, 1), Some(words("Esther", 1, 1)));
    // A Hebrew-based Bible whose note mentions the Septuagint keeps both.
    let mut units = esther_and_daniel(&[], "ESTHER", "DANIEL");
    let mut note = line("a 1:1 The Septuagint reads Artaxerxes", 2);
    note.note = true;
    units.insert(3, note);
    let r = recognise(&units, "en");
    assert_eq!(verse(&r, "EST", 1, 1), Some(words("Esther", 1, 1)));
    assert_eq!(verse(&r, "DAN", 1, 1), Some(words("Daniel", 1, 1)));
}

#[test]
fn a_greek_old_testament_in_lines_is_a_septuagint() {
    // "ΓΕΝΕΣΙΣ 1:1 …" lines: Greek names of Old Testament books. Its
    // "ΕΣΘΗΡ" is the Greek Esther; a book outside the 66 in such lines
    // ("ΤΩΒΙΤ 1:1") begins that book and is never read into the verse before.
    let mut units = Vec::new();
    for (name, code, chapter) in [
        ("ΓΕΝΕΣΙΣ", "GEN", 1u16),
        ("ΡΟΥΘ", "RUT", 1),
        ("ΙΩΒ", "JOB", 1),
    ] {
        let book = canon::by_code(code).unwrap();
        for v in 1..=canon::verse_count(book, chapter).unwrap() {
            units.push(unit(
                &format!("{name} {chapter}:{v} λόγοι {v} τοῦ στίχου."),
                None,
            ));
        }
    }
    units.push(unit("ΤΩΒΙΤ 1:1 λόγοι τοῦ Τωβίτ.", None));
    units.push(unit("ΤΩΒΙΤ 1:2 λόγοι τοῦ Τωβίτ.", None));
    for v in 1..=22u16 {
        units.push(unit(&format!("ΕΣΘΗΡ 1:{v} λόγοι {v} τῆς Ἐσθήρ."), None));
    }
    let r = recognise(&units, "el");
    assert_eq!(verse(&r, "EST", 1, 1), None, "{:?}", r.books);
    assert!(r.books.iter().any(|b| b.outside == Some("ESG")));
    assert!(r.books.iter().any(|b| b.outside == Some("TOB")));
    let last_of_job = verse(&r, "JOB", 1, 22).unwrap();
    assert!(!last_of_job.contains("Τωβίτ"), "{last_of_job}");
}

#[test]
fn four_books_of_kings_make_the_first_two_samuel() {
    // "KINGS I." … "KINGS III.": the four-book numbering; its first book is
    // 1 Samuel, its third 1 Kings.
    let mut units = book_pages("KINGS I.", "1SA", 1..=1, 1, 30);
    units.extend(book_pages("KINGS III.", "1KI", 1..=1, 3, 60));
    let r = recognise(&units, "en");
    assert!(verse(&r, "1SA", 1, 28).is_some(), "{:?}", r.books);
    assert!(verse(&r, "1KI", 1, 53).is_some());
    // Without a third or fourth book, "1 Kings" stays Kings.
    let units = book_pages("1 Kings", "1KI", 1..=1, 1, 60);
    let r = recognise(&units, "en");
    assert!(verse(&r, "1KI", 1, 53).is_some());
}

#[test]
fn a_septuagint_s_differently_numbered_psalms_are_refused() {
    // Psalm 22 of a Septuagint is the Hebrew Psalm 23: never filed as 22.
    let mut units = vec![{
        let mut title = line("Septuaginta", 1);
        title.large = true;
        title
    }];
    units.extend(book_pages("PSALMS", "PSA", 1..=1, 2, 30));
    units.extend({
        let mut more = book_pages("x", "PSA", 22..=22, 3, 40);
        more.remove(0);
        more
    });
    let r = recognise(&units, "en");
    assert!(usable(&r, "PSA", 1), "{:?}", errors(&r));
    assert!(!usable(&r, "PSA", 22), "{:?}", errors(&r));
}
