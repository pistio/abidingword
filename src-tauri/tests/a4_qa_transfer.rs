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

/// Raw rows include soft-deleted data, JSON origins/privacy, compatibility
/// columns, positions, revisions, timestamps and search/index side effects.
/// Comparing only teaching::get() would miss persisted rollback debris.
type DatabaseSnapshot = Vec<(String, Vec<Vec<rusqlite::types::Value>>)>;

fn database_snapshot(conn: &Connection) -> rusqlite::Result<DatabaseSnapshot> {
    let mut names =
        conn.prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")?;
    let names = names
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    let mut snapshot = Vec::new();
    for name in names {
        let quoted = name.replace('"', "\"\"");
        let mut stmt = conn.prepare(&format!("SELECT * FROM \"{quoted}\""))?;
        let columns = stmt.column_count();
        let mut rows = stmt
            .query_map([], |r| {
                (0..columns)
                    .map(|i| r.get::<_, rusqlite::types::Value>(i))
                    .collect::<Result<Vec<_>, _>>()
            })?
            .collect::<Result<Vec<_>, _>>()?;
        // WITHOUT ROWID and virtual/shadow tables need no special query path.
        rows.sort_by_cached_key(|row| format!("{row:?}"));
        snapshot.push((name, rows));
    }
    Ok(snapshot)
}

fn transfer_items(w: &World) -> Vec<StudyItemRef> {
    vec![
        item("writing", &w.writing, "Text"),
        item("passage", &w.passage, "Römer 8,1–11"),
        item("qa", &w.qa, "Frage und Antwort"),
        item("finding", &w.finding, "Beobachtung"),
        item("citation", &w.citation, "Zitat"),
    ]
}

#[test]
fn sql_failures_after_completed_copies_restore_every_persisted_row() {
    // Fail the second and third blocks, after a Scripture block is inserted,
    // after a passage is attached, and after revisions were already updated.
    // AFTER + FAIL deliberately keeps the failing statement's writes until
    // our transaction rolls them back. No production failure hook is needed.
    let failures = [
        ("second-block", "AFTER INSERT ON teaching_blocks WHEN (SELECT COUNT(*) FROM teaching_blocks WHERE json_extract(config, '$.origin') IS NOT NULL) = 3"),
        ("third-block", "AFTER INSERT ON teaching_blocks WHEN (SELECT COUNT(*) FROM teaching_blocks WHERE json_extract(config, '$.origin') IS NOT NULL) = 4"),
        ("passage-attachment", "AFTER INSERT ON teaching_passages WHEN NEW.block_id IS NOT NULL"),
        ("revision-update", "AFTER UPDATE ON teaching_preparations WHEN (SELECT COUNT(*) FROM teaching_passages WHERE block_id IS NOT NULL) > 0"),
        ("study-touch", "AFTER UPDATE ON studies WHEN (SELECT COUNT(*) FROM teaching_passages WHERE block_id IS NOT NULL) > 0"),
    ];
    for (name, event) in failures {
        let w = world();
        w.db.with_conn(|conn| {
            // Nontrivial pre-state: an existing editable snapshot and teaching-level
            // passage, a visible writing block, plus deleted rows.
            let existing = add(conn, &w, vec![item("writing", &w.writing, "Existing origin")])?;
            let original = existing.blocks.iter().find(|b| b.origin.is_none()).unwrap();
            teaching_blocks::update(conn, &UpdateTeachingBlock {
                id: original.id.clone(), participant_facing: Some(true), ..Default::default()
            })?;
            teaching::add_passage(conn, &abiding_word_lib::domain::teaching::AddTeachingPassage {
                preparation_id: w.prep.clone(), point_id: None, block_id: None, reference: "JHN.15.1".into()
            })?;
            let deleted = teaching_blocks::create(conn, &abiding_word_lib::domain::teaching::CreateTeachingBlock {
                preparation_id: w.prep.clone(), block_type: "writing".into(), title: "Deleted".into(), ..Default::default()
            })?;
            let deleted_id = deleted.blocks.iter().find(|b| b.title == "Deleted").unwrap().id.clone();
            teaching_blocks::delete(conn, &deleted_id)?;
            // A distinct timestamp proves that study touches are restored too.
            conn.execute("UPDATE studies SET updated_at = 'before-transfer' WHERE id = ?1", [&w.study])?;
            let before = database_snapshot(conn)?;
            let visible_before = serde_json::to_value(teaching::get(conn, &w.prep)?).unwrap();
            // Existing passage is not block-attached, so later triggers only
            // fire once this request has completed its new passage insertion.
            conn.execute_batch(&format!("CREATE TEMP TRIGGER inject_transfer_failure {event} BEGIN SELECT RAISE(FAIL, 'injected-transfer-failure'); END;"))?;
            let error = add(conn, &w, transfer_items(&w)).unwrap_err();
            assert!(matches!(&error, AppError::Database(_)), "{name}: {error:?}");
            assert!(error.to_string().contains("injected-transfer-failure"), "{name}: {error:?}");
            assert!(conn.is_autocommit(), "{name}: transaction leaked");
            assert_eq!(database_snapshot(conn)?, before, "{name}: raw database changed");
            assert_eq!(serde_json::to_value(teaching::get(conn, &w.prep)?).unwrap(), visible_before, "{name}: visible teaching changed");
            conn.execute_batch("DROP TRIGGER inject_transfer_failure")?;
            // The same request succeeds immediately after removing the fault.
            let result = add(conn, &w, transfer_items(&w))?;
            assert_eq!(result.blocks.iter().filter(|b| b.origin.is_some()).count(), 6);
            assert!(result.blocks.iter().filter(|b| b.origin.is_some()).all(|b| !b.participant_facing));
            let scripture = result.blocks.iter().find(|b| b.block_type == "scripture").unwrap();
            assert!(result.passages.iter().any(|p| p.block_id.as_deref() == Some(&scripture.id)));
            Ok(())
        }).unwrap();
    }
}

