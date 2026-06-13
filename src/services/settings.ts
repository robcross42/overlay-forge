import { invoke } from "@tauri-apps/api/core";

export type ApiKeyStatus = {
  isConfigured: boolean;
  source: string;
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
