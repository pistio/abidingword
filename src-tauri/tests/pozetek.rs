//! The POZETEK built-in template (schema v10): structured fields kept as a
//! snapshot, answers in study blocks, progress per part, and an upgrade from
//! a v9 database. All text is invented.

use std::path::Path;

use abiding_word_lib::db::migrations::{self, MIGRATIONS};
use abiding_word_lib::db::{blocks, projects, sections, studies, templates, workspaces, Database};
use abiding_word_lib::domain::{
    BlockType, CreateProject, CreateStudy, CreateStudyBlock, CreateStudySection, DuplicateStudy,
    FieldKind, SectionStatus, StudyDetail, TemplateInput, UpdateStudyBlock, UpdateStudySection,
    UpdateTemplate,
};
use abiding_word_lib::error::{AppError, AppResult};
use rusqlite::Connection;

const KEYS: [&str; 8] = [
    "persons",
    "place",
    "time",
    "events",
    "context",
    "explanations",
    "core",
    "application",
];

#[test]
fn a_customised_template_retains_fields_and_their_translation_source() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let builtin = templates::get_by_key(conn, "pozetek-study")?;
        let copy = templates::duplicate(conn, &builtin.id)?;
        assert!(!copy.is_builtin);
        assert_eq!(copy.sections.len(), KEYS.len());
        assert!(copy.sections.iter().all(|section| {
            !section.fields.is_empty() && section.fields_source.as_deref() == Some("pozetek-study")
        }));

        let mut edited = copy.sections.clone();
        edited[0].title = "People (customised)".into();
        let updated = templates::update(
            conn,
            &UpdateTemplate {
                id: copy.id.clone(),
                template: TemplateInput {
                    name: "POZETEK (personal)".into(),
                    description: None,
                    study_type: copy.study_type,
                    sections: edited,
                },
            },
        )?;
        assert_eq!(updated.sections[0].fields, builtin.sections[0].fields);

        let workspace = workspaces::get_default(conn)?;
        let project = projects::create(
            conn,
            &CreateProject {
                workspace_id: workspace.id,
                title: "Customised study".into(),
                description: None,
            },
        )?;
        let study = studies::create(
            conn,
            &CreateStudy {
                project_id: project.id.clone(),
                title: "Hosea 11".into(),
                template_id: Some(copy.id),
                ..Default::default()
            },
        )?;
        assert_eq!(study.sections[0].title, "People (customised)");
        let snapshot = study.sections[0].fields.as_ref().unwrap();
        assert_eq!(snapshot.template, "pozetek-study");
        assert_eq!(snapshot.version, builtin.sections[0].fields_version);
        assert_eq!(snapshot.fields, builtin.sections[0].fields);

        // Mark one structured part as optional in the customised template.
        // Adding it later must retain its fields and origin too.
        let mut later = updated.sections.clone();
        later[1].optional = true;
        templates::update(
            conn,
            &UpdateTemplate {
                id: updated.id.clone(),
                template: TemplateInput {
                    name: updated.name,
                    description: updated.description,
                    study_type: updated.study_type,
                    sections: later,
                },
            },
        )?;
        let deferred = studies::create(
            conn,
            &CreateStudy {
                project_id: project.id,
                title: "Hosea 11 (deferred)".into(),
                template_id: Some(updated.id),
                ..Default::default()
            },
        )?;
        assert_eq!(deferred.sections.len(), 7);
        let place = sections::create(
            conn,
            &CreateStudySection {
                study_id: deferred.study.id,
                title: "Place".into(),
                guidance: None,
                template_section_key: Some("place".into()),

                key: None,
                phase: None,
            },
        )?;
        assert_eq!(place.key.as_deref(), Some("place"));
        assert_eq!(place.fields.as_ref().unwrap().template, "pozetek-study");
        assert!(!place.blocks.is_empty());
        for invalid_key in ["place", "core", "does-not-exist"] {
            let rejected = sections::create(
                conn,
                &CreateStudySection {
                    study_id: place.study_id.clone(),
                    title: "Not allowed".into(),
                    guidance: None,
                    template_section_key: Some(invalid_key.into()),

                    key: None,
                    phase: None,
                },
            );
            assert!(matches!(rejected, Err(AppError::InvalidInput(_))));
        }
        assert_eq!(sections::list(conn, &place.study_id)?.len(), 8);
        Ok(())
    })
    .unwrap();
}

fn make(conn: &Connection) -> AppResult<StudyDetail> {
    let workspace = workspaces::get_default(conn)?;
    let project = projects::create(
        conn,
        &CreateProject {
            workspace_id: workspace.id,
            title: "P".into(),
            description: None,
        },
    )?;
    studies::create(
        conn,
        &CreateStudy {
            project_id: project.id,
            title: "Any passage".into(),
            template_key: Some("pozetek-study".into()),
            ..Default::default()
        },
    )
}

