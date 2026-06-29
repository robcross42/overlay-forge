use std::collections::HashSet;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::windows::{WindowKind, WindowManager};
use crate::{commands, lifecycle, AppState};
use serde::{Deserialize, Serialize};
use tauri::{App, Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

const KEYBINDS_SETTING_KEY: &str = "keybinds_v1";
const TOGGLE_OVERLAY_ACTION: &str = "toggle_overlay";
const TOGGLE_OVERLAY_WAS_VISIBLE_ACTION: &str = "toggle_overlay_was_visible";
const TOGGLE_OVERLAY_WAS_HIDDEN_ACTION: &str = "toggle_overlay_was_hidden";
const GAME_CHAT_OVERLAY_ACTION: &str = "game_chat_overlay";
const GAME_CHAT_OVERLAY_WAS_HIDDEN_ACTION: &str = "game_chat_overlay_was_hidden";
const GAME_CHAT_SCREENSHOT_CAPTURE_ACTION: &str = "game_chat_region_capture";
const GAME_BUILD_GUIDE_OVERLAY_ACTION: &str = "game_build_guide_overlay";
const RECORD_SMOKING_EVENT_ACTION: &str = "record_smoking_event";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KeybindConfig {
    pub action: String,
    pub label: String,
    pub keys: Vec<String>,
}

pub fn register_toggle_hotkey(app: &mut App) -> Result<(), Box<dyn Error>> {
    let active_keyboard_shortcuts = Arc::new(Mutex::new(HashSet::<u32>::new()));
    app.handle().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler({
                let active_keyboard_shortcuts = active_keyboard_shortcuts.clone();
                move |app, shortcut, event| {
                    let shortcut_id = shortcut.id();
                    if event.state() != ShortcutState::Pressed {
                        if let Ok(mut active) = active_keyboard_shortcuts.lock() {
                            active.remove(&shortcut_id);
                        }
                        return;
                    }

                    if let Ok(mut active) = active_keyboard_shortcuts.lock() {
                        if !active.insert(shortcut_id) {
                            return;
                        }
                    }

                    if let Ok(action) = resolve_shortcut_action(app, shortcut) {
                        match action.as_str() {
                            TOGGLE_OVERLAY_ACTION => toggle_overlay_window(app),
                            GAME_CHAT_OVERLAY_ACTION => {
                                trigger_game_chat_overlay_shortcut(app);
                            }
                            GAME_CHAT_SCREENSHOT_CAPTURE_ACTION => {
                                WindowManager::new(app).remember_foreground_window_as_game();
                                set_pending_shortcut_action(
                                    app,
                                    GAME_CHAT_SCREENSHOT_CAPTURE_ACTION,
                                );
                                let _ = app.emit("game-chat-screenshot-capture-requested", ());
                            }
                            GAME_BUILD_GUIDE_OVERLAY_ACTION => {
                                trigger_game_build_guide_overlay_shortcut(app);
                            }
                            RECORD_SMOKING_EVENT_ACTION => record_smoking_event_from_shortcut(app),
                            _ => {}
                        }
                    }
                }
            })
            .build(),
    )?;

    register_configured_shortcuts(app.handle())?;
    start_mouse_shortcut_monitor(app.handle().clone());

    Ok(())
}

pub fn default_keybinds() -> Vec<KeybindConfig> {
    vec![
        KeybindConfig {
            action: TOGGLE_OVERLAY_ACTION.to_string(),
            label: "Toggle Overlay Forge".to_string(),
            keys: vec!["Ctrl".to_string(), "Shift".to_string(), "Space".to_string()],
        },
        KeybindConfig {
            action: GAME_CHAT_OVERLAY_ACTION.to_string(),
            label: "Open Gaming Chat Overlay".to_string(),
            keys: vec!["Ctrl".to_string(), "Shift".to_string(), "C".to_string()],
        },
        KeybindConfig {
            action: GAME_CHAT_SCREENSHOT_CAPTURE_ACTION.to_string(),
            label: "Capture Screenshot For Gaming Chat".to_string(),
            keys: Vec::new(),
        },
        KeybindConfig {
            action: GAME_BUILD_GUIDE_OVERLAY_ACTION.to_string(),
            label: "Toggle Gaming Build Guide Overlay".to_string(),
            keys: vec!["Ctrl".to_string(), "Shift".to_string(), "G".to_string()],
        },
        KeybindConfig {
            action: RECORD_SMOKING_EVENT_ACTION.to_string(),
            label: "Record Cigarette".to_string(),
            keys: Vec::new(),
        },
    ]
}

