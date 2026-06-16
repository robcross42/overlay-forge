import { invoke } from "@tauri-apps/api/core";
import { LogicalSize } from "@tauri-apps/api/dpi";
import { getCurrentWindow } from "@tauri-apps/api/window";

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
