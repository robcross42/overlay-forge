import { invoke } from "@tauri-apps/api/core";

export type ApiKeyStatus = {
  isConfigured: boolean;
  source: string;
};

export type KeybindRecord = {
  action: string;
  label: string;
  keys: string[];
};

export function getOpenAiApiKeyStatus() {
  return invoke<ApiKeyStatus>("get_openai_api_key_status");
}

export function saveOpenAiApiKey(apiKey: string) {
  return invoke<ApiKeyStatus>("save_openai_api_key", { apiKey });
}

export function clearOpenAiApiKey() {
  return invoke<ApiKeyStatus>("clear_openai_api_key");
}

export function listKeybinds() {
  return invoke<KeybindRecord[]>("list_keybinds");
}

export function saveKeybinds(keybinds: KeybindRecord[]) {
  return invoke<KeybindRecord[]>("save_keybinds", { keybinds });
}

export function resetKeybinds() {
  return invoke<KeybindRecord[]>("reset_keybinds");
}
