use crate::db::{
    BridgeFileDraftRecord, CalendarEventRecord, CalendarEventUpdateDraft, GameBuildGuideDraft,
    GameBuildGuidePartDraft, GameBuildGuidePartRecord, GameBuildGuideRecord,
    GameBuildGuideStepDraft, GameBuildGuideStepRecord, GameCatalogObjectDraft,
    GameCatalogObjectRecord, GameCatalogReferenceDraft, GameChatConversationRecord,
    GameChatMessageRecord, GameConstructionDraft, GameConstructionRecord, GameDataLocationRecord,
    GameRecord, GameRuntimeConstructionExportDraft, GameRuntimeConstructionExportRecord,
    GameRuntimePartAliasDraft, GameRuntimePartAliasRecord, GameRuntimePartApiAttributeObservation,
    GameRuntimePartApiMemberObservation, GameRuntimePartApiMemberRecord,
    GameRuntimePartAttachmentObservation, GameRuntimePartAttachmentTypeDraft, GameRuntimePartDraft,
    GameRuntimePartIdentity, GameRuntimePartInstanceDraft, GameRuntimePartInstanceRecord,
    GameRuntimePartMetadataValueDraft, GameRuntimePartOutputChannelValueDraft,
    GameRuntimePartPropertyObservation, GameRuntimePartRecord, GameRuntimePartSettingValueDraft,
    GameRuntimePartSource, GameRuntimePartValueObservation, GameScreenshotCaptureRequestDraft,
    GameScreenshotCaptureRequestRecord, GameSettingRecord, GearBlocksApiCatalogRecord, NoteRecord,
    PlanningConversationContextRecord, PlanningConversationRecord, PlanningMessageRecord,
    PlanningPromptPreviewRecord, ProjectGitHubRepositoryRecord, ProjectMarkdownContextPayload,
    ProjectMarkdownContextRecord, ProjectRecord, SchedulerRecord, SmokingCessationSettingsRecord,
    SmokingEventRecord, TaskRecord, YouTubeReferenceRecord, YouTubeReferenceUpdateDraft,
};
use crate::gearblocks_api_scraper::{scrape_official_gearblocks_api, GearBlocksApiImportResult};
use crate::gearblocks_scene_context::GearBlocksSceneContextService;
use crate::github;
use crate::hotkeys;
use crate::lifecycle;
use crate::openai;
use crate::windows::{self, StandaloneWindowConfig, WindowKind, WindowManager};
use crate::{AppState, GameBuildGuideOverlaySelection, GameChatOverlaySelection};
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use bson::{Bson, Document};
use flate2::read::DeflateDecoder;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};

static GEARBLOCKS_RUNTIME_IMPORT_MONITOR_ACTIVE: AtomicBool = AtomicBool::new(false);
static SCHEDULER_WORKER_ACTIVE: AtomicBool = AtomicBool::new(false);
const ACTIVE_GAME_BUILD_GUIDE_OVERLAY_SETTING: &str = "active_game_build_guide_overlay_v1";
const GEARBLOCKS_RUNTIME_INITIAL_TAIL_BYTES: u64 = 2 * 1024 * 1024;
const GEARBLOCKS_RUNTIME_INCREMENTAL_READ_LIMIT_BYTES: u64 = 2 * 1024 * 1024;
const BUILD_GUIDE_SOURCE_HTML_MAX_BYTES: u64 = 4 * 1024 * 1024;
const BUILD_GUIDE_SOURCE_TEXT_MAX_CHARS: usize = 60_000;
const BUILD_GUIDE_SOURCE_IMAGE_MAX_COUNT: usize = 24;
const BUILD_GUIDE_SOURCE_IMAGE_MAX_BYTES: u64 = 10 * 1024 * 1024;

#[derive(Serialize)]
pub struct MilestoneStatus {
    milestone: String,
    hotkey: String,
    #[serde(rename = "databaseReady")]
    database_ready: bool,
}

