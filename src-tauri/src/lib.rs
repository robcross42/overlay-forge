mod commands;
mod db;
mod gearblocks_api;
mod github;
mod hotkeys;
mod openai;

use std::sync::Mutex;

use commands::{
    attach_planning_conversation_context, catalog_game_parts_from_screenshots,
    clear_game_runtime_part_images_for_category, clear_openai_api_key,
    consume_pending_shortcut_action, create_bridge_file_draft_from_conversation,
    create_calendar_event, create_game, create_game_chat_conversation,
    create_game_chat_screenshot_capture, create_game_screenshot_capture_request, create_note,
    create_planning_conversation, create_project, create_task, create_youtube_reference,
    decode_gearblocks_construction_file, decode_gearblocks_construction_folder,
    delete_bridge_file_draft, delete_calendar_event, delete_game, delete_game_chat_conversation,
    delete_game_data_location, delete_game_screenshot, delete_note, delete_planning_conversation,
    delete_project, delete_project_github_repository, delete_project_markdown_context, delete_task,
    delete_youtube_reference, fetch_project_github_metadata, focus_game_chat_overlay_window,
    focus_last_game_window, get_active_game_chat_overlay, get_bridge_file_draft,
    get_milestone_status, get_openai_api_key_status, get_project_github_repository,
    get_project_markdown_context, get_scratchpad, get_youtube_reference,
    import_gearblocks_catalog_screenshot_images, import_gearblocks_runtime_part_index,
    install_gearblocks_lua_exporter, list_bridge_file_drafts, list_calendar_events,
    list_game_catalog_objects, list_game_chat_conversations, list_game_chat_messages,
    list_game_constructions, list_game_data_locations, list_game_part_categories,
    list_game_runtime_parts, list_game_screenshots, list_games, list_gearblocks_api_catalog,
    list_gearblocks_construction_files, list_gearblocks_runtime_exports, list_keybinds, list_notes,
    list_planning_conversation_context, list_planning_conversations, list_planning_messages,
    list_projects, list_tasks, list_youtube_references, load_project_markdown_context,
    open_game_chat_overlay_window, open_youtube_reference, preview_planning_chat_prompt,
    remove_planning_conversation_context, reset_keybinds, save_game_data_location, save_keybinds,
    save_openai_api_key, save_project_github_repository, save_project_markdown_context,
    save_scratchpad, send_game_chat_message, send_planning_message,
    set_game_runtime_part_display_image, set_overlay_window_opacity, shutdown_app,
    start_manual_overlay_drag, sync_gearblocks_runtime_context,
    sync_gearblocks_saved_constructions, update_calendar_event, update_game_runtime_part_notes,
    update_note, update_project, update_task, update_youtube_reference,
};
use db::AppDatabase;
use serde::Serialize;
use tauri::Manager;

#[derive(Clone, Serialize)]
pub struct GameChatOverlaySelection {
    #[serde(rename = "gameId")]
    pub game_id: i64,
    #[serde(rename = "conversationId")]
    pub conversation_id: i64,
}

pub struct AppState {
    pub database: AppDatabase,
    pub pending_shortcut_action: Mutex<Option<String>>,
    pub last_game_window: Mutex<Option<isize>>,
    pub active_game_chat_overlay: Mutex<Option<GameChatOverlaySelection>>,
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if let Err(error) = terminate_existing_overlay_forge_instances() {
                eprintln!("Overlay Forge process cleanup failed: {error}");
            }

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

            app.manage(AppState {
                database,
                pending_shortcut_action: Mutex::new(None),
                last_game_window: Mutex::new(None),
                active_game_chat_overlay: Mutex::new(None),
            });
            hotkeys::register_toggle_hotkey(app)?;

            if let Some(window) = app.get_webview_window("main") {
                window.set_always_on_top(true)?;
            }
            if let Some(window) = app.get_webview_window("game-chat") {
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
            consume_pending_shortcut_action,
            list_keybinds,
            save_keybinds,
            reset_keybinds,
            get_milestone_status,
            shutdown_app,
            start_manual_overlay_drag,
            set_overlay_window_opacity,
            focus_last_game_window,
            open_game_chat_overlay_window,
            focus_game_chat_overlay_window,
            get_active_game_chat_overlay,
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
            list_game_data_locations,
            save_game_data_location,
            delete_game_data_location,
            list_gearblocks_construction_files,
            list_game_constructions,
            sync_gearblocks_saved_constructions,
            sync_gearblocks_runtime_context,
            decode_gearblocks_construction_file,
            decode_gearblocks_construction_folder,
            install_gearblocks_lua_exporter,
            list_gearblocks_runtime_exports,
            list_gearblocks_api_catalog,
            import_gearblocks_runtime_part_index,
            import_gearblocks_catalog_screenshot_images,
            list_game_runtime_parts,
            set_game_runtime_part_display_image,
            clear_game_runtime_part_images_for_category,
            update_game_runtime_part_notes,
            list_game_catalog_objects,
            list_game_part_categories,
            catalog_game_parts_from_screenshots,
            list_game_screenshots,
            create_game_screenshot_capture_request,
            create_game_chat_screenshot_capture,
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

#[cfg(target_os = "windows")]
fn terminate_existing_overlay_forge_instances() -> Result<(), String> {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

    let current_pid = std::process::id();
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return Err("Could not create a process snapshot.".to_string());
    }

    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..unsafe { std::mem::zeroed() }
    };

    let mut has_entry = unsafe { Process32FirstW(snapshot, &mut entry) != 0 };
    while has_entry {
        let process_name = utf16_process_name(&entry.szExeFile);
        if entry.th32ProcessID != current_pid
            && process_name.eq_ignore_ascii_case("overlay-forge.exe")
        {
            terminate_process(entry.th32ProcessID)?;
        }

        has_entry = unsafe { Process32NextW(snapshot, &mut entry) != 0 };
    }

    unsafe {
        CloseHandle(snapshot);
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn terminate_existing_overlay_forge_instances() -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "windows")]
fn utf16_process_name(value: &[u16]) -> String {
    let len = value
        .iter()
        .position(|character| *character == 0)
        .unwrap_or(value.len());
    String::from_utf16_lossy(&value[..len])
}

#[cfg(target_os = "windows")]
fn terminate_process(pid: u32) -> Result<(), String> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{
        OpenProcess, TerminateProcess, WaitForSingleObject, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
    };

    let handle = unsafe { OpenProcess(PROCESS_TERMINATE | PROCESS_SYNCHRONIZE, 0, pid) };
    if handle.is_null() {
        return Err(format!("Could not open stale overlay-forge process {pid}."));
    }

    let terminate_result = unsafe { TerminateProcess(handle, 0) };
    if terminate_result == 0 {
        unsafe {
            CloseHandle(handle);
        }
        return Err(format!(
            "Could not terminate stale overlay-forge process {pid}."
        ));
    }

    let wait_result = unsafe { WaitForSingleObject(handle, 5000) };
    unsafe {
        CloseHandle(handle);
    }

    if wait_result == windows_sys::Win32::Foundation::WAIT_TIMEOUT {
        return Err(format!(
            "Timed out waiting for stale overlay-forge process {pid} to exit."
        ));
    }

    Ok(())
}
