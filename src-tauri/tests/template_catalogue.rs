//! Catalogue workflows and preservation across copies, backups and reopening.
use abiding_word_lib::db::{
    backup, blocks, projects, sections, studies, study_search, templates, workspaces, Database,
};
use abiding_word_lib::domain::{
    CreateProject, CreateStudy, CreateStudySection, DuplicateStudy, ListStudies, SectionStatus,
    StudyType, UpdateStudyBlock, UpdateStudySection, UpdateTemplate,
};
use abiding_word_lib::error::AppResult;
use rusqlite::Connection;

fn project(conn: &Connection) -> AppResult<String> {
    Ok(projects::create(
        conn,
        &CreateProject {
            workspace_id: workspaces::get_default(conn)?.id,
            title: "Template regressions".into(),
            description: None,
        },
    )?
    .id)
}

#[test]
fn all_six_templates_create_empty_studies_with_stable_keys_and_types() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let project_id = project(conn)?;
        for (key, kind, count) in [
            ("pozetek-study", StudyType::Passage, 8),
            ("ten-step-exegesis", StudyType::Passage, 10),
            ("biblical-character-study", StudyType::Character, 7),
            ("word-study", StudyType::Word, 6),
            ("whole-book-study", StudyType::Book, 7),
            ("topical-study", StudyType::Topical, 6),
        ] {
            let template = templates::get_by_key(conn, key)?;
            let study = studies::create(
                conn,
                &CreateStudy {
                    project_id: project_id.clone(),
                    title: key.into(),
                    study_type: Some(kind),
                    template_key: Some(key.into()),
                    ..Default::default()
                },
            )?;
            assert_eq!(study.study.study_type, kind);
            assert_eq!(study.study.template_id.as_ref(), Some(&template.id));
            assert_eq!(study.sections.len(), count);
            assert_eq!(
                study
                    .sections
                    .iter()
                    .map(|s| s.key.clone())
                    .collect::<Vec<_>>(),
                template
                    .sections
                    .iter()
                    .filter(|s| s.starts())
                    .map(|s| s.key.clone())
                    .collect::<Vec<_>>()
            );
            assert!(study
                .sections
                .iter()
                .all(|s| s.status == SectionStatus::NotStarted
                    && s.blocks
                        .iter()
                        .all(|b| b.content.is_empty() && b.data.is_none())));
        }
        // Template-only callers also get the template's subject, including personal copies.
        for kind in [StudyType::Character, StudyType::Book] {
            let builtin = templates::get_by_key(conn, templates::builtin_key_for(kind).unwrap())?;
            let personal = templates::duplicate(conn, &builtin.id)?;
            for template in [builtin, personal] {
                let created = studies::create(
                    conn,
                    &CreateStudy {
                        project_id: project_id.clone(),
                        title: "Inferred subject".into(),
                        template_id: Some(template.id),
                        ..Default::default()
                    },
                )?;
                assert_eq!(created.study.study_type, kind);
            }
        }
        // Legacy keys still resolve, including legacy defaults used by older callers.
        for key in [
            "passage-study",
            "biblical-theology-study",
            "systematic-theology-study",
            "teaching-study",
        ] {
            assert!(templates::get_by_key(conn, key)?.is_builtin);
        }
        Ok(())
    })
    .unwrap();
}

