//! Block teachings (migration 0025). Legacy teachings are not converted.
//! Private leader notes stay out of a proposed presentation. All text is invented.

use abiding_word_lib::db::{presentations, teaching, Database};
use abiding_word_lib::domain::{
    CreateTeaching, CreateTeachingBlock, NewTeachingBlock, ProposalLabels, ReorderTeachingBlocks,
    UpdateTeaching, UpdateTeachingBlock,
};
use abiding_word_lib::error::AppResult;
use rusqlite::Connection;

fn with_db<T>(f: impl FnOnce(&Connection) -> AppResult<T>) -> T {
    Database::open_in_memory().unwrap().with_conn(f).unwrap()
}

fn study(conn: &Connection) -> AppResult<String> {
    use abiding_word_lib::db::{projects, studies, workspaces};
    use abiding_word_lib::domain::{CreateProject, CreateStudy};
    let workspace = workspaces::get_default(conn)?;
    let project = projects::create(
        conn,
        &CreateProject {
            workspace_id: workspace.id,
            title: "John".into(),
            description: None,
        },
    )?;
    Ok(studies::create(
        conn,
        &CreateStudy {
            project_id: project.id,
            title: "John 15".into(),
            study_type: None,
            template_id: None,
            template_key: None,
            primary_passage: None,
            description: None,
        },
    )?
    .study
    .id)
}

fn block(
    key: &str,
    parent: Option<&str>,
    kind: &str,
    title: &str,
    role: Option<&str>,
    facing: Option<bool>,
) -> NewTeachingBlock {
    NewTeachingBlock {
        key: Some(key.into()),
        parent_key: parent.map(str::to_string),
        block_type: kind.into(),
        title: title.into(),
        participant_facing: facing,
        role: role.map(str::to_string),
        ..Default::default()
    }
}

#[test]
fn a_block_teaching_dual_writes_roles_and_leaves_a_legacy_sermon_alone() {
    with_db(|conn| {
        let study = study(conn)?;
        let legacy = teaching::create(
            conn,
            &CreateTeaching {
                study_id: study.clone(),
                title: "Old sermon".into(),
                kind: Some("sermon".into()),
                points: vec![abiding_word_lib::domain::StartingPoint {
                    title: "Kept point".into(),
                    kind: None,
                }],
                ..Default::default()
            },
        )?;
        assert!(legacy.blocks.is_empty());
        assert_eq!(legacy.template_version, None);
        assert_eq!(legacy.points[0].title, "Kept point");

        let sermon = teaching::create(
            conn,
            &CreateTeaching {
                study_id: study,
                title: "Sunday".into(),
                kind: Some("sermon".into()),
                template_key: Some("sermon-talk".into()),
                template_version: Some(2),
                blocks: vec![
                    block("intro", None, "section", "Introduction", None, Some(true)),
                    block(
                        "intro-w",
                        Some("intro"),
                        "writing",
                        "",
                        Some("introduction"),
                        Some(true),
                    ),
                    block("notes", None, "teaching_notes", "For me", None, None),
                    block(
                        "ask",
                        None,
                        "discussion_questions",
                        "What do you see?",
                        None,
                        None,
                    ),
                ],
                ..Default::default()
            },
        )?;
        assert_eq!(sermon.template_key.as_deref(), Some("sermon-talk"));
        assert_eq!(sermon.template_version, Some(2));
        assert_eq!(sermon.blocks.len(), 4);
        let writing = sermon
            .blocks
            .iter()
            .find(|b| b.block_key.as_deref() == Some("intro-w"))
            .unwrap();
        assert_eq!(
            writing.parent_id,
            sermon
                .blocks
                .iter()
                .find(|b| b.block_type == "section")
                .map(|b| b.id.clone())
        );
        let notes = sermon
            .blocks
            .iter()
            .find(|b| b.block_type == "teaching_notes")
            .unwrap();
        assert!(!notes.participant_facing);
        let questions = sermon
            .blocks
            .iter()
            .find(|b| b.block_type == "discussion_questions")
            .unwrap();
        assert!(questions.participant_facing);

        let updated = teaching::update(
            conn,
            &UpdateTeaching {
                id: sermon.id.clone(),
                introduction: Some("Last week a vine.".into()),
                ..Default::default()
            },
        )?;
        // The column and the block that wears the role stay together.
        assert_eq!(updated.introduction, "Last week a vine.");
        // Updating the column mirrors into the block.
        let body = updated
            .blocks
            .iter()
            .find(|b| b.role.as_deref() == Some("introduction"))
            .unwrap();
        assert_eq!(body.body, "Last week a vine.");

        let from_block = abiding_word_lib::db::teaching_blocks::update(
            conn,
            &UpdateTeachingBlock {
                id: body.id.clone(),
                body: Some("Have you pruned a rose?".into()),
                ..Default::default()
            },
        )?;
        assert_eq!(from_block.introduction, "Have you pruned a rose?");

        let labels = ProposalLabels {
            introduction: "Introduction".into(),
            conclusion: "Conclusion".into(),
        };
        let slides = presentations::propose(&from_block, &labels);
        let kinds: Vec<&str> = slides
            .iter()
            .map(|s| s.source.as_ref().map(|x| x.kind.as_str()).unwrap_or(""))
            .collect();
        assert!(kinds.contains(&"preparation"));
        assert!(kinds.contains(&"block"));
        assert!(slides
            .iter()
            .all(|s| s.source.as_ref().is_none_or(|src| src.id != notes.id)));

        // The legacy sermon was not rewritten.
        let still = teaching::get(conn, &legacy.id)?;
        assert!(still.blocks.is_empty());
        assert_eq!(still.points[0].title, "Kept point");
        assert_eq!(still.introduction, "");
        Ok(())
    });
}

