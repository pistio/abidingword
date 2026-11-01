//! Additive PDF metadata must not touch existing source text or relationships.
use abiding_word_lib::db::{
    migrations::{self, MIGRATIONS},
    Database,
};
use rusqlite::{types::Value, Connection};
fn snapshot(c: &Connection) -> Vec<Vec<Vec<Value>>> {
    ["sources", "source_files", "source_segments", "settings"]
        .iter()
        .map(|table| {
            let mut s = c
                .prepare(&format!("SELECT * FROM {table} ORDER BY 1"))
                .unwrap();
            let n = s.column_count();
            s.query_map([], |r| {
                (0..n).map(|i| r.get(i)).collect::<Result<Vec<Value>, _>>()
            })
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
        })
        .collect()
}
#[test]
fn populated_v26_upgrades_additively_with_a_pre_migration_backup_and_immutable_history() {
    let t = tempfile::tempdir().unwrap();
    let path = t.path().join("old.db");
    let backups = t.path().join("backups");
    let mut c = Connection::open(&path).unwrap();
    c.pragma_update(None, "foreign_keys", true).unwrap();
    migrations::run_with(&mut c, &MIGRATIONS[..26]).unwrap();
    c.execute_batch("INSERT INTO workspaces(id,name,is_default,created_at,updated_at) VALUES('w','Invented',1,'T','T');
        INSERT INTO sources(id,workspace_id,source_type,title,passage_index_version,created_at,updated_at) VALUES('src','w','book','Invented PDF',4,'T','T');
        INSERT INTO source_files(id,source_id,relative_path,sha256,is_primary,format,created_at,updated_at) VALUES('file','src','book/src/test.pdf','hash',1,'pdf','T','T');
        INSERT INTO source_segments(id,source_id,source_file_id,position,content,page_number,page_label,char_start,char_end,locator,created_at,updated_at) VALUES('seg','src','file',0,'Existing words — 한글',7,'iv',0,25,'PDF page 7','T','T');
        INSERT INTO source_passages(id,source_id,segment_id,passage,start_key,end_key,matched_text,origin,created_at) VALUES('link','src','seg','ACT.6',44006000,44006999,'Acts 6','manual','T');").unwrap();
    let before = snapshot(&c);
    let history: Vec<(i64, String, String)> = c
        .prepare("SELECT version,name,applied_at FROM schema_versions ORDER BY version")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    drop(c);
    let db = Database::open(&path, &backups).unwrap();
    db.with_conn(|c| {
        // Startup seeds settings/defaults; migrations themselves are verified
        // separately below. Existing source-bearing tables remain exact.
        let after=snapshot(c);assert_eq!(&after[..3],&before[..3]);
        let actual:Vec<(i64,String,String)>=c.prepare("SELECT version,name,applied_at FROM schema_versions WHERE version<=26 ORDER BY version")?.query_map([],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?.collect::<Result<_,_>>()?;
        assert_eq!(actual,history);assert_eq!(migrations::current_version(c)?,27);
        assert_eq!(c.query_row("SELECT count(*) FROM source_pdf_structures",[],|r|r.get::<_,i64>(0))?,0);
        assert_eq!(c.query_row("SELECT origin FROM source_passages WHERE id='link'",[],|r|r.get::<_,String>(0))?,"manual");
        assert!(!c.prepare("PRAGMA foreign_key_check")?.exists([])?);Ok(())
    }).unwrap();
    assert_eq!(std::fs::read_dir(backups).unwrap().count(), 1);
}
#[test]
fn migration_failure_rolls_back_every_added_column_and_existing_rows() {
    let mut c = Connection::open_in_memory().unwrap();
    migrations::run_with(&mut c, &MIGRATIONS[..26]).unwrap();
    let before = snapshot(&c);
    let sql: &'static str = Box::leak(
        format!(
            "{}\nINSERT INTO no_such_table VALUES(1);",
            MIGRATIONS[26].sql
        )
        .into_boxed_str(),
    );
    let mut list: Vec<_> = MIGRATIONS[..26]
        .iter()
        .map(|m| migrations::Migration {
            version: m.version,
            name: m.name,
            sql: m.sql,
        })
        .collect();
    list.push(migrations::Migration {
        version: 27,
        name: "pdf_structure",
        sql,
    });
    assert!(migrations::run_with(&mut c, &list).is_err());
    assert_eq!(snapshot(&c), before);
    assert_eq!(migrations::current_version(&c).unwrap(), 26);
    assert!(c
        .prepare("SELECT structure_node_id FROM source_passages")
        .is_err());
}
