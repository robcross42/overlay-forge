import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useMemo, useState } from "react";
import { ComponentHost } from "../components/ComponentHost";
import { WindowResizeHandles, WindowTitlebar } from "../components/WindowControls";
import { Calendar } from "../features/calendar/Calendar";
import { Cessation } from "../features/cessation/Cessation";
import { Gaming } from "../features/gaming/Gaming";
import { RepairResell } from "../features/repair-resell/RepairResell";
import { Settings } from "../features/settings/Settings";
import { YouTube } from "../features/youtube/YouTube";
import { getAppStatus } from "../services/appStatus";
import type { AppStatus } from "../services/appStatus";
import {
  deleteGame,
  getActiveGameChatOverlay,
  listGameChatConversations,
  listGames,
  openGameChatOverlayWindow,
  toggleGameChatOverlayWindow
} from "../services/gaming";
import type { Game, GameChatConversation } from "../services/gaming";
import {
  hideOverlayWindow,
  setOverlayMinimumSize,
  setOverlayWindowOpacity,
  showOverlayWindow
} from "../services/windowControls";

type ComponentId =
  | "calendar"
  | "cessation"
  | "repair-resell"
  | "gaming"
  | "youtube"
  | "settings";

const navItems = [
  { id: "calendar", label: "Calendar" },
  { id: "cessation", label: "Cessation" },
  { id: "repair-resell", label: "Repair Resell" },
  { id: "gaming", label: "Gaming" },
  { id: "youtube", label: "YouTube" },
  { id: "settings", label: "Settings" }
] satisfies Array<{ id: ComponentId; label: string }>;

type GameNavAction = {
  type: "home" | "newChat" | "chat" | "screenshots" | "parts" | "build-guides";
  gameId?: number;
  conversationId?: number;
  nonce: number;
};

