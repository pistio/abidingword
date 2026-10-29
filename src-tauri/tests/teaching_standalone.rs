//! Project-owned Teaching: disposable invented data, offline and without a Bible install.
use abiding_word_lib::db::{
    findings, presentations, projects, studies, teaching, workspaces, Database,
};
use abiding_word_lib::domain::{
    CreateFinding, CreateProject, CreateStudy, CreateTeaching, FindingType, LinkTeachingFinding,
    NewTeachingBlock,
};
use abiding_word_lib::filesystem::DataDirs;
use abiding_word_lib::portability::{backup, project};
use abiding_word_lib::presentation::{document, ImageStore};
use rusqlite::{params, Connection};

fn project_id(c: &Connection, title: &str) -> String {
    projects::create(
        c,
        &CreateProject {
            workspace_id: workspaces::get_default(c).unwrap().id,
            title: title.into(),
            description: None,
        },
    )
    .unwrap()
    .id
}
fn study(c: &Connection, p: &str) -> String {
    studies::create(
        c,
        &CreateStudy {
            project_id: p.into(),
            title: "Synthetic study".into(),
            ..Default::default()
        },
    )
    .unwrap()
    .study
    .id
}
fn prep(c: &Connection, p: &str, s: Option<&str>) -> String {
    teaching::create(
        c,
        &CreateTeaching {
            project_id: Some(p.into()),
            study_id: s.unwrap_or("").into(),
            title: "Independent message".into(),
            template_key: Some("sermon-talk".into()),
            template_version: Some(2),
            blocks: vec![NewTeachingBlock {
                block_type: "writing".into(),
                title: "Manuscript".into(),
                body: "Remain in the vine.".into(),
                participant_facing: Some(false),
                ..Default::default()
            }],
            ..Default::default()
        },
    )
    .unwrap()
    .id
}
fn attach(c: &Connection, t: &str, s: &str) -> String {
    let f = findings::create(
        c,
        &CreateFinding {
            study_id: s.into(),
            finding_type: FindingType::Observation,
            content: Some("Saved observation — 한글".into()),
            section_id: None,
            passage: Some("JHN.15.5".into()),
        },
    )
    .unwrap();
    c.execute("INSERT INTO finding_sources(id,finding_id,cited_source_id,cited_title,cited_author,cited_content_hash,quote,locator,cited_section,created_at,updated_at) VALUES('citation',?1,'missing-source','Synthetic commentary','Invented author',?2,'Exact quotation','p. 7','Vine','T','T')",params![f.id,"a".repeat(64)]).unwrap();
    teaching::link_finding(
        c,
        &LinkTeachingFinding {
            preparation_id: t.into(),
            point_id: None,
            finding_id: f.id.clone(),
        },
    )
    .unwrap();
    c.execute("INSERT INTO presentations(id,preparation_id,title,theme,position,created_at,updated_at) VALUES('presentation',?1,'Message','light',0,'T','T')",[t]).unwrap();
    c.execute("INSERT INTO presentation_items(id,presentation_id,position,kind,source_kind,source_id,created_at,updated_at) VALUES('slide','presentation',0,'quote','finding',?1,'T','T')",[&f.id]).unwrap();
    f.id
}
fn fk(c: &Connection) {
    assert!(!c
        .prepare("PRAGMA foreign_key_check")
        .unwrap()
        .exists([])
        .unwrap());
}

fn assert_saved(
    original: &abiding_word_lib::domain::Finding,
    saved: &abiding_word_lib::domain::Finding,
) {
    let mut expected = original.clone();
    assert_eq!(expected.sources.len(), saved.sources.len());
    for (old, copy) in expected.sources.iter_mut().zip(&saved.sources) {
        old.id = copy.id.clone();
    }
    assert_eq!(&expected, saved);
}

