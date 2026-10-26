//! Finding books, chapters and verses in an ordinary document (v0.23).
//!
//! The units of a Bible document (`scripture::documents`: PDF and text
//! lines, EPUB/DOCX/Markdown paragraphs and headings) are read in order.
//! What counts as evidence, and what never does:
//!
//! * **Book names** count only as a whole heading or line ("Genesis",
//!   "1. Mose", "창세기", "בראשית", "ΚΑΤΑ ΙΩΑΝΝΗΝ"), read with the reference
//!   dictionaries of the six interface languages and of Hebrew and Greek. A
//!   file name, a table of contents or a reference inside a sentence never
//!   names a book.
//! * **Chapters** come from a heading ("Chapter 3", "Kapitel 3", "3장",
//!   "Psalm 23"), a line reference ("3:1 …"), or a chapter number standing at
//!   the start of a paragraph where the previous chapter is complete by the
//!   standard verse counts (the large chapter numbers printed Bibles use).
//! * **Verses** come from numbers in the text that continue the count: the
//!   next verse (or a bridge such as "4–5"), at the start of a line or after
//!   punctuation, or — for the very next verse only — after a space. A
//!   number glued to a word ("God1", "5th"), followed by ":" or "," and a
//!   digit (a reference), or after a word such as "chapter" or a book name is
//!   never a verse number. Where two places could start the same verse
//!   equally well, the chapter is refused ([`Severity::Error`]), never
//!   guessed.
//!
//! Text that belongs to no verse (introductions, headings inside a chapter,
//! notes sections, a document's own front matter) is left out and reported;
//! page headers and page numbers repeated on most pages of a PDF are taken
//! out and counted. A Psalm's title before its verse 1 is kept as its title.
//! Nothing is invented: a verse without a number in the text is a missing
//! verse, except the first verse of a chapter that starts right after its
//! heading or chapter number (reported).
//!
//! The result is the same staged content every Bible import fills
//! (`scripture::import::StagedContent`), checked by the same structure
//! checks, plus where each verse came from in the document.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::OnceLock;

use crate::library::adapters::pdf_layout::{RAISED_CLOSE, RAISED_OPEN};

use serde::Serialize;

use crate::error::UserMessage;
use crate::scripture::canon::{self, Book};
use crate::scripture::documents::{Place, Unit};
use crate::scripture::import::{Severity, StagedContent};
use crate::scripture::language::ReferenceLanguage;
use crate::scripture::parser::{fold_marks, normalise, ReferenceParser};

/// A verse longer than this cannot be one verse (the longest, Esther 8:9,
/// is about 540 characters in English): the structure was not recognised.
const VERSE_MAX_CHARS: usize = 2_500;
/// How far ahead a verse number looks for a better place for the same verse.
const LOOKAHEAD: usize = 400;
/// A chapter this many verses shorter than usual may be another numbering
/// (Joel 2 has 27 verses in Hebrew Bibles, 32 in English ones); more, and
/// its end was not found.
const SHORT_TOLERANCE: u16 = 6;
/// Samples of left-out text kept for the review.
const SAMPLES: usize = 12;

/// Where a recognised verse came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Origin {
    pub start: Place,
    pub end: Place,
}

/// A book heading found in the document, as the review lists it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FoundBook {
    /// Its place in the list (corrections name it by this).
    pub index: usize,
    /// The heading as written.
    pub heading: String,
    /// The book the heading names, if it names one.
    pub detected: Option<&'static str>,
    /// The book used: the detected one, or the person's correction (None:
    /// none, or left out).
    pub book: Option<&'static str>,
    pub left_out: bool,
    pub chapters: usize,
    pub verses: usize,
    pub place: Place,
}

/// Text left out of the verses, as the review shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeftOut {
    pub sample: String,
    pub chars: usize,
    pub place: Place,
    /// "heading", "beforeVerses", "noBook", "notes", "leftOutBook".
    pub reason: &'static str,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    /// Lines taken out as page headers or page numbers (PDF).
    pub page_lines: usize,
    /// Lines of footnotes at the foot of pages, left out (v0.24).
    pub footnote_lines: usize,
    /// Headings inside chapters left out of the text.
    pub headings: usize,
    /// Characters of text left out (all reasons).
    pub left_out_chars: usize,
    pub psalm_titles: usize,
    /// Chapters whose verse 1 had no number of its own.
    pub unnumbered_first: usize,
    /// Chapters found by their number alone (no heading).
    pub numbered_chapters: usize,
}

/// The person's corrections: for a found book (by index), another book or
/// none (leave it out).
#[derive(Debug, Clone, Default)]
pub struct Corrections {
    pub books: HashMap<usize, Option<&'static Book>>,
}

#[derive(Debug, Default)]
pub struct Recognition {
    pub content: StagedContent,
    pub origins: BTreeMap<(u8, u16, u16), Origin>,
    pub books: Vec<FoundBook>,
    pub left_out: Vec<LeftOut>,
    pub stats: Stats,
}

impl Recognition {
    pub fn verse_count(&self) -> usize {
        self.content.verse_count()
    }

    /// Whether the verses can be used: some were found and nothing
    /// contradicts itself.
    pub fn usable(&self) -> bool {
        self.verse_count() > 0 && !self.content.has_errors()
    }
}

// --- book names ---------------------------------------------------------------

/// A heading's text made comparable: marks folded, lower case, no full stops.
fn key(text: &str) -> String {
    normalise(&fold_marks(text))
}

/// Greek titles of the New Testament books (as printed in Greek editions).
const GREEK_TITLES: &[(&str, &str)] = &[
    ("ΚΑΤΑ ΜΑΤΘΑΙΟΝ", "MAT"),
    ("ΚΑΤΑ ΜΑΘΘΑΙΟΝ", "MAT"),
    ("ΚΑΤΑ ΜΑΡΚΟΝ", "MRK"),
    ("ΚΑΤΑ ΛΟΥΚΑΝ", "LUK"),
    ("ΚΑΤΑ ΙΩΑΝΝΗΝ", "JHN"),
    ("ΠΡΑΞΕΙΣ ΑΠΟΣΤΟΛΩΝ", "ACT"),
    ("ΠΡΑΞΕΙΣ", "ACT"),
    ("ΠΡΟΣ ΡΩΜΑΙΟΥΣ", "ROM"),
    ("ΠΡΟΣ ΚΟΡΙΝΘΙΟΥΣ Α", "1CO"),
    ("ΠΡΟΣ ΚΟΡΙΝΘΙΟΥΣ Β", "2CO"),
    ("ΠΡΟΣ ΓΑΛΑΤΑΣ", "GAL"),
    ("ΠΡΟΣ ΕΦΕΣΙΟΥΣ", "EPH"),
    ("ΠΡΟΣ ΦΙΛΙΠΠΗΣΙΟΥΣ", "PHP"),
    ("ΠΡΟΣ ΚΟΛΟΣΣΑΕΙΣ", "COL"),
    ("ΠΡΟΣ ΘΕΣΣΑΛΟΝΙΚΕΙΣ Α", "1TH"),
    ("ΠΡΟΣ ΘΕΣΣΑΛΟΝΙΚΕΙΣ Β", "2TH"),
    ("ΠΡΟΣ ΤΙΜΟΘΕΟΝ Α", "1TI"),
    ("ΠΡΟΣ ΤΙΜΟΘΕΟΝ Β", "2TI"),
    ("ΠΡΟΣ ΤΙΤΟΝ", "TIT"),
    ("ΠΡΟΣ ΦΙΛΗΜΟΝΑ", "PHM"),
    ("ΠΡΟΣ ΕΒΡΑΙΟΥΣ", "HEB"),
    ("ΙΑΚΩΒΟΥ", "JAS"),
    ("ΠΕΤΡΟΥ Α", "1PE"),
    ("ΠΕΤΡΟΥ Β", "2PE"),
    ("ΙΩΑΝΝΟΥ Α", "1JN"),
    ("ΙΩΑΝΝΟΥ Β", "2JN"),
    ("ΙΩΑΝΝΟΥ Γ", "3JN"),
    ("ΙΟΥΔΑ", "JUD"),
    ("ΑΠΟΚΑΛΥΨΙΣ ΙΩΑΝΝΟΥ", "REV"),
    ("ΑΠΟΚΑΛΥΨΙΣ", "REV"),
];

/// Words in front of a book's name in a title, dropped before looking it up.
const TITLE_PREFIXES: &[&str] = &[
    "the book of ",
    "book of ",
    // Longer forms first: the first prefix that matches is taken.
    "the gospel according to saint ",
    "the gospel according to st ",
    "the gospel according to ",
    "the gospel of ",
    "das buch ",
    "das evangelium nach ",
    "evangelium nach ",
    "le livre de ",
    "livre de ",
    "l'evangile selon ",
    "evangile selon ",
    "el libro de ",
    "libro de ",
    "el evangelio segun ",
    "evangelio segun ",
    "o livro de ",
    "livro de ",
    "o evangelho segundo ",
    "evangelho segundo ",
];

/// The languages whose book names are read: the six interface languages,
/// Hebrew and Greek.
fn name_languages() -> Vec<ReferenceLanguage> {
    let mut languages = ReferenceLanguage::REVIEWED.to_vec();
    for code in ["he", "el"] {
        if let Some(language) = ReferenceLanguage::from_label(code) {
            languages.push(language);
        }
    }
    languages
}

/// Every book name, by its key: the books it can mean, with the language.
fn names() -> &'static HashMap<String, Vec<(&'static Book, ReferenceLanguage)>> {
    static TABLE: OnceLock<HashMap<String, Vec<(&'static Book, ReferenceLanguage)>>> =
        OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table: HashMap<String, Vec<(&'static Book, ReferenceLanguage)>> = HashMap::new();
        let mut add = |name: &str, book: &'static Book, language: ReferenceLanguage| {
            let entry = table.entry(key(name)).or_default();
            if !entry
                .iter()
                .any(|(b, l)| b.code == book.code && *l == language)
            {
                entry.push((book, language));
            }
        };
        for book in canon::BOOKS.iter() {
            add(book.name, book, ReferenceLanguage::En);
        }
        for language in name_languages() {
            for (name, book) in ReferenceParser::for_language(language).lookup_entries() {
                add(name, book, language);
            }
        }
        for (title, code) in GREEK_TITLES {
            if let (Some(book), Some(greek)) =
                (canon::by_code(code), ReferenceLanguage::from_label("el"))
            {
                add(title, book, greek);
            }
        }
        table
    })
}

/// A book's printed title brought down to its name ("Das erste Buch Samuel"
/// → "1 samuel", "Der zweite Brief des Apostels Paulus an die Korinther" →
/// "2 korinther", "Der Prophet Jesaja" → "jesaja", "The Second Book of the
/// Kings" → "2 kings"): a leading article, an ordinal word (kept as its
/// number) and the words that describe the book (book, letter, prophet,
/// gospel, of, to, the apostle Paul …) are taken off. The candidates are only
/// looked up among the book names; a title that does not end in one names
/// no book.
fn title_forms(key: &str) -> Vec<String> {
    // The words of printed titles, by language (accents folded, lower case,
    // as `key` gives them). The reasoning below is the same for every
    // language; only these tables differ.
    const ORDINALS: &[(&str, u8)] = &[
        // German
        ("erste", 1),
        ("zweite", 2),
        ("dritte", 3),
        ("vierte", 4),
        ("funfte", 5),
        ("fuenfte", 5),
        // English
        ("first", 1),
        ("second", 2),
        ("third", 3),
        ("fourth", 4),
        ("fifth", 5),
        // French
        ("premier", 1),
        ("premiere", 1),
        ("deuxieme", 2),
        ("seconde", 2),
        ("troisieme", 3),
        ("quatrieme", 4),
        ("cinquieme", 5),
        // Spanish
        ("primer", 1),
        ("primero", 1),
        ("primera", 1),
        ("segundo", 2),
        ("segunda", 2),
        ("tercer", 3),
        ("tercero", 3),
        ("tercera", 3),
        ("cuarto", 4),
        ("quinto", 5),
        // Portuguese
        ("primeiro", 1),
        ("primeira", 1),
        ("terceiro", 3),
        ("terceira", 3),
        ("quarto", 4),
        // Figures and Roman numerals
        ("1", 1),
        ("2", 2),
        ("3", 3),
        ("4", 4),
        ("5", 5),
        ("i", 1),
        ("ii", 2),
        ("iii", 3),
        ("iv", 4),
        ("v", 5),
    ];
    const ARTICLES: &[&str] = &[
        "das", "der", "die", "the", "le", "la", "les", "l'", "el", "los", "las", "o", "os", "as",
    ];
    const NOUNS: &[&str] = &[
        // German and English
        "buch",
        "book",
        "brief",
        "letter",
        "epistle",
        "prophet",
        "propheten",
        "evangelium",
        "gospel",
        "apostels",
        "apostel",
        "apostle",
        "paulus",
        "paul",
        "pauls",
        "paul's",
        "general",
        "saint",
        "st",
        "heiligen",
        "sankt",
        // French
        "livre",
        "epitre",
        "evangile",
        "prophete",
        "apotre",
        "sainte",
        // Spanish
        "libro",
        "epistola",
        "carta",
        "evangelio",
        "profeta",
        "apostol",
        "san",
        "santo",
        "pablo",
        // Portuguese
        "livro",
        "evangelho",
        "apostolo",
        "sao",
        "paulo",
    ];
    // Only after one of the words above ("Book of", "Brief an die", "Épître
    // aux"): a line that begins "of Jeremiah" is a subtitle, never a title.
    const LINKING: &[&str] = &[
        "das", "der", "die", "des", "dem", "the", "an", "nach", "of", "to", "from", "by",
        "according", // German, English
        "de", "du", "d'", "aux", "au", "a", "selon", "la", "le", "les", "l'", // French
        "del", "los", "las", "segun", "el", // Spanish
        "do", "da", "dos", "das", "aos", "ao", "segundo", "o", "os", // Portuguese
    ];
    const AFTER: &[&str] = &[
        "jesu christi",
        "of jesus christ",
        "salomos",
        "salomo",
        "of solomon",
        "de jesus-christ",
        "de jesucristo",
        "de jesus cristo",
    ];
    // The books named by their first word alone: "The Acts of the Apostles",
    // "Les Actes des Apôtres", "The Revelation to John", "The Lamentations of
    // Jeremiah". Any other title must be the name and nothing else.
    const FIRST_WORD: &[&str] = &[
        "acts",
        "revelation",
        "lamentations",
        "apostelgeschichte",
        "offenbarung",
        "klagelieder",
        "actes",
        "apocalypse",
        "hechos",
        "apocalipsis",
        "lamentaciones",
        "atos",
        "apocalipse",
        "lamentacoes",
    ];
    // "l'evangile" → "l' evangile", "d'esaie" → "d' esaie".
    let key = key.replace('’', "'");
    let key = key.replace("l'", "l' ").replace("d'", "d' ");

    let all: Vec<&str> = key.split(' ').filter(|w| !w.is_empty()).collect();
    let mut extra: Vec<String> = Vec::new();
    // "Paul's First Letter to the Corinthians" → "1 corinthians";
    // "Peter's Second Letter" → "2 peter"; "The Letter from James" → "james".
    if let Some(at) = all
        .iter()
        .position(|w| matches!(*w, "letter" | "epistle" | "brief" | "epitre" | "epistola" | "carta"))
    {
        let n = all[..at]
            .iter()
            .find_map(|w| ORDINALS.iter().find(|(o, _)| o == w).map(|(_, n)| *n));
        let after: Vec<&str> = all[at + 1..]
            .iter()
            .copied()
            .skip_while(|w| LINKING.contains(w) || NOUNS.contains(w))
            .collect();
        let owner = all[..at]
            .iter()
            .rev()
            .find_map(|w| w.strip_suffix("'s"))
            .map(str::to_string);
        let name = if after.is_empty() {
            owner
        } else {
            Some(after.join(" "))
        };
        if let Some(name) = name {
            extra.push(match n {
                Some(n) => format!("{n} {name}"),
                None => name,
            });
        }
    }
    let mut words: Vec<&str> = all.clone();
    let mut number: Option<u8> = None;
    let mut changed = false;
    let mut noun = false;
    let mut at_start = true;
    while let Some(first) = words.first().copied() {
        if number.is_none() {
            if let Some((_, n)) = ORDINALS.iter().find(|(w, _)| *w == first) {
                number = Some(*n);
                words.remove(0);
                changed = true;
                at_start = false;
                continue;
            }
        }
        let take = NOUNS.contains(&first)
            || (at_start && ARTICLES.contains(&first))
            || (noun && LINKING.contains(&first));
        if take {
            noun |= NOUNS.contains(&first);
            words.remove(0);
            changed = true;
            at_start = false;
            continue;
        }
        break;
    }
    if !changed || words.is_empty() {
        return extra;
    }
    let mut rest = words.join(" ");
    for suffix in AFTER {
        if let Some(stripped) = rest.strip_suffix(suffix) {
            if !stripped.trim().is_empty() {
                rest = stripped.trim().to_string();
            }
        }
    }
    // "(Genesis)" and the like after the name.
    if let Some(open) = rest.find('(') {
        let name = rest[..open].trim().to_string();
        if !name.is_empty() {
            rest = name;
        }
    }
    let mut out = extra;
    match number {
        Some(n) => out.push(format!("{n} {rest}")),
        None => {
            // "The Acts of the Apostles", "The Revelation to John", "The
            // Lamentations of Jeremiah": only these books are named by their
            // first word. Otherwise the whole rest must be the name: "The
            // Song of Moses", "The Mark of the Beast", "The Job Speaks" name
            // no book.

            let mut parts = rest.splitn(2, ' ');
            if let (Some(first), Some(after)) = (parts.next(), parts.next()) {
                let linked = after
                    .split(' ')
                    .next()
                    .is_some_and(|w| LINKING.contains(&w));
                if FIRST_WORD.contains(&first) && linked {
                    out.push(first.to_string());
                }
            }
            out.insert(0, rest);
        }
    }
    out
}