#[test]
fn a_new_study_has_the_parts_in_order_with_empty_answers() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let detail = make(conn)?;
        let keys: Vec<_> = detail
            .sections
            .iter()
            .map(|s| s.key.clone().unwrap())
            .collect();
        assert_eq!(keys, KEYS);
        for section in &detail.sections {
            assert_eq!(section.status, SectionStatus::NotStarted);
            let snapshot = section.fields.as_ref().unwrap();
            assert_eq!(snapshot.template, "pozetek-study");
            assert_eq!(snapshot.version, 1);
            // Every answer is empty; single-text fields have one block each.
            assert!(section.blocks.iter().all(|b| b.content.is_empty()));
            let texts = snapshot
                .fields
                .iter()
                .filter(|f| f.kind == FieldKind::Text)
                .count();
            assert_eq!(section.blocks.len(), texts);
        }
        // Kern: three separate fields.
        let core = &detail.sections[6];
        let fields: Vec<_> = core
            .fields
            .as_ref()
            .unwrap()
            .fields
            .iter()
            .map(|f| f.key.as_str())
            .collect();
        assert_eq!(fields, ["subject", "statement", "theme"]);
        // The personal application is its own part after the core.
        assert_eq!(detail.sections[7].key.as_deref(), Some("application"));
        Ok(())
    })
    .unwrap();
}

#[test]
fn entries_status_and_text_survive_a_restart_and_deleting_keeps_neighbours() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("aw.db");
    let (study_id, keep) = {
        let db = Database::open(&path, &temp.path().join("bk")).unwrap();
        db.with_conn(|conn| {
            let detail = make(conn)?;
            let section = &detail.sections[5];
            let entry = |text: &str| -> AppResult<String> {
                let block = blocks::create(
                    conn,
                    &CreateStudyBlock {
                        section_id: section.id.clone(),
                        block_type: BlockType::Paragraph,
                        content: None,
                        field_key: Some("questions".into()),
                    },
                )?;
                blocks::update(
                    conn,
                    &UpdateStudyBlock {
                        id: block.id.clone(),
                        content: Some(text.into()),
                        data: Some(format!(r#"{{"question":"{text}","status":"open"}}"#)),
                    },
                )?;
                Ok(block.id)
            };
            let first = entry("first")?;
            let second = entry("second")?;
            let third = entry("third")?;
            blocks::delete(conn, &first)?;
            sections::update(
                conn,
                &UpdateStudySection {
                    id: section.id.clone(),
                    status: Some(SectionStatus::InProgress),
                    ..Default::default()
                },
            )?;
            let _ = third;
            Ok((detail.study.id, second))
        })
        .unwrap()
    };
    let db = Database::open(&path, &temp.path().join("bk")).unwrap();
    db.with_conn(|conn| {
        let detail = studies::get_detail(conn, &study_id)?;
        let section = &detail.sections[5];
        assert_eq!(section.status, SectionStatus::InProgress);
        let entries: Vec<_> = section
            .blocks
            .iter()
            .filter(|b| b.field_key.as_deref() == Some("questions"))
            .collect();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, keep);
        assert_eq!(
            entries[0].data.as_deref(),
            Some(r#"{"question":"second","status":"open"}"#)
        );
        assert_eq!(entries[1].content, "third");
        // Other parts are untouched.
        assert_eq!(detail.sections[0].status, SectionStatus::NotStarted);
        Ok(())
    })
    .unwrap();
}

#[test]
fn entries_are_validated() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let detail = make(conn)?;
        let section = &detail.sections[0];
        let create = |key: &str| {
            blocks::create(
                conn,
                &CreateStudyBlock {
                    section_id: section.id.clone(),
                    block_type: BlockType::Paragraph,
                    content: None,
                    field_key: Some(key.into()),
                },
            )
        };
        assert!(matches!(create("nonsense"), Err(AppError::InvalidInput(_))));
        let entry = create("people")?;
        for bad in ["[1]", "not json", r#"{"a":1}"#] {
            let result = blocks::update(
                conn,
                &UpdateStudyBlock {
                    id: entry.id.clone(),
                    content: None,
                    data: Some(bad.into()),
                },
            );
            assert!(matches!(result, Err(AppError::InvalidInput(_))), "{bad}");
        }
        // Free writing cannot carry entry data.
        let free = blocks::create(
            conn,
            &CreateStudyBlock {
                section_id: section.id.clone(),
                block_type: BlockType::Paragraph,
                content: None,
                field_key: None,
            },
        )?;
        let result = blocks::update(
            conn,
            &UpdateStudyBlock {
                id: free.id,
                content: None,
                data: Some("{}".into()),
            },
        );
        assert!(matches!(result, Err(AppError::InvalidInput(_))));
        Ok(())
    })
    .unwrap();
}

