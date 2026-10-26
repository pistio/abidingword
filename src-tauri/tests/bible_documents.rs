//! Bible documents (v0.23): PDF, EPUB, DOCX, Markdown and plain text read as
//! Bibles, with their verses found (or not) and traced to the original.
//!
//! Every text here is invented for the tests ("Ruth 1:3 — invented words"):
//! no Bible text is used. The books and their verse counts are real, because
//! the recogniser checks chapters against the standard counts.

#[path = "support/library_fixtures.rs"]
mod fixtures;

use std::path::Path;

use abiding_word_lib::scripture::canon;
use abiding_word_lib::scripture::documents::{self, DocumentFile};
use abiding_word_lib::scripture::import::Severity;
use abiding_word_lib::scripture::structure::{self, Corrections, Recognition};

fn read(name: &str, bytes: &[u8]) -> DocumentFile {
    let extension = name.rsplit('.').next().unwrap();
    documents::read_bytes(
        Path::new(name),
        name,
        extension,
        bytes,
        bytes.len() as u64,
        0,
    )
    .unwrap()
}

fn recognise(file: &DocumentFile) -> Recognition {
    structure::recognise(
        &file.units,
        file.metadata.language.as_deref(),
        &Corrections::default(),
    )
}

/// The invented text of a verse.
fn words(book: &str, chapter: u16, verse: u16) -> String {
    format!("Invented words of {book} {chapter}:{verse} for testing.")
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

fn errors(r: &Recognition) -> Vec<String> {
    r.content
        .issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .map(|i| format!("{:?}: {}", i.code, i.message))
        .collect()
}

fn codes(r: &Recognition) -> Vec<String> {
    r.content
        .issues
        .iter()
        .filter_map(|i| i.code.clone())
        .collect()
}

/// Ruth (4 chapters: 22, 23, 18, 22 verses) as text: book heading, chapter
/// headings, one verse per line.
fn ruth_lines(heading: impl Fn(u16) -> String) -> String {
    let mut text = String::from("Ruth\n\n");
    for (chapter, count) in [(1u16, 22u16), (2, 23), (3, 18), (4, 22)] {
        text.push_str(&heading(chapter));
        text.push_str("\n\n");
        for v in 1..=count {
            text.push_str(&format!("{v} {}\n", words("Ruth", chapter, v)));
        }
        text.push('\n');
    }
    text
}

fn assert_whole_ruth(r: &Recognition) {
    assert!(errors(r).is_empty(), "{:?}", errors(r));
    assert_eq!(r.verse_count(), 85);
    for (chapter, count) in [(1u16, 22u16), (2, 23), (3, 18), (4, 22)] {
        for v in 1..=count {
            assert_eq!(
                verse(r, "RUT", chapter, v).as_deref(),
                Some(words("Ruth", chapter, v).as_str()),
                "Ruth {chapter}:{v}"
            );
        }
    }
    assert!(r.usable());
}

#[test]
fn plain_text_with_headings_and_verse_lines() {
    let file = read(
        "ruth.txt",
        ruth_lines(|c| format!("Chapter {c}")).as_bytes(),
    );
    let r = recognise(&file);
    assert_whole_ruth(&r);
    assert_eq!(r.books.len(), 1);
    assert_eq!(r.books[0].book, Some("RUT"));
    assert_eq!(r.books[0].chapters, 4);
    // Each verse traces back to its paragraph.
    let origin = r.origins[&(8, 2, 3)];
    assert!(origin.start.segment.is_some());
    assert_eq!(origin.start.page, None);
}

#[test]
fn chapter_headings_in_six_languages() {
    for (heading, book) in [
        (
            Box::new(|c: u16| format!("Kapitel {c}")) as Box<dyn Fn(u16) -> String>,
            "Rut",
        ),
        (Box::new(|c: u16| format!("Chapitre {c}")), "Ruth"),
        (Box::new(|c: u16| format!("Capítulo {c}")), "Rut"),
        (Box::new(|c: u16| format!("제{c}장")), "룻기"),
        (Box::new(|c: u16| format!("CHAPTER {c}")), "RUTH"),
        (Box::new(|c: u16| format!("Ruth {c}")), "Ruth"),
    ] {
        let text = ruth_lines(&heading).replacen("Ruth\n", &format!("{book}\n"), 1);
        let r = recognise(&read("ruth.txt", text.as_bytes()));
        assert_whole_ruth(&r);
        assert_eq!(r.books[0].book, Some("RUT"), "{book}");
    }
}

#[test]
fn printed_bible_layout_chapter_numbers_start_paragraphs() {
    // No chapter headings: each chapter's number starts its first paragraph
    // and stands for verse 1; verses run on in paragraphs.
    let mut text = String::from("Ruth\n\n");
    for (chapter, count) in [(1u16, 22u16), (2, 23), (3, 18), (4, 22)] {
        let mut paragraph = format!("{chapter} {}", words("Ruth", chapter, 1));
        for v in 2..=count {
            paragraph.push_str(&format!(" {v} {}", words("Ruth", chapter, v)));
            if v % 6 == 0 {
                paragraph.push_str("\n\n");
            }
        }
        text.push_str(&paragraph);
        text.push_str("\n\n");
    }
    let r = recognise(&read("ruth.txt", text.as_bytes()));
    assert_whole_ruth(&r);
    assert!(r.stats.numbered_chapters >= 3, "{:?}", r.stats);
}

#[test]
fn line_references_name_chapter_and_verse() {
    let mut text = String::new();
    for (chapter, count) in [(1u16, 22u16), (2, 23), (3, 18), (4, 22)] {
        for v in 1..=count {
            text.push_str(&format!(
                "Ruth {chapter}:{v} {}\n",
                words("Ruth", chapter, v)
            ));
        }
    }
    let r = recognise(&read("ruth.txt", text.as_bytes()));
    assert_whole_ruth(&r);
}

#[test]
fn markdown_headings() {
    let text = ruth_lines(|c| format!("## Chapter {c}")).replacen("Ruth\n", "# Ruth\n", 1);
    // Markdown joins the lines of a paragraph: verses one per line become one paragraph.
    let r = recognise(&read("ruth.md", text.as_bytes()));
    assert_whole_ruth(&r);
}

/// Psalms 1–3 (6, 12, 8 verses), with a title before Psalm 3's verse 1.
fn psalms_text(title: bool) -> String {
    let mut text = String::from("Psalms\n\n");
    for (psalm, count) in [(1u16, 6u16), (2, 12), (3, 8)] {
        text.push_str(&format!("Psalm {psalm}\n\n"));
        if psalm == 3 && title {
            text.push_str("An invented title for the third psalm.\n\n");
        }
        for v in 1..=count {
            text.push_str(&format!("{v} {}\n", words("Psalm", psalm, v)));
        }
        text.push('\n');
    }
    text
}

#[test]
fn a_psalm_title_before_verse_1_stays_its_title() {
    let r = recognise(&read("psalms.txt", psalms_text(true).as_bytes()));
    assert!(errors(&r).is_empty(), "{:?}", errors(&r));
    let psalms = canon::by_code("PSA").unwrap();
    let three = &r.content.books[&psalms.ordinal].chapters[&3];
    assert_eq!(
        three.superscription.as_deref(),
        Some("An invented title for the third psalm.")
    );
    assert_eq!(three.verses[&1], words("Psalm", 3, 1));
    assert_eq!(r.stats.psalm_titles, 1);
    assert!(codes(&r).contains(&"bibleImport.documentPsalmTitles".to_string()));
}

#[test]
fn a_psalm_whose_verse_1_has_no_number_is_refused_not_guessed() {
    // Title and verse 1 run together with no number between them.
    let text = psalms_text(true).replace(
        "\n1 Invented words of Psalm 3:1",
        "\nInvented words of Psalm 3:1",
    );
    let r = recognise(&read("psalms.txt", text.as_bytes()));
    assert!(
        codes(&r).contains(&"bibleImport.documentPsalmTitle".to_string()),
        "{:?}",
        codes(&r)
    );
    assert!(!r.usable());
}

#[test]
fn verse_1_without_a_number_after_a_heading_is_reported() {
    let text = ruth_lines(|c| format!("Chapter {c}")).replace(
        "\n1 Invented words of Ruth 2:1",
        "\nInvented words of Ruth 2:1",
    );
    let r = recognise(&read("ruth.txt", text.as_bytes()));
    assert_eq!(
        verse(&r, "RUT", 2, 1).as_deref(),
        Some(words("Ruth", 2, 1).as_str())
    );
    assert_eq!(r.stats.unnumbered_first, 1);
    assert!(codes(&r).contains(&"bibleImport.documentUnnumbered".to_string()));
    assert!(r.usable());
}

#[test]
fn a_missing_verse_is_reported_and_its_neighbours_keep_their_text() {
    // Ruth 2:7's number is missing from the text: its words stay with 2:6 (reported).
    let text = ruth_lines(|c| format!("Chapter {c}")).replace(
        "\n7 Invented words of Ruth 2:7",
        "\nInvented words of Ruth 2:7",
    );
    let r = recognise(&read("ruth.txt", text.as_bytes()));
    assert!(errors(&r).is_empty(), "{:?}", errors(&r));
    assert_eq!(verse(&r, "RUT", 2, 7), None);
    assert_eq!(
        verse(&r, "RUT", 2, 6).as_deref(),
        Some(format!("{} {}", words("Ruth", 2, 6), words("Ruth", 2, 7)).as_str())
    );
    assert_eq!(
        verse(&r, "RUT", 2, 8).as_deref(),
        Some(words("Ruth", 2, 8).as_str())
    );
    let missing = r
        .content
        .issues
        .iter()
        .find(|i| i.code.as_deref() == Some("bibleImport.missingVerses"))
        .unwrap();
    assert_eq!(missing.chapter, Some(2));
}

#[test]
fn two_equal_places_for_one_verse_refuse_the_chapter() {
    // "… away. 3 And" twice before verse 4: which is verse 3 cannot be known.
    let text = ruth_lines(|c| format!("Chapter {c}")).replace(
        "\n3 Invented words of Ruth 1:3 for testing.\n",
        "\n3 Invented words of Ruth 1:3. 3 More invented words.\n",
    );
    let r = recognise(&read("ruth.txt", text.as_bytes()));
    assert!(
        codes(&r).contains(&"bibleImport.documentAmbiguous".to_string()),
        "{:?}",
        codes(&r)
    );
    assert!(!r.usable());
}

#[test]
fn numbers_inside_the_text_are_not_verse_numbers() {
    // Counts, references and ordinals in verse text stay text.
    let text = ruth_lines(|c| format!("Chapter {c}"))
        .replace(
            "\n4 Invented words of Ruth 1:4 for testing.\n",
            "\n4 Ruth 1:4 lived there 10 years, see chapter 5 and John 5 and the 5th day, 3:5 too.\n",
        )
        .replace(
            "\n9 Invented words of Ruth 1:9 for testing.\n",
            "\n9 Ruth 1:9 invented words; it was 2,000 cubits and 40% of the field. 12 men came.\n",
        );
    let r = recognise(&read("ruth.txt", text.as_bytes()));
    assert!(errors(&r).is_empty(), "{:?}", errors(&r));
    assert_eq!(
        verse(&r, "RUT", 1, 4).as_deref(),
        Some("Ruth 1:4 lived there 10 years, see chapter 5 and John 5 and the 5th day, 3:5 too.")
    );
    assert_eq!(
        verse(&r, "RUT", 1, 5).as_deref(),
        Some(words("Ruth", 1, 5).as_str())
    );
    assert_eq!(
        verse(&r, "RUT", 1, 9).as_deref(),
        // A number after a sentence end, ahead of the count, is not a
        // second column read across: the count does not go on from it.
        Some("Ruth 1:9 invented words; it was 2,000 cubits and 40% of the field. 12 men came.")
    );
}

#[test]
fn headings_and_notes_inside_chapters_are_left_out() {
    let mut text = String::from("# Ruth\n\n## Chapter 1\n\n");
    for v in 1..=22u16 {
        if v == 6 {
            text.push_str("### An invented section heading\n\n");
        }
        text.push_str(&format!("{v} {}\n\n", words("Ruth", 1, v)));
    }
    text.push_str("## Notes\n\n1 An invented footnote that is not Scripture.\n\n");
    let r = recognise(&read("ruth.md", text.as_bytes()));
    assert!(errors(&r).is_empty(), "{:?}", errors(&r));
    assert_eq!(
        verse(&r, "RUT", 1, 5).as_deref(),
        Some(words("Ruth", 1, 5).as_str())
    );
    assert_eq!(
        verse(&r, "RUT", 1, 22).as_deref(),
        Some(words("Ruth", 1, 22).as_str())
    );
    assert_eq!(r.stats.headings, 1);
    assert!(r
        .left_out
        .iter()
        .any(|l| l.reason == "notes" && l.sample.contains("footnote")));
    assert!(r.left_out.iter().any(|l| l.reason == "heading"));
}

#[test]
fn text_before_any_book_is_left_out_and_reported() {
    let text = format!(
        "Preface\n\nAn invented preface with 1 number in it.\n\n{}",
        ruth_lines(|c| format!("Chapter {c}"))
    );
    let r = recognise(&read("ruth.txt", text.as_bytes()));
    assert_whole_ruth(&r);
    assert!(r
        .left_out
        .iter()
        .any(|l| l.reason == "noBook" && l.sample.contains("preface")));
}

#[test]
fn verse_numbers_without_any_book_give_no_verses() {
    let mut text = String::new();
    for v in 1..=10u16 {
        text.push_str(&format!("{v} An invented line.\n"));
    }
    let r = recognise(&read("plain.txt", text.as_bytes()));
    assert_eq!(r.verse_count(), 0);
    assert!(!r.usable());
    assert!(codes(&r).contains(&"bibleImport.documentNoBook".to_string()));
}

#[test]
fn a_text_that_loses_its_numbers_is_refused_as_too_long() {
    // Ruth 1 with verse numbers to 3, then running text with no numbers at all.
    let mut text =
        String::from("Ruth\n\nChapter 1\n\n1 First invented verse.\n2 Second invented verse.\n3 ");
    for _ in 0..120 {
        text.push_str("An invented sentence with no verse number in it at all. ");
    }
    let r = recognise(&read("ruth.txt", text.as_bytes()));
    assert!(
        codes(&r).contains(&"bibleImport.documentLongVerse".to_string()),
        "{:?}",
        codes(&r)
    );
    assert!(!r.usable());
}

#[test]
fn korean_headings_and_verses() {
    let mut text = String::from("룻기\n\n");
    for (chapter, count) in [(1u16, 22u16), (2, 23), (3, 18), (4, 22)] {
        text.push_str(&format!("제 {chapter} 장\n\n"));
        for v in 1..=count {
            // Korean verses often end without punctuation.
            text.push_str(&format!(
                "{v} 시험을 위해 지어낸 룻기 {chapter}장 {v}절 문장이라 "
            ));
        }
        text.push_str("\n\n");
    }
    let r = recognise(&read("ruth-ko.txt", text.as_bytes()));
    assert!(errors(&r).is_empty(), "{:?}", errors(&r));
    assert_eq!(r.verse_count(), 85);
    assert_eq!(
        verse(&r, "RUT", 2, 5).as_deref(),
        Some("시험을 위해 지어낸 룻기 2장 5절 문장이라")
    );
}

#[test]
fn hebrew_right_to_left_text_keeps_its_points() {
    // Obadiah (one chapter, 21 verses), headed in Hebrew, verse text pointed and
    // ended with sof pasuq; bidirectional marks around the numbers.
    let mut text = String::from("עובדיה\n\nפרק א\n\n");
    for v in 1..=21u16 {
        text.push_str(&format!("\u{200F}{v}\u{200F} דְּבַר־בְּדִיקָה מִסְפָּר {v}׃ "));
    }
    let r = recognise(&read("obadiah-he.txt", text.as_bytes()));
    assert!(errors(&r).is_empty(), "{:?}", errors(&r));
    assert_eq!(r.books[0].book, Some("OBA"));
    assert_eq!(r.verse_count(), 21);
    // The verse's own number inside its text ("מִסְפָּר 3׃") is not the next verse.
    assert_eq!(verse(&r, "OBA", 1, 3).as_deref(), Some("דְּבַר־בְּדִיקָה מִסְפָּר 3׃"));
}

#[test]
fn greek_book_title() {
    let mut text = String::from("ΙΟΥΔΑ\n\n");
    for v in 1..=25u16 {
        text.push_str(&format!("{v} δοκιμαστικὸς λόγος {v}. "));
    }
    let r = recognise(&read("jude-grc.txt", text.as_bytes()));
    assert!(errors(&r).is_empty(), "{:?}", errors(&r));
    assert_eq!(r.books[0].book, Some("JUD"));
    assert_eq!(r.verse_count(), 25);
}

#[test]
fn a_title_that_gives_a_second_name_names_its_own_book() {
    // "The First Book of Samuel Otherwise Called the First Book of the Kings"
    // is 1 Samuel; "Commonly Called the Third Book of the Kings" under 1 Kings
    // is another name for it, not a new book.
    let mut text = String::from(
        "The First Book of Samuel Otherwise Called the\n\nFirst Book of the Kings\n\n",
    );
    for v in 1..=28u16 {
        text.push_str(&format!("{v} {}\n", words("1 Samuel", 1, v)));
    }
    text.push_str(
        "\nThe First Book of the Kings\n\nCommonly Called the Third Book of the Kings\n\n",
    );
    for v in 1..=53u16 {
        text.push_str(&format!("{v} {}\n", words("1 Kings", 1, v)));
    }
    let r = recognise(&read("kings.txt", text.as_bytes()));
    let books: Vec<_> = r.books.iter().map(|b| b.book).collect();
    assert_eq!(books, vec![Some("1SA"), Some("1KI")]);
    assert_eq!(
        verse(&r, "1SA", 1, 28).as_deref(),
        Some(words("1 Samuel", 1, 28).as_str())
    );
    assert_eq!(
        verse(&r, "1KI", 1, 53).as_deref(),
        Some(words("1 Kings", 1, 53).as_str())
    );
    // On one line, longer than a heading usually is; "called", "or".
    let detected = |title: &str| {
        let mut text = format!("{title}\n\n");
        for v in 1..=20u16 {
            text.push_str(&format!("{v} {}\n", words("Book", 1, v)));
        }
        recognise(&read("t.txt", text.as_bytes()))
            .books
            .first()
            .and_then(|b| b.detected)
    };
    assert_eq!(
        detected("The First Book of Samuel Otherwise Called the First Book of the Kings"),
        Some("1SA")
    );
    assert_eq!(
        detected("The First Book of Moses, called Genesis"),
        Some("GEN")
    );
    assert_eq!(detected("Ecclesiastes or, the Preacher"), Some("ECC"));
}

#[test]
fn a_chapter_far_longer_than_usual_is_refused() {
    // Ruth 1 has 22 verses; 40 numbered verses under it mean another chapter
    // or book was read into it (a missed title). Never used as Ruth 1.
    let mut text = String::from("Ruth\n\nChapter 1\n\n");
    for v in 1..=40u16 {
        text.push_str(&format!("{v} {}\n", words("Ruth", 1, v)));
    }
    let r = recognise(&read("ruth.txt", text.as_bytes()));
    assert!(
        codes(&r).contains(&"bibleImport.documentChapterLong".to_string()),
        "{:?}",
        codes(&r)
    );
    // An error on Ruth 1: the chapter is not used.
    assert!(r
        .content
        .issues
        .iter()
        .any(|i| i.severity == Severity::Error
            && i.code.as_deref() == Some("bibleImport.documentChapterLong")
            && i.book == Some("RUT")
            && i.chapter == Some(1)));
    assert!(!r.usable());
}

#[test]
fn a_title_without_its_number_follows_the_book_before_it() {
    // "Crónicas" over both books of Chronicles (Reina-Valera): the first
    // after 2 Kings is 1 Chronicles, the next 2 Chronicles.
    let mut text = String::from("2 Reyes\n\n");
    for v in 1..=18u16 {
        text.push_str(&format!("{v} {}\n", words("2 Reyes", 1, v)));
    }
    for (book, count) in [("1 Crónicas", 54u16), ("2 Crónicas", 17)] {
        text.push_str("\nCrónicas\n\n");
        for v in 1..=count {
            text.push_str(&format!("{v} {}\n", words(book, 1, v)));
        }
    }
    let r = structure::recognise(
        &read("cronicas.txt", text.as_bytes()).units,
        Some("es"),
        &Corrections::default(),
    );
    let books: Vec<_> = r.books.iter().map(|b| b.book).collect();
    assert_eq!(books, vec![Some("2KI"), Some("1CH"), Some("2CH")]);
    // A subtitle right under a title ("1 Crónicas" / "Crónicas") is never
    // the next book.
    let text = format!(
        "1 Crónicas\n\nCrónicas\n\n{}",
        (1..=54u16)
            .map(|v| format!("{v} {}\n", words("1 Crónicas", 1, v)))
            .collect::<String>()
    );
    let subtitle = structure::recognise(
        &read("cronicas.txt", text.as_bytes()).units,
        Some("es"),
        &Corrections::default(),
    );
    let books: Vec<_> = subtitle.books.iter().map(|b| b.book).collect();
    assert_eq!(books, vec![Some("1CH")]);
    assert_eq!(
        verse(&r, "2CH", 1, 17).as_deref(),
        Some(words("2 Crónicas", 1, 17).as_str())
    );
}

#[test]
fn books_outside_the_66_are_named_and_left_out() {
    // A Bible with the deuterocanon (Bíblia Portuguesa Mundial): Tobit and
    // the Greek text of Esther are named and left out, never read into
    // the book before them.
    let chapter = |book: &str, title: &str, count: u16| {
        let mut text = format!("{title}\n\n");
        for v in 1..=count {
            text.push_str(&format!("{v} {}\n", words(book, 1, v)));
        }
        text.push('\n');
        text
    };
    let text = [
        chapter("Ester", "ESTER", 22),
        chapter("Tobit", "TOBIT", 22),
        chapter("Ester grega", "ESTER\ntraduzido da Septuaginta grega", 22),
        chapter("Mateus", "MATEUS", 25),
    ]
    .concat();
    let r = structure::recognise(
        &read("deuterocanon.txt", text.as_bytes()).units,
        Some("pt"),
        &Corrections::default(),
    );
    let books: Vec<_> = r
        .books
        .iter()
        .map(|b| (b.book, b.outside, b.left_out))
        .collect();
    assert_eq!(
        books,
        vec![
            (Some("EST"), None, false),
            (None, Some("TOB"), true),
            (None, Some("ESG"), true),
            (Some("MAT"), None, false)
        ]
    );
    assert_eq!(
        verse(&r, "EST", 1, 22).as_deref(),
        Some(words("Ester", 1, 22).as_str())
    );
    assert_eq!(
        verse(&r, "MAT", 1, 25).as_deref(),
        Some(words("Mateus", 1, 25).as_str())
    );
    assert!(r.usable(), "{:?}", codes(&r));
    assert_eq!(
        codes(&r)
            .iter()
            .filter(|c| *c == "bibleImport.documentAdditionalBook")
            .count(),
        2
    );
}

#[test]
fn lines_named_by_usfm_codes() {
    // One verse per line, each named by the book's USFM code ("1SA 1:1 …"),
    // as Bible exports write them.
    let mut text = String::new();
    for v in 1..=28u16 {
        text.push_str(&format!("1SA 1:{v} {}\n", words("1 Samuel", 1, v)));
    }
    for v in 1..=17u16 {
        text.push_str(&format!("SNG 1:{v} {}\n", words("Song", 1, v)));
    }
    let r = recognise(&read("export.txt", text.as_bytes()));
    assert_eq!(
        verse(&r, "1SA", 1, 28).as_deref(),
        Some(words("1 Samuel", 1, 28).as_str())
    );
    assert_eq!(
        verse(&r, "SNG", 1, 17).as_deref(),
        Some(words("Song", 1, 17).as_str())
    );
}

#[test]
fn corrections_choose_or_leave_out_a_book() {
    // A title the dictionaries do not know, before "Chapter 1".
    let text = ruth_lines(|c| format!("Chapter {c}")).replacen(
        "Ruth\n",
        "The Story of the Moabite Woman\n",
        1,
    );
    let file = read("ruth.txt", text.as_bytes());
    let r = recognise(&file);
    assert_eq!(r.books.len(), 1);
    assert_eq!(r.books[0].detected, None);
    assert_eq!(r.verse_count(), 0);
    // The person says which book it is.
    let mut corrections = Corrections::default();
    corrections.books.insert(0, canon::by_code("RUT"));
    let r = structure::recognise(&file.units, None, &corrections);
    assert_whole_ruth(&r);
    assert_eq!(r.books[0].book, Some("RUT"));
    // … or leaves it out.
    corrections.books.insert(0, None);
    let r = structure::recognise(&file.units, None, &corrections);
    assert_eq!(r.verse_count(), 0);
    assert!(r.books[0].left_out);
}

#[test]
fn a_book_that_begins_twice_is_refused() {
    let text = format!(
        "{}{}",
        ruth_lines(|c| format!("Chapter {c}")),
        ruth_lines(|c| format!("Chapter {c}"))
    );
    let r = recognise(&read("ruth.txt", text.as_bytes()));
    assert!(
        codes(&r).contains(&"bibleImport.documentChapterTwice".to_string()),
        "{:?}",
        codes(&r)
    );
    assert!(!codes(&r).contains(&"bibleImport.verseTwice".to_string()));
    assert!(!r.usable());
}

/// Ruth as PDF pages: a running head and a page number on every page, the
/// verses run on, and a verse broken across two pages.
fn ruth_pdf() -> Vec<u8> {
    let mut lines: Vec<String> = vec!["Ruth".into()];
    for (chapter, count) in [(1u16, 22u16), (2, 23), (3, 18), (4, 22)] {
        lines.push(format!("Chapter {chapter}"));
        for v in 1..=count {
            lines.push(format!("{v} {}", words("Ruth", chapter, v)));
        }
    }
    // Ruth 2:5 is broken over a page end.
    let at = lines
        .iter()
        .position(|l| l.starts_with("5 Invented words of Ruth 2:5"))
        .unwrap();
    lines[at] = "5 Invented words of Ruth 2:5".into();
    lines.insert(at + 1, "for testing.".into());
    let mut pages: Vec<Vec<String>> = Vec::new();
    let mut page: Vec<String> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        page.push(line.clone());
        if page.len() == 9 || i == at {
            pages.push(std::mem::take(&mut page));
        }
    }
    if !page.is_empty() {
        pages.push(page);
    }
    let count = pages.len();
    let pages: Vec<Vec<String>> = pages
        .into_iter()
        .enumerate()
        .map(|(n, mut lines)| {
            lines.insert(0, "THE BOOK OF RUTH".into());
            lines.push(format!("{}", n + 1));
            lines
        })
        .collect();
    assert!(count > 6);
    let refs: Vec<Vec<&str>> = pages
        .iter()
        .map(|p| p.iter().map(String::as_str).collect())
        .collect();
    let refs: Vec<&[&str]> = refs.iter().map(Vec::as_slice).collect();
    fixtures::pdf(&refs, Some("Ruth (invented test text)"), None)
}

