import { useEffect, useMemo, useState } from "react";
import { ComponentHost } from "../components/ComponentHost";
import { WindowResizeHandles, WindowTitlebar } from "../components/WindowControls";
import { Calendar } from "../features/calendar/Calendar";
import { Notes } from "../features/notes/Notes";
import { Projects } from "../features/projects/Projects";
import { Scratchpad } from "../features/scratchpad/Scratchpad";
import { Tasks } from "../features/tasks/Tasks";
import { YouTube } from "../features/youtube/YouTube";
import { getMilestoneStatus } from "../services/appStatus";
import type { MilestoneStatus } from "../services/appStatus";
import { deleteProject, listProjects } from "../services/projects";
import type { Project } from "../services/projects";

type ComponentId =
  | "scratchpad"
  | "tasks"
  | "notes"
  | "calendar"
  | "projects"
  | "youtube";

const navItems = [
  { id: "scratchpad", label: "Scratchpad" },
  { id: "tasks", label: "Tasks" },
  { id: "notes", label: "Notes" },
  { id: "calendar", label: "Calendar" },
  { id: "projects", label: "Projects" },
  { id: "youtube", label: "YouTube" }
] satisfies Array<{ id: ComponentId; label: string }>;

type ProjectNavAction = {
  type: "create" | "edit";
  projectId?: number;
  nonce: number;
};