/// A title with one letter wrong, missing or extra ("Chonik" for "Chronik"):
/// the one book name, of six letters or more, it is that close to. Two or
/// more names that close: none. The review shows the heading as printed next
/// to the book it was read as, so the person can correct it.
fn near_name(key: &str) -> Option<&'static Vec<(&'static Book, ReferenceLanguage)>> {
    fn close(a: &str, b: &str) -> bool {
        let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
        if a.len().abs_diff(b.len()) > 1 || a == b {
            return false;
        }
        let (mut i, mut j, mut edits) = (0, 0, 0);
        while i < a.len() && j < b.len() {
            if a[i] == b[j] {
                i += 1;
                j += 1;
                continue;
            }
            edits += 1;
            if edits > 1 {
                return false;
            }
            match a.len().cmp(&b.len()) {
                std::cmp::Ordering::Greater => i += 1,
                std::cmp::Ordering::Less => j += 1,
                std::cmp::Ordering::Equal => {
                    i += 1;
                    j += 1;
                }
            }
        }
        edits + (a.len() - i) + (b.len() - j) <= 1
    }
    let letters = key.chars().filter(|c| c.is_alphabetic()).count();
    if letters < 6 {
        return None;
    }
    static BY_LENGTH: OnceLock<HashMap<usize, Vec<&'static String>>> = OnceLock::new();
    let by_length = BY_LENGTH.get_or_init(|| {
        let mut map: HashMap<usize, Vec<&'static String>> = HashMap::new();
        for name in names().keys() {
            map.entry(name.chars().count()).or_default().push(name);
        }
        map
    });
    let length = key.chars().count();
    let mut found: Option<&'static Vec<(&'static Book, ReferenceLanguage)>> = None;
    let mut book: Option<&'static str> = None;
    let candidates = (length.saturating_sub(1)..=length + 1)
        .filter_map(|l| by_length.get(&l))
        .flatten();
    for name in candidates {
        if !close(key, name) {
            continue;
        }
        let Some(entry) = names().get(name.as_str()) else {
            continue;
        };
        let code = entry.first()?.0.code;
        if entry.iter().any(|(b, _)| b.code != code) || book.is_some_and(|c| c != code) {
            return None;
        }
        book = Some(code);
        found = Some(entry);
    }
    found
}

/// The book a whole heading names, if exactly one (the document's language
/// decides between books that share a name).
fn book_named(text: &str, language: Option<ReferenceLanguage>) -> Option<&'static Book> {
    let mut k = key(text);
    for prefix in TITLE_PREFIXES {
        if let Some(rest) = k.strip_prefix(prefix) {
            k = rest.to_string();
            break;
        }
    }
    let forms = title_forms(&k);
    let found = match names().get(&k) {
        Some(found) => found,
        None => match forms.iter().find_map(|f| names().get(f)) {
            Some(found) => found,
            // Only for a line printed as a title ("Das erste Buch der …").
            None => forms.iter().find_map(|f| near_name(f))?,
        },
    };
    let mut books: Vec<&'static Book> = Vec::new();
    for (book, _) in found {
        if !books.iter().any(|b| b.code == book.code) {
            books.push(book);
        }
    }
    match books.as_slice() {
        [one] => Some(one),
        _ => {
            let language = language?;
            let own: Vec<&'static Book> = found
                .iter()
                .filter(|(_, l)| *l == language)
                .map(|(b, _)| *b)
                .collect();
            match own.as_slice() {
                [one] => Some(one),
                _ => None,
            }
        }
    }
}

// --- numbers -------------------------------------------------------------------

fn superscript_digit(c: char) -> Option<u32> {
    match c {
        '⁰' => Some(0),
        '¹' => Some(1),
        '²' => Some(2),
        '³' => Some(3),
        '⁴'..='⁹' => Some(c as u32 - '⁴' as u32 + 4),
        _ => None,
    }
}

fn roman(text: &str) -> Option<u16> {
    if text.is_empty() || text.len() > 9 || !text.chars().all(|c| "IVXLC".contains(c)) {
        return None;
    }
    let value = |c| match c {
        'I' => 1,
        'V' => 5,
        'X' => 10,
        'L' => 50,
        _ => 100,
    };
    let chars: Vec<i32> = text.chars().map(value).collect();
    let mut total = 0i32;
    for (i, v) in chars.iter().enumerate() {
        if chars.get(i + 1).is_some_and(|next| next > v) {
            total -= v;
        } else {
            total += v;
        }
    }
    (total > 0).then_some(total as u16)
}

/// A Hebrew numeral (א = 1 … ת = 400; ט״ו = 15), its marks ignored.
fn hebrew_numeral(text: &str) -> Option<u16> {
    let mut total = 0u16;
    let mut any = false;
    for c in text.chars() {
        let v = match c {
            'א'..='ט' => c as u16 - 'א' as u16 + 1,
            'י' => 10,
            'כ' | 'ך' => 20,
            'ל' => 30,
            'מ' | 'ם' => 40,
            'נ' | 'ן' => 50,
            'ס' => 60,
            'ע' => 70,
            'פ' | 'ף' => 80,
            'צ' | 'ץ' => 90,
            'ק' => 100,
            'ר' => 200,
            'ש' => 300,
            'ת' => 400,
            '\'' | '"' | '׳' | '״' => continue,
            _ => return None,
        };
        total += v;
        any = true;
    }
    (any && total > 0).then_some(total)
}

/// A number written as a chapter number: digits, Roman, or Hebrew letters.
fn chapter_number(text: &str) -> Option<u16> {
    let t = text.trim();
    if !t.is_empty() && t.len() <= 3 && t.chars().all(|c| c.is_ascii_digit()) {
        return t.parse().ok().filter(|n| *n > 0);
    }
    roman(t).or_else(|| hebrew_numeral(t))
}

// --- headings --------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Heading {
    Book(&'static Book),
    BookChapter(&'static Book, u16),
    Chapter(u16),
}

/// Words that name a chapter, before its number ("Chapter 3").
const CHAPTER_WORDS: &[&str] = &[
    "chapter",
    "chap",
    "kapitel",
    "chapitre",
    "capitulo",
    "capítulo",
    "κεφαλαιο",
    "κεφαλαιον",
    "פרק",
];
/// Words that name a Psalm, before its number ("Psalm 23").
const PSALM_WORDS: &[&str] = &[
    "psalm",
    "psalms",
    "psaume",
    "salmo",
    "salmos",
    "ψαλμος",
    "מזמור",
];

/// Removes the invisible marks a document may carry (bidirectional
/// controls, soft hyphens, zero-width characters); they are not text.
fn clean(text: &str) -> String {
    text.chars()
        .filter(|c| {
            !matches!(
                c,
                '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{00AD}' | '\u{FEFF}'
            )
        })
        .map(|c| if c == '\u{00A0}' { ' ' } else { c })
        .collect()
}

/// Whether a line is a table of contents entry: a name, dot leaders (three
/// dots or more, spaced or not, or ellipsis characters) and a page number.
fn contents_entry(text: &str) -> bool {
    use std::sync::OnceLock;
    static FORM: OnceLock<regex::Regex> = OnceLock::new();
    let form = FORM.get_or_init(|| {
        regex::Regex::new(r"\p{L}.*?(?:(?:\.\s*){3,}|…{2,}|(?:·\s*){3,})\s*\d{1,5}\s*$")
            .expect("valid pattern")
    });
    form.is_match(text.trim())
}

/// What a heading or a heading-like line says, if it names a book or a
/// chapter. The whole text must be the name: nothing else on the line.
fn read_heading(text: &str, language: Option<ReferenceLanguage>) -> Option<Heading> {
    // A table of contents entry ("Exodus . . . . . 60", "Exodus …… 60"): dot
    // leaders to a page number are never a heading.
    if contents_entry(text) {
        return None;
    }
    let text = text.trim().trim_end_matches(['.', ':', '·']).trim();
    if text.is_empty() || text.chars().count() > 60 {
        return None;
    }
    let lower = key(text);
    let words: Vec<&str> = lower.split(' ').collect();
    // "Chapter 3", "פרק ג", "Κεφάλαιο 3".
    if words.len() == 2 && CHAPTER_WORDS.contains(&words[0]) {
        let raw: Vec<&str> = text.split_whitespace().collect();
        let number = raw.last().map(|n| n.trim_end_matches('.'))?;
        return chapter_number(number).map(Heading::Chapter);
    }
    // "3. Kapitel"
    if words.len() == 2 && words[1] == "kapitel" {
        return chapter_number(words[0].trim_end_matches('.')).map(Heading::Chapter);
    }
    // Korean: "제3장", "3장", "제 3 장", "제3편", "3편".
    let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    if let Some(n) = korean_unit(&compact, &['장', '편']) {
        return Some(Heading::Chapter(n));
    }
    // "Psalm 23"
    if words.len() == 2 && PSALM_WORDS.contains(&words[0]) {
        let psalms = canon::by_code("PSA")?;
        return chapter_number(text.split_whitespace().last()?)
            .map(|n| Heading::BookChapter(psalms, n));
    }
    // A book's name alone.
    if let Some(book) = book_named(text, language) {
        return Some(Heading::Book(book));
    }
    // A book's name and a chapter: "Genesis 1", "1 Mose 3", "창세기 1장", "시편 제23편".
    let mut parts: Vec<&str> = text.split_whitespace().collect();
    if parts.len() >= 2 {
        let last = parts.pop()?;
        let number =
            korean_unit(last, &['장', '편']).or_else(|| chapter_number(last.trim_end_matches('.')));
        if let Some(number) = number {
            let name = parts.join(" ");
            if let Some(book) = book_named(&name, language) {
                return Some(Heading::BookChapter(book, number));
            }
        }
    }
    // "창세기1장"
    if let Some(at) = compact.find(|c: char| c.is_ascii_digit()) {
        if at > 0 {
            if let (Some(book), Some(n)) = (
                book_named(&compact[..at], language),
                korean_unit(&compact[at..], &['장', '편']),
            ) {
                return Some(Heading::BookChapter(book, n));
            }
        }
    }
    None
}

/// "제3장" / "3장" (and 편 for Psalms) → 3.
fn korean_unit(text: &str, units: &[char]) -> Option<u16> {
    let t = text.strip_prefix('제').unwrap_or(text);
    let last = t.chars().last()?;
    if !units.contains(&last) {
        return None;
    }
    let digits = &t[..t.len() - last.len_utf8()];
    if digits.is_empty() || digits.len() > 3 || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok().filter(|n| *n > 0)
}

/// Headings of notes sections: what follows them is not Scripture text.
fn is_notes_heading(text: &str) -> bool {
    matches!(
        key(text).as_str(),
        "footnotes"
            | "notes"
            | "endnotes"
            | "fussnoten"
            | "fußnoten"
            | "anmerkungen"
            | "각주"
            | "미주"
            | "notes de bas de page"
            | "notas"
            | "notas al pie"
            | "notas de rodape"
            | "cross references"
            | "querverweise"
    )
}

// --- verse numbers -----------------------------------------------------------------

/// How sure a number is to be a verse number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Strength {
    /// After a space, mid-sentence: only ever the very next verse.
    Medium = 1,
    /// At the start of a line or paragraph, after punctuation, or raised.
    Strong = 2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Marker {
    value: u16,
    end: Option<u16>,
    /// A line reference names its chapter ("3:16 …").
    chapter: Option<u16>,
    /// … and sometimes its book ("Genesis 1:1 …").
    book: Option<&'static Book>,
    strength: Strength,
    unit_start: bool,
    /// The number as written ("12", "(3)", "4–5", "3:16"), put back into the
    /// text when it turns out not to be a verse number.
    raw: String,
    /// Whether the text goes on right after it, with no space ("1In").
    glued: bool,
    /// Whether it follows the text before it with no space ("away.3").
    joined: bool,
}

/// Punctuation a verse can end with, before the next verse's number.
fn ends_sentence(c: char) -> bool {
    matches!(
        c,
        '.' | ','
            | ';'
            | ':'
            | '!'
            | '?'
            | '׃'
            | '·'
            | '\u{037E}'
            | '»'
            | '”'
            | '"'
            | '’'
            | '\''
            | ')'
            | ']'
            | '…'
            | '。'
            | '」'
            | '—'
            | '–'
            | '¶'
            | '*'
    )
}

/// Words after which a number is a reference, never a verse number.
const BEFORE_NUMBER: &[&str] = &[
    "chapter",
    "chapters",
    "verse",
    "verses",
    "v",
    "vv",
    "ver",
    "kapitel",
    "vers",
    "verse",
    "chapitre",
    "chapitres",
    "verset",
    "versets",
    "capitulo",
    "capitulos",
    "versiculo",
    "versiculos",
    "psalm",
    "psalms",
    "ps",
    "page",
    "p",
    "seite",
    "s",
    "pagina",
    "anno",
    "year",
    "jahr",
    "an",
];

/// A piece of a unit's text: words, or a verse number.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Piece {
    Text(String),
    Marker(Marker),
}

