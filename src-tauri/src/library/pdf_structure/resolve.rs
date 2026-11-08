//! Heading context only. Never infer a book/chapter from body citations.
use super::{Correction, Heading, PdfStructure, Presentation, ResolvedHeading};
use crate::scripture::{
    canon::Book,
    language::{fold, ReferenceLanguage},
    parser::ReferenceParser,
    recognise::Recogniser,
    reference::{Point, ScriptureRef},
};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;
use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};

fn key(text: &str) -> String {
    fold(text)
        .nfd()
        .filter(|c| !is_combining_mark(*c))
        .map(|c| if c == 'ς' { 'σ' } else { c })
        .filter(|c| c.is_alphanumeric())
        .collect()
}
fn core_title(text: &str) -> &str {
    for separator in [" — ", " – ", " - ", ": "] {
        if let Some((start, _)) = text.split_once(separator) {
            return start.trim();
        }
    }
    text.trim()
}
pub(super) fn book_title(title: &str, language: ReferenceLanguage) -> Option<&'static Book> {
    let title = core_title(title);
    let title = title
        .split_once(' ')
        .filter(|(first, _)| {
            [
                "the", "die", "das", "der", "le", "les", "la", "el", "los", "las", "o", "a", "os",
                "as",
            ]
            .contains(&first.to_lowercase().as_str())
        })
        .map(|(_, rest)| rest)
        .unwrap_or(title);
    let wanted = key(title);
    // These labels depend on canon/naming tradition. Keep their original
    // headings/Contents, but never force them into the 66-book registry.
    if [
        "jud",
        "1esdras",
        "2esdras",
        "3esdras",
        "4esdras",
        "1kingdoms",
        "2kingdoms",
        "3kingdoms",
        "4kingdoms",
    ]
    .contains(&wanted.as_str())
    {
        return None;
    }
    // Source-only, exact full Greek NT book titles. No Scripture text is
    // imported or verified, and no recogniser pipeline is changed.
    let greek = match wanted.as_str() {
        "καταματθαιον" => Some("MAT"),
        "καταμαρκον" => Some("MRK"),
        "καταλουκαν" => Some("LUK"),
        "καταιωαννην" => Some("JHN"),
        "πραξεισαποστολων" | "πραξειστωναποστολων" => {
            Some("ACT")
        }
        "προσρωμαιουσ" => Some("ROM"),
        "προσκορινθιουσα" => Some("1CO"),
        "προσκορινθιουσβ" => Some("2CO"),
        "προσγαλατασ" => Some("GAL"),
        "προσεφεσιουσ" => Some("EPH"),
        "προσφιλιππησιουσ" => Some("PHP"),
        "προσκολοσσαεισ" => Some("COL"),
        "προσθεσσαλονικεισα" => Some("1TH"),
        "προσθεσσαλονικεισβ" => Some("2TH"),
        "προστιμοθεονα" => Some("1TI"),
        "προστιμοθεονβ" => Some("2TI"),
        "προστιτον" => Some("TIT"),
        "προσφιλημονα" => Some("PHM"),
        "προσεβραιουσ" => Some("HEB"),
        "ιακωβουεπιστολη" => Some("JAS"),
        "πετρουα" => Some("1PE"),
        "πετρουβ" => Some("2PE"),
        "ιωαννουα" => Some("1JN"),
        "ιωαννουβ" => Some("2JN"),
        "ιωαννουγ" => Some("3JN"),
        "ιουδαεπιστολη" => Some("JUD"),
        "αποκαλυψισιωαννου" => Some("REV"),
        _ => None,
    };
    if let Some(code) = greek {
        return crate::scripture::canon::BOOKS
            .iter()
            .find(|b| b.code == code);
    }

    if let Some((_, book)) = ReferenceParser::for_language(language)
        .lookup_entries()
        .find(|(alias, _)| key(alias) == wanted)
    {
        return Some(book);
    }
    // A short foreign alias is accepted only when all supported dictionaries
    // agree on one book. E.g. German "Apg" in a PDF without language metadata.
    if wanted.chars().count() < 3 {
        return None;
    }
    let mut found: Option<&'static Book> = None;
    for lang in [
        ReferenceLanguage::En,
        ReferenceLanguage::De,
        ReferenceLanguage::Ko,
        ReferenceLanguage::Fr,
        ReferenceLanguage::Es,
        ReferenceLanguage::Pt,
    ] {
        for (_, book) in ReferenceParser::for_language(lang)
            .lookup_entries()
            .filter(|(alias, _)| key(alias) == wanted)
        {
            if found.is_some_and(|old| old.code != book.code) {
                return None;
            }
            found = Some(book);
        }
    }
    found
}
fn canonical(
    title: &str,
    language: ReferenceLanguage,
    parent: Option<(&ScriptureRef, &str)>,
) -> Option<(ScriptureRef, String)> {
    let title_key = key(title);
    if [
        "jud",
        "1esdras",
        "2esdras",
        "3esdras",
        "4esdras",
        "1kingdoms",
        "2kingdoms",
        "3kingdoms",
        "4kingdoms",
    ]
    .contains(&title_key.as_str())
        || ((title_key.contains("esther") || title_key.contains("daniel"))
            && ["greek", "griech", "lxx", "zusatz", "addition"]
                .iter()
                .any(|v| title_key.contains(v)))
    {
        return None;
    }
    if let Some(book) = book_title(title, language) {
        return ScriptureRef::new(
            book,
            Point {
                chapter: 1,
                verse: None,
            },
            Point {
                chapter: book.chapter_count,
                verse: None,
            },
        )
        .ok()
        .map(|r| (r, "book".into()));
    }
    let found = Recogniser::for_source(language).detect(title);
    if found.len() == 1 {
        let r = ScriptureRef::from_canonical(&found[0].reference.canonical).ok()?;
        return Some((
            r,
            if r.start.verse.is_none() && r.end.verse.is_none() && r.start.chapter == r.end.chapter
            {
                "chapter"
            } else {
                "passage"
            }
            .into(),
        ));
    }
    let (parent, scope) = parent?;
    let mut value = core_title(title).trim().to_lowercase();
    let mut explicit_chapter = false;
    for prefix in [
        "chapter ",
        "kapitel ",
        "chapitre ",
        "capítulo ",
        "capitulo ",
    ] {
        if let Some(rest) = value.strip_prefix(prefix) {
            value = rest.trim().into();
            explicit_chapter = true;
            break;
        }
    }
    if value.starts_with('제') {
        value = value.trim_start_matches('제').trim().into();
        explicit_chapter = true;
    }
    if value.contains('장') {
        explicit_chapter = true;
        value = value.replace('장', ":").replace('절', "");
        value = value.trim_end_matches(':').trim().into();
    }
    static NUMERIC: OnceLock<Regex> = OnceLock::new();
    let re=NUMERIC.get_or_init(||Regex::new(r"^([0-9]{1,3})(?:\s*[:,]\s*([0-9]{1,3}))?(?:\s*[-–—]\s*(?:([0-9]{1,3})\s*[:,]\s*)?([0-9]{1,3}))?$").unwrap());
    let caps = re.captures(&value)?;
    let ch = caps[1].parse().ok()?;
    let verse = caps.get(2).and_then(|v| v.as_str().parse().ok());
    if verse.is_none() && scope != "book" && !explicit_chapter {
        return None;
    }
    let last = caps.get(4).and_then(|v| v.as_str().parse::<u16>().ok());
    let end = if verse.is_some() {
        Point {
            chapter: caps
                .get(3)
                .and_then(|v| v.as_str().parse().ok())
                .unwrap_or(ch),
            verse: last.or(verse),
        }
    } else {
        Point {
            chapter: last.unwrap_or(ch),
            verse: None,
        }
    };
    ScriptureRef::new(parent.book, Point { chapter: ch, verse }, end)
        .ok()
        .map(|r| {
            (
                r,
                if verse.is_none() && ch == end.chapter {
                    "chapter"
                } else {
                    "passage"
                }
                .into(),
            )
        })
}

