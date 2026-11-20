//! Fault-injected complete .awbackup restores on disposable, invented data.
//! The live owner's Windows profile is never opened.
use std::fs;
use std::path::{Path, PathBuf};

use abiding_word_lib::db::Database;
use abiding_word_lib::filesystem::DataDirs;
use abiding_word_lib::portability::backup;
use rusqlite::{params, Connection, OpenFlags};

fn install() -> (tempfile::TempDir, DataDirs, Database) {
    let temp = tempfile::tempdir().unwrap();
    let dirs = DataDirs::ensure(temp.path().join("Abiding Word")).unwrap();
    let db = Database::open(&dirs.database_file(), &dirs.pre_migration_backups()).unwrap();
    (temp, dirs, db)
}

fn update_note(db: &Database, text: &str) {
    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO settings (key, value, created_at, updated_at)
             VALUES ('fixture.full.restore', ?1, 'now', 'now')
             ON CONFLICT (key) DO UPDATE SET value=excluded.value, updated_at=excluded.updated_at",
            params![text],
        )?;
        Ok(())
    })
    .unwrap();
}

fn note(db: &Database) -> String {
    db.with_conn(|conn| {
        Ok(conn.query_row(
            "SELECT value FROM settings WHERE key='fixture.full.restore'",
            [],
            |row| row.get(0),
        )?)
    })
    .unwrap()
}

fn safety_value(path: &Path) -> String {
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    conn.query_row(
        "SELECT value FROM settings WHERE key='fixture.full.restore'",
        [],
        |r| r.get(0),
    )
    .unwrap()
}

fn with_backup() -> (
    tempfile::TempDir,
    DataDirs,
    Database,
    tempfile::TempDir,
    PathBuf,
) {
    let (original, dirs, db) = install();
    update_note(&db, "\"From the saved backup\"");
    fs::write(
        dirs.sources().join("00-first.txt"),
        b"Archive edition of first source",
    )
    .unwrap();
    fs::write(
        dirs.sources().join("01-second.txt"),
        b"Archive edition of second source",
    )
    .unwrap();
    let out = tempfile::tempdir().unwrap();
    let path = out.path().join("my-private-backup.awbackup");
    backup::create(&db, &dirs, &path).unwrap();
    (original, dirs, db, out, path)
}

#[test]
fn failure_after_replacing_first_file_restores_its_old_bytes_and_keeps_live_database() {
    let (_old, _dirs, _db, _output, archive) = with_backup();
    let (_tmp, dirs, db) = install();
    update_note(&db, "\"My newer draft\"");
    let first = dirs.sources().join("00-first.txt");
    let second = dirs.sources().join("01-second.txt");
    fs::write(&first, b"Do not lose my newer source").unwrap();
    fs::create_dir(&second).unwrap(); // Deterministic failure at the SECOND file.
    assert!(backup::restore(&db, &dirs, &archive).is_err());

    assert_eq!(note(&db), "\"My newer draft\"");
    assert_eq!(fs::read(&first).unwrap(), b"Do not lose my newer source");
    assert!(second.is_dir());
    let copies: Vec<_> = fs::read_dir(dirs.pre_restore_backups())
        .unwrap()
        .flatten()
        .filter(|e| e.path().is_file())
        .collect();
    assert_eq!(copies.len(), 1);
    assert_eq!(safety_value(&copies[0].path()), "\"My newer draft\"");
}

#[test]
fn successful_restore_recovers_files_and_db_but_keeps_original_source_aside() {
    let (_old, _dirs, _db, _output, archive) = with_backup();
    let (_tmp, dirs, db) = install();
    update_note(&db, "\"Current unsaved variant\"");
    let first = dirs.sources().join("00-first.txt");
    fs::write(&first, b"Previously imported first source").unwrap();
    let report = backup::restore(&db, &dirs, &archive).unwrap();

    assert_eq!(note(&db), "\"From the saved backup\"");
    assert_eq!(
        fs::read(&first).unwrap(),
        b"Archive edition of first source"
    );
    assert_eq!(
        fs::read(dirs.sources().join("01-second.txt")).unwrap(),
        b"Archive edition of second source",
    );
    assert_eq!(report.files_set_aside, 1);
    assert_eq!(report.files_added, 2);
    assert_eq!(
        safety_value(Path::new(&report.safety_copy)),
        "\"Current unsaved variant\""
    );

    let preserved: Vec<_> = fs::read_dir(dirs.pre_restore_backups())
        .unwrap()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("files-"))
        .map(|e| e.path().join("sources/00-first.txt"))
        .collect();
    assert_eq!(preserved.len(), 1);
    assert_eq!(
        fs::read(&preserved[0]).unwrap(),
        b"Previously imported first source"
    );
}

#[cfg(unix)]
#[test]
fn local_symlink_to_an_external_file_is_never_followed_or_replaced() {
    use std::os::unix::fs::symlink;

    let (_old, _dirs, _db, _output, archive) = with_backup();
    let (_tmp, dirs, db) = install();
    update_note(&db, "\"Still mine\"");
    let outside = dirs
        .root()
        .parent()
        .unwrap()
        .join("external-personal-file.txt");
    fs::write(&outside, b"Outside file is private").unwrap();
    symlink(&outside, dirs.sources().join("00-first.txt")).unwrap();

    assert!(backup::restore(&db, &dirs, &archive).is_err());
    assert_eq!(note(&db), "\"Still mine\"");
    assert_eq!(fs::read(&outside).unwrap(), b"Outside file is private");
    assert!(fs::symlink_metadata(dirs.sources().join("00-first.txt"))
        .unwrap()
        .file_type()
        .is_symlink());
}
