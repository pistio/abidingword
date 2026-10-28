//! Ordered blocks of a teaching (migration 0025).
//!
//! A teaching made from a block template stores its writing here. A teaching
//! with no blocks is left exactly as it was: its columns and points stay the
//! source of what the earlier editor, the presentation and the materials read.
//! A block with a role also writes that column, so a presentation that names
//! `introduction` or a point still follows the person's text.

use rusqlite::{params, Connection, OptionalExtension};

use crate::db::teaching::{self, point_title};
use crate::db::{new_id, now};
use crate::domain::validate::{self, BLOCK_MAX_CHARS};
use crate::domain::{
    block_is_private_by_default, CreateTeachingBlock, NewTeachingBlock, ReorderTeachingBlocks,
    TeachingBlock, TeachingPreparation, UpdateTeachingBlock, BLOCK_ROLES, TEACHING_BLOCK_TYPES,
};
use crate::error::{AppError, AppResult, UserMessage};

/// How many blocks one teaching may hold. Enough for a custom template,
/// bounded so a damaged request cannot fill the database.
pub const BLOCKS_MAX: usize = 80;

#[derive(serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct BlockConfig {
    #[serde(default)]
    participant_facing: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    guidance_key: Option<String>,
}

struct StoredBlock {
    id: String,
    preparation_id: String,
    parent_id: Option<String>,
    position: i64,
    block_type: String,
    block_key: Option<String>,
    title: String,
    body: String,
    config: BlockConfig,
    point_id: Option<String>,
}

fn invalid(code: &'static str, text: &str) -> AppError {
    AppError::InvalidInput(UserMessage::new(code, text))
}

fn check_type(block_type: &str) -> AppResult<String> {
    let block_type = block_type.trim();
    if TEACHING_BLOCK_TYPES.contains(&block_type) {
        Ok(block_type.to_string())
    } else {
        Err(invalid(
            "teaching.blockType",
            "That is not a kind of block this teaching can hold.",
        ))
    }
}

fn check_role(role: Option<&str>) -> AppResult<Option<String>> {
    match role.map(str::trim).filter(|r| !r.is_empty()) {
        None => Ok(None),
        Some(role) if BLOCK_ROLES.contains(&role) => Ok(Some(role.to_string())),
        Some(_) => Err(invalid(
            "teaching.blockRole",
            "That block cannot be tied to that part of the teaching.",
        )),
    }
}

fn guidance_key(value: Option<&str>) -> AppResult<Option<String>> {
    match value.map(str::trim).filter(|k| !k.is_empty()) {
        None => Ok(None),
        Some(key)
            if key.chars().count() <= 200
                && key
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_') =>
        {
            Ok(Some(key.to_string()))
        }
        Some(_) => Err(invalid(
            "teaching.blockGuidance",
            "That guidance cannot be stored.",
        )),
    }
}

fn body_text(value: &str) -> AppResult<String> {
    if value.chars().count() > BLOCK_MAX_CHARS {
        return Err(invalid(
            "teaching.tooLong",
            "This part of the teaching is too long to save.",
        ));
    }
    Ok(value.to_string())
}

fn config_json(config: &BlockConfig) -> AppResult<String> {
    serde_json::to_string(config)
        .map_err(|e| AppError::Unexpected(format!("teaching block JSON: {e}")))
}