/// Headings that name this many different biblical books on one physical page
/// are a listing (a Contents page, a book index), not book introductions.
pub const LISTING_BOOKS: usize = 3;

/// Apply reversible app-owned corrections. Existing text/anchors never change.
///
/// A.4.1: inferred headings on a page listing several biblical books stay in
/// the hierarchy and Contents, but give no passage mapping (scope `contents`),
/// so a front-matter Contents line can never stand in for the book's own
/// section. A user mapping is never overridden.
pub fn effective(
    doc: &PdfStructure,
    corrections: &[Correction],
    language: ReferenceLanguage,
) -> Vec<ResolvedHeading> {
    let first = resolve(doc, corrections, language, &HashSet::new());
    let mut books: HashMap<u32, HashSet<&str>> = HashMap::new();
    for h in first.iter().filter(|h| h.heading.basis != "user") {
        if let Some(code) = h.canonical.as_deref().and_then(|c| c.split('.').next()) {
            books.entry(h.heading.anchor.page).or_default().insert(code);
        }
    }
    let listed: HashSet<String> = first
        .iter()
        .filter(|h| {
            h.heading.basis != "user"
                && h.canonical.is_some()
                && books
                    .get(&h.heading.anchor.page)
                    .is_some_and(|b| b.len() >= LISTING_BOOKS)
        })
        .map(|h| h.heading.id.clone())
        .collect();
    if listed.is_empty() {
        return first;
    }
    resolve(doc, corrections, language, &listed)
}

