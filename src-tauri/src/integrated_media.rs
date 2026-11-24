//! Embedded product initialization and the host's native authorization boundary.
use crate::{profiles::ProfileService, AppState};
use std::path::Path;
use tauri::{
    ipc::{Invoke, InvokeBody},
    AppHandle, Manager, Wry,
};

pub fn initialize(app: &AppHandle, host_root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let base = host_root
        .parent()
        .ok_or("The host app-data directory has no parent.")?;
    // Preserve the standalone product's data location; never use the host database.
    let media_root = base.join("com.overlayforge.media");
    std::fs::create_dir_all(&media_root)?;
    app.manage(overlay_forge_media_lib::embedded_state(
        media_root.join("overlay-forge-media.sqlite3"),
    )?);
    Ok(())
}

pub fn handler(
    host: impl Fn(Invoke<Wry>) -> bool + Send + Sync + 'static,
) -> impl Fn(Invoke<Wry>) -> bool + Send + Sync + 'static {
    let media = overlay_forge_media_lib::command_handler();
    move |invoke| {
        let command = invoke.message.command();
        if !overlay_forge_media_lib::handles_command(command) {
            return host(invoke);
        }
        let webview = invoke.message.webview();
        if webview.label() != "main" {
            invoke
                .resolver
                .reject("Media is available in the main app only.");
            return true;
        }
        let session_id = match invoke.message.payload() {
            InvokeBody::Json(value) => value.get("sessionId").and_then(|value| value.as_str()),
            _ => None,
        };
        let Some(session_id) = session_id else {
            invoke.resolver.reject("Sign in before using Media.");
            return true;
        };
        if webview
            .try_state::<overlay_forge_media_lib::AppState>()
            .is_none()
        {
            invoke.resolver.reject(
                "Media data could not be opened. Check the Media app-data folder and restart.",
            );
            return true;
        }
        let state = webview.state::<AppState>();
        let authorization = if matches!(
            command,
            "preview_legacy_import" | "import_legacy_data" | "update_media_settings"
        ) {
            ProfileService::require_admin(&state, session_id)
        } else {
            ProfileService::require_signed_in(&state, Some(session_id))
        };
        match authorization {
            // Keep the session guard until the synchronous product command completes.
            Ok(_profile) => media(invoke),
            Err(error) => {
                invoke.resolver.reject(error);
                true
            }
        }
    }
}
