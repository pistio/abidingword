//! Phase A1.3: safeguards for the non-EPUB readings, from the 20 verses with
//! wrong text and 2 verses the edition does not have that the disclosed PDF
//! and DOCX documents were verified with (`docs/audit/a1.3/cases.md`).
//!
//! Each safeguard only withholds verification, on evidence the document
//! itself shows; none removes, adds or rewrites a word. Every text here is
//! invented; books and verse counts are real.

#[path = "support/library_fixtures.rs"]
mod fixtures;

use abiding_word_lib::scripture::canon;
use abiding_word_lib::scripture::documents::{self, Place, Unit};
use abiding_word_lib::scripture::structure::{self, Corrections, Recognition};

/// A verse's invented words: no digit and no book's name in them (real
/// verse text has neither, and other rules weigh both).
fn words(_book: &str, chapter: u16, verse: u16) -> String {
    const NAMES: [&str; 10] = [
        "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
    ];
    let letters = |n: u16| -> String {
        n.to_string()
            .chars()
            .map(|c| NAMES[c.to_digit(10).unwrap() as usize])
            .collect::<Vec<_>>()
            .join("-")
    };
    format!(
        "Invented words of chapter {} verse {} for testing.",
        letters(chapter),
        letters(verse)
    )
}

fn line(text: &str, page: u32) -> Unit {
    Unit {
        text: text.to_string(),
        heading: false,
        place: Place {
            document: 0,
            page: Some(page),
            segment: Some(page),
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

fn recognise(units: &[Unit]) -> Recognition {
    structure::recognise(units, Some("en"), &Corrections::default())
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

/// A one-chapter book as a PDF prints it: its name, then a line per verse
/// (`raised`: the verse numbers as superscripts), page by page.
fn book(name: &str, code: &str, first_page: u32, raised: bool) -> Vec<Unit> {
    let count = canon::verse_count(canon::by_code(code).unwrap(), 1).unwrap();
    let mut units = vec![line(name, first_page)];
    for v in 1..=count {
        let number = if raised {
            superscript(v)
        } else {
            v.to_string()
        };
        units.push(line(
            &format!("{number} {}", words(name, 1, v)),
            first_page + u32::from(v) / 10,
        ));
    }
    units
}

fn superscript(n: u16) -> String {
    n.to_string()
        .chars()
        .map(|c| {
            ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'][c.to_digit(10).unwrap() as usize]
        })
        .collect()
}

fn at(units: &[Unit], start: &str) -> usize {
    units
        .iter()
        .position(|u| u.text.starts_with(start))
        .unwrap()
}

#[test]
fn a_whole_line_in_another_face_after_a_colon_withdraws_the_verse() {
    // `deutkw`'s eBible PDF: "¹² Damals sprach Salomo:" / "Die Sonne
    // stellte er ans Himmelszelt, …", the line the edition marks a heading
    // (\s1), set in another face.
    let build = |styled: bool| {
        let mut units = book("Philemon", "PHM", 1, false);
        let i = at(&units, "7 ");
        units[i].text = "7 Invented words of chapter one verse seven, and then he spoke:".into();
        let mut set = line("Invented heading words in another face, set as a line.", 1);
        // (In another face: its letters' widths show it, `partly_styled`
        // too; a centred poem's line is `styled` by its place alone.)
        set.styled = styled;
        set.partly_styled = styled;
        units.insert(i + 1, set);
        recognise(&units)
    };
    let r = build(true);
    assert!(!verified(&r, "PHM", 1, 7), "{:?}", verse(&r, "PHM", 1, 7));
    assert!(verified(&r, "PHM", 1, 6) && verified(&r, "PHM", 1, 8));
    // The control: in the body's face the line is the verse's.
    let r = build(false);
    assert!(verified(&r, "PHM", 1, 7));
    // Nor is a line `styled` only by being centred (a poem's line, eBible's
    // PDFs) after a comma put in doubt.
    let mut units = book("Philemon", "PHM", 1, false);
    let i = at(&units, "7 ");
    units[i].text = "7 Invented words of chapter one verse seven, and then he spoke,".into();
    let mut poem = line("and the invented poem goes on in its own centred line,", 1);
    poem.styled = true;
    poem.centred = true;
    units.insert(i + 1, poem);
    let r = recognise(&units);
    assert!(verified(&r, "PHM", 1, 7), "{:?}", r.unverified);
    // Nor words spoken, set in their own face after "said to him,"
    // (eBible's red-letter edition): the line opens a quotation.
    let mut units = book("Philemon", "PHM", 1, false);
    let i = at(&units, "7 ");
    units[i].text = "7 Invented words of chapter one verse seven, and he said to him,".into();
    let mut spoken = line(
        "“Invented words spoken in their own face, set as a line.”",
        1,
    );
    spoken.styled = true;
    spoken.partly_styled = true;
    units.insert(i + 1, spoken);
    let r = recognise(&units);
    assert!(verified(&r, "PHM", 1, 7), "{:?}", r.unverified);
    // Nor words spoken that began in the line before, in its own face
    // there too ("… Jesus answered her, “Woman," / "great is your faith!
    // …" in eBible's red-letter book edition).
    let mut units = book("Philemon", "PHM", 1, false);
    let i = at(&units, "7 ");
    units[i].text = "7 Invented words of chapter one verse seven, he said, “Woman,".into();
    units[i].partly_styled = true;
    let mut spoken = line(
        "invented words spoken go on in their own face, set as a line.”",
        1,
    );
    spoken.styled = true;
    spoken.partly_styled = true;
    units.insert(i + 1, spoken);
    let r = recognise(&units);
    assert!(verified(&r, "PHM", 1, 7), "{:?}", r.unverified);
}

#[test]
fn a_line_perhaps_in_another_face_beginning_a_block_withdraws_the_verse() {
    // `deutkw`'s derived PDF: LibreOffice's bold shares most letters'
    // widths with the regular face, so the line is only `partly_styled`;
    // it begins a block of its own after the verse's sentence ended.
    let build = |new_block: bool| {
        let mut units = book("Philemon", "PHM", 1, false);
        let i = at(&units, "7 ");
        let mut set = line("Invented words the edition calls a heading, here.", 1);
        set.partly_styled = true;
        if new_block {
            set.place.segment = Some(50);
            for u in units.iter_mut().skip(i + 1) {
                u.place.segment = Some(60);
            }
        }
        units.insert(i + 1, set);
        recognise(&units)
    };
    let r = build(true);
    assert!(!verified(&r, "PHM", 1, 7), "{:?}", verse(&r, "PHM", 1, 7));
    // The control: inside the same block (a line with a few words in
    // italics) it does not.
    let r = build(false);
    assert!(verified(&r, "PHM", 1, 7));
}

#[test]
fn a_line_in_smaller_type_after_the_verse_ended_withdraws_it() {
    // `engasv`'s PDF: "For the Chief Musician, on my stringed instruments."
    // after Habakkuk 3:19, set smaller than the verse (the edition's \d).
    let build = |small: bool| {
        let mut units = book("Philemon", "PHM", 1, false);
        let mut set = line("For the invented musician, on invented instruments.", 3);
        set.small = small;
        units.push(set);
        recognise(&units)
    };
    let r = build(true);
    assert!(!verified(&r, "PHM", 1, 25), "{:?}", verse(&r, "PHM", 1, 25));
    let r = build(false);
    assert!(verified(&r, "PHM", 1, 25));
}

#[test]
fn a_line_after_a_book_s_last_verse_read_into_the_next_book_s_heading_withdraws_the_verse() {
    // `porbr2018`'s PDF: Galatians 6:18 "… irmãos." and the line after it,
    // "Amém!Escrita de Roma para os Gálatas", read with "Carta de Paulo aos
    // Efésios" as the next book's heading. The verse's "Amém!" went with it.
    let build = |colophon: bool| {
        let mut units = book("2 John", "2JN", 1, false);
        if colophon {
            let mut set = line("Written by invented hands from Ephesus", 2);
            set.styled = true;
            units.push(set);
        }
        units.extend(book("3 John", "3JN", 2, false));
        recognise(&units)
    };
    let r = build(true);
    assert!(verse(&r, "2JN", 1, 13).is_some());
    assert!(!verified(&r, "2JN", 1, 13), "{:?}", r.unverified);
    assert!(
        verified(&r, "2JN", 1, 12) && verified(&r, "3JN", 1, 1),
        "{:?}",
        r.unverified
    );
    let r = build(false);
    assert!(verified(&r, "2JN", 1, 13));
}

#[test]
fn a_plain_number_inside_a_line_of_a_document_that_raises_its_numbers_is_not_shown_to_be_a_verse() {
    // `spav1602p-lodocx`: "⁷ Reunidos … diciendo de mí: 8 Cosa
    // pestilencial …", every verse number in a style of its own but this
    // one; the edition has no verse 8 there.
    let build = |raised: bool| {
        let mut units = book("Philemon", "PHM", 1, raised);
        units.extend(book("Jude", "JUD", 4, raised));
        units.extend(book("3 John", "3JN", 7, raised));
        let i = at(&units, if raised { "⁷ " } else { "7 " });
        let seven = if raised { "⁷" } else { "7" };
        let eight = if raised { "⁸ " } else { "8 " };
        units[i].text =
            format!("{seven} Invented words of chapter one verse seven, saying: 8 more words.");
        let j = at(&units, eight);
        units.remove(j);
        recognise(&units)
    };
    let r = build(true);
    let key = |v: u16| (canon::by_code("PHM").unwrap().ordinal, 1u16, v);
    assert!(
        !verified(&r, "PHM", 1, 7),
        "{:?} {:?} {:?}",
        verse(&r, "PHM", 1, 7),
        verse(&r, "PHM", 1, 8),
        (r.unverified.get(&key(7)), r.unverified.get(&key(8)))
    );
    assert!(!verified(&r, "PHM", 1, 8), "{:?}", verse(&r, "PHM", 1, 8));
    assert!(verified(&r, "PHM", 1, 9) && verified(&r, "JUD", 1, 7));
    // The control: a document of plain numbers reads the plain 8 as its
    // verse number, as before.
    let r = build(false);
    assert!(verified(&r, "PHM", 1, 8), "{:?}", r.unverified);
}

/// Ruth as a Word document whose verse numbers have a character style of
/// their own, bold and small ("Num", as LibreOffice wrote
/// `spav1602p-lodocx`), a verse to a paragraph; `stray` puts a plain "8"
/// inside 1:7's paragraph, as that document has it after Psalm 41:7's words.
fn ruth_docx(styled_numbers: bool, stray: bool) -> Recognition {
    let number = |v: u16| {
        if styled_numbers {
            format!(r#"<w:r><w:rPr><w:rStyle w:val="Num"/></w:rPr><w:t>{v}</w:t></w:r>"#)
        } else {
            format!(r#"<w:r><w:t xml:space="preserve">{v} </w:t></w:r>"#)
        }
    };
    let heading = |level: u8, text: &str| {
        format!(
            r#"<w:p><w:pPr><w:pStyle w:val="Heading{level}"/></w:pPr><w:r><w:t>{text}</w:t></w:r></w:p>"#
        )
    };
    let mut body = heading(1, "Ruth");
    for (chapter, count) in [(1u16, 22u16), (2, 23), (3, 18), (4, 22)] {
        body.push_str(&heading(2, &format!("Chapter {chapter}")));
        for v in 1..=count {
            if stray && (chapter, v) == (1, 8) {
                continue;
            }
            let text = if stray && (chapter, v) == (1, 7) {
                format!(
                    "{}, saying: 8 more words.",
                    words("Ruth", 1, 7).trim_end_matches('.')
                )
            } else {
                words("Ruth", chapter, v)
            };
            body.push_str(&format!(
                r#"<w:p>{}<w:r><w:t xml:space="preserve"> {text}</w:t></w:r></w:p>"#,
                number(v)
            ));
        }
    }
    let styles = br#"<?xml version="1.0"?><w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/></w:style><w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/></w:style><w:style w:type="character" w:styleId="Num"><w:name w:val="Num"/><w:rPr><w:b/><w:sz w:val="16"/></w:rPr></w:style></w:styles>"#;
    let rels = r#"<Relationship Id="rS" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>"#;
    let bytes = fixtures::docx(&body, rels, &[("word/styles.xml", styles)]);
    let file = documents::read_bytes(
        std::path::Path::new("ruth.docx"),
        "ruth.docx",
        "docx",
        &bytes,
        bytes.len() as u64,
        0,
    )
    .unwrap();
    structure::recognise(&file.units, Some("en"), &Corrections::default())
}

#[test]
fn a_word_document_s_own_verse_number_style_is_the_evidence_for_its_verse_numbers() {
    // The document sets every verse number in "Num" but the stray one: it
    // is not shown to be a verse number, and neither 1:7 nor an 1:8 made
    // from it is verified.
    let r = ruth_docx(true, true);
    assert!(!verified(&r, "RUT", 1, 7), "{:?}", verse(&r, "RUT", 1, 7));
    assert!(!verified(&r, "RUT", 1, 8), "{:?}", verse(&r, "RUT", 1, 8));
    assert!(verified(&r, "RUT", 1, 9) && verified(&r, "RUT", 4, 22));
    // Without the stray number every verse is verified.
    let r = ruth_docx(true, false);
    assert!(verified(&r, "RUT", 1, 7) && verified(&r, "RUT", 1, 8));
    // The control: a document of plain numbers reads its plain 8 as a verse
    // number, as before.
    let r = ruth_docx(false, true);
    assert!(verified(&r, "RUT", 1, 8), "{:?}", r.unverified);
}
