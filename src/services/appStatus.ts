import { invoke } from "@tauri-apps/api/core";

export type AppStatus = {
  hotkey: string;
  databaseReady: boolean;
};

export function getAppStatus() {
  return invoke<AppStatus>("get_app_status");
}
