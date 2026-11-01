//! Invented studies only. Restoring structure never resurrects deleted writing.
use abiding_word_lib::db::{blocks, projects, sections, studies, templates, workspaces, Database};
use abiding_word_lib::domain::{
    CreateProject, CreateStudy, CreateStudySection, DeleteStudySection, Reorder, UpdateStudyBlock,
};

#[test]
fn original_method_parts_restore_empty_in_order_without_losing_survivors_or_custom_parts() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let project = projects::create(
            conn,
            &CreateProject {
                workspace_id: workspaces::get_default(conn)?.id,
                title: "Disposable method recovery".into(),
                description: None,
            },
        )?;
        for key in [
            "pozetek-study",
            "pozetek-writing-study",
            "ten-step-exegesis",
            "ten-step-writing-study",
        ] {
            let template = templates::get_by_key(conn, key)?;
            let detail = studies::create(
                conn,
                &CreateStudy {
                    project_id: project.id.clone(),
                    title: key.into(),
                    template_id: Some(template.id),
                    ..Default::default()
                },
            )?;
            let original = detail.sections;
            let removed = original[2].clone();
            blocks::update(
                conn,
                &UpdateStudyBlock {
                    id: original[0].blocks[0].id.clone(),
                    content: Some("Surviving research — 한글".into()),
                    data: None,
                },
            )?;
            blocks::update(
                conn,
                &UpdateStudyBlock {
                    id: removed.blocks[0].id.clone(),
                    content: Some("Deleted writing must not reappear".into()),
                    data: None,
                },
            )?;
            sections::delete(
                conn,
                &DeleteStudySection {
                    id: removed.id.clone(),
                    force: true,
                },
            )?;
            let custom = sections::create(
                conn,
                &CreateStudySection {
                    study_id: detail.study.id.clone(),
                    title: "My own part".into(),
                    guidance: Some("My own guidance".into()),
                    key: None,
                    phase: None,
                    template_section_key: None,
                },
            )?;
            let mut current = sections::list(conn, &detail.study.id)?;
            current.reverse();
            sections::reorder(
                conn,
                &Reorder {
                    parent_id: detail.study.id.clone(),
                    ids: current.iter().map(|part| part.id.clone()).collect(),
                },
            )?;
            let restored = sections::restore_missing(
                conn,
                &detail.study.id,
                Some(&[removed.key.clone().unwrap()]),
            )?;
            let recovered = restored
                .iter()
                .find(|part| part.key == removed.key)
                .unwrap();
            assert_ne!(recovered.id, removed.id);
            assert_eq!(recovered.fields, removed.fields);
            assert_eq!(recovered.guidance, removed.guidance);
            assert_eq!(recovered.phase, removed.phase);
            assert!(recovered
                .blocks
                .iter()
                .all(|block| block.content.is_empty()));
            assert_eq!(
                restored[0].id, custom.id,
                "custom slot survives method reordering"
            );
            assert_eq!(
                restored
                    .iter()
                    .find(|part| part.id == original[0].id)
                    .unwrap()
                    .blocks[0]
                    .content,
                "Surviving research — 한글"
            );
            assert_eq!(
                restored
                    .iter()
                    .filter_map(|part| part.key.as_deref())
                    .collect::<Vec<_>>(),
                original
                    .iter()
                    .filter_map(|part| part.key.as_deref())
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                sections::restore_missing(conn, &detail.study.id, None)?,
                restored,
                "idempotent; no duplicates or rewriting surviving metadata"
            );
            assert!(sections::restore_missing(
                conn,
                &detail.study.id,
                Some(&["not-a-part".into()])
            )
            .is_err());
            assert_eq!(sections::list(conn, &detail.study.id)?, restored);
            for part in &restored {
                sections::delete(
                    conn,
                    &DeleteStudySection {
                        id: part.id.clone(),
                        force: true,
                    },
                )?;
            }
            let all = sections::restore_missing(conn, &detail.study.id, None)?;
            assert_eq!(all.len(), original.len());
            assert!(all.iter().all(|part| !part.blocks.is_empty()
                && part.blocks.iter().all(|block| block.content.is_empty())));
            assert_eq!(
                conn.query_row(
                    "SELECT content FROM study_blocks WHERE id=?1 AND deleted_at IS NOT NULL",
                    [&removed.blocks[0].id],
                    |row| row.get::<_, String>(0)
                )?,
                "Deleted writing must not reappear"
            );
        }
        // A user's copied method is their own template, even with original keys.
        let builtin = templates::get_by_key(conn, "pozetek-writing-study")?;
        let custom = templates::duplicate(conn, &builtin.id)?;
        let owned = studies::create(
            conn,
            &CreateStudy {
                project_id: project.id.clone(),
                title: "My customised method".into(),
                template_id: Some(custom.id),
                ..Default::default()
            },
        )?;
        sections::delete(
            conn,
            &DeleteStudySection {
                id: owned.sections[2].id.clone(),
                force: false,
            },
        )?;
        let before = sections::list(conn, &owned.study.id)?;
        assert!(sections::restore_missing(conn, &owned.study.id, None).is_err());
        assert_eq!(
            sections::list(conn, &owned.study.id)?,
            before,
            "never impose built-in restoration on a user's copied template"
        );
        assert!(!conn.prepare("PRAGMA foreign_key_check")?.exists([])?);
        Ok(())
    })
    .unwrap();
}
