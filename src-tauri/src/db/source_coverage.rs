//! Whole-source recommendations from saved intent or corroborated metadata.
//! Reads bounded indexed structure, never body citations or the original file.
use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::error::{AppError, AppResult};
use crate::library::{
    coverage::{self, CoverageBasis, CoverageMode, SourceCoverage},
    sections,
};
use crate::scripture::{language::ReferenceLanguage, reference::ScriptureRef};

const MAX_STRUCTURAL_ROWS: usize = 5_000;

struct Metadata {
    id: String,
    title: String,
    subtitle: Option<String>,
    author: Option<String>,
    kind: String,
    language: Option<String>,
    data: Option<String>,
}
const METADATA: &str = "SELECT id,title,subtitle,author,source_type,language,data FROM sources";
fn metadata(row: &Row) -> rusqlite::Result<Metadata> {
    Ok(Metadata {
        id: row.get(0)?,
        title: row.get(1)?,
        subtitle: row.get(2)?,
        author: row.get(3)?,
        kind: row.get(4)?,
        language: row.get(5)?,
        data: row.get(6)?,
    })
}

struct Evidence {
    reference: ScriptureRef,
    scope: Option<String>,
    node: Option<String>,
    heading: Option<String>,
    path: Option<String>,
    position: i64,
}
type EvidenceRow = (
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    i64,
);

fn heading_path(evidence: &Evidence) -> Option<String> {
    let heading = evidence.heading.as_deref()?;
    let parts: Vec<_> = sections::components(evidence.path.as_deref()?).collect();
    let at = parts.iter().rposition(|part| *part == heading)?;
    Some(parts[..=at].join(sections::PATH_SEPARATOR))
}

