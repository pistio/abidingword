//! A.4.4 (#27): paired Question + Answer blocks (no migration; schema 27) and
//! explicit "Add from Bible Study" into a Teaching: editable snapshots with
//! provenance, leader-only by default, never synchronised, never leaking
//! another study, all or nothing. Invented data only.
use abiding_word_lib::db::{
    blocks, findings, projects, studies, study_passages, teaching, teaching_blocks, workspaces,
    Database,
};
use abiding_word_lib::domain::teaching::{AddFromStudy, StudyItemRef};
use abiding_word_lib::domain::{
    BlockType, CreateFinding, CreateProject, CreateStudy, CreateStudyBlock, CreateTeaching,
    FindingType, NewTeachingBlock, UpdateStudyBlock, UpdateTeachingBlock,
};
use abiding_word_lib::error::AppError;
use abiding_word_lib::scripture::AddStudyPassage;
use rusqlite::Connection;

struct World {
    db: Database,
    study: String,
    other: String,
    prep: String,
    writing: String,
    qa: String,
    finding: String,
    secret: String,
    citation: String,
    passage: String,
}

fn qa(question: &str, answer: &str) -> String {
    serde_json::json!({"kind": "qa", "question": question, "answer": answer}).to_string()
}

fn world() -> World {
    let db = Database::open_in_memory().unwrap();
    let ids = db
        .with_conn(|conn| {
            let ws = workspaces::get_default(conn)?;
            let project = projects::create(conn, &CreateProject { workspace_id: ws.id, title: "Römerbrief".into(), description: None })?;
            let study = studies::create(conn, &CreateStudy { project_id: project.id.clone(), title: "Römer 8".into(), ..Default::default() })?;
            let other = studies::create(conn, &CreateStudy { project_id: project.id.clone(), title: "Privat".into(), ..Default::default() })?;
            let section = study.sections[0].id.clone();
            let writing = blocks::create(conn, &CreateStudyBlock { section_id: section.clone(), block_type: BlockType::Paragraph, content: Some("Kein Verdammungsurteil.".into()), field_key: None })?;
            let qa_block = blocks::create_with_data(conn, &CreateStudyBlock { section_id: section.clone(), block_type: BlockType::Question, content: None, field_key: None }, Some(&qa("Wer verdammt?", "Niemand, Christus tritt für uns ein.")))?;
            let finding = findings::create(conn, &CreateFinding { study_id: study.study.id.clone(), finding_type: FindingType::Observation, content: Some("Der Geist bezeugt.".into()), section_id: None, passage: None })?;
            let secret = findings::create(conn, &CreateFinding { study_id: other.study.id.clone(), finding_type: FindingType::Observation, content: Some("SECRET-OTHER-STUDY".into()), section_id: None, passage: None })?;
            conn.execute_batch(&format!(
                "INSERT INTO sources (id, workspace_id, source_type, title, author, created_at, updated_at, passage_index_version)
                     VALUES ('src-1', (SELECT id FROM workspaces LIMIT 1), 'book', 'Erfundenes Buch', 'A. Autor', 'T', 'T', 4);
                 INSERT INTO finding_sources (id, finding_id, source_id, locator, cited_title, cited_author, quote, created_at, updated_at)
                     VALUES ('cite-1', '{}', 'src-1', 'S. 12', 'Erfundenes Buch', 'A. Autor', 'Erfundenes Zitat.', 'T', 'T');",
                finding.id
            ))?;
            let passage = study_passages::add(conn, &AddStudyPassage { study_id: study.study.id.clone(), reference: "ROM.8.1-ROM.8.11".into(), role: None })?;
            let prep = teaching::create(
                conn,
                &CreateTeaching {
                    project_id: None,
                    study_id: study.study.id.clone(),
                    title: "Hauskreis".into(),
                    kind: Some("bible_study".into()),
                    points: Vec::new(),
                    template_key: Some("eva".into()),
                    template_version: Some(2),
                    blocks: vec![NewTeachingBlock { key: Some("w".into()), block_type: "writing".into(), title: "Erkennen".into(), body: "Eigener Text.".into(), ..Default::default() }],
                },
            )?;
            Ok((study.study.id, other.study.id, prep.id, writing.id, qa_block.id, finding.id, secret.id, passage.id))
        })
        .unwrap();
    World {
        db,
        study: ids.0,
        other: ids.1,
        prep: ids.2,
        writing: ids.3,
        qa: ids.4,
        finding: ids.5,
        secret: ids.6,
        citation: "cite-1".into(),
        passage: ids.7,
    }
}

fn item(kind: &str, id: &str, label: &str) -> StudyItemRef {
    StudyItemRef {
        kind: kind.into(),
        id: id.into(),
        label: label.into(),
    }
}