#[test]
fn pdf_with_a_text_layer_running_heads_and_a_verse_across_pages() {
    let file = read("ruth.pdf", &ruth_pdf());
    assert_eq!(file.format, "pdf");
    assert!(file.page_count.unwrap() > 6);
    let r = recognise(&file);
    assert_whole_ruth(&r);
    // Running heads and page numbers were taken out, and counted.
    assert!(
        r.stats.page_lines >= 2 * file.page_count.unwrap() as usize - 1,
        "{:?}",
        r.stats
    );
    // The broken verse is whole, and its origin spans two pages.
    let origin = r.origins[&(8, 2, 5)];
    assert_eq!(origin.end.page.unwrap(), origin.start.page.unwrap() + 1);
    // Every verse names its page.
    assert!(r.origins.values().all(|o| o.start.page.is_some()));
}

#[test]
fn a_scanned_pdf_has_pages_but_no_text() {
    let bytes = fixtures::pdf(&[&[], &[], &[]], Some("Scanned"), None);
    let file = read("scanned.pdf", &bytes);
    assert_eq!(file.extraction_status.as_str(), "ocr_required");
    assert_eq!(file.page_count, Some(3));
    assert!(file.segments.is_empty() && file.units.is_empty());
    assert_eq!(recognise(&file).verse_count(), 0);
}

