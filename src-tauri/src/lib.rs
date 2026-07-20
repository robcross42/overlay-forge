mod commands;
mod db;
mod gearblocks_api;
mod gearblocks_api_scraper;
mod gearblocks_scene_context;
mod hotkeys;
mod media;
mod lifecycle;
mod openai;
mod repair_resell;
mod windows;

use std::sync::Mutex;

use commands::{
    catalog_game_parts_from_screenshots, clear_game_runtime_part_images_for_category,
    clear_gearblocks_markers, clear_openai_api_key, consume_pending_shortcut_action,
    create_calendar_event, create_game, create_game_build_guide_from_chat,
    create_game_character_build, create_game_chat_conversation,
    create_game_chat_screenshot_capture, create_game_screenshot_capture_request, create_note,
    create_task, create_youtube_reference,
    decode_gearblocks_construction_file, decode_gearblocks_construction_folder,
    delete_calendar_event, delete_game, delete_game_build_guide,
    delete_game_character_build, delete_game_chat_conversation, delete_game_data_location,
    delete_game_screenshot, delete_note, delete_smoking_event, delete_task,
    delete_youtube_reference, export_smoking_cessation_chatgpt_context,
    focus_game_chat_overlay_window, focus_last_game_window,
    get_active_game_build_guide_overlay, get_active_game_chat_overlay,
    get_app_status, get_game_build_guide, get_game_setting,
    get_gearblocks_third_party_dependency_status, get_openai_api_key_status,
    get_overlay_forge_foreground_window_label,
    get_scratchpad, get_smoking_cessation_settings, get_youtube_reference,
    import_game_build_guide_markdown,
    import_game_build_guide_url, import_gearblocks_catalog_screenshot_images,
    import_gearblocks_official_api_docs, import_gearblocks_runtime_context,
    import_gearblocks_runtime_part_index, install_gearblocks_lua_exporter,
    is_overlay_forge_foreground, list_calendar_events,
    list_game_build_guides, list_game_catalog_objects, list_game_character_builds,
    list_game_chat_conversations, list_game_chat_messages, list_game_constructions,
    list_game_data_locations, list_game_part_categories, list_game_runtime_part_api_members,
    list_game_runtime_part_instances, list_game_runtime_parts, list_game_screenshots, list_games,
    list_gearblocks_api_catalog, list_gearblocks_construction_files,
    list_gearblocks_part_render_profiles, list_gearblocks_rotation_snap_angles,
    list_gearblocks_runtime_exports, list_keybinds, list_notes,
    list_repair_resell_categories, list_repair_resell_deal_estimates,
    list_repair_resell_keyword_flags, list_repair_resell_listings,
    list_repair_resell_sources, list_repair_resell_travel_profiles, list_schedulers,
    list_smoking_events, list_tasks, list_youtube_references,
    manual_import_repair_resell_listing, open_game_build_guide_overlay_window,
    open_game_chat_overlay_window, open_youtube_reference,
    record_smoking_event, refresh_repair_resell_source,
    reset_keybinds, save_game_data_location, save_gearblocks_part_render_profile_from_capture,
    save_keybinds, save_openai_api_key,
    save_repair_resell_deal_estimate, set_repair_resell_listing_watchlist,
    save_scratchpad, send_game_chat_message, send_gearblocks_marker_commands,
    set_active_game_character_build,
    set_game_runtime_part_display_image, set_overlay_window_opacity, shutdown_app,
    start_manual_overlay_drag, start_scheduler_worker, sync_gearblocks_runtime_context,
    sync_gearblocks_saved_constructions, toggle_game_build_guide_overlay_window,
    toggle_game_chat_overlay_window, update_calendar_event, update_game_character_build,
    update_game_runtime_part_notes, update_note, update_repair_resell_source_enabled,
    update_smoking_cigarette_count, update_task,
    update_youtube_reference,
};
use db::AppDatabase;
use media::commands::{
    add_catalog_media_to_library, add_media_to_watch_next, create_manual_media_entry,
    create_media_streaming_link, create_media_tag, delete_media_library_entry,
    delete_media_streaming_link, delete_media_tag, get_media_library_detail, get_media_settings,
    list_media_library, list_media_streaming_links, list_media_tags, mark_episodes_watched_through,
    move_media_watch_next_item, open_media_streaming_target, refresh_media_metadata,
    remove_media_from_watch_next, search_media_catalog, set_episode_watched, set_media_entry_tags,
    set_movie_watched, set_preferred_media_streaming_link, set_season_watched, set_series_watched,
    update_media_library_entry, update_media_settings, update_media_streaming_link,
};
use serde::{Deserialize, Serialize};
use tauri::{Manager, RunEvent, WindowEvent};
use windows::{OverlayWindowConfig, StandaloneWindowConfig, WindowManager};

