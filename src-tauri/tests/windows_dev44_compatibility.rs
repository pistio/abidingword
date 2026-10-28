//! Synthetic databases only. Historical SQL is pinned to ac4d7f4, not inferred
//! from the schema number of the current build. No installed user data is read.
use abiding_word_lib::db::{
    migrations::{self, Migration, MIGRATIONS},
    templates, Database,
};
use rusqlite::{types::Value, Connection};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;

const SEED: &str = r#"
INSERT INTO workspaces(id,name,is_default,created_at,updated_at) VALUES('w','Kept workspace',1,'t','t');
INSERT INTO projects(id,workspace_id,title,created_at,updated_at) VALUES('p','w','Kept project','t','t');
INSERT INTO study_templates(id,key,name,study_type,is_builtin,definition,created_at,updated_at)
 VALUES('custom','user-kept','Kept template','custom',0,'{"sections":[{"key":"my-part","title":"My own words","phase":"understanding"}]}','t','t');
INSERT INTO studies(id,project_id,template_id,title,study_type,created_at,updated_at)
 VALUES('s','p','custom','Kept study','custom','t','t');
INSERT INTO study_sections(id,study_id,key,title,guidance,phase,position,created_at,updated_at)
 VALUES('sec','s','my-part','Kept heading','Kept guidance','understanding',0,'t','t');
INSERT INTO study_blocks(id,section_id,block_type,position,content,data,created_at,updated_at)
 VALUES('block','sec','paragraph',0,'Kept writing — 한글','{"legacy":"metadata"}','t','t');
INSERT INTO findings(id,study_id,section_id,kind,content,created_at,updated_at) VALUES('note','s','sec','observation','Kept note','t','t');
INSERT INTO sources(id,workspace_id,source_type,title,created_at,updated_at) VALUES('source','w','book','Kept book','t','t');
INSERT INTO finding_sources(id,finding_id,source_id,cited_title,quote,created_at,updated_at) VALUES('cite','note','source','Kept book','Kept quotation','t','t');
INSERT INTO teaching_preparations(id,study_id,title,kind,introduction,position,created_at,updated_at) VALUES('teaching','s','Kept teaching','sermon','Kept introduction',0,'t','t');
INSERT INTO teaching_points(id,preparation_id,position,kind,title,explanation,created_at,updated_at) VALUES('point','teaching',0,'point','Kept point','Kept explanation','t','t');
INSERT INTO presentations(id,preparation_id,title,position,created_at,updated_at) VALUES('slides','teaching','Kept presentation',0,'t','t');
INSERT INTO presentation_items(id,presentation_id,position,kind,body,notes,created_at,updated_at) VALUES('slide','slides',0,'text','Kept audience text','Kept private notes','t','t');
INSERT INTO presentation_templates(id,name,definition,created_at,updated_at) VALUES('design','Kept design','{"original":true}','t','t');
INSERT INTO teaching_templates(id,name,kind,definition,created_at,updated_at) VALUES('outline','Kept outline','sermon','{"points":[]}','t','t');
INSERT INTO bible_translations(id,name,short_name,language,kind,source_type,source_origin,import_status,created_at,updated_at)
 VALUES('bible','Kept Bible','KEEP','en','structured','docx','managed_copy','ready','t','t');
INSERT OR IGNORE INTO bible_verses(id,book_id,chapter,verse) VALUES(43015005,43,15,5);
INSERT INTO bible_verse_texts(translation_id,verse_id,text) VALUES('bible',43015005,'Kept verse text');
INSERT INTO bible_word_annotations(id,translation_id,source_format,word_count,verse_count,lemma_count,strong_count,morph_count,morph_scheme,details,created_at,grammar_scheme,grammar_basis,grammar_evidence)
 VALUES(1,'bible','usfm',1,1,1,1,1,'raw','{}','t','robinson','person','{"kept":true}');
