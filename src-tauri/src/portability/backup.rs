//! Everything as one backup (`.awbackup`, v0.19), and restoring it.
//!
//! A backup holds a consistent copy of the database, made with SQLite's
//! `VACUUM INTO` (never a copy of the live file, which may be half-written
//! or have recent changes still in its WAL file), and the files Abiding Word
//! keeps in `sources/`: library files, imported Bibles' source files, kept
//! images and church templates' images. It is a private copy for the person
//! themselves, not something to share: it can hold books and Bibles whose
//! licences allow a personal copy but not passing them on.
//!
//! It never holds API keys or other secrets: those live only in the
//! operating system's credential store, never in the database or the data
//! folder. Nor does it hold the cache, logs, earlier backups or exports.
//!
//! Restoring replaces the managed files and database with the backup's, after:
//!
//! 1. reading the whole backup and checking every part against its manifest;
//! 2. checking the backup's database on its own (integrity, version);
//! 3. a verified safety copy of the current database in `backups/pre-restore/`.
//!
//! 4. a durable, checksummed undo journal, before the first live mutation.
//!
//! Source files are installed first, with differing originals retained beside
//! the safety copy. The restored database then goes through the same startup
//! checks. Failure rolls both back. After process interruption, startup recovers
//! the journal before opening/migrating SQLite; a damaged journal or incomplete
//! recovery blocks editing and exports instead of exposing a mixed state.

use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};

use super::{invalid, not_a_package, Limits, PackageFile, Reader, Staging, Writer};
use crate::db::{backup as db_backup, integrity, migrations, Database};
use crate::error::{AppError, AppResult, UserMessage};
use crate::filesystem::DataDirs;

pub const FORMAT: &str = "abiding-word-backup";
pub const FORMAT_VERSION: u32 = 1;
pub const EXTENSION: &str = "awbackup";

const DATABASE: &str = "database.db";
const FILES: &str = "files/";

const LIMITS: Limits = Limits {
    entries: 200_000,
    entry: 8 * 1024 * 1024 * 1024,
    total: 64 * 1024 * 1024 * 1024,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct Counts {
    pub projects: u32,
    pub studies: u32,
    pub teachings: u32,
    pub sources: u32,
    pub bibles: u32,
    pub files: u32,
}

/// Where a file of the backup belongs, relative to the data folder.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Place {
    pub entry: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub format: String,
    pub format_version: u32,
    pub schema_version: i64,
    pub app_version: String,
    pub created_at: String,
    #[serde(default)]
    pub counts: Counts,
    #[serde(default)]
    pub places: Vec<Place>,
    pub files: Vec<PackageFile>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub created_at: String,
    pub app_version: String,
    pub counts: Counts,
    /// Bytes of all its parts together.
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BackupReport {
    pub counts: Counts,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RestoreReport {
    pub counts: Counts,
    /// The copy of the database as it was just before restoring.
    pub safety_copy: String,
    /// Files of the backup added to `sources/`.
    pub files_added: u32,
    /// Files in `sources/` that differed from the backup's and were moved
    /// aside next to the safety copy.
    pub files_set_aside: u32,
}

fn counts(conn: &Connection, files: u32) -> AppResult<Counts> {
    let n = |sql: &str| -> AppResult<u32> { Ok(conn.query_row(sql, [], |r| r.get(0))?) };
    Ok(Counts {
        projects: n("SELECT COUNT(*) FROM projects WHERE deleted_at IS NULL")?,
        studies: n("SELECT COUNT(*) FROM studies WHERE deleted_at IS NULL")?,
        teachings: n("SELECT COUNT(*) FROM teaching_preparations WHERE deleted_at IS NULL")?,
        sources: n("SELECT COUNT(*) FROM sources WHERE deleted_at IS NULL")?,
        bibles: n("SELECT COUNT(*) FROM bible_translations WHERE is_builtin = 0")?,
        files,
    })
}

/// The files under `dir`, relative to `root`, in a stable order. Links are
/// not followed: only real files Abiding Word put there.
fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, PathBuf)>) -> AppResult<()> {
    let entries = fs::read_dir(dir).map_err(|e| AppError::filesystem(dir, e))?;
    let mut entries: Vec<_> = entries
        .collect::<Result<_, _>>()
        .map_err(|e| AppError::filesystem(dir, e))?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        let kind = entry
            .file_type()
            .map_err(|e| AppError::filesystem(&path, e))?;
        if kind.is_dir() {
            walk(root, &path, out)?;
        } else if kind.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|_| AppError::Unexpected("file outside the data folder".into()))?;
            let parts: Option<Vec<&str>> = relative
                .components()
                .map(|c| match c {
                    Component::Normal(p) => p.to_str(),
                    _ => None,
                })
                .collect();
            match parts {
                Some(parts) if safe_place(&parts.join("/")) => {
                    out.push((parts.join("/"), path));
                }
                // A name no other system could hold is left where it is.
                _ => log::warn!("not backed up (unusual name): {}", path.display()),
            }
        }
    }
    Ok(())
}