#[derive(Serialize)]
pub struct SmokingCessationExportRecord {
    #[serde(rename = "exportPath")]
    pub export_path: String,
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEventUpdateInput {
    pub id: i64,
    pub title: String,
    pub start_date: String,
    pub start_time: String,
    pub end_date: String,
    pub end_time: String,
    pub notes: String,
}

#[derive(Serialize)]
pub struct GameBuildGuidePayload {
    pub guide: GameBuildGuideRecord,
    pub parts: Vec<GameBuildGuidePartRecord>,
    pub steps: Vec<GameBuildGuideStepRecord>,
    pub checklist: Vec<String>,
    #[serde(rename = "imageReferenceCount")]
    pub image_reference_count: usize,
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
pub struct GearBlocksThirdPartyDependencyStatusRecord {
    pub name: String,
    #[serde(rename = "isDetected")]
    pub is_detected: bool,
    #[serde(rename = "isInstalledCorrectly")]
    pub is_installed_correctly: Option<bool>,
    #[serde(rename = "isActivated")]
    pub is_activated: Option<bool>,
    #[serde(rename = "installedVersion")]
    pub installed_version: Option<String>,
    #[serde(rename = "expectedPath")]
    pub expected_path: String,
    pub detail: String,
    #[serde(rename = "statusDetails")]
    pub status_details: Vec<String>,
    #[serde(rename = "logPaths")]
    pub log_paths: Vec<String>,
    #[serde(rename = "projectUrl")]
    pub project_url: String,
}

#[derive(Serialize)]
pub struct GearBlocksThirdPartyDependencyStatusPayload {
    #[serde(rename = "gameRoot")]
    pub game_root: String,
    pub dependencies: Vec<GearBlocksThirdPartyDependencyStatusRecord>,
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

#[derive(Clone)]
struct GearBlocksPartAliasLogRecord {
    part_instance_key: String,
    friendly_name: String,
    emitted_at: String,
    source_log_path: String,
    document: serde_json::Value,
}

struct BuildGuideSourceDocument {
    url: String,
    title: String,
    text: String,
    images: Vec<BuildGuideSourceImage>,
}

#[derive(Clone)]
struct BuildGuideSourceImage {
    url: String,
    title: String,
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
}

#[derive(Deserialize)]
pub struct GearBlocksMarkerInput {
    pub label: Option<String>,
    pub reason: Option<String>,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub color: Option<String>,
    #[serde(rename = "durationSeconds")]
    pub duration_seconds: Option<f64>,
    pub size: Option<f64>,
}

#[derive(Serialize)]
pub struct GearBlocksMarkerCommandResult {
    #[serde(rename = "commandCount")]
    pub command_count: usize,
    #[serde(rename = "commandDirectory")]
    pub command_directory: String,
    #[serde(rename = "statusDirectory")]
    pub status_directory: String,
}

#[derive(Deserialize, Serialize)]
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

#[derive(Deserialize, Serialize)]
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
pub fn list_schedulers(state: State<'_, AppState>) -> Result<Vec<SchedulerRecord>, String> {
    state
        .database
        .list_schedulers()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn shutdown_app(app: AppHandle) {
    lifecycle::request_shutdown();
    app.exit(0);
}

#[tauri::command]
pub fn start_manual_overlay_drag(window: WebviewWindow) -> Result<(), String> {
    windows::start_manual_drag(window)
}

#[tauri::command]
pub fn set_overlay_window_opacity(window: WebviewWindow, opacity: f64) -> Result<(), String> {
    windows::set_overlay_opacity(&window, opacity)
}

#[tauri::command]
pub fn focus_last_game_window(state: State<'_, AppState>) -> Result<bool, String> {
    windows::focus_last_game_window_from_state(state.inner())
}

#[tauri::command]
pub fn is_overlay_forge_foreground(app: AppHandle) -> Result<bool, String> {
    Ok(WindowManager::new(&app).foreground_label()?.is_some())
}

#[tauri::command]
pub fn get_overlay_forge_foreground_window_label(app: AppHandle) -> Result<Option<String>, String> {
    WindowManager::new(&app).foreground_label()
}

pub fn start_gearblocks_runtime_import_monitor(app: AppHandle) {
    if GEARBLOCKS_RUNTIME_IMPORT_MONITOR_ACTIVE.swap(true, Ordering::SeqCst) {
        return;
    }

    thread::spawn(move || {
        loop {
            if lifecycle::sleep_until_shutdown(Duration::from_millis(2500)) {
                break;
            }
            let state = app.state::<AppState>();
            let games = state.database.list_games().unwrap_or_default();
            for game in games.iter().filter(|game| game.slug == "gearblocks") {
                if lifecycle::is_shutdown_requested() {
                    break;
                }
                if let Err(error) =
                    import_latest_gearblocks_runtime_exports_for_monitor(&state, game.id)
                {
                    eprintln!("GearBlocks runtime import monitor failed: {error}");
                }
            }
        }
        GEARBLOCKS_RUNTIME_IMPORT_MONITOR_ACTIVE.store(false, Ordering::SeqCst);
    });
}

pub fn start_scheduler_worker(app: AppHandle) {
    if SCHEDULER_WORKER_ACTIVE.swap(true, Ordering::SeqCst) {
        return;
    }

    thread::spawn(move || {
        loop {
            if lifecycle::sleep_until_shutdown(Duration::from_secs(15)) {
                break;
            }
            let state = app.state::<AppState>();
            let schedulers = state.database.list_due_schedulers(5).unwrap_or_default();
            for scheduler in schedulers {
                if lifecycle::is_shutdown_requested() {
                    break;
                }
                if let Err(error) = run_scheduler(&app, state.inner(), scheduler) {
                    eprintln!("Overlay Forge scheduler failed: {error}");
                }
            }
        }
        SCHEDULER_WORKER_ACTIVE.store(false, Ordering::SeqCst);
    });
}

fn run_scheduler(
    app: &AppHandle,
    state: &AppState,
    scheduler: SchedulerRecord,
) -> Result<(), String> {
    if !state
        .database
        .try_acquire_scheduler(scheduler.id, 120)
        .map_err(|error| error.to_string())?
    {
        return Ok(());
    }

    let run_id = state
        .database
        .start_scheduler_run(&scheduler)
        .map_err(|error| error.to_string())?;
    let result = dispatch_scheduler(app, state, &scheduler);
    let (status, message) = match result {
        Ok(message) => ("success", message),
        Err(error) => ("failed", error),
    };
    state
        .database
        .complete_scheduler_run(&scheduler, run_id, status, &message)
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn dispatch_scheduler(
    app: &AppHandle,
    state: &AppState,
    scheduler: &SchedulerRecord,
) -> Result<String, String> {
    match scheduler.type_key.as_str() {
        "smoking_cessation_export" => {
            let export_path = update_smoking_cessation_chatgpt_export(app, state)?;
            Ok(format!(
                "Updated Smoking Cessation ChatGPT export: {}",
                export_path.to_string_lossy()
            ))
        }
        other => Err(format!(
            "No registered scheduler handler for type '{other}'."
        )),
    }
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
    let window_manager = WindowManager::new(app);
    let window = window_manager.required_window(WindowKind::GameChat)?;
    window_manager.prepare_for_interaction(&window)?;
    let state = app.state::<AppState>();
    if let Ok(conversation) = state
        .database
        .get_game_chat_conversation(selection.conversation_id)
    {
        if let (Some(overlay_x), Some(overlay_y)) = (conversation.overlay_x, conversation.overlay_y)
        {
            let _ = window_manager.set_position(WindowKind::GameChat, overlay_x, overlay_y);
        }
    }
    window.show().map_err(|error| error.to_string())?;
    let _ = windows::set_overlay_opacity(&window, windows::ACTIVE_WINDOW_OPACITY);
    window.set_focus().map_err(|error| error.to_string())?;
    let _ = app.emit("game-chat-overlay-selection-changed", selection.clone());
    let _ = app.emit("game-chat-overlay-focus-prompt", ());

    Ok(())
}

#[tauri::command]
pub fn focus_game_chat_overlay_window(app: AppHandle) -> Result<bool, String> {
    if WindowManager::new(&app)
        .window(WindowKind::GameChat)
        .is_none()
    {
        return Ok(false);
    }

    let app_for_window = app.clone();
    app.run_on_main_thread(move || {
        let window_manager = WindowManager::new(&app_for_window);
        if window_manager.show_and_focus(WindowKind::GameChat).is_ok() {
            let _ = app_for_window.emit("game-chat-overlay-focus-prompt", ());
        }
    })
    .map_err(|error| error.to_string())?;
    Ok(true)
}

#[tauri::command]
pub fn toggle_game_chat_overlay_window(
    app: AppHandle,
    _state: State<'_, AppState>,
) -> Result<bool, String> {
    toggle_active_game_chat_overlay_window(&app)
}

pub fn toggle_active_game_chat_overlay_window(app: &AppHandle) -> Result<bool, String> {
    let state = app.state::<AppState>();
    let selection = state
        .active_game_chat_overlay
        .lock()
        .map_err(|_| "Game chat overlay state is unavailable.".to_string())?
        .clone();
    let Some(selection) = selection else {
        return Ok(false);
    };

    let window_manager = WindowManager::new(app);
    if window_manager.is_visible(WindowKind::GameChat)? {
        window_manager.hide(WindowKind::GameChat)?;
        return Ok(false);
    }

    show_game_chat_overlay_window(app, &selection)?;
    Ok(true)
}

pub fn show_active_game_chat_overlay_window(app: &AppHandle) -> Result<bool, String> {
    let state = app.state::<AppState>();
    let selection = state
        .active_game_chat_overlay
        .lock()
        .map_err(|_| "Game chat overlay state is unavailable.".to_string())?
        .clone();
    let Some(selection) = selection else {
        return Ok(false);
    };

    show_game_chat_overlay_window(app, &selection)?;
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
pub fn list_game_build_guides(
    game_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<GameBuildGuideRecord>, String> {
    state
        .database
        .list_game_build_guides(game_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn import_game_build_guide_markdown(
    game_id: i64,
    markdown_path: String,
    state: State<'_, AppState>,
) -> Result<GameBuildGuidePayload, String> {
    let path = PathBuf::from(markdown_path.trim());
    if !path.is_file() {
        return Err("Build guide Markdown file was not found.".to_string());
    }
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            !extension.eq_ignore_ascii_case("md") && !extension.eq_ignore_ascii_case("markdown")
        })
        .unwrap_or(true)
    {
        return Err("Build guide import expects a Markdown (.md) file.".to_string());
    }

    let raw_markdown =
        fs::read_to_string(&path).map_err(|error| format!("Could not read guide: {error}"))?;
    import_game_build_guide_markdown_content(
        state.inner(),
        game_id,
        &path.to_string_lossy(),
        &raw_markdown,
    )
}

#[tauri::command]
pub async fn import_game_build_guide_url(
    game_id: i64,
    guide_url: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<GameBuildGuidePayload, String> {
    require_text(&guide_url, "Build guide URL")?;
    let game = state
        .database
        .get_game(game_id)
        .map_err(|error| error.to_string())?;
    if game.slug != "gearblocks" {
        return Err(
            "URL build-guide import is currently available for GearBlocks only.".to_string(),
        );
    }

    let source = fetch_build_guide_source_document(&guide_url).await?;
    import_latest_gearblocks_runtime_exports(state.inner(), game.id)?;
    let custom_context = gearblocks_build_guide_import_prompt_context(&state, game.id)?;
    let api_key = configured_openai_api_key(&state)?;
    let generated_markdown = openai::create_game_build_guide_from_source_response(
        &api_key,
        &game,
        &custom_context,
        &source.url,
        &source.title,
        &source.text,
    )
    .await?;
    let clean_markdown = clean_generated_build_guide_markdown(&generated_markdown);
    let mut payload = save_and_import_generated_game_build_guide(
        &app,
        state.inner(),
        &game,
        &source.title,
        &clean_markdown,
    )?;
    payload.image_reference_count =
        import_build_guide_source_images(state.inner(), &game, &payload.guide, &source).await?;
    let _ = app.emit("game-build-guides-changed", payload.guide.clone());
    Ok(payload)
}

#[tauri::command]
pub async fn create_game_build_guide_from_chat(
    conversation_id: i64,
    build_goal: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<GameBuildGuidePayload, String> {
    require_text(&build_goal, "Build guide goal")?;
    let conversation = state
        .database
        .get_game_chat_conversation(conversation_id)
        .map_err(|error| error.to_string())?;
    let game = state
        .database
        .get_game(conversation.game_id)
        .map_err(|error| error.to_string())?;
    if game.slug != "gearblocks" {
        return Err(
            "Chat-generated build guides are currently available for GearBlocks only.".to_string(),
        );
    }

    import_latest_gearblocks_runtime_exports(state.inner(), game.id)?;
    let recent_messages = state
        .database
        .recent_game_chat_messages(conversation_id, 12)
        .map_err(|error| error.to_string())?;
    let custom_context = game_custom_prompt_context(&state, &game, true)?;
    let api_key = configured_openai_api_key(&state)?;
    let generated_markdown = openai::create_game_build_guide_response(
        &api_key,
        &game,
        &recent_messages,
        &custom_context,
        &build_goal,
    )
    .await?;
    let clean_markdown = clean_generated_build_guide_markdown(&generated_markdown);
    let payload = save_and_import_generated_game_build_guide(
        &app,
        state.inner(),
        &game,
        "GearBlocks build guide",
        &clean_markdown,
    )?;
    let _ = app.emit("game-build-guides-changed", payload.guide.clone());
    Ok(payload)
}

fn save_and_import_generated_game_build_guide(
    app: &AppHandle,
    state: &AppState,
    game: &GameRecord,
    fallback_title: &str,
    clean_markdown: &str,
) -> Result<GameBuildGuidePayload, String> {
    let parsed = parse_game_build_guide_markdown(clean_markdown);
    let title = if parsed.title.trim().is_empty() {
        fallback_title.trim()
    } else {
        parsed.title.trim()
    };
    let guide_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("build-guides")
        .join(&game.slug);
    fs::create_dir_all(&guide_dir)
        .map_err(|error| format!("Could not create build guide directory: {error}"))?;
    let guide_path = guide_dir.join(format!(
        "{}_{}.md",
        unix_timestamp_label(),
        safe_filename_part(title)
    ));
    fs::write(&guide_path, clean_markdown)
        .map_err(|error| format!("Could not save generated build guide: {error}"))?;

    import_game_build_guide_markdown_content(
        state,
        game.id,
        &guide_path.to_string_lossy(),
        clean_markdown,
    )
}

fn import_game_build_guide_markdown_content(
    state: &AppState,
    game_id: i64,
    source_path: &str,
    raw_markdown: &str,
) -> Result<GameBuildGuidePayload, String> {
    let game = state
        .database
        .get_game(game_id)
        .map_err(|error| error.to_string())?;
    let parsed = parse_game_build_guide_markdown(raw_markdown);
    let checklist_json =
        serde_json::to_string(&parsed.checklist).map_err(|error| error.to_string())?;
    let guide = state
        .database
        .create_game_build_guide(GameBuildGuideDraft {
            game_id: game.id,
            title: &parsed.title,
            source_path,
            raw_markdown,
            build_goal: &parsed.build_goal,
            scale_reference: &parsed.scale_reference,
            geometry_notes: &parsed.geometry_notes,
            glossary_text: &parsed.glossary_text,
            checklist_json: &checklist_json,
        })
        .map_err(|error| error.to_string())?;
    let selection = GameBuildGuideOverlaySelection {
        game_id: game.id,
        guide_id: guide.id,
    };
    if let Ok(mut active_selection) = state.active_game_build_guide_overlay.lock() {
        active_selection.replace(selection.clone());
    }
    persist_game_build_guide_overlay_selection(state, &selection);
    state
        .database
        .replace_game_build_guide_parts(guide.id, &parsed.parts)
        .map_err(|error| error.to_string())?;
    state
        .database
        .replace_game_build_guide_steps(guide.id, &parsed.steps)
        .map_err(|error| error.to_string())?;

    get_game_build_guide_payload_from_state(state, guide.id)
}

#[tauri::command]
pub fn get_game_build_guide(
    guide_id: i64,
    state: State<'_, AppState>,
) -> Result<GameBuildGuidePayload, String> {
    get_game_build_guide_payload_from_state(state.inner(), guide_id)
}

#[tauri::command]
pub fn delete_game_build_guide(
    guide_id: i64,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let guide = state
        .database
        .get_game_build_guide(guide_id)
        .map_err(|error| error.to_string())?;
    let stored_selection = stored_build_guide_overlay_selection(state.inner());
    state
        .database
        .delete_game_build_guide(guide_id)
        .map_err(|error| error.to_string())?;

    let deleted_active_guide = {
        let mut active_selection = state
            .active_game_build_guide_overlay
            .lock()
            .map_err(|_| "Build guide overlay state is unavailable.".to_string())?;
        if active_selection
            .as_ref()
            .map(|selection| selection.guide_id == guide.id)
            .unwrap_or(false)
        {
            active_selection.take();
            true
        } else {
            false
        }
    };

    let deleted_stored_guide = stored_selection
        .map(|selection| selection.guide_id == guide.id)
        .unwrap_or(false);
    if deleted_active_guide || deleted_stored_guide {
        state
            .database
            .delete_app_setting(ACTIVE_GAME_BUILD_GUIDE_OVERLAY_SETTING)
            .map_err(|error| error.to_string())?;
        let app_for_window = app.clone();
        app.run_on_main_thread(move || {
            let _ = WindowManager::new(&app_for_window).hide(WindowKind::GameBuildGuide);
        })
        .map_err(|error| error.to_string())?;
    }

    Ok(())
}

fn get_game_build_guide_payload_from_state(
    state: &AppState,
    guide_id: i64,
) -> Result<GameBuildGuidePayload, String> {
    let guide = state
        .database
        .get_game_build_guide(guide_id)
        .map_err(|error| error.to_string())?;
    let parts = state
        .database
        .list_game_build_guide_parts(guide_id)
        .map_err(|error| error.to_string())?;
    let steps = state
        .database
        .list_game_build_guide_steps(guide_id)
        .map_err(|error| error.to_string())?;
    let checklist = serde_json::from_str::<Vec<String>>(&guide.checklist_json).unwrap_or_default();

    Ok(GameBuildGuidePayload {
        guide,
        parts,
        steps,
        checklist,
        image_reference_count: 0,
    })
}

#[tauri::command]
pub fn open_game_build_guide_overlay_window(
    game_id: i64,
    guide_id: i64,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<GameBuildGuideOverlaySelection, String> {
    let game = state
        .database
        .get_game(game_id)
        .map_err(|error| error.to_string())?;
    let guide = state
        .database
        .get_game_build_guide(guide_id)
        .map_err(|error| error.to_string())?;
    if guide.game_id != game.id {
        return Err("Selected build guide does not belong to the selected game.".to_string());
    }

    let selection = GameBuildGuideOverlaySelection {
        game_id: game.id,
        guide_id: guide.id,
    };
    state
        .active_game_build_guide_overlay
        .lock()
        .map_err(|_| "Build guide overlay state is unavailable.".to_string())?
        .replace(selection.clone());
    persist_game_build_guide_overlay_selection(state.inner(), &selection);

    let app_for_window = app.clone();
    let selection_for_window = selection.clone();
    app.run_on_main_thread(move || {
        if let Err(error) =
            show_game_build_guide_overlay_window(&app_for_window, &selection_for_window)
        {
            eprintln!("Could not open build guide overlay window: {error}");
        }
    })
    .map_err(|error| error.to_string())?;

    Ok(selection)
}

fn show_game_build_guide_overlay_window(
    app: &AppHandle,
    selection: &GameBuildGuideOverlaySelection,
) -> Result<(), String> {
    let window_manager = WindowManager::new(app);
    let window = window_manager.required_window(WindowKind::GameBuildGuide)?;
    window_manager.prepare_for_interaction(&window)?;
    let state = app.state::<AppState>();
    if let Ok(guide) = state.database.get_game_build_guide(selection.guide_id) {
        if let (Some(overlay_x), Some(overlay_y)) = (guide.overlay_x, guide.overlay_y) {
            let _ = window_manager.set_position(WindowKind::GameBuildGuide, overlay_x, overlay_y);
        }
        if let (Some(overlay_width), Some(overlay_height)) =
            (guide.overlay_width, guide.overlay_height)
        {
            let config = StandaloneWindowConfig::game_build_guide();
            let _ = window_manager.set_size(
                WindowKind::GameBuildGuide,
                overlay_width.max(config.min_width as i32) as u32,
                overlay_height.max(config.min_height as i32) as u32,
            );
        }
    }
    window.show().map_err(|error| error.to_string())?;
    let _ = windows::set_overlay_opacity(&window, windows::ACTIVE_WINDOW_OPACITY);
    window.set_focus().map_err(|error| error.to_string())?;
    let _ = app.emit(
        "game-build-guide-overlay-selection-changed",
        selection.clone(),
    );
    let app_for_retry = app.clone();
    let selection_for_retry = selection.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(150));
        let _ = app_for_retry.emit(
            "game-build-guide-overlay-selection-changed",
            selection_for_retry.clone(),
        );
        thread::sleep(Duration::from_millis(350));
        let _ = app_for_retry.emit(
            "game-build-guide-overlay-selection-changed",
            selection_for_retry,
        );
    });

    Ok(())
}

#[tauri::command]
pub fn toggle_game_build_guide_overlay_window(
    app: AppHandle,
    _state: State<'_, AppState>,
) -> Result<bool, String> {
    toggle_active_game_build_guide_overlay_window(&app)
}

pub fn toggle_active_game_build_guide_overlay_window(app: &AppHandle) -> Result<bool, String> {
    let state = app.state::<AppState>();
    let selection = state
        .active_game_build_guide_overlay
        .lock()
        .map_err(|_| "Build guide overlay state is unavailable.".to_string())?
        .clone();
    let Some(selection) = selection.or_else(|| latest_build_guide_selection(state.inner())) else {
        return Ok(false);
    };
    state
        .active_game_build_guide_overlay
        .lock()
        .map_err(|_| "Build guide overlay state is unavailable.".to_string())?
        .replace(selection.clone());
    persist_game_build_guide_overlay_selection(state.inner(), &selection);

    let window_manager = WindowManager::new(app);
    if window_manager.is_visible(WindowKind::GameBuildGuide)? {
        window_manager.hide(WindowKind::GameBuildGuide)?;
        return Ok(false);
    }

    show_game_build_guide_overlay_window(app, &selection)?;
    Ok(true)
}

fn latest_build_guide_selection(state: &AppState) -> Option<GameBuildGuideOverlaySelection> {
    if let Some(selection) = stored_build_guide_overlay_selection(state) {
        return Some(selection);
    }

    state
        .database
        .list_games()
        .ok()?
        .into_iter()
        .find_map(|game| {
            state
                .database
                .latest_game_build_guide(game.id)
                .ok()
                .flatten()
                .map(|guide| GameBuildGuideOverlaySelection {
                    game_id: game.id,
                    guide_id: guide.id,
                })
        })
}

fn persist_game_build_guide_overlay_selection(
    state: &AppState,
    selection: &GameBuildGuideOverlaySelection,
) {
    let payload = match serde_json::to_string(selection) {
        Ok(payload) => payload,
        Err(error) => {
            eprintln!("Could not serialize build guide overlay selection: {error}");
            return;
        }
    };
    if let Err(error) = state
        .database
        .save_app_setting(ACTIVE_GAME_BUILD_GUIDE_OVERLAY_SETTING, &payload)
    {
        eprintln!("Could not persist build guide overlay selection: {error}");
    }
}

fn stored_build_guide_overlay_selection(
    state: &AppState,
) -> Option<GameBuildGuideOverlaySelection> {
    let payload = state
        .database
        .get_app_setting(ACTIVE_GAME_BUILD_GUIDE_OVERLAY_SETTING)
        .ok()
        .flatten()?;
    let selection = serde_json::from_str::<GameBuildGuideOverlaySelection>(&payload).ok()?;
    let guide = state
        .database
        .get_game_build_guide(selection.guide_id)
        .ok()?;
    if guide.game_id != selection.game_id {
        return None;
    }
    Some(selection)
}

async fn fetch_build_guide_source_document(
    guide_url: &str,
) -> Result<BuildGuideSourceDocument, String> {
    let url = validate_build_guide_source_url(guide_url)?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("OverlayForge/0.9 GearBlocksBuildGuideImporter")
        .build()
        .map_err(|error| format!("Could not create URL import client: {error}"))?;
    let response = client
        .get(url.clone())
        .send()
        .await
        .map_err(|error| format!("Could not fetch build guide URL: {error}"))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("Build guide URL returned status {status}."));
    }
    if response
        .content_length()
        .map(|length| length > BUILD_GUIDE_SOURCE_HTML_MAX_BYTES)
        .unwrap_or(false)
    {
        return Err("Build guide page is too large to import.".to_string());
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|error| format!("Could not read build guide URL response: {error}"))?;
    if bytes.len() as u64 > BUILD_GUIDE_SOURCE_HTML_MAX_BYTES {
        return Err("Build guide page is too large to import.".to_string());
    }

    let html = String::from_utf8_lossy(&bytes);
    let (title, text, images) = extract_steam_guide_source_text(&html, &url)?;
    Ok(BuildGuideSourceDocument {
        url: url.to_string(),
        title,
        text: truncate_to_char_limit(&text, BUILD_GUIDE_SOURCE_TEXT_MAX_CHARS),
        images,
    })
}

fn validate_build_guide_source_url(value: &str) -> Result<reqwest::Url, String> {
    let url = reqwest::Url::parse(value.trim())
        .map_err(|_| "Build guide URL must be a valid absolute URL.".to_string())?;
    if url.scheme() != "https" {
        return Err("Build guide URL must use https.".to_string());
    }
    let host = url
        .host_str()
        .map(|host| host.to_ascii_lowercase())
        .unwrap_or_default();
    if host != "steamcommunity.com" && host != "www.steamcommunity.com" {
        return Err(
            "Build guide URL import currently supports Steam Community sharedfiles URLs."
                .to_string(),
        );
    }
    if url.path().trim_end_matches('/') != "/sharedfiles/filedetails" {
        return Err(
            "Steam build guide URL must use /sharedfiles/filedetails with an id query parameter."
                .to_string(),
        );
    }
    let has_numeric_id = url.query_pairs().any(|(key, value)| {
        key == "id" && !value.is_empty() && value.chars().all(|c| c.is_ascii_digit())
    });
    if !has_numeric_id {
        return Err("Steam build guide URL must include a numeric id query parameter.".to_string());
    }
    Ok(url)
}

fn extract_steam_guide_source_text(
    html: &str,
    page_url: &reqwest::Url,
) -> Result<(String, String, Vec<BuildGuideSourceImage>), String> {
    let title = extract_steam_guide_title(html).unwrap_or_else(|| "Steam build guide".to_string());
    let summary = extract_div_text_near_marker(html, "guideTopDescription").unwrap_or_default();
    let body_markup = extract_between(
        html,
        "<div class=\"guide subSections\">",
        "commentthread_area",
    )
    .ok_or_else(|| "Could not find Steam guide body in the fetched page.".to_string())?;
    let images = extract_steam_guide_images(body_markup, page_url);
    let body_markup = body_markup
        .replace("<div class=\"subSectionTitle\">", "\n\n## ")
        .replace("<div class=\"subSectionDesc\">", "\n");
    let body_text = html_fragment_to_text(&body_markup);
    if body_text.chars().count() < 120 {
        return Err("Fetched Steam guide did not contain enough readable text.".to_string());
    }

    let mut sections = vec![format!("# {title}")];
    if !summary.trim().is_empty() {
        sections.push(format!("## Overview\n{}", summary.trim()));
    }
    sections.push(body_text);
    Ok((title, sections.join("\n\n"), images))
}

fn extract_steam_guide_images(html: &str, page_url: &reqwest::Url) -> Vec<BuildGuideSourceImage> {
    let mut images = Vec::new();
    let mut remaining = html;
    while let Some(start) = remaining.find("<img") {
        remaining = &remaining[start + 4..];
        let Some(end) = remaining.find('>') else {
            break;
        };
        let tag = &remaining[..end];
        remaining = &remaining[end + 1..];

        let Some(source) = html_attribute(tag, "src") else {
            continue;
        };
        let Ok(url) = page_url.join(&decode_html_entities(&source)) else {
            continue;
        };
        if !is_allowed_guide_image_url(&url) {
            continue;
        }
        let url_text = url.to_string();
        if images
            .iter()
            .any(|image: &BuildGuideSourceImage| image.url == url_text)
        {
            continue;
        }
        let title = html_attribute(tag, "title")
            .or_else(|| html_attribute(tag, "alt"))
            .map(|value| decode_html_entities(&value))
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| format!("Guide image {}", images.len() + 1));
        images.push(BuildGuideSourceImage {
            url: url_text,
            title,
        });
        if images.len() >= BUILD_GUIDE_SOURCE_IMAGE_MAX_COUNT {
            break;
        }
    }
    images
}

fn html_attribute(tag: &str, name: &str) -> Option<String> {
    let prefix = format!("{name}=");
    let start = tag.find(&prefix)? + prefix.len();
    let quote = tag[start..].chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let value_start = start + quote.len_utf8();
    let value_end = tag[value_start..].find(quote)? + value_start;
    Some(tag[value_start..value_end].to_string())
}

fn is_allowed_guide_image_url(url: &reqwest::Url) -> bool {
    if url.scheme() != "https" {
        return false;
    }
    let host = url.host_str().unwrap_or_default().to_ascii_lowercase();
    host == "images.steamusercontent.com"
        || host.ends_with(".steamusercontent.com")
        || host.ends_with(".steamstatic.com")
        || host.ends_with(".akamaihd.net")
}

async fn import_build_guide_source_images(
    state: &AppState,
    game: &GameRecord,
    guide: &GameBuildGuideRecord,
    source: &BuildGuideSourceDocument,
) -> Result<usize, String> {
    if source.images.is_empty() {
        return Ok(0);
    }

    let image_dir = overlay_workspace_root()?
        .join("game-screenshots")
        .join(&game.slug)
        .join("build-guide-images")
        .join(guide.id.to_string());
    fs::create_dir_all(&image_dir)
        .map_err(|error| format!("Could not create build guide image directory: {error}"))?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("OverlayForge/0.9 GearBlocksBuildGuideImageImporter")
        .build()
        .map_err(|error| format!("Could not create image import client: {error}"))?;
    let mut imported_count = 0;

    for (index, image) in source
        .images
        .iter()
        .take(BUILD_GUIDE_SOURCE_IMAGE_MAX_COUNT)
        .enumerate()
    {
        match download_build_guide_source_image(&client, image, &image_dir, index + 1).await {
            Ok(local_path) => {
                let local_path_text = path_text(&local_path);
                let title = format!("{} image {}", guide.title, index + 1);
                let notes = format!(
                    "Imported from build guide {} for guide id {}. Source image title: {}",
                    source.url, guide.id, image.title
                );
                let tags = format!("gearblocks,build-guide,guide:{},guide-image", guide.id);
                state
                    .database
                    .create_game_catalog_reference(GameCatalogReferenceDraft {
                        game_id: game.id,
                        object_id: None,
                        title: &title,
                        reference_type: "build_guide_image",
                        url: &image.url,
                        local_path: &local_path_text,
                        notes: &notes,
                        tags: &tags,
                    })
                    .map_err(|error| error.to_string())?;
                imported_count += 1;
            }
            Err(error) => {
                eprintln!("Could not import build guide image {}: {error}", image.url);
            }
        }
    }

    Ok(imported_count)
}

async fn download_build_guide_source_image(
    client: &reqwest::Client,
    image: &BuildGuideSourceImage,
    image_dir: &Path,
    index: usize,
) -> Result<PathBuf, String> {
    let url =
        reqwest::Url::parse(&image.url).map_err(|_| "Guide image URL was invalid.".to_string())?;
    if !is_allowed_guide_image_url(&url) {
        return Err("Guide image URL is not from an allowed Steam image host.".to_string());
    }

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|error| format!("Could not fetch guide image: {error}"))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("Guide image returned status {status}."));
    }
    if response
        .content_length()
        .map(|length| length > BUILD_GUIDE_SOURCE_IMAGE_MAX_BYTES)
        .unwrap_or(false)
    {
        return Err("Guide image is too large to import.".to_string());
    }

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let bytes = response
        .bytes()
        .await
        .map_err(|error| format!("Could not read guide image response: {error}"))?;
    if bytes.len() as u64 > BUILD_GUIDE_SOURCE_IMAGE_MAX_BYTES {
        return Err("Guide image is too large to import.".to_string());
    }
    let extension = guide_image_extension(&content_type, bytes.as_ref())
        .ok_or_else(|| "Guide image is not a supported image type.".to_string())?;
    let filename_title = safe_filename_part(&image.title);
    let filename_stem = if filename_title.is_empty() {
        format!("guide_image_{index:02}")
    } else {
        format!("{index:02}_{filename_title}")
    };
    let local_path = image_dir.join(format!("{filename_stem}.{extension}"));
    fs::write(&local_path, &bytes)
        .map_err(|error| format!("Could not save guide image: {error}"))?;
    Ok(local_path)
}

fn guide_image_extension(content_type: &str, bytes: &[u8]) -> Option<&'static str> {
    match content_type.split(';').next().unwrap_or_default().trim() {
        "image/png" => Some("png"),
        "image/jpeg" | "image/jpg" => Some("jpg"),
        "image/webp" => Some("webp"),
        "image/gif" => Some("gif"),
        _ if bytes.starts_with(b"\x89PNG\r\n\x1a\n") => Some("png"),
        _ if bytes.starts_with(&[0xff, 0xd8, 0xff]) => Some("jpg"),
        _ if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") => Some("gif"),
        _ if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" => {
            Some("webp")
        }
        _ => None,
    }
}

fn extract_steam_guide_title(html: &str) -> Option<String> {
    extract_div_text_near_marker(html, "workshopItemTitle")
        .or_else(|| {
            extract_between(html, "<title>", "</title>").map(|title| {
                html_fragment_to_text(title)
                    .replace("Steam Community :: Guide ::", "")
                    .trim()
                    .to_string()
            })
        })
        .filter(|title| !title.trim().is_empty())
}

fn extract_div_text_near_marker(html: &str, marker: &str) -> Option<String> {
    let marker_index = html.find(marker)?;
    let div_start = html[..marker_index].rfind("<div")?;
    let open_end = html[marker_index..].find('>')? + marker_index;
    if open_end < div_start {
        return None;
    }
    let close = html[open_end + 1..].find("</div>")? + open_end + 1;
    Some(html_fragment_to_text(&html[open_end + 1..close]))
}

fn extract_between<'a>(value: &'a str, start_marker: &str, end_marker: &str) -> Option<&'a str> {
    let start = value.find(start_marker)? + start_marker.len();
    let end = value[start..].find(end_marker)? + start;
    Some(&value[start..end])
}

fn html_fragment_to_text(value: &str) -> String {
    let mut output = String::new();
    let mut chars = value.chars().peekable();
    while let Some(character) = chars.next() {
        if character != '<' {
            output.push(character);
            continue;
        }

        let mut tag = String::new();
        for tag_character in chars.by_ref() {
            if tag_character == '>' {
                break;
            }
            tag.push(tag_character);
        }
        let tag_name = tag
            .trim()
            .trim_start_matches('/')
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();
        if matches!(
            tag_name.as_str(),
            "br" | "div" | "p" | "li" | "ul" | "ol" | "hr" | "blockquote" | "h1" | "h2" | "h3"
        ) {
            output.push('\n');
        } else {
            output.push(' ');
        }
    }

    decode_html_entities(&output)
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn decode_html_entities(value: &str) -> String {
    let mut output = String::new();
    let mut chars = value.chars().peekable();
    while let Some(character) = chars.next() {
        if character != '&' {
            output.push(character);
            continue;
        }

        let mut entity = String::new();
        while let Some(next) = chars.peek().copied() {
            if next == ';' || entity.len() > 12 {
                break;
            }
            entity.push(next);
            chars.next();
        }
        if chars.peek() == Some(&';') {
            chars.next();
        }

        match decode_html_entity(&entity) {
            Some(decoded) => output.push(decoded),
            None => {
                output.push('&');
                output.push_str(&entity);
                output.push(';');
            }
        }
    }
    output
}

fn decode_html_entity(entity: &str) -> Option<char> {
    match entity {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" | "#39" => Some('\''),
        "nbsp" => Some(' '),
        _ if entity.starts_with("#x") || entity.starts_with("#X") => {
            u32::from_str_radix(&entity[2..], 16)
                .ok()
                .and_then(char::from_u32)
        }
        _ if entity.starts_with('#') => entity[1..].parse::<u32>().ok().and_then(char::from_u32),
        _ => None,
    }
}

fn truncate_to_char_limit(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_string();
    }
    let mut truncated = value.chars().take(limit).collect::<String>();
    truncated.push_str("\n\n[Source text truncated for prompt size.]");
    truncated
}

