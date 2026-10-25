//! Looking after the library (v0.24): archive, Trash, restore and removal
//! for good, for sources and Bibles alike; the Sources list in Settings
//! (search, filters, sorting, pages); metadata; collections and tags.
//!
//! States: `active` (as before), `archived` (kept and searchable, out of
//! the everyday lists and the Sources pane), `trashed` (hidden, recoverable
//! for [`TRASH_RETENTION_DAYS`], then removed for good). Removal for good is
//! only possible from Trash: a source becomes the citation tombstone it
//! always became, a Bible's rows go. Files are deleted only inside
//! Abiding Word's own folders, and only after the database has committed.

use rusqlite::types::Value;
use rusqlite::{params, params_from_iter, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::db::{bibles, library, new_id, now, workspaces};
use crate::domain::validate::{self, DESCRIPTION_MAX_CHARS};
use crate::error::{AppError, AppResult, UserMessage};
use crate::library::{LicenceStatus, SourceType};

/// How long a trashed item stays recoverable.
pub const TRASH_RETENTION_DAYS: i64 = 30;
/// Items one action may change at once.
pub const MAX_BATCH: usize = 5_000;
/// Rows one page of the Sources list may hold.
pub const MAX_PAGE: i64 = 200;
const NAME_MAX: usize = 120;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ItemState {
    Active,
    Archived,
    Trashed,
}

impl ItemState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Archived => "archived",
            Self::Trashed => "trashed",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value {
            "archived" => Self::Archived,
            "trashed" => Self::Trashed,
            _ => Self::Active,
        }
    }
}

fn check_batch(ids: &[String]) -> AppResult<()> {
    if ids.is_empty() {
        return Err(AppError::InvalidInput(UserMessage::new(
            "library.nothingChosen",
            "Choose at least one item.",
        )));
    }
    if ids.len() > MAX_BATCH {
        return Err(AppError::InvalidInput(
            UserMessage::new(
                "library.tooMany",
                format!("At most {MAX_BATCH} items can be changed at once."),
            )
            .with("count", MAX_BATCH),
        ));
    }
    Ok(())
}

// --- sources: state ------------------------------------------------------------

/// Moves sources to a state. Unknown or already removed ids are refused, and
/// then nothing changes. Returns how many changed.
pub fn set_source_state(conn: &Connection, ids: &[String], state: ItemState) -> AppResult<usize> {
    check_batch(ids)?;
    let tx = conn.unchecked_transaction()?;
    let at = now();
    let mut changed = 0;
    for id in ids {
        let current: Option<String> = tx
            .query_row(
                "SELECT state FROM sources WHERE id = ?1 AND deleted_at IS NULL",
                [id],
                |r| r.get(0),
            )
            .optional()?;
        let Some(current) = current else {
            return Err(AppError::not_found("source", id));
        };
        if current != state.as_str() {
            tx.execute(
                "UPDATE sources SET state = ?2, state_changed_at = ?3, updated_at = ?3,
                     revision = revision + 1
                 WHERE id = ?1",
                params![id, state.as_str(), at],
            )?;
            changed += 1;
        }
    }
    tx.commit()?;
    Ok(changed)
}

/// Removes trashed sources for good in one transaction. A source that is not
/// in Trash is refused (nothing is removed then). Returns each removed
/// source's managed file, to delete after this returns.
pub fn remove_sources(
    conn: &Connection,
    ids: &[String],
) -> AppResult<Vec<(String, Option<String>)>> {
    check_batch(ids)?;
    let tx = conn.unchecked_transaction()?;
    let at = now();
    let mut files = Vec::new();
    for id in ids {
        let state: Option<String> = tx
            .query_row(
                "SELECT state FROM sources WHERE id = ?1 AND deleted_at IS NULL",
                [id],
                |r| r.get(0),
            )
            .optional()?;
        match state.as_deref() {
            None => return Err(AppError::not_found("source", id)),
            Some("trashed") => {}
            Some(_) => return Err(not_in_trash()),
        }
        files.push((id.clone(), library::remove_in(&tx, id, &at)?));
    }
    tx.commit()?;
    log::info!("{} source(s) removed for good", files.len());
    Ok(files)
}