#[test]
fn a_malformed_pdf_is_refused_with_a_message() {
    let error = documents::read_bytes(
        Path::new("x.pdf"),
        "x.pdf",
        "pdf",
        b"%PDF-1.7 not really",
        19,
        0,
    )
    .unwrap_err();
    assert!(format!("{error:?}").contains("library.badPdf"));
    let error = documents::read_bytes(
        Path::new("x.epub"),
        "x.epub",
        "epub",
        b"PK\x03\x04broken",
        10,
        0,
    )
    .unwrap_err();
    assert!(
        format!("{error:?}").contains("library.badEpub"),
        "{error:?}"
    );
    let error = documents::read_bytes(Path::new("x.docx"), "x.docx", "docx", b"not a zip", 9, 0)
        .unwrap_err();
    assert!(
        format!("{error:?}").contains("library.badDocx"),
        "{error:?}"
    );
}

#[test]
fn epub_with_book_and_chapter_headings() {
    let mut chapters = Vec::new();
    let mut bodies = Vec::new();
    for (chapter, count) in [(1u16, 22u16), (2, 23), (3, 18), (4, 22)] {
        let mut body = String::new();
        if chapter == 1 {
            body.push_str("<h1>Ruth</h1>");
        }
        body.push_str(&format!("<h2>Chapter {chapter}</h2><p>"));
        for v in 1..=count {
            body.push_str(&format!(
                "<sup class=\"v\">{v}</sup>{} ",
                words("Ruth", chapter, v)
            ));
            if v % 5 == 0 {
                body.push_str("</p><p>");
            }
        }
        body.push_str("</p>");
        bodies.push(body);
    }
    let files: Vec<String> = (1..=4).map(|c| format!("c{c}.xhtml")).collect();
    for (i, body) in bodies.iter().enumerate() {
        chapters.push(fixtures::EpubChapter {
            file: &files[i],
            xhtml_body: body,
        });
    }
    let bytes = fixtures::epub(
        "<dc:title>Ruth (invented)</dc:title><dc:language>en</dc:language>",
        &chapters,
        &[0, 1, 2, 3],
    );
    let file = read("ruth.epub", &bytes);
    assert_eq!(file.format, "epub");
    let r = recognise(&file);
    assert_whole_ruth(&r);
    // Origins point at kept segments (paragraphs) of the EPUB.
    let origin = r.origins[&(8, 4, 22)];
    let segment = origin.start.segment.unwrap() as usize;
    assert!(file.segments[segment].text.contains("Ruth 4:22"));
}

