//! Safely inspect and restore the verified, database-only daily recovery points.
//! Unlike a full .awbackup, these NEVER contain original source/Bible/image files.
//! Restoration requires explicit user confirmation and retains a verified copy
//! of the CURRENT database before any change. No direct filesystem paths from UI.

use std::fs::{self, File};
use std::path::{Path, PathBuf};

use chrono::NaiveDateTime;
use rusqlite::{Connection, OpenFlags};
use serde::Serialize;

use crate::db::{self, backup, integrity, migrations, Database};
use crate::error::{AppError, AppResult, UserMessage};
use crate::filesystem::DataDirs;

const MAX_VISIBLE: usize = 100;
const MAX_BYTES: u64 = 8 * 1024 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    /// Strictly validated basename; never a filesystem path.
    pub id: String,
    pub created_at: String,
    pub schema_version: i64,
    pub bytes: u64,
    /// False for a damaged file, incompatible schema or failed integrity check.
    pub usable: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RestoredSnapshot {
    pub created_at: String,
    /// Verified backup of the database immediately before the restore.
    pub safety_copy: String,
    /// The managed source folders were not changed (not restored either).
    pub files_unchanged: bool,
}

fn unavailable() -> AppError {
    AppError::InvalidInput(UserMessage::new(
        "portability.snapshotUnavailable",
        "This recovery point is damaged or incompatible. Your work was not changed.",
    ))
}

fn parse_name(name: &str) -> Option<(i64, NaiveDateTime)> {
    backup::parse_snapshot_name(name)
}

fn created(when: NaiveDateTime) -> String {
    format!("{}Z", when.format("%Y-%m-%dT%H:%M:%S%.3f"))
}

fn open_snapshot(path: &Path, expected_version: i64) -> AppResult<(Connection, u64)> {
    backup::check_directory(path.parent().ok_or_else(unavailable)?).map_err(|_| unavailable())?;
    let metadata = fs::symlink_metadata(path).map_err(|_| unavailable())?;
    if !metadata.file_type().is_file()
        || backup::redirected(&metadata)
        || metadata.len() == 0
        || metadata.len() > MAX_BYTES
    {
        return Err(unavailable());
    }
    // Keep this connection when staging: copying the raw .db file would miss
    // committed WAL pages and reopen a filename which could have changed.
    let snap = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )
    .map_err(|_| unavailable())?;
    integrity::check(&snap).map_err(|_| unavailable())?;
    migrations::validate_existing_history(&snap).map_err(|_| unavailable())?;
    if migrations::current_version(&snap).map_err(|_| unavailable())? != expected_version {
        return Err(unavailable());
    }
    backup::validate_schema(&snap, expected_version).map_err(|_| unavailable())?;
    Ok((snap, metadata.len()))
}

/// Newest first. Includes corrupt and old-version candidates as disabled rows;
/// never returns arbitrary files, symlinks or user-supplied filesystem paths.
pub fn list(db: &Database, dirs: &DataDirs) -> AppResult<Vec<Snapshot>> {
    let dir = dirs.daily_backups();
    backup::check_directory(&dir).map_err(|_| unavailable())?;
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let version = db.with_conn(migrations::current_version)?;
    let mut candidates = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| AppError::filesystem(&dir, e))? {
        let entry = entry.map_err(|e| AppError::filesystem(&dir, e))?;
        if !entry
            .file_type()
            .map_err(|e| AppError::filesystem(&dir, e))?
            .is_file()
        {
            continue;
        }
        let id = entry.file_name().to_string_lossy().into_owned();
        if let Some((schema_version, when)) = parse_name(&id) {
            candidates.push((id, schema_version, when, entry.path()));
        }
    }
    candidates.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| b.0.cmp(&a.0)));
    Ok(candidates
        .into_iter()
        .take(MAX_VISIBLE)
        .map(|(id, schema_version, when, path)| {
            let bytes = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            Snapshot {
                id,
                created_at: created(when),
                schema_version,
                bytes,
                usable: schema_version == version && open_snapshot(&path, version).is_ok(),
            }
        })
        .collect())
}

