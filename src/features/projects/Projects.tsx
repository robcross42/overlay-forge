import { useEffect, useMemo, useState } from "react";
import { createProject, deleteProject, listProjects, updateProject } from "../../services/projects";
import type { Project, ProjectInput, ProjectStatus } from "../../services/projects";

const emptyProject: ProjectInput = {
  name: "",
  description: "",
  status: "ACTIVE"
};

type ProjectMode = "idle" | "create" | "view" | "edit";

export function Projects() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [projectMode, setProjectMode] = useState<ProjectMode>("idle");
  const [form, setForm] = useState<ProjectInput>(emptyProject);
  const [status, setStatus] = useState("Loading projects");

  const selectedProject = useMemo(
    () => projects.find((project) => project.id === selectedId) ?? null,
    [projects, selectedId]
  );

  useEffect(() => {
    listProjects()
      .then((nextProjects) => {
        setProjects(nextProjects);
        setStatus(
          nextProjects.length === 0 ? "No projects yet" : `${nextProjects.length} project(s)`
        );
      })
      .catch((error) => setStatus(formatError(error)));
  }, []);

  function selectProject(project: Project) {
    setSelectedId(project.id);
    setProjectMode("view");
    setForm({
      name: project.name,
      description: project.description,
      status: project.status
    });
  }

  function newProject() {
    setSelectedId(null);
    setProjectMode("create");
    setForm(emptyProject);
    setStatus("New project");
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
        setProjects((current) =>
          current.map((project) => (project.id === updated.id ? updated : project))
        );
        selectProject(updated);
        setStatus("Project saved");
      } else {
        const created = await createProject({
          name,
          description: form.description,
          status: form.status
        });
        setProjects((current) => [created, ...current]);
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

    try {
      await deleteProject(selectedProject.id);
      setProjects((current) => current.filter((project) => project.id !== selectedProject.id));
      setSelectedId(null);
      setProjectMode("idle");
      setForm(emptyProject);
      setStatus("Project deleted");
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  function updateField<K extends keyof ProjectInput>(field: K, value: ProjectInput[K]) {
    setForm((current) => ({ ...current, [field]: value }));
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

      <div className="split-feature-body">
        <aside className="sub-list" aria-label="Projects list">
          <button className="primary-button full-width" onClick={newProject} type="button">
            New Project
          </button>
          {projects.map((project) => (
            <button
              className={project.id === selectedId ? "sub-list-item active" : "sub-list-item"}
              key={project.id}
              onClick={() => selectProject(project)}
              type="button"
            >
              <strong>{project.name}</strong>
              <span>{formatProjectStatus(project.status)}</span>
            </button>
          ))}
        </aside>

        {selectedProject || projectMode === "create" ? (
          <form className="editor-form">
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
                <button className="primary-button" onClick={() => void onSaveProject()} type="button">
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
