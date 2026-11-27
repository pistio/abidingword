//! Real child-process termination, disposable profiles, no user data.

use super::*;
use crate::db::Database;
use crate::portability::backup;
use crate::state::AppState;
use std::process::{Command, Output};

fn open(dirs: &DataDirs) -> Database {
    Database::open(&dirs.database_file(), &dirs.pre_migration_backups()).unwrap()
}

fn set_marker(db: &Database, value: &str) {
    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO settings (key,value,created_at,updated_at) VALUES ('restore-fixture',?1,'test','test')",
            [value],
        )?;
        Ok(())
    }).unwrap();
}

fn marker(db: &Database) -> String {
    db.with_conn(|conn| {
        Ok(conn.query_row(
            "SELECT value FROM settings WHERE key='restore-fixture'",
            [],
            |r| r.get(0),
        )?)
    })
    .unwrap()
}

fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    let donor = DataDirs::ensure(root.path().join("donor")).unwrap();
    let source = open(&donor);
    set_marker(&source, "archived writing");
    for (name, data) in [
        ("first.txt", "archived first"),
        ("second.txt", "archived second"),
        ("third.txt", "archived third"),
        ("unchanged.txt", "unchanged bytes"),
    ] {
        fs::write(donor.sources().join(name), data).unwrap();
    }
    backup::create(&source, &donor, &root.path().join("saved.awbackup")).unwrap();
    let live = DataDirs::ensure(root.path().join("live")).unwrap();
    let db = open(&live);
    set_marker(&db, "current author writing");
    fs::write(live.sources().join("first.txt"), "current first").unwrap();
    fs::write(live.sources().join("third.txt"), "current third").unwrap();
    fs::write(live.sources().join("unchanged.txt"), "unchanged bytes").unwrap();
    fs::write(live.sources().join("unrelated.txt"), "unrelated bytes").unwrap();
    root
}

fn child_process(root: &Path, mode: &str, injection: &str) -> Output {
    Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "portability::restore_journal::tests::restore_child",
            "--ignored",
            "--nocapture",
        ])
        .env("AW_TEST_FULL_RESTORE_ROOT", root)
        .env("AW_TEST_FULL_RESTORE_MODE", mode)
        .env("AW_TEST_FULL_RESTORE_FAULT", injection)
        .output()
        .unwrap()
}

fn expect_exit(result: Output, code: i32, point: &str) {
    assert_eq!(
        result.status.code(),
        Some(code),
        "{point}\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}

fn start(root: &Path) -> AppState {
    AppState::start(DataDirs::ensure(root.join("live")), None, None)
}

fn check_old(root: &Path) {
    let state = start(root);
    assert!(
        state.startup_error().is_none(),
        "{:?}",
        state.startup_error()
    );
    let core = state.core().unwrap();
    assert_eq!(marker(&core.db), "current author writing");
    assert_eq!(
        fs::read(core.dirs.sources().join("first.txt")).unwrap(),
        b"current first"
    );
    assert_eq!(
        fs::read(core.dirs.sources().join("third.txt")).unwrap(),
        b"current third"
    );
    assert!(!core.dirs.sources().join("second.txt").exists());
    assert_eq!(
        fs::read(core.dirs.sources().join("unchanged.txt")).unwrap(),
        b"unchanged bytes"
    );
    assert_eq!(
        fs::read(core.dirs.sources().join("unrelated.txt")).unwrap(),
        b"unrelated bytes"
    );
    core.db
        .with_conn(|conn| {
            assert_eq!(
                conn.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))?,
                "ok"
            );
            assert!(!conn.prepare("PRAGMA foreign_key_check")?.exists([])?);
            // Recovery also restores a writable profile, not merely readable text.
            conn.execute(
                "UPDATE settings SET updated_at='after recovery' WHERE key='restore-fixture'",
                [],
            )?;
            Ok(())
        })
        .unwrap();
    assert!(!pending(&core.dirs));
}

