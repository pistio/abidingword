//! Populated disposable databases only. No user data; every string is invented.
use abiding_word_lib::db::migrations::{self, Migration, MIGRATIONS};
use rusqlite::{types::Value, Connection};
const TABLES: &[&str] = &[
    "teaching_preparations",
    "teaching_points",
    "teaching_blocks",
    "teaching_passages",
    "teaching_finding_links",
    "teaching_materials",
    "presentations",
    "presentation_items",
];
fn populated() -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();
    conn.pragma_update(None, "foreign_keys", true).unwrap();
    migrations::run_with(&mut conn, &MIGRATIONS[..25]).unwrap();
    conn.execute_batch(include_str!("fixtures/teaching-stage3.sql"))
        .unwrap();
    conn
}
fn snapshot(conn: &Connection, upgraded: bool) -> Vec<Vec<Vec<Value>>> {
    TABLES
        .iter()
        .map(|table| {
            let mut stmt = conn
                .prepare(&format!("SELECT * FROM {table} ORDER BY id"))
                .unwrap();
            let len = stmt.column_count();
            stmt.query_map([], |row| {
                let mut values = (0..len)
                    .map(|i| row.get(i))
                    .collect::<Result<Vec<Value>, _>>()?;
                if upgraded && *table == "teaching_preparations" {
                    values.remove(1);
                }
                if upgraded && *table == "teaching_finding_links" {
                    values.pop();
                }
                Ok(values)
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
        })
        .collect()
}
fn assert_fk(conn: &Connection) {
    assert!(conn
        .pragma_query_value(None, "foreign_keys", |r| r.get::<_, bool>(0))
        .unwrap());
    assert!(!conn
        .prepare("PRAGMA foreign_key_check")
        .unwrap()
        .exists([])
        .unwrap());
}
#[test]
fn populated_parent_rebuild_preserves_all_columns_children_tombstones_and_ids() {
    let mut conn = populated();
    let before = snapshot(&conn, false);
    migrations::run(&mut conn).unwrap();
    assert_eq!(snapshot(&conn, true), before);
    assert_fk(&conn);
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM teaching_preparations WHERE project_id='p'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        2
    );
    migrations::run(&mut conn).unwrap();
    assert_eq!(snapshot(&conn, true), before);
    conn.execute("DELETE FROM studies WHERE id='s'", [])
        .unwrap();
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM teaching_preparations WHERE study_id IS NULL",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        2
    );
    assert_fk(&conn);
    conn.execute("DELETE FROM projects WHERE id='p'", [])
        .unwrap();
    for table in TABLES {
        assert_eq!(
            conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0,
            "{table}"
        );
    }
    assert_fk(&conn);
}
#[test]
fn rebuild_failure_rolls_back_schema_data_history_and_restores_foreign_keys() {
    for injection in ["INSERT INTO missing_table VALUES(1);", "DELETE FROM teaching_points;", "INSERT INTO study_relations(id,from_study_id,to_study_id,relation_type,created_at,updated_at) VALUES('bad','absent','s','related','T','T');"] {
        let mut conn=populated();let before=snapshot(&conn,false);
        let sql: &'static str=Box::leak(format!("{}\n{injection}",MIGRATIONS[25].sql).into_boxed_str());
        let mut list:Vec<Migration>=MIGRATIONS[..25].iter().map(|m|Migration{version:m.version,name:m.name,sql:m.sql}).collect();
        list.push(Migration{version:26,name:"project_teaching",sql});
        assert!(migrations::run_with(&mut conn,&list).is_err());
        assert_eq!(snapshot(&conn,false),before);assert_eq!(migrations::current_version(&conn).unwrap(),25);assert_fk(&conn);
        assert!(!conn.prepare("SELECT project_id FROM teaching_preparations").is_ok());
    }
}
#[test]
fn unresolved_ownership_is_rejected_instead_of_guessed() {
    let mut conn = populated();
    conn.pragma_update(None, "foreign_keys", false).unwrap();
    conn.execute("UPDATE teaching_preparations SET study_id='missing'", [])
        .unwrap();
    conn.pragma_update(None, "foreign_keys", true).unwrap();
    let before = snapshot(&conn, false);
    assert!(migrations::run(&mut conn).is_err());
    assert_eq!(snapshot(&conn, false), before);
    assert_eq!(migrations::current_version(&conn).unwrap(), 25);
    assert!(conn
        .pragma_query_value(None, "foreign_keys", |r| r.get::<_, bool>(0))
        .unwrap());
}
