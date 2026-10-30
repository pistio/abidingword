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
/// A chapter with this many verses more than usual (or half as many again,
/// for a long chapter) is not another numbering of it: another chapter, or
/// another book, was read into it.
const LONG_TOLERANCE: u16 = 6;
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
    /// A book outside the 66 (its USFM code: "TOB", "1MA", "ESG"): named,
    /// and its text left out (v0.24.2).
    pub outside: Option<&'static str>,
    pub chapters: usize,
    pub verses: usize,
    pub place: Place,
    /// The book used is the person's choice, not what was found (v0.24.3):
    /// verses under it are mapped by a manual correction.
    pub corrected: bool,
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
    /// Verses whose place, book, chapter and verse were found but whose text
    /// is not verified (v0.24.3), with why: "possibleHeading" (its last line
    /// could be a heading in the body's own type), "reference" (a passage
    /// reference stands in it: a running head or a cross-reference read
    /// in), "noteSign" (a note's sign no note explains), "unnumbered" (verse
    /// 1 without a number: its end was inferred), "length" (far longer than
    /// the other verses of its chapter), "psalmTitle" (a psalm's verse 1 on
    /// a line of its own that may hold the psalm's title), "boundary" (where
    /// it ends or begins is not certain: a verse after it is missing, or the
    /// number between them is raised against a word). Such a verse can be found
    /// in the document; its text never becomes the Bible's verse.
    pub unverified: BTreeMap<(u8, u16, u16), &'static str>,
    /// What was set apart from the verses, with where it stands (v0.24.3).
    pub apparatus: Vec<Apparatus>,
}

/// A piece of the document's apparatus set apart from the verses (v0.24.3):
/// a footnote, a cross-reference, a heading, a running head, an
/// introduction … with its place, so that nothing excluded is lost.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Apparatus {
    /// "footnote", "crossReference", "heading", "subscription",
    /// "runningHead", "introduction", "frontMatter", "notes", "otherBook".
    pub kind: &'static str,
    pub text: String,
    pub place: Place,
    /// The verse it stands with, when one was open.
    pub verse: Option<(u8, u16, u16)>,
}

/// Most characters kept of one piece of apparatus, and of all of it.
const APPARATUS_PIECE_CHARS: usize = 4_000;
const APPARATUS_TOTAL_CHARS: usize = 40_000_000;

/// The kind of apparatus a reason for leaving text out stands for.
fn apparatus_kind(reason: &str) -> &'static str {
    match reason {
        "heading" => "heading",
        "subscription" => "subscription",
        "crossReference" => "crossReference",
        "noteOnly" => "footnote",
        "notes" => "notes",
        "noBook" => "frontMatter",
        "leftOutBook" => "otherBook",
        _ => "introduction",
    }
}

/// Adds text to the apparatus, joined to the piece before when it is of the
/// same kind and goes on from the unit before it.
#[allow(clippy::too_many_arguments)]
fn add_apparatus(
    list: &mut Vec<Apparatus>,
    last_unit: &mut Option<usize>,
    total: &mut usize,
    kind: &'static str,
    text: &str,
    place: Place,
    unit: usize,
    verse: Option<(u8, u16, u16)>,
) {
    let text = text.trim();
    if text.is_empty() || *total >= APPARATUS_TOTAL_CHARS {
        return;
    }
    let follows = last_unit.is_some_and(|u| unit == u || unit == u + 1);
    *last_unit = Some(unit);
    if let Some(last) = list.last_mut() {
        if follows
            && last.kind == kind
            && last.place.document == place.document
            && last.text.chars().count() + text.chars().count() < APPARATUS_PIECE_CHARS
        {
            last.text.push(' ');
            last.text.push_str(text);
            *total += text.len() + 1;
            return;
        }
    }
    let text: String = text.chars().take(APPARATUS_PIECE_CHARS).collect();
    *total += text.len();
    list.push(Apparatus {
        kind,
        text,
        place,
        verse,
    });
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
    // The ligatures of Latin titles: "ISAIÆ" is "isaiae".
    let text = text.replace(['Æ', 'æ'], "ae").replace(['Œ', 'œ'], "oe");
    normalise(&fold_marks(&text))
}

/// Titles of the books outside the 66 (`canons::ADDITIONAL_BOOKS`) as
/// editions print them, after a title is brought down to its name (articles,
/// "book of", ordinals as figures). Their text is left out, never read into
/// the book before (v0.24.2). `after`: only once that book has begun ("1
/// Esdras" after Nehemiah; in the Vulgate "Liber primus Esdrae" is Ezra).
const OUTSIDE_TITLES: &[(&str, &str, Option<&str>)] = &[
    ("tobit", "TOB", None),
    ("tobias", "TOB", None),
    ("tobie", "TOB", None),
    ("tobia", "TOB", None),
    ("judith", "JDT", None),
    ("judite", "JDT", None),
    ("judit", "JDT", None),
    ("sabedoria de salomao", "WIS", None),
    ("wisdom of solomon", "WIS", None),
    ("sagesse de salomon", "WIS", None),
    ("sabiduria de salomon", "WIS", None),
    ("weisheit salomos", "WIS", None),
    ("sapientia salomonis", "WIS", None),
    ("sapientiae", "WIS", None),
    ("sirach", "SIR", None),
    ("jesus sirach", "SIR", None),
    ("sirac", "SIR", None),
    ("siracide", "SIR", None),
    ("siracida", "SIR", None),
    ("eclesiastico", "SIR", None),
    ("ecclesiasticus", "SIR", None),
    ("ecclesiastique", "SIR", None),
    ("ecclesiastici", "SIR", None),
    ("baruc", "BAR", None),
    ("baruch", "BAR", None),
    ("baruque", "BAR", None),
    ("carta de jeremias", "LJE", None),
    ("epistle of jeremy", "LJE", None),
    ("letter of jeremiah", "LJE", None),
    ("lettre de jeremie", "LJE", None),
    ("brief des jeremia", "LJE", None),
    ("susanna", "SUS", None),
    ("susana", "SUS", None),
    ("suzanne", "SUS", None),
    ("bel and the dragon", "BEL", None),
    ("bel e o dragao", "BEL", None),
    ("bel y el dragon", "BEL", None),
    ("bel et le dragon", "BEL", None),
    ("1 macabeus", "1MA", None),
    ("2 macabeus", "2MA", None),
    ("3 macabeus", "3MA", None),
    ("4 macabeus", "4MA", None),
    ("1 maccabees", "1MA", None),
    ("2 maccabees", "2MA", None),
    ("3 maccabees", "3MA", None),
    ("4 maccabees", "4MA", None),
    ("1 macabeos", "1MA", None),
    ("2 macabeos", "2MA", None),
    ("1 makkabaer", "1MA", None),
    ("2 makkabaer", "2MA", None),
    ("1 machabaeorum", "1MA", None),
    ("2 machabaeorum", "2MA", None),
    ("1 maccabei", "1MA", None),
    ("2 maccabei", "2MA", None),
    ("1 esdras", "1ES", Some("NEH")),
    ("2 esdras", "2ES", Some("NEH")),
    ("3 esdras", "1ES", Some("NEH")),
    ("4 esdras", "2ES", Some("NEH")),
    ("oracao de manasses", "MAN", None),
    ("prayer of manasseh", "MAN", None),
    ("prayer of manasses", "MAN", None),
    ("priere de manasse", "MAN", None),
    ("oracion de manases", "MAN", None),
    ("gebet manasses", "MAN", None),
    ("oratio manassae", "MAN", None),
    // The Vulgate's own titles (Jerome's, as in the Stuttgart edition).
    ("tobiae", "TOB", None),
    ("iudith", "JDT", None),
    ("sapientiae solomonis", "WIS", None),
    ("sapientiae salomonis", "WIS", None),
    ("iesu filii sirach", "SIR", None),
    ("primus macchabeorum", "1MA", None),
    ("macchabeorum incipit liber secundus", "2MA", None),
    ("secundus macchabeorum", "2MA", None),
    ("토빗기", "TOB", None),
    ("유딧기", "JDT", None),
    ("지혜서", "WIS", None),
    ("집회서", "SIR", None),
    ("바룩서", "BAR", None),
    ("마카베오기 상권", "1MA", None),
    ("마카베오기 하권", "2MA", None),
];

/// Words in the subtitle of the Greek text of Esther or Daniel printed as
/// a book of its own ("Ester / traduzido da Septuaginta grega", "Daniel /
/// com as Porções Gregas", "Esther (Greek)").
const GREEK_MARKS: &[&str] = &["grec", "greg", "griech", "greek", "septuag", "lxx"];

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