/// Where a restored file may go: inside `sources/`, as plain relative parts
/// that are valid file names on every system Abiding Word runs on.
pub(crate) fn safe_place(path: &str) -> bool {
    const RESERVED: [&str; 22] = [
        "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
        "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
    ];
    let Some(rest) = path.strip_prefix("sources/") else {
        return false;
    };
    path.len() <= 1000
        && !rest.is_empty()
        && rest.split('/').all(|part| {
            let stem = part.split('.').next().unwrap_or("").to_ascii_lowercase();
            !part.is_empty()
                && part.len() <= 240
                && part != "."
                && part != ".."
                && !part.ends_with('.')
                && !part.ends_with(' ')
                && !RESERVED.contains(&stem.as_str())
                && !part.chars().any(|c| {
                    c.is_control() || matches!(c, '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
                })
        })
}

/// Writes a backup of everything to `target`.
pub fn create(db: &Database, dirs: &DataDirs, target: &Path) -> AppResult<BackupReport> {
    db_backup::ensure_directory(&dirs.cache())?;
    db_backup::check_directory(&dirs.sources())?;
    let staging = Staging::new(&dirs.cache(), "backup")?;
    let snapshot = staging.path().join(DATABASE);
    let version = db.with_conn(|conn| {
        let version = migrations::current_version(conn)?;
        snapshot_into(conn, &snapshot)?;
        Ok(version)
    })?;
    let copy = open_checked(&snapshot, version)?;

    let mut files = Vec::new();
    walk(dirs.root(), &dirs.sources(), &mut files)?;
    let counts = counts(&copy, files.len() as u32)?;
    drop(copy);

    let mut package = Writer::create(target)?;
    package.add_file(DATABASE, &snapshot)?;
    let mut places = Vec::new();
    for (index, (to, path)) in files.iter().enumerate() {
        let entry = format!("{FILES}{index}");
        package.add_file(&entry, path)?;
        places.push(Place {
            entry,
            to: to.clone(),
        });
    }
    let size = package.files.iter().map(|f| f.size).sum();
    let manifest = Manifest {
        format: FORMAT.into(),
        format_version: FORMAT_VERSION,
        schema_version: version,
        app_version: env!("CARGO_PKG_VERSION").into(),
        created_at: crate::db::now(),
        counts: counts.clone(),
        places,
        files: package.files.clone(),
    };
    package.finish(&manifest)?;
    Ok(BackupReport { counts, size })
}

/// A complete, consistent copy of the live database (WAL included).
fn snapshot_into(conn: &Connection, path: &Path) -> AppResult<()> {
    let target = path
        .to_str()
        .ok_or_else(|| AppError::Unexpected("path is not valid UTF-8".into()))?;
    conn.execute("VACUUM INTO ?1", [target])?;
    Ok(())
}

/// Opens a database copy read-only and checks it: intact, an Abiding Word
/// database, and at most `newest` in schema version.
pub(super) fn open_checked(path: &Path, newest: i64) -> AppResult<Connection> {
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|_| not_an_app_database())?;
    integrity::check(&conn).map_err(|_| not_an_app_database())?;
    let tables: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table'
             AND name IN ('schema_versions', 'workspaces', 'projects', 'studies', 'settings')",
            [],
            |r| r.get(0),
        )
        .map_err(|_| not_an_app_database())?;
    if tables != 5 {
        return Err(not_an_app_database());
    }
    let version = migrations::current_version(&conn).map_err(|_| not_an_app_database())?;
    if version > newest {
        return Err(newer(&format!("schema {version}")));
    }
    migrations::validate_existing_history(&conn)?;
    db_backup::validate_upgrade_schema(&conn, version).map_err(|_| not_an_app_database())?;
    Ok(conn)
}

fn not_an_app_database() -> AppError {
    invalid(
        "portability.notBackupDatabase",
        "The database in this backup is damaged or is not an Abiding Word database. Nothing was changed.",
    )
}