pub fn load_keybinds(app: &tauri::AppHandle) -> Result<Vec<KeybindConfig>, String> {
    let state = app.state::<AppState>();
    let stored = state
        .database
        .get_app_setting(KEYBINDS_SETTING_KEY)
        .map_err(|error| error.to_string())?;

    let keybinds = match stored {
        Some(value) if !value.trim().is_empty() => {
            serde_json::from_str::<Vec<KeybindConfig>>(&value)
                .unwrap_or_else(|_| default_keybinds())
        }
        _ => default_keybinds(),
    };

    validate_keybinds(&keybinds)?;
    Ok(merge_with_defaults(keybinds))
}

pub fn save_keybinds(
    app: &tauri::AppHandle,
    keybinds: Vec<KeybindConfig>,
) -> Result<Vec<KeybindConfig>, String> {
    let merged = normalize_keybinds(merge_with_defaults(keybinds))?;
    validate_keybinds(&merged)?;

    let payload = serde_json::to_string(&merged).map_err(|error| error.to_string())?;
    let state = app.state::<AppState>();
    state
        .database
        .save_app_setting(KEYBINDS_SETTING_KEY, &payload)
        .map_err(|error| error.to_string())?;

    register_configured_shortcuts(app).map_err(|error| error.to_string())?;
    Ok(merged)
}

pub fn reset_keybinds(app: &tauri::AppHandle) -> Result<Vec<KeybindConfig>, String> {
    let defaults = default_keybinds();
    let payload = serde_json::to_string(&defaults).map_err(|error| error.to_string())?;
    let state = app.state::<AppState>();
    state
        .database
        .save_app_setting(KEYBINDS_SETTING_KEY, &payload)
        .map_err(|error| error.to_string())?;

    register_configured_shortcuts(app).map_err(|error| error.to_string())?;
    Ok(defaults)
}

fn register_configured_shortcuts(app: &tauri::AppHandle) -> Result<(), Box<dyn Error>> {
    let _ = app.global_shortcut().unregister_all();
    let keybinds = load_keybinds(app).unwrap_or_else(|error| {
        eprintln!("Overlay Forge keybind settings were invalid, using defaults: {error}");
        default_keybinds()
    });

    for keybind in keybinds {
        let shortcut = shortcut_from_key_parts(&keybind.keys);
        if shortcut.trim().is_empty() {
            continue;
        }
        if is_mouse_shortcut_text(&shortcut) {
            continue;
        }

        if let Err(error) = app.global_shortcut().register(shortcut.as_str()) {
            eprintln!("Overlay Forge hotkey {shortcut} was not registered: {error}");
        }
    }

    Ok(())
}

fn resolve_shortcut_action(app: &tauri::AppHandle, shortcut: &Shortcut) -> Result<String, String> {
    let shortcut_id = shortcut.id();
    for keybind in load_keybinds(app)? {
        let configured_shortcut = shortcut_from_key_parts(&keybind.keys);
        if configured_shortcut.trim().is_empty() {
            continue;
        }
        if is_mouse_shortcut_text(&configured_shortcut) {
            continue;
        }

        let configured = parse_shortcut(&configured_shortcut)?;
        if configured.id() == shortcut_id {
            return Ok(keybind.action);
        }
    }

    Err("No action is mapped to this shortcut".to_string())
}

fn toggle_overlay_window(app: &tauri::AppHandle) {
    let was_visible = WindowManager::new(app)
        .window(WindowKind::Main)
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false);
    let action = if was_visible {
        TOGGLE_OVERLAY_WAS_VISIBLE_ACTION
    } else {
        TOGGLE_OVERLAY_WAS_HIDDEN_ACTION
    };

    set_pending_shortcut_action(app, action);
    if !was_visible {
        wake_overlay_window(app);
    }
    let _ = app.emit("overlay-toggle-requested", ());
}

fn set_pending_shortcut_action(app: &tauri::AppHandle, action: &str) {
    let state = app.state::<AppState>();
    let pending_result = state.pending_shortcut_action.lock();
    if let Ok(mut pending) = pending_result {
        *pending = Some(action.to_string());
    }
}

