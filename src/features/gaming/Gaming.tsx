import { convertFileSrc } from "@tauri-apps/api/core";
import { useEffect, useMemo, useRef, useState } from "react";
import { ChatWorkspace } from "../../components/ChatWorkspace";
import {
  catalogGamePartsFromScreenshots,
  createGame,
  createGameChatConversation,
  createGameChatScreenshotCapture,
  createGameScreenshotCaptureRequest,
  deleteGameChatConversation,
  deleteGameScreenshot,
  deleteGame,
  listGameChatConversations,
  listGameChatMessages,
  listGameCatalogObjects,
  listGamePartCategories,
  listGameScreenshots,
  sendGameChatMessage
} from "../../services/gaming";
import type {
  Game,
  GameChatConversation,
  GameChatMessage,
  GameCatalogObject,
  GamePartCategory,
  GameScreenshotCaptureRequest
} from "../../services/gaming";

type GamingProps = {
  chatOverlayMode?: boolean;
  chatOverlayRequestNonce?: number;
  chatScreenshotCaptureRequestNonce?: number;
  gameSections: Game[];
  navAction: GameNavAction | null;
  onEnterChatOverlayMode?: () => void;
  onGameCreated: (game: Game) => void;
  onGameChatConversationsChanged: (
    gameId: number,
    conversations: GameChatConversation[]
  ) => void;
  onGameDeleted: (gameId: number) => void;
  onExitChatOverlayMode?: () => void;
  onSelectGame: (gameId: number | null) => void;
  selectedGameId: number | null;
};

type GameNavAction = {
  type: "home" | "newChat" | "chat" | "screenshots" | "parts";
  gameId?: number;
  conversationId?: number;
  nonce: number;
};

type ScreenshotContextMenu = {
  screenshot: GameScreenshotCaptureRequest;
  x: number;
  y: number;
};

type GameView = "home" | "chat" | "screenshots" | "parts";
const CHAT_SCREENSHOTS_PER_PAGE = 8;