fn newer(version: &str) -> AppError {
    AppError::Import(
        UserMessage::new(
            "portability.newer",
            format!(
                "This file was saved by a newer version of Abiding Word ({version}). Please update Abiding Word first."
            ),
        )
        .with("version", version),
    )
}

fn accepted(path: &str) -> bool {
    path == DATABASE
        || path
            .strip_prefix(FILES)
            .is_some_and(|n| !n.is_empty() && n.len() <= 9 && n.bytes().all(|b| b.is_ascii_digit()))
}

fn open(db: &Database, path: &Path) -> AppResult<(Reader, Manifest)> {
    let (reader, manifest) = Reader::open::<Manifest>(path, LIMITS, accepted, |m| &m.files)?;
    if manifest.format != FORMAT {
        return Err(not_a_package());
    }
    let ours = db.with_conn(migrations::current_version)?;
    if manifest.format_version > FORMAT_VERSION || manifest.schema_version > ours {
        return Err(newer(&manifest.app_version));
    }
    if !reader.files.iter().any(|f| f.path == DATABASE) {
        return Err(not_a_package());
    }
    // Every file has one place inside sources/, and every place one file.
    let listed: std::collections::HashSet<&str> = reader
        .files
        .iter()
        .map(|f| f.path.as_str())
        .filter(|p| *p != DATABASE)
        .collect();
    let mut placed = std::collections::HashSet::new();
    let mut targets = std::collections::HashSet::new();
    for place in &manifest.places {
        if !listed.contains(place.entry.as_str())
            || !safe_place(&place.to)
            || !placed.insert(place.entry.as_str())
            || !targets.insert(place.to.to_lowercase())
        {
            return Err(not_a_package());
        }
    }
    if placed.len() != listed.len() {
        return Err(not_a_package());
    }
    Ok((reader, manifest))
}

/// What a backup holds; nothing is changed.
pub fn inspect(db: &Database, path: &Path) -> AppResult<Preview> {
    let (reader, manifest) = open(db, path)?;
    Ok(Preview {
        created_at: manifest.created_at,
        app_version: manifest.app_version,
        counts: manifest.counts,
        size: reader.files.iter().map(|f| f.size).sum(),
    })
}

