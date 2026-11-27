mod domain;
pub(crate) mod repository;
mod seeds;
#[cfg(test)]
mod tests;

use crate::{profiles::ProfileService, AppState};
use domain::{Project, ProjectDraft, ProjectQuery};
use repository::ProjectRepository;
use tauri::State;

#[tauri::command]
pub fn projects_list(
    state: State<'_, AppState>,
    session_id: String,
    query: ProjectQuery,
) -> Result<Vec<Project>, String> {
    let _profile = ProfileService::require_admin(&state, &session_id)?;
    ProjectRepository::new(&state.database).list(&query)
}

#[tauri::command]
pub fn projects_save(
    state: State<'_, AppState>,
    session_id: String,
    id: Option<String>,
    draft: ProjectDraft,
) -> Result<String, String> {
    let _profile = ProfileService::require_admin(&state, &session_id)?;
    ProjectRepository::new(&state.database).save(id.as_deref(), draft)
}

#[tauri::command]
pub fn projects_archive(
    state: State<'_, AppState>,
    session_id: String,
    id: String,
    archived: bool,
) -> Result<(), String> {
    let _profile = ProfileService::require_admin(&state, &session_id)?;
    ProjectRepository::new(&state.database).set_archived(&id, archived)
}

#[tauri::command]
pub fn projects_delete(
    state: State<'_, AppState>,
    session_id: String,
    id: String,
    confirmation: String,
) -> Result<(), String> {
    let _profile = ProfileService::require_admin(&state, &session_id)?;
    ProjectRepository::new(&state.database).delete(&id, &confirmation)
}

#[tauri::command]
pub fn projects_open_web_link(
    state: State<'_, AppState>,
    session_id: String,
    target: String,
) -> Result<(), String> {
    let _profile = ProfileService::require_admin(&state, &session_id)?;
    let url = reqwest::Url::parse(&target).map_err(|_| "Invalid HTTPS link.")?;
    if target.len() > 2048
        || url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err("Only HTTPS web links can be opened. Workspace and document paths remain text references.".into());
    }
    crate::browser_auth::open_system_browser(url.as_str())
}
