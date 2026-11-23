//! All app window creation and mutations are owned here. No renderer window permissions required.
use std::sync::Mutex;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

#[derive(Clone, Copy)]
pub enum WindowKind {
    Main,
    QuickPrompt,
}
impl WindowKind {
    fn label(self) -> &'static str {
        match self {
            Self::Main => "main",
            Self::QuickPrompt => "quick-prompt",
        }
    }
}
pub struct BaseWindowConfig {
    kind: WindowKind,
    title: &'static str,
    width: f64,
    height: f64,
}
pub struct OverlayWindowConfig {
    base: BaseWindowConfig,
}
pub struct StandaloneWindowConfig {
    base: BaseWindowConfig,
}
#[derive(Default)]
pub struct WindowManager {
    previous_foreground: Mutex<isize>,
}
impl WindowManager {
    pub fn preload(&self, app: &AppHandle) -> Result<(), String> {
        let main = StandaloneWindowConfig {
            base: BaseWindowConfig {
                kind: WindowKind::Main,
                title: "Overlay Forge",
                width: 1100.,
                height: 780.,
            },
        };
        let prompt = OverlayWindowConfig {
            base: BaseWindowConfig {
                kind: WindowKind::QuickPrompt,
                title: "Overlay Forge quick prompt",
                width: 620.,
                height: 600.,
            },
        };
        self.create(app, main.base, false)?;
        self.create(app, prompt.base, true)
    }
    fn create(
        &self,
        app: &AppHandle,
        config: BaseWindowConfig,
        overlay: bool,
    ) -> Result<(), String> {
        let path = if overlay {
            "index.html?surface=quick"
        } else {
            "index.html"
        };
        WebviewWindowBuilder::new(app, config.kind.label(), WebviewUrl::App(path.into()))
            .title(config.title)
            .inner_size(config.width, config.height)
            .min_inner_size(360., 320.)
            .visible(false)
            .focused(false)
            .skip_taskbar(overlay)
            .always_on_top(overlay)
            .build()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    pub fn show(&self, app: &AppHandle, kind: WindowKind) -> Result<(), String> {
        let window = app
            .get_webview_window(kind.label())
            .ok_or("Window unavailable.")?;
        if matches!(kind, WindowKind::QuickPrompt) {
            if window.is_visible().map_err(|e| e.to_string())? {
                return window
                    .unminimize()
                    .and_then(|_| window.set_focus())
                    .map_err(|e| e.to_string());
            }
            #[cfg(windows)]
            {
                *self
                    .previous_foreground
                    .lock()
                    .map_err(|_| "Focus state unavailable.")? =
                    unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow() }
                        as isize;
            }
            let cursor = app.cursor_position().map_err(|e| e.to_string())?;
            if let Some(monitor) = window
                .monitor_from_point(cursor.x, cursor.y)
                .map_err(|e| e.to_string())?
            {
                let area = monitor.work_area();
                let scale = monitor.scale_factor();
                let width = (620. * scale).min(area.size.width as f64) as u32;
                let height = (600. * scale).min(area.size.height as f64) as u32;
                window
                    .set_size(tauri::PhysicalSize::new(width, height))
                    .map_err(|e| e.to_string())?;
                window
                    .set_position(tauri::PhysicalPosition::new(
                        area.position.x + ((area.size.width - width) / 2) as i32,
                        area.position.y + ((area.size.height - height) / 2) as i32,
                    ))
                    .map_err(|e| e.to_string())?;
            }
        }
        window
            .show()
            .and_then(|_| window.unminimize())
            .and_then(|_| window.set_focus())
            .map_err(|e| e.to_string())
    }
    pub fn hide(&self, app: &AppHandle, kind: WindowKind) -> Result<(), String> {
        let window = app
            .get_webview_window(kind.label())
            .ok_or("Window unavailable.")?;
        let restore =
            matches!(kind, WindowKind::QuickPrompt) && window.is_focused().unwrap_or(false);
        window.hide().map_err(|e| e.to_string())?;
        if restore {
            #[cfg(windows)]
            {
                let previous = *self
                    .previous_foreground
                    .lock()
                    .map_err(|_| "Focus state unavailable.")?;
                let hwnd = previous as windows_sys::Win32::Foundation::HWND;
                if previous != 0
                    && unsafe { windows_sys::Win32::UI::WindowsAndMessaging::IsWindow(hwnd) } != 0
                    && unsafe {
                        windows_sys::Win32::UI::WindowsAndMessaging::SetForegroundWindow(hwnd)
                    } == 0
                {
                    return Err("Prompt hidden; Windows declined focus restoration.".into());
                }
            }
        }
        Ok(())
    }
    pub fn quit(&self, app: &AppHandle) -> Result<(), String> {
        if app.state::<crate::AppState>().chatgpt.is_busy() {
            app.state::<crate::AppState>().chatgpt.stop(app);
            return Err("Quit paused while ChatGPT connection work stops. Wait for its final status, then Quit again.".into());
        }
        let cleanup = app
            .state::<crate::AppState>()
            .hub
            .lock()
            .map_err(|_| "AI hub is unavailable; cleanup cannot be verified.")?
            .prepare_shutdown();
        use tauri::Emitter;
        let _ = app.emit("hub-changed", ());
        cleanup?;
        app.exit(0);
        Ok(())
    }
}
#[tauri::command]
pub fn show_main(app: AppHandle, state: tauri::State<'_, crate::AppState>) -> Result<(), String> {
    state.windows.show(&app, WindowKind::Main)
}
#[tauri::command]
pub fn show_quick_prompt(
    app: AppHandle,
    state: tauri::State<'_, crate::AppState>,
) -> Result<(), String> {
    state.windows.show(&app, WindowKind::QuickPrompt)
}
#[tauri::command]
pub fn hide_quick_prompt(
    app: AppHandle,
    state: tauri::State<'_, crate::AppState>,
) -> Result<(), String> {
    state.windows.hide(&app, WindowKind::QuickPrompt)
}