/// A line reference read: chapter, verse, a bridge's last verse, the book
/// it names, and the bytes it takes.
type LineReference = (u16, u16, Option<u16>, Option<&'static Book>, usize);

/// A line reference at the very start of a unit: "3:16 …", "3.16 …",
/// "Genesis 1:1 …", "1 Mose 1,1 …" — the chapter, verse, bridge end, book
/// and the byte length taken.
fn line_reference(text: &str, language: Option<ReferenceLanguage>) -> Option<LineReference> {
    let mut book = None;
    let mut rest = text;
    let mut taken = 0usize;
    if !text.starts_with(|c: char| c.is_ascii_digit())
        || text.starts_with(|c: char| c.is_ascii_digit())
            && text.chars().nth(1).is_some_and(|c| c == ' ')
    {
        // A book name of one to four words first ("1 Mose" starts with a digit too).
        let words: Vec<(usize, &str)> = text
            .split_inclusive(' ')
            .scan(0usize, |at, w| {
                let start = *at;
                *at += w.len();
                Some((start, w))
            })
            .take(5)
            .collect();
        for n in (1..=words.len().min(4)).rev() {
            let end = words[n - 1].0 + words[n - 1].1.len();
            let name = text[..end].trim();
            if let Some(found) = book_named(name, language) {
                let after = &text[end..];
                if after.starts_with(|c: char| c.is_ascii_digit()) {
                    book = Some(found);
                    rest = after;
                    taken = end;
                    break;
                }
            }
        }
    }
    let digits = |s: &str| s.chars().take_while(|c| c.is_ascii_digit()).count();
    let c_len = digits(rest);
    if c_len == 0 || c_len > 3 {
        return None;
    }
    let sep = rest[c_len..].chars().next()?;
    if !matches!(sep, ':' | '.' | ',') {
        return None;
    }
    let after_sep = &rest[c_len + 1..];
    let v_len = digits(after_sep);
    if v_len == 0 || v_len > 3 {
        return None;
    }
    let chapter: u16 = rest[..c_len].parse().ok()?;
    let verse: u16 = after_sep[..v_len].parse().ok()?;
    // "2,300" and "1.500" are numbers, not chapter and verse: a three-digit
    // verse after "," or "." is only Psalm 119's. No chapter has more than
    // 176 verses, nor any book more than 150 chapters.
    if sep != ':' && v_len == 3 && chapter != 119 {
        return None;
    }
    if verse > 176 || chapter > 150 {
        return None;
    }
    // Far beyond the chapter's standard count (other numberings differ by a few).
    if let Some(count) = book.and_then(|b| canon::verse_count(b, chapter)) {
        if verse > count + 10 {
            return None;
        }
    }
    let mut used = c_len + 1 + v_len;
    let mut end = None;
    let tail = &rest[used..];
    if let Some(dash) = tail.chars().next().filter(|c| matches!(c, '-' | '–' | '—')) {
        let more = &tail[dash.len_utf8()..];
        let e_len = digits(more);
        if (1..=3).contains(&e_len) {
            let last: u16 = more[..e_len].parse().ok()?;
            if last > verse && last - verse <= 10 {
                end = Some(last);
                used += dash.len_utf8() + e_len;
            }
        }
    }
    // A reference ends at a space or the line's end, never inside a word or number.
    let next = rest[used..].chars().next();
    if next.is_some_and(|c| !c.is_whitespace()) {
        return None;
    }
    (chapter > 0 && verse > 0).then_some((chapter, verse, end, book, taken + used))
}

/// The text of a unit cut into words and candidate verse numbers.
fn pieces(text: &str, language: Option<ReferenceLanguage>) -> Vec<Piece> {
    let mut out: Vec<Piece> = Vec::new();
    let mut from = 0usize;
    if let Some((chapter, verse, end, book, taken)) = line_reference(text, language) {
        out.push(Piece::Marker(Marker {
            value: verse,
            end,
            chapter: Some(chapter),
            book,
            strength: Strength::Strong,
            unit_start: true,
            raw: text[..taken].to_string(),
            glued: false,
            joined: false,
        }));
        from = taken;
    }
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut i = chars
        .iter()
        .position(|(at, _)| *at >= from)
        .unwrap_or(chars.len());
    let mut text_start = from;
    while i < chars.len() {
        let (at, c) = chars[i];
        let is_digit = c.is_ascii_digit();
        let raised = superscript_digit(c).is_some();
        if !(is_digit || raised) {
            i += 1;
            continue;
        }
        // The whole run of digits of one kind.
        let mut j = i;
        while j < chars.len()
            && (if raised {
                superscript_digit(chars[j].1).is_some()
            } else {
                chars[j].1.is_ascii_digit()
            })
        {
            j += 1;
        }
        let run: String = chars[i..j].iter().map(|(_, c)| *c).collect();
        let next_index = j;
        let candidate = (|| -> Option<(Marker, usize)> {
            if run.chars().count() > 3 {
                return None;
            }
            let value: u16 = if raised {
                run.chars()
                    .filter_map(superscript_digit)
                    .fold(0, |v, d| v * 10 + d) as u16
            } else {
                run.parse().ok()?
            };
            if value == 0 {
                return None;
            }
            // What comes before: nothing (the start), a space, punctuation, or a word.
            let before = &text[..at];
            let prev = before.chars().last();
            let opened = prev.is_some_and(|p| matches!(p, '(' | '['));
            let prev_solid = before.trim_end().chars().last();
            if prev.is_some_and(|p| p.is_alphanumeric()) {
                // Glued to a word: "God1", "v2", a raised footnote mark after a word.
                return None;
            }
            let number_before = prev
                .filter(|p| matches!(p, ':' | '.' | ',' | '-' | '–'))
                .is_some_and(|p| {
                    before[..before.len() - p.len_utf8()]
                        .chars()
                        .last()
                        .is_some_and(|c| c.is_ascii_digit())
                });
            if number_before {
                // The second number of a reference or range: "3:16", "3,16", "4-5".
                return None;
            }
            let at_start = before
                .trim_start_matches(['(', '[', '“', '"', '‘', '\'', '¶'])
                .trim()
                .is_empty();
            let strength = if at_start || raised || prev_solid.is_some_and(ends_sentence) || opened
            {
                Strength::Strong
            } else if prev.is_some_and(char::is_whitespace) {
                Strength::Medium
            } else {
                return None;
            };
            if strength == Strength::Medium {
                // "chapter 3", "John 3", "page 12": a reference or a count, not a verse.
                let word = before
                    .trim_end()
                    .rsplit(|c: char| c.is_whitespace())
                    .next()
                    .unwrap_or("");
                let w = key(word.trim_matches(|c: char| !c.is_alphanumeric()));
                if BEFORE_NUMBER.contains(&w.as_str()) {
                    return None;
                }
                // After a word that names a book ("John 3:16", but also the
                // place "Dan 15 und teilte sich"): a reference only when a
                // chapter-and-verse form follows; otherwise still possibly
                // the very next verse (Medium: never anything else).
                if book_named(word, language).is_some() {
                    let after: String = chars[next_index..].iter().take(3).map(|(_, c)| *c).collect();
                    let mut a = after.chars();
                    if matches!(a.next(), Some(':' | ',' | '.')) && a.next().is_some_and(|c| c.is_ascii_digit()) {
                        return None;
                    }
                }
            }
            // What comes after.
            let mut k = next_index;
            let mut end = None;
            if !raised {
                if let Some(&(_, dash)) = chars.get(k).filter(|(_, d)| matches!(d, '-' | '–' | '—'))
                {
                    let _ = dash;
                    let mut m = k + 1;
                    while m < chars.len() && chars[m].1.is_ascii_digit() {
                        m += 1;
                    }
                    if m > k + 1 && m - (k + 1) <= 3 {
                        let last: u16 = chars[k + 1..m]
                            .iter()
                            .map(|(_, c)| *c)
                            .collect::<String>()
                            .parse()
                            .ok()?;
                        if last > value && last - value <= 10 {
                            end = Some(last);
                            k = m;
                        }
                    }
                }
            }
            if opened {
                // "(2)" or "[2]": the bracket must close right after.
                match chars.get(k) {
                    Some((_, ')' | ']')) => k += 1,
                    _ => return None,
                }
            }
            match chars.get(k).map(|(_, c)| *c) {
                None => {}
                Some(c) if c.is_whitespace() => {}
                Some(':' | ',') if chars.get(k + 1).is_some_and(|(_, d)| d.is_ascii_digit()) => {
                    return None
                }
                Some('.') if chars.get(k + 1).is_some_and(|(_, d)| d.is_ascii_digit()) => {
                    return None
                }
                // "1. In the beginning": a numbered verse.
                Some('.') if chars.get(k + 1).is_none_or(|(_, d)| d.is_whitespace()) => k += 1,
                Some('%' | '°' | '/' | ':' | ',' | '.' | '€' | '$' | '£') => return None,
                // Korean units: "3장", "4절", "23편" are references, not verse numbers.
                Some('장' | '절' | '편') => return None,
                Some(c) if c.is_alphabetic() => {
                    // "5th", "3rd", "10km": a Latin small letter glued on is a word.
                    if c.is_ascii_lowercase() {
                        return None;
                    }
                    if strength == Strength::Medium && !c.is_uppercase() && c.is_ascii() {
                        return None;
                    }
                }
                Some('“' | '"' | '‘' | '\'' | '«' | '(' | '[' | '—' | '–' | '¿' | '¡') =>
                    {}
                Some(_) => return None,
            }
            // A bracketed number keeps its opening bracket out of the text.
            Some((
                Marker {
                    value,
                    end,
                    chapter: None,
                    book: None,
                    strength,
                    unit_start: at_start,
                    raw: String::new(),
                    glued: chars.get(k).is_some_and(|(_, c)| !c.is_whitespace()),
                    joined: !at_start && {
                        let lead = if opened {
                            &before[..before.len() - 1]
                        } else {
                            before
                        };
                        lead.chars().last().is_some_and(|c| !c.is_whitespace())
                    },
                },
                k,
            ))
        })();
        match candidate {
            Some((mut marker, after)) => {
                let cut = if text[..at].ends_with(['(', '[']) {
                    at - 1
                } else {
                    at
                };
                let words = text[text_start..cut].trim();
                if !words.is_empty() {
                    out.push(Piece::Text(words.to_string()));
                }
                let until = chars.get(after).map_or(text.len(), |(a, _)| *a);
                marker.raw = text[cut..until].trim().to_string();
                out.push(Piece::Marker(marker));
                text_start = until;
                i = after;
            }
            None => i = next_index,
        }
    }
    let words = text[text_start.min(text.len())..].trim();
    if !words.is_empty() {
        out.push(Piece::Text(words.to_string()));
    }
    out
}

// --- the event stream ---------------------------------------------------------------

#[derive(Debug, Clone)]
enum Event {
    /// A book heading; `entry` is its place in the found-books list.
    Book {
        entry: usize,
        chapter: Option<u16>,
    },
    Chapter(u16),
    /// A heading that names no book or chapter.
    Other {
        notes: bool,
        text: String,
    },
    Text(String),
    Marker(Marker),
}

#[derive(Debug, Clone)]
struct Item {
    event: Event,
    unit: usize,
}

/// Lines repeated at the top or bottom of most pages (running heads, page
/// numbers): the unit indices to take out.
fn page_furniture(units: &[Unit]) -> HashSet<usize> {
    let mut out = HashSet::new();
    // Units by (document, page), in order.
    let mut pages: BTreeMap<(u16, u32), Vec<usize>> = BTreeMap::new();
    for (i, unit) in units.iter().enumerate() {
        if let (false, Some(page)) = (unit.heading, unit.place.page) {
            pages
                .entry((unit.place.document, page))
                .or_default()
                .push(i);
        }
    }
    if pages.len() < 3 {
        return out;
    }
    let shape = |text: &str| -> String {
        key(text)
            .chars()
            .map(|c| if c.is_ascii_digit() { '#' } else { c })
            .collect::<String>()
    };
    let mut counts: HashMap<String, usize> = HashMap::new();
    for lines in pages.values() {
        let mut seen = HashSet::new();
        for &i in lines.iter().take(2).chain(lines.iter().rev().take(2)) {
            if !head_like(&units[i].text) {
                continue;
            }
            let s = shape(&units[i].text);
            if seen.insert(s.clone()) {
                *counts.entry(s).or_default() += 1;
            }
        }
    }
    let needed = (pages.len() * 3 / 10).max(3);
    // A number alone as a page's first or last line counts only when it runs
    // with the pages (page 7 of the PDF reads "5", page 8 "6" …).
    let mut offsets: HashMap<i64, usize> = HashMap::new();
    for ((_, page), lines) in &pages {
        for &i in lines.iter().take(1).chain(lines.iter().rev().take(1)) {
            if let Ok(n) = units[i]
                .text
                .trim()
                .trim_matches(['-', '–', ' '])
                .parse::<i64>()
            {
                *offsets.entry(i64::from(*page) - n).or_default() += 1;
            }
        }
    }
    let page_offsets: HashSet<i64> = offsets
        .into_iter()
        .filter(|(_, n)| *n >= needed)
        .map(|(o, _)| o)
        .collect();
    // Running heads that name the passage on the page ("249 Josua 2:1",
    // "Matthew 1:25 968", "GENESIS 3:4–4:2"): their book and numbers change on
    // every page, so they are counted by their form. Only a page's first or
    // last line, and only when that form runs through the pages.
    let reference_heads = pages
        .values()
        .filter(|lines| {
            lines
                .iter()
                .take(1)
                .chain(lines.iter().rev().take(1))
                .any(|&i| reference_head(&units[i].text))
        })
        .count();
    let reference_pages = reference_heads >= needed;
    for ((_, page), lines) in &pages {
        if reference_pages {
            for &i in lines.iter().take(1).chain(lines.iter().rev().take(1)) {
                if reference_head(&units[i].text) {
                    out.insert(i);
                }
            }
        }
        let edge: Vec<usize> = lines
            .iter()
            .take(2)
            .chain(lines.iter().rev().take(2))
            .copied()
            .collect();
        for i in edge {
            let text = units[i].text.trim();
            let numeric = text.trim_matches(['-', '–', ' ']).parse::<i64>().ok();
            if let Some(n) = numeric {
                if page_offsets.contains(&(i64::from(*page) - n)) {
                    out.insert(i);
                }
                continue;
            }
            // A running head: repeated on many pages (its numbers may change).
            if head_like(text) && counts.get(&shape(text)).is_some_and(|n| *n >= needed) {
                out.insert(i);
            }
        }
    }
    out
}

/// Footnotes printed at the foot of a page ("† 1:1 Messiah (Hebrew) …",
/// "* 2:6 Micah 5:2"): from the first line that starts with a note sign and a
/// chapter and verse to the end of that page. They are notes, not verses.
fn page_footnotes(units: &[Unit]) -> HashSet<usize> {
    use std::sync::OnceLock;
    static NOTE: OnceLock<regex::Regex> = OnceLock::new();
    let note = NOTE.get_or_init(|| {
        regex::Regex::new(r"^[†‡§¶*]\s*\d{1,3}[:,.]\d{1,3}(?:\s|$)").expect("valid pattern")
    });
    let mut out = HashSet::new();
    let mut in_notes: Option<(u16, u32)> = None;
    for (i, unit) in units.iter().enumerate() {
        let Some(page) = unit.place.page else {
            in_notes = None;
            continue;
        };
        let here = (unit.place.document, page);
        if in_notes != Some(here) {
            in_notes = None;
            if !unit.heading && note.is_match(unit.text.trim_start()) {
                in_notes = Some(here);
            }
        }
        if in_notes.is_some() {
            out.insert(i);
        }
    }
    out
}

/// Note references printed after a word ("Jesus,§ who", "behold,† an"): they
/// point to footnotes and are not words of the verse, so they are left out of
/// it, as the notes themselves are. Only these signs, and only straight after
/// a letter or punctuation; nothing else is changed. A space left before the
/// next word stays as it was.
const NOTE_SIGNS: [char; 5] = ['†', '‡', '§', '¶', '*'];

/// The "page" of the notes of a document without pages.
const WHOLE_DOCUMENT: u32 = u32::MAX;

/// A note at the foot of a page: the sign it begins with, and the chapter
/// and verse it names, if it names one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NoteSign {
    sign: char,
    verse: Option<(u16, u16)>,
}

/// Every note on a line of notes: its sign and the verse it names (several
/// short notes can share a line: "† 2:4 … ‡ 2:12 or, … § 2:18 …"). A line
/// that begins with a sign naming no verse gives that sign alone.
fn note_signs_in(text: &str) -> Vec<NoteSign> {
    use std::sync::OnceLock;
    static NOTE: OnceLock<regex::Regex> = OnceLock::new();
    let form = NOTE.get_or_init(|| {
        regex::Regex::new(r"(?:^|\s)([†‡§¶*])\s*(\d{1,3})[:,.](\d{1,3})").expect("valid pattern")
    });
    let mut out: Vec<NoteSign> = Vec::new();
    for c in form.captures_iter(text.trim_start()) {
        let (Some(sign), Some(chapter), Some(verse)) = (c.get(1), c.get(2), c.get(3)) else {
            continue;
        };
        let (Some(sign), Ok(chapter)) = (
            sign.as_str().chars().next(),
            chapter.as_str().parse::<u16>(),
        ) else {
            continue;
        };
        // "11:351Kings" (the space before "1 Kings" lost in the text): every
        // reading of the digits that can be a verse number.
        let digits = verse.as_str();
        for len in (1..=digits.len()).rev() {
            if let Ok(v) = digits[..len].parse::<u16>() {
                if (1..=176).contains(&v) {
                    out.push(NoteSign {
                        sign,
                        verse: Some((chapter, v)),
                    });
                }
            }
        }
    }
    if out.is_empty() {
        out.extend(note_sign(text));
    }
    out
}

/// Notes labelled with letters on a line of notes ("c 13:6 Ge 36:7. d 13:7 …"):
/// each letter and the verse it names.
fn note_letters_in(text: &str) -> Vec<NoteLetter> {
    use std::sync::OnceLock;
    static NOTE: OnceLock<regex::Regex> = OnceLock::new();
    let form = NOTE.get_or_init(|| {
        regex::Regex::new(r"(?:^|\s)(\p{Ll}{1,2})\s+(\d{1,3})[:,.](\d{1,3})")
            .expect("valid pattern")
    });
    form.captures_iter(text)
        .filter_map(|c| {
            Some((
                c.get(1)?.as_str().to_string(),
                c.get(2)?.as_str().parse().ok()?,
                c.get(3)?.as_str().parse().ok()?,
            ))
        })
        .collect()
}

/// A text with its raised-letter marks taken off (the letters kept).
fn unmark_raised(text: &str) -> String {
    text.chars()
        .filter(|c| *c != RAISED_OPEN && *c != RAISED_CLOSE)
        .collect()
}

/// Verse words with raised letters: those in `known` (note references) taken
/// out, the others kept as written. The second value tells whether any were
/// kept (unexplained).
fn resolve_raised(text: &str, known: &[String]) -> (String, bool) {
    let mut out = String::with_capacity(text.len());
    let mut unexplained = false;
    let mut rest = text;
    while let Some(open) = rest.find(RAISED_OPEN) {
        out.push_str(&rest[..open]);
        let after = &rest[open + RAISED_OPEN.len_utf8()..];
        let Some(close) = after.find(RAISED_CLOSE) else {
            out.push_str(after);
            rest = "";
            break;
        };
        let letters = &after[..close];
        if known.iter().any(|k| k == letters) {
            // "Lot,ᶜ qui" → "Lot, qui"; " ᶜEt" → " Et".
        } else {
            unexplained = true;
            out.push_str(letters);
        }
        rest = &after[close + RAISED_CLOSE.len_utf8()..];
    }
    out.push_str(rest);
    (out, unexplained)
}

