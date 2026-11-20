//! Heading context only. Never infer a book/chapter from body citations.
use super::{Correction, Heading, PdfStructure, Presentation, ResolvedHeading};
use crate::scripture::{
    canon::Book,
    language::{fold, ReferenceLanguage},
    parser::Prepared,
    recognise::Recogniser,
    reference::{Point, ScriptureRef},
};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;
use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};

#[path = "heading_books.rs"]
mod heading_books;

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
        for (at, _) in text.match_indices(separator) {
            let (start, rest) = (&text[..at], &text[at + separator.len()..]);
            // A spaced verse separator or range belongs to the reference,
            // not to an explanatory subtitle (`John 3: 16 – 18 — Love`).
            if start.chars().next_back().is_some_and(char::is_numeric)
                && rest
                    .trim_start()
                    .chars()
                    .next()
                    .is_some_and(char::is_numeric)
            {
                continue;
            }
            return start.trim();
        }
    }
    text.trim()
}

/// These headings describe lists of references, not the source's exposition.
/// Their entries remain visible in Contents and as literal references.
pub(crate) fn is_reference_list(title: &str) -> bool {
    let title = fold(title);
    [
        "contents",
        "table of contents",
        "outline",
        "cross-references",
        "cross references",
        "scripture references",
        "bible references",
        "parallel passages",
        "see also",
        "inhaltsverzeichnis",
        "inhalt",
        "gliederung",
        "bibelstellenregister",
        "bibelstellenverzeichnis",
        "stellenregister",
        "querverweise",
        "parallelstellen",
        "siehe auch",
        "table des matières",
        "plan du livre",
        "références croisées",
        "referencias cruzadas",
        "referências cruzadas",
        "índice de referencias",
        "목차",
        "개요",
        "교차 참조",
    ]
    .iter()
    .any(|label| {
        title == *label
            || title.strip_prefix(label).is_some_and(|rest| {
                rest.starts_with(':') || rest.starts_with(" — ") || rest.starts_with(" – ")
            })
    })
}
pub(super) fn book_title(title: &str, language: ReferenceLanguage) -> Option<&'static Book> {
    book_label(title, language).or_else(|| book_label(core_title(title), language))
}

/// The entire label, with no subtitle/prose truncation. Parenthetical names
/// are synonyms only when both independently resolve to the same book.
fn book_label(title: &str, language: ReferenceLanguage) -> Option<&'static Book> {
    let title = title.trim();
    if title
        .chars()
        .take(crate::scripture::resolve::MAX_TYPED + 1)
        .count()
        > crate::scripture::resolve::MAX_TYPED
    {
        return None;
    }
    if let Some((left, right)) = title.strip_suffix(')').and_then(|s| s.split_once(" (")) {
        let a = book_label(left, language)?;
        let b = book_label(right, language)?;
        return (a.code == b.code).then_some(a);
    }
    if let Some((left, right)) = title.split_once(" / ") {
        let a = book_label(left, language)?;
        let b = book_label(right, language)?;
        return (a.code == b.code).then_some(a);
    }
    let original_title = title;
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

    heading_books::lookup(original_title, language)
        .or_else(|| heading_books::lookup(title, language))
}

