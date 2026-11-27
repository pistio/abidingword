//! Exact source-text regression for the product USFM reader (#59).
//! These examples are invented text, not Bible quotations.

use abiding_word_lib::scripture::usfm;

fn one(body: &str) -> usfm::UsfmDocument {
    usfm::parse(&format!("\\id RUT\n\\c 1\n\\v 1 {body}\n"))
}

#[test]
fn removing_a_note_preserves_the_publishers_adjacent_characters() {
    for (body, expected) in [
        ("heel.\\f + \\ft note\\f*”", "heel.”"),
        ("word\\x - \\xt reference\\x*, next", "word, next"),
        ("joined\\fe + \\ft note\\fe*word", "joinedword"),
        ("word \\ef + \\ft note\\ef* next", "word next"),
        ("word\\f + \\ft note\\f* next", "word next"),
        ("word \\f + \\ft note\\f*next", "word next"),
        ("e\\f + \\ft note\\f*\u{301}", "e\u{301}"),
        ("말씀\\f + \\ft 주석\\f*이다.", "말씀이다."),
        ("שָׁלוֹם\\x - \\xt הערה\\x*־לָךְ", "שָׁלוֹם־לָךְ"),
        ("λόγος\\f + \\ft σημείωση\\f*·", "λόγος·"),
    ] {
        let doc = one(body);
        assert!(
            doc.malformed_markup.is_empty(),
            "{body}: {:?}",
            doc.malformed_markup
        );
        assert_eq!(doc.verses[0].text, expected, "{body}");
    }
}

#[test]
fn milestones_and_nested_character_spans_add_no_words_or_spaces() {
    let doc = one(concat!(
        "a\\qt-s |sid=\"q1\" who=\"speaker\"\\*",
        "\\wj b \\+nd c\\+nd*\\wj*",
        "\\qt-e |eid=\"q1\"\\*d\\ts\\*.",
    ));
    assert!(
        doc.malformed_markup.is_empty(),
        "{:?}",
        doc.malformed_markup
    );
    assert_eq!(doc.verses[0].text, "ab cd.");
}

#[test]
fn acrostic_headings_are_not_appended_to_the_previous_verse() {
    let doc = usfm::parse(
        "\\id PSA\n\\c 119\n\\v 8 Invented verse.\n\\qa BETH\n\\q1\n\\v 9 Next verse.\n",
    );
    assert_eq!(doc.verses[0].text, "Invented verse.");
    assert_eq!(doc.verses[1].text, "Next verse.");
    assert!(doc.malformed_markup.is_empty());
}

#[test]
fn layout_collapses_only_usfm_whitespace_and_keeps_word_offsets() {
    let doc = one("  A~\\w 말|lemma=\"말\"\\w*\u{202f}B\t C//D\n E\u{00a0}F  ");
    assert!(
        doc.malformed_markup.is_empty(),
        "{:?}",
        doc.malformed_markup
    );
    let verse = &doc.verses[0];
    assert_eq!(verse.text, "A\u{00a0}말\u{202f}B CD E\u{00a0}F");
    let word = &verse.words[0];
    assert_eq!(word.surface, "말");
    let utf16: Vec<u16> = verse.text.encode_utf16().collect();
    assert_eq!(
        String::from_utf16(&utf16[word.start..word.end]).unwrap(),
        "말"
    );
}

#[test]
fn malformed_notes_and_alternate_numbers_cannot_swallow_later_verses() {
    for broken in [
        "first\\f + \\ft incomplete",
        "first\\x + \\xt wrong end\\f*",
        "first\\f + \\ft outer\\x + \\xt inner\\x*",
        "first\\va 8",
        "first\\va 8\\vp*",
        "first\\fig description",
        "first\\qt-s |who=\"speaker\"",
        "first\\p\\*",
        "first\\ft detached note part",
    ] {
        let source = format!("\\id RUT\n\\c 1\n\\v 1 {broken}\n\\v 2 second\n\\c 2\n\\v 1 later\n");
        let unchanged = source.clone();
        let doc = usfm::parse(&source);
        assert!(
            !doc.malformed_markup.is_empty(),
            "accepted malformed markup: {broken}"
        );
        assert_eq!(doc.verses.len(), 3, "lost a verse: {broken}");
        assert_eq!(
            (
                &doc.verses[1].chapter,
                &doc.verses[1].label,
                doc.verses[1].text.as_str()
            ),
            (&"1".to_string(), &"2".to_string(), "second")
        );
        assert_eq!(
            (
                &doc.verses[2].chapter,
                &doc.verses[2].label,
                doc.verses[2].text.as_str()
            ),
            (&"2".to_string(), &"1".to_string(), "later")
        );
        assert_eq!(source, unchanged);
    }
}

#[test]
fn malformed_diagnostics_are_bounded() {
    let doc = one(&"\\f + ".repeat(10_000));
    assert!(!doc.malformed_markup.is_empty());
    assert!(doc.malformed_markup.len() <= 32);
}

#[test]
fn control_tainted_titles_are_reported_and_never_exported_as_titles() {
    let doc = usfm::parse(
        "\\id PSA\n\\c 1\n\\d A\u{1} title\u{3}\n\\v 1 words\n\\c 2\n\\d Clean title\n\\v 1 more\n",
    );
    assert_eq!(doc.superscriptions_left_out, 1);
    assert_eq!(
        doc.superscriptions,
        vec![("2".into(), "Clean title".into())]
    );
    assert_eq!(doc.verses[0].text, "words");
    assert!(doc.malformed_markup.is_empty());
    assert!(!one("a\u{1}b").malformed_markup.is_empty());
}
