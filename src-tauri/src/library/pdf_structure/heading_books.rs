//! Book labels in actual source headings. This index is deliberately separate
//! from prose reference detection and from Scripture-text recognition.
//!
//! Reuse every installed reference dictionary, including its explicitly
//! ambiguous spellings. Matching is exact after the parser's Unicode and
//! punctuation preparation; a lossy accent fold must never make `Jó` and `Jo`
//! depend on HashMap iteration order, or remove an Indic vowel sign.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use crate::scripture::{
    canon::Book,
    language::ReferenceLanguage,
    packs::{self, Script},
    parser::{normalise, ReferenceParser},
};

#[derive(Default)]
struct Names {
    own: HashMap<ReferenceLanguage, HashMap<String, Vec<&'static Book>>>,
    all: HashMap<String, Vec<&'static Book>>,
    reserved_stems: HashSet<String>,
    chapter_units: Vec<&'static str>,
    verse_units: Vec<&'static str>,
}

fn key(text: &str) -> String {
    let text = text.trim().trim_matches(|c: char| {
        matches!(
            c,
            '"' | '\'' | '“' | '”' | '‘' | '’' | '《' | '》' | '「' | '」'
        )
    });
    let value = normalise(text.trim_end_matches(':'));
    // Numbered books allow `1.Mose`, `1 Mose` and `1Mose`. Keep other
    // word boundaries: `I Sa` (1 Samuel) is not `Isa` (Isaiah).
    let digits = value.chars().take_while(char::is_ascii_digit).count();
    if digits > 0 && value.len() > digits {
        format!("{} {}", &value[..digits], value[digits..].trim_start())
    } else {
        value
    }
}

fn insert(table: &mut HashMap<String, Vec<&'static Book>>, name: String, book: &'static Book) {
    let books = table.entry(name).or_default();
    if !books.iter().any(|old| old.code == book.code) {
        books.push(book);
    }
}

fn names() -> &'static Names {
    static NAMES: OnceLock<Names> = OnceLock::new();
    NAMES.get_or_init(|| {
        let mut names = Names {
            chapter_units: vec!["장", "편"],
            verse_units: vec!["절"],
            ..Names::default()
        };
        let mut reviewed_names = HashSet::new();
        for language in ReferenceLanguage::all() {
            let dictionary = packs::pack(language).dictionary();
            for unit in dictionary.syntax.chapter_units {
                if !names.chapter_units.contains(unit) {
                    names.chapter_units.push(unit);
                }
            }
            for unit in dictionary.syntax.verse_units {
                if !names.verse_units.contains(unit) {
                    names.verse_units.push(unit);
                }
            }
            let own = names.own.entry(language).or_default();
            // Include raw entries too: collisions discovered while building
            // the parser are absent from lookup_entries, even when not listed
            // separately in the dictionary's ambiguous collection.
            for entry in &dictionary.entries {
                let alias = key(&entry.alias);
                insert(own, alias.clone(), entry.book);
                insert(&mut names.all, alias, entry.book);
            }
            for (alias, book) in ReferenceParser::for_language(language).lookup_entries() {
                let alias = key(alias);
                if ReferenceLanguage::REVIEWED.contains(&language) {
                    reviewed_names.insert(alias.clone());
                    if let Some((number, rest)) = alias.split_once(' ') {
                        if matches!(number, "1" | "2" | "3" | "4" | "5" | "i" | "ii" | "iii") {
                            names.reserved_stems.insert(rest.to_string());
                        }
                    }
                }
                insert(own, alias.clone(), book);
                insert(&mut names.all, alias, book);
            }
            // The parser omits ambiguous spellings from lookup_entries. Keep
            // their contenders here so fallback cannot silently pick one.
            for (alias, books) in dictionary.ambiguous {
                for book in books {
                    let alias = key(&alias);
                    insert(own, alias.clone(), book);
                    insert(&mut names.all, alias, book);
                }
            }
            if language == ReferenceLanguage::De {
                for entry in dictionary.entries {
                    // Conventional German keyboard spellings, generated from
                    // the existing dictionary rather than guessed in prose.
                    let alias = entry
                        .alias
                        .replace('ä', "ae")
                        .replace('ö', "oe")
                        .replace('ü', "ue")
                        .replace('ß', "ss");
                    let alias = key(&alias);
                    insert(own, alias.clone(), entry.book);
                    insert(&mut names.all, alias, entry.book);
                    // Formal German Pentateuch headings retain the same
                    // dictionary identity: `1. Buch Mose`, `erstes Buch Mose`.
                    for (n, ordinal) in [
                        ("1", "erste"),
                        ("2", "zweite"),
                        ("3", "dritte"),
                        ("4", "vierte"),
                        ("5", "fünfte"),
                    ] {
                        if entry.alias != format!("{n} mose") {
                            continue;
                        }
                        for label in [
                            format!("{n} buch mose"),
                            format!("{ordinal} buch mose"),
                            format!("{ordinal}s buch mose"),
                        ] {
                            for alias in [label.clone(), label.replace('ü', "ue")] {
                                let alias = key(&alias);
                                insert(own, alias.clone(), entry.book);
                                insert(&mut names.all, alias, entry.book);
                            }
                        }
                    }
                }
            }
        }
        // The same numbered-name safety used by typed references: an
        // unnumbered `Sam`/`Könige` cannot acquire a different foreign book.
        // A genuine unnumbered book such as John/Juan remains available.
        names
            .reserved_stems
            .retain(|stem| !reviewed_names.contains(stem));
        names
            .chapter_units
            .sort_by_key(|unit| std::cmp::Reverse(unit.len()));
        names
            .verse_units
            .sort_by_key(|unit| std::cmp::Reverse(unit.len()));
        names
    })
}

pub(super) fn chapter_units() -> &'static [&'static str] {
    &names().chapter_units
}

pub(super) fn verse_units() -> &'static [&'static str] {
    &names().verse_units
}

fn unique(books: &[&'static Book]) -> Option<&'static Book> {
    match books {
        [book] => Some(*book),
        _ => None,
    }
}

/// A declared source language can resolve its own short names. Otherwise
/// every installed dictionary must agree, and short foreign abbreviations
/// remain conservative. Two-character Han/Hangul names are complete names
/// in several books, so the Latin three-letter rule does not apply to them.
pub(super) fn lookup(title: &str, language: ReferenceLanguage) -> Option<&'static Book> {
    let wanted = key(title);
    let names = names();
    if let Some(books) = names.own.get(&language).and_then(|own| own.get(&wanted)) {
        return unique(books);
    }
    if names.reserved_stems.contains(&wanted) {
        return None;
    }
    let letters: Vec<_> = wanted.chars().filter(|c| c.is_alphabetic()).collect();
    let minimum = if letters
        .iter()
        .all(|c| matches!(Script::of(*c), Some(Script::Han | Script::Hangul)))
    {
        2
    } else {
        3
    };
    if letters.len() < minimum {
        return None;
    }
    // An ordinary word of the source's own language does not become a book
    // solely because a different dictionary has that short alias.
    if packs::pack(language)
        .ordinary_words
        .iter()
        .any(|word| key(word) == wanted)
    {
        return None;
    }
    names.all.get(&wanted).and_then(|books| unique(books))
}
