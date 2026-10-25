//! Adding many sources at once (v0.24): several files, or a folder.
//!
//! One job, kept in the database (`db::import_jobs`): the scan lists the
//! files; each step reads and checks **one** file (so memory stays bounded,
//! and the interface can show progress and stop between steps); the person
//! reviews and corrects the list; each import step adds **one** chosen file
//! the same way a single file is added (`library::add_staged`). Cancelling
//! stops between files; files already added stay. A job interrupted by a
//! crash is offered again, and a file added just before the crash is
//! recognised by its content and not added twice.

use std::path::{Path, PathBuf};

use tauri::State;

use super::library::add_staged;
use super::CommandResult;
use crate::db::import_jobs::{self, Analysis, ItemEdit, Job, JobItem, Problem};
use crate::db::{library, library_manage};
use crate::error::{AppError, AppResult, UserMessage};
use crate::library::{adapters, batch, import, CommitSourceImport, LicenceStatus, SourceType};
use crate::state::{AppState, Core};

/// Lists the readable files among the chosen files and folders, as a new job.
#[tauri::command(async)]
pub fn library_batch_scan(
    state: State<'_, AppState>,
    paths: Vec<String>,
    recursive: bool,
) -> CommandResult<Job> {
    let core = state.core()?;
    Ok(scan(core, &paths, recursive)?)
}

pub fn scan(core: &Core, paths: &[String], recursive: bool) -> AppResult<Job> {
    let paths: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
    let found = batch::scan(&paths, recursive);
    core.db.with_conn(|conn| {
        import_jobs::create(conn, &found.files, found.unsupported, found.over_limit)
    })
}

/// Reads and checks the next waiting file. Returns the job; nothing left to
/// read when `counts.waiting` is 0.
#[tauri::command(async)]
pub fn library_batch_step(state: State<'_, AppState>, job_id: String) -> CommandResult<Job> {
    let core = state.core()?;
    Ok(read_next(core, &job_id)?)
}

/// One step of any batch at a time: two overlapping steps (a double click,
/// two windows) would otherwise pick the same file.
static STEP: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn one_step() -> std::sync::MutexGuard<'static, ()> {
    STEP.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn read_next(core: &Core, job_id: &str) -> AppResult<Job> {
    let _step = one_step();
    let Some(item) = core
        .db
        .with_conn(|conn| import_jobs::next_waiting(conn, job_id))?
    else {
        return core.db.with_conn(|conn| import_jobs::get(conn, job_id));
    };
    let path = PathBuf::from(&item.path);
    match import::stage(&path) {
        Ok(staged) => {
            let analysis = analyse(core, &staged, &path)?;
            match staged.problem() {
                Some(problem) => core.db.with_conn(|conn| {
                    import_jobs::record_problem(conn, job_id, &item.id, false, &(&problem).into())
                })?,
                None => core.db.with_conn(|conn| {
                    import_jobs::record_analysis(conn, job_id, &item.id, &analysis)
                })?,
            }
        }
        Err(error) => {
            let message = problem_of(error);
            core.db.with_conn(|conn| {
                import_jobs::record_problem(conn, job_id, &item.id, false, &message)
            })?;
        }
    }
    core.db.with_conn(|conn| import_jobs::get(conn, job_id))
}

fn analyse(core: &Core, staged: &import::StagedSource, path: &Path) -> AppResult<Analysis> {
    let doc = &staged.document;
    let stem = staged
        .file_name
        .rsplit_once('.')
        .map(|(stem, _)| stem.to_string())
        .unwrap_or_else(|| staged.file_name.clone());
    let title = doc
        .metadata
        .title
        .clone()
        .filter(|t| !t.trim().is_empty())
        .unwrap_or(stem);
    let (kind, evidence) = batch::propose_type(doc.format, &title, path);
    let duplicate = core
        .db
        .with_conn(|conn| library::find_duplicate(conn, &staged.hash))?
        .map(|d| (d.id, d.title));
    Ok(Analysis {
        sha256: staged.hash.clone(),
        format: doc.format.to_string(),
        title,
        author: doc.metadata.author.clone(),
        language: doc
            .metadata
            .language
            .as_deref()
            .map(adapters::language_name),
        source_type: kind.as_str().to_string(),
        type_evidence: evidence.to_string(),
        page_count: doc.page_count.map(i64::from),
        duplicate_of: duplicate,
    })
}

fn problem_of(error: AppError) -> Problem {
    Problem::from(crate::error::ErrorPayload::from(error))
}

#[tauri::command(async)]
pub fn library_batch_get(state: State<'_, AppState>, job_id: String) -> CommandResult<Job> {
    state.db(|conn| import_jobs::get(conn, &job_id))
}

/// Jobs still open (in review, adding, or interrupted).
#[tauri::command(async)]
pub fn library_batch_jobs(state: State<'_, AppState>) -> CommandResult<Vec<Job>> {
    state.db(import_jobs::open_jobs)
}

#[tauri::command(async)]
pub fn library_batch_edit(
    state: State<'_, AppState>,
    job_id: String,
    edit: ItemEdit,
) -> CommandResult<JobItem> {
    state.db(|conn| import_jobs::edit_item(conn, &job_id, &edit))
}

