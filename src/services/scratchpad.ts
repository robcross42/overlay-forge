import { invoke } from "@tauri-apps/api/core";

export function loadScratchpad() {
  return invoke<string>("get_scratchpad");
}

export function saveScratchpad(content: string) {
  return invoke<void>("save_scratchpad", { content });
}