fn resolve(
    doc: &PdfStructure,
    corrections: &[Correction],
    language: ReferenceLanguage,
    listed: &HashSet<String>,
) -> Vec<ResolvedHeading> {
    let suppressed: std::collections::HashSet<_> = corrections
        .iter()
        .filter_map(|p| doc.headings.iter().find(|h| h.id == p.id))
        .map(|h| h.title.as_str())
        .collect();
    let mut headings = doc.headings.clone();
    for patch in corrections {
        if let Some(h) = headings.iter_mut().find(|h| h.id == patch.id) {
            h.title = patch.title.clone();
            h.level = patch.level;
            h.mapping = patch.mapping.clone();
            h.mapping_scope = patch.scope.clone();
            h.disabled = patch.disabled;
            h.basis = "user".into();
        } else if let Some(block) = doc.blocks.iter().find(|b| b.position == patch.position) {
            if let Some(anchor) = block.anchors.iter().find(|a| a.id() == patch.anchor_id) {
                headings.push(Heading {
                    id: patch.id.clone(),
                    title: patch.title.clone(),
                    level: patch.level,
                    parents: Vec::new(),
                    basis: "user".into(),
                    anchor: anchor.clone(),
                    segment_position: patch.position,
                    start: patch.start,
                    end: patch.end,
                    disabled: patch.disabled,
                    mapping: patch.mapping.clone(),
                    mapping_scope: patch.scope.clone(),
                });
            }
        }
    }
    headings.sort_by(|a, b| {
        a.segment_position
            .cmp(&b.segment_position)
            .then(a.start.cmp(&b.start))
            .then(a.id.cmp(&b.id))
    });
    let mut out: Vec<ResolvedHeading> = Vec::new();
    let mut open: Vec<usize> = Vec::new();
    for heading in headings.into_iter().filter(|h| !h.disabled) {
        while open.last().is_some_and(|i| {
            let old = &out[*i].heading;
            let original = doc
                .headings
                .iter()
                .find(|h| h.id == old.id)
                .map(|h| h.title.as_str())
                .unwrap_or(&old.title);
            old.level >= heading.level
                || (!heading.parents.is_empty() && !heading.parents.iter().any(|p| p == original))
        }) {
            let i = open.pop().unwrap();
            out[i].end_position = heading.segment_position;
        }
        // Supplied outline grouping titles can carry book context even when
        // the grouping node has no destination of its own.
        let supplied = heading
            .parents
            .iter()
            .rev()
            .filter(|p| !suppressed.contains(p.as_str()))
            .find_map(|p| canonical(p, language, None));
        let inherited = open.iter().rev().find_map(|i| {
            out[*i]
                .canonical
                .as_ref()
                .and_then(|s| ScriptureRef::from_canonical(s).ok())
                .map(|r| (r, out[*i].scope.clone()))
        });
        let parent = inherited.or(supplied);
        let resolved = heading
            .mapping
            .as_ref()
            .and_then(|m| ScriptureRef::from_canonical(m).ok())
            .map(|r| {
                let scope = heading.mapping_scope.clone().unwrap_or_else(|| {
                    if r.start.verse.is_none()
                        && r.end.verse.is_none()
                        && r.start.chapter == r.end.chapter
                    {
                        "chapter"
                    } else {
                        "passage"
                    }
                    .into()
                });
                (r, scope)
            })
            .or_else(|| {
                if heading.basis == "user" {
                    return None;
                }
                canonical(
                    &heading.title,
                    language,
                    parent.as_ref().map(|(r, s)| (r, s.as_str())),
                )
            });
        let listing = listed.contains(&heading.id);
        let (reference, scope) = match resolved {
            Some(_) if listing => (None, "contents".into()),
            Some((r, s)) => (Some(r.canonical()), s),
            None => (None, "unmapped".into()),
        };
        let mut path = if !heading.parents.is_empty() {
            heading
                .parents
                .iter()
                .map(|name| {
                    open.iter()
                        .find_map(|i| {
                            let old = &out[*i].heading;
                            let original = doc
                                .headings
                                .iter()
                                .find(|h| h.id == old.id)
                                .map(|h| h.title.as_str())
                                .unwrap_or(&old.title);
                            (original == name).then(|| old.title.clone())
                        })
                        .unwrap_or_else(|| name.clone())
                })
                .collect::<Vec<_>>()
        } else {
            open.iter().map(|i| out[*i].heading.title.clone()).collect()
        };
        path.push(heading.title.clone());
        out.push(ResolvedHeading {
            heading,
            path: path.join(" › "),
            canonical: reference,
            scope,
            end_position: i64::MAX,
        });
        open.push(out.len() - 1);
    }
    out
}

pub fn presentations(
    doc: &PdfStructure,
    headings: &[ResolvedHeading],
    positions: &std::collections::HashSet<i64>,
) -> HashMap<i64, Presentation> {
    doc.blocks
        .iter()
        .filter(|block| positions.contains(&block.position))
        .map(|block| {
            let local: Vec<_> = headings
                .iter()
                .filter(|h| h.heading.segment_position == block.position)
                .map(|h| h.heading.clone())
                .collect();
            let path = headings
                .iter()
                .rev()
                .find(|h| {
                    h.heading.segment_position <= block.position && block.position < h.end_position
                })
                .map(|h| h.path.clone());
            (
                block.position,
                Presentation {
                    role: block.role.clone(),
                    runs: block.runs.clone(),
                    basis: block.basis.clone(),
                    uncertain: block.uncertain,
                    printed_label: block.printed_label.clone(),
                    anchors: block.anchors.clone(),
                    headings: local,
                    section_path: if matches!(
                        block.role.as_str(),
                        "note" | "header" | "footer" | "artifact" | "uncertain"
                    ) {
                        None
                    } else {
                        path
                    },
                },
            )
        })
        .collect()
}