fn whole_book(book: &'static Book) -> Option<ScriptureRef> {
    ScriptureRef::new(
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
}

/// Strict source-only reference input: the complete value must be a book
/// label, or a known book label followed by a complete chapter/verse suffix.
/// Used by real source headings and manual whole-source coverage, without
/// relaxing the shared running-text or Scripture-import recognizers.
pub(crate) fn named_reference(
    title: &str,
    language: ReferenceLanguage,
) -> Option<(ScriptureRef, String)> {
    if title
        .chars()
        .take(crate::scripture::resolve::MAX_TYPED + 1)
        .count()
        > crate::scripture::resolve::MAX_TYPED
    {
        return None;
    }
    if let Some(book) = book_label(title, language) {
        return whole_book(book).map(|r| (r, "book".into()));
    }
    // Prefix lookup uses the cached lexicon; no language regex is compiled
    // per heading. Try the longest label first (including numbered books).
    for (at, _) in title.char_indices().rev() {
        if at == 0 {
            break;
        }
        let Some(book) = book_label(title[..at].trim_end(), language) else {
            continue;
        };
        let reference = whole_book(book)?;
        if let Some(found) = relative_reference(title[at..].trim(), &reference, "book", true) {
            return Some(found);
        }
    }
    None
}

/// An explicitly combined book opening is one real destination for each named
/// book. The shared title never chooses one book as implicit chapter context.
pub(crate) fn book_references(title: &str, language: ReferenceLanguage) -> Vec<ScriptureRef> {
    let mut books = Vec::new();
    if let Some(book) = book_title(title, language) {
        books.push(book);
    } else {
        let title = core_title(title);
        static COMBINED: OnceLock<Regex> = OnceLock::new();
        let combined = COMBINED.get_or_init(|| Regex::new(r"(?i)^([1-3])\.?\s*(?:\+|&|/|,|and\b|und\b|et\b|y\b|e\b)\s*([1-3])\.?\s*(\p{L}.*)$").unwrap());
        if let Some(parts) = combined.captures(title) {
            for number in [&parts[1], &parts[2]] {
                let Some(book) = book_title(&format!("{number} {}", &parts[3]), language) else {
                    return Vec::new();
                };
                if !books.iter().any(|b: &&Book| b.code == book.code) {
                    books.push(book);
                }
            }
        } else {
            for separator in [" + ", " & ", " / ", " and ", " und ", " et ", " y ", " e "] {
                let parts: Vec<_> = title.split(separator).collect();
                if parts.len() < 2 {
                    continue;
                }
                let found: Option<Vec<_>> = parts
                    .into_iter()
                    .map(|part| book_title(part.trim(), language))
                    .collect();
                if let Some(found) = found {
                    books = found;
                    break;
                }
            }
        }
        if books.len() < 2 {
            return Vec::new();
        }
    }
    books
        .into_iter()
        .filter_map(|book| {
            ScriptureRef::new(
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
        })
        .collect()
}
pub(crate) fn canonical(
    title: &str,
    language: ReferenceLanguage,
    parent: Option<(&ScriptureRef, &str)>,
) -> Option<(ScriptureRef, String)> {
    if is_reference_list(title) {
        return None;
    }
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
    if let Some(found) =
        named_reference(title, language).or_else(|| named_reference(core_title(title), language))
    {
        return Some(found);
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
    relative_reference(core_title(title), parent, scope, false)
}

fn relative_reference(
    title: &str,
    parent: &ScriptureRef,
    scope: &str,
    named: bool,
) -> Option<(ScriptureRef, String)> {
    let mut value = Prepared::new(title).text.trim().to_lowercase();
    let mut explicit_chapter = false;
    let mut explicit_verse = false;
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
    if value.starts_with(['제', '第']) {
        value = value.trim_start_matches(['제', '第']).trim().into();
        explicit_chapter = true;
    }
    let mut chapter_unit = false;
    let mut prefix_unit = false;
    for unit in heading_books::chapter_units() {
        if value.contains(unit) {
            prefix_unit |= value.trim_start().starts_with(unit);
            value = value.replace(unit, ":");
            explicit_chapter = true;
            chapter_unit = true;
        }
    }
    for unit in heading_books::verse_units() {
        if value.contains(unit) {
            // A unit separates numeric tokens. Removing it without a boundary
            // would turn malformed `1절2절` into the invented verse `12`.
            value = value.replace(unit, " ");
            explicit_verse = !explicit_chapter;
        }
    }
    if chapter_unit {
        // Repeated units belong to a complete range: `1장1절–2장2절`,
        // `1章–2章`. Preserve its endpoints rather than clipping the first.
        static UNIT_RANGE: OnceLock<Regex> = OnceLock::new();
        let re = UNIT_RANGE.get_or_init(|| Regex::new(r":\s*([-‐‑‒–—~])").unwrap());
        value = re.replace_all(&value, "$1").trim().into();
        if prefix_unit {
            value = value.strip_prefix(':').unwrap_or(&value).trim().into();
        }
        value = value.strip_suffix(':').unwrap_or(&value).trim().into();
    }
    for prefix in [
        "verses ",
        "verse ",
        "vers ",
        "verse: ",
        "verset ",
        "versets ",
        "versículo ",
        "versículos ",
        "versiculo ",
        "versiculos ",
    ] {
        if let Some(rest) = value.strip_prefix(prefix) {
            value = rest.trim().into();
            explicit_verse = true;
            break;
        }
    }
    static NUMERIC: OnceLock<Regex> = OnceLock::new();
    let re=NUMERIC.get_or_init(||Regex::new(r"^([0-9]{1,3})(?:\s*[:,]\s*([0-9]{1,3}))?(?:\s*[-‐‑‒–—~]\s*(?:([0-9]{1,3})\s*[:,]\s*)?([0-9]{1,3}))?$").unwrap());
    let caps = re.captures(value.trim())?;
    let ch = caps[1].parse().ok()?;
    let verse = caps.get(2).and_then(|v| v.as_str().parse().ok());
    let last = caps.get(4).and_then(|v| v.as_str().parse::<u16>().ok());
    // Explicit references in one-chapter books use verse shorthand, just as
    // the shared parser does (`Jude 5`, `Philemon 1–4`). A separate child
    // heading `1` under a book remains a structural chapter, and an explicit
    // chapter word/unit always retains its chapter meaning.
    if named && parent.book.chapter_count == 1 && verse.is_none() && !explicit_chapter {
        return ScriptureRef::new(
            parent.book,
            Point {
                chapter: 1,
                verse: Some(ch),
            },
            Point {
                chapter: 1,
                verse: Some(last.unwrap_or(ch)),
            },
        )
        .ok()
        .map(|r| (r, "passage".into()));
    }
    if verse.is_none() && !explicit_chapter && (explicit_verse || scope == "chapter") {
        if scope != "chapter" || parent.start.chapter != parent.end.chapter {
            return None;
        }
        return ScriptureRef::new(
            parent.book,
            Point {
                chapter: parent.start.chapter,
                verse: Some(ch),
            },
            Point {
                chapter: parent.start.chapter,
                verse: Some(last.unwrap_or(ch)),
            },
        )
        .ok()
        .map(|reference| (reference, "passage".into()));
    }
    if verse.is_none() && scope != "book" && !explicit_chapter {
        return None;
    }
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
        for canonical in h.canonical.iter().chain(&h.additional_canonicals) {
            if let Some(code) = canonical.split('.').next() {
                books.entry(h.heading.anchor.page).or_default().insert(code);
            }
        }
    }
    let listed: HashSet<String> = first
        .iter()
        .filter(|h| {
            h.heading.basis != "user"
                && (h.canonical.is_some() || !h.additional_canonicals.is_empty())
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
    let printed_boundary = |heading: &Heading| {
        doc.printed_contents
            .iter()
            .filter(|destination| {
                destination.page > heading.anchor.page && destination.level <= heading.level
            })
            .filter_map(|destination| {
                doc.blocks
                    .iter()
                    .find(|block| block.page >= destination.page)
                    .map(|block| block.position)
            })
            .min()
    };
    for heading in headings.into_iter().filter(|h| !h.disabled) {
        // An unanchored printed destination still closes the preceding book.
        // Its later headings must not inherit a book merely because the new
        // opening could not be aligned to a stored paragraph.
        while open.last().is_some_and(|i| {
            printed_boundary(&out[*i].heading).is_some_and(|end| end <= heading.segment_position)
        }) {
            let i = open.pop().unwrap();
            out[i].end_position = printed_boundary(&out[i].heading).unwrap();
        }
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
            .filter(|p| !suppressed.contains(p.as_str()))
            .fold(None, |parent: Option<(ScriptureRef, String)>, title| {
                if book_references(title, language).len() > 1 {
                    return None;
                }
                canonical(
                    title,
                    language,
                    parent.as_ref().map(|(r, scope)| (r, scope.as_str())),
                )
                .or(parent)
            });
        let inherited = open
            .iter()
            .rev()
            .find(|i| out[**i].canonical.is_some() || !out[**i].additional_canonicals.is_empty())
            .and_then(|i| {
                out[*i]
                    .canonical
                    .as_ref()
                    .and_then(|s| ScriptureRef::from_canonical(s).ok())
                    .map(|r| (r, out[*i].scope.clone()))
            });
        let parent = inherited.or(supplied);
        let reference_list = heading.parents.iter().any(|p| is_reference_list(p))
            || open
                .iter()
                .any(|i| is_reference_list(&out[*i].heading.title));
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
                if heading.basis == "user" || reference_list {
                    return None;
                }
                canonical(
                    &heading.title,
                    language,
                    parent.as_ref().map(|(r, s)| (r, s.as_str())),
                )
            })
            .filter(|(reference, _)| {
                heading.basis == "user"
                    || parent.as_ref().is_none_or(|(parent, _)| {
                        reference.start_key() >= parent.start_key()
                            && reference.end_key() <= parent.end_key()
                    })
            });
        let listing = listed.contains(&heading.id);
        let additional_canonicals: Vec<_> =
            if !listing && !reference_list && heading.basis != "user" && resolved.is_none() {
                let references = book_references(&heading.title, language);
                if references.len() > 1
                    && parent.as_ref().is_none_or(|(parent, _)| {
                        references.iter().all(|r| {
                            r.start_key() >= parent.start_key() && r.end_key() <= parent.end_key()
                        })
                    })
                {
                    references.into_iter().map(|r| r.canonical()).collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };
        let (reference, scope) = match resolved {
            Some(_) if listing => (None, "contents".into()),
            Some((r, s)) => (Some(r.canonical()), s),
            None if !additional_canonicals.is_empty() => (None, "book".into()),
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
            additional_canonicals,
            scope,
            end_position: i64::MAX,
        });
        open.push(out.len() - 1);
    }
    for resolved in &mut out {
        if let Some(boundary) = printed_boundary(&resolved.heading) {
            resolved.end_position = resolved.end_position.min(boundary);
        }
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
