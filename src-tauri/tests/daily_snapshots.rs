//! Database-only automatic safety snapshots: use disposable local databases.
//! They are separate from full .awbackup archives with sources.
use std::fs;
use std::path::Path;

use abiding_word_lib::db::{backup, Database};
use abiding_word_lib::filesystem::DataDirs;
use rusqlite::{Connection, OpenFlags};

fn seeded() -> (tempfile::TempDir, DataDirs, Database) {
    let temp = tempfile::tempdir().unwrap();
    let dirs = DataDirs::ensure(temp.path().join("my-data")).unwrap();
    let db = Database::open(&dirs.database_file(), &dirs.pre_migration_backups()).unwrap();
    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO settings (key,value,created_at,updated_at)
             VALUES ('test.sentence','\"My invented written notes\"','now','now')",
            [],
        )?;
        Ok(())
    }).unwrap();
    (temp, dirs, db)
}

fn get_note(path: &Path) -> String {
    let c = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    c.query_row("SELECT value FROM settings WHERE key='test.sentence'", [], |r| r.get(0)).unwrap()
}

#[test]
fn daily_snapshot_is_verified_and_one_per_day() {
    let (_temp, dirs, db) = seeded();
    let daily = dirs.daily_backups();
    let first = db.with_conn(|conn| backup::daily_snapshot(conn, &daily)).unwrap().unwrap();
    assert!(first.is_file());
    assert_eq!(get_note(&first), "\"My invented written notes\"");
    // Making a second one the same day would waste disk and race with saves.
    let again = db.with_conn(|conn| backup::daily_snapshot(conn, &daily)).unwrap();
    assert!(again.is_none());
    assert!(first.exists());
}

#[test]
fn damaged_same_day_snapshot_is_not_trusted_or_overwritten() {
    let (_temp, dirs, db) = seeded();
    let daily = dirs.daily_backups();
    let first = db.with_conn(|conn| backup::daily_snapshot(conn, &daily)).unwrap().unwrap();
    fs::write(&first, b"not a database").unwrap();
    // The damaged copy stays available for investigation; a new one is verified.
    let replacement = db.with_conn(|conn| backup::daily_snapshot(conn, &daily)).unwrap().unwrap();
    assert_ne!(first, replacement);
    assert_eq!(fs::read(&first).unwrap(), b"not a database");
    assert_eq!(get_note(&replacement), "\"My invented written notes\"");
}

#[test]
fn retention_deletes_only_old_verified_managed_snapshots() {
    let (_temp, dirs, db) = seeded();
    let daily = dirs.daily_backups();
    let current = db.with_conn(|conn| backup::daily_snapshot(conn, &daily)).unwrap().unwrap();
    let version = db.with_conn(abiding_word_lib::db::migrations::current_version).unwrap();
    for day in 1..=10 {
        let date = format!("2026-01-{day:02}");
        fs::copy(&current, daily.join(format!("abiding-word-v{version}-{date}T00-00-00.000Z.db"))).unwrap();
    }
    // An invalid, similarly named file and an unrelated personal file are never pruned.
    let damaged = daily.join(format!("abiding-word-v{version}-2026-01-01T05-00-00Z.db"));
    fs::write(&damaged, b"bad backup").unwrap();
    let unrelated = daily.join("my-important-notes.txt");
    fs::write(&unrelated, b"retain").unwrap();
    db.with_conn(|conn| backup::daily_snapshot(conn, &daily)).unwrap();
    let valid = fs::read_dir(&daily).unwrap().flatten().filter(|e| {
        let name=e.file_name().to_string_lossy().into_owned();
        name.starts_with("abiding-word-v") && name.ends_with(".db")
          && Connection::open(e.path()).ok().and_then(|c|
                c.query_row("PRAGMA integrity_check", [], |r| r.get::<_,String>(0)).ok()).as_deref()==Some("ok")
    }).count();
    assert_eq!(valid, 7);
    assert_eq!(fs::read(&damaged).unwrap(), b"bad backup");
    assert_eq!(fs::read(&unrelated).unwrap(), b"retain");
}