export function Gaming({
  chatOverlayMode = false,
  chatOverlayRequestNonce = 0,
  chatScreenshotCaptureRequestNonce = 0,
  gameSections,
  navAction,
  onEnterChatOverlayMode,
  onGameCreated,
  onGameChatConversationsChanged,
  onGameDeleted,
  onExitChatOverlayMode,
  onSelectGame,
  selectedGameId
}: GamingProps) {
  const [newGameName, setNewGameName] = useState("");
  const [status, setStatus] = useState(`${gameSections.length} game section(s)`);
  const [isRequestingScreenshot, setIsRequestingScreenshot] = useState(false);
  const [isCatalogingParts, setIsCatalogingParts] = useState(false);
  const [gameView, setGameView] = useState<GameView>("home");
  const [selectedPartCategory, setSelectedPartCategory] = useState("all");
  const [screenshots, setScreenshots] = useState<GameScreenshotCaptureRequest[]>([]);
  const [parts, setParts] = useState<GameCatalogObject[]>([]);
  const [partCategories, setPartCategories] = useState<GamePartCategory[]>([]);
  const [chatConversations, setChatConversations] = useState<GameChatConversation[]>([]);
  const [selectedChatConversationId, setSelectedChatConversationId] = useState<number | null>(null);
  const [chatMessages, setChatMessages] = useState<GameChatMessage[]>([]);
  const [newChatTitle, setNewChatTitle] = useState("");
  const [chatDraft, setChatDraft] = useState("");
  const [isSendingChat, setIsSendingChat] = useState(false);
  const [isCapturingPromptScreenshot, setIsCapturingPromptScreenshot] = useState(false);
  const [selectedPromptScreenshotIds, setSelectedPromptScreenshotIds] = useState<number[]>([]);
  const [chatScreenshotPage, setChatScreenshotPage] = useState(0);
  const [isChatScreenshotPickerOpen, setIsChatScreenshotPickerOpen] = useState(false);
  const [toastMessage, setToastMessage] = useState("");
  const [screenshotContextMenu, setScreenshotContextMenu] =
    useState<ScreenshotContextMenu | null>(null);
  const [deletingScreenshotId, setDeletingScreenshotId] = useState<number | null>(null);
  const toastTimeoutRef = useRef<number | null>(null);

  const selectedGame = useMemo(
    () => gameSections.find((game) => game.id === selectedGameId) ?? null,
    [gameSections, selectedGameId]
  );
  const displayedParts = useMemo(
    () =>
      selectedPartCategory === "all"
        ? parts
        : parts.filter((part) => part.category === selectedPartCategory),
    [parts, selectedPartCategory]
  );
  const chatScreenshotPageCount = Math.max(
    1,
    Math.ceil(screenshots.length / CHAT_SCREENSHOTS_PER_PAGE)
  );
  const visibleChatScreenshots = useMemo(
    () =>
      screenshots.slice(
        chatScreenshotPage * CHAT_SCREENSHOTS_PER_PAGE,
        chatScreenshotPage * CHAT_SCREENSHOTS_PER_PAGE + CHAT_SCREENSHOTS_PER_PAGE
      ),
    [chatScreenshotPage, screenshots]
  );

  useEffect(() => {
    if (!selectedGame) {
      setScreenshots([]);
      setParts([]);
      setPartCategories([]);
      setChatConversations([]);
      setSelectedChatConversationId(null);
      setChatMessages([]);
      setSelectedPromptScreenshotIds([]);
      setChatScreenshotPage(0);
      setIsChatScreenshotPickerOpen(false);
      setSelectedPartCategory("all");
      setGameView("home");
      return;
    }

    listGameScreenshots(selectedGame.id)
      .then(setScreenshots)
      .catch((error) => setStatus(formatError(error)));
    listGameCatalogObjects(selectedGame.id)
      .then(setParts)
      .catch((error) => setStatus(formatError(error)));
    listGamePartCategories(selectedGame.id)
      .then(setPartCategories)
      .catch((error) => setStatus(formatError(error)));
    listGameChatConversations(selectedGame.id)
      .then((conversations) => {
        setChatConversations(conversations);
        setSelectedChatConversationId((current) =>
          current && conversations.some((conversation) => conversation.id === current)
            ? current
            : null
        );
        onGameChatConversationsChanged(selectedGame.id, conversations);
      })
      .catch((error) => setStatus(formatError(error)));
  }, [selectedGame?.id]);

  useEffect(() => {
    if (!navAction || !navAction.gameId) {
      return;
    }

    if (navAction.gameId !== selectedGameId) {
      onSelectGame(navAction.gameId);
    }

    setGameView(navAction.type === "newChat" || navAction.type === "chat" ? "chat" : navAction.type);
    setSelectedPromptScreenshotIds([]);
    setIsChatScreenshotPickerOpen(false);

    if (navAction.type === "chat") {
      setSelectedChatConversationId(navAction.conversationId ?? null);
    } else if (navAction.type === "newChat") {
      setSelectedChatConversationId(null);
    }
  }, [navAction?.nonce]);

  useEffect(() => {
    if (chatOverlayMode && (!selectedGame || gameView !== "chat" || !selectedChatConversationId)) {
      onExitChatOverlayMode?.();
    }
  }, [chatOverlayMode, selectedGame?.id, gameView, selectedChatConversationId, onExitChatOverlayMode]);

  useEffect(() => {
    if (
      chatOverlayRequestNonce > 0 &&
      selectedGame &&
      gameView === "chat" &&
      selectedChatConversationId
    ) {
      onEnterChatOverlayMode?.();
    }
  }, [
    chatOverlayRequestNonce,
    selectedGame?.id,
    gameView,
    selectedChatConversationId,
    onEnterChatOverlayMode
  ]);

  useEffect(() => {
    if (
      chatScreenshotCaptureRequestNonce > 0 &&
      chatOverlayMode &&
      selectedGame &&
      gameView === "chat" &&
      selectedChatConversationId
    ) {
      void capturePromptScreenshot(selectedGame);
    }
  }, [
    chatScreenshotCaptureRequestNonce,
    chatOverlayMode,
    selectedGame?.id,
    gameView,
    selectedChatConversationId
  ]);

  useEffect(() => {
    setChatScreenshotPage((current) => Math.min(current, chatScreenshotPageCount - 1));
  }, [chatScreenshotPageCount]);

  useEffect(() => {
    if (!selectedChatConversationId) {
      setChatMessages([]);
      return;
    }

    listGameChatMessages(selectedChatConversationId)
      .then(setChatMessages)
      .catch((error) => setStatus(formatError(error)));
  }, [selectedChatConversationId]);

  useEffect(
    () => () => {
      if (toastTimeoutRef.current !== null) {
        window.clearTimeout(toastTimeoutRef.current);
      }
    },
    []
  );

  useEffect(() => {
    function closeContextMenuOnEscape(event: KeyboardEvent) {
      if (event.key === "Escape") {
        setScreenshotContextMenu(null);
      }
    }

    window.addEventListener("keydown", closeContextMenuOnEscape);
    window.addEventListener("resize", closeScreenshotContextMenu);
    return () => {
      window.removeEventListener("keydown", closeContextMenuOnEscape);
      window.removeEventListener("resize", closeScreenshotContextMenu);
    };
  }, []);

  function showToast(message: string) {
    setToastMessage(message);
    if (toastTimeoutRef.current !== null) {
      window.clearTimeout(toastTimeoutRef.current);
    }
    toastTimeoutRef.current = window.setTimeout(() => {
      setToastMessage("");
      toastTimeoutRef.current = null;
    }, 1800);
  }

  async function addGameSection() {
    const name = newGameName.trim();

    if (!name) {
      setStatus("Game name is required");
      return;
    }

    if (gameSections.some((game) => game.name.toLowerCase() === name.toLowerCase())) {
      setStatus("Game section already exists");
      return;
    }

    try {
      const created = await createGame({
        name,
        summary: `Game-specific workspace section for ${name} planning, object cataloging, references, and screenshots.`
      });
      onGameCreated(created);
      onSelectGame(created.id);
      setNewGameName("");
      setStatus("Game section added");
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  async function removeSelectedGame() {
    if (!selectedGame) {
      setStatus("Select a game section first");
      return;
    }

    try {
      await deleteGame(selectedGame.id);
      onGameDeleted(selectedGame.id);
      setStatus("Game section removed");
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  async function requestScreenshotCapture(game: Game) {
    setIsRequestingScreenshot(true);
    try {
      const request = await createGameScreenshotCaptureRequest(
        game.id,
        screenshotTimestampLabel(new Date())
      );
      setScreenshots((current) => [request, ...current.filter((item) => item.id !== request.id)]);
      setChatScreenshotPage(0);
      setStatus(`Screenshot saved: ${request.requestId}`);
      showToast("Successful");
    } catch (error) {
      setStatus(formatError(error));
      window.alert(formatError(error));
    } finally {
      setIsRequestingScreenshot(false);
    }
  }

  async function capturePromptScreenshot(game: Game) {
    if (!selectedChatConversationId || isCapturingPromptScreenshot) {
      return;
    }

    setIsCapturingPromptScreenshot(true);
    setStatus("Capturing screenshot");
    try {
      const request = await createGameChatScreenshotCapture(
        game.id,
        screenshotTimestampLabel(new Date())
      );
      setScreenshots((current) => [request, ...current.filter((item) => item.id !== request.id)]);
      setSelectedPromptScreenshotIds((current) =>
        current.includes(request.id) ? current : [...current, request.id]
      );
      setChatScreenshotPage(0);
      setStatus("Screenshot added to current prompt");
    } catch (error) {
      setStatus(formatError(error));
    } finally {
      setIsCapturingPromptScreenshot(false);
    }
  }

  async function catalogParts(game: Game) {
    setIsCatalogingParts(true);
    try {
      const catalogedParts = await catalogGamePartsFromScreenshots(game.id);
      const categories = await listGamePartCategories(game.id);
      setParts(catalogedParts);
      setPartCategories(categories);
      setSelectedPartCategory("all");
      setGameView("parts");
      setStatus(`${catalogedParts.length} parts cataloged`);
      showToast("Successful");
    } catch (error) {
      setStatus(formatError(error));
      window.alert(formatError(error));
    } finally {
      setIsCatalogingParts(false);
    }
  }

  async function createGameChat() {
    if (!selectedGame) {
      setStatus("Select a game first");
      return;
    }

    const title = newChatTitle.trim();
    if (!title) {
      setStatus("Chat title is required");
      return;
    }

    try {
      const created = await createGameChatConversation(selectedGame.id, title);
      const nextConversations = [created, ...chatConversations];
      setChatConversations(nextConversations);
      onGameChatConversationsChanged(selectedGame.id, nextConversations);
      setSelectedChatConversationId(created.id);
      setChatMessages([]);
      setNewChatTitle("");
      setGameView("chat");
      setStatus("Chat created");
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  async function sendGameChat() {
    if (!selectedChatConversationId) {
      setStatus("Create or select a chat first");
      return;
    }

    const content = chatDraft.trim();
    if (!content) {
      setStatus("Message is required");
      return;
    }

    const pendingMessage: GameChatMessage = {
      id: Date.now() * -1,
      conversationId: selectedChatConversationId,
      role: "user",
      content,
      createdAt: new Date().toISOString()
    };

    setChatDraft("");
    setIsSendingChat(true);
    setChatMessages((current) => [...current, pendingMessage]);
    setStatus("Waiting for OpenAI");

    try {
      const nextMessages = await sendGameChatMessage(
        selectedChatConversationId,
        content,
        selectedPromptScreenshotIds
      );
      setChatMessages(nextMessages);
      setSelectedPromptScreenshotIds([]);
      setIsChatScreenshotPickerOpen(false);
      if (selectedGame) {
        const nextConversations = await listGameChatConversations(selectedGame.id);
        setChatConversations(nextConversations);
        onGameChatConversationsChanged(selectedGame.id, nextConversations);
      }
      setStatus("Response saved");
    } catch (error) {
      setChatMessages((current) => current.filter((message) => message.id !== pendingMessage.id));
      setStatus(formatError(error));
    } finally {
      setIsSendingChat(false);
    }
  }

  async function removeGameChat(conversation: GameChatConversation) {
    if (!window.confirm(`Delete chat "${conversation.title}"?`)) {
      return;
    }

    try {
      await deleteGameChatConversation(conversation.id);
      const nextConversations = chatConversations.filter((item) => item.id !== conversation.id);
      setChatConversations(nextConversations);
      if (selectedGame) {
        onGameChatConversationsChanged(selectedGame.id, nextConversations);
      }
      setSelectedChatConversationId((current) => (current === conversation.id ? null : current));
      setSelectedPromptScreenshotIds([]);
      setIsChatScreenshotPickerOpen(false);
      setStatus("Chat deleted");
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  function openChatDefaults() {
    setGameView("chat");
    setSelectedChatConversationId(null);
    setSelectedPromptScreenshotIds([]);
    setIsChatScreenshotPickerOpen(false);
  }

  function togglePromptScreenshot(screenshotId: number) {
    setSelectedPromptScreenshotIds((current) =>
      current.includes(screenshotId)
        ? current.filter((id) => id !== screenshotId)
        : [...current, screenshotId]
    );
  }

  function openScreenshotContextMenu(
    event: React.MouseEvent<HTMLElement>,
    screenshot: GameScreenshotCaptureRequest
  ) {
    event.preventDefault();
    const menuWidth = 238;
    const menuHeight = 274;
    const x = Math.min(event.clientX, window.innerWidth - menuWidth - 8);
    const y = Math.min(event.clientY, window.innerHeight - menuHeight - 8);

    setScreenshotContextMenu({
      screenshot,
      x: Math.max(8, x),
      y: Math.max(8, y)
    });
  }

  function closeScreenshotContextMenu() {
    setScreenshotContextMenu(null);
  }

  function runScreenshotMenuPlaceholder(label: string) {
    setStatus(`${label} is a visual test action`);
    setScreenshotContextMenu(null);
  }

  async function removeScreenshot(screenshot: GameScreenshotCaptureRequest) {
    if (!window.confirm("Delete this screenshot and its capture metadata?")) {
      return;
    }

    setDeletingScreenshotId(screenshot.id);
    setScreenshotContextMenu(null);
    try {
      await deleteGameScreenshot(screenshot.id);
      setScreenshots((current) => current.filter((item) => item.id !== screenshot.id));
      setStatus("Screenshot deleted");
      showToast("Deleted");
    } catch (error) {
      setStatus(formatError(error));
      window.alert(formatError(error));
    } finally {
      setDeletingScreenshotId(null);
    }
  }

  if (selectedGame) {
    return (
      <section
        className={
          chatOverlayMode
            ? "game-element-canvas game-element-canvas-chat-overlay-mode"
            : "game-element-canvas"
        }
        aria-label="Selected game workspace"
      >
        {toastMessage && (
          <div className="game-toast" role="status">
            {toastMessage}
          </div>
        )}

        {!chatOverlayMode && (
        <div className="game-canvas-toolbar" aria-label="Game workspace actions">
          <button
            className={gameView === "home" ? "primary-button" : "ghost-button"}
            onClick={() => setGameView("home")}
            type="button"
          >
            Home
          </button>
          <button
            className={gameView === "chat" ? "primary-button" : "ghost-button"}
            onClick={openChatDefaults}
            type="button"
          >
            Chats
            <span className="button-count">{chatConversations.length}</span>
          </button>
          <button
            className={gameView === "screenshots" ? "primary-button" : "ghost-button"}
            onClick={() => setGameView("screenshots")}
            type="button"
          >
            Screenshots
            <span className="button-count">{screenshots.length}</span>
          </button>
          <button
            className={gameView === "parts" ? "primary-button" : "ghost-button"}
            onClick={() => setGameView("parts")}
            type="button"
          >
            Parts
            <span className="button-count">{parts.length}</span>
          </button>
          <button className="ghost-button" type="button">
            Add Reference
          </button>
        </div>
        )}

        {!chatOverlayMode && gameView === "parts" && partCategories.length > 0 && (
          <div className="game-part-filter-bar" aria-label="Part category filters">
            <button
              className={selectedPartCategory === "all" ? "active" : ""}
              onClick={() => setSelectedPartCategory("all")}
              type="button"
            >
              <span>All</span>
              <strong>{parts.length}</strong>
            </button>
            {partCategories.map((category) => (
              <button
                className={selectedPartCategory === category.name ? "active" : ""}
                key={category.name}
                onClick={() => setSelectedPartCategory(category.name)}
                title={category.name}
                type="button"
              >
                {category.iconPath ? (
                  <img alt="" src={convertFileSrc(category.iconPath)} />
                ) : (
                  <span>{category.fallbackIcon}</span>
                )}
                <strong>{category.count}</strong>
              </button>
            ))}
          </div>
        )}

        <div className="game-canvas-scroll-area">
          {gameView === "home" && (
            <section className="game-home-panel" aria-label={`${selectedGame.name} home`}>
              <div>
                <p>Game Workspace</p>
                <h3>{selectedGame.name}</h3>
              </div>
              <span>{screenshots.length} screenshots</span>
              <span>{parts.length} cataloged parts</span>
              <span>{chatConversations.length} chats</span>
            </section>
          )}

          {gameView === "chat" && (
            <ChatWorkspace
              conversations={chatConversations}
              chatOverlayMode={chatOverlayMode}
              contextSlot={
                <section className="chat-screenshot-context" aria-label="Screenshot context">
                  <div className="chat-screenshot-summary">
                    <div>
                      <p>Prompt context</p>
                      <strong>{selectedPromptScreenshotIds.length} screenshot(s) selected</strong>
                    </div>
                    <button
                      className="ghost-button"
                      onClick={() => setIsChatScreenshotPickerOpen((current) => !current)}
                      type="button"
                    >
                      {isChatScreenshotPickerOpen ? "Hide Screenshots" : "Add Screenshots"}
                    </button>
                  </div>

                  {isChatScreenshotPickerOpen && (
                    <>
                      <div className="chat-screenshot-actions">
                        <button
                          className="ghost-button"
                          disabled={chatScreenshotPage === 0 || screenshots.length === 0}
                          onClick={() =>
                            setChatScreenshotPage((current) => Math.max(0, current - 1))
                          }
                          type="button"
                        >
                          Prev
                        </button>
                        <span>
                          {screenshots.length === 0
                            ? "0 / 0"
                            : `${chatScreenshotPage + 1} / ${chatScreenshotPageCount}`}
                        </span>
                        <button
                          className="ghost-button"
                          disabled={
                            chatScreenshotPage >= chatScreenshotPageCount - 1 ||
                            screenshots.length === 0
                          }
                          onClick={() =>
                            setChatScreenshotPage((current) =>
                              Math.min(chatScreenshotPageCount - 1, current + 1)
                            )
                          }
                          type="button"
                        >
                          Next
                        </button>
                        <button
                          className="primary-button"
                          disabled={isRequestingScreenshot}
                          onClick={() => void requestScreenshotCapture(selectedGame)}
                          type="button"
                        >
                          Capture
                        </button>
                      </div>
                      {screenshots.length === 0 ? (
                        <p>No screenshots captured yet.</p>
                      ) : (
                        <div className="chat-screenshot-context-grid">
                          {visibleChatScreenshots.map((screenshot) => (
                            <label
                              className={
                                selectedPromptScreenshotIds.includes(screenshot.id)
                                  ? "chat-screenshot-option selected"
                                  : "chat-screenshot-option"
                              }
                              key={screenshot.id}
                            >
                              <input
                                checked={selectedPromptScreenshotIds.includes(screenshot.id)}
                                onChange={() => togglePromptScreenshot(screenshot.id)}
                                type="checkbox"
                              />
                              <img
                                alt={`Screenshot captured ${formatCapturedAt(screenshot)}`}
                                src={convertFileSrc(screenshot.filePath)}
                              />
                              <span>{formatCapturedAt(screenshot)}</span>
                            </label>
                          ))}
                        </div>
                      )}
                    </>
                  )}
                </section>
              }
              draft={chatDraft}
              emptyMainSlot={<GameChatDefaultsPane game={selectedGame} />}
              emptyConversationLabel="No game chats yet."
              hideSidebarWhenSelected
              inputPlaceholder={`Ask about ${selectedGame.name}...`}
              isSending={isSendingChat}
              messages={chatMessages}
              newConversationTitle={newChatTitle}
              onEnterChatOverlayMode={onEnterChatOverlayMode}
              onCaptureScreenshot={() => void capturePromptScreenshot(selectedGame)}
              onCreateConversation={() => void createGameChat()}
              onDeleteConversation={(conversation) => void removeGameChat(conversation)}
              onDraftChange={setChatDraft}
              onExitChatOverlayMode={onExitChatOverlayMode}
              onNewConversationTitleChange={setNewChatTitle}
              onSelectConversation={(conversationId) => {
                setSelectedChatConversationId(conversationId);
                setSelectedPromptScreenshotIds([]);
                setIsChatScreenshotPickerOpen(false);
              }}
              onSendMessage={() => void sendGameChat()}
              promptContextSummary={
                selectedPromptScreenshotIds.length > 0
                  ? `${selectedPromptScreenshotIds.length} screenshot(s) attached`
                  : ""
              }
              selectedConversationId={selectedChatConversationId}
              showFocusedToolbar={false}
              status={status}
              title={`${selectedGame.name} chat`}
            />
          )}

          {gameView === "parts" && (
            <section className="game-parts-panel" aria-label="Cataloged parts">
              <div className="game-view-head">
                <div>
                  <p>Catalog</p>
                  <h3>Parts</h3>
                </div>
                <button
                  className="primary-button"
                  disabled={isCatalogingParts}
                  onClick={() => void catalogParts(selectedGame)}
                  type="button"
                >
                  Catalog Parts
                </button>
              </div>
              <div className="game-parts-chart">
                {parts.length === 0 ? (
                  <p>No parts cataloged yet.</p>
                ) : displayedParts.length === 0 ? (
                  <p>No parts cataloged for this category yet.</p>
                ) : (
                  <>
                    <div className="game-parts-chart-head">
                      <span>Thumbnail</span>
                      <span>Part name</span>
                      <span>Practical physics use</span>
                    </div>
                    {displayedParts.map((part) => (
                      <article className="game-part-row" key={part.id}>
                        <div className="game-part-thumb">
                          {part.thumbnailPath ? (
                            <img alt={`${part.name} source screenshot`} src={convertFileSrc(part.thumbnailPath)} />
                          ) : (
                            <span>No image</span>
                          )}
                        </div>
                        <strong className="game-part-name">{part.name}</strong>
                        <p>{part.description}</p>
                      </article>
                    ))}
                  </>
                )}
              </div>
            </section>
          )}

          {gameView === "screenshots" && (
            <section className="game-screenshots-panel" aria-label="Screenshots">
              <div className="game-view-head">
                <div>
                  <p>Gallery</p>
                  <h3>Screenshots</h3>
                </div>
                <button
                  className="primary-button"
                  disabled={isRequestingScreenshot}
                  onClick={() => void requestScreenshotCapture(selectedGame)}
                  type="button"
                >
                  Capture Screenshot
                </button>
              </div>
              <div className="game-screenshot-list">
                {screenshots.length === 0 ? (
                  <p>No screenshots captured yet.</p>
                ) : (
                  screenshots.map((screenshot) => (
                    <article
                      className={
                        deletingScreenshotId === screenshot.id
                          ? "game-screenshot-card deleting"
                          : "game-screenshot-card"
                      }
                      key={screenshot.id}
                      onContextMenu={(event) => openScreenshotContextMenu(event, screenshot)}
                    >
                      <img
                        alt={`Screenshot captured ${formatCapturedAt(screenshot)}`}
                        src={convertFileSrc(screenshot.filePath)}
                      />
                      <div>
                        <strong>{formatCapturedAt(screenshot)}</strong>
                        <span>{screenshot.captureStatus}</span>
                      </div>
                    </article>
                  ))
                )}
              </div>
            </section>
          )}
        </div>

        {screenshotContextMenu && (
          <div className="game-context-menu-layer" onMouseDown={closeScreenshotContextMenu}>
            <div
              aria-label="Screenshot actions"
              className="game-context-menu"
              onMouseDown={(event) => event.stopPropagation()}
              role="menu"
              style={{
                left: screenshotContextMenu.x,
                top: screenshotContextMenu.y
              }}
            >
              <div className="game-context-menu-title">
                <strong>{formatCapturedAt(screenshotContextMenu.screenshot)}</strong>
                <span>{screenshotContextMenu.screenshot.captureStatus}</span>
              </div>
              <button
                onClick={() => runScreenshotMenuPlaceholder("Open preview")}
                role="menuitem"
                type="button"
              >
                Open preview
              </button>
              <button
                onClick={() => runScreenshotMenuPlaceholder("Create object from screenshot")}
                role="menuitem"
                type="button"
              >
                Create object from screenshot
              </button>
              <button
                onClick={() => runScreenshotMenuPlaceholder("Attach reference")}
                role="menuitem"
                type="button"
              >
                Attach reference
              </button>
              <button
                onClick={() => runScreenshotMenuPlaceholder("Mark reviewed")}
                role="menuitem"
                type="button"
              >
                Mark reviewed
              </button>
              <div className="game-context-menu-separator" />
              <button
                className="danger"
                disabled={deletingScreenshotId === screenshotContextMenu.screenshot.id}
                onClick={() => void removeScreenshot(screenshotContextMenu.screenshot)}
                role="menuitem"
                type="button"
              >
                Delete screenshot and references
              </button>
            </div>
          </div>
        )}
      </section>
    );
  }

  return (
    <section className="feature-panel">
      <div className="panel-heading">
        <div>
          <p>Game Workspaces</p>
          <h3>Gaming</h3>
        </div>
        <span className="save-pill">{status}</span>
      </div>

      <div className="split-feature-body">
        <aside className="sub-list" aria-label="Game section list">
          {gameSections.map((game) => (
            <button
              className={game.id === selectedGameId ? "sub-list-item active" : "sub-list-item"}
              key={game.id}
              onClick={() => {
                onSelectGame(game.id);
                setStatus("Game section selected");
              }}
              type="button"
            >
              <strong>{game.name}</strong>
              <span>Game section</span>
            </button>
          ))}
        </aside>

        <div className="gaming-detail-panel">
          <div className="inline-form">
            <input
              aria-label="New game section name"
              className="text-input"
              onChange={(event) => setNewGameName(event.target.value)}
              placeholder="Game section name"
              value={newGameName}
            />
            <button className="primary-button" onClick={() => void addGameSection()} type="button">
              Add Game
            </button>
            <button
              className="ghost-button"
              disabled={!selectedGame}
              onClick={() => void removeSelectedGame()}
              type="button"
            >
              Remove
            </button>
          </div>

          <div className="empty-editor-state">
            <p>Add a game section to begin.</p>
          </div>
        </div>
      </div>
    </section>
  );
}

function formatError(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

function screenshotTimestampLabel(date: Date) {
  const year = date.getFullYear();
  const month = padDatePart(date.getMonth() + 1);
  const day = padDatePart(date.getDate());
  const hours = padDatePart(date.getHours());
  const minutes = padDatePart(date.getMinutes());
  const seconds = padDatePart(date.getSeconds());

  return `${year}${month}${day}_${hours}${minutes}${seconds}`;
}

function padDatePart(value: number) {
  return value.toString().padStart(2, "0");
}

function formatCapturedAt(screenshot: GameScreenshotCaptureRequest) {
  const value = screenshot.capturedAt || screenshot.createdAt;
  const match = /^(\d{4})(\d{2})(\d{2})_(\d{2})(\d{2})(\d{2})/.exec(value);

  if (match) {
    const [, year, month, day, hour, minute, second] = match;
    return `${year}-${month}-${day} ${hour}:${minute}:${second}`;
  }

  return value || "Unknown capture time";
}

function GameChatDefaultsPane({ game }: { game: Game }) {
  const isGearBlocks = game.slug === "gearblocks";

  return (
    <section className="game-chat-defaults-panel" aria-label={`${game.name} chat defaults`}>
      <div className="game-view-head">
        <div>
          <p>Chat Defaults</p>
          <h3>{game.name}</h3>
        </div>
      </div>

      <div className="game-chat-defaults-grid">
        <article className={isGearBlocks ? "game-chat-default-item enabled" : "game-chat-default-item"}>
          <div>
            <strong>{isGearBlocks ? "Parts catalog context" : "Game context"}</strong>
            <span>
              {isGearBlocks
                ? "Enabled for every GearBlocks chat"
                : "No game-specific context configured yet"}
            </span>
          </div>
          <span>{isGearBlocks ? "On" : "Off"}</span>
        </article>

        <article className="game-chat-default-item enabled">
          <div>
            <strong>Screenshot context</strong>
            <span>Select screenshots after opening or creating a chat</span>
          </div>
          <span>Per prompt</span>
        </article>

        <article className="game-chat-default-item enabled">
          <div>
            <strong>Response speed</strong>
            <span>Low reasoning, concise output, low-detail images</span>
          </div>
          <span>Fast</span>
        </article>
      </div>

      <p className="game-chat-defaults-note">
        Select an existing chat or create a new chat to open the AI interface.
      </p>
    </section>
  );
}
