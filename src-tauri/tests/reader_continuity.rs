//! Reading continuity: real schema-27 SQLite and genuine backup round trip.
//! Only invented source identities and places are used.
use abiding_word_lib::db::{reader_positions as saved, workspaces, Database};
use abiding_word_lib::error::AppResult;
use abiding_word_lib::filesystem::DataDirs;
use abiding_word_lib::portability::backup;
use rusqlite::{params, Connection};

fn source(conn: &Connection, name: &str) -> AppResult<()> {
    let workspace = workspaces::get_default(conn)?;
    conn.execute(
        "INSERT INTO sources (id, workspace_id, source_type, title, created_at, updated_at)
         VALUES (?1, ?2, 'book', ?3, '2026-11-20T00:00:00Z', '2026-11-20T00:00:00Z')",
        params![name, workspace.id, name],
    )?;
    Ok(())
}

fn place(page: u32) -> saved::ReaderPosition {
    saved::ReaderPosition {
        mode: "original".into(),
        page: Some(page),
        place: Some(saved::ReaderLocation {
            page: Some(page),
            heading_id: Some("chapter-3".into()),
            fraction: 0.25,
            position: Some(32),
            offset: -70.0,
            item: None,
            fragment: None,
            item_offset: 0.0,
        }),
        history: Some(saved::ReaderHistory {
            entries: vec![],
            index: -1,
        }),
    }
}

#[test]
fn places_are_kept_per_source_and_updated_in_recent_order() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        source(conn, "one")?;
        source(conn, "two")?;
        saved::save(conn, "one", &place(3))?;
        saved::save(conn, "two", &place(7))?;
        saved::save(conn, "one", &place(8))?;
        let rows = saved::list(conn)?;
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].source_id, "one");
        assert_eq!(rows[0].position.page, Some(8));
        assert_eq!(rows[1].source_id, "two");
        assert_eq!(rows[1].position.page, Some(7));
        Ok(())
    })
    .unwrap();
}

#[test]
fn invalid_places_never_replace_a_last_good_place() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        source(conn, "one")?;
        saved::save(conn, "one", &place(5))?;
        let mut wrong = place(100);
        wrong.mode = "invented-mode".into();
        assert!(saved::save(conn, "one", &wrong).is_err());
        let mut wrong = place(100);
        wrong.place.as_mut().unwrap().fraction = f64::NAN;
        assert!(saved::save(conn, "one", &wrong).is_err());
        let mut wrong = place(100);
        wrong.history = Some(saved::ReaderHistory {
            entries: vec![],
            index: 0,
        });
        assert!(saved::save(conn, "one", &wrong).is_err());
        let mut wrong = place(100);
        wrong.history = Some(saved::ReaderHistory {
            entries: vec![wrong.place.clone().unwrap(); 201],
            index: 200,
        });
        assert!(saved::save(conn, "one", &wrong).is_err());
        assert!(saved::save(conn, "missing", &place(1)).is_err());
        assert_eq!(saved::list(conn)?[0].position.page, Some(5));
        Ok(())
    })
    .unwrap();
}

#[test]
fn trashed_sources_do_not_reappear_and_permanent_removal_forgets_place() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        source(conn, "one")?;
        saved::save(conn, "one", &place(4))?;
        conn.execute("UPDATE sources SET state='trashed' WHERE id='one'", [])?;
        assert!(saved::list(conn)?.is_empty());
        assert!(saved::save(conn, "one", &place(6)).is_err());
        conn.execute("UPDATE sources SET state='active' WHERE id='one'", [])?;
        assert_eq!(saved::list(conn)?[0].position.page, Some(4));
        saved::forget(conn, "one")?;
        assert!(saved::list(conn)?.is_empty());
        Ok(())
    })
    .unwrap();
}

#[test]
fn ordinary_backup_restore_preserves_reading_history_and_place() {
    let temp = tempfile::tempdir().unwrap();
    let dirs = DataDirs::ensure(temp.path().join("app")).unwrap();
    let db = Database::open(&dirs.database_file(), &dirs.pre_migration_backups()).unwrap();
    db.with_conn(|conn| {
        source(conn, "one")?;
        let mut position = place(6);
        position.history = Some(saved::ReaderHistory {
            entries: vec![position.place.clone().unwrap()],
            index: 0,
        });
        saved::save(conn, "one", &position)
    })
    .unwrap();
    let output = temp.path().join("safe.awbackup");
    backup::create(&db, &dirs, &output).unwrap();
    db.with_conn(|conn| saved::save(conn, "one", &place(10)))
        .unwrap();
    backup::restore(&db, &dirs, &output).unwrap();
    db.with_conn(|conn| {
        let records = saved::list(conn)?;
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].position.page, Some(6));
        assert_eq!(
            records[0].position.history.as_ref().unwrap().entries.len(),
            1
        );
        Ok(())
    })
    .unwrap();
}
