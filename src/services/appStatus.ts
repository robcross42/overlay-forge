import { invoke } from "@tauri-apps/api/core";

export type MilestoneStatus = {
  milestone: string;
  hotkey: string;
  databaseReady: boolean;
};

export function getMilestoneStatus() {
  return invoke<MilestoneStatus>("get_milestone_status");
}