struct ParsedGameBuildGuide {
    title: String,
    build_goal: String,
    scale_reference: String,
    geometry_notes: String,
    glossary_text: String,
    checklist: Vec<String>,
    parts: Vec<GameBuildGuidePartDraft>,
    steps: Vec<GameBuildGuideStepDraft>,
}

fn parse_game_build_guide_markdown(markdown: &str) -> ParsedGameBuildGuide {
    let parts = markdown_parts_tables(markdown);
    let steps = markdown_assembly_steps(markdown, &parts);
    ParsedGameBuildGuide {
        title: first_markdown_heading(markdown).unwrap_or_else(|| "Build guide".to_string()),
        build_goal: markdown_section_text(markdown, "Build Goal"),
        scale_reference: markdown_section_text(markdown, "Scale Reference"),
        geometry_notes: markdown_section_text(markdown, "Current Chosen Geometry"),
        glossary_text: markdown_section_text(markdown, "Glossary"),
        checklist: markdown_bullets_in_section(markdown, "First Test Checklist"),
        parts,
        steps,
    }
}

fn clean_generated_build_guide_markdown(markdown: &str) -> String {
    let trimmed = markdown.trim();
    if let Some(stripped) = trimmed.strip_prefix("```") {
        let without_language = stripped
            .strip_prefix("markdown")
            .or_else(|| stripped.strip_prefix("md"))
            .or_else(|| stripped.strip_prefix("text"))
            .unwrap_or(stripped)
            .trim_start();
        if let Some(content) = without_language.strip_suffix("```") {
            return content.trim().to_string();
        }
    }
    trimmed.to_string()
}

fn first_markdown_heading(markdown: &str) -> Option<String> {
    markdown.lines().find_map(|line| {
        line.strip_prefix("# ")
            .map(|title| title.trim().to_string())
            .filter(|title| !title.is_empty())
    })
}

fn markdown_section_text(markdown: &str, section_title: &str) -> String {
    collect_markdown_section(markdown, section_title)
        .into_iter()
        .map(|line| clean_build_guide_markdown_line(&line))
        .filter(|line| should_keep_build_guide_markdown_line(line))
        .filter(|line| !line.trim().starts_with('|'))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn markdown_bullets_in_section(markdown: &str, section_title: &str) -> Vec<String> {
    collect_markdown_section(markdown, section_title)
        .into_iter()
        .filter_map(|line| {
            let cleaned = clean_build_guide_markdown_line(&line);
            let trimmed = cleaned.trim();
            trimmed
                .strip_prefix("- ")
                .or_else(|| trimmed.strip_prefix("* "))
                .map(|item| item.trim().to_string())
                .filter(|item| !item.is_empty())
        })
        .collect()
}

fn collect_markdown_section(markdown: &str, section_title: &str) -> Vec<String> {
    let target = format!("## {}", section_title.trim()).to_ascii_lowercase();
    let mut in_section = false;
    let mut lines = Vec::new();

    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") && !trimmed.starts_with("### ") {
            if in_section {
                break;
            }
            in_section = trimmed.to_ascii_lowercase() == target;
            continue;
        }

        if in_section {
            lines.push(line.to_string());
        }
    }

    lines
}

fn markdown_parts_tables(markdown: &str) -> Vec<GameBuildGuidePartDraft> {
    let lines = collect_markdown_section(markdown, "Main Parts List");
    let mut parts = Vec::new();
    let mut current_section = String::new();
    let mut row_order = 0_i64;

    for line in lines {
        let trimmed = line.trim();
        if let Some(section) = trimmed.strip_prefix("### ") {
            current_section = clean_build_guide_markdown_line(section);
            continue;
        }
        if !should_keep_build_guide_markdown_line(trimmed) {
            continue;
        }
        if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
            continue;
        }

        let columns = trimmed
            .trim_matches('|')
            .split('|')
            .map(clean_build_guide_markdown_line)
            .collect::<Vec<_>>();
        if columns.len() < 2 {
            continue;
        }
        let first = columns[0].to_ascii_lowercase();
        let second = columns[1].to_ascii_lowercase();
        if first == "qty"
            || second == "part"
            || columns
                .iter()
                .all(|column| is_markdown_table_separator(column))
        {
            continue;
        }
        let part_name = columns.get(1).cloned().unwrap_or_default();
        if part_name.is_empty() || is_markdown_table_separator(&part_name) {
            continue;
        }

        row_order += 1;
        parts.push(GameBuildGuidePartDraft {
            section: current_section.clone(),
            quantity: columns.first().cloned().unwrap_or_default(),
            part_name,
            purpose: columns.get(2).cloned().unwrap_or_default(),
            row_order,
        });
    }

    parts
}

fn markdown_assembly_steps(
    markdown: &str,
    parts: &[GameBuildGuidePartDraft],
) -> Vec<GameBuildGuideStepDraft> {
    let lines = collect_markdown_section(markdown, "Assembly Instructions");
    let mut steps = Vec::new();
    let mut current_number = 0_i64;
    let mut current_title = String::new();
    let mut current_body = Vec::<String>::new();
    let mut row_order = 0_i64;

    for line in lines {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix("### ") {
            push_build_guide_step(
                &mut steps,
                &mut row_order,
                current_number,
                &current_title,
                &current_body,
                parts,
            );
            current_body.clear();
            let (number, title) = split_numbered_heading(heading);
            current_number = number.unwrap_or(row_order + 1);
            current_title = title;
            continue;
        }

        let cleaned = clean_build_guide_markdown_line(&line);
        if should_keep_build_guide_markdown_line(&cleaned) {
            current_body.push(cleaned);
        }
    }

    push_build_guide_step(
        &mut steps,
        &mut row_order,
        current_number,
        &current_title,
        &current_body,
        parts,
    );
    steps
}

fn clean_build_guide_markdown_line(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.starts_with("```") {
        return String::new();
    }
    trimmed
        .trim_start_matches("- ")
        .trim_start_matches("* ")
        .trim()
        .to_string()
}

fn should_keep_build_guide_markdown_line(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed == "---" || trimmed == "***" || trimmed == "```" || trimmed.starts_with("```") {
        return false;
    }
    !is_markdown_table_separator(trimmed)
}

fn is_markdown_table_separator(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.contains('-')
        && trimmed
            .chars()
            .all(|character| matches!(character, '|' | ':' | '-' | ' '))
}

fn push_build_guide_step(
    steps: &mut Vec<GameBuildGuideStepDraft>,
    row_order: &mut i64,
    _step_number: i64,
    title: &str,
    body: &[String],
    parts: &[GameBuildGuidePartDraft],
) {
    if title.trim().is_empty() && body.is_empty() {
        return;
    }
    let body_text = body.join("\n").trim().to_string();
    let mentioned_parts = build_guide_step_part_instance_labels(title, &body_text, parts);
    if mentioned_parts.len() > 3 {
        let connection_type = build_guide_step_connection_type(title, &body_text);
        for chunk in mentioned_parts.chunks(3) {
            *row_order += 1;
            let part_text = chunk.join(", ");
            let step_title = if title.trim().is_empty() {
                part_text.clone()
            } else {
                format!("{} - {}", title.trim(), part_text)
            };
            steps.push(GameBuildGuideStepDraft {
                step_number: *row_order,
                title: step_title,
                body: format!(
                    "Place {part_text}.\nConnection: connect these parts using {connection_type}."
                ),
                row_order: *row_order,
            });
        }
        return;
    }
    *row_order += 1;
    steps.push(GameBuildGuideStepDraft {
        step_number: *row_order,
        title: title.trim().to_string(),
        body: body_text,
        row_order: *row_order,
    });
}

fn build_guide_step_part_instance_labels(
    title: &str,
    body: &str,
    parts: &[GameBuildGuidePartDraft],
) -> Vec<String> {
    let step_text = format!("{title}\n{body}").to_ascii_lowercase();
    let mut labels = Vec::new();
    for part in parts {
        let part_name = part.part_name.trim();
        if part_name.is_empty() {
            continue;
        }
        if !step_text.contains(&part_name.to_ascii_lowercase()) {
            continue;
        }
        for _ in 0..build_guide_part_quantity_count(&part.quantity) {
            labels.push(part_name.to_string());
        }
    }
    labels
}

fn build_guide_part_quantity_count(quantity: &str) -> usize {
    let Some(start) = quantity.find(|character: char| character.is_ascii_digit()) else {
        return 1;
    };
    let digits = quantity[start..]
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .collect::<String>();
    digits
        .parse::<usize>()
        .ok()
        .filter(|count| *count > 0)
        .unwrap_or(1)
}

fn build_guide_step_connection_type(title: &str, body: &str) -> &'static str {
    let text = format!("{title} {body}").to_ascii_lowercase();
    if has_any_text(&text, &["crank", "axle", "shaft", "gear", "wheel", "hub", "drivetrain"]) {
        return "rotary connections";
    }
    if has_any_text(&text, &["steering", "suspension", "spring", "damper", "knuckle", "pivot"]) {
        return "pivot/rotary connections";
    }
    if has_any_text(&text, &["align", "jig", "reference"]) {
        return "aligned reference connections";
    }
    "static connections"
}

fn has_any_text(text: &str, values: &[&str]) -> bool {
    values.iter().any(|value| text.contains(value))
}

fn split_numbered_heading(heading: &str) -> (Option<i64>, String) {
    let trimmed = heading.trim();
    let Some((number_text, title)) = trimmed.split_once('.') else {
        return (None, trimmed.to_string());
    };
    let number = number_text.trim().parse::<i64>().ok();
    let title = title.trim().to_string();
    (number, title)
}

