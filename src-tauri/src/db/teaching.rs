//! Project-owned Teaching with optional Study links. Findings stay live while
//! linked; unlinking keeps snapshots with exact citation provenance. Scripture
//! is named by canonical reference, without changing the Study passages.

use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::db::study_passages::{columns_for, reference_from_columns};
use crate::db::{findings, new_id, now, projects, studies};
use crate::domain::validate::{self, BLOCK_MAX_CHARS, DESCRIPTION_MAX_CHARS};
use crate::domain::{
    AddTeachingPassage, CreateTeaching, CreateTeachingPoint, ExternalSource, ImageInfo,
    KeepTeachingMaterial, LinkTeachingFinding, Reorder, TeachingFindingLink, TeachingMaterial,
    TeachingPassage, TeachingPoint, TeachingPreparation, TeachingSummary, UpdateTeaching,
    UpdateTeachingPoint, MATERIAL_KINDS, MATERIAL_STANDINGS, STEP_KINDS, TEACHING_KINDS,
};
use crate::error::{AppError, AppResult, UserMessage};
use crate::scripture::reference::ScriptureRef;

const COLUMNS: &str = "id, study_id, title, kind, audience, purpose, central_message, \
                       introduction, conclusion, notes, created_at, updated_at, revision, \
                       template_key, template_version, project_id";

/// How many sources a kept item may carry, and how long each part may be.
const SOURCES_MAX: usize = 10;
const SOURCE_TEXT_MAX: usize = 2_000;

fn from_row(row: &Row) -> rusqlite::Result<TeachingPreparation> {
    Ok(TeachingPreparation {
        id: row.get(0)?,
        project_id: row.get(15)?,
        study_id: row.get(1)?,
        title: row.get(2)?,
        kind: row.get(3)?,
        audience: row.get(4)?,
        purpose: row.get(5)?,
        central_message: row.get(6)?,
        introduction: row.get(7)?,
        conclusion: row.get(8)?,
        notes: row.get(9)?,
        points: Vec::new(),
        findings: Vec::new(),
        passages: Vec::new(),
        materials: Vec::new(),
        template_key: row.get(13)?,
        template_version: row.get(14)?,
        blocks: Vec::new(),
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
        revision: row.get(12)?,
    })
}

fn invalid(code: &'static str, text: &str) -> AppError {
    AppError::InvalidInput(UserMessage::new(code, text))
}

pub(crate) fn check_kind(kind: Option<&str>) -> AppResult<Option<String>> {
    match kind.map(str::trim).filter(|k| !k.is_empty()) {
        None => Ok(None),
        Some(k) if TEACHING_KINDS.contains(&k) => Ok(Some(k.to_string())),
        Some(other) => Err(AppError::InvalidInput(
            UserMessage::new(
                "invalidChoice",
                format!("“{other}” is not a valid teaching kind."),
            )
            .with("value", other),
        )),
    }
}

pub(crate) fn check_step_kind(kind: Option<&str>) -> AppResult<Option<String>> {
    match kind.map(str::trim).filter(|k| !k.is_empty()) {
        None => Ok(None),
        Some(k) if STEP_KINDS.contains(&k) => Ok(Some(k.to_string())),
        Some(other) => Err(AppError::InvalidInput(
            UserMessage::new(
                "invalidChoice",
                format!("“{other}” is not a valid kind of step."),
            )
            .with("value", other),
        )),
    }
}

pub(crate) fn point_title(title: &str) -> AppResult<String> {
    let title = title.trim();
    if title.chars().count() > validate::TITLE_MAX_CHARS {
        return Err(invalid(
            "teaching.tooLong",
            "This point's title is too long.",
        ));
    }
    Ok(title.to_string())
}

/// A format starts with at most this many steps.
pub(crate) const STARTING_POINTS_MAX: usize = 20;

fn long_text(value: &str) -> AppResult<String> {
    if value.chars().count() > BLOCK_MAX_CHARS {
        return Err(invalid(
            "teaching.tooLong",
            "This part of the teaching is too long to save.",
        ));
    }
    Ok(value.to_string())
}

// ---------------------------------------------------------------------------
// Preparations
// ---------------------------------------------------------------------------