INSERT INTO bible_lemma_entries(id,annotation_id,lemma,lemma_fold,strong,occurrences) VALUES(1,1,'μένω','μενω','G3306',1);
INSERT INTO bible_entry_ids(value,entry_id) VALUES('G3306',1);
INSERT INTO bible_words(annotation_id,verse_id,position,start_offset,end_offset,surface,shape,entry_id,morph) VALUES(1,43015005,0,0,4,'Kept','w',1,'V-PAI-1S');
INSERT INTO bible_documents(id,translation_id,position,format,file_name,original_file_name,sha256,size_bytes,extraction_status,created_at)
 VALUES('document','bible',0,'docx','kept.docx','original.docx','original-sha',12,'extracted_text','t');
INSERT INTO bible_document_segments(document_id,position,content) VALUES('document',0,'Kept Bible document');
INSERT INTO bible_verse_origins(translation_id,verse_id,document_id,first_segment,last_segment) VALUES('bible',43015005,'document',0,0);
"#;

type Rows = Vec<Vec<Value>>;
type Snapshot = BTreeMap<String, (Vec<String>, Rows)>;
fn quote(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}
fn tables(conn: &Connection) -> Vec<String> {
    conn.prepare("SELECT name FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name <> 'schema_versions' ORDER BY name").unwrap()
        .query_map([], |r| r.get(0)).unwrap().collect::<Result<_,_>>().unwrap()
}
fn read_rows(conn: &Connection, table: &str, columns: &[String]) -> Rows {
    let filter = if table == "study_templates" {
        " WHERE is_builtin=0"
    } else {
        ""
    };
    let sql = format!(
        "SELECT {} FROM {}{}",
        columns
            .iter()
            .map(|s| quote(s))
            .collect::<Vec<_>>()
            .join(","),
        quote(table),
        filter
    );
    conn.prepare(&sql)
        .unwrap()
        .query_map([], |r| {
            columns.iter().enumerate().map(|(i, _)| r.get(i)).collect()
        })
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}
fn snapshot(conn: &Connection) -> Snapshot {
    tables(conn)
        .into_iter()
        .map(|table| {
            let cols: Vec<String> = conn
                .prepare(&format!("PRAGMA table_info({})", quote(&table)))
                .unwrap()
                .query_map([], |r| r.get(1))
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap();
            let rows = read_rows(conn, &table, &cols);
            (table, (cols, rows))
        })
        .collect()
}
fn history(conn: &Connection) -> Vec<(i64, String, String)> {
    conn.prepare("SELECT version,name,applied_at FROM schema_versions ORDER BY version")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}
fn assert_preserved(conn: &Connection, before: &Snapshot) {
    for (table, (columns, rows)) in before {
        let after = read_rows(conn, table, columns);
        for row in rows {
            assert!(
                after.contains(row),
                "{table}: an original row changed: {row:?}"
            );
        }
    }
    let integrity: String = conn
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");
    assert!(!conn
        .prepare("PRAGMA foreign_key_check")
        .unwrap()
        .exists([])
        .unwrap());
}