#[tauri::command]
pub fn get_active_game_build_guide_overlay(
    state: State<'_, AppState>,
) -> Result<Option<GameBuildGuideOverlaySelection>, String> {
    let active = state
        .active_game_build_guide_overlay
        .lock()
        .map_err(|_| "Build guide overlay state is unavailable.".to_string())?
        .clone();
    if active.is_some() {
        return Ok(active);
    }

    let fallback = stored_build_guide_overlay_selection(state.inner());
    if let Some(selection) = &fallback {
        state
            .active_game_build_guide_overlay
            .lock()
            .map_err(|_| "Build guide overlay state is unavailable.".to_string())?
            .replace(selection.clone());
    }
    Ok(fallback)
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
    input: CalendarEventUpdateInput,
    state: State<'_, AppState>,
) -> Result<CalendarEventRecord, String> {
    validate_calendar_event(
        &input.title,
        &input.start_date,
        &input.start_time,
        &input.end_date,
        &input.end_time,
    )?;
    state
        .database
        .update_calendar_event(CalendarEventUpdateDraft {
            id: input.id,
            title: &input.title,
            start_date: &input.start_date,
            start_time: &input.start_time,
            end_date: &input.end_date,
            end_time: &input.end_time,
            notes: &input.notes,
        })
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
pub fn list_smoking_events(state: State<'_, AppState>) -> Result<Vec<SmokingEventRecord>, String> {
    state
        .database
        .list_smoking_events()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn record_smoking_event(
    smoked_at: Option<String>,
    notes: Option<String>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SmokingEventRecord, String> {
    let record = state
        .database
        .create_smoking_event(
            smoked_at.as_deref(),
            "manual",
            notes.as_deref().unwrap_or_default(),
        )
        .map_err(|error| error.to_string())?;
    if let Err(error) = update_smoking_cessation_chatgpt_export(&app, state.inner()) {
        eprintln!("Could not update smoking cessation ChatGPT export: {error}");
    }
    Ok(record)
}

#[tauri::command]
pub fn delete_smoking_event(
    id: i64,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .database
        .delete_smoking_event(id)
        .map_err(|error| error.to_string())?;
    if let Err(error) = update_smoking_cessation_chatgpt_export(&app, state.inner()) {
        eprintln!("Could not update smoking cessation ChatGPT export: {error}");
    }
    Ok(())
}

#[tauri::command]
pub fn get_smoking_cessation_settings(
    state: State<'_, AppState>,
) -> Result<SmokingCessationSettingsRecord, String> {
    state
        .database
        .get_smoking_cessation_settings()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_smoking_cigarette_count(
    current_cigarette_count: i64,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SmokingCessationSettingsRecord, String> {
    let settings = state
        .database
        .update_smoking_cigarette_count(current_cigarette_count)
        .map_err(|error| error.to_string())?;
    if let Err(error) = update_smoking_cessation_chatgpt_export(&app, state.inner()) {
        eprintln!("Could not update smoking cessation ChatGPT export: {error}");
    }
    Ok(settings)
}

#[tauri::command]
pub fn export_smoking_cessation_chatgpt_context(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SmokingCessationExportRecord, String> {
    let export_path = update_smoking_cessation_chatgpt_export(&app, state.inner())?;
    Ok(SmokingCessationExportRecord {
        export_path: export_path.to_string_lossy().to_string(),
    })
}

pub fn update_smoking_cessation_chatgpt_export(
    app: &AppHandle,
    state: &AppState,
) -> Result<PathBuf, String> {
    let export_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("chatgpt-exports");
    fs::create_dir_all(&export_dir).map_err(|error| error.to_string())?;
    let export_path = export_dir.join("smoking-cessation.md");
    let events = state
        .database
        .list_smoking_events()
        .map_err(|error| error.to_string())?;
    let settings = state
        .database
        .get_smoking_cessation_settings()
        .map_err(|error| error.to_string())?;
    let content = render_smoking_cessation_chatgpt_export(&settings, &events);
    fs::write(&export_path, content).map_err(|error| error.to_string())?;
    Ok(export_path)
}

fn render_smoking_cessation_chatgpt_export(
    settings: &SmokingCessationSettingsRecord,
    events: &[SmokingEventRecord],
) -> String {
    let exported_at_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    let today_key = events
        .first()
        .and_then(|event| event.smoked_at.get(0..10))
        .unwrap_or_default();
    let today_count = if today_key.is_empty() {
        0
    } else {
        events
            .iter()
            .filter(|event| event.smoked_at.starts_with(today_key))
            .count()
    };
    let last_recorded = events
        .first()
        .map(|event| event.smoked_at.as_str())
        .unwrap_or("None");

    let mut content = String::new();
    content.push_str("# Overlay Forge Smoking Cessation Context\n\n");
    content.push_str("This file is automatically generated by Overlay Forge for ChatGPT context. Do not edit it directly.\n\n");
    content.push_str("## Current Status\n\n");
    content.push_str(&format!("- Exported at Unix time: {exported_at_unix}\n"));
    content.push_str(&format!("- Patch: {}\n", settings.patch_label));
    content.push_str(&format!(
        "- Patch started: {} {}\n",
        settings.patch_started_at, settings.patch_timezone
    ));
    content.push_str(&format!(
        "- Current cigarettes remaining: {}\n",
        settings.current_cigarette_count
    ));
    content.push_str(&format!("- Total cigarettes recorded: {}\n", events.len()));
    content.push_str(&format!("- Latest recorded event: {last_recorded}\n"));
    content.push_str(&format!(
        "- Count on latest recorded day ({today_key}): {today_count}\n\n"
    ));

    content.push_str("## Daily Counts\n\n");
    append_smoking_count_table(&mut content, "Day", smoking_counts_by_prefix(events, 10));
    content.push_str("\n## Monthly Counts\n\n");
    append_smoking_count_table(&mut content, "Month", smoking_counts_by_prefix(events, 7));
    content.push_str("\n## Yearly Counts\n\n");
    append_smoking_count_table(&mut content, "Year", smoking_counts_by_prefix(events, 4));

    content.push_str("\n## Event Log\n\n");
    if events.is_empty() {
        content.push_str("No cigarette events are currently recorded.\n");
    } else {
        content.push_str("| Smoked At | Source | Notes |\n");
        content.push_str("| --- | --- | --- |\n");
        for event in events {
            content.push_str(&format!(
                "| {} | {} | {} |\n",
                markdown_table_cell(&event.smoked_at),
                markdown_table_cell(&event.source),
                markdown_table_cell(&event.notes)
            ));
        }
    }

    content
}

fn smoking_counts_by_prefix(
    events: &[SmokingEventRecord],
    prefix_len: usize,
) -> Vec<(String, usize)> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for event in events {
        if let Some(key) = event.smoked_at.get(0..prefix_len) {
            *counts.entry(key.to_string()).or_default() += 1;
        }
    }
    let mut ordered = counts.into_iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| right.0.cmp(&left.0));
    ordered
}

fn append_smoking_count_table(content: &mut String, label: &str, counts: Vec<(String, usize)>) {
    if counts.is_empty() {
        content.push_str("No records.\n");
        return;
    }

    content.push_str(&format!("| {label} | Count |\n"));
    content.push_str("| --- | ---: |\n");
    for (key, count) in counts {
        content.push_str(&format!("| {} | {} |\n", markdown_table_cell(&key), count));
    }
}

fn markdown_table_cell(value: &str) -> String {
    value.replace('|', "\\|").replace(['\r', '\n'], " ")
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
        .update_youtube_reference(YouTubeReferenceUpdateDraft {
            id,
            title: &title,
            url: &url,
            video_id: &video_id,
            channel_name: &channel_name,
            notes: &notes,
            tags: &tags,
        })
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
pub fn get_game_setting(
    game_id: i64,
    setting_key: String,
    state: State<'_, AppState>,
) -> Result<Option<GameSettingRecord>, String> {
    require_text(&setting_key, "Game setting key")?;
    state
        .database
        .get_game_setting(game_id, &setting_key)
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

    let requested_export = request_gearblocks_scene_export_from_game(state.inner())?;
    let imported_count =
        import_requested_gearblocks_runtime_export(&state, game.id, &requested_export)?
            + import_latest_gearblocks_runtime_exports(&state, game.id)?
            + reconcile_latest_gearblocks_runtime_exports(&state, game.id)?;
    sync_gearblocks_saved_constructions_for_game(&state, game.id)?;

    let fingerprint = gearblocks_runtime_context_fingerprint(&state, game.id)?;
    let fingerprint_key = format!("gearblocks.runtime_context_fingerprint.{}", game.id);
    let previous_fingerprint = state
        .database
        .get_app_setting(&fingerprint_key)
        .map_err(|error| error.to_string())?;
    let changed = previous_fingerprint.as_deref() != Some(fingerprint.as_str());
    state
        .database
        .save_app_setting(&fingerprint_key, &fingerprint)
        .map_err(|error| error.to_string())?;

    let construction_count = state
        .database
        .count_game_constructions(game.id)
        .map_err(|error| error.to_string())?;
    let runtime_export_count = state
        .database
        .count_game_runtime_construction_exports(game.id)
        .map_err(|error| error.to_string())?;
    let runtime_part_count = state
        .database
        .count_game_runtime_parts(game.id)
        .map_err(|error| error.to_string())?;

    Ok(GearBlocksRuntimeContextSyncRecord {
        changed: changed || imported_count > 0,
        runtime_export_count,
        runtime_part_count,
        construction_count,
    })
}

#[tauri::command]
pub fn import_gearblocks_runtime_context(
    game_id: i64,
    state: State<'_, AppState>,
) -> Result<GearBlocksRuntimeContextSyncRecord, String> {
    let game = state
        .database
        .get_game(game_id)
        .map_err(|error| error.to_string())?;
    require_gearblocks_game(&game)?;

    let imported_count = import_latest_gearblocks_runtime_exports(&state, game.id)?
        + reconcile_latest_gearblocks_runtime_exports(&state, game.id)?;
    let construction_count = state
        .database
        .count_game_constructions(game.id)
        .map_err(|error| error.to_string())?;
    let runtime_export_count = state
        .database
        .count_game_runtime_construction_exports(game.id)
        .map_err(|error| error.to_string())?;
    let runtime_part_count = state
        .database
        .count_game_runtime_parts(game.id)
        .map_err(|error| error.to_string())?;

    Ok(GearBlocksRuntimeContextSyncRecord {
        changed: imported_count > 0,
        runtime_export_count,
        runtime_part_count,
        construction_count,
    })
}

#[tauri::command]
pub fn send_gearblocks_marker_commands(
    game_id: i64,
    markers: Vec<GearBlocksMarkerInput>,
    state: State<'_, AppState>,
) -> Result<GearBlocksMarkerCommandResult, String> {
    let game = state
        .database
        .get_game(game_id)
        .map_err(|error| error.to_string())?;
    require_gearblocks_game(&game)?;
    if markers.is_empty() {
        return Err("At least one marker is required.".to_string());
    }

    let command_directory = gearblocks_plugin_command_directory()?;
    let status_directory = gearblocks_plugin_status_directory()?;
    fs::create_dir_all(&command_directory).map_err(|error| {
        format!("Could not create GearBlocks marker command directory: {error}")
    })?;
    fs::create_dir_all(&status_directory)
        .map_err(|error| format!("Could not create GearBlocks marker status directory: {error}"))?;

    let batch_id = unix_timestamp_label();
    for (index, marker) in markers.iter().enumerate() {
        if !marker.x.is_finite() || !marker.y.is_finite() || !marker.z.is_finite() {
            return Err("Marker coordinates must be finite numbers.".to_string());
        }

        let label = marker
            .label
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("chat-marker");
        let id = format!(
            "chat_marker_{}_{}_{}",
            batch_id,
            index + 1,
            safe_filename_part(label)
        );
        let command = json!({
            "action": "spawn_world_marker",
            "id": id,
            "label": label,
            "reason": marker.reason.as_deref().unwrap_or_default(),
            "x": marker.x,
            "y": marker.y,
            "z": marker.z,
            "color": marker.color.as_deref().unwrap_or("#55f0c8"),
            "durationSeconds": marker.duration_seconds.unwrap_or(45.0),
            "size": marker.size.unwrap_or(4.0),
        });
        fs::write(
            command_directory.join(format!("{id}.json")),
            serde_json::to_string_pretty(&command)
                .map_err(|error| format!("Could not serialize marker command: {error}"))?,
        )
        .map_err(|error| format!("Could not write marker command file: {error}"))?;
    }

    Ok(GearBlocksMarkerCommandResult {
        command_count: markers.len(),
        command_directory: command_directory.to_string_lossy().to_string(),
        status_directory: status_directory.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub fn clear_gearblocks_markers(
    game_id: i64,
    state: State<'_, AppState>,
) -> Result<GearBlocksMarkerCommandResult, String> {
    let game = state
        .database
        .get_game(game_id)
        .map_err(|error| error.to_string())?;
    require_gearblocks_game(&game)?;

    let command_directory = gearblocks_plugin_command_directory()?;
    let status_directory = gearblocks_plugin_status_directory()?;
    fs::create_dir_all(&command_directory).map_err(|error| {
        format!("Could not create GearBlocks marker command directory: {error}")
    })?;
    fs::create_dir_all(&status_directory)
        .map_err(|error| format!("Could not create GearBlocks marker status directory: {error}"))?;

    let id = format!("clear_markers_{}", unix_timestamp_label());
    let command = json!({
        "action": "clear_markers",
        "id": id,
    });
    fs::write(
        command_directory.join(format!("{id}.json")),
        serde_json::to_string_pretty(&command)
            .map_err(|error| format!("Could not serialize clear marker command: {error}"))?,
    )
    .map_err(|error| format!("Could not write clear marker command file: {error}"))?;

    Ok(GearBlocksMarkerCommandResult {
        command_count: 1,
        command_directory: command_directory.to_string_lossy().to_string(),
        status_directory: status_directory.to_string_lossy().to_string(),
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
        let record = index_decoded_gearblocks_construction(state, game_id, &decoded, &indexed_at)?;
        records.push(record);
    }

    records.sort_by(|left, right| {
        left.name
            .to_ascii_lowercase()
            .cmp(&right.name.to_ascii_lowercase())
    });
    Ok(records)
}

fn index_decoded_gearblocks_construction(
    state: &AppState,
    game_id: i64,
    decoded: &GearBlocksConstructionDecodeRecord,
    indexed_at: &str,
) -> Result<GameConstructionRecord, String> {
    let summary_json =
        serde_json::to_string_pretty(&decoded.summary).map_err(|error| error.to_string())?;
    let document_json =
        serde_json::to_string_pretty(&decoded.document).map_err(|error| error.to_string())?;
    state
        .database
        .upsert_game_construction(GameConstructionDraft {
            game_id,
            name: &decoded.name,
            folder_path: &decoded.folder_path,
            construction_path: &decoded.construction_path,
            byte_size: decoded.byte_size as i64,
            decoded_byte_size: decoded.decoded_byte_size as i64,
            composite_count: decoded.summary.composite_count as i64,
            part_count: decoded.summary.part_count as i64,
            unique_asset_guid_count: decoded.summary.unique_asset_guid_count as i64,
            attachment_count: decoded.summary.attachment_count as i64,
            link_count: decoded.summary.link_count as i64,
            intersection_count: decoded.summary.intersection_count as i64,
            is_frozen: decoded.summary.is_frozen,
            is_invulnerable: decoded.summary.is_invulnerable,
            summary_json: &summary_json,
            document_json: &document_json,
            last_indexed_at: indexed_at,
        })
        .map_err(|error| error.to_string())
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

    let old_tools_path = gearblocks_root.join("ScriptMods").join("OverlayForgeTools");
    if old_tools_path.is_dir() {
        let _ = fs::remove_dir_all(old_tools_path);
    }

    Ok(GearBlocksLuaExporterInstallRecord {
        script_mod_path: script_mod_path.to_string_lossy().to_string(),
        main_lua_path: main_lua_path.to_string_lossy().to_string(),
        export_directory: export_directory.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub fn get_gearblocks_third_party_dependency_status(
    game_id: i64,
    state: State<'_, AppState>,
) -> Result<GearBlocksThirdPartyDependencyStatusPayload, String> {
    let game = state
        .database
        .get_game(game_id)
        .map_err(|error| error.to_string())?;
    require_gearblocks_game(&game)?;

    let game_root = gearblocks_game_install_root(state.inner(), game_id);
    let bepinex_status = gearblocks_bepinex_status(game_root.as_deref());
    let gearlib_status = gearblocks_gearlib_status(game_root.as_deref());

    Ok(GearBlocksThirdPartyDependencyStatusPayload {
        game_root: game_root
            .as_ref()
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_default(),
        dependencies: vec![bepinex_status, gearlib_status],
    })
}

struct GearBlocksRequestedRuntimeExport {
    log_path: PathBuf,
    initial_length: u64,
}

fn request_gearblocks_scene_export_from_game(
    state: &AppState,
) -> Result<GearBlocksRequestedRuntimeExport, String> {
    if !windows::focus_last_game_window_from_state(state)? {
        return Err(
            "No remembered GearBlocks window is available for scene export. Focus GearBlocks from Overlay Forge once, then refresh scene context again."
                .to_string(),
        );
    }

    let log_path = gearblocks_default_user_data_root()?.join("Player.log");
    let initial_length = fs::metadata(&log_path)
        .map(|metadata| metadata.len())
        .unwrap_or(0);

    thread::sleep(Duration::from_millis(40));
    send_virtual_key(OverlayToolKey {
        virtual_key: overlay_forge_scene_export_virtual_key(),
        shift: true,
        control: true,
    })?;
    if !wait_for_gearblocks_export_log_append(&log_path, initial_length) {
        return Err(
            "GearBlocks did not write a completed Overlay Forge scene export after the chat prompt requested one. Make sure the Overlay Forge GearBlocks script is loaded, then try again."
                .to_string(),
        );
    }
    Ok(GearBlocksRequestedRuntimeExport {
        log_path,
        initial_length,
    })
}

fn overlay_forge_scene_export_virtual_key() -> u16 {
    b'E' as u16
}

fn wait_for_gearblocks_export_log_append(log_path: &Path, initial_length: u64) -> bool {
    let started_at = SystemTime::now();
    loop {
        if gearblocks_log_has_completed_export_after(log_path, initial_length) {
            return true;
        }

        if started_at
            .elapsed()
            .map(|elapsed| elapsed >= Duration::from_secs(60))
            .unwrap_or(true)
        {
            return false;
        }

        thread::sleep(Duration::from_millis(160));
    }
}

fn gearblocks_log_has_completed_export_after(log_path: &Path, initial_length: u64) -> bool {
    let Ok(metadata) = fs::metadata(log_path) else {
        return false;
    };
    if metadata.len() <= initial_length {
        return false;
    }

    let Ok(mut file) = File::open(log_path) else {
        return false;
    };
    if file.seek(SeekFrom::Start(initial_length)).is_err() {
        return false;
    }
    let mut additions = String::new();
    if file.read_to_string(&mut additions).is_err() {
        return false;
    }
    additions.contains("[OverlayForgeExportEnd]")
}

#[derive(Clone, Copy)]
struct OverlayToolKey {
    virtual_key: u16,
    shift: bool,
    control: bool,
}

#[cfg(target_os = "windows")]
fn send_virtual_key(tool_key: OverlayToolKey) -> Result<(), String> {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_CONTROL,
        VK_SHIFT,
    };

    fn keyboard_input(virtual_key: u16, flags: u32) -> INPUT {
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: virtual_key,
                    wScan: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    let mut inputs = Vec::new();
    if tool_key.control {
        inputs.push(keyboard_input(VK_CONTROL, 0));
    }
    if tool_key.shift {
        inputs.push(keyboard_input(VK_SHIFT, 0));
    }
    inputs.push(keyboard_input(tool_key.virtual_key, 0));
    inputs.push(keyboard_input(tool_key.virtual_key, KEYEVENTF_KEYUP));
    if tool_key.shift {
        inputs.push(keyboard_input(VK_SHIFT, KEYEVENTF_KEYUP));
    }
    if tool_key.control {
        inputs.push(keyboard_input(VK_CONTROL, KEYEVENTF_KEYUP));
    }

    let sent = unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_mut_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        )
    };
    if sent == inputs.len() as u32 {
        Ok(())
    } else {
        Err("Could not send GearBlocks overlay tool input.".to_string())
    }
}

#[cfg(not(target_os = "windows"))]
fn send_virtual_key(_tool_key: OverlayToolKey) -> Result<(), String> {
    Err("GearBlocks overlay tool input is only available on Windows.".to_string())
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
pub fn list_gearblocks_api_catalog(
    state: State<'_, AppState>,
) -> Result<GearBlocksApiCatalogRecord, String> {
    state
        .database
        .list_gearblocks_api_catalog()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn import_gearblocks_official_api_docs(
    state: State<'_, AppState>,
) -> Result<GearBlocksApiImportResult, String> {
    let scrape = scrape_official_gearblocks_api()?;
    state
        .database
        .import_gearblocks_api_catalog(&scrape)
        .map_err(|error| error.to_string())
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
pub fn list_game_runtime_part_api_members(
    game_id: i64,
    part_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<GameRuntimePartApiMemberRecord>, String> {
    let game = state
        .database
        .get_game(game_id)
        .map_err(|error| error.to_string())?;
    require_gearblocks_game(&game)?;

    state
        .database
        .list_game_runtime_part_api_members(game.id, part_id)
        .map_err(|error| error.to_string())
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
            .upsert_game_catalog_object(GameCatalogObjectDraft {
                game_id: game.id,
                name: seed.name,
                object_type: "part",
                category: seed.category,
                category_icon: resolved_category.category_icon,
                category_icon_path: &resolved_category.category_icon_path,
                description: &description,
                notes: &notes,
                tags: &tags,
                thumbnail_path: &resolved_category.source_path_text,
                source_screenshot_path: &resolved_category.source_path_text,
            })
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
        GameScreenshotCaptureOptions {
            capture_scope: "visible-game-display",
            method_source: "windows-gdi-bitblt-foreground-window",
            overlay_handling: "hide Overlay Forge before capture, then restore it",
            capture_status: "captured_windows_gdi",
            notes: "Captured through Windows GDI BitBlt from the foreground window after hiding Overlay Forge. Alpha was forced to 255 before PNG encoding.",
            restore_overlay: true,
            restore_chat_overlay: false,
            capture: capture_foreground_window_to_png,
        },
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
        GameScreenshotCaptureOptions {
            capture_scope: "visible-game-display",
            method_source: "windows-gdi-bitblt-foreground-window",
            overlay_handling: "hide Overlay Forge before capture and leave focus with the game",
            capture_status: "captured_windows_gdi_chat",
            notes: "Captured through Windows GDI BitBlt from the foreground window after hiding Overlay Forge. Alpha was forced to 255 before PNG encoding and the screenshot was attached to the current Gaming chat prompt.",
            restore_overlay: false,
            restore_chat_overlay: true,
            capture: capture_foreground_window_to_png,
        },
    )
}

struct GameScreenshotCaptureOptions {
    capture_scope: &'static str,
    method_source: &'static str,
    overlay_handling: &'static str,
    capture_status: &'static str,
    notes: &'static str,
    restore_overlay: bool,
    restore_chat_overlay: bool,
    capture: fn(&std::path::Path) -> Result<(), String>,
}

fn create_game_screenshot_capture(
    game_id: i64,
    timestamp_label: String,
    app: AppHandle,
    window: WebviewWindow,
    state: State<'_, AppState>,
    options: GameScreenshotCaptureOptions,
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
        "captureScope": options.capture_scope,
        "includeOverlay": false,
        "targetFilePath": target_file_path_text,
        "method": {
            "source": options.method_source,
            "format": "png",
            "colorSpace": "sRGB",
            "forceAlpha": 255,
            "filenamePattern": "GameName_YYYYMMDD_HHMMSS_unique.png",
            "overlayHandling": options.overlay_handling,
            "knownRisk": "GDI capture may still fail or produce black frames for some hardware-accelerated or protected game surfaces."
        }
    });
    let request_payload_text =
        serde_json::to_string_pretty(&request_payload).map_err(|error| error.to_string())?;
    fs::write(&request_file_path, request_payload_text).map_err(|error| error.to_string())?;

    let invoking_window_label = window.label().to_string();
    let hidden_overlay_windows = hide_overlay_windows_for_capture(&app)?;
    thread::sleep(Duration::from_millis(350));

    let capture_result = (options.capture)(&target_file_path);

    if options.restore_overlay || capture_result.is_err() {
        let _ = window.show();
        let _ = window.set_focus();
    }
    restore_previously_visible_overlay_windows(&hidden_overlay_windows, &invoking_window_label);

    capture_result?;

    if options.restore_chat_overlay {
        restore_game_chat_overlay_after_capture(&app, &state)?;
    }

    state
        .database
        .create_game_screenshot_capture_request(GameScreenshotCaptureRequestDraft {
            game_id: game.id,
            title: &title,
            file_path: &target_file_path_text,
            request_id: &request_id,
            request_path: &request_file_path_text,
            capture_status: options.capture_status,
            captured_at: &timestamp_label,
            notes: options.notes,
        })
        .map_err(|error| error.to_string())
}

struct HiddenOverlayCaptureWindow {
    kind: WindowKind,
    window: WebviewWindow,
    was_visible: bool,
}

fn hide_overlay_windows_for_capture(
    app: &AppHandle,
) -> Result<Vec<HiddenOverlayCaptureWindow>, String> {
    let mut hidden_windows = Vec::new();
    let window_manager = WindowManager::new(app);

    for kind in windows::CAPTURE_WINDOW_KINDS {
        let Some(window) = window_manager.window(kind) else {
            continue;
        };
        let was_visible = window.is_visible().map_err(|error| error.to_string())?;
        if was_visible {
            window.hide().map_err(|error| error.to_string())?;
        }
        hidden_windows.push(HiddenOverlayCaptureWindow {
            kind,
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
        if !hidden_window.was_visible || hidden_window.kind.label() == invoking_window_label {
            continue;
        }
        let _ = hidden_window.window.set_always_on_top(true);
        let _ = windows::ensure_window_accepts_mouse_input(&hidden_window.window);
        let _ = windows::set_overlay_opacity(
            &hidden_window.window,
            hidden_window.kind.runtime_config().restore_opacity,
        );
        let _ = windows::show_window_without_activation(&hidden_window.window);
    }
}

fn restore_game_chat_overlay_after_capture(
    app: &AppHandle,
    state: &State<'_, AppState>,
) -> Result<(), String> {
    let window_manager = WindowManager::new(app);
    if window_manager.window(WindowKind::GameChat).is_none() {
        return Ok(());
    }
    window_manager.show_without_activation(WindowKind::GameChat)?;
    let _ = windows::focus_last_game_window_from_state(state.inner());

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
    include_scene_diff: Option<bool>,
    state: State<'_, AppState>,
) -> Result<Vec<GameChatMessageRecord>, String> {
    require_text(&content, "Message")?;
    let include_scene_diff = include_scene_diff.unwrap_or(false);
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
    let custom_context = game_custom_prompt_context(&state, &game, include_scene_diff)?;
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
    include_scene_diff: bool,
) -> Result<String, String> {
    match game.slug.as_str() {
        "gearblocks" => gearblocks_prompt_context(state, game.id, include_scene_diff),
        _ => Ok(String::new()),
    }
}

fn gearblocks_prompt_context(
    state: &State<'_, AppState>,
    game_id: i64,
    include_scene_diff: bool,
) -> Result<String, String> {
    let mut sections = vec![gearblocks_units_prompt_context()];
    if gearblocks_markers_enabled() {
        sections.push(gearblocks_marker_prompt_context());
    }
    sections.push(gearblocks_parts_catalog_prompt_context()?);
    if let Some(build_guide_context) =
        gearblocks_current_build_guide_prompt_context(state, game_id)?
    {
        sections.push(build_guide_context);
    }
    if let Some(saved_context) = gearblocks_latest_saved_construction_context(state, game_id)? {
        sections.push(saved_context);
    }
    if let Some(runtime_context) =
        gearblocks_latest_runtime_understanding_context(state, game_id, include_scene_diff)?
    {
        sections.push(runtime_context);
    }
    Ok(sections.join("\n\n---\n\n"))
}

fn gearblocks_build_guide_import_prompt_context(
    state: &State<'_, AppState>,
    game_id: i64,
) -> Result<String, String> {
    let mut sections = vec![
        gearblocks_units_prompt_context(),
        gearblocks_parts_catalog_prompt_context()?,
    ];
    if let Some(saved_context) = gearblocks_latest_saved_construction_context(state, game_id)? {
        sections.push(saved_context);
    }
    if let Some(runtime_context) =
        gearblocks_latest_runtime_understanding_context(state, game_id, false)?
    {
        sections.push(runtime_context);
    }
    Ok(sections.join("\n\n---\n\n"))
}

fn gearblocks_markers_enabled() -> bool {
    false
}

fn gearblocks_units_prompt_context() -> String {
    [
        "# GearBlocks Scale And Units",
        "Use GearBlocks metric scale for build advice: 1 GearBlocks unit = 10 cm in real life. A 0.5 unit plate is 5 cm thick, and 16 units is 160 cm.",
        "Use the user-tested player character height as 20 GearBlocks units / 20 blocks / 200 cm. Use this for cabins, roll cages, cockpit clearance, doors, standing clearance, ladders, steps, and other human-scale build features.",
        "When suggesting part movement, spacing, dimensions, or alignment, answer in centimeters and/or GearBlocks units such as 1 unit, 0.5 units, 16 units. Do not give imperial-distance suggestions such as inches or feet unless the user explicitly asks for imperial conversion.",
        "Scale caveat: the developer noted that the player character, wheels, and other parts are slightly oversized to allow room for gears and other parts inside vehicles. Treat those parts as gameplay-clearance exceptions rather than strict real-world scale references.",
    ]
    .join("\n")
}

fn gearblocks_marker_prompt_context() -> String {
    [
        "# Overlay Forge In-Game Markers",
        "When a response would be clearer with in-game visual references, include a final fenced code block labelled `overlay-forge-markers`.",
        "The marker block must be valid JSON shaped as `{ \"markers\": [{ \"label\": string, \"reason\": string, \"x\": number, \"y\": number, \"z\": number, \"color\": \"#55f0c8\", \"durationSeconds\": 45, \"size\": 4.0 }] }`.",
        "Use GearBlocks world coordinates from the latest runtime scene context when available. Prefer markers for connection points, part endpoints, missing drivetrain links, steering pivots, suspension hardpoints, alignment corrections, and places where a part should move or attach.",
        "Keep marker counts small, normally 1-5. Explain the marker purpose in normal prose before the block. Do not include markers when coordinates are uncertain; ask for a fresh scene export or screenshot instead.",
    ]
    .join("\n")
}

fn gearblocks_current_build_guide_prompt_context(
    state: &State<'_, AppState>,
    game_id: i64,
) -> Result<Option<String>, String> {
    let guide = active_or_latest_game_build_guide(state.inner(), game_id)?
        .filter(|guide| guide.game_id == game_id);
    let Some(guide) = guide else {
        return Ok(None);
    };

    let parts = state
        .database
        .list_game_build_guide_parts(guide.id)
        .map_err(|error| error.to_string())?;
    let steps = state
        .database
        .list_game_build_guide_steps(guide.id)
        .map_err(|error| error.to_string())?;
    let checklist = serde_json::from_str::<Vec<String>>(&guide.checklist_json).unwrap_or_default();

    let mut sections = vec![
        "# Current GearBlocks Build Guide".to_string(),
        "Treat this guide as the active build plan for this chat session. Use it when answering follow-up questions, explaining terminology, checking progress, or suggesting next steps. Keep advice relative to the guide's reference parts, jigs, and subassemblies rather than absolute world coordinates.".to_string(),
        format!("Title: {}", guide.title),
    ];
    if !guide.build_goal.trim().is_empty() {
        sections.push(format!("## Build Goal\n{}", guide.build_goal.trim()));
    }
    if !guide.scale_reference.trim().is_empty() {
        sections.push(format!(
            "## Scale Reference\n{}",
            guide.scale_reference.trim()
        ));
    }
    if !guide.geometry_notes.trim().is_empty() {
        sections.push(format!(
            "## Geometry Notes\n{}",
            guide.geometry_notes.trim()
        ));
    }
    if !guide.glossary_text.trim().is_empty() {
        sections.push(format!("## Glossary\n{}", guide.glossary_text.trim()));
    } else {
        sections.push("## Glossary\nNo glossary was parsed for this guide. When using real-life terms, define them in terms of exact GearBlocks parts or relative subassemblies before relying on them.".to_string());
    }

    if !parts.is_empty() {
        let mut rows = parts
            .iter()
            .take(80)
            .map(|part| {
                format!(
                    "- [{}] {} x{}: {}",
                    if part.section.trim().is_empty() {
                        "Parts"
                    } else {
                        part.section.trim()
                    },
                    part.part_name.trim(),
                    if part.quantity.trim().is_empty() {
                        "?"
                    } else {
                        part.quantity.trim()
                    },
                    part.purpose.trim()
                )
            })
            .collect::<Vec<_>>();
        if parts.len() > rows.len() {
            rows.push(format!(
                "- {} additional build-guide part row(s) omitted from prompt context for size.",
                parts.len() - rows.len()
            ));
        }
        sections.push(format!("## Build Guide Parts\n{}", rows.join("\n")));
    }

    if !steps.is_empty() {
        let mut rows = steps
            .iter()
            .take(30)
            .map(|step| {
                format!(
                    "### {}. {}\n{}",
                    step.step_number,
                    step.title.trim(),
                    step.body.trim()
                )
            })
            .collect::<Vec<_>>();
        if steps.len() > rows.len() {
            rows.push(format!(
                "{} additional build-guide step(s) omitted from prompt context for size.",
                steps.len() - rows.len()
            ));
        }
        sections.push(format!("## Build Guide Steps\n{}", rows.join("\n")));
    }

    if !checklist.is_empty() {
        sections.push(format!(
            "## First Test Checklist\n{}",
            checklist
                .iter()
                .take(30)
                .map(|item| format!("- {item}"))
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }

    Ok(Some(sections.join("\n\n")))
}

fn active_or_latest_game_build_guide(
    state: &AppState,
    game_id: i64,
) -> Result<Option<GameBuildGuideRecord>, String> {
    let active_selection = state
        .active_game_build_guide_overlay
        .lock()
        .ok()
        .and_then(|selection| selection.clone())
        .or_else(|| stored_build_guide_overlay_selection(state))
        .filter(|selection| selection.game_id == game_id);

    if let Some(selection) = active_selection {
        if let Ok(guide) = state.database.get_game_build_guide(selection.guide_id) {
            if guide.game_id == game_id {
                return Ok(Some(guide));
            }
        }
    }

    state
        .database
        .latest_game_build_guide(game_id)
        .map_err(|error| error.to_string())
}

fn gearblocks_latest_saved_construction_context(
    state: &AppState,
    game_id: i64,
) -> Result<Option<String>, String> {
    let root = gearblocks_saved_constructions_root(state, game_id)?;
    let Some(file) = latest_gearblocks_construction_file_in_root(&root)? else {
        return Ok(None);
    };
    let previous_records = state
        .database
        .list_game_constructions(game_id)
        .map_err(|error| error.to_string())?;
    let previous_summary = previous_records
        .iter()
        .find(|record| record.construction_path == file.construction_path)
        .and_then(|record| {
            serde_json::from_str::<GearBlocksConstructionSummaryRecord>(&record.summary_json).ok()
        });
    let decoded = decode_gearblocks_construction_path(Path::new(&file.construction_path))?;
    let indexed_at = unix_timestamp_label();
    index_decoded_gearblocks_construction(state, game_id, &decoded, &indexed_at)?;

    Ok(Some(gearblocks_saved_construction_context(
        &decoded,
        previous_summary.as_ref(),
    )))
}

fn latest_gearblocks_construction_file_in_root(
    root: &Path,
) -> Result<Option<GearBlocksConstructionFileRecord>, String> {
    let mut latest: Option<(SystemTime, GearBlocksConstructionFileRecord)> = None;
    for file in list_gearblocks_construction_files_in_root(root)? {
        let modified = fs::metadata(&file.construction_path)
            .and_then(|metadata| metadata.modified())
            .unwrap_or(UNIX_EPOCH);
        if latest
            .as_ref()
            .is_none_or(|(latest_modified, _)| modified > *latest_modified)
        {
            latest = Some((modified, file));
        }
    }

    Ok(latest.map(|(_, file)| file))
}

fn gearblocks_saved_construction_context(
    decoded: &GearBlocksConstructionDecodeRecord,
    previous_summary: Option<&GearBlocksConstructionSummaryRecord>,
) -> String {
    let summary = &decoded.summary;
    let mut sections = Vec::new();
    sections.push("# GearBlocks Saved Construction".to_string());
    sections.push(format!(
        "Source: latest modified saved construction file `{}`. This reflects the saved `construction.bytes` state, including saved part additions and removals, but does not include live runtime-only API metadata.",
        decoded.construction_path
    ));
    sections.push(format!(
        "Saved construction `{}`: {} composite(s), {} part(s), {} unique asset GUID(s), {} attachment(s), {} link(s), {} intersection(s). Frozen: {}. Invulnerable: {}.",
        decoded.name,
        summary.composite_count,
        summary.part_count,
        summary.unique_asset_guid_count,
        summary.attachment_count,
        summary.link_count,
        summary.intersection_count,
        option_bool_label(summary.is_frozen),
        option_bool_label(summary.is_invulnerable)
    ));

    if let Some(previous) = previous_summary {
        sections.push(gearblocks_saved_construction_change_summary(
            previous, summary,
        ));
    }
    sections.push(gearblocks_saved_construction_asset_inventory(summary));

    sections.join("\n\n")
}

fn gearblocks_saved_construction_change_summary(
    previous: &GearBlocksConstructionSummaryRecord,
    current: &GearBlocksConstructionSummaryRecord,
) -> String {
    let previous_counts = gearblocks_saved_asset_guid_counts(previous);
    let current_counts = gearblocks_saved_asset_guid_counts(current);
    let mut added = Vec::new();
    let mut removed = Vec::new();

    for (asset_guid, count) in &current_counts {
        let previous_count = previous_counts.get(asset_guid).copied().unwrap_or_default();
        if *count > previous_count {
            added.push(format!("{} x{}", asset_guid, count - previous_count));
        }
    }
    for (asset_guid, count) in &previous_counts {
        let current_count = current_counts.get(asset_guid).copied().unwrap_or_default();
        if *count > current_count {
            removed.push(format!("{} x{}", asset_guid, count - current_count));
        }
    }
    added.sort();
    removed.sort();

    let mut lines = vec![format!(
        "Part count changed by {:+}.",
        current.part_count as i64 - previous.part_count as i64
    )];
    if !added.is_empty() {
        lines.push(format!("Added asset GUID counts: {}.", added.join(", ")));
    }
    if !removed.is_empty() {
        lines.push(format!(
            "Removed asset GUID counts: {}.",
            removed.join(", ")
        ));
    }
    if added.is_empty() && removed.is_empty() && current.part_count == previous.part_count {
        lines.push(
            "No saved part count or asset GUID count changes detected since the previous index."
                .to_string(),
        );
    }

    format!(
        "## Saved File Change Since Previous Index\n{}",
        lines.join("\n")
    )
}

fn gearblocks_saved_asset_guid_counts(
    summary: &GearBlocksConstructionSummaryRecord,
) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for part in &summary.parts {
        let key = if part.asset_guid.trim().is_empty() {
            "unknown".to_string()
        } else {
            part.asset_guid.clone()
        };
        *counts.entry(key).or_default() += 1;
    }
    counts
}

fn gearblocks_saved_construction_asset_inventory(
    summary: &GearBlocksConstructionSummaryRecord,
) -> String {
    let mut rows = summary
        .parts
        .iter()
        .map(|part| {
            let dimensions = if part.dimensions.is_empty() {
                "dims=unknown".to_string()
            } else {
                format!(
                    "dims=({})",
                    part.dimensions
                        .iter()
                        .map(|value| format!("{value:.2}"))
                        .collect::<Vec<_>>()
                        .join(",")
                )
            };
            let behaviours = if part.behaviours.is_empty() {
                "behaviours=none".to_string()
            } else {
                format!("behaviours={}", part.behaviours.join(", "))
            };
            format!(
                "- idx {} composite {}.{} assetGuid={} {}; {}",
                part.index,
                part.composite_index,
                part.composite_part_index,
                if part.asset_guid.is_empty() {
                    "unknown"
                } else {
                    &part.asset_guid
                },
                dimensions,
                behaviours
            )
        })
        .take(140)
        .collect::<Vec<_>>();

    if summary.parts.len() > rows.len() {
        rows.push(format!(
            "- {} additional saved part(s) omitted from prompt context for size.",
            summary.parts.len() - rows.len()
        ));
    }

    if rows.is_empty() {
        "## Saved Parts\nNo saved parts identified.".to_string()
    } else {
        format!("## Saved Parts\n{}", rows.join("\n"))
    }
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

fn gearblocks_plugin_root() -> Result<PathBuf, String> {
    Ok(gearblocks_default_user_data_root()?.join("OverlayForgePlugin"))
}

fn gearblocks_plugin_command_directory() -> Result<PathBuf, String> {
    Ok(gearblocks_plugin_root()?.join("commands"))
}

fn gearblocks_plugin_status_directory() -> Result<PathBuf, String> {
    Ok(gearblocks_plugin_root()?.join("status"))
}

fn gearblocks_game_install_root(state: &AppState, game_id: i64) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(locations) = state.database.list_game_data_locations(game_id) {
        for location in locations {
            let configured = PathBuf::from(location.directory_path.trim());
            if configured.join("GearBlocks.exe").is_file() {
                candidates.push(configured.clone());
            }
            if configured
                .file_name()
                .is_some_and(|name| name.to_string_lossy() == "SavedConstructions")
            {
                if let Some(root) = configured.parent() {
                    if root.join("GearBlocks.exe").is_file() {
                        candidates.push(root.to_path_buf());
                    }
                }
            }
        }
    }

    if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
        candidates.push(
            PathBuf::from(program_files_x86)
                .join("Steam")
                .join("steamapps")
                .join("common")
                .join("GearBlocks"),
        );
    }
    if let Ok(program_files) = std::env::var("ProgramFiles") {
        candidates.push(
            PathBuf::from(program_files)
                .join("Steam")
                .join("steamapps")
                .join("common")
                .join("GearBlocks"),
        );
    }

    candidates
        .into_iter()
        .find(|candidate| candidate.join("GearBlocks.exe").is_file())
}

fn gearblocks_bepinex_status(
    game_root: Option<&Path>,
) -> GearBlocksThirdPartyDependencyStatusRecord {
    let expected_path = game_root
        .map(|root| root.join("BepInEx"))
        .unwrap_or_else(|| PathBuf::from("GearBlocks game root\\BepInEx"));
    let mut status_details = Vec::new();
    let mut log_paths = Vec::new();
    let mut installed_version = None;
    let mut is_activated = false;
    let mut is_installed_correctly = false;

    let detected = game_root.is_some_and(|root| {
        let bepinex_root = root.join("BepInEx");
        bepinex_root.is_dir()
            && (root.join("winhttp.dll").is_file()
                || root.join("doorstop_config.ini").is_file()
                || bepinex_root.join("config").is_dir()
                || bepinex_root.join("LogOutput.log").is_file()
                || bepinex_root.join("LogOutput.txt").is_file()
                || bepinex_root.join("plugins").is_dir())
    });

    if let Some(root) = game_root {
        let bepinex_root = root.join("BepInEx");
        let has_bepinex_dir = bepinex_root.is_dir();
        let has_winhttp = root.join("winhttp.dll").is_file();
        let has_doorstop_config = root.join("doorstop_config.ini").is_file();
        let has_core = bepinex_root.join("core").is_dir();
        let has_config = bepinex_root.join("config").is_dir();
        let log_records = read_bepinex_log_records(&bepinex_root);

        log_paths = log_records
            .iter()
            .map(|record| record.path.to_string_lossy().to_string())
            .collect();
        installed_version = log_records
            .iter()
            .find_map(|record| find_bepinex_version(&record.content));
        is_activated = log_records.iter().any(|record| {
            record.content.contains("Chainloader startup complete")
                || record.content.contains("Chainloader initialized")
        });
        let has_log_output = log_records.iter().any(|record| {
            record
                .path
                .file_name()
                .is_some_and(|name| name.to_string_lossy() == "LogOutput.log")
        });

        if has_bepinex_dir {
            status_details.push("BepInEx folder found.".to_string());
        }
        if has_winhttp {
            status_details.push("Doorstop loader winhttp.dll found.".to_string());
        }
        if has_doorstop_config {
            status_details.push("doorstop_config.ini found.".to_string());
        }
        if has_core {
            status_details.push("BepInEx/core folder found.".to_string());
        }
        if has_config {
            status_details.push("BepInEx/config folder found.".to_string());
        }
        if let Some(version) = &installed_version {
            status_details.push(format!("Log reports BepInEx version {version}."));
        }
        if is_activated {
            status_details.push("Log reports BepInEx chainloader startup complete.".to_string());
        }
        if !log_paths.is_empty() {
            status_details.push(format!("Read {} BepInEx log file(s).", log_paths.len()));
        }

        is_installed_correctly = has_bepinex_dir
            && has_winhttp
            && has_doorstop_config
            && has_core
            && has_config
            && has_log_output
            && installed_version.is_some();
    }

    let detail = if is_installed_correctly && is_activated {
        "Installed correctly and successfully activated.".to_string()
    } else if detected && is_installed_correctly {
        "Installed correctly, but activation was not confirmed in the BepInEx logs.".to_string()
    } else if detected {
        "Detected BepInEx files, but installation looks incomplete or has not generated a usable log yet.".to_string()
    } else if game_root.is_some() {
        "Not detected. Install BepInEx 6 for Unity IL2CPP into the GearBlocks game root and run the game once.".to_string()
    } else {
        "GearBlocks install root was not detected. Configure a GearBlocks game root or install location before status can be confirmed.".to_string()
    };

    GearBlocksThirdPartyDependencyStatusRecord {
        name: "BepInEx".to_string(),
        is_detected: detected,
        is_installed_correctly: Some(is_installed_correctly),
        is_activated: Some(is_activated),
        installed_version,
        expected_path: expected_path.to_string_lossy().to_string(),
        detail,
        status_details,
        log_paths,
        project_url:
            "https://docs.bepinex.dev/master/articles/user_guide/installation/unity_il2cpp.html"
                .to_string(),
    }
}

fn gearblocks_gearlib_status(
    game_root: Option<&Path>,
) -> GearBlocksThirdPartyDependencyStatusRecord {
    let expected_path = game_root
        .map(|root| root.join("BepInEx").join("plugins"))
        .unwrap_or_else(|| PathBuf::from("GearBlocks game root\\BepInEx\\plugins"));
    let detected = game_root
        .map(|root| gearblocks_gearlib_plugin_exists(&root.join("BepInEx").join("plugins")))
        .unwrap_or(false);
    let detail = if detected {
        "Detected GearLib under BepInEx/plugins.".to_string()
    } else if game_root.is_some() {
        "Not detected. GearLib is a third-party library and must be installed separately by the user into BepInEx/plugins.".to_string()
    } else {
        "GearBlocks install root was not detected. GearLib status cannot be confirmed.".to_string()
    };

    GearBlocksThirdPartyDependencyStatusRecord {
        name: "GearLib".to_string(),
        is_detected: detected,
        is_installed_correctly: Some(detected),
        is_activated: None,
        installed_version: None,
        expected_path: expected_path.to_string_lossy().to_string(),
        detail,
        status_details: Vec::new(),
        log_paths: Vec::new(),
        project_url: "https://github.com/KaBooMa/GearLib".to_string(),
    }
}

struct BepInExLogRecord {
    path: PathBuf,
    content: String,
}

fn read_bepinex_log_records(bepinex_root: &Path) -> Vec<BepInExLogRecord> {
    ["LogOutput.log", "ErrorLog.log", "LogOutput.txt"]
        .into_iter()
        .filter_map(|file_name| {
            let path = bepinex_root.join(file_name);
            fs::read_to_string(&path)
                .ok()
                .map(|content| BepInExLogRecord { path, content })
        })
        .collect()
}

fn find_bepinex_version(content: &str) -> Option<String> {
    content.lines().find_map(|line| {
        line.find("BepInEx ").and_then(|index| {
            line[index + "BepInEx ".len()..]
                .split_whitespace()
                .next()
                .map(|version| version.trim_matches('-').to_string())
                .filter(|version| !version.is_empty())
        })
    })
}

fn gearblocks_gearlib_plugin_exists(plugins_root: &Path) -> bool {
    if !plugins_root.is_dir() {
        return false;
    }

    if plugins_root.join("GearLib.dll").is_file() {
        return true;
    }

    fs::read_dir(plugins_root)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(|entry| entry.ok()))
        .any(|entry| {
            let path = entry.path();
            path.is_dir() && path.join("GearLib.dll").is_file()
        })
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
    start_byte: usize,
}

#[derive(Deserialize, Serialize)]
struct GearBlocksRuntimeLogCursor {
    path: String,
    offset: u64,
    modified: String,
}

struct GearBlocksRuntimeExportParseResult {
    exports: Vec<GearBlocksRuntimeExportRecord>,
    consumed_bytes: usize,
}

fn parse_gearblocks_runtime_exports_from_log(
    log_path: &Path,
) -> Result<Vec<GearBlocksRuntimeExportRecord>, String> {
    let text = fs::read_to_string(log_path)
        .map_err(|error| format!("Could not read GearBlocks runtime export log: {error}"))?;
    Ok(parse_gearblocks_runtime_exports_from_text(log_path, &text).exports)
}

fn parse_gearblocks_part_aliases_from_log(
    log_path: &Path,
) -> Result<Vec<GearBlocksPartAliasLogRecord>, String> {
    let text = fs::read_to_string(log_path)
        .map_err(|error| format!("Could not read GearBlocks runtime export log: {error}"))?;
    Ok(parse_gearblocks_part_aliases_from_text(log_path, &text))
}

fn parse_gearblocks_part_aliases_from_text(
    log_path: &Path,
    text: &str,
) -> Vec<GearBlocksPartAliasLogRecord> {
    const ALIAS_MARKER: &str = "[OverlayForgePartAlias]";
    let mut aliases = Vec::new();
    let source_log_path = log_path.to_string_lossy().to_string();

    for line in text.lines() {
        let Some(index) = line.find(ALIAS_MARKER) else {
            continue;
        };
        let payload = &line[index + ALIAS_MARKER.len()..];
        let Ok(document) = serde_json::from_str::<serde_json::Value>(payload) else {
            continue;
        };
        let part_instance_key = json_string(document.get("partInstanceKey"))
            .trim()
            .to_string();
        let friendly_name = json_string(document.get("friendlyName")).trim().to_string();
        let emitted_at = json_string(document.get("emittedAt"));
        if part_instance_key.is_empty() || friendly_name.is_empty() {
            continue;
        }
        aliases.push(GearBlocksPartAliasLogRecord {
            part_instance_key,
            friendly_name,
            emitted_at,
            source_log_path: source_log_path.clone(),
            document,
        });
    }

    aliases
}

fn parse_gearblocks_runtime_exports_from_text(
    log_path: &Path,
    text: &str,
) -> GearBlocksRuntimeExportParseResult {
    const BEGIN_MARKER: &str = "[OverlayForgeExportBegin]";
    const DATA_MARKER: &str = "[OverlayForgeExportData]";
    const END_MARKER: &str = "[OverlayForgeExportEnd]";

    let mut pending: HashMap<String, PendingGearBlocksRuntimeExport> = HashMap::new();
    let mut exports = Vec::new();
    let source_log_path = log_path.to_string_lossy().to_string();
    let mut line_start = 0usize;
    let mut consumed_bytes = 0usize;

    for raw_line in text.split_inclusive('\n') {
        let line_end = line_start + raw_line.len();
        let line = raw_line.trim_end_matches(['\r', '\n']);
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
                            start_byte: line_start,
                        },
                    );
                }
            }
            line_start = line_end;
            continue;
        }

        if let Some(index) = line.find(DATA_MARKER) {
            let payload = &line[index + DATA_MARKER.len()..];
            if let Some((id, chunk)) = payload.split_once('|') {
                if let Some(export) = pending.get_mut(id) {
                    export.chunks.push(chunk.to_string());
                }
            }
            line_start = line_end;
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
            if pending.is_empty() {
                consumed_bytes = line_end;
            }
        }

        line_start = line_end;
    }

    if pending.is_empty() {
        consumed_bytes = text.len();
    } else if let Some(first_pending) = pending.values().map(|pending| pending.start_byte).min() {
        consumed_bytes = if consumed_bytes == 0 {
            first_pending
        } else {
            consumed_bytes.min(first_pending)
        };
    }

    GearBlocksRuntimeExportParseResult {
        exports,
        consumed_bytes,
    }
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
    part_json: &'a serde_json::Value,
    id: i64,
    index: i64,
    instance_key: String,
    name: String,
    friendly_name: Option<String>,
    category: String,
    parent_construction_id: String,
    system: &'static str,
    purpose: &'static str,
    behaviours: Vec<String>,
    local_position: Option<&'a serde_json::Value>,
    world_position: Option<&'a serde_json::Value>,
    current_unit_size: Option<&'a serde_json::Value>,
    link_node_count: usize,
    mass: f64,
    is_structural: bool,
    is_functional: bool,
}

fn gearblocks_latest_runtime_understanding_context(
    state: &AppState,
    game_id: i64,
    include_scene_diff: bool,
) -> Result<Option<String>, String> {
    let scene_context_service = GearBlocksSceneContextService::new(&state.database);
    if let Some(mut context) = scene_context_service.render_current_scene_context(game_id)? {
        if include_scene_diff {
            if let Some(diff_summary) = state
                .database
                .get_app_setting(&gearblocks_runtime_scene_diff_key(game_id))
                .map_err(|error| error.to_string())?
            {
                context.push_str("\n\n");
                context.push_str(&diff_summary);
            }
        }
        return Ok(Some(context));
    }

    let Some(latest) = state
        .database
        .latest_game_runtime_construction_export(game_id)
        .map_err(|error| error.to_string())?
    else {
        return Ok(None);
    };
    if latest.document_json.trim() == "{}" {
        return Ok(None);
    }

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
    let aliases = state
        .database
        .list_game_runtime_part_aliases(game_id)
        .map_err(|error| error.to_string())?;

    let mut context = gearblocks_runtime_understanding_context(&export, &aliases);
    if include_scene_diff {
        if let Some(diff_summary) = state
            .database
            .get_app_setting(&gearblocks_runtime_scene_diff_key(game_id))
            .map_err(|error| error.to_string())?
        {
            context.push_str("\n\n");
            context.push_str(&diff_summary);
        }
    }

    Ok(Some(context))
}

fn gearblocks_runtime_understanding_context(
    export: &GearBlocksRuntimeExportRecord,
    aliases: &[GameRuntimePartAliasRecord],
) -> String {
    let parts_json = export
        .document
        .get("parts")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    let mut parts = Vec::new();
    let alias_map = aliases
        .iter()
        .map(|alias| (alias.part_instance_key.clone(), alias.friendly_name.clone()))
        .collect::<HashMap<_, _>>();

    for part in &parts_json {
        let instance_key = gearblocks_part_instance_key(part).unwrap_or_default();
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
            part_json: part,
            id: json_i64(part.get("id")),
            index: json_i64(part.get("index")),
            friendly_name: alias_map.get(&instance_key).cloned(),
            instance_key,
            name,
            category,
            parent_construction_id: part
                .get("parentConstructionId")
                .map(json_value_to_string)
                .unwrap_or_default(),
            system,
            purpose,
            behaviours,
            local_position: part.get("localPosition"),
            world_position: part.get("position"),
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
        "Source: latest runtime export `{}` reconstructed from `{}`. Intended output path: `{}`. Exported at: `{}`.",
        export.id,
        export.source_log_path,
        export.intended_path,
        json_string(export.document.get("exportedAt"))
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
    sections.push(gearblocks_part_aliases_section(&parts, aliases));
    sections.push(gearblocks_marker_coordinate_reference_section(&parts));
    sections.push(gearblocks_construction_groups_section(&parts));
    sections.push(gearblocks_build_guide_api_context_section());
    sections.push(gearblocks_build_guide_runtime_details_section(&parts));
    sections.push(gearblocks_structural_bounds_section(&parts));
    sections.push(gearblocks_functional_parts_section(&parts));

    sections.join("\n\n")
}

fn gearblocks_runtime_scene_diff_summary(
    previous_parts: &[GameRuntimePartInstanceRecord],
    current: &GearBlocksRuntimeExportRecord,
) -> Result<String, String> {
    let current_parts = gearblocks_runtime_part_count_map(&current.document);
    if previous_parts.is_empty() {
        return Ok(format!(
            "## Runtime Scene Change Since Previous Export\nNo previous runtime scene export was indexed. Current scene snapshot contains {} part key(s) across {} runtime part(s).",
            current_parts.len(),
            gearblocks_runtime_document_part_count(&current.document)
        ));
    }

    let previous_part_counts = gearblocks_runtime_part_count_map_from_instances(previous_parts);
    let mut added = Vec::new();
    let mut removed = Vec::new();

    for (key, current_part) in &current_parts {
        let previous_count = previous_part_counts
            .get(key)
            .map(|part| part.count)
            .unwrap_or_default();
        if current_part.count > previous_count {
            added.push(format!(
                "{} x{}",
                current_part.label,
                current_part.count - previous_count
            ));
        }
    }

    for (key, previous_part) in &previous_part_counts {
        let current_count = current_parts
            .get(key)
            .map(|part| part.count)
            .unwrap_or_default();
        if previous_part.count > current_count {
            removed.push(format!(
                "{} x{}",
                previous_part.label,
                previous_part.count - current_count
            ));
        }
    }

    added.sort();
    removed.sort();
    let previous_total = previous_parts.len();
    let current_total = gearblocks_runtime_document_part_count(&current.document);
    let mut lines = vec![format!(
        "Runtime scene part count changed by {:+} ({} -> {}).",
        current_total as i64 - previous_total as i64,
        previous_total,
        current_total
    )];

    if added.is_empty() && removed.is_empty() {
        lines.push(
            "No added or removed part keys detected since the previous runtime scene export."
                .to_string(),
        );
    } else {
        if !added.is_empty() {
            lines.push(format!(
                "Added part keys: {}.",
                summarize_change_items(&added, 40)
            ));
        }
        if !removed.is_empty() {
            lines.push(format!(
                "Removed part keys: {}.",
                summarize_change_items(&removed, 40)
            ));
        }
    }

    Ok(format!(
        "## Runtime Scene Change Since Previous Export\n{}",
        lines.join("\n")
    ))
}

struct GearBlocksRuntimePartCount {
    label: String,
    count: usize,
}

fn gearblocks_runtime_part_count_map(
    document: &serde_json::Value,
) -> HashMap<String, GearBlocksRuntimePartCount> {
    let mut counts: HashMap<String, GearBlocksRuntimePartCount> = HashMap::new();
    let Some(parts) = document.get("parts").and_then(serde_json::Value::as_array) else {
        return counts;
    };

    for part in parts {
        let key = runtime_part_key(part);
        if key.is_empty() {
            continue;
        }
        let name = preferred_part_name(part);
        let category = json_string(part.get("category"));
        let label = if category.trim().is_empty() {
            name
        } else {
            format!("{category}: {name}")
        };
        counts
            .entry(key)
            .and_modify(|entry| entry.count += 1)
            .or_insert(GearBlocksRuntimePartCount { label, count: 1 });
    }

    counts
}

fn gearblocks_runtime_part_count_map_from_instances(
    parts: &[GameRuntimePartInstanceRecord],
) -> HashMap<String, GearBlocksRuntimePartCount> {
    let mut counts: HashMap<String, GearBlocksRuntimePartCount> = HashMap::new();
    for part in parts {
        let key = part.part_key.trim();
        if key.is_empty() {
            continue;
        }
        let name = if part.full_display_name.trim().is_empty() {
            part.display_name.clone()
        } else {
            part.full_display_name.clone()
        };
        let label = if part.category.trim().is_empty() {
            name
        } else {
            format!("{}: {}", part.category, name)
        };
        counts
            .entry(key.to_string())
            .and_modify(|entry| entry.count += 1)
            .or_insert(GearBlocksRuntimePartCount { label, count: 1 });
    }
    counts
}

fn gearblocks_runtime_document_part_count(document: &serde_json::Value) -> usize {
    document
        .get("parts")
        .and_then(serde_json::Value::as_array)
        .map(Vec::len)
        .unwrap_or_default()
}

fn summarize_change_items(items: &[String], limit: usize) -> String {
    let mut visible = items.iter().take(limit).cloned().collect::<Vec<_>>();
    if items.len() > limit {
        visible.push(format!("{} more", items.len() - limit));
    }
    visible.join(", ")
}

fn persist_runtime_export(
    state: &AppState,
    game_id: i64,
    export: &GearBlocksRuntimeExportRecord,
    indexed_at: &str,
) -> Result<GameRuntimeConstructionExportRecord, String> {
    state
        .database
        .upsert_game_runtime_construction_export(GameRuntimeConstructionExportDraft {
            game_id,
            export_id: &export.id,
            name: &export.name,
            export_kind: &json_string(export.document.get("exportKind")),
            intended_path: &export.intended_path,
            source_log_path: &export.source_log_path,
            byte_size: export.byte_size as i64,
            construction_id: &export
                .document
                .get("id")
                .map(json_value_to_string)
                .unwrap_or_default(),
            exported_at: export
                .document
                .get("exportedAt")
                .and_then(|value| value.as_str())
                .unwrap_or_default(),
            part_count: json_i64(export.document.get("numParts")),
            mass: json_f64(export.document.get("mass")),
            is_frozen: json_optional_bool(export.document.get("isFrozen")),
            is_invulnerable: json_optional_bool(export.document.get("isInvulnerable")),
            is_player_character: json_optional_bool(export.document.get("isPlayerCharacter")),
            document_json: "{}",
            last_indexed_at: indexed_at,
        })
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
    let mut instance_drafts = Vec::new();

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
        let world_position = part.get("position").and_then(json_vector3);
        let local_position = part.get("localPosition").and_then(json_vector3);
        let world_position_json = compact_json_or_empty_object(part.get("position"))?;
        let local_position_json = compact_json_or_empty_object(part.get("localPosition"))?;
        let current_unit_size_json =
            compact_json_or_empty_object(part.get("currentUnitSize").or_else(|| {
                part.get("resizable")
                    .and_then(|value| value.get("currentUnitSize"))
            }))?;
        let link_node_count = part
            .get("linkNodes")
            .and_then(serde_json::Value::as_array)
            .map(|items| items.len() as i64)
            .unwrap_or_default();
        let behaviour_names_json = gearblocks_part_behaviour_names_json(&part)?;
        let dynamic_summary_json = gearblocks_part_dynamic_summary_json(&part)?;
        let part_instance_key = gearblocks_part_instance_key(&part)
            .unwrap_or_else(|| format!("{}:{}", export.id, json_i64(part.get("index"))));
        let part_source_construction_id = part
            .get("parentConstructionId")
            .map(json_value_to_string)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| source_construction_id.clone());
        let properties_json =
            serde_json::to_string_pretty(&part).map_err(|error| error.to_string())?;
        state
            .database
            .upsert_game_runtime_part(GameRuntimePartDraft {
                identity: GameRuntimePartIdentity {
                    game_id,
                    part_key: &part_key,
                    asset_guid: &asset_guid,
                    asset_name: &asset_name,
                    display_name: &display_name,
                    full_display_name: &full_display_name,
                    category: &category,
                },
                mass: json_f64(part.get("mass")),
                world_position,
                local_position,
                world_position_json: &world_position_json,
                local_position_json: &local_position_json,
                properties_json: &properties_json,
                source: GameRuntimePartSource {
                    source_export_id: &export.id,
                    source_construction_id: &part_source_construction_id,
                    seen_at: &last_seen_at,
                },
            })
            .map_err(|error| error.to_string())?;
        instance_drafts.push(GameRuntimePartInstanceDraft {
            part_key: part_key.clone(),
            asset_guid: asset_guid.clone(),
            asset_name: asset_name.clone(),
            display_name: display_name.clone(),
            full_display_name: full_display_name.clone(),
            category: category.clone(),
            source_export_id: export.id.clone(),
            source_construction_id: part_source_construction_id.clone(),
            part_instance_key,
            runtime_part_id: json_i64(part.get("id")),
            runtime_part_index: json_i64(part.get("index")),
            mass: json_f64(part.get("mass")),
            world_position,
            local_position,
            world_position_json: world_position_json.clone(),
            local_position_json: local_position_json.clone(),
            current_unit_size_json,
            link_node_count,
            behaviour_names_json,
            dynamic_summary_json,
            last_seen_at: last_seen_at.clone(),
        });

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

    state
        .database
        .replace_game_runtime_part_instances(game_id, &instance_drafts)
        .map_err(|error| error.to_string())?;
    state
        .database
        .clear_game_runtime_export_documents(game_id)
        .map_err(|error| error.to_string())?;

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

impl RuntimePartIndexContext {
    fn identity(&self) -> GameRuntimePartIdentity<'_> {
        GameRuntimePartIdentity {
            game_id: self.game_id,
            part_key: &self.part_key,
            asset_guid: &self.asset_guid,
            asset_name: &self.asset_name,
            display_name: &self.display_name,
            full_display_name: &self.full_display_name,
            category: &self.category,
        }
    }

    fn source(&self) -> GameRuntimePartSource<'_> {
        GameRuntimePartSource {
            source_export_id: &self.source_export_id,
            source_construction_id: &self.source_construction_id,
            seen_at: &self.seen_at,
        }
    }
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
                .upsert_game_runtime_part_api_attribute(GameRuntimePartApiAttributeObservation {
                    identity: context.identity(),
                    interface_name: &interface_name,
                    attribute_name: &attribute_name,
                    value_type: &json_string(attribute.get("valueType")),
                    availability: &json_string(attribute.get("availability")),
                    source: context.source(),
                })
                .map_err(|error| error.to_string())?;
            state
                .database
                .upsert_game_runtime_part_api_member(GameRuntimePartApiMemberObservation {
                    game_id: context.game_id,
                    part_key: &context.part_key,
                    interface_name: &interface_name,
                    attribute_name: &attribute_name,
                    availability: &json_string(attribute.get("availability")),
                    source: context.source(),
                })
                .map_err(|error| error.to_string())?;
        }
    }

    let mut value_fields = Vec::new();
    collect_named_value_fields(part, "", &mut value_fields);
    for (field_path, value) in value_fields {
        state
            .database
            .upsert_game_runtime_part_metadata_value(GameRuntimePartMetadataValueDraft {
                game_id: context.game_id,
                part_key: &context.part_key,
                source_area: "value",
                field_path: &field_path,
                value_type: json_type_label(value),
                value_json: &value.to_string(),
                source: context.source(),
            })
            .map_err(|error| error.to_string())?;
        state
            .database
            .upsert_game_runtime_part_value(GameRuntimePartValueObservation {
                identity: context.identity(),
                field_path: &field_path,
                value_type: json_type_label(value),
                value_json: &value.to_string(),
                source: context.source(),
            })
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
                .upsert_game_runtime_part_metadata_value(GameRuntimePartMetadataValueDraft {
                    game_id: context.game_id,
                    part_key: &context.part_key,
                    source_area: "property",
                    field_path: &property_path,
                    value_type: json_type_label(value),
                    value_json: &value.to_string(),
                    source: context.source(),
                })
                .map_err(|error| error.to_string())?;
            state
                .database
                .upsert_game_runtime_part_property(GameRuntimePartPropertyObservation {
                    identity: context.identity(),
                    property_path: &property_path,
                    value_type: json_type_label(value),
                    value_json: &value.to_string(),
                    source: context.source(),
                })
                .map_err(|error| error.to_string())?;
        }
    }

    if let Some(attachments) = part.get("attachments") {
        let mut attachment_values = Vec::new();
        collect_direct_json_children(attachments, "", &mut attachment_values);
        for (attachment_path, value) in attachment_values {
            state
                .database
                .upsert_game_runtime_part_attachment_type(GameRuntimePartAttachmentTypeDraft {
                    game_id: context.game_id,
                    part_key: &context.part_key,
                    attachment_path: &attachment_path,
                    type_name: &gearblocks_attachment_type_name(&attachment_path, value),
                    value_type: json_type_label(value),
                    attachment_json: &value.to_string(),
                    source: context.source(),
                })
                .map_err(|error| error.to_string())?;
            state
                .database
                .upsert_game_runtime_part_attachment(GameRuntimePartAttachmentObservation {
                    identity: context.identity(),
                    attachment_path: &attachment_path,
                    value_type: json_type_label(value),
                    attachment_json: &value.to_string(),
                    source: context.source(),
                })
                .map_err(|error| error.to_string())?;
        }
    }

    for setting in gearblocks_part_setting_values(part) {
        state
            .database
            .upsert_game_runtime_part_setting_value(GameRuntimePartSettingValueDraft {
                game_id: context.game_id,
                part_key: &context.part_key,
                setting_key: &setting.key,
                label: &setting.label,
                setting_area: &setting.area,
                value_type: json_type_label(setting.value),
                value_json: &setting.value.to_string(),
                source: context.source(),
            })
            .map_err(|error| error.to_string())?;
    }

    for channel in gearblocks_part_output_channel_values(part) {
        state
            .database
            .upsert_game_runtime_part_output_channel_value(GameRuntimePartOutputChannelValueDraft {
                game_id: context.game_id,
                part_key: &context.part_key,
                channel_key: &channel.key,
                label: &channel.label,
                channel_area: &channel.area,
                value_type: json_type_label(channel.value),
                value_json: &channel.value.to_string(),
                source: context.source(),
            })
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

fn gearblocks_part_behaviour_names_json(part: &serde_json::Value) -> Result<String, String> {
    let names = part
        .get("behaviours")
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("name").and_then(serde_json::Value::as_str))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    serde_json::to_string(&names).map_err(|error| error.to_string())
}

fn gearblocks_part_dynamic_summary_json(part: &serde_json::Value) -> Result<String, String> {
    let mut summary = serde_json::Map::new();
    for key in [
        "paint",
        "properties",
        "attachments",
        "linkNodes",
        "tweakables",
        "resizable",
        "behaviours",
        "behaviourByName",
        "currentUnitSize",
    ] {
        if let Some(value) = part.get(key) {
            summary.insert(key.to_string(), value.clone());
        }
    }
    serde_json::to_string(&serde_json::Value::Object(summary)).map_err(|error| error.to_string())
}

struct GearBlocksObservedValue<'a> {
    key: String,
    label: String,
    area: String,
    value: &'a serde_json::Value,
}

fn gearblocks_attachment_type_name(path: &str, value: &serde_json::Value) -> String {
    if let Some(object) = value.as_object() {
        for key in ["typeName", "type", "attachmentType", "name"] {
            if let Some(text) = object.get(key).and_then(serde_json::Value::as_str) {
                if !text.trim().is_empty() {
                    return text.trim().to_string();
                }
            }
        }
    }
    path.to_string()
}

fn gearblocks_part_setting_values(part: &serde_json::Value) -> Vec<GearBlocksObservedValue<'_>> {
    let mut values = Vec::new();
    if let Some(tweakables) = part.get("tweakables") {
        collect_named_setting_values(tweakables, "tweakables", "tweakable", &mut values);
    }
    if let Some(resizable) = part.get("resizable") {
        collect_named_setting_values(resizable, "resizable", "resizable", &mut values);
    }
    if let Some(behaviours) = part.get("behaviours").and_then(serde_json::Value::as_array) {
        for (index, behaviour) in behaviours.iter().enumerate() {
            let name = json_string(behaviour.get("name"));
            let base = if name.is_empty() {
                format!("behaviours[{index}]")
            } else {
                format!("behaviours.{name}")
            };
            collect_named_setting_values(behaviour, &base, "behaviour", &mut values);
        }
    }
    values
}

fn collect_named_setting_values<'a>(
    value: &'a serde_json::Value,
    path: &str,
    area: &str,
    values: &mut Vec<GearBlocksObservedValue<'a>>,
) {
    match value {
        serde_json::Value::Object(object) => {
            if let Some(setting_value) = object.get("value") {
                let label = ["label", "name", "displayName", "key"]
                    .iter()
                    .find_map(|key| object.get(*key).and_then(serde_json::Value::as_str))
                    .unwrap_or(path)
                    .to_string();
                values.push(GearBlocksObservedValue {
                    key: path.to_string(),
                    label,
                    area: area.to_string(),
                    value: setting_value,
                });
            }
            for (key, child) in object {
                let next_path = json_path_child(path, key);
                if key == "value" {
                    continue;
                }
                if matches!(
                    key.as_str(),
                    "currentUnitSize"
                        | "resizeStep"
                        | "isActivated"
                        | "isControlBound"
                        | "isControlOverridden"
                        | "controlInfo"
                        | "currentRotationSpeed"
                        | "crankAngle"
                        | "timingAngle"
                ) {
                    values.push(GearBlocksObservedValue {
                        key: next_path.clone(),
                        label: key.clone(),
                        area: area.to_string(),
                        value: child,
                    });
                }
                collect_named_setting_values(child, &next_path, area, values);
            }
        }
        serde_json::Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                collect_named_setting_values(child, &json_path_index(path, index), area, values);
            }
        }
        _ => {}
    }
}