export default function App() {
  const [status, setStatus] = useState<MilestoneStatus | null>(null);
  const [activeComponent, setActiveComponent] = useState<ComponentId>("scratchpad");
  const [projects, setProjects] = useState<Project[]>([]);
  const [selectedProjectId, setSelectedProjectId] = useState<number | null>(null);
  const [projectsExpanded, setProjectsExpanded] = useState(true);
  const [projectMenuId, setProjectMenuId] = useState<number | null>(null);
  const [projectNavAction, setProjectNavAction] = useState<ProjectNavAction | null>(null);

  useEffect(() => {
    getMilestoneStatus()
      .then(setStatus)
      .catch(() => {
        setStatus({
          milestone: "Milestone 11",
          hotkey: "Ctrl+Shift+Space",
          databaseReady: false
        });
      });
  }, []);

  useEffect(() => {
    listProjects()
      .then((nextProjects) => {
        setProjects(nextProjects);
        setSelectedProjectId((current) =>
          current && nextProjects.some((project) => project.id === current) ? current : null
        );
      })
      .catch(() => {
        setProjects([]);
        setSelectedProjectId(null);
      });
  }, []);

  const activeMeta = useMemo(
    () => ({
      title: navItems.find((item) => item.id === activeComponent)?.label ?? "Scratchpad",
      eyebrow: status?.milestone ?? "Milestone 11",
      hotkey: status?.hotkey ?? "Ctrl+Shift+Space"
    }),
    [activeComponent, status]
  );

  function openProjects() {
    setActiveComponent("projects");
  }

  function startProjectCreate() {
    openProjects();
    setProjectsExpanded(true);
    setProjectMenuId(null);
    setProjectNavAction({ type: "create", nonce: Date.now() });
  }

  function selectProject(project: Project) {
    openProjects();
    setProjectsExpanded(true);
    setProjectMenuId(null);
    setSelectedProjectId(project.id);
  }

  function startProjectEdit(project: Project) {
    selectProject(project);
    setProjectNavAction({ type: "edit", projectId: project.id, nonce: Date.now() });
  }

  async function removeProject(project: Project) {
    const confirmed = window.confirm(
      `Delete "${project.name}"? This removes the local project only.`
    );
    if (!confirmed) {
      return;
    }

    try {
      await deleteProject(project.id);
      setProjects((current) => current.filter((item) => item.id !== project.id));
      setProjectMenuId(null);
      setSelectedProjectId((current) => (current === project.id ? null : current));
    } catch (error) {
      window.alert(error instanceof Error ? error.message : String(error));
    }
  }

  return (
    <main className="overlay-frame">
      <WindowResizeHandles />
      <WindowTitlebar />

      <div className="overlay-shell">
        <aside className="sidebar">
          <div className="brand-block">
            <span className="brand-mark">OF</span>
            <div>
              <h1>Overlay Forge</h1>
              <p>Desktop command hub</p>
            </div>
          </div>

          <nav className="component-nav" aria-label="Components">
            {navItems.map((item) => (
              <div className={item.id === "projects" ? "nav-tree-group" : undefined} key={item.id}>
                {item.id === "projects" ? (
                  <>
                    <div
                      className={
                        item.id === activeComponent
                          ? "nav-item nav-item-active nav-tree-parent"
                          : "nav-item nav-tree-parent"
                      }
                    >
                      <button
                        aria-label={projectsExpanded ? "Collapse Projects" : "Expand Projects"}
                        className="nav-icon-button"
                        onClick={() => setProjectsExpanded((current) => !current)}
                        type="button"
                      >
                        {projectsExpanded ? "v" : ">"}
                      </button>
                      <button className="nav-label-button" onClick={openProjects} type="button">
                        {item.label}
                      </button>
                      <button
                        aria-label="Create project"
                        className="nav-icon-button"
                        onClick={startProjectCreate}
                        type="button"
                      >
                        +
                      </button>
                    </div>

                    {projectsExpanded && (
                      <div className="nav-tree-children" aria-label="Saved projects">
                        {projects.map((project) => (
                          <div
                            className={
                              project.id === selectedProjectId
                                ? "nav-child-row nav-child-row-active"
                                : "nav-child-row"
                            }
                            key={project.id}
                          >
                            <button
                              className="nav-child-label"
                              onClick={() => selectProject(project)}
                              title={project.name}
                              type="button"
                            >
                              {project.name}
                            </button>
                            <button
                              aria-label={`Project actions for ${project.name}`}
                              className="nav-icon-button"
                              onClick={() =>
                                setProjectMenuId((current) =>
                                  current === project.id ? null : project.id
                                )
                              }
                              type="button"
                            >
                              ...
                            </button>
                            {projectMenuId === project.id && (
                              <div className="nav-project-menu">
                                <button onClick={() => startProjectEdit(project)} type="button">
                                  Edit
                                </button>
                                <button onClick={() => void removeProject(project)} type="button">
                                  Delete
                                </button>
                              </div>
                            )}
                          </div>
                        ))}
                      </div>
                    )}
                  </>
                ) : (
                  <button
                    className={item.id === activeComponent ? "nav-item nav-item-active" : "nav-item"}
                    onClick={() => setActiveComponent(item.id)}
                    type="button"
                  >
                    {item.label}
                  </button>
                )}
              </div>
            ))}
          </nav>

          <div className="shell-status">
            <span className={status?.databaseReady ? "status-dot ready" : "status-dot"} />
            <span>{status?.databaseReady ? "SQLite ready" : "SQLite pending"}</span>
          </div>
        </aside>

        <section className="workspace" aria-label="Active component">
          <header className="workspace-header">
            <div>
              <p>{activeMeta.eyebrow}</p>
              <h2>{activeMeta.title}</h2>
            </div>
            <kbd>{activeMeta.hotkey}</kbd>
          </header>

          <ComponentHost>
            {activeComponent === "scratchpad" && <Scratchpad />}
            {activeComponent === "tasks" && <Tasks />}
            {activeComponent === "notes" && <Notes />}
            {activeComponent === "calendar" && <Calendar />}
            {activeComponent === "projects" && (
              <Projects
                navAction={projectNavAction}
                onProjectCreated={(project) => {
                  setProjects((current) => [project, ...current]);
                  setSelectedProjectId(project.id);
                  setProjectsExpanded(true);
                }}
                onProjectDeleted={(projectId) => {
                  setProjects((current) => current.filter((project) => project.id !== projectId));
                  setSelectedProjectId((current) => (current === projectId ? null : current));
                }}
                onProjectUpdated={(project) => {
                  setProjects((current) =>
                    current.map((item) => (item.id === project.id ? project : item))
                  );
                  setSelectedProjectId(project.id);
                }}
                onSelectProject={setSelectedProjectId}
                projects={projects}
                selectedProjectId={selectedProjectId}
              />
            )}
            {activeComponent === "youtube" && <YouTube />}
          </ComponentHost>
        </section>
      </div>
    </main>
  );
}
