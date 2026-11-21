// Audit probe. The functions and the class block below are copied byte for
// byte from the frozen harness (08c6faac:src-tauri/tests/real_document_editions.rs,
// lines 45-105 and 712-720); nothing in them is edited.
#![allow(dead_code)]
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

fn class_of(want: &str, text: &str) -> u8 {
                let case_only = dense(want) != dense(text)
                    && dense(want).to_lowercase() == dense(text).to_lowercase();
                let class = if want.trim() == text.trim() {
                    0
                } else if squash(want) == squash(text) || dense(want) == dense(text) || case_only {
                    1
                } else {
                    2
                };
    class
}

/// Where the harness counts a VERIFIED verse (lines 744-758), and how
/// score-set.py observe_report() treats it (lines 196-210): only class 2
/// enters `differences`; of those, kind typography|unicode is not wrong.
fn buckets(class: u8, kind: &str) -> (&'static str, &'static str) {
    let harness = match class {
        0 => "verified_exact",
        1 => "verified_normalised",
        _ => match kind {
            "typography" | "unicode" => "verified_typography",
            "extra" | "sign" => "verified_extra_text",
            "missing" => "verified_missing_text",
            "wrong_reference" => "verified_wrong_reference",
            _ => "verified_changed",
        },
    };
    let scorer = if class < 2 || matches!(kind, "typography" | "unicode") {
        "verified_correct"
    } else {
        "verified_wrong"
    };
    (harness, scorer)
}

fn esc(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_graphic() || c == ' ' {
                if c == '"' || c == '\\' { format!("\\{c}") } else { c.to_string() }
            } else {
                format!("\\u{{{:04X}}}", c as u32)
            }
        })
        .collect()
}