fn add(
    conn: &Connection,
    w: &World,
    items: Vec<StudyItemRef>,
) -> abiding_word_lib::error::AppResult<abiding_word_lib::domain::TeachingPreparation> {
    teaching_blocks::add_from_study(
        conn,
        &AddFromStudy {
            preparation_id: w.prep.clone(),
            parent_id: None,
            items,
        },
    )
}

#[test]
fn a_study_qa_block_keeps_question_and_answer_apart_and_derives_its_text() {
    let w = world();
    w.db.with_conn(|conn| {
        let block = blocks::get(conn, &w.qa)?;
        assert_eq!(block.block_type, BlockType::Question);
        assert_eq!(
            block.content,
            "Wer verdammt?\n\nNiemand, Christus tritt für uns ein."
        );
        let updated = blocks::update(
            conn,
            &UpdateStudyBlock {
                id: w.qa.clone(),
                content: Some("ignored".into()),
                data: Some(qa("Wer verdammt?", "Gott rechtfertigt.")),
            },
        )?;
        assert_eq!(updated.content, "Wer verdammt?\n\nGott rechtfertigt.");
        // An answer may be long: no artificial short limit.
        let long = "Antwort. ".repeat(200);
        let updated = blocks::update(
            conn,
            &UpdateStudyBlock {
                id: w.qa.clone(),
                content: None,
                data: Some(qa("Frage?", &long)),
            },
        )?;
        assert!(updated.content.ends_with(long.trim()));
        // Only a question block takes Q&A data; a malformed object is refused.
        assert!(blocks::update(
            conn,
            &UpdateStudyBlock {
                id: w.writing.clone(),
                content: None,
                data: Some(qa("x", "y"))
            }
        )
        .is_err());
        assert!(blocks::update(
            conn,
            &UpdateStudyBlock {
                id: w.qa.clone(),
                content: None,
                data: Some("{\"kind\":\"other\"}".into())
            }
        )
        .is_err());
        Ok(())
    })
    .unwrap();
}

#[test]
fn chosen_study_items_become_leader_only_editable_copies_with_provenance() {
    let w = world();
    w.db.with_conn(|conn| {
        let before = teaching::get(conn, &w.prep)?.blocks.len();
        let prep = add(
            conn,
            &w,
            vec![
                item("writing", &w.writing, "Text"),
                item("qa", &w.qa, "Frage und Antwort"),
                item("finding", &w.finding, "Beobachtung"),
                item("citation", &w.citation, "Zitat"),
                item("passage", &w.passage, "Römer 8,1–11"),
            ],
        )?;
        let added: Vec<_> = prep.blocks.iter().filter(|b| b.origin.is_some()).collect();
        assert_eq!(prep.blocks.len(), before + 5);
        assert_eq!(added.len(), 5);
        assert!(
            added.iter().all(|b| !b.participant_facing),
            "copies start for the leader only"
        );
        assert!(added
            .iter()
            .all(|b| b.origin.as_ref().unwrap().study_id == w.study));
        let by_kind = |k: &str| {
            added
                .iter()
                .find(|b| b.origin.as_ref().unwrap().kind == k)
                .unwrap()
        };
        assert_eq!(by_kind("writing").body, "Kein Verdammungsurteil.");
        let qa = by_kind("qa");
        assert_eq!(
            (qa.block_type.as_str(), qa.title.as_str(), qa.body.as_str()),
            (
                "qa",
                "Wer verdammt?",
                "Niemand, Christus tritt für uns ein."
            )
        );
        assert_eq!(by_kind("finding").body, "Der Geist bezeugt.");
        let quote = by_kind("citation");
        assert!(
            quote.body.contains("Erfundenes Zitat.")
                && quote.body.contains("Erfundenes Buch, A. Autor, S. 12")
        );
        assert_eq!(
            quote.origin.as_ref().unwrap().citation.as_deref(),
            Some("Erfundenes Buch, A. Autor, S. 12")
        );
        let scripture = by_kind("passage");
        assert_eq!(scripture.block_type, "scripture");
        assert!(prep
            .passages
            .iter()
            .any(|p| p.block_id.as_deref() == Some(scripture.id.as_str())
                && p.reference.canonical == "ROM.8.1-ROM.8.11"));
        // The person's own writing is untouched and stays first.
        assert_eq!(prep.blocks[0].body, "Eigener Text.");
        assert!(prep.blocks[0].origin.is_none());
        Ok(())
    })
    .unwrap();
}

