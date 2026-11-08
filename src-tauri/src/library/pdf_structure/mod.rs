//! Source-only PDF structure, not Scripture verification. No source byte or
//! stored segment is rewritten. Every proposal keeps its own evidence/anchor.
mod native;
mod printed;
mod resolve;

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use super::adapters::{
    pdf::PdfText,
    pdf_layout::{LineRole, PageOrder},
    NewSegment,
};
use super::generated_contents;

pub const VERSION: i64 = 1;
/// The structure-analysis rules a document was built with. The stored JSON
/// shape stays [`VERSION`] 1 (so A.3 documents still load); a document built
/// before A.4.1 reads as 0 and the diagnostic recommends **Rebuild structure**.
/// 2: margin-only, size-aware running furniture; Contents listings of books.
/// 3: grounded printed book-index targets, ordered evidence levels and safe Source titles.
pub const ANALYSIS: u32 = 3;
pub const MAX_HEADINGS: usize = 5_000;
pub const MAX_LINES: usize = 120_000;
pub const MAX_CHARS: usize = 8_000_000;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Anchor {
    pub page: u32,
    pub page_width: f64,
    pub page_height: f64,
    pub x0: f64,
    pub x1: f64,
    /// Original top-down baseline and type size; not a claimed glyph bbox.
    pub baseline: f64,
    pub size: f64,
    pub first_char: usize,
    pub column: u8,
    pub byte_start: usize,
    pub byte_end: usize,
    pub original_text: String,
}
impl Anchor {
    pub fn top_fraction(&self) -> f64 {
        ((self.baseline - self.size) / self.page_height.max(1.0)).clamp(0.0, 1.0)
    }
    pub fn id(&self) -> String {
        format!("p{}-c{}", self.page, self.first_char)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Heading {
    pub id: String,
    pub title: String,
    pub level: u8,
    #[serde(default)]
    pub parents: Vec<String>,
    /// outline, tagged, layout, user. None of these verifies commentary.
    pub basis: String,
    pub anchor: Anchor,
    pub segment_position: i64,
    pub start: u32,
    pub end: u32,
    #[serde(default)]
    pub disabled: bool,
    /// A user mapping is canonical and never silently inferred from body text.
    #[serde(default)]
    pub mapping: Option<String>,
    #[serde(default)]
    pub mapping_scope: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub position: i64,
    pub page: u32,
    /// paragraph, list, quote, note, header, footer, uncertain.
    pub role: String,
    pub basis: String,
    pub anchors: Vec<Anchor>,
    pub printed_label: Option<String>,
    pub uncertain: bool,
    #[serde(default)]
    pub runs: Vec<TextRole>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextRole {
    pub start: u32,
    pub end: u32,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfStructure {
    pub version: i64,
    #[serde(default)]
    pub analysis: u32,
    pub page_count: u32,
    pub complete: bool,
    pub tagged: bool,
    pub unaligned_tags: usize,
    pub headings: Vec<Heading>,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Correction {
    pub id: String,
    pub position: i64,
    pub title: String,
    pub level: u8,
    pub mapping: Option<String>,
    pub disabled: bool,
    pub anchor_id: String,
    pub start: u32,
    pub end: u32,
    #[serde(default)]
    pub scope: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedHeading {
    #[serde(flatten)]
    pub heading: Heading,
    pub path: String,
    pub canonical: Option<String>,
    /// book, chapter, passage, or unmapped.
    pub scope: String,
    pub end_position: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Presentation {
    pub role: String,
    pub basis: String,
    pub uncertain: bool,
    pub printed_label: Option<String>,
    pub anchors: Vec<Anchor>,
    pub headings: Vec<Heading>,
    pub section_path: Option<String>,
    pub runs: Vec<TextRole>,
}

/// Existing segments only: positions and text are immutable during analysis.
#[derive(Debug, Clone)]
pub struct Piece<'a> {
    pub position: i64,
    pub page: u32,
    pub text: &'a str,
    pub byte_start: Option<u32>,
    pub byte_end: Option<u32>,
}

pub fn pieces(segments: &[NewSegment]) -> Vec<Piece<'_>> {
    segments
        .iter()
        .enumerate()
        .filter_map(|(index, s)| {
            Some(Piece {
                position: index as i64,
                page: s.page_number?,
                text: &s.text,
                byte_start: s.char_start,
                byte_end: s.char_end,
            })
        })
        .collect()
}

pub(crate) fn compact(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// An exact text alignment gives UTF-16 offsets in the unchanged stored text.
fn align(piece: &Piece<'_>, line: &str) -> Option<(u32, u32)> {
    let needle = compact(line);
    if needle.is_empty() {
        return None;
    }
    let mut matches = piece.text.match_indices(&needle);
    let (start, _) = matches.next()?;
    if matches.next().is_some() {
        return None;
    }
    Some((
        piece.text[..start].encode_utf16().count() as u32,
        piece.text[..start + needle.len()].encode_utf16().count() as u32,
    ))
}

fn in_margin(anchor: &Anchor) -> bool {
    anchor.top_fraction() < 0.10 || anchor.top_fraction() > 0.90
}

/// Running heads/feet ignore their numbers: "Apostelgeschichte 6" and
/// "Apostelgeschichte 7" are the same furniture.
fn furniture_key(text: &str) -> Option<String> {
    let key: String = compact(
        &text
            .chars()
            .filter(|c| !c.is_ascii_digit())
            .collect::<String>(),
    )
    .to_lowercase();
    (key.chars().filter(|c| c.is_alphabetic()).count() >= 2).then_some(key)
}

/// The size most characters are set in.
fn body_size(all: &[(Anchor, String, bool, u32)]) -> f64 {
    let mut weight: HashMap<i64, usize> = HashMap::new();
    for (a, _, _, _) in all {
        *weight.entry((a.size * 2.0).round() as i64).or_default() += a.original_text.len();
    }
    weight
        .into_iter()
        .max_by_key(|(b, w)| (*w, -*b))
        .map_or(0.0, |(b, _)| b as f64 / 2.0)
}

/// Running furniture: text in the top or bottom margin of three or more pages,
/// set in its usual size.
///
/// A.4.1: only *margin* occurrences are furniture. A.3 reclassified every line
/// with that text anywhere, so a handbook whose running head is the book name
/// ("Apostelgeschichte") lost the book's own opening title, and Source results
/// fell back to the earliest literal mention in the front matter. A margin line
/// set clearly larger than that furniture (a chapter-opening title at the top
/// of its page) is kept as text too. Numbers are ignored ("Apostelgeschichte 6"
/// / "… 7") only for furniture set no larger than the body text, so headings
/// such as "Kapitel 6" / "Kapitel 7" near the top of their pages survive.
fn classify_furniture(all: &mut [(Anchor, String, bool, u32)]) {
    let body = body_size(all);
    let mut exact: HashMap<String, (HashSet<u32>, Vec<f64>)> = HashMap::new();
    let mut loose: HashMap<String, (HashSet<u32>, Vec<f64>)> = HashMap::new();
    for (a, _, _, _) in all.iter() {
        if in_margin(a) {
            let entry = exact.entry(compact(&a.original_text)).or_default();
            entry.0.insert(a.page);
            entry.1.push(a.size);
            if let Some(key) = furniture_key(&a.original_text) {
                let entry = loose.entry(key).or_default();
                entry.0.insert(a.page);
                entry.1.push(a.size);
            }
        }
    }
    let usual = |map: HashMap<String, (HashSet<u32>, Vec<f64>)>| -> HashMap<String, f64> {
        map.into_iter()
            .filter(|(_, (pages, _))| pages.len() >= 3)
            .map(|(key, (_, mut sizes))| {
                sizes.sort_by(f64::total_cmp);
                (key, sizes[sizes.len() / 2])
            })
            .collect()
    };
    let exact = usual(exact);
    let loose: HashMap<String, f64> = usual(loose)
        .into_iter()
        .filter(|(_, size)| body <= 0.0 || *size <= body * 1.1)
        .collect();
    for (a, role, _, _) in all.iter_mut() {
        if !in_margin(a) {
            continue;
        }
        let size = exact
            .get(&compact(&a.original_text))
            .copied()
            .or_else(|| furniture_key(&a.original_text).and_then(|k| loose.get(&k).copied()));
        let Some(size) = size else {
            continue;
        };
        if a.size > size * 1.2 {
            continue;
        }
        *role = if a.top_fraction() < 0.10 {
            "header"
        } else {
            "footer"
        }
        .into();
    }
}

pub fn extract(bytes: &[u8], pdf: &PdfText, pieces: &[Piece<'_>]) -> PdfStructure {
    extract_with_progress(bytes, pdf, pieces, |_, _, _| Ok(())).expect("uninterrupted analysis")
}

pub fn extract_with_progress(
    bytes: &[u8],
    pdf: &PdfText,
    pieces: &[Piece<'_>],
    mut progress: impl FnMut(&str, usize, usize) -> crate::error::AppResult<()>,
) -> crate::error::AppResult<PdfStructure> {
    let mut all: Vec<(Anchor, String, bool, u32)> = Vec::new();
    let mut complete = pdf.failed_pages.is_empty();
    let mut chars = 0usize;
    for (page_index, page) in pdf.layouts.iter().enumerate() {
        progress("alignment", page_index, pdf.layouts.len())?;
        if all.len() >= MAX_LINES || chars >= MAX_CHARS {
            complete = false;
            break;
        }
        let mut at = 0usize;
        let mut block = None;
        for line in &page.lines {
            if all.len() >= MAX_LINES || chars >= MAX_CHARS {
                complete = false;
                break;
            }
            if block.is_some() && block != Some(line.block) {
                at += 1;
            }
            block = Some(line.block);
            let start = at;
            at += line.text.len() + 1;
            chars += line.text.len();
            if ![line.x0, line.x1, line.y, line.size, page.width, page.height]
                .iter()
                .all(|v| v.is_finite())
            {
                complete = false;
                continue;
            }
            let role = match line.role {
                LineRole::Note => "note",
                LineRole::Foot => "footer",
                LineRole::Rotated | LineRole::Drawn => "uncertain",
                _ => "paragraph",
            };
            all.push((
                Anchor {
                    page: page.number,
                    page_width: page.width,
                    page_height: page.height,
                    x0: line.x0,
                    x1: line.x1,
                    baseline: line.y,
                    size: line.size,
                    first_char: line.first_char,
                    column: line.column,
                    byte_start: start,
                    byte_end: at - 1,
                    original_text: line.text.clone(),
                },
                role.into(),
                matches!(page.order, PageOrder::Uncertain),
                line.block,
            ));
        }
    }
    classify_furniture(&mut all);
    let mut by_page: HashMap<u32, Vec<usize>> = HashMap::new();
    for (index, (anchor, _, _, _)) in all.iter().enumerate() {
        by_page.entry(anchor.page).or_default().push(index);
    }
    let mut pieces_by_page: HashMap<u32, Vec<&Piece<'_>>> = HashMap::new();
    for piece in pieces {
        pieces_by_page.entry(piece.page).or_default().push(piece);
    }
    progress("native", 0, 0)?;
    let native = native::read(bytes, &all.iter().map(|x| x.0.clone()).collect::<Vec<_>>());
    progress("headings", 0, 0)?;
    complete &= !native.truncated;
    let mut headings = Vec::new();
    let mut selected = HashSet::new();
    // Supplied structure wins at the same anchored line. Native parser returns
    // only internal destinations/tags that align to an original drawn line.
    for candidate in &native.headings {
        let a = &candidate.anchor;
        if let Some((piece, start, end)) = pieces_by_page
            .get(&a.page)
            .into_iter()
            .flatten()
            .copied()
            .find_map(|p| {
                (p.byte_start
                    .zip(p.byte_end)
                    .is_none_or(|(s, e)| a.byte_start >= s as usize && a.byte_start < e as usize))
                .then(|| align(p, &a.original_text))
                .flatten()
                .map(|(s, e)| (p, s, e))
            })
        {
            if selected.insert(a.id()) {
                headings.push(Heading {
                    id: a.id(),
                    title: candidate.title.clone(),
                    level: candidate.level,
                    parents: candidate.parents.clone(),
                    basis: candidate.basis.clone(),
                    anchor: a.clone(),
                    segment_position: piece.position,
                    start,
                    end,
                    disabled: false,
                    mapping: None,
                    mapping_scope: None,
                });
            }
        }
    }
    let mut previous: HashMap<(u32, u8), f64> = HashMap::new();
    let lines: Vec<_> = all
        .iter()
        .filter(|(_, role, uncertain, _)| !*uncertain && role == "paragraph")
        .map(|(a, _, _, _)| {
            let gap = previous
                .insert((a.page, a.column), a.baseline)
                .map(|y| (a.baseline - y) / a.size.max(1.0))
                .unwrap_or(99.0);
            generated_contents::Line {
                page: a.page,
                top: a.top_fraction(),
                size: a.size,
                gap_above: gap,
                text: a.original_text.clone(),
            }
        })
        .collect();
    for (level, page, title) in generated_contents::headings(&lines, pdf.layouts.len() as u32) {
        // Do not turn explicit cross-reference labels into section context.
        let lower = title.to_lowercase();
        if ["see ", "siehe ", "vgl.", "cf.", "ver ", "voir ", "참조"]
            .iter()
            .any(|p| lower.starts_with(p))
        {
            continue;
        }
        // Multi-line generated headings must still have a uniquely anchored
        // opening line; the original title/text and coordinates stay separate.
        if let Some((a, _, _, _)) = by_page
            .get(&page)
            .into_iter()
            .flatten()
            .map(|i| &all[*i])
            .find(|(a, role, u, _)| {
                !*u && role == "paragraph" && title.starts_with(&compact(&a.original_text))
            })
        {
            if selected.contains(&a.id()) {
                continue;
            }
            if let Some((piece, start, end)) = pieces_by_page
                .get(&page)
                .into_iter()
                .flatten()
                .copied()
                .find_map(|p| {
                    (p.byte_start.zip(p.byte_end).is_none_or(|(s, e)| {
                        a.byte_start >= s as usize && a.byte_start < e as usize
                    }))
                    .then(|| align(p, &a.original_text))
                    .flatten()
                    .map(|(s, e)| (p, s, e))
                })
            {
                selected.insert(a.id());
                headings.push(Heading {
                    id: a.id(),
                    title,
                    level: level as u8,
                    parents: Vec::new(),
                    basis: "layout".into(),
                    anchor: a.clone(),
                    segment_position: piece.position,
                    start,
                    end,
                    disabled: false,
                    mapping: None,
                    mapping_scope: None,
                });
            }
        }
    }
    printed::promote(&all, &native.labels, &mut headings);
    headings.sort_by(|a, b| {
        a.segment_position
            .cmp(&b.segment_position)
            .then(a.start.cmp(&b.start))
            .then(a.id.cmp(&b.id))
    });
    if headings.len() >= MAX_HEADINGS {
        complete = false;
    }
    headings.truncate(MAX_HEADINGS);
    let role_lookup: HashMap<_, _> = native
        .roles
        .iter()
        .map(|r| ((r.page, compact(&r.text)), r))
        .collect();
    let mut blocks = Vec::new();
    for (piece_index, piece) in pieces.iter().enumerate() {
        if piece_index % 64 == 0 {
            progress("alignment", piece_index, pieces.len())?;
        }
        let found: Vec<_> =
            by_page
                .get(&piece.page)
                .into_iter()
                .flatten()
                .map(|i| &all[*i])
                .filter(|(a, _, _, _)| {
                    piece.byte_start.zip(piece.byte_end).is_none_or(|(s, e)| {
                        a.byte_start >= s as usize && a.byte_start < e as usize
                    }) && align(piece, &a.original_text).is_some()
                })
                .collect();
        if found.is_empty() {
            continue;
        }
        let mut roles: HashSet<&str> = found.iter().map(|(_, r, _, _)| r.as_str()).collect();
        let tag = role_lookup.get(&(piece.page, compact(piece.text))).copied();
        let role = if let Some(tag) = tag {
            tag.role.as_str()
        } else if roles.len() == 1 {
            roles.drain().next().unwrap_or("paragraph")
        } else {
            "paragraph"
        };
        let role = if role == "paragraph"
            && ["• ", "- ", "– "].iter().any(|p| piece.text.starts_with(p))
        {
            "list"
        } else {
            role
        };
        blocks.push(Block {
            position: piece.position,
            page: piece.page,
            role: role.into(),
            basis: if tag.is_some() { "tagged" } else { "layout" }.into(),
            anchors: found.iter().map(|x| x.0.clone()).collect(),
            printed_label: native.labels.get(&piece.page).cloned(),
            uncertain: found.iter().any(|x| x.2),
            runs: found
                .iter()
                .filter_map(|(a, role, _, _)| {
                    let role = role_lookup
                        .get(&(a.page, compact(&a.original_text)))
                        .map(|r| r.role.as_str())
                        .unwrap_or(role);
                    let (start, end) = align(piece, &a.original_text)?;
                    Some(TextRole {
                        start,
                        end,
                        role: role.into(),
                    })
                })
                .collect(),
        });
    }
    Ok(PdfStructure {
        version: VERSION,
        analysis: ANALYSIS,
        page_count: pdf.layouts.len() as u32,
        complete,
        tagged: native.tagged,
        unaligned_tags: native.unaligned_tags,
        headings,
        blocks,
    })
}

pub use resolve::{effective, presentations};