fn parse_config(raw: Option<String>) -> BlockConfig {
    raw.and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn facing(block_type: &str, chosen: Option<bool>) -> bool {
    chosen.unwrap_or(!block_is_private_by_default(block_type))
}

pub(crate) fn check_template_block(
    block: &crate::domain::TeachingTemplateBlock,
) -> AppResult<crate::domain::TeachingTemplateBlock> {
    let block_type = check_type(&block.block_type)?;
    Ok(crate::domain::TeachingTemplateBlock {
        key: validate::part_key(block.key.as_deref())?,
        parent_key: validate::part_key(block.parent_key.as_deref())?,
        block_type,
        title: point_title(&block.title)?,
        participant_facing: block.participant_facing,
        role: check_role(block.role.as_deref())?,
        guidance_key: guidance_key(block.guidance_key.as_deref())?,
    })
}

pub fn list(conn: &Connection, preparation_id: &str) -> AppResult<Vec<TeachingBlock>> {
    let mut stmt = conn.prepare(
        "SELECT id, parent_id, position, block_type, block_key, title, body, config, point_id
         FROM teaching_blocks
         WHERE preparation_id = ?1 AND deleted_at IS NULL
         ORDER BY position, created_at, rowid",
    )?;
    let rows = stmt
        .query_map([preparation_id], |row| {
            Ok(StoredBlock {
                id: row.get(0)?,
                preparation_id: preparation_id.to_string(),
                parent_id: row.get(1)?,
                position: row.get(2)?,
                block_type: row.get(3)?,
                block_key: row.get(4)?,
                title: row.get(5)?,
                body: row.get(6)?,
                config: parse_config(row.get(7)?),
                point_id: row.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows.into_iter().map(into_block).collect())
}

fn into_block(stored: StoredBlock) -> TeachingBlock {
    TeachingBlock {
        id: stored.id,
        parent_id: stored.parent_id,
        position: stored.position,
        block_type: stored.block_type,
        block_key: stored.block_key,
        title: stored.title,
        body: stored.body,
        participant_facing: stored.config.participant_facing,
        role: stored.config.role,
        guidance_key: stored.config.guidance_key,
        point_id: stored.point_id,
    }
}

fn load(conn: &Connection, id: &str) -> AppResult<StoredBlock> {
    conn.query_row(
        "SELECT id, preparation_id, parent_id, position, block_type, block_key, title, body, config, point_id
         FROM teaching_blocks WHERE id = ?1 AND deleted_at IS NULL",
        [id],
        |row| {
            Ok(StoredBlock {
                id: row.get(0)?,
                preparation_id: row.get(1)?,
                parent_id: row.get(2)?,
                position: row.get(3)?,
                block_type: row.get(4)?,
                block_key: row.get(5)?,
                title: row.get(6)?,
                body: row.get(7)?,
                config: parse_config(row.get(8)?),
                point_id: row.get(9)?,
            })
        },
    )
    .optional()?
    .ok_or_else(|| AppError::not_found("teaching block", id))
}

fn count(conn: &Connection, preparation_id: &str) -> AppResult<usize> {
    let n: i64 = conn.query_row(
        "SELECT COUNT(*) FROM teaching_blocks WHERE preparation_id = ?1 AND deleted_at IS NULL",
        [preparation_id],
        |r| r.get(0),
    )?;
    Ok(n as usize)
}

/// Copies a template's blocks into a new teaching and writes any role into
/// the column a presentation already reads. Outline points become real points.
pub fn insert_starting(
    conn: &Connection,
    preparation_id: &str,
    blocks: &[NewTeachingBlock],
    now: &str,
) -> AppResult<()> {
    if blocks.len() > BLOCKS_MAX {
        return Err(invalid(
            "teaching.tooManyBlocks",
            "This teaching has too many blocks.",
        ));
    }
    let mut ids_by_key: Vec<(String, String)> = Vec::new();
    let mut pending: Vec<(String, NewTeachingBlock, BlockConfig, Option<String>)> = Vec::new();
    for (index, block) in blocks.iter().enumerate() {
        let block_type = check_type(&block.block_type)?;
        let title = point_title(&block.title)?;
        let body = body_text(&block.body)?;
        let role = check_role(block.role.as_deref())?;
        let guidance = guidance_key(block.guidance_key.as_deref())?;
        let key = validate::part_key(block.key.as_deref())?;
        if let Some(key) = &key {
            if ids_by_key.iter().any(|(k, _)| k == key) {
                return Err(invalid(
                    "teaching.duplicateBlockKey",
                    "Two blocks in this template share a key.",
                ));
            }
        }
        let id = new_id();
        if let Some(key) = &key {
            ids_by_key.push((key.clone(), id.clone()));
        }
        let config = BlockConfig {
            participant_facing: facing(&block_type, block.participant_facing),
            role,
            guidance_key: guidance,
        };
        let point_id = if block_type == "outline_point" {
            Some(insert_point(
                conn,
                preparation_id,
                index as i64,
                &title,
                &body,
                now,
            )?)
        } else {
            None
        };
        pending.push((
            id,
            NewTeachingBlock {
                key,
                parent_key: validate::part_key(block.parent_key.as_deref())?,
                block_type,
                title,
                body,
                ..NewTeachingBlock::default()
            },
            config,
            point_id,
        ));
    }
    for (index, (id, block, config, point_id)) in pending.iter().enumerate() {
        let parent_id = match &block.parent_key {
            Some(key) => Some(
                ids_by_key
                    .iter()
                    .find(|(k, _)| k == key)
                    .map(|(_, id)| id.clone())
                    .ok_or_else(|| {
                        invalid(
                            "teaching.blockParent",
                            "That block names a section it does not have.",
                        )
                    })?,
            ),
            None => None,
        };
        conn.execute(
            "INSERT INTO teaching_blocks
                (id, preparation_id, parent_id, position, block_type, block_key, title, body, config, point_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)",
            params![
                id,
                preparation_id,
                parent_id,
                index as i64,
                block.block_type,
                block.key,
                block.title,
                block.body,
                config_json(config)?,
                point_id,
                now,
            ],
        )?;
        mirror_role(
            conn,
            preparation_id,
            point_id.as_deref(),
            config.role.as_deref(),
            &block.body,
            &block.title,
        )?;
    }
    Ok(())
}

fn insert_point(
    conn: &Connection,
    preparation_id: &str,
    position: i64,
    title: &str,
    explanation: &str,
    now: &str,
) -> AppResult<String> {
    let id = new_id();
    conn.execute(
        "INSERT INTO teaching_points (id, preparation_id, position, title, explanation, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
        params![id, preparation_id, position, title, explanation, now],
    )?;
    Ok(id)
}

fn mirror_role(
    conn: &Connection,
    preparation_id: &str,
    point_id: Option<&str>,
    role: Option<&str>,
    body: &str,
    title: &str,
) -> AppResult<()> {
    match role {
        Some("introduction") => set_column(conn, preparation_id, "introduction", body),
        Some("conclusion") => set_column(conn, preparation_id, "conclusion", body),
        Some("central_message") => set_column(conn, preparation_id, "central_message", body),
        Some("notes") => set_column(conn, preparation_id, "notes", body),
        Some("explanation") => set_point_field(conn, point_id, "explanation", body),
        Some("application") => set_point_field(conn, point_id, "application", body),
        Some("illustration") => set_point_field(conn, point_id, "illustration", body),
        _ => Ok(()),
    }?;
    if let Some(point_id) = point_id {
        conn.execute(
            "UPDATE teaching_points SET title = ?2, explanation = ?3, updated_at = ?4, revision = revision + 1
             WHERE id = ?1 AND deleted_at IS NULL",
            params![point_id, title, body, now()],
        )?;
    }
    Ok(())
}

fn set_column(conn: &Connection, preparation_id: &str, column: &str, value: &str) -> AppResult<()> {
    // Column names are fixed roles, never request text.
    let sql = format!(
        "UPDATE teaching_preparations SET {column} = ?2, updated_at = ?3, revision = revision + 1 WHERE id = ?1"
    );
    conn.execute(&sql, params![preparation_id, value, now()])?;
    Ok(())
}

fn set_point_field(
    conn: &Connection,
    point_id: Option<&str>,
    column: &str,
    value: &str,
) -> AppResult<()> {
    let Some(point_id) = point_id else {
        return Ok(());
    };
    let sql = format!(
        "UPDATE teaching_points SET {column} = ?2, updated_at = ?3, revision = revision + 1 WHERE id = ?1"
    );
    conn.execute(&sql, params![point_id, value, now()])?;
    Ok(())
}

/// When the earlier columns are saved, a block that wears that role follows,
/// so the two cannot drift if both are written.
pub fn mirror_column_into_blocks(
    conn: &Connection,
    preparation_id: &str,
    role: &str,
    value: &str,
) -> AppResult<()> {
    let blocks = list(conn, preparation_id)?;
    let now = now();
    for block in blocks
        .into_iter()
        .filter(|b| b.role.as_deref() == Some(role))
    {
        conn.execute(
            "UPDATE teaching_blocks SET body = ?2, updated_at = ?3, revision = revision + 1 WHERE id = ?1",
            params![block.id, value, now],
        )?;
    }
    Ok(())
}

fn siblings(
    conn: &Connection,
    preparation_id: &str,
    parent_id: Option<&str>,
) -> AppResult<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT id FROM teaching_blocks
         WHERE preparation_id = ?1 AND deleted_at IS NULL
           AND ((?2 IS NULL AND parent_id IS NULL) OR parent_id = ?2)
         ORDER BY position, created_at, rowid",
    )?;
    let ids = stmt
        .query_map(params![preparation_id, parent_id], |r| r.get(0))?
        .collect::<Result<Vec<String>, _>>()?;
    Ok(ids)
}

fn renumber(conn: &Connection, ids: &[String], now: &str) -> AppResult<()> {
    for (index, id) in ids.iter().enumerate() {
        conn.execute(
            "UPDATE teaching_blocks SET position = ?2, updated_at = ?3, revision = revision + 1 WHERE id = ?1",
            params![id, index as i64, now],
        )?;
    }
    Ok(())
}

pub fn create(conn: &Connection, input: &CreateTeachingBlock) -> AppResult<TeachingPreparation> {
    let prep = teaching::row(conn, &input.preparation_id)?;
    if count(conn, &prep.id)? >= BLOCKS_MAX {
        return Err(invalid(
            "teaching.tooManyBlocks",
            "This teaching has too many blocks.",
        ));
    }
    let block_type = check_type(&input.block_type)?;
    let title = point_title(&input.title)?;
    let body = body_text(&input.body)?;
    let role = check_role(input.role.as_deref())?;
    let guidance = guidance_key(input.guidance_key.as_deref())?;
    let parent_id = input.parent_id.as_deref().filter(|p| !p.is_empty());
    if let Some(parent) = parent_id {
        let parent = load(conn, parent)?;
        if parent.preparation_id != prep.id || parent.block_type != "section" {
            return Err(invalid(
                "teaching.blockParent",
                "A block can sit in a section of this teaching.",
            ));
        }
    }
    let siblings = siblings(conn, &prep.id, parent_id)?;
    let at = match input.after_block_id.as_deref().filter(|id| !id.is_empty()) {
        Some(after) => siblings
            .iter()
            .position(|id| id == after)
            .map(|i| i + 1)
            .unwrap_or(siblings.len()),
        None => siblings.len(),
    };
    let mut order = siblings;
    let id = new_id();
    order.insert(at, id.clone());
    let config = BlockConfig {
        participant_facing: facing(&block_type, input.participant_facing),
        role,
        guidance_key: guidance,
    };
    let stamp = now();
    let tx = conn.unchecked_transaction()?;
    let point_id = if block_type == "outline_point" {
        Some(insert_point(
            &tx, &prep.id, at as i64, &title, &body, &stamp,
        )?)
    } else {
        None
    };
    tx.execute(
        "INSERT INTO teaching_blocks
            (id, preparation_id, parent_id, position, block_type, title, body, config, point_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
        params![id, prep.id, parent_id, at as i64, block_type, title, body, config_json(&config)?, point_id, stamp],
    )?;
    renumber(&tx, &order, &stamp)?;
    mirror_role(
        &tx,
        &prep.id,
        point_id.as_deref(),
        config.role.as_deref(),
        &body,
        &title,
    )?;
    // A block teaching is edited as blocks even when it was saved without a version.
    tx.execute(
        "UPDATE teaching_preparations SET template_version = COALESCE(template_version, 2), updated_at = ?2, revision = revision + 1 WHERE id = ?1",
        params![prep.id, stamp],
    )?;
    teaching::touch(&tx, &prep, &stamp)?;
    tx.commit()?;
    teaching::get(conn, &prep.id)
}

pub fn update(conn: &Connection, input: &UpdateTeachingBlock) -> AppResult<TeachingPreparation> {
    let current = load(conn, &input.id)?;
    let prep = teaching::row(conn, &current.preparation_id)?;
    let block_type = match &input.block_type {
        Some(t) => check_type(t)?,
        None => current.block_type,
    };
    let title = match &input.title {
        Some(t) => point_title(t)?,
        None => current.title,
    };
    let body = match &input.body {
        Some(b) => body_text(b)?,
        None => current.body,
    };
    let role = match &input.role {
        Some(r) => check_role(Some(r))?,
        None => current.config.role,
    };
    let guidance = match &input.guidance_key {
        Some(k) => guidance_key(Some(k))?,
        None => current.config.guidance_key,
    };
    let participant_facing = input
        .participant_facing
        .unwrap_or(current.config.participant_facing);
    let previous_parent = current.parent_id.clone();
    let parent_id = match &input.parent_id {
        Some(p) if p.is_empty() => None,
        Some(p) => {
            let parent = load(conn, p)?;
            if parent.preparation_id != prep.id
                || parent.id == current.id
                || parent.block_type != "section"
            {
                return Err(invalid(
                    "teaching.blockParent",
                    "A block can sit in a section of this teaching.",
                ));
            }
            Some(parent.id)
        }
        None => previous_parent.clone(),
    };
    let config = BlockConfig {
        participant_facing,
        role,
        guidance_key: guidance,
    };
    let stamp = now();
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE teaching_blocks
         SET parent_id = ?2, block_type = ?3, title = ?4, body = ?5, config = ?6, updated_at = ?7, revision = revision + 1
         WHERE id = ?1",
        params![current.id, parent_id, block_type, title, body, config_json(&config)?, stamp],
    )?;
    mirror_role(
        &tx,
        &prep.id,
        current.point_id.as_deref(),
        config.role.as_deref(),
        &body,
        &title,
    )?;
    if parent_id != previous_parent {
        let old = siblings(&tx, &prep.id, previous_parent.as_deref())?
            .into_iter()
            .filter(|id| id != &current.id)
            .collect::<Vec<_>>();
        let mut new_siblings = siblings(&tx, &prep.id, parent_id.as_deref())?;
        if !new_siblings.iter().any(|id| id == &current.id) {
            new_siblings.push(current.id.clone());
        }
        renumber(&tx, &old, &stamp)?;
        renumber(&tx, &new_siblings, &stamp)?;
    }
    teaching::touch(&tx, &prep, &stamp)?;
    tx.commit()?;
    teaching::get(conn, &prep.id)
}

pub fn delete(conn: &Connection, id: &str) -> AppResult<TeachingPreparation> {
    let current = load(conn, id)?;
    let prep = teaching::row(conn, &current.preparation_id)?;
    let stamp = now();
    let tx = conn.unchecked_transaction()?;
    // A section goes away; what was written under it stays, one level up.
    tx.execute(
        "UPDATE teaching_blocks SET parent_id = ?2, updated_at = ?3, revision = revision + 1
         WHERE parent_id = ?1 AND deleted_at IS NULL",
        params![current.id, current.parent_id, stamp],
    )?;
    tx.execute(
        "UPDATE teaching_blocks SET deleted_at = ?2, updated_at = ?2, revision = revision + 1 WHERE id = ?1",
        params![current.id, stamp],
    )?;
    if let Some(point_id) = &current.point_id {
        tx.execute(
            "UPDATE teaching_points SET deleted_at = ?2, updated_at = ?2, revision = revision + 1 WHERE id = ?1",
            params![point_id, stamp],
        )?;
    }
    let left = siblings(&tx, &prep.id, current.parent_id.as_deref())?;
    renumber(&tx, &left, &stamp)?;
    teaching::touch(&tx, &prep, &stamp)?;
    tx.commit()?;
    teaching::get(conn, &prep.id)
}

pub fn reorder(conn: &Connection, input: &ReorderTeachingBlocks) -> AppResult<TeachingPreparation> {
    let prep = teaching::row(conn, &input.preparation_id)?;
    let parent = input.parent_id.as_deref().filter(|p| !p.is_empty());
    let current = siblings(conn, &prep.id, parent)?;
    crate::db::check_same_members(&current, &input.ids, "blocks")?;
    let stamp = now();
    let tx = conn.unchecked_transaction()?;
    renumber(&tx, &input.ids, &stamp)?;
    teaching::touch(&tx, &prep, &stamp)?;
    tx.commit()?;
    teaching::get(conn, &prep.id)
}

pub fn duplicate(conn: &Connection, id: &str) -> AppResult<TeachingPreparation> {
    let current = load(conn, id)?;
    create(
        conn,
        &CreateTeachingBlock {
            preparation_id: current.preparation_id,
            parent_id: current.parent_id,
            after_block_id: Some(current.id),
            block_type: current.block_type,
            title: current.title,
            body: current.body,
            participant_facing: Some(current.config.participant_facing),
            role: current.config.role,
            guidance_key: current.config.guidance_key,
        },
    )
}
