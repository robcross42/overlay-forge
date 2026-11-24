//! Native boundary for host presentation preferences; SQLite mapping stays in HostDatabase.
use crate::{profiles::ProfileService, AppState};
use tauri::State;

#[tauri::command]
pub fn navigation_preference(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<bool, String> {
    let _profile = ProfileService::require_signed_in(&state, Some(&session_id))?;
    state
        .database
        .navigation_collapsed()
        .map_err(|_| "Could not restore navigation preference.".into())
}

#[tauri::command]
pub fn save_navigation_preference(
    state: State<'_, AppState>,
    session_id: String,
    collapsed: bool,
) -> Result<(), String> {
    let _profile = ProfileService::require_signed_in(&state, Some(&session_id))?;
    state
        .database
        .save_navigation_collapsed(collapsed)
        .map_err(|_| "Could not save navigation preference. Please try again.".into())
}
