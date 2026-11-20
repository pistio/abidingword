//! Real schema-27, disposable-database tests of daily recovery.
//! No owner data, imported book or private source is ever opened.
use std::fs;
use std::path::Path;

use abiding_word_lib::db::{backup, migrations, Database};
use abiding_word_lib::filesystem::DataDirs;
use abiding_word_lib::portability::snapshots;
use rusqlite::{params, Connection, OpenFlags};

fn new_install() -> (tempfile::TempDir, DataDirs, Database) {
    let temp = tempfile::tempdir().unwrap();
    let dirs = DataDirs::ensure(temp.path().join("Abiding Word")).unwrap();
    let db = Database::open(&dirs.database_file(), &dirs.pre_migration_backups()).unwrap();
    (temp, dirs, db)
}

fn write_note(db: &Database, value: &str) {
    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO settings (key, value, created_at, updated_at)
             VALUES ('test.durable', ?1, 'now', 'now')
             ON CONFLICT (key) DO UPDATE SET value=excluded.value, updated_at=excluded.updated_at",
            params![value],
        )?;
        Ok(())
    })
    .unwrap();
}

fn note(conn: &Connection) -> String {
    conn.query_row(
        "SELECT value FROM settings WHERE key='test.durable'",
        [],
        |row| row.get(0),
    )
    .unwrap()
}

fn snapshot(db: &Database, dirs: &DataDirs) -> String {
    let path = db
        .with_conn(|conn| backup::daily_snapshot(conn, &dirs.daily_backups()))
        .unwrap()
        .unwrap();
    path.file_name().unwrap().to_string_lossy().into_owned()
}

fn from_copy(path: &Path) -> String {
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    note(&conn)
}

#[test]
fn old_work_can_be_restored_without_touching_managed_files() {
    let (_tmp, dirs, db) = new_install();
    write_note(&db, "\"My first Bible study\"");
    let source = dirs.sources().join("my-own-source.txt");
    fs::write(&source, b"Never overwrite this private source").unwrap();
    let id = snapshot(&db, &dirs);
    write_note(&db, "\"My newer teaching\"");

    let before = snapshots::list(&db, &dirs).unwrap();
    assert_eq!(before.len(), 1);
    assert_eq!(before[0].id, id);
    assert!(before[0].usable);
    assert!(before[0].created_at.ends_with('Z'));

    let restored = snapshots::restore(&db, &dirs, &id).unwrap();
    assert!(restored.files_unchanged);
    assert_eq!(fs::read(&source).unwrap(), b"Never overwrite this private source");
    assert_eq!(db.with_conn(|conn| Ok(note(conn))).unwrap(), "\"My first Bible study\"");
    assert_eq!(
        from_copy(Path::new(&restored.safety_copy)),
        "\"My newer teaching\""
    );
    // The original snapshot remains intact and useful after restoring.
    assert_eq!(snapshots::list(&db, &dirs).unwrap().len(), 1);
    db.with_conn(|conn| {
        assert_eq!(migrations::current_version(conn)?, migrations::MIGRATIONS.len() as i64);
        Ok(())
    }).unwrap();
}

#[test]
fn corrupted_candidate_is_visible_but_disabled_and_never_changes_work() {
    let (_tmp, dirs, db) = new_install();
    write_note(&db, "\"Keep me\"");
    let id = snapshot(&db, &dirs);
    fs::write(dirs.daily_backups().join(&id), b"not a SQLite database").unwrap();
    let points = snapshots::list(&db, &dirs).unwrap();
    assert_eq!(points.len(), 1);
    assert!(!points[0].usable);
    assert!(snapshots::restore(&db, &dirs, &id).is_err());
    assert_eq!(db.with_conn(|conn| Ok(note(conn))).unwrap(), "\"Keep me\"");
    assert!(!dirs.pre_restore_backups().exists());
}

#[test]
fn foreign_paths_and_schema_versions_are_rejected_before_safety_copy() {
    let (_tmp, dirs, db) = new_install();
    write_note(&db, "\"No change\"");
    let id = snapshot(&db, &dirs);
    for invalid in [
        format!("../{id}"),
        format!("folder/{id}"),
        format!("C:\\Windows\\{id}"),
        "not-a-daily-backup.db".into(),
    ] {
        assert!(snapshots::restore(&db, &dirs, &invalid).is_err());
    }
    let future = id.replacen("abiding-word-v27-", "abiding-word-v999-", 1);
    fs::copy(dirs.daily_backups().join(&id), dirs.daily_backups().join(&future)).unwrap();
    assert!(!snapshots::list(&db, &dirs)
        .unwrap()
        .iter()
        .find(|s| s.id == future)
        .unwrap()
        .usable);
    assert!(snapshots::restore(&db, &dirs, &future).is_err());
    assert_eq!(db.with_conn(|conn| Ok(note(conn))).unwrap(), "\"No change\"");
    assert!(!dirs.pre_restore_backups().exists());
}

#[cfg(unix)]
#[test]
fn symlink_snapshot_is_not_listed_or_opened() {
    use std::os::unix::fs::symlink;

    let (_tmp, dirs, db) = new_install();
    write_note(&db, "\"Keep me\"");
    let id = snapshot(&db, &dirs);
    let link_id = id.replacen("abiding-word-v27-", "abiding-word-v26-", 1);
    symlink(dirs.daily_backups().join(&id), dirs.daily_backups().join(&link_id)).unwrap();
    assert!(!snapshots::list(&db, &dirs)
        .unwrap()
        .iter()
        .any(|s| s.id == link_id));
    assert!(snapshots::restore(&db, &dirs, &link_id).is_err());
}
