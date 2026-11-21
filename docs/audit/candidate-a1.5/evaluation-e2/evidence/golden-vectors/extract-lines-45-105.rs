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
