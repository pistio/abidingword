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
                project_id: None,
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
                project_id: None,
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
                project_id: None,
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

#[test]
fn role_copies_keep_independent_live_slides_and_deletion_clears_the_mirror() {
    use abiding_word_lib::db::teaching_blocks as blocks;
    use abiding_word_lib::domain::CreatePresentation;
    use abiding_word_lib::presentation::{document, ImageStore};
    with_db(|conn| {
        let prep = teaching::create(
            conn,
            &CreateTeaching {
                project_id: None,
                study_id: study(conn)?,
                title: "Invented sermon".into(),
                template_version: Some(2),
                blocks: vec![NewTeachingBlock {
                    body: "Owner words".into(),
                    ..block(
                        "intro",
                        None,
                        "writing",
                        "Opening",
                        Some("introduction"),
                        Some(true),
                    )
                }],
                ..Default::default()
            },
        )?;
        let owner = prep.blocks[0].id.clone();
        let copied = blocks::duplicate(conn, &owner)?;
        let copy = copied
            .blocks
            .iter()
            .find(|b| b.id != owner)
            .unwrap()
            .id
            .clone();
        assert_eq!(
            copied.blocks.iter().find(|b| b.id == copy).unwrap().role,
            None
        );
        blocks::update(
            conn,
            &UpdateTeachingBlock {
                id: copy.clone(),
                body: Some("Copy words".into()),
                ..Default::default()
            },
        )?;
        let presentation = presentations::create(
            conn,
            &CreatePresentation {
                preparation_id: prep.id.clone(),
                propose: true,
                ..Default::default()
            },
        )?;
        let temp = tempfile::tempdir().unwrap();
        let store = ImageStore {
            generated_dir: temp.path().join("g"),
            offline_dir: temp.path().join("o"),
            template_dir: temp.path().join("t"),
        };
        let text = |conn: &Connection| -> AppResult<String> {
            Ok(serde_json::to_string(&document::build(conn, &presentation.id, &store)?).unwrap())
        };
        let before = text(conn)?;
        assert!(before.contains("Owner words") && before.contains("Copy words"));
        blocks::update(
            conn,
            &UpdateTeachingBlock {
                id: copy.clone(),
                body: Some("Different copy".into()),
                ..Default::default()
            },
        )?;
        assert_eq!(teaching::get(conn, &prep.id)?.introduction, "Owner words");
        let after = text(conn)?;
        assert!(after.contains("Owner words") && after.contains("Different copy"));
        assert!(blocks::update(
            conn,
            &UpdateTeachingBlock {
                id: copy,
                role: Some("introduction".into()),
                ..Default::default()
            }
        )
        .is_err());
        let deleted = blocks::delete(conn, &owner)?;
        assert_eq!(deleted.introduction, "");
        assert!(!text(conn)?.contains("Owner words"));
        Ok(())
    });
}

#[test]
fn scripture_links_reject_unknown_cross_preparation_and_ineligible_blocks_without_writes() {
    use abiding_word_lib::db::teaching_blocks as blocks;
    use abiding_word_lib::domain::AddTeachingPassage;
    with_db(|conn| {
        let make = || {
            teaching::create(
                conn,
                &CreateTeaching {
                    project_id: None,
                    study_id: study(conn)?,
                    title: "Invented".into(),
                    template_version: Some(2),
                    blocks: vec![
                        block("s", None, "scripture", "Reading", None, Some(true)),
                        block("w", None, "writing", "Writing", None, Some(true)),
                    ],
                    ..Default::default()
                },
            )
        };
        let prep = make()?;
        let other = make()?;
        let scripture = prep
            .blocks
            .iter()
            .find(|b| b.block_type == "scripture")
            .unwrap()
            .id
            .clone();
        let add = |block_id: &str| {
            teaching::add_passage(
                conn,
                &AddTeachingPassage {
                    preparation_id: prep.id.clone(),
                    point_id: None,
                    block_id: Some(block_id.into()),
                    reference: "JHN.15.1".into(),
                },
            )
        };
        for bad in [
            "missing",
            other.blocks[0].id.as_str(),
            prep.blocks
                .iter()
                .find(|b| b.block_type == "writing")
                .unwrap()
                .id
                .as_str(),
        ] {
            assert!(add(bad).is_err());
            assert!(teaching::get(conn, &prep.id)?.passages.is_empty());
        }
        assert_eq!(add(&scripture)?.passages.len(), 1);
        let deleted = blocks::delete(conn, &scripture)?;
        assert!(deleted.passages.is_empty());
        assert!(add(&scripture).is_err());
        Ok(())
    });
}