#[test]
fn dev43_v19_objects_rows_history_and_backup_survive_upgrade_and_restart() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("original.db");
    let backups = temp.path().join("backups");
    let mut historical = Connection::open(&path).unwrap();
    migrations::run_with(&mut historical, &MIGRATIONS[..19]).unwrap();
    historical.execute_batch(SEED).unwrap();
    let before = snapshot(&historical);
    let original_history = history(&historical);
    let objects: Vec<(String, String)> = historical
        .prepare(
            "SELECT type,name FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%' ORDER BY type,name",
        )
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    // First prove the actual migrations preserve every original value exactly.
    migrations::upgrade(&mut historical, MIGRATIONS, Some(&backups)).unwrap();
    assert_preserved(&historical, &before);
    // Existing startup maintenance refreshes only derived source passage links.
    // Capture that separately, rather than confusing it with a schema migration.
    abiding_word_lib::db::library::ensure_passage_index(&historical).unwrap();
    let startup_before = snapshot(&historical);
    drop(historical);
    let db = Database::open(&path, &backups).unwrap();
    let installed=db.with_conn(|conn| {
        assert_eq!(migrations::current_version(conn)?,24); assert_preserved(conn,&startup_before);
        assert_eq!(&history(conn)[..19],original_history.as_slice());
        assert_eq!(history(conn)[19..].iter().map(|r|r.0).collect::<Vec<_>>(),vec![20,21,22,23,24]);
        for (kind,name) in &objects { assert!(conn.query_row("SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type=?1 AND name=?2)",[kind,name],|r|r.get::<_,bool>(0))?); }
        assert_eq!(conn.query_row("SELECT status || ':' || COALESCE(fields,'NULL') FROM study_sections WHERE id='sec'",[],|r|r.get::<_,String>(0))?,"not_started:NULL");
        templates::list(conn)
    }).unwrap();
    drop(db);
    let paths = fs::read_dir(&backups)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect::<Vec<_>>();
    assert_eq!(paths.len(), 1);
    let original = Connection::open(&paths[0]).unwrap();
    assert_eq!(migrations::current_version(&original).unwrap(), 19);
    assert_preserved(&original, &before);
    assert_eq!(history(&original), original_history);
    drop(original);
    for _ in 0..2 {
        let reopened = Database::open(&path, &backups).unwrap();
        reopened
            .with_conn(|conn| {
                assert_preserved(conn, &startup_before);
                assert_eq!(templates::list(conn)?, installed);
                Ok(())
            })
            .unwrap();
    }
    assert_eq!(
        fs::read_dir(&backups).unwrap().count(),
        1,
        "no backup or migrations on repeated startup"
    );
    let restored = temp.path().join("restored.db");
    fs::copy(&paths[0], &restored).unwrap();
    let restored = Database::open(&restored, &temp.path().join("restore-backup")).unwrap();
    restored
        .with_conn(|conn| {
            assert_preserved(conn, &startup_before);
            assert_eq!(&history(conn)[..19], original_history.as_slice());
            Ok(())
        })
        .unwrap();
}

#[test]
fn fresh_schema_contains_both_presentations_and_structured_templates() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        assert_eq!(history(conn).iter().map(|r|r.0).collect::<Vec<_>>(),(1..=24).collect::<Vec<_>>());
        assert_eq!(history(conn)[9].1,"presentations"); assert_eq!(history(conn)[23].1,"section_status_and_fields");
        assert!(conn.prepare("SELECT * FROM presentations, presentation_items, bible_documents, bible_words, study_sections WHERE status='not_started'").is_ok());
        for key in ["pozetek-study","ten-step-exegesis","biblical-character-study","word-study","whole-book-study","topical-study"] { assert!(templates::get_by_key(conn,key)?.is_builtin); }
        Ok(())
    }).unwrap();
}

#[test]
fn conflicting_main_v10_is_refused_without_relabeling_or_applying_any_migration() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("main10.db");
    let mut conn = Connection::open(&path).unwrap();
    migrations::run_with(&mut conn, &MIGRATIONS[..9]).unwrap();
    let main10 = Migration {
        version: 10,
        name: "section_status_and_fields",
        sql: include_str!("fixtures/main-template-migration-10.sql"),
    };
    let mut main_history: Vec<_> = MIGRATIONS[..9]
        .iter()
        .map(|m| Migration {
            version: m.version,
            name: m.name,
            sql: m.sql,
        })
        .collect();
    main_history.push(main10);
    migrations::run_with(&mut conn, &main_history).unwrap();
    let before = history(&conn);
    let schema: String = conn
        .query_row("SELECT group_concat(sql) FROM sqlite_schema", [], |r| {
            r.get(0)
        })
        .unwrap();
    drop(conn);
    assert!(Database::open(&path, &temp.path().join("backups")).is_err());
    let conn = Connection::open(&path).unwrap();
    assert_eq!(history(&conn), before);
    assert_eq!(
        conn.query_row("SELECT group_concat(sql) FROM sqlite_schema", [], |r| {
            r.get::<_, String>(0)
        })
        .unwrap(),
        schema
    );
    assert!(!temp.path().join("backups").exists());
}