#[test]
fn the_snapshot_does_not_follow_a_changed_template_and_duplicates_keep_it() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let detail = make(conn)?;
        // Simulate a later app version that changed the built-in definition.
        conn.execute(
            "UPDATE study_templates SET definition = '{\"sections\":[{\"title\":\"Only\"}]}'
             WHERE key = 'pozetek-study'",
            [],
        )?;
        let again = studies::get_detail(conn, &detail.study.id)?;
        assert_eq!(again.sections.len(), 8);
        assert!(again.sections.iter().all(|s| s.fields.is_some()));
        let _ = templates::list(conn)?;

        // Write something, then duplicate with and without content.
        let section = &again.sections[6];
        let block = &section.blocks[0];
        blocks::update(
            conn,
            &UpdateStudyBlock {
                id: block.id.clone(),
                content: Some("My subject".into()),
                data: None,
            },
        )?;
        sections::update(
            conn,
            &UpdateStudySection {
                id: section.id.clone(),
                status: Some(SectionStatus::Complete),
                ..Default::default()
            },
        )?;
        let copy = |include| {
            studies::duplicate(
                conn,
                &DuplicateStudy {
                    study_id: detail.study.id.clone(),
                    include_content: include,
                },
            )
        };
        let full = copy(true)?;
        assert_eq!(full.sections[6].blocks[0].content, "My subject");
        assert_eq!(
            full.sections[6].blocks[0].field_key.as_deref(),
            Some("subject")
        );
        assert_eq!(full.sections[6].status, SectionStatus::Complete);
        let bare = copy(false)?;
        assert_eq!(bare.sections[6].blocks[0].content, "");
        assert_eq!(
            bare.sections[6].blocks[0].field_key.as_deref(),
            Some("subject")
        );
        assert_eq!(bare.sections[6].status, SectionStatus::NotStarted);
        assert!(bare.sections[6].fields.is_some());
        Ok(())
    })
    .unwrap();
}

fn database_v9(path: &Path) {
    let mut conn = Connection::open(path).unwrap();
    migrations::run_with(&mut conn, &MIGRATIONS[..9]).unwrap();
    let t = "2026-10-27T10:00:00.000Z";
    conn.execute_batch(&format!(
        "INSERT INTO workspaces (id, name, is_default, created_at, updated_at) VALUES ('w', 'W', 1, '{t}', '{t}');
         INSERT INTO projects (id, workspace_id, title, created_at, updated_at) VALUES ('p', 'w', 'C', '{t}', '{t}');
         INSERT INTO studies (id, project_id, title, study_type, created_at, updated_at) VALUES ('s', 'p', 'Old', 'passage', '{t}', '{t}');
         INSERT INTO study_sections (id, study_id, key, title, position, created_at, updated_at) VALUES ('sec', 's', 'meaning', 'Meaning', 0, '{t}', '{t}');
         INSERT INTO study_blocks (id, section_id, block_type, position, content, created_at, updated_at) VALUES ('b', 'sec', 'paragraph', 0, 'Old text', '{t}', '{t}');
         INSERT INTO study_templates (id, key, name, study_type, is_builtin, definition, created_at, updated_at) VALUES ('t', 'user-t', 'Mine', 'passage', 0, '{{\"sections\":[{{\"title\":\"Prayer\"}}]}}', '{t}', '{t}');"
    ))
    .unwrap();
}

#[test]
fn a_v9_database_upgrades_without_loss() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("aw.db");
    database_v9(&path);
    let db = Database::open(&path, &temp.path().join("bk")).unwrap();
    db.with_conn(|conn| {
        assert_eq!(migrations::current_version(conn)?, 24);
        let detail = studies::get_detail(conn, "s")?;
        let section = &detail.sections[0];
        assert_eq!(section.status, SectionStatus::NotStarted);
        assert!(section.fields.is_none());
        assert_eq!(section.blocks[0].content, "Old text");
        assert!(section.blocks[0].field_key.is_none() && section.blocks[0].data.is_none());
        assert_eq!(templates::get(conn, "t")?.sections[0].title, "Prayer");
        assert!(templates::get_by_key(conn, "pozetek-study")?.is_builtin);
        // The old study works as before: edit, add free writing.
        blocks::create(
            conn,
            &CreateStudyBlock {
                section_id: "sec".into(),
                block_type: BlockType::Note,
                content: Some("More".into()),
                field_key: None,
            },
        )?;
        Ok(())
    })
    .unwrap();
    // The automatically created pre-migration backup keeps the old schema
    // and data. Restoring it elsewhere upgrades safely without touching live data.
    let backup_path = std::fs::read_dir(temp.path().join("bk"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let old = Connection::open_with_flags(&backup_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .unwrap();
    assert_eq!(migrations::current_version(&old).unwrap(), 9);
    assert_eq!(
        old.query_row("SELECT content FROM study_blocks WHERE id = 'b'", [], |r| r
            .get::<_, String>(0))
            .unwrap(),
        "Old text"
    );
    drop(old);
    let restored_path = temp.path().join("restored.db");
    std::fs::copy(backup_path, &restored_path).unwrap();
    let restored = Database::open(&restored_path, &temp.path().join("restored-backups")).unwrap();
    restored
        .with_conn(|conn| {
            assert_eq!(migrations::current_version(conn)?, 24);
            assert_eq!(
                studies::get_detail(conn, "s")?.sections[0].blocks[0].content,
                "Old text"
            );
            assert_eq!(templates::get(conn, "t")?.sections[0].title, "Prayer");
            Ok(())
        })
        .unwrap();
}
