//! A printed contents entry supplies a title and printed page, never a body
//! citation. Its destination must be corroborated against physical pages and
//! anchored to an actual heading. No targets are invented from book order.
use super::{native::Candidate, Anchor, Heading, PrintedDestination, MAX_HEADINGS};
use crate::library::generated_contents::heading_shape;
use crate::scripture::language::ReferenceLanguage;
use regex::Regex;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::OnceLock,
};

struct Entry {
    title: String,
    key: String,
    book: Option<&'static str>,
    label: String,
    anchor: Anchor,
}
struct Opening {
    anchor: Anchor,
    key: String,
    book: Option<&'static str>,
}
pub(super) struct Printed {
    pub headings: Vec<Candidate>,
    pub listing_pages: HashSet<u32>,
    pub destinations: Vec<PrintedDestination>,
}
fn key(title: &str) -> String {
    title
        .chars()
        .flat_map(char::to_lowercase)
        .filter(|c| c.is_alphanumeric())
        .collect()
}
fn book(title: &str) -> Option<&'static str> {
    super::resolve::book_title(title, ReferenceLanguage::En).map(|b| b.code)
}
fn matches(entry: &Entry, opening: &Opening) -> bool {
    entry.key == opening.key || entry.book.is_some() && entry.book == opening.book
}