#[test]
fn response_read_and_deferred_commit_failures_also_roll_back_the_whole_transfer() {
    for fault in ["response-read", "commit"] {
        let w = world();
        w.db.with_conn(|conn| {
            if fault == "commit" {
                // Every statement succeeds; only COMMIT checks this constraint.
                conn.execute_batch("CREATE TABLE deferred_transfer_fault (
                    preparation_id TEXT REFERENCES teaching_preparations(id) DEFERRABLE INITIALLY DEFERRED
                );
                CREATE TEMP TRIGGER inject_late_failure AFTER INSERT ON teaching_passages
                BEGIN INSERT INTO deferred_transfer_fault VALUES ('missing-preparation'); END;")?;
            } else {
                // The final response cannot decode the damaged title. This
                // must fail before committing any of the otherwise valid copy.
                conn.execute_batch("CREATE TEMP TRIGGER inject_late_failure AFTER INSERT ON teaching_passages
                BEGIN UPDATE teaching_preparations SET title = X'80' WHERE id = NEW.preparation_id; END;")?;
            }
            let before = database_snapshot(conn)?;
            let error = add(conn, &w, vec![
                item("writing", &w.writing, "Text"),
                item("passage", &w.passage, "Passage"),
            ]).unwrap_err();
            assert!(matches!(&error, AppError::Database(_)), "{fault}: {error:?}");
            if fault == "commit" {
                assert!(error.to_string().contains("FOREIGN KEY constraint failed"), "{error:?}");
            } else {
                assert!(matches!(error, AppError::Database(rusqlite::Error::InvalidColumnType(..))), "{error:?}");
            }
            assert!(conn.is_autocommit(), "{fault}: transaction leaked");
            assert_eq!(database_snapshot(conn)?, before, "{fault}: raw database changed");
            conn.execute_batch("DROP TRIGGER inject_late_failure")?;
            add(conn, &w, transfer_items(&w))?;
            Ok(())
        }).unwrap();
    }
}
