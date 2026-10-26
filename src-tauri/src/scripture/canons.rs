//! Canons beyond the 66 books (v0.24): foundations only.
//!
//! Three things are kept apart, and must stay apart:
//!
//! - **which books** a tradition counts as Scripture (its canon);
//! - **the order** it puts them in;
//! - **how chapters and verses are numbered** (versification,
//!   [`super::versification`]).
//!
//! The 66-book registry ([`super::canon`]) is unchanged and is what every
//! stored reference, verse key (`ordinal × 1 000 000 + chapter × 1 000 +
//! verse`), note and mark uses. Nothing here renumbers it: a book added
//! later gets a new ordinal above 66 ([`AdditionalBook::ordinal`]), so a
//! Protestant reference never changes meaning.
//!
//! What Abiding Word supports today, and no more:
//!
//! - the deuterocanonical books and the Greek additions to Esther and Daniel
//!   are **named** when a Bible's files contain them (instead of "a book it
//!   does not recognise"), and left out of the import with a note;
//! - their text is **not** stored, shown or searched, and references to them
//!   are not recognised in notes or sources;
//! - the orders below describe the traditions; the app still lists the 66
//!   books in the Protestant order.

use serde::Serialize;

/// Why a tradition counts a book outside the 66.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    /// In the Catholic and Orthodox Old Testament (Tobit, Wisdom, 1 Maccabees…).
    Deuterocanonical,
    /// Greek text added to a Hebrew book (Esther, Daniel), kept with it or apart.
    GreekAddition,
    /// In the Septuagint and Orthodox Bibles, not in the Catholic canon.
    Septuagint,
}

/// A book outside the 66, by its USFM identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdditionalBook {
    pub code: &'static str,
    /// English name, as the USFM standard gives it.
    pub name: &'static str,
    pub kind: Kind,
    /// Reserved for when its text can be stored: always above 66, never reused.
    pub ordinal: u8,
}

const fn book(code: &'static str, name: &'static str, kind: Kind, ordinal: u8) -> AdditionalBook {
    AdditionalBook {
        code,
        name,
        kind,
        ordinal,
    }
}

use Kind::{Deuterocanonical as D, GreekAddition as G, Septuagint as S};

/// The books, with ordinals reserved from 67 in USFM's own order.
pub const ADDITIONAL_BOOKS: &[AdditionalBook] = &[
    book("TOB", "Tobit", D, 67),
    book("JDT", "Judith", D, 68),
    book("ESG", "Esther (Greek)", G, 69),
    book("WIS", "Wisdom of Solomon", D, 70),
    book("SIR", "Sirach", D, 71),
    book("BAR", "Baruch", D, 72),
    book("LJE", "Letter of Jeremiah", D, 73),
    book("S3Y", "Song of the Three Young Men", G, 74),
    book("SUS", "Susanna", G, 75),
    book("BEL", "Bel and the Dragon", G, 76),
    book("1MA", "1 Maccabees", D, 77),
    book("2MA", "2 Maccabees", D, 78),
    book("3MA", "3 Maccabees", S, 79),
    book("4MA", "4 Maccabees", S, 80),
    book("1ES", "1 Esdras (Greek)", S, 81),
    book("MAN", "Prayer of Manasseh", S, 82),
    book("PS2", "Psalm 151", S, 83),
    book("DAG", "Daniel (Greek)", G, 84),
    book("2ES", "2 Esdras (Latin)", S, 85),
];

pub fn additional(code: &str) -> Option<&'static AdditionalBook> {
    ADDITIONAL_BOOKS.iter().find(|b| b.code == code)
}

/// A tradition's canon and the order of its Old Testament.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Tradition {
    /// 66 books: the registry Abiding Word uses.
    Protestant,
    /// 73 books: Esther and Daniel with their Greek parts; the Letter of
    /// Jeremiah as Baruch's sixth chapter.
    Catholic,
    /// The Septuagint's books and order, as in Greek Orthodox Bibles; other
    /// Orthodox churches differ in a few books (and 4 Maccabees is often an
    /// appendix).
    Septuagint,
}

const NEW_TESTAMENT: &[&str] = &[
    "MAT", "MRK", "LUK", "JHN", "ACT", "ROM", "1CO", "2CO", "GAL", "EPH", "PHP", "COL", "1TH",
    "2TH", "1TI", "2TI", "TIT", "PHM", "HEB", "JAS", "1PE", "2PE", "1JN", "2JN", "3JN", "JUD",
    "REV",
];

