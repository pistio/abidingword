//! Explicit additive repair. Existing sections, answers and links are never
//! renamed or reinterpreted. A persistent receipt permits conservative Undo.
use crate::{
    db::{now, sections, settings, studies, templates},
    domain::{SectionFields, StudySection},
    error::{AppError, AppResult, UserMessage},
};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
const CANONICAL: [&str; 7] = [
    "persons",
    "place",
    "time",
    "events",
    "context",
    "explanations",
    "core",
];
#[derive(Serialize, Deserialize)]
struct Added {
    id: String,
    snapshot: serde_json::Value,
}
#[derive(Serialize, Deserialize)]
struct Receipt {
    before: Vec<String>,
    after: Vec<String>,
    added: Vec<Added>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepairView {
    pub sections: Vec<StudySection>,
    pub can_undo: bool,
}
fn conflict() -> AppError {
    AppError::Conflict(UserMessage::new("sections.repairChanged", "The parts changed. Review them again. Undo is available only while added parts remain unchanged; all writing is kept."))
}
pub fn run(
    conn: &Connection,
    study_id: &str,
    action: &str,
    expected_ids: Option<&[String]>,
) -> AppResult<RepairView> {
    let study = studies::get(conn, study_id)?;
    let template = templates::get(conn, study.template_id.as_deref().unwrap_or(""))?;
    if !template.is_builtin || template.key != "pozetek-writing-study" {
        return Err(conflict());
    }
    let key = format!("pozetek.repair.{study_id}");
    let tx = conn.unchecked_transaction()?;
    let mut current = sections::list(&tx, study_id)?;
    let ids: Vec<_> = current.iter().map(|s| s.id.clone()).collect();
    let receipt: Option<Receipt> = settings::read(&tx, &key)?;
    let at = now();
    match action {
        "preview" => {}
        "apply" => {
            if expected_ids != Some(ids.as_slice()) || receipt.is_some() {
                return Err(conflict());
            }
            let mut added_ids = Vec::new();
            for part_key in CANONICAL {
                if current.iter().any(|s| s.key.as_deref() == Some(part_key)) {
                    continue;
                }
                let part = template
                    .sections
                    .iter()
                    .find(|p| p.key.as_deref() == Some(part_key))
                    .ok_or_else(conflict)?;
                let fields = SectionFields {
                    template: template.key.clone(),
                    version: part.fields_version,
                    fields: part.fields.clone(),
                };
                let id = sections::insert_structured(
                    &tx,
                    study_id,
                    sections::NewSection {
                        key: Some(part_key),
                        title: &part.title,
                        guidance: part.guidance.as_deref(),
                        phase: part.phase,
                    },
                    current.len() as i64,
                    Some(&fields),
                    &at,
                )?;
                sections::seed_blocks(&tx, &id, Some(&fields), &at)?;
                current.push(sections::get(&tx, &id)?);
                added_ids.push(id);
            }
            // First occurrence of each canonical key, then every other part in
            // its original relative order (including duplicate Zeit).
            let mut order = Vec::new();
            for part_key in CANONICAL {
                if let Some(s) = current.iter().find(|s| s.key.as_deref() == Some(part_key)) {
                    order.push(s.id.clone());
                }
            }
            for s in &current {
                if !order.contains(&s.id) {
                    order.push(s.id.clone());
                }
            }
            if order != ids {
                for (position, id) in order.iter().enumerate() {
                    tx.execute("UPDATE study_sections SET position=?2,updated_at=?3,revision=revision+1 WHERE id=?1",params![id,position as i64,at])?;
                }
                studies::touch(&tx, study_id, &at)?;
                let added = added_ids
                    .iter()
                    .map(|id| {
                        let section = sections::get(&tx, id)?;
                        Ok(Added {
                            id: id.clone(),
                            snapshot: serde_json::to_value(section).map_err(|_| conflict())?,
                        })
                    })
                    .collect::<AppResult<Vec<_>>>()?;
                settings::write(
                    &tx,
                    &key,
                    &Receipt {
                        before: ids,
                        after: order,
                        added,
                    },
                )?;
            }
        }
        "undo" => {
            let saved = receipt.ok_or_else(conflict)?;
            if saved.after != ids {
                return Err(conflict());
            }
            for original in &saved.added {
                let live = sections::get(&tx, &original.id)?;
                if serde_json::to_value(&live).map_err(|_| conflict())? != original.snapshot
                    || sections::has_content(&tx, &live.id)?
                {
                    return Err(conflict());
                }
            }
            for added in &saved.added {
                tx.execute("UPDATE study_blocks SET deleted_at=?2,updated_at=?2,revision=revision+1 WHERE section_id=?1 AND deleted_at IS NULL",params![added.id,at])?;
                tx.execute("UPDATE study_sections SET deleted_at=?2,updated_at=?2,revision=revision+1 WHERE id=?1",params![added.id,at])?;
            }
            for (position, id) in saved.before.iter().enumerate() {
                tx.execute("UPDATE study_sections SET position=?2,updated_at=?3,revision=revision+1 WHERE id=?1",params![id,position as i64,at])?;
            }
            studies::touch(&tx, study_id, &at)?;
            settings::clear(&tx, &key)?;
        }
        _ => return Err(conflict()),
    }
    let result = RepairView {
        sections: sections::list(&tx, study_id)?,
        can_undo: settings::read::<Receipt>(&tx, &key)?.is_some(),
    };
    tx.commit()?;
    Ok(result)
}
