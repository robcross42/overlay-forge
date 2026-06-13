use crate::db::{
    BridgeFileDraftRecord, CalendarEventRecord, GameCatalogObjectRecord,
    GameChatConversationRecord, GameChatMessageRecord, GameRecord,
    GameScreenshotCaptureRequestRecord, NoteRecord, PlanningConversationContextRecord,
    PlanningConversationRecord, PlanningMessageRecord, PlanningPromptPreviewRecord,
    ProjectGitHubRepositoryRecord, ProjectMarkdownContextPayload, ProjectMarkdownContextRecord,
    ProjectRecord, TaskRecord, YouTubeReferenceRecord,
};
use crate::github;
use crate::hotkeys;
use crate::openai;
use crate::AppState;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager, PhysicalPosition, State};

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
pub fn start_manual_overlay_drag(app: AppHandle) -> Result<(), String> {
    manual_overlay_drag(app)
}

#[tauri::command]
pub fn set_overlay_window_opacity(app: AppHandle, opacity: f64) -> Result<(), String> {
    set_overlay_opacity(app, opacity)
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
        .list_game_catalog_objects(game.id)
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
    state: State<'_, AppState>,
) -> Result<GameScreenshotCaptureRequestRecord, String> {
    create_game_screenshot_capture(
        game_id,
        timestamp_label,
        app,
        state,
        "visible-game-display",
        "windows-gdi-bitblt-foreground-window",
        "hide Overlay Forge before capture, then restore it",
        "captured_windows_gdi",
        "Captured through Windows GDI BitBlt from the foreground window after hiding Overlay Forge. Alpha was forced to 255 before PNG encoding.",
        true,
        capture_foreground_window_to_png,
    )
}

#[tauri::command]
pub fn create_game_chat_screenshot_capture(
    game_id: i64,
    timestamp_label: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<GameScreenshotCaptureRequestRecord, String> {
    create_game_screenshot_capture(
        game_id,
        timestamp_label,
        app,
        state,
        "visible-game-display",
        "windows-gdi-bitblt-foreground-window",
        "hide Overlay Forge before capture and leave focus with the game",
        "captured_windows_gdi_chat",
        "Captured through Windows GDI BitBlt from the foreground window after hiding Overlay Forge. Alpha was forced to 255 before PNG encoding and the screenshot was attached to the current Gaming chat prompt.",
        false,
        capture_foreground_window_to_png,
    )
}

fn create_game_screenshot_capture(
    game_id: i64,
    timestamp_label: String,
    app: AppHandle,
    state: State<'_, AppState>,
    capture_scope: &str,
    method_source: &str,
    overlay_handling: &str,
    capture_status: &str,
    notes: &str,
    restore_overlay: bool,
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

    let window = app.get_webview_window("main");
    if let Some(window) = window.as_ref() {
        window.hide().map_err(|error| error.to_string())?;
    }
    thread::sleep(Duration::from_millis(350));

    let capture_result = capture(&target_file_path);

    if restore_overlay || capture_result.is_err() {
        if let Some(window) = window.as_ref() {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }

    capture_result?;

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
    let custom_context = game_custom_prompt_context(&game)?;
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

fn game_custom_prompt_context(game: &GameRecord) -> Result<String, String> {
    match game.slug.as_str() {
        "gearblocks" => gearblocks_parts_catalog_prompt_context(),
        _ => Ok(String::new()),
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

#[cfg(target_os = "windows")]
fn manual_overlay_drag(app: AppHandle) -> Result<(), String> {
    use std::mem;
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
    use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;

    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "Overlay window was not found.".to_string())?;
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
        thread::sleep(Duration::from_millis(8));
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn manual_overlay_drag(_app: AppHandle) -> Result<(), String> {
    Err("Manual no-snap overlay drag is only available on Windows.".to_string())
}

#[cfg(target_os = "windows")]
fn set_overlay_opacity(app: AppHandle, opacity: f64) -> Result<(), String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetLayeredWindowAttributes, SetWindowLongPtrW, GWL_EXSTYLE,
        LWA_ALPHA, WS_EX_LAYERED,
    };

    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "Overlay window was not found.".to_string())?;
    let hwnd = window.hwnd().map_err(|error| error.to_string())?.0
        as windows_sys::Win32::Foundation::HWND;
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
fn set_overlay_opacity(_app: AppHandle, _opacity: f64) -> Result<(), String> {
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

fn path_text(path: &std::path::Path) -> String {
    path.to_string_lossy().to_string()
}

struct GearBlocksPartSeed {
    name: &'static str,
    category: &'static str,
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
            "Engine Inlet",
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
            "Joystick Display",
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
            "Coil-over (Barrel) Large",
            "Coil-over (Piston) Large",
            "Coil-over (Barrel) Medium",
            "Coil-over (Piston) Medium",
            "Coil-over (Barrel) Small",
            "Coil-over (Piston) Small",
            "Coil-over (Barrel) Small Strong",
            "Coil-over (Piston) Small Strong",
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
        seeds.push(GearBlocksPartSeed { name, category });
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

    write_rgba_png(output_path, crop.width, crop.height, &rgba)
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