/// The sign and verse a note line begins with ("§ 2:18 Or …", "*9:44 …").
fn note_sign(text: &str) -> Option<NoteSign> {
    let text = text.trim_start();
    let sign = text.chars().next().filter(|c| NOTE_SIGNS.contains(c))?;
    let rest = text[sign.len_utf8()..].trim_start();
    let digits = |s: &str| -> Option<(u16, usize)> {
        let n: String = s.chars().take_while(char::is_ascii_digit).collect();
        if n.is_empty() || n.len() > 3 {
            return None;
        }
        Some((n.parse().ok()?, n.len()))
    };
    let verse = digits(rest).and_then(|(c, len)| {
        let after = &rest[len..];
        let sep = after
            .chars()
            .next()
            .filter(|c| matches!(c, ':' | ',' | '.'))?;
        digits(&after[sep.len_utf8()..]).map(|(v, _)| (c, v))
    });
    Some(NoteSign { sign, verse })
}

#[cfg(test)]
fn without_note_callers(text: &str) -> std::borrow::Cow<'_, str> {
    without_signs(text, &NOTE_SIGNS)
}

/// [`without_note_callers`] for only the given signs.
fn without_signs<'a>(text: &'a str, signs: &[char]) -> std::borrow::Cow<'a, str> {
    if signs.is_empty() || !text.contains(signs) {
        return std::borrow::Cow::Borrowed(text);
    }
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        if !NOTE_SIGNS.contains(&chars[i]) {
            out.push(chars[i]);
            i += 1;
            continue;
        }
        // A run of signs ("*‡"), as one: raised against the word before it,
        // set apart as a word of its own, or at the very start.
        let mut end = i;
        while end < chars.len() && NOTE_SIGNS.contains(&chars[end]) {
            end += 1;
        }
        let previous = i.checked_sub(1).map(|p| chars[p]);
        let next = chars.get(end).copied();
        let raised = previous.is_some_and(|p| !p.is_whitespace());
        let alone =
            previous.is_none_or(char::is_whitespace) && next.is_none_or(char::is_whitespace);
        let start = previous.is_none();
        if raised || alone || start {
            let kept: String = chars[i..end]
                .iter()
                .filter(|c| !signs.contains(c))
                .collect();
            out.push_str(&kept);
            if kept.is_empty() && alone && !start {
                // "cause † will": one space stays, not two.
                while out.ends_with(' ') && chars.get(end).is_some_and(|c| c.is_whitespace()) {
                    out.pop();
                }
            }
            if kept.is_empty() && start {
                end += chars[end..]
                    .iter()
                    .take_while(|c| c.is_whitespace())
                    .count();
            }
        } else {
            out.extend(&chars[i..end]);
        }
        i = end;
    }
    let trimmed = out.trim_end().len();
    out.truncate(trimmed);
    std::borrow::Cow::Owned(out)
}

/// Whether a line has the form of a running head naming a passage: an
/// optional page number, at most four words, a chapter and verse (or a range),
/// an optional page number and optionally a second such reference — nothing
/// else ("249 Josua 2:1", "Matthew 1:25 968", "Josua 4:1 252 Josua 4:23").
fn reference_head(text: &str) -> bool {
    use std::sync::OnceLock;
    static FORM: OnceLock<regex::Regex> = OnceLock::new();
    let form = FORM.get_or_init(|| {
        let reference = r"(?:\d\.?\s?)?[\p{L}][\p{L}'’.\-]*(?:\s+[\p{L}][\p{L}'’.\-]*){0,3}\s+\d{1,3}[:,.]\d{1,3}(?:\s*[-–—]\s*(?:\d{1,3}[:,.])?\d{1,3})?";
        regex::Regex::new(&format!(
            r"^(?:\d{{1,4}}\s+)?{reference}(?:\s+\d{{1,4}})?(?:\s+{reference})?(?:\s+\d{{1,4}})?$"
        ))
        .expect("valid pattern")
    });
    // A book of one chapter: verse numbers alone, the same name twice and the
    // page number between ("Obadja 1 972 Obadja 19", "Judas 21 1320 Judas 25").
    static SINGLE: OnceLock<regex::Regex> = OnceLock::new();
    let single = SINGLE.get_or_init(|| {
        let name = r"((?:\d\.?\s?)?[\p{L}][\p{L}'’.\-]*(?:\s+[\p{L}][\p{L}'’.\-]*){0,3})";
        regex::Regex::new(&format!(
            r"^{name}\s+\d{{1,3}}\s+\d{{1,4}}\s+{name}\s+\d{{1,3}}$"
        ))
        .expect("valid pattern")
    });
    let text = text.trim();
    if text.chars().count() > 80 {
        return false;
    }
    form.is_match(text)
        || single
            .captures(text)
            .is_some_and(|c| c.get(1).map(|m| m.as_str()) == c.get(2).map(|m| m.as_str()))
}

/// Lines of one or two words, letters only, starting with a capital and
/// without punctuation, that stand as whole lines three times or more: in a
/// typeset Bible perhaps a speaker's name ("Friends", "Beloved") or a heading
/// set in another style. Found between two sentences of a verse, such a line
/// cannot be told from the verse's words without its style, so the chapter
/// is refused (see `Reader::text`); nothing is left out on a guess.
fn repeated_labels(units: &[Unit], furniture: &HashSet<usize>) -> HashSet<String> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for (i, unit) in units.iter().enumerate() {
        if unit.heading || furniture.contains(&i) {
            continue;
        }
        let text = clean(&unit.text);
        let text = text.trim();
        let words = text.split_whitespace().count();
        let shape = (1..=2).contains(&words)
            && text.chars().count() <= 24
            && text.chars().next().is_some_and(|c| c.is_uppercase())
            && text.chars().all(|c| c.is_alphabetic() || c == ' ');
        if shape {
            *counts.entry(text.to_string()).or_default() += 1;
        }
    }
    counts
        .into_iter()
        .filter(|(_, n)| *n >= 3)
        .map(|(t, _)| t)
        .collect()
}

/// Whether a line begins with a conjunction that follows a suspended hyphen
/// ("Lob- und Dankopfer", "Vor- oder Nachteil"): the hyphen
/// at the end of the line before is the word's own, not a line break.
fn suspended_hyphen(words: &str) -> bool {
    // German only, and only conjunctions that no word ends in at a correct
    // hyphenation point ("Lob- und Dankopfer"). English "-or", "-nor", "-to"
    // and Romance "-et", "-o", "-e" are ordinary word endings ("hon- or",
    // "gover- nor"): those line ends are joined as one word.
    const JOINING: &[&str] = &["und", "oder", "sowie"];
    let first = words
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_end_matches([',', ';']);
    JOINING.contains(&first)
}

/// A verse: book ordinal, chapter, verse.
type VerseKey = (u8, u16, u16);
/// A passage a running head names: book name as printed, chapter, verse.
type HeadReference = (String, u16, u16);
/// A note's letter and the chapter and verse it names ("c 13:6").
type NoteLetter = (String, u16, u16);
/// What a page's running head says: its book, first passage, last chapter
/// and verse, and its text.
type PageHead = (&'static Book, (u16, u16), u16, u16, String);

/// The passages a running head names ("Genesis 24:48 27 Genesis 25:11",
/// "Obadja 1 972 Obadja 19"): its first and last reference, as book name,
/// chapter and verse (a book of one chapter: chapter 1).
fn head_references(text: &str) -> Option<(HeadReference, HeadReference)> {
    use std::sync::OnceLock;
    static REF: OnceLock<regex::Regex> = OnceLock::new();
    let form = REF.get_or_init(|| {
        regex::Regex::new(r"(?:^|\s)((?:\d\.?\s?)?[\p{L}][\p{L}'’.\-]*(?:\s+[\p{L}][\p{L}'’.\-]*){0,3})\s+(\d{1,3})(?:[:,.](\d{1,3}))?")
            .expect("valid pattern")
    });
    let found: Vec<HeadReference> = form
        .captures_iter(text.trim())
        .filter_map(|c| {
            let name = c.get(1)?.as_str().trim().to_string();
            let first: u16 = c.get(2)?.as_str().parse().ok()?;
            Some(match c.get(3) {
                Some(v) => (name, first, v.as_str().parse().ok()?),
                None => (name, 1, first),
            })
        })
        .collect();
    Some((found.first()?.clone(), found.last()?.clone()))
}

/// The running heads of typeset Bibles name the first and last verse of
/// their page. A page whose last verse begun on it is in another chapter
/// than its head names shows that the chapters were counted wrongly there
/// (a chapter number missed, a chapter run into the next): the chapters on
/// that page are refused. Only the document's own heads are used; a page
/// without a head that can be read is not checked.
fn check_running_heads(
    units: &[Unit],
    furniture: &HashSet<usize>,
    language: Option<ReferenceLanguage>,
    out: &mut Recognition,
) {
    // The first running head of each page, by (document, page).
    let mut heads: BTreeMap<(u16, u32), PageHead> = BTreeMap::new();
    for &i in furniture {
        let unit = &units[i];
        let Some(page) = unit.place.page else {
            continue;
        };
        if heads.contains_key(&(unit.place.document, page)) || !reference_head(&unit.text) {
            continue;
        }
        let Some(((first_name, fc, fv), (name, chapter, verse))) = head_references(&unit.text)
        else {
            continue;
        };
        // A head whose last passage comes before its first is no evidence.
        if first_name == name && (chapter, verse) < (fc, fv) {
            continue;
        }
        if let Some(book) = book_named(&name, language) {
            heads.insert(
                (unit.place.document, page),
                (book, (fc, fv), chapter, verse, unit.text.trim().to_string()),
            );
        }
    }
    if heads.is_empty() {
        return;
    }
    // The first and last verse begun on each page.
    let mut begun: BTreeMap<(u16, u32), (VerseKey, VerseKey)> = BTreeMap::new();
    for (&(ordinal, chapter, verse), origin) in &out.origins {
        let Some(page) = origin.start.page else {
            continue;
        };
        let here = (ordinal, chapter, verse);
        let entry = begun
            .entry((origin.start.document, page))
            .or_insert((here, here));
        entry.0 = entry.0.min(here);
        entry.1 = entry.1.max(here);
    }
    let begins_on = |key: &(u16, u32), at: (u8, u16, u16)| {
        out.origins
            .get(&at)
            .is_some_and(|o| o.start.document == key.0 && o.start.page == Some(key.1))
    };
    let mut flagged: HashSet<(u8, u16)> = HashSet::new();
    for (key, (book, first, chapter, verse, text)) in &heads {
        let Some(&(first_found, (ordinal, found_chapter, found_verse))) = begun.get(key) else {
            continue;
        };
        let agrees = if ordinal != book.ordinal {
            // A running head of two passages naming another book: the page
            // must hold that book's verses too (a page where one book ends
            // and the next begins); otherwise a heading was read as a book
            // that is not there. One passage alone may be a cross-reference
            // ("Psalms 110:4") and proves nothing.
            first_found.0 == book.ordinal || (first.0, first.1) == (*chapter, *verse)
        } else {
            // Some typesetting keeps an earlier chapter in the head ("3:7 …
            // 3:9" over 3:7–4:9, or "49:8 … 49:26" over 50:8–50:26):
            // accepted only when the next chapter's verse 1 is read on this
            // page, or the chapter before it was read complete — a chapter
            // shifted by one would leave it short.
            let previous_complete = || {
                let Some(counted) = canon::by_ordinal(ordinal)
                    .and_then(|b| canon::verse_count(b, found_chapter - 1))
                else {
                    return false;
                };
                out.content
                    .books
                    .get(&ordinal)
                    .and_then(|b| b.chapters.get(&(found_chapter - 1)))
                    .is_some_and(|c| (1..=counted).all(|v| c.verses.contains_key(&v)))
            };
            let kept_chapter = first.0 == *chapter
                && found_chapter == chapter + 1
                && found_verse == *verse
                && (begins_on(key, (ordinal, found_chapter, 1)) || previous_complete());
            // (The head's first passage is not compared: eBible's heads get
            // it wrong at chapter starts, "John 7:10" over John 8.)
            (found_chapter == *chapter && found_verse.abs_diff(*verse) <= 1) || kept_chapter
        };
        if agrees || !flagged.insert((ordinal, found_chapter)) {
            continue;
        }
        let Some(found_book) = canon::by_ordinal(ordinal) else {
            continue;
        };
        out.content.issue(
            Severity::Error,
            Some(found_book),
            Some(found_chapter),
            UserMessage::new(
                "bibleImport.documentRunningHead",
                format!(
                    "{} {found_chapter}: page {} ends with verse {found_chapter}:{found_verse}, but its running head ({text}) says {chapter}:{verse}. This chapter is not used.",
                    found_book.name, key.1
                ),
            )
            .with("bookCode", found_book.code)
            .with("chapter", found_chapter)
            .with("page", key.1)
            .with("found", format!("{found_chapter}:{found_verse}"))
            .with("head", text.as_str())
            .with("named", format!("{chapter}:{verse}")),
        );
    }
}

/// Whether a line could be a running head ("GENESIS 1", "THE BOOK OF RUTH",
/// "2 Kings 3:4–4:2"): short, and not ending as a sentence does. A line of
/// verse text never counts, however alike verse lines look.
fn head_like(text: &str) -> bool {
    let text = text.trim();
    text.split_whitespace().count() <= 6
        && text.chars().count() <= 50
        && !text.ends_with(['.', '!', '?', ';', ',', '׃', '。', '"', '”', '’'])
}

/// The shape of a line that can be a book's title: short, no sentence end
/// or closing quotation mark, and — in scripts with capitals — starting
/// with one.
fn book_line_shape(text: &str) -> bool {
    let text = text.trim();
    // "GENESIS." — a title in capitals may end with a full stop.
    let capitals = text.chars().filter(|c| c.is_alphabetic()).count() > 1
        && !text.chars().any(|c| c.is_lowercase());
    let text = if capitals {
        text.trim_end_matches('.')
    } else {
        text
    };
    // "Das 1. Buch Mose (Genesis)": a name in brackets may end the title.
    let bracketed = text.ends_with(')') && text.contains('(');
    text.chars().count() <= 60
        && (bracketed
            || !text.ends_with([
                '.', ',', ';', ':', '!', '?', '׃', '。', '"', '”', '’', '\'', ')',
            ]))
        && text
            .chars()
            .find(|c| c.is_alphabetic())
            .is_some_and(|c| !c.is_lowercase())
}

struct Builder<'a> {
    units: &'a [Unit],
    language: Option<ReferenceLanguage>,
    items: Vec<Item>,
    books: Vec<FoundBook>,
    last_detected: Option<&'static str>,
}