#[test]
fn docx_with_headings_numbered_verses_and_footnotes() {
    let heading = |level: u8, text: &str| {
        format!(
            r#"<w:p><w:pPr><w:pStyle w:val="Heading{level}"/></w:pPr><w:r><w:t>{text}</w:t></w:r></w:p>"#
        )
    };
    let para =
        |text: &str| format!(r#"<w:p><w:r><w:t xml:space="preserve">{text}</w:t></w:r></w:p>"#);
    let mut body = heading(1, "Ruth");
    for (chapter, count) in [(1u16, 22u16), (2, 23), (3, 18), (4, 22)] {
        body.push_str(&heading(2, &format!("Chapter {chapter}")));
        for v in 1..=count {
            let mut text = format!("{v} {}", words("Ruth", chapter, v));
            if chapter == 1 && v == 2 {
                body.push_str(&format!(
                    r#"<w:p><w:r><w:t xml:space="preserve">{text}</w:t></w:r><w:r><w:footnoteReference w:id="1"/></w:r></w:p>"#
                ));
                continue;
            }
            text.push(' ');
            body.push_str(&para(&text));
        }
    }
    let styles = br#"<?xml version="1.0"?><w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/></w:style><w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/></w:style></w:styles>"#;
    let notes = br#"<?xml version="1.0"?><w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:footnote w:id="1"><w:p><w:r><w:t>3 An invented footnote that must not join a verse.</w:t></w:r></w:p></w:footnote></w:footnotes>"#;
    let rels = r#"<Relationship Id="rS" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/><Relationship Id="rN" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes" Target="footnotes.xml"/>"#;
    let bytes = fixtures::docx(
        &body,
        rels,
        &[("word/styles.xml", styles), ("word/footnotes.xml", notes)],
    );
    let file = read("ruth.docx", &bytes);
    assert_eq!(file.format, "docx");
    assert_eq!(file.notes, 1, "the footnote is kept apart");
    let r = recognise(&file);
    assert_whole_ruth(&r);
    assert!(!verse(&r, "RUT", 4, 22).unwrap().contains("footnote"));
}

// --- adding a Bible from documents: the import, the database, removal -----------------

use abiding_word_lib::db::bible_documents::{self as kept_documents, KeptFile};
use abiding_word_lib::db::bibles::{self, SourceRecord};
use abiding_word_lib::db::migrations::{self, MIGRATIONS};
use abiding_word_lib::db::Database;
use abiding_word_lib::scripture::import::{self, ImportOrigin, StagedImport};
use abiding_word_lib::scripture::CommitImport;
use rusqlite::Connection;
use std::path::PathBuf;

fn write(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

fn details(staged: &StagedImport, short: &str, as_document: bool) -> CommitImport {
    CommitImport {
        preview_id: staged.id.clone(),
        name: format!("Invented {short}"),
        short_name: short.into(),
        language: "English".into(),
        language_code: Some("en".into()),
        publisher: None,
        copyright_notice: None,
        license_type: None,
        words: Default::default(),
        as_document,
    }
}

fn add(
    conn: &Connection,
    staged: &StagedImport,
    short: &str,
    as_document: bool,
) -> abiding_word_lib::error::AppResult<String> {
    let id = bibles::new_translation_id();
    let stage = staged.documents.as_ref().unwrap();
    let kept: Vec<KeptFile> = stage
        .files
        .iter()
        .enumerate()
        .map(|(i, f)| KeptFile {
            id: format!("{id}-doc-{i}"),
            file_name: format!("{}.{}", i + 1, f.extension),
        })
        .collect();
    kept_documents::commit(
        conn,
        &id,
        &staged.content,
        stage,
        &details(staged, short, as_document),
        &SourceRecord {
            origin: "managed_copy",
            path: Some(format!("sources/bibles/{id}")),
            original_file_name: Some("ruth".into()),
            format: staged.format,
            is_builtin: false,
        },
        &kept,
        !as_document,
    )
    .map(|t| t.id)
}

fn count(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |r| r.get(0)).unwrap()
}

#[test]
fn a_text_bible_is_staged_as_a_document_import_with_its_verses() {
    let temp = tempfile::tempdir().unwrap();
    let path = write(
        temp.path(),
        "ruth.txt",
        ruth_lines(|c| format!("Chapter {c}")).as_bytes(),
    );
    let staged = import::stage(&[path], ImportOrigin::UserFiles).unwrap();
    assert_eq!(staged.format, "text");
    let preview = staged.preview(Default::default());
    let document = preview.document.expect("a document preview");
    assert!(document.verses_usable);
    assert_eq!(preview.verse_count, 85);
    assert!(preview.can_import);
    assert_eq!(document.files.len(), 1);
    assert!(!document.files[0].scanned);
    assert!(!document.files[0].keeps_layout);
    // Ordinary text has no word annotations, and none are made up.
    assert_eq!(preview.words.words, 0);
    assert!(!preview.words.indexed);
}

#[test]
fn a_usfm_text_file_is_still_read_as_usfm() {
    let temp = tempfile::tempdir().unwrap();
    let usfm = "\\id RUT\n\\c 1\n\\v 1 Invented words.\n";
    let path = write(temp.path(), "ruth.txt", usfm.as_bytes());
    let staged = import::stage(&[path], ImportOrigin::UserFiles).unwrap();
    assert_eq!(staged.format, "usfm");
    assert!(staged.documents.is_none());
}

#[test]
fn documents_and_bible_format_files_together_are_refused() {
    let temp = tempfile::tempdir().unwrap();
    let a = write(temp.path(), "ruth.pdf", &ruth_pdf());
    let b = write(temp.path(), "ruth.usfm", b"\\id RUT\n\\c 1\n\\v 1 x\n");
    let error = import::stage(&[a, b], ImportOrigin::UserFiles)
        .err()
        .unwrap();
    assert!(format!("{error:?}").contains("documentMixed"));
}

#[test]
fn added_by_verse_it_reads_as_any_bible_and_remembers_where_each_verse_was() {
    let db = Database::open_in_memory().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let path = write(temp.path(), "ruth.pdf", &ruth_pdf());
    let staged = import::stage(&[path], ImportOrigin::UserFiles).unwrap();
    db.with_conn(|conn| {
        let id = add(conn, &staged, "RUTHPDF", false).unwrap();
        let translation = bibles::get(conn, &id).unwrap();
        assert_eq!(translation.kind, "structured");
        assert_eq!(translation.import_status, "ready");
        assert_eq!(translation.source_type, "pdf");
        assert_eq!(translation.verse_count, 85);
        let chapter = bibles::chapter(conn, &id, "RUT", 2).unwrap();
        assert_eq!(chapter.verses.len(), 23);
        assert_eq!(chapter.verses[4].text, words("Ruth", 2, 5));
        // The documents and their text are kept, for reading and searching.
        let documents = kept_documents::list(conn, &id).unwrap();
        assert_eq!(documents.len(), 1);
        assert_eq!(documents[0].view, Some("pdf"));
        assert!(documents[0].segment_count > 80);
        let hits = kept_documents::search(conn, &id, "Ruth 3:7").unwrap();
        assert!(!hits.is_empty());
        assert!(hits.iter().all(|h| h.page_number.is_some()));
        // Ruth 2:5 runs over two pages.
        let origin = kept_documents::origin(conn, &id, abiding_word_lib::scripture::reference::verse_key(8, 2, 5)).unwrap().unwrap();
        assert_eq!(origin.last_page.unwrap(), origin.first_page.unwrap() + 1);
        assert_eq!(origin.document_name, "ruth.pdf");
        // Removing the Bible removes all of it.
        bibles::remove(conn, &id).unwrap();
        assert_eq!(count(conn, "SELECT COUNT(*) FROM bible_documents"), 0);
        assert_eq!(count(conn, "SELECT COUNT(*) FROM bible_document_segments"), 0);
        assert_eq!(count(conn, "SELECT COUNT(*) FROM bible_verse_origins"), 0);
        assert_eq!(count(conn, "SELECT COUNT(*) FROM bible_document_search WHERE bible_document_search MATCH 'Ruth'"), 0);
        Ok(())
    })
    .unwrap();
}

#[test]
fn added_as_a_document_it_has_no_verses_and_can_be_given_them_later() {
    let db = Database::open_in_memory().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let path = write(
        temp.path(),
        "ruth.md",
        ruth_lines(|c| format!("## Chapter {c}"))
            .replacen("Ruth\n", "# Ruth\n", 1)
            .as_bytes(),
    );
    let staged = import::stage(std::slice::from_ref(&path), ImportOrigin::UserFiles).unwrap();
    db.with_conn(|conn| {
        let id = add(conn, &staged, "RUTHDOC", true).unwrap();
        let translation = bibles::get(conn, &id).unwrap();
        assert_eq!(translation.kind, "document");
        assert_eq!(translation.import_status, "document_only");
        assert_eq!(translation.verse_count, 0);
        assert_eq!(count(conn, "SELECT COUNT(*) FROM bible_verse_texts"), 0);
        assert!(bibles::chapter(conn, &id, "RUT", 1)
            .unwrap()
            .verses
            .is_empty());
        // Its text can be read and searched.
        let documents = kept_documents::list(conn, &id).unwrap();
        assert_eq!(documents[0].view, None);
        let window = kept_documents::segments(conn, &documents[0].id, 0, 10, None).unwrap();
        assert_eq!(
            window.segments.len(),
            4.min(window.total as usize).max(window.segments.len())
        );
        assert!(!kept_documents::search(conn, &id, "invented")
            .unwrap()
            .is_empty());
        // Later: read again for verses, without adding it again.
        let again = import::stage_documents(
            std::slice::from_ref(&path),
            ImportOrigin::Existing {
                translation_id: id.clone(),
            },
        )
        .unwrap();
        let translation = kept_documents::add_verses(
            conn,
            &id,
            &again.content,
            again.documents.as_ref().unwrap(),
            false,
        )
        .unwrap();
        assert_eq!(translation.kind, "structured");
        assert_eq!(translation.verse_count, 85);
        assert_eq!(
            bibles::chapter(conn, &id, "RUT", 4).unwrap().verses.len(),
            22
        );
        assert_eq!(count(conn, "SELECT COUNT(*) FROM bible_verse_origins"), 85);
        Ok(())
    })
    .unwrap();
}

#[test]
fn verses_with_problems_are_never_saved_and_nothing_is_left_behind() {
    let db = Database::open_in_memory().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let text = format!(
        "{}{}",
        ruth_lines(|c| format!("Chapter {c}")),
        ruth_lines(|c| format!("Chapter {c}"))
    );
    let path = write(temp.path(), "ruth.txt", text.as_bytes());
    let staged = import::stage(&[path], ImportOrigin::UserFiles).unwrap();
    assert!(
        !staged
            .preview(Default::default())
            .document
            .unwrap()
            .verses_usable
    );
    db.with_conn(|conn| {
        let error = add(conn, &staged, "BROKEN", false).unwrap_err();
        assert!(format!("{error:?}").contains("hasProblems"));
        assert_eq!(
            count(
                conn,
                "SELECT COUNT(*) FROM bible_translations WHERE short_name = 'BROKEN'"
            ),
            0
        );
        assert_eq!(count(conn, "SELECT COUNT(*) FROM bible_documents"), 0);
        // As a document it can be added.
        add(conn, &staged, "BROKEN", true).unwrap();
        Ok(())
    })
    .unwrap();
}

#[test]
fn a_cancelled_import_writes_nothing() {
    let db = Database::open_in_memory().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let path = write(temp.path(), "ruth.pdf", &ruth_pdf());
    let before = db
        .with_conn(|conn| Ok(count(conn, "SELECT COUNT(*) FROM bible_translations")))
        .unwrap();
    let staged = import::stage(&[path], ImportOrigin::UserFiles).unwrap();
    drop(staged);
    db.with_conn(|conn| {
        assert_eq!(
            count(conn, "SELECT COUNT(*) FROM bible_translations"),
            before
        );
        assert_eq!(count(conn, "SELECT COUNT(*) FROM bible_documents"), 0);
        assert_eq!(
            count(conn, "SELECT COUNT(*) FROM bible_document_segments"),
            0
        );
        Ok(())
    })
    .unwrap();
}

#[test]
fn a_scanned_pdf_can_be_added_to_read_page_by_page() {
    let db = Database::open_in_memory().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let path = write(
        temp.path(),
        "scan.pdf",
        &fixtures::pdf(&[&[], &[], &[], &[]], Some("Scan"), None),
    );
    let staged = import::stage(&[path], ImportOrigin::UserFiles).unwrap();
    let preview = staged.preview(Default::default());
    let document = preview.document.unwrap();
    assert!(document.files[0].scanned);
    assert_eq!(document.files[0].pages, Some(4));
    assert!(!document.verses_usable);
    assert!(preview.can_import);
    db.with_conn(|conn| {
        let id = add(conn, &staged, "SCAN", true).unwrap();
        let documents = kept_documents::list(conn, &id).unwrap();
        assert_eq!(documents[0].extraction_status, "ocr_required");
        assert_eq!(documents[0].page_count, Some(4));
        assert_eq!(documents[0].segment_count, 0);
        Ok(())
    })
    .unwrap();
}

#[test]
fn corrections_in_the_review_read_the_documents_again() {
    let temp = tempfile::tempdir().unwrap();
    let text = ruth_lines(|c| format!("Chapter {c}")).replacen(
        "Ruth\n",
        "The Story of the Moabite Woman\n",
        1,
    );
    let path = write(temp.path(), "ruth.txt", text.as_bytes());
    let mut staged = import::stage(&[path], ImportOrigin::UserFiles).unwrap();
    assert_eq!(staged.content.verse_count(), 0);
    staged.correct(&[(0, Some("RUT".into()))]).unwrap();
    assert_eq!(staged.content.verse_count(), 85);
    let preview = staged.preview(Default::default());
    assert_eq!(preview.document.unwrap().books[0].book, Some("RUT"));
}

/// A dev.42 database (schema 18) with an imported Bible, a note on a verse and
/// a library source, upgraded to schema 19: backed up first, every row kept,
/// the new tables empty.
#[test]
fn a_dev42_database_upgrades_and_keeps_everything() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("abiding-word.db");
    let backups = temp.path().join("pre-migration");
    {
        let mut conn = Connection::open(&path).unwrap();
        migrations::run_with(&mut conn, &MIGRATIONS[..18]).unwrap();
        let t = "2026-10-22T10:00:00.000Z";
        conn.execute_batch(&format!(
            "INSERT INTO bible_translations (id, name, short_name, language, kind, source_type, source_origin,
                 import_status, book_count, verse_count, created_at, updated_at)
             VALUES ('tr-1', 'Invented', 'INV', 'English', 'structured', 'usfm', 'managed_copy', 'ready', 1, 1, '{t}', '{t}');
             INSERT OR IGNORE INTO bible_verses (id, book_id, chapter, verse) VALUES (8001001, 8, 1, 1);
             INSERT INTO bible_verse_texts (translation_id, verse_id, text) VALUES ('tr-1', 8001001, 'Invented words.');"
        ))
        .unwrap();
    }
    let db = Database::open(&path, &backups).unwrap();
    let backup = std::fs::read_dir(&backups)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    assert!(backup
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .starts_with("abiding-word-v18-"));
    db.with_conn(|conn| {
        assert_eq!(
            migrations::current_version(conn).unwrap(),
            MIGRATIONS.last().unwrap().version
        );
        let bible = bibles::get(conn, "tr-1").unwrap();
        assert_eq!((bible.kind.as_str(), bible.verse_count), ("structured", 1));
        assert_eq!(
            bibles::chapter(conn, "tr-1", "RUT", 1).unwrap().verses[0].text,
            "Invented words."
        );
        assert_eq!(count(conn, "SELECT COUNT(*) FROM bible_documents"), 0);
        assert_eq!(count(conn, "SELECT COUNT(*) FROM bible_verse_origins"), 0);
        assert!(kept_documents::list(conn, "tr-1").unwrap().is_empty());
        Ok(())
    })
    .unwrap();
}