#[test]
fn standalone_link_unlink_cross_project_and_multiple_sessions() {
    Database::open_in_memory()
        .unwrap()
        .with_conn(|c| {
            let p = project_id(c, "P");
            let q = project_id(c, "Q");
            let s = study(c, &p);
            let other = study(c, &q);
            let t = prep(c, &p, None);
            assert_eq!(teaching::get(c, &t)?.study_id, None);
            assert!(teaching::set_study(c, &t, Some(&other)).is_err());
            let linked = teaching::set_study(c, &t, Some(&s))?;
            assert_eq!(linked.study_id.as_deref(), Some(s.as_str()));
            prep(c, &p, Some(&s));
            assert_eq!(teaching::list(c, &s)?.len(), 2);
            attach(c, &t, &s);
            let before = teaching::get(c, &t)?;
            let detached = teaching::set_study(c, &t, None)?;
            assert!(detached.revision > before.revision);
            assert_eq!(detached.blocks, before.blocks);
            assert_saved(&before.findings[0].finding, &detached.findings[0].finding);
            assert!(detached.findings[0].snapshot);
            let again = teaching::set_study(c, &t, None)?;
            assert_eq!(again, detached);
            teaching::set_study(c, &t, Some(&s))?;
            teaching::set_study(c, &t, None)?;
            assert_eq!(teaching::get(c, &t)?.findings.len(), 1);
            assert_eq!(
                presentations::get(c, "presentation")?.items[0]
                    .source
                    .as_ref()
                    .unwrap()
                    .id,
                before.findings[0].finding.id
            );
            assert_eq!(teaching::list_project(c, &p)?.len(), 2);
            fk(c);
            Ok(())
        })
        .unwrap();
}

#[test]
fn study_delete_keeps_teaching_and_snapshots_including_trashed_sessions() {
    for permanent in [false, true] {
        Database::open_in_memory()
            .unwrap()
            .with_conn(|c| {
                let p = project_id(c, "P");
                let s = study(c, &p);
                let t = prep(c, &p, Some(&s));
                attach(c, &t, &s);
                let trashed = prep(c, &p, Some(&s));
                teaching::delete(c, &trashed)?;
                let before = teaching::get(c, &t)?;
                studies::delete(c, &s, permanent)?;
                assert!(studies::get(c, &s).is_err());
                let after = teaching::get(c, &t)?;
                assert_eq!(after.study_id, None);
                assert_eq!(after.blocks, before.blocks);
                assert_saved(&before.findings[0].finding, &after.findings[0].finding);
                assert!(after.findings[0].snapshot);
                let temp = tempfile::tempdir().unwrap();
                let store = ImageStore {
                    generated_dir: temp.path().join("g"),
                    offline_dir: temp.path().join("o"),
                    template_dir: temp.path().join("t"),
                };
                let doc = document::build(c, "presentation", &store)?;
                let audience = serde_json::to_string(&doc.audience()).unwrap();
                assert!(audience.contains("Saved observation"));
                assert!(!audience.contains("Remain in the vine."));
                assert_eq!(
                    after.findings[0].finding.sources[0].quote.as_deref(),
                    Some("Exact quotation")
                );
                assert_eq!(teaching::restore(c, &trashed)?.study_id, None);
                if !permanent {
                    studies::restore(c, &s)?;
                    assert_eq!(teaching::get(c, &t)?.study_id, None);
                }
                projects::delete(c, &p, false)?;
                assert!(teaching::get(c, &t).is_err());
                projects::restore(c, &p)?;
                assert_eq!(teaching::get(c, &t)?.blocks, after.blocks);
                teaching::delete(c, &trashed)?;
                projects::delete(c, &p, true)?;
                for table in [
                    "teaching_preparations",
                    "teaching_blocks",
                    "teaching_finding_links",
                    "presentations",
                    "presentation_items",
                ] {
                    assert_eq!(
                        c.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r
                            .get::<_, i64>(0))?,
                        0
                    );
                }
                fk(c);
                Ok(())
            })
            .unwrap();
    }
}

