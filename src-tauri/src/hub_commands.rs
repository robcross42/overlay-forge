use crate::{
    ai_hub::{AiProvider, HubSnapshot, RequestAction},
    AppState,
};
use serde::Deserialize;
use tauri::{AppHandle, Emitter, State};

fn notify(app: &AppHandle) {
    let _ = app.emit("hub-changed", ());
}
#[tauri::command]
pub fn hub_stop_all(state: State<'_, AppState>, app: AppHandle, session_id: String) -> Result<(), String> {
    let _profile = crate::profiles::ProfileService::require_admin(&state, &session_id)?;
    state
        .hub
        .lock()
        .map_err(|_| "AI hub is unavailable; Stop could not be confirmed.")?
        .stop_all();
    notify(&app);
    state.chatgpt.stop(&app);
    Ok(())
}
#[tauri::command]
pub fn hub_snapshot(
    state: State<'_, AppState>,
    conversation_id: Option<String>,
    session_id: String,
) -> Result<HubSnapshot, String> {
    let _profile = crate::profiles::ProfileService::require_admin(&state, &session_id)?;
    let hub = state.hub.lock().map_err(|_| "AI hub is unavailable.")?;
    let connection = state.chatgpt.snapshot()?;
    let mut snapshot = hub.snapshot_for(
        connection.registration.as_deref(),
        conversation_id.as_deref(),
    );
    snapshot.chatgpt = Some(connection);
    Ok(snapshot)
}
#[tauri::command]
pub fn hub_draft(
    provider: AiProvider,
    text: String,
    state: State<'_, AppState>,
    app: AppHandle,
    session_id: String,
) -> Result<(), String> {
    let _profile = crate::profiles::ProfileService::require_admin(&state, &session_id)?;
    state
        .hub
        .lock()
        .map_err(|_| "AI hub is unavailable.")?
        .draft(provider, text)?;
    notify(&app);
    Ok(())
}
#[tauri::command]
pub fn hub_submit(
    provider: AiProvider,
    demo: bool,
    text: Option<String>,
    conversation_id: Option<String>,
    state: State<'_, AppState>,
    app: AppHandle,
    session_id: String,
) -> Result<Option<String>, String> {
    let _profile = crate::profiles::ProfileService::require_admin(&state, &session_id)?;
    if provider == AiProvider::Chatgpt && !demo {
        return crate::chatgpt::ChatgptService::start_chat(
            &app,
            text.as_deref().ok_or("Enter a prompt first.")?,
            conversation_id.as_deref(),
        )
        .map(Some);
    }
    state
        .hub
        .lock()
        .map_err(|_| "AI hub is unavailable.")?
        .submit(provider, demo)?;
    notify(&app);
    Ok(None)
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionInput {
    id: u64,
    sequence: u64,
    action: RequestAction,
}
#[tauri::command]
pub fn hub_action(
    input: ActionInput,
    state: State<'_, AppState>,
    app: AppHandle,
    session_id: String,
) -> Result<(), String> {
    let _profile = crate::profiles::ProfileService::require_admin(&state, &session_id)?;
    state
        .hub
        .lock()
        .map_err(|_| "AI hub is unavailable.")?
        .act(input.id, input.sequence, input.action)?;
    notify(&app);
    Ok(())
}