// --- Independent review (v0.23): words wrapped onto their own line, weak
// numbers, numbers that look like references, and chapters cut short.

/// Ruth as lines, with verse 1:3 wrapped so its last word stands alone.
fn ruth_with_wrapped_word(word: &str) -> String {
    ruth_lines(|c| format!("Chapter {c}")).replacen(
        &format!("3 {}\n", words("Ruth", 1, 3)),
        &format!("3 {}\n{word}\n", words("Ruth", 1, 3)),
        1,
    )
}

#[test]
fn a_book_name_wrapped_onto_its_own_line_stays_in_the_verse() {
    for word in ["Ruth.", "Ruth", "song.", "Acts"] {
        let r = recognise(&read("ruth.txt", ruth_with_wrapped_word(word).as_bytes()));
        assert_eq!(
            verse(&r, "RUT", 1, 3),
            Some(format!("{} {word}", words("Ruth", 1, 3))),
            "{word}"
        );
        assert_eq!(
            verse(&r, "RUT", 1, 22),
            Some(words("Ruth", 1, 22)),
            "{word}"
        );
        assert_eq!(r.verse_count(), 85, "{word}");
        assert!(errors(&r).is_empty(), "{word}: {:?}", errors(&r));
    }
}

#[test]
fn two_equally_weak_places_for_one_verse_refuse_the_chapter() {
    let text = ruth_lines(|c| format!("Chapter {c}")).replacen(
        &format!("4 {}\n5 {}\n", words("Ruth", 1, 4), words("Ruth", 1, 5)),
        "4 and he gave them 5 loaves 5 and so on\n",
        1,
    );
    let r = recognise(&read("ruth.txt", text.as_bytes()));
    assert!(
        codes(&r).contains(&"bibleImport.documentAmbiguous".to_string()),
        "{:?}",
        codes(&r)
    );
    assert!(!r.usable());
}

#[test]
fn a_large_number_starting_a_line_is_not_a_chapter_and_verse() {
    let text = ruth_lines(|c| format!("Chapter {c}")).replacen(
        &format!("3 {}\n", words("Ruth", 1, 3)),
        "3 And they counted\n2,300 days and nights.\n",
        1,
    );
    let r = recognise(&read("ruth.txt", text.as_bytes()));
    assert_eq!(
        verse(&r, "RUT", 1, 3).as_deref(),
        Some("And they counted 2,300 days and nights.")
    );
    assert!(errors(&r).is_empty(), "{:?}", errors(&r));
}

#[test]
fn the_gospel_according_to_saint_mark_names_mark() {
    let mut text = String::from("The Gospel According to Saint Mark\n\nChapter 1\n\n");
    for v in 1..=45u16 {
        text.push_str(&format!("{v} {}\n", words("Mark", 1, v)));
    }
    let r = recognise(&read("mark.txt", text.as_bytes()));
    assert_eq!(r.books[0].book, Some("MRK"));
    assert_eq!(verse(&r, "MRK", 1, 45), Some(words("Mark", 1, 45)));
}

#[test]
fn a_chapter_cut_short_is_refused_and_a_few_verses_short_is_a_warning() {
    // Ruth 1 stops after verse 3 (22 expected): refused.
    let mut text = String::from("Ruth\n\nChapter 1\n\n");
    for v in 1..=3u16 {
        text.push_str(&format!("{v} {}\n", words("Ruth", 1, v)));
    }
    let r = recognise(&read("ruth.txt", text.as_bytes()));
    assert!(codes(&r).contains(&"bibleImport.documentChapterShort".to_string()));
    assert!(!r.usable());
    // Ruth 1 with 19 of its 22 verses: another numbering, perhaps: a warning.
    let mut text = String::from("Ruth\n\nChapter 1\n\n");
    for v in 1..=19u16 {
        text.push_str(&format!("{v} {}\n", words("Ruth", 1, v)));
    }
    let r = recognise(&read("ruth.txt", text.as_bytes()));
    assert!(codes(&r).contains(&"bibleImport.documentChapterShort".to_string()));
    assert!(errors(&r).is_empty(), "{:?}", errors(&r));
}

// --- v0.24: document Bibles made navigable ------------------------------------------

/// Ruth with its chapter 2 doubtful (verse 5 could begin in two places).
fn ruth_with_a_doubtful_chapter() -> String {
    ruth_lines(|c| format!("Chapter {c}")).replacen(
        &format!("4 {}\n5 {}\n", words("Ruth", 2, 4), words("Ruth", 2, 5)),
        "4 and he gave them 5 loaves 5 and so on\n",
        1,
    )
}

fn as_document(conn: &Connection, temp: &Path, text: &str, short: &str) -> (String, StagedImport) {
    let path = write(temp, &format!("{short}.txt"), text.as_bytes());
    let staged = import::stage(&[path], ImportOrigin::UserFiles).unwrap();
    let id = add(conn, &staged, short, true).unwrap();
    (id, staged)
}

fn passage(conn: &Connection, id: &str, reference: &str) -> kept_documents::DocumentPassage {
    let reference =
        abiding_word_lib::scripture::reference::ScriptureRef::from_canonical(reference).unwrap();
    kept_documents::locate(conn, id, &reference).unwrap()
}

#[test]
fn the_chapters_recognised_without_doubt_can_be_used_while_the_rest_stays_unverified() {
    let temp = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let (id, staged) = as_document(conn, temp.path(), &ruth_with_a_doubtful_chapter(), "PART");
        let preview = staged.preview(Default::default()).document.unwrap();
        assert!(!preview.verses_usable);
        assert_eq!(preview.partial_chapters, 3);
        assert_eq!(preview.partial_verses, 22 + 18 + 22);
        assert_eq!(preview.held_chapters, 1);
        // Refused without asking for the partial use.
        assert!(kept_documents::add_verses(
            conn,
            &id,
            &staged.content,
            staged.documents.as_ref().unwrap(),
            false
        )
        .is_err());
        let bible = kept_documents::add_verses(
            conn,
            &id,
            &staged.content,
            staged.documents.as_ref().unwrap(),
            true,
        )?;
        assert_eq!((bible.kind.as_str(), bible.verse_count), ("document", 62));
        // A verified passage reads as written, and says where it is in the document.
        let found = passage(conn, &id, "RUT.1.3-RUT.1.5");
        assert!(found.complete);
        assert_eq!(found.verses[0].text, words("Ruth", 1, 3));
        assert!(found.origin.as_ref().unwrap().first_segment.is_some());
        // The doubtful chapter has no text at all: never something unrelated.
        let doubtful = passage(conn, &id, "RUT.2");
        assert!(!doubtful.complete);
        assert!(doubtful.verses.is_empty());
        let coverage = kept_documents::coverage(conn, &id)?;
        assert_eq!(coverage.verses, 62);
        assert_eq!(coverage.books[0].chapters, 3);
        assert_eq!(coverage.books[0].of, 4);
        Ok(())
    })
    .unwrap();
}

