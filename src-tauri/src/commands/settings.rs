use tauri::State;

use super::CommandResult;
use crate::db::{reader_positions, settings};
use crate::domain::{AppSettings, UpdateSettings};
use crate::state::AppState;

#[tauri::command(async)]
pub fn settings_get(state: State<'_, AppState>) -> CommandResult<AppSettings> {
    state.db(settings::get)
}

#[tauri::command(async)]
pub fn settings_update(
    state: State<'_, AppState>,
    input: UpdateSettings,
) -> CommandResult<AppSettings> {
    state.db(|conn| settings::update(conn, &input))
}

/// Saved library reading places, included in ordinary database backups.
#[tauri::command(async)]
pub fn source_reader_positions(
    state: State<'_, AppState>,
) -> CommandResult<Vec<reader_positions::SavedReaderPosition>> {
    state.db(reader_positions::list)
}

#[tauri::command(async)]
pub fn source_reader_position_save(
    state: State<'_, AppState>,
    input: reader_positions::SavedReaderPosition,
) -> CommandResult<()> {
    state.db(|conn| reader_positions::save(conn, &input.source_id, &input.position))
}
