import { useEffect, useMemo, useState } from "react";
import { ComponentHost } from "../components/ComponentHost";
import { WindowResizeHandles, WindowTitlebar } from "../components/WindowControls";
import { Calendar } from "../features/calendar/Calendar";
import { Gaming } from "../features/gaming/Gaming";
import { Notes } from "../features/notes/Notes";
import { Projects } from "../features/projects/Projects";
import { Scratchpad } from "../features/scratchpad/Scratchpad";
import { Settings } from "../features/settings/Settings";
import { Tasks } from "../features/tasks/Tasks";
import { YouTube } from "../features/youtube/YouTube";
import { getMilestoneStatus } from "../services/appStatus";
import type { MilestoneStatus } from "../services/appStatus";
import { deleteGame, listGameChatConversations, listGames } from "../services/gaming";
import type { Game, GameChatConversation } from "../services/gaming";
import { deletePlanningConversation, listPlanningConversations } from "../services/planningChat";
import type { PlanningConversation } from "../services/planningChat";
import { deleteProject, listProjects } from "../services/projects";
import type { Project } from "../services/projects";
import { setOverlayMinimumSize } from "../services/windowControls";

type ComponentId =
  | "scratchpad"
  | "tasks"
  | "notes"
  | "calendar"
  | "gaming"
  | "projects"
  | "youtube"
  | "settings";

const navItems = [
  { id: "scratchpad", label: "Scratchpad" },
  { id: "tasks", label: "Tasks" },
  { id: "notes", label: "Notes" },
  { id: "calendar", label: "Calendar" },
  { id: "gaming", label: "Gaming" },
  { id: "projects", label: "Projects" },
  { id: "youtube", label: "YouTube" },
  { id: "settings", label: "Settings" }
] satisfies Array<{ id: ComponentId; label: string }>;

type ProjectNavAction = {
  type: "create" | "overview" | "newChat" | "chat" | "references" | "edit";
  projectId?: number;
  conversationId?: number;
  nonce: number;
};

type GameNavAction = {
  type: "home" | "newChat" | "chat" | "screenshots" | "parts";
  gameId?: number;
  conversationId?: number;
  nonce: number;
};

