//! Safety copies of the database.
//!
//! Uses SQLite's `VACUUM INTO`, which writes a complete, consistent copy of
//! the live database (including anything still in the WAL file) as a new
//! file. Plain file copying could capture a half-written state.

use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};

use crate::db::{integrity, migrations};
use crate::error::{AppError, AppResult};

/// Writes `abiding-word-v<version>-<UTC timestamp>.db` into `dir` and checks
/// that the copy opens, passes an integrity check and has the same schema
/// version. Returns the path of the verified backup.
pub fn pre_migration(conn: &Connection, dir: &Path, version: i64) -> AppResult<PathBuf> {
    fs::create_dir_all(dir).map_err(|e| backup_error(dir, e.to_string()))?;

    // Colons are not allowed in Windows file names.
    let stamp = chrono::Utc::now().format("%Y-%m-%dT%H-%M-%S%.3fZ");
    let path = dir.join(format!("abiding-word-v{version}-{stamp}.db"));
    if path.exists() {
        return Err(backup_error(
            &path,
            "a backup with this name already exists".into(),
        ));
    }
    let target = path
        .to_str()
        .ok_or_else(|| backup_error(&path, "path is not valid UTF-8".into()))?;

    conn.execute("VACUUM INTO ?1", [target])
        .map_err(|e| backup_error(&path, e.to_string()))?;

    verify(&path, version).map_err(|e| backup_error(&path, format!("verification failed: {e}")))?;
    Ok(path)
}

fn verify(path: &Path, expected_version: i64) -> AppResult<()> {
    let copy = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    integrity::check(&copy)?;
    let version = migrations::current_version(&copy)?;
    if version != expected_version {
        return Err(AppError::Unexpected(format!(
            "backup has schema version {version}, expected {expected_version}"
        )));
    }
    Ok(())
}

fn backup_error(path: &Path, message: String) -> AppError {
    AppError::Backup {
        path: path.to_path_buf(),
        message,
    }
}

/// Creates an automatic, verified **database-only** safety snapshot at most
/// once per UTC day. The managed source files are deliberately NOT copied:
/// the user still needs a normal .awbackup for a complete portable backup.
/// Only files this function named are eligible for retention cleanup.
pub fn daily_snapshot(conn: &Connection, dir: &Path) -> AppResult<Option<PathBuf>> {
    const RETAIN: usize = 7;
    fs::create_dir_all(dir).map_err(|e| backup_error(dir, e.to_string()))?;
    let version = migrations::current_version(conn)?;
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let prefix = format!("abiding-word-v{version}-{today}T");

    let candidates = fs::read_dir(dir).map_err(|e| backup_error(dir, e.to_string()))?;
    let mut already_saved = false;
    for item in candidates {
        let item = item.map_err(|e| backup_error(dir, e.to_string()))?;
        let name = item.file_name().to_string_lossy().into_owned();
        // Only regular files we created, not symlinks, other backups or a
        // database file placed here manually.
        if !name.starts_with(&prefix)
            || !name.ends_with(".db")
            || !item
                .file_type()
                .map_err(|e| backup_error(dir, e.to_string()))?
                .is_file()
        {
            continue;
        }
        if verify(&item.path(), version).is_ok() {
            already_saved = true;
            break;
        }
    }
    let created = if already_saved {
        None
    } else {
        Some(pre_migration(conn, dir, version)?)
    };

    let mut snapshots = Vec::new();
    for item in fs::read_dir(dir).map_err(|e| backup_error(dir, e.to_string()))? {
        let item = item.map_err(|e| backup_error(dir, e.to_string()))?;
        let name = item.file_name().to_string_lossy().into_owned();
        if !name.starts_with("abiding-word-v")
            || !name.ends_with(".db")
            || !item
                .file_type()
                .map_err(|e| backup_error(dir, e.to_string()))?
                .is_file()
        {
            continue;
        }
        // Keep an invalid snapshot for diagnosis, rather than silently
        // deleting something that might still be recoverable.
        let path = item.path();
        let version_in_name = name
            .split('-')
            .nth(2)
            .and_then(|s| s.strip_prefix('v'))
            .and_then(|s| s.parse::<i64>().ok());
        if let Some(its_version) = version_in_name {
            if verify(&path, its_version).is_ok() {
                // The timestamp in our own verified filename, NOT the file
                // modification time, determines which recovery point is oldest.
                // Copying an old backup today cannot evict today's snapshot.
                if let Some(stamp) = name.splitn(4, '-').nth(3) {
                    snapshots.push((stamp.to_string(), path));
                }
            }
        }
    }
    snapshots.sort_by(|(a, ap), (b, bp)| a.cmp(b).then_with(|| ap.cmp(bp)));
    let excess = snapshots.len().saturating_sub(RETAIN);
    for (_, old) in snapshots.into_iter().take(excess) {
        if let Err(error) = fs::remove_file(&old) {
            log::warn!(
                "could not prune older automatic database snapshot {}: {error}",
                old.display()
            );
        }
    }
    Ok(created)
}
