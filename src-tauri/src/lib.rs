mod commands;
mod db;
mod hotkeys;

use commands::{
    create_calendar_event, create_note, create_task, delete_calendar_event, delete_note,
    delete_task, get_milestone_status, get_scratchpad, list_calendar_events, list_notes,
    list_tasks, save_scratchpad, shutdown_app, update_calendar_event, update_note, update_task,
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
            delete_calendar_event
        ])
        .run(tauri::generate_context!())
        .expect("error while running Overlay Forge");
}
