//! Invented structural headings only. These cases do not import a Bible or
//! relax the recognizer's policy for incidental references in running prose.
use abiding_word_lib::{
    library::{
        pdf_structure::{self, Anchor, Heading, PdfStructure},
        sections,
    },
    scripture::{canon, language::ReferenceLanguage, packs, recognise::Recogniser},
};
use std::collections::BTreeMap;

fn resolved(
    labels: &[(&str, u8)],
    language: ReferenceLanguage,
) -> Vec<pdf_structure::ResolvedHeading> {
    let headings = labels
        .iter()
        .enumerate()
        .map(|(i, (label, level))| Heading {
            id: format!("heading-{i}"),
            title: (*label).into(),
            level: *level,
            parents: vec![],
            basis: "outline".into(),
            segment_position: i as i64,
            start: 0,
            end: label.chars().count() as u32,
            disabled: false,
            mapping: None,
            mapping_scope: None,
            anchor: Anchor {
                page: i as u32 + 1,
                page_width: 600.0,
                page_height: 800.0,
                x0: 40.0,
                x1: 550.0,
                baseline: 100.0,
                size: 20.0,
                first_char: 0,
                column: 0,
                byte_start: 0,
                byte_end: label.len(),
                original_text: (*label).into(),
            },
        })
        .collect();
    pdf_structure::effective(
        &PdfStructure {
            version: pdf_structure::VERSION,
            analysis: pdf_structure::ANALYSIS,
            page_count: labels.len() as u32,
            complete: true,
            tagged: false,
            unaligned_tags: 0,
            headings,
            blocks: vec![],
            printed_contents: vec![],
        },
        &[],
        language,
    )
}

fn whole(code: &str) -> String {
    let book = canon::by_code(code).unwrap();
    if book.chapter_count == 1 {
        format!("{code}.1")
    } else {
        format!("{code}.1-{code}.{}", book.chapter_count)
    }
}

#[test]
fn german_variants_keep_one_book_identity_and_the_actual_hierarchy() {
    for (heading, code) in [
        ("Sprüche", "PRO"),
        ("Sprichwörter", "PRO"),
        ("Sprueche", "PRO"),
        ("SPRICHWOERTER", "PRO"),
        ("Sprüche (Sprichwörter)", "PRO"),
        ("Sprüche / Sprichwörter", "PRO"),
        ("Prediger", "ECC"),
        ("Kohelet", "ECC"),
        ("Prediger (Kohelet)", "ECC"),
        ("1. Mose", "GEN"),
        ("1.Mose", "GEN"),
        ("1Mose", "GEN"),
        ("Genesis", "GEN"),
        ("1. Mose (Genesis)", "GEN"),
        ("1. Buch Mose", "GEN"),
        ("Das erste Buch Mose", "GEN"),
        ("Erstes Buch Mose", "GEN"),
        ("Fuenftes Buch Mose", "DEU"),
        ("Matthaeus", "MAT"),
        ("Roemer", "ROM"),
    ] {
        for language in [ReferenceLanguage::De, ReferenceLanguage::En] {
            let out = resolved(
                &[(heading, 1), ("Kapitel 3", 2), ("Verse 1–4", 3)],
                language,
            );
            let expected = [
                whole(code),
                format!("{code}.3"),
                format!("{code}.3.1-{code}.3.4"),
            ];
            for (i, expected) in expected.iter().enumerate() {
                assert_eq!(
                    out[i].canonical.as_ref(),
                    Some(expected),
                    "{} {heading}: {out:?}",
                    language.code()
                );
                assert_eq!(out[i].heading.anchor.page, i as u32 + 1);
                assert_eq!(
                    out[i].heading.anchor.original_text,
                    [heading, "Kapitel 3", "Verse 1–4"][i]
                );
            }
        }
    }
}

#[test]
fn every_installed_language_uses_its_available_book_dictionary_for_headings() {
    let mut checked = 0;
    let mut unresolved = Vec::new();
    for language in ReferenceLanguage::all() {
        let mut by_book: BTreeMap<&str, Vec<String>> = BTreeMap::new();
        for entry in packs::pack(language).dictionary().entries {
            by_book
                .entry(entry.book.code)
                .or_default()
                .push(entry.alias);
        }
        assert!(!by_book.is_empty(), "{} has no dictionary", language.code());
        for (code, mut aliases) in by_book {
            aliases.sort_by_key(|alias| std::cmp::Reverse(alias.chars().count()));
            let expected = whole(code);
            let has_heading = aliases.iter().any(|alias| {
                let out = resolved(&[(alias, 1)], language);
                out[0].canonical.as_deref() == Some(expected.as_str()) && out[0].scope == "book"
            });
            // The existing canon-ambiguity rule deliberately withholds bare
            // `Jud`. Igbo's supplied JUD data contains only that abbreviation;
            // do not invent a full Igbo name or weaken the safety rule.
            if language.code() == "ig" && code == "JUD" {
                assert!(aliases
                    .iter()
                    .all(|alias| alias.eq_ignore_ascii_case("jud")));
                assert!(!has_heading);
            } else if !has_heading {
                unresolved.push(format!("{} {code}: {aliases:?}", language.code()));
            }
            checked += 1;
        }
    }
    assert!(
        checked > 6_000,
        "only {checked} language/book pairs checked"
    );
    assert!(
        unresolved.is_empty(),
        "unresolved catalogue pairs: {unresolved:#?}"
    );
}

