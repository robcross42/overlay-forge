import { useEffect, useMemo, useState } from "react";
import {
  deleteProjectGitHubRepository,
  fetchProjectGitHubMetadata,
  getProjectGitHubRepository,
  saveProjectGitHubRepository
} from "../../services/github";
import type { ProjectGitHubRepository } from "../../services/github";
import { PlanningChat } from "../planning-chat/PlanningChat";
import {
  deleteProjectMarkdownContext,
  getProjectMarkdownContext,
  loadProjectMarkdownContext,
  saveProjectMarkdownContext
} from "../../services/planningChat";
import type {
  PlanningConversation,
  ProjectMarkdownContext,
  ProjectMarkdownContextPayload
} from "../../services/planningChat";
import { createProject, deleteProject, updateProject } from "../../services/projects";
import type { Project, ProjectInput, ProjectStatus } from "../../services/projects";
import { formatUnknownError as formatError } from "../../utils/errors";

const emptyProject: ProjectInput = {
  name: "",
  description: "",
  status: "ACTIVE"
};

type ProjectMode = "idle" | "create" | "view" | "edit";
type WorkspaceSection = "overview" | "chat" | "references" | "edit";

type ProjectNavAction = {
  type: "create" | "overview" | "newChat" | "chat" | "references" | "edit";
  projectId?: number;
  conversationId?: number;
  nonce: number;
};

type ProjectsProps = {
  chatOverlayMode?: boolean;
  projects: Project[];
  selectedProjectId: number | null;
  navAction: ProjectNavAction | null;
  onEnterChatOverlayMode?: () => void;
  onExitChatOverlayMode?: () => void;
  onSelectProject: (projectId: number | null) => void;
  onProjectCreated: (project: Project) => void;
  onProjectUpdated: (project: Project) => void;
  onProjectDeleted: (projectId: number) => void;
  onProjectConversationsChanged: (
    projectId: number,
    conversations: PlanningConversation[]
  ) => void;
};

