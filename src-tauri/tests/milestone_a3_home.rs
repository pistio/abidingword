//! Disposable ownership and backup regression; no migrations or real user data.
use abiding_word_lib::db::{findings, projects, studies, teaching, workspaces, Database};
use abiding_word_lib::domain::{
    CreateFinding, CreateProject, CreateStudy, CreateTeaching, FindingType, LinkTeachingFinding,
    NewTeachingBlock,
};
use abiding_word_lib::filesystem::DataDirs;
use abiding_word_lib::portability::backup;

#[test]
fn root_adopts_only_an_unambiguous_legacy_default_and_survives_language_changes() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|c| {
        let w = workspaces::get_default(c)?.id;
        let named = projects::create(
            c,
            &CreateProject {
                workspace_id: w.clone(),
                title: "Named folder".into(),
                description: None,
            },
        )?;
        let old = projects::create(
            c,
            &CreateProject {
                workspace_id: w.clone(),
                title: "Meine Bibelstudien".into(),
                description: None,
            },
        )?;
        assert_eq!(projects::root(c, &w, "My studies")?.id, old.id);
        assert_eq!(projects::root(c, &w, "내 성경공부")?.id, old.id);
        assert_eq!(projects::get(c, &named.id)?.title, "Named folder");
        assert_eq!(projects::list(c, &w)?.len(), 2);
        c.execute("INSERT INTO workspaces(id,name,is_default,created_at,updated_at) VALUES('ambiguous','Another workspace',0,'T','T')", [])?;
        let mut ambiguous = Vec::new();
        for title in ["My studies", "Mis estudios"] {
            ambiguous.push(projects::create(c, &CreateProject { workspace_id: "ambiguous".into(), title: title.into(), description: None })?.id);
        }
        let other_root = projects::root(c, "ambiguous", "My studies")?;
        assert!(!ambiguous.contains(&other_root.id), "ambiguous legacy containers remain named folders");
        let study = studies::create(c, &CreateStudy { project_id: old.id.clone(), title: "Keep ownership".into(), ..Default::default() })?.study;
        assert!(studies::move_to(c, &study.id, &other_root.id).is_err());
        assert_eq!(studies::get(c, &study.id)?.project_id, old.id);
        Ok(())
    })
    .unwrap();
}

