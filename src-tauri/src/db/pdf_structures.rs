//! App-owned PDF metadata and reversible corrections. No source/segment text,
//! annotation, quotation or reader-position record is replaced by this service.
use super::{library, new_id, now};
use crate::{
    error::{AppError, AppResult, UserMessage},
    library::{
        contents::{ContentsItem, ContentsTarget},
        passage_index,
        pdf_structure::{self, Correction, PdfStructure, Presentation, ResolvedHeading},
    },
    scripture::reference::ScriptureRef,
};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{Arc, Mutex, OnceLock},
};

pub struct Record {
    pub hash: String,
    pub document: PdfStructure,
    pub overrides: Vec<Correction>,
    pub history: Vec<Vec<Correction>>,
    pub headings: Vec<ResolvedHeading>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub source_id: String,
    pub file_hash: Option<String>,
    pub status: String,
    pub complete: bool,
    pub tagged: bool,
    pub unaligned_tags: usize,
    pub uncertain_pages: usize,
    pub headings: Vec<ResolvedHeading>,
    pub corrections: Vec<Correction>,
    pub can_undo: bool,
    /// The analysis rules the stored structure was built with (0 = before A.4.1).
    pub analysis: u32,
    /// Built with earlier rules: **Rebuild structure** applies the current ones.
    pub rebuild_recommended: bool,
}
fn corrupt() -> AppError {
    AppError::InvalidInput(UserMessage::new("library.structure.corrupt","The saved PDF structure could not be read. The source text and original file are unchanged."))
}
fn invalid() -> AppError {
    AppError::InvalidInput(UserMessage::new(
        "library.structure.invalid",
        "This structure correction no longer matches the source. Reopen its text and try again.",
    ))
}
// Two documents at most; an immutable random revision also separates disposable
// databases and restores. Never key cached content by a reused source ID alone.
type Cache = VecDeque<(String, Arc<Record>)>;
static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();
pub fn load(conn: &Connection, id: &str) -> AppResult<Option<Arc<Record>>> {
    let identity = conn.query_row("SELECT p.file_hash,p.etag,s.language FROM source_pdf_structures p JOIN source_files f ON f.source_id=p.source_id AND f.is_primary=1 AND f.deleted_at IS NULL AND f.sha256=p.file_hash JOIN sources s ON s.id=p.source_id AND s.deleted_at IS NULL WHERE p.source_id=?1",[id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,Option<String>>(2)?))).optional()?;
    let Some((hash, etag, language)) = identity else {
        return Ok(None);
    };
    let key = format!("{id}/{hash}/{etag}/{language:?}");
    let cache = CACHE.get_or_init(|| Mutex::new(VecDeque::new()));
    if let Some(record) = cache
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .find(|(k, _)| k == &key)
        .map(|(_, r)| Arc::clone(r))
    {
        return Ok(Some(record));
    }
    let (doc, overrides, history): (String, String, String) = conn.query_row(
        "SELECT document,overrides,history FROM source_pdf_structures WHERE source_id=?1",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    let document: PdfStructure = serde_json::from_str(&doc).map_err(|_| corrupt())?;
    if document.version != pdf_structure::VERSION {
        return Err(corrupt());
    }
    let overrides: Vec<Correction> = serde_json::from_str(&overrides).map_err(|_| corrupt())?;
    let headings = pdf_structure::effective(
        &document,
        &overrides,
        passage_index::language_of(language.as_deref(), "PDF section mapping"),
    );
    let record = Arc::new(Record {
        hash,
        document,
        overrides,
        history: serde_json::from_str(&history).map_err(|_| corrupt())?,
        headings,
    });
    let mut cache = cache.lock().unwrap_or_else(|e| e.into_inner());
    cache.push_front((key, Arc::clone(&record)));
    cache.truncate(2);
    Ok(Some(record))
}
pub fn view(conn: &Connection, id: &str) -> AppResult<View> {
    let source = library::get(conn, id)?;
    if source.format.as_deref() != Some("pdf") {
        return Err(invalid());
    }
    let record = load(conn, id)?;
    if let Some(record) = record {
        let headings = record.headings.clone();
        let uncertain: std::collections::HashSet<_> = record
            .document
            .blocks
            .iter()
            .filter(|b| b.uncertain)
            .map(|b| b.page)
            .collect();
        Ok(View {
            source_id: id.into(),
            file_hash: Some(record.hash.clone()),
            status: "ready".into(),
            complete: record.document.complete,
            tagged: record.document.tagged,
            unaligned_tags: record.document.unaligned_tags,
            uncertain_pages: uncertain.len(),
            headings,
            corrections: record.overrides.clone(),
            can_undo: !record.history.is_empty(),
            analysis: record.document.analysis,
            rebuild_recommended: record.document.analysis < pdf_structure::ANALYSIS,
        })
    } else {
        Ok(View {
            source_id: id.into(),
            file_hash: source.content_hash,
            status: "missing".into(),
            complete: false,
            tagged: false,
            unaligned_tags: 0,
            uncertain_pages: 0,
            headings: Vec::new(),
            corrections: Vec::new(),
            can_undo: false,
            analysis: 0,
            rebuild_recommended: false,
        })
    }
}
pub fn overlay(
    conn: &Connection,
    id: &str,
    positions: &HashSet<i64>,
) -> AppResult<HashMap<i64, Presentation>> {
    let record = match load(conn, id) {
        Ok(Some(record)) => record,
        Ok(None) => return Ok(HashMap::new()),
        Err(_) => {
            log::warn!("PDF structure overlay unavailable; reading original stored text");
            return Ok(HashMap::new());
        }
    };
    Ok(pdf_structure::presentations(
        &record.document,
        &record.headings,
        positions,
    ))
}
/// Called inside the source import/rebuild transaction. Hash identity must
/// already match the managed file record; rebuilding keeps corrections/history.
pub fn store(conn: &Connection, id: &str, hash: &str, document: &PdfStructure) -> AppResult<()> {
    let actual:Option<String>=conn.query_row("SELECT sha256 FROM source_files WHERE source_id=?1 AND is_primary=1 AND deleted_at IS NULL",[id],|r|r.get(0)).optional()?;
    if actual.as_deref() != Some(hash) {
        return Err(invalid());
    }
    let json = serde_json::to_string(document).map_err(|_| corrupt())?;
    conn.execute("INSERT INTO source_pdf_structures(source_id,file_hash,version,document,updated_at,etag) VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(source_id) DO UPDATE SET file_hash=excluded.file_hash,version=excluded.version,document=excluded.document,updated_at=excluded.updated_at,etag=excluded.etag",params![id,hash,pdf_structure::VERSION,json,now(),new_id()])?;
    Ok(())
}
pub fn replace(
    conn: &Connection,
    id: &str,
    hash: &str,
    document: &PdfStructure,
) -> AppResult<View> {
    replace_observed(conn, id, hash, document, |_, _, _| Ok(()))
}