/// A Greek title brought to the forms of [`GREEK_TITLES`]: the words that
/// describe the book (Paul the apostle's, the letter, catholic, the gospel)
/// left out, an ordinal word as its letter ("ΠΡΩΤΗ" → "α"), with and
/// without "προς".
fn greek_forms(k: &str) -> Vec<String> {
    if !k.chars().any(|c| ('\u{0370}'..='\u{03FF}').contains(&c)) {
        return Vec::new();
    }
    const FILLER: &[&str] = &[
        "παυλου",
        "του",
        "αποστολου",
        "η",
        "το",
        "ο",
        "επιστολη",
        "καθολικη",
        "ευαγγελιον",
        "αγιου",
        "θεολογου",
    ];
    let mut letter = None;
    let mut rest: Vec<&str> = Vec::new();
    for word in k.split(' ') {
        match word {
            "πρωτη" | "α" => letter = Some("α"),
            "δευτερα" | "β" => letter = Some("β"),
            "τριτη" | "γ" => letter = Some("γ"),
            w if FILLER.contains(&w) => {}
            w => rest.push(w),
        }
    }
    if rest.is_empty() {
        return Vec::new();
    }
    let name = rest.join(" ");
    let bare = name.strip_prefix("προς ").unwrap_or(&name).to_string();
    let mut out = Vec::new();
    for base in [bare.clone(), format!("προς {bare}")] {
        match letter {
            Some(l) => out.push(format!("{base} {l}")),
            None => out.push(base),
        }
    }
    out
}

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
        // Latin, for the Vulgate and scholarly editions: its full names
        // only (its short forms, "In", "Ap", "Can", would name books where
        // a line is something else).
        if let Some(latin) = ReferenceLanguage::from_label("la") {
            for (name, book) in ReferenceParser::for_language(latin).lookup_entries() {
                if key(name).chars().filter(|c| c.is_alphabetic()).count() >= 5 {
                    add(name, book, latin);
                }
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
        // Latin
        ("primus", 1),
        ("prima", 1),
        ("secundus", 2),
        ("secunda", 2),
        ("secundis", 2),
        ("tertius", 3),
        ("tertia", 3),
        ("quartus", 4),
        ("quarta", 4),
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
        "a",
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
        // "Carta Universal de Tiago", "Epístola Universal de Pedro"
        "universal",
        // Latin ("Incipit liber …": "here begins the book …")
        "incipit",
        "liber",
        "epistula",
        "evangelium",
    ];
    // Only after one of the words above ("Book of", "Brief an die", "Épître
    // aux"): a line that begins "of Jeremiah" is a subtitle, never a title.
    const LINKING: &[&str] = &[
        "das",
        "der",
        "die",
        "des",
        "dem",
        "the",
        "an",
        "nach",
        "of",
        "to",
        "from",
        "by",
        "according", // German, English
        "de",
        "du",
        "d'",
        "aux",
        "au",
        "a",
        "selon",
        "la",
        "le",
        "les",
        "l'", // French
        "del",
        "los",
        "las",
        "segun",
        "el", // Spanish
        "do",
        "da",
        "dos",
        "das",
        "aos",
        "ao",
        "segundo",
        "o",
        "os", // Portuguese
        "secundum",
        "ad", // Latin
    ];
    const AFTER: &[&str] = &[
        "jesu christi",
        "of jesus christ",
        "salomos",
        "salomo",
        "of solomon",
        "de salomao",
        "de salomon",
        "de jesus-christ",
        "de jesucristo",
        "de jesus cristo",
        // Latin: "Isaiae prophetae", "Amos propheta", "Apocalypsis Iohannis apostoli"
        "prophetae",
        "propheta",
        "apostoli",
        "salomonis",
        "solomonis",
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
        "revelacao",
        "apocalipse",
        "lamentacoes",
        "apocalypsis",
        "lamentationes",
    ];
    // "l'evangile" → "l' evangile", "d'esaie" → "d' esaie".
    let key = key.replace('’', "'");
    let key = key.replace("l'", "l' ").replace("d'", "d' ");

    let all: Vec<&str> = key.split(' ').filter(|w| !w.is_empty()).collect();
    let mut extra: Vec<String> = Vec::new();
    // "Paul's First Letter to the Corinthians" → "1 corinthians";
    // "Peter's Second Letter" → "2 peter"; "The Letter from James" → "james".
    if let Some(at) = all.iter().position(|w| {
        matches!(
            *w,
            "letter" | "epistle" | "brief" | "epitre" | "epistola" | "carta" | "epistula"
        )
    }) {
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
        // (The Portuguese "a" only in a title of three words or more: "A
        // Revelação a João", "A Carta de Tiago"; "A Song", "A Psalm" over a
        // psalm are its superscription, never Song of Songs or Psalms.)
        let take = NOUNS.contains(&first)
            || (at_start && ARTICLES.contains(&first) && (first != "a" || words.len() >= 3))
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
    // A number after the name ("Ad Corinthios I", "Regum primus"): a word
    // or a Roman numeral, never a figure ("Jó 1" is Job's chapter 1).
    if number.is_none() && words.len() >= 2 {
        if let Some((_, n)) = words.last().and_then(|last| {
            ORDINALS
                .iter()
                .find(|(w, _)| w == last && !w.chars().all(|c| c.is_ascii_digit()))
        }) {
            number = Some(*n);
            words.pop();
            changed = true;
        }
    }
    // (Nothing taken off: the first-word books and the words after a name
    // are still looked at, "Lamentações de Jeremias".)
    let _ = changed;
    if words.is_empty() {
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

/// A word without the h between two vowels ("samuhelis" → "samuelis").
fn without_medial_h(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let vowel = |c: char| "aeiouy".contains(c);
    chars
        .iter()
        .enumerate()
        .filter(|&(i, &c)| {
            !(c == 'h'
                && i > 0
                && i + 1 < chars.len()
                && vowel(chars[i - 1])
                && vowel(chars[i + 1]))
        })
        .map(|(_, &c)| c)
        .collect()
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

/// Words joining a book's title to another name for it ("The First Book of
/// Samuel, Otherwise Called the First Book of the Kings"; "Liber Beresit id
/// est Genesis"), accents folded.
const ALSO_CALLED: &[&str] = &[
    "otherwise called",
    "commonly called",
    "also called",
    "auch genannt",
    "genannt",
    "autrement appele",
    "appele",
    "comunmente llamado",
    "llamado",
    "chamado",
    "chamada",
    "llamada",
    "appelee",
    "nomme",
    "nommee",
    "dit",
    "dite",
    "called",
    "id est",
    "or",
    "oder",
    "ou",
];

/// Whether a title joins two names ("… Otherwise Called …", "… or, the
/// Preacher").
fn gives_second_name(text: &str) -> bool {
    let k = joint_key(text);
    ALSO_CALLED.iter().any(|j| k.contains(&format!(" {j} ")))
}

/// A title's key with commas as spaces, padded, for finding the words
/// that join two names ("Ecclesiastes or, the Preacher").
fn joint_key(text: &str) -> String {
    let k = key(text).replace([',', ';'], " ");
    format!(" {} ", k.split_whitespace().collect::<Vec<_>>().join(" "))
}

/// A subtitle giving a book another name ("Commonly Called the Third Book
/// of the Kings").
fn second_name(text: &str) -> bool {
    let k = key(text);
    text.chars().count() <= 80
        && !text.trim_end().ends_with(['.', '!', '?', ';', ','])
        && ALSO_CALLED.iter().any(|j| k.starts_with(&format!("{j} ")))
}

/// The book a whole heading names, if exactly one (the document's language
/// decides between books that share a name). A title that gives two names
/// ("… Otherwise Called …", "… id est …") is read by the first when it
/// names a book, else by the second.
/// The Vulgate's titles that its Hebrew names, its numbering of the four
/// books of Kings or its inflected spellings make unreadable by the general
/// rules, as the edition prints them (after "Incipit" and "Liber"): an
/// explicit table, read only for Latin, whole titles only. "Regum primus"
/// alone is not here: in the Vulgate it is 1 Samuel, in the Nova Vulgata
/// 1 Kings, and the title names no other book to tell them apart.
const VULGATE_TITLES: &[(&str, &str)] = &[
    ("iosue bennun id est iesu nave", "JOS"),
    ("malachim id est regum tertius et quartus", "1KI"),
    ("malachim id est regum tertius", "1KI"),
    ("malachim id est regum quartus", "2KI"),
    ("dabreiamin", "1CH"),
    ("secundus paralipomenon", "2CH"),
    ("hiezechielis prophetae", "EZK"),
    ("actuum apostolorum", "ACT"),
];

/// A title of the Vulgate's own (see `VULGATE_TITLES`).
fn vulgate_title(text: &str) -> Option<&'static Book> {
    let mut k = key(text);
    for word in ["incipit ", "liber "] {
        if let Some(rest) = k.strip_prefix(word) {
            k = rest.to_string();
        }
    }
    VULGATE_TITLES
        .iter()
        .find(|(title, _)| *title == k)
        .and_then(|(_, code)| canon::by_code(code))
}

fn book_named(text: &str, language: Option<ReferenceLanguage>) -> Option<&'static Book> {
    if language.is_some_and(|l| l.code() == "la") {
        if let Some(book) = vulgate_title(text) {
            return Some(book);
        }
    }
    let padded = joint_key(text);
    for joint in ALSO_CALLED {
        let pattern = format!(" {joint} ");
        if let Some(at) = padded.find(&pattern) {
            let (before, after) = (&padded[..at], &padded[at + pattern.len()..]);
            // Exact names only: a name one letter away ("Malachim", the
            // Hebrew name of Kings, is not "Malachi") is never taken when
            // the title has another name to read.
            return book_named_once(before.trim(), language, false)
                .or_else(|| book_named_once(after.trim(), language, false));
        }
    }
    book_named_once(text, language, true)
}

/// The book outside the 66 a title names ("O Primeiro Livro dos Macabeus" →
/// 1MA), and the book that must have begun before it counts as one.
fn outside_named(text: &str) -> Option<(&'static str, Option<&'static str>)> {
    let mut k = key(text);
    for prefix in TITLE_PREFIXES {
        if let Some(rest) = k.strip_prefix(prefix) {
            k = rest.to_string();
            break;
        }
    }
    let mut forms = title_forms(&k);
    forms.insert(0, k);
    // "A Sabedoria de Salomão", "The Wisdom of Solomon".
    let bare: Vec<String> = forms
        .iter()
        .filter_map(|f| {
            [
                "a ", "o ", "the ", "la ", "le ", "el ", "die ", "das ", "der ",
            ]
            .iter()
            .find_map(|article| f.strip_prefix(article).map(str::to_string))
        })
        .collect();
    forms.extend(bare);
    forms.iter().find_map(|form| {
        OUTSIDE_TITLES
            .iter()
            .find(|(name, _, _)| key(name) == *form)
            .map(|(_, code, after)| (*code, *after))
    })
}

fn book_named_once(
    text: &str,
    language: Option<ReferenceLanguage>,
    near: bool,
) -> Option<&'static Book> {
    let mut k = key(text);
    // A title is a name, never one letter or syllable ("마", "시": Korean
    // abbreviations used in references only).
    if k.chars().filter(|c| c.is_alphabetic()).count() < 2 {
        return None;
    }
    for prefix in TITLE_PREFIXES {
        if let Some(rest) = k.strip_prefix(prefix) {
            k = rest.to_string();
            break;
        }
    }
    let mut forms = title_forms(&k);
    // Latin titles name a book in another case than the dictionary's
    // ("Secundum Iohannem", "Danihelis prophetae"): the dictionary has it
    // with its noun ("Evangelium secundum Ioannem", "Prophetia Danielis").
    // Medieval spelling writes an h between vowels ("Samuhelis",
    // "Iohannis").
    if language.is_some_and(|l| l.code() == "la") {
        let plain: Vec<String> = forms.iter().map(|f| without_medial_h(f)).collect();
        forms.extend(plain);
        let latin: Vec<String> = forms
            .iter()
            .flat_map(|f| {
                ["evangelium secundum", "prophetia", "epistula", "liber"]
                    .iter()
                    .map(move |noun| format!("{noun} {f}"))
            })
            .collect();
        forms.extend(latin);
    }
    // Greek titles in their traditional shape ("ΚΟΡΙΝΘΙΟΥΣ ΕΠΙΣΤΟΛΗ ΠΡΩΤΗ",
    // "ΠΕΤΡΟΥ ΕΠΙΣΤΟΛΗ ΚΑΘΟΛΙΚΗ ΠΡΩΤΗ", "ΕΥΑΓΓΕΛΙΟΝ ΤΟ ΚΑΤΑ ΜΑΤΘΑΙΟΝ"):
    // the forms of the Greek titles table.
    forms.extend(greek_forms(&k));
    let found = match names().get(&k) {
        Some(found) => found,
        None => match forms.iter().find_map(|f| names().get(f)) {
            Some(found) => found,
            // Only for a line printed as a title ("Das erste Buch der …").
            None if near => forms.iter().find_map(|f| near_name(f))?,
            None => return None,
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
            // Names that differ only by an accent ("Jó", Job; "Jo", João):
            // the document's language as written decides.
            // The name is the title's last words ("O Livro de Jó" → "Jó").
            let words: Vec<&str> = text.split_whitespace().collect();
            let written: Vec<String> = (1..=words.len())
                .map(|n| normalise(&words[words.len() - n..].join(" ")))
                .collect();
            let exact: Vec<&'static Book> = ReferenceParser::for_language(language)
                .lookup_entries()
                .filter(|(name, _)| written.contains(&normalise(name)))
                .map(|(_, b)| b)
                // (Only among the books the title can name.)
                .filter(|b| books.iter().any(|c| c.code == b.code))
                .collect();
            if let [one, rest @ ..] = exact.as_slice() {
                if rest.iter().all(|b| b.code == one.code) {
                    return Some(one);
                }
            }
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
    // A title that gives a second name may be longer.
    let limit = if gives_second_name(text) { 100 } else { 60 };
    if text.is_empty() || text.chars().count() > limit {
        return None;
    }
    let lower = key(text);
    let words: Vec<&str> = lower.split(' ').collect();
    // "Psaume 118 (Vulg. CXVII)": a chapter's label with another numbering
    // in brackets after it (v0.24.3).
    if let Some((head, _)) = text.strip_suffix(')').and_then(|t| t.rsplit_once('(')) {
        let head = head.trim();
        let hw: Vec<String> = key(head).split(' ').map(str::to_string).collect();
        if hw.len() == 2 && CHAPTER_WORDS.contains(&hw[0].as_str()) {
            if let Some(n) = head.split_whitespace().last().and_then(chapter_number) {
                return Some(Heading::Chapter(n));
            }
        }
    }
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
    /// A raised number set against the word before it ("Erde⁶Und"): a verse
    /// number or a note's number; the boundary it makes is not verified.
    against_word: bool,
    /// Set as a superscript (raised digits).
    raised: bool,
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

/// The older three-letter book codes of verse-per-line exports (eBible's
/// VPL files, and the Bible programs whose format they follow) where they
/// differ from USFM's (v0.24.3).
fn legacy_code(code: &str) -> Option<&'static Book> {
    let usfm = match code {
        "SOL" => "SNG",
        "EZE" => "EZK",
        "JOE" => "JOL",
        "NAH" => "NAM",
        "MAR" => "MRK",
        "JOH" => "JHN",
        "PHI" => "PHP",
        "JAM" => "JAS",
        "1JO" => "1JN",
        "2JO" => "2JN",
        "3JO" => "3JN",
        _ => return None,
    };
    canon::by_code(usfm)
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
    // A book's code as USFM writes it, in capitals ("1SA 1:1 …", "SNG 2:3
    // …"), as Bible exports one verse per line do.
    let code = text.split(' ').next().filter(|w| {
        w.len() == 3
            && w.chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
            && w.chars().any(|c| c.is_ascii_uppercase())
    });
    if let Some(found) = code.and_then(|c| canon::by_code(c).or_else(|| legacy_code(c))) {
        let after = &text[3..];
        if after.starts_with(' ') && after[1..].starts_with(|c: char| c.is_ascii_digit()) {
            book = Some(found);
            rest = &after[1..];
            taken = 4;
        }
    }
    if book.is_none()
        && (!text.starts_with(|c: char| c.is_ascii_digit())
            || text.starts_with(|c: char| c.is_ascii_digit())
                && text.chars().nth(1).is_some_and(|c| c == ' '))
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
            against_word: false,
            raised: false,
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
            // A raised number against the word before it ("Kain,⁵Und",
            // "Erde⁶Und"): a verse number written without a space (v0.24.3),
            // or a note's number — only ever the very next verse, and a
            // better place for the same verse wins (`best_place`).
            let raised_after_word = raised && prev.is_some_and(|p| p.is_alphanumeric());
            if prev.is_some_and(|p| p.is_alphanumeric()) && !raised {
                // Glued to a word: "God1", "v2".
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
            // After an abbreviation that names a verse or page ("in V. 2 und
            // nach V. 11", "cf. Ps. 23"): the full stop ends the abbreviation,
            // not a sentence; a reference, never a verse number.
            let abbreviation = !raised && prev_solid == Some('.') && {
                let word = before
                    .trim_end()
                    .trim_end_matches('.')
                    .rsplit(|c: char| c.is_whitespace() || c == '(')
                    .next()
                    .unwrap_or("");
                ["v", "vv", "vgl", "cf"].contains(&key(word).as_str())
            };
            if abbreviation {
                return None;
            }
            let strength = if raised_after_word {
                Strength::Medium
            } else if at_start || raised || prev_solid.is_some_and(ends_sentence) || opened {
                Strength::Strong
            } else if prev.is_some_and(char::is_whitespace) {
                Strength::Medium
            } else {
                return None;
            };
            // A line that is only a word in capitals and a number ("BOOK 2",
            // "LIVRO 3", "PART 1"): a label of the book's divisions.
            if strength == Strength::Medium
                && chars[next_index..].iter().all(|(_, c)| c.is_whitespace())
                && {
                    let word = before.trim();
                    word.chars().count() >= 2 && word.chars().all(char::is_uppercase)
                }
            {
                return None;
            }
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
                    let after: String = chars[next_index..]
                        .iter()
                        .take(3)
                        .map(|(_, c)| *c)
                        .collect();
                    let mut a = after.chars();
                    if matches!(a.next(), Some(':' | ',' | '.'))
                        && a.next().is_some_and(|c| c.is_ascii_digit())
                    {
                        return None;
                    }
                }
            }
            // A numbered book's name with its chapter and verse ("2 Kön.
            // 8,17-22", "1 Cor 3:4"): a reference, never a verse number.
            if !raised && (1..=4).contains(&value) {
                let named = numbered_reference();
                let after: String = chars[next_index..]
                    .iter()
                    .take(24)
                    .map(|(_, c)| *c)
                    .collect();
                if let Some(c) = named.captures(&after) {
                    let name = format!("{value} {}", c[1].trim_end_matches('.'));
                    if book_named(&name, language).is_some() {
                        return None;
                    }
                }
            }
            // What comes after.
            let mut k = next_index;
            let mut end = None;
            if raised {
                // A raised bridge: "⁴-⁵", "⁴–⁵".
                if chars.get(k).is_some_and(|(_, d)| matches!(d, '-' | '–')) {
                    let mut m = k + 1;
                    while m < chars.len() && superscript_digit(chars[m].1).is_some() {
                        m += 1;
                    }
                    if m > k + 1 && m - (k + 1) <= 3 {
                        let last = chars[k + 1..m]
                            .iter()
                            .filter_map(|(_, c)| superscript_digit(*c))
                            .fold(0u32, |v, d| v * 10 + d)
                            as u16;
                        if last > value && last - value <= 10 {
                            end = Some(last);
                            k = m;
                        }
                    }
                }
            } else {
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
                // "¹⁵...Katath": a raised number before an ellipsis.
                Some('.' | ',' | ':' | ';') if raised => {}
                // "1. In the beginning": a numbered verse.
                Some('.') if chars.get(k + 1).is_none_or(|(_, d)| d.is_whitespace()) => k += 1,
                Some('%' | '°' | '/' | ':' | ',' | '.' | '€' | '$' | '£') => return None,
                // Korean units: "3장", "4절", "23편" are references, not verse
                // numbers — unless the number is raised: "¹⁴장래에 …" is verse
                // 14 beginning with 장래 (v0.24.3).
                Some('장' | '절' | '편') if !raised => return None,
                // A raised number is set off from the word after it by its
                // look, not by a space ("⁵auf Kain").
                Some(c) if c.is_alphabetic() && raised => {}
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
                // A raised number before a mark the edition sets at a verse's
                // start ("⁴*Ce sont …", v0.24.3).
                Some(_) if raised => {}
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
                    against_word: raised_after_word,
                    raised,
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
    // A line the layout sets at a page's edge, not large, with the same
    // wording on three pages or more ("GÉNESIS" over each page of Genesis):
    // a running head, however few of the document's pages it runs through
    // (v0.24.3).
    let mut edge_counts: HashMap<String, usize> = HashMap::new();
    for lines in pages.values() {
        let mut seen = HashSet::new();
        for &i in lines {
            let u = &units[i];
            // (Without numbers: those are counted by their form above, and a
            // chapter's label, "Capítulo 3", may open a page.)
            if u.edge
                && !u.large
                && head_like(&u.text)
                && numbered_name_only(&u.text)
                && seen.insert(shape(&u.text))
            {
                *edge_counts.entry(shape(&u.text)).or_default() += 1;
            }
        }
    }
    for ((_, page), lines) in &pages {
        for &i in lines {
            let u = &units[i];
            if u.edge
                && !u.large
                && head_like(&u.text)
                && edge_counts.get(&shape(&u.text)).is_some_and(|n| *n >= 3)
            {
                out.insert(i);
            }
        }
        // A line the page's layout sets apart at its top or bottom edge that
        // names a passage (v0.24.3): the running head or page
        // number, whatever its wording ("Le premier livre de Samuel 1:1 364
        // Le premier livre de Samuel 1:17"). A title set at a page's top
        // names no chapter and verse, and stays.
        for &i in lines {
            if units[i].edge && edge_furniture(&units[i].text) {
                out.insert(i);
            }
        }
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

/// Whether a line has no number but a book's own ("1 SAMUEL", "GÉNESIS";
/// not "Capítulo 3").
fn numbered_name_only(text: &str) -> bool {
    let text = text.trim();
    let rest = match text.split_once(char::is_whitespace) {
        Some((n, rest)) if matches!(n, "1" | "2" | "3" | "4") => rest,
        _ => text,
    };
    !rest.chars().any(|c| c.is_ascii_digit())
}

/// What a line set apart at a page's edge says when it is page furniture: a
/// passage reference with at most a few words and numbers around it (a
/// running head), never a sentence.
fn edge_furniture(text: &str) -> bool {
    use std::sync::OnceLock;
    static PAIR: OnceLock<regex::Regex> = OnceLock::new();
    // A name, then a chapter and verse ("Samuel 1:17", "Α΄ 4:13").
    let pair = PAIR.get_or_init(|| {
        // (With a full stop, a verse of one or two digits: "Judah 74.600" is
        // a number.)
        regex::Regex::new(r"\p{L}[\p{L}'’.\-΄ʹ]*\s+\d{1,3}(?::\d{1,3}|\.\d{1,2}(?:\D|$))")
            .expect("valid")
    });
    // (A number alone is a page number only when it runs with the pages:
    // a chapter's number may open a page; see `page_furniture`.)
    let text = text.trim();
    // (Or a name written twice, each with its number: "요한2서 1 780 요한2서
    // 13", the first and last chapter or verse of the page.)
    let named_twice = || {
        let words: Vec<&str> = text.split_whitespace().collect();
        let numbers = words
            .iter()
            .filter(|w| w.chars().all(|c| c.is_ascii_digit()))
            .count();
        numbers >= 2
            && words
                .iter()
                .enumerate()
                .any(|(i, w)| w.chars().any(char::is_alphabetic) && words[i + 1..].contains(w))
    };
    // A running head has no clause punctuation and ends with a number (its
    // last passage or the page's number).
    if text.chars().count() > 120
        || !(pair.is_match(text) || named_twice())
        || head_sentence(text)
        || text.contains([',', ';', '!', '?', '«', '»', '“', '”'])
        || !text.ends_with(|c: char| c.is_ascii_digit())
    {
        return false;
    }
    let words = text
        .split_whitespace()
        .filter(|w| w.chars().any(char::is_alphabetic))
        .count();
    words <= 14
}

/// Whether a verse ends, after its last sentence, with a book's name alone
/// ("… o Senhor me mandou. Zacarias", "… cantarei.” 2 Samuel"): the source
/// of a quotation whose chapter and verse were read elsewhere (v0.24.3).
fn trailing_book_name(text: &str, language: Option<ReferenceLanguage>) -> bool {
    let words: Vec<&str> = text.split_whitespace().collect();
    (1..=3).any(|n| {
        n < words.len()
            && ends_as_sentence(words[words.len() - n - 1])
            && words[words.len() - n..]
                .iter()
                .all(|w| !w.ends_with(['.', '!', '?', ',', ';', ':']))
            && {
                let tail = words[words.len() - n..].join(" ");
                tail.chars()
                    .find(|c| c.is_alphabetic())
                    .is_some_and(char::is_uppercase)
                    && book_named(&tail, language)
                        .is_some_and(|_| read_heading(&tail, language).is_some())
            }
    })
}

/// A name, then chapter and verse, after a number ("2 Kön. 8,17").
fn numbered_reference() -> &'static regex::Regex {
    use std::sync::OnceLock;
    static NAMED: OnceLock<regex::Regex> = OnceLock::new();
    NAMED.get_or_init(|| {
        regex::Regex::new(r"^\s*(\p{L}[\p{L}'’]*\.?)\s*\d{1,3}[:,.]\d").expect("valid")
    })
}

/// The line a verse's words come from, when they are the whole line, or the
/// first piece of a short line (100 characters at most) that a number in it
/// splits (`first`: the verse has had nothing from this line yet).
fn whole_line<'a>(whole: &'a str, words: &'a str, first: bool) -> Option<&'a str> {
    let (whole, words) = (whole.trim(), words.trim());
    if whole == words {
        Some(words)
    } else if first && !words.is_empty() && whole.chars().count() <= 100 && whole.starts_with(words)
    {
        Some(whole)
    } else {
        None
    }
}

/// Whether Korean text ends with a sentence's final ending ("… 하시니라").
fn korean_end(text: &str) -> bool {
    text.chars().last().is_some_and(|c| {
        matches!(
            c,
            '다' | '라' | '요' | '까' | '니' | '서' | '오' | '네' | '지' | '세' | '자'
        )
    })
}

/// Whether a line ends as a sentence does (closing marks aside).
fn ends_as_sentence(line: &str) -> bool {
    line.trim_end()
        .trim_end_matches(is_sign)
        .trim_end_matches(['”', '’', '"', '»', ')', ']', '\''])
        .ends_with(['.', '!', '?', ';', ':', '…', '。'])
}

/// What in a verse's text puts it in doubt (v0.24.3): a passage reference
/// ("Samuel 1:17": Scripture never writes book, chapter and verse; a running
/// head or a cross-reference read in), or a note's sign that no note on its
/// page explained ("†", "‡", "§").
fn verse_doubt(text: &str, labels: &HashSet<String>) -> Option<&'static str> {
    // A heading or a speaker's name the document writes inside the verse's
    // line: a word in capitals closing it after a sentence's end ("… forsake
    // me. BETH"), or a label of the document's own after one ("… rooms.
    // Friends We will be glad …"; see `inline_labels`).
    let words: Vec<&str> = text.split_whitespace().collect();
    let sentence_end = |w: &str| w.ends_with(['.', '!', '?', ';', ':', '”', '’', '"', '»']);
    // (Several words in capitals too: "… à sa place. LES ROIS DE JUDA ET
    // LES ROIS D’ISRAËL,"; and a number alone after a sentence's end, "…
    // n’y entrera point. 30.")
    let capitals = |w: &str| {
        w.chars().filter(|c| c.is_alphabetic()).count() >= 2
            && !w.chars().any(char::is_lowercase)
            && w.chars().any(char::is_uppercase)
    };
    // A word in capitals opening the verse before a capitalised word, without
    // punctuation ("ALEPH Blessed are …"): a heading written into its line.
    if let [first, second, ..] = words[..] {
        if first.chars().count() >= 3
            && first
                .chars()
                .all(|c| c.is_alphabetic() && !c.is_lowercase())
            && second.chars().next().is_some_and(char::is_uppercase)
            && second.chars().skip(1).any(char::is_lowercase)
        {
            return Some("possibleHeading");
        }
    }
    let run = words.iter().rev().take_while(|w| capitals(w)).count();
    if run >= 1 && run < words.len() && sentence_end(words[words.len() - run - 1]) {
        let last = words[words.len() - 1];
        let one = last
            .trim_end_matches(['.', ',', ';', ':'])
            .chars()
            .all(|c| c.is_alphabetic() && !c.is_lowercase());
        if run >= 2 || one {
            return Some("possibleHeading");
        }
    }
    // A short abbreviation closing the verse after a sentence's end ("… que
    // Deus me deu. Ref."): a note's or cross-reference's label.
    if let [.., before, last] = words[..] {
        let letters = last.trim_end_matches('.');
        if last.ends_with('.')
            && (2..=3).contains(&letters.chars().count())
            && letters.chars().all(char::is_alphabetic)
            && letters.chars().next().is_some_and(char::is_uppercase)
            && letters.chars().skip(1).all(char::is_lowercase)
            && sentence_end(before)
        {
            return Some("reference");
        }
    }
    if let [.., before, last] = words[..] {
        // (Or with a closing bracket: "… dividing up the land. 14)", the end
        // of a reference read in.)
        let bracketed = last.ends_with(')')
            && last.len() <= 4
            && last
                .trim_end_matches(')')
                .chars()
                .all(|c| c.is_ascii_digit())
            && last.len() > 1;
        if bracketed && sentence_end(before) {
            return Some("reference");
        }
        let number = last.ends_with('.')
            && last.len() <= 4
            && last
                .trim_end_matches('.')
                .chars()
                .all(|c| c.is_ascii_digit())
            && last.len() > 1;
        if number && sentence_end(before) {
            return Some("possibleHeading");
        }
    }
    if !labels.is_empty()
        && words.windows(3).any(|w| {
            sentence_end(w[0])
                && labels.contains(w[1])
                && w[2].chars().next().is_some_and(char::is_uppercase)
        })
    {
        return Some("possibleHeading");
    }
    use std::sync::OnceLock;
    static REFERENCE: OnceLock<regex::Regex> = OnceLock::new();
    let reference = REFERENCE.get_or_init(|| {
        regex::Regex::new(r"(?:^|\s)\p{L}[\p{L}.΄ʹ]*\s\d{1,3}:\d{1,3}(?:\D|$)").expect("valid")
    });
    // The end of a line of references read into a verse ("… sabbat. 11-15.)").
    static RANGE: OnceLock<regex::Regex> = OnceLock::new();
    let range = RANGE.get_or_init(|| {
        regex::Regex::new(r"(?:^|\s)\(?\d{1,3}\s?[-–]\s?\d{1,3}\.?\)").expect("valid")
    });
    // With a full stop or a comma ("Isaías 7.14", "Samuel 1,17"): a name
    // with a capital and a verse of one or two digits, never a number such as
    // "de 1.000".
    static DOTTED: OnceLock<regex::Regex> = OnceLock::new();
    let dotted = DOTTED.get_or_init(|| {
        regex::Regex::new(r"(?:^|\s)\p{Lu}[\p{L}.΄ʹ]*\s\d{1,3}[.,]\d{1,2}(?:[^\d]|$)")
            .expect("valid")
    });
    // A bracket holding a chapter and verse ("(Vergl. ff. 96,1b-13a …)",
    // "(Auszug aus 1 Mos. 36,1 ff.)"): a cross-reference read in. ("2,000"
    // has three digits after its comma, and is a number.)
    static BRACKET: OnceLock<regex::Regex> = OnceLock::new();
    let bracket = BRACKET.get_or_init(|| {
        regex::Regex::new(r"\([^()]*\d{1,3}[:.,]\d{1,2}(?:[^\d][^()]*)?\)").expect("valid")
    });
    if reference.is_match(text)
        || range.is_match(text)
        || dotted.is_match(text)
        || bracket.is_match(text)
    {
        return Some("reference");
    }
    // A raised number left in the verse (a note's number) or a sign.
    // A note's letter left against the sentence's end ("Louez Jah.a b
    // Bienheureux"): a label whose raised type the page lost.
    // An editor's note in square brackets, opened by an abbreviation ("zehn[O.
    // zehn seiner]Knechte": "oder", "wörtlich", "eigentlich").
    static EDITORIAL: OnceLock<regex::Regex> = OnceLock::new();
    let editorial =
        EDITORIAL.get_or_init(|| regex::Regex::new(r"\[\p{Lu}\p{Ll}{0,3}\.\s").expect("valid"));
    if editorial.is_match(text) {
        return Some("noteSign");
    }
    static LETTER: OnceLock<regex::Regex> = OnceLock::new();
    let letter =
        LETTER.get_or_init(|| regex::Regex::new(r#"[.!?;:,”")]\p{Ll}(?:\s|$)"#).expect("valid"));
    // (After a single quote closing a sentence too, "… adultery.’i"; never
    // an elision, "il l’a dit".)
    static QUOTE_LETTER: OnceLock<regex::Regex> = OnceLock::new();
    let quote_letter =
        QUOTE_LETTER.get_or_init(|| regex::Regex::new(r"[.!?,;:]’\p{Ll}(?:\s|$)").expect("valid"));
    // (Or a lone letter closing the verse after its sentence's end, "… si
    // me equivoco?” b", that no language writes as a word there.)
    let lone_letter = matches!(words[..], [.., before, last]
        if sentence_end(before)
            && last.chars().count() == 1
            && last.chars().all(|c| c.is_lowercase())
            && !matches!(last, "a" | "à" | "e" | "é" | "o" | "y" | "u" | "i"));
    if text.chars().any(|c| superscript_digit(c).is_some())
        || text.contains('*')
        || letter.is_match(text)
        || quote_letter.is_match(text)
        || lone_letter
    {
        return Some("noteSign");
    }
    if text.contains(['†', '‡', '§']) {
        return Some("noteSign");
    }
    None
}

/// Whether a line reads as a sentence: a word of three letters or more
/// ending with sentence punctuation before more words.
fn head_sentence(text: &str) -> bool {
    let words: Vec<&str> = text.split_whitespace().collect();
    words.windows(2).any(|w| {
        w[0].ends_with(['.', '!', '?', ';'])
            && w[0].chars().filter(|c| c.is_alphabetic()).count() >= 3
            && w[1].chars().next().is_some_and(char::is_alphabetic)
    })
}

/// Footnotes printed at the foot of a page ("† 1:1 Messiah (Hebrew) …",
/// "* 2:6 Micah 5:2"): from the first line that starts with a note sign and a
/// chapter and verse to the end of that page. They are notes, not verses.
fn page_footnotes(units: &[Unit]) -> HashSet<usize> {
    use std::sync::OnceLock;
    static NOTE: OnceLock<regex::Regex> = OnceLock::new();
    let note = NOTE.get_or_init(|| {
        regex::Regex::new(r#"^[^\p{L}\p{N}\s.,;:!?'"()\[\]«»„“”‘’\-–—/\x{E000}\x{E001}]+\s*\d{1,3}[:,.]\d{1,3}(?:\s|$|\p{L})"#).expect("valid pattern")
    });
    let mut out = HashSet::new();
    let mut in_notes: Option<(u16, u32)> = None;
    for (i, unit) in units.iter().enumerate() {
        let Some(page) = unit.place.page else {
            // Without pages (EPUB, Word, text): a passage that begins as a
            // note does ("*1:1 A palavra …") is one. (Some PDFs print notes
            // without spaces: "† 2:6there…: or, amist…".)
            in_notes = None;
            if !unit.heading && note.is_match(unit.text.trim_start()) {
                out.insert(i);
            }
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

/// A character that can label a note: the usual signs, and any other symbol
/// a document uses so ("✡ 12:4 …") — never a letter, a digit, a space or
/// ordinary punctuation. A sign is only ever taken out of a verse where a
/// note labelled with it names that verse.
fn is_sign(c: char) -> bool {
    NOTE_SIGNS.contains(&c)
        || !(c.is_alphanumeric()
            || c.is_whitespace()
            || c.is_control()
            || c == RAISED_OPEN
            || c == RAISED_CLOSE
            || ".,;:!?'\"()[]«»„“”‘’-–—/…".contains(c))
}

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
        regex::Regex::new(r#"(?:^|\s)([^\p{L}\p{N}\s.,;:!?'"()\[\]«»„“”‘’\-–—/\x{E000}\x{E001}])[^\p{L}\p{N}\s.,;:!?'"()\[\]«»„“”‘’\-–—/\x{E000}\x{E001}]*\s*([0-9]{1,3})[:,.]([0-9]{1,3})"#).expect("valid pattern")
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
        regex::Regex::new(r"(?:^|\s)(\p{Ll}{1,2})\s+([0-9]{1,3})[:,.]([0-9]{1,3})")
            .expect("valid pattern")
    });
    // "g 9:542R1:10" (the space before "2 R" lost in the text): every
    // reading of the digits that can be a verse number, as for note signs.
    form.captures_iter(text)
        .filter_map(|c| {
            let label = c.get(1)?.as_str().to_string();
            let chapter: u16 = c.get(2)?.as_str().parse().ok()?;
            let digits = c.get(3)?.as_str();
            Some(
                (1..=digits.len())
                    .rev()
                    .filter_map(|len| digits[..len].parse::<u16>().ok())
                    .filter(|v| (1..=176).contains(v))
                    .map(|v| (label.clone(), chapter, v))
                    .collect::<Vec<_>>(),
            )
        })
        .flatten()
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
    let sign = text.chars().next().filter(|c| is_sign(*c))?;
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
        if !is_sign(chars[i]) {
            out.push(chars[i]);
            i += 1;
            continue;
        }
        // A run of signs ("*‡"), as one: raised against the word before it,
        // set apart as a word of its own, or at the very start.
        let mut end = i;
        while end < chars.len() && is_sign(chars[end]) {
            end += 1;
        }
        let previous = i.checked_sub(1).map(|p| chars[p]);
        let next = chars.get(end).copied();
        let raised = previous.is_some_and(|p| !p.is_whitespace());
        let alone =
            previous.is_none_or(char::is_whitespace) && next.is_none_or(char::is_whitespace);
        let start = previous.is_none();
        // Or set before the word it marks ("Χριστοῦ, †ᾧ ἡ δόξα", Textus
        // Receptus): the note on the page names this verse either way.
        let before_word =
            previous.is_some_and(char::is_whitespace) && next.is_some_and(|n| !n.is_whitespace());
        if raised || alone || start || before_word {
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
        let reference = r"(?:\d\.?\s?)?[\p{L}][\p{L}'’.\-΄ʹ]*(?:\s+[\p{L}][\p{L}'’.\-΄ʹ]*){0,6}\s+\d{1,3}[:,.]\d{1,3}(?:\s*[-–—]\s*(?:\d{1,3}[:,.])?\d{1,3})?";
        regex::Regex::new(&format!(
            r"^(?:\d{{1,4}}\s+)?{reference}(?:\s+\d{{1,4}})?(?:\s+{reference})?(?:\s+\d{{1,4}})?$"
        ))
        .expect("valid pattern")
    });
    // A book of one chapter: verse numbers alone, the same name twice and the
    // page number between ("Obadja 1 972 Obadja 19", "Judas 21 1320 Judas 25").
    static SINGLE: OnceLock<regex::Regex> = OnceLock::new();
    let single = SINGLE.get_or_init(|| {
        let name = r"((?:\d\.?\s?)?[\p{L}][\p{L}'’.\-΄ʹ]*(?:\s+[\p{L}][\p{L}'’.\-΄ʹ]*){0,6})";
        regex::Regex::new(&format!(
            r"^{name}\s+\d{{1,3}}\s+\d{{1,4}}\s+{name}\s+\d{{1,3}}$"
        ))
        .expect("valid pattern")
    });
    let text = text.trim();
    if text.chars().count() > 120 {
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
        // (Or a speaker's label of up to three words with its colon:
        // "Mujer:", "Mujeres de Jerusalén:", v0.24.3.)
        let colon = text.ends_with(':');
        let bare = text.trim_end_matches(':');
        let shape = (1..=if colon { 3 } else { 2 }).contains(&words)
            && bare.chars().count() <= 24
            && bare.chars().next().is_some_and(|c| c.is_uppercase())
            && bare.chars().all(|c| c.is_alphabetic() || c == ' ');
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
        regex::Regex::new(r"(?:^|\s)((?:\d\.?\s?)?[\p{L}][\p{L}'’.\-΄ʹ]*(?:\s+[\p{L}][\p{L}'’.\-΄ʹ]*){0,6})\s+(\d{1,3})(?:[:,.](\d{1,3}))?")
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
    let first = found.first()?.clone();
    let mut last = found.last()?.clone();
    // A range whose end names no book ("GENESIS 8:13–9:15", "Ruth 2:4-23"):
    // the same book, to the chapter and verse after the dash (v0.24.3).
    static RANGE_END: OnceLock<regex::Regex> = OnceLock::new();
    let range_end = RANGE_END.get_or_init(|| {
        regex::Regex::new(
            r"\d[:,.]\d{1,3}\s*[–-]\s*(\d{1,3})(?:[:,.](\d{1,3}))?(?:\s+\d{1,4})?\s*$",
        )
        .expect("valid pattern")
    });
    if let Some(c) = range_end.captures(text.trim()) {
        let a: Option<u16> = c.get(1).and_then(|m| m.as_str().parse().ok());
        let b: Option<u16> = c.get(2).and_then(|m| m.as_str().parse().ok());
        match (a, b) {
            (Some(chapter), Some(verse)) => last = (last.0, chapter, verse),
            (Some(verse), None) => last = (last.0.clone(), last.1, verse),
            _ => {}
        }
    }
    Some((first, last))
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
        // (A passage in brackets at a page's top is a parallel reference
        // under a heading, "(Isaiah 66:1-2)", not the page's head.)
        if heads.contains_key(&(unit.place.document, page))
            || !reference_head(&unit.text)
            || unit.text.trim_start().starts_with('(')
        {
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
            // (A range within one chapter, "Isaiah 66:1-2", is one passage.)
            first_found.0 == book.ordinal || first.0 == *chapter
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
    let limit = if gives_second_name(text) { 100 } else { 60 };
    // A sentence ending inside the line ("Moïse. Ce dernier changea son nom
    // Osée ou …") is a wrapped line of prose, never a title.
    let sentence_inside = text.split(". ").collect::<Vec<_>>().windows(2).any(|w| {
        let word = w[0].rsplit(' ').next().unwrap_or("");
        word.chars().count() >= 3
            && word.chars().all(|c| c.is_alphabetic())
            && word.chars().skip(1).all(char::is_lowercase)
            && w[1].chars().next().is_some_and(char::is_uppercase)
    });
    !sentence_inside
        && text.chars().count() <= limit
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
    /// PDF lines that begin a block of text of their own after a gap, not
    /// at the top of their page (see `block_starts`).
    block_starts: HashSet<usize>,
}

impl Builder<'_> {
    fn found_book(&mut self, heading: &str, detected: Option<&'static Book>, unit: usize) -> usize {
        self.books.push(FoundBook {
            index: self.books.len(),
            heading: unmark_raised(heading),
            detected: detected.map(|b| b.code),
            book: detected.map(|b| b.code),
            left_out: false,
            outside: None,
            chapters: 0,
            verses: 0,
            place: self.units[unit].place,
            corrected: false,
        });
        self.last_detected = detected.map(|b| b.code);
        self.books.len() - 1
    }

    /// A book outside the 66 that a title names: one of `OUTSIDE_TITLES`,
    /// Psalm 151, or the Greek text of Esther or Daniel printed as a book of
    /// its own (see `greek_part`). A line that is not set apart as a title
    /// must open its chapter 1, as any book's title must.
    fn outside_title(
        &self,
        unit: &Unit,
        text: &str,
        index: usize,
        skip: &HashSet<usize>,
    ) -> Option<&'static str> {
        if !book_line_shape(text) {
            return None;
        }
        let code = match outside_named(text) {
            Some((code, after)) => {
                if after.is_some_and(|after| !self.books.iter().any(|b| b.detected == Some(after)))
                {
                    return None;
                }
                code
            }
            None => match read_heading(text, self.language)? {
                Heading::BookChapter(book, 151) if book.code == "PSA" => "PS2",
                Heading::Book(book) => self.greek_part(book, index, skip)?,
                _ => return None,
            },
        };
        (unit.heading || self.opens_chapter(index, skip)).then_some(code)
    }

    /// Esther or Daniel once more, after the book has begun, with "Greek"
    /// in the lines under the title ("Ester / traduzido da Septuaginta
    /// grega", "Daniel / com as Porções Gregas"): the Greek text, a book
    /// outside the 66.
    fn greek_part(
        &self,
        book: &'static Book,
        index: usize,
        skip: &HashSet<usize>,
    ) -> Option<&'static str> {
        let code = match book.code {
            "EST" => "ESG",
            "DAN" => "DAG",
            _ => return None,
        };
        if !self.books.iter().any(|b| b.detected == Some(book.code)) {
            return None;
        }
        let greek = (index + 1..self.units.len())
            .filter(|i| !skip.contains(i))
            .map(|i| key(&clean(&self.units[i].text)))
            .filter(|t| !t.is_empty())
            .take(2)
            .any(|t| {
                t.split(|c: char| !c.is_alphanumeric())
                    .any(|w| GREEK_MARKS.iter().any(|m| w.starts_with(m)))
            });
        greek.then_some(code)
    }

    fn outside_book(&mut self, heading: &str, code: &'static str, unit: usize) -> usize {
        let entry = self.found_book(heading, None, unit);
        self.books[entry].outside = Some(code);
        entry
    }

    /// Whether a heading has the form of two or more titles already read as
    /// books, all but their last word ("Le livre du prophète …").
    fn title_form(&self, text: &str) -> bool {
        let k = key(text);
        let Some((form, _)) = k.rsplit_once(' ') else {
            return false;
        };
        if form.split(' ').count() < 2 {
            return false;
        }
        self.books
            .iter()
            .filter(|b| b.detected.is_some() && key(&b.heading).starts_with(&format!("{form} ")))
            .count()
            >= 2
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
        let own = match read_heading(title.trim(), self.language) {
            Some(Heading::Book(book)) => Some(book.code),
            _ => None,
        };
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
            // (A sentence of an introduction that names a book, "les Actes
            // des Apôtres, relate la vie …", is not a heading.)
            if let Some(heading) = read_heading(&line, self.language)
                .filter(|h| !matches!(h, Heading::Book(_)) || book_line_shape(&line))
            {
                // A section heading that begins with the title's own book
                // ("Josué à la tête du peuple" under "JOSUÉ") is not another title.
                if strong && matches!(heading, Heading::Book(book) if Some(book.code) == own) {
                    continue;
                }
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
                // A sentence of an introduction that opens with a numbered
                // book's name ("2 Samuel présente …") is not verse 2. (Verse 1
                // may begin with one: "1 Pierre, apôtre de Jésus-Christ".)
                let names_book = digits != "1"
                    && rest.starts_with(char::is_whitespace)
                    && matches!(
                        read_heading(&format!("{digits} {first_word}"), self.language),
                        Some(Heading::Book(book)) if book.code.starts_with(|c: char| c.is_ascii_digit())
                    );
                if strong && names_book {
                    continue;
                }
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
            if !strong && !head_like(&line) && !second_name(&line) {
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
        // (A note's sign after the last word does not hide how the line
        // ends: "Praise the LORD!†" closing a psalm is a sentence.)
        let bare = line.trim_end_matches(is_sign).trim_end();
        let shape = line.chars().count() <= 60
            && line.chars().next().is_some_and(char::is_uppercase)
            && !bare.ends_with(['.', '!', '?', ';', ':', ',', '”', '’', '"']);
        let after_sentence = match &self.items[n - 2].event {
            Event::Text(t) => t
                .trim_end()
                .trim_end_matches(is_sign)
                .trim_end()
                .ends_with(['.', '!', '?', '”', '’', '"', ')']),
            _ => true,
        };
        if whole && alone && shape && after_sentence && !is_selah(line) {
            let text = unmark_raised(line);
            self.items[n - 1].event = Event::Other { notes: false, text };
        }
    }

    /// Whether a book has begun (a chapter's label before any book is no
    /// chapter of anything).
    fn book_open(&self) -> bool {
        self.items
            .iter()
            .any(|i| matches!(i.event, Event::Book { .. }))
    }

    /// A title naming one of numbered books without its number
    /// ("Crónicas" over each book of Chronicles): the one that follows the
    /// book before it in the canon's order, if one does. Otherwise none.
    fn next_in_sequence(&self, text: &str) -> Option<Heading> {
        if !book_line_shape(text) || text.chars().count() > 40 {
            return None;
        }
        let last = canon::by_code(self.last_detected?)?;
        // Only once the book before has some of its text: a line right under
        // its title ("1 Crónicas" / "Crónicas") is a subtitle of it.
        let since_title = self
            .items
            .iter()
            .rev()
            .take_while(|i| !matches!(i.event, Event::Book { .. }))
            .any(|i| matches!(i.event, Event::Chapter(_) | Event::Marker(_)));
        if !since_title {
            return None;
        }
        (1..=3u8).find_map(
            |n| match read_heading(&format!("{n} {text}"), self.language) {
                Some(Heading::Book(book)) if book.ordinal == last.ordinal + 1 => {
                    Some(Heading::Book(book))
                }
                _ => None,
            },
        )
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
            // (A first line that ends in the middle of a second name, "… Samuel
            // Otherwise Called the", is joined with the next too.)
            let dangling = gives_second_name(text) && {
                let k = key(text);
                ["the", "der", "die", "das", "le", "la", "el", "o", "a"]
                    .iter()
                    .any(|w| k.ends_with(&format!(" {w}")))
            };
            if !unit.heading
                && (dangling || read_heading(text, self.language).is_none())
                && book_line_shape(text)
            {
                if let Some(next) = (index + 1..self.units.len())
                    .find(|i| !skip.contains(i) && !clean(&self.units[*i].text).trim().is_empty())
                {
                    let second = unmark_raised(&clean(&self.units[next].text));
                    let second = second.trim();
                    let same_page = self.units[next].place.page == unit.place.page;
                    let both = format!("{text} {second}");
                    // The second line may begin in lower case ("the Corinthians").
                    // (Never a number alone: "Jó" over its chapter's "1" is not "Jó 1".)
                    let second_ok = second.chars().count() <= 60
                        && second.chars().any(char::is_alphabetic)
                        && !second.ends_with(['.', ',', ';', ':', '!', '?', '"', '”', '’']);
                    // (A title with a second name may be longer than a heading.)
                    let named = if dangling {
                        book_named(&both, self.language).map(Heading::Book)
                    } else if book_line_shape(&both) {
                        read_heading(&both, self.language)
                    } else {
                        None
                    };
                    if same_page && second_ok {
                        if let Some(Heading::Book(book)) = named {
                            if read_heading(second, self.language) != Some(Heading::Book(book))
                                && self.opens_chapter(next, skip)
                            {
                                let entry = match self.greek_part(book, next, skip) {
                                    Some(code) => self.outside_book(&both, code, index),
                                    None => self.book_entry(&both, book, index),
                                };
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
            // A title over three lines set large ("ΠΕΤΡΟΥ" / "ΕΠΙΣΤΟΛΗ
            // ΚΑΘΟΛΙΚΗ" / "ΔΕΥΤΕΡΑ", v0.24.3): the three together name the
            // book when neither one nor two of them do, and chapter or verse
            // 1 follows.
            if unit.large
                && !unit.heading
                && read_heading(text, self.language).is_none_or(|h| matches!(h, Heading::Book(_)))
            {
                let next: Vec<usize> = (index + 1..self.units.len())
                    .filter(|i| !skip.contains(i) && !clean(&self.units[*i].text).trim().is_empty())
                    .take(2)
                    .collect();
                if let [second, third] = next[..] {
                    let both_large = [second, third].iter().all(|&i| {
                        self.units[i].large && self.units[i].place.page == unit.place.page
                    });
                    let line = |i: usize| {
                        unmark_raised(&clean(&self.units[i].text))
                            .trim()
                            .to_string()
                    };
                    let two = format!("{text} {}", line(second));
                    let three = format!("{two} {}", line(third));
                    let named_two =
                        matches!(read_heading(&two, self.language), Some(Heading::Book(_)));
                    if both_large && !named_two && book_line_shape(&three) {
                        if let Some(Heading::Book(book)) = read_heading(&three, self.language) {
                            if self.opens_chapter(third, skip) {
                                let entry = self.book_entry(&three, book, index);
                                self.push(
                                    Event::Book {
                                        entry,
                                        chapter: None,
                                    },
                                    index,
                                );
                                joined.insert(second);
                                joined.insert(third);
                                continue;
                            }
                        }
                    }
                }
            }
            // Headings: the document's own, or a line that is nothing but a book or chapter.
            // A number alone in a block set apart by its look (EPUB, Word:
            // bold, larger, centred; PDF: large) is its chapter's label, never a
            // verse number, even where the chapter has fewer verses than the
            // standard numbering's (LSG Job 38 has 38, not 41).
            let number_only =
                !text.is_empty() && text.len() <= 3 && text.chars().all(|c| c.is_ascii_digit());
            if number_only
                && !unit.note
                && ((unit.place.page.is_none() && unit.styled) || unit.large)
                && self.book_open()
            {
                if let Ok(n) = text.parse::<u16>() {
                    if n > 0 {
                        self.push(Event::Chapter(n), index);
                        continue;
                    }
                }
            }
            // A book outside the 66 (Tobit, 1 Maccabees, Esther in Greek):
            // named, and its text left out, never read into the book before.
            if let Some(code) = self.outside_title(unit, text, index, skip) {
                let entry = self.outside_book(text, code, index);
                self.push(
                    Event::Book {
                        entry,
                        chapter: None,
                    },
                    index,
                );
                continue;
            }
            // A heading in the document's own form for its books' titles ("Le
            // livre du prophète Habaquq", after "Le livre du prophète Nahum"
            // and others) whose name no dictionary knows: a book the
            // recogniser cannot name. Its text is never read into the book
            // before; it is offered for correction (v0.24.3).
            if unit.heading && read_heading(text, self.language).is_none() && self.title_form(text)
            {
                let entry = self.found_book(text, None, index);
                self.push(
                    Event::Book {
                        entry,
                        chapter: None,
                    },
                    index,
                );
                continue;
            }
            if let Some(heading) =
                read_heading(text, self.language).or_else(|| self.next_in_sequence(text))
            {
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
            // The end of a line of references wrapped onto a line of its own
            // ("… Jé 6:6-15," / "27-30."): numbers only, with a range or a
            // chapter and verse, after a line of references that goes on.
            let wrapped_references = text.chars().any(|c| c.is_ascii_digit())
                && text.contains(['-', '–', ':'])
                && text
                    .chars()
                    .all(|c| c.is_ascii_digit() || " -–,;:.".contains(c))
                && matches!(self.items.last(), Some(Item { event: Event::Other { notes: false, text: before }, .. })
                    if before.trim_end().ends_with([',', ';']) && reference_line(before, self.language));
            // A line of passage references ("V. 18-24: cf. (1 Co 11:7-10.)",
            // "(Ps 103:7. Né 9:9-20.)"): cross-references, not Scripture.
            if wrapped_references || reference_line(text, self.language) {
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
        self.introduction_before_chapter_one();
        self.headings_before_verses();
        (self.items, self.books)
    }

    /// Numbered text between a book's title and its chapter 1 is the book's
    /// introduction, never verses of a chapter 1 that has not begun yet:
    /// numbered text before the chapter 1 heading ("1 Les hymnes … : Ps 8;
    /// 19."), or a numbered list that starts again at 1 before chapter 1
    /// has all its verses ("1. la guérison d'un démoniaque 4:33-36", …,
    /// then "1 Plusieurs ayant entrepris …"): a list's numbers end with a
    /// full stop or their line, or its items cite chapters and verses.
    fn introduction_before_chapter_one(&mut self) {
        use std::sync::OnceLock;
        static CITED: OnceLock<regex::Regex> = OnceLock::new();
        let cited = CITED.get_or_init(|| regex::Regex::new(r"\d{1,3}[:.]\d{1,3}").expect("valid"));
        // From the book's title: (title, markers, listed, citing, strong).
        // Only strong numbers (at a line's start, after a sentence) are
        // compared with chapter 1's verses: an introduction's prose is full
        // of other numbers ("3 Jean", "(9-10)") that are no list (v0.24.3).
        let mut run: Option<(usize, usize, usize, usize, usize)> = None;
        let mut first_chapter_verses = 0u16;
        let mut i = 0;
        while i < self.items.len() {
            let mut cut = None;
            match &self.items[i].event {
                Event::Book {
                    chapter: None,
                    entry,
                } => {
                    run = Some((i, 0, 0, 0, 0));
                    first_chapter_verses = self.books[*entry]
                        .detected
                        .and_then(canon::by_code)
                        .and_then(|b| canon::verse_count(b, 1))
                        .unwrap_or(0);
                }
                Event::Book { .. } => run = None,
                Event::Marker(m) => {
                    if let Some((from, markers, listed, citing, strong)) = run.as_mut() {
                        if m.value == 1 && m.unit_start && *markers > 0 {
                            let list = *listed * 2 >= *markers || *citing * 2 >= *markers;
                            if list && (*strong as u16) < first_chapter_verses {
                                cut = Some(*from);
                            }
                        }
                        *markers += 1;
                        if m.strength == Strength::Strong {
                            *strong += 1;
                        }
                        // "1.", or a number ending its line ("Adresse et
                        // salutations 1-2"): a verse's number comes before
                        // its words.
                        let ends_line = self
                            .items
                            .get(i + 1)
                            .is_none_or(|n| n.unit != self.items[i].unit);
                        if m.raw.trim_end().ends_with('.') || (ends_line && !m.unit_start) {
                            *listed += 1;
                        }
                        let text = match self.items.get(i + 1).map(|n| &n.event) {
                            Some(Event::Text(t)) => cited.is_match(t),
                            _ => false,
                        };
                        if text {
                            *citing += 1;
                        }
                    }
                }
                Event::Chapter(n) => {
                    if let (Some((from, markers, ..)), 1) = (run, *n) {
                        if markers > 0 {
                            cut = Some(from);
                        }
                    }
                    run = None;
                }
                _ => {}
            }
            if let Some(from) = cut {
                for item in &mut self.items[from + 1..i] {
                    let text = match &item.event {
                        Event::Text(t) => t.trim().to_string(),
                        Event::Marker(m) => m.raw.clone(),
                        _ => continue,
                    };
                    item.event = Event::Other { notes: false, text };
                }
                // The numbering that starts here is counted afresh.
                if let Some(r) = run.as_mut() {
                    *r = (
                        i - 1,
                        1,
                        0,
                        0,
                        usize::from(
                            matches!(&self.items[i].event, Event::Marker(m) if m.strength == Strength::Strong),
                        ),
                    );
                }
            }
            i += 1;
        }
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
        // `more`: another heading line, or the passage's cross-references,
        // follows; a heading of several parts may then end with a full stop
        // or a comma ("Défense à Aaron … tabernacle." over "Portions des
        // offrandes …"; "Le territoire conquis distribué aux Rubénites," over
        // its second line).
        let language = self.language;
        // `block`: a paragraph of its own that the document styles (EPUB,
        // Word): a heading there may be long, of several parts ("Défense à
        // Aaron … tabernacle. Portions des offrandes …").
        // `continued`: the second line of a heading may go on in lower case
        // ("Prédiction de la naissance de Jean-Baptiste, et de" / "celle de
        // Jésus-Christ").
        // `styled`: a line in another typeface, bold or italic, may be as
        // wide as the column ("Défense à Aaron … tabernacle. Portions des").
        // It still may not end as a sentence does: poetry and spoken words
        // are set in another typeface too ("Bénissez l’Éternel!").
        let shape = |line: &str,
                     set_apart: bool,
                     more: bool,
                     block: bool,
                     styled: bool,
                     continued: bool| {
            // A note's sign after the last word does not end the line
            // ("Her steps lead straight to Sheol.†" ends as a sentence).
            let line = line.trim().trim_end_matches(is_sign).trim_end();
            // A parallel passage in brackets after a heading ("… 재판하다(신
            // 1:9-18)", "(Mt 5:1-12)") is part of the heading.
            let line = match line.strip_suffix(')').and_then(|l| l.rsplit_once('(')) {
                Some((head, inside))
                    if !head.trim().is_empty() && reference_line(inside, language) =>
                {
                    head.trim()
                }
                _ => line,
            };
            let ends = line.ends_with([
                '.', '!', '?', ';', ':', ',', '”', '’', '"', '»', '\'', ')', '-', '‐', '–',
            ]);
            line.chars().count()
                <= if block {
                    400
                } else if styled {
                    110
                } else {
                    70
                }
                && (continued
                    || line.chars().find(|c| c.is_alphabetic()).is_some_and(|c| !c.is_lowercase()))
                // A heading set apart and in capitals may end with a full
                // stop ("ב BETH."); any other must not end as a sentence.
                && (!ends
                    || block
                    || (set_apart && capitals(line) && line.ends_with('.'))
                    || (set_apart && more && line.ends_with(['.', ','])))
        };
        // Korean sentences end with a final ending, not a full stop
        // ("… 안식하셨음이더라", "… 하시니라", "… 하소서").
        let korean_end = |t: &str| {
            t.chars().last().is_some_and(|c| {
                matches!(
                    c,
                    '다' | '라' | '요' | '까' | '니' | '서' | '오' | '네' | '지' | '세' | '자'
                )
            })
        };
        let ended = |item: &Item| match &item.event {
            Event::Text(t) => {
                // (Note signs after the last word do not count: "counsellors.§".)
                let t = t.trim_end().trim_end_matches(is_sign).trim_end();
                t.ends_with(['.', '!', '?', '”', '’', '"', '»', ')', '。']) || korean_end(t)
            }
            _ => true,
        };
        let whole_alone = |b: &Self, i: usize| -> Option<(String, bool)> {
            let item = &b.items[i];
            let Event::Text(line) = &item.event else {
                return None;
            };
            let whole = unmark_raised(&clean(&b.units[item.unit].text)).trim()
                == unmark_raised(line.trim());
            let alone = (i == 0 || b.items[i - 1].unit != item.unit)
                && b.items.get(i + 1).is_none_or(|n| n.unit != item.unit);
            let unit = &b.units[item.unit];
            let styled = unit.heading || unit.large || unit.styled;
            (whole && alone).then(|| (line.clone(), styled))
        };
        let candidate_line =
            |b: &Self, i: usize, references_follow: bool, more: bool, continued: bool| -> bool {
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
                // (A line the page sets apart at its top or bottom edge that
                // is no running head and is short is set apart by its space,
                // v0.24.3.)
                let styled = unit.heading || unit.large || unit.styled || (unit.edge && unit.short);
                // (A line with its passage's cross-references right under it is
                // set apart by them.)
                let typographic = styled || capitals(line);
                // A parallel passage in brackets at the line's end ("셈의
                // 후예(대상 1:24-27)", "… (Mt 5:1-12)") sets a heading apart as
                // its cross-references do: Scripture never writes one.
                let parallel = line
                    .trim_end()
                    .strip_suffix(')')
                    .and_then(|l| l.rsplit_once('('))
                    .is_some_and(|(head, inside)| {
                        !head.trim().is_empty() && reference_line(inside, language)
                    });
                // A PDF line that begins a block of its own, after space
                // (v0.24.3): set apart by the page's layout even in the
                // body's type ("Das Paradies und der Sündenfall."). Never one
                // ending as a clause does (a poem's line after a stanza's gap).
                // (Centred: the first line of a column, or of a poem's
                // stanza, begins a block too, at the column's edge.)
                let block_start = b.block_starts.contains(&item.unit)
                    && unit.centred
                    && !line.trim_end().ends_with([',', ';', ':'])
                    && line.chars().count() <= 80;
                let set_apart = typographic || references_follow || parallel || block_start;
                // A single word only when its style sets it apart (a styled block
                // or line is never the wrapped end of a verse). A full stop or a
                // comma at the end only where the look sets the line apart: a
                // poem's last line ("Moi, l'Éternel, je crée ces choses.") can
                // stand right over the cross-references too.
                let words = line.split_whitespace().count();
                // A paragraph the document styles (EPUB, Word) may be long and
                // end as it likes only where the document calls it a heading
                // (a heading style) or the passage's cross-references follow
                // it: italic poetry and quotations are styled paragraphs too
                // ("My God, in whom I trust!”").
                let block = unit.place.page.is_none()
                    && (unit.heading || unit.styled)
                    && (unit.heading || references_follow);
                whole
                    && alone
                    && !is_selah(line)
                    && set_apart
                    && shape(
                        line,
                        typographic || block_start,
                        more || block || block_start,
                        block,
                        styled,
                        continued,
                    )
                    // (The last line of a heading, too short for its style
                    // to show, may be one word: "Un démoniaque" / "guéri",
                    // with the passage's cross-references under it.)
                    && (words >= 2 || styled || (continued && references_follow))
            };
        let candidate = |b: &Self, i: usize, references_follow: bool, more: bool| {
            candidate_line(b, i, references_follow, more, false)
        };
        // From the end, so that a heading over another heading (a section
        // title over a subtitle) is seen once the later one is set aside.
        let mut k = self.items.len();
        while k > 1 {
            k -= 1;
            // What comes next, past lines already left out (cross-references,
            // other headings).
            let next = (k + 1..self.items.len())
                .find(|&i| !matches!(self.items[i].event, Event::Other { .. }));
            // A styled passage after a book's last verse, with only the next
            // book (or nothing) after it: a colophon ("Written to the Romans
            // from Corinthus …"), not words of the verse.
            // It may run over a few lines (up to six in a narrow column, 300
            // characters at most), not all of them long enough for their
            // style to show ("Written to the Romans … servant of the" /
            // "church at Cenchrea.").
            let book_ends = next.is_none_or(|i| matches!(self.items[i].event, Event::Book { .. }));
            if book_ends {
                let (mut j, mut total, mut styled) = (k, 0usize, false);
                let (mut start, mut run) = (None, String::new());
                while j >= 1 && k - j < 6 {
                    let Some((line, st)) = whole_alone(self, j) else {
                        break;
                    };
                    total += line.chars().count();
                    // The subscription of an epistle reads as one even
                    // without a style ("Πρὸς Ῥωμαίους ἐγράφη ἀπὸ Κορίνθου …",
                    // "Written to the Romans from Corinthus …"), read over
                    // the lines it is wrapped on.
                    run = format!("{line} {run}");
                    styled |= st || subscription(&run);
                    if total > 300 {
                        break;
                    }
                    if styled && ended(&self.items[j - 1]) {
                        start = Some(j);
                        break;
                    }
                    j -= 1;
                }
                if let Some(start) = start {
                    for i in start..=k {
                        if let Event::Text(line) = &self.items[i].event {
                            let text = unmark_raised(line.trim());
                            self.items[i].event = Event::Other { notes: false, text };
                        }
                    }
                    k = start;
                    continue;
                }
            }
            // The passage's cross-references right under a line ("Léa et
            // Rachel" / "V. 15-30: cf. Os 12:13."): a section heading, even
            // where the verse before it goes on after it.
            let references_follow = self.items.get(k + 1).is_some_and(|n| {
                matches!(&n.event, Event::Other { notes: false, text }
                    if reference_line(text, self.language))
            });
            // A heading already set aside right after this line.
            let heading_follows = self
                .items
                .get(k + 1)
                .is_some_and(|n| matches!(&n.event, Event::Other { notes: false, .. }));
            // A heading over several lines (up to four, each in another
            // typeface but perhaps the last), its later lines going on in lower
            // case ("Jésus à Capernaüm. Enseignement dans la" / "synagogue.
            // Guérison d’un démoniaque, de la" / …): from line `k` back to the
            // one that begins it, after a verse that ended its sentence.
            let styled_at = |b: &Self, i: usize| {
                let unit = &b.units[b.items[i].unit];
                unit.heading || unit.large || unit.styled || (unit.edge && unit.short)
            };
            let chain = |b: &Self, k: usize| -> Option<usize> {
                let mut j = k;
                while j >= 2 && k - j < 3 {
                    j -= 1;
                    if !candidate_line(b, j, false, true, true) {
                        return None;
                    }
                    if candidate(b, j, false, true) && ended(&b.items[j - 1]) {
                        return Some(j);
                    }
                    if !styled_at(b, j) {
                        return None;
                    }
                }
                None
            };
            let continuation = if candidate(self, k, references_follow, heading_follows) {
                None
            } else if candidate_line(self, k, references_follow, heading_follows, true) {
                match chain(self, k) {
                    Some(first) => Some(first),
                    None => continue,
                }
            } else {
                continue;
            };
            // A verse or chapter number, or a chapter heading, comes next.
            let starts_verse = next.is_some_and(|i| {
                matches!(&self.items[i].event,
                    Event::Marker(m) if m.unit_start && m.chapter.is_none() && m.book.is_none())
                    || matches!(
                        &self.items[i].event,
                        Event::Chapter(_)
                            | Event::Book {
                                chapter: Some(_),
                                ..
                            }
                    )
            });
            // A paragraph in a heading style inside a verse (EPUB, Word): a heading where
            // the verse before it ended its sentence and the text after it
            // begins one ("… changé l’eau en vin." / "Guérison du fils d’un
            // officier du roi" / "Il y avait à Capernaüm …").
            let inside_verse = {
                let unit = &self.units[self.items[k].unit];
                unit.place.page.is_none()
                    && unit.heading
                    && k >= 1
                    && ended(&self.items[k - 1])
                    && next.is_some_and(|i| match &self.items[i].event {
                        Event::Text(t) => t
                            .trim_start()
                            .chars()
                            .find(|c| c.is_alphabetic())
                            .is_some_and(char::is_uppercase),
                        _ => false,
                    })
            };
            if !starts_verse && !references_follow && !inside_verse {
                continue;
            }
            // One line, or two (a heading wrapped over two lines), or more.
            let two = k >= 2 && candidate(self, k - 1, false, true) && ended(&self.items[k - 2]);
            let mut first = match continuation {
                Some(first) => first,
                None if two => k - 1,
                None => k,
            };
            if !ended(&self.items[first - 1]) && styled_at(self, first) {
                if let Some(start) = chain(self, first) {
                    first = start;
                }
            }
            // A paragraph of its own set apart by its look (EPUB, Word), with
            // a verse number starting the next: never the end of the verse
            // before, even where verses end without a full stop (Korean
            // "… 안식하셨음이더라").
            let paragraph = {
                let unit = &self.units[self.items[k].unit];
                unit.place.page.is_none() && (unit.heading || unit.styled) && starts_verse
            };
            // (A styled paragraph that ends as a sentence does, "Qui est le
            // plus grand?", only after a verse that ended its own.)
            let sentence = match &self.items[k].event {
                Event::Text(t) => t
                    .trim_end()
                    .trim_end_matches(is_sign)
                    .trim_end()
                    // (past closing quotes: "… trust!”")
                    .trim_end_matches(['”', '’', '"', '»', '\'', ')'])
                    .ends_with(['.', '!', '?', ',', ';', ':']),
                _ => false,
            };
            if (paragraph && !sentence) || ended(&self.items[first - 1]) {
                for i in first..=k {
                    if let Event::Text(line) = &self.items[i].event {
                        let text = unmark_raised(line.trim());
                        self.items[i].event = Event::Other { notes: false, text };
                    }
                }
                k = first;
            }
        }
    }
}

/// PDF lines that begin a new block of text (a paragraph set off by space
/// above it, as the page's layout reads it) below other running text of the
/// same column of the same page (v0.24.3; a column's first line is no block
/// start). A verse's last line never begins a block of its own;
/// a heading set off by space does, even in the body's own type.
fn block_starts(units: &[Unit], skip: &HashSet<usize>) -> HashSet<usize> {
    units
        .iter()
        .enumerate()
        .filter(|(i, u)| u.gap_before && !skip.contains(i) && !u.heading && !u.note)
        .map(|(i, _)| i)
        .collect()
}

/// Words a document writes as labels inside its lines (v0.24.3): after a
/// sentence's end, a word starting with a capital, without punctuation, then
/// another starting with a capital ("… rooms. Friends We will be glad …",
/// speakers' names in the Song of Songs) — at least twice, and never
/// followed by a word in lower case between its first and last use as a
/// label, as a sentence's first word is ("Then he said", "Jesus answered";
/// "Beloved" begins sentences elsewhere in the Bible, not in the Song). Not
/// "I".
fn inline_labels(units: &[Unit]) -> HashSet<String> {
    let mut label: HashMap<String, Vec<usize>> = HashMap::new();
    let mut sentence: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, unit) in units.iter().enumerate() {
        let text = clean(&unit.text);
        let words: Vec<&str> = text.split_whitespace().collect();
        for w in words.windows(3) {
            // (After a sentence's end, or a verse's number: "SOL 1:2 Beloved
            // Let him kiss me".)
            let number = w[0]
                .chars()
                .all(|c| c.is_ascii_digit() || c == ':' || superscript_digit(c).is_some());
            if !number && !w[0].ends_with(['.', '!', '?', ';', ':', '”', '’', '"', '»']) {
                continue;
            }
            let word = w[1];
            let shaped = word.chars().count() >= 2
                && word.chars().all(char::is_alphabetic)
                && word.chars().next().is_some_and(char::is_uppercase)
                && word.chars().skip(1).all(char::is_lowercase);
            if !shaped {
                continue;
            }
            match w[2].chars().next() {
                Some(c) if c.is_uppercase() => label.entry(word.to_string()).or_default().push(i),
                Some(c) if c.is_lowercase() => {
                    sentence.entry(word.to_string()).or_default().push(i)
                }
                _ => {}
            }
        }
    }
    label
        .into_iter()
        .filter(|(w, at)| {
            let (first, last) = (at[0], at[at.len() - 1]);
            at.len() >= 2
                && !sentence
                    .get(w)
                    .is_some_and(|s| s.iter().any(|i| (first..=last).contains(i)))
        })
        .map(|(w, _)| w)
        .collect()
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
            while end + 1 < lines.len()
                && number(lines[end + 1]) == number(lines[end]).map(|n| n + 1)
            {
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
            && parts
                .iter()
                .enumerate()
                .all(|(i, p)| *p == (i + 1).to_string())
        {
            out.insert(lines[k]);
        }
        k += 1;
    }
    out
}

/// The wording of an epistle's subscription, which some editions print
/// after its last verse ("Πρὸς Ῥωμαίους ἐγράφη ἀπὸ Κορίνθου …", "Written to
/// the Romans from Corinthus …", "The first epistle to the Corinthians was
/// written from Philippi …").
fn subscription(line: &str) -> bool {
    let k = key(line);
    let words: Vec<&str> = k.split_whitespace().collect();
    // (Πρὸς, its final sigma as the key writes it, as the first word.)
    (words.first().is_some_and(|w| ["προς", "προσ"].contains(w)) && words.contains(&"εγραφη"))
        || k.starts_with("written to ")
        || k.contains(" was written from ")
}

/// "Selah" (a word of the psalms themselves, often set on a line of its own,
/// in italics, at a psalm's end): part of its verse in every edition, never a
/// heading or a label.
fn is_selah(line: &str) -> bool {
    const SELAH: &[&str] = &["selah", "sela", "pausa", "셀라", "סלה", "διαψαλμα"];
    let line = line.trim().trim_end_matches(is_sign).trim_end();
    SELAH.contains(&key(line.trim_end_matches(['.', ')']).trim_start_matches('(')).as_str())
}

/// Words written with a hyphen inside a line of the documents ("Padan-aram",
/// "El-Beth-el"), in lower case: each joined pair.
fn hyphenated_words(units: &[Unit]) -> HashSet<String> {
    use std::sync::OnceLock;
    static WORD: OnceLock<regex::Regex> = OnceLock::new();
    let word = WORD.get_or_init(|| regex::Regex::new(r"\p{L}+(?:-\p{L}+)+").expect("valid"));
    let mut out = HashSet::new();
    for unit in units {
        let text = clean(&unit.text);
        for m in word.find_iter(&text) {
            let parts: Vec<&str> = m.as_str().split('-').collect();
            for pair in parts.windows(2) {
                out.insert(format!("{}-{}", pair[0], pair[1]).to_lowercase());
            }
        }
    }
    out
}

/// Every word the documents write whole (letters only, in lower case): a
/// word broken at a line's end ("Jéru-" / "salem") that is found whole
/// elsewhere is joined without its hyphen.
fn whole_words(units: &[Unit]) -> HashSet<String> {
    let mut out = HashSet::new();
    for unit in units {
        for word in clean(&unit.text).split(|c: char| !c.is_alphabetic()) {
            if word.chars().count() > 2 {
                out.insert(word.to_lowercase());
            }
        }
    }
    out
}

/// Where a passage reference closing a verse begins ("… Deus conosco.
/// Isaías 7:14", "… (Ps 2:7)"): after the verse's last sentence, and nothing
/// after it but closing punctuation. None when the verse does not end so.
/// Passage references inside a verse, right after a quotation or a
/// sentence ("… não sacrifício’. Oseías 6:6 Porque eu não vim …"): the
/// source of the quotation, set in the text by the edition. Scripture
/// itself never writes "book chapter:verse", so only such a reference — a
/// book and a chapter and verse, with any list after it ("Salmos 14:1-3;
/// 53:1-3; Eclesiastes 7:20") — standing after closing punctuation is
/// taken out. Byte ranges, in order.
fn inline_references(text: &str, language: Option<ReferenceLanguage>) -> Vec<(usize, usize)> {
    let languages: Vec<ReferenceLanguage> = match language {
        Some(l) => vec![l],
        None => name_languages(),
    };
    let mut matches: Vec<(usize, usize)> = languages
        .iter()
        .flat_map(|l| ReferenceParser::for_language(*l).matches(text))
        .filter(|m| m.written_verse)
        .map(|m| (m.start, m.end))
        .collect();
    matches.sort_unstable();
    matches.dedup();
    let mut out: Vec<(usize, usize)> = Vec::new();
    let mut i = 0;
    while i < matches.len() {
        let (start, mut end) = matches[i];
        let before = text[..start].trim_end();
        let after_quote = before.ends_with(['.', '!', '?', '»', '”', '’', '"', ',', ';', ')']);
        if before.is_empty() || !after_quote || out.last().is_some_and(|(_, e)| *e > start) {
            i += 1;
            continue;
        }
        // The list going on: more verses ("…12-13; 32:6-9"), more references.
        loop {
            let rest = &text[end..];
            let more = rest
                .char_indices()
                .take_while(|(_, c)| {
                    c.is_ascii_digit() || matches!(c, ',' | ';' | '-' | '–' | ':' | ' ')
                })
                .last()
                .map_or(0, |(at, c)| at + c.len_utf8());
            let mut extended = end + rest[..more].trim_end().len();
            // A reference to another book next in the list.
            if let Some(&(s, e)) = matches.get(i + 1) {
                // (The next reference may begin inside the numbers just read.)
                if s < extended
                    || text[extended..s].trim().is_empty()
                    || text[end..s].trim_matches([';', ',', ' ']).is_empty()
                {
                    extended = extended.max(e);
                    i += 1;
                }
            }
            if extended == end {
                break;
            }
            end = extended;
        }
        out.push((start, end));
        i += 1;
    }
    out
}

fn trailing_reference(text: &str, language: Option<ReferenceLanguage>) -> Option<usize> {
    let languages: Vec<ReferenceLanguage> = match language {
        Some(l) => vec![l],
        None => name_languages(),
    };
    let matches: Vec<(usize, usize, bool)> = languages
        .iter()
        .flat_map(|l| ReferenceParser::for_language(*l).matches(text))
        .map(|m| (m.start, m.end, m.written_verse))
        .collect();
    // Closing punctuation and the separators of a list of references.
    let closing = |c: char| c.is_whitespace() || matches!(c, '.' | ')' | ']' | ';' | ',');
    let mut end = text.trim_end_matches(closing).len();
    let mut cut: Option<usize> = None;
    // From the end: references one after another ("Êxodo 21:24; Levítico
    // 24:20;"), the verse's own last sentence before them.
    // A reference ending here; a verse list after it ("91:11,12") is part
    // of it.
    while let Some(&(start, _, _)) = matches
        .iter()
        .filter(|(s, e, verse)| {
            *verse && *s < end && {
                let tail = &text[*e..end];
                tail.chars()
                    .all(|c| c.is_ascii_digit() || matches!(c, ',' | '-' | '–' | ' '))
            }
        })
        .min_by_key(|(s, _, _)| *s)
    {
        let mut start = start;
        let before = text[..start].trim_end();
        let before = match before.strip_suffix(['(', '[']) {
            Some(b) => {
                start = b.len();
                b.trim_end()
            }
            None => before,
        };
        if before.is_empty() {
            break;
        }
        if before.ends_with(['.', '!', '?', '»', '”', '’', '"', ':']) {
            cut = Some(start);
            break;
        }
        // Another reference of the list before this one.
        if before.ends_with([';', ',']) {
            cut = Some(start);
            end = before.trim_end_matches(closing).len();
            continue;
        }
        break;
    }
    // Only a list that follows the verse's last sentence is taken off.
    let cut = cut?;
    let before = text[..cut]
        .trim_end()
        .trim_end_matches(['(', '['])
        .trim_end();
    before
        .ends_with(['.', '!', '?', '»', '”', '’', '"', ':'])
        .then_some(cut)
}

/// A line made of passage references and nothing else of substance ("V.
/// 18-24: cf. (1 Co 11:7-10. 1 Ti 2:11-13.)", "(Ps 103:7. Né 9:9-20.)", "Ch.
/// 1 à 11: 9. (Ac 17:24-28.)"): at least one chapter-and-verse reference,
/// and outside the book names the references give, no word longer than
/// three letters (abbreviations, "cf", "v", "à").
fn reference_line(text: &str, language: Option<ReferenceLanguage>) -> bool {
    use std::sync::OnceLock;
    static PAIR: OnceLock<regex::Regex> = OnceLock::new();
    // A chapter and verse ("21:9"), a list of chapters ("Ps 70; 142."), or
    // a passage's verses or chapters named as such ("V. 1-28: cf. De 33.",
    // "Ch. 31 à 34.").
    let pair = PAIR.get_or_init(|| {
        regex::Regex::new(
            r"\d{1,3}\s?[:,.]\s?\d{1,3}|\d{1,3}\s?;\s?\d{1,3}|^\s*[Vv]{1,2}\.\s?\d{1,3}(\s?[-–]\s?\d{1,3})?\s?:|^\s*[Cc]h(ap)?\.\s?\d{1,3}",
        )
        .expect("valid")
    });
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
        // In Hangul, Chinese and Japanese a word of two syllables is a word,
        // not an abbreviation (Korean references abbreviate a book to one:
        // "마", "창"); "너는 가증한 것을" is prose.
        let syllabic = word.chars().any(|c| {
            matches!(c as u32, 0xAC00..=0xD7AF | 0x3040..=0x30FF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF)
        });
        if word.chars().count() > 3 || (syllabic && word.chars().count() >= 2) {
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
    /// Why the verse's text cannot be verified, when something is in doubt
    /// (v0.24.3; see [`Recognition::unverified`]).
    doubt: Option<&'static str>,
    /// Its last line is a whole line that could be a heading in the body's
    /// own type (see `Reader::text`).
    heading_like_end: bool,
    /// Whether its last line is a whole line beginning a sentence after one
    /// ended (whatever ends it): set apart if a heading or a line of
    /// references follows it.
    fresh_last_line: bool,
    /// Whether its last line was a short one (a paragraph's end), and the
    /// line before it (None: none in this verse).
    last_short: bool,
    before_last_short: Option<bool>,
}

struct Reader<'a> {
    units: &'a [Unit],
    items: &'a [Item],
    language: Option<ReferenceLanguage>,
    /// Words the document writes with a hyphen inside a line ("padan-aram"):
    /// at a line's end such a hyphen is the word's own, not a break.
    hyphenated: HashSet<String>,
    /// Words the document writes whole (see `whole_words`).
    whole: HashSet<String>,
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
    /// The unit the last piece of apparatus came from, and the characters of
    /// apparatus kept (see `add_apparatus`).
    apparatus_unit: Option<usize>,
    apparatus_chars: usize,
    /// PDF lines that begin a block of their own (see `block_starts`).
    block_starts: HashSet<usize>,
    /// Words the document uses as labels inside its lines (see `inline_labels`).
    inline_labels: HashSet<String>,
    /// A heading was left out since the open verse began: words after it
    /// without a verse number may not be that verse's (v0.24.3).
    heading_in_verse: bool,
}

impl Reader<'_> {
    fn place(&self, unit: usize) -> Place {
        self.units[unit].place
    }

    fn leave_out(&mut self, text: &str, unit: usize, reason: &'static str) {
        let chars = text.chars().count();
        self.out.stats.left_out_chars += chars;
        let verse = self
            .open
            .as_ref()
            .map(|o| (o.ordinal, o.chapter, o.verse))
            .or_else(|| {
                let (book, chapter) = (self.book?, self.chapter?);
                Some((book.ordinal, chapter, self.expect.saturating_sub(1)))
            })
            .filter(|v| v.2 > 0);
        let place = self.place(unit);
        add_apparatus(
            &mut self.out.apparatus,
            &mut self.apparatus_unit,
            &mut self.apparatus_chars,
            apparatus_kind(reason),
            text,
            place,
            unit,
            verse,
        );
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
    fn append(
        open: &mut Open,
        words: &str,
        unit: usize,
        place: Place,
        glue: bool,
        hyphenated: &HashSet<String>,
        whole: &HashSet<String>,
    ) {
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
            // "Padan-" / "aram": the hyphen stays when the document writes
            // the word so elsewhere, inside a line.
            let left: String = open.text[..open.text.len() - 1]
                .chars()
                .rev()
                .take_while(|c| c.is_alphabetic())
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            let right: String = words.chars().take_while(|c| c.is_alphabetic()).collect();
            let compound = format!("{left}-{right}").to_lowercase();
            // Without that evidence the hyphen stays only in a chain of
            // hyphens ("Beerot-bene-" / "jacaam") the joined word is nowhere
            // written whole; elsewhere it is a break ("Notwith-" / "standing",
            // "Sündopfer-" / "bock": a capital proves nothing).
            let before = open.text[..open.text.len() - 1 - left.len()].chars().last();
            let chained = before == Some('-');
            let keep = hyphenated.contains(&compound)
                || (!whole.contains(&format!("{left}{right}").to_lowercase()) && chained);
            if !keep {
                open.text.pop();
            }
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
        for (start, end) in inline_references(&text, self.language).into_iter().rev() {
            let reference = text[start..end].trim().to_string();
            let before = text[..start].trim_end().to_string();
            let after = text[end..].trim_start().to_string();
            text = if after.is_empty() {
                before
            } else {
                format!("{before} {after}")
            };
            self.leave_out(&reference, open.last_unit, "crossReference");
        }
        let key = (open.ordinal, open.chapter, open.verse);
        let book = canon::by_ordinal(open.ordinal).expect("a staged book is canonical");
        // A verse the edition prints only in its apparatus: its number, then
        // nothing but the labels of notes that name this very verse ("14 a",
        // the note "a 23:14 Le « Texte reçu » ajoute le v. 14 : …"). The
        // label is no word of Scripture and the note's text is no verse: the
        // verse has no text in this edition (v0.24.3).
        if !text.is_empty() && self.only_note_labels(&text, &open) {
            self.leave_out(&text, open.last_unit, "noteOnly");
            text.clear();
        }
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
        let doubt = open
            .doubt
            .or_else(|| open.heading_like_end.then_some("possibleHeading"))
            .or_else(|| verse_doubt(&text, &self.inline_labels))
            .or_else(|| trailing_book_name(&text, self.language).then_some("reference"));
        chapter.verses.insert(open.verse, text);
        if let Some(end) = open.end {
            chapter.bridges.insert(open.verse, end);
        }
        if let Some(doubt) = doubt {
            self.out.unverified.insert(key, doubt);
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

    /// A whole line added to an open verse that could be a section heading
    /// set in the body's own type (v0.24.3): it is the whole unit, after a
    /// sentence's end, beginning with a capital and not ending as a sentence
    /// or clause does ("… vor Gott" cannot be told from "Die Sintflut"
    /// without its look). Kept in the verse, as the document shows no
    /// difference, but when it is the verse's last line the verse's text is
    /// not verified.
    fn heading_like_line(&self, words: &str, unit: usize) -> bool {
        let Some(open) = self.open.as_ref() else {
            return false;
        };
        let whole = unmark_raised(&clean(&self.units[unit].text));
        // (Or the first piece of a short line a number splits: "맺는 말(1)",
        // "이스라엘 왕 여로보암 2 세"; then the whole line is judged.)
        let Some(line) = whole_line(&whole, words, open.last_unit != unit) else {
            return false;
        };
        if open.text.trim().is_empty() || is_selah(line) {
            return false;
        }
        // A whole line beginning a block of its own after space, inside a
        // verse that began before it ("… Himmel machte -" / "Das Paradies und
        // der Sündenfall." / "⁵ es gab …"): a heading in the body's type, or
        // the first line of a poem's stanza. Not known; not verified.
        if self.block_starts.contains(&unit) && open.last_unit != unit {
            return true;
        }
        // The first line of running text on a page, beginning a sentence
        // after one ended ("1. Mose 5:1 6 1. Mose 5:32" / "Die Patriarchen von
        // Adam bis Noah."): a heading opening a page has no space above it
        // that the layout can see.
        if open.last_unit != unit
            && self.page_top(unit)
            && ends_as_sentence(&open.text)
            && line
                .chars()
                .find(|c| c.is_alphabetic())
                .is_some_and(char::is_uppercase)
        {
            return true;
        }
        // A short line, flush with its column's left edge, after a short
        // line that ended a sentence a line of full width ran into ("…
        // solange sie" / "lebten." / "Baesa, König von Israel." / "³³ Im
        // dritten …"): a paragraph of its own that begins without a verse
        // number, after prose, not indented as poetry is — a heading in the
        // body's type. (Poetry set flush left is short line after short line.)
        let u = &self.units[unit];
        if open.last_unit != unit
            && u.short
            && !u.indented
            && open.last_short
            && open.before_last_short == Some(false)
            && ends_as_sentence(&open.text)
            && line
                .chars()
                .find(|c| c.is_alphabetic())
                .is_some_and(char::is_uppercase)
        {
            return true;
        }
        let bare = line.trim_end_matches(is_sign).trim_end();
        let before = open.text.trim_end().trim_end_matches(is_sign).trim_end();
        let korean_end = before.chars().last().is_some_and(|c| {
            matches!(
                c,
                '다' | '라' | '요' | '까' | '니' | '서' | '오' | '네' | '지' | '세' | '자'
            )
        });
        bare.chars().count() <= 100
            && bare
                .chars()
                .find(|c| c.is_alphabetic())
                .is_some_and(char::is_uppercase)
            && !bare.ends_with([
                '.', '!', '?', ';', ':', ',', '”', '’', '"', '»', ')', '-', '–', '—', '。',
            ])
            && (before.ends_with(['.', '!', '?', '”', '’', '"', '»', ')', '。', '…']) || korean_end)
    }

    /// Whether a unit is the first line of running text on its PDF page
    /// (after the running head, if any).
    fn page_top(&self, unit: usize) -> bool {
        let Some(page) = self.units[unit].place.page else {
            return false;
        };
        let mut j = unit;
        while j > 0 {
            j -= 1;
            let u = &self.units[j];
            if u.place.page != Some(page) {
                return true;
            }
            if !u.edge {
                return false;
            }
        }
        true
    }

    /// Whether a line is whole and begins a sentence after the verse's text
    /// ended one (see `Open::fresh_last_line`).
    fn fresh_line(&self, words: &str, unit: usize) -> bool {
        let Some(open) = self.open.as_ref() else {
            return false;
        };
        let whole = unmark_raised(&clean(&self.units[unit].text));
        let line = words.trim();
        whole.trim() == line
            && open.last_unit != unit
            && line
                .chars()
                .find(|c| c.is_alphabetic())
                .is_some_and(char::is_uppercase)
            && ends_as_sentence(&open.text)
    }

    /// A whole PDF line set apart by its look (another typeface, or larger)
    /// added to a verse after a sentence's end (v0.24.3): a section
    /// heading that ends as a sentence does ("Qui est le plus grand?",
    /// "Comment il se venge et meurt.") cannot be told from a line of poetry
    /// set in italics; it stays where the document puts it, but the verse's
    /// text is not verified.
    fn set_apart_line(&self, words: &str, unit: usize) -> bool {
        let Some(open) = self.open.as_ref() else {
            return false;
        };
        let u = &self.units[unit];
        // (Centred alone is not enough: an indented line of poetry can stand
        // in the middle of its column.)
        // (A line at a page's edge only when short: a heading opening a
        // page, not the last line of a column standing apart.)
        if !(u.styled || u.large || u.heading || (u.edge && u.short)) {
            return false;
        }
        // A paragraph of its own that an EPUB or Word document styles (bold,
        // italic, centred), inside a verse: a heading made without a heading
        // style, wherever the sentence stands ("… Himmel machte -" / "Das
        // Paradies und der Sündenfall."), or a line of poetry. Not known.
        if u.place.page.is_none() {
            // (Its first piece: a number in it may split it, "맺는 말(1)";
            // a whole paragraph, or a short one so split.)
            let whole = clean(&u.text);
            return (whole.trim() == words.trim()
                || (open.last_unit != unit && whole.chars().count() <= 120))
                && !open.text.trim().is_empty()
                && !is_selah(words);
        }
        let whole = unmark_raised(&clean(&u.text));
        let Some(line) = whole_line(&whole, words, open.last_unit != unit) else {
            return false;
        };
        if open.text.trim().is_empty() || is_selah(line) {
            return false;
        }
        let before = open.text.trim_end().trim_end_matches(is_sign).trim_end();
        // (A short line in another typeface beginning with a capital, even
        // where the sentence before it did not end: "³⁸ Dort war er drei
        // Jahre" / "Absaloms Begnadigung".)
        let capital = line
            .chars()
            .find(|c| c.is_alphabetic())
            .is_some_and(char::is_uppercase);
        line.chars().count() <= 200
            && (before.ends_with(['.', '!', '?', '”', '’', '"', '»', ')', '。', '…'])
                || korean_end(before)
                || (u.short && u.styled && capital))
    }

    /// Whether a verse's whole text is labels of notes that name this verse
    /// (letters or signs, each a word of its own): on its page or the next
    /// for a PDF, anywhere in a document without pages.
    fn only_note_labels(&self, text: &str, open: &Open) -> bool {
        let place = open.start;
        let (letters, signs): (Vec<String>, Vec<char>) = match place.page {
            Some(page) => (
                self.referenced_letters(place.document, page, open.chapter, open.verse),
                self.referenced_signs(place.document, page, open.chapter, open.verse),
            ),
            None => (
                self.note_letters
                    .get(&(place.document, WHOLE_DOCUMENT))
                    .into_iter()
                    .flatten()
                    .filter(|(_, c, v)| (*c, *v) == (open.chapter, open.verse))
                    .map(|(l, _, _)| l.clone())
                    .collect(),
                self.referenced_signs_anywhere(place.document, open.chapter, open.verse),
            ),
        };
        if letters.is_empty() && signs.is_empty() {
            return false;
        }
        text.split_whitespace().all(|word| {
            let word = word.trim_matches(|c: char| matches!(c, ',' | ';' | '.'));
            letters.iter().any(|l| l == word)
                || (!word.is_empty() && word.chars().all(|c| signs.contains(&c)))
        })
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
        if is_selah(whole) {
            return;
        }
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
        for note in self
            .note_signs
            .get(&(document, WHOLE_DOCUMENT))
            .into_iter()
            .flatten()
        {
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
        self.doubt_skipped_edges(unit);
        self.close();
        self.heading_in_verse = false;
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
            doubt: None,
            heading_like_end: false,
            fresh_last_line: false,
            last_short: self.units[unit].short,
            before_last_short: None,
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
                if m.value == self.expect + 1 && self.expect > 1 {
                    self.doubt_boundary();
                }
                self.open_verse(m.value, m.end, unit);
                // A line naming book, chapter and verse 1 of a psalm ("PSA
                // 3:1 A Psalm by David … LORD, how …"): an export that writes
                // the psalm's title into its verse 1 cannot be told from one
                // that does not (v0.24.3). Found, not verified.
                // (Only the psalms that have a title in the Hebrew text.)
                const UNTITLED: &[u16] = &[
                    1, 2, 10, 33, 43, 71, 91, 93, 94, 95, 96, 97, 99, 104, 105, 106, 107, 111, 112,
                    113, 114, 115, 116, 117, 118, 119, 135, 136, 137, 146, 147, 148, 149, 150,
                ];
                if m.book.is_some_and(|b| b.code == "PSA")
                    && m.value == 1
                    && m.chapter.is_some_and(|c| !UNTITLED.contains(&c))
                {
                    if let Some(open) = self.open.as_mut() {
                        open.doubt = Some("psalmTitle");
                    }
                }
            } else {
                // Counting back, or jumping ahead, within a chapter: out of order.
                self.doubt_boundary();
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
        // A number in the middle of a sentence beyond the chapter's standard
        // count, followed by a word in lower case, is a quantity ("… pleura
        // Aaron 30 jours.", Numbers 20 has 29 verses), never a verse
        // (v0.24.3). In scripts without capitals the text cannot tell, and
        // other numberings go beyond the count (Jonah 2:11 in Hebrew Bibles).
        let beyond = canon::verse_count(book, chapter).is_some_and(|count| m.value > count);
        let lower_after = matches!(self.items.get(at + 1).map(|i| &i.event),
            Some(Event::Text(t)) if t.trim_start().chars().next().is_some_and(char::is_lowercase));
        // (Or after a word, with no more than two words after it before the
        // next number or heading: "… das ganze Haus Israel Aaron 30 Tage."
        // A verse runs on: "… car 13 En Dieu nous combattrons …" is verse 13
        // in a numbering with the psalm's title as verse 1.)
        let word_before = at > 0
            && matches!(self.items.get(at - 1).map(|i| &i.event),
                Some(Event::Text(t)) if t.trim_end().chars().last().is_some_and(|c| c.is_uppercase() || c.is_lowercase()))
            && matches!(self.items.get(at + 1).map(|i| &i.event),
                Some(Event::Text(t)) if t.split_whitespace().count() <= 2)
            && !matches!(
                self.items.get(at + 2).map(|i| &i.event),
                Some(Event::Text(_))
            );
        let quantity = beyond && m.strength == Strength::Medium && (lower_after || word_before);
        // A chapter's number set as a drop numeral before verse 1 ("17When
        // Abram …", "17Lorsque Abram …"), at a paragraph's start once this
        // chapter has all its verses, with verse 2 numbered next: the next
        // chapter begins, even where its number is also the next verse's
        // (Genesis 16 has 16 verses), v0.24.3.
        if strong
            && m.unit_start
            && !m.raised
            && beyond
            && m.end.is_none()
            && m.value == chapter + 1
            && chapter < book.chapter_count
            && self.chapter_complete()
            && self.next_value(at) == Some(2)
        {
            self.start_chapter(chapter + 1, unit);
            self.out.stats.numbered_chapters += 1;
            self.open_verse(1, None, unit);
            return true;
        }
        if m.value == expect && !quantity {
            match self.best_place(at, m.value, m.strength) {
                Some(true) => {}
                Some(false) => return false,
                None => self.ambiguous_issue(m.value),
            }
            if expect == 1 {
                self.settle_before(true);
            }
            // A raised number against a word: where the verse before ends and
            // this one begins rests on a number that could be a note's.
            // So does a number that is a whole PDF line of its own, not set
            // large ("… 규례를" / "18-19" / "삼가 지키며 …"): a raised number
            // the page could not place within its line.
            let alone = {
                let u = &self.units[unit];
                u.place.page.is_some()
                    && !u.large
                    && unmark_raised(&clean(&u.text))
                        .chars()
                        .all(|c| c.is_ascii_digit() || c.is_whitespace() || matches!(c, '-' | '–'))
            };
            let doubt = m.against_word || alone;
            if doubt {
                self.doubt_boundary();
            }
            self.open_verse(m.value, m.end, unit);
            if doubt {
                self.doubt_boundary();
            }
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
            // The missing verse's words may stand at the end of the verse
            // before it: that verse's text is not verified (v0.24.3).
            self.doubt_boundary();
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
        // A raised verse number counting back within the chapter ("⁴" read
        // after verse 6 began): the verses were counted wrongly before it.
        // The chapter is refused, never guessed (v0.24.3) — also for verse 1
        // again: an introduction's "1 Corinthiens …" read as verse 1 before
        // the chapter's own "¹" would give verse 1 the introduction's words.
        if m.raised && !m.against_word && m.value < expect && m.value >= 1 {
            self.order_issue(book, m.value);
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
            // Where verse 1 ends was inferred from verse 2's number: what
            // stands between the heading and it may not all be verse 1.
            doubt: Some("unnumbered"),
            heading_like_end: false,
            fresh_last_line: false,
            last_short: false,
            before_last_short: None,
        });
        self.close();
    }

    /// The open verse's end is not certain (v0.24.3): its text is not verified.
    /// Lines at a page's edge left out as furniture between the open verse's
    /// last line and `unit`, while its sentence has not ended ("… 인함이니라"
    /// / "하시니라" at the foot): the verse's last words may be among them.
    fn doubt_skipped_edges(&mut self, unit: usize) {
        let Some(open) = self.open.as_ref() else {
            return;
        };
        if open.text.trim().is_empty() || ends_as_sentence(&open.text) {
            return;
        }
        // (Or a number alone left out between them: "… beliefen sich auf" /
        // "53.400.".)
        let between = (open.last_unit + 1..unit.min(self.units.len())).any(|j| {
            let u = &self.units[j];
            let text = clean(&u.text);
            let number = !text.trim().is_empty()
                && text
                    .trim()
                    .chars()
                    .all(|c| c.is_ascii_digit() || matches!(c, '.' | ',' | '\''))
                // (Not a page's number: written with a separator.)
                && text.trim().trim_end_matches('.').contains(['.', ',', '\'']);
            // (A line at the edge only when it ends as a sentence does, as
            // no running head does: "하시니라".)
            let ending = ends_as_sentence(&text) || korean_end(text.trim_end());
            (u.edge && ending && !text.chars().any(|c| c.is_ascii_digit())) || (number && !u.edge)
        });
        if between {
            self.doubt_boundary();
        }
    }

    fn doubt_boundary(&mut self) {
        if let Some(open) = self.open.as_mut() {
            open.doubt = open.doubt.or(Some("boundary"));
        }
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
                    // (A document without pages: its notes anywhere, by the
                    // verse they name, v0.24.3.)
                    let known = match place.page {
                        Some(page) => {
                            self.referenced_letters(place.document, page, open.chapter, open.verse)
                        }
                        None => {
                            let mut out: Vec<String> = Vec::new();
                            for (label, c, v) in self
                                .note_letters
                                .get(&(place.document, WHOLE_DOCUMENT))
                                .into_iter()
                                .flatten()
                            {
                                if (*c, *v) == (open.chapter, open.verse) && !out.contains(label) {
                                    out.push(label.clone());
                                }
                            }
                            out
                        }
                    };
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
            self.doubt_skipped_edges(unit);
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
            let heading_like = self.heading_like_line(&words, unit);
            let fresh = self.fresh_line(&words, unit);
            let set_apart = self.set_apart_line(&words, unit);
            let open = self.open.as_mut().expect("a verse is open");
            // As its last line, or anywhere in the verse when the line is a
            // short one or has space above it: a heading in the middle of a
            // verse is as much not Scripture as one at its end, but a line of
            // prose wrapped in the middle of a sentence fills its column.
            // (A heading over two lines or more: "Achab, roi d’Israël. — Son
            // mariage avec Jézabel," / "fille du roi des Sidoniens". The lines
            // after a heading-like one that never end a sentence go on with
            // it.)
            open.heading_like_end =
                heading_like || (open.heading_like_end && !ends_as_sentence(&words));
            open.fresh_last_line = fresh;
            if open.last_unit != unit {
                open.before_last_short = Some(open.last_short);
                open.last_short = self.units[unit].short;
            }
            let u = &self.units[unit];
            if heading_like && (u.short || u.gap_before) {
                open.doubt = open.doubt.or(Some("possibleHeading"));
            }
            if set_apart {
                open.doubt = open.doubt.or(Some("possibleHeading"));
            }
            // Words after a heading, before any verse number ("Psaume 118
            // (Vulg. CXVII)" read as a heading / "Au départ de la
            // procession."): not known to be this verse's.
            if self.heading_in_verse && !open.text.trim().is_empty() {
                open.doubt = open.doubt.or(Some("boundary"));
            }
            Self::append(
                open,
                &words,
                unit,
                place,
                glue,
                &self.hyphenated,
                &self.whole,
            );
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
            // Corrected only where the choice differs from what was found
            // (a book outside the 66 is left out unless chosen).
            found.corrected = self.corrections.books.get(&entry).is_some_and(|choice| {
                let default_left_out = found.outside.is_some();
                match choice {
                    Some(book) => Some(book.code) != found.detected || default_left_out,
                    None => !default_left_out,
                }
            });
            // A book outside the 66 is left out unless the person chose a
            // book for it.
            found.left_out = match self.corrections.books.get(&entry) {
                Some(choice) => choice.is_none(),
                None => found.outside.is_some(),
            };
            self.skip = found.left_out;
            // (The note stays while the book is left out, whatever else the
            // person corrected.)
            if let (Some(code), true) = (found.outside, found.left_out) {
                let name = crate::scripture::canons::additional(code).map_or(code, |b| b.name);
                let heading = found.heading.clone();
                self.out.content.issue(
                    Severity::Note,
                    None,
                    None,
                    UserMessage::new(
                        "bibleImport.documentAdditionalBook",
                        format!(
                            "“{heading}” is {name}, which Abiding Word cannot show yet. Its text was left out; the other books are not affected."
                        ),
                    )
                    .with("heading", &heading)
                    .with("book", name)
                    .with("code", code),
                );
            }
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
                        let reason = if subscription(text) {
                            "subscription"
                        } else if reference_line(text, self.language) {
                            "crossReference"
                        } else {
                            "heading"
                        };
                        // The verse's last line, a whole line beginning a
                        // sentence, just before a heading or a line of
                        // references: the first line of the heading, as
                        // likely as the verse's end ("… aucune question." /
                        // "De qui le Christ est-il fils?" / "V. 35-37: cf.
                        // …", headings in the body's type). Not verified.
                        // A line set apart as a heading just before the next
                        // chapter's number, after the verse's text: as likely
                        // the verse's last line ("… les bêtes?" / "Quiconque
                        // passera près d'elle, …" / "3"). Not verified.
                        let before_chapter = matches!(
                            self.items.get(at + 1).map(|i| &i.event),
                            Some(Event::Chapter(_))
                        );
                        if self.open.is_some() {
                            self.heading_in_verse = true;
                        }
                        if let Some(open) = self.open.as_mut() {
                            if open.fresh_last_line {
                                open.doubt = open.doubt.or(Some("possibleHeading"));
                            }
                            if before_chapter && !open.text.trim().is_empty() {
                                open.doubt = open.doubt.or(Some("boundary"));
                            }
                        }
                        self.leave_out(text, item.unit, reason);
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
        self.long_verses();
        self.out.content.check_structure();
        self.out
    }

    /// A verse far longer than the others of its chapter (v0.24.3): more
    /// than five times their median and over 500 characters. The longest
    /// verses of Scripture are about three times their chapter's median
    /// (Esther 8:9); one beyond that has something else read into it (an
    /// introduction, a heading, a column), so its text is not verified.
    fn long_verses(&mut self) {
        for (ordinal, book) in &self.out.content.books {
            for (chapter, text) in &book.chapters {
                if text.verses.len() < 5 {
                    continue;
                }
                let mut lengths: Vec<usize> =
                    text.verses.values().map(|t| t.chars().count()).collect();
                lengths.sort_unstable();
                let median = lengths[lengths.len() / 2];
                for (verse, words) in &text.verses {
                    let n = words.chars().count();
                    if n > 500 && n > 5 * median {
                        self.out
                            .unverified
                            .entry((*ordinal, *chapter, *verse))
                            .or_insert("length");
                    }
                }
            }
        }
    }

    /// Verses missing at the end of a chapter (a chapter cut short) are not
    /// gaps the common checks see: compared with the standard verse counts.
    /// A few fewer is another numbering (a warning); many fewer means the
    /// chapter's end was not found, and its verses are not used (an error).
    /// Far more than usual means another chapter or book was read into it
    /// (an error too).
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
                // Beyond the usual count while the next chapter was never found
                // (2 Timothy 1 running on to "verse 22"): the next chapter's
                // number was missed and its verses read into this one, however
                // few (v0.24.3).
                // (Not where only the book's last chapter is missing: some
                // numberings fold it into the one before, Malachi 3 running
                // to verse 24 in the Hebrew Bible.)
                let next_missing = *chapter < book.chapter_count
                    && !staged.chapters.contains_key(&(chapter + 1))
                    && (chapter + 1 < book.chapter_count
                        || staged.chapters.keys().any(|c| *c > chapter + 1));
                if last > expected + LONG_TOLERANCE.max(expected / 2)
                    || (last > expected && next_missing)
                {
                    found.push((Severity::Error, book, *chapter, last, expected));
                    continue;
                }
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
            if last > expected {
                self.out.content.issue(
                    severity,
                    Some(book),
                    Some(chapter),
                    UserMessage::new(
                        "bibleImport.documentChapterLong",
                        format!(
                            "{} {chapter} goes on to verse {last}; it usually has {expected} verses. Another chapter or book may have been read into it. This chapter is not used.",
                            book.name
                        ),
                    )
                    .with("bookCode", book.code)
                    .with("chapter", chapter)
                    .with("last", last)
                    .with("expected", expected),
                );
                continue;
            }
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
    let stated = language.and_then(ReferenceLanguage::from_label);
    let first = recognise_in(units, stated, corrections);
    // No language stated (a PDF without metadata), or one the document's own
    // titles contradict (LibreOffice writes its interface language, "en-US",
    // into a Korean Bible's PDF): the book titles the document prints say
    // which language it is in, when one language names clearly more of them
    // than any other and the stated one names fewer than one in five. Read
    // again in it: names that differ only by an accent ("Jó", Job; "Jo",
    // João) and the language's own rules then apply. The running text is
    // never used to guess.
    match titles_language(&first.books) {
        Some(language)
            if stated.is_none_or(|s| {
                s != language && {
                    let (hits, titles) = title_hits(&first.books, s);
                    hits * 5 < titles
                }
            }) =>
        {
            recognise_in(units, Some(language), corrections)
        }
        _ => first,
    }
}

/// The found titles, each as the forms its name may take, with its book.
fn title_names(books: &[FoundBook]) -> Vec<(Vec<String>, &'static str)> {
    books
        .iter()
        .filter_map(|b| {
            let code = b.detected?;
            let k = key(&b.heading);
            let mut forms = title_forms(&k);
            forms.push(k);
            Some((forms, code))
        })
        .collect()
}

/// How many of the found titles a language's own book names give, of all.
fn title_hits(books: &[FoundBook], language: ReferenceLanguage) -> (usize, usize) {
    let titles = title_names(books);
    let own: HashSet<(String, &str)> = ReferenceParser::for_language(language)
        .lookup_entries()
        .map(|(name, book)| (key(name), book.code))
        .collect();
    // The Vulgate's titles say "Incipit liber …", "Incipit epistula …", "…
    // id est …" around the name (see `vulgate_title`).
    let latin = language.code() == "la";
    let bare = |f: &str| -> String {
        let mut k = f.to_string();
        if let Some((_, rest)) = k.rsplit_once(" id est ") {
            k = rest.to_string();
        } else if let Some(rest) = k.strip_prefix("id est ") {
            k = rest.to_string();
        }
        for word in ["incipit ", "liber ", "epistula ", "ad "] {
            if let Some(rest) = k.strip_prefix(word) {
                k = rest.to_string();
            }
        }
        k
    };
    let hits = titles
        .iter()
        .filter(|(forms, code)| {
            forms.iter().any(|f| {
                own.contains(&(f.clone(), *code))
                    || (latin
                        && (vulgate_title(f).is_some_and(|b| b.code == *code)
                            || own.contains(&(bare(f), *code))))
            })
        })
        .count();
    (hits, titles.len())
}

/// The language whose own book names the found titles use, when it names at
/// least three of them and three in five, and more than any other language.
fn titles_language(books: &[FoundBook]) -> Option<ReferenceLanguage> {
    let mut candidates = name_languages();
    candidates.extend(ReferenceLanguage::from_label("la"));
    let mut counts: Vec<(usize, usize, ReferenceLanguage)> = candidates
        .into_iter()
        .map(|language| {
            let (hits, titles) = title_hits(books, language);
            (hits, titles, language)
        })
        .collect();
    counts.sort_by_key(|c| std::cmp::Reverse(c.0));
    let (best, titles, language) = *counts.first()?;
    let second = counts.get(1).map_or(0, |c| c.0);
    (best >= 3 && best * 5 >= titles * 3 && best > second).then_some(language)
}

fn recognise_in(
    units: &[Unit],
    language: Option<ReferenceLanguage>,
    corrections: &Corrections,
) -> Recognition {
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
    // A note in a document without pages written as a paragraph of its own,
    // opened by its letter and the verse it names ("a 1:1 Ou “Cristo.” …",
    // gathered under "Notas"): a note (v0.24.3).
    {
        use std::sync::OnceLock;
        static LETTER_NOTE: OnceLock<regex::Regex> = OnceLock::new();
        let letter_note = LETTER_NOTE.get_or_init(|| {
            regex::Regex::new(r"^\p{Ll}{1,2}\s+\d{1,3}[:.,]\d{1,3}\s").expect("valid")
        });
        for (i, u) in units.iter().enumerate() {
            if u.place.page.is_none()
                && !u.heading
                && letter_note.is_match(clean(&u.text).trim_start())
            {
                footnotes.insert(i);
            }
        }
    }
    // A note's further paragraphs in a document without pages (an EPUB's
    // note over several paragraphs, the first marked as a note): a plain
    // paragraph right after a note, with no number at its start, before
    // anything set apart as a heading — never verse text (v0.24.3).
    for i in 1..units.len() {
        let u = &units[i];
        // (A verse's number: digits alone before the words; "24: Væ …", a
        // reference continuing the note, is not one.)
        let text = clean(&u.text);
        let first = text.split_whitespace().next().unwrap_or("");
        let starts_numbered = !first.is_empty()
            && first
                .chars()
                .all(|c| c.is_ascii_digit() || superscript_digit(c).is_some())
            || text
                .trim_start()
                .chars()
                .next()
                .is_some_and(|c| superscript_digit(c).is_some());
        if u.place.page.is_none()
            && footnotes.contains(&(i - 1))
            && !u.note
            && !u.heading
            && !u.styled
            && !starts_numbered
            && units[i - 1].place.document == u.place.document
        {
            footnotes.insert(i);
        }
    }
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
    let note_units: Vec<usize> = {
        let mut v: Vec<usize> = footnotes.iter().copied().collect();
        v.sort_unstable();
        v
    };
    furniture.extend(footnotes);
    let starts = block_starts(units, &furniture);
    let builder = Builder {
        units,
        language,
        items: Vec::new(),
        books: Vec::new(),
        last_detected: None,
        block_starts: starts.clone(),
    };
    let (items, books) = builder.build(&furniture);
    let reader = Reader {
        units,
        items: &items,
        language,
        hyphenated: hyphenated_words(units),
        whole: whole_words(units),
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
        apparatus_unit: None,
        apparatus_chars: 0,
        block_starts: starts,
        inline_labels: inline_labels(units),
        heading_in_verse: false,
    };
    let mut out = reader.run();
    check_running_heads(units, &furniture, language, &mut out);
    // The page furniture and the notes, kept with their places as the rest
    // of the apparatus is (v0.24.3), in the document's order.
    let notes: HashSet<usize> = note_units.iter().copied().collect();
    let mut edge: Vec<usize> = furniture.difference(&notes).copied().collect();
    edge.sort_unstable();
    let mut total = out.apparatus.iter().map(|a| a.text.len()).sum();
    let mut extra = Vec::new();
    for (kind, list) in [("footnote", &note_units), ("runningHead", &edge)] {
        let mut last = None;
        for &i in list.iter() {
            let unit = &units[i];
            add_apparatus(
                &mut extra,
                &mut last,
                &mut total,
                kind,
                &unmark_raised(&clean(&unit.text)),
                unit.place,
                i,
                None,
            );
        }
    }
    // A note names the verse it is on ("a 23:14 …"): the verse of that
    // chapter and number begun on the note's page, or the page before.
    {
        static CITED: OnceLock<regex::Regex> = OnceLock::new();
        let cited = CITED.get_or_init(|| {
            regex::Regex::new(r"^\W{0,3}\p{L}{0,2}\s*(\d{1,3})[:.,](\d{1,3})").expect("valid")
        });
        let mut on_page: HashMap<(u16, u32), Vec<VerseKey>> = HashMap::new();
        for (&key, origin) in &out.origins {
            if let Some(page) = origin.start.page {
                on_page
                    .entry((origin.start.document, page))
                    .or_default()
                    .push(key);
            }
        }
        for piece in extra.iter_mut().filter(|p| p.kind == "footnote") {
            let (Some(page), Some(c)) = (piece.place.page, cited.captures(piece.text.trim_start()))
            else {
                continue;
            };
            let (Ok(chapter), Ok(verse)) = (c[1].parse::<u16>(), c[2].parse::<u16>()) else {
                continue;
            };
            piece.verse = [page, page.saturating_sub(1)]
                .iter()
                .filter_map(|p| on_page.get(&(piece.place.document, *p)))
                .flatten()
                .find(|k| k.1 == chapter && k.2 == verse)
                .copied();
        }
    }
    out.apparatus.extend(extra);
    out.apparatus.sort_by_key(|a| {
        (
            a.place.document,
            a.place.page.unwrap_or(0),
            a.place.segment.unwrap_or(0),
            a.place.offset.unwrap_or(0),
        )
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(title: &str, language: Option<&str>) -> Option<&'static str> {
        book_named(title, language.and_then(ReferenceLanguage::from_label)).map(|b| b.code)
    }

    #[test]
    fn greek_titles_in_their_traditional_shape() {
        let el = ReferenceLanguage::from_label("el");
        let book = |t: &str| match read_heading(t, el) {
            Some(Heading::Book(b)) => Some(b.code),
            _ => None,
        };
        assert_eq!(book("ΚΟΡΙΝΘΙΟΥΣ ΕΠΙΣΤΟΛΗ ΠΡΩΤΗ"), Some("1CO"));
        assert_eq!(book("ΠΕΤΡΟΥ ΕΠΙΣΤΟΛΗ ΚΑΘΟΛΙΚΗ ΠΡΩΤΗ"), Some("1PE"));
        assert_eq!(book("ΕΥΑΓΓΕΛΙΟΝ ΤΟ ΚΑΤΑ ΜΑΤΘΑΙΟΝ"), Some("MAT"));
        assert_eq!(book("ΙΩΑΝΝΟΥ ΕΠΙΣΤΟΛΗ ΤΡΙΤΗ"), Some("3JN"));
        assert_eq!(book("ΠΕΤΡΟΥ ΕΠΙΣΤΟΛΗ ΚΑΘΟΛΙΚΗ ΔΕΥΤΕΡΑ"), Some("2PE"));
        assert_eq!(book("ΠΕΤΡΟΥ ΕΠΙΣΤΟΛΗ ΚΑΘΟΛΙΚΗ"), None);
        assert_eq!(book("Η ΠΡΟΣ ΕΒΡΑΙΟΥΣ"), Some("HEB"));
        // Without its number, Corinthians names neither letter.
        assert_eq!(book("ΚΟΡΙΝΘΙΟΥΣ"), None);
    }

    #[test]
    fn printed_titles_name_their_books() {
        // Darby 1885: the Pentateuch's titles give the book's second name
        // after "dit" (v0.24.3).
        assert_eq!(
            named("Le premier livre de Moïse dit la Genèse", Some("fr")),
            Some("GEN")
        );
        assert_eq!(
            named("Le cinquième livre de Moïse dit le Deutéronome", Some("fr")),
            Some("DEU")
        );
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
        assert_eq!(
            named("Das 3. Buch Mose (Levitikus)", Some("de")),
            Some("LEV")
        );
        assert_eq!(named("Das erste Buch der Könige", Some("de")), Some("1KI"));
        assert_eq!(named("Genèse", Some("fr")), Some("GEN"));
        assert_eq!(named("Jean", Some("fr")), Some("JHN"));
        assert_eq!(named("LA GENÈSE", Some("fr")), Some("GEN"));
        assert_eq!(named("ÉVANGILE SELON LUC", Some("fr")), Some("LUK"));
        assert_eq!(named("Première épître de Pierre", Some("fr")), Some("1PE"));
        assert_eq!(named("Épître de Paul aux Romains", Some("fr")), Some("ROM"));
        assert_eq!(named("Les Actes des Apôtres", Some("fr")), Some("ACT"));
        assert_eq!(
            named("El Santo Evangelio según San Lucas", Some("es")),
            Some("LUK")
        );
        assert_eq!(
            named("Primera epístola de San Pablo a los Corintios", Some("es")),
            Some("1CO")
        );
        assert_eq!(named("Evangelho segundo Lucas", Some("pt")), Some("LUK"));
        assert_eq!(named("Primeira epístola de Pedro", Some("pt")), Some("1PE"));
    }

    #[test]
    fn quotation_references_closing_a_verse_are_found() {
        let pt = Some(ReferenceLanguage::from_label("pt").unwrap());
        let text = "Não adulterarás. Êxodo 21:24; Levítico 24:20;";
        assert_eq!(
            trailing_reference(text, pt),
            Some("Não adulterarás. ".len())
        );
        let text = "e eles te sustentarão. Salmos 91:11,12";
        assert_eq!(
            trailing_reference(text, pt),
            Some("e eles te sustentarão. ".len())
        );
        // A reference inside the verse, or without a sentence before it, stays.
        assert_eq!(trailing_reference("como diz Salmos 91:11 hoje", pt), None);
        assert_eq!(trailing_reference("Salmos 91:11", pt), None);
        // Inside the verse, after the quotation it is the source of.
        let text = "Mas ide aprender o que significa: ‘Quero misericórdia, e não sacrifício’. Oseías 6:6 Porque eu não vim chamar os justos.";
        let found = inline_references(text, pt);
        assert_eq!(found.len(), 1);
        assert_eq!(text[found[0].0..found[0].1].trim(), "Oseías 6:6");
        let text = "não há quem faça o bem, nem um sequer. Salmos 14:1-3; 53:1-3; Eclesiastes 7:20 Todos se desviaram.";
        let found = inline_references(text, pt);
        assert_eq!(found.len(), 1);
        assert_eq!(
            text[found[0].0..found[0].1].trim(),
            "Salmos 14:1-3; 53:1-3; Eclesiastes 7:20"
        );
        // A list whose numbers run into the next book's ordinal ("14:1, 2 Reis").
        let text = "assim está escrito. Salmos 14:1, 2 Reis 3:4 E depois disto.";
        let found = inline_references(text, pt);
        assert_eq!(found.len(), 1);
        assert_eq!(
            text[found[0].0..found[0].1].trim(),
            "Salmos 14:1, 2 Reis 3:4"
        );
        // Never a reference the verse's own words lead into.
        assert!(inline_references("como diz Salmos 91:11 hoje", pt).is_empty());
    }

    #[test]
    fn the_vulgate_s_own_titles_are_read_by_its_table() {
        let la = ReferenceLanguage::from_label("la");
        let code = |t: &str, l| match read_heading(t, l) {
            Some(Heading::Book(b)) => Some(b.code),
            _ => None,
        };
        assert_eq!(
            code("INCIPIT LIBER MALACHIM ID EST REGUM TERTIUS ET QUARTUS", la),
            Some("1KI")
        );
        assert_eq!(
            code("INCIPIT LIBER MALACHIM ID EST REGUM QUARTUS", la),
            Some("2KI")
        );
        assert_eq!(code("INCIPIT LIBER HIEZECHIELIS PROPHETÆ", la), Some("EZK"));
        assert_eq!(code("INCIPIT LIBER ACTUUM APOSTOLORUM", la), Some("ACT"));
        assert_eq!(code("INCIPIT LIBER DABREIAMIN", la), Some("1CH"));
        // Only for Latin; and never "Regum primus" alone (1 Samuel in the
        // Vulgate, 1 Kings in the Nova Vulgata).
        assert_eq!(
            code(
                "INCIPIT LIBER DABREIAMIN",
                ReferenceLanguage::from_label("en")
            ),
            None
        );
        assert_ne!(code("REGUM PRIMUS", la), Some("1SA"));
        assert_eq!(
            outside_named("INCIPIT LIBER TOBIÆ").map(|o| o.0),
            Some("TOB")
        );
    }

    #[test]
    fn korean_prose_is_never_a_line_of_references() {
        let ko = ReferenceLanguage::from_label("ko");
        assert!(!reference_line(
            "7:28) 너는 가증한 것을 네 집에 들이지 말라 너도",
            ko
        ));
        assert!(reference_line("(마 5:3; 눅 6:20)", ko));
    }

    #[test]
    fn a_wrapped_line_of_prose_is_never_a_title() {
        let fr = ReferenceLanguage::from_label("fr");
        let line = "Moïse. Ce dernier changea son nom Osée ou Hoshéa (il sauve, il donne la victoire ou il";
        assert!(!book_line_shape(line));
        assert!(book_line_shape("St. Matthew"));
        assert!(book_line_shape("GENESIS."));
        assert!(book_line_shape("Ecclesiastes or, the Preacher"));
        let pt = ReferenceLanguage::from_label("pt");
        let en = ReferenceLanguage::from_label("en");
        assert_eq!(read_heading("A Song", en), None);
        // An epistle's subscription, read whole or over its wrapped lines;
        // a verse that merely begins "Πρὸς" is none.
        assert!(subscription("Πρὸς Γαλάτας ἐγράφη ἀπὸ Ῥώμης."));
        assert!(subscription(
            "Πρὸς Τίτον τῆς Κρητῶν ἐκκλησίας πρῶτον ἐπίσκοπον χειροτονηθέντα ἐγράφη ἀπὸ Νικοπόλεως"
        ));
        assert!(!subscription("Πρὸς τὸν Θεὸν ἐγράφησαν."));
        assert!(!subscription("Πρὸς ταῦτα τίς ἱκανός;"));
        assert!(subscription("Written to the Romans from Corinthus."));
        assert_eq!(read_heading("A PSALM", en), None);
        assert_eq!(
            read_heading("A Revelação a João", pt),
            Some(Heading::Book(canon::by_code("REV").unwrap()))
        );
        assert_eq!(
            read_heading("O Livro de Jó", pt),
            Some(Heading::Book(canon::by_code("JOB").unwrap()))
        );
        assert_eq!(
            read_heading("LE SECOND LIVRE DE SAMUEL", fr),
            Some(Heading::Book(canon::by_code("2SA").unwrap()))
        );
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
        // A range that does not name its book again.
        assert_eq!(
            head_references("20 GENESIS 8:13–9:15"),
            Some((("GENESIS".into(), 8, 13), ("GENESIS".into(), 9, 15)))
        );
        assert_eq!(
            head_references("GENESIS 1:22–2:13 13"),
            Some((("GENESIS".into(), 1, 22), ("GENESIS".into(), 2, 13)))
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
        // Digits of other scripts are not read as numbers (and never cut).
        assert!(note_letters_in("a 1:1\u{0661}").contains(&("a".into(), 1, 1)));
        let _ = note_letters_in("ａ 1:１２");
        let _ = note_signs_in("* 1:\u{0661}\u{0662}");
        // "g 9:542R1:10": 9:54 is among the readings.
        assert!(note_letters_in("f 9:53Jn4:9. g 9:542R1:10,").contains(&("g".into(), 9, 54)));
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
        assert_eq!(
            without_signs("Χριστοῦ, †ᾧ ἡ δόξα", &['†']),
            "Χριστοῦ, ᾧ ἡ δόξα"
        );
    }
}