fn gearblocks_part_output_channel_values(
    part: &serde_json::Value,
) -> Vec<GearBlocksObservedValue<'_>> {
    let mut values = Vec::new();
    collect_output_channel_values(part, "", &mut values);
    values
}

fn collect_output_channel_values<'a>(
    value: &'a serde_json::Value,
    path: &str,
    values: &mut Vec<GearBlocksObservedValue<'a>>,
) {
    match value {
        serde_json::Value::Object(object) => {
            for (key, child) in object {
                let next_path = json_path_child(path, key);
                let normalized = key.to_ascii_lowercase();
                if matches!(
                    normalized.as_str(),
                    "output"
                        | "outputs"
                        | "outputchannel"
                        | "outputchannels"
                        | "channel"
                        | "channels"
                        | "controlinfo"
                        | "controlbinding"
                ) {
                    values.push(GearBlocksObservedValue {
                        key: next_path.clone(),
                        label: key.clone(),
                        area: "output/control".to_string(),
                        value: child,
                    });
                }
                collect_output_channel_values(child, &next_path, values);
            }
        }
        serde_json::Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                collect_output_channel_values(child, &json_path_index(path, index), values);
            }
        }
        _ => {}
    }
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
    let mut imported_count = 0usize;
    for log_path in [root.join("Player-prev.log"), root.join("Player.log")] {
        if log_path.is_file() {
            imported_count +=
                import_new_gearblocks_runtime_exports_from_log(state, game_id, &log_path, false)?;
        }
    }

    Ok(imported_count)
}