#[cfg(windows)]
fn start_mouse_shortcut_monitor(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        let mut cached_keybinds = Vec::new();
        let mut last_refresh = Instant::now() - Duration::from_secs(1);
        let mut active_shortcuts = HashSet::<String>::new();

        loop {
            if lifecycle::is_shutdown_requested() {
                break;
            }
            if last_refresh.elapsed() >= Duration::from_millis(500) {
                cached_keybinds = load_keybinds(&app).unwrap_or_default();
                last_refresh = Instant::now();
            }

            let mut currently_pressed = HashSet::<String>::new();
            for keybind in &cached_keybinds {
                let shortcut = shortcut_from_key_parts(&keybind.keys);
                if shortcut.trim().is_empty() || !is_mouse_shortcut_text(&shortcut) {
                    continue;
                }

                if is_mouse_shortcut_pressed(&shortcut) {
                    currently_pressed.insert(shortcut.to_lowercase());
                    if !active_shortcuts.contains(&shortcut.to_lowercase()) {
                        trigger_shortcut_action(&app, &keybind.action);
                    }
                }
            }

            active_shortcuts = currently_pressed;
            if lifecycle::sleep_until_shutdown(Duration::from_millis(25)) {
                break;
            }
        }
    });
}

#[cfg(not(windows))]
fn start_mouse_shortcut_monitor(_app: tauri::AppHandle) {}

fn trigger_shortcut_action(app: &tauri::AppHandle, action: &str) {
    match action {
        TOGGLE_OVERLAY_ACTION => toggle_overlay_window(app),
        GAME_CHAT_OVERLAY_ACTION => trigger_game_chat_overlay_shortcut(app),
        GAME_CHAT_SCREENSHOT_CAPTURE_ACTION => {
            WindowManager::new(app).remember_foreground_window_as_game();
            set_pending_shortcut_action(app, GAME_CHAT_SCREENSHOT_CAPTURE_ACTION);
            let _ = app.emit("game-chat-screenshot-capture-requested", ());
        }
        GAME_BUILD_GUIDE_OVERLAY_ACTION => trigger_game_build_guide_overlay_shortcut(app),
        RECORD_SMOKING_EVENT_ACTION => record_smoking_event_from_shortcut(app),
        _ => {}
    }
}

fn record_smoking_event_from_shortcut(app: &tauri::AppHandle) {
    let state = app.state::<AppState>();
    match state
        .database
        .create_smoking_event(None, "keybind", "Recorded from keybind")
    {
        Ok(record) => {
            if let Err(error) =
                commands::update_smoking_cessation_chatgpt_export(app, state.inner())
            {
                eprintln!("Could not update smoking cessation ChatGPT export: {error}");
            }
            let _ = app.emit("smoking-event-recorded", record);
        }
        Err(error) => {
            eprintln!("Could not record smoking event: {error}");
        }
    }
}

fn trigger_game_chat_overlay_shortcut(app: &tauri::AppHandle) {
    let window_manager = WindowManager::new(app);
    let chat_was_visible = window_manager
        .window(WindowKind::GameChat)
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false);
    let has_active_chat_context = app
        .state::<AppState>()
        .active_game_chat_overlay
        .lock()
        .map(|selection| selection.is_some())
        .unwrap_or(false);
    let chat_is_foreground = window_manager.is_foreground(WindowKind::GameChat);
    if !chat_is_foreground {
        window_manager.remember_foreground_window_as_game();
    }

    if has_active_chat_context {
        let result = if chat_was_visible && chat_is_foreground {
            commands::toggle_active_game_chat_overlay_window(app)
        } else {
            commands::show_active_game_chat_overlay_window(app)
        };
        if let Err(error) = result {
            eprintln!("Could not cycle game chat overlay from shortcut: {error}");
        }
        return;
    }

    let action = if chat_was_visible {
        GAME_CHAT_OVERLAY_ACTION
    } else {
        GAME_CHAT_OVERLAY_WAS_HIDDEN_ACTION
    };

    set_pending_shortcut_action(app, action);
    if !chat_was_visible {
        wake_overlay_window(app);
    }
    let _ = app.emit("game-chat-overlay-requested", ());
}

