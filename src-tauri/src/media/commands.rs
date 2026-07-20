use super::domain::{
    AddCatalogMediaInput, CreateManualMediaInput, MediaCatalogSearchResult, MediaError,
    MediaErrorKind, MediaLibraryDetail, MediaLibraryFilter, MediaLibrarySummary,
    MediaSettingsRecord, MediaStreamingLinkInput, MediaStreamingLinkRecord, MediaTagRecord,
    UpdateMediaEntryInput, UpdateMediaSettingsInput,
};
use super::service::{
    MediaAvailabilityService, MediaCatalogService, MediaLibraryService, MediaProgressService,
};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn search_media_catalog(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<MediaCatalogSearchResult>, String> {
    MediaCatalogService::new(&state.database)
        .search(&query)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn add_catalog_media_to_library(
    input: AddCatalogMediaInput,
    state: State<'_, AppState>,
) -> Result<MediaLibraryDetail, String> {
    MediaCatalogService::new(&state.database)
        .add_catalog_title(&input)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_manual_media_entry(
    input: CreateManualMediaInput,
    state: State<'_, AppState>,
) -> Result<MediaLibraryDetail, String> {
    MediaLibraryService::new(&state.database)
        .create_manual(&input)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_media_library(
    filter: MediaLibraryFilter,
    state: State<'_, AppState>,
) -> Result<Vec<MediaLibrarySummary>, String> {
    MediaLibraryService::new(&state.database)
        .list(&filter)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_media_library_detail(
    entry_id: i64,
    state: State<'_, AppState>,
) -> Result<MediaLibraryDetail, String> {
    MediaLibraryService::new(&state.database)
        .get(entry_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_media_library_entry(
    entry_id: i64,
    input: UpdateMediaEntryInput,
    state: State<'_, AppState>,
) -> Result<MediaLibraryDetail, String> {
    MediaLibraryService::new(&state.database)
        .update(entry_id, &input)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_media_library_entry(entry_id: i64, state: State<'_, AppState>) -> Result<(), String> {
    MediaLibraryService::new(&state.database)
        .delete(entry_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn refresh_media_metadata(
    entry_id: i64,
    state: State<'_, AppState>,
) -> Result<MediaLibraryDetail, String> {
    MediaCatalogService::new(&state.database)
        .refresh(entry_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_movie_watched(
    entry_id: i64,
    is_watched: bool,
    watched_at: String,
    state: State<'_, AppState>,
) -> Result<MediaLibraryDetail, String> {
    MediaProgressService::new(&state.database)
        .set_movie_watched(entry_id, is_watched, &watched_at)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_episode_watched(
    entry_id: i64,
    episode_id: i64,
    is_watched: bool,
    watched_at: String,
    state: State<'_, AppState>,
) -> Result<MediaLibraryDetail, String> {
    MediaProgressService::new(&state.database)
        .set_episode_watched(entry_id, episode_id, is_watched, &watched_at)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_season_watched(
    entry_id: i64,
    season_number: i64,
    is_watched: bool,
    state: State<'_, AppState>,
) -> Result<MediaLibraryDetail, String> {
    MediaProgressService::new(&state.database)
        .set_season_watched(entry_id, season_number, is_watched)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_series_watched(
    entry_id: i64,
    is_watched: bool,
    state: State<'_, AppState>,
) -> Result<MediaLibraryDetail, String> {
    MediaProgressService::new(&state.database)
        .set_series_watched(entry_id, is_watched)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn mark_episodes_watched_through(
    entry_id: i64,
    episode_id: i64,
    state: State<'_, AppState>,
) -> Result<MediaLibraryDetail, String> {
    MediaProgressService::new(&state.database)
        .mark_watched_through(entry_id, episode_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn add_media_to_watch_next(
    entry_id: i64,
    state: State<'_, AppState>,
) -> Result<MediaLibraryDetail, String> {
    MediaLibraryService::new(&state.database)
        .add_to_watch_next(entry_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn remove_media_from_watch_next(
    entry_id: i64,
    state: State<'_, AppState>,
) -> Result<MediaLibraryDetail, String> {
    MediaLibraryService::new(&state.database)
        .remove_from_watch_next(entry_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn move_media_watch_next_item(
    entry_id: i64,
    direction: String,
    state: State<'_, AppState>,
) -> Result<MediaLibraryDetail, String> {
    MediaLibraryService::new(&state.database)
        .move_watch_next(entry_id, &direction)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_media_tags(state: State<'_, AppState>) -> Result<Vec<MediaTagRecord>, String> {
    MediaLibraryService::new(&state.database)
        .list_tags()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_media_tag(
    name: String,
    state: State<'_, AppState>,
) -> Result<MediaTagRecord, String> {
    MediaLibraryService::new(&state.database)
        .create_tag(&name)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_media_tag(tag_id: i64, state: State<'_, AppState>) -> Result<(), String> {
    MediaLibraryService::new(&state.database)
        .delete_tag(tag_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_media_entry_tags(
    entry_id: i64,
    tag_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<MediaLibraryDetail, String> {
    MediaLibraryService::new(&state.database)
        .set_entry_tags(entry_id, &tag_ids)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_media_streaming_links(
    entry_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<MediaStreamingLinkRecord>, String> {
    MediaAvailabilityService::new(&state.database)
        .list_links(entry_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_media_streaming_link(
    entry_id: i64,
    input: MediaStreamingLinkInput,
    state: State<'_, AppState>,
) -> Result<MediaStreamingLinkRecord, String> {
    MediaAvailabilityService::new(&state.database)
        .create_link(entry_id, &input)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_media_streaming_link(
    link_id: i64,
    input: MediaStreamingLinkInput,
    state: State<'_, AppState>,
) -> Result<MediaStreamingLinkRecord, String> {
    MediaAvailabilityService::new(&state.database)
        .update_link(link_id, &input)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_media_streaming_link(link_id: i64, state: State<'_, AppState>) -> Result<(), String> {
    MediaAvailabilityService::new(&state.database)
        .delete_link(link_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_preferred_media_streaming_link(
    entry_id: i64,
    link_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<MediaStreamingLinkRecord>, String> {
    MediaAvailabilityService::new(&state.database)
        .set_preferred(entry_id, link_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn open_media_streaming_target(
    entry_id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let url = MediaAvailabilityService::new(&state.database)
        .resolve_open_target(entry_id)
        .map_err(|error| error.to_string())?;
    crate::commands::open_external_url(&url)
        .map_err(|error| MediaError::new(MediaErrorKind::ExternalOpen, error).to_string())
}

#[tauri::command]
pub fn get_media_settings(state: State<'_, AppState>) -> Result<MediaSettingsRecord, String> {
    MediaLibraryService::new(&state.database)
        .get_settings()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_media_settings(
    input: UpdateMediaSettingsInput,
    state: State<'_, AppState>,
) -> Result<MediaSettingsRecord, String> {
    MediaLibraryService::new(&state.database)
        .update_settings(&input)
        .map_err(|error| error.to_string())
}
