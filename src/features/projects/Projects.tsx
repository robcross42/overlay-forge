import { useEffect, useMemo, useState } from "react";
import {
  deleteProjectGitHubRepository,
  fetchProjectGitHubMetadata,
  getProjectGitHubRepository,
  saveProjectGitHubRepository
} from "../../services/github";
import type { ProjectGitHubRepository } from "../../services/github";
import { PlanningChat } from "../planning-chat/PlanningChat";
import { createProject, deleteProject, updateProject } from "../../services/projects";
import type { Project, ProjectInput, ProjectStatus } from "../../services/projects";

const emptyProject: ProjectInput = {
  name: "",
  description: "",
  status: "ACTIVE"
};

type ProjectMode = "idle" | "create" | "view" | "edit";
type WorkspaceSection = "overview" | "github" | "chat" | "references";

type ProjectNavAction = {
  type: "create" | "edit";
  projectId?: number;
  nonce: number;
};

type ProjectsProps = {
  projects: Project[];
  selectedProjectId: number | null;
  navAction: ProjectNavAction | null;
  onSelectProject: (projectId: number | null) => void;
  onProjectCreated: (project: Project) => void;
  onProjectUpdated: (project: Project) => void;
  onProjectDeleted: (projectId: number) => void;
};

export function Projects({
  projects,
  selectedProjectId,
  navAction,
  onSelectProject,
  onProjectCreated,
  onProjectUpdated,
  onProjectDeleted
}: ProjectsProps) {
  const [projectMode, setProjectMode] = useState<ProjectMode>("idle");
  const [form, setForm] = useState<ProjectInput>(emptyProject);
  const [status, setStatus] = useState("Loading projects");
  const [githubLink, setGithubLink] = useState<ProjectGitHubRepository | null>(null);
  const [githubFullName, setGithubFullName] = useState("");
  const [githubStatus, setGithubStatus] = useState("Select a project");
  const [isFetchingGithub, setIsFetchingGithub] = useState(false);
  const [workspaceSection, setWorkspaceSection] = useState<WorkspaceSection>("overview");

  const selectedProject = useMemo(
    () => projects.find((project) => project.id === selectedProjectId) ?? null,
    [projects, selectedProjectId]
  );

  useEffect(() => {
    setStatus(projects.length === 0 ? "No projects yet" : `${projects.length} project(s)`);
  }, [projects.length]);

  useEffect(() => {
    if (!selectedProject) {
      if (projectMode === "view" || projectMode === "edit") {
        setProjectMode("idle");
        setForm(emptyProject);
      }
      return;
    }

    if (projectMode === "idle") {
      selectProject(selectedProject);
    }
  }, [selectedProject?.id]);

  useEffect(() => {
    if (!navAction) {
      return;
    }

    if (navAction.type === "create") {
      newProject();
      return;
    }

    if (navAction.type === "edit" && navAction.projectId) {
      const project = projects.find((item) => item.id === navAction.projectId);
      if (project) {
        selectProject(project);
        setProjectMode("edit");
        setStatus("Editing project");
      }
    }
  }, [navAction?.nonce]);

  useEffect(() => {
    if (!selectedProjectId) {
      setGithubLink(null);
      setGithubFullName("");
      setGithubStatus("Select a project");
      setIsFetchingGithub(false);
      return;
    }

    setGithubStatus("Loading GitHub link");
    getProjectGitHubRepository(selectedProjectId)
      .then((link) => {
        setGithubLink(link);
        setGithubFullName(link?.repositoryFullName ?? "");
        setGithubStatus(link ? link.lastFetchStatus || "Repository linked" : "No repository linked");
      })
      .catch((error) => {
        setGithubLink(null);
        setGithubFullName("");
        setGithubStatus(formatError(error));
      });
  }, [selectedProjectId]);

  function selectProject(project: Project) {
    onSelectProject(project.id);
    setProjectMode("view");
    setWorkspaceSection("overview");
    setForm({
      name: project.name,
      description: project.description,
      status: project.status
    });
  }

  function newProject() {
    onSelectProject(null);
    setProjectMode("create");
    setForm(emptyProject);
    setStatus("New project");
    setGithubLink(null);
    setGithubFullName("");
    setGithubStatus("Save the project before linking GitHub");
    setWorkspaceSection("overview");
  }

  async function onSaveProject() {
    const name = form.name.trim();

    if (!name) {
      setStatus("Project name is required");
      return;
    }

    try {
      if (selectedProject) {
        const updated = await updateProject(selectedProject.id, {
          name,
          description: form.description,
          status: form.status
        });
        onProjectUpdated(updated);
        selectProject(updated);
        setStatus("Project saved");
      } else {
        const created = await createProject({
          name,
          description: form.description,
          status: form.status
        });
        onProjectCreated(created);
        selectProject(created);
        setStatus("Project added");
      }
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  async function onDeleteProject() {
    if (!selectedProject) {
      return;
    }

    const confirmed = window.confirm(
      `Delete "${selectedProject.name}"? This removes the local project only.`
    );
    if (!confirmed) {
      return;
    }

    try {
      await deleteProject(selectedProject.id);
      onProjectDeleted(selectedProject.id);
      onSelectProject(null);
      setProjectMode("idle");
      setForm(emptyProject);
      setGithubLink(null);
      setGithubFullName("");
      setGithubStatus("Select a project");
      setWorkspaceSection("overview");
      setStatus("Project deleted");
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  function updateField<K extends keyof ProjectInput>(field: K, value: ProjectInput[K]) {
    setForm((current) => ({ ...current, [field]: value }));
  }

  async function onSaveGitHubLink() {
    if (!selectedProject) {
      setGithubStatus("Select a project before linking GitHub");
      return;
    }

    const repositoryFullName = githubFullName.trim();
    if (!repositoryFullName) {
      setGithubStatus("GitHub repository full name is required");
      return;
    }

    try {
      const saved = await saveProjectGitHubRepository(selectedProject.id, repositoryFullName);
      setGithubLink(saved);
      setGithubFullName(saved.repositoryFullName);
      setGithubStatus("Repository link saved");
    } catch (error) {
      setGithubStatus(formatError(error));
    }
  }

  async function onDeleteGitHubLink() {
    if (!selectedProject) {
      return;
    }

    try {
      await deleteProjectGitHubRepository(selectedProject.id);
      setGithubLink(null);
      setGithubFullName("");
      setGithubStatus("Repository link removed");
    } catch (error) {
      setGithubStatus(formatError(error));
    }
  }

  async function onFetchGitHubMetadata() {
    if (!selectedProject) {
      setGithubStatus("Select a project before fetching GitHub metadata");
      return;
    }

    setIsFetchingGithub(true);
    setGithubStatus("Fetching GitHub metadata");
    try {
      const fetched = await fetchProjectGitHubMetadata(selectedProject.id);
      setGithubLink(fetched);
      setGithubFullName(fetched.repositoryFullName);
      setGithubStatus(fetched.lastFetchStatus || "GitHub metadata fetched");
    } catch (error) {
      setGithubStatus(formatError(error));
      try {
        const link = await getProjectGitHubRepository(selectedProject.id);
        setGithubLink(link);
        setGithubFullName(link?.repositoryFullName ?? githubFullName);
      } catch {
        // Keep the visible error from the failed fetch.
      }
    } finally {
      setIsFetchingGithub(false);
    }
  }

  return (
    <section className="feature-panel">
      <div className="panel-heading">
        <div>
          <p>Local Projects</p>
          <h3>Projects</h3>
        </div>
        <span className="save-pill">{status}</span>
      </div>

      <div className="split-feature-body project-feature-body">
        {selectedProject || projectMode === "create" ? (
          <div className="project-workspace">
            {selectedProject && (
              <div className="workspace-context-header">
                <div className="workspace-context-copy">
                  <p>Active Workspace</p>
                  <h4>{selectedProject.name}</h4>
                  <span>{formatProjectStatus(selectedProject.status)}</span>
                </div>

                <nav className="workspace-tabs" aria-label="Project workspace sections">
                  <button
                    className={
                      workspaceSection === "overview" ? "workspace-tab active" : "workspace-tab"
                    }
                    onClick={() => setWorkspaceSection("overview")}
                    type="button"
                  >
                    Overview
                  </button>
                  <button
                    className={
                      workspaceSection === "github" ? "workspace-tab active" : "workspace-tab"
                    }
                    onClick={() => setWorkspaceSection("github")}
                    type="button"
                  >
                    GitHub
                  </button>
                  <button
                    className={workspaceSection === "chat" ? "workspace-tab active" : "workspace-tab"}
                    onClick={() => setWorkspaceSection("chat")}
                    type="button"
                  >
                    Chat
                  </button>
                  <button
                    className={
                      workspaceSection === "references" ? "workspace-tab active" : "workspace-tab"
                    }
                    onClick={() => setWorkspaceSection("references")}
                    type="button"
                  >
                    References
                  </button>
                </nav>
              </div>
            )}

            <div className="project-workspace-content">
              {(workspaceSection === "overview" || projectMode === "create") && (
                <form className="editor-form project-editor-form">
                  <input
                    aria-label="Project name"
                    className="text-input"
                    readOnly={projectMode === "view"}
                    onChange={(event) => updateField("name", event.target.value)}
                    placeholder="Project name"
                    value={form.name}
                  />

                  <textarea
                    aria-label="Project description"
                    className="body-input"
                    readOnly={projectMode === "view"}
                    onChange={(event) => updateField("description", event.target.value)}
                    placeholder="Project description"
                    value={form.description}
                  />

                  <label className="field-label">
                    <span>Status</span>
                    <select
                      aria-label="Project status"
                      className="text-input"
                      disabled={projectMode === "view"}
                      onChange={(event) => updateField("status", event.target.value as ProjectStatus)}
                      value={form.status}
                    >
                      <option value="ACTIVE">Active</option>
                      <option value="ARCHIVED">Archived</option>
                    </select>
                  </label>

                  <div className="form-actions">
                    {projectMode === "view" ? (
                      <button
                        className="primary-button"
                        onClick={() => {
                          setProjectMode("edit");
                          setStatus("Editing project");
                        }}
                        type="button"
                      >
                        Edit
                      </button>
                    ) : (
                      <button
                        className="primary-button"
                        onClick={() => void onSaveProject()}
                        type="button"
                      >
                        {selectedProject ? "Save" : "Add"}
                      </button>
                    )}
                    {selectedProject && (
                      <button
                        className="primary-button"
                        onClick={() => void onDeleteProject()}
                        type="button"
                      >
                        Delete
                      </button>
                    )}
                  </div>
                </form>
              )}

              {selectedProject && workspaceSection === "github" && (
                <section className="github-link-panel" aria-label="Project GitHub repository">
                  <div className="github-link-heading">
                    <div>
                      <p>GitHub Repository</p>
                      <h4>Project Link</h4>
                    </div>
                    <span className={isFetchingGithub ? "save-pill save-pill-loading" : "save-pill"}>
                      {githubStatus}
                    </span>
                  </div>

                  <label className="field-label">
                    <span>Repository full name</span>
                    <input
                      aria-label="GitHub repository full name"
                      className="text-input"
                      disabled={isFetchingGithub}
                      onChange={(event) => setGithubFullName(event.target.value)}
                      placeholder="owner/repository-name"
                      value={githubFullName}
                    />
                  </label>

                  <div className="form-actions">
                    <button
                      className="primary-button"
                      disabled={isFetchingGithub}
                      onClick={() => void onSaveGitHubLink()}
                      type="button"
                    >
                      Save Link
                    </button>
                    <button
                      className="primary-button"
                      disabled={!githubLink || isFetchingGithub}
                      onClick={() => void onFetchGitHubMetadata()}
                      type="button"
                    >
                      Fetch Metadata
                    </button>
                    {githubLink && (
                      <button
                        className="ghost-button"
                        disabled={isFetchingGithub}
                        onClick={() => void onDeleteGitHubLink()}
                        type="button"
                      >
                        Remove Link
                      </button>
                    )}
                  </div>

                  {githubLink && (
                    <dl className="metadata-grid">
                      <div>
                        <dt>Repository</dt>
                        <dd>{githubLink.repositoryFullName}</dd>
                      </div>
                      <div>
                        <dt>Default branch</dt>
                        <dd>{githubLink.defaultBranch || "Not fetched"}</dd>
                      </div>
                      <div>
                        <dt>Visibility</dt>
                        <dd>{githubLink.visibility || "Not fetched"}</dd>
                      </div>
                      <div>
                        <dt>Repository URL</dt>
                        <dd>{githubLink.repositoryUrl || "Not fetched"}</dd>
                      </div>
                      <div>
                        <dt>Last fetched</dt>
                        <dd>{githubLink.lastFetchedAt || "Not fetched"}</dd>
                      </div>
                      <div>
                        <dt>Fetch status</dt>
                        <dd>{githubLink.lastFetchStatus || "Not fetched"}</dd>
                      </div>
                    </dl>
                  )}
                </section>
              )}

              {selectedProject && workspaceSection === "chat" && <PlanningChat project={selectedProject} />}

              {selectedProject && workspaceSection === "references" && (
                <section className="references-panel" aria-label="Project references">
                  <div className="references-heading">
                    <div>
                      <p>References</p>
                      <h4>Local Context Sources</h4>
                    </div>
                    <span className="save-pill">Planned</span>
                  </div>

                  <div className="reference-summary-grid">
                    <article className="reference-summary-item">
                      <span>Project Details</span>
                      <strong>{selectedProject.name}</strong>
                      <p>Available from the selected project overview.</p>
                    </article>
                    <article className="reference-summary-item">
                      <span>GitHub Repository</span>
                      <strong>{githubLink?.repositoryFullName || "No repository linked"}</strong>
                      <p>Available when a repository is linked in the GitHub section.</p>
                    </article>
                    <article className="reference-summary-item">
                      <span>Attachments</span>
                      <strong>Planned later</strong>
                      <p>Manual context attachment workflows are deferred beyond Milestone 8.</p>
                    </article>
                    <article className="reference-summary-item">
                      <span>Prompt Context</span>
                      <strong>Planned later</strong>
                      <p>Prompt preview and context inclusion are deferred beyond Milestone 8.</p>
                    </article>
                  </div>
                </section>
              )}
            </div>
          </div>
        ) : (
          <div className="empty-editor-state">
            <p>Create or select a project to begin.</p>
          </div>
        )}
      </div>
    </section>
  );
}

function formatProjectStatus(status: ProjectStatus) {
  return status === "ARCHIVED" ? "Archived" : "Active";
}

function formatError(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
