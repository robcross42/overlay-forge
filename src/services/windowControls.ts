import { invoke } from "@tauri-apps/api/core";
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
  return overlayWindow.startDragging();
}

export function startOverlayResize(direction: ResizeDirection) {
  return overlayWindow.startResizeDragging(direction);
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

export function shutdownOverlayApp() {
  return invoke<void>("shutdown_app");
}