fn main() {
    // (id, reference text (want), document text (found), what differs)
    let cases: &[(&str, &str, &str, &str)] = &[
        ("T01", "Jesus wept.", "Jesus wept.", "identical"),
        ("T02", "Jesus  wept.", "Jesus wept.", "double space"),
        ("T03", "Then he said, \u{201C}Come.\u{201D}", "Then he said, \"Come.\"", "curly vs straight quotes"),
        ("T04", "que \u{00E9}lir\u{00E1}", "que\u{00E9}lir\u{00E1}", "owner case: reference 'que élirá' (USFM delimiter rule) vs document 'queélirá'"),
        ("T05", "que \u{00E9}l ir\u{00E1}", "que\u{00E9}lir\u{00E1}", "same, if the reference were read as 'que él irá'"),
        ("M01", "and he shall be set apart for me", "and he shall be set a part for me", "word boundary: apart / a part"),
        ("M02", "God is now here.", "God is nowhere.", "word boundary: now here / nowhere"),
        ("M03", "no por obras, si no por gracia", "no por obras, sino por gracia", "Spanish: si no (if not) / sino (but rather)"),
        ("M04", "the Spirit of God moved", "the spirit of God moved", "case: Spirit / spirit"),
        ("M05", "the LORD is my shepherd", "the Lord is my shepherd", "case: LORD (YHWH) / Lord (Adonai)"),
        ("M06", "It's the LORD's doing.", "Its the LORD's doing.", "apostrophe: It's / Its"),
        ("M07", "He'll come again.", "Hell come again.", "apostrophe: He'll / Hell"),
        ("M08", "We're his people.", "Were his people.", "apostrophe: We're / Were"),
        ("M09", "Truly I tell you, today you will be with me in paradise.", "Truly I tell you today, you will be with me in paradise.", "comma placement (Luke 23:43)"),
        ("M10", "You are the Christ.", "You are the Christ?", "statement / question"),
        ("M11", "[[Jesus wept.]]", "Jesus wept.", "text-critical brackets dropped"),
        ("M12", "the man \u{2014} not God \u{2014} said it", "the man not God said it", "dashes / parenthesis dropped"),
        ("N01", "el que cree", "e\u{0301}l que cree", "document NFD adds acute: el (the) -> él (he)"),
        ("N02", "e\u{0301}l que cree", "el que cree", "reference NFD, document drops the acute: él -> el"),
        ("N03", "\u{00E9}l que cree", "el que cree", "both NFC, document drops the acute: él -> el"),
        ("N04", "\u{00E9}l que cree", "e\u{0301}l que cree", "NFC vs NFD of the same text (canonically equal)"),
        ("N05", "\u{1F24} \u{03C3}\u{03BF}\u{03C6}\u{03AF}\u{03B1}", "\u{03B7}\u{0314}\u{0301} \u{03C3}\u{03BF}\u{03C6}\u{03AF}\u{03B1}", "Greek: reference NFC eta+smooth+acute (or), document NFD eta+rough+acute (fem. relative)"),
        ("N06", "\u{03B7}\u{0313}\u{0301} \u{03C3}\u{03BF}\u{03C6}\u{03AF}\u{03B1}", "\u{03B7}\u{0314}\u{0301} \u{03C3}\u{03BF}\u{03C6}\u{03AF}\u{03B1}", "Greek: both NFD, breathing differs (smooth / rough)"),
        ("N07", "\u{05D3}\u{05B8}\u{05D1}\u{05B8}\u{05E8}", "\u{05D3}\u{05B0}\u{05D1}\u{05B7}\u{05E8}", "Hebrew points differ (dabar vowels)"),
        ("N08", "\u{05D3}\u{05BC}\u{05B8}\u{05D1}\u{05B8}\u{05E8}", "\u{05D3}\u{05B8}\u{05D1}\u{05B8}\u{05E8}", "Hebrew dagesh dropped"),
        ("N09", "\u{05D1}\u{05BC}\u{05B0}\u{05E8}\u{05B5}\u{0596}\u{05D0}\u{05E9}\u{05B4}\u{05C1}\u{05D9}\u{05EA}", "\u{05D1}\u{05BC}\u{05B0}\u{05E8}\u{05B5}\u{05D0}\u{05E9}\u{05B4}\u{05C1}\u{05D9}\u{05EA}", "Hebrew cantillation mark dropped"),
        ("N10", "\u{F9B3}", "\u{9748}", "CJK compatibility ideograph vs unified (harness's own example)"),
        ("N11", "el que cree", "e\u{00B4}l que cree", "document has a spacing acute (PDF artefact) on a plain letter"),
        ("S01", "word", "word*", "note sign added in document"),
        ("S02", "word*", "word", "note sign absent in document"),
        ("S03", "In the beginning God created", "In the beginning God created the heavens", "extra words"),
        ("S04", "In the beginning God created the heavens", "In the beginning God created", "words missing"),
        ("S05", "forty days", "four days", "one word changed"),
        ("S06", "40 days", "4 days", "digit dropped"),
        ("P01", "\u{2014}", "", "punctuation-only placeholder vs empty text"),
        ("P02", "[ ]", "\u{2014}", "punctuation-only placeholder vs another one"),
        ("P03", "[Omitted]", "Omitted", "placeholder word, brackets dropped"),
        ("P04", "\u{2014}", "Jesus wept.", "punctuation placeholder vs real text"),
    ];
    let book: HashSet<String> = HashSet::new();
    for (id, want, found, note) in cases {
        let class = class_of(want, found);
        let kind = if class == 2 { difference_kind(want, found, &book) } else { "-" };
        let (h, s) = buckets(class, kind);
        println!(
            "{{\"id\":\"{id}\",\"class\":{class},\"kind\":\"{kind}\",\"harness\":\"{h}\",\"scorer\":\"{s}\",\"want\":\"{}\",\"found\":\"{}\",\"note\":\"{}\"}}",
            esc(want), esc(found), esc(note)
        );
    }
    // Which marks letters() keeps (char::is_alphanumeric).
    for c in ['\u{0301}', '\u{0313}', '\u{0314}', '\u{0342}', '\u{0345}', '\u{05B0}', '\u{05B8}', '\u{05BC}', '\u{05C1}', '\u{0596}', '\u{05BE}', '\u{064E}', '\u{00B4}', '\u{200C}', '\u{00AD}'] {
        println!("{{\"char\":\"U+{:04X}\",\"is_alphanumeric\":{}}}", c as u32, c.is_alphanumeric());
    }
}
