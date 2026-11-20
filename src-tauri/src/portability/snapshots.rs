//! Safely inspect and restore the verified, database-only daily recovery points.
//! Unlike a full .awbackup, these NEVER contain original source/Bible/image files.
//! Restoration requires explicit user confirmation and retains a verified copy
//! of the CURRENT database before any change. No direct filesystem paths from UI.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use chrono::NaiveDateTime;
use rusqlite::{Connection, OpenFlags};
use serde::Serialize;

use crate::db::{self, backup, integrity, migrations, Database};
use crate::error::{AppError, AppResult, UserMessage};
use crate::filesystem::DataDirs;

const MAX_VISIBLE: usize = 100;
const MAX_BYTES: u64 = 8 * 1024 * 1024 * 1024;
const DATE_PATTERN: &str = "%Y-%m-%dT%H-%M-%S%.3fZ";

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
    if name.len() > 100 || !name.is_ascii() || (name.contains('/') || name.contains('\\')) {
        return None;
    }
    let stem = name.strip_prefix("abiding-word-v")?.strip_suffix(".db")?;
    let (version, stamp) = stem.split_once('-')?;
    if version.is_empty() || !version.bytes().all(|b| b.is_ascii_digit()) || stamp.len() != 24 {
        return None;
    }
    let version = version.parse::<i64>().ok()?;
    if !(1..=10_000).contains(&version) {
        return None;
    }
    let when = NaiveDateTime::parse_from_str(stamp, DATE_PATTERN).ok()?;
    Some((version, when))
}

fn created(when: NaiveDateTime) -> String {
    format!("{}Z", when.format("%Y-%m-%dT%H:%M:%S%.3f"))
}

fn check_snapshot(path: &Path, expected_version: i64) -> AppResult<u64> {
    let metadata = fs::symlink_metadata(path).map_err(|_| unavailable())?;
    if !metadata.file_type().is_file() || metadata.len() == 0 || metadata.len() > MAX_BYTES {
        return Err(unavailable());
    }
    // Opening read-only does not create or modify a candidate snapshot.
    let snap = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|_| unavailable())?;
    integrity::check(&snap).map_err(|_| unavailable())?;
    migrations::validate_existing_history(&snap).map_err(|_| unavailable())?;
    if migrations::current_version(&snap).map_err(|_| unavailable())? != expected_version {
        return Err(unavailable());
    }
    Ok(metadata.len())
}

/// Newest first. Includes corrupt and old-version candidates as disabled rows;
/// never returns arbitrary files, symlinks or user-supplied filesystem paths.
pub fn list(db: &Database, dirs: &DataDirs) -> AppResult<Vec<Snapshot>> {
    let dir = dirs.daily_backups();
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
                usable: schema_version == version
                    && check_snapshot(&path, version).is_ok(),
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
    fs::create_dir_all(destination_dir)
        .map_err(|e| AppError::filesystem(destination_dir, e))?;
    let name = format!("recovery-{}.db", uuid::Uuid::new_v4());
    let target = destination_dir.join(name);
    let stage = StagedFile(target.clone());
    let mut input = File::open(source).map_err(|_| unavailable())?;
    let mut output = File::create_new(&target)
        .map_err(|e| AppError::filesystem(&target, e))?;
    let bytes = std::io::copy(&mut input.take(MAX_BYTES + 1), &mut output)
        .map_err(|e| AppError::filesystem(&target, e))?;
    if bytes > MAX_BYTES {
        return Err(unavailable());
    }
    output.flush().map_err(|e| AppError::filesystem(&target, e))?;
    output.sync_all().map_err(|e| AppError::filesystem(&target, e))?;
    drop(output);
    check_snapshot(&target, version)?;
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
    check_snapshot(&candidate, current)?;
    let stage = staged_copy(&candidate, &dirs.cache(), current)?;

    // This verified backup is made before ANY mutation of the active database.
    let safety = db.with_conn(|conn| {
        backup::pre_migration(conn, &dirs.pre_restore_backups(), migrations::current_version(conn)?)
    })?;

    db.with_conn_mut(|conn| {
        let result = (|| {
            conn.restore(
                rusqlite::MAIN_DB,
                &stage.0,
                None::<fn(rusqlite::backup::Progress)>,
            )?;
            db::reinitialise(conn, None)
        })();
        match result {
            Ok(()) => Ok(()),
            Err(error) => {
                log::error!("daily recovery failed; restoring the pre-recovery safety copy: {error}");
                let rollback = conn
                    .restore(
                        rusqlite::MAIN_DB,
                        &safety,
                        None::<fn(rusqlite::backup::Progress)>,
                    )
                    .map_err(AppError::from)
                    .and_then(|_| db::reinitialise(conn, None));
                match rollback {
                    Ok(()) => Err(AppError::Import(UserMessage::new(
                        "portability.restoreFailed",
                        "The recovery point could not be restored. Your earlier work was put back.",
                    ))),
                    Err(again) => {
                        log::error!("daily recovery rollback failed: {again}");
                        Err(AppError::Import(
                            UserMessage::new(
                                "portability.rollbackFailed",
                                "Recovery failed; your previous database has been preserved in a safety copy.",
                            )
                            .with("path", safety.display()),
                        ))
                    }
                }
            }
        }
    })?;
    Ok(RestoredSnapshot {
        created_at: created(when),
        safety_copy: safety.display().to_string(),
        files_unchanged: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_identifiers_are_strict_basenames_with_real_dates() {
        assert_eq!(parse_name("abiding-word-v27-2026-11-20T17-35-12.123Z.db").unwrap().0, 27);
        for bad in [
            "../abiding-word-v27-2026-11-20T17-35-12.123Z.db",
            "C:\\abiding-word-v27-2026-11-20T17-35-12.123Z.db",
            "abiding-word-v27-2026-22-20T17-35-12.123Z.db",
            "abiding-word-v27-2026-11-20T17-35-12Z.db",
            "abiding-word-v0-2026-11-20T17-35-12.123Z.db",
            "abiding-word-v27-2026-11-20T17-35-12.123Z.db/other",
            "not-a-snapshot.db",
        ] {
            assert!(parse_name(bad).is_none(), "{bad}");
        }
    }
}