impl Builder<'_> {
    fn found_book(&mut self, heading: &str, detected: Option<&'static Book>, unit: usize) -> usize {
        self.books.push(FoundBook {
            index: self.books.len(),
            heading: unmark_raised(heading),
            detected: detected.map(|b| b.code),
            book: detected.map(|b| b.code),
            left_out: false,
            chapters: 0,
            verses: 0,
            place: self.units[unit].place,
        });
        self.last_detected = detected.map(|b| b.code);
        self.books.len() - 1
    }

    /// A heading (or a short line on its own) right before "Chapter 1"
    /// that names no book: perhaps a book's title the dictionaries do not
    /// know ("The First Book of Moses"). Offered for correction.
    fn untitled_book(&self) -> Option<(String, usize)> {
        let prev = self.items.last()?;
        let before = self.items.len().checked_sub(2).map(|i| &self.items[i]);
        match &prev.event {
            Event::Other { notes: false, text } => Some((text.clone(), prev.unit)),
            // A whole line of its own: no verse number, no sentence end, short.
            Event::Text(text)
                if before.is_none_or(|b| b.unit != prev.unit)
                    && clean(&self.units[prev.unit].text).trim() == text
                    && text.chars().count() <= 60
                    && book_line_shape(text) =>
            {
                Some((text.clone(), prev.unit))
            }
            _ => None,
        }
    }

    /// Whether a PDF or text line naming a book opens it: a book begins at
    /// its chapter 1 or verse 1, so the next line (after a subtitle or two)
    /// must be "Chapter 1" or start with the number 1. A book's name wrapped
    /// onto its own line inside a verse ("Ruth.", "Acts") is followed by
    /// the next verse's number instead, and stays text. The same book named
    /// again and followed by 1 begins twice (reported).
    fn opens_chapter(&self, index: usize, skip: &HashSet<usize>) -> bool {
        // A title set apart as one (a heading style, large type, capitals)
        // may be followed by an introduction of any length before its
        // chapter 1 (pages of it); any other line naming a book must be
        // followed by it within three lines (a subtitle or two between).
        let title = clean(&self.units[index].text);
        let capitals = title.chars().filter(|c| c.is_alphabetic()).count() > 2
            && !title.chars().any(char::is_lowercase)
            && title.chars().any(char::is_uppercase);
        let strong = self.units[index].heading || self.units[index].large || capitals;
        let limit = if strong { 3000 } else { 3 };
        let lines = self
            .units
            .iter()
            .enumerate()
            .skip(index + 1)
            .filter(|(i, _)| !skip.contains(i))
            .map(|(_, u)| unmark_raised(&clean(&u.text)).trim().to_string())
            .filter(|t| !t.is_empty())
            .take(limit);
        for line in lines {
            if strong && reference_line(&line, self.language) {
                continue;
            }
            if let Some(heading) = read_heading(&line, self.language) {
                return matches!(heading, Heading::Chapter(1) | Heading::BookChapter(_, 1));
            }
            let digits: String = line
                .chars()
                .map_while(|c| {
                    superscript_digit(c)
                        .and_then(|d| char::from_digit(d, 10))
                        .or(c.is_ascii_digit().then_some(c))
                })
                .collect();
            if !digits.is_empty() {
                // A verse's number: alone on its line, or followed by a space
                // and the verse's first word ("1 Au commencement"). "4:1-16)"
                // or "1 à 11" in an introduction is not one.
                let rest = line[line
                    .char_indices()
                    .nth(digits.chars().count())
                    .map_or(line.len(), |(i, _)| i)..]
                    .to_string();
                // Words that join two numbers ("1 à 11", "1 to 3"): a range in
                // an introduction, not a verse.
                const RANGE: &[&str] = &[
                    "à", "a", "au", "to", "bis", "al", "até", "et", "and", "und", "y", "e", "-",
                    "–",
                ];
                let first_word = rest.split_whitespace().next().unwrap_or("");
                let verse_like = rest.is_empty()
                    || (rest.starts_with(char::is_whitespace)
                        && first_word.chars().next().is_some_and(char::is_alphabetic)
                        // Case matters: "1 und 2" is a range, "1 Und der HERR" a verse.
                        && !RANGE.contains(&first_word))
                    || rest.starts_with(|c: char| c.is_alphabetic() && !c.is_lowercase());
                if verse_like {
                    return digits == "1";
                }
                if !strong {
                    return false;
                }
                continue;
            }
            if !strong && !head_like(&line) {
                return false;
            }
        }
        false
    }

    /// A book's title printed over two lines ("The Good News According to",
    /// then "Matthew"): the first line, a whole short line standing after a
    /// verse that ended its sentence, is part of the title, not of that verse.
    /// Taken back out of the text and returned.
    fn title_start(&mut self, marked: bool) -> Option<String> {
        if marked {
            return None;
        }
        // Up to three whole lines before the name ("The Third Book of Moses,",
        // "Commonly Called", then "Leviticus"), each on a line of its own,
        // after a verse that ended its sentence.
        let n = self.items.len();
        let mut taken = 0;
        while taken < 3 && taken < n {
            let item = &self.items[n - 1 - taken];
            let Event::Text(line) = &item.event else {
                break;
            };
            let whole = clean(&self.units[item.unit].text).trim() == line.trim();
            let alone = n - 1 - taken == 0 || self.items[n - 2 - taken].unit != item.unit;
            let shape = line.trim().chars().count() <= 60
                && line.trim().chars().next().is_some_and(|c| c.is_uppercase())
                && !line
                    .trim_end()
                    .ends_with(['.', '!', '?', ';', ':', '”', '’', '"']);
            if !whole || !alone || !shape {
                break;
            }
            taken += 1;
        }
        if taken == 0 || taken == n {
            return None;
        }
        let ended = matches!(&self.items[n - 1 - taken].event, Event::Text(t)
            if t.trim_end().ends_with(['.', '!', '?', '”', '’', '"', ')']));
        if !ended {
            return None;
        }
        let lines: Vec<String> = self.items[n - taken..]
            .iter()
            .filter_map(|i| match &i.event {
                // Never with the raised-letter marks: this is a title.
                Event::Text(t) => Some(unmark_raised(t.trim()).trim().to_string()),
                _ => None,
            })
            .collect();
        self.items.truncate(n - taken);
        Some(lines.join(" "))
    }

    fn push(&mut self, event: Event, unit: usize) {
        if matches!(event, Event::Chapter(_)) {
            self.heading_before_chapter();
        }
        self.items.push(Item { event, unit });
    }

    /// A section heading just before a chapter heading ("The Song of
    /// Deborah", then "Chapter 5"): a short whole line of its own, beginning
    /// in capitals and not ending as a sentence does, after a verse that
    /// ended its sentence. It is a heading, never words of the verse before.
    fn heading_before_chapter(&mut self) {
        let n = self.items.len();
        if n < 2 {
            return;
        }
        let last = &self.items[n - 1];
        let Event::Text(line) = &last.event else {
            return;
        };
        let line = line.trim();
        let whole =
            unmark_raised(&clean(&self.units[last.unit].text)).trim() == unmark_raised(line);
        let alone = self.items[n - 2].unit != last.unit;
        let shape = line.chars().count() <= 60
            && line.chars().next().is_some_and(char::is_uppercase)
            && !line.ends_with(['.', '!', '?', ';', ':', ',', '”', '’', '"']);
        let after_sentence = match &self.items[n - 2].event {
            Event::Text(t) => t.trim_end().ends_with(['.', '!', '?', '”', '’', '"', ')']),
            _ => true,
        };
        if whole && alone && shape && after_sentence {
            let text = unmark_raised(line);
            self.items[n - 1].event = Event::Other { notes: false, text };
        }
    }

    /// The found-book entry for this book: a new one when the book changes.
    fn book_entry(&mut self, heading: &str, book: &'static Book, unit: usize) -> usize {
        if self.last_detected == Some(book.code) {
            if let Some(last) = self
                .books
                .iter()
                .rposition(|b| b.detected == Some(book.code))
            {
                return last;
            }
        }
        self.found_book(heading, Some(book), unit)
    }

    fn build(mut self, skip: &HashSet<usize>) -> (Vec<Item>, Vec<FoundBook>) {
        // Stage A, the document's structure, before any verse is read: what
        // is navigation and what are cross-reference lines is never Scripture.
        let navigation = chapter_indexes(self.units, skip);
        let mut skip_all: HashSet<usize> = skip.clone();
        skip_all.extend(navigation.iter().copied());
        let skip = &skip_all;
        let mut joined: HashSet<usize> = HashSet::new();
        // Whether the document is written in lines that each name their book,
        // chapter and verse ("Genesis 1:1 In the beginning …"): many such lines,
        // a tenth of its lines or more.
        let book_lines = {
            let (mut lines, mut named) = (0usize, 0usize);
            for (i, unit) in self.units.iter().enumerate() {
                if skip.contains(&i) || unit.text.trim().is_empty() {
                    continue;
                }
                lines += 1;
                if line_reference(clean(&unit.text).trim(), self.language)
                    .is_some_and(|r| r.3.is_some())
                {
                    named += 1;
                }
            }
            named >= 2 && named * 10 >= lines
        };
        for (index, unit) in self.units.iter().enumerate() {
            if skip.contains(&index) || joined.contains(&index) {
                continue;
            }
            let marked = clean(&unit.text);
            let marked = marked.trim();
            // Raised letters are marked only for verse text (see `Reader::text`);
            // everything else reads the plain line.
            let plain = unmark_raised(marked);
            let text = plain.as_str();
            if text.is_empty() {
                continue;
            }
            // A book's title over two lines whose first line names nothing
            // ("Der zweite Brief des Apostels Paulus an die" / "Korinther"):
            // the two together name the book, and a chapter or verse 1 follows.
            if !unit.heading && read_heading(text, self.language).is_none() && book_line_shape(text)
            {
                if let Some(next) = (index + 1..self.units.len())
                    .find(|i| !skip.contains(i) && !clean(&self.units[*i].text).trim().is_empty())
                {
                    let second = unmark_raised(&clean(&self.units[next].text));
                    let second = second.trim();
                    let same_page = self.units[next].place.page == unit.place.page;
                    let both = format!("{text} {second}");
                    // The second line may begin in lower case ("the Corinthians").
                    let second_ok = second.chars().count() <= 60
                        && !second.ends_with(['.', ',', ';', ':', '!', '?', '"', '”', '’']);
                    if same_page && second_ok && book_line_shape(&both) {
                        if let Some(Heading::Book(book)) = read_heading(&both, self.language) {
                            if read_heading(second, self.language) != Some(Heading::Book(book))
                                && self.opens_chapter(next, skip)
                            {
                                let entry = self.book_entry(&both, book, index);
                                self.push(
                                    Event::Book {
                                        entry,
                                        chapter: None,
                                    },
                                    index,
                                );
                                joined.insert(next);
                                continue;
                            }
                        }
                    }
                }
            }
            // Headings: the document's own, or a line that is nothing but a book or chapter.
            if let Some(heading) = read_heading(text, self.language) {
                // A line that is only a number is a verse number in poetry layout,
                // never a chapter, unless the document marks it as a heading.
                let bare = text.chars().all(|c| c.is_ascii_digit());
                // A PDF or text line naming a book is a heading only when it
                // looks like one and a chapter or verse 1 follows it; a word
                // wrapped onto a line of its own ("Ruth.", "song") stays text.
                let line_book_ok = unit.heading
                    || match heading {
                        Heading::Book(_) => {
                            book_line_shape(text) && self.opens_chapter(index, skip)
                        }
                        Heading::BookChapter(..) => book_line_shape(text),
                        Heading::Chapter(_) => true,
                    };
                if (unit.heading || !bare) && line_book_ok {
                    match heading {
                        Heading::Book(book) => {
                            let title = match self.title_start(unit.heading) {
                                Some(start) => format!("{start} {text}"),
                                None => text.to_string(),
                            };
                            let entry = self.book_entry(&title, book, index);
                            self.push(
                                Event::Book {
                                    entry,
                                    chapter: None,
                                },
                                index,
                            );
                        }
                        Heading::BookChapter(book, chapter) => {
                            let entry = self.book_entry(text, book, index);
                            self.push(
                                Event::Book {
                                    entry,
                                    chapter: Some(chapter),
                                },
                                index,
                            );
                        }
                        Heading::Chapter(n) => {
                            // A heading naming no book right before "Chapter 1": perhaps a
                            // book's title the dictionaries do not know. Offered for correction.
                            if n == 1 {
                                if let Some((title, at)) = self.untitled_book() {
                                    self.items.pop();
                                    let entry = self.found_book(&title, None, at);
                                    self.push(
                                        Event::Book {
                                            entry,
                                            chapter: None,
                                        },
                                        at,
                                    );
                                }
                            }
                            self.push(Event::Chapter(n), index);
                        }
                    }
                    continue;
                }
            }
            // A PDF line set larger than the running text that names no book
            // or chapter ("BOOK 2", "BETH", "The Good News According to") is a
            // heading: never words of a verse. Numbers alone stay numbers
            // (a chapter's number is set large).
            // A line that begins with a number (a verse or chapter marker) is
            // never a heading, nor is a line inside a sentence: the line
            // before ends without a sentence end and the next goes on in
            // lower case.
            let mid_sentence = || {
                let before = (0..index)
                    .rev()
                    .map(|i| {
                        unmark_raised(&clean(&self.units[i].text))
                            .trim()
                            .to_string()
                    })
                    .find(|t| !t.is_empty());
                let after = (index + 1..self.units.len())
                    .map(|i| {
                        unmark_raised(&clean(&self.units[i].text))
                            .trim()
                            .to_string()
                    })
                    .find(|t| !t.is_empty());
                let open = before.is_some_and(|b| {
                    !b.ends_with(['.', '!', '?', ':', ';', '’', '”', '"', '\'', '»', '«'])
                });
                let goes_on = after
                    .and_then(|a| a.chars().next())
                    .is_some_and(char::is_lowercase);
                open && goes_on
            };
            if unit.large && !text.starts_with(|c: char| c.is_ascii_digit()) && !mid_sentence() {
                self.push(
                    Event::Other {
                        notes: false,
                        text: text.to_string(),
                    },
                    index,
                );
                continue;
            }
            // A line of passage references ("V. 18-24: cf. (1 Co 11:7-10.)",
            // "(Ps 103:7. Né 9:9-20.)"): cross-references, not Scripture.
            if reference_line(text, self.language) {
                self.push(
                    Event::Other {
                        notes: false,
                        text: text.to_string(),
                    },
                    index,
                );
                continue;
            }
            if unit.heading {
                // A bare number marked as a heading is a chapter number.
                if let Some(n) =
                    chapter_number(text).filter(|_| text.chars().all(|c| c.is_ascii_digit()))
                {
                    self.push(Event::Chapter(n), index);
                } else {
                    self.push(
                        Event::Other {
                            notes: is_notes_heading(text),
                            text: text.to_string(),
                        },
                        index,
                    );
                }
                continue;
            }
            for piece in pieces(marked, self.language) {
                match piece {
                    Piece::Text(words) => self.push(Event::Text(words), index),
                    Piece::Marker(marker) => {
                        // A line naming book, chapter and verse ("Luke 17:36 and
                        // Acts 15:34, which are …" in a preface) is a verse only in a
                        // document written in such lines; elsewhere it is text.
                        if marker.book.is_some() && !book_lines {
                            self.push(Event::Text(marker.raw.clone()), index);
                            continue;
                        }
                        if let Some(book) = marker.book {
                            let entry = self.book_entry(text, book, index);
                            self.push(
                                Event::Book {
                                    entry,
                                    chapter: None,
                                },
                                index,
                            );
                        }
                        self.push(Event::Marker(marker), index);
                    }
                }
            }
        }
        self.headings_before_verses();
        (self.items, self.books)
    }

    /// Section headings standing between two verses ("Le jardin d’Éden, et
    /// le péché d’Adam", then "25 L’homme …"): one or two short whole lines
    /// of their own, beginning with a capital and not ending as a sentence
    /// does, after a verse that ended its sentence, with a verse number
    /// right after them. They are headings, never words of the verse
    /// before.
    fn headings_before_verses(&mut self) {
        // In capitals throughout ("BOOK 2", "ב BETH."): set apart by itself.
        let capitals = |line: &str| {
            line.chars().filter(|c| c.is_alphabetic()).count() > 1
                && !line.chars().any(char::is_lowercase)
                && line.chars().any(char::is_uppercase)
        };
        let shape = |line: &str, set_apart: bool| {
            let line = line.trim();
            let ends = line.ends_with([
                '.', '!', '?', ';', ':', ',', '”', '’', '"', '»', '\'', ')', '-', '‐', '–',
            ]);
            line.chars().count() <= 70
                && line.chars().find(|c| c.is_alphabetic()).is_some_and(|c| !c.is_lowercase())
                // A heading set apart and in capitals may end with a full
                // stop ("ב BETH."); any other must not end as a sentence.
                && (!ends || (set_apart && capitals(line) && line.ends_with('.')))
        };
        let ended = |item: &Item| match &item.event {
            Event::Text(t) => {
                let t = t.trim_end();
                t.ends_with(['.', '!', '?', '”', '’', '"', '»', ')'])
            }
            _ => true,
        };
        let candidate = |b: &Self, i: usize| -> bool {
            let item = &b.items[i];
            let Event::Text(line) = &item.event else {
                return false;
            };
            let whole = unmark_raised(&clean(&b.units[item.unit].text)).trim()
                == unmark_raised(line.trim());
            let alone = (i == 0 || b.items[i - 1].unit != item.unit)
                && b.items.get(i + 1).is_none_or(|n| n.unit != item.unit);
            // The document must show it is set apart: a heading style
            // (EPUB, Word, Markdown), large type, another typeface (PDF), or
            // capitals. Text alone cannot tell a heading from a verse's last
            // line ending without punctuation ("… vor Gott", the sentence
            // going on in the next verse). One word alone may be the last
            // word of the verse, wrapped ("Ruth").
            let unit = &b.units[item.unit];
            let set_apart = unit.heading || unit.large || unit.styled || capitals(line);
            whole && alone && set_apart && shape(line, set_apart) && line.split_whitespace().count() >= 2
        };
        let mut k = 1;
        while k < self.items.len() {
            if !candidate(self, k) {
                k += 1;
                continue;
            }
            // A verse or chapter number comes next, perhaps after lines
            // already left out (cross-references, other headings).
            let next = (k + 1..self.items.len())
                .find(|&i| !matches!(self.items[i].event, Event::Other { .. }));
            let starts_verse = next.is_some_and(|i| {
                matches!(&self.items[i].event,
                    Event::Marker(m) if m.unit_start && m.chapter.is_none() && m.book.is_none())
            });
            if !starts_verse {
                k += 1;
                continue;
            }
            // One line, or two (a heading wrapped over two lines).
            let two = k >= 2 && candidate(self, k - 1) && ended(&self.items[k - 2]);
            let first = if two { k - 1 } else { k };
            if ended(&self.items[first - 1]) {
                for i in first..=k {
                    if let Event::Text(line) = &self.items[i].event {
                        let text = unmark_raised(line.trim());
                        self.items[i].event = Event::Other { notes: false, text };
                    }
                }
            }
            k += 1;
        }
    }
}

