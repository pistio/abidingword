//! Looking after sources and Bibles (v0.24): archive, Trash, restore and
//! removal for good; the Sources list in Settings; metadata; collections
//! and tags. Files are deleted only inside Abiding Word's own folders, and
//! only once the database has committed: a failure leaves a recoverable
//! item (still in Trash), never a database row without its file.

use tauri::State;

use super::CommandResult;
use crate::db::library_manage::{
    self, Collection, ItemState, ManagePage, ManageQuery, Tag, UpdateSource,
};
use crate::db::{bibles, now};
use crate::filesystem;
use crate::library::Source;
use crate::scripture::BibleTranslation;
use crate::state::{AppState, Core};

/// Deletes a removed source's own folder (`sources/<category>/<id>/`).
fn delete_source_folder(core: &Core, id: &str, relative: Option<&str>) {
    let Some(relative) = relative else {
        return;
    };
    // Only plain names: a stored path with "..", a root or a drive is refused.
    let plain = std::path::Path::new(relative)
        .components()
        .all(|c| matches!(c, std::path::Component::Normal(_)));
    if !plain {
        log::warn!("a stored file path was not a plain path; nothing was deleted");
        return;
    }
    if let Some(folder) = core.dirs.sources().join(relative).parent() {
        // Only ever the source's own folder inside sources/.
        if folder.starts_with(core.dirs.sources()) && folder.ends_with(id) {
            filesystem::remove_managed(folder);
        }
    }
}

fn delete_bible_folder(core: &Core, removed: &library_manage::RemovedBible) {
    if removed.managed_copy {
        filesystem::remove_managed(&core.dirs.bible_sources(&removed.id));
    }
}

/// Removes for good what has been in Trash longer than the retention period
/// (run when the app starts). Failures are logged; nothing is lost by
/// trying again next time.
pub(crate) fn empty_expired_trash(core: &Core) {
    let expired = core
        .db
        .with_conn(|conn| library_manage::expired(conn, &now()));
    let (sources, bibles) = match expired {
        Ok(found) => found,
        Err(error) => {
            log::warn!("Trash could not be checked: {error}");
            return;
        }
    };
    // One item at a time: one that cannot be removed never holds back the rest.
    for id in sources {
        match core
            .db
            .with_conn(|conn| library_manage::remove_sources(conn, std::slice::from_ref(&id)))
        {
            Ok(files) => {
                for (id, file) in files {
                    delete_source_folder(core, &id, file.as_deref());
                }
            }
            Err(error) => log::warn!("an expired source could not be removed: {error}"),
        }
    }
    for id in bibles {
        match core
            .db
            .with_conn(|conn| library_manage::remove_bibles(conn, std::slice::from_ref(&id)))
        {
            Ok(removed) => removed.iter().for_each(|b| delete_bible_folder(core, b)),
            Err(error) => log::warn!("an expired Bible could not be removed: {error}"),
        }
    }
    sweep_leftovers(core);
}

