use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewWindow};

use crate::AppState;

static MANUAL_OVERLAY_DRAG_ACTIVE: AtomicBool = AtomicBool::new(false);

pub const ACTIVE_WINDOW_OPACITY: f64 = 1.0;
pub const MAIN_CAPTURE_RESTORE_OPACITY: f64 = 0.78;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WindowKind {
    Main,
    GameChat,
    GameBuildGuide,
}

impl WindowKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Main => "main",
            Self::GameChat => "game-chat",
            Self::GameBuildGuide => "game-build-guide",
        }
    }

    pub const fn is_standalone(self) -> bool {
        matches!(self, Self::GameChat | Self::GameBuildGuide)
    }

    pub const fn runtime_config(self) -> BaseWindowConfig {
        BaseWindowConfig {
            kind: self,
            always_on_top: self.is_standalone(),
            hide_on_focus_loss: matches!(self, Self::Main),
            restore_opacity: if self.is_standalone() {
                ACTIVE_WINDOW_OPACITY
            } else {
                MAIN_CAPTURE_RESTORE_OPACITY
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BaseWindowConfig {
    pub kind: WindowKind,
    pub always_on_top: bool,
    pub hide_on_focus_loss: bool,
    pub restore_opacity: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct OverlayWindowConfig {
    pub base: BaseWindowConfig,
}

#[derive(Clone, Copy, Debug)]
pub struct StandaloneWindowConfig {
    pub base: BaseWindowConfig,
    pub min_width: u32,
    pub min_height: u32,
}

impl OverlayWindowConfig {
    pub const fn main() -> Self {
        Self {
            base: WindowKind::Main.runtime_config(),
        }
    }
}

impl StandaloneWindowConfig {
    pub const fn game_chat() -> Self {
        Self {
            base: WindowKind::GameChat.runtime_config(),
            min_width: 320,
            min_height: 210,
        }
    }

    pub const fn game_build_guide() -> Self {
        Self {
            base: WindowKind::GameBuildGuide.runtime_config(),
            min_width: 700,
            min_height: 520,
        }
    }
}

pub const CAPTURE_WINDOW_KINDS: [WindowKind; 3] = [
    WindowKind::Main,
    WindowKind::GameChat,
    WindowKind::GameBuildGuide,
];

pub struct WindowManager<'a> {
    app: &'a AppHandle,
}

impl<'a> WindowManager<'a> {
    pub const fn new(app: &'a AppHandle) -> Self {
        Self { app }
    }

    pub fn window(&self, kind: WindowKind) -> Option<WebviewWindow> {
        self.app.get_webview_window(kind.label())
    }

    pub fn required_window(&self, kind: WindowKind) -> Result<WebviewWindow, String> {
        self.window(kind).ok_or_else(|| {
            format!(
                "Overlay Forge window '{}' was not created at startup.",
                kind.label()
            )
        })
    }

    pub fn configure_runtime(&self, kind: WindowKind) -> Result<(), String> {
        let config = kind.runtime_config();
        let window = self.required_window(config.kind)?;
        window
            .set_always_on_top(config.always_on_top)
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn register_focus_loss_behavior(&self, kind: WindowKind) -> Result<(), String> {
        let config = kind.runtime_config();
        if !config.hide_on_focus_loss {
            return Ok(());
        }

        let window = self.required_window(kind)?;
        let app = self.app.clone();
        window.on_window_event(move |event| {
            if matches!(event, tauri::WindowEvent::Focused(false)) {
                if let Err(error) = WindowManager::new(&app).hide(kind) {
                    eprintln!(
                        "Could not hide '{}' after focus loss: {error}",
                        kind.label()
                    );
                }
            }
        });
        Ok(())
    }

    pub fn show_and_focus(&self, kind: WindowKind) -> Result<WebviewWindow, String> {
        let window = self.required_window(kind)?;
        self.prepare_for_interaction(kind, &window)?;
        window.show().map_err(|error| error.to_string())?;
        let _ = set_overlay_opacity(&window, ACTIVE_WINDOW_OPACITY);
        window.set_focus().map_err(|error| error.to_string())?;
        Ok(window)
    }

    pub fn show_without_activation(&self, kind: WindowKind) -> Result<WebviewWindow, String> {
        let window = self.required_window(kind)?;
        let config = kind.runtime_config();
        self.prepare_for_interaction(kind, &window)?;
        let _ = set_overlay_opacity(&window, config.restore_opacity);
        show_window_without_activation(&window, config.always_on_top)?;
        Ok(window)
    }

    pub fn hide(&self, kind: WindowKind) -> Result<(), String> {
        let window = self.required_window(kind)?;
        hide_window(&window)
    }

    pub fn is_visible(&self, kind: WindowKind) -> Result<bool, String> {
        let window = self.required_window(kind)?;
        window_is_visible(&window)
    }

    pub fn prepare_for_interaction(
        &self,
        kind: WindowKind,
        window: &WebviewWindow,
    ) -> Result<(), String> {
        window
            .set_always_on_top(kind.runtime_config().always_on_top)
            .map_err(|error| error.to_string())?;
        ensure_window_accepts_mouse_input(window)
    }

    pub fn set_position(&self, kind: WindowKind, x: i32, y: i32) -> Result<(), String> {
        self.required_window(kind)?
            .set_position(PhysicalPosition::new(x, y))
            .map_err(|error| error.to_string())
    }

    pub fn set_minimum_size(&self, config: StandaloneWindowConfig) -> Result<(), String> {
        self.required_window(config.base.kind)?
            .set_min_size(Some(PhysicalSize::new(config.min_width, config.min_height)))
            .map_err(|error| error.to_string())
    }

    pub fn set_size(&self, kind: WindowKind, width: u32, height: u32) -> Result<(), String> {
        self.required_window(kind)?
            .set_size(PhysicalSize::new(width, height))
            .map_err(|error| error.to_string())
    }

    pub fn foreground_label(&self) -> Result<Option<String>, String> {
        foreground_window_label(self.app)
    }

    pub fn is_foreground(&self, kind: WindowKind) -> bool {
        is_window_foreground(self.app, kind)
    }

    pub fn remember_foreground_window_as_game(&self) {
        remember_foreground_window_as_game(self.app);
    }
}

pub fn start_manual_drag(window: WebviewWindow) -> Result<(), String> {
    manual_overlay_drag(window)
}

#[cfg(target_os = "windows")]
fn manual_overlay_drag(window: WebviewWindow) -> Result<(), String> {
    if MANUAL_OVERLAY_DRAG_ACTIVE.swap(true, Ordering::SeqCst) {
        return Ok(());
    }

    thread::spawn(move || {
        let _ = manual_overlay_drag_loop(window);
        MANUAL_OVERLAY_DRAG_ACTIVE.store(false, Ordering::SeqCst);
    });

    Ok(())
}

#[cfg(target_os = "windows")]
fn manual_overlay_drag_loop(window: WebviewWindow) -> Result<(), String> {
    use std::mem;
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
    use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;

    let started_at = std::time::Instant::now();
    let start_window_position = window.outer_position().map_err(|error| error.to_string())?;
    let start_cursor = unsafe {
        let mut point: POINT = mem::zeroed();
        if GetCursorPos(&mut point) == 0 {
            return Err("Could not read the mouse position.".to_string());
        }
        point
    };

    loop {
        let left_button_down =
            unsafe { (GetAsyncKeyState(VK_LBUTTON as i32) & 0x8000u16 as i16) != 0 };
        if !left_button_down {
            break;
        }

        let cursor = unsafe {
            let mut point: POINT = mem::zeroed();
            if GetCursorPos(&mut point) == 0 {
                break;
            }
            point
        };
        let next_x = start_window_position.x + (cursor.x - start_cursor.x);
        let next_y = start_window_position.y + (cursor.y - start_cursor.y);
        let _ = window.set_position(PhysicalPosition::new(next_x, next_y));
        if started_at.elapsed() > Duration::from_secs(8) {
            break;
        }
        thread::sleep(Duration::from_millis(8));
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn manual_overlay_drag(_window: WebviewWindow) -> Result<(), String> {
    Err("Manual no-snap overlay drag is only available on Windows.".to_string())
}

#[cfg(target_os = "windows")]
pub fn focus_last_game_window_from_state(state: &AppState) -> Result<bool, String> {
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::WindowsAndMessaging::{IsWindow, SetForegroundWindow};

    let hwnd = state
        .last_game_window
        .lock()
        .map_err(|_| "Last game window state is unavailable.".to_string())?
        .unwrap_or_default();
    if hwnd == 0 {
        return Ok(false);
    }

    let hwnd = hwnd as HWND;
    unsafe {
        if IsWindow(hwnd) == 0 {
            return Ok(false);
        }
        Ok(SetForegroundWindow(hwnd) != 0)
    }
}

#[cfg(not(target_os = "windows"))]
pub fn focus_last_game_window_from_state(_state: &AppState) -> Result<bool, String> {
    Ok(false)
}

#[cfg(target_os = "windows")]
pub fn foreground_window_label(app: &AppHandle) -> Result<Option<String>, String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

    let foreground = unsafe { GetForegroundWindow() };
    if foreground.is_null() {
        return Ok(None);
    }

    for window in app.webview_windows().values() {
        let hwnd = window.hwnd().map_err(|error| error.to_string())?.0
            as windows_sys::Win32::Foundation::HWND;
        if foreground == hwnd {
            return Ok(Some(window.label().to_string()));
        }
    }

    Ok(None)
}

#[cfg(not(target_os = "windows"))]
pub fn foreground_window_label(_app: &AppHandle) -> Result<Option<String>, String> {
    Ok(Some(WindowKind::Main.label().to_string()))
}

#[cfg(target_os = "windows")]
pub fn is_window_foreground(app: &AppHandle, kind: WindowKind) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

    let Some(window) = app.get_webview_window(kind.label()) else {
        return false;
    };
    let Ok(hwnd) = window.hwnd() else {
        return false;
    };
    unsafe { std::ptr::eq(GetForegroundWindow(), hwnd.0) }
}

#[cfg(not(target_os = "windows"))]
pub fn is_window_foreground(_app: &AppHandle, _kind: WindowKind) -> bool {
    false
}

#[cfg(target_os = "windows")]
pub fn remember_foreground_window_as_game(app: &AppHandle) {
    use windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

    let foreground = unsafe { GetForegroundWindow() };
    if foreground.is_null() {
        return;
    }

    for kind in CAPTURE_WINDOW_KINDS {
        if let Some(window) = app.get_webview_window(kind.label()) {
            if let Ok(hwnd) = window.hwnd() {
                if std::ptr::eq(foreground, hwnd.0) {
                    return;
                }
            }
        }
    }

    let state = app.state::<AppState>();
    if let Ok(mut last_game_window) = state.last_game_window.lock() {
        *last_game_window = Some(foreground as isize);
    };
}

#[cfg(not(target_os = "windows"))]
pub fn remember_foreground_window_as_game(_app: &AppHandle) {}

#[cfg(target_os = "windows")]
pub fn show_window_without_activation(
    window: &WebviewWindow,
    always_on_top: bool,
) -> Result<(), String> {
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, ShowWindow, HWND_NOTOPMOST, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE,
        SWP_NOSIZE, SWP_SHOWWINDOW, SW_SHOWNOACTIVATE,
    };

    let hwnd = window.hwnd().map_err(|error| error.to_string())?;
    unsafe {
        ShowWindow(hwnd.0 as HWND, SW_SHOWNOACTIVATE);
        SetWindowPos(
            hwnd.0 as HWND,
            if always_on_top {
                HWND_TOPMOST
            } else {
                HWND_NOTOPMOST
            },
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
        );
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn show_window_without_activation(
    window: &WebviewWindow,
    _always_on_top: bool,
) -> Result<(), String> {
    window.show().map_err(|error| error.to_string())
}

#[cfg(target_os = "windows")]
pub fn hide_window(window: &WebviewWindow) -> Result<(), String> {
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};

    let tauri_result = window.hide().map_err(|error| error.to_string());
    let hwnd = window.hwnd().map_err(|error| error.to_string())?;
    unsafe {
        ShowWindow(hwnd.0 as HWND, SW_HIDE);
    }
    tauri_result
}

#[cfg(not(target_os = "windows"))]
pub fn hide_window(window: &WebviewWindow) -> Result<(), String> {
    window.hide().map_err(|error| error.to_string())
}

#[cfg(target_os = "windows")]
pub fn window_is_visible(window: &WebviewWindow) -> Result<bool, String> {
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::WindowsAndMessaging::IsWindowVisible;

    let tauri_visible = window.is_visible().map_err(|error| error.to_string())?;
    if tauri_visible {
        return Ok(true);
    }

    let hwnd = window.hwnd().map_err(|error| error.to_string())?;
    Ok(unsafe { IsWindowVisible(hwnd.0 as HWND) != 0 })
}

#[cfg(not(target_os = "windows"))]
pub fn window_is_visible(window: &WebviewWindow) -> Result<bool, String> {
    window.is_visible().map_err(|error| error.to_string())
}

#[cfg(target_os = "windows")]
pub fn ensure_window_accepts_mouse_input(window: &WebviewWindow) -> Result<(), String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_NOACTIVATE, WS_EX_TRANSPARENT,
    };

    let hwnd =
        window.hwnd().map_err(|error| error.to_string())?.0 as windows_sys::Win32::Foundation::HWND;
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let next_style = style & !(WS_EX_TRANSPARENT as isize) & !(WS_EX_NOACTIVATE as isize);
        if next_style != style {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, next_style);
        }
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn ensure_window_accepts_mouse_input(_window: &WebviewWindow) -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn set_overlay_opacity(window: &WebviewWindow, opacity: f64) -> Result<(), String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetLayeredWindowAttributes, SetWindowLongPtrW, GWL_EXSTYLE, LWA_ALPHA,
        WS_EX_LAYERED,
    };

    let hwnd =
        window.hwnd().map_err(|error| error.to_string())?.0 as windows_sys::Win32::Foundation::HWND;
    let alpha = (opacity.clamp(0.2, 1.0) * 255.0).round() as u8;

    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | WS_EX_LAYERED as isize);
        if SetLayeredWindowAttributes(hwnd, 0, alpha, LWA_ALPHA) == 0 {
            return Err("Could not set overlay window opacity.".to_string());
        }
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn set_overlay_opacity(_window: &WebviewWindow, _opacity: f64) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::WindowKind;

    #[test]
    fn main_window_is_transient_and_not_always_on_top() {
        let config = WindowKind::Main.runtime_config();

        assert!(!config.always_on_top);
        assert!(config.hide_on_focus_loss);
    }

    #[test]
    fn standalone_windows_remain_always_on_top_and_visible_after_focus_loss() {
        for kind in [WindowKind::GameChat, WindowKind::GameBuildGuide] {
            let config = kind.runtime_config();

            assert!(config.always_on_top);
            assert!(!config.hide_on_focus_loss);
        }
    }
}