const CATHOLIC_OT: &[&str] = &[
    "GEN", "EXO", "LEV", "NUM", "DEU", "JOS", "JDG", "RUT", "1SA", "2SA", "1KI", "2KI", "1CH",
    "2CH", "EZR", "NEH", "TOB", "JDT", "ESG", "1MA", "2MA", "JOB", "PSA", "PRO", "ECC", "SNG",
    "WIS", "SIR", "ISA", "JER", "LAM", "BAR", "EZK", "DAG", "HOS", "JOL", "AMO", "OBA", "JON",
    "MIC", "NAM", "HAB", "ZEP", "HAG", "ZEC", "MAL",
];

const SEPTUAGINT_OT: &[&str] = &[
    "GEN", "EXO", "LEV", "NUM", "DEU", "JOS", "JDG", "RUT", "1SA", "2SA", "1KI", "2KI", "1CH",
    "2CH", "1ES", "EZR", "NEH", "TOB", "JDT", "ESG", "1MA", "2MA", "3MA", "PSA", "PS2", "MAN",
    "JOB", "PRO", "ECC", "SNG", "WIS", "SIR", "HOS", "AMO", "MIC", "JOL", "OBA", "JON", "NAM",
    "HAB", "ZEP", "HAG", "ZEC", "MAL", "ISA", "JER", "BAR", "LAM", "LJE", "EZK", "DAG", "4MA",
];

impl Tradition {
    /// The books in this tradition's order, by USFM identifier. A Greek book
    /// that contains a Hebrew one (ESG, DAG) stands in its place: the
    /// Hebrew book's verses are the same verses, not a second book.
    pub fn books(self) -> Vec<&'static str> {
        let old: Vec<&'static str> = match self {
            Tradition::Protestant => super::canon::BOOKS
                .iter()
                .take(39)
                .map(|b| b.code)
                .collect(),
            Tradition::Catholic => CATHOLIC_OT.to_vec(),
            Tradition::Septuagint => SEPTUAGINT_OT.to_vec(),
        };
        old.into_iter()
            .chain(NEW_TESTAMENT.iter().copied())
            .collect()
    }
}

/// The 66-book identity of a Greek book that contains a Hebrew one: its
/// shared verses keep the Hebrew book's references.
pub fn contains(code: &str) -> Option<&'static str> {
    match code {
        "ESG" => Some("EST"),
        "DAG" => Some("DAN"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripture::canon::{self, BOOKS, NON_CANONICAL_USFM_CODES};
    use std::collections::HashSet;

    #[test]
    fn the_66_books_keep_their_ordinals_and_new_books_never_reuse_one() {
        for (index, b) in BOOKS.iter().enumerate() {
            assert_eq!(usize::from(b.ordinal), index + 1);
        }
        let mut seen = HashSet::new();
        for b in ADDITIONAL_BOOKS {
            assert!(b.ordinal > 66, "{}", b.code);
            assert!(seen.insert(b.ordinal), "{}", b.code);
            assert!(canon::by_code(b.code).is_none(), "{}", b.code);
            // The importer already sets them aside rather than calling them unknown.
            assert!(NON_CANONICAL_USFM_CODES.contains(&b.code), "{}", b.code);
        }
    }

    #[test]
    fn each_tradition_keeps_every_protestant_book_and_the_new_testament_order() {
        let protestant = Tradition::Protestant.books();
        assert_eq!(protestant.len(), 66);
        assert_eq!(protestant, BOOKS.iter().map(|b| b.code).collect::<Vec<_>>());
        for tradition in [Tradition::Catholic, Tradition::Septuagint] {
            let books = tradition.books();
            let unique: HashSet<_> = books.iter().collect();
            assert_eq!(unique.len(), books.len(), "{tradition:?}");
            for b in BOOKS.iter() {
                let present = books.contains(&b.code)
                    || ADDITIONAL_BOOKS
                        .iter()
                        .any(|a| books.contains(&a.code) && contains(a.code) == Some(b.code));
                assert!(present, "{tradition:?} lacks {}", b.code);
            }
            for code in &books {
                assert!(
                    canon::by_code(code).is_some() || additional(code).is_some(),
                    "{code}"
                );
            }
            assert_eq!(&books[books.len() - 27..], NEW_TESTAMENT);
        }
        assert_eq!(Tradition::Catholic.books().len(), 73);
    }

    #[test]
    fn greek_books_that_contain_a_hebrew_one_point_at_it() {
        assert_eq!(contains("ESG"), Some("EST"));
        assert_eq!(contains("DAG"), Some("DAN"));
        assert_eq!(contains("TOB"), None);
    }
}
