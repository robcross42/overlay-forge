mod commands;
mod db;
use commands::{launch_product, list_products, shutdown_app};
use db::HostDatabase;
use tauri::Manager;

pub struct AppState {
    database: HostDatabase,
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let root = app.path().app_data_dir()?;
            std::fs::create_dir_all(&root)?;
            let state = AppState {
                database: HostDatabase::new(root.join("overlay-forge-host.sqlite3"))?,
            };
            let launched = commands::launch_requested_or_last(app.handle(), &state)
                .map_err(std::io::Error::other)?;
            app.manage(state);
            if !launched {
                commands::show_picker(app.handle()).map_err(std::io::Error::other)?
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_products,
            launch_product,
            shutdown_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running Overlay Forge host");
}