#[test]
#[ignore = "only launched by the disposable full-restore crash tests"]
fn restore_child() {
    let Some(root) = std::env::var_os("AW_TEST_FULL_RESTORE_ROOT") else {
        return;
    };
    let root = PathBuf::from(root);
    let dirs = DataDirs::ensure(root.join("live")).unwrap();
    if std::env::var("AW_TEST_FULL_RESTORE_MODE").unwrap() == "recover" {
        recover_before_open(&dirs).unwrap();
        return;
    }
    let db = open(&dirs);
    match backup::restore(&db, &dirs, &root.join("saved.awbackup")) {
        Ok(_) => fs::write(root.join("result"), "success").unwrap(),
        Err(AppError::Import(message)) => {
            let blocked = db.with_conn(|_| Ok(())).is_err();
            fs::write(
                root.join("result"),
                format!("{};blocked={blocked}", message.code.unwrap_or("unknown")),
            )
            .unwrap();
        }
        Err(error) => panic!("unexpected error: {error}"),
    }
}

#[test]
fn interrupted_full_restore_recovers_at_each_durable_boundary() {
    for point in [
        "journal-published",
        "file-0-old-moved",
        "file-0-installed",
        "file-1-installed",
        "file-2-old-moved",
        "files-installed",
        "database-restored",
        "database-reinitialised",
        "database-flushed",
    ] {
        let root = fixture();
        expect_exit(
            child_process(root.path(), "restore", &format!("crash:{point}")),
            86,
            point,
        );
        check_old(root.path());
        // Reopening must not repeat a completed rollback over newer writing.
        check_old(root.path());
    }
}

#[test]
fn durable_commit_survives_crash_before_archive_cleanup() {
    let root = fixture();
    expect_exit(
        child_process(root.path(), "restore", "crash:committed"),
        86,
        "committed",
    );
    let state = start(root.path());
    let core = state.core().unwrap();
    assert_eq!(marker(&core.db), "archived writing");
    for (name, expected) in [
        ("first.txt", "archived first"),
        ("second.txt", "archived second"),
        ("third.txt", "archived third"),
        ("unchanged.txt", "unchanged bytes"),
        ("unrelated.txt", "unrelated bytes"),
    ] {
        assert_eq!(
            fs::read(core.dirs.sources().join(name)).unwrap(),
            expected.as_bytes()
        );
    }
    assert!(!pending(&core.dirs));
    let archive = fs::read_dir(core.dirs.pre_restore_backups())
        .unwrap()
        .filter_map(Result::ok)
        .find(|e| e.file_name().to_string_lossy().starts_with("files-"))
        .unwrap()
        .path();
    assert_eq!(
        fs::read(archive.join("sources/first.txt")).unwrap(),
        b"current first"
    );
}

#[test]
fn file_and_database_errors_roll_back_live_connection_without_stale_writes() {
    for point in [
        "copy-chunk",
        "file-0-before-install",
        "file-0-old-moved",
        "file-0-installed",
        "file-1-installed",
        "database-restored",
        "database-reinitialised",
        "database-flushed",
    ] {
        let root = fixture();
        expect_exit(
            child_process(root.path(), "restore", &format!("fail:{point}")),
            0,
            point,
        );
        assert_eq!(
            fs::read_to_string(root.path().join("result")).unwrap(),
            "portability.restoreFailed;blocked=false",
            "{point}"
        );
        check_old(root.path());
    }
}

#[test]
fn a_recovery_error_blocks_live_database_until_restart() {
    let root = fixture();
    // Intent is published but a failed journal transition cannot safely be
    // declared absent. The existing connection must refuse later commands.
    expect_exit(
        child_process(root.path(), "restore", "fail:journal-published"),
        0,
        "journal failure",
    );
    assert_eq!(
        fs::read_to_string(root.path().join("result")).unwrap(),
        "portability.recoveryRequired;blocked=true"
    );
    check_old(root.path());
}

