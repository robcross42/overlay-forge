import { invoke } from "@tauri-apps/api/core";

export type ProjectGitHubRepository = {
  id: number;
  projectId: number;
  repositoryFullName: string;
  repositoryUrl: string;
  defaultBranch: string;
  visibility: string;
  lastFetchedAt: string;
  lastFetchStatus: string;
  createdAt: string;
  updatedAt: string;
};

export function getProjectGitHubRepository(projectId: number) {
  return invoke<ProjectGitHubRepository | null>("get_project_github_repository", { projectId });
}

export function saveProjectGitHubRepository(projectId: number, repositoryFullName: string) {
  return invoke<ProjectGitHubRepository>("save_project_github_repository", {
    projectId,
    repositoryFullName
  });
}

export function deleteProjectGitHubRepository(projectId: number) {
  return invoke<void>("delete_project_github_repository", { projectId });
}

export function fetchProjectGitHubMetadata(projectId: number) {
  return invoke<ProjectGitHubRepository>("fetch_project_github_metadata", { projectId });
}
