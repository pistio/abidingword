//! Safety copies of the database.
//!
//! Uses SQLite's `VACUUM INTO`, which writes a complete, consistent copy of
//! the live database (including anything still in the WAL file) as a new
//! file. Plain file copying could capture a half-written state.

use std::collections::BTreeMap;
use std::fs::{self, File, Metadata};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use chrono::NaiveDateTime;

use rusqlite::{Connection, OpenFlags};

use crate::db::{integrity, migrations};
use crate::error::{AppError, AppResult};

/// Writes `abiding-word-v<version>-<UTC timestamp>.db` into `dir` and checks
/// that the copy opens, passes an integrity check and has the same schema
/// version. Returns the path of the verified backup.
pub fn pre_migration(conn: &Connection, dir: &Path, version: i64) -> AppResult<PathBuf> {
    // A safety copy is useful only if the current database is itself readable
    // and belongs to the supported migration lineage. Do this before writing.
    integrity::check(conn)?;
    migrations::validate_existing_history(conn)?;
    if migrations::current_version(conn)? != version {
        return Err(backup_error(dir, "the live schema version changed".into()));
    }
    validate_upgrade_schema(conn, version)?;
    ensure_directory(dir)?;

    // Reserve a new filename, including when two backups start within the same
    // millisecond. Never overwrite a previous safety copy or follow a symlink.
    let now = chrono::Utc::now();
    let mut reserved = None;
    for offset in 0..1000 {
        let stamp = (now + chrono::Duration::milliseconds(offset)).format("%Y-%m-%dT%H-%M-%S%.3fZ");
        let path = dir.join(format!("abiding-word-v{version}-{stamp}.db"));
        match File::create_new(&path) {
            Ok(file) => {
                drop(file);
                reserved = Some(path);
                break;
            }
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(backup_error(&path, error.to_string())),
        }
    }
    let path = reserved.ok_or_else(|| backup_error(dir, "no unused backup filename".into()))?;

    let target = path
        .to_str()
        .ok_or_else(|| backup_error(&path, "path is not valid UTF-8".into()))?;

    conn.execute("VACUUM INTO ?1", [target])
        .map_err(|e| backup_error(&path, e.to_string()))?;

    verify_with_policy(&path, version, true)
        .map_err(|e| backup_error(&path, format!("verification failed: {e}")))?;
    Ok(path)
}

fn verify(path: &Path, expected_version: i64) -> AppResult<()> {
    verify_with_policy(path, expected_version, false)
}