#[derive(Clone, Serialize)]
pub struct GameChatOverlaySelection {
    #[serde(rename = "gameId")]
    pub game_id: i64,
    #[serde(rename = "conversationId")]
    pub conversation_id: i64,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct GameBuildGuideOverlaySelection {
    #[serde(rename = "gameId")]
    pub game_id: i64,
    #[serde(rename = "guideId")]
    pub guide_id: i64,
}

pub struct AppState {
    pub database: AppDatabase,
    pub pending_shortcut_action: Mutex<Option<String>>,
    pub last_game_window: Mutex<Option<isize>>,
    pub active_game_chat_overlay: Mutex<Option<GameChatOverlaySelection>>,
    pub active_game_build_guide_overlay: Mutex<Option<GameBuildGuideOverlaySelection>>,
}

pub fn run() {
    lifecycle::reset_shutdown();

    let app = tauri::Builder::default()
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
                active_game_build_guide_overlay: Mutex::new(None),
            });
            hotkeys::register_toggle_hotkey(app)?;
            commands::start_gearblocks_runtime_import_monitor(app.handle().clone());
            start_scheduler_worker(app.handle().clone());

            let app_handle = app.handle().clone();
            let window_manager = WindowManager::new(&app_handle);
            let main_window_config = OverlayWindowConfig::main();
            window_manager
                .configure_runtime(main_window_config.base.kind)
                .map_err(std::io::Error::other)?;
            window_manager
                .register_focus_loss_behavior(main_window_config.base.kind)
                .map_err(std::io::Error::other)?;
            let game_chat_window_config = StandaloneWindowConfig::game_chat();
            if let Some(window) = window_manager.window(game_chat_window_config.base.kind) {
                window_manager
                    .configure_runtime(game_chat_window_config.base.kind)
                    .map_err(std::io::Error::other)?;
                window_manager
                    .set_minimum_size(game_chat_window_config)
                    .map_err(std::io::Error::other)?;
                let app_handle = app.handle().clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::Moved(position) = event {
                        let state = app_handle.state::<AppState>();
                        let selection = state
                            .active_game_chat_overlay
                            .lock()
                            .ok()
                            .and_then(|selection| selection.clone());
                        if let Some(selection) = selection {
                            if let Err(error) = state.database.update_game_chat_overlay_position(
                                selection.conversation_id,
                                position.x,
                                position.y,
                            ) {
                                eprintln!("Could not save game chat overlay position: {error}");
                            }
                        }
                    }
                });
            }
            let build_guide_window_config = StandaloneWindowConfig::game_build_guide();
            if let Some(window) = window_manager.window(build_guide_window_config.base.kind) {
                window_manager
                    .configure_runtime(build_guide_window_config.base.kind)
                    .map_err(std::io::Error::other)?;
                window_manager
                    .set_minimum_size(build_guide_window_config)
                    .map_err(std::io::Error::other)?;
                let app_handle = app.handle().clone();
                window.on_window_event(move |event| {
                    let state = app_handle.state::<AppState>();
                    let selection = state
                        .active_game_build_guide_overlay
                        .lock()
                        .ok()
                        .and_then(|selection| selection.clone());
                    let Some(selection) = selection else {
                        return;
                    };

                    match event {
                        WindowEvent::Moved(position) => {
                            if let Err(error) =
                                state.database.update_game_build_guide_overlay_bounds(
                                    selection.guide_id,
                                    Some(position.x),
                                    Some(position.y),
                                    None,
                                    None,
                                )
                            {
                                eprintln!("Could not save build guide overlay position: {error}");
                            }
                        }
                        WindowEvent::Resized(size) => {
                            if let Err(error) =
                                state.database.update_game_build_guide_overlay_bounds(
                                    selection.guide_id,
                                    None,
                                    None,
                                    Some(size.width as i32),
                                    Some(size.height as i32),
                                )
                            {
                                eprintln!("Could not save build guide overlay size: {error}");
                            }
                        }
                        _ => {}
                    }
                });
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
            get_app_status,
            list_schedulers,
            shutdown_app,
            start_manual_overlay_drag,
            set_overlay_window_opacity,
            focus_last_game_window,
            is_overlay_forge_foreground,
            get_overlay_forge_foreground_window_label,
            open_game_chat_overlay_window,
            focus_game_chat_overlay_window,
            toggle_game_chat_overlay_window,
            get_active_game_chat_overlay,
            get_game_setting,
            list_game_build_guides,
            create_game_build_guide_from_chat,
            import_game_build_guide_markdown,
            import_game_build_guide_url,
            get_game_build_guide,
            delete_game_build_guide,
            open_game_build_guide_overlay_window,
            toggle_game_build_guide_overlay_window,
            get_active_game_build_guide_overlay,
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
            list_youtube_references,
            get_youtube_reference,
            create_youtube_reference,
            update_youtube_reference,
            delete_youtube_reference,
            open_youtube_reference,
            search_media_catalog,
            add_catalog_media_to_library,
            create_manual_media_entry,
            list_media_library,
            get_media_library_detail,
            update_media_library_entry,
            delete_media_library_entry,
            refresh_media_metadata,
            set_movie_watched,
            set_episode_watched,
            set_season_watched,
            set_series_watched,
            mark_episodes_watched_through,
            add_media_to_watch_next,
            remove_media_from_watch_next,
            move_media_watch_next_item,
            list_media_tags,
            create_media_tag,
            delete_media_tag,
            set_media_entry_tags,
            list_media_streaming_links,
            create_media_streaming_link,
            update_media_streaming_link,
            delete_media_streaming_link,
            set_preferred_media_streaming_link,
            open_media_streaming_target,
            get_media_settings,
            update_media_settings,
            list_smoking_events,
            record_smoking_event,
            delete_smoking_event,
            get_smoking_cessation_settings,
            update_smoking_cigarette_count,
            export_smoking_cessation_chatgpt_context,
            list_repair_resell_sources,
            update_repair_resell_source_enabled,
            list_repair_resell_categories,
            list_repair_resell_keyword_flags,
            list_repair_resell_travel_profiles,
            list_repair_resell_listings,
            manual_import_repair_resell_listing,
            refresh_repair_resell_source,
            set_repair_resell_listing_watchlist,
            list_repair_resell_deal_estimates,
            save_repair_resell_deal_estimate,
            list_games,
            create_game,
            delete_game,
            list_game_character_builds,
            create_game_character_build,
            update_game_character_build,
            set_active_game_character_build,
            delete_game_character_build,
            list_game_data_locations,
            save_game_data_location,
            delete_game_data_location,
            list_gearblocks_construction_files,
            list_game_constructions,
            sync_gearblocks_saved_constructions,
            sync_gearblocks_runtime_context,
            import_gearblocks_runtime_context,
            send_gearblocks_marker_commands,
            clear_gearblocks_markers,
            decode_gearblocks_construction_file,
            decode_gearblocks_construction_folder,
            install_gearblocks_lua_exporter,
            get_gearblocks_third_party_dependency_status,
            list_gearblocks_runtime_exports,
            list_gearblocks_api_catalog,
            import_gearblocks_official_api_docs,
            import_gearblocks_runtime_part_index,
            import_gearblocks_catalog_screenshot_images,
            list_gearblocks_rotation_snap_angles,
            list_gearblocks_part_render_profiles,
            save_gearblocks_part_render_profile_from_capture,
            list_game_runtime_part_instances,
            list_game_runtime_parts,
            list_game_runtime_part_api_members,
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
        .build(tauri::generate_context!())
        .expect("error while building Overlay Forge");

    app.run(|_app_handle, event| match event {
        RunEvent::ExitRequested { .. } | RunEvent::Exit => lifecycle::request_shutdown(),
        _ => {}
    });
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
