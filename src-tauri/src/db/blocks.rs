use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::db::{new_id, now, sections, studies};
use crate::domain::{validate, CreateStudyBlock, StudyBlock, UpdateStudyBlock};
use crate::error::{AppError, AppResult};

const COLUMNS: &str =
    "b.id, b.section_id, b.block_type, b.position, b.content, b.field_key, b.data, \
     b.created_at, b.updated_at, b.revision";

fn from_row(row: &Row) -> rusqlite::Result<StudyBlock> {
    let block_type: String = row.get(2)?;
    Ok(StudyBlock {
        id: row.get(0)?,
        section_id: row.get(1)?,
        block_type: block_type.parse().map_err(|e: AppError| {
            rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, e.into())
        })?,
        position: row.get(3)?,
        content: row.get(4)?,
        field_key: row.get(5)?,
        data: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
        revision: row.get(9)?,
    })
}

pub fn list_for_section(conn: &Connection, section_id: &str) -> AppResult<Vec<StudyBlock>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM study_blocks b
         WHERE b.section_id = ?1 AND b.deleted_at IS NULL
         ORDER BY b.position, b.created_at"
    ))?;
    let rows = stmt.query_map([section_id], from_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// All live blocks of a study's live sections, in section then block order.
pub fn list_for_study(conn: &Connection, study_id: &str) -> AppResult<Vec<StudyBlock>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM study_blocks b
         JOIN study_sections s ON s.id = b.section_id
         WHERE s.study_id = ?1 AND s.deleted_at IS NULL AND b.deleted_at IS NULL
         ORDER BY s.position, b.position, b.created_at"
    ))?;
    let rows = stmt.query_map([study_id], from_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<StudyBlock> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM study_blocks b WHERE b.id = ?1 AND b.deleted_at IS NULL"),
        [id],
        from_row,
    )
    .optional()?
    .ok_or_else(|| AppError::not_found("block", id))
}

pub(crate) fn insert(
    conn: &Connection,
    section_id: &str,
    block_type: crate::domain::BlockType,
    position: i64,
    content: &str,
    now: &str,
) -> AppResult<String> {
    insert_answer(
        conn, section_id, block_type, position, content, None, None, now,
    )
}

/// Inserts a block that may answer a field of its section's snapshot.
#[allow(clippy::too_many_arguments)]
pub(crate) fn insert_answer(
    conn: &Connection,
    section_id: &str,
    block_type: crate::domain::BlockType,
    position: i64,
    content: &str,
    field_key: Option<&str>,
    data: Option<&str>,
    now: &str,
) -> AppResult<String> {
    let id = new_id();
    conn.execute(
        "INSERT INTO study_blocks
             (id, section_id, block_type, position, content, field_key, data, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
        params![
            id,
            section_id,
            block_type.as_str(),
            position,
            content,
            field_key,
            data,
            now
        ],
    )?;
    Ok(id)
}

/// A block's entry values must be a JSON object of text values.
fn validated_data(data: &str) -> AppResult<()> {
    validate::block_content(data)?;
    match serde_json::from_str::<serde_json::Value>(data) {
        Ok(serde_json::Value::Object(map)) if map.values().all(|v| v.is_string()) => Ok(()),
        _ => Err(AppError::InvalidInput(crate::error::UserMessage::new(
            "validation.entryInvalid",
            "This entry could not be saved.",
        ))),
    }
}

/// Appends a block to the end of a section.
pub fn create(conn: &Connection, input: &CreateStudyBlock) -> AppResult<StudyBlock> {
    let content = input.content.as_deref().unwrap_or("");
    validate::block_content(content)?;
    let section = sections::get(conn, &input.section_id)?;
    let field_key = input.field_key.as_deref().filter(|k| !k.is_empty());
    if let Some(key) = field_key {
        let known = section
            .fields
            .as_ref()
            .is_some_and(|f| f.fields.iter().any(|field| field.key == key));
        if !known {
            return Err(AppError::InvalidInput(crate::error::UserMessage::new(
                "validation.unknownField",
                "This part has no such field.",
            )));
        }
    }
    let position: i64 = conn.query_row(
        "SELECT COALESCE(MAX(position), -1) + 1 FROM study_blocks
         WHERE section_id = ?1 AND deleted_at IS NULL",
        [&section.id],
        |row| row.get(0),
    )?;
    let now = now();
    let id = insert_answer(
        conn,
        &section.id,
        input.block_type,
        position,
        content,
        field_key,
        None,
        &now,
    )?;
    studies::touch(conn, &section.study_id, &now)?;
    get(conn, &id)
}

pub fn update(conn: &Connection, input: &UpdateStudyBlock) -> AppResult<StudyBlock> {
    let current = get(conn, &input.id)?;
    if input.content.is_none() && input.data.is_none() {
        return Ok(current);
    }
    let content = input.content.as_ref().unwrap_or(&current.content);
    validate::block_content(content)?;
    if let Some(data) = &input.data {
        if current.field_key.is_none() {
            return Err(AppError::InvalidInput(crate::error::UserMessage::new(
                "validation.entryInvalid",
                "This entry could not be saved.",
            )));
        }
        validated_data(data)?;
    }
    let data = input.data.as_ref().or(current.data.as_ref());
    let now = now();
    conn.execute(
        "UPDATE study_blocks SET content = ?2, data = ?3, updated_at = ?4, revision = revision + 1
         WHERE id = ?1",
        params![input.id, content, data, now],
    )?;
    touch_study_of_section(conn, &current.section_id, &now)?;
    get(conn, &input.id)
}

/// Soft delete: the block disappears from the study but stays in the file.
pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    let current = get(conn, id)?;
    let now = now();
    conn.execute(
        "UPDATE study_blocks SET deleted_at = ?2, updated_at = ?2, revision = revision + 1
         WHERE id = ?1",
        params![id, now],
    )?;
    touch_study_of_section(conn, &current.section_id, &now)
}

/// Puts a section's blocks in the given order. The list must name exactly
/// the section's live blocks.
pub fn reorder(conn: &Connection, input: &crate::domain::Reorder) -> AppResult<Vec<StudyBlock>> {
    let section = sections::get(conn, &input.parent_id)?;
    let current: Vec<String> = section.blocks.iter().map(|b| b.id.clone()).collect();
    crate::db::check_same_members(&current, &input.ids, "blocks")?;
    let now = now();
    let tx = conn.unchecked_transaction()?;
    for (index, id) in input.ids.iter().enumerate() {
        tx.execute(
            "UPDATE study_blocks SET position = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, index as i64, now],
        )?;
    }
    studies::touch(&tx, &section.study_id, &now)?;
    tx.commit()?;
    list_for_section(conn, &input.parent_id)
}

fn touch_study_of_section(conn: &Connection, section_id: &str, now: &str) -> AppResult<()> {
    let study_id: String = conn.query_row(
        "SELECT study_id FROM study_sections WHERE id = ?1",
        [section_id],
        |row| row.get(0),
    )?;
    studies::touch(conn, &study_id, now)
}