/// A study's preparations, in the order they were made.
pub fn list(conn: &Connection, study_id: &str) -> AppResult<Vec<TeachingSummary>> {
    studies::get(conn, study_id)?;
    let mut stmt = conn.prepare(
        "SELECT id, study_id, title, kind, updated_at, project_id FROM teaching_preparations
         WHERE study_id = ?1 AND deleted_at IS NULL ORDER BY position, created_at, rowid",
    )?;
    let rows = stmt.query_map([study_id], |row| {
        Ok(TeachingSummary {
            id: row.get(0)?,
            project_id: row.get(5)?,
            study_id: row.get(1)?,
            title: row.get(2)?,
            kind: row.get(3)?,
            updated_at: row.get(4)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub(crate) fn row(conn: &Connection, id: &str) -> AppResult<TeachingPreparation> {
    conn.query_row(
        &format!(
            "SELECT {COLUMNS} FROM teaching_preparations WHERE id = ?1 AND deleted_at IS NULL AND EXISTS (SELECT 1 FROM projects p WHERE p.id=project_id AND p.deleted_at IS NULL)"
        ),
        [id],
        from_row,
    )
    .optional()?
    .ok_or_else(|| AppError::not_found("teaching preparation", id))
}

/// A preparation with its points, findings (as they are now), passages and
/// kept material.
pub fn get(conn: &Connection, id: &str) -> AppResult<TeachingPreparation> {
    let mut prep = row(conn, id)?;
    prep.points = points_for(conn, id)?;
    prep.findings = findings_for(conn, id)?;
    prep.passages = passages_for(conn, id)?;
    prep.materials = materials_for(conn, id)?;
    prep.blocks = crate::db::teaching_blocks::list(conn, id)?;
    if prep.template_version.unwrap_or(0) >= 2 || !prep.blocks.is_empty() {
        let body = |role: &str| {
            prep.blocks
                .iter()
                .find(|b| b.role.as_deref() == Some(role))
                .map(|b| b.body.clone())
                .unwrap_or_default()
        };
        prep.introduction = body("introduction");
        prep.conclusion = body("conclusion");
        prep.notes = body("notes");
        prep.central_message = Some(body("central_message")).filter(|s| !s.is_empty());
    }
    Ok(prep)
}

pub fn create(conn: &Connection, input: &CreateTeaching) -> AppResult<TeachingPreparation> {
    let study_id = input.study_id.trim();
    let linked = if study_id.is_empty() {
        None
    } else {
        Some(studies::get(conn, study_id)?)
    };
    let project_id = input
        .project_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .or_else(|| linked.as_ref().map(|s| s.project_id.as_str()))
        .ok_or_else(|| {
            invalid(
                "teaching.projectRequired",
                "Choose a project for this teaching.",
            )
        })?;
    projects::get(conn, project_id)?;
    if linked.as_ref().is_some_and(|s| s.project_id != project_id) {
        return Err(invalid(
            "teaching.otherProject",
            "Choose a study in this project.",
        ));
    }
    let study_id = linked.as_ref().map(|s| s.id.as_str());
    let title = validate::title(&input.title, "teaching")?;
    let kind = check_kind(input.kind.as_deref())?;
    if input.points.len() > STARTING_POINTS_MAX {
        return Err(invalid("teaching.tooLong", "Too many starting steps."));
    }
    let starting = input
        .points
        .iter()
        .map(|p| Ok((point_title(&p.title)?, check_step_kind(p.kind.as_deref())?)))
        .collect::<AppResult<Vec<_>>>()?;
    let position: i64 = conn.query_row(
        "SELECT COALESCE(MAX(position), -1) + 1 FROM teaching_preparations
         WHERE project_id = ?1 AND deleted_at IS NULL",
        [project_id],
        |r| r.get(0),
    )?;
    let id = new_id();
    let now = now();
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO teaching_preparations (id, study_id, title, kind, position, created_at, updated_at, project_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, ?7)",
        params![id, study_id, title, kind, position, now, project_id],
    )?;
    for (index, (point, step)) in starting.iter().enumerate() {
        tx.execute(
            "INSERT INTO teaching_points (id, preparation_id, position, kind, title, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![new_id(), id, index as i64, step, point, now],
        )?;
    }
    let block_session = input.template_version.unwrap_or(0) >= 2 || !input.blocks.is_empty();
    if block_session {
        let version = input.template_version.unwrap_or(2);
        tx.execute(
            "UPDATE teaching_preparations SET template_key = ?2, template_version = ?3 WHERE id = ?1",
            params![id, input.template_key, version],
        )?;
        crate::db::teaching_blocks::insert_starting(&tx, &id, &input.blocks, &now)?;
    }
    if let Some(study) = study_id {
        studies::touch(&tx, study, &now)?;
    }
    tx.commit()?;
    get(conn, &id)
}

pub fn update(conn: &Connection, input: &UpdateTeaching) -> AppResult<TeachingPreparation> {
    let current = row(conn, &input.id)?;
    let title = match &input.title {
        Some(t) => validate::title(t, "teaching")?,
        None => current.title,
    };
    let kind = match &input.kind {
        Some(k) => check_kind(Some(k))?,
        None => current.kind,
    };
    let short = |value: &Option<String>, current: Option<String>, what: &str| match value {
        Some(v) => validate::optional_text(Some(v), DESCRIPTION_MAX_CHARS, what),
        None => Ok(current),
    };
    let audience = short(&input.audience, current.audience, "audience")?;
    let purpose = short(&input.purpose, current.purpose, "purpose")?;
    let central_message = short(&input.central_message, current.central_message, "message")?;
    let long = |value: &Option<String>, current: String| match value {
        Some(v) => long_text(v),
        None => Ok(current),
    };
    let introduction = long(&input.introduction, current.introduction)?;
    let conclusion = long(&input.conclusion, current.conclusion)?;
    let notes = long(&input.notes, current.notes)?;
    let now = now();
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE teaching_preparations SET title = ?2, kind = ?3, audience = ?4, purpose = ?5,
             central_message = ?6, introduction = ?7, conclusion = ?8, notes = ?9,
             updated_at = ?10, revision = revision + 1
         WHERE id = ?1",
        params![
            input.id,
            title,
            kind,
            audience,
            purpose,
            central_message,
            introduction,
            conclusion,
            notes,
            now
        ],
    )?;
    // A block that wears one of these roles follows the column, so the block
    // editor and a presentation that reads the column stay on the same words.
    if input.introduction.is_some() {
        crate::db::teaching_blocks::mirror_column_into_blocks(
            &tx,
            &input.id,
            "introduction",
            &introduction,
        )?;
    }
    if input.conclusion.is_some() {
        crate::db::teaching_blocks::mirror_column_into_blocks(
            &tx,
            &input.id,
            "conclusion",
            &conclusion,
        )?;
    }
    if input.notes.is_some() {
        crate::db::teaching_blocks::mirror_column_into_blocks(&tx, &input.id, "notes", &notes)?;
    }
    if input.central_message.is_some() {
        crate::db::teaching_blocks::mirror_column_into_blocks(
            &tx,
            &input.id,
            "central_message",
            central_message.as_deref().unwrap_or(""),
        )?;
    }
    if let Some(study) = &current.study_id {
        studies::touch(&tx, study, &now)?;
    }
    tx.commit()?;
    get(conn, &input.id)
}

/// Soft delete. The study, its findings and its passages are untouched.
pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    let current = row(conn, id)?;
    let now = now();
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE teaching_preparations SET deleted_at = ?2, updated_at = ?2, revision = revision + 1
         WHERE id = ?1",
        params![id, now],
    )?;
    if let Some(study) = &current.study_id {
        studies::touch(&tx, study, &now)?;
    }
    tx.commit()?;
    Ok(())
}

pub(crate) fn touch(conn: &Connection, prep: &TeachingPreparation, now: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE teaching_preparations SET updated_at = ?2, revision = revision + 1 WHERE id = ?1",
        params![prep.id, now],
    )?;
    if let Some(study) = &prep.study_id {
        studies::touch(conn, study, now)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Points
// ---------------------------------------------------------------------------

fn points_for(conn: &Connection, prep_id: &str) -> AppResult<Vec<TeachingPoint>> {
    let mut stmt = conn.prepare(
        "SELECT id, position, title, explanation, application, illustration, kind FROM teaching_points
         WHERE preparation_id = ?1 AND deleted_at IS NULL ORDER BY position, created_at, rowid",
    )?;
    let rows = stmt.query_map([prep_id], |row| {
        Ok(TeachingPoint {
            id: row.get(0)?,
            position: row.get(1)?,
            title: row.get(2)?,
            explanation: row.get(3)?,
            application: row.get(4)?,
            illustration: row.get(5)?,
            kind: row.get(6)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// The preparation a live point belongs to.
fn point_owner(conn: &Connection, point_id: &str) -> AppResult<String> {
    conn.query_row(
        "SELECT preparation_id FROM teaching_points WHERE id = ?1 AND deleted_at IS NULL",
        [point_id],
        |r| r.get(0),
    )
    .optional()?
    .ok_or_else(|| AppError::not_found("teaching point", point_id))
}

/// A point named by a link or passage must be one of this preparation's.
fn check_point(conn: &Connection, prep_id: &str, point_id: Option<&str>) -> AppResult<()> {
    if let Some(point) = point_id {
        if point_owner(conn, point)? != prep_id {
            return Err(invalid(
                "teaching.otherPoint",
                "That point belongs to another teaching.",
            ));
        }
    }
    Ok(())
}

fn renumber(conn: &Connection, ids: &[String], now: &str) -> AppResult<()> {
    for (index, id) in ids.iter().enumerate() {
        conn.execute(
            "UPDATE teaching_points SET position = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, index as i64, now],
        )?;
    }
    Ok(())
}

pub fn create_point(
    conn: &Connection,
    input: &CreateTeachingPoint,
) -> AppResult<TeachingPreparation> {
    let prep = row(conn, &input.preparation_id)?;
    let title = point_title(input.title.as_deref().unwrap_or(""))?;
    let kind = check_step_kind(input.kind.as_deref())?;
    let mut order: Vec<String> = points_for(conn, &prep.id)?
        .into_iter()
        .map(|p| p.id)
        .collect();
    let at = match input.after_point_id.as_deref().filter(|p| !p.is_empty()) {
        Some(after) => {
            check_point(conn, &prep.id, Some(after))?;
            order
                .iter()
                .position(|id| id == after)
                .map_or(order.len(), |i| i + 1)
        }
        None => order.len(),
    };
    let id = new_id();
    let now = now();
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO teaching_points (id, preparation_id, position, kind, title, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
        params![id, prep.id, at as i64, kind, title, now],
    )?;
    order.insert(at, id);
    renumber(&tx, &order, &now)?;
    touch(&tx, &prep, &now)?;
    tx.commit()?;
    get(conn, &prep.id)
}

pub fn update_point(
    conn: &Connection,
    input: &UpdateTeachingPoint,
) -> AppResult<TeachingPreparation> {
    let prep = row(conn, &point_owner(conn, &input.id)?)?;
    let current = points_for(conn, &prep.id)?
        .into_iter()
        .find(|p| p.id == input.id)
        .ok_or_else(|| AppError::not_found("teaching point", &input.id))?;
    let title = match &input.title {
        Some(t) => point_title(t)?,
        None => current.title,
    };
    let kind = match &input.kind {
        Some(k) => check_step_kind(Some(k))?,
        None => current.kind,
    };
    let field = |value: &Option<String>, current: String| match value {
        Some(v) => long_text(v),
        None => Ok(current),
    };
    let explanation = field(&input.explanation, current.explanation)?;
    let application = field(&input.application, current.application)?;
    let illustration = field(&input.illustration, current.illustration)?;
    let now = now();
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE teaching_points SET title = ?2, explanation = ?3, application = ?4,
             illustration = ?5, kind = ?6, updated_at = ?7, revision = revision + 1
         WHERE id = ?1",
        params![
            input.id,
            title,
            explanation,
            application,
            illustration,
            kind,
            now
        ],
    )?;
    touch(&tx, &prep, &now)?;
    tx.commit()?;
    get(conn, &prep.id)
}

/// Removes a point with what is attached to it (links, passages, kept
/// material); linked findings themselves stay in the study.
pub fn delete_point(conn: &Connection, point_id: &str) -> AppResult<TeachingPreparation> {
    let prep = row(conn, &point_owner(conn, point_id)?)?;
    let now = now();
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE teaching_points SET deleted_at = ?2, updated_at = ?2, revision = revision + 1
         WHERE id = ?1",
        params![point_id, now],
    )?;
    for table in [
        "teaching_finding_links",
        "teaching_passages",
        "teaching_materials",
    ] {
        tx.execute(
            &format!(
                "UPDATE {table} SET deleted_at = ?2 WHERE point_id = ?1 AND deleted_at IS NULL"
            ),
            params![point_id, now],
        )?;
    }
    let order: Vec<String> = points_for(&tx, &prep.id)?
        .into_iter()
        .map(|p| p.id)
        .collect();
    renumber(&tx, &order, &now)?;
    touch(&tx, &prep, &now)?;
    tx.commit()?;
    get(conn, &prep.id)
}

/// Puts the points in the given order; it must name exactly the live points.
pub fn reorder_points(conn: &Connection, input: &Reorder) -> AppResult<TeachingPreparation> {
    let prep = row(conn, &input.parent_id)?;
    let current: Vec<String> = points_for(conn, &prep.id)?
        .into_iter()
        .map(|p| p.id)
        .collect();
    crate::db::check_same_members(&current, &input.ids, "points")?;
    let now = now();
    let tx = conn.unchecked_transaction()?;
    renumber(&tx, &input.ids, &now)?;
    touch(&tx, &prep, &now)?;
    tx.commit()?;
    get(conn, &prep.id)
}

// ---------------------------------------------------------------------------
// Findings (linked, never copied)
// ---------------------------------------------------------------------------

fn findings_for(conn: &Connection, prep_id: &str) -> AppResult<Vec<TeachingFindingLink>> {
    let mut stmt = conn.prepare(
        "SELECT l.id,l.point_id,l.finding_id,l.snapshot FROM teaching_finding_links l
        LEFT JOIN findings f ON f.id=l.finding_id
        LEFT JOIN teaching_points p ON p.id=l.point_id
        WHERE l.preparation_id=?1 AND l.deleted_at IS NULL
        AND (l.point_id IS NULL OR p.deleted_at IS NULL)
        AND ((l.finding_id IS NOT NULL AND f.id IS NOT NULL AND f.deleted_at IS NULL)
             OR (l.finding_id IS NULL AND l.snapshot IS NOT NULL))
        ORDER BY l.position,l.created_at,l.rowid",
    )?;
    let rows = stmt
        .query_map([prep_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, Option<String>>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    rows.into_iter()
        .map(|(id, point_id, finding_id, snapshot)| {
            let kept = finding_id.is_none();
            let presentation_only = snapshot
                .as_deref()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
                .and_then(|v| v.get("presentationOnly").and_then(|v| v.as_bool()))
                .unwrap_or(false);
            let finding = match finding_id {
                Some(f) => findings::get(conn, &f)?,
                None => read_saved_finding(snapshot.as_deref().unwrap_or(""))?,
            };
            let mut finding = finding;
            if kept {
                hydrate_saved_citations(conn, snapshot.as_deref().unwrap_or(""), &mut finding)?;
            }
            Ok(TeachingFindingLink {
                id,
                point_id,
                finding,
                snapshot: kept,
                presentation_only,
            })
        })
        .collect()
}

/// Uses one of the study's findings here. Linking the same finding to the
/// same place again changes nothing.
pub fn link_finding(
    conn: &Connection,
    input: &LinkTeachingFinding,
) -> AppResult<TeachingPreparation> {
    let prep = row(conn, &input.preparation_id)?;
    let point_id = input.point_id.as_deref().filter(|p| !p.is_empty());
    check_point(conn, &prep.id, point_id)?;
    let finding = findings::get(conn, &input.finding_id)?;
    if prep.study_id.as_deref() != Some(finding.study_id.as_str()) {
        return Err(invalid(
            "teaching.otherStudyFinding",
            "That finding belongs to another study.",
        ));
    }
    let exists: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM teaching_finding_links
         WHERE preparation_id = ?1 AND COALESCE(point_id, '') = COALESCE(?2, '')
           AND finding_id = ?3 AND deleted_at IS NULL)",
        params![prep.id, point_id, finding.id],
        |r| r.get(0),
    )?;
    if exists {
        return get(conn, &prep.id);
    }
    let position: i64 = conn.query_row(
        "SELECT COALESCE(MAX(position), -1) + 1 FROM teaching_finding_links
         WHERE preparation_id = ?1 AND deleted_at IS NULL",
        [&prep.id],
        |r| r.get(0),
    )?;
    let now = now();
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO teaching_finding_links (id, preparation_id, point_id, finding_id, position, created_at, snapshot)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![new_id(), prep.id, point_id, finding.id, position, now, saved_finding(conn,&finding)?],
    )?;
    touch(&tx, &prep, &now)?;
    tx.commit()?;
    get(conn, &prep.id)
}

/// Stops using a finding here. The finding stays in the study as it is.
pub fn unlink_finding(conn: &Connection, link_id: &str) -> AppResult<TeachingPreparation> {
    let prep_id: String = conn
        .query_row(
            "SELECT preparation_id FROM teaching_finding_links WHERE id = ?1 AND deleted_at IS NULL",
            [link_id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| AppError::not_found("teaching finding link", link_id))?;
    let prep = row(conn, &prep_id)?;
    let now = now();
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE teaching_finding_links SET deleted_at = ?2 WHERE id = ?1",
        params![link_id, now],
    )?;
    touch(&tx, &prep, &now)?;
    tx.commit()?;
    get(conn, &prep.id)
}

// ---------------------------------------------------------------------------
// Scripture
// ---------------------------------------------------------------------------

fn passages_for(conn: &Connection, prep_id: &str) -> AppResult<Vec<TeachingPassage>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.point_id, t.block_id, t.book, t.start_chapter, t.start_verse, t.end_chapter, t.end_verse
         FROM teaching_passages t LEFT JOIN teaching_points p ON p.id = t.point_id
         WHERE t.preparation_id = ?1 AND t.deleted_at IS NULL
           AND (t.point_id IS NULL OR p.deleted_at IS NULL)
         ORDER BY t.position, t.created_at, t.rowid",
    )?;
    type Raw = (
        String,
        Option<String>,
        Option<String>,
        String,
        u16,
        Option<u16>,
        Option<u16>,
        Option<u16>,
    );
    let rows: Vec<Raw> = stmt
        .query_map([prep_id], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
                r.get(7)?,
            ))
        })?
        .collect::<Result<_, _>>()?;
    rows.into_iter()
        .map(|(id, point_id, block_id, book, sc, sv, ec, ev)| {
            let reference = reference_from_columns(&id, &book, (sc, sv), (ec, ev))?;
            Ok(TeachingPassage {
                id,
                point_id,
                block_id,
                reference: reference.to_dto(),
            })
        })
        .collect()
}

