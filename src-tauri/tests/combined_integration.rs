//! Invented data only: adopting Study flow on an existing schema-25 Teaching
//! database must preserve both models and must not rerun migrations.
use abiding_word_lib::db::{
    blocks, migrations, projects, studies, teaching, templates, workspaces, Database,
};
use abiding_word_lib::domain::{
    BlockType, CreateProject, CreateStudy, CreateStudyBlock, CreateTeaching, NewTeachingBlock,
    UpdateStudyBlock,
};

#[test]
fn existing_v25_teaching_and_structured_answers_survive_study_flow_startup() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("synthetic.db");
    let backups = temp.path().join("backups");
    let db = Database::open(&path, &backups).unwrap();
    let (saved_study, saved_teaching) = db
        .with_conn(|conn| {
            let project = projects::create(
                conn,
                &CreateProject {
                    workspace_id: workspaces::get_default(conn)?.id,
                    title: "Synthetic integration".into(),
                    description: None,
                },
            )?;
            let study = studies::create(
                conn,
                &CreateStudy {
                    project_id: project.id,
                    title: "Existing structured study".into(),
                    template_key: Some("pozetek-study".into()),
                    ..Default::default()
                },
            )?;
            blocks::create(
                conn,
                &CreateStudyBlock {
                    section_id: study.sections[0].id.clone(),
                    block_type: BlockType::Paragraph,
                    field_key: None,
                    content: Some("Existing answer — 한글".into()),
                },
            )?;
            let prep = teaching::create(
                conn,
                &CreateTeaching {
                    study_id: study.study.id.clone(),
                    title: "Existing block teaching".into(),
                    kind: Some("sermon".into()),
                    template_key: Some("sermon-talk".into()),
                    template_version: Some(2),
                    blocks: vec![NewTeachingBlock {
                        block_type: "writing".into(),
                        title: "Message".into(),
                        body: "Independent Teaching manuscript".into(),
                        role: Some("central_message".into()),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
            )?;
            // These six definitions did not exist in the Teaching-only application.
            conn.execute(
                "DELETE FROM study_templates WHERE key IN (
            'pozetek-writing-study','ten-step-writing-study','word-research-study',
            'book-research-study','character-research-study','topical-research-study')",
                [],
            )?;
            assert_eq!(migrations::current_version(conn)?, 25);
            Ok((studies::get_detail(conn, &study.study.id)?, prep))
        })
        .unwrap();
    drop(db);

    for _ in 0..2 {
        let db = Database::open(&path, &backups).unwrap();
        db.with_conn(|conn| {
            migrations::validate_existing_history(conn)?;
            assert_eq!(migrations::current_version(conn)?, 25);
            assert_eq!(
                studies::get_detail(conn, &saved_study.study.id)?,
                saved_study
            );
            assert_eq!(teaching::get(conn, &saved_teaching.id)?, saved_teaching);
            assert_eq!(
                templates::get_by_key(conn, "pozetek-writing-study")?
                    .sections
                    .len(),
                7
            );
            assert_eq!(
                templates::get_by_key(conn, "ten-step-writing-study")?
                    .sections
                    .len(),
                10
            );
            Ok(())
        })
        .unwrap();
    }
    assert!(!backups.exists() || std::fs::read_dir(&backups).unwrap().count() == 0);

    let db = Database::open(&path, &backups).unwrap();
    db.with_conn(|conn| {
        let answer = saved_study.sections[0].blocks.last().unwrap();
        blocks::update(
            conn,
            &UpdateStudyBlock {
                id: answer.id.clone(),
                content: Some("Changed Study answer only".into()),
                data: None,
            },
        )?;
        assert_eq!(teaching::get(conn, &saved_teaching.id)?, saved_teaching);
        assert_ne!(
            studies::get_detail(conn, &saved_study.study.id)?,
            saved_study
        );
        Ok(())
    })
    .unwrap();
}