/// Runs of three or more lines that are nothing but consecutive numbers
/// from 1 ("1", "2", "3" …), or a line of such numbers ("1 2 3 4 5"): a
/// chapter index (an EPUB's navigation, a book's list of chapters), never
/// chapter or verse numbers.
fn chapter_indexes(units: &[Unit], skip: &HashSet<usize>) -> HashSet<usize> {
    let number = |i: usize| -> Option<u32> {
        let t = clean(&units[i].text);
        let t = t.trim();
        (!t.is_empty() && t.len() <= 3 && t.chars().all(|c| c.is_ascii_digit()))
            .then(|| t.parse().ok())
            .flatten()
    };
    let mut out = HashSet::new();
    let lines: Vec<usize> = (0..units.len())
        .filter(|i| !skip.contains(i) && !clean(&units[*i].text).trim().is_empty())
        .collect();
    let mut k = 0;
    while k < lines.len() {
        if number(lines[k]) == Some(1) {
            let mut end = k;
            while end + 1 < lines.len() && number(lines[end + 1]) == number(lines[end]).map(|n| n + 1) {
                end += 1;
            }
            if end - k >= 2 {
                out.extend(lines[k..=end].iter().copied());
                k = end + 1;
                continue;
            }
        }
        // "1 2 3 4 5 6": a line of consecutive numbers from 1.
        let t = clean(&units[lines[k]].text);
        let parts: Vec<&str> = t.split_whitespace().collect();
        if parts.len() >= 3
            && parts.iter().enumerate().all(|(i, p)| *p == (i + 1).to_string())
        {
            out.insert(lines[k]);
        }
        k += 1;
    }
    out
}

/// Where a passage reference closing a verse begins ("… Deus conosco.
/// Isaías 7:14", "… (Ps 2:7)"): after the verse's last sentence, and nothing
/// after it but closing punctuation. None when the verse does not end so.
fn trailing_reference(text: &str, language: Option<ReferenceLanguage>) -> Option<usize> {
    let end = text.trim_end_matches(|c: char| c.is_whitespace() || matches!(c, '.' | ')' | ']')).len();
    let languages: Vec<ReferenceLanguage> = match language {
        Some(l) => vec![l],
        None => name_languages(),
    };
    for l in languages {
        for m in ReferenceParser::for_language(l).matches(text) {
            if m.end != end || !m.written_verse {
                continue;
            }
            let mut start = m.start;
            let before = text[..start].trim_end();
            let before = match before.strip_suffix(['(', '[']) {
                Some(b) => {
                    start = b.len();
                    b.trim_end()
                }
                None => before,
            };
            if !before.is_empty() && before.ends_with(['.', '!', '?', '»', '”', '’', '"', ':', ';']) {
                return Some(start);
            }
        }
    }
    None
}

/// A line made of passage references and nothing else of substance ("V.
/// 18-24: cf. (1 Co 11:7-10. 1 Ti 2:11-13.)", "(Ps 103:7. Né 9:9-20.)", "Ch.
/// 1 à 11: 9. (Ac 17:24-28.)"): at least one chapter-and-verse reference,
/// and outside the book names the references give, no word longer than
/// three letters (abbreviations, "cf", "v", "à").
fn reference_line(text: &str, language: Option<ReferenceLanguage>) -> bool {
    use std::sync::OnceLock;
    static PAIR: OnceLock<regex::Regex> = OnceLock::new();
    let pair = PAIR.get_or_init(|| regex::Regex::new(r"\d{1,3}\s?[:,.]\s?\d{1,3}").expect("valid"));
    if !pair.is_match(text) {
        return false;
    }
    // Book names as the references write them, by byte range.
    let mut covered: Vec<(usize, usize)> = Vec::new();
    let languages: Vec<ReferenceLanguage> = match language {
        Some(l) => vec![l],
        None => name_languages(),
    };
    for l in languages {
        for m in ReferenceParser::for_language(l).matches(text) {
            covered.push((m.start, m.end));
        }
    }
    let mut long_words = 0;
    let mut short_words = 0;
    let mut at = 0;
    for word in text.split(|c: char| !c.is_alphabetic()) {
        let start = text[at..].find(word).map_or(at, |p| at + p);
        at = start + word.len();
        if word.is_empty() {
            continue;
        }
        if covered.iter().any(|(a, b)| start >= *a && start < *b) {
            continue;
        }
        if word.chars().count() > 3 {
            long_words += 1;
        } else {
            short_words += 1;
        }
    }
    long_words == 0 && short_words <= 12
}

// --- reading the events ------------------------------------------------------------------

struct Open {
    ordinal: u8,
    chapter: u16,
    verse: u16,
    end: Option<u16>,
    text: String,
    start: Place,
    last: Place,
    last_unit: usize,
}

struct Reader<'a> {
    units: &'a [Unit],
    items: &'a [Item],
    language: Option<ReferenceLanguage>,
    corrections: &'a Corrections,
    out: Recognition,
    book: Option<&'static Book>,
    /// The found-book entry being read, and whether it is left out.
    entry: Option<usize>,
    skip: bool,
    notes: bool,
    chapter: Option<u16>,
    /// The next verse expected in the chapter.
    expect: u16,
    /// Whether the chapter began at a heading (or its number) just now.
    fresh: bool,
    open: Option<Open>,
    /// Text since the chapter began, before its first verse.
    before: Option<(String, Place, usize)>,
    ambiguous: HashSet<(u8, u16, u16)>,
    seen_books: HashSet<u8>,
    long: HashSet<(u8, u16, u16)>,
    /// The chapter heading named a chapter the book does not have.
    skip_chapter: bool,
    /// Verse numbers met where no book had been named.
    unplaced: usize,
    /// The next words follow the last ones with no space (a number put back).
    glue: bool,
    /// Chapters refused for a verse number out of sequence (v0.24).
    sequence_flagged: HashSet<(u8, u16)>,
    /// Chapters reported for text from a page whose reading order is uncertain.
    order_flagged: HashSet<(u8, u16)>,
    /// The signs the notes of each PDF page begin with.
    note_signs: HashMap<(u16, u32), Vec<NoteSign>>,
    /// Notes by the letter they begin with and the verse they name ("c 13:6").
    note_letters: HashMap<(u16, u32), Vec<NoteLetter>>,
    /// Chapters reported for unexplained raised letters.
    raised_flagged: HashSet<(u8, u16)>,
    /// Short lines repeated as whole lines (perhaps speakers' names).
    labels: HashSet<String>,
    /// Chapters reported for such a line inside a verse.
    label_flagged: HashSet<(u8, u16)>,
}

