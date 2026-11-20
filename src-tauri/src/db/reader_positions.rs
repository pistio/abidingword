//! Durable per-source reading continuity in the existing SQLite settings table.
//! Only navigation coordinates are stored, never the source's text or user notes.
//! No schema migration: backed up and restored with the existing settings table.
use std::collections::HashSet;

use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::db::settings;
use crate::error::{AppError, AppResult, UserMessage};

const KEY: &str = "library.source_reader_positions.v1";
const MAX_SOURCES: usize = 60;
const MAX_HISTORY: usize = 200;
const MAX_JSON_BYTES: usize = 80_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReaderLocation {
    pub page: Option<u32>,
    #[serde(default)]
    pub heading_id: Option<String>,
    pub fraction: f64,
    pub position: Option<u32>,
    pub offset: f64,
    pub item: Option<String>,
    pub fragment: Option<String>,
    pub item_offset: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReaderHistory {
    pub entries: Vec<ReaderLocation>,
    pub index: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReaderPosition {
    pub mode: String,
    pub page: Option<u32>,
    #[serde(default)]
    pub history: Option<ReaderHistory>,
    #[serde(default)]
    pub place: Option<ReaderLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SavedReaderPosition {
    pub source_id: String,
    pub position: ReaderPosition,
}

fn invalid() -> AppError {
    AppError::InvalidInput(UserMessage::new(
        "library.reader.invalidSavedPlace",
        "This reading place could not be saved. The source was not changed.",
    ))
}

fn valid_location(place: &ReaderLocation) -> bool {
    let bounded = |number: f64, limit: f64| number.is_finite() && number.abs() <= limit;
    place.page.is_none_or(|n| (1..=200_000).contains(&n))
        && place.position.is_none_or(|n| n <= 100_000_000)
        && bounded(place.fraction, 1.0)
        && place.fraction >= 0.0
        && bounded(place.offset, 10_000_000.0)
        && bounded(place.item_offset, 10_000_000.0)
        && place.heading_id.as_ref().is_none_or(|s| s.len() <= 500)
        && place.item.as_ref().is_none_or(|s| s.len() <= 500)
        && place.fragment.as_ref().is_none_or(|s| s.len() <= 500)
}

fn valid(position: &ReaderPosition) -> bool {
    if !matches!(position.mode.as_str(), "reading" | "original")
        || !position.page.is_none_or(|n| (1..=200_000).contains(&n))
        || !position.place.as_ref().is_none_or(valid_location)
    {
        return false;
    }
    position.history.as_ref().is_none_or(|history| {
        history.entries.len() <= MAX_HISTORY
            && if history.entries.is_empty() {
                history.index == -1
            } else {
                history.index >= 0 && (history.index as usize) < history.entries.len()
            }
            && history.entries.iter().all(valid_location)
    })
}

fn source_exists(conn: &Connection, source_id: &str) -> AppResult<bool> {
    let exists: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM sources WHERE id=?1 AND deleted_at IS NULL AND state != 'trashed'",
            [source_id],
            |row| row.get(0),
        )
        .optional()?;
    Ok(exists.is_some())
}

/// Newest first. Bad or stale entries cannot prevent the app starting.
pub fn list(conn: &Connection) -> AppResult<Vec<SavedReaderPosition>> {
    let stored: Vec<SavedReaderPosition> = settings::read(conn, KEY)?.unwrap_or_default();
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for row in stored {
        if result.len() == MAX_SOURCES {
            break;
        }
        if row.source_id.len() > 128
            || row.source_id.is_empty()
            || !seen.insert(row.source_id.clone())
            || !valid(&row.position)
            || !source_exists(conn, &row.source_id)?
        {
            continue;
        }
        result.push(row);
    }
    Ok(result)
}

/// Atomic read/replace under the database mutex; one source update cannot
/// overwrite another source's reading place. The latest 60 are retained.
pub fn save(conn: &Connection, source_id: &str, position: &ReaderPosition) -> AppResult<()> {
    if source_id.is_empty() || source_id.len() > 128 || !valid(position) {
        return Err(invalid());
    }
    if !source_exists(conn, source_id)? {
        return Err(AppError::not_found("source", source_id));
    }
    let mut entries = list(conn)?;
    entries.retain(|row| row.source_id != source_id);
    entries.insert(
        0,
        SavedReaderPosition {
            source_id: source_id.to_string(),
            position: position.clone(),
        },
    );
    entries.truncate(MAX_SOURCES);
    let encoded = serde_json::to_vec(&entries).map_err(|e| AppError::Unexpected(e.to_string()))?;
    if encoded.len() > MAX_JSON_BYTES {
        return Err(invalid());
    }
    settings::write(conn, KEY, &entries)
}

/// Called inside the same transaction as permanent source removal; a
/// tombstone remains for citations, but no obsolete reading state remains.
pub fn forget(conn: &Connection, source_id: &str) -> AppResult<()> {
    let mut entries: Vec<SavedReaderPosition> = settings::read(conn, KEY)?.unwrap_or_default();
    entries.retain(|row| row.source_id != source_id);
    settings::write(conn, KEY, &entries)
}