fn trigger_game_build_guide_overlay_shortcut(app: &tauri::AppHandle) {
    let window_manager = WindowManager::new(app);
    let build_guide_was_visible = window_manager
        .is_visible(WindowKind::GameBuildGuide)
        .unwrap_or(false);
    if !window_manager.is_foreground(WindowKind::GameBuildGuide) {
        window_manager.remember_foreground_window_as_game();
    }
    match commands::toggle_active_game_build_guide_overlay_window(app) {
        Ok(true) => {}
        Ok(false) if build_guide_was_visible => {}
        Ok(false) => {
            set_pending_shortcut_action(app, GAME_BUILD_GUIDE_OVERLAY_ACTION);
            wake_overlay_window(app);
            let _ = app.emit("game-build-guide-overlay-requested", ());
        }
        Err(error) => {
            eprintln!("Could not toggle game build guide overlay from shortcut: {error}");
            set_pending_shortcut_action(app, GAME_BUILD_GUIDE_OVERLAY_ACTION);
            wake_overlay_window(app);
            let _ = app.emit("game-build-guide-overlay-requested", ());
        }
    }
}

fn wake_overlay_window(app: &tauri::AppHandle) {
    let _ = WindowManager::new(app).show_and_focus(WindowKind::Main);
}

fn merge_with_defaults(keybinds: Vec<KeybindConfig>) -> Vec<KeybindConfig> {
    default_keybinds()
        .into_iter()
        .map(|default| {
            keybinds
                .iter()
                .find(|keybind| keybind.action == default.action)
                .map(|keybind| KeybindConfig {
                    action: default.action.clone(),
                    label: default.label.clone(),
                    keys: normalize_key_parts(&keybind.keys),
                })
                .unwrap_or(default)
        })
        .collect()
}

fn normalize_keybinds(keybinds: Vec<KeybindConfig>) -> Result<Vec<KeybindConfig>, String> {
    keybinds
        .into_iter()
        .map(|keybind| {
            Ok(KeybindConfig {
                action: keybind.action,
                label: keybind.label,
                keys: normalize_key_parts(&keybind.keys),
            })
        })
        .collect()
}

fn validate_keybinds(keybinds: &[KeybindConfig]) -> Result<(), String> {
    let mut seen = std::collections::HashSet::new();

    for keybind in keybinds {
        if ![
            TOGGLE_OVERLAY_ACTION,
            GAME_CHAT_OVERLAY_ACTION,
            GAME_CHAT_SCREENSHOT_CAPTURE_ACTION,
            GAME_BUILD_GUIDE_OVERLAY_ACTION,
            RECORD_SMOKING_EVENT_ACTION,
        ]
        .contains(&keybind.action.as_str())
        {
            return Err(format!("Unknown keybind function: {}", keybind.action));
        }

        if keybind.keys.len() > 3 {
            return Err(format!("{} can have at most three keybinds", keybind.label));
        }

        let shortcut = shortcut_from_key_parts(&keybind.keys);
        if shortcut.trim().is_empty() {
            continue;
        }

        let normalized = normalize_shortcut_text(&shortcut)?;
        validate_shortcut_policy(&normalized)?;

        if !seen.insert(normalized.to_lowercase()) {
            return Err(format!("Duplicate keybind is not allowed: {normalized}"));
        }
    }

    Ok(())
}

fn normalize_key_parts(keys: &[String]) -> Vec<String> {
    let mut parts = if keys.len() == 1 && keys[0].contains('+') {
        keys[0]
            .split('+')
            .map(normalize_key_part)
            .filter(|key| !key.is_empty())
            .collect::<Vec<_>>()
    } else {
        keys.iter()
            .take(3)
            .map(|key| normalize_key_part(key))
            .filter(|key| !key.is_empty())
            .collect::<Vec<_>>()
    };

    parts.truncate(3);
    parts
}

fn normalize_key_part(key: &str) -> String {
    match key.trim().to_lowercase().as_str() {
        "control" | "ctrl" => "Ctrl".to_string(),
        "shift" => "Shift".to_string(),
        "option" | "alt" => "Alt".to_string(),
        "space" => "Space".to_string(),
        "esc" | "escape" => "Escape".to_string(),
        "mouseleft" | "mouse1" | "leftmouse" | "leftclick" => "MouseLeft".to_string(),
        "mouseright" | "mouse2" | "rightmouse" | "rightclick" => "MouseRight".to_string(),
        "mousemiddle" | "mouse3" | "middlemouse" | "middleclick" => "MouseMiddle".to_string(),
        "mouse4" | "xbutton1" | "backmouse" => "Mouse4".to_string(),
        "mouse5" | "xbutton2" | "forwardmouse" => "Mouse5".to_string(),
        value if value.len() == 1 => value.to_uppercase(),
        value => value.to_string(),
    }
}