#[test]
fn interrupted_rollback_is_idempotent_including_sqlite_sidecars() {
    for point in [
        "undo-2-replacement-moved",
        "undo-2-original-restored",
        "files-rolled-back",
        "database-0-moved",
        "database-rolled-back",
        "rolled-back",
    ] {
        let root = fixture();
        expect_exit(
            child_process(root.path(), "restore", "crash:database-restored"),
            86,
            "restored",
        );
        expect_exit(
            child_process(root.path(), "recover", &format!("crash:{point}")),
            86,
            point,
        );
        check_old(root.path());
        check_old(root.path());
    }
}

#[test]
fn recovery_precedes_opening_even_a_damaged_active_sqlite_file() {
    let root = fixture();
    expect_exit(
        child_process(root.path(), "restore", "crash:files-installed"),
        86,
        "files installed",
    );
    fs::write(
        root.path().join("live/database/abiding-word.db"),
        b"invented corrupt SQLite bytes",
    )
    .unwrap();
    check_old(root.path());
}

#[test]
fn an_interrupted_partial_undo_copy_can_be_retried_from_the_intact_original() {
    let root = fixture();
    let original = vec![b'Q'; 192 * 1024];
    fs::write(root.path().join("live/sources/third.txt"), &original).unwrap();
    expect_exit(
        child_process(root.path(), "restore", "crash:database-restored"),
        86,
        "database restored",
    );
    expect_exit(
        child_process(root.path(), "recover", "crash:undo-copy-chunk"),
        86,
        "partial rollback copy",
    );
    let dirs = DataDirs::ensure(root.path().join("live")).unwrap();
    let journal = dirs.pre_restore_backups().join(PENDING);
    assert_eq!(
        fs::read(journal.join("sources/third.txt")).unwrap(),
        original
    );
    let partial = fs::read_dir(journal.join("undo"))
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.metadata().unwrap().len())
        .collect::<Vec<_>>();
    assert!(partial
        .iter()
        .any(|&size| size > 0 && size < original.len() as u64));
    for _ in 0..2 {
        let state = start(root.path());
        let core = state.core().unwrap();
        assert_eq!(marker(&core.db), "current author writing");
        assert_eq!(
            fs::read(core.dirs.sources().join("third.txt")).unwrap(),
            original
        );
        assert_eq!(
            fs::read(core.dirs.sources().join("first.txt")).unwrap(),
            b"current first"
        );
        assert!(!core.dirs.sources().join("second.txt").exists());
        assert!(!pending(&core.dirs));
    }
}

#[test]
fn damaged_journal_blocks_startup_without_touching_database_or_sources() {
    let root = fixture();
    expect_exit(
        child_process(root.path(), "restore", "crash:file-0-installed"),
        86,
        "file installed",
    );
    let dirs = DataDirs::ensure(root.path().join("live")).unwrap();
    let journal = dirs.pre_restore_backups().join(PENDING).join(JOURNAL);
    let before_db = fs::read(dirs.database_file()).unwrap();
    let before_file = fs::read(dirs.sources().join("first.txt")).unwrap();
    let mut content: serde_json::Value =
        serde_json::from_slice(&fs::read(&journal).unwrap()).unwrap();
    content["journal"]["entries"][0]["target"] = serde_json::json!("sources/../outside.txt");
    fs::write(&journal, serde_json::to_vec(&content).unwrap()).unwrap();
    let state = start(root.path());
    assert!(state.core().is_err());
    assert_eq!(
        state.startup_error().unwrap().code.as_deref(),
        Some("portability.recoveryRequired")
    );
    assert_eq!(fs::read(dirs.database_file()).unwrap(), before_db);
    assert_eq!(
        fs::read(dirs.sources().join("first.txt")).unwrap(),
        before_file
    );
    assert!(pending(&dirs));
}

