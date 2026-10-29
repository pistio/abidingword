//! Synthetic fixtures: reusable blocks must survive personalisation, restart and full backup.
use abiding_word_lib::db::{
    blocks, migrations, projects, studies, templates, workspaces, Database,
};
use abiding_word_lib::domain::{
    BlockType, CreateProject, CreateStudy, CreateStudyBlock, DuplicateStudy, TemplateInput,
    UpdateStudyBlock, UpdateTemplate,
};
use abiding_word_lib::filesystem::DataDirs;
use abiding_word_lib::portability::backup;

#[test]
fn personal_blocks_and_evidence_survive_template_changes_restart_and_backup() {
    let temp = tempfile::tempdir().unwrap();
    let dirs = DataDirs::ensure(temp.path().join("original")).unwrap();
    let db = Database::open(&dirs.database_file(), &dirs.pre_migration_backups()).unwrap();
    let (saved, personal) = db.with_conn(|conn| {
        let project = projects::create(conn, &CreateProject { workspace_id: workspaces::get_default(conn)?.id, title: "Synthetic".into(), description: None })?;
        let mut definition = templates::get_by_key(conn, "word-research-study")?.sections;
        let kinds = ["guidance", "heading", "divider", "passage", "concordance", "cross_references", "word_lookup", "search", "writing"];
        definition[0].fields = kinds.iter().enumerate().map(|(i, kind)| serde_json::from_value(serde_json::json!({"key":format!("block-{i}"),"kind":kind,"label":format!("My {kind}"),"config":{"query":"vine","translationId":"edition-a","guidance":"My unchanged guidance"}})).unwrap()).collect();
        let personal = templates::create(conn, &TemplateInput { name: "My research".into(), description: None, study_type: abiding_word_lib::domain::StudyType::Word, sections: definition })?;
        let study = studies::create(conn, &CreateStudy { project_id: project.id, title: "Kept research".into(), template_id: Some(personal.id.clone()), ..Default::default() })?;
        let part = &study.sections[0];
        assert_eq!(part.blocks.len(), 1, "only writing creates an empty answer");
        blocks::update(conn, &UpdateStudyBlock { id: part.blocks[0].id.clone(), content: Some("My own conclusion — 한글".into()), data: None })?;
        blocks::create_with_data(conn, &CreateStudyBlock { section_id: part.id.clone(), block_type: BlockType::Quotation, field_key: Some("block-7".into()), content: Some("JHN 15:5 (SRC)\nKept source text".into()) }, Some(r#"{"kind":"search","query":"vine","translationId":"edition-a","translationName":"Source edition (SRC)","reference":"JHN 15:5","text":"Kept source text"}"#))?;
        let saved = studies::get_detail(conn, &study.study.id)?;
        let full = studies::duplicate(conn, &DuplicateStudy { study_id: saved.study.id.clone(), include_content: true })?;
        assert_eq!(full.sections[0].fields, saved.sections[0].fields);
        assert_eq!(full.sections[0].blocks.iter().map(|block| (&block.content, &block.data)).collect::<Vec<_>>(), saved.sections[0].blocks.iter().map(|block| (&block.content, &block.data)).collect::<Vec<_>>());
        let structure = studies::duplicate(conn, &DuplicateStudy { study_id: saved.study.id.clone(), include_content: false })?;
        assert_eq!(structure.sections[0].fields, saved.sections[0].fields);
        assert_eq!(structure.sections[0].blocks.len(), 1);
        assert!(structure.sections.iter().flat_map(|section| &section.blocks).all(|block| block.content.is_empty() && block.data.is_none()));
        let mut changed = personal.sections.clone();
        changed[0].fields.reverse();
        changed[0].fields[0].label = Some("Changed only for future studies".into());
        let personal = templates::update(conn, &UpdateTemplate { id: personal.id, template: TemplateInput { name: personal.name, description: personal.description, study_type: personal.study_type, sections: changed } })?;
        templates::ensure_builtin(conn)?;
        templates::ensure_builtin(conn)?;
        assert_eq!(studies::get_detail(conn, &saved.study.id)?, saved);
        assert_eq!(templates::get(conn, &personal.id)?, personal);
        assert_eq!(migrations::current_version(conn)?, 26);
        migrations::validate_existing_history(conn)?;
        Ok((saved, personal))
    }).unwrap();
    let archive = temp.path().join("blocks.awbackup");
    backup::create(&db, &dirs, &archive).unwrap();
    drop(db);
    let reopened = Database::open(&dirs.database_file(), &dirs.pre_migration_backups()).unwrap();
    reopened
        .with_conn(|conn| {
            assert_eq!(studies::get_detail(conn, &saved.study.id)?, saved);
            Ok(())
        })
        .unwrap();
    let target_dirs = DataDirs::ensure(temp.path().join("restored")).unwrap();
    let target = Database::open(
        &target_dirs.database_file(),
        &target_dirs.pre_migration_backups(),
    )
    .unwrap();
    backup::restore(&target, &target_dirs, &archive).unwrap();
    target
        .with_conn(|conn| {
            assert_eq!(studies::get_detail(conn, &saved.study.id)?, saved);
            assert_eq!(templates::get(conn, &personal.id)?, personal);
            assert_eq!(migrations::current_version(conn)?, 26);
            migrations::validate_existing_history(conn)?;
            Ok(())
        })
        .unwrap();
}

#[test]
fn legacy_definitions_and_new_writing_templates_coexist_and_reject_ambiguous_blocks() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        for (key, count) in [
            ("pozetek-study", 8),
            ("ten-step-exegesis", 10),
            ("pozetek-writing-study", 7),
            ("ten-step-writing-study", 10),
        ] {
            let template = templates::get_by_key(conn, key)?;
            assert_eq!(template.sections.len(), count);
            if key.contains("writing") {
                assert!(template
                    .sections
                    .iter()
                    .all(|section| section.fields.len() == 1));
            }
        }
        let source = templates::get_by_key(conn, "word-research-study")?;
        let mut legacy_sections = source.sections.clone();
        legacy_sections[0].fields[0].key = "legacy_identifier_한글".into();
        let preserved = templates::create(
            conn,
            &TemplateInput {
                name: "Legacy identifiers".into(),
                description: None,
                study_type: source.study_type,
                sections: legacy_sections.clone(),
            },
        )?;
        assert_eq!(preserved.sections, legacy_sections);
        let mut sections = source.sections;
        let duplicate = sections[0].fields[0].clone();
        sections[0].fields.push(duplicate);
        assert!(templates::create(
            conn,
            &TemplateInput {
                name: "Ambiguous".into(),
                description: None,
                study_type: source.study_type,
                sections
            }
        )
        .is_err());
        Ok(())
    })
    .unwrap();
}

