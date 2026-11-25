//! How a recognised document numbers its chapters and verses, measured
//! against the app's reference numbering (Phase A1.5).
//!
//! The recogniser keeps the numbers a document prints. It never renumbers:
//! a Hebrew-numbered Psalter keeps its titles as verse 1, a Vulgate Psalter
//! its Psalm 9 of 39 verses. Each verse is filed under the app's book, the
//! printed chapter and the printed verse. "Verified" then means that the
//! text is the document's text at that printed number, not that the number
//! names the same verse in another edition (independent canon review,
//! findings 1, 2 and 9).
//!
//! Anything that joins verses of different editions by book, chapter and
//! verse must therefore know where an edition's numbering departs from the
//! reference. This module says so, per book, from the recognised content
//! alone: no other tradition's table is assumed, so nothing is guessed.
//!
//! The reference is `canon::verse_count`: for each chapter, the largest
//! count across five English Protestant systems. A book's numbering is
//! [`Agreement::Departs`] when a chapter is printed beyond the reference's
//! last chapter, or a chapter's last printed verse is beyond the reference's
//! last verse of that chapter: no English numbering has such a verse, so
//! the document numbers otherwise. A chapter that ends sooner than the
//! reference is listed as [`Difference::Shorter`] but does not by itself
//! make the book depart: English editions omit verses too (Matthew 17:21,
//! 3 John 15).

use serde::Serialize;

use super::canon;
use super::import::StagedContent;

/// How one chapter's printed numbering differs from the reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Difference {
    /// A chapter the reference does not have (Joel 4, Malachi 4 missing
    /// is the other way: see `chapters_printed`).
    ChapterBeyond { chapter: u16 },
    /// Its last printed verse is beyond the reference's last verse.
    Longer {
        chapter: u16,
        printed: u16,
        reference: u16,
    },
    /// Its last printed verse is before the reference's last verse.
    Shorter {
        chapter: u16,
        printed: u16,
        reference: u16,
    },
}

/// Whether a book's printed numbering agrees with the reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Agreement {
    /// No chapter beyond the reference's, and no verse beyond any of its
    /// chapters' last verses. (Not proof of English numbering: a numbering
    /// can differ only inside a chapter.)
    WithinReference,
    /// A chapter or a verse the reference does not have: the document
    /// numbers this book otherwise. Its verses must not be joined with
    /// another edition's by number without a mapping.
    Departs,
}

/// One book's numbering, as recognised.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookNumbering {
    pub code: &'static str,
    pub agreement: Agreement,
    /// The highest chapter number recognised in the document.
    pub chapters_printed: u16,
    pub chapters_reference: u16,
    pub differences: Vec<Difference>,
}

/// The numbering of every book the content holds, in canon order.
pub fn assess(content: &StagedContent) -> Vec<BookNumbering> {
    let mut out = Vec::new();
    for (ordinal, book) in &content.books {
        let Some(canon_book) = canon::by_ordinal(*ordinal) else {
            continue;
        };
        let mut differences = Vec::new();
        for (chapter, staged) in &book.chapters {
            // A bridge's last verse counts as printed ("24-25" prints 25).
            let printed = staged
                .verses
                .keys()
                .map(|v| staged.bridges.get(v).copied().unwrap_or(*v))
                .max();
            let Some(printed) = printed else {
                continue;
            };
            match canon::verse_count(canon_book, *chapter) {
                None => differences.push(Difference::ChapterBeyond { chapter: *chapter }),
                Some(reference) if printed > reference => differences.push(Difference::Longer {
                    chapter: *chapter,
                    printed,
                    reference,
                }),
                Some(reference) if printed < reference => differences.push(Difference::Shorter {
                    chapter: *chapter,
                    printed,
                    reference,
                }),
                Some(_) => {}
            }
        }
        let departs = differences.iter().any(|d| {
            matches!(
                d,
                Difference::ChapterBeyond { .. } | Difference::Longer { .. }
            )
        });
        out.push(BookNumbering {
            code: canon_book.code,
            agreement: if departs {
                Agreement::Departs
            } else {
                Agreement::WithinReference
            },
            chapters_printed: book.chapters.keys().copied().max().unwrap_or(0),
            chapters_reference: canon_book.chapter_count,
            differences,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripture::import::{StagedBook, StagedChapter};

    fn content(code: &str, chapters: &[(u16, u16)]) -> StagedContent {
        let mut content = StagedContent::default();
        let mut book = StagedBook::default();
        for &(chapter, last) in chapters {
            let mut c = StagedChapter::default();
            for v in 1..=last {
                c.verses.insert(v, "invented".into());
            }
            book.chapters.insert(chapter, c);
        }
        content
            .books
            .insert(canon::by_code(code).unwrap().ordinal, book);
        content
    }

    #[test]
    fn a_psalter_numbering_its_titles_departs() {
        // Psalm 3: 8 verses in English numbering, 9 in Hebrew (the title is
        // verse 1).
        let hebrew = assess(&content("PSA", &[(3, 9)]));
        assert_eq!(hebrew[0].agreement, Agreement::Departs);
        assert_eq!(
            hebrew[0].differences,
            vec![Difference::Longer {
                chapter: 3,
                printed: 9,
                reference: 8
            }]
        );
        let english = assess(&content("PSA", &[(3, 8)]));
        assert_eq!(english[0].agreement, Agreement::WithinReference);
        assert!(english[0].differences.is_empty());
    }

    #[test]
    fn a_vulgate_psalm_nine_departs() {
        // Psalms 9 and 10 printed as one Psalm 9 of 39 verses.
        let vulgate = assess(&content("PSA", &[(9, 39), (10, 7)]));
        assert_eq!(vulgate[0].agreement, Agreement::Departs);
        assert!(vulgate[0].differences.contains(&Difference::Shorter {
            chapter: 10,
            printed: 7,
            reference: 18
        }));
    }

    #[test]
    fn a_fourth_chapter_of_joel_departs_and_a_shorter_chapter_alone_does_not() {
        let joel = assess(&content("JOL", &[(1, 20), (2, 27), (3, 5), (4, 21)]));
        assert_eq!(joel[0].agreement, Agreement::Departs);
        assert!(joel[0]
            .differences
            .contains(&Difference::ChapterBeyond { chapter: 4 }));
        assert_eq!(joel[0].chapters_printed, 4);
        // Matthew 17 without verse 21 is still the reference's numbering;
        // a chapter ending at 26 instead of 27 is only listed.
        let short = assess(&content("MAT", &[(17, 26)]));
        assert_eq!(short[0].agreement, Agreement::WithinReference);
        assert_eq!(short[0].differences.len(), 1);
    }
}
