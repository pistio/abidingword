//! Durable undo for a complete backup restore.
//!
//! An immutable, checksummed journal is published before changing a managed
//! file or SQLite. Original files are moved into that journal's directory and
//! never consumed by rollback. Until a durable terminal record exists, startup
//! rolls the transaction back BEFORE opening/migrating the active database.
//! A damaged journal, unexpected file or incomplete rollback blocks startup.
//!
//! Files and directories are synced on Unix. Windows renames use the documented
//! MoveFileExW WRITE_THROUGH flag, without COPY_ALLOWED or REPLACE_EXISTING.
//! Cross-volume/unsupported operations fail safely; there is no partial-copy
//! fallback for publishing live files. Native Windows/power-loss acceptance is
//! still distinct from the deterministic process-termination tests.

use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{backup::safe_place, hex, sha256};
use crate::db::{backup as db_backup, migrations};
use crate::error::{AppError, AppResult, UserMessage};
use crate::filesystem::DataDirs;

const PENDING: &str = "pending-full-restore";
const JOURNAL: &str = "journal.json";
const TERMINAL: &str = "outcome.json";
const MAX_JOURNAL: u64 = 128 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Stamp {
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    target: String,
    original: Option<Stamp>,
    replacement: Stamp,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Journal {
    version: u32,
    id: String,
    safety_file: String,
    safety: Stamp,
    schema: i64,
    entries: Vec<Entry>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SealedJournal {
    journal: Journal,
    sha256: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Outcome {
    journal_sha256: String,
    committed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SealedOutcome {
    outcome: Outcome,
    sha256: String,
}

pub(crate) struct Input {
    pub target: String,
    pub staged: PathBuf,
    pub stamp: Stamp,
}

pub(crate) struct Transaction {
    root: PathBuf,
    journal: Journal,
    seal: String,
}

pub(crate) fn recovery_message(dirs: &DataDirs) -> UserMessage {
    UserMessage::new(
        "portability.recoveryRequired",
        "A backup operation needs recovery before you can continue. Close Abiding Word and open it again. If this message returns, keep the recovery files at the indicated location and ask for help.",
    )
    .with("path", dirs.pre_restore_backups().display())
}

pub(crate) fn pending(dirs: &DataDirs) -> bool {
    // An unreadable entry is a blocker too, never permission to start a second
    // restore or to open a possibly half-restored database.
    !matches!(fs::symlink_metadata(dirs.pre_restore_backups().join(PENDING)), Err(e) if e.kind() == io::ErrorKind::NotFound)
}

fn encode<T: Serialize>(value: &T) -> AppResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|error| AppError::Unexpected(error.to_string()))
}

fn fault(path: &Path, text: impl Into<String>) -> AppError {
    AppError::filesystem(
        path,
        io::Error::new(io::ErrorKind::InvalidData, text.into()),
    )
}

fn regular(path: &Path) -> AppResult<Option<fs::Metadata>> {
    db_backup::check_directory(path.parent().ok_or_else(|| fault(path, "missing parent"))?)?;
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_file() && !db_backup::redirected(&meta) => Ok(Some(meta)),
        Ok(_) => Err(fault(path, "a recovery file is not a regular local file")),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(AppError::filesystem(path, e)),
    }
}

/// Bounded memory even for multi-gigabyte imported originals.
pub(crate) fn stamp(path: &Path) -> AppResult<Stamp> {
    let metadata = regular(path)?.ok_or_else(|| fault(path, "recovery file is missing"))?;
    let mut file = File::open(path).map_err(|e| AppError::filesystem(path, e))?;
    let mut hash = Sha256::new();
    let mut size = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let n = file
            .read(&mut buffer)
            .map_err(|e| AppError::filesystem(path, e))?;
        if n == 0 {
            break;
        }
        size += n as u64;
        if size > metadata.len() {
            return Err(fault(path, "file changed while its recovery hash was read"));
        }
        hash.update(&buffer[..n]);
    }
    if size != metadata.len() {
        return Err(fault(path, "file changed while its recovery hash was read"));
    }
    Ok(Stamp {
        size,
        sha256: hex(&hash.finalize()),
    })
}

fn matches(path: &Path, expected: &Stamp) -> AppResult<bool> {
    let Some(meta) = regular(path)? else {
        return Ok(false);
    };
    Ok(meta.len() == expected.size && stamp(path)? == *expected)
}

pub(crate) fn sync_file(path: &Path) -> AppResult<()> {
    regular(path)?.ok_or_else(|| fault(path, "file to sync is missing"))?;
    // Windows FlushFileBuffers requires a write-capable handle.
    fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .and_then(|f| f.sync_all())
        .map_err(|e| AppError::filesystem(path, e))
}

