//! A.4.1: a local, exportable, **metadata-only** report of how a PDF Source's
//! structure was read and which place a passage opens, so the owner can check
//! the real MacArthur document without sharing it.
//!
//! The report never contains paragraph text, notes, writing, keys or paths.
//! It carries the managed file's SHA-256/size, structure counts, heading
//! titles shortened to [`TITLE_CHARS`] (headings are structural labels), each
//! candidate's matched reference words (shortened the same way), ranks,
//! physical/printed pages and the exact target the Reader is handed.
use super::{library, now, pdf_structures};
use crate::{
    error::{AppError, AppResult, UserMessage},
    library::{passage_index, pdf_structure},
    scripture::{recognise::Recogniser, reference::ScriptureRef},
};
use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;
use std::collections::BTreeMap;

pub const SCHEMA: &str = "abidingword.pdf-structure-diagnostic/1";
pub const TITLE_CHARS: usize = 80;
pub const MAX_HEADINGS: usize = 2_000;
pub const MAX_CANDIDATES: usize = 25;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing: Option<crate::library::pdf_build::Progress>,
    pub schema: &'static str,
    pub app_version: &'static str,
    pub generated_at: String,
    pub privacy: &'static str,
    pub source: SourceFacts,
    pub structure: StructureFacts,
    pub headings: Vec<HeadingFacts>,
    pub headings_truncated: bool,
    pub query: Option<Query>,
    /// What the Reader showed when the report was made (after the click).
    pub reader_observed: Option<ReaderObserved>,
}

/// The Reader's own state, supplied by the interface. Only identifiers,
/// mode and page numbers are accepted; each value is length-capped.
#[derive(Debug, Clone, Default, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReaderObserved {
    pub mode: Option<String>,
    pub physical_page: Option<u32>,
    pub printed_page: Option<String>,
    pub opened_segment_id: Option<String>,
    pub opened_structure_node_id: Option<String>,
    pub focused_heading_id: Option<String>,
    pub variant: Option<String>,
}