/// Names a passage in the teaching (or one point). The study's own passages
/// are not changed. The same passage in the same place is kept once.
pub fn add_passage(
    conn: &Connection,
    input: &AddTeachingPassage,
) -> AppResult<TeachingPreparation> {
    let prep = row(conn, &input.preparation_id)?;
    let point_id = input.point_id.as_deref().filter(|p| !p.is_empty());
    let block_id = input.block_id.as_deref().filter(|p| !p.is_empty());
    check_point(conn, &prep.id, point_id)?;
    if let Some(block) = block_id {
        if point_id.is_some() {
            return Err(invalid(
                "teaching.blockReference",
                "Choose a live Scripture block in this teaching.",
            ));
        }
        crate::db::teaching_blocks::check_passage_target(conn, &prep.id, block)?;
    }
    let reference = ScriptureRef::from_canonical(&input.reference)?;
    let canonical = reference.canonical();
    if passages_for(conn, &prep.id)?.iter().any(|p| {
        p.point_id.as_deref() == point_id
            && p.block_id.as_deref() == block_id
            && p.reference.canonical == canonical
    }) {
        return get(conn, &prep.id);
    }
    let (end_chapter, end_verse) = columns_for(&reference);
    let position: i64 = conn.query_row(
        "SELECT COALESCE(MAX(position), -1) + 1 FROM teaching_passages
         WHERE preparation_id = ?1 AND deleted_at IS NULL",
        [&prep.id],
        |r| r.get(0),
    )?;
    let now = now();
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO teaching_passages (id, preparation_id, point_id, block_id, book, start_chapter,
             start_verse, end_chapter, end_verse, position, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            new_id(),
            prep.id,
            point_id,
            block_id,
            reference.book.code,
            reference.start.chapter,
            reference.start.verse,
            end_chapter,
            end_verse,
            position,
            now
        ],
    )?;
    touch(&tx, &prep, &now)?;
    tx.commit()?;
    get(conn, &prep.id)
}

