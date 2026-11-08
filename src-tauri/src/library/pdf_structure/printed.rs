//! A printed book index can corroborate an actual opening heading. A listing
//! line is NEVER itself a book section. No targets are invented from book order.
use super::{Anchor, Heading};
use crate::scripture::language::ReferenceLanguage;
use regex::Regex;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::OnceLock,
};

pub(super) fn promote(
    lines: &[(Anchor, String, bool, u32)],
    labels: &BTreeMap<u32, String>,
    headings: &mut [Heading],
) {
    static ENTRY: OnceLock<Regex> = OnceLock::new();
    let pattern = ENTRY.get_or_init(|| {
        Regex::new(r"^(.+?)(?:\s*[.·…](?:[.·…\s])*|\s{2,})([0-9]{1,4}|[ivxlcdmIVXLCDM]{1,8})\s*$")
            .unwrap()
    });
    let mut pages: HashMap<u32, Vec<(&'static str, String)>> = HashMap::new();
    for (anchor, role, uncertain, _) in lines {
        if *uncertain || matches!(role.as_str(), "header" | "footer" | "note" | "artifact") {
            continue;
        }
        let Some(caps) = pattern.captures(anchor.original_text.trim()) else {
            continue;
        };
        let Some(book) = super::resolve::book_title(caps[1].trim(), ReferenceLanguage::En) else {
            continue;
        };
        pages
            .entry(anchor.page)
            .or_default()
            .push((book.code, caps[2].to_lowercase()));
    }
    let listings: HashSet<u32> = pages
        .iter()
        .filter(|(_, entries)| {
            entries
                .iter()
                .map(|(book, _)| *book)
                .collect::<HashSet<_>>()
                .len()
                >= 3
        })
        .map(|(page, _)| *page)
        .collect();
    let mut openings: HashMap<&str, Vec<usize>> = HashMap::new();
    for (i, heading) in headings.iter().enumerate() {
        if listings.contains(&heading.anchor.page) {
            continue;
        }
        if let Some(book) = super::resolve::book_title(&heading.title, ReferenceLanguage::En) {
            openings.entry(book.code).or_default().push(i);
        }
    }
    for page in &listings {
        let entries = &pages[page];
        let mut candidates = Vec::new();
        let mut offsets: HashMap<i64, HashSet<&str>> = HashMap::new();
        for (book, label) in entries {
            let Some(found) = openings.get(book) else {
                continue;
            };
            // Duplicate book openings are ambiguous unless actual page labels
            // disambiguate the target. Never choose the first merely by order.
            let labelled: Vec<_> = found
                .iter()
                .copied()
                .filter(|i| {
                    labels
                        .get(&headings[*i].anchor.page)
                        .is_some_and(|v| v.to_lowercase() == *label)
                })
                .collect();
            if labelled.len() == 1 {
                candidates.push((labelled[0], None));
                continue;
            }
            if found.len() != 1 {
                continue;
            }
            let Ok(printed) = label.parse::<i64>() else {
                continue;
            };
            let offset = i64::from(headings[found[0]].anchor.page) - printed;
            offsets.entry(offset).or_default().insert(book);
            candidates.push((found[0], Some(offset)));
        }
        for (i, offset) in candidates {
            if offset.is_none_or(|v| offsets.get(&v).is_some_and(|books| books.len() >= 3))
                && headings[i].basis == "layout"
            {
                headings[i].basis = "printed".into();
            }
        }
    }
}
