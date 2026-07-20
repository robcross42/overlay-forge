import { invoke } from "@tauri-apps/api/core";
import { LogicalSize } from "@tauri-apps/api/dpi";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { Window as TauriWindow } from "@tauri-apps/api/window";

type ResizeDirection =
  | "East"
  | "North"
  | "NorthEast"
  | "NorthWest"
  | "South"
  | "SouthEast"
  | "SouthWest"
  | "West";

const overlayWindow = getCurrentWindow();

export function startOverlayDrag() {
  return invoke<void>("start_manual_overlay_drag");
}

export function startOverlayResize(direction: ResizeDirection) {
  return overlayWindow.startResizeDragging(direction);
}

export function setOverlayMinimumSize(width: number, height: number) {
  return overlayWindow.setMinSize(new LogicalSize(width, height));
}

export function setOverlayWindowOpacity(opacity: number) {
  return invoke<void>("set_overlay_window_opacity", { opacity });
}

export function isOverlayForgeForeground() {
  return invoke<boolean>("is_overlay_forge_foreground");
}

export function getOverlayForgeForegroundWindowLabel() {
  return invoke<string | null>("get_overlay_forge_foreground_window_label");
}

export async function applyStandaloneOverlayFocusState(window: TauriWindow = overlayWindow) {
  async function applyFocusState() {
    const [currentWindowFocused, foregroundOverlayLabel] = await Promise.all([
      window.isFocused().catch(() => false),
      getOverlayForgeForegroundWindowLabel().catch(() => null)
    ]);
    const overlayIsForeground =
      currentWindowFocused || foregroundOverlayLabel === window.label;

    document.documentElement.classList.toggle("standalone-overlay-focused", overlayIsForeground);
    document.documentElement.classList.toggle("standalone-overlay-unfocused", !overlayIsForeground);
    await setOverlayWindowOpacity(1).catch(() => {});
  }

  await applyFocusState();

  const focusPollId = globalThis.setInterval(() => {
    void applyFocusState();
  }, 250);
  const unlistenFocus = await window.onFocusChanged(() => {
    globalThis.setTimeout(() => {
      void applyFocusState();
    }, 50);
  });

  return () => {
    globalThis.clearInterval(focusPollId);
    unlistenFocus();
  };
}

export function focusLastGameWindow() {
  return invoke<boolean>("focus_last_game_window");
}

export function clearOverlayMinimumSize() {
  return overlayWindow.setMinSize(null);
}

export function minimizeOverlayWindow() {
  return overlayWindow.minimize();
}

export async function toggleOverlayMaximize(shouldToggle: boolean) {
  if (shouldToggle) {
    await overlayWindow.toggleMaximize();
  }

  return overlayWindow.isMaximized();
}

export function hideOverlayWindow() {
  return overlayWindow.hide();
}

export function showOverlayWindow() {
  return overlayWindow.show().then(() => overlayWindow.setFocus());
}

export function isOverlayWindowVisible() {
  return overlayWindow.isVisible();
}

export function shutdownOverlayApp() {
  return invoke<void>("shutdown_app");
}