fn corroborated(conn: &Connection, id: &str, candidate: &ScriptureRef) -> AppResult<bool> {
    // The cap is on real indexed headings, never on mention frequency. A
    // truncated hierarchy is insufficient evidence for an automatic claim.
    let raw: Vec<EvidenceRow> = conn.prepare(
        "SELECT p.passage,p.structure_scope,p.structure_node_id,p.section_heading,g.section_path,g.position
         FROM source_passages p INDEXED BY source_passages_source JOIN source_segments g ON g.id=p.segment_id
         WHERE p.source_id=?1 AND p.origin='section'
           AND COALESCE(p.structure_role,'') NOT IN ('contents','note','header','footer','artifact')
         ORDER BY g.position,p.id LIMIT ?2",
    )?.query_map(params![id, (MAX_STRUCTURAL_ROWS + 1) as i64], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?)))?.collect::<Result<_,_>>()?;
    if raw.is_empty() || raw.len() > MAX_STRUCTURAL_ROWS {
        return Ok(false);
    }
    let evidence: Vec<_> = raw
        .into_iter()
        .filter_map(|(canonical, scope, node, heading, path, position)| {
            ScriptureRef::from_canonical(&canonical)
                .ok()
                .map(|reference| Evidence {
                    reference,
                    scope,
                    node,
                    heading,
                    path,
                    position,
                })
        })
        .collect();
    let inside: Vec<_> = evidence
        .iter()
        .filter(|e| coverage::contains(candidate, &e.reference))
        .collect();
    if inside.is_empty() {
        return Ok(false);
    }
    let record = super::pdf_structures::load(conn, id).ok().flatten();
    if evidence.iter().any(|e| {
        e.node
            .as_deref()
            .is_some_and(|node| !node.starts_with("epub-toc:"))
    }) && record.is_none()
    {
        return Ok(false);
    }
    for outer in evidence
        .iter()
        .filter(|e| !coverage::contains(candidate, &e.reference))
    {
        if !coverage::contains(&outer.reference, candidate) {
            return Ok(false);
        }
        // A broader book/chapter may just be the real ancestor of the scoped
        // discussion. A sibling/outside section is competing content instead.
        if !matches!(outer.scope.as_deref(), Some("book" | "chapter")) {
            return Ok(false);
        }
        let generic_path = heading_path(outer);
        let generic_end = if let Some(path) = &generic_path {
            let prefix = format!("{path}{}", sections::PATH_SEPARATOR);
            conn.query_row("SELECT MIN(position) FROM source_segments WHERE source_id=?1 AND position>?2 AND (section_path IS NULL OR (section_path<>?3 AND substr(section_path,1,length(?4))<>?4))",params![id,outer.position,path,prefix],|r|r.get::<_,Option<i64>>(0))?
        } else {
            None
        };
        for child in &inside {
            if child.position < outer.position {
                return Ok(false);
            }
            let actual_child = if let Some((path, start, end)) =
                outer.node.as_deref().and_then(sections::contents_identity)
            {
                child.position >= start
                    && child.position < end
                    && match child.node.as_deref().and_then(sections::contents_identity) {
                        Some((child_path, _, _)) => child_path.starts_with(&format!("{path}/")),
                        None => child.scope != outer.scope,
                    }
            } else if let Some(record) = &record {
                let parent = record
                    .headings
                    .iter()
                    .find(|h| Some(h.heading.id.as_str()) == outer.node.as_deref());
                let child_heading = record
                    .headings
                    .iter()
                    .find(|h| Some(h.heading.id.as_str()) == child.node.as_deref());
                parent.zip(child_heading).is_some_and(|(parent, child)| {
                    child
                        .path
                        .starts_with(&format!("{}{}", parent.path, sections::PATH_SEPARATOR))
                        && child.heading.segment_position < parent.end_position
                })
            } else {
                generic_path.as_ref().is_some_and(|path| {
                    generic_end.is_none_or(|end| child.position < end)
                        && heading_path(child).is_some_and(|child_path| {
                            child_path.starts_with(&format!("{path}{}", sections::PATH_SEPARATOR))
                        })
                })
            };
            if !actual_child {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

fn effective(
    conn: &Connection,
    source: &Metadata,
    query: Option<&ScriptureRef>,
) -> AppResult<(CoverageMode, Vec<SourceCoverage>)> {
    let (mode, mut ranges) = coverage::stored(source.data.as_deref());
    if mode == CoverageMode::Automatic {
        let language = ReferenceLanguage::resolve(source.language.as_deref()).language;
        if let Some(candidate) = coverage::candidate(
            &source.title,
            source.subtitle.as_deref(),
            &source.kind,
            language,
        ) {
            if query.is_none_or(|query| coverage::overlaps(query, &candidate))
                && corroborated(conn, &source.id, &candidate)?
            {
                ranges.push(SourceCoverage::new(candidate, CoverageBasis::Metadata));
            }
        }
    }
    if let Some(query) = query {
        ranges.retain(|range| {
            ScriptureRef::from_canonical(&range.reference.canonical)
                .is_ok_and(|reference| coverage::overlaps(query, &reference))
        });
    }
    Ok((mode, ranges))
}

pub(crate) fn for_source(
    conn: &Connection,
    id: &str,
) -> AppResult<(CoverageMode, Vec<SourceCoverage>)> {
    let source = conn
        .query_row(
            &format!("{METADATA} WHERE id=?1 AND deleted_at IS NULL"),
            [id],
            metadata,
        )
        .optional()?
        .ok_or_else(|| AppError::not_found("source", id))?;
    effective(conn, &source, None)
}

pub(crate) struct CoveredSource {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub ranges: Vec<SourceCoverage>,
}

pub(crate) fn for_passage(
    conn: &Connection,
    reference: &ScriptureRef,
) -> AppResult<Vec<CoveredSource>> {
    let sources = conn
        .prepare(&format!(
            "{METADATA} WHERE deleted_at IS NULL AND state='active' ORDER BY id"
        ))?
        .query_map([], metadata)?
        .collect::<Result<Vec<_>, _>>()?;
    let mut out = Vec::new();
    for source in sources {
        let (_, ranges) = effective(conn, &source, Some(reference))?;
        if !ranges.is_empty() {
            out.push(CoveredSource {
                id: source.id,
                title: source.title,
                author: source.author,
                ranges,
            });
        }
    }
    Ok(out)
}
