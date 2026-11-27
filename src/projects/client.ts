import { invoke } from "@tauri-apps/api/core";

export const stages = ["Idea", "Planned", "In progress", "Ready for review", "On hold", "Completed"] as const;
export type ProjectStage = typeof stages[number];
export interface ProjectLink { label: string; target: string }
export interface ProjectDraft {
  title: string; description: string; stage: ProjectStage;
  nextStep: string; waitingOnRob: string; links: ProjectLink[];
}
export interface Project extends ProjectDraft { id: string; archived: boolean }
export interface ProjectQuery { search: string; stage: ProjectStage | null; archived: boolean; waitingOnly: boolean }

export class ProjectsClient {
  constructor(private readonly sessionId: string) {}
  static empty(): ProjectDraft {
    return { title: "", description: "", stage: "Idea", nextStep: "", waitingOnRob: "", links: [] };
  }
  static draft(project: Project): ProjectDraft {
    const { title, description, stage, nextStep, waitingOnRob, links } = project;
    return { title, description, stage, nextStep, waitingOnRob, links: links.map(link => ({ ...link })) };
  }
  list(query: ProjectQuery) { return invoke<Project[]>("projects_list", { sessionId: this.sessionId, query }); }
  save(id: string | null, draft: ProjectDraft) { return invoke<string>("projects_save", { sessionId: this.sessionId, id, draft }); }
  archive(id: string, archived: boolean) { return invoke<void>("projects_archive", { sessionId: this.sessionId, id, archived }); }
  openWebLink(target: string) { return invoke<void>("projects_open_web_link", { sessionId: this.sessionId, target }); }
  delete(id: string, confirmation: string) { return invoke<void>("projects_delete", { sessionId: this.sessionId, id, confirmation }); }
}
