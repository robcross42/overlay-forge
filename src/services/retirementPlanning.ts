import { invoke } from "@tauri-apps/api/core";

export type RetirementPlanningProfile = {
  id: number;
  name: string;
  currencyCode: "CAD";
  retirementDefinition: string;
  profileStatus: "foundation" | "active" | "archived";
  createdAt: string;
  modifiedAt: string;
};

export function getRetirementPlanningProfile() {
  return invoke<RetirementPlanningProfile>("get_retirement_planning_profile");
}