#[test]
fn moving_a_study_preserves_linked_and_trashed_teaching_provenance_and_backup_identity() {
    let tmp = tempfile::tempdir().unwrap();
    let dirs = DataDirs::ensure(tmp.path().join("profile")).unwrap();
    let db = Database::open(&dirs.database_file(), &dirs.pre_migration_backups()).unwrap();
    let (w, root, folder, study, active, trashed, standalone) = db.with_conn(|c| {
        let w = workspaces::get_default(c)?.id;
        let root = projects::root(c, &w, "My studies")?.id;
        let folder = projects::create(c, &CreateProject { workspace_id: w.clone(), title: "Folder".into(), description: None })?.id;
        let study = studies::create(c, &CreateStudy { project_id: root.clone(), title: "Research".into(), ..Default::default() })?.study.id;
        let create = |linked: bool| teaching::create(c, &CreateTeaching {
            project_id: Some(root.clone()), study_id: if linked { study.clone() } else { String::new() }, title: "Saved message".into(), template_version: Some(2),
            blocks: vec![NewTeachingBlock { block_type: "writing".into(), title: "Private answer".into(), body: "Private manuscript — 한글".into(), participant_facing: Some(false), ..Default::default() }], ..Default::default()
        });
        let active = create(true)?.id; let trashed = create(true)?.id; let standalone = create(false)?.id;
        teaching::delete(c, &trashed)?;
        c.execute("INSERT INTO presentations(id,preparation_id,title,theme,position,created_at,updated_at) VALUES('kept-slide-deck',?1,'Saved presentation','light',0,'T','T')", [&active])?;
        let finding = findings::create(c, &CreateFinding { study_id: study.clone(), section_id: None, finding_type: FindingType::Quotation, content: Some("Invented research with exact citation".into()), passage: Some("JHN.15.5".into()) })?;
        c.execute("INSERT INTO finding_sources(id,finding_id,cited_title,quote,locator,created_at,updated_at) VALUES('kept-citation',?1,'Invented source','Exact invented quote','p. 7','T','T')", [&finding.id])?;
        teaching::link_finding(c, &LinkTeachingFinding { preparation_id: active.clone(), point_id: None, finding_id: finding.id })?;
        let before = teaching::get(c, &active)?;
        c.execute_batch("CREATE TEMP TRIGGER fail_move BEFORE UPDATE OF project_id ON teaching_preparations WHEN NEW.study_id IS NOT NULL BEGIN SELECT RAISE(ABORT,'injected move failure'); END;")?;
        assert!(studies::move_to(c, &study, &folder).is_err());
        assert_eq!(studies::get(c, &study)?.project_id, root);
        assert_eq!(teaching::get(c, &active)?.study_id, Some(study.clone()));
        assert_eq!(teaching::get(c, &active)?.project_id, root);
        c.execute_batch("DROP TRIGGER fail_move")?;
        studies::move_to(c, &study, &folder)?;
        let after = teaching::get(c, &active)?;
        assert_eq!(after.study_id, before.study_id);
        assert_eq!(after.blocks, before.blocks);
        assert_eq!(serde_json::to_value(&after.findings).unwrap(), serde_json::to_value(&before.findings).unwrap());
        assert_eq!(after.project_id, folder);
        assert_eq!(teaching::get(c, &standalone)?.project_id, root);
        teaching::restore(c, &trashed)?;
        assert_eq!(teaching::get(c, &trashed)?.project_id, folder);
        assert!(studies::move_to(c, &study, "missing-folder").is_err());
        assert_eq!(studies::get(c, &study)?.project_id, folder);
        assert_eq!(c.query_row("SELECT preparation_id FROM presentations WHERE id='kept-slide-deck'", [], |r| r.get::<_,String>(0))?, active);
        assert!(!c.prepare("PRAGMA foreign_key_check")?.exists([])?);
        Ok((w, root, folder, study, active, trashed, standalone))
    }).unwrap();
    let saved = tmp.path().join("workspace.awbackup");
    backup::create(&db, &dirs, &saved).unwrap();
    db.with_conn(|c| {
        studies::move_to(c, &study, &root)?;
        Ok(())
    })
    .unwrap();
    backup::restore(&db, &dirs, &saved).unwrap();
    db.with_conn(|c| {
        let marker = c.query_row(
            "SELECT value FROM settings WHERE key=?1",
            [format!("studies.rootProject.{w}")],
            |row| row.get::<_, String>(0),
        )?;
        assert_eq!(
            marker,
            serde_json::to_string(&root).unwrap(),
            "backup restores the actual root marker, not just a name-based fallback"
        );
        assert_eq!(projects::root(c, &w, "Mes études")?.id, root);
        assert_eq!(studies::get(c, &study)?.project_id, folder);
        for id in [active, trashed] {
            assert_eq!(teaching::get(c, &id)?.study_id, Some(study.clone()));
        }
        assert_eq!(teaching::get(c, &standalone)?.study_id, None);
        studies::move_to(c, &study, &root)?;
        projects::delete(c, &folder, true)?;
        assert_eq!(studies::get(c, &study)?.project_id, root);
        assert!(!c.prepare("PRAGMA foreign_key_check")?.exists([])?);
        Ok(())
    })
    .unwrap();
}

#[test]
fn concurrent_app_connections_create_one_root_container() {
    use std::sync::{Arc, Barrier};
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("concurrent.db");
    let db = Database::open(&path, &tmp.path().join("backups")).unwrap();
    let workspace = db
        .with_conn(|c| Ok(workspaces::get_default(c)?.id))
        .unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let jobs: Vec<_> = (0..2)
        .map(|_| {
            let path = path.clone();
            let workspace = workspace.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                let conn = rusqlite::Connection::open(path).unwrap();
                conn.busy_timeout(std::time::Duration::from_secs(5))
                    .unwrap();
                conn.pragma_update(None, "foreign_keys", true).unwrap();
                barrier.wait();
                projects::root(&conn, &workspace, "My studies").unwrap().id
            })
        })
        .collect();
    let ids: Vec<_> = jobs.into_iter().map(|job| job.join().unwrap()).collect();
    assert_eq!(ids[0], ids[1]);
    db.with_conn(|c| {
        assert_eq!(projects::list(c, &workspace)?.len(), 1);
        Ok(())
    })
    .unwrap();
}