#[test]
fn standalone_snapshots_and_presentation_sources_move_and_restore() {
    let temp = tempfile::tempdir().unwrap();
    let dirs = DataDirs::ensure(temp.path().join("original")).unwrap();
    let db = Database::open(&dirs.database_file(), &dirs.pre_migration_backups()).unwrap();
    let p = db
        .with_conn(|c| {
            let p = project_id(c, "Portable");
            let s = study(c, &p);
            let t = prep(c, &p, Some(&s));
            attach(c, &t, &s);
            studies::delete(c, &s, true)?;
            prep(c, &p, None);
            Ok(p)
        })
        .unwrap();
    let package = temp.path().join("project.awproject");
    db.with_conn(|c| project::export(c, &p, &dirs.teaching_images(), &package))
        .unwrap();
    let other = DataDirs::ensure(temp.path().join("other")).unwrap();
    let dest = Database::open(&other.database_file(), &other.pre_migration_backups()).unwrap();
    let report = dest
        .with_conn(|c| project::import(c, &package, &other.teaching_images(), &other.cache()))
        .unwrap();
    assert_ne!(report.project_id, p);
    dest.with_conn(|c| {
        let list = teaching::list_project(c, &report.project_id)?;
        assert_eq!(list.len(), 2);
        assert!(list.iter().all(|p| p.study_id.is_none()));
        let mut snapshot = None;
        for t in list {
            let t = teaching::get(c, &t.id)?;
            assert_eq!(t.blocks[0].body, "Remain in the vine.");
            if !t.findings.is_empty() {
                snapshot = Some(t);
            }
        }
        let t = snapshot.unwrap();
        let f = &t.findings[0].finding;
        assert_eq!(f.content, "Saved observation — 한글");
        assert_eq!(f.passages[0].reference.canonical, "JHN.15.5");
        assert_eq!(f.sources[0].title, "Synthetic commentary");
        assert_eq!(f.sources[0].quote.as_deref(), Some("Exact quotation"));
        assert!(f.sources[0].unavailable);
        let pres = presentations::list(c, &t.id)?;
        let pres = presentations::get(c, &pres[0].id)?;
        assert_eq!(pres.items[0].source.as_ref().unwrap().id, f.id);
        fk(c);
        Ok(())
    })
    .unwrap();
    let saved = temp.path().join("all.awbackup");
    backup::create(&dest, &other, &saved).unwrap();
    dest.with_conn(|c| projects::delete(c, &report.project_id, true))
        .unwrap();
    backup::restore(&dest, &other, &saved).unwrap();
    dest.with_conn(|c| {
        assert_eq!(teaching::list_project(c, &report.project_id)?.len(), 2);
        fk(c);
        Ok(())
    })
    .unwrap();
}

#[test]
fn current_schema_and_nullable_wire_input_are_explicit() {
    Database::open_in_memory()
        .unwrap()
        .with_conn(|c| {
            assert_eq!(abiding_word_lib::db::migrations::current_version(c)?, 26);
            let p = project_id(c, "Wire input");
            for input in [
                serde_json::json!({"projectId":p,"title":"No study"}),
                serde_json::json!({"projectId":p,"studyId":null,"title":"Explicit null"}),
            ] {
                let input: CreateTeaching = serde_json::from_value(input).unwrap();
                assert!(teaching::create(c, &input)?.study_id.is_none());
            }
            fk(c);
            Ok(())
        })
        .unwrap();
}