#[test]
fn character_and_book_workflows_survive_custom_templates_deletion_backup_and_restore() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("live.db");
    let db = Database::open(&path, &temp.path().join("before-upgrade")).unwrap();
    let (ids, backup_path) = db
        .with_conn(|conn| {
            let project_id = project(conn)?;
            let mut ids = Vec::new();
            for kind in [StudyType::Character, StudyType::Book] {
                let key = templates::builtin_key_for(kind).unwrap();
                let builtin = templates::get_by_key(conn, key)?;
                let mut own = templates::duplicate(conn, &builtin.id)?;
                own.sections[0].title = "My original heading".into();
                let own = templates::update(
                    conn,
                    &UpdateTemplate {
                        id: own.id,
                        template: abiding_word_lib::domain::TemplateInput {
                            name: own.name,
                            description: own.description,
                            study_type: kind,
                            sections: own.sections,
                        },
                    },
                )?;
                let detail = studies::create(
                    conn,
                    &CreateStudy {
                        project_id: project_id.clone(),
                        title: format!("My {}", kind.as_str()),
                        study_type: Some(kind),
                        template_id: Some(own.id.clone()),
                        ..Default::default()
                    },
                )?;
                let first = &detail.sections[0];
                blocks::update(
                    conn,
                    &UpdateStudyBlock {
                        id: first.blocks[0].id.clone(),
                        content: Some("Evidence preserved".into()),
                        data: None,
                    },
                )?;
                sections::update(
                    conn,
                    &UpdateStudySection {
                        id: first.id.clone(),
                        status: Some(SectionStatus::Complete),
                        ..Default::default()
                    },
                )?;
                let optional = own.sections.iter().find(|s| s.optional).unwrap();
                let added = sections::create(
                    conn,
                    &CreateStudySection {
                        study_id: detail.study.id.clone(),
                        title: optional.title.clone(),
                        guidance: optional.guidance.clone(),
                        template_section_key: optional.key.clone(),
                    },
                )?;
                assert_eq!(added.key, optional.key);
                assert!(sections::create(
                    conn,
                    &CreateStudySection {
                        study_id: detail.study.id.clone(),
                        title: optional.title.clone(),
                        guidance: None,
                        template_section_key: optional.key.clone(),
                    }
                )
                .is_err());
                assert_eq!(
                    studies::list(
                        conn,
                        &ListStudies {
                            study_type: Some(kind),
                            ..Default::default()
                        }
                    )?
                    .len(),
                    1
                );
                assert!(study_search::search(conn, "preserved")?
                    .iter()
                    .any(|h| h.study_id == detail.study.id));
                let full = studies::duplicate(
                    conn,
                    &DuplicateStudy {
                        study_id: detail.study.id.clone(),
                        include_content: true,
                    },
                )?;
                let empty = studies::duplicate(
                    conn,
                    &DuplicateStudy {
                        study_id: detail.study.id.clone(),
                        include_content: false,
                    },
                )?;
                assert_eq!(full.sections[0].blocks[0].content, "Evidence preserved");
                assert_eq!(full.sections[0].status, SectionStatus::Complete);
                assert!(empty
                    .sections
                    .iter()
                    .all(|s| s.status == SectionStatus::NotStarted
                        && s.blocks.iter().all(|b| b.content.is_empty())));
                templates::delete(conn, &own.id)?;
                templates::ensure_builtin(conn)?;
                let saved = studies::get_detail(conn, &detail.study.id)?;
                assert_eq!(saved.sections[0].title, "My original heading");
                assert_eq!(saved.sections.len(), 8);
                ids.push(detail.study.id);
            }
            let path = backup::pre_migration(conn, &temp.path().join("snapshot"), 10)?;
            Ok((ids, path))
        })
        .unwrap();
    drop(db);
    // Restore a verified backup to a different location; never overwrite the live file.
    let restore = temp.path().join("restored.db");
    std::fs::copy(backup_path, &restore).unwrap();
    for path in [&path, &restore] {
        let reopened = Database::open(path, &temp.path().join("restored-backups")).unwrap();
        reopened
            .with_conn(|conn| {
                for (id, kind) in ids.iter().zip([StudyType::Character, StudyType::Book]) {
                    let detail = studies::get_detail(conn, id)?;
                    assert_eq!(detail.study.study_type, kind);
                    assert_eq!(detail.sections[0].title, "My original heading");
                    assert_eq!(detail.sections[0].blocks[0].content, "Evidence preserved");
                    assert_eq!(detail.sections[0].status, SectionStatus::Complete);
                    assert_eq!(detail.sections.len(), 8);
                }
                Ok(())
            })
            .unwrap();
    }
}

#[test]
fn copying_a_completed_part_without_blocks_keeps_its_progress() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let detail = studies::create(
            conn,
            &CreateStudy {
                project_id: project(conn)?,
                title: "Empty part".into(),
                template_key: Some("pozetek-study".into()),
                ..Default::default()
            },
        )?;
        let part = &detail.sections[0];
        for block in &part.blocks {
            blocks::delete(conn, &block.id)?;
        }
        sections::update(
            conn,
            &UpdateStudySection {
                id: part.id.clone(),
                status: Some(SectionStatus::Complete),
                ..Default::default()
            },
        )?;
        for include_content in [true, false] {
            let copied = studies::duplicate(
                conn,
                &DuplicateStudy {
                    study_id: detail.study.id.clone(),
                    include_content,
                },
            )?;
            let part_copy = copied.sections.iter().find(|s| s.key == part.key).unwrap();
            assert_eq!(
                part_copy.status,
                if include_content {
                    SectionStatus::Complete
                } else {
                    SectionStatus::NotStarted
                }
            );
            if include_content {
                assert!(part_copy.blocks.is_empty());
            } else {
                assert!(part_copy.blocks.iter().all(|b| b.content.is_empty()));
            }
        }
        Ok(())
    })
    .unwrap();
}

#[test]
fn entry_data_without_display_text_is_not_silently_deleted() {
    use abiding_word_lib::domain::{BlockType, CreateStudyBlock, DeleteStudySection};
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let detail = studies::create(
            conn,
            &CreateStudy {
                project_id: project(conn)?,
                title: "Entry data".into(),
                template_key: Some("pozetek-study".into()),
                ..Default::default()
            },
        )?;
        let part = detail
            .sections
            .iter()
            .find(|s| s.key.as_deref() == Some("explanations"))
            .unwrap();
        let entry = blocks::create(
            conn,
            &CreateStudyBlock {
                section_id: part.id.clone(),
                block_type: BlockType::Paragraph,
                field_key: Some("questions".into()),
                content: None,
            },
        )?;
        blocks::update(
            conn,
            &UpdateStudyBlock {
                id: entry.id.clone(),
                content: None,
                data: Some(r#"{"status":"answered"}"#.into()),
            },
        )?;
        assert!(sections::has_content(conn, &part.id)?);
        assert!(sections::delete(
            conn,
            &DeleteStudySection {
                id: part.id.clone(),
                force: false
            }
        )
        .is_err());
        assert_eq!(
            blocks::get(conn, &entry.id)?.data.as_deref(),
            Some(r#"{"status":"answered"}"#)
        );
        Ok(())
    })
    .unwrap();
}