fn shortcut_from_key_parts(keys: &[String]) -> String {
    normalize_key_parts(keys).join("+")
}

fn normalize_shortcut_text(value: &str) -> Result<String, String> {
    if is_mouse_shortcut_text(value) {
        return normalize_mouse_shortcut_text(value);
    }

    let parsed = parse_shortcut(value)?;
    let mut parts = Vec::new();
    if parsed.mods.contains(Modifiers::CONTROL) {
        parts.push("Ctrl".to_string());
    }
    if parsed.mods.contains(Modifiers::SHIFT) {
        parts.push("Shift".to_string());
    }
    if parsed.mods.contains(Modifiers::ALT) {
        parts.push("Alt".to_string());
    }
    if parsed.mods.contains(Modifiers::SUPER) {
        parts.push("Super".to_string());
    }
    parts.push(display_code(parsed.key));
    Ok(parts.join("+"))
}

fn parse_shortcut(value: &str) -> Result<Shortcut, String> {
    value
        .parse::<Shortcut>()
        .map_err(|error| format!("Invalid keybind \"{value}\": {error}"))
}

fn validate_shortcut_policy(value: &str) -> Result<(), String> {
    if is_mouse_shortcut_text(value) {
        return validate_mouse_shortcut_policy(value);
    }

    let parsed = parse_shortcut(value)?;
    let has_modifier = !parsed.mods.is_empty();
    let modifier_count = [
        parsed.mods.contains(Modifiers::CONTROL),
        parsed.mods.contains(Modifiers::SHIFT),
        parsed.mods.contains(Modifiers::ALT),
        parsed.mods.contains(Modifiers::SUPER),
    ]
    .into_iter()
    .filter(|enabled| *enabled)
    .count();

    if modifier_count > 2 {
        return Err("Use at most two modifier keys plus one normal key".to_string());
    }

    if matches!(
        parsed.key,
        Code::Escape | Code::Tab | Code::CapsLock | Code::PrintScreen
    ) {
        return Err(format!(
            "{} is reserved and cannot be mapped",
            display_code(parsed.key)
        ));
    }

    if parsed.mods.contains(Modifiers::SUPER) {
        return Err(
            "Windows/Super key shortcuts are reserved for the operating system".to_string(),
        );
    }

    if parsed.mods.contains(Modifiers::ALT) && parsed.key == Code::F4 {
        return Err("Alt+F4 is reserved for closing windows".to_string());
    }

    if !has_modifier && is_printable_key(parsed.key) {
        return Err("Single printable keys are not allowed as global shortcuts".to_string());
    }

    if parsed.mods == Modifiers::CONTROL && is_common_editing_key(parsed.key) {
        return Err(format!(
            "Ctrl+{} is reserved for common app editing commands",
            display_code(parsed.key)
        ));
    }

    Ok(())
}

fn normalize_mouse_shortcut_text(value: &str) -> Result<String, String> {
    let parts = value
        .split('+')
        .map(normalize_key_part)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    let (modifiers, mouse_key) = split_mouse_shortcut_parts(&parts)?;

    let mut normalized = modifiers;
    normalized.push(mouse_key);
    Ok(normalized.join("+"))
}

fn validate_mouse_shortcut_policy(value: &str) -> Result<(), String> {
    let parts = value
        .split('+')
        .map(normalize_key_part)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    let (modifiers, mouse_key) = split_mouse_shortcut_parts(&parts)?;
    let mut unique_modifiers = HashSet::new();

    if modifiers.len() > 2 {
        return Err("Use at most two modifier keys plus one normal key".to_string());
    }

    for modifier in &modifiers {
        if !is_supported_modifier(modifier) {
            return Err(format!(
                "{modifier} is not supported as a mouse shortcut modifier"
            ));
        }
        if !unique_modifiers.insert(modifier.to_lowercase()) {
            return Err(format!("Duplicate keybind part is not allowed: {modifier}"));
        }
    }

    if modifiers.is_empty() && matches!(mouse_key.as_str(), "MouseLeft" | "MouseRight") {
        return Err(format!(
            "{mouse_key} requires Ctrl, Shift, or Alt so normal clicks are not intercepted"
        ));
    }

    Ok(())
}