#[test]
fn conflicting_starting_roles_keep_all_writing_and_restore_never_steals_ownership() {
    use abiding_word_lib::db::teaching_blocks as blocks;
    with_db(|conn| {
        let prep = teaching::create(
            conn,
            &CreateTeaching {
                project_id: None,
                study_id: study(conn)?,
                title: "Invented conflicts".into(),
                template_version: Some(2),
                blocks: vec![
                    NewTeachingBlock {
                        body: "FIRST".into(),
                        ..block(
                            "a",
                            None,
                            "writing",
                            "First",
                            Some("introduction"),
                            Some(true),
                        )
                    },
                    NewTeachingBlock {
                        body: "SECOND".into(),
                        ..block(
                            "b",
                            None,
                            "writing",
                            "Second",
                            Some("introduction"),
                            Some(true),
                        )
                    },
                ],
                ..Default::default()
            },
        )?;
        assert_eq!(prep.introduction, "FIRST");
        assert_eq!(prep.blocks[1].body, "SECOND");
        assert_eq!(prep.blocks[1].role, None);
        let owner = prep.blocks[0].id.clone();
        let other = prep.blocks[1].id.clone();
        assert!(blocks::create(
            conn,
            &CreateTeachingBlock {
                preparation_id: prep.id.clone(),
                block_type: "writing".into(),
                role: Some("introduction".into()),
                ..Default::default()
            }
        )
        .is_err());
        blocks::delete(conn, &owner)?;
        let restored = blocks::restore(conn, &owner)?;
        assert_eq!(restored.introduction, "FIRST");
        blocks::delete(conn, &owner)?;
        blocks::update(
            conn,
            &UpdateTeachingBlock {
                id: other.clone(),
                role: Some("introduction".into()),
                ..Default::default()
            },
        )?;
        let restored = blocks::restore(conn, &owner)?;
        assert_eq!(restored.introduction, "SECOND");
        assert_eq!(
            restored.blocks.iter().find(|b| b.id == owner).unwrap().role,
            None
        );
        assert_eq!(
            restored.blocks.iter().find(|b| b.id == owner).unwrap().body,
            "FIRST"
        );
        // Reordering never changes the role owner.
        blocks::reorder(
            conn,
            &ReorderTeachingBlocks {
                preparation_id: prep.id.clone(),
                ids: vec![other, owner],
                ..Default::default()
            },
        )?;
        assert_eq!(teaching::get(conn, &prep.id)?.introduction, "SECOND");
        Ok(())
    });
}

