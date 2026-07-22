use super::domain::{
    AddCatalogBookInput, BookCatalogSearchResponse, BookLibraryDetail, BookLibraryFilter,
    BookLibraryItem, BookLinkInput, BookProviderStatus, BookSearchInput, BookSeriesOverrideInput,
    CreateManualBookInput, ManualBookEditionInput, SetBookProgressInput,
    UpdateBookReaderStateInput,
};
use super::service::{
    book_data_source_url, external_open_error, BookCatalogService, BookLibraryService,
    BookProgressService,
};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn search_book_catalog(
    input: BookSearchInput,
    state: State<'_, AppState>,
) -> Result<BookCatalogSearchResponse, String> {
    BookCatalogService::new(&state.database)
        .search(&input)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn add_catalog_book_to_library(
    input: AddCatalogBookInput,
    state: State<'_, AppState>,
) -> Result<BookLibraryDetail, String> {
    BookCatalogService::new(&state.database)
        .add(&input)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_manual_book_entry(
    input: CreateManualBookInput,
    state: State<'_, AppState>,
) -> Result<BookLibraryDetail, String> {
    BookLibraryService::new(&state.database)
        .create_manual(&input)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_book_library(
    filter: BookLibraryFilter,
    state: State<'_, AppState>,
) -> Result<Vec<BookLibraryItem>, String> {
    BookLibraryService::new(&state.database)
        .list(&filter)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_book_library_detail(
    entry_id: i64,
    state: State<'_, AppState>,
) -> Result<BookLibraryDetail, String> {
    BookLibraryService::new(&state.database)
        .get(entry_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn refresh_book_metadata(
    entry_id: i64,
    state: State<'_, AppState>,
) -> Result<BookLibraryDetail, String> {
    BookCatalogService::new(&state.database)
        .refresh(entry_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_book_reader_state(
    entry_id: i64,
    input: UpdateBookReaderStateInput,
    state: State<'_, AppState>,
) -> Result<BookLibraryDetail, String> {
    BookLibraryService::new(&state.database)
        .update_reader_state(entry_id, &input)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_book_preferred_edition(
    entry_id: i64,
    edition_id: i64,
    state: State<'_, AppState>,
) -> Result<BookLibraryDetail, String> {
    BookLibraryService::new(&state.database)
        .set_preferred_edition(entry_id, edition_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_book_progress(
    entry_id: i64,
    input: SetBookProgressInput,
    state: State<'_, AppState>,
) -> Result<BookLibraryDetail, String> {
    BookProgressService::new(&state.database)
        .set_progress(entry_id, &input)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn mark_book_read(
    entry_id: i64,
    state: State<'_, AppState>,
) -> Result<BookLibraryDetail, String> {
    BookProgressService::new(&state.database)
        .mark_read(entry_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn reset_book_progress(
    entry_id: i64,
    state: State<'_, AppState>,
) -> Result<BookLibraryDetail, String> {
    BookProgressService::new(&state.database)
        .reset(entry_id)
        .map_err(|error| error.to_string())
}

macro_rules! entry_command {
    ($name:ident, $method:ident) => {
        #[tauri::command]
        pub fn $name(
            entry_id: i64,
            state: State<'_, AppState>,
        ) -> Result<BookLibraryDetail, String> {
            BookLibraryService::new(&state.database)
                .$method(entry_id)
                .map_err(|error| error.to_string())
        }
    };
}

entry_command!(add_book_to_read_next, add_to_read_next);
entry_command!(remove_book_from_read_next, remove_from_read_next);

#[tauri::command]
pub fn move_book_read_next_item(
    entry_id: i64,
    direction: String,
    state: State<'_, AppState>,
) -> Result<BookLibraryDetail, String> {
    BookLibraryService::new(&state.database)
        .move_read_next(entry_id, &direction)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_manual_book_edition(
    entry_id: i64,
    input: ManualBookEditionInput,
    state: State<'_, AppState>,
) -> Result<BookLibraryDetail, String> {
    BookLibraryService::new(&state.database)
        .create_edition(entry_id, &input)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_manual_book_edition(
    entry_id: i64,
    edition_id: i64,
    input: ManualBookEditionInput,
    state: State<'_, AppState>,
) -> Result<BookLibraryDetail, String> {
    BookLibraryService::new(&state.database)
        .update_edition(entry_id, edition_id, &input)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_manual_book_edition(
    entry_id: i64,
    edition_id: i64,
    state: State<'_, AppState>,
) -> Result<BookLibraryDetail, String> {
    BookLibraryService::new(&state.database)
        .delete_edition(entry_id, edition_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_book_link(
    entry_id: i64,
    input: BookLinkInput,
    state: State<'_, AppState>,
) -> Result<BookLibraryDetail, String> {
    BookLibraryService::new(&state.database)
        .create_link(entry_id, &input)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_book_link(
    entry_id: i64,
    link_id: i64,
    input: BookLinkInput,
    state: State<'_, AppState>,
) -> Result<BookLibraryDetail, String> {
    BookLibraryService::new(&state.database)
        .update_link(entry_id, link_id, &input)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_book_link(
    entry_id: i64,
    link_id: i64,
    state: State<'_, AppState>,
) -> Result<BookLibraryDetail, String> {
    BookLibraryService::new(&state.database)
        .delete_link(entry_id, link_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_preferred_book_link(
    entry_id: i64,
    link_id: i64,
    state: State<'_, AppState>,
) -> Result<BookLibraryDetail, String> {
    BookLibraryService::new(&state.database)
        .set_preferred_link(entry_id, link_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn open_book_link(
    entry_id: i64,
    link_id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let url = BookLibraryService::new(&state.database)
        .resolve_link(entry_id, link_id)
        .map_err(|error| error.to_string())?;
    crate::commands::open_external_url(&url).map_err(external_open_error)
}

#[tauri::command]
pub fn set_book_series_override(
    entry_id: i64,
    input: BookSeriesOverrideInput,
    state: State<'_, AppState>,
) -> Result<BookLibraryDetail, String> {
    BookLibraryService::new(&state.database)
        .set_series_override(entry_id, &input)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn clear_book_series_override(
    entry_id: i64,
    member_id: i64,
    state: State<'_, AppState>,
) -> Result<BookLibraryDetail, String> {
    BookLibraryService::new(&state.database)
        .clear_series_override(entry_id, member_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_book_provider_status(state: State<'_, AppState>) -> Result<BookProviderStatus, String> {
    Ok(BookCatalogService::new(&state.database).provider_status())
}

#[tauri::command]
pub fn open_book_data_source(source_key: String) -> Result<(), String> {
    let url = book_data_source_url(&source_key).map_err(|error| error.to_string())?;
    crate::commands::open_external_url(url).map_err(external_open_error)
}