#[test]
fn every_verse_kept_is_exactly_the_documents_own_text_and_traces_back_to_it() {
    let temp = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let (id, staged) = as_document(conn, temp.path(), &ruth_lines(|c| format!("Chapter {c}")), "EXACT");
        kept_documents::add_verses(conn, &id, &staged.content, staged.documents.as_ref().unwrap(), false)?;
        let normalise = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
        let document = kept_documents::list(conn, &id)?.remove(0);
        let segments = kept_documents::segments(conn, &document.id, 0, 10_000, None)?.segments;
        let mut stmt = conn.prepare(
            "SELECT t.text, o.first_segment, o.last_segment FROM bible_verse_texts t
             JOIN bible_verse_origins o ON o.translation_id = t.translation_id AND o.verse_id = t.verse_id
             WHERE t.translation_id = ?1",
        )?;
        let rows: Vec<(String, i64, i64)> = stmt
            .query_map([&id], |r| Ok((r.get(0)?, r.get(1)?, r.get::<_, Option<i64>>(2)?.unwrap_or(r.get(1)?))))?
            .collect::<Result<_, _>>()?;
        assert_eq!(rows.len(), 85);
        for (text, first, last) in rows {
            let original: String = segments
                .iter()
                .filter(|s| s.position >= first && s.position <= last)
                .map(|s| s.text.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            assert!(normalise(&original).contains(&normalise(&text)), "{text:?} not in {original:?}");
        }
        Ok(())
    })
    .unwrap();
}

#[test]
fn a_place_marked_by_hand_is_found_by_passage_and_changes_nothing_else() {
    let temp = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        // A scanned PDF Bible: no text, only pages.
        let bytes = fixtures::pdf(&[&[], &[], &[]], Some("Scanned"), None);
        let path = write(temp.path(), "scanned.pdf", &bytes);
        let staged = import::stage(&[path], ImportOrigin::UserFiles).unwrap();
        let id = add(conn, &staged, "SCAN", true)?;
        let other = as_document(
            conn,
            temp.path(),
            &ruth_lines(|c| format!("Chapter {c}")),
            "OTHER",
        )
        .0;
        let document = kept_documents::list(conn, &id)?.remove(0);
        let input = |reference: &str, page: Option<i64>, document_id: &str, translation: &str| {
            kept_documents::AddLocation {
                translation_id: translation.into(),
                reference: reference.into(),
                document_id: document_id.into(),
                page,
                segment: None,
                item: None,
                quote: None,
            }
        };
        let annotations_before = count(conn, "SELECT COUNT(*) FROM annotations");
        let marked = kept_documents::add_location(
            conn,
            &input("LUK.19.1-LUK.19.10", Some(2), &document.id, &id),
        )?;
        assert_eq!(marked.reference, "LUK.19.1-LUK.19.10");
        // Found for any passage it overlaps; not for others.
        let found = passage(conn, &id, "LUK.19.5");
        assert_eq!(found.locations.len(), 1);
        assert_eq!(found.locations[0].page, Some(2));
        assert!(found.verses.is_empty() && !found.complete);
        assert!(passage(conn, &id, "LUK.20.1").locations.is_empty());
        // A page the document lacks, no place, or another Bible's document: refused.
        assert!(
            kept_documents::add_location(conn, &input("LUK.19.1", Some(9), &document.id, &id))
                .is_err()
        );
        assert!(
            kept_documents::add_location(conn, &input("LUK.19.1", None, &document.id, &id))
                .is_err()
        );
        assert!(kept_documents::add_location(
            conn,
            &input("LUK.19.1", Some(1), &document.id, &other)
        )
        .is_err());
        // Removing it changes nothing else.
        kept_documents::remove_location(conn, &marked.id)?;
        assert!(passage(conn, &id, "LUK.19.5").locations.is_empty());
        assert_eq!(
            count(conn, "SELECT COUNT(*) FROM annotations"),
            annotations_before
        );
        Ok(())
    })
    .unwrap();
}

/// Ruth 1 on a two-column page: verses 1–11 in the left column, 12–22 in the
/// right, written column by column or line by line across the columns.
fn two_columns(across: bool) -> Vec<u8> {
    let mut left: Vec<String> = vec!["Ruth".into(), "Chapter 1".into()];
    left.extend((1..=11).map(|v| format!("{v} {}", words("Ruth", 1, v))));
    let right: Vec<String> = (12..=22)
        .map(|v| format!("{v} {}", words("Ruth", 1, v)))
        .collect();
    let mut lines = Vec::new();
    if across {
        for (i, l) in left.iter().enumerate() {
            lines.push((40, 780 - 14 * i as i64, l.clone()));
            if i >= 2 {
                if let Some(r) = right.get(i - 2) {
                    lines.push((310, 780 - 14 * i as i64, r.clone()));
                }
            }
        }
    } else {
        for (i, l) in left.iter().enumerate() {
            lines.push((40, 780 - 14 * i as i64, l.clone()));
        }
        for (i, r) in right.iter().enumerate() {
            lines.push((310, 780 - 14 * (i as i64 + 2), r.clone()));
        }
    }
    fixtures::pdf_placed(&[lines])
}

#[test]
fn a_two_column_pdf_is_read_in_column_order_however_its_lines_are_drawn() {
    // v0.24.1: the reading order comes from where the text stands on the page,
    // not from the order it is drawn in. Drawn column by column, or line by
    // line across both columns: the same verses, each with its own words.
    for across in [false, true] {
        let r = recognise(&read("columns.pdf", &two_columns(across)));
        assert!(r.usable(), "across {across}: {:?}", codes(&r));
        for v in 1..=22 {
            assert_eq!(
                verse(&r, "RUT", 1, v),
                Some(words("Ruth", 1, v)),
                "across {across}, 1:{v}"
            );
        }
    }
}

#[test]
fn a_short_second_column_drawn_across_is_read_in_order() {
    // Left column 1–17, right column 18–22, drawn line by line across.
    let mut left: Vec<String> = vec!["Ruth".into(), "Chapter 1".into()];
    left.extend((1..=17).map(|v| format!("{v} {}", words("Ruth", 1, v))));
    let right: Vec<String> = (18..=22)
        .map(|v| format!("{v} {}", words("Ruth", 1, v)))
        .collect();
    let mut lines = Vec::new();
    for (i, l) in left.iter().enumerate() {
        lines.push((40, 800 - 14 * i as i64, l.clone()));
        if let Some(r) = i.checked_sub(2).and_then(|j| right.get(j)) {
            lines.push((310, 800 - 14 * i as i64, r.clone()));
        }
    }
    let r = recognise(&read("short.pdf", &fixtures::pdf_placed(&[lines])));
    assert!(r.usable(), "{:?}", codes(&r));
    for v in 1..=22 {
        assert_eq!(verse(&r, "RUT", 1, v), Some(words("Ruth", 1, v)), "1:{v}");
    }
}

#[test]
fn text_read_across_two_columns_without_positions_is_still_refused() {
    // A text file has no positions: two columns run into each other line by
    // line can only be detected, and the chapter is refused, never guessed.
    // Each line holds a line of the left column, then one of the right.
    let mut text = String::from("Ruth\n\nChapter 1\n\n");
    for v in 1..=11u16 {
        text.push_str(&format!(
            "{v} {}     {} {}\n",
            words("Ruth", 1, v),
            v + 11,
            words("Ruth", 1, v + 11)
        ));
    }
    let r = recognise(&read("across.txt", text.as_bytes()));
    assert!(!r.usable());
    assert!(
        codes(&r).contains(&"bibleImport.documentSequence".to_string()),
        "{:?}",
        codes(&r)
    );
}

#[test]
fn a_page_whose_reading_order_cannot_be_established_is_not_used() {
    // Two columns, and a line in the middle running across the gutter with
    // column text beside it: the order of that page is uncertain, so its
    // chapter is refused (and reported), never read in a guessed order.
    let mut lines: Vec<(f64, f64, f64, String)> = vec![
        (40.0, 800.0, 9.0, "Ruth".into()),
        (40.0, 786.0, 9.0, "Chapter 1".into()),
    ];
    for v in 1..=11u16 {
        lines.push((
            40.0,
            772.0 - 14.0 * f64::from(v - 1),
            9.0,
            format!("{v} {}", words("Ruth", 1, v)),
        ));
        lines.push((
            310.0,
            772.0 - 14.0 * f64::from(v - 1),
            9.0,
            format!("{} {}", v + 11, words("Ruth", 1, v + 11)),
        ));
    }
    // On the same height as the fifth line of each column.
    lines.push((
        200.0,
        716.0,
        9.0,
        "a line that runs over the gutter between the columns".into(),
    ));
    let r = recognise(&read("uncertain.pdf", &fixtures::pdf_sized(&[lines])));
    assert!(!r.usable());
    assert!(
        codes(&r).contains(&"bibleImport.documentReadingOrder".to_string()),
        "{:?}",
        codes(&r)
    );
    assert!(r
        .content
        .usable_chapters()
        .is_some_and(|u| !u.contains(&(8, 1))));
}

#[test]
fn notes_at_the_foot_of_a_column_are_not_read_between_the_columns() {
    // A note in smaller type at the foot of the left column, below all the
    // running text: the right column follows the left directly, and the note
    // is left out (its sign taken out of the verse that refers to it).
    let mut lines: Vec<(f64, f64, f64, String)> = vec![
        (40.0, 800.0, 10.0, "Ruth".into()),
        (40.0, 786.0, 10.0, "Chapter 1".into()),
    ];
    for v in 1..=11u16 {
        let text = if v == 3 {
            format!("{v} Invented words of Ruth 1:3* for testing.")
        } else {
            format!("{v} {}", words("Ruth", 1, v))
        };
        lines.push((40.0, 772.0 - 14.0 * f64::from(v - 1), 10.0, text));
        lines.push((
            310.0,
            772.0 - 14.0 * f64::from(v - 1),
            10.0,
            format!("{} {}", v + 11, words("Ruth", 1, v + 11)),
        ));
    }
    lines.push((40.0, 560.0, 7.0, "* 1:3 A note about this verse.".into()));
    let r = recognise(&read("notes.pdf", &fixtures::pdf_sized(&[lines])));
    assert!(r.usable(), "{:?}", codes(&r));
    for v in 1..=22 {
        assert_eq!(verse(&r, "RUT", 1, v), Some(words("Ruth", 1, v)), "1:{v}");
    }
}