export function Projects({
  chatOverlayMode = false,
  projects,
  selectedProjectId,
  navAction,
  onEnterChatOverlayMode,
  onExitChatOverlayMode,
  onSelectProject,
  onProjectCreated,
  onProjectUpdated,
  onProjectDeleted,
  onProjectConversationsChanged
}: ProjectsProps) {
  const [projectMode, setProjectMode] = useState<ProjectMode>("idle");
  const [form, setForm] = useState<ProjectInput>(emptyProject);
  const [status, setStatus] = useState("Loading projects");
  const [githubLink, setGithubLink] = useState<ProjectGitHubRepository | null>(null);
  const [githubFullName, setGithubFullName] = useState("");
  const [githubStatus, setGithubStatus] = useState("Select a project");
  const [isFetchingGithub, setIsFetchingGithub] = useState(false);
  const [workspaceSection, setWorkspaceSection] = useState<WorkspaceSection>("overview");
  const [markdownContext, setMarkdownContext] = useState<ProjectMarkdownContext | null>(null);
  const [markdownRootPath, setMarkdownRootPath] = useState("");
  const [markdownReadmePath, setMarkdownReadmePath] = useState("README.md");
  const [markdownPayload, setMarkdownPayload] = useState<ProjectMarkdownContextPayload>({
    files: [],
    warnings: []
  });
  const [isSavingMarkdownContext, setIsSavingMarkdownContext] = useState(false);

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
        setWorkspaceSection("edit");
        setStatus("Editing project");
      }
      return;
    }

    if (navAction.projectId) {
      const project = projects.find((item) => item.id === navAction.projectId);
      if (project) {
      selectProject(project, navAction.type === "newChat" ? "chat" : navAction.type);
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

  useEffect(() => {
    if (!selectedProjectId) {
      setMarkdownContext(null);
      setMarkdownRootPath("");
      setMarkdownReadmePath("README.md");
      setMarkdownPayload({ files: [], warnings: [] });
      return;
    }

    Promise.all([
      getProjectMarkdownContext(selectedProjectId),
      loadProjectMarkdownContext(selectedProjectId)
    ])
      .then(([context, payload]) => {
        setMarkdownContext(context);
        setMarkdownRootPath(context?.rootPath ?? "");
        setMarkdownReadmePath(context?.readmePath ?? "README.md");
        setMarkdownPayload(payload);
      })
      .catch(() => {
        setMarkdownContext(null);
        setMarkdownPayload({ files: [], warnings: [] });
      });
  }, [selectedProjectId]);

  function selectProject(project: Project, section: WorkspaceSection = "overview") {
    onSelectProject(project.id);
    setProjectMode("view");
    setWorkspaceSection(section);
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
    setWorkspaceSection("edit");
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
        selectProject(updated, "overview");
        setStatus("Project saved");
      } else {
        const created = await createProject({
          name,
          description: form.description,
          status: form.status
        });
        onProjectCreated(created);
        selectProject(created, "edit");
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

  async function onSaveMarkdownContext() {
    if (!selectedProject) {
      setStatus("Select a project before configuring Markdown context");
      return;
    }

    const rootPath = markdownRootPath.trim();
    if (!rootPath) {
      setStatus("Markdown context root is required");
      return;
    }

    setIsSavingMarkdownContext(true);
    setStatus("Saving Markdown context root");
    try {
      const saved = await saveProjectMarkdownContext({
        projectId: selectedProject.id,
        rootPath,
        readmePath: markdownReadmePath.trim() || "README.md"
      });
      const payload = await loadProjectMarkdownContext(selectedProject.id);
      setMarkdownContext(saved);
      setMarkdownRootPath(saved.rootPath);
      setMarkdownReadmePath(saved.readmePath);
      setMarkdownPayload(payload);
      setStatus("Markdown context root saved");
    } catch (error) {
      setStatus(formatError(error));
    } finally {
      setIsSavingMarkdownContext(false);
    }
  }

  async function onClearMarkdownContext() {
    if (!selectedProject) {
      return;
    }

    setIsSavingMarkdownContext(true);
    setStatus("Clearing Markdown context root");
    try {
      await deleteProjectMarkdownContext(selectedProject.id);
      setMarkdownContext(null);
      setMarkdownRootPath("");
      setMarkdownReadmePath("README.md");
      setMarkdownPayload({ files: [], warnings: [] });
      setStatus("Markdown context root cleared");
    } catch (error) {
      setStatus(formatError(error));
    } finally {
      setIsSavingMarkdownContext(false);
    }
  }

  async function onReloadMarkdownContext() {
    if (!selectedProject) {
      return;
    }

    setStatus("Reloading Markdown context");
    try {
      const payload = await loadProjectMarkdownContext(selectedProject.id);
      setMarkdownPayload(payload);
      setStatus("Markdown context reloaded");
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  return (
    <section className="project-focus-panel">
      {selectedProject || projectMode === "create" ? (
        <>
          {(workspaceSection === "overview" || projectMode === "create") && (
            <section className="project-overview-panel" aria-label="Project overview">
              <div className="project-focus-heading">
                <div>
                  <p>{projectMode === "create" ? "New Project" : "Project Overview"}</p>
                  <h3>{selectedProject?.name ?? "Create project"}</h3>
                </div>
                <span className="save-pill">{status}</span>
              </div>

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
                        setWorkspaceSection("edit");
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
            </section>
          )}

          {selectedProject && workspaceSection === "edit" && (
            <section className="project-edit-panel" aria-label="Project edit">
              <div className="project-focus-heading">
                <div>
                  <p>Project Edit</p>
                  <h3>{selectedProject.name}</h3>
                </div>
                <span className="save-pill">{status}</span>
              </div>

              <div className="project-edit-grid">
                <form className="editor-form project-editor-form project-edit-card">
                  <input
                    aria-label="Project name"
                    className="text-input"
                    onChange={(event) => updateField("name", event.target.value)}
                    placeholder="Project name"
                    value={form.name}
                  />
                  <textarea
                    aria-label="Project description"
                    className="body-input"
                    onChange={(event) => updateField("description", event.target.value)}
                    placeholder="Project description"
                    value={form.description}
                  />
                  <label className="field-label">
                    <span>Status</span>
                    <select
                      aria-label="Project status"
                      className="text-input"
                      onChange={(event) => updateField("status", event.target.value as ProjectStatus)}
                      value={form.status}
                    >
                      <option value="ACTIVE">Active</option>
                      <option value="ARCHIVED">Archived</option>
                    </select>
                  </label>
                  <div className="form-actions">
                    <button
                      className="primary-button"
                      onClick={() => void onSaveProject()}
                      type="button"
                    >
                      Save Project
                    </button>
                    <button
                      className="primary-button"
                      onClick={() => void onDeleteProject()}
                      type="button"
                    >
                      Delete
                    </button>
                  </div>
                </form>

                <section className="github-link-panel project-edit-card" aria-label="Project GitHub repository">
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
                    <dl className="metadata-grid compact-metadata">
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
                        <dt>Fetch status</dt>
                        <dd>{githubLink.lastFetchStatus || "Not fetched"}</dd>
                      </div>
                    </dl>
                  )}
                </section>

                <section className="markdown-context-config project-edit-card" aria-label="Project Markdown context">
                  <div className="markdown-context-heading">
                    <div>
                      <p>Project Markdown</p>
                      <h4>{markdownContext ? "Configured" : "Not configured"}</h4>
                    </div>
                    <button
                      className="ghost-button"
                      disabled={isSavingMarkdownContext}
                      onClick={() => void onReloadMarkdownContext()}
                      type="button"
                    >
                      Reload
                    </button>
                  </div>

                  <label className="field-label">
                    <span>Root path</span>
                    <input
                      aria-label="Markdown context root path"
                      className="text-input"
                      disabled={isSavingMarkdownContext}
                      onChange={(event) => setMarkdownRootPath(event.target.value)}
                      placeholder="C:\\Dev\\repos\\overlay-forge"
                      value={markdownRootPath}
                    />
                  </label>

                  <label className="field-label">
                    <span>README path</span>
                    <input
                      aria-label="Markdown context README path"
                      className="text-input"
                      disabled={isSavingMarkdownContext}
                      onChange={(event) => setMarkdownReadmePath(event.target.value)}
                      placeholder="README.md"
                      value={markdownReadmePath}
                    />
                  </label>

                  <div className="markdown-context-actions">
                    <button
                      className="primary-button"
                      disabled={isSavingMarkdownContext || markdownRootPath.trim().length === 0}
                      onClick={() => void onSaveMarkdownContext()}
                      type="button"
                    >
                      Save
                    </button>
                    <button
                      className="ghost-button"
                      disabled={!markdownContext || isSavingMarkdownContext}
                      onClick={() => void onClearMarkdownContext()}
                      type="button"
                    >
                      Clear
                    </button>
                  </div>

                  <div className="markdown-context-summary">
                    <span>{markdownPayload.files.filter((file) => file.included).length} file(s) loaded</span>
                    {markdownPayload.warnings.length > 0 && (
                      <span>{markdownPayload.warnings.length} warning(s)</span>
                    )}
                  </div>
                </section>
              </div>
            </section>
          )}

          {selectedProject && workspaceSection === "chat" && (
            <PlanningChat
              chatOverlayMode={chatOverlayMode}
              focused
              initialConversationId={navAction?.conversationId}
              onEnterChatOverlayMode={onEnterChatOverlayMode}
              onExitChatOverlayMode={onExitChatOverlayMode}
              onConversationsChanged={(conversations) =>
                onProjectConversationsChanged(selectedProject.id, conversations)
              }
              project={selectedProject}
              startInNewConversation={navAction?.type === "newChat"}
            />
          )}

          {selectedProject && workspaceSection === "references" && (
            <section className="references-panel" aria-label="Project references">
              <div className="references-heading">
                <div>
                  <p>References</p>
                  <h4>Local Context Sources</h4>
                </div>
                <span className="save-pill">Project scoped</span>
              </div>

              <div className="reference-summary-grid">
                <article className="reference-summary-item">
                  <span>Project Details</span>
                  <strong>{selectedProject.name}</strong>
                  <p>Available from Project Overview and Project Edit.</p>
                </article>
                <article className="reference-summary-item">
                  <span>GitHub Repository</span>
                  <strong>{githubLink?.repositoryFullName || "No repository linked"}</strong>
                  <p>Configured from Project Edit.</p>
                </article>
                <article className="reference-summary-item">
                  <span>Project Markdown</span>
                  <strong>{markdownPayload.files.filter((file) => file.included).length} file(s)</strong>
                  <p>Configured from Project Edit and included in chat, preview, and bridge drafts.</p>
                </article>
                <article className="reference-summary-item">
                  <span>Manual Attachments</span>
                  <strong>Conversation scoped</strong>
                  <p>Managed from the selected conversation context panel.</p>
                </article>
              </div>
            </section>
          )}
        </>
      ) : (
        <div className="empty-editor-state">
          <p>Create or select a project to begin.</p>
        </div>
      )}
    </section>
  );
}

function formatProjectStatus(status: ProjectStatus) {
  return status === "ARCHIVED" ? "Archived" : "Active";
}
