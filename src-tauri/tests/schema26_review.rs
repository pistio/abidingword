//! Independent on-disk review. Only disposable directories and invented data.
use abiding_word_lib::db::{
    migrations::{self, MIGRATIONS},
    projects, studies, teaching, Database,
};
use abiding_word_lib::domain::{StudyStatus, UpdateStudy};
use rusqlite::{params, types::Value, Connection};

const TEACHING: &[&str] = &[
    "teaching_preparations",
    "teaching_points",
    "teaching_blocks",
    "teaching_passages",
    "teaching_finding_links",
    "teaching_materials",
    "presentations",
    "presentation_items",
];
type Snapshot = Vec<(String, Vec<String>, Vec<Vec<Value>>)>;
fn integrity(c: &Connection) {
    assert!(c
        .pragma_query_value(None, "foreign_keys", |r| r.get::<_, bool>(0))
        .unwrap());
    assert!(!c
        .prepare("PRAGMA foreign_key_check")
        .unwrap()
        .exists([])
        .unwrap());
    assert_eq!(
        c.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
}
fn seed(c: &mut Connection, version: usize) {
    c.pragma_update(None, "foreign_keys", true).unwrap();
    migrations::run_with(c, &MIGRATIONS[..version]).unwrap();
    c.execute_batch(r#"
INSERT INTO workspaces(id,name,is_default,created_at,updated_at) VALUES('w','Review workspace',1,'T','T');
INSERT INTO projects(id,workspace_id,title,created_at,updated_at) VALUES('p','w','Review project','T','T');
INSERT INTO study_templates(id,key,name,study_type,is_builtin,definition,created_at,updated_at) VALUES('personal','review-personal','Personal review','custom',0,'{"sections":[{"key":"answer","title":"My answer"}]}','T','T');
INSERT INTO studies(id,project_id,template_id,title,study_type,created_at,updated_at,revision) VALUES('s','p','personal','Structured study','custom','T','T',7);
INSERT INTO study_sections(id,study_id,key,title,position,created_at,updated_at) VALUES('sec','s','answer','Structured answer',0,'T','T');
INSERT INTO study_blocks(id,section_id,block_type,position,content,data,created_at,updated_at,revision) VALUES('answer','sec','paragraph',0,'Keep my answer — 한글','{"entry":{"text":"Exact structured entry"}}','T','T',11);
INSERT INTO findings(id,study_id,section_id,kind,content,created_at,updated_at) VALUES('f','s','sec','observation','Retain this finding','T','T');
INSERT INTO sources(id,workspace_id,source_type,title,created_at,updated_at) VALUES('source','w','book','Invented source','T','T');
INSERT INTO finding_sources(id,finding_id,source_id,cited_title,quote,locator,created_at,updated_at) VALUES('cite','f','source','Invented source','Exact cited words','p. 2','T','T');
INSERT INTO teaching_templates(id,name,kind,definition,created_at,updated_at) VALUES('v1-template','Old personal template','sermon','{"points":[{"title":"Kept legacy point"}]}','T','T');
"#).unwrap();
    for (i, id) in ["legacy", "blocks", "trash"].iter().enumerate() {
        let deleted = (*id == "trash").then_some("T");
        c.execute("INSERT INTO teaching_preparations(id,study_id,title,kind,introduction,notes,position,created_at,updated_at,deleted_at,revision) VALUES(?1,'s',?1,'sermon','Exact introduction','Private teacher note',?2,'T','T',?3,9)",params![id,i as i64,deleted]).unwrap();
        let pt = format!("pt-{id}");
        let pr = format!("pr-{id}");
        c.execute("INSERT INTO teaching_points(id,preparation_id,position,title,explanation,created_at,updated_at) VALUES(?1,?2,0,'Point','Exact explanation','T','T')",params![pt,id]).unwrap();
        c.execute("INSERT INTO teaching_passages(id,preparation_id,point_id,book,start_chapter,start_verse,end_chapter,end_verse,position,created_at) VALUES(?1,?2,?3,'JHN',15,1,15,5,0,'T')",params![format!("ps-{id}"),id,pt]).unwrap();
        c.execute("INSERT INTO teaching_finding_links(id,preparation_id,point_id,finding_id,position,created_at) VALUES(?1,?2,?3,'f',0,'T')",params![format!("fl-{id}"),id,pt]).unwrap();
        c.execute("INSERT INTO teaching_materials(id,preparation_id,point_id,kind,standing,title,content,position,created_at) VALUES(?1,?2,?3,'illustration','hypothetical','Invented example','Retained material',0,'T')",params![format!("m-{id}"),id,pt]).unwrap();
        c.execute("INSERT INTO presentations(id,preparation_id,title,theme,position,created_at,updated_at) VALUES(?1,?2,'Retained slides','light',0,'T','T')",params![pr,id]).unwrap();
        c.execute("INSERT INTO presentation_items(id,presentation_id,position,kind,source_kind,source_id,notes,created_at,updated_at) VALUES(?1,?2,0,'heading','point',?3,'Speaker only','T','T')",params![format!("slide-{id}"),pr,pt]).unwrap();
    }
    if version >= 24 {
        c.execute("UPDATE study_sections SET fields=?1,status='in_progress' WHERE id='sec'",[r#"{"template":"personal","version":1,"fields":[{"key":"answer","kind":"text","label":"Answer"}]}"#]).unwrap();
        c.execute(
            "UPDATE study_blocks SET field_key='answer' WHERE id='answer'",
            [],
        )
        .unwrap();
    }
    if version >= 25 {
        c.execute("UPDATE teaching_preparations SET template_key='sermon-talk',template_version=2 WHERE id='blocks'",[]).unwrap();
        c.execute_batch("INSERT INTO teaching_blocks(id,preparation_id,position,block_type,title,body,config,created_at,updated_at,revision) VALUES('b','blocks',0,'writing','Kept heading','Exact block body','{\"participantFacing\":false}','T','T',4); INSERT INTO teaching_blocks(id,preparation_id,position,block_type,title,body,created_at,updated_at,deleted_at) VALUES('b-trash','trash',0,'writing','Old heading','Tombstoned body','T','T','T');").unwrap();
    }
    integrity(c);
}
fn capture(c: &Connection) -> Snapshot {
    let mut names = TEACHING.to_vec();
    names.extend([
        "studies",
        "study_sections",
        "study_blocks",
        "findings",
        "finding_sources",
        "teaching_templates",
        "study_templates",
    ]);
    names
        .into_iter()
        .filter(|t| c.prepare(&format!("SELECT * FROM {t}")).is_ok())
        .map(|table| {
            let columns = c
                .prepare(&format!("PRAGMA table_info({table})"))
                .unwrap()
                .query_map([], |r| r.get::<_, String>(1))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            let rows = read(c, table, &columns);
            (table.into(), columns, rows)
        })
        .collect()
}
fn read(c: &Connection, table: &str, cols: &[String]) -> Vec<Vec<Value>> {
    let filter = if table == "study_templates" {
        " WHERE is_builtin=0"
    } else {
        ""
    };
    c.prepare(&format!(
        "SELECT {} FROM {table}{filter} ORDER BY id",
        cols.iter()
            .map(|s| format!("\"{s}\""))
            .collect::<Vec<_>>()
            .join(",")
    ))
    .unwrap()
    .query_map([], |r| {
        (0..cols.len())
            .map(|i| r.get(i))
            .collect::<Result<Vec<Value>, _>>()
    })
    .unwrap()
    .collect::<Result<Vec<_>, _>>()
    .unwrap()
}
fn preserved(c: &Connection, before: &Snapshot) {
    for (t, cols, rows) in before {
        assert_eq!(&read(c, t, cols), rows, "{t}: original values changed");
    }
    integrity(c);
}
#[test]
fn on_disk_19_24_25_upgrade_keeps_answers_graph_history_and_one_backup_across_restarts() {
    for version in [19, 24, 25] {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("Review — 한글 with spaces.db");
        let backups = temp.path().join("backups");
        let mut c = Connection::open(&path).unwrap();
        seed(&mut c, version);
        let before = capture(&c);
        let history = c
            .prepare("SELECT version,name,applied_at FROM schema_versions ORDER BY version")
            .unwrap()
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        drop(c);
        for _ in 0..3 {
            let db = Database::open(&path, &backups).unwrap();
            db.with_conn(|c| {
                assert_eq!(migrations::current_version(c)?, 26);
                preserved(c, &before);
                for (v, n, t) in &history {
                    assert_eq!(
                        c.query_row(
                            "SELECT name,applied_at FROM schema_versions WHERE version=?1",
                            [v],
                            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
                        )?,
                        (n.clone(), t.clone())
                    );
                }
                Ok(())
            })
            .unwrap();
            assert_eq!(std::fs::read_dir(&backups).unwrap().count(), 1);
        }
        let backup = std::fs::read_dir(&backups)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let c = Connection::open(backup).unwrap();
        c.pragma_update(None, "foreign_keys", true).unwrap();
        assert_eq!(migrations::current_version(&c).unwrap(), version as i64);
        preserved(&c, &before);
    }
}
#[test]
fn archive_unlink_delete_restore_and_project_removal_retain_all_teaching_ids_until_owner_removal() {
    let mut c = Connection::open_in_memory().unwrap();
    seed(&mut c, 25);
    migrations::run(&mut c).unwrap();
    let ids = TEACHING
        .iter()
        .map(|t| (t.to_string(), read(&c, t, &["id".into()])))
        .collect::<Vec<_>>();
    studies::update(
        &c,
        &UpdateStudy {
            id: "s".into(),
            status: Some(StudyStatus::Archived),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(teaching::list_project(&c, "p").unwrap().len(), 2);
    teaching::set_study(&c, "blocks", None).unwrap();
    teaching::set_study(&c, "blocks", Some("s")).unwrap();
    studies::delete(&c, "s", false).unwrap();
    studies::restore(&c, "s").unwrap();
    studies::delete(&c, "s", true).unwrap();
    teaching::restore(&c, "trash").unwrap();
    for id in ["legacy", "blocks", "trash"] {
        let t = teaching::get(&c, id).unwrap();
        assert!(t.study_id.is_none());
        assert_eq!(
            t.findings[0].finding.sources[0].quote.as_deref(),
            Some("Exact cited words")
        );
    }
    projects::delete(&c, "p", false).unwrap();
    assert!(teaching::get(&c, "blocks").is_err());
    projects::restore(&c, "p").unwrap();
    for (t, expected) in ids {
        assert_eq!(
            read(&c, &t, &["id".into()]),
            expected,
            "{t}: lifecycle lost an ID"
        );
    }
    integrity(&c);
    projects::delete(&c, "p", true).unwrap();
    for t in TEACHING {
        assert_eq!(
            c.query_row(&format!("SELECT COUNT(*) FROM {t}"), [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
    integrity(&c);
}