export default function App() {
  const [status, setStatus] = useState<AppStatus | null>(null);
  const [activeComponent, setActiveComponent] = useState<ComponentId>("calendar");
  const [gamingExpanded, setGamingExpanded] = useState(true);
  const [gameSections, setGameSections] = useState<Game[]>([]);
  const [selectedGameId, setSelectedGameId] = useState<number | null>(null);
  const [gameMenuId, setGameMenuId] = useState<number | null>(null);
  const [gameNavAction, setGameNavAction] = useState<GameNavAction | null>(null);
  const [isChatOverlayMode, setIsChatOverlayMode] = useState(false);
  const [isSidebarMinimized, setIsSidebarMinimized] = useState(true);
  const [conversationsByGame, setConversationsByGame] = useState<
    Record<number, GameChatConversation[]>
  >({});

  useEffect(() => {
    getAppStatus()
      .then(setStatus)
      .catch(() => {
        setStatus({
          hotkey: "Ctrl+Shift+Space",
          databaseReady: false
        });
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
    void setOverlayWindowOpacity(isChatOverlayMode ? 0.78 : 1);
  }, [isChatOverlayMode]);

  useEffect(() => {
    let isMounted = true;
    let unlisten: (() => void) | null = null;
    let unlistenOverlayToggle: (() => void) | null = null;
    let unlistenBuildGuideOverlay: (() => void) | null = null;

    function consumePendingShortcut() {
      invoke<string | null>("consume_pending_shortcut_action")
        .then((action) => {
          if (!isMounted || !action) {
            return;
          }

          if (action === "toggle_overlay") {
            void handleMainOverlayShortcut();
          } else if (action === "toggle_overlay_was_visible") {
            void handleMainOverlayShortcut(true);
          } else if (action === "toggle_overlay_was_hidden") {
            void handleMainOverlayShortcut(false);
          } else if (action === "game_chat_overlay") {
            void handleGameChatOverlayShortcut();
          } else if (action === "game_chat_overlay_was_hidden") {
            void handleGameChatOverlayShortcut();
          } else if (action === "game_chat_overlay_focus_chat") {
            void handleGameChatOverlayShortcut();
          } else if (action === "game_chat_overlay_focus_game") {
            void handleGameChatOverlayShortcut();
          } else if (action === "game_build_guide_overlay") {
            void handleGameBuildGuideOverlayShortcut();
          }
        })
        .catch(() => {});
    }

    listen("game-chat-overlay-requested", () => {
      if (!isMounted) {
        return;
      }

      consumePendingShortcut();
    })
      .then((cleanup) => {
        if (isMounted) {
          unlisten = cleanup;
        } else {
          cleanup();
        }
      })
      .catch(() => {});

    listen("overlay-toggle-requested", () => {
      if (!isMounted) {
        return;
      }

      consumePendingShortcut();
    })
      .then((cleanup) => {
        if (isMounted) {
          unlistenOverlayToggle = cleanup;
        } else {
          cleanup();
        }
      })
      .catch(() => {});

    listen("game-build-guide-overlay-requested", () => {
      if (!isMounted) {
        return;
      }

      consumePendingShortcut();
    })
      .then((cleanup) => {
        if (isMounted) {
          unlistenBuildGuideOverlay = cleanup;
        } else {
          cleanup();
        }
      })
      .catch(() => {});

    window.addEventListener("focus", consumePendingShortcut);
    consumePendingShortcut();

    return () => {
      isMounted = false;
      unlisten?.();
      unlistenOverlayToggle?.();
      unlistenBuildGuideOverlay?.();
      window.removeEventListener("focus", consumePendingShortcut);
    };
  }, [activeComponent, gameNavAction, gameSections, isChatOverlayMode, selectedGameId]);

  const activeMeta = useMemo(
    () => ({
      title: navItems.find((item) => item.id === activeComponent)?.label ?? "Calendar",
      eyebrow: "Local-first command hub"
    }),
    [activeComponent]
  );

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

  function openGaming() {
    setIsChatOverlayMode(false);
    setActiveComponent("gaming");
  }

  async function hideChatOverlayWindow() {
    await hideOverlayWindow();
  }

  async function closeChatOverlayWindow() {
    try {
      await hideOverlayWindow();
    } finally {
      setIsChatOverlayMode(false);
    }
  }

  async function handleMainOverlayShortcut(preShortcutWasVisible?: boolean) {
    if (preShortcutWasVisible === true && !isChatOverlayMode) {
      await hideOverlayWindow().catch(() => {});
      return;
    }

    setIsChatOverlayMode(false);
    if (preShortcutWasVisible !== true) {
      await showOverlayWindow().catch(() => {});
    }
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

  async function handleGameChatOverlayShortcut() {
    const activeOverlay = await getActiveGameChatOverlay().catch(() => null);
    if (activeOverlay) {
      await toggleGameChatOverlayWindow().catch(() => {});
      return;
    }

    const selectedGame =
      gameSections.find((game) => game.id === selectedGameId) ?? gameSections[0] ?? null;
    if (!selectedGame) {
      openGaming();
      await showOverlayWindow().catch(() => {});
      return;
    }

    const selectedConversationId =
      activeComponent === "gaming" &&
      gameNavAction?.type === "chat" &&
      gameNavAction.gameId === selectedGame.id
        ? gameNavAction.conversationId
        : null;

    openGaming();
    setGamingExpanded(true);
    setGameMenuId(null);
    setSelectedGameId(selectedGame.id);

    if (selectedConversationId) {
      await openGameChatOverlayWindow(selectedGame.id, selectedConversationId).catch(() => {});
      return;
    }

    await showOverlayWindow().catch(() => {});
    setGameNavAction({
      type: "newChat",
      gameId: selectedGame.id,
      nonce: Date.now()
    });
  }

  async function handleGameBuildGuideOverlayShortcut() {
    const selectedGame =
      gameSections.find((game) => game.slug === "gearblocks") ??
      gameSections.find((game) => game.id === selectedGameId) ??
      gameSections[0] ??
      null;
    if (!selectedGame) {
      openGaming();
      await showOverlayWindow().catch(() => {});
      return;
    }

    openGaming();
    setGamingExpanded(true);
    setGameMenuId(null);
    setSelectedGameId(selectedGame.id);
    await showOverlayWindow().catch(() => {});
    setGameNavAction({
      type: "build-guides",
      gameId: selectedGame.id,
      nonce: Date.now()
    });
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

  return (
    <main className={isChatOverlayMode ? "overlay-frame overlay-frame-chat-mode" : "overlay-frame"}>
      {!isChatOverlayMode && <WindowResizeHandles />}
      {!isChatOverlayMode && <WindowTitlebar />}

      <div className={isSidebarMinimized ? "overlay-shell overlay-shell-sidebar-minimized" : "overlay-shell"}>
        {!isChatOverlayMode && (
        <aside className={isSidebarMinimized ? "sidebar sidebar-minimized" : "sidebar"}>
          {isSidebarMinimized ? (
            <>
              <button
                aria-label="Restore navigation"
                className="sidebar-toggle sidebar-toggle-collapsed"
                onClick={() => setIsSidebarMinimized(false)}
                title="Restore navigation"
                type="button"
              >
                &gt;
              </button>
              <nav className="component-nav component-nav-compact" aria-label="Components">
                {navItems.map((item) => (
                  <button
                    aria-label={item.label}
                    className={
                      item.id === activeComponent
                        ? "nav-item nav-item-active nav-item-compact"
                        : "nav-item nav-item-compact"
                    }
                    key={item.id}
                    onClick={() => {
                      setIsChatOverlayMode(false);
                      if (item.id === "gaming") {
                        openGaming();
                      } else {
                        setActiveComponent(item.id);
                      }
                    }}
                    title={item.label}
                    type="button"
                  >
                    {compactNavLabel(item.label)}
                  </button>
                ))}
              </nav>
            </>
          ) : (
          <>
          <div className="brand-block">
            <span className="brand-mark">OF</span>
            <div>
              <h1>Overlay Forge</h1>
              <p>Desktop command hub</p>
            </div>
            <button
              aria-label="Minimize navigation"
              className="sidebar-toggle"
              onClick={() => setIsSidebarMinimized(true)}
              title="Minimize navigation"
              type="button"
            >
              &lt;
            </button>
          </div>

          <nav className="component-nav" aria-label="Components">
            {navItems.map((item) => (
              <div
                className={item.id === "gaming" ? "nav-tree-group" : undefined}
                key={item.id}
              >
                {item.id === "gaming" ? (
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
                          <div className="nav-tree-branch" key={game.id}>
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
                                <div className="nav-tree-menu">
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
          </>
          )}
        </aside>
        )}

        <section
          className={
            activeComponent === "gaming" && isChatOverlayMode
                ? "workspace workspace-chat-overlay-mode"
                : "workspace"
          }
          aria-label="Active component"
        >
          {activeComponent !== "gaming" && (
            <header className="workspace-header">
              <div>
                <p>{activeMeta.eyebrow}</p>
                <h2>{activeMeta.title}</h2>
              </div>
            </header>
          )}

          <ComponentHost>
            {activeComponent === "calendar" && <Calendar />}
            {activeComponent === "cessation" && <Cessation />}
            {activeComponent === "repair-resell" && <RepairResell />}
            {activeComponent === "gaming" && (
              <Gaming
                chatOverlayMode={isChatOverlayMode}
                gameSections={gameSections}
                navAction={gameNavAction}
                onEnterChatOverlayMode={(game, conversationId) =>
                  void openGameChatOverlayWindow(game.id, conversationId)
                }
                onGameCreated={onGameCreated}
                onGameChatConversationsChanged={onGameChatConversationsChanged}
                onGameDeleted={onGameDeleted}
                onExitChatOverlayMode={() => void closeChatOverlayWindow()}
                onSelectGame={setSelectedGameId}
                selectedGameId={selectedGameId}
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

function compactNavLabel(label: string) {
  const words = label.split(/\s+/).filter(Boolean);
  if (words.length > 1) {
    return words.map((word) => word[0]).join("").slice(0, 2).toUpperCase();
  }
  return label.slice(0, 2).toUpperCase();
}
