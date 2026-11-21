// Before-probe: the frozen A1.5 comparison (lines 45-105 and 712-720 of
// 08c6faac:src-tauri/tests/real_document_editions.rs, pasted unchanged) on the
// E2 golden vectors, with the frozen scorer's rule (score-set.py: wrong =
// verified class 2 whose kind is not typography or unicode).
#![allow(dead_code, unused_variables)]
use std::collections::HashSet;
fn squash(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The same text with every whitespace removed: letters, digits and
/// punctuation in order.
fn dense(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Letters and digits only, in lower case: what is left when punctuation,
/// typography (quotes, dashes, apostrophes), spacing and case are set aside.
fn letters(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// What kind of difference a found verse has from the edition's own text
/// (only for verses that differ beyond spacing and letter case):
/// - `unicode`: canonically the same characters (Unicode NFC): the document
///   encodes them differently (a CJK compatibility ideograph, 靈 U+F9B3,
///   written as the unified one, U+9748), the words are the same;
/// - `sign`: the same letters, with a note's sign the edition does not have
///   left in the verse ("†ᾧ"): apparatus, counted with `extra`;
/// - `typography`: the same letters and digits; punctuation or quotes differ;
/// - `extra`: the edition's words and more (a heading, note, reference or
///   another verse read into it: apparatus contamination);
/// - `missing`: part of the edition's words only (the verse is cut short);
/// - `wrong_reference`: the words of another verse of the book (a verse
///   given the wrong number or chapter);
/// - `changed`: anything else (the words themselves differ).
fn difference_kind(want: &str, found: &str, book: &HashSet<String>) -> &'static str {
    use unicode_normalization::UnicodeNormalization;
    let nfc = |t: &str| {
        t.split_whitespace()
            .collect::<String>()
            .nfc()
            .collect::<String>()
    };
    let (w, f) = (letters(want), letters(found));
    if nfc(want) == nfc(found) {
        "unicode"
    } else if w == f {
        let signs = |t: &str| t.chars().filter(|c| "*†‡§¶".contains(*c)).count();
        if signs(found) > signs(want) {
            "sign"
        } else {
            "typography"
        }
    } else if !w.is_empty() && f.contains(&w) {
        "extra"
    } else if !f.is_empty() && w.contains(&f) {
        "missing"
    } else if book.contains(&f) {
        "wrong_reference"
    } else {
        "changed"
    }
}

fn main() {
    let cases: &[(&str, &str, &str, bool)] = &[
    ("F01", "The shepherd rested by the gate.", "The shepherd rested by the gate.", false),
    ("F02", " The shepherd rested by the gate. ", "The shepherd rested by the gate.", false),
    ("F03", "\u{e9}l que cree", "e\u{301}l que cree", false),
    ("F04", "\u{f9b3}", "\u{9748}", false),
    ("F05", "\u{1f24} \u{3c3}\u{3bf}\u{3c6}\u{3af}\u{3b1}", "\u{3b7}\u{313}\u{301} \u{3c3}\u{3bf}\u{3c6}\u{3af}\u{3b1}", false),
    ("F06", "\u{5d1}\u{5bc}\u{5b0}\u{5e8}\u{5b5}\u{5d0}", "\u{5d1}\u{5b0}\u{5bc}\u{5e8}\u{5b5}\u{5d0}", false),
    ("F07", "The shepherd rested by the gate.", "The shepherd  rested by the gate.", false),
    ("F08", "\u{ab} Le berger \u{bb}", "\u{ab}\u{a0}Le berger\u{a0}\u{bb}", false),
    ("F09", "Le berger ; la veuve", "Le\u{9}berger\u{202f};\u{a}la veuve", false),
    ("F10", "\u{e9}l que cree", "e\u{301}l  que cree", false),
    ("W01", "She shall be set apart for the harvest.", "She shall be set a part for the harvest.", false),
    ("W02", "Her keeper is now here in the field.", "Her keeper is nowhere in the field.", false),
    ("W03", "no por obras, si no por gracia", "no por obras, sino por gracia", false),
    ("W04", "Y dijo que \u{e9}lir\u{e1} a la ciudad.", "Y dijo que\u{e9}lir\u{e1} a la ciudad.", false),
    ("W05", "Y dijo que \u{e9}l ir\u{e1} a la ciudad.", "Y dijo que\u{e9}lir\u{e1} a la ciudad.", false),
    ("W06", "rest, today", "rest,today", false),
    ("W07", "the shepherd rested by the gate", "theshepherdrestedbythegate", false),
    ("W08", "the shepherd", "the\u{200b}shepherd", false),
    ("W09", "\u{5db}\u{5bc}\u{5b8}\u{5dc}\u{5be}\u{5d4}\u{5b8}\u{5e2}\u{5b8}\u{5dd}", "\u{5db}\u{5bc}\u{5b8}\u{5dc} \u{5d4}\u{5b8}\u{5e2}\u{5b8}\u{5dd}", false),
    ("S01", "Le berger ; la veuve", "Le berger; la veuve", false),
    ("C01", "The Spirit rested on the reapers.", "The spirit rested on the reapers.", false),
    ("C02", "the LORD is my keeper", "the Lord is my keeper", false),
    ("C03", "The shepherd rested.", "THE SHEPHERD RESTED.", false),
    ("C04", "\u{39b}\u{3cc}\u{3b3}\u{3bf}\u{3c2}", "\u{3bb}\u{3cc}\u{3b3}\u{3bf}\u{3c2}", false),
    ("P01", "It's the elders' doing.", "Its the elders' doing.", false),
    ("P02", "He'll come again at dawn.", "Hell come again at dawn.", false),
    ("P03", "We're the reapers of the field.", "Were the reapers of the field.", false),
    ("P04", "Truly I tell you, today you will rest in the village.", "Truly I tell you today, you will rest in the village.", false),
    ("P05", "You are the kinsman.", "You are the kinsman?", false),
    ("P06", "[[The women wept at the gate.]]", "The women wept at the gate.", false),
    ("P07", "A servant \u{2014} not the master \u{2014} spoke first.", "A servant not the master spoke first.", false),
    ("P08", "He said, \u{201c}Come.\u{201d}", "He said, \u{22}Come.\u{22}", false),
    ("P09", "the elders\u{2019} gate", "the elders' gate", false),
    ("P10", "the gate", "the gate*", false),
    ("P11", "the gate*", "the gate", false),
    ("P12", "\u{2014}", "", false),
    ("P13", "[ ]", "\u{2014}", false),
    ("P14", "[Omitted]", "Omitted", false),
    ("P15", "well-known", "well\u{2013}known", false),
    ("P16", "the LORD's field", "the Lord\u{2019}s field", false),
    ("M01", "\u{e9}l que cree", "el que cree", false),
    ("M02", "e\u{301}l que cree", "el que cree", false),
    ("M03", "el que cree", "e\u{301}l que cree", false),
    ("M04", "\u{1f24} \u{3c3}\u{3bf}\u{3c6}\u{3af}\u{3b1}", "\u{3b7}\u{314}\u{301} \u{3c3}\u{3bf}\u{3c6}\u{3af}\u{3b1}", false),
    ("M05", "\u{3b7}\u{313}\u{301} \u{3c3}\u{3bf}\u{3c6}\u{3af}\u{3b1}", "\u{3b7}\u{314}\u{301} \u{3c3}\u{3bf}\u{3c6}\u{3af}\u{3b1}", false),
    ("M06", "\u{5d3}\u{5b8}\u{5d1}\u{5b8}\u{5e8}", "\u{5d3}\u{5b0}\u{5d1}\u{5b7}\u{5e8}", false),
    ("M07", "\u{5d3}\u{5bc}\u{5b8}\u{5d1}\u{5b8}\u{5e8}", "\u{5d3}\u{5b8}\u{5d1}\u{5b8}\u{5e8}", false),
    ("M08", "\u{5d1}\u{5bc}\u{5b0}\u{5e8}\u{5b5}\u{596}\u{5d0}", "\u{5d1}\u{5bc}\u{5b0}\u{5e8}\u{5b5}\u{5d0}", false),
    ("M09", "el que cree", "e\u{b4}l que cree", false),
    ("M10", "\u{1f41} \u{3c0}\u{3bf}\u{3b9}\u{3bc}\u{1f74}\u{3bd} \u{1f10}\u{3ba}\u{3ac}\u{3b8}\u{3b9}\u{3c3}\u{3b5}\u{3bd}", "\u{1f41} \u{3c0}\u{3bf}\u{3b9}\u{3bc}\u{3b7}\u{3bd} \u{1f10}\u{3ba}\u{3ac}\u{3b8}\u{3b9}\u{3c3}\u{3b5}\u{3bd}", false),
    ("K01", "the field", "the \u{fb01}eld", false),
    ("K02", "row 2", "row \u{b2}", false),
    ("Z01", "the gatekeeper", "the gate\u{ad}keeper", false),
    ("Z02", "the gate", "the ga\u{200d}te", false),
    ("X01", "They waited forty days by the river.", "They waited four days by the river.", false),
    ("X02", "40 days", "4 days", false),
    ("X03", "forty days", "40 days", false),
    ("X04", "The shepherd rested.", "The shepherd rested by the gate.", false),
    ("X05", "The shepherd rested by the gate.", "The shepherd rested.", false),
    ("X06", "The shepherd rested.", "The Shepherd at Rest The shepherd rested.", false),
    ("X07", "The weaver sang at midday.", "The potter worked at dawn.", true),
    ("X08", "The shepherd rested.", "", false),
    ("X09", "\u{2014}", "The shepherd rested.", false),
    ("X10", "The elders were glad.", "The elders were not glad.", false),
    ("X11", "\u{5d3}\u{5bc}\u{5b8}\u{5d1}\u{5b8}\u{5e8}", "\u{5d3}\u{5bc}\u{5b8}\u{5d1}\u{5b8}\u{5d3}", false),
    ("X12", "\u{3bb}\u{3cc}\u{3b3}\u{3bf}\u{3c2}", "\u{3bb}\u{3cc}\u{3b3}\u{3bf}\u{3b9}", false),
    ("X13", "the shepherd rested", "rested the shepherd", false),
    ("X14", "The weaver sang at midday.", "The potter worked at dawn.", false)
    ];
    for (id, want, text, matched) in cases {
        let mut book: HashSet<String> = HashSet::new();
        if *matched { book.insert(letters(text)); }
        let (want, text) = (*want, *text);
                let case_only = dense(want) != dense(text)
                    && dense(want).to_lowercase() == dense(text).to_lowercase();
                let class = if want.trim() == text.trim() {
                    0
                } else if squash(want) == squash(text) || dense(want) == dense(text) || case_only {
                    1
                } else {
                    2
                };

        let kind = difference_kind(want, text, &book);
        let scorer = if class < 2 { "correct" } else if matches!(kind, "typography" | "unicode") { "correct" } else { "wrong" };
        println!("{{\"id\": \"{id}\", \"class\": {class}, \"kind\": \"{}\", \"scorer\": \"{scorer}\"}}", if class == 2 { kind } else { "-" });
    }
}