#[test]
fn incomplete_history_is_not_falsely_marked_as_applied() {
    let mut conn = Connection::open_in_memory().unwrap();
    migrations::run_with(&mut conn, &MIGRATIONS[..19]).unwrap();
    conn.execute("DELETE FROM schema_versions WHERE version=11", [])
        .unwrap();
    let before = history(&conn);
    assert!(migrations::run(&mut conn).is_err());
    assert_eq!(history(&conn), before);
}

#[test]
fn development_migrations_one_through_twenty_three_are_byte_identical() {
    let expected = [
        "a0631bbc74b12b9900074ce4aaeb3437a0bec8c1814ec0848bf3379de2bbdcc6", // 0001_initial_schema.sql
        "7aa670dfddfeb164c579d6a2d2dbda8c59cff26bf121aee614746d439c8ec9e7", // 0002_scripture.sql
        "edbf84bb0d15f379a0e46e836f4082f4f9627332cbdb6378ffee67006d909ac5", // 0003_studies.sql
        "6a69101c45e35ddfd9492728f282f2cae7bd0ad3e7851523ac8fc37e03c34f49", // 0004_library.sql
        "4c2988058e5db2729a8ccd71abf4cbc750ca6805283e7e46f8f70db435e33819", // 0005_passage_sections.sql
        "d037f4a9f1afc755541a4245f439d649e4b979a3f70ab679e9a18c2ce21bcb87", // 0006_source_annotations.sql
        "2e643740eb7662b5af982e178d3d3af672ce0c93659a9ff766278219b4d1fede", // 0007_finding_source_quotes.sql
        "24358f2726358ab84c2be6edb381962d44d664686e212fb68ad3b5fb0cc19181", // 0008_teaching_preparations.sql
        "30b51c5269c4df0c041e54fdb7697ec48f99fad5723b1c3e907fcfad93cff26f", // 0009_teaching_step_kinds.sql
        "75a1d3100aed2daa2c61db1f0632c780a1b1dccdacae9b48f0cf77f4b7092c3e", // 0010_presentations.sql
        "cad7c6dbaa04546d6bb359948912511afc08679f6e196ee631162a616df0920e", // 0011_presentation_target.sql
        "4897e69f1772ecc4739d8b48b68cc67339c2fbf891d5bc7431ac503bd98ad600", // 0012_presentation_templates.sql
        "b21b8d7376bcefffd7af7c1612faae3e75b380f551d71be4a469646ed92d5752", // 0013_study_phases.sql
        "c2bb3d53a9534dc935bf92685a5ce89f1138955c108c6dd7d930c8b4a28a7f2f", // 0014_teaching_templates.sql
        "6fe4b7bf54c5e7bc24cf3dc5d162618868abe9ec65dbb381f735db503a1ebf5c", // 0015_verse_bridges.sql
        "9d8fc7cdbec5c5c7ce1b7c05e412d6396695d106d05e8e36f61cf4cf7424846a", // 0016_unresolved_citations.sql
        "0328b622162d1849e110c0e1d872ef678bb97dc41fd7b4fc68dfd317ef408f20", // 0017_word_annotations.sql
        "360fa1b64e04d6577414af653b2a75ae5e3e5e2167e37ac73341592734f2a0e2", // 0018_grammar_provenance.sql
        "9c5f902a99211c4c02e6789908639eb75542bbeab2a379092247728bcafe13d4", // 0019_bible_documents.sql
        "3c3b94a6448ffb8449a90161126cb8ea838cd5d53948ea62dabb656655dcc71c", // 0020_library_organisation.sql
        "58408192a7f7a8ee5570dff1b22c3a3227516cd1feb7bfe3f6f1d0d29d518cf1", // 0021_bible_passage_locations.sql
        "b77b53f8186ba795f8acdec482eac93250edf90e2ad140dd073fe175d3220521", // 0022_import_jobs.sql
        "beb1121b087d028b657c464ff65a351475cc307fe2b08430112ed608790c334a", // 0023_recognition_corrections.sql
    ];
    for (migration, expected) in MIGRATIONS.iter().take(23).zip(expected) {
        assert_eq!(
            format!("{:x}", Sha256::digest(migration.sql.as_bytes())),
            expected,
            "migration {} was rewritten",
            migration.version
        );
    }
}

