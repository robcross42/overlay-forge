use crate::db::{
    BridgeFileDraftRecord, CalendarEventRecord, GameCatalogObjectRecord,
    GameChatConversationRecord, GameChatMessageRecord, GameConstructionRecord,
    GameDataLocationRecord, GameRecord, GameRuntimeConstructionExportRecord, GameRuntimePartRecord,
    GameScreenshotCaptureRequestRecord, NoteRecord, PlanningConversationContextRecord,
    PlanningConversationRecord, PlanningMessageRecord, PlanningPromptPreviewRecord,
    ProjectGitHubRepositoryRecord, ProjectMarkdownContextPayload, ProjectMarkdownContextRecord,
    ProjectRecord, TaskRecord, YouTubeReferenceRecord,
};
use crate::github;
use crate::hotkeys;
use crate::openai;
use crate::{AppState, GameChatOverlaySelection};
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use bson::{Bson, Document};
use flate2::read::DeflateDecoder;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, State, WebviewWindow};

static MANUAL_OVERLAY_DRAG_ACTIVE: AtomicBool = AtomicBool::new(false);

#[derive(Serialize)]
pub struct MilestoneStatus {
    milestone: String,
    hotkey: String,
    #[serde(rename = "databaseReady")]
    database_ready: bool,
}

#[derive(Serialize)]
pub struct ApiKeyStatus {
    #[serde(rename = "isConfigured")]
    pub is_configured: bool,
    pub source: String,
}

#[derive(Serialize)]
pub struct KeybindRecord {
    pub action: String,
    pub label: String,
    pub keys: Vec<String>,
}

#[derive(Serialize)]
pub struct GamePartCategoryRecord {
    pub name: String,
    #[serde(rename = "fallbackIcon")]
    pub fallback_icon: String,
    #[serde(rename = "iconPath")]
    pub icon_path: String,
    pub count: i64,
}

#[derive(Serialize)]
pub struct GearBlocksConstructionFileRecord {
    pub name: String,
    #[serde(rename = "folderPath")]
    pub folder_path: String,
    #[serde(rename = "constructionPath")]
    pub construction_path: String,
    #[serde(rename = "byteSize")]
    pub byte_size: u64,
}

#[derive(Serialize)]
pub struct GearBlocksConstructionDecodeRecord {
    pub name: String,
    #[serde(rename = "folderPath")]
    pub folder_path: String,
    #[serde(rename = "constructionPath")]
    pub construction_path: String,
    #[serde(rename = "byteSize")]
    pub byte_size: u64,
    #[serde(rename = "decodedByteSize")]
    pub decoded_byte_size: usize,
    pub summary: GearBlocksConstructionSummaryRecord,
    pub document: serde_json::Value,
}

#[derive(Serialize)]
pub struct GearBlocksLuaExporterInstallRecord {
    #[serde(rename = "scriptModPath")]
    pub script_mod_path: String,
    #[serde(rename = "mainLuaPath")]
    pub main_lua_path: String,
    #[serde(rename = "exportDirectory")]
    pub export_directory: String,
}

#[derive(Serialize)]
pub struct GearBlocksRuntimeExportRecord {
    pub id: String,
    pub name: String,
    #[serde(rename = "intendedPath")]
    pub intended_path: String,
    #[serde(rename = "sourceLogPath")]
    pub source_log_path: String,
    #[serde(rename = "byteSize")]
    pub byte_size: usize,
    pub document: serde_json::Value,
}

#[derive(Serialize)]
pub struct GearBlocksRuntimeContextSyncRecord {
    pub changed: bool,
    #[serde(rename = "runtimeExportCount")]
    pub runtime_export_count: usize,
    #[serde(rename = "runtimePartCount")]
    pub runtime_part_count: usize,
    #[serde(rename = "constructionCount")]
    pub construction_count: usize,
    #[serde(rename = "runtimeExports")]
    pub runtime_exports: Vec<GameRuntimeConstructionExportRecord>,
    #[serde(rename = "runtimeParts")]
    pub runtime_parts: Vec<GameRuntimePartRecord>,
    pub constructions: Vec<GameConstructionRecord>,
}

#[derive(Serialize)]
pub struct GearBlocksConstructionSummaryRecord {
    #[serde(rename = "isFrozen")]
    pub is_frozen: Option<bool>,
    #[serde(rename = "isInvulnerable")]
    pub is_invulnerable: Option<bool>,
    #[serde(rename = "compositeCount")]
    pub composite_count: usize,
    #[serde(rename = "partCount")]
    pub part_count: usize,
    #[serde(rename = "uniqueAssetGuidCount")]
    pub unique_asset_guid_count: usize,
    #[serde(rename = "attachmentCount")]
    pub attachment_count: usize,
    #[serde(rename = "linkCount")]
    pub link_count: usize,
    #[serde(rename = "intersectionCount")]
    pub intersection_count: usize,
    pub parts: Vec<GearBlocksConstructionPartSummaryRecord>,
}

#[derive(Serialize)]
pub struct GearBlocksConstructionPartSummaryRecord {
    pub index: usize,
    #[serde(rename = "compositeIndex")]
    pub composite_index: usize,
    #[serde(rename = "compositePartIndex")]
    pub composite_part_index: usize,
    #[serde(rename = "assetGuid")]
    pub asset_guid: String,
    pub dimensions: Vec<f64>,
    pub behaviours: Vec<String>,
}

fn to_keybind_records(keybinds: Vec<hotkeys::KeybindConfig>) -> Vec<KeybindRecord> {
    keybinds
        .into_iter()
        .map(|keybind| KeybindRecord {
            action: keybind.action,
            label: keybind.label,
            keys: keybind.keys,
        })
        .collect()
}