fn sync_directory(path: &Path) -> AppResult<()> {
    db_backup::check_directory(path)?;
    #[cfg(unix)]
    File::open(path)
        .and_then(|f| f.sync_all())
        .map_err(|e| AppError::filesystem(path, e))?;
    // Windows has no portable, unprivileged directory-fsync equivalent.
    // All namespace publication/moves below instead use WRITE_THROUGH.
    Ok(())
}

fn ensure_directory(path: &Path) -> AppResult<()> {
    db_backup::check_directory(path)?;
    let mut missing = Vec::new();
    for parent in path.ancestors().filter(|p| !p.as_os_str().is_empty()) {
        if parent.exists() {
            break;
        }
        missing.push(parent.to_path_buf());
    }
    for directory in missing.iter().rev() {
        fs::create_dir(directory).map_err(|e| AppError::filesystem(directory, e))?;
        sync_directory(directory)?;
        if let Some(parent) = directory.parent() {
            sync_directory(parent)?;
        }
    }
    db_backup::check_directory(path)
}

/// A durable, no-overwrite move. No cross-device copy/delete simulation.
fn move_new(from: &Path, to: &Path) -> AppResult<()> {
    db_backup::check_directory(from.parent().ok_or_else(|| fault(from, "missing parent"))?)?;
    db_backup::check_directory(to.parent().ok_or_else(|| fault(to, "missing parent"))?)?;
    match fs::symlink_metadata(to) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(AppError::filesystem(to, e)),
        Ok(_) => return Err(fault(to, "a recovery destination already exists")),
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_WRITE_THROUGH};
        let from_w: Vec<u16> = from.as_os_str().encode_wide().chain(Some(0)).collect();
        let to_w: Vec<u16> = to.as_os_str().encode_wide().chain(Some(0)).collect();
        // SAFETY: both UTF-16 buffers remain alive and NUL-terminated throughout
        // the call. Omitting REPLACE_EXISTING protects an unexpected target.
        if unsafe { MoveFileExW(from_w.as_ptr(), to_w.as_ptr(), MOVEFILE_WRITE_THROUGH) } == 0 {
            return Err(AppError::filesystem(to, io::Error::last_os_error()));
        }
    }
    #[cfg(not(windows))]
    {
        if fs::symlink_metadata(from)
            .map_err(|e| AppError::filesystem(from, e))?
            .is_dir()
        {
            // Both names are private transaction directories; the destination
            // is never a managed user file. A nonempty pending dir is refused.
            fs::rename(from, to).map_err(|e| AppError::filesystem(to, e))?;
        } else {
            // link() publishes a complete regular file without replacing any
            // concurrently-created target. A crash before unlink leaves both
            // names; rollback recognises identical original/replacement hashes.
            fs::hard_link(from, to).map_err(|e| AppError::filesystem(to, e))?;
            sync_directory(to.parent().expect("checked parent"))?;
            fs::remove_file(from).map_err(|e| AppError::filesystem(from, e))?;
        }
    }
    sync_directory(from.parent().expect("checked parent"))?;
    sync_directory(to.parent().expect("checked parent"))
}

fn write_new(path: &Path, bytes: &[u8]) -> AppResult<()> {
    let mut file = File::create_new(path).map_err(|e| AppError::filesystem(path, e))?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|e| AppError::filesystem(path, e))?;
    sync_directory(path.parent().expect("private transaction file"))
}

fn copy_checked(from: &Path, to: &Path, expected: &Stamp) -> AppResult<()> {
    regular(from)?.ok_or_else(|| fault(from, "copy source is missing"))?;
    let mut input = File::open(from).map_err(|e| AppError::filesystem(from, e))?;
    let mut output = File::create_new(to).map_err(|e| AppError::filesystem(to, e))?;
    let mut hash = Sha256::new();
    let mut size = 0;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let n = input
            .read(&mut buffer)
            .map_err(|e| AppError::filesystem(from, e))?;
        if n == 0 {
            break;
        }
        if size + n as u64 > expected.size {
            return Err(fault(from, "recovery copy exceeds its recorded size"));
        }
        output
            .write_all(&buffer[..n])
            .map_err(|e| AppError::filesystem(to, e))?;
        size += n as u64;
        hash.update(&buffer[..n]);
        #[cfg(test)]
        if to.parent().and_then(Path::file_name) == Some(std::ffi::OsStr::new("undo")) {
            checkpoint("undo-copy-chunk")?;
        }
        checkpoint("copy-chunk")?;
    }
    output.sync_all().map_err(|e| AppError::filesystem(to, e))?;
    sync_directory(to.parent().expect("private transaction file"))?;
    if size != expected.size || hex(&hash.finalize()) != expected.sha256 {
        return Err(fault(to, "a recovery copy failed its size/hash check"));
    }
    Ok(())
}