fn import_latest_gearblocks_runtime_exports_for_monitor(
    state: &AppState,
    game_id: i64,
) -> Result<usize, String> {
    let root = gearblocks_default_user_data_root()?;
    let mut imported_count = 0usize;
    for log_path in [root.join("Player-prev.log"), root.join("Player.log")] {
        if log_path.is_file() {
            imported_count +=
                import_new_gearblocks_runtime_exports_from_log(state, game_id, &log_path, true)?;
        }
    }

    Ok(imported_count)
}

fn reconcile_latest_gearblocks_runtime_exports(
    state: &AppState,
    game_id: i64,
) -> Result<usize, String> {
    let root = gearblocks_default_user_data_root()?;
    let mut imported_count = 0usize;
    for log_path in [root.join("Player-prev.log"), root.join("Player.log")] {
        if log_path.is_file() {
            imported_count +=
                reconcile_gearblocks_runtime_exports_from_log(state, game_id, &log_path)?;
        }
    }

    Ok(imported_count)
}

fn import_requested_gearblocks_runtime_export(
    state: &AppState,
    game_id: i64,
    request: &GearBlocksRequestedRuntimeExport,
) -> Result<usize, String> {
    let metadata = fs::metadata(&request.log_path)
        .map_err(|error| format!("Could not read GearBlocks runtime log metadata: {error}"))?;
    if metadata.len() <= request.initial_length {
        return Ok(0);
    }

    let mut file = File::open(&request.log_path)
        .map_err(|error| format!("Could not open GearBlocks runtime export log: {error}"))?;
    file.seek(SeekFrom::Start(request.initial_length))
        .map_err(|error| format!("Could not seek GearBlocks runtime export log: {error}"))?;
    let mut text = String::new();
    file.read_to_string(&mut text).map_err(|error| {
        format!("Could not read requested GearBlocks runtime export log additions: {error}")
    })?;

    let parse_result = parse_gearblocks_runtime_exports_from_text(&request.log_path, &text);
    let part_aliases = parse_gearblocks_part_aliases_from_text(&request.log_path, &text);
    let mut imported_count =
        persist_newer_gearblocks_runtime_exports(state, game_id, &parse_result.exports, false)?;
    imported_count += persist_gearblocks_part_aliases(state, game_id, &part_aliases)?;

    let consumed_offset = if parse_result.consumed_bytes > 0 {
        request.initial_length + parse_result.consumed_bytes as u64
    } else {
        metadata.len()
    };
    save_gearblocks_runtime_log_cursor(
        state,
        game_id,
        &request.log_path,
        consumed_offset.min(metadata.len()),
        &metadata,
    )?;

    Ok(imported_count)
}