#[test]
fn a_sign_with_no_note_on_its_page_stays_in_the_text() {
    // "*" is taken out of a verse only when a note on the page begins with
    // it: here no note does, so it is text, kept as written.
    let mut lines: Vec<(f64, f64, f64, String)> = vec![
        (40.0, 800.0, 10.0, "Ruth".into()),
        (40.0, 786.0, 10.0, "Chapter 1".into()),
    ];
    for v in 1..=22u16 {
        let text = if v == 3 {
            format!("{v} Invented words * of Ruth 1:3 for testing.")
        } else {
            format!("{v} {}", words("Ruth", 1, v))
        };
        lines.push((40.0, 772.0 - 14.0 * f64::from(v - 1), 10.0, text));
    }
    let r = recognise(&read("sign.pdf", &fixtures::pdf_sized(&[lines])));
    assert_eq!(
        verse(&r, "RUT", 1, 3).as_deref(),
        Some("Invented words * of Ruth 1:3 for testing.")
    );
}

#[test]
fn an_editions_own_numbering_is_kept_as_written_and_reported() {
    // Malachi as Hebrew Bibles number it: chapter 3 has 24 verses, no chapter 4.
    let mut text = String::from("Malachi\n\n");
    for (chapter, count) in [(1u16, 14u16), (2, 17), (3, 24)] {
        text.push_str(&format!("Chapter {chapter}\n\n"));
        for v in 1..=count {
            text.push_str(&format!("{v} {}\n", words("Malachi", chapter, v)));
        }
    }
    let r = recognise(&read("malachi.txt", text.as_bytes()));
    // 3:19–24 stay 3:19–24: never renumbered to 4:1–6.
    assert_eq!(verse(&r, "MAL", 3, 24), Some(words("Malachi", 3, 24)));
    assert_eq!(verse(&r, "MAL", 4, 1), None);
    assert!(!r.content.issues.is_empty(), "the difference is reported");
}

// --- v0.24.1: patterns found on real typeset Bibles --------------------------------

/// A one-page PDF: `title`, then Ruth 1 in one column (10 pt), with `edit`
/// changing a verse's line and `extra` lines added (x, y, size, text).
fn ruth_page(edit: impl Fn(u16, String) -> String, extra: &[(f64, f64, f64, &str)]) -> Vec<u8> {
    let mut lines: Vec<(f64, f64, f64, String)> = vec![
        (40.0, 800.0, 10.0, "Ruth".into()),
        (40.0, 786.0, 10.0, "Chapter 1".into()),
    ];
    for v in 1..=22u16 {
        lines.push((
            40.0,
            772.0 - 14.0 * f64::from(v - 1),
            10.0,
            edit(v, format!("{v} {}", words("Ruth", 1, v))),
        ));
    }
    lines.extend(extra.iter().map(|(x, y, s, t)| (*x, *y, *s, t.to_string())));
    fixtures::pdf_sized(&[lines])
}

#[test]
fn mark_9_44_a_cluster_of_note_signs_is_taken_out_when_the_notes_name_the_verse() {
    // WEB Mark 9:44 ends "…not quenched.’ *‡", and the page's notes begin
    // "* 9:44 Isaiah 66:24" and "‡ 9:44 NU omits verse 44.": both signs are
    // references there, so both are taken out; nothing else changes.
    let pdf = ruth_page(
        |v, line| {
            if v == 3 {
                "3 the fire is not quenched.’ *‡".to_string()
            } else {
                line
            }
        },
        &[
            (40.0, 420.0, 7.0, "* 1:3 Isaiah 66:24"),
            (40.0, 410.0, 7.0, "‡ 1:3 NU omits verse 44."),
        ],
    );
    let r = recognise(&read("signs.pdf", &pdf));
    assert!(r.usable(), "{:?}", codes(&r));
    assert_eq!(
        verse(&r, "RUT", 1, 3).as_deref(),
        Some("the fire is not quenched.’")
    );
    assert_eq!(verse(&r, "RUT", 1, 4), Some(words("Ruth", 1, 4)));
}

#[test]
fn a_note_sign_is_taken_out_only_where_a_note_names_its_verse() {
    // "†" with a note naming verse 3 goes from verse 3; the same sign in verse
    // 5, whose verse no note names, stays as written.
    let pdf = ruth_page(
        |v, line| match v {
            3 => "3 Invented words of Ruth† 1:3 for testing.".into(),
            5 => "5 Invented words of Ruth† 1:5 for testing.".into(),
            _ => line,
        },
        &[(40.0, 420.0, 7.0, "† 1:3 A note on verse three.")],
    );
    let r = recognise(&read("one-note.pdf", &pdf));
    assert_eq!(
        verse(&r, "RUT", 1, 3).as_deref(),
        Some("Invented words of Ruth 1:3 for testing.")
    );
    assert_eq!(
        verse(&r, "RUT", 1, 5).as_deref(),
        Some("Invented words of Ruth† 1:5 for testing.")
    );
}

#[test]
fn a_large_heading_between_verses_is_left_out() {
    // "BOOK 2" set larger than the text, between two verses: a heading.
    let mut lines: Vec<(f64, f64, f64, String)> = vec![
        (40.0, 800.0, 10.0, "Ruth".into()),
        (40.0, 786.0, 10.0, "Chapter 1".into()),
    ];
    let mut y = 772.0;
    for v in 1..=22u16 {
        if v == 12 {
            lines.push((120.0, y, 16.0, "BOOK 2".into()));
            y -= 22.0;
        }
        lines.push((40.0, y, 10.0, format!("{v} {}", words("Ruth", 1, v))));
        y -= 14.0;
    }
    let r = recognise(&read("heading.pdf", &fixtures::pdf_sized(&[lines])));
    assert!(r.usable(), "{:?}", codes(&r));
    assert_eq!(verse(&r, "RUT", 1, 11), Some(words("Ruth", 1, 11)));
}

#[test]
fn a_repeated_speaker_label_inside_a_verse_refuses_the_chapter() {
    // "Beloved" stands as a line of its own three times, once between two
    // sentences of a verse: perhaps a speaker's name, perhaps words of the
    // verse — not known without its style. The chapter is not used.
    let mut text = String::from("Ruth\n\nChapter 1\n\n");
    for v in 1..=22u16 {
        if v == 4 {
            text.push_str("4 Invented words of Ruth 1:4.\nBeloved\nMore invented words.\n");
            continue;
        }
        text.push_str(&format!("{v} {}\n", words("Ruth", 1, v)));
    }
    text.push_str("\nChapter 2\n\nBeloved\n1 Invented.\nBeloved\n");
    let r = recognise(&read("labels.txt", text.as_bytes()));
    assert!(
        codes(&r).contains(&"bibleImport.documentLabel".to_string()),
        "{:?}",
        codes(&r)
    );
    assert!(r
        .content
        .usable_chapters()
        .is_some_and(|u| !u.contains(&(8, 1))));
}

#[test]
fn a_contents_page_with_dot_leaders_is_not_read_as_headings() {
    // "Ruth . . . . . 5" in a table of contents is not "Ruth, chapter 5"; the
    // book is then found once, where it begins.
    let text = format!(
        "Contents\n\nGenesis . . . . . . . 1\nRuth . . . . . . . 5\n\n{}",
        ruth_lines(|c| format!("Chapter {c}"))
    );
    let r = recognise(&read("contents.txt", text.as_bytes()));
    assert!(r.usable(), "{:?}", codes(&r));
    assert_eq!(verse(&r, "RUT", 1, 1), Some(words("Ruth", 1, 1)));
}

#[test]
fn a_chapter_number_alone_on_its_line_starts_that_chapter_even_after_a_short_chapter() {
    // Revelation 12 in Luther 1912 has 17 verses where the standard count has
    // 18: the printed "13" with verse 1 next still begins chapter 13, so the
    // chapters after it are not shifted; the missing verse is reported.
    let mut text = String::from("Ruth\n\n1\n\n");
    for v in 1..=21u16 {
        text.push_str(&format!("{v} {}\n", words("Ruth", 1, v)));
    }
    text.push_str("\n2\n\n");
    for v in 1..=23u16 {
        text.push_str(&format!("{v} {}\n", words("Ruth", 2, v)));
    }
    let r = recognise(&read("short.txt", text.as_bytes()));
    assert_eq!(verse(&r, "RUT", 2, 1), Some(words("Ruth", 2, 1)));
    assert_eq!(verse(&r, "RUT", 2, 23), Some(words("Ruth", 2, 23)));
    assert_eq!(verse(&r, "RUT", 1, 22), None, "never filled");
    // A chapter whose number was lost: the next printed number puts the count right.
    let mut text = String::from("Ruth\n\n1\n\n");
    for v in 1..=22u16 {
        text.push_str(&format!("{v} {}\n", words("Ruth", 1, v)));
    }
    text.push_str("\n3\n\n");
    for v in 1..=18u16 {
        text.push_str(&format!("{v} {}\n", words("Ruth", 3, v)));
    }
    let r = recognise(&read("jump.txt", text.as_bytes()));
    assert_eq!(verse(&r, "RUT", 3, 1), Some(words("Ruth", 3, 1)));
    assert_eq!(verse(&r, "RUT", 2, 1), None);
    assert!(
        codes(&r).contains(&"bibleImport.documentChapterJump".to_string()),
        "{:?}",
        codes(&r)
    );
    // The jumped-to chapter is refused: the number may not be its number.
    let ruth = canon::by_code("RUT").unwrap().ordinal;
    let usable = r.content.usable_chapters().unwrap();
    assert!(!usable.contains(&(ruth, 3)), "{usable:?}");
    assert!(usable.contains(&(ruth, 1)));
}

/// Every verse of a usable chapter is the text it should be: whatever the
/// recogniser cannot read is refused, never accepted wrong.
fn assert_no_wrong_usable_verse(
    r: &Recognition,
    code: &str,
    expected: impl Fn(u16, u16) -> String,
) {
    let book = canon::by_code(code).unwrap();
    let Some(usable) = r.content.usable_chapters() else {
        return;
    };
    for &(ordinal, chapter) in &usable {
        assert_eq!(ordinal, book.ordinal, "{usable:?}");
        let verses = &r.content.books[&ordinal].chapters[&chapter].verses;
        for (&v, text) in verses {
            assert_eq!(
                text,
                &expected(chapter, v),
                "{code} {chapter}:{v} accepted wrong"
            );
        }
    }
}