/// Only structure-derived data changes. Re-parsing all literal references here
/// used to hold the application DB mutex for the entire source a second time.
pub fn replace_observed(
    conn: &Connection,
    id: &str,
    hash: &str,
    document: &PdfStructure,
    mut progress: impl FnMut(&str, usize, usize) -> AppResult<()>,
) -> AppResult<View> {
    let tx = conn.unchecked_transaction()?;
    progress("store", 0, 0)?;
    library::get(&tx, id)?;
    store(&tx, id, hash, document)?;
    tx.execute(
        "DELETE FROM source_passages WHERE source_id=?1 AND origin IN ('section','inherited')",
        [id],
    )?;
    tx.execute(
        "UPDATE source_passages SET structure_role=NULL WHERE source_id=?1 AND origin='explicit'",
        [id],
    )?;
    let indexed = index_observed(&tx, id, |done, total| progress("index", done, total))?;
    progress("indexed", indexed, indexed)?;
    let result = view(&tx, id)?;
    progress("commit", 0, 0)?;
    tx.commit()?;
    Ok(result)
}

pub fn correct(
    conn: &Connection,
    id: &str,
    expected_hash: &str,
    mut correction: Correction,
) -> AppResult<View> {
    let record = load(conn, id)?.ok_or_else(invalid)?;
    if record.hash != expected_hash
        || correction.id.len() > 200
        || correction.title.trim().is_empty()
        || correction.title.chars().count() > 300
        || !(1..=12).contains(&correction.level)
    {
        return Err(invalid());
    }
    correction.title = correction.title.trim().into();
    let block = record
        .document
        .blocks
        .iter()
        .find(|b| b.position == correction.position)
        .ok_or_else(invalid)?;
    let anchor = block
        .anchors
        .iter()
        .find(|a| a.id() == correction.anchor_id)
        .ok_or_else(invalid)?;
    if let Some(existing) = record
        .document
        .headings
        .iter()
        .find(|h| h.id == correction.id)
    {
        if existing.segment_position != correction.position || existing.anchor.id() != anchor.id() {
            return Err(invalid());
        }
    } else if !correction.id.starts_with("manual-") {
        return Err(invalid());
    }
    let text: String = conn.query_row(
        "SELECT content FROM source_segments WHERE source_id=?1 AND position=?2",
        params![id, correction.position],
        |r| r.get(0),
    )?;
    let original = pdf_structure::compact(&anchor.original_text);
    let Some(start) = text.find(&original) else {
        return Err(invalid());
    };
    if text[start + original.len()..].contains(&original)
        || correction.start != text[..start].encode_utf16().count() as u32
        || correction.end != text[..start + original.len()].encode_utf16().count() as u32
    {
        return Err(invalid());
    }
    let boundaries: std::collections::HashSet<u32> = std::iter::once(0)
        .chain(text.chars().scan(0, |at, c| {
            *at += c.len_utf16() as u32;
            Some(*at)
        }))
        .collect();
    if correction.start >= correction.end
        || !boundaries.contains(&correction.start)
        || !boundaries.contains(&correction.end)
    {
        return Err(invalid());
    }
    if let Some(mapping) = correction.mapping.as_ref().filter(|m| !m.trim().is_empty()) {
        let source = library::get(conn, id)?;
        let language = passage_index::language_of(source.language.as_deref(), "PDF correction");
        let reference = ScriptureRef::from_canonical(mapping).or_else(|_| {
            crate::scripture::parser::ReferenceParser::for_language(language).parse(mapping)
        })?;
        let scope = correction.scope.as_deref().unwrap_or(
            if reference.start.verse.is_none() && reference.start == reference.end {
                "chapter"
            } else {
                "passage"
            },
        );
        if !matches!(scope, "book" | "chapter" | "passage")
            || scope == "book"
                && !(reference.start.chapter == 1
                    && reference.end.chapter == reference.book.chapter_count
                    && reference.start.verse.is_none()
                    && reference.end.verse.is_none())
            || scope == "chapter"
                && !(reference.start == reference.end && reference.start.verse.is_none())
        {
            return Err(invalid());
        }
        correction.mapping = Some(reference.canonical());
        correction.scope = Some(scope.into());
    } else {
        correction.mapping = None;
        correction.scope = None;
    }
    let mut overrides = record.overrides.clone();
    if let Some(old) = overrides.iter_mut().find(|p| p.id == correction.id) {
        *old = correction;
    } else {
        if overrides.len() >= 5000 {
            return Err(invalid());
        }
        overrides.push(correction);
    }
    save_corrections(conn, id, record, overrides)
}
fn save_corrections(
    conn: &Connection,
    id: &str,
    record: Arc<Record>,
    overrides: Vec<Correction>,
) -> AppResult<View> {
    let mut history = record.history.clone();
    history.push(record.overrides.clone());
    if history.len() > 20 {
        history.remove(0);
    }
    write_corrections(conn, id, &overrides, &history)
}
fn write_corrections(
    conn: &Connection,
    id: &str,
    overrides: &[Correction],
    history: &[Vec<Correction>],
) -> AppResult<View> {
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE source_pdf_structures SET overrides=?2,history=?3,updated_at=?4,etag=?5 WHERE source_id=?1",
        params![
            id,
            serde_json::to_string(overrides).map_err(|_| corrupt())?,
            serde_json::to_string(history).map_err(|_| corrupt())?,
            now(), new_id()
        ],
    )?;
    library::reindex_passages_in_transaction(&tx, id)?;
    tx.commit()?;
    view(conn, id)
}
pub fn undo(conn: &Connection, id: &str, expected_hash: &str, reset: bool) -> AppResult<View> {
    let record = load(conn, id)?.ok_or_else(invalid)?;
    if record.hash != expected_hash {
        return Err(invalid());
    }
    if reset {
        return save_corrections(conn, id, record, Vec::new());
    }
    let mut history = record.history.clone();
    let previous = history.pop().ok_or_else(invalid)?;
    write_corrections(conn, id, &previous, &history)
}