fn import_new_gearblocks_runtime_exports_from_log(
    state: &AppState,
    game_id: i64,
    log_path: &Path,
    seed_missing_cursor_at_end: bool,
) -> Result<usize, String> {
    let metadata = fs::metadata(log_path)
        .map_err(|error| format!("Could not read GearBlocks runtime log metadata: {error}"))?;
    let cursor_key = gearblocks_runtime_log_cursor_key(game_id, log_path);
    let cursor = state
        .database
        .get_app_setting(&cursor_key)
        .map_err(|error| error.to_string())?
        .and_then(|value| serde_json::from_str::<GearBlocksRuntimeLogCursor>(&value).ok());

    if cursor.is_none() && seed_missing_cursor_at_end {
        save_gearblocks_runtime_log_cursor(state, game_id, log_path, metadata.len(), &metadata)?;
        return Ok(0);
    }

    let log_path_text = log_path.to_string_lossy().to_string();
    let cursor_matches_path = cursor
        .as_ref()
        .is_some_and(|cursor| cursor.path == log_path_text);
    let cursor_is_rotated = cursor
        .as_ref()
        .is_some_and(|cursor| cursor.path == log_path_text && metadata.len() < cursor.offset);
    let initial_offset = if cursor.is_none() || !cursor_matches_path {
        metadata
            .len()
            .saturating_sub(GEARBLOCKS_RUNTIME_INITIAL_TAIL_BYTES)
    } else {
        0
    };
    let mut offset = cursor
        .as_ref()
        .filter(|cursor| cursor.path == log_path_text && metadata.len() >= cursor.offset)
        .map(|cursor| cursor.offset)
        .unwrap_or(initial_offset);
    if !cursor_is_rotated
        && metadata.len().saturating_sub(offset) > GEARBLOCKS_RUNTIME_INCREMENTAL_READ_LIMIT_BYTES
    {
        offset = metadata
            .len()
            .saturating_sub(GEARBLOCKS_RUNTIME_INCREMENTAL_READ_LIMIT_BYTES);
    }

    if offset >= metadata.len() {
        let imported_count = recover_missed_gearblocks_runtime_exports_from_log(
            state, game_id, log_path, &metadata,
        )?;
        save_gearblocks_runtime_log_cursor(state, game_id, log_path, offset, &metadata)?;
        return Ok(imported_count);
    }

    let mut file = File::open(log_path)
        .map_err(|error| format!("Could not open GearBlocks runtime export log: {error}"))?;
    file.seek(SeekFrom::Start(offset))
        .map_err(|error| format!("Could not seek GearBlocks runtime export log: {error}"))?;
    let mut text = String::new();
    file.read_to_string(&mut text).map_err(|error| {
        format!("Could not read GearBlocks runtime export log additions: {error}")
    })?;

    let parse_result = parse_gearblocks_runtime_exports_from_text(log_path, &text);
    let part_aliases = parse_gearblocks_part_aliases_from_text(log_path, &text);
    let mut imported_count =
        persist_newer_gearblocks_runtime_exports(state, game_id, &parse_result.exports, false)?;
    imported_count += persist_gearblocks_part_aliases(state, game_id, &part_aliases)?;

    if parse_result.consumed_bytes > 0 {
        offset += parse_result.consumed_bytes as u64;
        save_gearblocks_runtime_log_cursor(state, game_id, log_path, offset, &metadata)?;
    } else if imported_count > 0 {
        save_gearblocks_runtime_log_cursor(state, game_id, log_path, metadata.len(), &metadata)?;
    }

    Ok(imported_count)
}

fn recover_missed_gearblocks_runtime_exports_from_log(
    state: &AppState,
    game_id: i64,
    log_path: &Path,
    metadata: &fs::Metadata,
) -> Result<usize, String> {
    let file_signature = format!("{}:{}", metadata.len(), file_modified_timestamp(metadata));
    let recovery_key = gearblocks_runtime_log_recovery_key(game_id, log_path);
    let already_checked = state
        .database
        .get_app_setting(&recovery_key)
        .map_err(|error| error.to_string())?
        .is_some_and(|value| value == file_signature);
    if already_checked {
        return Ok(0);
    }

    let imported_count = reconcile_gearblocks_runtime_exports_from_log(state, game_id, log_path)?;
    state
        .database
        .save_app_setting(&recovery_key, &file_signature)
        .map_err(|error| error.to_string())?;
    Ok(imported_count)
}

fn reconcile_gearblocks_runtime_exports_from_log(
    state: &AppState,
    game_id: i64,
    log_path: &Path,
) -> Result<usize, String> {
    let exports = parse_gearblocks_runtime_exports_from_log(log_path)?;
    let mut imported_count =
        persist_newer_gearblocks_runtime_exports(state, game_id, &exports, true)?;
    let part_aliases = parse_gearblocks_part_aliases_from_log(log_path)?;
    imported_count += persist_gearblocks_part_aliases(state, game_id, &part_aliases)?;
    Ok(imported_count)
}

fn persist_newer_gearblocks_runtime_exports(
    state: &AppState,
    game_id: i64,
    exports: &[GearBlocksRuntimeExportRecord],
    only_newer_than_latest: bool,
) -> Result<usize, String> {
    let indexed_at = unix_timestamp_label();
    let mut previous_export = state
        .database
        .latest_game_runtime_construction_export(game_id)
        .map_err(|error| error.to_string())?;
    let mut previous_parts = state
        .database
        .list_game_runtime_part_instances(game_id)
        .map_err(|error| error.to_string())?;
    let latest_exported_at = previous_export
        .as_ref()
        .map(|export| export.exported_at.clone())
        .unwrap_or_default();
    let mut imported_count = 0usize;

    for export in exports {
        if previous_export
            .as_ref()
            .is_some_and(|previous| previous.export_id == export.id)
        {
            continue;
        }
        let exported_at = export
            .document
            .get("exportedAt")
            .and_then(|value| value.as_str())
            .unwrap_or_default();
        if only_newer_than_latest
            && !latest_exported_at.is_empty()
            && exported_at < latest_exported_at.as_str()
        {
            continue;
        }

        let diff_summary = gearblocks_runtime_scene_diff_summary(&previous_parts, export)?;
        let persisted = persist_runtime_export(state, game_id, export, &indexed_at)?;
        import_runtime_export_parts(state, game_id, export)?;
        state
            .database
            .save_app_setting(&gearblocks_runtime_scene_diff_key(game_id), &diff_summary)
            .map_err(|error| error.to_string())?;
        previous_export = Some(persisted);
        previous_parts = state
            .database
            .list_game_runtime_part_instances(game_id)
            .map_err(|error| error.to_string())?;
        imported_count += 1;
    }

    Ok(imported_count)
}

fn persist_gearblocks_part_aliases(
    state: &AppState,
    game_id: i64,
    aliases: &[GearBlocksPartAliasLogRecord],
) -> Result<usize, String> {
    let mut imported_count = 0usize;
    for alias in aliases {
        let source_construction_id = alias
            .document
            .get("parentConstructionId")
            .map(json_value_to_string)
            .unwrap_or_default();
        let world_position_json = compact_json_or_empty_object(alias.document.get("position"))?;
        let local_position_json =
            compact_json_or_empty_object(alias.document.get("localPosition"))?;
        let current_unit_size_json =
            compact_json_or_empty_object(alias.document.get("currentUnitSize"))?;
        let payload_json =
            serde_json::to_string_pretty(&alias.document).map_err(|error| error.to_string())?;
        state
            .database
            .upsert_game_runtime_part_alias(GameRuntimePartAliasDraft {
                game_id,
                part_instance_key: &alias.part_instance_key,
                friendly_name: &alias.friendly_name,
                asset_guid: &json_string(alias.document.get("assetGuid")),
                asset_name: &json_string(alias.document.get("assetName")),
                display_name: &json_string(alias.document.get("displayName")),
                full_display_name: &json_string(alias.document.get("fullDisplayName")),
                category: &json_string(alias.document.get("category")),
                source_log_path: &alias.source_log_path,
                source_construction_id: &source_construction_id,
                world_position_json: &world_position_json,
                local_position_json: &local_position_json,
                current_unit_size_json: &current_unit_size_json,
                payload_json: &payload_json,
                last_seen_at: &alias.emitted_at,
            })
            .map_err(|error| error.to_string())?;
        imported_count += 1;
    }

    Ok(imported_count)
}

fn gearblocks_part_instance_key(part: &serde_json::Value) -> Option<String> {
    if let Some(key) = part.get("key").and_then(serde_json::Value::as_str) {
        if !key.is_empty() {
            return Some(key.to_string());
        }
    }
    let construction_id = part
        .get("parentConstructionId")
        .map(json_value_to_string)
        .filter(|value| !value.trim().is_empty());
    if let Some(id) = part.get("id").and_then(serde_json::Value::as_i64) {
        if let Some(construction_id) = construction_id.as_deref() {
            return Some(format!("construction:{construction_id}:id:{id}"));
        }
        return Some(format!("id:{id}"));
    }
    if let Some(index) = part.get("index").and_then(serde_json::Value::as_i64) {
        if let Some(construction_id) = construction_id.as_deref() {
            return Some(format!("construction:{construction_id}:idx:{index}"));
        }
        return Some(format!("idx:{index}"));
    }
    None
}

fn gearblocks_runtime_log_cursor_key(game_id: i64, log_path: &Path) -> String {
    let file_name = log_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("Player.log");
    format!("gearblocks.runtime_log_cursor.{game_id}.{file_name}")
}

fn gearblocks_runtime_log_recovery_key(game_id: i64, log_path: &Path) -> String {
    let file_name = log_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("Player.log");
    format!("gearblocks.runtime_log_recovery_checked.{game_id}.{file_name}")
}

fn save_gearblocks_runtime_log_cursor(
    state: &AppState,
    game_id: i64,
    log_path: &Path,
    offset: u64,
    metadata: &fs::Metadata,
) -> Result<(), String> {
    let cursor = GearBlocksRuntimeLogCursor {
        path: log_path.to_string_lossy().to_string(),
        offset,
        modified: file_modified_timestamp(metadata),
    };
    let cursor_json = serde_json::to_string(&cursor).map_err(|error| error.to_string())?;
    state
        .database
        .save_app_setting(
            &gearblocks_runtime_log_cursor_key(game_id, log_path),
            &cursor_json,
        )
        .map_err(|error| error.to_string())
}

fn file_modified_timestamp(metadata: &fs::Metadata) -> String {
    metadata
        .modified()
        .ok()
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map(|value| format!("{}.{}", value.as_secs(), value.subsec_nanos()))
        .unwrap_or_else(|| "unknown".to_string())
}

