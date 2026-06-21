import { invoke } from "@tauri-apps/api/core";

export type SmokingEvent = {
  id: number;
  smokedAt: string;
  source: string;
  notes: string;
  createdAt: string;
};

export type SmokingCessationSettings = {
  id: number;
  patchLabel: string;
  patchStartedAt: string;
  patchTimezone: string;
  currentCigaretteCount: number;
  createdAt: string;
  updatedAt: string;
};

export type SmokingCessationExport = {
  exportPath: string;
};

export function listSmokingEvents() {
  return invoke<SmokingEvent[]>("list_smoking_events");
}

export function recordSmokingEvent(smokedAt?: string, notes?: string) {
  return invoke<SmokingEvent>("record_smoking_event", {
    smokedAt: smokedAt ?? null,
    notes: notes ?? null
  });
}

export function deleteSmokingEvent(id: number) {
  return invoke<void>("delete_smoking_event", { id });
}

export function getSmokingCessationSettings() {
  return invoke<SmokingCessationSettings>("get_smoking_cessation_settings");
}

export function updateSmokingCigaretteCount(currentCigaretteCount: number) {
  return invoke<SmokingCessationSettings>("update_smoking_cigarette_count", {
    currentCigaretteCount
  });
}

export function exportSmokingCessationChatGptContext() {
  return invoke<SmokingCessationExport>("export_smoking_cessation_chatgpt_context");
}