#[test]
fn full_app_backup_restores_template_fields_answers_phases_and_files() {
    use abiding_word_lib::db::{blocks, projects, sections, studies, workspaces};
    use abiding_word_lib::domain::{
        CreateProject, CreateStudy, SectionStatus, UpdateStudyBlock, UpdateStudySection,
    };
    use abiding_word_lib::filesystem::DataDirs;
    use abiding_word_lib::portability::backup;
    let temp = tempfile::tempdir().unwrap();
    let original_dirs = DataDirs::ensure(temp.path().join("original-folder")).unwrap();
    let db = Database::open(
        &original_dirs.database_file(),
        &original_dirs.pre_migration_backups(),
    )
    .unwrap();
    let saved = db
        .with_conn(|conn| {
            let project = projects::create(
                conn,
                &CreateProject {
                    workspace_id: workspaces::get_default(conn)?.id,
                    title: "Backup fixture".into(),
                    description: None,
                },
            )?;
            let study = studies::create(
                conn,
                &CreateStudy {
                    project_id: project.id,
                    title: "Kept structured study".into(),
                    template_key: Some("pozetek-study".into()),
                    ..Default::default()
                },
            )?;
            let part = &study.sections[0];
            blocks::update(
                conn,
                &UpdateStudyBlock {
                    id: part.blocks[0].id.clone(),
                    content: Some("Original answer".into()),
                    data: None,
                },
            )?;
            sections::update(
                conn,
                &UpdateStudySection {
                    id: part.id.clone(),
                    status: Some(SectionStatus::Complete),
                    ..Default::default()
                },
            )?;
            studies::get_detail(conn, &study.study.id)
        })
        .unwrap();
    let source = original_dirs.sources().join("private-fixture.txt");
    fs::write(&source, b"invented private bytes").unwrap();
    let path = temp.path().join("full.awbackup");
    backup::create(&db, &original_dirs, &path).unwrap();
    let target_dirs = DataDirs::ensure(temp.path().join("restored-folder")).unwrap();
    let target = Database::open(
        &target_dirs.database_file(),
        &target_dirs.pre_migration_backups(),
    )
    .unwrap();
    backup::inspect(&target, &path).unwrap();
    let report = backup::restore(&target, &target_dirs, &path).unwrap();
    assert!(std::path::Path::new(&report.safety_copy).exists());
    target
        .with_conn(|conn| {
            let restored = studies::get_detail(conn, &saved.study.id)?;
            assert_eq!(restored, saved);
            assert_eq!(migrations::current_version(conn)?, 24);
            migrations::validate_existing_history(conn)?;
            Ok(())
        })
        .unwrap();
    assert_eq!(
        fs::read(target_dirs.sources().join("private-fixture.txt")).unwrap(),
        b"invented private bytes"
    );
}

#[test]
fn unrecorded_structured_columns_stop_before_any_pending_migration() {
    let mut conn = Connection::open_in_memory().unwrap();
    migrations::run_with(&mut conn, &MIGRATIONS[..19]).unwrap();
    conn.execute_batch(
        "ALTER TABLE study_sections ADD COLUMN status TEXT DEFAULT 'kept-custom-value'",
    )
    .unwrap();
    let before = history(&conn);
    let temp = tempfile::tempdir().unwrap();
    let error = migrations::upgrade(&mut conn, MIGRATIONS, Some(temp.path())).unwrap_err();
    assert!(error.to_string().contains("study_sections.status"));
    assert_eq!(history(&conn), before);
    assert_eq!(migrations::current_version(&conn).unwrap(), 19);
    assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
}