fn gearblocks_runtime_scene_diff_key(game_id: i64) -> String {
    format!("gearblocks.runtime_scene_diff.{game_id}")
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

fn gearblocks_part_label(part: &GearBlocksRuntimePart<'_>) -> String {
    match part.friendly_name.as_deref() {
        Some(alias) if !alias.trim().is_empty() => format!("{} (alias: {})", part.name, alias),
        _ => part.name.clone(),
    }
}

fn gearblocks_part_reference(part: &GearBlocksRuntimePart<'_>) -> String {
    if part.parent_construction_id.trim().is_empty() {
        format!("#{} idx {}", part.id, part.index)
    } else {
        format!(
            "construction {} / #{} idx {}",
            part.parent_construction_id, part.id, part.index
        )
    }
}

fn gearblocks_part_aliases_section(
    parts: &[GearBlocksRuntimePart<'_>],
    aliases: &[GameRuntimePartAliasRecord],
) -> String {
    if aliases.is_empty() {
        return "## Friendly Part Names\nNo friendly part names have been imported yet."
            .to_string();
    }

    let part_map = parts
        .iter()
        .map(|part| (part.instance_key.clone(), part))
        .collect::<HashMap<_, _>>();
    let mut lines = Vec::new();
    for alias in aliases.iter().take(120) {
        if let Some(part) = part_map.get(&alias.part_instance_key) {
            let world_position = part
                .world_position
                .and_then(json_vector3)
                .map(|(x, y, z)| format!("world=({x:.2},{y:.2},{z:.2})"))
                .unwrap_or_else(|| "world=unavailable".to_string());
            let local_position = part
                .local_position
                .and_then(json_vector3)
                .map(|(x, y, z)| format!("local=({x:.2},{y:.2},{z:.2})"))
                .unwrap_or_else(|| "local=unavailable".to_string());
            lines.push(format!(
                "- `{}` = #{} idx {} [{} / {}] {}; {} {}; instance={}",
                alias.friendly_name,
                part.id,
                part.index,
                part.system,
                part.category,
                part.name,
                world_position,
                local_position,
                alias.part_instance_key
            ));
        } else {
            let label = if alias.full_display_name.trim().is_empty() {
                alias.display_name.as_str()
            } else {
                alias.full_display_name.as_str()
            };
            lines.push(format!(
                "- `{}` = {} [{} / {}]; last seen {}; instance={} (not present in latest runtime export)",
                alias.friendly_name,
                label,
                alias.category,
                alias.asset_name,
                alias.last_seen_at,
                alias.part_instance_key
            ));
        }
    }

    if aliases.len() > lines.len() {
        lines.push(format!(
            "- {} additional friendly part name(s) omitted from prompt context for size.",
            aliases.len() - lines.len()
        ));
    }

    format!(
        "## Friendly Part Names\nUse these aliases when the user refers to exact physical parts. Do not apply an alias to all parts of the same catalog type unless the user asks for that.\n{}",
        lines.join("\n")
    )
}

fn gearblocks_marker_coordinate_reference_section(parts: &[GearBlocksRuntimePart<'_>]) -> String {
    let mut sorted_parts = parts.iter().collect::<Vec<_>>();
    sorted_parts.sort_by(|left, right| {
        left.index
            .cmp(&right.index)
            .then(left.name.cmp(&right.name))
            .then(left.category.cmp(&right.category))
    });

    let mut lines = Vec::new();
    for part in sorted_parts.iter().take(320) {
        let world_position = part
            .world_position
            .and_then(json_vector3)
            .map(|(x, y, z)| format!("world=({x:.2},{y:.2},{z:.2})"))
            .unwrap_or_else(|| "world=unavailable".to_string());
        let local_position = part
            .local_position
            .and_then(json_vector3)
            .map(|(x, y, z)| format!("local=({x:.2},{y:.2},{z:.2})"))
            .unwrap_or_else(|| "local=unavailable".to_string());
        let size = part
            .current_unit_size
            .and_then(json_vector3)
            .map(|(x, y, z)| format!(" size=({x:.2},{y:.2},{z:.2})"))
            .unwrap_or_default();
        lines.push(format!(
            "- {} [{} / {}] {}: {} {}{}",
            gearblocks_part_reference(part),
            part.system,
            part.category,
            gearblocks_part_label(part),
            world_position,
            local_position,
            size
        ));
    }

    if parts.len() > lines.len() {
        lines.push(format!(
            "- {} additional part coordinate row(s) omitted from prompt context for size.",
            parts.len() - lines.len()
        ));
    }

    if lines.is_empty() {
        "## Runtime Coordinate Reference\nNo runtime parts were available for coordinate reference."
            .to_string()
    } else {
        format!(
            "## Runtime Coordinate Reference\nCoordinates are GearBlocks units; 1 unit equals 10 cm. Use coordinates for spatial reasoning, measurements, and part identification. Do not request or emit Overlay Forge marker blocks; in-game visual markers are disabled for now.\n{}",
            lines.join("\n")
        )
    }
}

fn gearblocks_construction_groups_section(parts: &[GearBlocksRuntimePart<'_>]) -> String {
    let mut groups: HashMap<String, Vec<&GearBlocksRuntimePart<'_>>> = HashMap::new();
    for part in parts
        .iter()
        .filter(|part| !part.parent_construction_id.trim().is_empty())
    {
        groups
            .entry(part.parent_construction_id.clone())
            .or_default()
            .push(part);
    }

    if groups.is_empty() {
        return "## Runtime Construction Groups\nNo parent construction groups were exposed in the latest runtime export.".to_string();
    }

    let mut rows = groups.into_iter().collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        left.0
            .parse::<i64>()
            .ok()
            .cmp(&right.0.parse::<i64>().ok())
            .then(left.0.cmp(&right.0))
    });

    let mut lines = Vec::new();
    for (construction_id, mut group_parts) in rows.into_iter().take(120) {
        group_parts.sort_by(|left, right| {
            left.index
                .cmp(&right.index)
                .then(left.id.cmp(&right.id))
                .then(left.name.cmp(&right.name))
        });

        let mut inventory: HashMap<String, Vec<String>> = HashMap::new();
        for part in &group_parts {
            inventory
                .entry(part.name.clone())
                .or_default()
                .push(format!("#{} idx {}", part.id, part.index));
        }
        let mut inventory_rows = inventory.into_iter().collect::<Vec<_>>();
        inventory_rows
            .sort_by(|left, right| right.1.len().cmp(&left.1.len()).then(left.0.cmp(&right.0)));

        let inventory_text = inventory_rows
            .into_iter()
            .take(10)
            .map(|(name, refs)| {
                let total_count = refs.len();
                let visible_refs = refs.into_iter().take(24).collect::<Vec<_>>();
                let suffix = if total_count > visible_refs.len() {
                    format!(", ... {} more", total_count - visible_refs.len())
                } else {
                    String::new()
                };
                format!(
                    "{} x{} [{}{}]",
                    name,
                    total_count,
                    visible_refs.join(", "),
                    suffix
                )
            })
            .collect::<Vec<_>>()
            .join("; ");

        lines.push(format!(
            "- construction {}: {} part(s); {}",
            construction_id,
            group_parts.len(),
            inventory_text
        ));
    }

    format!(
        "## Runtime Construction Groups\nUse this section to reason about parts that are attached into the same GearBlocks construction. Part `#id` values can repeat across different parent constructions, so disambiguate with construction id, index, and coordinates before saying a part is missing.\n{}",
        lines.join("\n")
    )
}

fn gearblocks_build_guide_api_context_section() -> String {
    [
        "## Build Guide API Context",
        "Use these exported GearBlocks API surfaces when explaining build guides and current scene state:",
        "- `IPart`: part identity, category, mass, visibility/collision/selectability, world position, local position, orientation, and current unit size.",
        "- `IPartPaint` and `IPartProperties`: paint target colour, paintability, material name, strength, density, and material swap capability.",
        "- `IPartAttachments`, `IAttachment`, `ILinkNode`, and `ILink`: owned/associated attachments, attached parts, attachment type names, locked state, joint/interior flags, link-node type names, link availability, and connection positions.",
        "- `ITweakables`, `IResizable`, and `IControllablePartBehaviour`: configurable settings such as tweakable labels/values, resize step, current unit size, control bindings, activation state, direction/inversion options, and RPM/limit settings when the game exposes them.",
        "- `IEngineCrank`, `IEngineDrivenCrank`, `IEngineCylinder`, and `IEngineHead`: combustion-engine relationships, including crank, driven crank, crank shaft, linked cylinders, cylinder head, crank angle, timing angle, and current rotation speed.",
        "If a requested value is absent, say it was not exposed in the latest export rather than guessing.",
    ]
    .join("\n")
}

fn gearblocks_build_guide_runtime_details_section(parts: &[GearBlocksRuntimePart<'_>]) -> String {
    let mut sorted_parts = parts.iter().collect::<Vec<_>>();
    sorted_parts.sort_by(|left, right| {
        left.index
            .cmp(&right.index)
            .then(left.name.cmp(&right.name))
            .then(left.category.cmp(&right.category))
    });

    let mut lines = Vec::new();
    for part in sorted_parts {
        let mut details = Vec::new();
        if let Some(summary) = gearblocks_part_material_paint_summary(part.part_json) {
            details.push(summary);
        }
        if let Some(summary) = gearblocks_part_attachment_summary(part.part_json) {
            details.push(summary);
        }
        if let Some(summary) = gearblocks_part_link_node_summary(part.part_json) {
            details.push(summary);
        }
        if let Some(summary) = gearblocks_part_tweakable_summary(part.part_json) {
            details.push(summary);
        }
        if let Some(summary) = gearblocks_part_behaviour_detail_summary(part.part_json) {
            details.push(summary);
        }

        if details.is_empty() {
            continue;
        }

        lines.push(format!(
            "- #{} idx {} [{} / {}] {}: {}",
            part.id,
            part.index,
            part.system,
            part.category,
            gearblocks_part_label(part),
            details.join("; ")
        ));

        if lines.len() >= 180 {
            break;
        }
    }

    if parts.len() > lines.len() {
        let detailed_count = parts
            .iter()
            .filter(|part| {
                gearblocks_part_material_paint_summary(part.part_json).is_some()
                    || gearblocks_part_attachment_summary(part.part_json).is_some()
                    || gearblocks_part_link_node_summary(part.part_json).is_some()
                    || gearblocks_part_tweakable_summary(part.part_json).is_some()
                    || gearblocks_part_behaviour_detail_summary(part.part_json).is_some()
            })
            .count();
        if detailed_count > lines.len() {
            lines.push(format!(
                "- {} additional detailed part row(s) omitted from prompt context for size.",
                detailed_count - lines.len()
            ));
        }
    }

    if lines.is_empty() {
        "## Build Guide Runtime Details\nNo paint, attachment, link-node, tweakable, or engine detail values were exposed in the latest runtime export.".to_string()
    } else {
        format!(
            "## Build Guide Runtime Details\nThese details come from the latest full runtime export. Use them to keep build-guide advice grounded in current paint/materials, attachment types, configurable settings, and engine connections.\n{}",
            lines.join("\n")
        )
    }
}

fn gearblocks_part_material_paint_summary(part: &serde_json::Value) -> Option<String> {
    let mut items = Vec::new();
    if let Some(paint) = part.get("paint") {
        for key in ["targetColour", "color", "colour"] {
            let value = json_string(paint.get(key));
            if !value.is_empty() {
                items.push(format!("{key}={value}"));
            }
        }
    }
    if let Some(properties) = part.get("properties") {
        let material = json_string(properties.get("materialName"));
        if !material.is_empty() {
            items.push(format!("material={material}"));
        }
        for key in ["density", "strength", "mass"] {
            if let Some(value) = properties.get(key).and_then(serde_json::Value::as_f64) {
                items.push(format!("{key}={value:.2}"));
            }
        }
        for key in ["isPaintable", "isSwappable"] {
            if let Some(value) = properties.get(key).and_then(serde_json::Value::as_bool) {
                items.push(format!("{key}={value}"));
            }
        }
    }
    for key in ["isPaintable", "isMaterialSwappable"] {
        if let Some(value) = part.get(key).and_then(serde_json::Value::as_bool) {
            items.push(format!("{key}={value}"));
        }
    }
    (!items.is_empty()).then(|| format!("paint/material {}", items.join(", ")))
}

fn gearblocks_part_attachment_summary(part: &serde_json::Value) -> Option<String> {
    let attachments = part.get("attachments")?;
    let mut items = Vec::new();
    if let Some(count) = attachments
        .get("ownedCount")
        .and_then(serde_json::Value::as_i64)
    {
        items.push(format!("owned={count}"));
    }
    if let Some(count) = attachments
        .get("associatedCount")
        .and_then(serde_json::Value::as_i64)
    {
        items.push(format!("associated={count}"));
    }
    for group in ["owned", "associated"] {
        if let Some(values) = attachments.get(group).and_then(serde_json::Value::as_array) {
            let group_items = values
                .iter()
                .take(4)
                .filter_map(gearblocks_attachment_item_summary)
                .collect::<Vec<_>>();
            if !group_items.is_empty() {
                items.push(format!("{group}=[{}]", group_items.join(" | ")));
            }
        }
    }
    if let Some(attached_parts) = attachments
        .get("attachedParts")
        .and_then(serde_json::Value::as_array)
    {
        let refs = attached_parts
            .iter()
            .take(6)
            .map(gearblocks_runtime_ref_label)
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();
        if !refs.is_empty() {
            items.push(format!("attachedParts=[{}]", refs.join(", ")));
        }
    }
    (!items.is_empty()).then(|| format!("attachments {}", items.join(", ")))
}

fn gearblocks_attachment_item_summary(value: &serde_json::Value) -> Option<String> {
    let mut items = Vec::new();
    for key in ["typeName", "type"] {
        let text = json_string(value.get(key));
        if !text.is_empty() {
            items.push(format!("{key}={text}"));
        }
    }
    for key in ["isLocked", "isInterior", "isJointAttachment"] {
        if let Some(flag) = value.get(key).and_then(serde_json::Value::as_bool) {
            items.push(format!("{key}={flag}"));
        }
    }
    if let Some(owner) = value.get("ownerPart") {
        let label = gearblocks_runtime_ref_label(owner);
        if !label.is_empty() {
            items.push(format!("owner={label}"));
        }
    }
    if let Some(connected) = value.get("connectedPart") {
        let label = gearblocks_runtime_ref_label(connected);
        if !label.is_empty() {
            items.push(format!("connected={label}"));
        }
    }
    for key in ["ownerPosition", "connectedPosition"] {
        if let Some(position) = value.get(key).and_then(gearblocks_vector_label) {
            items.push(format!("{key}={position}"));
        }
    }
    (!items.is_empty()).then(|| items.join(" "))
}

fn gearblocks_part_link_node_summary(part: &serde_json::Value) -> Option<String> {
    let link_nodes = part.get("linkNodes")?.as_array()?;
    if link_nodes.is_empty() {
        return None;
    }
    let items = link_nodes
        .iter()
        .take(6)
        .map(|node| {
            let mut values = Vec::new();
            let type_name = json_string(node.get("typeName"));
            if !type_name.is_empty() {
                values.push(format!("type={type_name}"));
            }
            for key in [
                "hasLinks",
                "linkFromAvailable",
                "linkToAvailable",
                "isTypeHidden",
            ] {
                if let Some(flag) = node.get(key).and_then(serde_json::Value::as_bool) {
                    values.push(format!("{key}={flag}"));
                }
            }
            if let Some(position) = node.get("position").and_then(gearblocks_vector_label) {
                values.push(format!("world={position}"));
            }
            if let Some(position) = node.get("localPosition").and_then(gearblocks_vector_label) {
                values.push(format!("local={position}"));
            }
            values.join(" ")
        })
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    (!items.is_empty()).then(|| format!("linkNodes {}", items.join(" | ")))
}

fn gearblocks_part_tweakable_summary(part: &serde_json::Value) -> Option<String> {
    let tweakables = part.get("tweakables")?;
    let mut items = Vec::new();
    if let Some(count) = tweakables
        .get("numTweakables")
        .and_then(serde_json::Value::as_i64)
    {
        items.push(format!("count={count}"));
    }
    if let Some(values) = tweakables
        .get("tweakables")
        .and_then(serde_json::Value::as_array)
    {
        let visible = values
            .iter()
            .take(12)
            .map(gearblocks_compact_json_value)
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();
        if !visible.is_empty() {
            items.push(format!("values=[{}]", visible.join(", ")));
        }
    }
    if let Some(flag) = tweakables
        .get("syncTweakablesAvailable")
        .and_then(serde_json::Value::as_bool)
    {
        items.push(format!("syncAvailable={flag}"));
    }
    (!items.is_empty()).then(|| format!("tweakables {}", items.join(", ")))
}

fn gearblocks_part_behaviour_detail_summary(part: &serde_json::Value) -> Option<String> {
    let behaviours = part.get("behaviours")?.as_array()?;
    let items = behaviours
        .iter()
        .take(10)
        .map(gearblocks_behaviour_item_summary)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    (!items.is_empty()).then(|| format!("behaviours {}", items.join(" | ")))
}

fn gearblocks_behaviour_item_summary(behaviour: &serde_json::Value) -> String {
    let mut items = Vec::new();
    let name = json_string(behaviour.get("name"));
    if !name.is_empty() {
        items.push(name);
    }
    for key in [
        "isTweakable",
        "isActivatable",
        "isActivated",
        "isControllable",
        "isControlBound",
        "isControlOverridden",
        "onlyControlWhenPlayerLocked",
    ] {
        if let Some(flag) = behaviour.get(key).and_then(serde_json::Value::as_bool) {
            items.push(format!("{key}={flag}"));
        }
    }
    let control_info = json_string(behaviour.get("controlInfo"));
    if !control_info.is_empty() {
        items.push(format!("control={control_info}"));
    }
    for key in [
        "capacityRemaining",
        "capacityUsed",
        "numLinkedCylinders",
        "currentRotationSpeed",
        "crankAngle",
        "timingAngle",
    ] {
        if let Some(value) = behaviour.get(key) {
            let text = gearblocks_compact_json_value(value);
            if !text.is_empty() && text != "null" {
                items.push(format!("{key}={text}"));
            }
        }
    }
    for key in ["drivenCrank", "crankShaft", "crank", "head", "cylinder"] {
        if let Some(reference) = behaviour.get(key) {
            let label = gearblocks_runtime_ref_label(reference);
            if !label.is_empty() {
                items.push(format!("{key}={label}"));
            }
        }
    }
    if let Some(linked) = behaviour
        .get("linkedCylinders")
        .and_then(serde_json::Value::as_array)
    {
        let refs = linked
            .iter()
            .take(8)
            .map(gearblocks_runtime_ref_label)
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();
        if !refs.is_empty() {
            items.push(format!("linkedCylinders=[{}]", refs.join(", ")));
        }
    }
    items.join(" ")
}

fn gearblocks_runtime_ref_label(value: &serde_json::Value) -> String {
    if value.is_null() {
        return String::new();
    }
    let mut label = json_string(value.get("name"));
    if label.is_empty() {
        label = json_string(value.get("assetName"));
    }
    if label.is_empty() {
        label = json_string(value.get("type"));
    }
    if label.is_empty() {
        label = json_string(value.get("typeName"));
    }
    let id = value
        .get("id")
        .and_then(serde_json::Value::as_i64)
        .map(|id| format!("id={id}"));
    let index = value
        .get("index")
        .and_then(serde_json::Value::as_i64)
        .map(|index| format!("idx={index}"));
    match (label.is_empty(), id, index) {
        (true, None, None) => String::new(),
        (true, id, index) => [id, index]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join("/"),
        (false, None, None) => label,
        (false, id, index) => format!(
            "{} ({})",
            label,
            [id, index]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join("/")
        ),
    }
}

fn gearblocks_vector_label(value: &serde_json::Value) -> Option<String> {
    json_vector3(value).map(|(x, y, z)| format!("({x:.2},{y:.2},{z:.2})"))
}

fn gearblocks_compact_json_value(value: &serde_json::Value) -> String {
    let text = match value {
        serde_json::Value::String(text) => text.trim().to_string(),
        serde_json::Value::Number(_) | serde_json::Value::Bool(_) | serde_json::Value::Null => {
            value.to_string()
        }
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => value.to_string(),
    };
    if text.len() > 120 {
        let truncated = text.chars().take(120).collect::<String>();
        format!("{truncated}...")
    } else {
        text
    }
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
        let world_position = part
            .world_position
            .and_then(json_vector3)
            .map(|(x, y, z)| format!(" world=({x:.2},{y:.2},{z:.2})"))
            .unwrap_or_default();
        lines.push(format!(
            "- #{} idx {} [{} / {}] {}: {}; behaviours={}; links={}{}{}{}",
            part.id,
            part.index,
            part.system,
            part.category,
            gearblocks_part_label(part),
            part.purpose,
            behaviours,
            part.link_node_count,
            position,
            world_position,
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

fn compact_json_or_empty_object(value: Option<&serde_json::Value>) -> Result<String, String> {
    match value {
        Some(value) => serde_json::to_string(value).map_err(|error| error.to_string()),
        None => Ok("{}".to_string()),
    }
}

fn option_bool_label(value: Option<bool>) -> &'static str {
    match value {
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
        if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
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