#[test]
fn copies_are_snapshots_never_synchronised_and_keep_provenance_when_edited_or_duplicated() {
    let w = world();
    w.db.with_conn(|conn| {
        let prep = add(conn, &w, vec![item("writing", &w.writing, "Text")])?;
        let copy = prep
            .blocks
            .iter()
            .find(|b| b.origin.is_some())
            .unwrap()
            .clone();
        blocks::update(
            conn,
            &UpdateStudyBlock {
                id: w.writing.clone(),
                content: Some("Später geändert.".into()),
                data: None,
            },
        )?;
        let prep = teaching::get(conn, &w.prep)?;
        assert_eq!(
            prep.blocks.iter().find(|b| b.id == copy.id).unwrap().body,
            "Kein Verdammungsurteil."
        );
        let prep = teaching_blocks::update(
            conn,
            &UpdateTeachingBlock {
                id: copy.id.clone(),
                body: Some("Für die Gruppe umformuliert.".into()),
                participant_facing: Some(true),
                ..Default::default()
            },
        )?;
        let edited = prep.blocks.iter().find(|b| b.id == copy.id).unwrap();
        assert_eq!(edited.origin.as_ref().unwrap().item_id, w.writing);
        assert!(edited.participant_facing);
        let prep = teaching_blocks::duplicate(conn, &copy.id)?;
        assert_eq!(
            prep.blocks
                .iter()
                .filter(|b| b.origin.as_ref().is_some_and(|o| o.item_id == w.writing))
                .count(),
            2
        );
        // The study is never changed by copying.
        assert_eq!(blocks::get(conn, &w.writing)?.content, "Später geändert.");
        Ok(())
    })
    .unwrap();
}

#[test]
fn another_studys_note_or_a_deleted_item_is_refused_and_nothing_is_added() {
    let w = world();
    w.db.with_conn(|conn| {
        let before = teaching::get(conn, &w.prep)?.blocks.len();
        let refused = add(conn, &w, vec![item("writing", &w.writing, "Text"), item("finding", &w.secret, "Beobachtung")]);
        assert!(matches!(refused, Err(AppError::InvalidInput(ref m)) if m.code == Some("teaching.fromStudyChanged")), "{refused:?}");
        let prep = teaching::get(conn, &w.prep)?;
        assert_eq!(prep.blocks.len(), before, "all or nothing");
        assert!(!format!("{prep:?}").contains("SECRET-OTHER-STUDY"));
        blocks::delete(conn, &w.qa)?;
        assert!(add(conn, &w, vec![item("qa", &w.qa, "Frage")]).is_err());
        assert!(add(conn, &w, vec![]).is_err());
        assert!(add(conn, &w, vec![item("unknown", "x", "")]).is_err());
        let _ = &w.other;
        Ok(())
    })
    .unwrap();
}

#[test]
fn an_independent_teaching_has_no_study_to_copy_from() {
    let w = world();
    w.db.with_conn(|conn| {
        let ws = workspaces::get_default(conn)?;
        let project = projects::create(conn, &CreateProject { workspace_id: ws.id, title: "Einzeln".into(), description: None })?;
        let alone = teaching::create(conn, &CreateTeaching { project_id: Some(project.id), study_id: String::new(), title: "Ohne Studie".into(), kind: None, points: vec![], template_key: None, template_version: Some(2), blocks: vec![] })?;
        let refused = teaching_blocks::add_from_study(conn, &AddFromStudy { preparation_id: alone.id, parent_id: None, items: vec![item("writing", &w.writing, "Text")] });
        assert!(matches!(refused, Err(AppError::InvalidInput(ref m)) if m.code == Some("teaching.noStudy")), "{refused:?}");
        Ok(())
    })
    .unwrap();
}

#[test]
fn a_qa_teaching_block_is_participant_facing_by_default_and_a3_configs_still_load() {
    let w = world();
    w.db.with_conn(|conn| {
        let prep = teaching_blocks::create(
            conn,
            &abiding_word_lib::domain::teaching::CreateTeachingBlock {
                preparation_id: w.prep.clone(),
                block_type: "qa".into(),
                title: "Was sehen wir?".into(),
                body: "Den Weinstock.".into(),
                ..Default::default()
            },
        )?;
        let qa = prep.blocks.iter().find(|b| b.block_type == "qa").unwrap();
        assert!(qa.participant_facing);
        assert!(qa.origin.is_none());
        // A block stored by A.3 has no origin in its JSON config.
        conn.execute(
            "UPDATE teaching_blocks SET config = '{\"participantFacing\":false}' WHERE id = ?1",
            [&qa.id],
        )?;
        let reloaded = teaching::get(conn, &w.prep)?;
        assert!(reloaded
            .blocks
            .iter()
            .any(|b| b.id == qa.id && b.origin.is_none() && !b.participant_facing));
        Ok(())
    })
    .unwrap();
}