/// Derived links are opt-in to this validated PDF metadata, never enabled by
/// a format string. Literal references and user-confirmed links stay separate.
pub fn index(conn: &Connection, id: &str) -> AppResult<usize> {
    index_observed(conn, id, |_, _| Ok(()))
}

fn index_observed(
    conn: &Connection,
    id: &str,
    mut progress: impl FnMut(usize, usize) -> AppResult<()>,
) -> AppResult<usize> {
    let Some(record) = load(conn, id)? else {
        return Ok(0);
    };
    let headings = record.headings.clone();
    let rows: HashMap<i64, String> = conn
        .prepare("SELECT position,id FROM source_segments WHERE source_id=?1")?
        .query_map([id], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<_, _>>()?;
    let mut statement=conn.prepare("INSERT INTO source_passages(id,source_id,segment_id,passage,start_key,end_key,matched_text,created_at,origin,section_segment_id,section_heading,structure_node_id,structure_scope,structure_basis) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)")?;
    // A single ordered sweep: each body block inherits only its nearest mapped
    // ancestor. Unmapped siblings still close the preceding section.
    let blocks: HashMap<_, _> = record
        .document
        .blocks
        .iter()
        .map(|b| (b.position, b))
        .collect();
    let eligible: Vec<_> = headings
        .iter()
        .filter(|h| h.canonical.is_some())
        .filter(|h| {
            h.heading.basis == "user"
                || !blocks.get(&h.heading.segment_position).is_some_and(|b| {
                    let role = b
                        .runs
                        .iter()
                        .find(|r| r.start <= h.heading.start && r.end >= h.heading.end)
                        .map(|r| r.role.as_str())
                        .unwrap_or(&b.role);
                    matches!(role, "note" | "header" | "footer" | "artifact")
                })
        })
        .map(|h| {
            Ok((
                h,
                ScriptureRef::from_canonical(h.canonical.as_ref().unwrap())?,
            ))
        })
        .collect::<AppResult<_>>()?;
    let listing_pages: HashSet<u32> = headings
        .iter()
        .filter(|h| h.scope == "contents")
        .map(|h| h.heading.anchor.page)
        .collect();
    let mut active: Vec<usize> = Vec::new();
    let mut at = 0;
    let mut count = 0usize;
    let mut mentions = conn.prepare("SELECT id,start_offset,end_offset FROM source_passages WHERE segment_id=?1 AND origin='explicit'")?;
    let mut role = conn.prepare("UPDATE source_passages SET structure_role=?2 WHERE id=?1")?;
    for (block_index, block) in record.document.blocks.iter().enumerate() {
        if block_index % 64 == 0 {
            progress(block_index, record.document.blocks.len())?;
        }
        let Some(segment) = rows.get(&block.position) else {
            continue;
        };
        let literal = mentions
            .query_map([segment], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<u32>>(1)?,
                    r.get::<_, Option<u32>>(2)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        for (link, start, end) in literal {
            let kind = start
                .zip(end)
                .and_then(|(s, e)| block.runs.iter().find(|r| r.start <= s && e <= r.end))
                .map(|r| r.role.as_str())
                .unwrap_or(&block.role);
            // A.4.1: a reference on a Contents/listing page is a listing entry.
            let kind = if listing_pages.contains(&block.page) && kind == "paragraph" {
                "contents"
            } else {
                kind
            };
            role.execute(params![link, kind])?;
        }
        active.retain(|i| block.position < eligible[*i].0.end_position);
        let mut openers = Vec::new();
        while at < eligible.len() && eligible[at].0.heading.segment_position <= block.position {
            if eligible[at].0.heading.segment_position == block.position {
                openers.push(at);
            }
            if block.position < eligible[at].0.end_position {
                active.push(at);
            }
            at += 1;
        }
        let choices = if !openers.is_empty() {
            openers
        } else if !block.uncertain
            && !matches!(
                block.role.as_str(),
                "note" | "header" | "footer" | "artifact" | "uncertain"
            )
        {
            active.last().copied().into_iter().collect()
        } else {
            Vec::new()
        };
        for i in choices {
            let (heading, reference) = &eligible[i];
            let Some(opener) = rows.get(&heading.heading.segment_position) else {
                continue;
            };
            statement.execute(params![
                new_id(),
                id,
                segment,
                reference.canonical(),
                reference.start_key(),
                reference.end_key(),
                heading.heading.title,
                now(),
                if block.position == heading.heading.segment_position {
                    "section"
                } else {
                    "inherited"
                },
                opener,
                heading.heading.title,
                heading.heading.id,
                heading.scope,
                heading.heading.basis
            ])?;
            count += 1;
        }
    }
    Ok(count)
}

pub fn contents(conn: &Connection, id: &str) -> AppResult<Vec<ContentsItem>> {
    let Some(record) = load(conn, id)? else {
        return Ok(Vec::new());
    };
    let headings = record.headings.clone();
    fn tree(
        headings: &[ResolvedHeading],
        at: &mut usize,
        level: u8,
        end: i64,
    ) -> Vec<ContentsItem> {
        let mut out = Vec::new();
        while *at < headings.len()
            && headings[*at].heading.level >= level
            && headings[*at].heading.segment_position < end
        {
            let h = &headings[*at];
            let this = h.heading.level;
            *at += 1;
            let children = if *at < headings.len() && headings[*at].heading.level > this {
                tree(headings, at, headings[*at].heading.level, h.end_position)
            } else {
                Vec::new()
            };
            out.push(ContentsItem {
                id: h.heading.id.clone(),
                title: h.heading.title.clone(),
                target: Some(ContentsTarget::Segment {
                    position: h.heading.segment_position,
                }),
                children,
            });
        }
        out
    }
    // Keep producer grouping titles with no drawable destination. They are
    // navigation groups, never fabricated text targets or passage openers.
    fn append(out: &mut Vec<ContentsItem>, branch: ContentsItem) {
        if let Some(last) = out
            .last_mut()
            .filter(|last| last.target.is_none() && branch.target.is_none() && last.id == branch.id)
        {
            for child in branch.children {
                append(&mut last.children, child);
            }
        } else {
            out.push(branch);
        }
    }
    fn groups(
        items: Vec<ContentsItem>,
        headings: &[ResolvedHeading],
        document: &PdfStructure,
    ) -> Vec<ContentsItem> {
        let mut out: Vec<ContentsItem> = Vec::new();
        for mut item in items {
            item.children = groups(item.children, headings, document);
            let parents = headings
                .iter()
                .find(|h| h.heading.id == item.id)
                .map(|h| h.heading.parents.clone())
                .unwrap_or_default();
            let missing: Vec<_> = parents
                .iter()
                .enumerate()
                .filter(|(_, p)| !document.headings.iter().any(|h| &h.title == *p))
                .collect();
            let mut branch = item;
            for (index, title) in missing.into_iter().rev() {
                branch = ContentsItem {
                    id: format!(
                        "pdf-group:{}",
                        serde_json::to_string(&parents[..=index]).unwrap_or_default()
                    ),
                    title: title.clone(),
                    target: None,
                    children: vec![branch],
                };
            }
            append(&mut out, branch);
        }
        out
    }
    Ok(if headings.is_empty() {
        Vec::new()
    } else {
        groups(
            tree(&headings, &mut 0, 1, i64::MAX),
            &headings,
            &record.document,
        )
    })
}