impl Reader<'_> {
    fn place(&self, unit: usize) -> Place {
        self.units[unit].place
    }

    fn leave_out(&mut self, text: &str, unit: usize, reason: &'static str) {
        let chars = text.chars().count();
        self.out.stats.left_out_chars += chars;
        if self.out.left_out.len() < SAMPLES {
            self.out.left_out.push(LeftOut {
                sample: text.chars().take(80).collect(),
                chars,
                place: self.place(unit),
                reason,
            });
        } else if let Some(same) = self
            .out
            .left_out
            .iter_mut()
            .rev()
            .find(|l| l.reason == reason)
        {
            same.chars += chars;
        }
    }

    /// Appends words to the open verse (a hyphen before "und", "oder" … at a
    /// line's end stands for a word part left out — "Lob- und Dankopfer" —
    /// and is kept with its space): a word broken at a line's end is
    /// joined again (PDF and text lines).
    fn append(open: &mut Open, words: &str, unit: usize, place: Place, glue: bool) {
        if open.text.is_empty() || glue {
            open.text.push_str(words);
        } else if unit != open.last_unit
            && open.text.ends_with('-')
            // A dash standing alone ("Salut, -") is punctuation, not a word
            // broken at the line's end.
            && open.text.chars().rev().nth(1).is_some_and(char::is_alphabetic)
            && words.chars().next().is_some_and(char::is_lowercase)
            && !suspended_hyphen(words)
        {
            open.text.pop();
            open.text.push_str(words);
        } else {
            open.text.push(' ');
            open.text.push_str(words);
        }
        open.last = place;
        open.last_unit = unit;
    }

    fn close(&mut self) {
        let Some(open) = self.open.take() else {
            return;
        };
        let mut text = open.text.split_whitespace().collect::<Vec<_>>().join(" ");
        // A passage reference after the verse's last sentence ("… Deus
        // conosco. Isaías 7:14"): the source of a quotation, not Scripture.
        if let Some(cut) = trailing_reference(&text, self.language) {
            let tail = text[cut..].trim().to_string();
            text.truncate(cut);
            let kept = text.trim_end().len();
            text.truncate(kept);
            self.leave_out(&tail, open.last_unit, "crossReference");
        }
        let key = (open.ordinal, open.chapter, open.verse);
        let book = canon::by_ordinal(open.ordinal).expect("a staged book is canonical");
        if text.is_empty() {
            self.out.content.issue(
                Severity::Warning,
                Some(book),
                Some(open.chapter),
                UserMessage::new(
                    "bibleImport.emptyVerse",
                    format!("{} {}:{} has no text.", book.name, open.chapter, open.verse),
                )
                .with("bookCode", book.code)
                .with("chapter", open.chapter)
                .with("verse", open.verse),
            );
            return;
        }
        if text.chars().count() > VERSE_MAX_CHARS {
            self.long.insert(key);
        }
        let chapter = self
            .out
            .content
            .books
            .entry(open.ordinal)
            .or_default()
            .chapters
            .entry(open.chapter)
            .or_default();
        if chapter.verses.contains_key(&open.verse) {
            self.out.content.issue(
                Severity::Error,
                Some(book),
                Some(open.chapter),
                UserMessage::new(
                    "bibleImport.verseTwice",
                    format!(
                        "{} {}:{} appears twice.",
                        book.name, open.chapter, open.verse
                    ),
                )
                .with("bookCode", book.code)
                .with("chapter", open.chapter)
                .with("verse", open.verse),
            );
            return;
        }
        chapter.verses.insert(open.verse, text);
        if let Some(end) = open.end {
            chapter.bridges.insert(open.verse, end);
        }
        self.out.origins.insert(
            key,
            Origin {
                start: open.start,
                end: open.last,
            },
        );
        if let Some(entry) = self.entry {
            self.out.books[entry].verses += 1;
        }
    }

    /// Text gathered before the chapter's first verse: a Psalm's title, or
    /// left out.
    fn settle_before(&mut self, first_verse_numbered: bool) {
        let Some((text, _, unit)) = self.before.take() else {
            return;
        };
        let (Some(book), Some(chapter)) = (self.book, self.chapter) else {
            self.leave_out(&text, unit, "beforeVerses");
            return;
        };
        if book.code == "PSA" && first_verse_numbered {
            let title = text.split_whitespace().collect::<Vec<_>>().join(" ");
            self.out
                .content
                .books
                .entry(book.ordinal)
                .or_default()
                .chapters
                .entry(chapter)
                .or_default()
                .superscription = Some(title);
            self.out.stats.psalm_titles += 1;
        } else {
            self.leave_out(&text, unit, "beforeVerses");
        }
    }

    fn start_chapter(&mut self, chapter: u16, unit: usize) {
        self.close();
        if let Some((text, _, at)) = self.before.take() {
            self.leave_out(&text, at, "beforeVerses");
        }
        self.chapter = None;
        let Some(book) = self.book else {
            return;
        };
        if chapter > book.chapter_count {
            self.out.content.issue(
                Severity::Error,
                Some(book),
                Some(chapter),
                UserMessage::new(
                    "bibleImport.chapterOutOfRange",
                    format!(
                        "{} {chapter}: {} has {} chapters in the standard numbering.",
                        book.name, book.name, book.chapter_count
                    ),
                )
                .with("bookCode", book.code)
                .with("chapter", chapter)
                .with("count", book.chapter_count),
            );
            self.skip_chapter = true;
            return;
        }
        let _ = unit;
        let exists = self
            .out
            .content
            .books
            .get(&book.ordinal)
            .is_some_and(|b| b.chapters.contains_key(&chapter));
        if exists {
            self.out.content.issue(
                Severity::Error,
                Some(book),
                Some(chapter),
                UserMessage::new(
                    "bibleImport.documentChapterTwice",
                    format!("{} {chapter} appears twice in the text.", book.name),
                )
                .with("bookCode", book.code)
                .with("chapter", chapter),
            );
            self.skip_chapter = true;
            return;
        }
        self.skip_chapter = false;
        self.chapter = Some(chapter);
        self.expect = 1;
        self.fresh = true;
        if let Some(entry) = self.entry {
            self.out.books[entry].chapters += 1;
        }
    }

    /// A repeated short line (see [`repeated_labels`]) between two sentences
    /// of a verse: perhaps a speaker's name set in another style, perhaps
    /// words of the verse. The chapter is refused rather than guessed.
    fn check_label(&mut self, words: &str, unit: usize) {
        let whole = clean(&self.units[unit].text);
        let whole = whole.trim();
        // One word in capitals alone on a line ("BETH", "BOOK"): a heading in
        // another style, or a word of the verse — not known without the style.
        let capitals = !whole.contains(' ')
            && whole.chars().count() >= 2
            && whole
                .chars()
                .all(|c| c.is_alphabetic() && !c.is_lowercase())
            && whole.chars().any(char::is_uppercase);
        if whole != words.trim() || !(self.labels.contains(whole) || capitals) {
            return;
        }
        let ended = self.open.as_ref().is_some_and(|o| {
            o.text
                .trim_end()
                .ends_with(['.', '!', '?', ';', ':', '”', '’', '"', ')'])
        });
        let (Some(book), Some(chapter)) = (self.book, self.chapter) else {
            return;
        };
        if !ended || !self.label_flagged.insert((book.ordinal, chapter)) {
            return;
        }
        self.out.content.issue(
            Severity::Error,
            Some(book),
            Some(chapter),
            UserMessage::new(
                "bibleImport.documentLabel",
                format!(
                    "{} {chapter}: the line “{whole}” stands between two sentences of a verse and is repeated as a line of its own elsewhere (perhaps a speaker’s name or a heading). This chapter is not used.",
                    book.name
                ),
            )
            .with("bookCode", book.code)
            .with("chapter", chapter)
            .with("line", whole),
        );
    }

    /// The raised letters that are references in verse `chapter:verse` on
    /// this page: a note beginning with the letter and naming this verse on
    /// this page or the next (or the one before).
    fn referenced_letters(
        &self,
        document: u16,
        page: u32,
        chapter: u16,
        verse: u16,
    ) -> Vec<String> {
        let mut out = Vec::new();
        for p in [page, page + 1, page.saturating_sub(1)] {
            for (label, c, v) in self.note_letters.get(&(document, p)).into_iter().flatten() {
                if *c == chapter && *v == verse && !out.contains(label) {
                    out.push(label.clone());
                }
            }
        }
        out
    }

    /// Raised letters in a verse that no note on the page explains: the
    /// chapter is refused.
    fn raised_issue(&mut self) {
        let (Some(book), Some(chapter)) = (self.book, self.chapter) else {
            return;
        };
        if !self.raised_flagged.insert((book.ordinal, chapter)) {
            return;
        }
        self.out.content.issue(
            Severity::Error,
            Some(book),
            Some(chapter),
            UserMessage::new(
                "bibleImport.documentRaisedLetters",
                format!(
                    "{} {chapter}: a verse has small raised letters that no note on the page explains (perhaps cross-reference letters). This chapter is not used.",
                    book.name
                ),
            )
            .with("bookCode", book.code)
            .with("chapter", chapter),
        );
    }

    /// The note signs that are references in verse `chapter:verse` on this
    /// page: a note beginning with the sign and naming this verse stands on
    /// this page or the next (notes run on), or a note beginning with the
    /// sign and naming no verse stands on this page.
    /// The signs that notes of a document without pages name for this
    /// chapter and verse.
    fn referenced_signs_anywhere(&self, document: u16, chapter: u16, verse: u16) -> Vec<char> {
        let mut out = Vec::new();
        for note in self.note_signs.get(&(document, WHOLE_DOCUMENT)).into_iter().flatten() {
            if note.verse == Some((chapter, verse)) && !out.contains(&note.sign) {
                out.push(note.sign);
            }
        }
        out
    }

    fn referenced_signs(&self, document: u16, page: u32, chapter: u16, verse: u16) -> Vec<char> {
        let mut out = Vec::new();
        for (p, same) in [
            (page, true),
            (page + 1, false),
            (page.saturating_sub(1), false),
        ] {
            for note in self.note_signs.get(&(document, p)).into_iter().flatten() {
                let fits = match note.verse {
                    Some((c, v)) => c == chapter && v == verse,
                    None => same,
                };
                if fits && !out.contains(&note.sign) {
                    out.push(note.sign);
                }
            }
        }
        out
    }

    /// Text of a verse read from a PDF page whose reading order could not be
    /// established: the chapter is refused, never used out of order.
    fn check_order(&mut self, unit: usize) {
        if !self.units[unit].uncertain_order {
            return;
        }
        let (Some(book), Some(chapter)) = (self.book, self.chapter) else {
            return;
        };
        if !self.order_flagged.insert((book.ordinal, chapter)) {
            return;
        }
        let page = self.units[unit].place.page.unwrap_or(0);
        self.out.content.issue(
            Severity::Error,
            Some(book),
            Some(chapter),
            UserMessage::new(
                "bibleImport.documentReadingOrder",
                format!(
                    "{} {chapter}: page {page} is laid out in a way whose reading order could not be established. This chapter is not used.",
                    book.name
                ),
            )
            .with("bookCode", book.code)
            .with("chapter", chapter)
            .with("page", page),
        );
    }

    fn open_verse(&mut self, verse: u16, end: Option<u16>, unit: usize) {
        self.check_order(unit);
        self.close();
        let (Some(book), Some(chapter)) = (self.book, self.chapter) else {
            return;
        };
        let place = self.place(unit);
        self.open = Some(Open {
            ordinal: book.ordinal,
            chapter,
            verse,
            end,
            text: String::new(),
            start: place,
            last: place,
            last_unit: unit,
        });
        self.expect = end.unwrap_or(verse) + 1;
        self.fresh = false;
    }

    /// Whether the chapter being read has all its verses by the standard count.
    fn chapter_complete(&self) -> bool {
        match (self.book, self.chapter) {
            (Some(book), Some(chapter)) => {
                canon::verse_count(book, chapter).is_some_and(|count| self.expect > count)
            }
            _ => false,
        }
    }

    /// Looks ahead from item `at` for a better or equal place for verse `value`.
    /// Some(true): this one; Some(false): a better one comes; None: two equal places.
    fn best_place(&self, at: usize, value: u16, strength: Strength) -> Option<bool> {
        let mut equal = false;
        for (offset, item) in self.items.iter().skip(at + 1).take(LOOKAHEAD).enumerate() {
            match &item.event {
                Event::Book { .. } | Event::Chapter(_) => break,
                Event::Marker(m) if m.chapter.is_some() => break,
                // A number alone on its line with verse 1 next: the next
                // chapter's number, not a second place for this verse.
                Event::Marker(m)
                    if m.unit_start
                        && m.end.is_none()
                        && clean(&self.units[item.unit].text).trim() == m.raw.trim()
                        && self.next_value(at + 1 + offset) == Some(1) =>
                {
                    break
                }
                // Numbering starting again at a paragraph: the next chapter.
                Event::Marker(m)
                    if m.unit_start && m.strength == Strength::Strong && m.value < value =>
                {
                    break
                }
                Event::Marker(m) if m.value == value => {
                    if m.strength > strength {
                        return Some(false);
                    }
                    // Two places equally good, however strong: refused, never guessed.
                    if m.strength == strength {
                        equal = true;
                    }
                }
                Event::Marker(m)
                    if (m.value == value + 1 || m.value == value + 2)
                        && m.strength == Strength::Strong =>
                {
                    break
                }
                _ => {}
            }
        }
        if equal {
            None
        } else {
            Some(true)
        }
    }

    /// The next candidate verse number after item `at` in this chapter.
    fn next_value(&self, at: usize) -> Option<u16> {
        for item in self.items.iter().skip(at + 1).take(LOOKAHEAD) {
            match &item.event {
                Event::Book { .. } | Event::Chapter(_) => return None,
                Event::Marker(m) => return Some(m.value),
                _ => {}
            }
        }
        None
    }

    /// Whether `value + 1` is among the next three numbers of this chapter.
    fn count_goes_on(&self, at: usize, value: u16) -> bool {
        let mut seen = 0;
        for item in self.items.iter().skip(at + 1).take(LOOKAHEAD) {
            match &item.event {
                Event::Book { .. } | Event::Chapter(_) => return false,
                Event::Marker(m) => {
                    if m.value == value + 1 {
                        return true;
                    }
                    seen += 1;
                    if seen == 3 {
                        return false;
                    }
                }
                _ => {}
            }
        }
        false
    }

    fn ambiguous_issue(&mut self, verse: u16) {
        let (Some(book), Some(chapter)) = (self.book, self.chapter) else {
            return;
        };
        if self.ambiguous.insert((book.ordinal, chapter, verse)) {
            self.out.content.issue(
                Severity::Error,
                Some(book),
                Some(chapter),
                UserMessage::new(
                    "bibleImport.documentAmbiguous",
                    format!(
                        "{} {chapter}:{verse}: two places in the text could be where this verse begins."
                    , book.name),
                )
                .with("bookCode", book.code)
                .with("chapter", chapter)
                .with("verse", verse),
            );
        }
    }

    /// Reads a candidate verse number; false when it is not one (its digits
    /// then stay in the text).
    fn marker(&mut self, at: usize, m: &Marker, unit: usize) -> bool {
        if self.skip || self.notes || self.skip_chapter {
            return false;
        }
        let Some(book) = self.book else {
            if m.strength == Strength::Strong {
                self.unplaced += 1;
            }
            return false;
        };
        // A line reference names its chapter: "3:1 …".
        if let Some(chapter) = m.chapter {
            if self.chapter != Some(chapter) {
                self.start_chapter(chapter, unit);
                if self.chapter.is_none() {
                    return false;
                }
            }
            if m.value == self.expect || m.value == self.expect + 1 {
                if self.expect == 1 {
                    self.settle_before(m.value == 1);
                }
                self.open_verse(m.value, m.end, unit);
            } else {
                // Counting back, or jumping ahead, within a chapter: out of order.
                self.order_issue(book, m.value);
                self.open_verse(m.value, m.end, unit);
            }
            return true;
        }
        let strong = m.strength == Strength::Strong;
        let Some(chapter) = self.chapter else {
            // A book with no chapter yet: "1" at the start of a paragraph begins chapter 1.
            if m.unit_start && strong && m.value == 1 {
                self.start_chapter(1, unit);
                self.out.stats.numbered_chapters += 1;
                // The chapter's number on a line of its own, then its verse 1
                // with its own number: the number is not verse text.
                let alone = m.end.is_none() && clean(&self.units[unit].text).trim() == m.raw.trim();
                if alone && self.next_value(at) == Some(1) {
                    return true;
                }
                self.open_verse(1, m.end, unit);
                return true;
            }
            return false;
        };
        let expect = self.expect;
        if m.value == expect {
            match self.best_place(at, m.value, m.strength) {
                Some(true) => {}
                Some(false) => return false,
                None => self.ambiguous_issue(m.value),
            }
            if expect == 1 {
                self.settle_before(true);
            }
            self.open_verse(m.value, m.end, unit);
            return true;
        }
        // The chapter's own number where its verse 1 starts (printed Bibles):
        // verse 2 comes next. (In chapter 2, a "2" followed by verse 3 is
        // verse 2, after a verse 1 that has no number.)
        if expect == 1
            && strong
            && m.unit_start
            && m.value == chapter
            && self.next_value(at) != Some(chapter + 1)
        {
            // "4" on its own, then "1 Da nun …": the chapter's number, and verse 1
            // has its own. The number is not verse text.
            if self.next_value(at) == Some(1) && m.end.is_none() {
                return true;
            }
            self.drop_cap(unit, m.end);
            return true;
        }
        // A chapter's number printed on a line of its own, with its verse 1
        // numbered next ("13", then "1 Und ich trat …"): the document says
        // which chapter begins, and that is followed — even when the chapter
        // before has fewer verses than the standard count (the missing ones
        // are reported, never filled), and even when it is not the chapter
        // counted next (a chapter whose number was lost before it: the count
        // is put right instead of every later chapter being shifted).
        let alone =
            m.unit_start && m.end.is_none() && clean(&self.units[unit].text).trim() == m.raw.trim();
        if strong
            && alone
            && m.value > chapter
            && m.value <= book.chapter_count
            && m.value != expect
            && self.next_value(at) == Some(1)
        {
            // A jump over chapters is never followed blindly: the number may
            // be a page number or a stray figure on its own line. The text is
            // kept apart as this chapter, and the chapter is refused.
            if m.value > chapter + 1 {
                self.out.content.issue(
                    Severity::Error,
                    Some(book),
                    Some(m.value),
                    UserMessage::new(
                        "bibleImport.documentChapterJump",
                        format!(
                            "{} {}: the document goes on with this chapter after chapter {chapter}; the chapters between were not found. This chapter is not used.",
                            book.name, m.value
                        ),
                    )
                    .with("bookCode", book.code)
                    .with("chapter", m.value)
                    .with("previous", chapter),
                );
            }
            self.start_chapter(m.value, unit);
            self.out.stats.numbered_chapters += 1;
            return true;
        }
        // The next chapter's number at the end of a line, once this chapter
        // has all its verses, with its verse 1 numbered next ("… wandelten. 9"
        // then "1 Und …", a two-column page): the chapter begins; the number is
        // not verse text.
        if strong
            && !m.unit_start
            && m.end.is_none()
            && self.chapter_complete()
            && m.value == chapter + 1
            && chapter < book.chapter_count
            && self.next_value(at) == Some(1)
        {
            self.start_chapter(chapter + 1, unit);
            self.out.stats.numbered_chapters += 1;
            return true;
        }
        // The next chapter's number, or verse 1 again, at the start of a paragraph once
        // this chapter has all its verses.
        if strong
            && m.unit_start
            && self.chapter_complete()
            && (m.value == chapter + 1 || m.value == 1)
            && chapter < book.chapter_count
        {
            self.start_chapter(chapter + 1, unit);
            self.out.stats.numbered_chapters += 1;
            // The chapter's number, then its verse 1 with its own number.
            if m.value == chapter + 1 && m.end.is_none() && self.next_value(at) == Some(1) {
                return true;
            }
            let end = if m.value == 1 { m.end } else { None };
            self.open_verse(1, end, unit);
            return true;
        }
        // Verse 1 had no number: the text since the heading is verse 1.
        if expect == 1 && strong && m.value == 2 && self.fresh && self.before.is_some() {
            match self.best_place(at, 2, m.strength) {
                Some(true) => {}
                Some(false) => return false,
                None => self.ambiguous_issue(2),
            }
            self.unnumbered_first(book, chapter);
            self.open_verse(2, m.end, unit);
            return true;
        }
        // One verse missing (left out by the translation, or its number lost).
        if strong && m.value == expect + 1 && expect > 1 {
            match self.best_place(at, m.value, m.strength) {
                Some(true) => {}
                Some(false) => return false,
                None => self.ambiguous_issue(m.value),
            }
            // The missing verse's number standing in the verse before: its
            // number may have been read as text, and the two verses run
            // together. Refused, never guessed.
            let missing = expect.to_string();
            // Standalone only: "2:7" or "7,3" is a reference, not a verse's
            // number.
            let inside = self.open.as_ref().is_some_and(|o| {
                o.text.split_whitespace().any(|w| {
                    w.trim_matches(|c: char| matches!(c, '(' | ')' | ',' | '.' | ';' | '«' | '»'))
                        == missing
                })
            });
            if inside && self.sequence_flagged.insert((book.ordinal, chapter)) {
                self.out.content.issue(
                    Severity::Error,
                    Some(book),
                    Some(chapter),
                    UserMessage::new(
                        "bibleImport.documentNumberInVerse",
                        format!(
                            "{} {chapter}: verse {expect} was not found, but the number {expect} stands in the verse before it; the two may run together. This chapter is not used.",
                            book.name
                        ),
                    )
                    .with("bookCode", book.code)
                    .with("chapter", chapter)
                    .with("verse", expect),
                );
            }
            self.open_verse(m.value, m.end, unit);
            return true;
        }
        // A verse number well ahead of the count, after a sentence end inside
        // a verse: a second column read across the first, or a lost passage.
        // The verses around it cannot be trusted: the chapter is refused,
        // never guessed (a short tail would otherwise only be a warning).
        if strong
            && !m.unit_start
            && m.end.is_none()
            && m.value > expect + 1
            && canon::verse_count(book, chapter).is_some_and(|count| m.value <= count)
            // … and the count goes on from it within the next few numbers
            // (a second column: "3 … 18 …", "4 … 19 …"): a lone number in
            // the text ("Moab, 10 years") is not a verse out of place.
            && self.count_goes_on(at, m.value)
            && self.sequence_flagged.insert((book.ordinal, chapter))
        {
            self.out.content.issue(
                Severity::Error,
                Some(book),
                Some(chapter),
                UserMessage::new(
                    "bibleImport.documentSequence",
                    format!(
                        "{} {chapter}: verse {} appears inside an earlier verse (perhaps two columns were read across each other). This chapter is not used.",
                        book.name, m.value
                    ),
                )
                .with("bookCode", book.code)
                .with("chapter", chapter)
                .with("verse", m.value),
            );
        }
        // Anything else is a number in the text.
        false
    }

    fn drop_cap(&mut self, unit: usize, end: Option<u16>) {
        if let Some((text, _, at)) = self.before.take() {
            // A Psalm's title stands before its number; elsewhere it is left out.
            self.before = Some((text, self.place(at), at));
            self.settle_before(true);
        }
        self.open_verse(1, end, unit);
    }

    fn unnumbered_first(&mut self, book: &'static Book, chapter: u16) {
        let Some((text, place, at)) = self.before.take() else {
            return;
        };
        if book.code == "PSA" {
            // A Psalm's title and its verse 1 cannot be told apart without the number.
            self.out.content.issue(
                Severity::Error,
                Some(book),
                Some(chapter),
                UserMessage::new(
                    "bibleImport.documentPsalmTitle",
                    format!(
                        "Psalm {chapter}: verse 1 has no number, so its title cannot be told apart from it."
                    ),
                )
                .with("bookCode", book.code)
                .with("chapter", chapter),
            );
            self.leave_out(&text, at, "beforeVerses");
            return;
        }
        self.out.stats.unnumbered_first += 1;
        self.open = Some(Open {
            ordinal: book.ordinal,
            chapter,
            verse: 1,
            end: None,
            text,
            start: place,
            last: place,
            last_unit: at,
        });
        self.close();
    }

    fn order_issue(&mut self, book: &'static Book, verse: u16) {
        let Some(chapter) = self.chapter else {
            return;
        };
        self.out.content.issue(
            Severity::Error,
            Some(book),
            Some(chapter),
            UserMessage::new(
                "bibleImport.documentOrder",
                format!(
                    "{} {chapter}:{verse} is out of order in the text.",
                    book.name
                ),
            )
            .with("bookCode", book.code)
            .with("chapter", chapter)
            .with("verse", verse),
        );
    }

    fn text(&mut self, words: &str, unit: usize) {
        // Raised small letters (see `pdf_layout::RAISED_OPEN`): in a verse, taken
        // out where a note on the page shows them to be its references; else
        // kept as written — and then the chapter is not used. Outside a verse
        // they are plain text.
        let resolved;
        let words = if words.contains(RAISED_OPEN) {
            resolved = match self.open.as_ref() {
                Some(open) => {
                    let place = self.units[unit].place;
                    let known = place
                        .page
                        .map(|page| {
                            self.referenced_letters(place.document, page, open.chapter, open.verse)
                        })
                        .unwrap_or_default();
                    let (text, unexplained) = resolve_raised(words, &known);
                    if unexplained {
                        self.raised_issue();
                    }
                    text
                }
                None => unmark_raised(words),
            };
            resolved.as_str()
        } else {
            words
        };
        if self.skip {
            self.leave_out(words, unit, "leftOutBook");
            return;
        }
        if self.notes {
            self.leave_out(words, unit, "notes");
            return;
        }
        if self.skip_chapter {
            self.leave_out(words, unit, "beforeVerses");
            return;
        }
        let place = self.place(unit);
        let glue = std::mem::take(&mut self.glue);
        if self.open.is_some() {
            self.check_order(unit);
            self.check_label(words, unit);
        }
        if let Some(open) = self.open.as_ref() {
            // A PDF page: only the signs the document shows are note
            // references there (below). Elsewhere, the usual signs.
            let place = self.units[unit].place;
            // Only the signs the document's own notes show to be references
            // to this verse are taken out (on a PDF page, its notes; else the
            // document's); any other sign is text ("¶" in the King James
            // Version marks a paragraph).
            let signs = match place.page {
                Some(page) => self.referenced_signs(place.document, page, open.chapter, open.verse),
                None => self.referenced_signs_anywhere(place.document, open.chapter, open.verse),
            };
            let words = without_signs(words, &signs).into_owned();
            let open = self.open.as_mut().expect("a verse is open");
            Self::append(open, &words, unit, place, glue);
            return;
        }
        if self.chapter.is_some() {
            match self.before.as_mut() {
                Some((text, _, last)) => {
                    if !glue {
                        text.push(' ');
                    }
                    text.push_str(words);
                    *last = unit;
                }
                None => self.before = Some((words.to_string(), place, unit)),
            }
            return;
        }
        let reason = if self.book.is_some() {
            "beforeVerses"
        } else {
            "noBook"
        };
        self.leave_out(words, unit, reason);
    }

    fn book(&mut self, entry: usize, chapter: Option<u16>, unit: usize) {
        self.notes = false;
        let detected = self.out.books[entry].detected.and_then(canon::by_code);
        let chosen = match self.corrections.books.get(&entry) {
            Some(choice) => *choice,
            None => detected,
        };
        let same_entry = self.entry == Some(entry);
        if !same_entry {
            self.close();
            if let Some((text, _, at)) = self.before.take() {
                self.leave_out(&text, at, "beforeVerses");
            }
            self.entry = Some(entry);
            self.chapter = None;
            self.skip_chapter = false;
            let found = &mut self.out.books[entry];
            found.book = chosen.map(|b| b.code);
            found.left_out = self
                .corrections
                .books
                .get(&entry)
                .is_some_and(Option::is_none);
            self.skip = found.left_out;
            self.book = chosen;
            if let Some(book) = chosen {
                if !self.seen_books.insert(book.ordinal) {
                    self.out.content.issue(
                        Severity::Error,
                        Some(book),
                        None,
                        UserMessage::new(
                            "bibleImport.documentBookTwice",
                            format!("{} begins twice in the text.", book.name),
                        )
                        .with("bookCode", book.code),
                    );
                    self.skip = true;
                }
            }
        }
        if let Some(chapter) = chapter {
            if !self.skip {
                self.start_chapter(chapter, unit);
            }
        }
    }

    fn run(mut self) -> Recognition {
        for (at, item) in self.items.iter().enumerate() {
            match &item.event {
                Event::Book { entry, chapter } => self.book(*entry, *chapter, item.unit),
                Event::Chapter(n) => {
                    self.notes = false;
                    if self.skip {
                        continue;
                    }
                    if self.book.is_none() {
                        self.unplaced += 1;
                        continue;
                    }
                    self.start_chapter(*n, item.unit);
                }
                Event::Other { notes, text } => {
                    if *notes {
                        self.close();
                        self.notes = true;
                    } else if !self.skip {
                        // A heading inside a chapter is not Scripture text.
                        self.out.stats.headings += 1;
                        self.leave_out(text, item.unit, "heading");
                    }
                }
                Event::Text(words) => self.text(words, item.unit),
                Event::Marker(m) => {
                    if !self.marker(at, m, item.unit) {
                        // Not a verse number: the number is part of the text.
                        self.glue = m.joined;
                        self.text(&m.raw, item.unit);
                        self.glue = m.glued;
                    }
                }
            }
        }
        self.close();
        if let Some((text, _, at)) = self.before.take() {
            self.leave_out(&text, at, "beforeVerses");
        }
        self.finish()
    }

    fn finish(mut self) -> Recognition {
        let mut long: Vec<(u8, u16, u16)> = self.long.iter().copied().collect();
        long.sort_unstable();
        for (ordinal, chapter, verse) in long {
            let book = canon::by_ordinal(ordinal).expect("canonical");
            self.out.content.issue(
                Severity::Error,
                Some(book),
                Some(chapter),
                UserMessage::new(
                    "bibleImport.documentLongVerse",
                    format!(
                        "{} {chapter}:{verse} is far longer than any verse: the verses after it were not found."
                    , book.name),
                )
                .with("bookCode", book.code)
                .with("chapter", chapter)
                .with("verse", verse),
            );
        }
        if self.unplaced > 0 {
            self.out.content.issue(
                Severity::Warning,
                None,
                None,
                UserMessage::new(
                    "bibleImport.documentNoBook",
                    format!(
                        "{} verse numbers were found where no book name comes before them; they were left out.",
                        self.unplaced
                    ),
                )
                .with("count", self.unplaced),
            );
        }
        let s = self.out.stats.clone();
        if s.unnumbered_first > 0 {
            self.out.content.issue(
                Severity::Warning,
                None,
                None,
                UserMessage::new(
                    "bibleImport.documentUnnumbered",
                    format!(
                        "In {} chapters verse 1 has no number of its own: the text between the chapter's heading and verse 2 was taken as verse 1.",
                        s.unnumbered_first
                    ),
                )
                .with("count", s.unnumbered_first),
            );
        }
        if s.psalm_titles > 0 {
            self.out.content.issue(
                Severity::Note,
                None,
                None,
                UserMessage::new(
                    "bibleImport.documentPsalmTitles",
                    format!(
                        "{} Psalm titles were kept as titles, before verse 1.",
                        s.psalm_titles
                    ),
                )
                .with("count", s.psalm_titles),
            );
        }
        if s.footnote_lines > 0 {
            self.out.content.issue(
                Severity::Note,
                None,
                None,
                UserMessage::new(
                    "bibleImport.documentFootnotes",
                    format!(
                        "{} lines of footnotes at the foot of pages were left out.",
                        s.footnote_lines
                    ),
                )
                .with("count", s.footnote_lines),
            );
        }
        if s.page_lines > 0 {
            self.out.content.issue(
                Severity::Note,
                None,
                None,
                UserMessage::new(
                    "bibleImport.documentPageLines",
                    format!(
                        "{} page headers and page numbers repeated across the pages were left out.",
                        s.page_lines
                    ),
                )
                .with("count", s.page_lines),
            );
        }
        if s.left_out_chars > 0 {
            self.out.content.issue(
                Severity::Note,
                None,
                None,
                UserMessage::new(
                    "bibleImport.documentLeftOut",
                    format!(
                        "Text that belongs to no verse was left out ({} characters: headings, introductions, notes).",
                        s.left_out_chars
                    ),
                )
                .with("count", s.left_out_chars),
            );
        }
        self.short_chapters();
        self.out.content.check_structure();
        self.out
    }

    /// Verses missing at the end of a chapter (a chapter cut short) are not
    /// gaps the common checks see: compared with the standard verse counts.
    /// A few fewer is another numbering (a warning); many fewer means the
    /// chapter's end was not found, and its verses are not used (an error).
    fn short_chapters(&mut self) {
        let mut found = Vec::new();
        for (ordinal, staged) in &self.out.content.books {
            let Some(book) = canon::by_ordinal(*ordinal) else {
                continue;
            };
            for (chapter, text) in &staged.chapters {
                let Some(expected) = canon::verse_count(book, *chapter) else {
                    continue;
                };
                let last = text
                    .verses
                    .keys()
                    .chain(text.bridges.values())
                    .copied()
                    .max()
                    .unwrap_or(0);
                if last == 0 || last >= expected {
                    continue;
                }
                let severity = if expected - last > SHORT_TOLERANCE {
                    Severity::Error
                } else {
                    Severity::Warning
                };
                found.push((severity, book, *chapter, last, expected));
            }
        }
        for (severity, book, chapter, last, expected) in found {
            self.out.content.issue(
                severity,
                Some(book),
                Some(chapter),
                UserMessage::new(
                    "bibleImport.documentChapterShort",
                    format!(
                        "{} {chapter} ends at verse {last}; it usually has {expected} verses. The verses after {last} were not found.",
                        book.name
                    ),
                )
                .with("bookCode", book.code)
                .with("chapter", chapter)
                .with("last", last)
                .with("expected", expected),
            );
        }
    }
}

