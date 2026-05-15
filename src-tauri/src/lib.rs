mod commands;
mod db;
mod hotkeys;

use commands::{get_milestone_status, get_scratchpad, save_scratchpad};
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
                window.set_focus()?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_scratchpad,
            save_scratchpad,
            get_milestone_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running Overlay Forge");
}
