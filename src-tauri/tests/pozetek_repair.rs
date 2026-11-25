//! Invented old schema-27 Studies. No owner writing is used.
use abiding_word_lib::{
    db::{blocks, pozetek_repair, projects, sections, studies, templates, workspaces, Database},
    domain::{CreateProject, CreateStudy, CreateStudySection, UpdateStudyBlock},
};
use rusqlite::{params, Connection};
fn old(conn: &Connection) -> String {
    let project = projects::create(
        conn,
        &CreateProject {
            workspace_id: workspaces::get_default(conn).unwrap().id,
            title: "Repair review".into(),
            description: None,
        },
    )
    .unwrap();
    let template = templates::get_by_key(conn, "pozetek-writing-study").unwrap();
    let detail = studies::create(
        conn,
        &CreateStudy {
            project_id: project.id,
            title: "Old writing with duplicates".into(),
            template_id: Some(template.id),
            ..Default::default()
        },
    )
    .unwrap();
    for (part, old_key) in detail.sections.iter().zip([
        "persons",
        "place",
        "time",
        "events",
        "theme",
        "understanding",
        "consequences",
    ]) {
        conn.execute(
            "UPDATE study_sections SET key=?2,title=?3 WHERE id=?1",
            params![part.id, old_key, format!("Saved {old_key}")],
        )
        .unwrap();
        blocks::update(
            conn,
            &UpdateStudyBlock {
                id: part.blocks[0].id.clone(),
                content: Some(format!("Original {old_key} — 한글")),
                data: None,
            },
        )
        .unwrap();
    }
    sections::create(
        conn,
        &CreateStudySection {
            study_id: detail.study.id.clone(),
            title: "Zeit — my separate notes".into(),
            key: None,
            guidance: None,
            phase: None,
            template_section_key: None,
        },
    )
    .unwrap();
    conn.execute("UPDATE study_sections SET key='time' WHERE study_id=?1 AND title='Zeit — my separate notes'",[&detail.study.id]).unwrap();
    detail.study.id
}
fn ids(conn: &Connection, id: &str) -> Vec<String> {
    sections::list(conn, id)
        .unwrap()
        .iter()
        .map(|s| s.id.clone())
        .collect()
}
fn writing(conn: &Connection, id: &str) -> Vec<(String, String, String, Vec<String>)> {
    sections::list(conn, id)
        .unwrap()
        .into_iter()
        .map(|s| {
            (
                s.id,
                s.title,
                s.key.unwrap_or_default(),
                s.blocks
                    .into_iter()
                    .map(|b| format!("{}:{}:{:?}", b.id, b.content, b.data))
                    .collect(),
            )
        })
        .collect()
}
#[test]
fn canonical_method_and_existing_answers_survive_explicit_additive_repair_and_persistent_undo() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("profile.db");
    let backups = dir.path().join("backups");
    let db = Database::open(&path, &backups).unwrap();
    let (id, before, order) = db
        .with_conn(|conn| {
            let id = old(conn);
            let before = writing(conn, &id);
            let order = ids(conn, &id);
            let view = pozetek_repair::run(conn, &id, "preview", None)?;
            assert!(!view.can_undo);
            assert_eq!(ids(conn, &id), order);
            let view = pozetek_repair::run(conn, &id, "apply", Some(&order))?;
            assert!(view.can_undo);
            assert_eq!(
                view.sections
                    .iter()
                    .take(7)
                    .map(|s| s.key.as_deref().unwrap())
                    .collect::<Vec<_>>(),
                [
                    "persons",
                    "place",
                    "time",
                    "events",
                    "context",
                    "explanations",
                    "core"
                ]
            );
            assert_eq!(
                view.sections
                    .iter()
                    .filter(|s| s.key.as_deref() == Some("time"))
                    .count(),
                2
            );
            for item in &before {
                assert_eq!(
                    writing(conn, &id).iter().find(|row| row.0 == item.0),
                    Some(item)
                );
            }
            assert!(
                view.sections
                    .iter()
                    .filter(|s| ["context", "explanations", "core"]
                        .contains(&s.key.as_deref().unwrap()))
                    .all(|s| s.blocks.iter().all(|b| b.content.is_empty()))
            );
            Ok((id, before, order))
        })
        .unwrap();
    drop(db);
    let db = Database::open(&path, &backups).unwrap();
    db.with_conn(|conn| {
        assert!(pozetek_repair::run(conn, &id, "preview", None)?.can_undo);
        let restored = pozetek_repair::run(conn, &id, "undo", None)?;
        assert!(!restored.can_undo);
        assert_eq!(ids(conn, &id), order);
        assert_eq!(writing(conn, &id), before);
        Ok(())
    })
    .unwrap();
}
#[test]
fn changed_added_writing_and_stale_preview_refuse_without_mutation() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let id = old(conn);
        let order = ids(conn, &id);
        assert!(pozetek_repair::run(conn, &id, "apply", Some(&[])).is_err());
        assert_eq!(ids(conn, &id), order);
        let view = pozetek_repair::run(conn, &id, "apply", Some(&order))?;
        let added = view
            .sections
            .iter()
            .find(|s| s.key.as_deref() == Some("core"))
            .unwrap();
        blocks::update(
            conn,
            &UpdateStudyBlock {
                id: added.blocks[0].id.clone(),
                content: Some("New writing must survive Undo".into()),
                data: None,
            },
        )?;
        let before = serde_json::to_value(sections::list(conn, &id)?).unwrap();
        assert!(pozetek_repair::run(conn, &id, "undo", None).is_err());
        assert_eq!(
            serde_json::to_value(sections::list(conn, &id)?).unwrap(),
            before
        );
        Ok(())
    })
    .unwrap();
}
#[test]
fn receipt_storage_failure_rolls_back_parts_order_and_revisions() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn|{let id=old(conn);let order=ids(conn,&id);let before=serde_json::to_value(sections::list(conn,&id)?).unwrap();let revision=studies::get(conn,&id)?.revision;
 conn.execute_batch("CREATE TRIGGER fail_repair BEFORE INSERT ON settings WHEN NEW.key LIKE 'pozetek.repair.%' BEGIN SELECT RAISE(ABORT, 'injected receipt failure'); END;")?;
 assert!(pozetek_repair::run(conn,&id,"apply",Some(&order)).is_err());assert_eq!(serde_json::to_value(sections::list(conn,&id)?).unwrap(),before);assert_eq!(studies::get(conn,&id)?.revision,revision);
 conn.execute_batch("DROP TRIGGER fail_repair")?;assert!(pozetek_repair::run(conn,&id,"apply",Some(&order))?.can_undo);Ok(())}).unwrap();
}