fn not_in_trash() -> AppError {
    AppError::Conflict(UserMessage::new(
        "library.notInTrash",
        "Only items in Trash can be deleted permanently. Move them to Trash first.",
    ))
}

// --- Bibles: state -------------------------------------------------------------

/// Moves Bibles to a state. A Bible leaving `active` is no longer the one
/// being read.
pub fn set_bible_state(conn: &Connection, ids: &[String], state: ItemState) -> AppResult<usize> {
    check_batch(ids)?;
    let tx = conn.unchecked_transaction()?;
    let at = now();
    let mut changed = 0;
    for id in ids {
        let current: Option<String> = tx
            .query_row(
                "SELECT state FROM bible_translations WHERE id = ?1",
                [id],
                |r| r.get(0),
            )
            .optional()?;
        let Some(current) = current else {
            return Err(AppError::not_found("Bible", id));
        };
        if current == state.as_str() {
            continue;
        }
        tx.execute(
            "UPDATE bible_translations SET state = ?2, state_changed_at = ?3, updated_at = ?3,
                 revision = revision + 1
             WHERE id = ?1",
            params![id, state.as_str(), at],
        )?;
        if state != ItemState::Active {
            tx.execute(
                "DELETE FROM settings WHERE key = 'scripture.translation' AND value = ?1",
                [serde_json::to_string(id).unwrap_or_default()],
            )?;
        }
        changed += 1;
    }
    tx.commit()?;
    Ok(changed)
}

/// A Bible removed for good, and whether its folder is Abiding Word's copy.
#[derive(Debug, Clone)]
pub struct RemovedBible {
    pub id: String,
    pub managed_copy: bool,
}

/// Removes trashed Bibles for good. A Bible not in Trash is refused before
/// anything is removed.
pub fn remove_bibles(conn: &Connection, ids: &[String]) -> AppResult<Vec<RemovedBible>> {
    check_batch(ids)?;
    for id in ids {
        let state: Option<String> = conn
            .query_row(
                "SELECT state FROM bible_translations WHERE id = ?1",
                [id],
                |r| r.get(0),
            )
            .optional()?;
        match state.as_deref() {
            None => return Err(AppError::not_found("Bible", id)),
            Some("trashed") => {}
            Some(_) => return Err(not_in_trash()),
        }
    }
    let mut removed = Vec::new();
    for id in ids {
        let bible = bibles::remove(conn, id)?;
        removed.push(RemovedBible {
            id: bible.id,
            managed_copy: bible.source_origin == "managed_copy",
        });
    }
    Ok(removed)
}

// --- retention -----------------------------------------------------------------