/// Folders left behind when deleting a file failed, or the app stopped
/// between copying a file and saving its record, or between the database
/// commit and the file deletion. Only folders directly inside Abiding Word's
/// own category folders are looked at:
///
/// - a source this database removed for good (its tombstone is there): the
///   folder is deleted, as the removal meant to;
/// - a source or Bible this database does not know at all (a crash before its
///   record was saved — or a backup restored from before it was added, whose
///   safety copy still refers to it): the folder is **moved** to
///   `leftovers/` in the data folder, never deleted, so undoing a restore
///   loses nothing.
pub(crate) fn sweep_leftovers(core: &Core) {
    let root = core.dirs.sources();
    let Ok(categories) = std::fs::read_dir(&root) else {
        return;
    };
    for category in categories.flatten() {
        let Ok(name) = category.file_name().into_string() else {
            continue;
        };
        let bibles = name == "bibles";
        if !(bibles || ["books", "notes", "documents"].contains(&name.as_str())) {
            continue;
        }
        let Ok(folders) = std::fs::read_dir(category.path()) else {
            continue;
        };
        for folder in folders.flatten() {
            let Ok(id) = folder.file_name().into_string() else {
                continue;
            };
            if !folder.path().is_dir() {
                continue;
            }
            // (known at all, live)
            let sql = if bibles {
                "SELECT COUNT(*) > 0, COUNT(*) > 0 FROM bible_translations WHERE id = ?1"
            } else {
                "SELECT COUNT(*) > 0, COALESCE(MAX(deleted_at IS NULL), 0) FROM sources WHERE id = ?1"
            };
            let found = core.db.with_conn(|conn| {
                Ok(conn.query_row(sql, [&id], |r| {
                    Ok((r.get::<_, bool>(0)?, r.get::<_, bool>(1)?))
                })?)
            });
            match found {
                Ok((true, false)) => {
                    log::info!("removing a leftover folder of a removed source");
                    filesystem::remove_managed(&folder.path());
                }
                Ok((false, _)) => {
                    let target = core.dirs.root().join("leftovers").join(&name).join(&id);
                    let moved = target
                        .parent()
                        .map(std::fs::create_dir_all)
                        .transpose()
                        .and_then(|_| std::fs::rename(folder.path(), &target));
                    match moved {
                        Ok(()) => {
                            log::info!("moved a folder this library does not know to leftovers/")
                        }
                        Err(error) => {
                            log::warn!("an unknown library folder could not be moved: {error}")
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

// --- sources ---------------------------------------------------------------------

#[tauri::command(async)]
pub fn library_manage_list(
    state: State<'_, AppState>,
    query: ManageQuery,
) -> CommandResult<ManagePage> {
    state.db(|conn| library_manage::list(conn, &query))
}

#[tauri::command(async)]
pub fn library_set_state(
    state: State<'_, AppState>,
    ids: Vec<String>,
    to: ItemState,
) -> CommandResult<usize> {
    state.db(|conn| library_manage::set_source_state(conn, &ids, to))
}

/// Deletes trashed sources permanently: their text and Abiding Word's copy
/// of their files. Citations keep their record of what they cited.
#[tauri::command(async)]
pub fn library_remove_permanently(
    state: State<'_, AppState>,
    ids: Vec<String>,
) -> CommandResult<usize> {
    let core = state.core()?;
    let files = core
        .db
        .with_conn(|conn| library_manage::remove_sources(conn, &ids))?;
    for (id, file) in &files {
        delete_source_folder(core, id, file.as_deref());
    }
    Ok(files.len())
}

#[tauri::command(async)]
pub fn library_update_source(
    state: State<'_, AppState>,
    input: UpdateSource,
) -> CommandResult<Source> {
    state.db(|conn| library_manage::update_source(conn, &input))
}

#[tauri::command(async)]
pub fn library_collections(state: State<'_, AppState>) -> CommandResult<Vec<Collection>> {
    state.db(library_manage::collections)
}

#[tauri::command(async)]
pub fn library_collection_create(
    state: State<'_, AppState>,
    name: String,
    parent_id: Option<String>,
) -> CommandResult<Collection> {
    state.db(|conn| library_manage::create_collection(conn, &name, parent_id.as_deref()))
}

#[tauri::command(async)]
pub fn library_collection_rename(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> CommandResult<()> {
    state.db(|conn| library_manage::rename_collection(conn, &id, &name))
}

#[tauri::command(async)]
pub fn library_collection_move(
    state: State<'_, AppState>,
    id: String,
    parent_id: Option<String>,
) -> CommandResult<()> {
    state.db(|conn| library_manage::move_collection(conn, &id, parent_id.as_deref()))
}

#[tauri::command(async)]
pub fn library_collection_remove(state: State<'_, AppState>, id: String) -> CommandResult<()> {
    state.db(|conn| library_manage::remove_collection(conn, &id))
}

#[tauri::command(async)]
pub fn library_collect(
    state: State<'_, AppState>,
    collection_id: String,
    ids: Vec<String>,
    add: bool,
) -> CommandResult<usize> {
    state.db(|conn| library_manage::collect(conn, &collection_id, &ids, add))
}

#[tauri::command(async)]
pub fn library_tags(state: State<'_, AppState>) -> CommandResult<Vec<Tag>> {
    state.db(library_manage::tags)
}

#[tauri::command(async)]
pub fn library_tag_rename(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> CommandResult<()> {
    state.db(|conn| library_manage::rename_tag(conn, &id, &name))
}

#[tauri::command(async)]
pub fn library_tag_remove(state: State<'_, AppState>, id: String) -> CommandResult<()> {
    state.db(|conn| library_manage::remove_tag(conn, &id))
}

#[tauri::command(async)]
pub fn library_retag(
    state: State<'_, AppState>,
    ids: Vec<String>,
    add: Vec<String>,
    remove: Vec<String>,
) -> CommandResult<()> {
    state.db(|conn| library_manage::retag(conn, &ids, &add, &remove))
}

// --- Bibles ----------------------------------------------------------------------

/// Every Bible, archived and trashed ones included (Settings).
#[tauri::command(async)]
pub fn bible_list_all(state: State<'_, AppState>) -> CommandResult<Vec<BibleTranslation>> {
    state.db(bibles::list_all)
}

#[tauri::command(async)]
pub fn bible_set_state(
    state: State<'_, AppState>,
    ids: Vec<String>,
    to: ItemState,
) -> CommandResult<usize> {
    state.db(|conn| library_manage::set_bible_state(conn, &ids, to))
}

/// Deletes trashed Bibles permanently with Abiding Word's copy of their
/// files. Study passages, notes and highlights stay with their verses.
#[tauri::command(async)]
pub fn bible_remove_permanently(
    state: State<'_, AppState>,
    ids: Vec<String>,
) -> CommandResult<usize> {
    let core = state.core()?;
    let removed = core
        .db
        .with_conn(|conn| library_manage::remove_bibles(conn, &ids))?;
    removed.iter().for_each(|b| delete_bible_folder(core, b));
    Ok(removed.len())
}