#[test]
fn blocks_reorder_exactly_and_a_section_delete_keeps_what_was_written() {
    with_db(|conn| {
        let study = study(conn)?;
        let prep = teaching::create(
            conn,
            &CreateTeaching {
                study_id: study,
                title: "EVA".into(),
                kind: Some("bible_study".into()),
                template_key: Some("eva".into()),
                template_version: Some(2),
                blocks: vec![
                    block("a", None, "section", "Erkennen", None, Some(true)),
                    block("aw", Some("a"), "writing", "", None, Some(false)),
                    block("b", None, "section", "Verstehen", None, Some(true)),
                ],
                ..Default::default()
            },
        )?;
        let top: Vec<String> = prep
            .blocks
            .iter()
            .filter(|b| b.parent_id.is_none())
            .map(|b| b.id.clone())
            .collect();
        assert_eq!(top.len(), 2);
        let mut reversed = top.clone();
        reversed.reverse();
        let moved = abiding_word_lib::db::teaching_blocks::reorder(
            conn,
            &ReorderTeachingBlocks {
                preparation_id: prep.id.clone(),
                parent_id: None,
                ids: reversed.clone(),
            },
        )?;
        let order: Vec<String> = moved
            .blocks
            .iter()
            .filter(|b| b.parent_id.is_none())
            .map(|b| b.id.clone())
            .collect();
        assert_eq!(order, reversed);
        assert!(abiding_word_lib::db::teaching_blocks::reorder(
            conn,
            &ReorderTeachingBlocks {
                preparation_id: prep.id.clone(),
                parent_id: None,
                ids: vec![top[0].clone()],
            },
        )
        .is_err());

        let section = moved.blocks.iter().find(|b| b.title == "Erkennen").unwrap();
        let kept = abiding_word_lib::db::teaching_blocks::delete(conn, &section.id)?;
        assert!(kept.blocks.iter().all(|b| b.id != section.id));
        assert!(kept
            .blocks
            .iter()
            .any(|b| b.block_type == "writing" && b.parent_id.is_none()));

        let added = abiding_word_lib::db::teaching_blocks::create(
            conn,
            &CreateTeachingBlock {
                preparation_id: kept.id.clone(),
                block_type: "outline_point".into(),
                title: "Main point".into(),
                body: "Say this.".into(),
                ..Default::default()
            },
        )?;
        let point = added
            .points
            .iter()
            .find(|p| p.title == "Main point")
            .unwrap();
        assert_eq!(point.explanation, "Say this.");
        let block = added
            .blocks
            .iter()
            .find(|b| b.block_type == "outline_point")
            .unwrap();
        assert_eq!(block.point_id.as_deref(), Some(point.id.as_str()));
        Ok(())
    });
}