/// A staging copy ensures the exact bytes checked are those that are restored,
/// even if another process changes a daily file during the operation.
struct StagedFile(PathBuf);

impl Drop for StagedFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn staged_copy(source: &Path, destination_dir: &Path, version: i64) -> AppResult<StagedFile> {
    let (input, _) = open_snapshot(source, version)?;
    backup::ensure_directory(destination_dir)?;
    let name = format!("recovery-{}.db", uuid::Uuid::new_v4());
    let target = destination_dir.join(name);
    let stage = StagedFile(target.clone());
    // Reserve our own file before asking SQLite to populate it. SQLite's backup
    // API takes a consistent database snapshot, including committed WAL data.
    drop(File::create_new(&target).map_err(|e| AppError::filesystem(&target, e))?);
    input.backup(rusqlite::MAIN_DB, &target, None)?;
    File::options()
        .write(true)
        .open(&target)
        .and_then(|file| file.sync_all())
        .map_err(|error| AppError::filesystem(&target, error))?;
    open_snapshot(&target, version)?;
    Ok(stage)
}

/// Replace the database only, after an independently verified staging copy and
/// verified pre-restore safety backup. The user must explicitly confirm in UI.
/// Original PDFs/Bibles/images remain untouched; old references might not
/// resolve if those files were separately removed.
pub fn restore(db: &Database, dirs: &DataDirs, id: &str) -> AppResult<RestoredSnapshot> {
    let (file_version, when) = parse_name(id).ok_or_else(unavailable)?;
    let current = db.with_conn(migrations::current_version)?;
    if file_version != current {
        return Err(unavailable());
    }
    let candidate = dirs.daily_backups().join(id);
    let stage = staged_copy(&candidate, &dirs.cache(), current)?;

    // Hold one database lock from current-work validation through safety copy,
    // replacement and rollback. A concurrent native write cannot fall between
    // the safety copy and database replacement and disappear from both.
    let safety = db.with_conn_mut(|conn| {
        if migrations::current_version(conn)? != current {
            return Err(unavailable());
        }
        let safety = backup::pre_migration(conn, &dirs.pre_restore_backups(), current)?;
        restore_staged(conn, &stage.0, &safety, |conn| db::reinitialise(conn, None))?;
        Ok(safety)
    })?;
    Ok(RestoredSnapshot {
        created_at: created(when),
        safety_copy: safety.display().to_string(),
        files_unchanged: true,
    })
}

