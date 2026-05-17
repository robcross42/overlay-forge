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
import { deletePlanningConversation, listPlanningConversations } from "../services/planningChat";
import type { PlanningConversation } from "../services/planningChat";
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
  type: "create" | "overview" | "newChat" | "chat" | "references" | "edit";
  projectId?: number;
  conversationId?: number;
  nonce: number;
};

export default function App() {
  const [status, setStatus] = useState<MilestoneStatus | null>(null);
  const [activeComponent, setActiveComponent] = useState<ComponentId>("scratchpad");
  const [projects, setProjects] = useState<Project[]>([]);
  const [selectedProjectId, setSelectedProjectId] = useState<number | null>(null);
  const [projectsExpanded, setProjectsExpanded] = useState(true);
  const [projectMenuId, setProjectMenuId] = useState<number | null>(null);
  const [conversationMenuId, setConversationMenuId] = useState<number | null>(null);
  const [projectNavAction, setProjectNavAction] = useState<ProjectNavAction | null>(null);
  const [conversationsByProject, setConversationsByProject] = useState<
    Record<number, PlanningConversation[]>
  >({});

  useEffect(() => {
    getMilestoneStatus()
      .then(setStatus)
      .catch(() => {
        setStatus({
          milestone: "Milestone 13",
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
        void refreshProjectConversations(nextProjects);
      })
      .catch(() => {
        setProjects([]);
        setSelectedProjectId(null);
      });
  }, []);

  const activeMeta = useMemo(
    () => ({
      title: navItems.find((item) => item.id === activeComponent)?.label ?? "Scratchpad",
      eyebrow: status?.milestone ?? "Milestone 13",
      hotkey: status?.hotkey ?? "Ctrl+Shift+Space"
    }),
    [activeComponent, status]
  );

  async function refreshProjectConversations(nextProjects = projects) {
    const entries = await Promise.all(
      nextProjects.map(async (project) => {
        try {
          const conversations = await listPlanningConversations(project.id);
          return [project.id, conversations] as const;
        } catch {
          return [project.id, []] as const;
        }
      })
    );
    setConversationsByProject(Object.fromEntries(entries));
  }

  function openProjects() {
    setActiveComponent("projects");
  }

  function startProjectCreate() {
    openProjects();
    setProjectsExpanded(true);
    setProjectMenuId(null);
    setConversationMenuId(null);
    setProjectNavAction({ type: "create", nonce: Date.now() });
  }

  function selectProject(project: Project) {
    openProjects();
    setProjectsExpanded(true);
    setProjectMenuId(null);
    setConversationMenuId(null);
    setSelectedProjectId(project.id);
    setProjectNavAction({ type: "overview", projectId: project.id, nonce: Date.now() });
  }

  function startProjectEdit(project: Project) {
    openProjectAction(project, "edit");
  }

  function openProjectAction(
    project: Project,
    type: Exclude<ProjectNavAction["type"], "create">,
    conversationId?: number
  ) {
    openProjects();
    setProjectsExpanded(true);
    setProjectMenuId(null);
    setConversationMenuId(null);
    setSelectedProjectId(project.id);
    setProjectNavAction({ type, projectId: project.id, conversationId, nonce: Date.now() });
  }

  function selectConversation(project: Project, conversation: PlanningConversation) {
    openProjectAction(project, "chat", conversation.id);
  }

  function onProjectConversationsChanged(projectId: number, conversations: PlanningConversation[]) {
    setConversationsByProject((current) => ({ ...current, [projectId]: conversations }));
  }

  function onProjectActionMenu(project: Project, type: Exclude<ProjectNavAction["type"], "create">) {
    openProjectAction(project, type);
    setProjectMenuId(null);
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

  async function removeConversation(project: Project, conversation: PlanningConversation) {
    const confirmed = window.confirm(
      `Delete chat "${conversation.title}"? This removes the local conversation only.`
    );
    if (!confirmed) {
      return;
    }

    try {
      await deletePlanningConversation(conversation.id);
      setConversationMenuId(null);
      setConversationsByProject((current) => ({
        ...current,
        [project.id]: (current[project.id] ?? []).filter((item) => item.id !== conversation.id)
      }));

      if (
        selectedProjectId === project.id &&
        projectNavAction?.type === "chat" &&
        projectNavAction.conversationId === conversation.id
      ) {
        setProjectNavAction({ type: "newChat", projectId: project.id, nonce: Date.now() });
      }
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
                          <div className="nav-project-branch" key={project.id}>
                            <div
                              className={
                                project.id === selectedProjectId
                                  ? "nav-child-row nav-child-row-active"
                                  : "nav-child-row"
                              }
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
                                  <button
                                    onClick={() => onProjectActionMenu(project, "overview")}
                                    type="button"
                                  >
                                    Overview
                                  </button>
                                  <button
                                    onClick={() => onProjectActionMenu(project, "newChat")}
                                    type="button"
                                  >
                                    New Chat
                                  </button>
                                  <button
                                    onClick={() => onProjectActionMenu(project, "references")}
                                    type="button"
                                  >
                                    References
                                  </button>
                                  <button onClick={() => startProjectEdit(project)} type="button">
                                    Edit
                                  </button>
                                  <button onClick={() => void removeProject(project)} type="button">
                                    Delete
                                  </button>
                                </div>
                              )}
                            </div>
                            {(conversationsByProject[project.id] ?? []).map((conversation) => (
                              <div
                                className={
                                  project.id === selectedProjectId &&
                                  projectNavAction?.type === "chat" &&
                                  projectNavAction.conversationId === conversation.id
                                    ? "nav-conversation-row nav-conversation-row-active"
                                    : "nav-conversation-row"
                                }
                                key={conversation.id}
                                title={conversation.title}
                              >
                                <button
                                  className="nav-conversation-label"
                                  onClick={() => selectConversation(project, conversation)}
                                  type="button"
                                >
                                  <span className="nav-chat-icon">chat</span>
                                  <span>{conversation.title}</span>
                                </button>
                                <button
                                  aria-label={`Chat actions for ${conversation.title}`}
                                  className="nav-icon-button"
                                  onClick={() =>
                                    setConversationMenuId((current) =>
                                      current === conversation.id ? null : conversation.id
                                    )
                                  }
                                  type="button"
                                >
                                  ...
                                </button>
                                {conversationMenuId === conversation.id && (
                                  <div className="nav-project-menu nav-conversation-menu">
                                    <button
                                      onClick={() => void removeConversation(project, conversation)}
                                      type="button"
                                    >
                                      Delete
                                    </button>
                                  </div>
                                )}
                              </div>
                            ))}
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

        <section
          className={activeComponent === "projects" ? "workspace workspace-projects" : "workspace"}
          aria-label="Active component"
        >
          {activeComponent !== "projects" && (
            <header className="workspace-header">
              <div>
                <p>{activeMeta.eyebrow}</p>
                <h2>{activeMeta.title}</h2>
              </div>
              <kbd>{activeMeta.hotkey}</kbd>
            </header>
          )}

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
                  setConversationsByProject((current) => ({ ...current, [project.id]: [] }));
                }}
                onProjectDeleted={(projectId) => {
                  setProjects((current) => current.filter((project) => project.id !== projectId));
                  setSelectedProjectId((current) => (current === projectId ? null : current));
                  setConversationsByProject((current) => {
                    const next = { ...current };
                    delete next[projectId];
                    return next;
                  });
                }}
                onProjectUpdated={(project) => {
                  setProjects((current) =>
                    current.map((item) => (item.id === project.id ? project : item))
                  );
                  setSelectedProjectId(project.id);
                }}
                onSelectProject={setSelectedProjectId}
                onProjectConversationsChanged={onProjectConversationsChanged}
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
