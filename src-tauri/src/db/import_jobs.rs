//! Import jobs (v0.24): many sources added at once, kept in the database so
//! they can be reviewed, corrected, cancelled, and continued after a crash.
//!
//! A job's items move `waiting` → (read and checked) `ready` / `duplicate` /
//! `problem` → (added) `imported` / `failed`. Only chosen `ready` and
//! `duplicate` items are added. Reading and adding happen one item per call,
//! so a job of thousands of files never holds more than one file's text.

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::db::{new_id, now};
use crate::error::{AppError, AppResult, UserMessage};

/// Files one job may hold.
pub const MAX_ITEMS: usize = 2_000;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct JobItem {
    pub id: String,
    pub position: i64,
    pub path: String,
    pub file_name: String,
    pub size: i64,
    pub status: String,
    pub chosen: bool,
    #[serde(skip)]
    pub sha256: Option<String>,
    pub format: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub language: Option<String>,
    pub source_type: Option<String>,
    pub type_evidence: Option<String>,
    pub page_count: Option<i64>,
    pub duplicate_of: Option<String>,
    pub duplicate_title: Option<String>,
    pub problem_code: Option<String>,
    pub problem: Option<String>,
    pub problem_params: Option<serde_json::Value>,
    pub source_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct JobCounts {
    pub total: i64,
    pub waiting: i64,
    pub ready: i64,
    pub duplicate: i64,
    pub problem: i64,
    pub imported: i64,
    pub failed: i64,
    /// Chosen and still to be added.
    pub to_add: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: String,
    pub state: String,
    pub collection_id: Option<String>,
    pub tags: Vec<String>,
    pub unsupported_count: i64,
    pub over_limit_count: i64,
    pub created_at: String,
    pub counts: JobCounts,
    pub items: Vec<JobItem>,
}

/// A file found by the scan.
#[derive(Debug, Clone)]
pub struct FoundFile {
    pub path: String,
    pub file_name: String,
    pub size: i64,
}

pub fn create(
    conn: &Connection,
    files: &[FoundFile],
    unsupported: usize,
    over_limit: usize,
) -> AppResult<Job> {
    if files.is_empty() {
        return Err(AppError::InvalidInput(UserMessage::new(
            "batch.noFiles",
            "No files the library can read were found there.",
        )));
    }
    if files.len() > MAX_ITEMS {
        return Err(AppError::InvalidInput(
            UserMessage::new(
                "batch.tooMany",
                format!("At most {MAX_ITEMS} files can be added at once."),
            )
            .with("max", MAX_ITEMS),
        ));
    }
    let id = new_id();
    let at = now();
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO import_jobs (id, kind, state, unsupported_count, over_limit_count, created_at, updated_at)
         VALUES (?1, 'sources', 'review', ?2, ?3, ?4, ?4)",
        params![id, unsupported as i64, over_limit as i64, at],
    )?;
    for (position, file) in files.iter().enumerate() {
        tx.execute(
            "INSERT INTO import_job_items (id, job_id, position, path, file_name, size, status, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'waiting', ?7)",
            params![new_id(), id, position as i64, file.path, file.file_name, file.size, at],
        )?;
    }
    tx.commit()?;
    get(conn, &id)
}

const ITEM_SELECT: &str =
    "SELECT id, position, path, file_name, size, status, chosen, format, title,
        author, language, source_type, type_evidence, page_count, duplicate_of, duplicate_title,
        problem_code, problem, problem_params, source_id, sha256
     FROM import_job_items";

