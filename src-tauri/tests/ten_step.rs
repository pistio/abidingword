//! The Ten-Step Exegesis built-in: ordered, structured, empty on creation,
//! persistent after editing, and safely copyable. The optional Matthew 8
//! illustration is in the UI translation resources, never in the study data.

use abiding_word_lib::db::{blocks, projects, sections, studies, templates, workspaces, Database};
use abiding_word_lib::domain::{
    BlockType, CreateProject, CreateStudy, CreateStudyBlock, FieldKind, SectionStatus,
    TemplateInput, UpdateStudyBlock, UpdateStudySection, UpdateTemplate,
};
use abiding_word_lib::error::AppResult;

const KEY: &str = "ten-step-exegesis";
const PARTS: [&str; 10] = [
    "step-01-familiarity",
    "step-02-text",
    "step-03-situation",
    "step-04-context",
    "step-05-genre",
    "step-06-language",
    "step-07-flow",
    "step-08-theology",
    "step-09-message",
    "step-10-application",
];

fn make(conn: &rusqlite::Connection, template_id: Option<String>) -> AppResult<String> {
    let workspace = workspaces::get_default(conn)?;
    let project = projects::create(
        conn,
        &CreateProject {
            workspace_id: workspace.id,
            title: "Matthew".into(),
            description: None,
        },
    )?;
    let study = studies::create(
        conn,
        &CreateStudy {
            project_id: project.id,
            title: "Matthew 8:18–27".into(),
            template_id,
            template_key: Some(KEY.into()),
            ..Default::default()
        },
    )?;
    Ok(study.study.id)
}

#[test]
fn the_ten_steps_have_stable_fields_and_start_without_example_answers() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let builtin = templates::get_by_key(conn, KEY)?;
        assert!(builtin.is_builtin);
        assert_eq!(builtin.sections.len(), PARTS.len());
        assert!(builtin
            .sections
            .iter()
            .all(|part| !part.optional && !part.retired));

        let study_id = make(conn, None)?;
        let study = studies::get_detail(conn, &study_id)?;
        let keys: Vec<&str> = study
            .sections
            .iter()
            .map(|s| s.key.as_deref().unwrap())
            .collect();
        assert_eq!(keys, PARTS);
        for section in &study.sections {
            assert_eq!(section.status, SectionStatus::NotStarted);
            let snapshot = section.fields.as_ref().unwrap();
            assert_eq!(snapshot.template, KEY);
            assert_eq!(snapshot.version, 1);
            assert!(!snapshot.fields.is_empty());
            assert_eq!(
                section.blocks.len(),
                snapshot
                    .fields
                    .iter()
                    .filter(|f| f.kind == FieldKind::Text)
                    .count()
            );
            assert!(section
                .blocks
                .iter()
                .all(|b| b.content.is_empty() && b.data.is_none()));
        }

        let fields: Vec<&str> = study.sections[8]
            .fields
            .as_ref()
            .unwrap()
            .fields
            .iter()
            .map(|f| f.key.as_str())
            .collect();
        assert_eq!(fields, ["subject", "statement", "theme", "text-check"]);
        assert_eq!(
            study.sections[9].key.as_deref(),
            Some("step-10-application")
        );
        Ok(())
    })
    .unwrap();
}

#[test]
fn a_customised_copy_retains_its_fields_and_reference_to_built_in_text() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let builtin = templates::get_by_key(conn, KEY)?;
        let copy = templates::duplicate(conn, &builtin.id)?;
        assert!(!copy.is_builtin);
        assert!(copy
            .sections
            .iter()
            .all(|s| s.fields_source.as_deref() == Some(KEY)));
        let mut parts = copy.sections;
        parts[8].title = "Central claim".into();
        let changed = templates::update(
            conn,
            &UpdateTemplate {
                id: copy.id.clone(),
                template: TemplateInput {
                    name: "My ten steps".into(),
                    description: None,
                    study_type: copy.study_type,
                    sections: parts,
                },
            },
        )?;
        let study_id = make(conn, Some(changed.id))?;
        let detail = studies::get_detail(conn, &study_id)?;
        assert_eq!(detail.sections[8].title, "Central claim");
        let snapshot = detail.sections[8].fields.as_ref().unwrap();
        assert_eq!(snapshot.template, KEY);
        assert_eq!(snapshot.fields, builtin.sections[8].fields);
        Ok(())
    })
    .unwrap();
}

#[test]
fn text_entries_and_progress_remain_after_database_reopen() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("ten-steps.db");
    let (study_id, entry_id) = {
        let db = Database::open(&path, &temp.path().join("bk")).unwrap();
        db.with_conn(|conn| {
            let study_id = make(conn, None)?;
            let detail = studies::get_detail(conn, &study_id)?;
            let question_part = &detail.sections[7];
            let new_entry = blocks::create(
                conn,
                &CreateStudyBlock {
                    section_id: question_part.id.clone(),
                    block_type: BlockType::Paragraph,
                    content: None,
                    field_key: Some("questions".into()),
                },
            )?;
            blocks::update(
                conn,
                &UpdateStudyBlock {
                    id: new_entry.id.clone(),
                    content: Some("What does Jesus mean?".into()),
                    data: Some(r#"{"question":"What does Jesus mean?","status":"open"}"#.into()),
                },
            )?;
            let core = &detail.sections[8];
            let theme_block = core
                .blocks
                .iter()
                .find(|b| b.field_key.as_deref() == Some("theme"))
                .unwrap();
            blocks::update(
                conn,
                &UpdateStudyBlock {
                    id: theme_block.id.clone(),
                    content: Some("Trust Christ in danger.".into()),
                    data: None,
                },
            )?;
            sections::update(
                conn,
                &UpdateStudySection {
                    id: core.id.clone(),
                    status: Some(SectionStatus::InProgress),
                    ..Default::default()
                },
            )?;
            Ok((study_id, new_entry.id))
        })
        .unwrap()
    };
    let db = Database::open(&path, &temp.path().join("bk")).unwrap();
    db.with_conn(|conn| {
        let detail = studies::get_detail(conn, &study_id)?;
        let entry = detail.sections[7]
            .blocks
            .iter()
            .find(|b| b.id == entry_id)
            .unwrap();
        assert_eq!(entry.field_key.as_deref(), Some("questions"));
        assert_eq!(
            entry.data.as_deref(),
            Some(r#"{"question":"What does Jesus mean?","status":"open"}"#)
        );
        let core = &detail.sections[8];
        assert_eq!(core.status, SectionStatus::InProgress);
        assert_eq!(
            core.blocks
                .iter()
                .find(|b| b.field_key.as_deref() == Some("theme"))
                .unwrap()
                .content,
            "Trust Christ in danger."
        );
        Ok(())
    })
    .unwrap();
}