#[test]
fn a_stray_number_on_its_own_line_never_renumbers_a_chapter_silently() {
    // A page number "3" standing alone after Ruth 1:22, then Ruth 2's verses.
    let mut text = String::from("Ruth\n\n1\n\n");
    for v in 1..=22u16 {
        text.push_str(&format!("{v} {}\n", words("Ruth", 1, v)));
    }
    text.push_str("3\n");
    for v in 1..=23u16 {
        text.push_str(&format!("{v} {}\n", words("Ruth", 2, v)));
    }
    let r = recognise(&read("stray.txt", text.as_bytes()));
    // Ruth 2's words must never be usable as Ruth 3.
    let ruth = canon::by_code("RUT").unwrap().ordinal;
    let usable = r.content.usable_chapters().unwrap_or_default();
    assert!(!usable.contains(&(ruth, 3)), "{usable:?}");
    assert_no_wrong_usable_verse(&r, "RUT", |c, v| words("Ruth", c, v));
}

#[test]
fn a_word_broken_at_a_line_end_before_or_and_nor_is_joined() {
    let text = ruth_lines(|c| format!("Chapter {c}")).replace(
        "\n3 Invented words of Ruth 1:3 for testing.\n",
        "\n3 Give glory and hon-\nor to the gover-\nnor of the land.\n",
    );
    let r = recognise(&read("hyphen.txt", text.as_bytes()));
    assert_eq!(
        verse(&r, "RUT", 1, 3).as_deref(),
        Some("Give glory and honor to the governor of the land.")
    );
}

#[test]
fn a_section_heading_naming_a_book_word_is_not_a_book() {
    // Judges 5 begins with "The Song of Deborah": never the Song of Songs.
    let mut text = String::from("Judges\n\n");
    for (chapter, count) in [(1u16, 36u16), (2, 23), (3, 31), (4, 24), (5, 31), (6, 40)] {
        if chapter == 5 {
            text.push_str("The Song of Deborah\n\n");
        }
        text.push_str(&format!("Chapter {chapter}\n\n"));
        for v in 1..=count {
            text.push_str(&format!("{v} {}\n", words("Judges", chapter, v)));
        }
    }
    let r = recognise(&read("judges.txt", text.as_bytes()));
    let song = canon::by_code("SNG").unwrap().ordinal;
    assert!(!r.content.books.contains_key(&song));
    assert_no_wrong_usable_verse(&r, "JDG", |c, v| words("Judges", c, v));
    assert_eq!(verse(&r, "JDG", 5, 1), Some(words("Judges", 5, 1)));
}

/// Ruth on PDF pages, each chapter's number printed at `size` at the start
/// of its first line (chapter 1's as a large initial).
fn ruth_inline_numbers(size: f64) -> Vec<u8> {
    let mut pages: Vec<Vec<(f64, f64, f64, String)>> = Vec::new();
    let mut lines: Vec<(f64, f64, f64, String)> = vec![(40.0, 800.0, 16.0, "Ruth".into())];
    let mut y = 770.0;
    for (chapter, count) in [(1u16, 22u16), (2, 23), (3, 18), (4, 22)] {
        for v in 1..=count {
            if y < 80.0 {
                pages.push(std::mem::take(&mut lines));
                y = 800.0;
            }
            if v == 1 {
                let (at, sz) = if chapter == 1 {
                    (y - 10.0, 28.0)
                } else {
                    (y, size)
                };
                lines.push((40.0, at, sz, format!("{chapter}")));
                lines.push((
                    54.0,
                    y,
                    10.0,
                    format!("Invented first line of Ruth {chapter}:1"),
                ));
                y -= 14.0;
                lines.push((40.0, y, 10.0, "and its second line here.".into()));
            } else {
                lines.push((40.0, y, 10.0, format!("{v} {}", words("Ruth", chapter, v))));
            }
            y -= 14.0;
        }
    }
    pages.push(lines);
    fixtures::pdf_sized(&pages)
}

#[test]
fn a_chapter_number_set_a_little_larger_in_its_line_does_not_make_it_a_heading() {
    for size in [12.0, 14.0] {
        let r = recognise(&read("inline.pdf", &ruth_inline_numbers(size)));
        assert_no_wrong_usable_verse(&r, "RUT", |c, v| {
            if v == 1 {
                format!("Invented first line of Ruth {c}:1 and its second line here.")
            } else {
                words("Ruth", c, v)
            }
        });
        assert_eq!(
            verse(&r, "RUT", 2, 1).as_deref(),
            Some("Invented first line of Ruth 2:1 and its second line here."),
            "size {size}"
        );
    }
}

#[test]
fn a_page_dense_with_smaller_notes_is_not_read_as_scripture() {
    let mut p1: Vec<(f64, f64, f64, String)> = vec![
        (40.0, 800.0, 10.0, "Ruth".into()),
        (40.0, 786.0, 10.0, "Chapter 1".into()),
    ];
    let mut y = 772.0;
    for v in 1..=11u16 {
        p1.push((40.0, y, 10.0, format!("{v} {}", words("Ruth", 1, v))));
        y -= 14.0;
    }
    let mut p2: Vec<(f64, f64, f64, String)> = Vec::new();
    let mut y = 800.0;
    for v in 12..=22u16 {
        p2.push((40.0, y, 10.0, format!("{v} {}", words("Ruth", 1, v))));
        y -= 14.0;
    }
    y -= 20.0;
    for i in 0..40 {
        p2.push((
            40.0,
            y,
            8.0,
            format!("note {i} explaining at length the meaning of a word in it"),
        ));
        y -= 10.0;
    }
    let mut p3: Vec<(f64, f64, f64, String)> = vec![(40.0, 800.0, 10.0, "Chapter 2".into())];
    let mut y = 786.0;
    for v in 1..=23u16 {
        p3.push((40.0, y, 10.0, format!("{v} {}", words("Ruth", 2, v))));
        y -= 14.0;
    }
    let r = recognise(&read("heavy.pdf", &fixtures::pdf_sized(&[p1, p2, p3])));
    assert_no_wrong_usable_verse(&r, "RUT", |c, v| words("Ruth", c, v));
}

#[test]
fn a_suspended_hyphen_at_a_line_end_is_kept() {
    // "Lob-" at the end of a line, "und Dankopfer" on the next: the hyphen is
    // the word's own (Luther 1912, Leviticus 7:13); "Dankop-" / "fer" is a
    // word broken at the line's end.
    let text = ruth_lines(|c| format!("Chapter {c}"))
        .replace(
            "\n3 Invented words of Ruth 1:3 for testing.\n",
            "\n3 mit ihrem Lob-\nund Dankopfer.\n",
        )
        .replace(
            "\n4 Invented words of Ruth 1:4 for testing.\n",
            "\n4 ein Dankop-\nfer bringen.\n",
        );
    let r = recognise(&read("hyphen.txt", text.as_bytes()));
    assert_eq!(
        verse(&r, "RUT", 1, 3).as_deref(),
        Some("mit ihrem Lob- und Dankopfer.")
    );
    assert_eq!(
        verse(&r, "RUT", 1, 4).as_deref(),
        Some("ein Dankopfer bringen.")
    );
}

#[test]
fn a_title_over_two_lines_with_a_possessive_names_its_book() {
    // "Paul’s Second Letter to" / "the Corinthians" (WEB), then chapter 1.
    let mut lines: Vec<(f64, f64, f64, String)> = vec![
        (60.0, 800.0, 16.0, "Paul’s Second Letter to".into()),
        (80.0, 782.0, 16.0, "the Corinthians".into()),
    ];
    for v in 1..=24u16 {
        lines.push((
            40.0,
            760.0 - 14.0 * f64::from(v - 1),
            10.0,
            format!("{v} {}", words("2 Corinthians", 1, v)),
        ));
    }
    let r = recognise(&read("2co.pdf", &fixtures::pdf_sized(&[lines])));
    assert_eq!(
        verse(&r, "2CO", 1, 1),
        Some(words("2 Corinthians", 1, 1)),
        "{:?}",
        codes(&r)
    );
}

#[test]
fn raised_letters_are_taken_out_only_where_the_notes_explain_them() {
    // Louis Segond 1910 sets cross-reference letters small and raised against
    // the next word ("ᶜEt la contrée"). With a note "c 1:3 …" on the page the
    // letter goes; without one, the chapter is not used (never kept as a word
    // of the verse, never dropped on a guess).
    let with_note = |note: bool| {
        let mut lines: Vec<(f64, f64, f64, String)> = vec![
            (40.0, 800.0, 10.0, "Ruth".into()),
            (40.0, 786.0, 10.0, "Chapter 1".into()),
        ];
        for v in 1..=22u16 {
            let y = 772.0 - 14.0 * f64::from(v - 1);
            if v == 3 {
                lines.push((40.0, y, 10.0, "3 Invented words".into()));
                lines.push((126.0, y + 3.5, 6.0, "c".into()));
                lines.push((130.0, y, 10.0, "Et of Ruth 1:3 for testing.".into()));
            } else {
                lines.push((40.0, y, 10.0, format!("{v} {}", words("Ruth", 1, v))));
            }
        }
        if note {
            lines.push((40.0, 420.0, 7.0, "c 1:3 Ge 1:1.".into()));
        }
        fixtures::pdf_sized(&[lines])
    };
    let r = recognise(&read("letters.pdf", &with_note(true)));
    assert!(r.usable(), "{:?}", codes(&r));
    assert_eq!(
        verse(&r, "RUT", 1, 3).as_deref(),
        Some("Invented words Et of Ruth 1:3 for testing.")
    );
    let r = recognise(&read("letters.pdf", &with_note(false)));
    assert!(
        codes(&r).contains(&"bibleImport.documentRaisedLetters".to_string()),
        "{:?}",
        codes(&r)
    );
    assert!(r
        .content
        .usable_chapters()
        .is_some_and(|u| !u.contains(&(8, 1))));
}

#[test]
fn a_preface_line_naming_a_passage_is_not_a_bible_line() {
    // "Luke 17:36 and Acts 15:34, which are …" in a preface (WEB) is prose:
    // only a document written in such lines has its verses named that way.
    let text = format!(
        "Preface\n\nLuke 17:36 and Acts 15:34, which are not in some manuscripts.\n\n{}",
        ruth_lines(|c| format!("Chapter {c}"))
    );
    let r = recognise(&read("preface.txt", text.as_bytes()));
    assert!(r.usable(), "{:?}", codes(&r));
    assert!(r.books.iter().all(|b| b.book != Some("LUK")));
}

#[test]
fn a_psalm_number_alone_on_its_line_is_not_verse_text() {
    // "Psalms", then "1" alone, then "1 Blessed is the man …".
    let text = "Psalms\n\n1\n\n1 Invented words of Psalms 1:1 for testing.\n2 Invented words of Psalms 1:2 for testing.\n3 Invented words of Psalms 1:3 for testing.\n4 Invented words of Psalms 1:4 for testing.\n5 Invented words of Psalms 1:5 for testing.\n6 Invented words of Psalms 1:6 for testing.\n";
    let r = recognise(&read("psalm.txt", text.as_bytes()));
    assert_eq!(
        verse(&r, "PSA", 1, 1),
        Some(words("Psalms", 1, 1)),
        "{:?}",
        codes(&r)
    );
}