#[test]
fn evidence_create_is_atomic_and_rejects_invalid_metadata_before_writing() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let project = projects::create(conn, &CreateProject { workspace_id: workspaces::get_default(conn)?.id, title: "Atomic research".into(), description: None })?;
        let study = studies::create(conn, &CreateStudy { project_id: project.id, title: "Atomic".into(), template_key: Some("word-research-study".into()), ..Default::default() })?;
        let input = CreateStudyBlock { section_id: study.sections[0].id.clone(), block_type: BlockType::Quotation, content: Some("JHN.15.1 (SRC)\nExact source".into()), field_key: Some("passage".into()) };
        let data = r#"{"kind":"passage","query":"John 15:1–3","reference":"JHN.15.1","translationId":"source","translationName":"Source Bible (SRC)","text":"Exact source"}"#;
        let before = studies::get_detail(conn, &study.study.id)?;
        for invalid in ["[]", "{\"query\":3}", "not JSON"] {
            assert!(blocks::create_with_data(conn, &input, Some(invalid)).is_err());
            assert_eq!(studies::get_detail(conn, &study.study.id)?, before);
        }
        let mut unkeyed = input.clone();
        unkeyed.field_key = None;
        assert!(blocks::create_with_data(conn, &unkeyed, Some(data)).is_err());
        conn.execute_batch("CREATE TRIGGER fail_evidence_touch BEFORE UPDATE ON studies BEGIN SELECT RAISE(ABORT, 'simulated failed write'); END;")?;
        assert!(blocks::create_with_data(conn, &input, Some(data)).is_err());
        assert_eq!(studies::get_detail(conn, &study.study.id)?, before, "the insert must roll back if touching the study fails");
        conn.execute_batch("DROP TRIGGER fail_evidence_touch;")?;
        let kept = blocks::create_with_data(conn, &input, Some(data))?;
        assert_eq!(kept.content, input.content.unwrap());
        assert_eq!(kept.data.as_deref(), Some(data));
        let after = studies::get_detail(conn, &study.study.id)?;
        assert_eq!(after.sections[0].blocks.len(), before.sections[0].blocks.len() + 1);
        Ok(())
    }).unwrap();
}

#[test]
fn block_command_payload_preserves_old_calls_and_accepts_atomic_provenance() {
    use abiding_word_lib::domain::CreateStudyBlockRequest;
    let old: CreateStudyBlockRequest = serde_json::from_value(serde_json::json!({"sectionId":"section","blockType":"paragraph","content":"Legacy answer"})).unwrap();
    assert_eq!(old.block.content.as_deref(), Some("Legacy answer"));
    assert!(old.data.is_none());
    let new: CreateStudyBlockRequest = serde_json::from_value(serde_json::json!({"sectionId":"section","blockType":"quotation","fieldKey":"passage","content":"Exact quote","data":"{\"reference\":\"JHN.15.1\"}"})).unwrap();
    assert_eq!(new.block.field_key.as_deref(), Some("passage"));
    assert_eq!(new.data.as_deref(), Some("{\"reference\":\"JHN.15.1\"}"));
}
