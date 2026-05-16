mod commands;
mod db;
mod hotkeys;
mod openai;

use commands::{
    create_calendar_event, create_note, create_planning_conversation, create_project, create_task,
    delete_calendar_event, delete_note, delete_planning_conversation, delete_project, delete_task,
    get_milestone_status, get_scratchpad, list_calendar_events, list_notes,
    list_planning_conversations, list_planning_messages, list_projects, list_tasks,
    save_scratchpad, send_planning_message, shutdown_app, update_calendar_event, update_note,
    update_project, update_task,
};
use db::AppDatabase;
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

            app.manage(AppState { database });
            hotkeys::register_toggle_hotkey(app)?;

            if let Some(window) = app.get_webview_window("main") {
                window.set_always_on_top(true)?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_scratchpad,
            save_scratchpad,
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
            list_planning_conversations,
            create_planning_conversation,
            list_planning_messages,
            send_planning_message,
            delete_planning_conversation
        ])
        .run(tauri::generate_context!())
        .expect("error while running Overlay Forge");
}