impl Transaction {
    pub(crate) fn prepare(
        dirs: &DataDirs,
        safety: &Path,
        schema: i64,
        inputs: &[Input],
    ) -> AppResult<Self> {
        if pending(dirs) {
            return Err(AppError::Import(recovery_message(dirs)));
        }
        let id = crate::db::new_id();
        let root = dirs.pre_restore_backups().join(format!("preparing-{id}"));
        ensure_directory(&dirs.pre_restore_backups())?;
        fs::create_dir(&root).map_err(|e| AppError::filesystem(&root, e))?;
        ensure_directory(&root.join("incoming"))?;
        ensure_directory(&root.join("discarded"))?;
        ensure_directory(&root.join("undo"))?;
        // The standalone pre-restore database remains available after success
        // and rollback, outside journal retention/cleanup.
        sync_file(safety)?;
        sync_directory(&dirs.pre_restore_backups())?;
        let mut entries = Vec::new();
        let mut names = HashSet::new();
        for input in inputs {
            let lower = input.target.to_lowercase();
            if !safe_place(&input.target) || !names.insert(lower) {
                return Err(fault(&root, "invalid or repeated restore target"));
            }
            let target = dirs.root().join(&input.target);
            let original = regular(&target)?.map(|_| stamp(&target)).transpose()?;
            if original.is_some() {
                sync_file(&target)?;
                sync_directory(target.parent().expect("safe source path"))?;
            }
            if original.as_ref() == Some(&input.stamp) {
                continue;
            }
            let index = entries.len();
            copy_checked(
                &input.staged,
                &root.join("incoming").join(index.to_string()),
                &input.stamp,
            )?;
            ensure_directory(root.join(&input.target).parent().expect("safe source path"))?;
            entries.push(Entry {
                target: input.target.clone(),
                original,
                replacement: input.stamp.clone(),
            });
        }
        // A file and a child of that file cannot both be restore destinations.
        // Reject this before publishing intent or changing SQLite/source files.
        for name in &names {
            let mut parent = name.as_str();
            while let Some((ancestor, _)) = parent.rsplit_once('/') {
                if names.contains(ancestor) {
                    return Err(fault(&root, "conflicting restore destinations"));
                }
                parent = ancestor;
            }
        }
        let journal = Journal {
            version: 1,
            id,
            safety_file: safety
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| fault(safety, "invalid safety filename"))?
                .into(),
            safety: stamp(safety)?,
            schema,
            entries,
        };
        let seal = sha256(&encode(&journal)?);
        let bytes = encode(&SealedJournal {
            journal,
            sha256: seal.clone(),
        })?;
        if bytes.len() as u64 > MAX_JOURNAL {
            return Err(fault(&root, "restore journal is too large"));
        }
        write_new(&root.join(JOURNAL), &bytes)?;
        let target = dirs.pre_restore_backups().join(PENDING);
        move_new(&root, &target)?;
        let transaction = Self::load(dirs)?;
        checkpoint("journal-published")?;
        Ok(transaction)
    }

    fn load(dirs: &DataDirs) -> AppResult<Self> {
        let root = dirs.pre_restore_backups().join(PENDING);
        db_backup::check_directory(&root)?;
        let path = root.join(JOURNAL);
        let meta = regular(&path)?.ok_or_else(|| fault(&path, "missing restore journal"))?;
        if meta.len() > MAX_JOURNAL {
            return Err(fault(&path, "oversized restore journal"));
        }
        let stored: SealedJournal =
            serde_json::from_slice(&fs::read(&path).map_err(|e| AppError::filesystem(&path, e))?)
                .map_err(|error| fault(&path, format!("invalid restore journal: {error}")))?;
        let j = &stored.journal;
        if stored.sha256 != sha256(&encode(j)?)
            || j.version != 1
            || uuid::Uuid::parse_str(&j.id)
                .map(|id| id.to_string())
                .ok()
                .as_deref()
                != Some(&j.id)
            || db_backup::parse_snapshot_name(&j.safety_file).is_none_or(|(v, _)| v != j.schema)
            || j.entries.len() > 200_000
        {
            return Err(fault(&path, "invalid restore journal identity or checksum"));
        }
        let mut targets = HashSet::new();
        for entry in &j.entries {
            if !safe_place(&entry.target) || !targets.insert(entry.target.to_lowercase()) {
                return Err(fault(&path, "invalid restore journal target"));
            }
        }
        let safety = dirs.pre_restore_backups().join(&j.safety_file);
        if !matches(&safety, &j.safety)? {
            return Err(fault(&safety, "safety database checksum mismatch"));
        }
        // Validation reads the safety copy only, never the active database.
        let checked = super::backup::open_checked(&safety, j.schema)?;
        if migrations::current_version(&checked)? != j.schema {
            return Err(fault(
                &safety,
                "safety database schema does not match its journal",
            ));
        }
        drop(checked);
        Ok(Self {
            root,
            journal: stored.journal,
            seal: stored.sha256,
        })
    }

    pub(crate) fn safety_path(&self, dirs: &DataDirs) -> PathBuf {
        dirs.pre_restore_backups().join(&self.journal.safety_file)
    }

    pub(crate) fn files_added(&self) -> u32 {
        self.journal.entries.len() as u32
    }
    pub(crate) fn files_set_aside(&self) -> u32 {
        self.journal
            .entries
            .iter()
            .filter(|e| e.original.is_some())
            .count() as u32
    }

    pub(crate) fn install_files(&self, dirs: &DataDirs) -> AppResult<()> {
        for (index, entry) in self.journal.entries.iter().enumerate() {
            let target = dirs.root().join(&entry.target);
            ensure_directory(target.parent().expect("safe source path"))?;
            let current = regular(&target)?.map(|_| stamp(&target)).transpose()?;
            if current != entry.original {
                return Err(fault(&target, "source changed after restore was prepared"));
            }
            checkpoint(&format!("file-{index}-before-install"))?;
            if entry.original.is_some() {
                move_new(&target, &self.root.join(&entry.target))?;
                checkpoint(&format!("file-{index}-old-moved"))?;
            }
            move_new(&self.root.join("incoming").join(index.to_string()), &target)?;
            if !matches(&target, &entry.replacement)? {
                return Err(fault(&target, "installed file failed verification"));
            }
            checkpoint(&format!("file-{index}-installed"))?;
        }
        checkpoint("files-installed")
    }

    fn outcome(&self) -> AppResult<Option<bool>> {
        let path = self.root.join(TERMINAL);
        let Some(meta) = regular(&path)? else {
            return Ok(None);
        };
        if meta.len() > 4096 {
            return Err(fault(&path, "oversized restore outcome"));
        }
        let sealed: SealedOutcome =
            serde_json::from_slice(&fs::read(&path).map_err(|e| AppError::filesystem(&path, e))?)
                .map_err(|error| fault(&path, format!("invalid restore outcome: {error}")))?;
        if sealed.sha256 != sha256(&encode(&sealed.outcome)?)
            || sealed.outcome.journal_sha256 != self.seal
        {
            return Err(fault(&path, "restore outcome checksum mismatch"));
        }
        Ok(Some(sealed.outcome.committed))
    }

    pub(crate) fn has_outcome(&self) -> bool {
        // A damaged/unreadable terminal record is also a blocker: do not
        // silently begin rollback after a commit that may already be durable.
        !matches!(self.outcome(), Ok(None))
    }

    pub(crate) fn mark_done(&self, committed: bool) -> AppResult<()> {
        let outcome = Outcome {
            journal_sha256: self.seal.clone(),
            committed,
        };
        let hash = sha256(&encode(&outcome)?);
        let path = self
            .root
            .join(format!("outcome-{}.tmp", crate::db::new_id()));
        write_new(
            &path,
            &encode(&SealedOutcome {
                outcome,
                sha256: hash,
            })?,
        )?;
        move_new(&path, &self.root.join(TERMINAL))?;
        checkpoint(if committed {
            "committed"
        } else {
            "rolled-back"
        })
    }

    pub(crate) fn finish(&self, dirs: &DataDirs) -> AppResult<()> {
        if self.outcome()?.is_none() {
            return Err(fault(&self.root, "restore has no durable outcome"));
        }
        // Retain originals, failed replacement bytes and the journal. Only
        // completed transaction directories leave the active recovery slot.
        move_new(
            &self.root,
            &dirs
                .pre_restore_backups()
                .join(format!("files-{}", self.journal.id)),
        )
    }

    pub(crate) fn undo_files(&self, dirs: &DataDirs) -> AppResult<()> {
        for (index, entry) in self.journal.entries.iter().enumerate().rev() {
            let target = dirs.root().join(&entry.target);
            let original = self.root.join(&entry.target);
            let current = regular(&target)?.map(|_| stamp(&target)).transpose()?;
            if current == entry.original {
                continue;
            }
            if current.as_ref().is_some_and(|s| s != &entry.replacement) {
                return Err(fault(
                    &target,
                    "unexpected source content; retained without overwriting",
                ));
            }
            if let Some(expected) = &entry.original {
                if !matches(&original, expected)? {
                    return Err(fault(
                        &original,
                        "original source is unavailable or damaged",
                    ));
                }
            }
            if current.is_some() {
                // Do not delete even a known replacement; retain it so a
                // failed/interruptible rollback never discards either copy.
                let aside = self
                    .root
                    .join("discarded")
                    .join(format!("{index}-{}", crate::db::new_id()));
                move_new(&target, &aside)?;
                checkpoint(&format!("undo-{index}-replacement-moved"))?;
            }
            if let Some(expected) = &entry.original {
                ensure_directory(target.parent().expect("safe source path"))?;
                let copy = self
                    .root
                    .join("undo")
                    .join(format!("{index}-{}", crate::db::new_id()));
                copy_checked(&original, &copy, expected)?;
                move_new(&copy, &target)?;
                if !matches(&target, expected)? {
                    return Err(fault(&target, "restored original failed verification"));
                }
                checkpoint(&format!("undo-{index}-original-restored"))?;
            }
        }
        checkpoint("files-rolled-back")
    }

    fn undo_database_before_open(&self, dirs: &DataDirs) -> AppResult<()> {
        let path = dirs.database_file();
        let paths = [
            path.clone(),
            PathBuf::from(format!("{}-wal", path.display())),
            PathBuf::from(format!("{}-shm", path.display())),
        ];
        // Refuse all unsafe entries before moving any of the SQLite triplet.
        for p in &paths {
            regular(p)?;
        }
        for (index, p) in paths.iter().enumerate() {
            if regular(p)?.is_some() {
                let aside = self
                    .root
                    .join("discarded")
                    .join(format!("database-{index}-{}", crate::db::new_id()));
                move_new(p, &aside)?;
                checkpoint(&format!("database-{index}-moved"))?;
            }
        }
        let copy = self
            .root
            .join("undo")
            .join(format!("database-{}", crate::db::new_id()));
        copy_checked(&self.safety_path(dirs), &copy, &self.journal.safety)?;
        move_new(&copy, &path)?;
        checkpoint("database-rolled-back")
    }
}