#[test]
fn presentations_exclude_private_scripture_even_after_public_slides_were_saved() {
    use abiding_word_lib::db::teaching_blocks as blocks;
    use abiding_word_lib::domain::{
        AddTeachingPassage, CreatePresentation, CreatePresentationItem, ItemSource,
    };
    use abiding_word_lib::presentation::{document, ImageStore};
    with_db(|conn| {
        let prep = teaching::create(
            conn,
            &CreateTeaching {
                project_id: None,
                study_id: study(conn)?,
                title: "Private reading".into(),
                template_version: Some(2),
                blocks: vec![block(
                    "reading",
                    None,
                    "scripture",
                    "Reading",
                    None,
                    Some(true),
                )],
                ..Default::default()
            },
        )?;
        let id = prep.blocks[0].id.clone();
        let prep = teaching::add_passage(
            conn,
            &AddTeachingPassage {
                preparation_id: prep.id.clone(),
                point_id: None,
                block_id: Some(id.clone()),
                reference: "JHN.15.3".into(),
            },
        )?;
        let presentation = presentations::create(
            conn,
            &CreatePresentation {
                preparation_id: prep.id.clone(),
                propose: true,
                ..Default::default()
            },
        )?;
        // An old cached override must not defeat later privacy changes.
        conn.execute("UPDATE presentation_items SET body = 'PRIVATE CACHED WORDS', reference = 'JHN.15.3' WHERE presentation_id = ?1 AND source_kind = 'passage'", [&presentation.id])?;
        let hidden = blocks::update(
            conn,
            &UpdateTeachingBlock {
                id: id.clone(),
                participant_facing: Some(false),
                ..Default::default()
            },
        )?;
        assert!(presentations::propose(&hidden, &ProposalLabels::default())
            .iter()
            .all(|item| item.source.as_ref().is_none_or(|s| s.kind != "passage")));
        assert!(presentations::create_item(
            conn,
            &CreatePresentationItem {
                presentation_id: presentation.id.clone(),
                kind: "scripture".into(),
                source: Some(ItemSource {
                    kind: "passage".into(),
                    id: prep.passages[0].id.clone(),
                    field: None
                }),
                ..Default::default()
            }
        )
        .is_err());
        let temp = tempfile::tempdir().unwrap();
        let store = ImageStore {
            generated_dir: temp.path().join("g"),
            offline_dir: temp.path().join("o"),
            template_dir: temp.path().join("t"),
        };
        let doc = document::build(conn, &presentation.id, &store)?;
        assert!(doc.primary_passage.is_none());
        let json = serde_json::to_string(&doc).unwrap();
        assert!(!json.contains("JHN.15.3") && !json.contains("PRIVATE CACHED WORDS"));
        blocks::delete(conn, &id)?;
        let restored = blocks::restore(conn, &id)?;
        assert!(restored.passages.is_empty());
        Ok(())
    });
}

#[test]
fn deleting_a_private_section_keeps_lifted_children_private_and_legacy_sessions_unchanged() {
    use abiding_word_lib::db::teaching_blocks as blocks;
    with_db(|conn| {
        let legacy = teaching::create(
            conn,
            &CreateTeaching {
                project_id: None,
                study_id: study(conn)?,
                title: "Legacy".into(),
                ..Default::default()
            },
        )?;
        teaching::update(
            conn,
            &UpdateTeaching {
                id: legacy.id.clone(),
                introduction: Some("Legacy writing".into()),
                ..Default::default()
            },
        )?;
        assert!(blocks::create(
            conn,
            &CreateTeachingBlock {
                preparation_id: legacy.id.clone(),
                block_type: "writing".into(),
                ..Default::default()
            }
        )
        .is_err());
        let still = teaching::get(conn, &legacy.id)?;
        assert_eq!(still.introduction, "Legacy writing");
        assert!(still.blocks.is_empty());
        assert!(still.template_version.is_none());
        let prep = teaching::create(
            conn,
            &CreateTeaching {
                project_id: None,
                study_id: study(conn)?,
                title: "Private section".into(),
                template_version: Some(2),
                blocks: vec![
                    block("private", None, "section", "Private", None, Some(false)),
                    NewTeachingBlock {
                        body: "Leader words".into(),
                        ..block(
                            "child",
                            Some("private"),
                            "writing",
                            "Writing",
                            None,
                            Some(true),
                        )
                    },
                ],
                ..Default::default()
            },
        )?;
        assert!(!blocks::participant_visible(&prep, &prep.blocks[1].id));
        let lifted = blocks::delete(conn, &prep.blocks[0].id)?;
        assert_eq!(lifted.blocks.len(), 1);
        assert!(!lifted.blocks[0].participant_facing);
        assert_eq!(lifted.blocks[0].body, "Leader words");
        assert!(!blocks::participant_visible(&lifted, &lifted.blocks[0].id));
        Ok(())
    });
}