#[test]
fn unrelated_new_content_at_a_managed_target_is_retained_and_blocks_recovery() {
    let root = fixture();
    expect_exit(
        child_process(root.path(), "restore", "crash:file-0-installed"),
        86,
        "file installed",
    );
    let dirs = DataDirs::ensure(root.path().join("live")).unwrap();
    fs::write(
        dirs.sources().join("first.txt"),
        b"externally changed bytes must not be deleted",
    )
    .unwrap();
    let state = start(root.path());
    assert!(state.core().is_err());
    assert_eq!(
        fs::read(dirs.sources().join("first.txt")).unwrap(),
        b"externally changed bytes must not be deleted"
    );
    assert_eq!(
        fs::read(
            dirs.pre_restore_backups()
                .join(PENDING)
                .join("sources/first.txt")
        )
        .unwrap(),
        b"current first"
    );
}

#[test]
fn damaged_commit_record_blocks_startup_without_undoing_completed_work() {
    let root = fixture();
    expect_exit(
        child_process(root.path(), "restore", "crash:committed"),
        86,
        "committed",
    );
    let dirs = DataDirs::ensure(root.path().join("live")).unwrap();
    let before_database = fs::read(dirs.database_file()).unwrap();
    let outcome = dirs.pre_restore_backups().join(PENDING).join(TERMINAL);
    let mut content: serde_json::Value =
        serde_json::from_slice(&fs::read(&outcome).unwrap()).unwrap();
    content["outcome"]["committed"] = serde_json::json!(false);
    fs::write(&outcome, serde_json::to_vec(&content).unwrap()).unwrap();
    assert!(start(root.path()).core().is_err());
    assert_eq!(fs::read(dirs.database_file()).unwrap(), before_database);
    assert_eq!(
        fs::read(dirs.sources().join("first.txt")).unwrap(),
        b"archived first"
    );
    assert!(pending(&dirs));
}

#[cfg(unix)]
#[test]
fn both_names_left_by_interrupted_link_publication_are_recoverable() {
    for replacement in [false, true] {
        let root = fixture();
        let point = if replacement {
            "crash:file-0-installed"
        } else {
            "crash:file-0-old-moved"
        };
        expect_exit(child_process(root.path(), "restore", point), 86, point);
        let dirs = DataDirs::ensure(root.path().join("live")).unwrap();
        let transaction = dirs.pre_restore_backups().join(PENDING);
        let target = dirs.sources().join("first.txt");
        // Recreate exactly the hard-link-plus-unlink crash window: both names
        // refer to the same complete bytes. Neither is a partial copy.
        if replacement {
            fs::hard_link(&target, transaction.join("incoming/0")).unwrap();
        } else {
            fs::hard_link(transaction.join("sources/first.txt"), &target).unwrap();
        }
        check_old(root.path());
        check_old(root.path());
    }
}

#[cfg(unix)]
#[test]
fn a_recovery_symlink_never_redirects_file_writes() {
    use std::os::unix::fs::symlink;
    let root = fixture();
    expect_exit(
        child_process(root.path(), "restore", "crash:file-0-old-moved"),
        86,
        "old moved",
    );
    let outside = root.path().join("outside");
    fs::write(&outside, b"outside unchanged").unwrap();
    symlink(&outside, root.path().join("live/sources/first.txt")).unwrap();
    assert!(start(root.path()).core().is_err());
    assert_eq!(fs::read(&outside).unwrap(), b"outside unchanged");
}

#[cfg(target_os = "linux")]
#[test]
fn cross_volume_live_file_publication_is_refused_without_copying() {
    use std::os::unix::fs::MetadataExt;
    let here = tempfile::tempdir().unwrap();
    let Ok(there) = tempfile::tempdir_in("/dev/shm") else {
        return;
    };
    if fs::metadata(here.path()).unwrap().dev() == fs::metadata(there.path()).unwrap().dev() {
        return;
    }
    let original = here.path().join("original");
    let target = there.path().join("target");
    fs::write(&original, b"original bytes").unwrap();
    assert!(move_new(&original, &target).is_err());
    assert_eq!(fs::read(&original).unwrap(), b"original bytes");
    assert!(!target.exists());
}