/// Items in Trash for longer than the retention period, as of `now`
/// (RFC 3339): sources, then Bibles.
pub fn expired(conn: &Connection, now: &str) -> AppResult<(Vec<String>, Vec<String>)> {
    let cutoff = cutoff(now)?;
    let ids = |sql: &str| -> AppResult<Vec<String>> {
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map([&cutoff], |r| r.get(0))?;
        Ok(rows.collect::<Result<_, _>>()?)
    };
    Ok((
        ids(
            "SELECT id FROM sources WHERE deleted_at IS NULL AND state = 'trashed'
             AND state_changed_at IS NOT NULL AND state_changed_at < ?1 LIMIT 5000",
        )?,
        ids("SELECT id FROM bible_translations WHERE state = 'trashed'
             AND state_changed_at IS NOT NULL AND state_changed_at < ?1 LIMIT 5000")?,
    ))
}

fn cutoff(now: &str) -> AppResult<String> {
    let at = chrono::DateTime::parse_from_rfc3339(now)
        .map_err(|e| AppError::Unexpected(format!("bad time {now}: {e}")))?;
    Ok((at - chrono::Duration::days(TRASH_RETENTION_DAYS))
        .with_timezone(&chrono::Utc)
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
}

// --- the Sources list ----------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum SortBy {
    Title,
    Author,
    Category,
    #[default]
    Added,
    Language,
    State,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ManageQuery {
    /// Words in the title or author.
    pub text: Option<String>,
    pub source_type: Option<SourceType>,
    pub language: Option<String>,
    pub format: Option<String>,
    /// None: active and archived (not Trash).
    pub state: Option<ItemState>,
    pub collection_id: Option<String>,
    pub tag_id: Option<String>,
    /// Only files whose text could not be read (scanned PDFs).
    pub without_text: bool,
    /// Only sources whose file is in the library more than once.
    pub duplicates: bool,
    pub sort: SortBy,
    pub descending: bool,
    pub offset: i64,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TagRef {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ManagedSource {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub source_type: SourceType,
    pub format: Option<String>,
    pub language: Option<String>,
    pub publication_year: Option<i64>,
    pub created_at: String,
    pub state: ItemState,
    pub state_changed_at: Option<String>,
    pub extraction_status: Option<String>,
    pub page_count: Option<i64>,
    pub original_filename: Option<String>,
    /// Other live sources with the same file content.
    pub duplicates: i64,
    pub tags: Vec<TagRef>,
    pub collections: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ManagePage {
    pub total: i64,
    pub offset: i64,
    pub items: Vec<ManagedSource>,
    /// Counts by state, for the filter (live sources only).
    pub active: i64,
    pub archived: i64,
    pub trashed: i64,
}

fn like(text: &str) -> String {
    let escaped = text
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{escaped}%")
}

/// One page of the Sources list. Every filter is a bound value; the page is
/// at most [`MAX_PAGE`] rows, whatever the library's size.
pub fn list(conn: &Connection, query: &ManageQuery) -> AppResult<ManagePage> {
    let mut clauses = vec!["s.deleted_at IS NULL".to_string()];
    let mut values: Vec<Value> = Vec::new();
    match query.state {
        Some(state) => {
            values.push(Value::Text(state.as_str().into()));
            clauses.push(format!("s.state = ?{}", values.len()));
        }
        None => clauses.push("s.state <> 'trashed'".into()),
    }
    if let Some(text) = query
        .text
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
    {
        values.push(Value::Text(like(text)));
        let n = values.len();
        clauses.push(format!(
            "(s.title LIKE ?{n} ESCAPE '\\' OR s.author LIKE ?{n} ESCAPE '\\' OR f.original_filename LIKE ?{n} ESCAPE '\\')"
        ));
    }
    if let Some(kind) = query.source_type {
        values.push(Value::Text(kind.as_str().into()));
        clauses.push(format!("s.source_type = ?{}", values.len()));
    }
    if let Some(language) = query.language.as_deref().filter(|l| !l.is_empty()) {
        values.push(Value::Text(language.into()));
        clauses.push(format!("s.language = ?{}", values.len()));
    }
    if let Some(format) = query.format.as_deref().filter(|f| !f.is_empty()) {
        values.push(Value::Text(format.into()));
        clauses.push(format!("f.format = ?{}", values.len()));
    }
    if let Some(collection) = query.collection_id.as_deref() {
        values.push(Value::Text(collection.into()));
        clauses.push(format!(
            "s.id IN (SELECT source_id FROM source_collection_items WHERE collection_id = ?{})",
            values.len()
        ));
    }
    if let Some(tag) = query.tag_id.as_deref() {
        values.push(Value::Text(tag.into()));
        clauses.push(format!(
            "s.id IN (SELECT source_id FROM source_tag_items WHERE tag_id = ?{})",
            values.len()
        ));
    }
    if query.without_text {
        clauses.push("f.extraction_status = 'ocr_required'".into());
    }
    if query.duplicates {
        clauses.push(
            "s.content_hash IS NOT NULL AND EXISTS (SELECT 1 FROM sources d
               WHERE d.content_hash = s.content_hash AND d.id <> s.id AND d.deleted_at IS NULL)"
                .into(),
        );
    }
    let filter = clauses.join(" AND ");
    let from = "FROM sources s
         LEFT JOIN source_files f ON f.source_id = s.id AND f.is_primary = 1 AND f.deleted_at IS NULL";
    let total: i64 = conn.query_row(
        &format!("SELECT COUNT(*) {from} WHERE {filter}"),
        params_from_iter(values.iter()),
        |r| r.get(0),
    )?;
    let direction = if query.descending { "DESC" } else { "ASC" };
    let order = match query.sort {
        SortBy::Title => format!("s.title COLLATE NOCASE {direction}"),
        SortBy::Author => format!("s.author IS NULL, s.author COLLATE NOCASE {direction}"),
        SortBy::Category => format!("s.source_type {direction}, s.title COLLATE NOCASE"),
        SortBy::Added => format!("s.created_at {direction}"),
        SortBy::Language => format!("s.language IS NULL, s.language COLLATE NOCASE {direction}"),
        SortBy::State => format!("s.state {direction}, s.title COLLATE NOCASE"),
    };
    let limit = query.limit.unwrap_or(100).clamp(1, MAX_PAGE);
    let offset = query.offset.max(0);
    let mut page_values = values.clone();
    page_values.push(Value::Integer(limit));
    page_values.push(Value::Integer(offset));
    let n = page_values.len();
    let mut stmt = conn.prepare(&format!(
        "SELECT s.id, s.title, s.author, s.source_type, f.format, s.language, s.publication_year,
                s.created_at, s.state, s.state_changed_at, f.extraction_status, f.page_count,
                f.original_filename,
                CASE WHEN s.content_hash IS NULL THEN 0 ELSE
                  (SELECT COUNT(*) FROM sources d WHERE d.content_hash = s.content_hash
                     AND d.id <> s.id AND d.deleted_at IS NULL) END
         {from} WHERE {filter}
         ORDER BY {order}, s.id
         LIMIT ?{} OFFSET ?{n}",
        n - 1
    ))?;
    let mut items = stmt
        .query_map(params_from_iter(page_values.iter()), |r| {
            let kind: String = r.get(3)?;
            Ok(ManagedSource {
                id: r.get(0)?,
                title: r.get(1)?,
                author: r.get(2)?,
                source_type: SourceType::parse(&kind).unwrap_or(SourceType::Document),
                format: r.get(4)?,
                language: r.get(5)?,
                publication_year: r.get(6)?,
                created_at: r.get(7)?,
                state: ItemState::parse(&r.get::<_, String>(8)?),
                state_changed_at: r.get(9)?,
                extraction_status: r.get(10)?,
                page_count: r.get(11)?,
                original_filename: r.get(12)?,
                duplicates: r.get(13)?,
                tags: Vec::new(),
                collections: Vec::new(),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    for item in &mut items {
        item.tags = tags_of(conn, &item.id)?;
        item.collections = collections_of(conn, &item.id)?;
    }
    let count = |state: &str| -> AppResult<i64> {
        Ok(conn.query_row(
            "SELECT COUNT(*) FROM sources WHERE deleted_at IS NULL AND state = ?1",
            [state],
            |r| r.get(0),
        )?)
    };
    Ok(ManagePage {
        total,
        offset,
        items,
        active: count("active")?,
        archived: count("archived")?,
        trashed: count("trashed")?,
    })
}

fn tags_of(conn: &Connection, source_id: &str) -> AppResult<Vec<TagRef>> {
    let mut stmt = conn.prepare_cached(
        "SELECT t.id, t.name FROM source_tag_items i JOIN source_tags t ON t.id = i.tag_id
         WHERE i.source_id = ?1 ORDER BY t.name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map([source_id], |r| {
        Ok(TagRef {
            id: r.get(0)?,
            name: r.get(1)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

fn collections_of(conn: &Connection, source_id: &str) -> AppResult<Vec<String>> {
    let mut stmt = conn.prepare_cached(
        "SELECT collection_id FROM source_collection_items WHERE source_id = ?1 ORDER BY collection_id",
    )?;
    let rows = stmt.query_map([source_id], |r| r.get(0))?;
    Ok(rows.collect::<Result<_, _>>()?)
}

// --- metadata --------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSource {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub author: Option<String>,
    pub publisher: Option<String>,
    pub edition: Option<String>,
    pub publication_year: Option<i64>,
    pub isbn: Option<String>,
    pub doi: Option<String>,
    pub language: Option<String>,
    pub source_type: SourceType,
    pub licence_status: LicenceStatus,
    pub licence: Option<String>,
    pub notes: Option<String>,
}

fn text(value: &Option<String>, max: usize, what: &str) -> AppResult<Option<String>> {
    validate::optional_text(value.as_deref(), max, what)
}

/// Saves a source's details. Its file, text and links are unchanged.
pub fn update_source(conn: &Connection, input: &UpdateSource) -> AppResult<crate::library::Source> {
    let title = validate::title(&input.title, "source")?;
    if let Some(year) = input.publication_year {
        if !(0..=3000).contains(&year) {
            return Err(AppError::InvalidInput(UserMessage::new(
                "library.badYear",
                "The year must be a number between 0 and 3000.",
            )));
        }
    }
    let changed = conn.execute(
        "UPDATE sources SET title = ?2, subtitle = ?3, author = ?4, publisher = ?5, edition = ?6,
             publication_year = ?7, isbn = ?8, doi = ?9, language = ?10, source_type = ?11,
             licence_status = ?12, licence = ?13, notes = ?14, updated_at = ?15,
             revision = revision + 1
         WHERE id = ?1 AND deleted_at IS NULL",
        params![
            input.id,
            title,
            text(&input.subtitle, 300, "subtitle")?,
            text(&input.author, 300, "author")?,
            text(&input.publisher, 300, "publisher")?,
            text(&input.edition, 100, "edition")?,
            input.publication_year,
            text(&input.isbn, 40, "ISBN")?,
            text(&input.doi, 200, "DOI")?,
            text(&input.language, 40, "language")?,
            input.source_type.as_str(),
            input.licence_status.as_str(),
            text(&input.licence, DESCRIPTION_MAX_CHARS, "licence")?,
            text(&input.notes, DESCRIPTION_MAX_CHARS, "notes")?,
            now(),
        ],
    )?;
    if changed == 0 {
        return Err(AppError::not_found("source", &input.id));
    }
    library::get(conn, &input.id)
}

// --- collections -----------------------------------------------------------------

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
    /// Live sources in it, not counting Trash.
    pub count: i64,
}

fn name(value: &str, what: &str) -> AppResult<String> {
    let name = value.trim();
    if name.is_empty() || name.chars().count() > NAME_MAX {
        return Err(AppError::InvalidInput(
            UserMessage::new(
                "library.badName",
                format!("A {what} needs a name of 1 to {NAME_MAX} characters."),
            )
            .with("max", NAME_MAX),
        ));
    }
    Ok(name.to_string())
}

pub fn collections(conn: &Connection) -> AppResult<Vec<Collection>> {
    let workspace = workspaces::get_default(conn)?;
    let mut stmt = conn.prepare(
        "SELECT c.id, c.parent_id, c.name,
                (SELECT COUNT(*) FROM source_collection_items i JOIN sources s ON s.id = i.source_id
                 WHERE i.collection_id = c.id AND s.deleted_at IS NULL AND s.state <> 'trashed')
         FROM source_collections c WHERE c.workspace_id = ?1
         ORDER BY c.name COLLATE NOCASE, c.id",
    )?;
    let rows = stmt.query_map([&workspace.id], |r| {
        Ok(Collection {
            id: r.get(0)?,
            parent_id: r.get(1)?,
            name: r.get(2)?,
            count: r.get(3)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

fn collection_exists(conn: &Connection, id: &str) -> AppResult<()> {
    conn.query_row(
        "SELECT 1 FROM source_collections WHERE id = ?1",
        [id],
        |_| Ok(()),
    )
    .optional()?
    .ok_or_else(|| AppError::not_found("collection", id))
}

fn same_name_in(
    conn: &Connection,
    parent: Option<&str>,
    name: &str,
    except: Option<&str>,
) -> AppResult<()> {
    let taken: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM source_collections
           WHERE parent_id IS ?1 AND name = ?2 COLLATE NOCASE AND id IS NOT ?3)",
        params![parent, name, except],
        |r| r.get(0),
    )?;
    if taken {
        return Err(AppError::Conflict(
            UserMessage::new(
                "library.collectionExists",
                format!("There is already a collection called “{name}” here."),
            )
            .with("name", name),
        ));
    }
    Ok(())
}

pub fn create_collection(
    conn: &Connection,
    name_: &str,
    parent: Option<&str>,
) -> AppResult<Collection> {
    let name = name(name_, "collection")?;
    if let Some(parent) = parent {
        collection_exists(conn, parent)?;
    }
    same_name_in(conn, parent, &name, None)?;
    let workspace = workspaces::get_default(conn)?;
    let id = new_id();
    let at = now();
    conn.execute(
        "INSERT INTO source_collections (id, workspace_id, parent_id, name, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
        params![id, workspace.id, parent, name, at],
    )?;
    Ok(Collection {
        id,
        parent_id: parent.map(str::to_string),
        name,
        count: 0,
    })
}

pub fn rename_collection(conn: &Connection, id: &str, name_: &str) -> AppResult<()> {
    let name = name(name_, "collection")?;
    collection_exists(conn, id)?;
    let parent: Option<String> = conn.query_row(
        "SELECT parent_id FROM source_collections WHERE id = ?1",
        [id],
        |r| r.get(0),
    )?;
    same_name_in(conn, parent.as_deref(), &name, Some(id))?;
    conn.execute(
        "UPDATE source_collections SET name = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, name, now()],
    )?;
    Ok(())
}

/// Moves a collection under another (or to the top). A collection cannot go
/// inside itself or one of its own.
pub fn move_collection(conn: &Connection, id: &str, parent: Option<&str>) -> AppResult<()> {
    collection_exists(conn, id)?;
    if let Some(target) = parent {
        collection_exists(conn, target)?;
        let mut at = Some(target.to_string());
        let mut steps = 0;
        while let Some(current) = at {
            if current == id {
                return Err(AppError::InvalidInput(UserMessage::new(
                    "library.collectionCycle",
                    "A collection cannot be moved into itself or into one of its own collections.",
                )));
            }
            steps += 1;
            if steps > 1_000 {
                break;
            }
            at = conn.query_row(
                "SELECT parent_id FROM source_collections WHERE id = ?1",
                [&current],
                |r| r.get(0),
            )?;
        }
    }
    let name: String = conn.query_row(
        "SELECT name FROM source_collections WHERE id = ?1",
        [id],
        |r| r.get(0),
    )?;
    same_name_in(conn, parent, &name, Some(id))?;
    conn.execute(
        "UPDATE source_collections SET parent_id = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, parent, now()],
    )?;
    Ok(())
}

/// Removes a collection and those inside it. The sources stay in the library.
pub fn remove_collection(conn: &Connection, id: &str) -> AppResult<()> {
    collection_exists(conn, id)?;
    conn.execute("DELETE FROM source_collections WHERE id = ?1", [id])?;
    Ok(())
}

fn live_sources(conn: &Connection, ids: &[String]) -> AppResult<()> {
    for id in ids {
        let live: bool = conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM sources WHERE id = ?1 AND deleted_at IS NULL)",
            [id],
            |r| r.get(0),
        )?;
        if !live {
            return Err(AppError::not_found("source", id));
        }
    }
    Ok(())
}

/// Puts sources in a collection (or takes them out). Their files are not
/// copied or moved.
pub fn collect(
    conn: &Connection,
    collection_id: &str,
    ids: &[String],
    add: bool,
) -> AppResult<usize> {
    check_batch(ids)?;
    collection_exists(conn, collection_id)?;
    let tx = conn.unchecked_transaction()?;
    live_sources(&tx, ids)?;
    let at = now();
    let mut changed = 0;
    for id in ids {
        changed += if add {
            tx.execute(
                "INSERT OR IGNORE INTO source_collection_items (collection_id, source_id, added_at)
                 VALUES (?1, ?2, ?3)",
                params![collection_id, id, at],
            )?
        } else {
            tx.execute(
                "DELETE FROM source_collection_items WHERE collection_id = ?1 AND source_id = ?2",
                params![collection_id, id],
            )?
        };
    }
    tx.commit()?;
    Ok(changed)
}

// --- tags ------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub count: i64,
}

pub fn tags(conn: &Connection) -> AppResult<Vec<Tag>> {
    let workspace = workspaces::get_default(conn)?;
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name,
                (SELECT COUNT(*) FROM source_tag_items i JOIN sources s ON s.id = i.source_id
                 WHERE i.tag_id = t.id AND s.deleted_at IS NULL AND s.state <> 'trashed')
         FROM source_tags t WHERE t.workspace_id = ?1 ORDER BY t.name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map([&workspace.id], |r| {
        Ok(Tag {
            id: r.get(0)?,
            name: r.get(1)?,
            count: r.get(2)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// The tag with this name (any case), created if new.
pub fn tag_named(conn: &Connection, name_: &str) -> AppResult<Tag> {
    let name = name(name_, "tag")?;
    let workspace = workspaces::get_default(conn)?;
    if let Some((id, existing)) = conn
        .query_row(
            "SELECT id, name FROM source_tags WHERE workspace_id = ?1 AND name = ?2 COLLATE NOCASE",
            params![workspace.id, name],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
        )
        .optional()?
    {
        return Ok(Tag {
            id,
            name: existing,
            count: 0,
        });
    }
    let id = new_id();
    conn.execute(
        "INSERT INTO source_tags (id, workspace_id, name, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![id, workspace.id, name, now()],
    )?;
    Ok(Tag { id, name, count: 0 })
}

pub fn rename_tag(conn: &Connection, id: &str, name_: &str) -> AppResult<()> {
    let name = name(name_, "tag")?;
    let workspace = workspaces::get_default(conn)?;
    let taken: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM source_tags WHERE workspace_id = ?1 AND name = ?2 COLLATE NOCASE AND id <> ?3)",
        params![workspace.id, name, id],
        |r| r.get(0),
    )?;
    if taken {
        return Err(AppError::Conflict(
            UserMessage::new(
                "library.tagExists",
                format!("There is already a tag “{name}”."),
            )
            .with("name", &name),
        ));
    }
    if conn.execute(
        "UPDATE source_tags SET name = ?2 WHERE id = ?1",
        params![id, name],
    )? == 0
    {
        return Err(AppError::not_found("tag", id));
    }
    Ok(())
}

/// Removes a tag from every source and deletes it. The sources stay.
pub fn remove_tag(conn: &Connection, id: &str) -> AppResult<()> {
    if conn.execute("DELETE FROM source_tags WHERE id = ?1", [id])? == 0 {
        return Err(AppError::not_found("tag", id));
    }
    Ok(())
}

/// Adds tags (by name, created if new) to sources and takes others (by id)
/// off them, in one transaction.
pub fn retag(
    conn: &Connection,
    ids: &[String],
    add: &[String],
    remove: &[String],
) -> AppResult<()> {
    check_batch(ids)?;
    let tx = conn.unchecked_transaction()?;
    live_sources(&tx, ids)?;
    let mut adding = Vec::new();
    for name in add {
        adding.push(tag_named(&tx, name)?.id);
    }
    for id in ids {
        for tag in &adding {
            tx.execute(
                "INSERT OR IGNORE INTO source_tag_items (tag_id, source_id) VALUES (?1, ?2)",
                params![tag, id],
            )?;
        }
        for tag in remove {
            tx.execute(
                "DELETE FROM source_tag_items WHERE tag_id = ?1 AND source_id = ?2",
                params![tag, id],
            )?;
        }
    }
    tx.commit()?;
    Ok(())
}

/// The short names of every Bible kept (in use, archived or in Trash).
pub fn bible_short_names(conn: &Connection) -> AppResult<Vec<String>> {
    Ok(conn
        .prepare("SELECT short_name FROM bible_translations")?
        .query_map([], |r| r.get(0))?
        .collect::<Result<_, _>>()?)
}