impl ReaderObserved {
    fn sanitized(self) -> Self {
        let cap = |v: Option<String>, n: usize| {
            v.map(|s| {
                s.chars()
                    .filter(|c| !c.is_control())
                    .take(n)
                    .collect::<String>()
            })
        };
        Self {
            mode: cap(self.mode, 16),
            physical_page: self.physical_page,
            printed_page: cap(self.printed_page, 16),
            opened_segment_id: cap(self.opened_segment_id, 64),
            opened_structure_node_id: cap(self.opened_structure_node_id, 64),
            focused_heading_id: cap(self.focused_heading_id, 64),
            variant: cap(self.variant, 16),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceFacts {
    pub id: String,
    pub format: Option<String>,
    pub language: Option<String>,
    pub index_language: String,
    pub file_sha256: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub segment_count: i64,
    pub pages_with_text: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructureFacts {
    /// none (never built), ready, stale (file changed), corrupt.
    pub status: String,
    pub analysis: Option<u32>,
    pub current_analysis: u32,
    pub rebuild_recommended: bool,
    pub page_count: Option<u32>,
    pub complete: Option<bool>,
    pub tagged: Option<bool>,
    pub unaligned_tags: Option<usize>,
    pub heading_count: usize,
    pub by_basis: BTreeMap<String, usize>,
    pub by_scope: BTreeMap<String, usize>,
    pub by_level: BTreeMap<u8, usize>,
    pub listing_pages: Vec<u32>,
    pub corrections: usize,
    pub block_roles: BTreeMap<String, usize>,
    pub derived_links: BTreeMap<String, usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeadingFacts {
    pub id: String,
    pub level: u8,
    pub depth: usize,
    pub basis: String,
    pub scope: String,
    pub canonical: Option<String>,
    pub physical_page: u32,
    pub printed_page: Option<String>,
    pub segment_position: i64,
    pub top_fraction: f64,
    pub type_size: f64,
    pub title: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Query {
    pub input: String,
    pub canonical: Option<String>,
    pub candidates: Vec<Candidate>,
    /// What a click on the Source result hands the Reader.
    pub reader_target: Option<ReaderTarget>,
    pub verdict: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub evidence_tier: u8,
    pub rank: usize,
    pub class: u8,
    pub origin: String,
    pub evidence: String,
    pub scope: Option<String>,
    pub basis: Option<String>,
    pub role: Option<String>,
    pub structure_node_id: Option<String>,
    pub segment_position: i64,
    pub physical_page: Option<i64>,
    pub printed_page: Option<String>,
    pub passage: String,
    pub matched: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReaderTarget {
    pub segment_id: String,
    pub segment_position: i64,
    pub structure_node_id: Option<String>,
    pub physical_page: Option<i64>,
    pub printed_page: Option<String>,
    /// Where the heading is drawn on its original page (0 = top).
    pub heading_top_fraction: Option<f64>,
}

pub(crate) fn shorten(text: &str) -> String {
    let text = pdf_structure::compact(text);
    if text.chars().count() <= TITLE_CHARS {
        return text;
    }
    let mut out: String = text.chars().take(TITLE_CHARS - 1).collect();
    out.push('…');
    out
}

/// The evidence wording the owner asked for (#22), in English: the report is
/// a technical record, the interface shows the localised labels.
fn evidence(origin: &str, scope: Option<&str>, role: Option<&str>) -> String {
    match (origin, scope, role) {
        ("manual", _, _) => "Linked by the user".into(),
        ("section", Some("passage"), _) => "Section about the passage".into(),
        ("section", Some("chapter"), _) => "Chapter heading".into(),
        ("section", Some("book"), _) => "Book section (broader)".into(),
        ("section", _, _) => "Section heading".into(),
        ("explicit", _, Some("contents")) => "Contents listing entry".into(),
        ("explicit", _, Some("note")) => "Mention in a note".into(),
        ("explicit", _, _) => "Mentions the passage".into(),
        (other, _, _) => other.into(),
    }
}

fn parse_passage(
    input: &str,
    language: crate::scripture::language::ReferenceLanguage,
) -> Option<ScriptureRef> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }
    if let Ok(r) = ScriptureRef::from_canonical(input) {
        return Some(r);
    }
    // The source's language first, then the six interface languages: the owner
    // may type "Acts 6" while the source is German.
    use crate::scripture::language::ReferenceLanguage as L;
    [language, L::De, L::En, L::Ko, L::Fr, L::Es, L::Pt]
        .into_iter()
        .find_map(|lang| {
            let found = Recogniser::for_source(lang).detect(input);
            let first = found.first()?;
            ScriptureRef::from_canonical(&first.reference.canonical).ok()
        })
}

pub fn diagnostic(conn: &Connection, id: &str, passage: Option<&str>) -> AppResult<Diagnostic> {
    diagnostic_with(conn, id, passage, None)
}

pub fn diagnostic_with(
    conn: &Connection,
    id: &str,
    passage: Option<&str>,
    observed: Option<ReaderObserved>,
) -> AppResult<Diagnostic> {
    let source = library::get(conn, id)?;
    if source.format.as_deref() != Some("pdf") {
        return Err(AppError::InvalidInput(UserMessage::new(
            "library.structure.invalid",
            "This source is not a PDF.",
        )));
    }
    let language = passage_index::language_of(source.language.as_deref(), "PDF diagnostic");
    let file: Option<(Option<String>, Option<i64>)> = conn
        .query_row(
            "SELECT sha256,size_bytes FROM source_files WHERE source_id=?1 AND is_primary=1 AND deleted_at IS NULL",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let (segment_count, pages_with_text): (i64, i64) = conn.query_row(
        "SELECT count(*), count(DISTINCT page_number) FROM source_segments WHERE source_id=?1",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let stored_hash: Option<String> = conn
        .query_row(
            "SELECT file_hash FROM source_pdf_structures WHERE source_id=?1",
            [id],
            |r| r.get(0),
        )
        .optional()?;
    let mut derived_links = BTreeMap::new();
    {
        let mut statement = conn.prepare(
            "SELECT origin, count(*) FROM source_passages WHERE source_id=?1 GROUP BY origin",
        )?;
        for row in
            statement.query_map([id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?
        {
            let (origin, count) = row?;
            derived_links.insert(origin, count as usize);
        }
    }
    let file_sha256 = file.as_ref().and_then(|f| f.0.clone());
    let loaded = pdf_structures::load(conn, id);
    let mut structure = StructureFacts {
        status: "none".into(),
        analysis: None,
        current_analysis: pdf_structure::ANALYSIS,
        rebuild_recommended: true,
        page_count: None,
        complete: None,
        tagged: None,
        unaligned_tags: None,
        heading_count: 0,
        by_basis: BTreeMap::new(),
        by_scope: BTreeMap::new(),
        by_level: BTreeMap::new(),
        listing_pages: Vec::new(),
        corrections: 0,
        block_roles: BTreeMap::new(),
        derived_links,
    };
    let mut headings = Vec::new();
    let mut truncated = false;
    let mut labels = std::collections::HashMap::new();
    match loaded {
        Err(_) => structure.status = "corrupt".into(),
        Ok(None) if stored_hash.is_some() => structure.status = "stale".into(),
        Ok(None) => {}
        Ok(Some(record)) => {
            let doc = &record.document;
            structure.status = "ready".into();
            structure.analysis = Some(doc.analysis);
            structure.rebuild_recommended = doc.analysis < pdf_structure::ANALYSIS;
            structure.page_count = Some(doc.page_count);
            structure.complete = Some(doc.complete);
            structure.tagged = Some(doc.tagged);
            structure.unaligned_tags = Some(doc.unaligned_tags);
            structure.corrections = record.overrides.len();
            structure.heading_count = record.headings.len();
            for block in &doc.blocks {
                *structure.block_roles.entry(block.role.clone()).or_default() += 1;
                if let Some(label) = &block.printed_label {
                    labels.insert(block.page, label.clone());
                }
            }
            let mut listing = std::collections::BTreeSet::new();
            for h in &record.headings {
                *structure
                    .by_basis
                    .entry(h.heading.basis.clone())
                    .or_default() += 1;
                *structure.by_scope.entry(h.scope.clone()).or_default() += 1;
                *structure.by_level.entry(h.heading.level).or_default() += 1;
                if h.scope == "contents" {
                    listing.insert(h.heading.anchor.page);
                }
            }
            structure.listing_pages = listing.into_iter().collect();
            truncated = record.headings.len() > MAX_HEADINGS;
            headings = record
                .headings
                .iter()
                .take(MAX_HEADINGS)
                .map(|h| HeadingFacts {
                    id: h.heading.id.clone(),
                    level: h.heading.level,
                    depth: h.path.matches(" › ").count(),
                    basis: h.heading.basis.clone(),
                    scope: h.scope.clone(),
                    canonical: h.canonical.clone(),
                    physical_page: h.heading.anchor.page,
                    printed_page: labels.get(&h.heading.anchor.page).cloned(),
                    segment_position: h.heading.segment_position,
                    top_fraction: (h.heading.anchor.top_fraction() * 1000.0).round() / 1000.0,
                    type_size: (h.heading.anchor.size * 10.0).round() / 10.0,
                    title: shorten(&h.heading.title),
                })
                .collect();
        }
    }
    let query = match passage {
        None => None,
        Some(input) => Some(query(conn, id, input, language, &labels, &headings)?),
    };
    Ok(Diagnostic {
        processing: None,
        schema: SCHEMA,
        app_version: env!("CARGO_PKG_VERSION"),
        generated_at: now(),
        privacy: "Metadata only: no paragraph text, notes, writing, file paths or keys. Heading titles and matched reference words are shortened to 80 characters. Review before sharing.",
        source: SourceFacts {
            id: id.into(),
            format: source.format.clone(),
            language: source.language.clone(),
            index_language: format!("{language:?}"),
            file_sha256,
            file_size_bytes: file.and_then(|f| f.1),
            segment_count,
            pages_with_text,
        },
        structure,
        headings,
        headings_truncated: truncated,
        query,
        reader_observed: observed.map(ReaderObserved::sanitized),
    })
}

fn query(
    conn: &Connection,
    id: &str,
    input: &str,
    language: crate::scripture::language::ReferenceLanguage,
    labels: &std::collections::HashMap<u32, String>,
    headings: &[HeadingFacts],
) -> AppResult<Query> {
    let input = input.chars().take(60).collect::<String>();
    let Some(reference) = parse_passage(&input, language) else {
        return Ok(Query {
            input,
            canonical: None,
            candidates: Vec::new(),
            reader_target: None,
            verdict: "The passage could not be read. Use a form like \"Apg 6\" or \"Acts 6\"."
                .into(),
        });
    };
    let places = library::passage_source_diagnostics(conn, &reference, id)?;
    let mut page_of = conn.prepare("SELECT page_number FROM source_segments WHERE id=?1")?;
    let mut candidates = Vec::new();
    for (index, place) in places.iter().take(MAX_CANDIDATES).enumerate() {
        let page: Option<i64> = page_of
            .query_row([&place.location.segment_id], |r| r.get(0))
            .optional()?
            .flatten();
        let origin = serde_json::to_value(place.location.origin)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default();
        candidates.push(Candidate {
            rank: index + 1,
            evidence_tier: place.evidence_tier,
            class: place.class,
            evidence: evidence(
                &origin,
                place.location.structure_scope.as_deref(),
                place.role.as_deref(),
            ),
            origin,
            scope: place.location.structure_scope.clone(),
            basis: place.location.structure_basis.clone(),
            role: place.role.clone(),
            structure_node_id: place.location.structure_node_id.clone(),
            segment_position: place.position,
            physical_page: page,
            printed_page: page.and_then(|p| labels.get(&(p as u32)).cloned()),
            passage: ScriptureRef::from_canonical(&place.location.reference.canonical)
                .map(|r| r.canonical())
                .unwrap_or_default(),
            matched: shorten(&place.location.matched_text),
        });
    }
    let reader_target = places.first().map(|first| {
        let page = candidates.first().and_then(|c| c.physical_page);
        ReaderTarget {
            segment_id: first.location.segment_id.clone(),
            segment_position: first.position,
            structure_node_id: first.location.structure_node_id.clone(),
            physical_page: page,
            printed_page: page.and_then(|p| labels.get(&(p as u32)).cloned()),
            heading_top_fraction: first
                .location
                .structure_node_id
                .as_ref()
                .and_then(|node| headings.iter().find(|h| &h.id == node))
                .map(|h| h.top_fraction),
        }
    });
    let verdict = match candidates.first() {
        None => "No place in this source shares a verse with the passage.".into(),
        Some(c) if c.origin == "section" || c.origin == "manual" => format!(
            "First result is grounded document structure: {} on PDF page {}.",
            c.evidence.to_lowercase(),
            c.physical_page.map_or("?".into(), |p| p.to_string())
        ),
        Some(c) => format!(
            "No matching section identified; the first result is only a literal mention ({}) on PDF page {}.",
            c.evidence.to_lowercase(),
            c.physical_page.map_or("?".into(), |p| p.to_string())
        ),
    };
    Ok(Query {
        input,
        canonical: Some(reference.canonical()),
        candidates,
        reader_target,
        verdict,
    })
}
