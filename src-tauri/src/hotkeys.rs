use std::error::Error;

use tauri::{App, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

pub fn register_toggle_hotkey(app: &mut App) -> Result<(), Box<dyn Error>> {
    let toggle_shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Space);

    app.handle().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, shortcut, event| {
                if shortcut != &toggle_shortcut {
                    return;
                }

                if event.state() != ShortcutState::Pressed {
                    return;
                }

                if let Some(window) = app.get_webview_window("main") {
                    let is_visible = window.is_visible().unwrap_or(false);

                    if is_visible {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_always_on_top(true);
                        let _ = window.set_focus();
                    }
                }
            })
            .build(),
    )?;

    if let Err(error) = app.global_shortcut().register(toggle_shortcut) {
        eprintln!("Overlay Forge hotkey was not registered: {error}");
    }

    Ok(())
}