fn item_from_row(r: &rusqlite::Row) -> rusqlite::Result<JobItem> {
    let params: Option<String> = r.get(18)?;
    Ok(JobItem {
        id: r.get(0)?,
        position: r.get(1)?,
        path: r.get(2)?,
        file_name: r.get(3)?,
        size: r.get(4)?,
        status: r.get(5)?,
        chosen: r.get::<_, i64>(6)? == 1,
        format: r.get(7)?,
        title: r.get(8)?,
        author: r.get(9)?,
        language: r.get(10)?,
        source_type: r.get(11)?,
        type_evidence: r.get(12)?,
        page_count: r.get(13)?,
        duplicate_of: r.get(14)?,
        duplicate_title: r.get(15)?,
        problem_code: r.get(16)?,
        problem: r.get(17)?,
        problem_params: params.and_then(|p| serde_json::from_str(&p).ok()),
        source_id: r.get(19)?,
        sha256: r.get(20)?,
    })
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Job> {
    let row = conn
        .query_row(
            "SELECT state, collection_id, tags, unsupported_count, over_limit_count, created_at
             FROM import_jobs WHERE id = ?1",
            [id],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, i64>(4)?,
                    r.get::<_, String>(5)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| AppError::not_found("import", id))?;
    let mut stmt = conn.prepare(&format!(
        "{ITEM_SELECT} WHERE job_id = ?1 ORDER BY position"
    ))?;
    let items = stmt
        .query_map([id], item_from_row)?
        .collect::<Result<Vec<_>, _>>()?;
    let mut counts = JobCounts {
        total: items.len() as i64,
        ..JobCounts::default()
    };
    for item in &items {
        match item.status.as_str() {
            "waiting" => counts.waiting += 1,
            "ready" => counts.ready += 1,
            "duplicate" => counts.duplicate += 1,
            "problem" => counts.problem += 1,
            "imported" => counts.imported += 1,
            _ => counts.failed += 1,
        }
        if item.chosen && matches!(item.status.as_str(), "ready" | "duplicate") {
            counts.to_add += 1;
        }
    }
    Ok(Job {
        id: id.to_string(),
        state: row.0,
        collection_id: row.1,
        tags: serde_json::from_str(&row.2).unwrap_or_default(),
        unsupported_count: row.3,
        over_limit_count: row.4,
        created_at: row.5,
        counts,
        items,
    })
}

/// Jobs not finished, cancelled or discarded: shown so they can be continued.
pub fn open_jobs(conn: &Connection) -> AppResult<Vec<Job>> {
    let ids: Vec<String> = conn
        .prepare(
            "SELECT id FROM import_jobs WHERE state IN ('review', 'importing', 'interrupted')
             ORDER BY created_at DESC",
        )?
        .query_map([], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    ids.iter().map(|id| get(conn, id)).collect()
}

/// At start-up: a job that was adding files when the app stopped is
/// interrupted (its added files are kept; the rest can be continued).
pub fn mark_interrupted(conn: &Connection) -> AppResult<usize> {
    Ok(conn.execute(
        "UPDATE import_jobs SET state = 'interrupted', updated_at = ?1 WHERE state = 'importing'",
        [now()],
    )?)
}

pub fn next_waiting(conn: &Connection, job: &str) -> AppResult<Option<JobItem>> {
    Ok(conn
        .query_row(
            &format!(
                "{ITEM_SELECT} WHERE job_id = ?1 AND status = 'waiting' ORDER BY position LIMIT 1"
            ),
            [job],
            item_from_row,
        )
        .optional()?)
}

pub fn next_to_add(conn: &Connection, job: &str) -> AppResult<Option<JobItem>> {
    Ok(conn
        .query_row(
            &format!(
                "{ITEM_SELECT} WHERE job_id = ?1 AND chosen = 1 AND status IN ('ready', 'duplicate')
                 ORDER BY position LIMIT 1"
            ),
            [job],
            item_from_row,
        )
        .optional()?)
}

pub fn item(conn: &Connection, job: &str, item: &str) -> AppResult<JobItem> {
    conn.query_row(
        &format!("{ITEM_SELECT} WHERE job_id = ?1 AND id = ?2"),
        [job, item],
        item_from_row,
    )
    .optional()?
    .ok_or_else(|| AppError::not_found("import item", item))
}

/// What reading one file found.
#[derive(Debug, Clone, Default)]
pub struct Analysis {
    pub sha256: String,
    pub format: String,
    pub title: String,
    pub author: Option<String>,
    pub language: Option<String>,
    pub source_type: String,
    pub type_evidence: String,
    pub page_count: Option<i64>,
    pub duplicate_of: Option<(String, String)>,
}

/// Records a file read and checked. A file whose content is already earlier
/// in the job is a duplicate of it (and not chosen).
pub fn record_analysis(conn: &Connection, job: &str, item: &str, a: &Analysis) -> AppResult<()> {
    let earlier: Option<(String, String)> = conn
        .query_row(
            "SELECT id, COALESCE(title, file_name) FROM import_job_items
             WHERE job_id = ?1 AND sha256 = ?2 AND id <> ?3 AND status <> 'waiting'
             ORDER BY position LIMIT 1",
            params![job, a.sha256, item],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let duplicate = a
        .duplicate_of
        .clone()
        .map(|d| (Some(d.0), d.1))
        .or(earlier.map(|e| (None, e.1)));
    let status = if duplicate.is_some() {
        "duplicate"
    } else {
        "ready"
    };
    conn.execute(
        "UPDATE import_job_items SET status = ?3, chosen = ?4, sha256 = ?5, format = ?6, title = ?7,
             author = ?8, language = ?9, source_type = ?10, type_evidence = ?11, page_count = ?12,
             duplicate_of = ?13, duplicate_title = ?14, problem_code = NULL, problem = NULL,
             problem_params = NULL, updated_at = ?15
         WHERE job_id = ?1 AND id = ?2",
        params![
            job,
            item,
            status,
            i64::from(duplicate.is_none()),
            a.sha256,
            a.format,
            a.title,
            a.author,
            a.language,
            a.source_type,
            a.type_evidence,
            a.page_count,
            duplicate.as_ref().and_then(|d| d.0.clone()),
            duplicate.as_ref().map(|d| d.1.clone()),
            now()
        ],
    )?;
    touch(conn, job)
}

/// Why a file cannot be added, as the interface translates it.
#[derive(Debug, Clone, Default)]
pub struct Problem {
    pub code: Option<String>,
    pub text: String,
    pub params: std::collections::BTreeMap<String, String>,
}

impl From<&UserMessage> for Problem {
    fn from(m: &UserMessage) -> Self {
        Self {
            code: m.code.map(String::from),
            text: m.text.clone(),
            params: m
                .params
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
        }
    }
}

impl From<crate::error::ErrorPayload> for Problem {
    fn from(e: crate::error::ErrorPayload) -> Self {
        Self {
            code: e.code,
            text: e.message,
            params: e.params,
        }
    }
}

/// Records why a file cannot be added (`problem`) or failed while adding (`failed`).
pub fn record_problem(
    conn: &Connection,
    job: &str,
    item: &str,
    failed: bool,
    message: &Problem,
) -> AppResult<()> {
    conn.execute(
        "UPDATE import_job_items SET status = ?3, chosen = 0, problem_code = ?4, problem = ?5,
             problem_params = ?6, updated_at = ?7
         WHERE job_id = ?1 AND id = ?2",
        params![
            job,
            item,
            if failed { "failed" } else { "problem" },
            message.code,
            message.text,
            serde_json::to_string(&message.params).ok(),
            now()
        ],
    )?;
    touch(conn, job)
}

pub fn record_imported(conn: &Connection, job: &str, item: &str, source_id: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE import_job_items SET status = 'imported', source_id = ?3, problem_code = NULL,
             problem = NULL, problem_params = NULL, updated_at = ?4
         WHERE job_id = ?1 AND id = ?2",
        params![job, item, source_id, now()],
    )?;
    touch(conn, job)
}

/// A source with this content added since the job began: a file added just
/// before a crash, before its item was marked (so it is not added twice).
pub fn added_since(
    conn: &Connection,
    job: &Job,
    sha256: &str,
    file_name: &str,
) -> AppResult<Option<String>> {
    Ok(conn
        .query_row(
            "SELECT s.id FROM source_files f JOIN sources s ON s.id = f.source_id
             WHERE f.sha256 = ?1 AND f.original_filename = ?3 AND s.deleted_at IS NULL
               AND s.state <> 'trashed' AND s.created_at >= ?2
               AND NOT EXISTS (SELECT 1 FROM import_job_items i WHERE i.source_id = s.id)
             ORDER BY s.created_at LIMIT 1",
            params![sha256, job.created_at, file_name],
            |r| r.get(0),
        )
        .optional()?)
}

/// The person's corrections to one item.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemEdit {
    pub id: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub language: Option<String>,
    pub source_type: Option<String>,
    pub chosen: Option<bool>,
}

pub fn edit_item(conn: &Connection, job: &str, edit: &ItemEdit) -> AppResult<JobItem> {
    let current = item(conn, job, &edit.id)?;
    editable(conn, job)?;
    if edit.chosen == Some(true) && !matches!(current.status.as_str(), "ready" | "duplicate") {
        return Err(AppError::InvalidInput(UserMessage::new(
            "batch.cannotChoose",
            "This file cannot be added.",
        )));
    }
    if let Some(kind) = &edit.source_type {
        crate::library::SourceType::parse(kind)?;
    }
    let title = match &edit.title {
        Some(t) => Some(crate::domain::validate::title(t, "source")?),
        None => current.title.clone(),
    };
    let clean = |v: &Option<String>, old: &Option<String>| match v {
        Some(v) if v.trim().is_empty() => None,
        Some(v) => Some(v.trim().to_string()),
        None => old.clone(),
    };
    conn.execute(
        "UPDATE import_job_items SET title = ?3, author = ?4, language = ?5, source_type = ?6,
             type_evidence = CASE WHEN ?7 THEN 'person' ELSE type_evidence END, chosen = ?8, updated_at = ?9
         WHERE job_id = ?1 AND id = ?2",
        params![
            job,
            edit.id,
            title,
            clean(&edit.author, &current.author),
            clean(&edit.language, &current.language),
            edit.source_type.clone().or(current.source_type.clone()),
            edit.source_type.is_some(),
            i64::from(edit.chosen.unwrap_or(current.chosen)),
            now()
        ],
    )?;
    item(conn, job, &edit.id)
}

/// The collection and tags every added source gets.
pub fn set_shared(
    conn: &Connection,
    job: &str,
    collection: Option<&str>,
    tags: &[String],
) -> AppResult<()> {
    editable(conn, job)?;
    let tags: Vec<String> = tags
        .iter()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect();
    // Checked now, as tags are when given to one source: a tag refused later
    // would fail every file after it was added.
    for tag in &tags {
        crate::db::library_manage::check_tag_name(tag)?;
    }
    if let Some(collection) = collection {
        conn.query_row(
            "SELECT 1 FROM source_collections WHERE id = ?1",
            [collection],
            |_| Ok(()),
        )
        .optional()?
        .ok_or_else(|| AppError::not_found("collection", collection))?;
    }
    conn.execute(
        "UPDATE import_jobs SET collection_id = ?2, tags = ?3, updated_at = ?4 WHERE id = ?1",
        params![
            job,
            collection,
            serde_json::to_string(&tags).unwrap_or_else(|_| "[]".into()),
            now()
        ],
    )?;
    Ok(())
}

/// Files to read again (after a problem or failure was fixed outside the app).
pub fn retry(conn: &Connection, job: &str, items: &[String]) -> AppResult<usize> {
    editable(conn, job)?;
    let tx = conn.unchecked_transaction()?;
    let mut changed = 0;
    for id in items {
        changed += tx.execute(
            "UPDATE import_job_items SET status = 'waiting', chosen = 1, problem_code = NULL,
                 problem = NULL, problem_params = NULL, updated_at = ?3
             WHERE job_id = ?1 AND id = ?2 AND status IN ('problem', 'failed')",
            params![job, id, now()],
        )?;
    }
    tx.commit()?;
    touch(conn, job)?;
    Ok(changed)
}

/// Moves a job on. An ended job (finished or cancelled) stays ended: a cancel
/// that lands while a step runs is never undone by the step.
pub fn set_state(conn: &Connection, job: &str, state: &str) -> AppResult<()> {
    let changed = conn.execute(
        "UPDATE import_jobs SET state = ?2, updated_at = ?3
         WHERE id = ?1 AND state NOT IN ('finished', 'cancelled')",
        params![job, state, now()],
    )?;
    if changed == 0 {
        // Unknown, or already ended (then nothing to do).
        conn.query_row("SELECT 1 FROM import_jobs WHERE id = ?1", [job], |_| Ok(()))
            .optional()?
            .ok_or_else(|| AppError::not_found("import", job))?;
    }
    Ok(())
}

/// Forgets a job (its added sources stay in the library).
pub fn discard(conn: &Connection, job: &str) -> AppResult<()> {
    conn.execute("DELETE FROM import_jobs WHERE id = ?1", [job])?;
    Ok(())
}

fn editable(conn: &Connection, job: &str) -> AppResult<()> {
    let state: String = conn
        .query_row("SELECT state FROM import_jobs WHERE id = ?1", [job], |r| {
            r.get(0)
        })
        .optional()?
        .ok_or_else(|| AppError::not_found("import", job))?;
    if matches!(state.as_str(), "finished" | "cancelled") {
        return Err(AppError::Conflict(UserMessage::new(
            "batch.closed",
            "This import has ended. Start a new one to add more files.",
        )));
    }
    Ok(())
}

fn touch(conn: &Connection, job: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE import_jobs SET updated_at = ?2 WHERE id = ?1",
        params![job, now()],
    )?;
    Ok(())
}