/// Keep recovery and rollback on the same connection/lock. The initializer is
/// injectable only internally so tests can force both failure paths with real
/// disposable SQLite databases instead of relying on filesystem permissions.
fn restore_staged(
    conn: &mut Connection,
    stage: &Path,
    safety: &Path,
    mut initialise: impl FnMut(&mut Connection) -> AppResult<()>,
) -> AppResult<()> {
    let result = conn
        .restore(
            rusqlite::MAIN_DB,
            stage,
            None::<fn(rusqlite::backup::Progress)>,
        )
        .map_err(AppError::from)
        .and_then(|_| initialise(conn));
    if let Err(error) = result {
        log::error!("daily recovery failed; restoring the pre-recovery safety copy: {error}");
        let rollback = conn
            .restore(
                rusqlite::MAIN_DB,
                safety,
                None::<fn(rusqlite::backup::Progress)>,
            )
            .map_err(AppError::from)
            .and_then(|_| initialise(conn));
        return match rollback {
            Ok(()) => Err(AppError::Import(UserMessage::new(
                "portability.restoreFailed",
                "The recovery point could not be restored. Your earlier work was put back.",
            ))),
            Err(again) => {
                log::error!("daily recovery rollback failed: {again}");
                Err(AppError::Import(UserMessage::new(
                    "portability.rollbackFailed",
                    format!("Recovery failed and your previous database could not be put back automatically. A safety copy is retained at {}.", safety.display()),
                ).with("path", safety.display())))
            }
        };
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_identifiers_are_strict_basenames_with_real_dates() {
        assert_eq!(
            parse_name("abiding-word-v27-2026-11-20T17-35-12.123Z.db")
                .unwrap()
                .0,
            27
        );
        for bad in [
            "../abiding-word-v27-2026-11-20T17-35-12.123Z.db",
            "C:\\abiding-word-v27-2026-11-20T17-35-12.123Z.db",
            "abiding-word-v27-2026-22-20T17-35-12.123Z.db",
            "abiding-word-v27-2026-11-20T17-35-12Z.db",
            "abiding-word-v0-2026-11-20T17-35-12.123Z.db",
            "abiding-word-v027-2026-11-20T17-35-12.123Z.db",
            "abiding-word-v+27-2026-11-20T17-35-12.123Z.db",
            "abiding-word-v27-2026-11-20T17-35-12.123Z.db/other",
            "not-a-snapshot.db",
        ] {
            assert!(parse_name(bad).is_none(), "{bad}");
        }
    }
    fn sqlite_note(conn: &Connection) -> String {
        conn.query_row(
            "SELECT value FROM settings WHERE key='recovery.test'",
            [],
            |row| row.get(0),
        )
        .unwrap()
    }

    fn forced_restore_failure(rollback_also_fails: bool) {
        let temp = tempfile::tempdir().unwrap();
        let dirs = DataDirs::ensure(temp.path().join("profile")).unwrap();
        let db = Database::open(&dirs.database_file(), &dirs.pre_migration_backups()).unwrap();
        db.with_conn(|conn| {
            conn.execute("INSERT INTO settings(key,value,created_at,updated_at) VALUES('recovery.test','older','now','now')", [])?;
            Ok(())
        }).unwrap();
        let original = db
            .with_conn(|conn| backup::daily_snapshot(conn, &dirs.daily_backups()))
            .unwrap()
            .unwrap();
        db.with_conn(|conn| {
            conn.execute(
                "UPDATE settings SET value='newer' WHERE key='recovery.test'",
                [],
            )?;
            Ok(())
        })
        .unwrap();
        let version = db.with_conn(migrations::current_version).unwrap();
        let stage = staged_copy(&original, &dirs.cache(), version).unwrap();
        db.with_conn_mut(|conn| {
            let safety = backup::pre_migration(conn, &dirs.pre_restore_backups(), version)?;
            let mut attempts = 0;
            let error = restore_staged(conn, &stage.0, &safety, |restored| {
                attempts += 1;
                assert_eq!(
                    sqlite_note(restored),
                    if attempts == 1 { "older" } else { "newer" }
                );
                if attempts == 1 || rollback_also_fails {
                    Err(AppError::Unexpected(
                        "Invented post-restore validation failure".into(),
                    ))
                } else {
                    db::reinitialise(restored, None)
                }
            })
            .unwrap_err();
            assert_eq!(attempts, 2);
            assert_eq!(sqlite_note(conn), "newer");
            let (code, params) = error.user_code();
            if rollback_also_fails {
                assert_eq!(code, Some("portability.rollbackFailed"));
                assert_eq!(params.get("path"), Some(&safety.display().to_string()));
                assert!(error.to_string().contains(&safety.display().to_string()));
            } else {
                assert_eq!(code, Some("portability.restoreFailed"));
                conn.execute(
                    "UPDATE settings SET value='still writable' WHERE key='recovery.test'",
                    [],
                )?;
                assert_eq!(sqlite_note(conn), "still writable");
            }
            let saved = Connection::open_with_flags(&safety, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
            assert_eq!(sqlite_note(&saved), "newer");
            assert!(original.exists());
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn a_failed_post_restore_validation_rolls_back_real_sqlite_and_allows_writing() {
        forced_restore_failure(false);
    }

    #[test]
    fn a_failed_rollback_reports_the_retained_readable_safety_copy() {
        forced_restore_failure(true);
    }
}