/// Replaces everything with the backup at `path` (see the module notes).
pub fn restore(db: &Database, dirs: &DataDirs, path: &Path) -> AppResult<RestoreReport> {
    let (mut reader, manifest) = open(db, path)?;
    let ours = db.with_conn(migrations::current_version)?;

    // 1–2. Everything unpacked and checked before anything changes.
    db_backup::ensure_directory(&dirs.cache())?;
    let staging = Staging::new(&dirs.cache(), "restore")?;
    let staged_db = staging.path().join(DATABASE);
    reader.extract(DATABASE, &staged_db)?;
    drop(open_checked(&staged_db, ours)?);
    for place in &manifest.places {
        reader.extract(&place.entry, &staging.path().join(&place.entry))?;
    }

    // Refuse inaccessible routes, symlinks/reparse points and a directory
    // standing where a file belongs before replacing any database content.
    // Repeat the checks when each file is installed; a missing source is not
    // silently reported as a successful full restore.
    for place in &manifest.places {
        check_target(&dirs.root().join(&place.to))?;
    }

    // One lock spans journal preparation, source/database replacement and
    // rollback. Queued database work is refused if recovery cannot complete.
    db.with_conn_mut(|conn| {
        use super::restore_journal::{self as journal, Input, Stamp, Transaction};
        if journal::pending(dirs) {
            let message = journal::recovery_message(dirs);
            db.block_after_restore_failure(message.clone());
            return Err(AppError::Import(message));
        }
        migrations::validate_existing_history(conn)?;
        let version = migrations::current_version(conn)?;
        let safety = db_backup::pre_migration(conn, &dirs.pre_restore_backups(), version)?;
        log::info!("safety copy before restoring: {}", safety.display());
        let inputs: Vec<_> = manifest
            .places
            .iter()
            .map(|place| {
                let file = reader
                    .files
                    .iter()
                    .find(|file| file.path == place.entry)
                    .expect("the package reader validated every entry");
                Input {
                    target: place.to.clone(),
                    staged: staging.path().join(&place.entry),
                    stamp: Stamp {
                        size: file.size,
                        sha256: file.sha256.clone(),
                    },
                }
            })
            .collect();
        let transaction = match Transaction::prepare(dirs, &safety, version, &inputs) {
            Ok(transaction) => transaction,
            Err(error) => {
                log::error!("full-backup restore preparation failed: {error}");
                if journal::pending(dirs) {
                    let message = journal::recovery_message(dirs);
                    db.block_after_restore_failure(message.clone());
                    return Err(AppError::Import(message));
                }
                return Err(AppError::Import(UserMessage::new(
                    "portability.restoreFailed",
                    "The backup could not be restored. Your work was put back as it was.",
                )));
            }
        };
        let restored: AppResult<RestoreReport> = (|| {
            // Files first; any interruption is covered by the already-durable
            // journal. The database is never exposed with partial source files.
            transaction.install_files(dirs)?;
            conn.execute_batch("PRAGMA synchronous=FULL;")?;
            conn.restore(
                rusqlite::MAIN_DB,
                &staged_db,
                None::<fn(rusqlite::backup::Progress)>,
            )?;
            journal::checkpoint("database-restored")?;
            crate::db::reinitialise(conn, Some(&dirs.pre_migration_backups()))?;
            db_backup::validate_schema(conn, migrations::current_version(conn)?)?;
            journal::checkpoint("database-reinitialised")?;
            let report = RestoreReport {
                counts: counts(conn, manifest.places.len() as u32)?,
                safety_copy: safety.display().to_string(),
                files_added: transaction.files_added(),
                files_set_aside: transaction.files_set_aside(),
            };
            journal::flush_database(conn, db.path())?;
            journal::checkpoint("database-flushed")?;
            transaction.mark_done(true)?;
            transaction.finish(dirs)?;
            Ok(report)
        })();
        match restored {
            Ok(report) => Ok(report),
            Err(error) => {
                log::error!("complete backup restore interrupted: {error}");
                // Once the durable commit exists, never undo a completed
                // restore. Startup will finish retaining its recovery files.
                let rolled_back = if transaction.has_outcome() {
                    false
                } else {
                    let rollback = (|| {
                        transaction.undo_files(dirs)?;
                        conn.restore(
                            rusqlite::MAIN_DB,
                            &safety,
                            None::<fn(rusqlite::backup::Progress)>,
                        )?;
                        crate::db::reinitialise(conn, None)?;
                        db_backup::validate_schema(conn, migrations::current_version(conn)?)?;
                        journal::flush_database(conn, db.path())?;
                        journal::checkpoint("database-rolled-back")?;
                        transaction.mark_done(false)?;
                        transaction.finish(dirs)
                    })();
                    if let Err(error) = &rollback {
                        log::error!("restore rollback incomplete: {error}");
                    }
                    rollback.is_ok()
                };
                if rolled_back {
                    Err(AppError::Import(UserMessage::new(
                        "portability.restoreFailed",
                        "The backup could not be restored. Your work was put back as it was.",
                    )))
                } else {
                    // Do not let stale autosaves, exports or snapshots use the
                    // half-restored connection. Startup recovery runs first.
                    let message = journal::recovery_message(dirs);
                    db.block_after_restore_failure(message.clone());
                    Err(AppError::Import(message))
                }
            }
        }
    })
}

fn check_target(target: &Path) -> AppResult<()> {
    db_backup::check_directory(target.parent().expect("managed file parent"))?;
    match fs::symlink_metadata(target) {
        Ok(metadata) => {
            #[cfg(windows)]
            let reparse = {
                use std::os::windows::fs::MetadataExt;
                metadata.file_attributes() & 0x400 != 0
            };
            #[cfg(not(windows))]
            let reparse = false;
            if !metadata.file_type().is_file() || reparse {
                return Err(AppError::filesystem(
                    target,
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "a recovery target must be a regular file, never a link or directory",
                    ),
                ));
            }
            Ok(())
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(AppError::filesystem(target, error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn places_stay_inside_sources() {
        for good in [
            "sources/bibles/web/John.usfm",
            "sources/3f2a/Calvin, Institutes (1559).pdf",
            "sources/teaching-images/0a1b.png",
            "sources/한국어/책.epub",
        ] {
            assert!(safe_place(good), "{good}");
        }
        for bad in [
            "database/abiding-word.db",
            "sources/",
            "sources/../database/abiding-word.db",
            "sources/a/../../x",
            "/sources/x",
            "sources/C:/x",
            "sources/a\\b",
            "sources/con.txt",
            "sources/x.",
            "sources//x",
            "logs/x",
        ] {
            assert!(!safe_place(bad), "{bad:?}");
        }
    }
}