#[test]
fn unambiguous_full_foreign_names_work_without_matching_source_metadata() {
    for (heading, code) in [
        ("箴言", "PRO"),
        ("욥기", "JOB"),
        ("उत्पत्ति", "GEN"),
        ("Księga Rodzaju", "GEN"),
        ("Genesi", "GEN"),
        ("Книга Бытия", "GEN"),
        ("سفر الأمثال", "PRO"),
        ("בראשית", "GEN"),
        ("Κατὰ Ματθαῖον", "MAT"),
        ("ヨハネによる福音書", "JHN"),
    ] {
        let out = resolved(&[(heading, 1)], ReferenceLanguage::En);
        assert_eq!(out[0].canonical, Some(whole(code)), "{heading}: {out:?}");
    }
}

#[test]
fn exact_accents_and_word_boundaries_do_not_choose_a_different_book() {
    for (heading, language, code) in [
        ("Jó", ReferenceLanguage::Pt, "JOB"),
        ("Jo", ReferenceLanguage::Pt, "JHN"),
        ("Jo\u{301}", ReferenceLanguage::Pt, "JOB"),
        ("I Sa", ReferenceLanguage::En, "1SA"),
        ("Isa", ReferenceLanguage::En, "ISA"),
    ] {
        let out = resolved(&[(heading, 1)], language);
        assert_eq!(out[0].canonical, Some(whole(code)), "{heading}: {out:?}");
    }
    for heading in [
        "Johannes (Judas)",
        "Sprüche (Prediger)",
        "Jud",
        "1 Esdras",
        "2 Kingdoms",
        "Sam",
        "Könige",
    ] {
        let out = resolved(&[(heading, 1)], ReferenceLanguage::De);
        assert!(
            out[0].canonical.is_none(),
            "must remain unresolved: {heading}: {out:?}"
        );
    }
    let language = ReferenceLanguage::from_label("ar").unwrap();
    let out = resolved(&[("رسالة boutros", 1)], language);
    assert!(
        out[0].canonical.is_none(),
        "dictionary ambiguity must be retained: {out:?}"
    );
}

#[test]
fn localized_numeric_suffixes_and_children_share_the_same_book_identity() {
    for (heading, expected) in [
        ("Sprueche 3,5–6", "PRO.3.5-PRO.3.6"),
        ("Sprichwoerter 3：5 – 6", "PRO.3.5-PRO.3.6"),
        ("1. Mose (Genesis) 1:1", "GEN.1.1"),
        ("בראשית ١:١", "GEN.1.1"),
        ("उत्पत्ति ३:५", "GEN.3.5"),
        ("約翰福音 ３章１６節", "JHN.3.16"),
        ("ヨハネによる福音書3章16節", "JHN.3.16"),
        ("요한복음 3장 16절", "JHN.3.16"),
        ("John 3: 16 – 18 — Love", "JHN.3.16-JHN.3.18"),
    ] {
        let out = resolved(&[(heading, 1)], ReferenceLanguage::En);
        assert_eq!(
            out[0].canonical.as_deref(),
            Some(expected),
            "{heading}: {out:?}"
        );
    }
    let paths = ["箴言", "箴言 › 第３章", "箴言 › 第３章 › ５‑６節"];
    let out = sections::derive(
        paths.iter().map(|p| Some(*p)),
        Recogniser::for_source(ReferenceLanguage::En),
    );
    for (index, expected) in [(0, "PRO.1-PRO.31"), (1, "PRO.3"), (2, "PRO.3.5-PRO.3.6")] {
        assert!(
            out.iter()
                .any(|link| link.segment_index == index && link.canonical == expected),
            "{expected}: {out:?}"
        );
    }
}

#[test]
fn explicit_single_chapter_books_keep_the_shared_verse_shorthand() {
    for (title, expected) in [
        ("Jude 1", "JUD.1.1"),
        ("Jude 5–7", "JUD.1.5-JUD.1.7"),
        ("Philemon 1", "PHM.1.1"),
        ("2 John 1", "2JN.1.1"),
        ("Jude Chapter 1", "JUD.1"),
        ("Jude 1:1", "JUD.1.1"),
    ] {
        let out = resolved(&[(title, 1)], ReferenceLanguage::En);
        assert_eq!(
            out[0].canonical.as_deref(),
            Some(expected),
            "{title}: {out:?}"
        );
    }
    let out = resolved(
        &[("Jude", 1), ("1", 2), ("Verses 1–4", 3)],
        ReferenceLanguage::En,
    );
    assert_eq!(out[1].canonical.as_deref(), Some("JUD.1"));
    assert_eq!(out[1].scope, "chapter");
    assert_eq!(out[2].canonical.as_deref(), Some("JUD.1.1-JUD.1.4"));
}

#[test]
fn repeated_localized_units_keep_complete_bookless_ranges() {
    for (title, expected) in [
        ("1장 1절–2절", "JHN.1.1-JHN.1.2"),
        ("1장 1절–2장 2절", "JHN.1.1-JHN.2.2"),
        ("1章1節–2章2節", "JHN.1.1-JHN.2.2"),
        ("1장–2장", "JHN.1-JHN.2"),
    ] {
        let out = resolved(&[("요한복음", 1), (title, 2)], ReferenceLanguage::En);
        assert_eq!(
            out[1].canonical.as_deref(),
            Some(expected),
            "{title}: {out:?}"
        );
    }
    for title in ["1절2절", "1節2節"] {
        let out = resolved(
            &[("요한복음", 1), ("Chapter 1", 2), (title, 3)],
            ReferenceLanguage::En,
        );
        assert!(
            out[2].canonical.is_none(),
            "adjacent verse tokens must not become verse 12: {title}: {out:?}"
        );
    }
    let out = resolved(&[("Jude 1절2절", 1)], ReferenceLanguage::En);
    assert!(
        out[0].canonical.is_none(),
        "a named reference must not concatenate verse tokens: {out:?}"
    );
}