fn split_mouse_shortcut_parts(parts: &[String]) -> Result<(Vec<String>, String), String> {
    if parts.is_empty() {
        return Err("Mouse shortcut cannot be empty".to_string());
    }

    let mouse_parts = parts
        .iter()
        .filter(|part| is_mouse_key(part))
        .collect::<Vec<_>>();
    if mouse_parts.len() != 1 {
        return Err("Mouse shortcuts must include exactly one mouse button".to_string());
    }

    let mouse_key = parts
        .last()
        .filter(|part| is_mouse_key(part))
        .cloned()
        .ok_or_else(|| "Mouse button must be the final keybind part".to_string())?;
    let modifiers = parts[..parts.len() - 1].to_vec();
    Ok((modifiers, mouse_key))
}

fn is_supported_modifier(value: &str) -> bool {
    matches!(value, "Ctrl" | "Shift" | "Alt")
}

fn is_mouse_shortcut_text(value: &str) -> bool {
    value
        .split('+')
        .map(normalize_key_part)
        .any(|part| is_mouse_key(&part))
}

fn is_mouse_key(value: &str) -> bool {
    matches!(
        value,
        "MouseLeft" | "MouseRight" | "MouseMiddle" | "Mouse4" | "Mouse5"
    )
}

#[cfg(windows)]
fn is_mouse_shortcut_pressed(value: &str) -> bool {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, VK_CONTROL, VK_LBUTTON, VK_MBUTTON, VK_MENU, VK_RBUTTON, VK_SHIFT,
        VK_XBUTTON1, VK_XBUTTON2,
    };

    let parts = value
        .split('+')
        .map(normalize_key_part)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    let Ok((modifiers, mouse_key)) = split_mouse_shortcut_parts(&parts) else {
        return false;
    };

    let is_pressed = |vkey: i32| unsafe { (GetAsyncKeyState(vkey) & 0x8000u16 as i16) != 0 };

    for modifier in modifiers {
        let modifier_pressed = match modifier.as_str() {
            "Ctrl" => is_pressed(VK_CONTROL as i32),
            "Shift" => is_pressed(VK_SHIFT as i32),
            "Alt" => is_pressed(VK_MENU as i32),
            _ => false,
        };
        if !modifier_pressed {
            return false;
        }
    }

    let mouse_vkey = match mouse_key.as_str() {
        "MouseLeft" => VK_LBUTTON,
        "MouseRight" => VK_RBUTTON,
        "MouseMiddle" => VK_MBUTTON,
        "Mouse4" => VK_XBUTTON1,
        "Mouse5" => VK_XBUTTON2,
        _ => return false,
    };

    is_pressed(mouse_vkey as i32)
}

fn is_printable_key(code: Code) -> bool {
    matches!(
        code,
        Code::KeyA
            | Code::KeyB
            | Code::KeyC
            | Code::KeyD
            | Code::KeyE
            | Code::KeyF
            | Code::KeyG
            | Code::KeyH
            | Code::KeyI
            | Code::KeyJ
            | Code::KeyK
            | Code::KeyL
            | Code::KeyM
            | Code::KeyN
            | Code::KeyO
            | Code::KeyP
            | Code::KeyQ
            | Code::KeyR
            | Code::KeyS
            | Code::KeyT
            | Code::KeyU
            | Code::KeyV
            | Code::KeyW
            | Code::KeyX
            | Code::KeyY
            | Code::KeyZ
            | Code::Digit0
            | Code::Digit1
            | Code::Digit2
            | Code::Digit3
            | Code::Digit4
            | Code::Digit5
            | Code::Digit6
            | Code::Digit7
            | Code::Digit8
            | Code::Digit9
            | Code::Space
            | Code::Minus
            | Code::Equal
            | Code::BracketLeft
            | Code::BracketRight
            | Code::Backslash
            | Code::Semicolon
            | Code::Quote
            | Code::Comma
            | Code::Period
            | Code::Slash
            | Code::Backquote
    )
}

fn is_common_editing_key(code: Code) -> bool {
    matches!(
        code,
        Code::KeyA
            | Code::KeyC
            | Code::KeyF
            | Code::KeyN
            | Code::KeyP
            | Code::KeyR
            | Code::KeyS
            | Code::KeyT
            | Code::KeyV
            | Code::KeyW
            | Code::KeyX
            | Code::KeyY
            | Code::KeyZ
    )
}

fn display_code(code: Code) -> String {
    let raw = code.to_string();
    raw.strip_prefix("Key")
        .or_else(|| raw.strip_prefix("Digit"))
        .unwrap_or(&raw)
        .to_string()
}
