//! A developer's view of what the verse recogniser reads (v0.24.3): the
//! units of one document, with their flags, for a range of pages or units.
//! Not run by default.
//!
//! ```text
//! AW_DUMP_FILE=<file> AW_DUMP_PAGES=120-122 \
//!   cargo test --release --test document_units_dump -- --ignored --nocapture
//! ```
//!
//! `AW_DUMP_UNITS=from-to` selects by unit index instead (documents without
//! pages); `AW_DUMP_FIND=<text>` prints the units containing the text, with
//! two units around each.

use std::path::PathBuf;

use abiding_word_lib::scripture::documents;

fn range(name: &str) -> Option<(usize, usize)> {
    let value = std::env::var(name).ok()?;
    let (a, b) = value.split_once('-').unwrap_or((&value, &value));
    Some((a.trim().parse().ok()?, b.trim().parse().ok()?))
}

#[test]
#[ignore]
fn document_units_dump() {
    let Some(file) = std::env::var_os("AW_DUMP_FILE").map(PathBuf::from) else {
        eprintln!("AW_DUMP_FILE not set; nothing to do");
        return;
    };
    let doc = documents::read(&file, 0).expect("readable document");
    let pages = range("AW_DUMP_PAGES");
    let units = range("AW_DUMP_UNITS");
    let find = std::env::var("AW_DUMP_FIND").ok();
    let wanted: Vec<usize> = match &find {
        Some(needle) => {
            let mut out = Vec::new();
            for (i, u) in doc.units.iter().enumerate() {
                if u.text.contains(needle.as_str()) {
                    out.extend(i.saturating_sub(2)..(i + 3).min(doc.units.len()));
                }
            }
            out.dedup();
            out
        }
        None => (0..doc.units.len())
            .filter(|&i| {
                let u = &doc.units[i];
                pages.is_none_or(|(a, b)| {
                    u.place
                        .page
                        .is_some_and(|p| (a..=b).contains(&(p as usize)))
                }) && units.is_none_or(|(a, b)| (a..=b).contains(&i))
            })
            .collect(),
    };
    // AW_DUMP_RECOGNISE=BOOK: read the selected units for verses and print
    // that book's verses and the issues.
    if let Ok(code) = std::env::var("AW_DUMP_RECOGNISE") {
        let units: Vec<_> = wanted.iter().map(|&i| doc.units[i].clone()).collect();
        let r = abiding_word_lib::scripture::structure::recognise(
            &units,
            std::env::var("AW_DUMP_LANGUAGE").ok().as_deref(),
            &Default::default(),
        );
        let book = abiding_word_lib::scripture::canon::by_code(&code).expect("a book code");
        if let Some(b) = r.content.books.get(&book.ordinal) {
            for (c, ch) in &b.chapters {
                for (v, t) in &ch.verses {
                    let doubt = r.unverified.get(&(book.ordinal, *c, *v));
                    println!("{code} {c}:{v} {doubt:?} «{t}»");
                }
            }
        }
        for issue in &r.content.issues {
            println!("{:?} {}", issue.severity, issue.message);
        }
        return;
    }
    for i in wanted {
        let u = &doc.units[i];
        let flags: String = [
            (u.heading, 'H'),
            (u.note, 'N'),
            (u.large, 'L'),
            (u.styled, 'S'),
            (u.edge, 'E'),
            (u.centred, 'C'),
            (u.uncertain_order, '?'),
        ]
        .iter()
        .map(|(on, c)| if *on { *c } else { '.' })
        .collect();
        if std::env::var_os("AW_DUMP_SEGMENTS").is_some() {
            if let Some(seg) = u.place.segment.and_then(|s| doc.segments.get(s as usize)) {
                println!(
                    "        segment @{:?}: {}",
                    u.place.offset,
                    seg.text.chars().take(90).collect::<String>()
                );
            }
        }
        println!(
            "{i:>7} p{:<5} s{:<6} {flags} {}",
            u.place.page.map_or("-".into(), |p| p.to_string()),
            u.place.segment.map_or("-".into(), |s| s.to_string()),
            u.text.replace(['\u{E000}', '\u{E001}'], "^")
        );
    }
}