#[test]
fn saved_citation_reopens_exact_library_words_after_study_deletion() {
    use abiding_word_lib::db::library;
    use abiding_word_lib::library::{
        import, CitedPassage, CommitSourceImport, CreateFindingFromSource, GetSegments,
    };
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("synthetic.md");
    std::fs::write(
        &path,
        "# Vine\n\nThese are exact invented words about abiding.\n",
    )
    .unwrap();
    Database::open_in_memory()
        .unwrap()
        .with_conn(|c| {
            let p = project_id(c, "Citation project");
            let s = study(c, &p);
            let t = prep(c, &p, Some(&s));
            let staged = import::stage(&path)?;
            let id = "exact-source";
            library::commit_import(
                c,
                id,
                &staged,
                &CommitSourceImport {
                    preview_id: staged.id.clone(),
                    title: "Exact invented source".into(),
                    author: Some("Synthetic author".into()),
                    ..Default::default()
                },
                &library::managed_relative_path(id, &staged),
            )?;
            let segments = library::segments(
                c,
                &GetSegments {
                    source_id: id.into(),
                    around_segment_id: None,
                    around_page: None,
                    offset: None,
                    limit: None,
                },
            )?;
            let segment = segments
                .segments
                .iter()
                .find(|s| s.text.contains("invented words"))
                .unwrap();
            let f = findings::create_from_source(
                c,
                &CreateFindingFromSource {
                    study_id: s.clone(),
                    finding_type: "quotation".into(),
                    content: Some("My own note".into()),
                    section_id: None,
                    cited: CitedPassage {
                        source_id: Some(id.into()),
                        start: Some(abiding_word_lib::library::annotations::TextPoint {
                            segment_id: segment.id.clone(),
                            offset: 0,
                        }),
                        end: Some(abiding_word_lib::library::annotations::TextPoint {
                            segment_id: segment.id.clone(),
                            offset: segment.text.encode_utf16().count() as u32,
                        }),
                        quote: Some(segment.text.clone()),
                        ..Default::default()
                    },
                },
            )?;
            teaching::link_finding(
                c,
                &LinkTeachingFinding {
                    preparation_id: t.clone(),
                    point_id: None,
                    finding_id: f.id.clone(),
                },
            )?;
            let before = findings::locate_source(c, &f.sources[0].id)?.unwrap();
            // Historical citations can lack snapshot metadata while their
            // original Library source still proves title and exact file hash.
            c.execute("UPDATE finding_sources SET cited_title=NULL,cited_content_hash=NULL WHERE id=?1",[&f.sources[0].id])?;
            teaching::set_study(c, &t, None)?;
            let kept = teaching::get(c, &t)?;
            assert_saved(&f, &kept.findings[0].finding);
            // The original citation is still mutable. The saved copy must
            // open its own exact words, before AND after deleting the Study.
            c.execute("UPDATE finding_sources SET quote='Different live words',anchor=NULL,source_segment_id=NULL,cited_section=NULL WHERE id=?1",[&f.sources[0].id])?;
            let saved_id = &kept.findings[0].finding.sources[0].id;
            assert_ne!(saved_id, &f.sources[0].id);
            assert_eq!(
                findings::locate_source(c, saved_id)?.unwrap(),
                before
            );
            studies::delete(c, &s, true)?;
            assert_eq!(findings::locate_source(c, saved_id)?.unwrap(), before);
            let raw: String = c.query_row(
                "SELECT snapshot FROM teaching_finding_links WHERE preparation_id=?1",
                [&t],
                |r| r.get(0),
            )?;
            let raw: serde_json::Value = serde_json::from_str(&raw).unwrap();
            assert_eq!(raw["citations"][0]["cited_content_hash"], staged.hash);
            assert_eq!(raw["citations"][0]["originCitationId"], f.sources[0].id);
            fk(c);
            Ok(())
        })
        .unwrap();
}

#[test]
fn a_deleted_live_finding_is_not_resurrected_from_its_cache_on_import() {
    let temp = tempfile::tempdir().unwrap();
    let dirs = DataDirs::ensure(temp.path().join("source")).unwrap();
    let db = Database::open(&dirs.database_file(), &dirs.pre_migration_backups()).unwrap();
    let p = db
        .with_conn(|c| {
            let p = project_id(c, "Deleted finding");
            let s = study(c, &p);
            let t = prep(c, &p, Some(&s));
            let f = attach(c, &t, &s);
            c.execute("UPDATE findings SET deleted_at='T' WHERE id=?1", [f])?;
            assert!(teaching::get(c, &t)?.findings.is_empty());
            Ok(p)
        })
        .unwrap();
    let path = temp.path().join("deleted.awproject");
    db.with_conn(|c| project::export(c, &p, &dirs.teaching_images(), &path))
        .unwrap();
    let dest = Database::open_in_memory().unwrap();
    dest.with_conn(|c| {
        let imported = project::import(
            c,
            &path,
            &temp.path().join("images"),
            &temp.path().join("cache"),
        )?;
        let list = teaching::list_project(c, &imported.project_id)?;
        assert_eq!(list.len(), 1);
        assert!(teaching::get(c, &list[0].id)?.findings.is_empty());
        fk(c);
        Ok(())
    })
    .unwrap();
    db.with_conn(|c| {
        let list = teaching::list_project(c, &p)?;
        let detached = teaching::set_study(c, &list[0].id, None)?;
        assert!(detached.findings.is_empty());
        assert_eq!(
            c.query_row(
                "SELECT COUNT(*) FROM teaching_finding_links WHERE deleted_at IS NOT NULL",
                [],
                |r| r.get::<_, i64>(0)
            )?,
            1
        );
        fk(c);
        Ok(())
    })
    .unwrap();
}