export default function App() {
  const [status, setStatus] = useState<MilestoneStatus | null>(null);
  const [activeComponent, setActiveComponent] = useState<ComponentId>("scratchpad");
  const [projects, setProjects] = useState<Project[]>([]);
  const [selectedProjectId, setSelectedProjectId] = useState<number | null>(null);
  const [projectsExpanded, setProjectsExpanded] = useState(true);
  const [gamingExpanded, setGamingExpanded] = useState(true);
  const [gameSections, setGameSections] = useState<Game[]>([]);
  const [selectedGameId, setSelectedGameId] = useState<number | null>(null);
  const [gameMenuId, setGameMenuId] = useState<number | null>(null);
  const [projectMenuId, setProjectMenuId] = useState<number | null>(null);
  const [conversationMenuId, setConversationMenuId] = useState<number | null>(null);
  const [projectNavAction, setProjectNavAction] = useState<ProjectNavAction | null>(null);
  const [gameNavAction, setGameNavAction] = useState<GameNavAction | null>(null);
  const [isChatOverlayMode, setIsChatOverlayMode] = useState(false);
  const [conversationsByProject, setConversationsByProject] = useState<
    Record<number, PlanningConversation[]>
  >({});
  const [conversationsByGame, setConversationsByGame] = useState<
    Record<number, GameChatConversation[]>
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

  useEffect(() => {
    listGames()
      .then((nextGames) => {
        setGameSections(nextGames);
        setSelectedGameId((current) =>
          current && nextGames.some((game) => game.id === current)
            ? current
            : nextGames[0]?.id ?? null
        );
        void refreshGameConversations(nextGames);
      })
      .catch(() => {
        setGameSections([]);
        setSelectedGameId(null);
        setConversationsByGame({});
      });
  }, []);

  useEffect(() => {
    void setOverlayMinimumSize(
      isChatOverlayMode ? 320 : 760,
      isChatOverlayMode ? 210 : 480
    );
  }, [isChatOverlayMode]);

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

  async function refreshGameConversations(nextGames = gameSections) {
    const entries = await Promise.all(
      nextGames.map(async (game) => {
        try {
          const conversations = await listGameChatConversations(game.id);
          return [game.id, conversations] as const;
        } catch {
          return [game.id, []] as const;
        }
      })
    );
    setConversationsByGame(Object.fromEntries(entries));
  }

  function openProjects() {
    setIsChatOverlayMode(false);
    setActiveComponent("projects");
  }

  function openGaming() {
    setIsChatOverlayMode(false);
    setActiveComponent("gaming");
  }

  function selectGameSection(game: Game) {
    openGaming();
    setGamingExpanded(true);
    setGameMenuId(null);
    setSelectedGameId(game.id);
    setGameNavAction({ type: "home", gameId: game.id, nonce: Date.now() });
  }

  function openGamingCreate() {
    openGaming();
    setGamingExpanded(true);
    setGameMenuId(null);
    setSelectedGameId(null);
    setGameNavAction(null);
  }

  function onGameCreated(game: Game) {
    setGameSections((current) => [game, ...current]);
    setSelectedGameId(game.id);
    setGamingExpanded(true);
    setConversationsByGame((current) => ({ ...current, [game.id]: [] }));
    setGameNavAction({ type: "home", gameId: game.id, nonce: Date.now() });
  }

  function onGameDeleted(gameId: number) {
    setGameSections((current) => {
      const nextGameSections = current.filter((game) => game.id !== gameId);
      setSelectedGameId((selected) =>
        selected === gameId ? nextGameSections[0]?.id ?? null : selected
      );
      return nextGameSections;
    });
    setGameMenuId(null);
    setConversationsByGame((current) => {
      const next = { ...current };
      delete next[gameId];
      return next;
    });
    setGameNavAction((current) => (current?.gameId === gameId ? null : current));
  }

  function openGameAction(game: Game, type: Exclude<GameNavAction["type"], "chat">) {
    openGaming();
    setGamingExpanded(true);
    setGameMenuId(null);
    setSelectedGameId(game.id);
    setGameNavAction({ type, gameId: game.id, nonce: Date.now() });
  }

  function selectGameConversation(game: Game, conversation: GameChatConversation) {
    openGaming();
    setGamingExpanded(true);
    setGameMenuId(null);
    setSelectedGameId(game.id);
    setGameNavAction({
      type: "chat",
      gameId: game.id,
      conversationId: conversation.id,
      nonce: Date.now()
    });
  }

  function onGameChatConversationsChanged(gameId: number, conversations: GameChatConversation[]) {
    setConversationsByGame((current) => ({ ...current, [gameId]: conversations }));
  }

  async function removeGame(game: Game) {
    const confirmed = window.confirm(
      `Delete "${game.name}"? This removes the local game section and its catalog records.`
    );
    if (!confirmed) {
      return;
    }

    try {
      await deleteGame(game.id);
      onGameDeleted(game.id);
    } catch (error) {
      window.alert(error instanceof Error ? error.message : String(error));
    }
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
    <main className={isChatOverlayMode ? "overlay-frame overlay-frame-chat-mode" : "overlay-frame"}>
      {!isChatOverlayMode && <WindowResizeHandles />}
      {!isChatOverlayMode && <WindowTitlebar />}

      <div className="overlay-shell">
        {!isChatOverlayMode && (
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
              <div
                className={
                  item.id === "projects" || item.id === "gaming" ? "nav-tree-group" : undefined
                }
                key={item.id}
              >
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
                ) : item.id === "gaming" ? (
                  <>
                    <div
                      className={
                        item.id === activeComponent
                          ? "nav-item nav-item-active nav-tree-parent"
                          : "nav-item nav-tree-parent"
                      }
                    >
                      <button
                        aria-label={gamingExpanded ? "Collapse Gaming" : "Expand Gaming"}
                        className="nav-icon-button"
                        onClick={() => setGamingExpanded((current) => !current)}
                        type="button"
                      >
                        {gamingExpanded ? "v" : ">"}
                      </button>
                      <button className="nav-label-button" onClick={openGaming} type="button">
                        {item.label}
                      </button>
                      <button
                        aria-label="Open game section creator"
                        className="nav-icon-button"
                        onClick={openGamingCreate}
                        type="button"
                      >
                        +
                      </button>
                    </div>

                    {gamingExpanded && (
                      <div className="nav-tree-children" aria-label="Game sections">
                        {gameSections.map((game) => (
                          <div className="nav-project-branch" key={game.id}>
                            <div
                              className={
                                game.id === selectedGameId && activeComponent === "gaming"
                                  ? "nav-child-row nav-child-row-active"
                                  : "nav-child-row"
                              }
                            >
                              <button
                                className="nav-child-label"
                                onClick={() => selectGameSection(game)}
                                title={game.name}
                                type="button"
                              >
                                {game.name}
                              </button>
                              <button
                                aria-label={`Game actions for ${game.name}`}
                                className="nav-icon-button"
                                onClick={() =>
                                  setGameMenuId((current) => (current === game.id ? null : game.id))
                                }
                                type="button"
                              >
                                ...
                              </button>
                              {gameMenuId === game.id && (
                                <div className="nav-project-menu">
                                  <button
                                    onClick={() => openGameAction(game, "home")}
                                    type="button"
                                  >
                                    Home
                                  </button>
                                  <button
                                    onClick={() => openGameAction(game, "newChat")}
                                    type="button"
                                  >
                                    New Chat
                                  </button>
                                  <button
                                    onClick={() => openGameAction(game, "screenshots")}
                                    type="button"
                                  >
                                    Screenshots
                                  </button>
                                  <button
                                    onClick={() => openGameAction(game, "parts")}
                                    type="button"
                                  >
                                    Parts
                                  </button>
                                  <button onClick={() => void removeGame(game)} type="button">
                                    Delete
                                  </button>
                                </div>
                              )}
                            </div>

                            {(conversationsByGame[game.id] ?? []).map((conversation) => (
                              <div
                                className={
                                  game.id === selectedGameId &&
                                  gameNavAction?.type === "chat" &&
                                  gameNavAction.conversationId === conversation.id
                                    ? "nav-conversation-row nav-conversation-row-active"
                                    : "nav-conversation-row"
                                }
                                key={conversation.id}
                                title={conversation.title}
                              >
                                <button
                                  className="nav-conversation-label"
                                  onClick={() => selectGameConversation(game, conversation)}
                                  type="button"
                                >
                                  <span className="nav-chat-icon">chat</span>
                                  <span>{conversation.title}</span>
                                </button>
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
                    onClick={() => {
                      setIsChatOverlayMode(false);
                      setActiveComponent(item.id);
                    }}
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
        )}

        <section
          className={
            activeComponent === "projects"
              ? isChatOverlayMode
                ? "workspace workspace-projects workspace-chat-overlay-mode"
                : "workspace workspace-projects"
              : activeComponent === "gaming" && isChatOverlayMode
                ? "workspace workspace-chat-overlay-mode"
              : "workspace"
          }
          aria-label="Active component"
        >
          {activeComponent !== "projects" && activeComponent !== "gaming" && (
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
            {activeComponent === "gaming" && (
              <Gaming
                chatOverlayMode={isChatOverlayMode}
                gameSections={gameSections}
                navAction={gameNavAction}
                onEnterChatOverlayMode={() => setIsChatOverlayMode(true)}
                onGameCreated={onGameCreated}
                onGameChatConversationsChanged={onGameChatConversationsChanged}
                onGameDeleted={onGameDeleted}
                onExitChatOverlayMode={() => setIsChatOverlayMode(false)}
                onSelectGame={setSelectedGameId}
                selectedGameId={selectedGameId}
              />
            )}
            {activeComponent === "projects" && (
              <Projects
                chatOverlayMode={isChatOverlayMode}
                navAction={projectNavAction}
                onEnterChatOverlayMode={() => setIsChatOverlayMode(true)}
                onExitChatOverlayMode={() => setIsChatOverlayMode(false)}
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
          {activeComponent === "settings" && <Settings />}
        </ComponentHost>
        </section>
      </div>
    </main>
  );
}