fn verify_with_policy(path: &Path, expected_version: i64, before_upgrade: bool) -> AppResult<()> {
    let metadata =
        fs::symlink_metadata(path).map_err(|error| backup_error(path, error.to_string()))?;
    if !metadata.is_file() || redirected(&metadata) {
        return Err(backup_error(
            path,
            "the backup is not a regular local file".into(),
        ));
    }
    let copy = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )?;
    integrity::check(&copy)?;
    migrations::validate_existing_history(&copy)?;
    validate_schema_with_policy(&copy, expected_version, before_upgrade)?;
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
    ensure_directory(dir)?;
    let version = migrations::current_version(conn)?;
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let candidates = fs::read_dir(dir).map_err(|e| backup_error(dir, e.to_string()))?;
    let mut already_saved = false;
    for item in candidates {
        let item = item.map_err(|e| backup_error(dir, e.to_string()))?;
        let name = item.file_name().to_string_lossy().into_owned();
        // Only regular files we created, not symlinks, other backups or a
        // database file placed here manually.
        let Some((named_version, when)) = parse_snapshot_name(&name) else {
            continue;
        };
        if named_version != version
            || when.format("%Y-%m-%d").to_string() != today
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
        let Some((its_version, when)) = parse_snapshot_name(&name) else {
            continue;
        };
        if !item
            .file_type()
            .map_err(|e| backup_error(dir, e.to_string()))?
            .is_file()
        {
            continue;
        }
        // Keep invalid or manually named files for diagnosis. Only our complete,
        // canonical filename and a verified database authorise retention cleanup.
        let path = item.path();
        if verify(&path, its_version).is_ok() {
            snapshots.push((when, path));
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

/// Accept only the complete canonical name generated by the snapshot writer.
/// Shared by listing, restoration and retention, so manual files are not pruned.
pub(crate) fn parse_snapshot_name(name: &str) -> Option<(i64, NaiveDateTime)> {
    const PATTERN: &str = "%Y-%m-%dT%H-%M-%S%.3fZ";
    if name.len() > 100 || !name.is_ascii() || name.contains(['/', '\\']) {
        return None;
    }
    let stem = name.strip_prefix("abiding-word-v")?.strip_suffix(".db")?;
    let (version, stamp) = stem.split_once('-')?;
    let number = version.parse::<i64>().ok()?;
    if !(1..=10_000).contains(&number) || number.to_string() != version || stamp.len() != 24 {
        return None;
    }
    let when = NaiveDateTime::parse_from_str(stamp, PATTERN).ok()?;
    (when.format(PATTERN).to_string() == stamp).then_some((number, when))
}

/// Windows junctions and other reparse points need the same refusal as symlinks.
pub(crate) fn redirected(metadata: &Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
    }
    #[cfg(not(windows))]
    false
}

/// Check existing parents before creating/reading a managed recovery directory.
/// A symlinked daily/cache/safety folder must not redirect recovery outside it.
pub(crate) fn check_directory(dir: &Path) -> AppResult<()> {
    for path in dir.ancestors().filter(|path| !path.as_os_str().is_empty()) {
        match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.is_dir() && !redirected(&metadata) => {}
            Ok(_) => {
                return Err(backup_error(
                    path,
                    "a recovery directory is redirected or is not a directory".into(),
                ))
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(backup_error(path, error.to_string())),
        }
    }
    Ok(())
}

pub(crate) fn ensure_directory(dir: &Path) -> AppResult<()> {
    check_directory(dir)?;
    fs::create_dir_all(dir).map_err(|error| backup_error(dir, error.to_string()))?;
    check_directory(dir)
}

type Schema = Vec<(String, String, String, String)>;

fn schema(conn: &Connection) -> AppResult<Schema> {
    let mut statement = conn.prepare(
        "SELECT type, name, tbl_name, sql FROM sqlite_schema
         WHERE sql IS NOT NULL ORDER BY type, name",
    )?;
    let rows = statement.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

/// Read-only schema compatibility against this version's unchanged migrations.
/// Columns alone do not validate a database: an added trigger, altered CHECK or
/// missing index can silently destroy writing after an otherwise successful
/// restore. Keep exact schema definitions, including constraints and triggers.
/// The reference lives in memory and never migrates the candidate or live work.
pub(crate) fn validate_schema(conn: &Connection, version: i64) -> AppResult<()> {
    validate_schema_with_policy(conn, version, false)
}

/// Migration 0016 deliberately repairs historical dangling citation references.
/// Its pre-upgrade safety copy must preserve those original rows, and an old full
/// backup may enter that same upgrade path. This exception never accepts changed
/// schema definitions or unrelated broken references, and is inapplicable after
/// version 15. The restored/upgraded database must pass strict validation again.
pub(crate) fn validate_upgrade_schema(conn: &Connection, version: i64) -> AppResult<()> {
    validate_schema_with_policy(conn, version, true)
}

fn validate_schema_with_policy(
    conn: &Connection,
    version: i64,
    before_upgrade: bool,
) -> AppResult<()> {
    static EXPECTED: OnceLock<Mutex<BTreeMap<i64, Schema>>> = OnceLock::new();
    if version < 1 || version > migrations::MIGRATIONS.last().map_or(0, |m| m.version) {
        return Err(AppError::Integrity(
            "The backup schema version is unsupported".into(),
        ));
    }
    let expected = {
        let mut cached = EXPECTED
            .get_or_init(Default::default)
            .lock()
            .map_err(|_| AppError::Unexpected("snapshot schema cache lock poisoned".into()))?;
        if let std::collections::btree_map::Entry::Vacant(entry) = cached.entry(version) {
            let mut reference = Connection::open_in_memory()?;
            let count = migrations::MIGRATIONS
                .iter()
                .take_while(|m| m.version <= version)
                .count();
            migrations::run_with(&mut reference, &migrations::MIGRATIONS[..count])?;
            entry.insert(schema(&reference)?);
        }
        cached[&version].clone()
    };
    if schema(conn)? != expected {
        return Err(AppError::Integrity(format!(
            "The database schema definitions do not match version {version}"
        )));
    }
    let mut statement = conn.prepare("PRAGMA foreign_key_check")?;
    let violations = statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(2)?))
    })?;
    for violation in violations {
        let (table, parent) = violation?;
        let repaired_by_v16 = before_upgrade
            && (3..16).contains(&version)
            && table == "finding_sources"
            && matches!(parent.as_str(), "findings" | "sources" | "source_segments");
        if !repaired_by_v16 {
            return Err(AppError::Integrity(
                "The database contains broken record references".into(),
            ));
        }
    }
    Ok(())
}