pub fn remove_passage(conn: &Connection, passage_id: &str) -> AppResult<TeachingPreparation> {
    let prep_id: String = conn
        .query_row(
            "SELECT preparation_id FROM teaching_passages WHERE id = ?1 AND deleted_at IS NULL",
            [passage_id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| AppError::not_found("teaching passage", passage_id))?;
    let prep = row(conn, &prep_id)?;
    let now = now();
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE teaching_passages SET deleted_at = ?2 WHERE id = ?1",
        params![passage_id, now],
    )?;
    touch(&tx, &prep, &now)?;
    tx.commit()?;
    get(conn, &prep.id)
}

// ---------------------------------------------------------------------------
// Material kept from the assistant
// ---------------------------------------------------------------------------

fn materials_for(conn: &Connection, prep_id: &str) -> AppResult<Vec<TeachingMaterial>> {
    let mut stmt = conn.prepare(
        "SELECT m.id, m.point_id, m.kind, m.standing, m.title, m.content, m.sources, m.image,
             m.assistant, m.created_at
         FROM teaching_materials m LEFT JOIN teaching_points p ON p.id = m.point_id
         WHERE m.preparation_id = ?1 AND m.deleted_at IS NULL
           AND (m.point_id IS NULL OR p.deleted_at IS NULL)
         ORDER BY m.position, m.created_at, m.rowid",
    )?;
    let rows = stmt.query_map([prep_id], |r| {
        let sources: Option<String> = r.get(6)?;
        let image: Option<String> = r.get(7)?;
        Ok(TeachingMaterial {
            id: r.get(0)?,
            point_id: r.get(1)?,
            kind: r.get(2)?,
            standing: r.get(3)?,
            title: r.get(4)?,
            content: r.get(5)?,
            sources: sources
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default(),
            image: image.and_then(|s| serde_json::from_str(&s).ok()),
            assistant: r.get(8)?,
            created_at: r.get(9)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Only web addresses are kept as links (no `javascript:`, no local files).
pub(crate) fn is_web_address(url: &str) -> bool {
    let lower = url.trim().to_ascii_lowercase();
    (lower.starts_with("https://") || lower.starts_with("http://"))
        && !url.chars().any(char::is_whitespace)
        && url.len() <= SOURCE_TEXT_MAX
}

/// A generated image is saved by the core under a name it chose itself; a
/// kept item may only name such a file.
pub(crate) fn is_generated_file_name(name: &str) -> bool {
    let Some(stem) = name.strip_suffix(".png") else {
        return false;
    };
    !stem.is_empty() && stem.len() <= 64 && stem.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

fn check_sources(sources: &[ExternalSource]) -> AppResult<()> {
    if sources.len() > SOURCES_MAX {
        return Err(invalid(
            "teaching.badMaterial",
            "This item has too many sources.",
        ));
    }
    for s in sources {
        if !is_web_address(&s.url)
            || [&s.title, &s.site, &s.excerpt]
                .iter()
                .any(|t| t.chars().count() > SOURCE_TEXT_MAX)
        {
            return Err(invalid(
                "teaching.badMaterial",
                "A source of this item cannot be kept.",
            ));
        }
    }
    Ok(())
}

fn check_image(image: &ImageInfo, standing: &str) -> AppResult<()> {
    let urls = [
        &image.url,
        &image.page_url,
        &image.thumb_url,
        &image.licence_url,
    ];
    if urls
        .iter()
        .any(|u| u.as_deref().is_some_and(|u| !is_web_address(u)))
    {
        return Err(invalid(
            "teaching.badMaterial",
            "This image's address cannot be kept.",
        ));
    }
    // A found image is kept only with where it is from and its licence.
    let blank = |v: &Option<String>| v.as_deref().is_none_or(|v| v.trim().is_empty());
    if standing == "found" && (blank(&image.page_url) || blank(&image.licence) || blank(&image.url))
    {
        return Err(invalid(
            "teaching.badMaterial",
            "An image is only kept with its source and licence.",
        ));
    }
    match (&image.file, standing) {
        (Some(file), "generated") if is_generated_file_name(file) => Ok(()),
        (Some(_), _) => Err(invalid(
            "teaching.badMaterial",
            "This image cannot be kept.",
        )),
        (None, "generated") => Err(invalid(
            "teaching.badMaterial",
            "This image cannot be kept.",
        )),
        (None, _) => Ok(()),
    }
}

/// Keeps something from the assistant, at the person's request, with its
/// standing. The rules keep the standing honest: a story is verified only
/// with two sources from different sites; images are found (with an
/// address) or generated (with a saved file); illustrations are invented.
pub fn keep_material(
    conn: &Connection,
    input: &KeepTeachingMaterial,
) -> AppResult<TeachingPreparation> {
    let prep = row(conn, &input.preparation_id)?;
    let point_id = input.point_id.as_deref().filter(|p| !p.is_empty());
    check_point(conn, &prep.id, point_id)?;
    let kind = input.kind.as_str();
    let standing = input.standing.as_str();
    if !MATERIAL_KINDS.contains(&kind) || !MATERIAL_STANDINGS.contains(&standing) {
        return Err(invalid("teaching.badMaterial", "This item cannot be kept."));
    }
    let allowed = match kind {
        "story" => matches!(standing, "verified" | "unverified"),
        "illustration" => standing == "hypothetical",
        "image" => matches!(standing, "found" | "generated"),
        _ => standing == "suggestion",
    };
    if !allowed {
        return Err(invalid("teaching.badMaterial", "This item cannot be kept."));
    }
    check_sources(&input.sources)?;
    if standing == "verified" {
        let mut sites: Vec<String> = input
            .sources
            .iter()
            .filter_map(|s| crate::assistant::research::site_of(&s.url))
            .collect();
        sites.sort();
        sites.dedup();
        if sites.len() < 2 {
            return Err(invalid(
                "teaching.notVerified",
                "A story is only kept as checked with two independent sources.",
            ));
        }
    }
    match (&input.image, kind) {
        (Some(image), "image") => check_image(image, standing)?,
        (None, "image") => {
            return Err(invalid(
                "teaching.badMaterial",
                "This image cannot be kept.",
            ))
        }
        (Some(_), _) => return Err(invalid("teaching.badMaterial", "This item cannot be kept.")),
        (None, _) => {}
    }
    let title = input.title.trim();
    let content = input.content.trim();
    if title.chars().count() > validate::TITLE_MAX_CHARS * 2 {
        return Err(invalid(
            "teaching.tooLong",
            "This item's title is too long.",
        ));
    }
    long_text(content)?;
    let assistant = input.assistant.as_deref().map(|a| {
        a.chars()
            .take(validate::TITLE_MAX_CHARS)
            .collect::<String>()
    });
    let sources = if input.sources.is_empty() {
        None
    } else {
        Some(
            serde_json::to_string(&input.sources)
                .map_err(|e| AppError::Unexpected(e.to_string()))?,
        )
    };
    let image = input
        .image
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .map_err(|e| AppError::Unexpected(e.to_string()))?;
    let position: i64 = conn.query_row(
        "SELECT COALESCE(MAX(position), -1) + 1 FROM teaching_materials
         WHERE preparation_id = ?1 AND deleted_at IS NULL",
        [&prep.id],
        |r| r.get(0),
    )?;
    let now = now();
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO teaching_materials (id, preparation_id, point_id, kind, standing, title,
             content, sources, image, assistant, position, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            new_id(),
            prep.id,
            point_id,
            kind,
            standing,
            title,
            content,
            sources,
            image,
            assistant,
            position,
            now
        ],
    )?;
    touch(&tx, &prep, &now)?;
    tx.commit()?;
    get(conn, &prep.id)
}

pub fn remove_material(conn: &Connection, material_id: &str) -> AppResult<TeachingPreparation> {
    let prep_id: String = conn
        .query_row(
            "SELECT preparation_id FROM teaching_materials WHERE id = ?1 AND deleted_at IS NULL",
            [material_id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| AppError::not_found("teaching material", material_id))?;
    let prep = row(conn, &prep_id)?;
    let now = now();
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE teaching_materials SET deleted_at = ?2 WHERE id = ?1",
        params![material_id, now],
    )?;
    touch(&tx, &prep, &now)?;
    tx.commit()?;
    get(conn, &prep.id)
}

/// Project ownership is independent of the optional study relationship.
pub fn list_project(conn: &Connection, project_id: &str) -> AppResult<Vec<TeachingSummary>> {
    projects::get(conn, project_id)?;
    let mut stmt=conn.prepare("SELECT id,project_id,study_id,title,kind,updated_at FROM teaching_preparations WHERE project_id=?1 AND deleted_at IS NULL ORDER BY position,created_at,rowid")?;
    let result = stmt
        .query_map([project_id], |r| {
            Ok(TeachingSummary {
                id: r.get(0)?,
                project_id: r.get(1)?,
                study_id: r.get(2)?,
                title: r.get(3)?,
                kind: r.get(4)?,
                updated_at: r.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(result)
}

pub fn set_study(
    conn: &Connection,
    id: &str,
    study_id: Option<&str>,
) -> AppResult<TeachingPreparation> {
    let prep = row(conn, id)?;
    let study_id = study_id.filter(|s| !s.is_empty());
    if let Some(study) = study_id {
        if studies::get(conn, study)?.project_id != prep.project_id {
            return Err(invalid(
                "teaching.otherProject",
                "Choose a study in this project.",
            ));
        }
    }
    if prep.study_id.as_deref() == study_id {
        return get(conn, id);
    }
    let tx = conn.unchecked_transaction()?;
    detach_findings(&tx, id)?;
    let at = now();
    tx.execute("UPDATE teaching_preparations SET study_id=?2,updated_at=?3,revision=revision+1 WHERE id=?1",params![id,study_id,at])?;
    tx.commit()?;
    get(conn, id)
}

/// Captures current writing AND raw citation provenance before severing live
/// finding FKs. Existing link IDs, positions, points, blocks and tombstones stay.
pub(crate) fn detach_findings(conn: &Connection, preparation_id: &str) -> AppResult<()> {
    let mut sources=conn.prepare("SELECT DISTINCT i.source_id FROM presentation_items i JOIN presentations p ON p.id=i.presentation_id WHERE p.preparation_id=?1 AND i.source_kind='finding' AND i.source_id IS NOT NULL AND NOT EXISTS(SELECT 1 FROM teaching_finding_links l WHERE l.preparation_id=?1 AND l.finding_id=i.source_id AND l.deleted_at IS NULL)")?;
    let source_ids = sources
        .query_map([preparation_id], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    for source in source_ids {
        let saved: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM teaching_finding_links WHERE preparation_id=?1 AND deleted_at IS NULL AND json_extract(snapshot,'$.finding.id')=?2)", params![preparation_id, source], |r|r.get(0))?;
        if saved {
            continue;
        }
        {
            let finding = match findings::get(conn, &source) {
                Ok(finding) => finding,
                Err(AppError::NotFound { .. }) => continue,
                Err(error) => return Err(error),
            };
            let mut snapshot: serde_json::Value =
                serde_json::from_str(&saved_finding(conn, &finding)?)
                    .map_err(|e| AppError::Unexpected(e.to_string()))?;
            snapshot["presentationOnly"] = serde_json::json!(true);
            conn.execute("INSERT INTO teaching_finding_links(id,preparation_id,finding_id,position,created_at,snapshot) VALUES(?1,?2,NULL,0,?3,?4)",params![new_id(),preparation_id,now(),snapshot.to_string()])?;
        }
    }
    let mut stmt=conn.prepare("SELECT id,finding_id FROM teaching_finding_links WHERE preparation_id=?1 AND finding_id IS NOT NULL")?;
    let rows = stmt
        .query_map([preparation_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    for (link, id) in rows {
        match findings::get(conn, &id) {
            Ok(finding) => {
                conn.execute(
                    "UPDATE teaching_finding_links SET snapshot=?2 WHERE id=?1",
                    params![link, saved_finding(conn, &finding)?],
                )?;
            }
            Err(AppError::NotFound { .. }) => {
                // A deleted live Finding must not reappear from its cache
                // when clearing the FK. Keep the link as a tombstone.
                conn.execute("UPDATE teaching_finding_links SET deleted_at=COALESCE(deleted_at,?2) WHERE id=?1",params![link,now()])?;
            }
            Err(error) => return Err(error),
        }
    }
    conn.execute(
        "UPDATE teaching_finding_links SET finding_id=NULL WHERE preparation_id=?1",
        [preparation_id],
    )?;
    Ok(())
}

pub(crate) fn detach_study(conn: &Connection, study: &str, at: &str) -> AppResult<()> {
    let mut stmt = conn.prepare("SELECT id FROM teaching_preparations WHERE study_id=?1")?;
    let ids = stmt
        .query_map([study], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    for id in ids {
        detach_findings(conn, &id)?;
    }
    conn.execute("UPDATE teaching_preparations SET study_id=NULL,updated_at=?2,revision=revision+1 WHERE study_id=?1",params![study,at])?;
    Ok(())
}

pub fn restore(conn: &Connection, id: &str) -> AppResult<TeachingPreparation> {
    let tx = conn.unchecked_transaction()?;
    let at = now();
    let count=tx.execute("UPDATE teaching_preparations SET deleted_at=NULL,updated_at=?2,revision=revision+1 WHERE id=?1 AND deleted_at IS NOT NULL AND EXISTS(SELECT 1 FROM projects WHERE id=project_id AND deleted_at IS NULL)",params![id,at])?;
    if count == 0 {
        row(&tx, id)?;
    }
    tx.commit()?;
    get(conn, id)
}

pub(crate) fn saved_finding(
    conn: &Connection,
    finding: &crate::domain::Finding,
) -> AppResult<String> {
    use rusqlite::types::ValueRef;
    let mut stmt = conn.prepare(
        "SELECT * FROM finding_sources WHERE finding_id=?1 AND deleted_at IS NULL ORDER BY rowid",
    )?;
    let names = stmt
        .column_names()
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let mut citations = stmt
        .query_map([&finding.id], |r| {
            let mut out = serde_json::Map::new();
            for (i, name) in names.iter().enumerate() {
                let v = match r.get_ref(i)? {
                    ValueRef::Null => serde_json::Value::Null,
                    ValueRef::Integer(n) => serde_json::json!(n),
                    ValueRef::Real(n) => serde_json::json!(n),
                    ValueRef::Text(s) => {
                        serde_json::Value::String(String::from_utf8_lossy(s).into_owned())
                    }
                    ValueRef::Blob(_) => return Err(rusqlite::Error::InvalidQuery),
                };
                out.insert(name.clone(), v);
            }
            Ok(serde_json::Value::Object(out))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    // A saved citation must not reopen the mutable live Study citation with
    // the same ID. Give the copy its own locator ID and retain the original
    // ID explicitly as provenance; Finding and presentation source IDs stay.
    let mut finding = finding.clone();
    for raw in &mut citations {
        let original = raw["id"]
            .as_str()
            .ok_or_else(|| {
                invalid(
                    "teaching.badSnapshot",
                    "This saved study note cannot be read.",
                )
            })?
            .to_string();
        // Older citations may predate stored display/hash snapshots. Capture
        // the known source metadata now without changing any historical row.
        if let Some(citation) = finding.sources.iter().find(|c| c.id == original) {
            for (key, value) in [
                ("cited_title", Some(citation.title.as_str())),
                ("cited_author", citation.author.as_deref()),
                ("cited_section", citation.section.as_deref()),
            ] {
                if raw[key].is_null() {
                    raw[key] = serde_json::json!(value);
                }
            }
        }
        if let Some(source) = raw["source_id"].as_str().map(String::from) {
            if raw["cited_source_id"].is_null() {
                raw["cited_source_id"] = serde_json::json!(source);
            }
            if raw["cited_content_hash"].is_null() {
                let hash: Option<String> = conn
                    .query_row(
                        "SELECT content_hash FROM sources WHERE id=?1",
                        [source],
                        |r| r.get(0),
                    )
                    .optional()?
                    .flatten();
                raw["cited_content_hash"] = serde_json::json!(hash);
            }
        }
        let id = new_id();
        raw["originCitationId"] = serde_json::json!(original);
        raw["id"] = serde_json::json!(id);
        if let Some(citation) = finding.sources.iter_mut().find(|c| c.id == original) {
            citation.id = id;
        }
    }
    serde_json::to_string(&serde_json::json!({"version":1,"finding":finding,"citations":citations}))
        .map_err(|e| AppError::Unexpected(e.to_string()))
}

/// Validate a saved note without deserializing a &'static book-code field.
/// References are reconstructed only from their checked canonical form.
pub(crate) fn read_saved_finding(snapshot: &str) -> AppResult<crate::domain::Finding> {
    use crate::domain::{Finding, FindingPassage};
    let value: serde_json::Value =
        serde_json::from_str(snapshot).map_err(|e| AppError::Unexpected(e.to_string()))?;
    if value.get("version").and_then(|v| v.as_i64()) != Some(1) {
        return Err(invalid(
            "teaching.badSnapshot",
            "This saved study note cannot be read.",
        ));
    }
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Saved {
        id: String,
        study_id: String,
        section_id: Option<String>,
        finding_type: crate::domain::FindingType,
        content: String,
        sort_order: i64,
        parent_id: Option<String>,
        passages: Vec<Passage>,
        tags: Vec<String>,
        sources: Vec<crate::library::FindingCitation>,
        assisted_by: Option<String>,
        created_at: String,
        updated_at: String,
        revision: i64,
    }
    #[derive(serde::Deserialize)]
    struct Passage {
        id: String,
        reference: Reference,
    }
    #[derive(serde::Deserialize)]
    struct Reference {
        canonical: String,
    }
    let f: Saved = serde_json::from_value(value.get("finding").cloned().unwrap_or_default())
        .map_err(|e| AppError::Unexpected(e.to_string()))?;
    let passages = f
        .passages
        .into_iter()
        .map(|p| {
            Ok(FindingPassage {
                id: p.id,
                reference: ScriptureRef::from_canonical(&p.reference.canonical)?.to_dto(),
            })
        })
        .collect::<AppResult<Vec<_>>>()?;
    Ok(Finding {
        id: f.id,
        study_id: f.study_id,
        section_id: f.section_id,
        finding_type: f.finding_type,
        content: f.content,
        sort_order: f.sort_order,
        parent_id: f.parent_id,
        passages,
        tags: f.tags,
        sources: f.sources,
        assisted_by: f.assisted_by,
        created_at: f.created_at,
        updated_at: f.updated_at,
        revision: f.revision,
    })
}

/// Seed snapshots for existing live links at the schema-26 boundary, before
/// raw ON DELETE SET NULL actions could make their old Study unavailable.
pub(crate) fn initialise_finding_snapshots(conn: &Connection) -> AppResult<()> {
    let mut stmt=conn.prepare("SELECT DISTINCT l.finding_id FROM teaching_finding_links l JOIN findings f ON f.id=l.finding_id WHERE l.snapshot IS NULL AND f.deleted_at IS NULL")?;
    let ids = stmt
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    for id in ids {
        let finding = findings::get(conn, &id)?;
        conn.execute("UPDATE teaching_finding_links SET snapshot=?2 WHERE finding_id=?1 AND snapshot IS NULL",params![id,saved_finding(conn,&finding)?])?;
    }
    Ok(())
}

fn saved_source(conn: &Connection, c: &serde_json::Value) -> AppResult<Option<String>> {
    let id = c
        .get("source_id")
        .and_then(|v| v.as_str())
        .or_else(|| c.get("cited_source_id").and_then(|v| v.as_str()));
    if let Some(id) = id {
        if conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM sources WHERE id=?1)",
            [id],
            |r| r.get::<_, bool>(0),
        )? {
            return Ok(Some(id.to_string()));
        }
    }
    let Some(hash) = c
        .get("cited_content_hash")
        .and_then(|v| v.as_str())
        .filter(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit()))
    else {
        return Ok(None);
    };
    let mut stmt = conn.prepare("SELECT id FROM sources WHERE content_hash=?1 AND deleted_at IS NULL ORDER BY rowid LIMIT 2")?;
    let ids = stmt
        .query_map([hash], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(if ids.len() == 1 {
        ids.into_iter().next()
    } else {
        None
    })
}
fn hydrate_saved_citations(
    conn: &Connection,
    snapshot: &str,
    finding: &mut crate::domain::Finding,
) -> AppResult<()> {
    let v: serde_json::Value =
        serde_json::from_str(snapshot).map_err(|e| AppError::Unexpected(e.to_string()))?;
    for citation in &mut finding.sources {
        if let Some(raw) = v["citations"].as_array().and_then(|a| {
            a.iter()
                .find(|c| c["id"].as_str() == Some(citation.id.as_str()))
        }) {
            let source = saved_source(conn, raw)?;
            if source != citation.source_id {
                citation.segment_id = None;
                citation.annotation_id = None;
            }
            citation.source_id = source;
        }
        citation.unavailable = citation.source_id.is_none();
        citation.removed = match &citation.source_id {
            Some(id) => !conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM sources WHERE id=?1 AND deleted_at IS NULL)",
                [id],
                |r| r.get::<_, bool>(0),
            )?,
            None => true,
        };
    }
    Ok(())
}
pub(crate) fn locate_saved_citation(
    conn: &Connection,
    id: &str,
) -> AppResult<Option<crate::library::CitationPlace>> {
    let mut stmt=conn.prepare("SELECT l.snapshot FROM teaching_finding_links l JOIN teaching_preparations p ON p.id=l.preparation_id JOIN projects j ON j.id=p.project_id WHERE l.finding_id IS NULL AND l.deleted_at IS NULL AND p.deleted_at IS NULL AND j.deleted_at IS NULL AND l.snapshot IS NOT NULL")?;
    let snapshots = stmt
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    for snapshot in snapshots {
        let v: serde_json::Value =
            serde_json::from_str(&snapshot).map_err(|e| AppError::Unexpected(e.to_string()))?;
        if let Some(raw) = v["citations"]
            .as_array()
            .and_then(|a| a.iter().find(|c| c["id"].as_str() == Some(id)))
        {
            let Some(source) = saved_source(conn, raw)? else {
                return Ok(None);
            };
            let text = |k: &str| raw.get(k).and_then(|v| v.as_str());
            let anchor = crate::library::annotations::CitationAnchor::read(text("anchor"));
            return crate::db::source_annotations::locate_citation(
                conn,
                &source,
                text("source_segment_id"),
                text("quote"),
                text("cited_section"),
                anchor.as_ref(),
            );
        }
    }
    Err(AppError::not_found("citation", id))
}
