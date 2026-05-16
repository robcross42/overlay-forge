import { invoke } from "@tauri-apps/api/core";

export type ProjectStatus = "ACTIVE" | "ARCHIVED";

export type Project = {
  id: number;
  name: string;
  description: string;
  status: ProjectStatus;
  createdAt: string;
  updatedAt: string;
};

export type ProjectInput = {
  name: string;
  description: string;
  status: ProjectStatus;
};

export function listProjects() {
  return invoke<Project[]>("list_projects");
}

export function createProject(input: ProjectInput) {
  return invoke<Project>("create_project", input);
}

export function updateProject(id: number, input: ProjectInput) {
  return invoke<Project>("update_project", { id, ...input });
}

export function deleteProject(id: number) {
  return invoke<void>("delete_project", { id });
}