/// The restore's final database state must be durable before a commit/rollback
/// marker is published. NORMAL (the usual editing policy) alone is not enough.
pub(crate) fn flush_database(conn: &Connection, path: Option<&Path>) -> AppResult<()> {
    conn.execute_batch("PRAGMA synchronous=FULL;")?;
    let (busy, _, _): (i64, i64, i64) =
        conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?;
    if busy != 0 {
        return Err(AppError::Unexpected(
            "SQLite checkpoint is busy during restore".into(),
        ));
    }
    if let Some(path) = path {
        sync_file(path)?;
        sync_directory(path.parent().expect("database directory"))?;
    }
    Ok(())
}

/// Called before Database::open; on failure AppState keeps the core unavailable,
/// preventing autosaves, migrations, snapshots, editing and exports.
pub(crate) fn recover_before_open(dirs: &DataDirs) -> AppResult<()> {
    if !pending(dirs) {
        return Ok(());
    }
    let recovery = (|| {
        let transaction = Transaction::load(dirs)?;
        if transaction.outcome()?.is_none() {
            transaction.undo_files(dirs)?;
            transaction.undo_database_before_open(dirs)?;
            transaction.mark_done(false)?;
        }
        transaction.finish(dirs)
    })();
    recovery.map_err(|error| {
        log::error!(
            "full-backup recovery blocked: {error}; retained at {}",
            dirs.pre_restore_backups().display()
        );
        AppError::Import(recovery_message(dirs))
    })
}

/// Test-only deterministic faults/real process termination. Production builds
/// neither read this environment variable nor contain a process-exit hook.
pub(crate) fn checkpoint(_point: &str) -> AppResult<()> {
    #[cfg(test)]
    if let Ok(wanted) = std::env::var("AW_TEST_FULL_RESTORE_FAULT") {
        if wanted == format!("crash:{_point}") {
            std::process::exit(86);
        }
        if wanted == format!("fail:{_point}") {
            return Err(AppError::filesystem(
                Path::new(_point),
                io::Error::other("injected restore I/O failure"),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "restore_journal_tests.rs"]
mod tests;
