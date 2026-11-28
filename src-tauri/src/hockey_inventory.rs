use crate::{profiles::ProfileService, AppState};
use hockey_comc_inventory::{Inventory, InventoryPage, InventoryQuery};
use std::path::Path;
use tauri::{AppHandle, Manager, WebviewWindow};

pub fn initialize(app: &AppHandle, host_root: &Path) {
    if let Some(base) = host_root.parent() {
        let database = base
            .join("com.robcross42.hockeycardcompanion")
            .join("collection.sqlite3");
        app.manage(Inventory::new(database));
    }
}

#[tauri::command]
pub async fn hockey_inventory_query(
    window: WebviewWindow,
    session_id: String,
    query: InventoryQuery,
) -> Result<InventoryPage, String> {
    if window.label() != "main" {
        return Err("Hockey Cards is available in the main app only.".into());
    }
    let app = window.app_handle().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let _profile = ProfileService::require_admin(&state, &session_id)?;
        let inventory = app
            .try_state::<Inventory>()
            .ok_or("Saved COMC inventory is unavailable.")?;
        inventory.query(query).map_err(|error| error.to_string())
    })
    .await
    .map_err(|_| "Saved COMC inventory could not be read.".to_owned())?
}
