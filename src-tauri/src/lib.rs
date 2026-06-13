mod commands;
mod db;
mod github;
mod hotkeys;
mod openai;

use commands::{
    attach_planning_conversation_context, catalog_game_parts_from_screenshots,
    clear_openai_api_key, create_bridge_file_draft_from_conversation, create_calendar_event,
    create_game, create_game_chat_conversation, create_game_screenshot_capture_request,
    create_note, create_planning_conversation, create_project, create_task,
    create_youtube_reference, delete_bridge_file_draft, delete_calendar_event, delete_game,
    delete_game_chat_conversation, delete_game_screenshot, delete_note,
    delete_planning_conversation, delete_project, delete_project_github_repository,
    delete_project_markdown_context, delete_task, delete_youtube_reference,
    fetch_project_github_metadata, get_bridge_file_draft, get_milestone_status,
    get_openai_api_key_status, get_project_github_repository, get_project_markdown_context,
    get_scratchpad, get_youtube_reference, list_bridge_file_drafts, list_calendar_events,
    list_game_catalog_objects, list_game_chat_conversations, list_game_chat_messages,
    list_game_part_categories, list_game_screenshots, list_games, list_notes,
    list_planning_conversation_context, list_planning_conversations, list_planning_messages,
    list_projects, list_tasks, list_youtube_references, load_project_markdown_context,
    open_youtube_reference, preview_planning_chat_prompt, remove_planning_conversation_context,
    save_openai_api_key, save_project_github_repository, save_project_markdown_context,
    save_scratchpad, send_game_chat_message, send_planning_message, shutdown_app,
    update_calendar_event, update_note, update_project, update_task, update_youtube_reference,
};
use db::AppDatabase;
use tauri::webview::Color;
use tauri::Manager;

pub struct AppState {
    pub database: AppDatabase,
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            let database_path = app_data_dir.join("overlay-forge.sqlite3");
            let database = AppDatabase::new(database_path)?;
            let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let workspace_root = manifest_dir.parent().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not resolve Overlay Forge workspace root",
                )
            })?;
            let screenshots_dir = workspace_root.join("game-screenshots");
            std::fs::create_dir_all(&screenshots_dir)?;
            app.asset_protocol_scope()
                .allow_directory(&screenshots_dir, true)?;

            app.manage(AppState { database });
            hotkeys::register_toggle_hotkey(app)?;

            if let Some(window) = app.get_webview_window("main") {
                window.set_background_color(Some(Color(13, 18, 24, 255)))?;
                window.set_always_on_top(true)?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_scratchpad,
            save_scratchpad,
            get_openai_api_key_status,
            save_openai_api_key,
            clear_openai_api_key,
            get_milestone_status,
            shutdown_app,
            list_tasks,
            create_task,
            update_task,
            delete_task,
            list_notes,
            create_note,
            update_note,
            delete_note,
            list_calendar_events,
            create_calendar_event,
            update_calendar_event,
            delete_calendar_event,
            list_projects,
            create_project,
            update_project,
            delete_project,
            get_project_github_repository,
            save_project_github_repository,
            delete_project_github_repository,
            fetch_project_github_metadata,
            get_project_markdown_context,
            save_project_markdown_context,
            delete_project_markdown_context,
            load_project_markdown_context,
            list_planning_conversations,
            create_planning_conversation,
            list_planning_messages,
            send_planning_message,
            delete_planning_conversation,
            list_planning_conversation_context,
            attach_planning_conversation_context,
            remove_planning_conversation_context,
            preview_planning_chat_prompt,
            list_bridge_file_drafts,
            get_bridge_file_draft,
            create_bridge_file_draft_from_conversation,
            delete_bridge_file_draft,
            list_youtube_references,
            get_youtube_reference,
            create_youtube_reference,
            update_youtube_reference,
            delete_youtube_reference,
            open_youtube_reference,
            list_games,
            create_game,
            delete_game,
            list_game_catalog_objects,
            list_game_part_categories,
            catalog_game_parts_from_screenshots,
            list_game_screenshots,
            create_game_screenshot_capture_request,
            delete_game_screenshot,
            list_game_chat_conversations,
            create_game_chat_conversation,
            list_game_chat_messages,
            send_game_chat_message,
            delete_game_chat_conversation
        ])
        .run(tauri::generate_context!())
        .expect("error while running Overlay Forge");
}