#[tauri::command]
pub fn get_scratchpad(state: State<'_, AppState>) -> Result<String, String> {
    state
        .database
        .get_scratchpad()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_openai_api_key_status(state: State<'_, AppState>) -> Result<ApiKeyStatus, String> {
    let saved_key = state
        .database
        .get_app_setting("openai_api_key")
        .map_err(|error| error.to_string())?
        .unwrap_or_default();
    if !saved_key.trim().is_empty() {
        return Ok(ApiKeyStatus {
            is_configured: true,
            source: "local plaintext settings".to_string(),
        });
    }

    let env_key = std::env::var("OPENAI_API_KEY")
        .map(|value| value.trim().to_string())
        .unwrap_or_default();
    Ok(ApiKeyStatus {
        is_configured: !env_key.is_empty(),
        source: if env_key.is_empty() {
            "not configured".to_string()
        } else {
            "OPENAI_API_KEY environment variable".to_string()
        },
    })
}

#[tauri::command]
pub fn save_openai_api_key(
    api_key: String,
    state: State<'_, AppState>,
) -> Result<ApiKeyStatus, String> {
    require_text(&api_key, "OpenAI API key")?;
    state
        .database
        .save_app_setting("openai_api_key", api_key.trim())
        .map_err(|error| error.to_string())?;
    Ok(ApiKeyStatus {
        is_configured: true,
        source: "local plaintext settings".to_string(),
    })
}

#[tauri::command]
pub fn clear_openai_api_key(state: State<'_, AppState>) -> Result<ApiKeyStatus, String> {
    state
        .database
        .delete_app_setting("openai_api_key")
        .map_err(|error| error.to_string())?;
    get_openai_api_key_status(state)
}

#[tauri::command]
pub fn consume_pending_shortcut_action(
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let mut pending = state
        .pending_shortcut_action
        .lock()
        .map_err(|_| "Shortcut action state is unavailable.".to_string())?;
    Ok(pending.take())
}

#[tauri::command]
pub fn list_keybinds(app: AppHandle) -> Result<Vec<KeybindRecord>, String> {
    hotkeys::load_keybinds(&app).map(to_keybind_records)
}

#[tauri::command]
pub fn save_keybinds(
    app: AppHandle,
    keybinds: Vec<hotkeys::KeybindConfig>,
) -> Result<Vec<KeybindRecord>, String> {
    hotkeys::save_keybinds(&app, keybinds).map(to_keybind_records)
}

#[tauri::command]
pub fn reset_keybinds(app: AppHandle) -> Result<Vec<KeybindRecord>, String> {
    hotkeys::reset_keybinds(&app).map(to_keybind_records)
}

#[tauri::command]
pub fn save_scratchpad(content: String, state: State<'_, AppState>) -> Result<(), String> {
    state
        .database
        .save_scratchpad(&content)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_milestone_status(state: State<'_, AppState>) -> Result<MilestoneStatus, String> {
    Ok(MilestoneStatus {
        milestone: "Milestone 13".to_string(),
        hotkey: "Ctrl+Shift+Space / Ctrl+Shift+C".to_string(),
        database_ready: state.database.is_ready(),
    })
}

#[tauri::command]
pub fn shutdown_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
pub fn start_manual_overlay_drag(window: WebviewWindow) -> Result<(), String> {
    manual_overlay_drag(window)
}

#[tauri::command]
pub fn set_overlay_window_opacity(window: WebviewWindow, opacity: f64) -> Result<(), String> {
    set_overlay_opacity(&window, opacity)
}

#[tauri::command]
pub fn focus_last_game_window(state: State<'_, AppState>) -> Result<bool, String> {
    focus_last_game_window_impl(state)
}

#[tauri::command]
pub fn open_game_chat_overlay_window(
    game_id: i64,
    conversation_id: i64,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<GameChatOverlaySelection, String> {
    let game = state
        .database
        .get_game(game_id)
        .map_err(|error| error.to_string())?;
    let conversation = state
        .database
        .get_game_chat_conversation(conversation_id)
        .map_err(|error| error.to_string())?;
    if conversation.game_id != game.id {
        return Err("Selected chat does not belong to the selected game.".to_string());
    }

    let selection = GameChatOverlaySelection {
        game_id: game.id,
        conversation_id: conversation.id,
    };
    state
        .active_game_chat_overlay
        .lock()
        .map_err(|_| "Game chat overlay state is unavailable.".to_string())?
        .replace(selection.clone());

    let app_for_window = app.clone();
    let selection_for_window = selection.clone();
    app.run_on_main_thread(move || {
        if let Err(error) = show_game_chat_overlay_window(&app_for_window, &selection_for_window) {
            eprintln!("Could not open game chat overlay window: {error}");
        }
    })
    .map_err(|error| error.to_string())?;

    Ok(selection)
}

fn show_game_chat_overlay_window(
    app: &AppHandle,
    selection: &GameChatOverlaySelection,
) -> Result<(), String> {
    let window = app
        .get_webview_window("game-chat")
        .ok_or_else(|| "Game chat overlay window was not created at startup.".to_string())?;

    window
        .set_always_on_top(true)
        .map_err(|error| error.to_string())?;
    ensure_window_accepts_mouse_input(&window)?;
    window.show().map_err(|error| error.to_string())?;
    let _ = set_overlay_opacity(&window, 0.78);
    window.set_focus().map_err(|error| error.to_string())?;
    let _ = app.emit("game-chat-overlay-selection-changed", selection.clone());
    let _ = app.emit("game-chat-overlay-focus-prompt", ());

    Ok(())
}

#[tauri::command]
pub fn focus_game_chat_overlay_window(app: AppHandle) -> Result<bool, String> {
    if app.get_webview_window("game-chat").is_none() {
        return Ok(false);
    }

    let app_for_window = app.clone();
    app.run_on_main_thread(move || {
        if let Some(window) = app_for_window.get_webview_window("game-chat") {
            let _ = ensure_window_accepts_mouse_input(&window);
            let _ = window.show();
            let _ = window.set_always_on_top(true);
            let _ = set_overlay_opacity(&window, 0.78);
            let _ = window.set_focus();
            let _ = app_for_window.emit("game-chat-overlay-focus-prompt", ());
        }
    })
    .map_err(|error| error.to_string())?;
    Ok(true)
}

#[tauri::command]
pub fn get_active_game_chat_overlay(
    state: State<'_, AppState>,
) -> Result<Option<GameChatOverlaySelection>, String> {
    state
        .active_game_chat_overlay
        .lock()
        .map_err(|_| "Game chat overlay state is unavailable.".to_string())
        .map(|selection| selection.clone())
}

#[tauri::command]
pub fn list_tasks(state: State<'_, AppState>) -> Result<Vec<TaskRecord>, String> {
    state
        .database
        .list_tasks()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_task(
    title: String,
    body: String,
    deadline: String,
    state: State<'_, AppState>,
) -> Result<TaskRecord, String> {
    require_text(&title, "Task title")?;
    state
        .database
        .create_task(&title, &body, &deadline)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_task(
    id: i64,
    title: Option<String>,
    body: Option<String>,
    deadline: Option<String>,
    is_completed: Option<bool>,
    state: State<'_, AppState>,
) -> Result<TaskRecord, String> {
    if let Some(next_title) = title.as_ref() {
        require_text(next_title, "Task title")?;
    }

    state
        .database
        .update_task(
            id,
            title.as_deref(),
            body.as_deref(),
            deadline.as_deref(),
            is_completed,
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_task(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    state
        .database
        .delete_task(id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_notes(state: State<'_, AppState>) -> Result<Vec<NoteRecord>, String> {
    state
        .database
        .list_notes()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_note(
    title: String,
    body: String,
    state: State<'_, AppState>,
) -> Result<NoteRecord, String> {
    require_text(&title, "Note title")?;
    state
        .database
        .create_note(&title, &body)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_note(
    id: i64,
    title: String,
    body: String,
    state: State<'_, AppState>,
) -> Result<NoteRecord, String> {
    require_text(&title, "Note title")?;
    state
        .database
        .update_note(id, &title, &body)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_note(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    state
        .database
        .delete_note(id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_calendar_events(
    state: State<'_, AppState>,
) -> Result<Vec<CalendarEventRecord>, String> {
    state
        .database
        .list_calendar_events()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_calendar_event(
    title: String,
    start_date: String,
    start_time: String,
    end_date: String,
    end_time: String,
    notes: String,
    state: State<'_, AppState>,
) -> Result<CalendarEventRecord, String> {
    validate_calendar_event(&title, &start_date, &start_time, &end_date, &end_time)?;
    state
        .database
        .create_calendar_event(
            &title,
            &start_date,
            &start_time,
            &end_date,
            &end_time,
            &notes,
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_calendar_event(
    id: i64,
    title: String,
    start_date: String,
    start_time: String,
    end_date: String,
    end_time: String,
    notes: String,
    state: State<'_, AppState>,
) -> Result<CalendarEventRecord, String> {
    validate_calendar_event(&title, &start_date, &start_time, &end_date, &end_time)?;
    state
        .database
        .update_calendar_event(
            id,
            &title,
            &start_date,
            &start_time,
            &end_date,
            &end_time,
            &notes,
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_calendar_event(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    state
        .database
        .delete_calendar_event(id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_projects(state: State<'_, AppState>) -> Result<Vec<ProjectRecord>, String> {
    state
        .database
        .list_projects()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_project(
    name: String,
    description: String,
    status: String,
    state: State<'_, AppState>,
) -> Result<ProjectRecord, String> {
    validate_project(&name, &status)?;
    state
        .database
        .create_project(&name, &description, &status)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_project(
    id: i64,
    name: String,
    description: String,
    status: String,
    state: State<'_, AppState>,
) -> Result<ProjectRecord, String> {
    validate_project(&name, &status)?;
    state
        .database
        .update_project(id, &name, &description, &status)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_project(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    state
        .database
        .delete_project(id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_project_github_repository(
    project_id: i64,
    state: State<'_, AppState>,
) -> Result<Option<ProjectGitHubRepositoryRecord>, String> {
    state
        .database
        .get_project_github_repository(project_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn save_project_github_repository(
    project_id: i64,
    repository_full_name: String,
    state: State<'_, AppState>,
) -> Result<ProjectGitHubRepositoryRecord, String> {
    let normalized = github::normalize_repository_full_name(&repository_full_name)?;
    state
        .database
        .save_project_github_repository(project_id, &normalized)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_project_github_repository(
    project_id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .database
        .delete_project_github_repository(project_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn fetch_project_github_metadata(
    project_id: i64,
    state: State<'_, AppState>,
) -> Result<ProjectGitHubRepositoryRecord, String> {
    let link = state
        .database
        .get_project_github_repository(project_id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "Link a GitHub repository before fetching metadata.".to_string())?;

    match github::fetch_repository_metadata(&link.repository_full_name).await {
        Ok(metadata) => state
            .database
            .update_project_github_metadata(
                project_id,
                &metadata.repository_full_name,
                &metadata.repository_url,
                &metadata.default_branch,
                &metadata.visibility,
                "Fetched GitHub repository metadata successfully",
            )
            .map_err(|error| error.to_string()),
        Err(error) => {
            let _ = state
                .database
                .update_project_github_fetch_status(project_id, &error);
            Err(error)
        }
    }
}

#[tauri::command]
pub fn get_project_markdown_context(
    project_id: i64,
    state: State<'_, AppState>,
) -> Result<Option<ProjectMarkdownContextRecord>, String> {
    state
        .database
        .get_project_markdown_context(project_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn save_project_markdown_context(
    project_id: i64,
    root_path: String,
    readme_path: String,
    state: State<'_, AppState>,
) -> Result<ProjectMarkdownContextRecord, String> {
    require_text(&root_path, "Markdown context root")?;
    state
        .database
        .save_project_markdown_context(project_id, &root_path, &readme_path)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_project_markdown_context(
    project_id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .database
        .delete_project_markdown_context(project_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn load_project_markdown_context(
    project_id: i64,
    state: State<'_, AppState>,
) -> Result<ProjectMarkdownContextPayload, String> {
    state
        .database
        .load_project_markdown_context(project_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_planning_conversations(
    project_id: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<PlanningConversationRecord>, String> {
    state
        .database
        .list_planning_conversations(project_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_planning_conversation(
    project_id: i64,
    title: Option<String>,
    state: State<'_, AppState>,
) -> Result<PlanningConversationRecord, String> {
    state
        .database
        .create_planning_conversation(project_id, title.as_deref().unwrap_or_default())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_planning_messages(
    conversation_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<PlanningMessageRecord>, String> {
    state
        .database
        .list_planning_messages(conversation_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn send_planning_message(
    conversation_id: i64,
    content: String,
    state: State<'_, AppState>,
) -> Result<Vec<PlanningMessageRecord>, String> {
    require_text(&content, "Message")?;
    let conversation = state
        .database
        .get_planning_conversation(conversation_id)
        .map_err(|error| error.to_string())?;
    let project = state
        .database
        .get_project(conversation.project_id)
        .map_err(|error| error.to_string())?;

    state
        .database
        .create_planning_message(conversation_id, "user", &content)
        .map_err(|error| error.to_string())?;

    let recent_messages = state
        .database
        .recent_planning_messages(conversation_id, 20)
        .map_err(|error| error.to_string())?;
    let context_payload = state
        .database
        .planning_conversation_context_payload(conversation_id)
        .map_err(|error| error.to_string())?;
    let api_key = configured_openai_api_key(&state)?;
    let assistant_content = openai::create_planning_response(
        &api_key,
        &project,
        &recent_messages,
        &context_payload.content,
    )
    .await?;

    state
        .database
        .create_planning_message(conversation_id, "assistant", &assistant_content)
        .map_err(|error| error.to_string())?;

    state
        .database
        .list_planning_messages(conversation_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_planning_conversation(
    conversation_id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .database
        .delete_planning_conversation(conversation_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_planning_conversation_context(
    conversation_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<PlanningConversationContextRecord>, String> {
    state
        .database
        .list_planning_conversation_context(conversation_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn attach_planning_conversation_context(
    conversation_id: i64,
    context_type: String,
    source_id: Option<i64>,
    label: String,
    state: State<'_, AppState>,
) -> Result<PlanningConversationContextRecord, String> {
    validate_context_type(&context_type)?;
    require_text(&label, "Context label")?;
    state
        .database
        .attach_planning_conversation_context(conversation_id, &context_type, source_id, &label)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn remove_planning_conversation_context(
    id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .database
        .remove_planning_conversation_context(id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn preview_planning_chat_prompt(
    conversation_id: i64,
    draft_message: String,
    state: State<'_, AppState>,
) -> Result<PlanningPromptPreviewRecord, String> {
    state
        .database
        .preview_planning_chat_prompt(
            conversation_id,
            &draft_message,
            openai::PLANNING_SYSTEM_INSTRUCTION,
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_bridge_file_drafts(
    project_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<BridgeFileDraftRecord>, String> {
    state
        .database
        .list_bridge_file_drafts(project_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_bridge_file_draft(
    id: i64,
    state: State<'_, AppState>,
) -> Result<BridgeFileDraftRecord, String> {
    state
        .database
        .get_bridge_file_draft(id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_bridge_file_draft_from_conversation(
    conversation_id: i64,
    state: State<'_, AppState>,
) -> Result<BridgeFileDraftRecord, String> {
    state
        .database
        .create_bridge_file_draft_from_conversation(conversation_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_bridge_file_draft(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    state
        .database
        .delete_bridge_file_draft(id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_youtube_references(
    state: State<'_, AppState>,
) -> Result<Vec<YouTubeReferenceRecord>, String> {
    state
        .database
        .list_youtube_references()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_youtube_reference(
    id: i64,
    state: State<'_, AppState>,
) -> Result<YouTubeReferenceRecord, String> {
    state
        .database
        .get_youtube_reference(id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_youtube_reference(
    title: String,
    url: String,
    channel_name: String,
    notes: String,
    tags: String,
    state: State<'_, AppState>,
) -> Result<YouTubeReferenceRecord, String> {
    require_text(&title, "YouTube reference title")?;
    let video_id = extract_youtube_video_id(&url)?;
    state
        .database
        .create_youtube_reference(&title, &url, &video_id, &channel_name, &notes, &tags)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_youtube_reference(
    id: i64,
    title: String,
    url: String,
    channel_name: String,
    notes: String,
    tags: String,
    state: State<'_, AppState>,
) -> Result<YouTubeReferenceRecord, String> {
    require_text(&title, "YouTube reference title")?;
    let video_id = extract_youtube_video_id(&url)?;
    state
        .database
        .update_youtube_reference(id, &title, &url, &video_id, &channel_name, &notes, &tags)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_youtube_reference(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    state
        .database
        .delete_youtube_reference(id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn open_youtube_reference(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let reference = state
        .database
        .get_youtube_reference(id)
        .map_err(|error| error.to_string())?;
    open_external_url(&reference.url)
}

#[tauri::command]
pub fn list_games(state: State<'_, AppState>) -> Result<Vec<GameRecord>, String> {
    state
        .database
        .list_games()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_game(
    name: String,
    summary: String,
    state: State<'_, AppState>,
) -> Result<GameRecord, String> {
    require_text(&name, "Game name")?;
    state
        .database
        .create_game(&name, &summary)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_game(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    state
        .database
        .delete_game(id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_game_data_locations(
    game_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<GameDataLocationRecord>, String> {
    let game = state
        .database
        .get_game(game_id)
        .map_err(|error| error.to_string())?;
    if !game_exposes_data_locations(&game) {
        return Ok(Vec::new());
    }

    state
        .database
        .list_game_data_locations(game_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn save_game_data_location(
    game_id: i64,
    location_type: String,
    directory_path: String,
    state: State<'_, AppState>,
) -> Result<GameDataLocationRecord, String> {
    let game = state
        .database
        .get_game(game_id)
        .map_err(|error| error.to_string())?;
    require_game_data_locations_enabled(&game)?;
    let normalized_type = normalize_game_location_type(&location_type)?;
    let canonical_path = validate_directory_path(&directory_path)?;

    state
        .database
        .save_game_data_location(
            game.id,
            normalized_type,
            game_location_type_label(normalized_type),
            &canonical_path,
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_game_data_location(
    game_id: i64,
    location_type: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let game = state
        .database
        .get_game(game_id)
        .map_err(|error| error.to_string())?;
    require_game_data_locations_enabled(&game)?;
    let normalized_type = normalize_game_location_type(&location_type)?;

    state
        .database
        .delete_game_data_location(game.id, normalized_type)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_gearblocks_construction_files(
    game_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<GearBlocksConstructionFileRecord>, String> {
    let game = state
        .database
        .get_game(game_id)
        .map_err(|error| error.to_string())?;
    require_gearblocks_game(&game)?;
    let root = gearblocks_saved_constructions_root(&state, game.id)?;
    list_gearblocks_construction_files_in_root(&root)
}

#[tauri::command]
pub fn list_game_constructions(
    game_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<GameConstructionRecord>, String> {
    let game = state
        .database
        .get_game(game_id)
        .map_err(|error| error.to_string())?;
    require_gearblocks_game(&game)?;
    state
        .database
        .list_game_constructions(game.id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn sync_gearblocks_saved_constructions(
    game_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<GameConstructionRecord>, String> {
    let game = state
        .database
        .get_game(game_id)
        .map_err(|error| error.to_string())?;
    require_gearblocks_game(&game)?;
    sync_gearblocks_saved_constructions_for_game(&state, game.id)
}

#[tauri::command]
pub fn sync_gearblocks_runtime_context(
    game_id: i64,
    state: State<'_, AppState>,
) -> Result<GearBlocksRuntimeContextSyncRecord, String> {
    let game = state
        .database
        .get_game(game_id)
        .map_err(|error| error.to_string())?;
    require_gearblocks_game(&game)?;

    let fingerprint = gearblocks_runtime_context_fingerprint(&state, game.id)?;
    let fingerprint_key = format!("gearblocks.runtime_context_fingerprint.{}", game.id);
    let previous_fingerprint = state
        .database
        .get_app_setting(&fingerprint_key)
        .map_err(|error| error.to_string())?;
    let changed = previous_fingerprint.as_deref() != Some(fingerprint.as_str());
    if changed {
        import_latest_gearblocks_runtime_exports(&state, game.id)?;
        sync_gearblocks_saved_constructions_for_game(&state, game.id)?;
        state
            .database
            .save_app_setting(&fingerprint_key, &fingerprint)
            .map_err(|error| error.to_string())?;
    }

    let constructions = state
        .database
        .list_game_constructions(game.id)
        .map_err(|error| error.to_string())?;
    let runtime_exports = state
        .database
        .list_game_runtime_construction_exports(game.id)
        .map_err(|error| error.to_string())?;
    let runtime_parts = state
        .database
        .list_game_runtime_parts(game.id)
        .map_err(|error| error.to_string())?;
    let runtime_parts = gearblocks_runtime_parts_all_in_catalog_order(runtime_parts);

    Ok(GearBlocksRuntimeContextSyncRecord {
        changed,
        runtime_export_count: runtime_exports.len(),
        runtime_part_count: runtime_parts.len(),
        construction_count: constructions.len(),
        runtime_exports,
        runtime_parts,
        constructions,
    })
}

fn sync_gearblocks_saved_constructions_for_game(
    state: &AppState,
    game_id: i64,
) -> Result<Vec<GameConstructionRecord>, String> {
    let root = gearblocks_saved_constructions_root(state, game_id)?;
    let files = list_gearblocks_construction_files_in_root(&root)?;
    let indexed_at = unix_timestamp_label();
    let mut records = Vec::new();

    for file in files {
        let decoded = decode_gearblocks_construction_path(Path::new(&file.construction_path))?;
        let summary_json =
            serde_json::to_string_pretty(&decoded.summary).map_err(|error| error.to_string())?;
        let document_json =
            serde_json::to_string_pretty(&decoded.document).map_err(|error| error.to_string())?;
        let record = state
            .database
            .upsert_game_construction(
                game_id,
                &decoded.name,
                &decoded.folder_path,
                &decoded.construction_path,
                decoded.byte_size as i64,
                decoded.decoded_byte_size as i64,
                decoded.summary.composite_count as i64,
                decoded.summary.part_count as i64,
                decoded.summary.unique_asset_guid_count as i64,
                decoded.summary.attachment_count as i64,
                decoded.summary.link_count as i64,
                decoded.summary.intersection_count as i64,
                decoded.summary.is_frozen,
                decoded.summary.is_invulnerable,
                &summary_json,
                &document_json,
                &indexed_at,
            )
            .map_err(|error| error.to_string())?;
        records.push(record);
    }

    records.sort_by(|left, right| {
        left.name
            .to_ascii_lowercase()
            .cmp(&right.name.to_ascii_lowercase())
    });
    Ok(records)
}

fn gearblocks_runtime_context_fingerprint(
    state: &AppState,
    game_id: i64,
) -> Result<String, String> {
    let root = gearblocks_default_user_data_root()?;
    let mut entries = Vec::new();

    for log_path in [root.join("Player-prev.log"), root.join("Player.log")] {
        entries.push(file_fingerprint_entry("log", &log_path));
    }

    let constructions_root = gearblocks_saved_constructions_root(state, game_id)?;
    for file in list_gearblocks_construction_files_in_root(&constructions_root)? {
        entries.push(file_fingerprint_entry(
            "construction",
            Path::new(&file.construction_path),
        ));
    }

    entries.sort();
    Ok(entries.join("|"))
}

fn file_fingerprint_entry(kind: &str, path: &Path) -> String {
    let normalized_path = path.to_string_lossy();
    match fs::metadata(path) {
        Ok(metadata) => {
            let modified = metadata
                .modified()
                .ok()
                .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
                .map(|value| format!("{}.{}", value.as_secs(), value.subsec_nanos()))
                .unwrap_or_else(|| "unknown".to_string());
            format!(
                "{}:{}:{}:{}",
                kind,
                normalized_path,
                metadata.len(),
                modified
            )
        }
        Err(_) => format!("{}:{}:missing", kind, normalized_path),
    }
}

#[tauri::command]
pub fn decode_gearblocks_construction_file(
    construction_path: String,
) -> Result<GearBlocksConstructionDecodeRecord, String> {
    let path = validate_gearblocks_construction_file_path(&construction_path)?;
    decode_gearblocks_construction_path(&path)
}

#[tauri::command]
pub fn decode_gearblocks_construction_folder(
    folder_path: String,
) -> Result<GearBlocksConstructionDecodeRecord, String> {
    require_text(&folder_path, "Construction folder")?;
    let folder = PathBuf::from(folder_path.trim());
    if !folder.is_dir() {
        return Err("Selected construction path is not a folder.".to_string());
    }
    let path = find_construction_file_in_folder(&folder)
        .ok_or_else(|| "Selected folder does not contain construction.bytes.".to_string())?;
    decode_gearblocks_construction_path(&path)
}

#[tauri::command]
pub fn install_gearblocks_lua_exporter(
    game_id: i64,
    state: State<'_, AppState>,
) -> Result<GearBlocksLuaExporterInstallRecord, String> {
    let game = state
        .database
        .get_game(game_id)
        .map_err(|error| error.to_string())?;
    require_gearblocks_game(&game)?;

    let gearblocks_root = gearblocks_default_user_data_root()?;
    let script_mod_path = gearblocks_root
        .join("ScriptMods")
        .join("OverlayForgeConstructionExporter");
    let export_directory = gearblocks_runtime_export_dir(&state, game.id, &gearblocks_root)?;
    fs::create_dir_all(&script_mod_path).map_err(|error| error.to_string())?;
    fs::create_dir_all(&export_directory).map_err(|error| error.to_string())?;

    let main_lua = include_str!(
        "../../gearblocks-script-mods/OverlayForgeConstructionExporter/main.lua.template"
    )
    .replace("{{EXPORT_DIR}}", &lua_long_bracket_path(&export_directory))
    .replace(
        "{{KNOWN_API_INDEX}}",
        &gearblocks_known_api_index_lua(&state, game.id)?,
    );
    let main_lua_path = script_mod_path.join("main.lua");
    fs::write(&main_lua_path, main_lua).map_err(|error| error.to_string())?;
    fs::write(
        script_mod_path.join("meta.json"),
        include_str!("../../gearblocks-script-mods/OverlayForgeConstructionExporter/meta.json"),
    )
    .map_err(|error| error.to_string())?;

    Ok(GearBlocksLuaExporterInstallRecord {
        script_mod_path: script_mod_path.to_string_lossy().to_string(),
        main_lua_path: main_lua_path.to_string_lossy().to_string(),
        export_directory: export_directory.to_string_lossy().to_string(),
    })
}

fn gearblocks_known_api_index_lua(
    state: &State<'_, AppState>,
    game_id: i64,
) -> Result<String, String> {
    let parts = state
        .database
        .list_game_runtime_parts(game_id)
        .map_err(|error| error.to_string())?;
    let mut index = serde_json::Map::new();

    for part in parts {
        let Ok(properties) = serde_json::from_str::<serde_json::Value>(&part.properties_json)
        else {
            continue;
        };
        let mut attributes = Vec::new();
        collect_gearblocks_known_api_attributes(&properties, &mut attributes);
        if attributes.is_empty() {
            continue;
        }

        let attributes_json = serde_json::Value::Array(attributes);
        for key in gearblocks_runtime_part_api_index_keys(&part) {
            index.insert(key, attributes_json.clone());
        }
    }

    Ok(serde_json_to_lua_literal(&serde_json::Value::Object(index)))
}

fn gearblocks_runtime_part_api_index_keys(part: &GameRuntimePartRecord) -> Vec<String> {
    let mut keys = Vec::new();
    if !part.asset_guid.trim().is_empty() && part.asset_guid.trim() != "nil" {
        keys.push(format!("asset-guid:{}", part.asset_guid.trim()));
    }
    if !part.asset_name.trim().is_empty() {
        keys.push(format!(
            "asset-name:{}",
            part.asset_name.trim().to_ascii_lowercase()
        ));
    }
    let display_name = if !part.display_name.trim().is_empty() {
        part.display_name.trim()
    } else if !part.full_display_name.trim().is_empty() {
        part.full_display_name.trim()
    } else {
        ""
    };
    if !display_name.is_empty() {
        keys.push(format!(
            "display:{}:{}",
            part.category.trim().to_ascii_lowercase(),
            display_name.to_ascii_lowercase()
        ));
    }
    keys
}

fn collect_gearblocks_known_api_attributes(
    value: &serde_json::Value,
    attributes: &mut Vec<serde_json::Value>,
) {
    let Some(items) = value
        .get("apiAttributes")
        .and_then(serde_json::Value::as_array)
    else {
        return;
    };

    for item in items {
        let interface = item
            .get("interface")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim();
        let name = item
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim();
        if interface.is_empty() || name.is_empty() {
            continue;
        }
        attributes.push(json!({
            "interface": interface,
            "name": name,
            "valueType": "available",
            "availability": item
                .get("availability")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("known-index")
        }));
    }
}

fn serde_json_to_lua_literal(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "nil".to_string(),
        serde_json::Value::Bool(value) => value.to_string(),
        serde_json::Value::Number(value) => value.to_string(),
        serde_json::Value::String(value) => {
            serde_json::to_string(value).unwrap_or_else(|_| format!("{:?}", value))
        }
        serde_json::Value::Array(items) => {
            let items = items
                .iter()
                .map(serde_json_to_lua_literal)
                .collect::<Vec<_>>()
                .join(",");
            format!("{{{items}}}")
        }
        serde_json::Value::Object(object) => {
            let items = object
                .iter()
                .map(|(key, value)| {
                    let key = serde_json::to_string(key).unwrap_or_else(|_| format!("{key:?}"));
                    format!("[{key}]={}", serde_json_to_lua_literal(value))
                })
                .collect::<Vec<_>>()
                .join(",");
            format!("{{{items}}}")
        }
    }
}

#[tauri::command]
pub fn list_gearblocks_runtime_exports(
    game_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<GearBlocksRuntimeExportRecord>, String> {
    let game = state
        .database
        .get_game(game_id)
        .map_err(|error| error.to_string())?;
    require_gearblocks_game(&game)?;

    let root = gearblocks_default_user_data_root()?;
    let mut exports = Vec::new();
    for log_path in [root.join("Player-prev.log"), root.join("Player.log")] {
        if log_path.is_file() {
            exports.extend(parse_gearblocks_runtime_exports_from_log(&log_path)?);
        }
    }

    Ok(exports)
}

#[tauri::command]
pub fn import_gearblocks_runtime_part_index(
    game_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<GameRuntimePartRecord>, String> {
    let game = state
        .database
        .get_game(game_id)
        .map_err(|error| error.to_string())?;
    require_gearblocks_game(&game)?;

    import_latest_gearblocks_runtime_exports(&state, game.id)?;

    let parts = state
        .database
        .list_game_runtime_parts(game.id)
        .map_err(|error| error.to_string())?;

    Ok(gearblocks_runtime_parts_all_in_catalog_order(parts))
}

#[tauri::command]
pub fn list_game_runtime_parts(
    game_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<GameRuntimePartRecord>, String> {
    let game = state
        .database
        .get_game(game_id)
        .map_err(|error| error.to_string())?;
    let parts = state
        .database
        .list_game_runtime_parts(game_id)
        .map_err(|error| error.to_string())?;

    if game.slug == "gearblocks" {
        Ok(gearblocks_runtime_parts_all_in_catalog_order(parts))
    } else {
        Ok(parts)
    }
}

#[tauri::command]
pub fn update_game_runtime_part_notes(
    game_id: i64,
    part_id: i64,
    notes: String,
    state: State<'_, AppState>,
) -> Result<GameRuntimePartRecord, String> {
    state
        .database
        .update_game_runtime_part_notes(game_id, part_id, &notes)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn clear_game_runtime_part_images_for_category(
    game_id: i64,
    category: String,
    state: State<'_, AppState>,
) -> Result<Vec<GameRuntimePartRecord>, String> {
    let game = state
        .database
        .get_game(game_id)
        .map_err(|error| error.to_string())?;
    require_gearblocks_game(&game)?;

    let category = category.trim();
    if category.is_empty() || category.eq_ignore_ascii_case("all") {
        return Err(
            "Select a specific GearBlocks part category before clearing images.".to_string(),
        );
    }

    let parts = state
        .database
        .clear_game_runtime_part_images_for_category(game.id, category)
        .map_err(|error| error.to_string())?;

    Ok(gearblocks_runtime_parts_in_catalog_order(category, parts))
}

#[tauri::command]
pub fn set_game_runtime_part_display_image(
    game_id: i64,
    part_id: i64,
    image_path: String,
    state: State<'_, AppState>,
) -> Result<GameRuntimePartRecord, String> {
    let game = state
        .database
        .get_game(game_id)
        .map_err(|error| error.to_string())?;
    require_gearblocks_game(&game)?;

    let source_path = PathBuf::from(image_path.trim());
    if !source_path.is_file() {
        return Err("Selected image file was not found.".to_string());
    }

    let extension = source_path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.trim().to_ascii_lowercase())
        .unwrap_or_default();
    if !is_supported_catalog_image_extension(&extension) {
        return Err("Select a PNG, JPG, JPEG, WEBP, or BMP image.".to_string());
    }

    let part = state
        .database
        .get_game_runtime_part(part_id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "Runtime part was not found.".to_string())?;
    if part.game_id != game.id {
        return Err("Runtime part does not belong to the selected game.".to_string());
    }

    let screenshots_root = overlay_workspace_root()?.join("game-screenshots");
    let part_image_dir = screenshots_root.join(&game.slug).join("part-images");
    fs::create_dir_all(&part_image_dir).map_err(|error| error.to_string())?;

    let part_label = if !part.asset_name.trim().is_empty() {
        part.asset_name.as_str()
    } else if !part.display_name.trim().is_empty() {
        part.display_name.as_str()
    } else {
        part.part_key.as_str()
    };
    let file_stem = safe_filename_part(part_label);
    let image_file_name = if file_stem.is_empty() {
        format!("part_{}.{}", part.id, extension)
    } else {
        format!("{}_{}.{}", file_stem, part.id, extension)
    };
    let target_path = part_image_dir.join(image_file_name);
    fs::copy(&source_path, &target_path).map_err(|error| error.to_string())?;

    state
        .database
        .update_game_runtime_part_display_image(
            game.id,
            part.id,
            &path_text(&target_path),
            &path_text(&source_path),
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn import_gearblocks_catalog_screenshot_images(
    game_id: i64,
    category: String,
    image_path: String,
    state: State<'_, AppState>,
) -> Result<Vec<GameRuntimePartRecord>, String> {
    let game = state
        .database
        .get_game(game_id)
        .map_err(|error| error.to_string())?;
    require_gearblocks_game(&game)?;

    let category = category.trim();
    if category.is_empty() || category.eq_ignore_ascii_case("all") {
        return Err(
            "Select a specific GearBlocks part category before importing a catalog screenshot."
                .to_string(),
        );
    }

    let source_path = PathBuf::from(image_path.trim());
    if !source_path.is_file() {
        return Err("Selected catalog screenshot was not found.".to_string());
    }
    if source_path.extension().and_then(|value| value.to_str()) != Some("png") {
        return Err("Catalog screenshot import currently expects a PNG screenshot.".to_string());
    }

    import_latest_gearblocks_runtime_exports(&state, game.id)?;

    let parts = state
        .database
        .list_game_runtime_parts(game.id)
        .map_err(|error| error.to_string())?
        .into_iter()
        .filter(|part| part.category == category)
        .collect::<Vec<_>>();
    let parts = gearblocks_runtime_parts_in_catalog_order(category, parts);
    if parts.is_empty() {
        return Err(format!(
            "No runtime API parts are indexed for the {category} category."
        ));
    }

    let screenshots_root = overlay_workspace_root()?.join("game-screenshots");
    let part_image_dir = screenshots_root.join(&game.slug).join("part-images");
    fs::create_dir_all(&part_image_dir).map_err(|error| error.to_string())?;

    let first_missing_part_index = parts
        .iter()
        .position(|part| part.display_image_path.trim().is_empty())
        .unwrap_or(parts.len());
    if first_missing_part_index >= parts.len() {
        return Err(format!(
            "All {category} parts already have image associations. Clear the category images before re-importing."
        ));
    }

    let (screenshot_width, screenshot_height) = png_image_dimensions(&source_path)?;
    let import_plan = build_gearblocks_catalog_import_plan(
        &source_path,
        &parts,
        first_missing_part_index,
        screenshot_width,
        screenshot_height,
    )?;

    let mut updated_parts = Vec::new();
    for (part_index, crop, cropped_rgba) in import_plan {
        let part = &parts[part_index];
        let part_label = if !part.asset_name.trim().is_empty() {
            part.asset_name.as_str()
        } else if !part.display_name.trim().is_empty() {
            part.display_name.as_str()
        } else {
            part.part_key.as_str()
        };
        let file_stem = safe_filename_part(part_label);
        let file_name = if file_stem.is_empty() {
            format!("catalog_{}_part_{}.png", safe_tag_part(category), part.id)
        } else {
            format!(
                "catalog_{}_{}_{}.png",
                safe_tag_part(category),
                file_stem,
                part.id
            )
        };
        let output_path = part_image_dir.join(file_name);
        write_rgba_png(&output_path, crop.width, crop.height, &cropped_rgba)?;
        let updated = state
            .database
            .update_game_runtime_part_display_image(
                game.id,
                part.id,
                &path_text(&output_path),
                &path_text(&source_path),
            )
            .map_err(|error| error.to_string())?;
        updated_parts.push(updated);
    }

    if updated_parts.is_empty() {
        return Err(
            "No complete catalog icons with visible part-name text were found in the selected screenshot."
                .to_string(),
        );
    }

    Ok(updated_parts)
}

#[tauri::command]
pub fn list_game_catalog_objects(
    game_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<GameCatalogObjectRecord>, String> {
    state
        .database
        .list_game_catalog_objects(game_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_game_part_categories(
    game_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<GamePartCategoryRecord>, String> {
    let game = state
        .database
        .get_game(game_id)
        .map_err(|error| error.to_string())?;
    if game.slug != "gearblocks" {
        return Ok(Vec::new());
    }

    let parts = state
        .database
        .list_game_runtime_parts(game.id)
        .map_err(|error| error.to_string())?;
    let mut counts = HashMap::new();
    for part in parts {
        *counts.entry(part.category).or_insert(0_i64) += 1;
    }

    let category_icon_dir = overlay_workspace_root()?
        .join("game-screenshots")
        .join("category-icons");

    let categories = gearblocks_part_categories()
        .into_iter()
        .map(|category| {
            let icon_path = existing_category_icon_path(&category_icon_dir, category.name)
                .map(|path| path_text(&path))
                .unwrap_or_default();

            GamePartCategoryRecord {
                name: category.name.to_string(),
                fallback_icon: category.category_icon.to_string(),
                icon_path,
                count: counts.get(category.name).copied().unwrap_or(0),
            }
        })
        .collect();

    Ok(categories)
}

#[tauri::command]
pub fn catalog_game_parts_from_screenshots(
    game_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<GameCatalogObjectRecord>, String> {
    let game = state
        .database
        .get_game(game_id)
        .map_err(|error| error.to_string())?;
    if game.slug != "gearblocks" {
        return Err("Parts catalog import is currently configured for GearBlocks.".to_string());
    }

    let screenshots_root = overlay_workspace_root()?.join("game-screenshots");
    let game_screenshot_dir = screenshots_root.join(&game.slug);
    let category_icon_dir = screenshots_root.join("category-icons");
    fs::create_dir_all(&category_icon_dir).map_err(|error| error.to_string())?;
    let seeds = gearblocks_catalog_part_seeds();
    let resolved_categories =
        resolve_gearblocks_categories(&game_screenshot_dir, &category_icon_dir)?;

    state
        .database
        .delete_game_screenshot_catalog_objects(game.id)
        .map_err(|error| error.to_string())?;

    for seed in seeds {
        let resolved_category = resolved_categories.get(seed.category).ok_or_else(|| {
            format!(
                "Missing GearBlocks category definition for {}",
                seed.category
            )
        })?;
        let description = practical_part_description(seed.name, seed.category);
        let notes = format!(
            "Cataloged from GearBlocks screenshot category icon: {}.",
            resolved_category.category_icon
        );
        let tags = format!(
            "gearblocks,screenshot-catalog,{}",
            safe_tag_part(seed.category)
        );

        state
            .database
            .upsert_game_catalog_object(
                game.id,
                seed.name,
                "part",
                seed.category,
                resolved_category.category_icon,
                &resolved_category.category_icon_path,
                &description,
                &notes,
                &tags,
                &resolved_category.source_path_text,
                &resolved_category.source_path_text,
            )
            .map_err(|error| error.to_string())?;
    }

    state
        .database
        .list_game_catalog_objects(game.id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_game_screenshots(
    game_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<GameScreenshotCaptureRequestRecord>, String> {
    let screenshots = state
        .database
        .list_game_screenshots(game_id)
        .map_err(|error| error.to_string())?;

    Ok(screenshots
        .into_iter()
        .filter(|screenshot| std::path::Path::new(&screenshot.file_path).is_file())
        .collect())
}

#[tauri::command]
pub fn delete_game_screenshot(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let screenshot = state
        .database
        .get_game_screenshot(id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "Screenshot was not found.".to_string())?;
    let screenshots_root = overlay_workspace_root()?.join("game-screenshots");
    fs::create_dir_all(&screenshots_root).map_err(|error| error.to_string())?;

    remove_screenshot_file_if_present(&screenshot.file_path, &screenshots_root)?;
    remove_screenshot_file_if_present(&screenshot.request_path, &screenshots_root)?;

    state
        .database
        .delete_game_screenshot_references(
            screenshot.game_id,
            &screenshot.file_path,
            &screenshot.request_path,
        )
        .map_err(|error| error.to_string())?;
    state
        .database
        .delete_game_screenshot(id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_game_screenshot_capture_request(
    game_id: i64,
    timestamp_label: String,
    app: AppHandle,
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<GameScreenshotCaptureRequestRecord, String> {
    create_game_screenshot_capture(
        game_id,
        timestamp_label,
        app,
        window,
        state,
        "visible-game-display",
        "windows-gdi-bitblt-foreground-window",
        "hide Overlay Forge before capture, then restore it",
        "captured_windows_gdi",
        "Captured through Windows GDI BitBlt from the foreground window after hiding Overlay Forge. Alpha was forced to 255 before PNG encoding.",
        true,
        false,
        capture_foreground_window_to_png,
    )
}

#[tauri::command]
pub fn create_game_chat_screenshot_capture(
    game_id: i64,
    timestamp_label: String,
    app: AppHandle,
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<GameScreenshotCaptureRequestRecord, String> {
    create_game_screenshot_capture(
        game_id,
        timestamp_label,
        app,
        window,
        state,
        "visible-game-display",
        "windows-gdi-bitblt-foreground-window",
        "hide Overlay Forge before capture and leave focus with the game",
        "captured_windows_gdi_chat",
        "Captured through Windows GDI BitBlt from the foreground window after hiding Overlay Forge. Alpha was forced to 255 before PNG encoding and the screenshot was attached to the current Gaming chat prompt.",
        false,
        true,
        capture_foreground_window_to_png,
    )
}

fn create_game_screenshot_capture(
    game_id: i64,
    timestamp_label: String,
    app: AppHandle,
    window: WebviewWindow,
    state: State<'_, AppState>,
    capture_scope: &str,
    method_source: &str,
    overlay_handling: &str,
    capture_status: &str,
    notes: &str,
    restore_overlay: bool,
    restore_chat_overlay: bool,
    capture: fn(&std::path::Path) -> Result<(), String>,
) -> Result<GameScreenshotCaptureRequestRecord, String> {
    let game = state
        .database
        .get_game(game_id)
        .map_err(|error| error.to_string())?;
    let workspace_root = overlay_workspace_root()?;
    let screenshots_root = workspace_root.join("game-screenshots");
    let game_screenshot_dir = screenshots_root.join(&game.slug);
    let request_dir = screenshots_root.join("capture-requests");
    fs::create_dir_all(&game_screenshot_dir).map_err(|error| error.to_string())?;
    fs::create_dir_all(&request_dir).map_err(|error| error.to_string())?;

    let request_id = screenshot_request_id(&timestamp_label);
    let game_file_stem = safe_filename_part(&game.name);
    let target_file_path = game_screenshot_dir.join(format!("{game_file_stem}_{request_id}.png"));
    let request_file_path = request_dir.join(format!("{request_id}.json"));
    let title = format!("{} screenshot {}", game.name, request_id);
    let target_file_path_text = path_text(&target_file_path);
    let request_file_path_text = path_text(&request_file_path);

    let request_payload = json!({
        "schema": "overlay-forge.game-screenshot-capture.v1",
        "requestId": request_id,
        "gameId": game.id,
        "gameName": game.name,
        "gameSlug": game.slug,
        "captureScope": capture_scope,
        "includeOverlay": false,
        "targetFilePath": target_file_path_text,
        "method": {
            "source": method_source,
            "format": "png",
            "colorSpace": "sRGB",
            "forceAlpha": 255,
            "filenamePattern": "GameName_YYYYMMDD_HHMMSS_unique.png",
            "overlayHandling": overlay_handling,
            "knownRisk": "GDI capture may still fail or produce black frames for some hardware-accelerated or protected game surfaces."
        }
    });
    let request_payload_text =
        serde_json::to_string_pretty(&request_payload).map_err(|error| error.to_string())?;
    fs::write(&request_file_path, request_payload_text).map_err(|error| error.to_string())?;

    let invoking_window_label = window.label().to_string();
    let hidden_overlay_windows = hide_overlay_windows_for_capture(&app)?;
    thread::sleep(Duration::from_millis(350));

    let capture_result = capture(&target_file_path);

    if restore_overlay || capture_result.is_err() {
        let _ = window.show();
        let _ = window.set_focus();
    }
    restore_previously_visible_overlay_windows(&hidden_overlay_windows, &invoking_window_label);

    capture_result?;

    if restore_chat_overlay {
        restore_game_chat_overlay_after_capture(&app, &state)?;
    }

    state
        .database
        .create_game_screenshot_capture_request(
            game.id,
            &title,
            &target_file_path_text,
            &request_id,
            &request_file_path_text,
            capture_status,
            &timestamp_label,
            notes,
        )
        .map_err(|error| error.to_string())
}

const OVERLAY_CAPTURE_WINDOW_LABELS: [&str; 2] = ["main", "game-chat"];

struct HiddenOverlayCaptureWindow {
    label: &'static str,
    window: WebviewWindow,
    was_visible: bool,
}

fn hide_overlay_windows_for_capture(
    app: &AppHandle,
) -> Result<Vec<HiddenOverlayCaptureWindow>, String> {
    let mut hidden_windows = Vec::new();

    for label in OVERLAY_CAPTURE_WINDOW_LABELS {
        let Some(window) = app.get_webview_window(label) else {
            continue;
        };
        let was_visible = window.is_visible().map_err(|error| error.to_string())?;
        if was_visible {
            window.hide().map_err(|error| error.to_string())?;
        }
        hidden_windows.push(HiddenOverlayCaptureWindow {
            label,
            window,
            was_visible,
        });
    }

    Ok(hidden_windows)
}

fn restore_previously_visible_overlay_windows(
    hidden_windows: &[HiddenOverlayCaptureWindow],
    invoking_window_label: &str,
) {
    for hidden_window in hidden_windows {
        if !hidden_window.was_visible || hidden_window.label == invoking_window_label {
            continue;
        }
        let _ = hidden_window.window.set_always_on_top(true);
        let _ = ensure_window_accepts_mouse_input(&hidden_window.window);
        let _ = set_overlay_opacity(&hidden_window.window, 0.78);
        let _ = show_window_without_activation(&hidden_window.window);
    }
}

fn restore_game_chat_overlay_after_capture(
    app: &AppHandle,
    state: &State<'_, AppState>,
) -> Result<(), String> {
    let Some(window) = app.get_webview_window("game-chat") else {
        return Ok(());
    };

    window
        .set_always_on_top(true)
        .map_err(|error| error.to_string())?;
    ensure_window_accepts_mouse_input(&window)?;
    let _ = set_overlay_opacity(&window, 0.78);
    show_window_without_activation(&window)?;
    let _ = focus_last_game_window_from_state(state.inner());

    Ok(())
}

#[tauri::command]
pub fn list_game_chat_conversations(
    game_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<GameChatConversationRecord>, String> {
    state
        .database
        .list_game_chat_conversations(game_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_game_chat_conversation(
    game_id: i64,
    title: Option<String>,
    state: State<'_, AppState>,
) -> Result<GameChatConversationRecord, String> {
    state
        .database
        .create_game_chat_conversation(game_id, title.as_deref().unwrap_or_default())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_game_chat_messages(
    conversation_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<GameChatMessageRecord>, String> {
    state
        .database
        .list_game_chat_messages(conversation_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn send_game_chat_message(
    conversation_id: i64,
    content: String,
    screenshot_ids: Option<Vec<i64>>,
    state: State<'_, AppState>,
) -> Result<Vec<GameChatMessageRecord>, String> {
    require_text(&content, "Message")?;
    let conversation = state
        .database
        .get_game_chat_conversation(conversation_id)
        .map_err(|error| error.to_string())?;
    let game = state
        .database
        .get_game(conversation.game_id)
        .map_err(|error| error.to_string())?;

    state
        .database
        .create_game_chat_message(conversation_id, "user", &content)
        .map_err(|error| error.to_string())?;

    let recent_messages = state
        .database
        .recent_game_chat_messages(conversation_id, 20)
        .map_err(|error| error.to_string())?;
    let image_inputs = game_chat_image_inputs(
        &state,
        game.id,
        screenshot_ids.unwrap_or_default().as_slice(),
    )?;
    let custom_context = game_custom_prompt_context(&state, &game)?;
    let api_key = configured_openai_api_key(&state)?;
    let assistant_content = openai::create_game_response(
        &api_key,
        &game,
        &recent_messages,
        &custom_context,
        &image_inputs,
    )
    .await?;

    state
        .database
        .create_game_chat_message(conversation_id, "assistant", &assistant_content)
        .map_err(|error| error.to_string())?;

    state
        .database
        .list_game_chat_messages(conversation_id)
        .map_err(|error| error.to_string())
}

fn game_custom_prompt_context(
    state: &State<'_, AppState>,
    game: &GameRecord,
) -> Result<String, String> {
    match game.slug.as_str() {
        "gearblocks" => gearblocks_prompt_context(state, game.id),
        _ => Ok(String::new()),
    }
}

fn gearblocks_prompt_context(state: &State<'_, AppState>, game_id: i64) -> Result<String, String> {
    let mut sections = vec![gearblocks_parts_catalog_prompt_context()?];
    if let Some(runtime_context) = gearblocks_latest_runtime_understanding_context(state, game_id)?
    {
        sections.push(runtime_context);
    }
    Ok(sections.join("\n\n---\n\n"))
}

fn configured_openai_api_key(state: &State<'_, AppState>) -> Result<String, String> {
    let saved_key = state
        .database
        .get_app_setting("openai_api_key")
        .map_err(|error| error.to_string())?
        .unwrap_or_default();
    if !saved_key.trim().is_empty() {
        return Ok(saved_key.trim().to_string());
    }

    Ok(std::env::var("OPENAI_API_KEY")
        .map(|value| value.trim().to_string())
        .unwrap_or_default())
}

fn gearblocks_parts_catalog_prompt_context() -> Result<String, String> {
    let path = overlay_workspace_root()?
        .join("docs")
        .join("GEARBLOCKS_PARTS_CATALOG.md");
    let catalog = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let mut sections = vec!["# GearBlocks Parts Catalog".to_string()];

    for heading in [
        "## Category Order",
        "## Category Use Guide",
        "## Parts By Category",
    ] {
        if let Some(section) = markdown_section(&catalog, heading) {
            sections.push(section);
        }
    }

    Ok(sections.join("\n\n"))
}

fn markdown_section(markdown: &str, heading: &str) -> Option<String> {
    let lines = markdown.lines().collect::<Vec<_>>();
    let start = lines.iter().position(|line| line.trim() == heading)?;
    let end = lines
        .iter()
        .enumerate()
        .skip(start + 1)
        .find_map(|(index, line)| {
            if line.starts_with("## ") {
                Some(index)
            } else {
                None
            }
        })
        .unwrap_or(lines.len());

    Some(lines[start..end].join("\n").trim().to_string())
}

fn game_chat_image_inputs(
    state: &State<'_, AppState>,
    game_id: i64,
    screenshot_ids: &[i64],
) -> Result<Vec<openai::GameChatImageInput>, String> {
    if screenshot_ids.is_empty() {
        return Ok(Vec::new());
    }

    let screenshots = state
        .database
        .list_game_screenshots(game_id)
        .map_err(|error| error.to_string())?;
    let requested_ids = screenshot_ids
        .iter()
        .copied()
        .collect::<std::collections::HashSet<_>>();
    let mut images = Vec::new();

    for screenshot in screenshots {
        if !requested_ids.contains(&screenshot.id) {
            continue;
        }

        let path = PathBuf::from(&screenshot.file_path);
        if !path.is_file() {
            continue;
        }

        let mime_type = image_mime_type(&path)?;
        let bytes = fs::read(&path).map_err(|error| error.to_string())?;
        let data_url = format!("data:{mime_type};base64,{}", BASE64_STANDARD.encode(bytes));
        let label = if screenshot.captured_at.trim().is_empty() {
            screenshot.title
        } else {
            format!("{} ({})", screenshot.title, screenshot.captured_at)
        };

        images.push(openai::GameChatImageInput { label, data_url });
    }

    Ok(images)
}

fn image_mime_type(path: &std::path::Path) -> Result<&'static str, String> {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => Ok("image/png"),
        "jpg" | "jpeg" => Ok("image/jpeg"),
        "webp" => Ok("image/webp"),
        "gif" => Ok("image/gif"),
        _ => Err("Selected screenshot is not a supported image type.".to_string()),
    }
}

#[tauri::command]
pub fn delete_game_chat_conversation(
    conversation_id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .database
        .delete_game_chat_conversation(conversation_id)
        .map_err(|error| error.to_string())
}

fn validate_calendar_event(
    title: &str,
    start_date: &str,
    start_time: &str,
    end_date: &str,
    end_time: &str,
) -> Result<(), String> {
    require_text(title, "Event title")?;
    require_text(start_date, "Start date")?;
    require_text(start_time, "Start time")?;
    require_text(end_date, "End date")?;
    require_text(end_time, "End time")?;
    Ok(())
}

fn validate_project(name: &str, status: &str) -> Result<(), String> {
    require_text(name, "Project name")?;
    match status.trim() {
        "ACTIVE" | "ARCHIVED" => Ok(()),
        _ => Err("Project status must be ACTIVE or ARCHIVED".to_string()),
    }
}

fn validate_context_type(context_type: &str) -> Result<(), String> {
    match context_type.trim() {
        "project" | "github_repository" | "note" | "task" | "calendar_event"
        | "youtube_reference" | "scratchpad" => Ok(()),
        _ => Err("Unsupported context type".to_string()),
    }
}

fn extract_youtube_video_id(url: &str) -> Result<String, String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err("YouTube URL is required".to_string());
    }

    if let Some(rest) = trimmed.strip_prefix("https://www.youtube.com/watch?") {
        return extract_watch_video_id(rest);
    }

    if let Some(rest) = trimmed.strip_prefix("https://youtube.com/watch?") {
        return extract_watch_video_id(rest);
    }

    if let Some(rest) = trimmed.strip_prefix("https://youtu.be/") {
        return extract_path_video_id(rest);
    }

    if let Some(rest) = trimmed.strip_prefix("https://www.youtube.com/shorts/") {
        return extract_path_video_id(rest);
    }

    Err("Enter a valid YouTube URL, such as https://www.youtube.com/watch?v=VIDEO_ID or https://youtu.be/VIDEO_ID".to_string())
}

fn extract_watch_video_id(query: &str) -> Result<String, String> {
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        if parts.next() == Some("v") {
            let value = parts.next().unwrap_or_default();
            return validate_youtube_video_id(value);
        }
    }

    Err("YouTube watch URLs must include a video id in the v parameter".to_string())
}

fn extract_path_video_id(path: &str) -> Result<String, String> {
    let value = path.split(['?', '&', '#', '/']).next().unwrap_or_default();
    validate_youtube_video_id(value)
}

fn validate_youtube_video_id(value: &str) -> Result<String, String> {
    let video_id = value.trim();
    if video_id.len() != 11 {
        return Err("YouTube video id must be 11 characters long".to_string());
    }

    if !video_id
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '-' || character == '_')
    {
        return Err("YouTube video id contains unsupported characters".to_string());
    }

    Ok(video_id.to_string())
}

#[cfg(target_os = "windows")]
fn manual_overlay_drag(window: WebviewWindow) -> Result<(), String> {
    if MANUAL_OVERLAY_DRAG_ACTIVE.swap(true, Ordering::SeqCst) {
        return Ok(());
    }

    thread::spawn(move || {
        let _ = manual_overlay_drag_loop(window);
        MANUAL_OVERLAY_DRAG_ACTIVE.store(false, Ordering::SeqCst);
    });

    Ok(())
}

#[cfg(target_os = "windows")]
fn manual_overlay_drag_loop(window: WebviewWindow) -> Result<(), String> {
    use std::mem;
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
    use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;

    let started_at = std::time::Instant::now();
    let start_window_position = window.outer_position().map_err(|error| error.to_string())?;
    let start_cursor = unsafe {
        let mut point: POINT = mem::zeroed();
        if GetCursorPos(&mut point) == 0 {
            return Err("Could not read the mouse position.".to_string());
        }
        point
    };

    loop {
        let left_button_down =
            unsafe { (GetAsyncKeyState(VK_LBUTTON as i32) & 0x8000u16 as i16) != 0 };
        if !left_button_down {
            break;
        }

        let cursor = unsafe {
            let mut point: POINT = mem::zeroed();
            if GetCursorPos(&mut point) == 0 {
                break;
            }
            point
        };
        let next_x = start_window_position.x + (cursor.x - start_cursor.x);
        let next_y = start_window_position.y + (cursor.y - start_cursor.y);
        let _ = window.set_position(PhysicalPosition::new(next_x, next_y));
        if started_at.elapsed() > Duration::from_secs(8) {
            break;
        }
        thread::sleep(Duration::from_millis(8));
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn manual_overlay_drag(_window: WebviewWindow) -> Result<(), String> {
    Err("Manual no-snap overlay drag is only available on Windows.".to_string())
}

#[cfg(target_os = "windows")]
fn focus_last_game_window_impl(state: State<'_, AppState>) -> Result<bool, String> {
    focus_last_game_window_from_state(state.inner())
}

#[cfg(target_os = "windows")]
fn focus_last_game_window_from_state(state: &AppState) -> Result<bool, String> {
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::WindowsAndMessaging::{IsWindow, SetForegroundWindow};

    let hwnd = state
        .last_game_window
        .lock()
        .map_err(|_| "Last game window state is unavailable.".to_string())?
        .unwrap_or_default();
    if hwnd == 0 {
        return Ok(false);
    }

    let hwnd = hwnd as HWND;
    unsafe {
        if IsWindow(hwnd) == 0 {
            return Ok(false);
        }
        Ok(SetForegroundWindow(hwnd) != 0)
    }
}

#[cfg(not(target_os = "windows"))]
fn focus_last_game_window_impl(_state: State<'_, AppState>) -> Result<bool, String> {
    Ok(false)
}

#[cfg(not(target_os = "windows"))]
fn focus_last_game_window_from_state(_state: &AppState) -> Result<bool, String> {
    Ok(false)
}

#[cfg(target_os = "windows")]
fn show_window_without_activation(window: &WebviewWindow) -> Result<(), String> {
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, ShowWindow, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
        SWP_SHOWWINDOW, SW_SHOWNOACTIVATE,
    };

    let hwnd = window.hwnd().map_err(|error| error.to_string())?;
    unsafe {
        ShowWindow(hwnd.0 as HWND, SW_SHOWNOACTIVATE);
        SetWindowPos(
            hwnd.0 as HWND,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
        );
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn ensure_window_accepts_mouse_input(window: &WebviewWindow) -> Result<(), String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_NOACTIVATE, WS_EX_TRANSPARENT,
    };

    let hwnd =
        window.hwnd().map_err(|error| error.to_string())?.0 as windows_sys::Win32::Foundation::HWND;
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let next_style = style & !(WS_EX_TRANSPARENT as isize) & !(WS_EX_NOACTIVATE as isize);
        if next_style != style {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, next_style);
        }
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn ensure_window_accepts_mouse_input(_window: &WebviewWindow) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn show_window_without_activation(window: &WebviewWindow) -> Result<(), String> {
    window.show().map_err(|error| error.to_string())
}

#[cfg(target_os = "windows")]
fn set_overlay_opacity(window: &WebviewWindow, opacity: f64) -> Result<(), String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetLayeredWindowAttributes, SetWindowLongPtrW, GWL_EXSTYLE, LWA_ALPHA,
        WS_EX_LAYERED,
    };

    let hwnd =
        window.hwnd().map_err(|error| error.to_string())?.0 as windows_sys::Win32::Foundation::HWND;
    let alpha = (opacity.clamp(0.2, 1.0) * 255.0).round() as u8;

    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | WS_EX_LAYERED as isize);
        if SetLayeredWindowAttributes(hwnd, 0, alpha, LWA_ALPHA) == 0 {
            return Err("Could not set overlay window opacity.".to_string());
        }
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn set_overlay_opacity(_window: &WebviewWindow, _opacity: f64) -> Result<(), String> {
    Ok(())
}

fn open_external_url(url: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("rundll32")
        .args(["url.dll,FileProtocolHandler", url])
        .spawn();

    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("open").arg(url).spawn();

    #[cfg(all(unix, not(target_os = "macos")))]
    let result = std::process::Command::new("xdg-open").arg(url).spawn();

    result
        .map(|_| ())
        .map_err(|error| format!("Could not open YouTube URL externally: {error}"))
}

fn require_text(value: &str, field_name: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{field_name} is required"));
    }

    Ok(())
}

fn game_exposes_data_locations(game: &GameRecord) -> bool {
    game.slug == "gearblocks"
}

fn require_game_data_locations_enabled(game: &GameRecord) -> Result<(), String> {
    if game_exposes_data_locations(game) {
        Ok(())
    } else {
        Err(format!(
            "{} does not expose save or alternate data location settings.",
            game.name
        ))
    }
}

fn require_gearblocks_game(game: &GameRecord) -> Result<(), String> {
    if game.slug == "gearblocks" {
        Ok(())
    } else {
        Err(format!(
            "{} does not expose GearBlocks construction decoding.",
            game.name
        ))
    }
}

fn normalize_game_location_type(location_type: &str) -> Result<&'static str, String> {
    match location_type.trim().to_ascii_lowercase().as_str() {
        "save" => Ok("save"),
        "alternate" => Ok("alternate"),
        _ => Err("Game data location type must be save or alternate.".to_string()),
    }
}

fn game_location_type_label(location_type: &str) -> &'static str {
    match location_type {
        "save" => "Save Location",
        "alternate" => "Alternate Data Location",
        _ => "Data Location",
    }
}

fn validate_directory_path(directory_path: &str) -> Result<String, String> {
    require_text(directory_path, "Directory path")?;
    let path = PathBuf::from(directory_path.trim());
    let canonical = fs::canonicalize(&path)
        .map_err(|error| format!("Could not read selected directory: {error}"))?;
    if !canonical.is_dir() {
        return Err("Selected path is not a directory.".to_string());
    }

    Ok(canonical.to_string_lossy().to_string())
}

fn gearblocks_saved_constructions_root(state: &AppState, game_id: i64) -> Result<PathBuf, String> {
    let locations = state
        .database
        .list_game_data_locations(game_id)
        .map_err(|error| error.to_string())?;
    if let Some(location) = locations
        .iter()
        .find(|location| location.location_type == "save")
    {
        let configured = PathBuf::from(location.directory_path.trim());
        if configured.is_dir() {
            return Ok(configured);
        }
    }

    Ok(gearblocks_default_user_data_root()?.join("SavedConstructions"))
}

fn gearblocks_default_user_data_root() -> Result<PathBuf, String> {
    let profile = std::env::var("USERPROFILE").map_err(|_| {
        "USERPROFILE is not available for GearBlocks user data discovery.".to_string()
    })?;
    Ok(PathBuf::from(profile)
        .join("AppData")
        .join("LocalLow")
        .join("SmashHammer Games")
        .join("GearBlocks"))
}

fn gearblocks_runtime_export_dir(
    state: &State<'_, AppState>,
    game_id: i64,
    gearblocks_root: &std::path::Path,
) -> Result<PathBuf, String> {
    let locations = state
        .database
        .list_game_data_locations(game_id)
        .map_err(|error| error.to_string())?;
    if let Some(location) = locations
        .iter()
        .find(|location| location.location_type == "alternate")
    {
        let configured = PathBuf::from(location.directory_path.trim());
        if configured.is_dir() {
            return Ok(configured);
        }
    }

    Ok(gearblocks_root.join("OverlayForgeExports"))
}

fn lua_long_bracket_path(path: &std::path::Path) -> String {
    path.to_string_lossy().replace("]]", "] ]")
}

struct PendingGearBlocksRuntimeExport {
    intended_path: String,
    chunks: Vec<String>,
}

fn parse_gearblocks_runtime_exports_from_log(
    log_path: &Path,
) -> Result<Vec<GearBlocksRuntimeExportRecord>, String> {
    const BEGIN_MARKER: &str = "[OverlayForgeExportBegin]";
    const DATA_MARKER: &str = "[OverlayForgeExportData]";
    const END_MARKER: &str = "[OverlayForgeExportEnd]";

    let text = fs::read_to_string(log_path)
        .map_err(|error| format!("Could not read GearBlocks runtime export log: {error}"))?;
    let mut pending: HashMap<String, PendingGearBlocksRuntimeExport> = HashMap::new();
    let mut exports = Vec::new();
    let source_log_path = log_path.to_string_lossy().to_string();

    for line in text.lines() {
        if let Some(index) = line.find(BEGIN_MARKER) {
            let payload = &line[index + BEGIN_MARKER.len()..];
            if let Ok(document) = serde_json::from_str::<serde_json::Value>(payload) {
                if let Some(id) = document.get("id").and_then(|value| value.as_str()) {
                    let intended_path = document
                        .get("path")
                        .and_then(|value| value.as_str())
                        .unwrap_or("")
                        .to_string();
                    pending.insert(
                        id.to_string(),
                        PendingGearBlocksRuntimeExport {
                            intended_path,
                            chunks: Vec::new(),
                        },
                    );
                }
            }
            continue;
        }

        if let Some(index) = line.find(DATA_MARKER) {
            let payload = &line[index + DATA_MARKER.len()..];
            if let Some((id, chunk)) = payload.split_once('|') {
                if let Some(export) = pending.get_mut(id) {
                    export.chunks.push(chunk.to_string());
                }
            }
            continue;
        }

        if let Some(index) = line.find(END_MARKER) {
            let id = line[index + END_MARKER.len()..].trim();
            if let Some(export) = pending.remove(id) {
                let content = export.chunks.join("");
                if let Ok(mut document) = serde_json::from_str::<serde_json::Value>(&content) {
                    hydrate_gearblocks_api_attribute_refs(&mut document);
                    let name = Path::new(&export.intended_path)
                        .file_name()
                        .and_then(|value| value.to_str())
                        .filter(|value| !value.is_empty())
                        .unwrap_or(id)
                        .to_string();
                    exports.push(GearBlocksRuntimeExportRecord {
                        id: id.to_string(),
                        name,
                        intended_path: export.intended_path,
                        source_log_path: source_log_path.clone(),
                        byte_size: content.len(),
                        document,
                    });
                }
            }
        }
    }

    Ok(exports)
}

fn hydrate_gearblocks_api_attribute_refs(document: &mut serde_json::Value) {
    let catalog = document
        .get("apiAttributeCatalog")
        .and_then(serde_json::Value::as_object)
        .cloned();
    let Some(catalog) = catalog else {
        return;
    };

    let Some(parts) = document
        .get_mut("parts")
        .and_then(serde_json::Value::as_array_mut)
    else {
        return;
    };

    for part in parts {
        let has_attributes = part
            .get("apiAttributes")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|attributes| !attributes.is_empty());
        if has_attributes {
            continue;
        }

        let Some(key) = part
            .get("apiAttributeKey")
            .and_then(serde_json::Value::as_str)
        else {
            continue;
        };
        let Some(attributes) = catalog.get(key) else {
            continue;
        };
        if let Some(object) = part.as_object_mut() {
            object.insert("apiAttributes".to_string(), attributes.clone());
        }
    }
}

struct GearBlocksRuntimePart<'a> {
    id: i64,
    index: i64,
    name: String,
    category: String,
    system: &'static str,
    purpose: &'static str,
    behaviours: Vec<String>,
    local_position: Option<&'a serde_json::Value>,
    current_unit_size: Option<&'a serde_json::Value>,
    link_node_count: usize,
    mass: f64,
    is_structural: bool,
    is_functional: bool,
}

fn gearblocks_latest_runtime_understanding_context(
    state: &AppState,
    game_id: i64,
) -> Result<Option<String>, String> {
    import_latest_gearblocks_runtime_exports(state, game_id)?;
    let Some(latest) = state
        .database
        .latest_game_runtime_construction_export(game_id)
        .map_err(|error| error.to_string())?
    else {
        return Ok(None);
    };
    let document = serde_json::from_str::<serde_json::Value>(&latest.document_json)
        .map_err(|error| format!("Latest GearBlocks runtime export JSON is invalid: {error}"))?;
    let export = GearBlocksRuntimeExportRecord {
        id: latest.export_id,
        name: latest.name,
        intended_path: latest.intended_path,
        source_log_path: latest.source_log_path,
        byte_size: latest.byte_size.max(0) as usize,
        document,
    };

    Ok(Some(gearblocks_runtime_understanding_context(&export)))
}

fn gearblocks_runtime_understanding_context(export: &GearBlocksRuntimeExportRecord) -> String {
    let parts_json = export
        .document
        .get("parts")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    let mut parts = Vec::new();

    for part in &parts_json {
        let name = preferred_part_name(part);
        let category = json_string(part.get("category"));
        let behaviours = part
            .get("behaviours")
            .and_then(|value| value.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.get("name").and_then(|value| value.as_str()))
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let behaviour_text = behaviours.join(" ").to_ascii_lowercase();
        let combined_text = format!(
            "{} {} {} {}",
            name,
            category,
            behaviour_text,
            json_string(part.get("assetName"))
        )
        .to_ascii_lowercase();
        let system = classify_gearblocks_part_system(&combined_text, &category, &behaviours);
        let purpose = gearblocks_part_purpose(system, &combined_text, &behaviour_text);
        let link_node_count = part
            .get("linkNodes")
            .and_then(|value| value.as_array())
            .map(|items| items.len())
            .unwrap_or(0);
        let is_structural = system == "structural frame" || system == "mounts and connectors";
        let is_functional = !is_structural || !behaviours.is_empty() || link_node_count > 0;

        parts.push(GearBlocksRuntimePart {
            id: json_i64(part.get("id")),
            index: json_i64(part.get("index")),
            name,
            category,
            system,
            purpose,
            behaviours,
            local_position: part.get("localPosition"),
            current_unit_size: part.get("currentUnitSize"),
            link_node_count,
            mass: json_f64(part.get("mass")),
            is_structural,
            is_functional,
        });
    }

    let mut sections = Vec::new();
    sections.push("# GearBlocks Runtime Construction Understanding".to_string());
    sections.push(format!(
        "Source: latest runtime export reconstructed from `{}`. Intended output path: `{}`.",
        export.source_log_path, export.intended_path
    ));
    sections.push(format!(
        "Construction ID: {}. Parts: {}. Runtime-reported parts: {}. Mass: {:.2}. Frozen: {}. Invulnerable: {}. Player character: {}.",
        json_i64(export.document.get("id")),
        parts.len(),
        json_i64(export.document.get("numParts")),
        json_f64(export.document.get("mass")),
        json_bool_label(export.document.get("isFrozen")),
        json_bool_label(export.document.get("isInvulnerable")),
        json_bool_label(export.document.get("isPlayerCharacter"))
    ));

    sections.push(gearblocks_system_counts_section(&parts));
    sections.push(gearblocks_inventory_section(
        "Functional inventory by system",
        &parts,
        false,
    ));
    sections.push(gearblocks_inventory_section(
        "Structural inventory",
        &parts,
        true,
    ));
    sections.push(gearblocks_structural_bounds_section(&parts));
    sections.push(gearblocks_functional_parts_section(&parts));

    sections.join("\n\n")
}

fn persist_runtime_export(
    state: &AppState,
    game_id: i64,
    export: &GearBlocksRuntimeExportRecord,
    indexed_at: &str,
) -> Result<GameRuntimeConstructionExportRecord, String> {
    let document_json =
        serde_json::to_string_pretty(&export.document).map_err(|error| error.to_string())?;
    state
        .database
        .upsert_game_runtime_construction_export(
            game_id,
            &export.id,
            &export.name,
            &json_string(export.document.get("exportKind")),
            &export.intended_path,
            &export.source_log_path,
            export.byte_size as i64,
            &export
                .document
                .get("id")
                .map(json_value_to_string)
                .unwrap_or_default(),
            export
                .document
                .get("exportedAt")
                .and_then(|value| value.as_str())
                .unwrap_or_default(),
            json_i64(export.document.get("numParts")),
            json_f64(export.document.get("mass")),
            json_optional_bool(export.document.get("isFrozen")),
            json_optional_bool(export.document.get("isInvulnerable")),
            json_optional_bool(export.document.get("isPlayerCharacter")),
            &document_json,
            indexed_at,
        )
        .map_err(|error| error.to_string())
}

fn import_runtime_export_parts(
    state: &AppState,
    game_id: i64,
    export: &GearBlocksRuntimeExportRecord,
) -> Result<(), String> {
    let parts = export
        .document
        .get("parts")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    let source_construction_id = export
        .document
        .get("id")
        .map(json_value_to_string)
        .unwrap_or_default();
    let last_seen_at = export
        .document
        .get("exportedAt")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string();

    for part in parts {
        let part_key = runtime_part_key(&part);
        if part_key.is_empty() {
            continue;
        }
        let asset_guid = json_string(part.get("assetGuid"));
        let asset_name = json_string(part.get("assetName"));
        let display_name = json_string(part.get("displayName"));
        let full_display_name = json_string(part.get("fullDisplayName"));
        let category = json_string(part.get("category"));
        let properties_json =
            serde_json::to_string_pretty(&part).map_err(|error| error.to_string())?;
        state
            .database
            .upsert_game_runtime_part(
                game_id,
                &part_key,
                &asset_guid,
                &asset_name,
                &display_name,
                &full_display_name,
                &category,
                json_f64(part.get("mass")),
                &properties_json,
                &export.id,
                &source_construction_id,
                &last_seen_at,
            )
            .map_err(|error| error.to_string())?;

        let context = RuntimePartIndexContext {
            game_id,
            part_key,
            asset_guid,
            asset_name,
            display_name,
            full_display_name,
            category,
            source_export_id: export.id.clone(),
            source_construction_id: source_construction_id.clone(),
            seen_at: last_seen_at.clone(),
        };
        index_runtime_part_discovery(state, &context, &part)?;
    }

    Ok(())
}

struct RuntimePartIndexContext {
    game_id: i64,
    part_key: String,
    asset_guid: String,
    asset_name: String,
    display_name: String,
    full_display_name: String,
    category: String,
    source_export_id: String,
    source_construction_id: String,
    seen_at: String,
}

fn index_runtime_part_discovery(
    state: &AppState,
    context: &RuntimePartIndexContext,
    part: &serde_json::Value,
) -> Result<(), String> {
    if let Some(attributes) = part
        .get("apiAttributes")
        .and_then(serde_json::Value::as_array)
    {
        for attribute in attributes {
            let interface_name = json_string(attribute.get("interface"));
            let attribute_name = json_string(attribute.get("name"));
            if interface_name.is_empty() || attribute_name.is_empty() {
                continue;
            }
            state
                .database
                .upsert_game_runtime_part_api_attribute(
                    context.game_id,
                    &context.part_key,
                    &context.asset_guid,
                    &context.asset_name,
                    &context.display_name,
                    &context.full_display_name,
                    &context.category,
                    &interface_name,
                    &attribute_name,
                    &json_string(attribute.get("valueType")),
                    &json_string(attribute.get("availability")),
                    &context.source_export_id,
                    &context.source_construction_id,
                    &context.seen_at,
                )
                .map_err(|error| error.to_string())?;
            state
                .database
                .upsert_game_runtime_part_api_member(
                    context.game_id,
                    &context.part_key,
                    &interface_name,
                    &attribute_name,
                    &json_string(attribute.get("availability")),
                    &context.source_export_id,
                    &context.source_construction_id,
                    &context.seen_at,
                )
                .map_err(|error| error.to_string())?;
        }
    }

    let mut value_fields = Vec::new();
    collect_named_value_fields(part, "", &mut value_fields);
    for (field_path, value) in value_fields {
        state
            .database
            .upsert_game_runtime_part_value(
                context.game_id,
                &context.part_key,
                &context.asset_guid,
                &context.asset_name,
                &context.display_name,
                &context.full_display_name,
                &context.category,
                &field_path,
                json_type_label(value),
                &value.to_string(),
                &context.source_export_id,
                &context.source_construction_id,
                &context.seen_at,
            )
            .map_err(|error| error.to_string())?;
    }

    if let Some(properties) = part.get("properties") {
        let mut property_values = Vec::new();
        collect_leaf_json_values(properties, "", &mut property_values);
        if property_values.is_empty() && !properties.is_object() && !properties.is_array() {
            property_values.push(("properties".to_string(), properties));
        }
        for (property_path, value) in property_values {
            state
                .database
                .upsert_game_runtime_part_property(
                    context.game_id,
                    &context.part_key,
                    &context.asset_guid,
                    &context.asset_name,
                    &context.display_name,
                    &context.full_display_name,
                    &context.category,
                    &property_path,
                    json_type_label(value),
                    &value.to_string(),
                    &context.source_export_id,
                    &context.source_construction_id,
                    &context.seen_at,
                )
                .map_err(|error| error.to_string())?;
        }
    }

    if let Some(attachments) = part.get("attachments") {
        let mut attachment_values = Vec::new();
        collect_direct_json_children(attachments, "", &mut attachment_values);
        for (attachment_path, value) in attachment_values {
            state
                .database
                .upsert_game_runtime_part_attachment(
                    context.game_id,
                    &context.part_key,
                    &context.asset_guid,
                    &context.asset_name,
                    &context.display_name,
                    &context.full_display_name,
                    &context.category,
                    &attachment_path,
                    json_type_label(value),
                    &value.to_string(),
                    &context.source_export_id,
                    &context.source_construction_id,
                    &context.seen_at,
                )
                .map_err(|error| error.to_string())?;
        }
    }

    Ok(())
}

fn collect_named_value_fields<'a>(
    value: &'a serde_json::Value,
    path: &str,
    values: &mut Vec<(String, &'a serde_json::Value)>,
) {
    match value {
        serde_json::Value::Object(object) => {
            for (key, child) in object {
                let child_path = json_path_child(path, key);
                if key == "value" {
                    values.push((child_path.clone(), child));
                }
                collect_named_value_fields(child, &child_path, values);
            }
        }
        serde_json::Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                collect_named_value_fields(child, &json_path_index(path, index), values);
            }
        }
        _ => {}
    }
}

fn collect_leaf_json_values<'a>(
    value: &'a serde_json::Value,
    path: &str,
    values: &mut Vec<(String, &'a serde_json::Value)>,
) {
    match value {
        serde_json::Value::Object(object) => {
            for (key, child) in object {
                collect_leaf_json_values(child, &json_path_child(path, key), values);
            }
        }
        serde_json::Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                collect_leaf_json_values(child, &json_path_index(path, index), values);
            }
        }
        _ => {
            if !path.is_empty() {
                values.push((path.to_string(), value));
            }
        }
    }
}

fn collect_direct_json_children<'a>(
    value: &'a serde_json::Value,
    path: &str,
    values: &mut Vec<(String, &'a serde_json::Value)>,
) {
    match value {
        serde_json::Value::Object(object) => {
            if object.is_empty() && !path.is_empty() {
                values.push((path.to_string(), value));
            } else {
                for (key, child) in object {
                    values.push((json_path_child(path, key), child));
                }
            }
        }
        serde_json::Value::Array(items) => {
            if items.is_empty() && !path.is_empty() {
                values.push((path.to_string(), value));
            } else {
                for (index, child) in items.iter().enumerate() {
                    values.push((json_path_index(path, index), child));
                }
            }
        }
        _ => {
            values.push((
                if path.is_empty() {
                    "attachments".to_string()
                } else {
                    path.to_string()
                },
                value,
            ));
        }
    }
}

fn json_path_child(path: &str, key: &str) -> String {
    if path.is_empty() {
        key.to_string()
    } else {
        format!("{path}.{key}")
    }
}

fn json_path_index(path: &str, index: usize) -> String {
    if path.is_empty() {
        format!("[{index}]")
    } else {
        format!("{path}[{index}]")
    }
}

fn json_type_label(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

fn import_latest_gearblocks_runtime_exports(
    state: &AppState,
    game_id: i64,
) -> Result<usize, String> {
    let root = gearblocks_default_user_data_root()?;
    let mut exports = Vec::new();
    for log_path in [root.join("Player-prev.log"), root.join("Player.log")] {
        if log_path.is_file() {
            exports.extend(parse_gearblocks_runtime_exports_from_log(&log_path)?);
        }
    }

    let indexed_at = unix_timestamp_label();
    for export in &exports {
        persist_runtime_export(state, game_id, export, &indexed_at)?;
        import_runtime_export_parts(state, game_id, export)?;
    }

    Ok(exports.len())
}

fn runtime_part_key(part: &serde_json::Value) -> String {
    let asset_guid = json_string(part.get("assetGuid"));
    if !asset_guid.trim().is_empty() && asset_guid.trim() != "nil" {
        return format!("asset-guid:{}", asset_guid.trim());
    }

    let asset_name = json_string(part.get("assetName"));
    if !asset_name.trim().is_empty() {
        return format!("asset-name:{}", asset_name.trim().to_ascii_lowercase());
    }

    let display_name = preferred_part_name(part);
    let category = json_string(part.get("category"));
    if display_name.trim().is_empty() {
        String::new()
    } else {
        format!(
            "display:{}:{}",
            category.trim().to_ascii_lowercase(),
            display_name.trim().to_ascii_lowercase()
        )
    }
}

fn json_value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) => text.clone(),
        serde_json::Value::Null => String::new(),
        value => value.to_string(),
    }
}

fn classify_gearblocks_part_system(
    text: &str,
    category: &str,
    behaviours: &[String],
) -> &'static str {
    let category = category.to_ascii_lowercase();
    let has_behaviour = |needle: &str| {
        behaviours
            .iter()
            .any(|behaviour| behaviour.to_ascii_lowercase().contains(needle))
    };

    if category.contains("wheel")
        || text.contains("wheel")
        || text.contains("tire")
        || text.contains("tyre")
    {
        "wheels and tires"
    } else if category.contains("suspension")
        || text.contains("spring")
        || text.contains("damper")
        || text.contains("control arm")
        || text.contains("wishbone")
        || has_behaviour("ball")
    {
        "suspension"
    } else if category.contains("steering") || text.contains("steering") || text.contains("rack") {
        "steering"
    } else if category.contains("combustion")
        || category.contains("engine")
        || text.contains("engine")
        || text.contains("cylinder")
        || text.contains("crank")
        || has_behaviour("engine")
    {
        "powertrain engine"
    } else if category.contains("gear")
        || text.contains("gear")
        || text.contains("differential")
        || text.contains("shaft")
        || text.contains("axle")
        || text.contains("cv joint")
        || has_behaviour("gear")
        || has_behaviour("differential")
        || has_behaviour("velocity")
    {
        "drivetrain"
    } else if category.contains("brake")
        || category.contains("clutch")
        || text.contains("brake")
        || text.contains("clutch")
        || has_behaviour("clutch")
    {
        "brakes and clutches"
    } else if category.contains("control")
        || category.contains("logic")
        || text.contains("seat")
        || text.contains("lever")
        || text.contains("button")
        || text.contains("sensor")
        || has_behaviour("control")
        || has_behaviour("data")
    {
        "controls and data"
    } else if text.contains("beam")
        || text.contains("plate")
        || text.contains("bracket")
        || text.contains("frame")
        || text.contains("panel")
        || text.contains("bar")
        || text.contains("strut")
    {
        "structural frame"
    } else if category.contains("connector") || text.contains("connector") || text.contains("mount")
    {
        "mounts and connectors"
    } else if category.contains("body")
        || text.contains("body")
        || text.contains("fender")
        || text.contains("panel")
    {
        "bodywork"
    } else {
        "unknown or miscellaneous"
    }
}

fn gearblocks_part_purpose(system: &str, text: &str, behaviour_text: &str) -> &'static str {
    if behaviour_text.contains("spring") || text.contains("coil-over") || text.contains("damper") {
        "spring/damper element controlling suspension travel"
    } else if text.contains("control arm") || behaviour_text.contains("ball") {
        "articulated suspension locating member"
    } else if text.contains("differential") {
        "differential gear element distributing drive torque"
    } else if text.contains("clutch") {
        "engageable drivetrain coupling or clutch gear"
    } else if text.contains("brake") {
        "braking element"
    } else if text.contains("gear") {
        "gear train element transmitting or changing rotation"
    } else if text.contains("cv joint") {
        "constant-velocity joint or axle segment"
    } else if text.contains("axle") || text.contains("shaft") {
        "rotating shaft or axle segment"
    } else if text.contains("crank") || behaviour_text.contains("engine") {
        "engine crank or combustion powertrain element"
    } else if text.contains("wheel") || text.contains("tire") || text.contains("tyre") {
        "ground contact rolling element"
    } else if text.contains("steering") || text.contains("rack") {
        "steering input or linkage element"
    } else if system == "structural frame" {
        "rigid welded structural member"
    } else if system == "mounts and connectors" {
        "mounting connector or rigid attachment element"
    } else if system == "controls and data" {
        "control, signal, or data-bearing element"
    } else {
        "part role inferred from category and behaviours"
    }
}

fn gearblocks_system_counts_section(parts: &[GearBlocksRuntimePart<'_>]) -> String {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    let mut mass_by_system: HashMap<&str, f64> = HashMap::new();
    for part in parts {
        *counts.entry(part.system).or_default() += 1;
        *mass_by_system.entry(part.system).or_default() += part.mass;
    }

    let mut systems = counts.keys().copied().collect::<Vec<_>>();
    systems.sort_unstable();
    let lines = systems
        .into_iter()
        .map(|system| {
            format!(
                "- {}: {} part(s), {:.2} mass",
                system,
                counts.get(system).copied().unwrap_or_default(),
                mass_by_system.get(system).copied().unwrap_or_default()
            )
        })
        .collect::<Vec<_>>();

    format!("## System Counts\n{}", lines.join("\n"))
}

fn gearblocks_inventory_section(
    title: &str,
    parts: &[GearBlocksRuntimePart<'_>],
    structural_only: bool,
) -> String {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for part in parts {
        if structural_only != part.is_structural {
            continue;
        }
        let key = format!("{} | {}", part.system, part.name);
        *counts.entry(key).or_default() += 1;
    }

    if counts.is_empty() {
        return format!("## {title}\nNo matching parts identified.");
    }

    let mut rows = counts.into_iter().collect::<Vec<_>>();
    rows.sort_by(|left, right| left.0.cmp(&right.0));
    let lines = rows
        .into_iter()
        .take(80)
        .map(|(name, count)| format!("- {} x{}", name, count))
        .collect::<Vec<_>>();

    format!("## {title}\n{}", lines.join("\n"))
}

fn gearblocks_structural_bounds_section(parts: &[GearBlocksRuntimePart<'_>]) -> String {
    let mut min_values = [f64::MAX; 3];
    let mut max_values = [f64::MIN; 3];
    let mut found = false;

    for part in parts.iter().filter(|part| part.is_structural) {
        let Some(position) = part.local_position else {
            continue;
        };
        let Some((x, y, z)) = json_vector3(position) else {
            continue;
        };
        for (index, value) in [x, y, z].into_iter().enumerate() {
            min_values[index] = min_values[index].min(value);
            max_values[index] = max_values[index].max(value);
        }
        found = true;
    }

    if !found {
        return "## Structural Envelope\nNo structural bounds could be inferred.".to_string();
    }

    format!(
        "## Structural Envelope\nStructural member local-position bounds: x {:.2}..{:.2}, y {:.2}..{:.2}, z {:.2}..{:.2}. This is a coarse chassis envelope, not a visual mesh.",
        min_values[0], max_values[0], min_values[1], max_values[1], min_values[2], max_values[2]
    )
}

fn gearblocks_functional_parts_section(parts: &[GearBlocksRuntimePart<'_>]) -> String {
    let mut functional_parts = parts
        .iter()
        .filter(|part| part.is_functional)
        .collect::<Vec<_>>();
    functional_parts.sort_by(|left, right| {
        left.system
            .cmp(right.system)
            .then(left.index.cmp(&right.index))
    });

    let mut lines = Vec::new();
    for part in functional_parts.iter().take(140) {
        let behaviours = if part.behaviours.is_empty() {
            "none".to_string()
        } else {
            part.behaviours.join(", ")
        };
        let size = part
            .current_unit_size
            .and_then(json_vector3)
            .map(|(x, y, z)| format!(" size=({x:.2},{y:.2},{z:.2})"))
            .unwrap_or_default();
        let position = part
            .local_position
            .and_then(json_vector3)
            .map(|(x, y, z)| format!(" local=({x:.2},{y:.2},{z:.2})"))
            .unwrap_or_default();
        lines.push(format!(
            "- #{} idx {} [{} / {}] {}: {}; behaviours={}; links={}{}{}",
            part.id,
            part.index,
            part.system,
            part.category,
            part.name,
            part.purpose,
            behaviours,
            part.link_node_count,
            position,
            size
        ));
    }

    if functional_parts.len() > lines.len() {
        lines.push(format!(
            "- {} additional functional part(s) omitted from prompt context for size.",
            functional_parts.len() - lines.len()
        ));
    }

    if lines.is_empty() {
        "## Functional Parts\nNo functional parts identified.".to_string()
    } else {
        format!("## Functional Parts\n{}", lines.join("\n"))
    }
}

fn preferred_part_name(part: &serde_json::Value) -> String {
    for key in ["fullDisplayName", "displayName", "assetName"] {
        let value = json_string(part.get(key));
        if !value.is_empty() {
            return value;
        }
    }
    "Unnamed part".to_string()
}

fn json_string(value: Option<&serde_json::Value>) -> String {
    value
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn json_i64(value: Option<&serde_json::Value>) -> i64 {
    value.and_then(|value| value.as_i64()).unwrap_or_default()
}

fn json_f64(value: Option<&serde_json::Value>) -> f64 {
    value.and_then(|value| value.as_f64()).unwrap_or_default()
}

fn json_optional_bool(value: Option<&serde_json::Value>) -> Option<bool> {
    value.and_then(|value| value.as_bool())
}

fn json_bool_label(value: Option<&serde_json::Value>) -> &'static str {
    match value.and_then(|value| value.as_bool()) {
        Some(true) => "true",
        Some(false) => "false",
        None => "unknown",
    }
}

fn json_vector3(value: &serde_json::Value) -> Option<(f64, f64, f64)> {
    Some((
        value.get("x")?.as_f64()?,
        value.get("y")?.as_f64()?,
        value.get("z")?.as_f64()?,
    ))
}

fn list_gearblocks_construction_files_in_root(
    root: &std::path::Path,
) -> Result<Vec<GearBlocksConstructionFileRecord>, String> {
    if !root.is_dir() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(root).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if let Some(construction_path) = find_construction_file_in_folder(&path) {
            files.push(gearblocks_construction_file_record(&construction_path)?);
        }
    }
    files.sort_by(|left, right| {
        left.name
            .to_ascii_lowercase()
            .cmp(&right.name.to_ascii_lowercase())
    });
    Ok(files)
}

fn find_construction_file_in_folder(folder: &std::path::Path) -> Option<PathBuf> {
    for file_name in ["construction.bytes", "construction.byte"] {
        let path = folder.join(file_name);
        if path.is_file() {
            return Some(path);
        }
    }
    None
}

fn validate_gearblocks_construction_file_path(path_text: &str) -> Result<PathBuf, String> {
    require_text(path_text, "Construction file path")?;
    let path = PathBuf::from(path_text.trim());
    if !path.is_file() {
        return Err("Selected construction path is not a file.".to_string());
    }
    let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
        return Err("Selected construction path has no file name.".to_string());
    };
    if !matches!(file_name, "construction.bytes" | "construction.byte") {
        return Err("Selected file must be construction.bytes or construction.byte.".to_string());
    }
    Ok(path)
}

fn gearblocks_construction_file_record(
    construction_path: &std::path::Path,
) -> Result<GearBlocksConstructionFileRecord, String> {
    let metadata = fs::metadata(construction_path).map_err(|error| error.to_string())?;
    let folder_path = construction_path
        .parent()
        .ok_or_else(|| "Construction file has no parent folder.".to_string())?;
    let name = folder_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("Construction")
        .to_string();
    Ok(GearBlocksConstructionFileRecord {
        name,
        folder_path: folder_path.to_string_lossy().to_string(),
        construction_path: construction_path.to_string_lossy().to_string(),
        byte_size: metadata.len(),
    })
}

fn decode_gearblocks_construction_path(
    construction_path: &std::path::Path,
) -> Result<GearBlocksConstructionDecodeRecord, String> {
    let compressed = fs::read(construction_path).map_err(|error| error.to_string())?;
    let mut decoder = DeflateDecoder::new(compressed.as_slice());
    let mut decoded = Vec::new();
    decoder
        .read_to_end(&mut decoded)
        .map_err(|error| format!("Could not deflate GearBlocks construction bytes: {error}"))?;
    let document = Document::from_reader(decoded.as_slice())
        .map_err(|error| format!("Could not parse GearBlocks construction BSON: {error}"))?;
    let document_json = bson_document_to_json(&document);
    let summary = summarize_gearblocks_construction(&document_json);
    let file_record = gearblocks_construction_file_record(construction_path)?;

    Ok(GearBlocksConstructionDecodeRecord {
        name: file_record.name,
        folder_path: file_record.folder_path,
        construction_path: file_record.construction_path,
        byte_size: compressed.len() as u64,
        decoded_byte_size: decoded.len(),
        summary,
        document: document_json,
    })
}

fn bson_document_to_json(document: &Document) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (key, value) in document.iter() {
        map.insert(key.to_string(), bson_value_to_json(key, value));
    }
    serde_json::Value::Object(map)
}

fn bson_value_to_json(key: &str, value: &Bson) -> serde_json::Value {
    match value {
        Bson::Double(value) => json!(value),
        Bson::String(value) => json!(value),
        Bson::Array(values) => serde_json::Value::Array(
            values
                .iter()
                .enumerate()
                .map(|(index, value)| bson_value_to_json(&index.to_string(), value))
                .collect(),
        ),
        Bson::Document(document) => bson_document_to_json(document),
        Bson::Boolean(value) => json!(value),
        Bson::Null => serde_json::Value::Null,
        Bson::Int32(value) => json!(value),
        Bson::Int64(value) => json!(value),
        Bson::Binary(binary) => bson_binary_to_json(key, &binary.bytes),
        Bson::DateTime(value) => json!(value.to_string()),
        Bson::ObjectId(value) => json!(value.to_string()),
        Bson::RegularExpression(value) => json!(value.to_string()),
        Bson::Timestamp(value) => json!({
            "time": value.time,
            "increment": value.increment
        }),
        Bson::Decimal128(value) => json!(value.to_string()),
        Bson::Undefined => serde_json::Value::Null,
        Bson::MaxKey => json!("MaxKey"),
        Bson::MinKey => json!("MinKey"),
        Bson::DbPointer(value) => json!(format!("{value:?}")),
        Bson::JavaScriptCode(value) => json!(value),
        Bson::JavaScriptCodeWithScope(value) => json!({
            "code": value.code,
            "scope": bson_document_to_json(&value.scope)
        }),
        Bson::Symbol(value) => json!(value),
    }
}

fn bson_binary_to_json(key: &str, bytes: &[u8]) -> serde_json::Value {
    if key.eq_ignore_ascii_case("assetGUID") && bytes.len() == 8 {
        return json!({
            "type": "assetGuid",
            "u64": u64::from_le_bytes(bytes.try_into().unwrap()).to_string(),
            "hex": bytes_to_hex(bytes)
        });
    }
    if matches!(bytes.len(), 12 | 16) {
        return json!({
            "type": if bytes.len() == 12 { "vector3" } else { "quaternion" },
            "values": bytes_to_f32_values(bytes)
        });
    }
    if key.eq_ignore_ascii_case("col") && bytes.len() == 4 {
        return json!({
            "type": "rgba",
            "values": bytes
        });
    }

    json!({
        "type": "binary",
        "byteLength": bytes.len(),
        "hex": bytes_to_hex(bytes)
    })
}

fn bytes_to_f32_values(bytes: &[u8]) -> Vec<f64> {
    bytes
        .chunks_exact(4)
        .map(|chunk| {
            let mut value = [0_u8; 4];
            value.copy_from_slice(chunk);
            f32::from_le_bytes(value) as f64
        })
        .collect()
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join("")
}

fn summarize_gearblocks_construction(
    document: &serde_json::Value,
) -> GearBlocksConstructionSummaryRecord {
    let composites = document.get("composites");
    let part_data = document.get("partData");
    let mut parts = Vec::new();
    let mut unique_asset_guids = std::collections::HashSet::new();

    for (composite_index, composite) in json_collection_values(composites).into_iter().enumerate() {
        let composite_parts = composite.get("parts");
        for (composite_part_index, part) in json_collection_values(composite_parts)
            .into_iter()
            .enumerate()
        {
            let index = parts.len();
            let asset_guid = json_asset_guid(part.get("assetGUID")).unwrap_or_default();
            if !asset_guid.is_empty() {
                unique_asset_guids.insert(asset_guid.clone());
            }
            let part_data_item = json_collection_value_at(part_data, index);
            let dimensions = part_data_item
                .and_then(|value| value.get("dims"))
                .and_then(json_vector_values)
                .unwrap_or_default();
            let behaviours = part_data_item
                .and_then(|value| value.get("behaviours"))
                .map(json_object_keys)
                .unwrap_or_default();

            parts.push(GearBlocksConstructionPartSummaryRecord {
                index,
                composite_index,
                composite_part_index,
                asset_guid,
                dimensions,
                behaviours,
            });
        }
    }

    GearBlocksConstructionSummaryRecord {
        is_frozen: document.get("isFrozen").and_then(|value| value.as_bool()),
        is_invulnerable: document
            .get("isInvulnerable")
            .and_then(|value| value.as_bool()),
        composite_count: json_collection_len(composites),
        part_count: parts.len(),
        unique_asset_guid_count: unique_asset_guids.len(),
        attachment_count: json_collection_len(document.get("attachments")),
        link_count: json_collection_len(document.get("links")),
        intersection_count: json_collection_len(document.get("intersections")),
        parts,
    }
}

fn json_collection_len(value: Option<&serde_json::Value>) -> usize {
    match value {
        Some(serde_json::Value::Array(values)) => values.len(),
        Some(serde_json::Value::Object(values)) => values.len(),
        _ => 0,
    }
}

fn json_collection_values(value: Option<&serde_json::Value>) -> Vec<&serde_json::Value> {
    match value {
        Some(serde_json::Value::Array(values)) => values.iter().collect(),
        Some(serde_json::Value::Object(values)) => {
            let mut pairs = values.iter().collect::<Vec<_>>();
            pairs.sort_by_key(|(key, _)| key.parse::<usize>().unwrap_or(usize::MAX));
            pairs.into_iter().map(|(_, value)| value).collect()
        }
        _ => Vec::new(),
    }
}

fn json_collection_value_at(
    value: Option<&serde_json::Value>,
    index: usize,
) -> Option<&serde_json::Value> {
    match value {
        Some(serde_json::Value::Array(values)) => values.get(index),
        Some(serde_json::Value::Object(values)) => values.get(&index.to_string()),
        _ => None,
    }
}

fn json_asset_guid(value: Option<&serde_json::Value>) -> Option<String> {
    value
        .and_then(|value| value.get("u64"))
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

fn json_vector_values(value: &serde_json::Value) -> Option<Vec<f64>> {
    value.get("values").and_then(|values| {
        values
            .as_array()
            .map(|items| items.iter().filter_map(|item| item.as_f64()).collect())
    })
}

fn json_object_keys(value: &serde_json::Value) -> Vec<String> {
    match value {
        serde_json::Value::Object(map) => map.keys().cloned().collect(),
        _ => Vec::new(),
    }
}

fn overlay_workspace_root() -> Result<PathBuf, String> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .map(PathBuf::from)
        .ok_or_else(|| "Could not resolve Overlay Forge workspace root".to_string())
}

fn screenshot_request_id(timestamp_label: &str) -> String {
    let label = safe_filename_part(timestamp_label);
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    let suffix = format!("{millis}_{}", std::process::id());

    if label.is_empty() {
        suffix
    } else {
        format!("{label}_{suffix}")
    }
}

fn unix_timestamp_label() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn safe_filename_part(value: &str) -> String {
    let mut output = String::new();
    let mut previous_was_separator = false;

    for character in value.trim().chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character);
            previous_was_separator = false;
        } else if matches!(character, '-' | '_') {
            output.push(character);
            previous_was_separator = false;
        } else if !previous_was_separator && !output.is_empty() {
            output.push('_');
            previous_was_separator = true;
        }
    }

    while output.ends_with('_') || output.ends_with('-') {
        output.pop();
    }

    output
}

fn is_supported_catalog_image_extension(extension: &str) -> bool {
    matches!(extension, "png" | "jpg" | "jpeg" | "webp" | "bmp")
}

const GEARBLOCKS_CATALOG_COLUMNS: usize = 8;
const GEARBLOCKS_CATALOG_START_X: u32 = 226;
const GEARBLOCKS_CATALOG_START_Y: u32 = 88;
const GEARBLOCKS_CATALOG_SCROLLED_START_Y: u32 = 187;
const GEARBLOCKS_CATALOG_TILE_WIDTH: u32 = 180;
const GEARBLOCKS_CATALOG_TILE_HEIGHT: u32 = 180;
const GEARBLOCKS_CATALOG_GAP_X: u32 = 9;
const GEARBLOCKS_CATALOG_GAP_Y: u32 = 9;

fn gearblocks_catalog_tile_crops(count: usize, start_y: u32) -> Vec<IconCrop> {
    (0..count)
        .map(|index| {
            let index = index as u32;
            let column = index % GEARBLOCKS_CATALOG_COLUMNS as u32;
            let row = index / GEARBLOCKS_CATALOG_COLUMNS as u32;
            IconCrop {
                x: GEARBLOCKS_CATALOG_START_X
                    + column * (GEARBLOCKS_CATALOG_TILE_WIDTH + GEARBLOCKS_CATALOG_GAP_X),
                y: start_y + row * (GEARBLOCKS_CATALOG_TILE_HEIGHT + GEARBLOCKS_CATALOG_GAP_Y),
                width: GEARBLOCKS_CATALOG_TILE_WIDTH,
                height: GEARBLOCKS_CATALOG_TILE_HEIGHT,
            }
        })
        .collect()
}

fn build_gearblocks_catalog_import_plan(
    source_path: &std::path::Path,
    parts: &[GameRuntimePartRecord],
    first_missing_part_index: usize,
    screenshot_width: u32,
    screenshot_height: u32,
) -> Result<Vec<(usize, IconCrop, Vec<u8>)>, String> {
    let first_missing_row = first_missing_part_index / GEARBLOCKS_CATALOG_COLUMNS;
    let candidate_offsets = if first_missing_part_index == 0 {
        vec![0]
    } else {
        let first_candidate_row = first_missing_row.saturating_sub(4);
        (first_candidate_row..=first_missing_row)
            .map(|row| row * GEARBLOCKS_CATALOG_COLUMNS)
            .collect::<Vec<_>>()
    };
    let candidate_start_ys = if first_missing_part_index == 0 {
        vec![GEARBLOCKS_CATALOG_START_Y]
    } else {
        vec![
            GEARBLOCKS_CATALOG_START_Y,
            GEARBLOCKS_CATALOG_SCROLLED_START_Y,
        ]
    };

    let mut best_plan = Vec::new();
    let mut best_matching_overlap_count = 0_usize;
    for part_index_offset in candidate_offsets {
        for start_y in &candidate_start_ys {
            let mut plan = Vec::new();
            let mut matching_overlap_count = 0_usize;
            for (screen_index, crop) in gearblocks_catalog_tile_crops(parts.len(), *start_y)
                .into_iter()
                .enumerate()
            {
                if !icon_crop_is_within_bounds(crop, screenshot_width, screenshot_height) {
                    continue;
                }

                let part_index = part_index_offset + screen_index;
                let Some(part) = parts.get(part_index) else {
                    continue;
                };
                let cropped_rgba = read_png_crop_rgba(source_path, crop)?;
                if !gearblocks_catalog_crop_looks_complete(&cropped_rgba, crop.width, crop.height) {
                    continue;
                }

                if !part.display_image_path.trim().is_empty() {
                    if gearblocks_catalog_crop_matches_existing_image(
                        &cropped_rgba,
                        crop.width,
                        crop.height,
                        &part.display_image_path,
                    ) {
                        matching_overlap_count += 1;
                    }
                    continue;
                }

                plan.push((part_index, crop, cropped_rgba));
            }

            if matching_overlap_count > best_matching_overlap_count
                || (matching_overlap_count == best_matching_overlap_count
                    && plan.len() > best_plan.len())
            {
                best_matching_overlap_count = matching_overlap_count;
                best_plan = plan;
            }
        }
    }

    if best_plan.is_empty() {
        return Err(
            "No complete catalog icons with visible part-name text were found in the selected screenshot."
                .to_string(),
        );
    }

    Ok(best_plan)
}

fn icon_crop_is_within_bounds(crop: IconCrop, image_width: u32, image_height: u32) -> bool {
    crop.x
        .checked_add(crop.width)
        .is_some_and(|right| right <= image_width)
        && crop
            .y
            .checked_add(crop.height)
            .is_some_and(|bottom| bottom <= image_height)
}

fn png_image_dimensions(source_path: &std::path::Path) -> Result<(u32, u32), String> {
    let file = File::open(source_path).map_err(|error| error.to_string())?;
    let decoder = png::Decoder::new(BufReader::new(file));
    let reader = decoder.read_info().map_err(|error| error.to_string())?;
    let info = reader.info();
    Ok((info.width, info.height))
}

fn gearblocks_catalog_crop_matches_existing_image(
    cropped_rgba: &[u8],
    width: u32,
    height: u32,
    existing_image_path: &str,
) -> bool {
    let path = Path::new(existing_image_path.trim());
    if !path.is_file() {
        return false;
    }

    let Ok((existing_width, existing_height)) = png_image_dimensions(path) else {
        return false;
    };
    if existing_width != width || existing_height != height {
        return false;
    }

    let Ok(existing_rgba) = read_png_crop_rgba(
        path,
        IconCrop {
            x: 0,
            y: 0,
            width,
            height,
        },
    ) else {
        return false;
    };

    gearblocks_catalog_images_are_similar(cropped_rgba, &existing_rgba)
}

fn gearblocks_catalog_images_are_similar(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() || left.len() < 4 {
        return false;
    }

    let mut sampled_pixels = 0_u64;
    let mut total_difference = 0_u64;
    for index in (0..left.len()).step_by(64) {
        if index + 2 >= left.len() {
            break;
        }
        sampled_pixels += 1;
        total_difference += left[index].abs_diff(right[index]) as u64;
        total_difference += left[index + 1].abs_diff(right[index + 1]) as u64;
        total_difference += left[index + 2].abs_diff(right[index + 2]) as u64;
    }

    sampled_pixels > 0 && (total_difference / (sampled_pixels * 3)) <= 18
}

fn gearblocks_runtime_parts_in_catalog_order(
    category: &str,
    parts: Vec<GameRuntimePartRecord>,
) -> Vec<GameRuntimePartRecord> {
    let expected_parts = gearblocks_catalog_part_seeds()
        .into_iter()
        .filter(|seed| seed.category == category)
        .collect::<Vec<_>>();
    if expected_parts.is_empty() {
        return parts;
    }

    let mut remaining = parts;
    let mut ordered = Vec::new();
    for expected in expected_parts {
        let expected_key = normalized_part_name_key(expected.name);
        let expected_asset_key = expected.asset_name.map(normalized_part_name_key);
        let Some(index) = remaining.iter().position(|part| {
            let name_matches = normalized_part_name_key(&part.display_name) == expected_key
                || normalized_part_name_key(&part.full_display_name) == expected_key;
            let asset_matches = expected_asset_key
                .as_ref()
                .is_none_or(|key| normalized_part_name_key(&part.asset_name) == *key);
            name_matches && asset_matches
        }) else {
            continue;
        };
        ordered.push(remaining.remove(index));
    }

    ordered.extend(remaining);
    ordered
}

fn gearblocks_runtime_parts_all_in_catalog_order(
    parts: Vec<GameRuntimePartRecord>,
) -> Vec<GameRuntimePartRecord> {
    let categories = gearblocks_part_categories()
        .into_iter()
        .map(|category| category.name)
        .collect::<Vec<_>>();
    let mut ordered = Vec::new();
    let mut remaining = parts;

    for category in categories {
        let mut category_parts = Vec::new();
        let mut index = 0;
        while index < remaining.len() {
            if remaining[index].category == category {
                category_parts.push(remaining.remove(index));
            } else {
                index += 1;
            }
        }
        ordered.extend(gearblocks_runtime_parts_in_catalog_order(
            category,
            category_parts,
        ));
    }

    ordered.extend(remaining);
    ordered
}

fn normalized_part_name_key(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(|character| character.to_lowercase())
        .collect()
}

fn path_text(path: &std::path::Path) -> String {
    path.to_string_lossy().to_string()
}

struct GearBlocksPartSeed {
    name: &'static str,
    category: &'static str,
    asset_name: Option<&'static str>,
}

#[derive(Clone, Copy)]
struct GearBlocksPartCategory {
    name: &'static str,
    category_icon: &'static str,
    file_hint: &'static str,
    icon_crop: IconCrop,
}

struct ResolvedGearBlocksCategory {
    category_icon: &'static str,
    category_icon_path: String,
    source_path_text: String,
}

#[derive(Clone, Copy)]
struct IconCrop {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

fn gearblocks_part_categories() -> Vec<GearBlocksPartCategory> {
    vec![
        gearblocks_category("Aero", "air", "225806", 39, 84),
        gearblocks_category("Blocks", "blk", "225829", 114, 84),
        gearblocks_category("Bodies", "bod", "225836", 39, 159),
        gearblocks_category("Brakes & Clutches", "brk", "225842", 114, 159),
        gearblocks_category("Checkpoints", "chk", "230153", 39, 234),
        gearblocks_category("Combustion Engines", "eng", "230200", 114, 234),
        gearblocks_category("Connectors", "con", "230214", 39, 309),
        gearblocks_category("Control Wheels", "ctl", "230230", 114, 309),
        gearblocks_category("Electronics", "ele", "230237", 39, 384),
        gearblocks_category("Fuel", "ful", "230254", 114, 384),
        gearblocks_category("Gears", "ger", "230259", 39, 459),
        gearblocks_category("Lights", "lit", "230330", 114, 459),
        gearblocks_category("Linear Actuators", "act", "230338", 39, 534),
        gearblocks_category("Motors", "mot", "230347", 114, 534),
        gearblocks_category("Pipes", "pip", "230354", 39, 609),
        gearblocks_category("Power", "pwr", "230410", 114, 609),
        gearblocks_category("Props", "prp", "230417", 39, 684),
        gearblocks_category("Pulleys", "pul", "230422", 114, 684),
        gearblocks_category("Seats", "sea", "230427", 39, 759),
        gearblocks_category("Suspension", "sus", "230432", 114, 759),
        gearblocks_category("Wheels", "whl", "230438", 39, 834),
    ]
}

fn gearblocks_category(
    name: &'static str,
    category_icon: &'static str,
    file_hint: &'static str,
    x: u32,
    y: u32,
) -> GearBlocksPartCategory {
    GearBlocksPartCategory {
        name,
        category_icon,
        file_hint,
        icon_crop: IconCrop {
            x,
            y,
            width: 75,
            height: 75,
        },
    }
}

fn resolve_gearblocks_categories(
    game_screenshot_dir: &std::path::Path,
    category_icon_dir: &std::path::Path,
) -> Result<HashMap<&'static str, ResolvedGearBlocksCategory>, String> {
    let mut resolved_categories = HashMap::new();

    for category in gearblocks_part_categories() {
        let source_path = find_screenshot_source_path(game_screenshot_dir, category.file_hint);
        let source_path_text = source_path
            .as_ref()
            .map(|path| path_text(path))
            .unwrap_or_default();
        let category_icon_path = source_path
            .as_ref()
            .and_then(|path| {
                ensure_category_icon_crop(
                    path,
                    category_icon_dir,
                    category.name,
                    category.icon_crop,
                )
                .ok()
            })
            .map(|path| path_text(&path))
            .unwrap_or_default();

        resolved_categories.insert(
            category.name,
            ResolvedGearBlocksCategory {
                category_icon: category.category_icon,
                category_icon_path,
                source_path_text,
            },
        );
    }

    Ok(resolved_categories)
}

fn gearblocks_catalog_part_seeds() -> Vec<GearBlocksPartSeed> {
    let mut seeds = Vec::new();
    push_part_seeds(
        &mut seeds,
        "Aero",
        &["Propeller 3 Blade", "Propeller 3 Blade Reversed"],
    );
    push_part_seeds(
        &mut seeds,
        "Blocks",
        &[
            "Angle 3 x 120 Beam",
            "Angle 5 x 72 Beam",
            "Angle 9 x 40 Beam",
            "Angle 120 Beam",
            "Angle 135 Beam",
            "Angle 150 Beam",
            "Angle 157.5 Beam",
            "Half Rounded Beam",
            "Beam",
            "Rounded Beam",
            "Scaffold Beam",
            "Block",
            "Circle Plate",
            "Cylinder",
            "Gusset x1",
            "Corrugated Plate 9x25",
            "L-Plate",
            "Labelled Plate",
            "Plate",
            "U-Plate",
            "Sloped Beam",
            "Sloped Beam Plate",
            "Sloped Plate",
            "Sphere",
            "Offset Tile 1x2",
            "Offset Tile 2x2",
            "V 60 Beam",
            "V 90 Beam",
            "W 45 Beam",
            "Wedge Plate",
        ],
    );
    push_part_seeds(
        &mut seeds,
        "Bodies",
        &[
            "Dummy Lower Left Arm",
            "Dummy Upper Left Arm",
            "Dummy Lower Right Arm",
            "Dummy Upper Right Arm",
            "Dummy Head",
            "Dummy Lower Left Leg",
            "Dummy Upper Left Leg",
            "Dummy Lower Right Leg",
            "Dummy Upper Right Leg",
            "Dummy Lower Torso",
            "Dummy Upper Torso",
            "Male Lower Left Arm",
            "Male Upper Left Arm",
            "Male Lower Right Arm",
            "Male Upper Right Arm",
            "Male Hair",
            "Male Head",
            "Male Lower Left Leg",
            "Male Upper Left Leg",
            "Male Lower Right Leg",
            "Male Upper Right Leg",
            "Male Lower Torso",
            "Male Upper Torso",
            "Racing Helmet",
        ],
    );
    push_part_seeds(
        &mut seeds,
        "Brakes & Clutches",
        &[
            "Disk Brake x3",
            "Disk Brake x4",
            "Centrifugal Clutch & Ring Gear x3 (24T)",
            "Centrifugal Clutch x2",
            "Centrifugal Clutch x3",
            "Clutch & Ring Gear x3 (24T)",
            "Clutch & Ring Gear x4 (32T)",
            "Clutch x3",
            "Ratchet (Axle to Axle)",
            "Ratchet (Block to Axle)",
        ],
    );
    push_part_seeds(
        &mut seeds,
        "Checkpoints",
        &["Box Checkpoint", "Cylinder Checkpoint"],
    );
    push_part_seeds(
        &mut seeds,
        "Combustion Engines",
        &[
            "Engine Crank Nose & Axle",
            "Engine Rear (Driven) Crank x1 & Axle",
            "Engine Rear (Driven) Crank x2 & Axle",
            "Engine Crank x1",
            "Engine Crank x2",
            "Engine Cylinder 1x1 0.7L",
            "Engine Cylinder 1x1 0.7L (Transparent)",
            "Engine Cylinder 2x2 2L",
            "Engine Cylinder 2x2 2L (Transparent)",
            "Engine Head x1",
            "Engine Head x2",
            "Engine Throttle x1",
            "4-Blade Fan x3",
            "7-Blade Fan x4",
            "Air-cooled Fan x3",
        ],
    );
    push_part_seeds(
        &mut seeds,
        "Connectors",
        &[
            "Axle",
            "Scaffold Axle",
            "1-Hole & Axle",
            "1-Hole & Plate (H)",
            "1-Hole & Slider (H)",
            "1-Hole & Plate (V)",
            "1-Hole & Slider (V)",
            "2-Hole & Axle (Perp)",
            "2-Hole & Axle",
            "2-Hole & Plate (H)",
            "2-Hole & Slider (H)",
            "2-Hole & Plate (V)",
            "2-Hole & Slider (V)",
            "Angle 0",
            "Angle 3 x 90",
            "Angle 3 x 120",
            "Angle 4 x 90",
            "Angle 90",
            "Angle 120",
            "Angle 135",
            "Angle 150",
            "Angle 157.5",
            "Angle 180",
            "Angle Axle 3 x 90",
            "Angle Axle 3 x 120",
            "Angle Axle 4 x 90",
            "Angle Axle 90",
            "Angle Axle 120",
            "Angle Axle 135",
            "Angle Axle 150",
            "Angle Axle 157.5",
            "Angle Axle 180",
            "Angle Limiter (Axle to Axle)",
            "Angle Limiter (Block to Axle)",
            "Ball",
            "Ball & Axle",
            "CV Joint (Inner)",
            "CV Joint (Inner) & Axle",
            "CV Joint (Outer)",
            "CV Joint (Outer) & Axle",
            "Knuckle Joint (Inner)",
            "Knuckle Joint (Inner) & Axle",
            "Knuckle Joint (Outer 90)",
            "Knuckle Joint (Outer 90) & Axle",
            "Knuckle Joint (Outer 180)",
            "Knuckle Joint (Outer 180) & Axle",
            "Offset 3-Hole x3",
            "Offset 3-Hole x5",
            "Socket (H)",
            "Socket & Axle (H)",
            "Socket (V)",
            "Socket & Axle (V)",
            "Pin",
            "2-Plate & Axle",
            "Plate & Axle",
            "U-Joint Yoke & Axle",
            "Rotor 2",
            "Rotor 3",
            "Rotor 4",
            "Slider Rail",
        ],
    );
    push_part_seeds(
        &mut seeds,
        "Control Wheels",
        &[
            "Hand Wheel x3",
            "Hand Wheel x5",
            "Sports Steering Wheel x4",
            "Steering Wheel x4",
        ],
    );
    push_part_seeds(
        &mut seeds,
        "Electronics",
        &[
            "Joystick Control",
            "Lever Control",
            "Rotary Knob Control",
            "Slider Control",
            "1 Line Display 3x1",
            "2 Line Display 3x1",
            "2 Line Display 5x2",
            "2 Line Display 5x1",
            "2 Line Display 9x2",
            "4 Line Display 5x2",
            "4 Line Display 9x4",
            "Joypad (Dual Axis)",
            "Keypad (1 Key)",
            "Keypad (4 Keys)",
            "Keypad (9 Keys)",
            "Edge Detector",
            "Boolean Operator",
            "Boolean Multi-Operator",
            "Pulse Generator",
            "Boolean Toggle",
            "Number Calculus",
            "Number Comparator",
            "Number Expression",
            "Number Filter",
            "Number Formatter",
            "Number Junction",
            "Number Multi-Junction",
            "Number Operator",
            "Number Register",
            "Number Selector",
            "Number Multi-Selector",
            "PID Controller",
            "String Selector",
            "String Multi-Selector",
            "Timer",
            "Accelerometer Sensor",
            "Angle Sensor",
            "Attitude Sensor",
            "Contact Pad Sensor",
            "Distance Sensor 250m",
            "Distance Sensor 50m",
            "GPS Receiver",
            "Inertial Sensor",
            "Proximity Sensor 100m",
            "Proximity Sensor 20m",
            "Speed & Altitude Sensor",
            "Clock",
            "Button Switch",
            "Rocker Switch",
            "Toggle Switch",
        ],
    );
    push_part_seeds(
        &mut seeds,
        "Fuel",
        &[
            "Fuel Tank 9 Litre",
            "Fuel Tank 70 Litre",
            "Fuel Tank 375 Litre",
        ],
    );
    push_part_seeds(
        &mut seeds,
        "Gears",
        &[
            "Bevel Gear Hi x2 (16T)",
            "Bevel Gear Hi x3 (24T)",
            "Bevel Gear Hi x4 (32T)",
            "Bevel Gear Hi x5 (40T)",
            "Bevel Gear Hi x6 (48T)",
            "Bevel Gear Lo x2 (16T)",
            "Bevel Gear Lo x3 (24T)",
            "Bevel Gear Lo x4 (32T)",
            "Bevel Gear Lo x5 (40T)",
            "Bevel Gear Lo x6 (48T)",
            "Crown Gear Hi x2 (16T)",
            "Crown Gear Hi x3 (24T)",
            "Crown Gear Hi x4 (32T)",
            "Crown Gear Hi x5 (40T)",
            "Crown Gear Hi x6 (48T)",
            "Crown Gear Lo x2 (16T)",
            "Crown Gear Lo x3 (24T)",
            "Crown Gear Lo x4 (32T)",
            "Crown Gear Lo x5 (40T)",
            "Crown Gear Lo x6 (48T)",
            "Differential Crown Gear (32T)",
            "Differential Crown Gear (48T)",
            "Differential Spur Gear (32T)",
            "Differential Spur Gear (48T)",
            "Rack Gear 2-Ball & Slider x3",
            "Rack Gear 2-Ball & Slider x5",
            "Rack Gear 2-Ball & Slider x7",
            "Rack Gear 2-Hole & Slider x3",
            "Rack Gear 2-Hole & Slider x5",
            "Rack Gear 2-Hole & Slider x7",
            "Rack Gear 2-Hole & Slider x13",
            "Rack Gear x3",
        ],
    );
    push_part_seeds(
        &mut seeds,
        "Gears",
        &[
            "Rack Gear x7",
            "Spur Gear x1 (8T)",
            "Spur Gear x1.25 (10T)",
            "Spur Gear x1.5 (12T)",
            "Spur Gear x1.75 (14T)",
            "Spur Gear x2 (16T)",
            "Spur Gear x2.25 (18T)",
            "Spur Gear x2.5 (20T)",
            "Spur Gear x2.75 (22T)",
            "Spur Gear x3 (24T)",
            "Spur Gear x3.5 (28T)",
            "Spur Gear x4 (32T)",
            "Spur Gear x4.5 (36T)",
            "Spur Gear x5 (40T)",
            "Spur Gear x6 (48T)",
            "Spur Gear x7 (56T)",
            "Spur Gear x8 (64T)",
            "Spur Gear x9 (72T)",
            "Clutch Gear x1 (8T)",
            "Clutch Gear x1.25 (10T)",
            "Clutch Gear x1.5 (12T)",
            "Clutch Gear x1.75 (14T)",
            "Clutch Gear x2 (16T)",
            "Clutch Gear x2.25 (18T)",
            "Clutch Gear x2.5 (20T)",
            "Clutch Gear x2.75 (22T)",
            "Clutch Gear x3 (24T)",
            "Ratchet Gear x1 (8T)",
            "Ratchet Gear x1.5 (12T)",
            "Ratchet Gear x2 (16T)",
            "Ratchet Gear x2.5 (20T)",
            "Ratchet Gear x3 (24T)",
            "Worm Gear CCW x1",
            "Worm Gear CCW x3",
            "Worm Gear CCW x7",
            "Worm Gear CW x1",
            "Worm Gear CW x3",
            "Worm Gear CW x7",
        ],
    );
    push_part_seeds(
        &mut seeds,
        "Lights",
        &[
            "Cone Light x1",
            "Rectangular Light 1x1",
            "Rectangular Light 1x2",
            "Rectangular Light 2x2",
            "Upright Rectangular Light x1.5",
            "Upright Rectangular Light x2",
            "Upright Round Light x1.5",
            "Upright Round Light x2",
        ],
    );
    push_part_seeds(
        &mut seeds,
        "Linear Actuators",
        &[
            "Linear Actuator (Barrel) Large",
            "Linear Actuator (Piston) Large",
            "Linear Actuator (Barrel) Large Long",
            "Linear Actuator (Piston) Large Long",
            "Linear Actuator (Barrel) Medium",
            "Linear Actuator (Piston) Medium",
            "Linear Actuator (Barrel) Small",
            "Linear Actuator (Piston) Small",
        ],
    );
    push_part_seeds(
        &mut seeds,
        "Motors",
        &[
            "Electric Motor Large",
            "Electric Motor Medium",
            "Electric Motor Small",
            "Continuous Servo Motor Medium",
            "Continuous Servo Motor Small",
            "Servo Motor Medium",
            "Servo Motor Small",
            "Starter Motor Small",
            "Stepper Motor Medium",
            "Stepper Motor Small",
        ],
    );
    push_part_seeds(
        &mut seeds,
        "Pipes",
        &[
            "Clamped Pipe",
            "Corner Pipe",
            "Corner 90 Pipe",
            "Small Corner 90 Pipe",
            "Small Corner Pipe",
            "Cross Pipe",
            "Small Cross Pipe",
            "Straight Pipe",
            "Tee Pipe",
            "Tee 90 Pipe",
            "Small Tee 90 Pipe",
            "Small Tee Pipe",
        ],
    );
    push_part_seeds(
        &mut seeds,
        "Power",
        &[
            "Battery 0.5 kWh",
            "Battery 1.25 kWh",
            "Battery 2 kWh",
            "Battery 50 kWh",
            "Battery 200 kWh",
            "Alternator Medium",
            "Solar Panel 9x5",
            "Solar Panel 15x9",
            "Solar Panel 25x15",
        ],
    );
    push_part_seeds(
        &mut seeds,
        "Props",
        &["Football", "Concrete Traffic Barrier", "Traffic Cone"],
    );
    push_part_seeds(
        &mut seeds,
        "Pulleys",
        &[
            "Pulley x1",
            "Pulley x1.5",
            "Pulley x2",
            "Pulley x2.5",
            "Pulley x3",
            "Pulley x4",
            "Pulley x5",
            "Pulley x6",
            "Pulley x7",
            "Differential Pulley x4",
            "Differential Pulley x6",
        ],
    );
    push_part_seeds(
        &mut seeds,
        "Seats",
        &[
            "Car Seat",
            "Go-kart Seat",
            "Racing Seat",
            "Porcelain Throne",
            "Vintage Seat",
        ],
    );
    push_part_seeds(
        &mut seeds,
        "Suspension",
        &[
            "Control Arm 1x4",
            "Control Arm 1x5",
            "Control Arm 1x6",
            "Control Arm 3x5",
            "Control Arm 3x6",
            "Control Arm 3x7",
        ],
    );
    push_part_seed_variants(
        &mut seeds,
        "Suspension",
        &[
            ("Coil-over (Barrel) Large", "SpringDamper Ball Large Barrel"),
            ("Coil-over (Piston) Large", "SpringDamper Ball Large Piston"),
            (
                "Coil-over (Barrel) Medium",
                "SpringDamper Ball Medium Barrel",
            ),
            (
                "Coil-over (Piston) Medium",
                "SpringDamper Ball Medium Piston",
            ),
            ("Coil-over (Barrel) Small", "SpringDamper Ball Small Barrel"),
            ("Coil-over (Piston) Small", "SpringDamper Ball Small Piston"),
            (
                "Coil-over (Barrel) Small Strong",
                "SpringDamper Ball Small Strong Barrel",
            ),
            (
                "Coil-over (Piston) Small Strong",
                "SpringDamper Ball Small Strong Piston",
            ),
            ("Coil-over (Barrel) Large", "SpringDamper Large Barrel"),
            ("Coil-over (Piston) Large", "SpringDamper Large Piston"),
            ("Coil-over (Barrel) Medium", "SpringDamper Medium Barrel"),
            ("Coil-over (Piston) Medium", "SpringDamper Medium Piston"),
            ("Coil-over (Barrel) Small", "SpringDamper Small Barrel"),
            ("Coil-over (Piston) Small", "SpringDamper Small Piston"),
            (
                "Coil-over (Barrel) Small Strong",
                "SpringDamper Small Strong Barrel",
            ),
            (
                "Coil-over (Piston) Small Strong",
                "SpringDamper Small Strong Piston",
            ),
        ],
    );
    push_part_seeds(
        &mut seeds,
        "Suspension",
        &[
            "Steering Arm 1-Ball 1-Axle x4",
            "Steering Arm 2-Axle x4",
            "Steering Arm 2-Ball x4",
            "Steering Arm 3-Ball x4",
            "Steering Arm 3x4 A",
            "Steering Arm 3x4 B",
            "Steering Arm 3x5 A",
            "Steering Arm 3x5 B",
            "Torsion Spring (Axle to Axle)",
            "Torsion Spring (Block to Axle)",
        ],
    );
    push_part_seeds(
        &mut seeds,
        "Wheels",
        &[
            "Aircraft Wheel 2.5x9",
            "Aircraft Wheel 2x6",
            "Car Wheel 2.5x6.5",
            "Car Wheel 2.5x7",
            "Car Wheel 2x6.5",
            "Car Wheel 2x7",
            "Car Wheel 2x8",
            "Car Wheel 3x6.5",
            "Car Wheel 3x7",
            "Car Wheel 3x8",
            "Car Wheel 4x8",
            "Go-kart Wheel 2.5x4",
            "Go-kart Wheel 2x5",
            "Motorcycle Wheel 1x8",
            "Off-road Wheel 5.5x11",
            "Off-road Wheel 5x15",
            "Off-road Wheel 5x18",
            "Off-road Wheel 10x16",
            "Racing Wheel 4x8",
            "Racing Wheel 5x8",
            "Trolley Wheel 1x3",
            "Truck Wheel 2.5x9",
            "Truck Wheel 3x11",
        ],
    );
    seeds
}

fn push_part_seeds(
    seeds: &mut Vec<GearBlocksPartSeed>,
    category: &'static str,
    names: &[&'static str],
) {
    for name in names {
        seeds.push(GearBlocksPartSeed {
            name,
            category,
            asset_name: None,
        });
    }
}

fn push_part_seed_variants(
    seeds: &mut Vec<GearBlocksPartSeed>,
    category: &'static str,
    variants: &[(&'static str, &'static str)],
) {
    for (name, asset_name) in variants {
        seeds.push(GearBlocksPartSeed {
            name,
            category,
            asset_name: Some(asset_name),
        });
    }
}

fn find_screenshot_source_path(
    game_screenshot_dir: &std::path::Path,
    file_hint: &str,
) -> Option<PathBuf> {
    let entries = fs::read_dir(game_screenshot_dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("png") {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if file_name.contains(file_hint) {
            return Some(path);
        }
    }
    None
}

fn existing_category_icon_path(
    category_icon_dir: &std::path::Path,
    category: &str,
) -> Option<PathBuf> {
    let path = category_icon_dir.join(format!("{}.png", safe_tag_part(category)));
    path.is_file().then_some(path)
}

fn ensure_category_icon_crop(
    source_path: &std::path::Path,
    output_dir: &std::path::Path,
    category: &str,
    crop: IconCrop,
) -> Result<PathBuf, String> {
    let output_path = output_dir.join(format!("{}.png", safe_tag_part(category)));
    crop_png_region(source_path, &output_path, crop)?;
    Ok(output_path)
}

fn crop_png_region(
    source_path: &std::path::Path,
    output_path: &std::path::Path,
    crop: IconCrop,
) -> Result<(), String> {
    let rgba = read_png_crop_rgba(source_path, crop)?;
    write_rgba_png(output_path, crop.width, crop.height, &rgba)
}

fn read_png_crop_rgba(source_path: &std::path::Path, crop: IconCrop) -> Result<Vec<u8>, String> {
    let file = File::open(source_path).map_err(|error| error.to_string())?;
    let decoder = png::Decoder::new(BufReader::new(file));
    let mut reader = decoder.read_info().map_err(|error| error.to_string())?;
    let output_buffer_size = reader
        .output_buffer_size()
        .ok_or_else(|| "Could not determine screenshot PNG buffer size.".to_string())?;
    let mut buffer = vec![0; output_buffer_size];
    let info = reader
        .next_frame(&mut buffer)
        .map_err(|error| error.to_string())?;
    let bytes = &buffer[..info.buffer_size()];
    let channels = match info.color_type {
        png::ColorType::Rgba => 4,
        png::ColorType::Rgb => 3,
        _ => return Err("Only RGB and RGBA screenshot PNGs can be cropped.".to_string()),
    };

    if crop.x + crop.width > info.width || crop.y + crop.height > info.height {
        return Err("Category icon crop is outside the screenshot bounds.".to_string());
    }

    let mut rgba = Vec::with_capacity((crop.width * crop.height * 4) as usize);
    for y in crop.y..(crop.y + crop.height) {
        for x in crop.x..(crop.x + crop.width) {
            let index = ((y * info.width + x) as usize) * channels;
            rgba.push(bytes[index]);
            rgba.push(bytes[index + 1]);
            rgba.push(bytes[index + 2]);
            rgba.push(255);
        }
    }

    Ok(rgba)
}

fn gearblocks_catalog_crop_looks_complete(rgba: &[u8], width: u32, height: u32) -> bool {
    if width < 80 || height < 100 {
        return false;
    }
    if gearblocks_catalog_crop_has_internal_separator(rgba, width, height) {
        return false;
    }

    let footer_top = height.saturating_sub(70);
    let footer_bottom = height.saturating_sub(8);
    let left = 8;
    let right = width.saturating_sub(8);
    let mut blue_footer_pixels = 0_u32;
    let mut bright_footer_pixels = 0_u32;
    let mut sampled_pixels = 0_u32;

    for y in footer_top..footer_bottom {
        for x in left..right {
            let index = ((y * width + x) as usize) * 4;
            if index + 2 >= rgba.len() {
                continue;
            }
            sampled_pixels += 1;
            let red = rgba[index];
            let green = rgba[index + 1];
            let blue = rgba[index + 2];

            if blue > 145 && green > 95 && red < 120 {
                blue_footer_pixels += 1;
            }
            if red > 205 && green > 205 && blue > 205 {
                bright_footer_pixels += 1;
            }
        }
    }

    sampled_pixels > 0 && blue_footer_pixels < sampled_pixels / 2 && bright_footer_pixels >= 350
}

fn gearblocks_catalog_crop_has_internal_separator(rgba: &[u8], width: u32, height: u32) -> bool {
    let start_y = height / 3;
    let end_y = (height * 2) / 3;
    let required_pixels = (width * 3) / 4;

    for y in start_y..end_y {
        let mut separator_pixels = 0_u32;
        for x in 0..width {
            let index = ((y * width + x) as usize) * 4;
            if index + 2 >= rgba.len() {
                continue;
            }
            let red = rgba[index];
            let green = rgba[index + 1];
            let blue = rgba[index + 2];

            if (40..=95).contains(&red) && (55..=130).contains(&green) && (85..=190).contains(&blue)
            {
                separator_pixels += 1;
            }
        }

        if separator_pixels >= required_pixels {
            return true;
        }
    }

    false
}

fn practical_part_description(name: &str, category: &str) -> String {
    match category {
        "Aero" => format!(
            "{name} is useful for moving air, generating thrust, or studying lift and drag where blade direction, pitch, and rotational speed affect vehicle behavior."
        ),
        "Blocks" => format!(
            "{name} is useful as a rigid structural element for frames, brackets, mounting surfaces, spacers, and load paths where forces need to transfer cleanly through an assembly."
        ),
        "Bodies" => format!(
            "{name} is useful for approximating driver, passenger, or payload geometry so mass distribution, clearance, restraints, and impact zones can be reasoned about in a physical build."
        ),
        "Brakes & Clutches" => format!(
            "{name} is useful for controlling rotational energy, coupling or decoupling drivetrains, and converting motion into heat or staged torque transfer."
        ),
        "Checkpoints" => format!(
            "{name} is useful as a spatial trigger for validating paths, measuring route completion, or marking target volumes during motion tests."
        ),
        "Combustion Engines" => format!(
            "{name} is useful for modeling internal-combustion power delivery where crank geometry, cylinder layout, airflow, and cooling affect torque and reliability."
        ),
        "Connectors" => format!(
            "{name} is useful for pivots, hinges, steering links, rotating shafts, and constrained motion where torque or linear force needs to move between two mechanical points."
        ),
        "Control Wheels" => format!(
            "{name} is useful as a human input surface for steering, trim, or manual control where rotational motion maps to a mechanism or signal."
        ),
        "Electronics" => format!(
            "{name} is useful for control loops, input handling, signal routing, threshold behavior, feedback displays, and automation logic that turns mechanical motion into repeatable behavior."
        ),
        "Fuel" => format!(
            "{name} is useful for storing consumable energy mass, planning range, and understanding how tank placement changes center of gravity as fuel is used."
        ),
        "Gears" => format!(
            "{name} is useful for changing torque, speed, direction, or mechanical advantage between rotating shafts in drivetrain, steering, lifting, and timing mechanisms."
        ),
        "Lights" => format!(
            "{name} is useful for visual signaling, orientation, status indication, and making machine state easier to interpret during motion or low-visibility testing."
        ),
        "Linear Actuators" => format!(
            "{name} is useful for producing controlled straight-line force in steering, lifting, landing gear, locking, and positioning mechanisms."
        ),
        "Motors" => format!(
            "{name} is useful for converting electrical energy into rotation, servo positioning, or stepwise motion in drivetrains and automated mechanisms."
        ),
        "Pipes" => format!(
            "{name} is useful for routing fluids or gases, building manifolds, and laying out flow paths where bends, tees, and restrictions affect pressure and packaging."
        ),
        "Power" => format!(
            "{name} is useful for supplying, converting, or harvesting electrical energy where capacity, generation rate, and placement affect machine endurance."
        ),
        "Props" => format!(
            "{name} is useful as environmental mass, obstacles, payload, or test geometry for collision, clearance, and stability experiments."
        ),
        "Pulleys" => format!(
            "{name} is useful for belt routing, torque transfer, speed ratio changes, and differential mechanisms where wrap angle and pulley size affect grip."
        ),
        "Seats" => format!(
            "{name} is useful for locating operators, payload mass, and ergonomic reference points that influence visibility, clearance, and center of gravity."
        ),
        "Suspension" => format!(
            "{name} is useful for controlling wheel movement, absorbing impacts, preserving tire contact, and tuning camber, toe, ride height, and roll behavior."
        ),
        "Wheels" => format!(
            "{name} is useful for ground contact, rolling resistance, traction, suspension tuning, and load support where diameter, width, and tire profile affect stability and grip."
        ),
        _ => format!(
            "{name} is useful as a cataloged build object for practical testing, fit checks, and physics-driven assembly planning."
        ),
    }
}

fn safe_tag_part(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn remove_screenshot_file_if_present(
    path_text: &str,
    screenshots_root: &std::path::Path,
) -> Result<(), String> {
    let trimmed_path = path_text.trim();
    if trimmed_path.is_empty() {
        return Ok(());
    }

    let path = PathBuf::from(trimmed_path);
    if !path.exists() {
        return Ok(());
    }

    let canonical_root = fs::canonicalize(screenshots_root).map_err(|error| error.to_string())?;
    let canonical_path = fs::canonicalize(&path).map_err(|error| error.to_string())?;
    if !canonical_path.starts_with(&canonical_root) {
        return Err("Refusing to delete a screenshot file outside game-screenshots.".to_string());
    }
    if !canonical_path.is_file() {
        return Err("Screenshot cleanup expected a file path.".to_string());
    }

    fs::remove_file(canonical_path).map_err(|error| error.to_string())
}

#[cfg(target_os = "windows")]
fn capture_foreground_window_to_png(path: &std::path::Path) -> Result<(), String> {
    use std::mem;
    use std::ptr;
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::Graphics::Gdi::{
        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC,
        GetDIBits, ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CAPTUREBLT,
        DIB_RGB_COLORS, HGDIOBJ, RGBQUAD, SRCCOPY,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowRect};

    let (width, height, bgra) = unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return Err("No foreground window was available to capture.".to_string());
        }

        let mut rect: RECT = mem::zeroed();
        if GetWindowRect(hwnd, &mut rect) == 0 {
            return Err("Could not read the foreground window bounds.".to_string());
        }

        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        if width <= 0 || height <= 0 {
            return Err("Foreground window bounds are empty.".to_string());
        }

        let screen_dc = GetDC(ptr::null_mut());
        if screen_dc.is_null() {
            return Err("Could not acquire the screen device context.".to_string());
        }

        let memory_dc = CreateCompatibleDC(screen_dc);
        if memory_dc.is_null() {
            ReleaseDC(ptr::null_mut(), screen_dc);
            return Err("Could not create a memory device context.".to_string());
        }

        let bitmap = CreateCompatibleBitmap(screen_dc, width, height);
        if bitmap.is_null() {
            DeleteDC(memory_dc);
            ReleaseDC(ptr::null_mut(), screen_dc);
            return Err("Could not create a compatible capture bitmap.".to_string());
        }

        let previous_object = SelectObject(memory_dc, bitmap as HGDIOBJ);
        let blit_ok = BitBlt(
            memory_dc,
            0,
            0,
            width,
            height,
            screen_dc,
            rect.left,
            rect.top,
            SRCCOPY | CAPTUREBLT,
        ) != 0;

        let mut bitmap_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                biSizeImage: (width * height * 4) as u32,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [RGBQUAD {
                rgbBlue: 0,
                rgbGreen: 0,
                rgbRed: 0,
                rgbReserved: 0,
            }],
        };
        let mut bgra = vec![0u8; (width as usize) * (height as usize) * 4];
        let copied_lines = if blit_ok {
            GetDIBits(
                memory_dc,
                bitmap,
                0,
                height as u32,
                bgra.as_mut_ptr().cast(),
                &mut bitmap_info,
                DIB_RGB_COLORS,
            )
        } else {
            0
        };

        if !previous_object.is_null() {
            SelectObject(memory_dc, previous_object);
        }
        DeleteObject(bitmap as HGDIOBJ);
        DeleteDC(memory_dc);
        ReleaseDC(ptr::null_mut(), screen_dc);

        if !blit_ok {
            return Err("Windows GDI BitBlt capture failed.".to_string());
        }
        if copied_lines == 0 {
            return Err("Windows GDI capture did not return bitmap pixels.".to_string());
        }

        (width as u32, height as u32, bgra)
    };

    let mut rgba = Vec::with_capacity(bgra.len());
    for pixel in bgra.chunks_exact(4) {
        rgba.push(pixel[2]);
        rgba.push(pixel[1]);
        rgba.push(pixel[0]);
        rgba.push(255);
    }

    write_rgba_png(path, width, height, &rgba)
}

#[cfg(not(target_os = "windows"))]
fn capture_foreground_window_to_png(_path: &std::path::Path) -> Result<(), String> {
    Err("Windows GDI screenshot capture is only available on Windows.".to_string())
}

fn write_rgba_png(
    path: &std::path::Path,
    width: u32,
    height: u32,
    rgba: &[u8],
) -> Result<(), String> {
    let file = File::create(path).map_err(|error| error.to_string())?;
    let writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut png_writer = encoder.write_header().map_err(|error| error.to_string())?;
    png_writer
        .write_image_data(rgba)
        .map_err(|error| error.to_string())
}
