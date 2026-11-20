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
    let managed = [
        dirs.sources().join("invented.pdf"),
        dirs.sources().join("invented.epub"),
        dirs.bible_sources("invented").join("invented.usfm"),
        dirs.teaching_images().join("invented.png"),
    ];
    for (index, path) in managed.iter().enumerate() {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, format!("Invented original file {index}")).unwrap();
    }
    let id = snapshot(&db, &dirs);
    write_note(&db, "\"My newer teaching\"");
    // These files are newer than the point. Database-only recovery must leave
    // every original file exactly as it is, including imported Bibles/images.
    let before_files = managed
        .iter()
        .enumerate()
        .map(|(index, path)| {
            let bytes = format!("Invented newer file {index}").into_bytes();
            fs::write(path, &bytes).unwrap();
            bytes
        })
        .collect::<Vec<_>>();

    let before = snapshots::list(&db, &dirs).unwrap();
    assert_eq!(before.len(), 1);
    assert_eq!(before[0].id, id);
    assert!(before[0].usable);
    assert!(before[0].created_at.ends_with('Z'));

    let restored = snapshots::restore(&db, &dirs, &id).unwrap();
    assert!(restored.files_unchanged);
    for (path, bytes) in managed.iter().zip(before_files) {
        assert_eq!(fs::read(path).unwrap(), bytes);
    }
    assert_eq!(
        fs::read(&source).unwrap(),
        b"Never overwrite this private source"
    );
    assert_eq!(
        db.with_conn(|conn| Ok(note(conn))).unwrap(),
        "\"My first Bible study\""
    );
    assert_eq!(
        from_copy(Path::new(&restored.safety_copy)),
        "\"My newer teaching\""
    );
    // The original snapshot remains intact and useful after restoring.
    assert_eq!(snapshots::list(&db, &dirs).unwrap().len(), 1);
    db.with_conn(|conn| {
        assert_eq!(
            migrations::current_version(conn)?,
            migrations::MIGRATIONS.len() as i64
        );
        Ok(())
    })
    .unwrap();
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
    fs::copy(
        dirs.daily_backups().join(&id),
        dirs.daily_backups().join(&future),
    )
    .unwrap();
    assert!(
        !snapshots::list(&db, &dirs)
            .unwrap()
            .iter()
            .find(|s| s.id == future)
            .unwrap()
            .usable
    );
    assert!(snapshots::restore(&db, &dirs, &future).is_err());
    assert_eq!(
        db.with_conn(|conn| Ok(note(conn))).unwrap(),
        "\"No change\""
    );
    assert!(!dirs.pre_restore_backups().exists());
}

#[cfg(unix)]
#[test]
fn symlink_snapshot_is_not_listed_or_opened() {
    use std::os::unix::fs::symlink;

    let (_tmp, dirs, db) = new_install();
    write_note(&db, "\"Keep me\"");
    let id = snapshot(&db, &dirs);
    let link_id = "abiding-word-v27-2001-01-01T00-00-00.000Z.db".to_string();
    symlink(
        dirs.daily_backups().join(&id),
        dirs.daily_backups().join(&link_id),
    )
    .unwrap();
    assert!(!snapshots::list(&db, &dirs)
        .unwrap()
        .iter()
        .any(|s| s.id == link_id));
    assert!(snapshots::restore(&db, &dirs, &link_id).is_err());
}

#[test]
fn a_structurally_broken_snapshot_cannot_replace_the_live_database() {
    let (_tmp, dirs, db) = new_install();
    write_note(&db, "\"Older note\"");
    let id = snapshot(&db, &dirs);
    write_note(&db, "\"Newer work stays safe\"");

    // SQLite can report page integrity 'ok' while an application table is
    // missing. This tests the *post-restore* validation and rollback, not
    // merely the corrupt-file or wrong-version preflight.
    let candidate = dirs.daily_backups().join(&id);
    let copy = Connection::open(&candidate).unwrap();
    copy.execute_batch("PRAGMA foreign_keys=OFF; DROP TABLE workspaces;")
        .unwrap();
    drop(copy);

    assert!(snapshots::restore(&db, &dirs, &id).is_err());
    assert_eq!(
        db.with_conn(|conn| Ok(note(conn))).unwrap(),
        "\"Newer work stays safe\""
    );
    // If the invalid snapshot reached the restore step, a safety copy of
    // current work was made and has to remain available to the owner.
    if dirs.pre_restore_backups().exists() {
        let files: Vec<_> = fs::read_dir(dirs.pre_restore_backups())
            .unwrap()
            .flatten()
            .collect();
        assert!(!files.is_empty());
        assert!(files
            .iter()
            .any(|item| { from_copy(&item.path()) == "\"Newer work stays safe\"" }));
    }
}

#[test]
fn committed_wal_content_is_staged_and_restored_instead_of_stale_database_pages() {
    let (_tmp, dirs, db) = new_install();
    write_note(&db, "\"Before snapshot WAL edit\"");
    let id = snapshot(&db, &dirs);
    let candidate = dirs.daily_backups().join(&id);
    let writer = Connection::open(&candidate).unwrap();
    writer
        .execute_batch("PRAGMA journal_mode=WAL; PRAGMA wal_autocheckpoint=0;")
        .unwrap();
    writer
        .execute(
            "UPDATE settings SET value='\"Committed only in WAL\"' WHERE key='test.durable'",
            [],
        )
        .unwrap();
    assert!(
        fs::metadata(candidate.with_extension("db-wal"))
            .unwrap()
            .len()
            > 0
    );
    write_note(&db, "\"Current live writing\"");

    let restored = snapshots::restore(&db, &dirs, &id).unwrap();
    assert_eq!(
        db.with_conn(|conn| Ok(note(conn))).unwrap(),
        "\"Committed only in WAL\""
    );
    assert_eq!(
        from_copy(Path::new(&restored.safety_copy)),
        "\"Current live writing\""
    );
    assert_eq!(note(&writer), "\"Committed only in WAL\"");
    assert!(fs::read_dir(dirs.cache()).unwrap().next().is_none());
    drop(writer);
    drop(db);
    let reopened = Database::open(&dirs.database_file(), &dirs.pre_migration_backups()).unwrap();
    assert_eq!(
        reopened.with_conn(|conn| Ok(note(conn))).unwrap(),
        "\"Committed only in WAL\""
    );
}

