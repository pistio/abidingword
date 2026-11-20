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
//! Restoring first verifies every backup entry, then makes a verified safety
//! copy of the current database. It installs staged managed files with an
//! in-process rollback log before replacing SQLite. If any file operation
//! fails, originals are moved back and the database remains unchanged.
//! If database replacement or reinitialisation fails, the database safety
//! copy is restored and the staged file changes are reversed. Originals
//! replaced by a successful restore are kept beside the safety copy.
//!
//! A power loss during the file phase still needs a durable, on-disk
//! recovery journal: this is a draft safety improvement, NOT the complete
//! crash-recovery release gate.

use std::fs;
use std::path::{Component, Path, PathBuf};

use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};

use super::{file_stamp, invalid, not_a_package, Limits, PackageFile, Reader, Staging, Writer};
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
    let Ok(entries) = fs::read_dir(dir) else {
        return Ok(());
    };
    let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();
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
fn open_checked(path: &Path, newest: i64) -> AppResult<Connection> {
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
    let staging = Staging::new(&dirs.cache(), "restore")?;
    let staged_db = staging.path().join(DATABASE);
    reader.extract(DATABASE, &staged_db)?;
    drop(open_checked(&staged_db, ours)?);
    for place in &manifest.places {
        reader.extract(&place.entry, &staging.path().join(&place.entry))?;
    }

    // 3. The safety copy, verified.
    let safety = db.with_conn(|conn| {
        db_backup::pre_migration(
            conn,
            &dirs.pre_restore_backups(),
            migrations::current_version(conn)?,
        )
    })?;
    log::info!("safety copy before restoring: {}", safety.display());

    // 4. Install staged files before replacing the database, retaining a
    //    reversible record of every source file that changes. A failed
    //    move cannot leave the database pointing at half-restored files.
    let aside =
        dirs.pre_restore_backups()
            .join(format!("files-{}-{}", file_stamp(), uuid::Uuid::new_v4()));
    let mut changes = Vec::<FileChange>::new();
    let mut added = 0;
    let mut set_aside = 0;
    let files_result = (|| -> AppResult<()> {
        for place in &manifest.places {
            let from = staging.path().join(&place.entry);
            let target = dirs.root().join(&place.to);
            check_restore_target(dirs.root(), &target)?;
            let previous = match fs::symlink_metadata(&target) {
                Ok(meta) if meta.file_type().is_file() => {
                    let listed = reader
                        .files
                        .iter()
                        .find(|f| f.path == place.entry)
                        .ok_or_else(not_a_package)?;
                    if existing_equals(&target, listed) {
                        continue;
                    }
                    let saved = aside.join(&place.to);
                    if let Some(dir) = saved.parent() {
                        fs::create_dir_all(dir).map_err(|e| AppError::filesystem(dir, e))?;
                    }
                    fs::rename(&target, &saved).map_err(|e| AppError::filesystem(&target, e))?;
                    set_aside += 1;
                    Some(saved)
                }
                Ok(_) => return Err(restore_failed()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                Err(error) => return Err(AppError::filesystem(&target, error)),
            };
            changes.push(FileChange {
                target: target.clone(),
                prior: previous,
                installed: false,
            });
            if let Some(dir) = target.parent() {
                fs::create_dir_all(dir).map_err(|e| AppError::filesystem(dir, e))?;
            }
            // Staging and managed sources are under the same data root.
            // Never perform an unverified partial copy into the final path.
            fs::rename(&from, &target).map_err(|e| AppError::filesystem(&target, e))?;
            changes.last_mut().expect("change pushed").installed = true;
            added += 1;
        }
        Ok(())
    })();
    if let Err(error) = files_result {
        log::error!("restoring source files failed: {error}");
        rollback_files_or_report(&changes, &safety, &aside)?;
        return Err(restore_failed());
    }

    // 5. Replace the SQLite database ONLY after all managed files are ready.
    //    On an error, restore the verified safety database and undo the
    //    managed file changes too.
    let result = db.with_conn_mut(|conn| {
        let restored = (|| {
            conn.restore(
                rusqlite::MAIN_DB,
                &staged_db,
                None::<fn(rusqlite::backup::Progress)>,
            )?;
            crate::db::reinitialise(conn, Some(&dirs.pre_migration_backups()))?;
            counts(conn, manifest.places.len() as u32)
        })();
        match restored {
            Ok(counts) => Ok(counts),
            Err(error) => {
                log::error!("restoring the backup failed, putting the safety copy back: {error}");
                let back = conn
                    .restore(
                        rusqlite::MAIN_DB,
                        &safety,
                        None::<fn(rusqlite::backup::Progress)>,
                    )
                    .map_err(AppError::from)
                    .and_then(|_| crate::db::reinitialise(conn, None));
                match back {
                    Ok(()) => Err(restore_failed()),
                    Err(e) => {
                        log::error!("the database safety copy could not be put back: {e}");
                        Err(AppError::Import(
                            UserMessage::new(
                                "portability.rollbackFailed",
                                "The backup could not be restored and your earlier work could not be put back automatically.",
                            ).with("path", safety.display()),
                        ))
                    }
                }
            }
        }
    });
    match result {
        Ok(counts) => Ok(RestoreReport {
            counts,
            safety_copy: safety.display().to_string(),
            files_added: added,
            files_set_aside: set_aside,
        }),
        Err(error) => {
            rollback_files_or_report(&changes, &safety, &aside)?;
            Err(error)
        }
    }
}

struct FileChange {
    target: PathBuf,
    prior: Option<PathBuf>,
    installed: bool,
}

fn restore_failed() -> AppError {
    AppError::Import(UserMessage::new(
        "portability.restoreFailed",
        "The backup could not be restored. Your earlier work was put back as it was.",
    ))
}

// Existing symlinks in any managed path are never followed. This protects
// against redirects from sources/ into an unrelated folder.
fn check_restore_target(root: &Path, target: &Path) -> AppResult<()> {
    let relative = target.strip_prefix(root).map_err(|_| not_a_package())?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        current.push(component.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(meta) if meta.file_type().is_symlink() => return Err(restore_failed()),
            Ok(meta) if current.as_path() != target && !meta.is_dir() => {
                return Err(restore_failed())
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(AppError::filesystem(&current, error)),
        }
    }
    Ok(())
}

// Match a potentially huge source file without reading it all into RAM.
fn existing_equals(path: &Path, listed: &PackageFile) -> bool {
    use sha2::{Digest, Sha256};
    let Ok(meta) = fs::symlink_metadata(path) else {
        return false;
    };
    if !meta.file_type().is_file() || meta.len() != listed.size {
        return false;
    }
    let Ok(mut input) = fs::File::open(path) else {
        return false;
    };
    let mut digest = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        use std::io::Read;
        match input.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => digest.update(&buf[..n]),
            Err(_) => return false,
        }
    }
    format!("{:x}", digest.finalize()) == listed.sha256
}

// Restore originals in reverse order, even when failure occurred after
// setting a prior file aside but before installing its replacement.
fn rollback_file_changes(changes: &[FileChange]) -> AppResult<()> {
    for change in changes.iter().rev() {
        if change.installed {
            fs::remove_file(&change.target).map_err(|e| AppError::filesystem(&change.target, e))?;
        }
        if let Some(original) = &change.prior {
            fs::rename(original, &change.target).map_err(|e| AppError::filesystem(original, e))?;
        }
    }
    Ok(())
}

fn rollback_files_or_report(changes: &[FileChange], safety: &Path, aside: &Path) -> AppResult<()> {
    if let Err(error) = rollback_file_changes(changes) {
        log::error!(
            "source rollback failed: {error}; database safety: {}; saved originals: {}",
            safety.display(),
            aside.display()
        );
        return Err(AppError::Import(
            UserMessage::new(
                "portability.rollbackFailed",
                "Restore failed and source files could not all be recovered automatically.",
            )
            .with("path", safety.display()),
        ));
    }
    Ok(())
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