/// The collection (an existing one, or a new one by name) and tags every
/// added source gets.
#[tauri::command(async)]
pub fn library_batch_shared(
    state: State<'_, AppState>,
    job_id: String,
    collection_id: Option<String>,
    new_collection: Option<String>,
    tags: Vec<String>,
) -> CommandResult<Job> {
    state.db(|conn| {
        let collection = match new_collection.as_deref().map(str::trim) {
            Some(name) if !name.is_empty() => {
                Some(library_manage::create_collection(conn, name, collection_id.as_deref())?.id)
            }
            _ => collection_id.clone(),
        };
        import_jobs::set_shared(conn, &job_id, collection.as_deref(), &tags)?;
        import_jobs::get(conn, &job_id)
    })
}

#[tauri::command(async)]
pub fn library_batch_retry(
    state: State<'_, AppState>,
    job_id: String,
    item_ids: Vec<String>,
) -> CommandResult<Job> {
    state.db(|conn| {
        import_jobs::retry(conn, &job_id, &item_ids)?;
        import_jobs::get(conn, &job_id)
    })
}

/// Adds the next chosen file. When none is left the job is finished.
#[tauri::command(async)]
pub fn library_batch_import_step(state: State<'_, AppState>, job_id: String) -> CommandResult<Job> {
    let core = state.core()?;
    Ok(import_next(core, &job_id)?)
}

pub fn import_next(core: &Core, job_id: &str) -> AppResult<Job> {
    let _step = one_step();
    let job = core.db.with_conn(|conn| import_jobs::get(conn, job_id))?;
    if matches!(job.state.as_str(), "finished" | "cancelled") {
        return Ok(job);
    }
    if job.counts.waiting > 0 {
        return Err(AppError::InvalidInput(UserMessage::new(
            "batch.notRead",
            "Some files have not been read yet.",
        )));
    }
    let Some(item) = core
        .db
        .with_conn(|conn| import_jobs::next_to_add(conn, job_id))?
    else {
        core.db
            .with_conn(|conn| import_jobs::set_state(conn, job_id, "finished"))?;
        return core.db.with_conn(|conn| import_jobs::get(conn, job_id));
    };
    if job.state != "importing" {
        core.db
            .with_conn(|conn| import_jobs::set_state(conn, job_id, "importing"))?;
    }
    if let Err(message) = add_one(core, &job, &item) {
        core.db.with_conn(|conn| {
            import_jobs::record_problem(conn, job_id, &item.id, true, &message)
        })?;
    }
    core.db.with_conn(|conn| import_jobs::get(conn, job_id))
}

fn add_one(core: &Core, job: &Job, item: &JobItem) -> Result<(), Problem> {
    let fail = problem_of;
    // Added just before an interruption, before the item was marked.
    if let Some(sha) = &item.sha256 {
        if let Some(existing) = core
            .db
            .with_conn(|conn| import_jobs::added_since(conn, job, sha, &item.file_name))
            .map_err(fail)?
        {
            return finish(core, job, item, &existing).map_err(fail);
        }
    }
    let staged = import::stage(Path::new(&item.path)).map_err(fail)?;
    // The file must still be the one reviewed.
    if item.sha256.as_deref() != Some(staged.hash.as_str()) {
        return Err((&UserMessage::new(
            "batch.changed",
            format!(
                "“{}” changed after it was checked. Read it again.",
                item.file_name
            ),
        )
        .with("fileName", &item.file_name))
            .into());
    }
    let input = CommitSourceImport {
        preview_id: staged.id.clone(),
        title: item.title.clone().unwrap_or_else(|| item.file_name.clone()),
        author: item.author.clone(),
        language: item.language.clone(),
        source_type: item
            .source_type
            .as_deref()
            .and_then(|k| SourceType::parse(k).ok()),
        licence_status: Some(LicenceStatus::Unknown),
        allow_duplicate: item.status == "duplicate",
        ..CommitSourceImport::default()
    };
    let source = add_staged(core, &staged, &input).map_err(Problem::from)?;
    finish(core, job, item, &source.id).map_err(fail)
}

/// Marks the item added, then gives the source the job's collection and
/// tags. The source is in the library from the moment it was saved: if the
/// collection or tags cannot be given (checked when they were chosen, so only
/// if something changed meanwhile), that is logged, never reported as the
/// file not being added.
fn finish(core: &Core, job: &Job, item: &JobItem, source_id: &str) -> AppResult<()> {
    core.db
        .with_conn(|conn| import_jobs::record_imported(conn, &job.id, &item.id, source_id))?;
    // Each of these is one transaction of its own.
    let organised = core.db.with_conn(|conn| {
        let ids = [source_id.to_string()];
        if let Some(collection) = &job.collection_id {
            library_manage::collect(conn, collection, &ids, true)?;
        }
        if !job.tags.is_empty() {
            library_manage::retag(conn, &ids, &job.tags, &[])?;
        }
        Ok(())
    });
    if let Err(error) = organised {
        log::warn!("an added source could not be given the import's collection or tags: {error}");
    }
    Ok(())
}

/// Stops the job: nothing more is added; what was added stays.
#[tauri::command(async)]
pub fn library_batch_cancel(state: State<'_, AppState>, job_id: String) -> CommandResult<Job> {
    state.db(|conn| {
        import_jobs::set_state(conn, &job_id, "cancelled")?;
        import_jobs::get(conn, &job_id)
    })
}

/// Forgets a job (its added sources stay in the library).
#[tauri::command(async)]
pub fn library_batch_discard(state: State<'_, AppState>, job_id: String) -> CommandResult<()> {
    state.db(|conn| import_jobs::discard(conn, &job_id))
}