pub(super) fn recover(
    lines: &[(Anchor, String, bool, u32)],
    labels: &BTreeMap<u32, String>,
    headings: &[Heading],
    page_count: u32,
) -> Printed {
    static ENTRY: OnceLock<Regex> = OnceLock::new();
    let pattern = ENTRY.get_or_init(|| {
        Regex::new(r"^(.+?)(?:\s*[.·…](?:[.·…\s])*|\s{2,})([0-9]{1,4}|[ivxlcdmIVXLCDM]{1,8})\s*$")
            .unwrap()
    });
    let mut pages: HashMap<u32, Vec<Entry>> = HashMap::new();
    for (anchor, role, uncertain, _) in lines {
        if *uncertain || matches!(role.as_str(), "header" | "footer" | "note" | "artifact") {
            continue;
        }
        let Some(caps) = pattern.captures(anchor.original_text.trim()) else {
            continue;
        };
        let title = caps[1].trim();
        if title.chars().filter(|c| c.is_alphabetic()).count() < 2 || title.chars().count() > 300 {
            continue;
        }
        pages.entry(anchor.page).or_default().push(Entry {
            title: title.into(),
            key: key(title),
            book: book(title),
            label: caps[2].to_lowercase(),
            anchor: anchor.clone(),
        });
    }
    let listing_pages: HashSet<_> = pages
        .iter()
        .filter(|(_, entries)| entries.iter().map(|e| &e.key).collect::<HashSet<_>>().len() >= 3)
        .map(|(page, _)| *page)
        .collect();
    let mut entries: Vec<_> = pages
        .iter()
        .filter(|(page, _)| listing_pages.contains(page))
        .flat_map(|(_, entries)| entries)
        .collect();
    entries.sort_by_key(|e| (e.anchor.page, e.anchor.byte_start));
    if entries.is_empty() {
        return Printed {
            headings: Vec::new(),
            listing_pages,
            destinations: Vec::new(),
        };
    }
    let entry_anchors: HashSet<_> = entries.iter().map(|e| e.anchor.id()).collect();

    // The ordinary typography fallback is not a prerequisite. A printed TOC
    // can corroborate a modest real heading even when cover typography or a
    // noisy document prevented a whole-document generated-contents result.
    let body = super::body_size(lines);
    let mut openings = Vec::new();
    let mut selected = HashSet::new();
    for h in headings {
        if !entry_anchors.contains(&h.anchor.id()) && selected.insert(h.anchor.id()) {
            openings.push(Opening {
                key: key(&h.title),
                book: book(&h.title),
                anchor: h.anchor.clone(),
            });
        }
    }
    let mut previous: HashMap<(u32, u8), f64> = HashMap::new();
    let mut first: HashMap<u32, &Anchor> = HashMap::new();
    for (a, role, uncertain, _) in lines {
        if *uncertain || role != "paragraph" || entry_anchors.contains(&a.id()) {
            continue;
        }
        if a.original_text
            .chars()
            .filter(|c| c.is_alphabetic())
            .count()
            < 2
        {
            continue;
        }
        first.entry(a.page).or_insert(a);
        let gap = previous
            .insert((a.page, a.column), a.baseline)
            .map_or(99.0, |y| (a.baseline - y) / a.size.max(1.0));
        let shape = heading_shape(&a.original_text);
        if body > 0.0
            && a.size >= body * 1.1
            && gap >= 1.5
            && shape.chars().count() <= 160
            && shape.split_whitespace().count() <= 20
            && !a.original_text.ends_with([',', ';', '-', '‐'])
            && selected.insert(a.id())
        {
            openings.push(Opening {
                key: key(shape),
                book: book(shape),
                anchor: a.clone(),
            });
        }
    }

    // Infer a printed/physical offset only from at least three distinct
    // contents titles whose real headings each occur on one physical page.
    // A duplicate opening or conflicting offset cannot silently choose one.
    let mut offsets: HashMap<i64, HashSet<&str>> = HashMap::new();
    for entry in &entries {
        let Ok(printed) = entry.label.parse::<i64>() else {
            continue;
        };
        let found: HashSet<_> = openings
            .iter()
            .filter(|o| matches(entry, o))
            .map(|o| o.anchor.page)
            .collect();
        if found.len() == 1 {
            let page = found.into_iter().next().unwrap();
            offsets
                .entry(i64::from(page) - printed)
                .or_default()
                .insert(&entry.key);
        }
    }
    let offsets: Vec<_> = offsets
        .into_iter()
        .filter(|(_, titles)| titles.len() >= 3)
        .map(|(offset, _)| offset)
        .collect();
    // Do not reinterpret a book's outline, cross-reference table or index of
    // verse numbers as a page TOC merely because its rows end in numbers.
    // A contents heading (including continued pages) or three independently
    // matched destinations is required on each listing page.
    let named: HashSet<_> = lines
        .iter()
        .filter(|(a, _, _, _)| {
            let title: String = key(&a.original_text)
                .chars()
                .filter(|c| !c.is_ascii_digit())
                .collect();
            a.top_fraction() < 0.25
                && [
                    "contents",
                    "tableofcontents",
                    "inhalt",
                    "inhaltsverzeichnis",
                    "sommaire",
                    "tabledesmatieres",
                    "tabledesmatières",
                    "índice",
                    "indice",
                    "sumário",
                    "sumario",
                    "목차",
                ]
                .contains(&title.as_str())
        })
        .map(|(a, _, _, _)| a.page)
        .collect();
    let listing_pages: HashSet<_> = listing_pages
        .into_iter()
        .filter(|page| {
            named.contains(page)
                || pages[page]
                    .iter()
                    .filter(|entry| {
                        let Ok(label) = entry.label.parse::<i64>() else {
                            return false;
                        };
                        openings.iter().any(|o| {
                            o.anchor.page != *page
                                && matches(entry, o)
                                && offsets.contains(&(i64::from(o.anchor.page) - label))
                        })
                    })
                    .count()
                    >= 3
        })
        .collect();
    let mut label_counts = HashMap::<&str, usize>::new();
    let mut left = HashMap::<(u32, u8), f64>::new();
    for entry in &entries {
        *label_counts.entry(&entry.label).or_default() += 1;
        left.entry((entry.anchor.page, entry.anchor.column))
            .and_modify(|x| *x = x.min(entry.anchor.x0))
            .or_insert(entry.anchor.x0);
    }
    let mut recovered = Vec::new();
    let mut destinations = Vec::new();
    for entry in entries.into_iter().take(MAX_HEADINGS) {
        if !listing_pages.contains(&entry.anchor.page) {
            continue;
        }
        let labelled: Vec<_> = labels
            .iter()
            .filter(|(_, value)| value.to_lowercase() == entry.label)
            .map(|(page, _)| *page)
            .collect();
        let mut targets = if labelled.len() == 1 {
            labelled
        } else if labelled.is_empty() {
            entry
                .label
                .parse::<i64>()
                .ok()
                .into_iter()
                .flat_map(|p| {
                    offsets
                        .iter()
                        .filter_map(move |o| u32::try_from(p + o).ok())
                })
                .filter(|p| *p != entry.anchor.page && (1..=page_count).contains(p))
                .collect()
        } else {
            Vec::new()
        };
        if targets.len() > 1 {
            targets.retain(|p| {
                openings
                    .iter()
                    .any(|o| o.anchor.page == *p && matches(entry, o))
            });
        }
        let [page] = targets.as_slice() else {
            continue;
        };
        let exact: Vec<_> = openings
            .iter()
            .filter(|o| o.anchor.page == *page && matches(entry, o))
            .collect();
        let anchor = if exact.len() == 1 {
            Some(&exact[0].anchor)
        } else if exact.is_empty() && label_counts[entry.label.as_str()] == 1 {
            // A TOC title may differ from its actual opening (including an
            // original typo). With a corroborated destination, use only the
            // unique prominent opening at the beginning of that physical
            // page. Never search a neighbouring page or borrow body prose.
            first.get(page).copied().filter(|a| {
                a.top_fraction() < 0.35
                    && body > 0.0
                    && a.size >= body * 1.2
                    && openings.iter().any(|o| o.anchor.id() == a.id())
                    && openings
                        .iter()
                        .filter(|o| {
                            o.anchor.page == *page
                                && (o.anchor.baseline - a.baseline).abs() <= a.size * 0.5
                        })
                        .count()
                        == 1
            })
        } else {
            None
        };
        let indent = entry.anchor.x0 - left[&(entry.anchor.page, entry.anchor.column)];
        let level =
            (1.0 + (indent / (entry.anchor.size.max(1.0) * 1.5)).floor()).clamp(1.0, 3.0) as u8;
        destinations.push(PrintedDestination {
            id: format!("printed-{}", entry.anchor.id()),
            title: entry.title.clone(),
            level,
            page: *page,
            anchor_id: anchor.map(Anchor::id),
        });
        let Some(anchor) = anchor else { continue };
        recovered.push(Candidate {
            title: entry.title.clone(),
            level,
            parents: Vec::new(),
            basis: "printed".into(),
            anchor: anchor.clone(),
        });
    }
    recovered.sort_by(|a, b| {
        a.anchor
            .page
            .cmp(&b.anchor.page)
            .then(a.anchor.first_char.cmp(&b.anchor.first_char))
    });
    Printed {
        headings: recovered,
        listing_pages,
        destinations,
    }
}