/// Reads the units of a Bible's documents for its books, chapters and verses.
/// `language` is the document's language when known (it decides between
/// books that share a name across languages).
pub fn recognise(units: &[Unit], language: Option<&str>, corrections: &Corrections) -> Recognition {
    let language = language.and_then(ReferenceLanguage::from_label);
    let mut furniture = page_furniture(units);
    let mut footnotes = page_footnotes(units);
    // Notes known from the page's layout (smaller text below the running text).
    footnotes.extend(
        units
            .iter()
            .enumerate()
            .filter(|(_, u)| u.note)
            .map(|(i, _)| i),
    );
    let footnote_lines = footnotes.difference(&furniture).count();
    // The notes of each page, by the sign they begin with and the verse they
    // name ("§ 2:18 …"): on a PDF page a sign is taken out of a verse only
    // when such a note shows it is a reference there.
    let mut note_signs: HashMap<(u16, u32), Vec<NoteSign>> = HashMap::new();
    let mut note_letters: HashMap<(u16, u32), Vec<NoteLetter>> = HashMap::new();
    for &i in &footnotes {
        let unit = &units[i];
        // Notes of a document without pages (EPUB, Word, text) are kept for
        // the whole document: a sign is then a reference only where a note
        // names that very chapter and verse.
        let page = unit.place.page.unwrap_or(WHOLE_DOCUMENT);
        note_signs
            .entry((unit.place.document, page))
            .or_default()
            .extend(note_signs_in(&unmark_raised(&unit.text)));
        note_letters
            .entry((unit.place.document, page))
            .or_default()
            .extend(note_letters_in(&unmark_raised(&unit.text)));
    }
    furniture.extend(footnotes);
    let builder = Builder {
        units,
        language,
        items: Vec::new(),
        books: Vec::new(),
        last_detected: None,
    };
    let (items, books) = builder.build(&furniture);
    let reader = Reader {
        units,
        items: &items,
        language,
        corrections,
        out: Recognition {
            books,
            stats: Stats {
                page_lines: furniture.len() - footnote_lines,
                footnote_lines,
                ..Default::default()
            },
            ..Default::default()
        },
        book: None,
        entry: None,
        skip: false,
        notes: false,
        chapter: None,
        expect: 1,
        fresh: false,
        open: None,
        before: None,
        ambiguous: HashSet::new(),
        seen_books: HashSet::new(),
        long: HashSet::new(),
        skip_chapter: false,
        unplaced: 0,
        glue: false,
        sequence_flagged: HashSet::new(),
        order_flagged: HashSet::new(),
        note_signs,
        note_letters,
        raised_flagged: HashSet::new(),
        labels: repeated_labels(units, &furniture),
        label_flagged: HashSet::new(),
    };
    let mut out = reader.run();
    check_running_heads(units, &furniture, language, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(title: &str, language: Option<&str>) -> Option<&'static str> {
        book_named(title, language.and_then(ReferenceLanguage::from_label)).map(|b| b.code)
    }

    #[test]
    fn printed_titles_name_their_books() {
        // Luther 1912
        assert_eq!(named("Das 1. Buch Mose (Genesis)", Some("de")), Some("GEN"));
        assert_eq!(named("Das erste Buch Samuel", Some("de")), Some("1SA"));
        assert_eq!(named("Das zweite Buch der Könige", Some("de")), Some("2KI"));
        assert_eq!(named("Der Prophet Jesaja", Some("de")), Some("ISA"));
        assert_eq!(
            named("Das Evangelium nach Matthäus", Some("de")),
            Some("MAT")
        );
        assert_eq!(
            named(
                "Der zweite Brief des Apostels Paulus an die Korinther",
                Some("de")
            ),
            Some("2CO")
        );
        assert_eq!(
            named("Der erste Brief des Johannes", Some("de")),
            Some("1JN")
        );
        assert_eq!(
            named("Das Buch der Offenbarung Jesu Christi", Some("de")),
            Some("REV")
        );
        // A misprint one letter away from one book's name ("Chonik").
        assert_eq!(named("Das erste Buch der Chonik", Some("de")), Some("1CH"));
        // World English Bible
        assert_eq!(named("The Second Book of Kings", Some("en")), Some("2KI"));
        assert_eq!(
            named("Paul’s Second Letter to the Corinthians", Some("en")),
            Some("2CO")
        );
        assert_eq!(
            named("Paul’s Letter to the Romans", Some("en")),
            Some("ROM")
        );
        assert_eq!(named("Peter’s First Letter", Some("en")), Some("1PE"));
        assert_eq!(named("The Letter from James", Some("en")), Some("JAS"));
        assert_eq!(named("The Acts of the Apostles", Some("en")), Some("ACT"));
        assert_eq!(named("The Revelation to John", Some("en")), Some("REV"));
        // A subtitle is never a title by itself.
        assert_eq!(named("of Jeremiah", Some("en")), None);
        assert_eq!(named("The Good News According to", Some("en")), None);
        // Section headings that begin with a book's name are not titles.
        for heading in [
            "The Song of Moses",
            "The Song of Deborah",
            "The Number of the Beast",
            "The Mark of the Beast",
            "The Job Speaks",
            "The Hebrew Midwives",
            "The Romans Arrest Paul",
            "The Lamentation over Saul",
        ] {
            assert_eq!(named(heading, Some("en")), None, "{heading}");
        }
        assert_eq!(named("The Song of Solomon", Some("en")), Some("SNG"));
        assert_eq!(named("Das 3. Buch Mose (Levitikus)", Some("de")), Some("LEV"));
        assert_eq!(named("Das erste Buch der Könige", Some("de")), Some("1KI"));
        assert_eq!(named("Genèse", Some("fr")), Some("GEN"));
        assert_eq!(named("Jean", Some("fr")), Some("JHN"));
        assert_eq!(named("LA GENÈSE", Some("fr")), Some("GEN"));
        assert_eq!(named("ÉVANGILE SELON LUC", Some("fr")), Some("LUK"));
        assert_eq!(named("Première épître de Pierre", Some("fr")), Some("1PE"));
        assert_eq!(named("Épître de Paul aux Romains", Some("fr")), Some("ROM"));
        assert_eq!(named("Les Actes des Apôtres", Some("fr")), Some("ACT"));
        assert_eq!(named("El Santo Evangelio según San Lucas", Some("es")), Some("LUK"));
        assert_eq!(named("Primera epístola de San Pablo a los Corintios", Some("es")), Some("1CO"));
        assert_eq!(named("Evangelho segundo Lucas", Some("pt")), Some("LUK"));
        assert_eq!(named("Primeira epístola de Pedro", Some("pt")), Some("1PE"));
    }

    #[test]
    fn contents_entries_running_heads_and_note_lines_are_read_as_such() {
        assert!(contents_entry("Exodus . . . . . . . . . . 60"));
        assert!(contents_entry("1 Thessalonians . . . . . . . .1200"));
        assert!(contents_entry("Exodus …… 60"));
        assert!(!contents_entry("Exodus 60"));
        assert!(!contents_entry("and they said. 12 men came"));
        assert!(reference_head("Genesis 24:48 27 Genesis 25:11"));
        assert!(reference_head("Obadja 1 972 Obadja 19"));
        assert!(!reference_head("Obadja 1 972 Jona 19"));
        assert_eq!(
            head_references("1 Samuel 3:7 295 1 Samuel 3:9"),
            Some((("1 Samuel".into(), 3, 7), ("1 Samuel".into(), 3, 9)))
        );
        let notes = note_signs_in("“הוהי”). ‡ 2:12 or, aromatic resin § 2:18 or, suitable");
        assert!(notes.contains(&NoteSign {
            sign: '‡',
            verse: Some((2, 12))
        }));
        assert!(notes.contains(&NoteSign {
            sign: '§',
            verse: Some((2, 18))
        }));
        // "11:351Kings": the space before "1 Kings" lost; 11:35 is among the readings.
        assert!(note_signs_in("* 11:351Kings17:17-23;").contains(&NoteSign {
            sign: '*',
            verse: Some((11, 35))
        }));
        assert_eq!(
            note_letters_in("c 13:6 Ge 36:7. d 13:7 Ge 12:6."),
            vec![("c".into(), 13, 6), ("d".into(), 13, 7)]
        );
        assert!(suspended_hyphen("und Dankopfer,"));
        assert!(!suspended_hyphen("fer bringen"));
        assert!(!suspended_hyphen("or to the governor"));
        assert!(!suspended_hyphen("nor of the land."));
    }

    #[test]
    fn raised_letters_are_resolved_by_the_notes() {
        let text =
            format!("Lot,{RAISED_OPEN}c{RAISED_CLOSE} qui et {RAISED_OPEN}d{RAISED_CLOSE}Et");
        assert_eq!(
            resolve_raised(&text, &["c".into(), "d".into()]),
            ("Lot, qui et Et".into(), false)
        );
        assert_eq!(
            resolve_raised(&text, &["c".into()]),
            ("Lot, qui et dEt".into(), true)
        );
        assert_eq!(unmark_raised(&text), "Lot,c qui et dEt");
    }

    #[test]
    fn note_callers_are_taken_out_and_nothing_else() {
        assert_eq!(without_note_callers("quenched.’ *"), "quenched.’");
        assert_eq!(without_note_callers("a cause † will"), "a cause will");
        assert_eq!(without_note_callers("Christ.§ Then"), "Christ. Then");
        assert_eq!(without_note_callers("§Now when"), "Now when");
        assert_eq!(without_note_callers("no callers"), "no callers");
    }
}