#[test]
fn missing_writing_tables_and_forged_history_are_unavailable_before_replacement() {
    for damage in [
        "PRAGMA foreign_keys=OFF; DROP TABLE study_blocks;",
        "UPDATE schema_versions SET name='invented_wrong_lineage' WHERE version=27;",
        "ALTER TABLE study_blocks RENAME COLUMN content TO unusable_content;",
        "CREATE TRIGGER destroy_writing AFTER UPDATE ON study_sections BEGIN DELETE FROM study_sections; END;",
        "CREATE TRIGGER sqliteXerase AFTER UPDATE ON study_sections BEGIN DELETE FROM study_sections; END;",
        "DROP INDEX passage_references_one_primary;",
        "PRAGMA writable_schema=ON; UPDATE sqlite_schema SET sql=replace(sql, 'CHECK (from_study_id <> to_study_id)', 'CHECK (1)') WHERE name='study_relations'; PRAGMA writable_schema=OFF;",

    ] {
        let (_tmp, dirs, db) = new_install();
        write_note(&db, "\"Original\"");
        let id = snapshot(&db, &dirs);
        write_note(&db, "\"Current writing must survive\"");
        let copy = Connection::open(dirs.daily_backups().join(&id)).unwrap();
        copy.execute_batch(damage).unwrap();
        drop(copy);

        let points = snapshots::list(&db, &dirs).unwrap();
        assert_eq!(points.len(), 1);
        assert!(!points[0].usable, "{damage}");
        assert!(snapshots::restore(&db, &dirs, &id).is_err(), "{damage}");
        assert!(!dirs.pre_restore_backups().exists());
        assert_eq!(
            db.with_conn(|conn| Ok(note(conn))).unwrap(),
            "\"Current writing must survive\""
        );
    }
}

#[test]
fn current_database_history_is_verified_before_any_safety_copy_or_replacement() {
    let (_tmp, dirs, db) = new_install();
    write_note(&db, "\"Older snapshot\"");
    let id = snapshot(&db, &dirs);
    write_note(&db, "\"Preserve current writing\"");
    db.with_conn(|conn| {
        conn.execute(
            "UPDATE schema_versions SET name='unrecognised_history' WHERE version=27",
            [],
        )?;
        Ok(())
    })
    .unwrap();
    assert!(snapshots::restore(&db, &dirs, &id).is_err());
    assert!(!dirs.pre_restore_backups().exists());
    assert_eq!(
        db.with_conn(|conn| Ok(note(conn))).unwrap(),
        "\"Preserve current writing\""
    );
}

#[test]
fn a_failed_safety_copy_never_replaces_the_current_database() {
    let (_tmp, dirs, db) = new_install();
    write_note(&db, "\"Snapshot\"");
    let id = snapshot(&db, &dirs);
    write_note(&db, "\"Keep current work\"");
    fs::write(dirs.pre_restore_backups(), b"invented path blocker").unwrap();
    assert!(snapshots::restore(&db, &dirs, &id).is_err());
    assert_eq!(
        db.with_conn(|conn| Ok(note(conn))).unwrap(),
        "\"Keep current work\""
    );
    assert_eq!(
        fs::read(dirs.pre_restore_backups()).unwrap(),
        b"invented path blocker"
    );
    assert!(
        fs::read_dir(dirs.cache()).unwrap().next().is_none(),
        "staging files must be cleaned"
    );
}

#[cfg(unix)]
#[test]
fn redirected_daily_cache_and_safety_directories_are_rejected_without_external_writes() {
    use std::os::unix::fs::symlink;
    for location in ["daily", "backups", "cache", "pre-restore"] {
        let (temp, dirs, db) = new_install();
        write_note(&db, "\"Original snapshot\"");
        let id = snapshot(&db, &dirs);
        write_note(&db, "\"Current writing stays\"");
        let inside = match location {
            "daily" => dirs.daily_backups(),
            "backups" => dirs.root().join("backups"),
            "cache" => dirs.cache(),
            _ => dirs.pre_restore_backups(),
        };
        let outside = temp.path().join(format!("outside-{location}"));
        if inside.exists() {
            fs::rename(&inside, &outside).unwrap();
        } else {
            fs::create_dir(&outside).unwrap();
        }
        symlink(&outside, &inside).unwrap();
        let before = fs::read_dir(&outside).unwrap().count();
        if matches!(location, "daily" | "backups") {
            assert!(snapshots::list(&db, &dirs).is_err());
        }
        assert!(snapshots::restore(&db, &dirs, &id).is_err(), "{location}");
        assert_eq!(
            fs::read_dir(&outside).unwrap().count(),
            before,
            "{location}"
        );
        assert_eq!(
            db.with_conn(|conn| Ok(note(conn))).unwrap(),
            "\"Current writing stays\""
        );
    }
}
