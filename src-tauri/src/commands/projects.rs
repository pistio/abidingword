use tauri::State;

use super::CommandResult;
use crate::db::projects;
use crate::domain::{CreateProject, Project, UpdateProject};
use crate::state::AppState;

#[tauri::command(async)]
pub fn project_list(
    state: State<'_, AppState>,
    workspace_id: String,
) -> CommandResult<Vec<Project>> {
    state.db(|conn| projects::list(conn, &workspace_id))
}

#[tauri::command(async)]
pub fn project_get(state: State<'_, AppState>, id: String) -> CommandResult<Project> {
    state.db(|conn| projects::get(conn, &id))
}

#[tauri::command(async)]
pub fn project_create(state: State<'_, AppState>, input: CreateProject) -> CommandResult<Project> {
    state.db(|conn| projects::create(conn, &input))
}

#[tauri::command(async)]
pub fn project_update(state: State<'_, AppState>, input: UpdateProject) -> CommandResult<Project> {
    state.db(|conn| projects::update(conn, &input))
}

#[tauri::command(async)]
pub fn project_delete(
    state: State<'_, AppState>,
    id: String,
    permanent: Option<bool>,
) -> CommandResult<()> {
    state.db(|c| projects::delete(c, &id, permanent.unwrap_or(false)))
}
#[tauri::command(async)]
pub fn project_restore(state: State<'_, AppState>, id: String) -> CommandResult<Project> {
    state.db(|c| projects::restore(c, &id))
}
