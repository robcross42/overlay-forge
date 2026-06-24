import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useMemo, useRef, useState } from "react";
import type { MouseEvent } from "react";
import { ChatWorkspace } from "../components/ChatWorkspace";
import { WindowResizeHandles } from "../components/WindowControls";
import {
  createGameChatScreenshotCapture,
  createGameBuildGuideFromChat,
  focusGameChatOverlayWindow,
  getActiveGameChatOverlay,
  listGameChatConversations,
  listGameChatMessages,
  listGames,
  sendGameChatMessage
} from "../services/gaming";
import type {
  Game,
  GameChatConversation,
  GameChatMessage
} from "../services/gaming";
import {
  applyStandaloneOverlayFocusState,
  focusLastGameWindow,
  setOverlayWindowOpacity,
  startOverlayDrag
} from "../services/windowControls";

export default function GameChatOverlayApp() {
  const [game, setGame] = useState<Game | null>(null);
  const [conversation, setConversation] = useState<GameChatConversation | null>(null);
  const [messages, setMessages] = useState<GameChatMessage[]>([]);
  const [draft, setDraft] = useState("");
  const [status, setStatus] = useState("Loading chat");
  const [isSending, setIsSending] = useState(false);
  const [isGeneratingBuildGuide, setIsGeneratingBuildGuide] = useState(false);
  const [isCapturingPromptScreenshot, setIsCapturingPromptScreenshot] = useState(false);
  const [selectedPromptScreenshotIds, setSelectedPromptScreenshotIds] = useState<number[]>([]);
  const [focusInputRequestNonce, setFocusInputRequestNonce] = useState(0);
  const pendingScreenshotIdsByChatRef = useRef<Record<string, number[]>>({});

  const conversations = useMemo(
    () => (conversation ? [conversation] : []),
    [conversation]
  );

  useEffect(() => {
    let isMounted = true;
    let cleanup: (() => void) | null = null;
    void prepareChatOverlayWindow().then((nextCleanup) => {
      if (isMounted) {
        cleanup = nextCleanup;
      } else {
        nextCleanup?.();
      }
    });
    void loadActiveChat();

    return () => {
      isMounted = false;
      cleanup?.();
    };
  }, []);

  useEffect(() => {
    const cleanups: Array<() => void> = [];
    let isMounted = true;

    listen("game-chat-overlay-selection-changed", () => {
      if (isMounted) {
        void loadActiveChat();
      }
    }).then((cleanup) => {
      if (isMounted) {
        cleanups.push(cleanup);
      } else {
        cleanup();
      }
    });

    listen("game-chat-overlay-focus-prompt", () => {
      if (isMounted) {
        setFocusInputRequestNonce(Date.now());
      }
    }).then((cleanup) => {
      if (isMounted) {
        cleanups.push(cleanup);
      } else {
        cleanup();
      }
    });

    return () => {
      isMounted = false;
      cleanups.forEach((cleanup) => cleanup());
    };
  }, []);

  useEffect(() => {
    let isMounted = true;
    let cleanup: (() => void) | null = null;

    listen("game-chat-screenshot-capture-requested", () => {
      if (isMounted) {
        void capturePromptScreenshot();
      }
    }).then((nextCleanup) => {
      if (isMounted) {
        cleanup = nextCleanup;
      } else {
        nextCleanup();
      }
    });

    return () => {
      isMounted = false;
      cleanup?.();
    };
  }, [game?.id, isCapturingPromptScreenshot]);

  async function loadActiveChat() {
    try {
      const selection = await getActiveGameChatOverlay();
      if (!selection) {
        setGame(null);
        setConversation(null);
        setMessages([]);
        setStatus("No selected game chat");
        return;
      }

      const [games, conversations, nextMessages] = await Promise.all([
        listGames(),
        listGameChatConversations(selection.gameId),
        listGameChatMessages(selection.conversationId)
      ]);
      const nextGame = games.find((item) => item.id === selection.gameId) ?? null;
      const nextConversation =
        conversations.find((item) => item.id === selection.conversationId) ?? null;
      const selectedChatKey = chatAttachmentKey(selection.gameId, selection.conversationId);

      setGame(nextGame);
      setConversation(nextConversation);
      setMessages(nextMessages);
      setSelectedPromptScreenshotIds(pendingScreenshotIdsByChatRef.current[selectedChatKey] ?? []);
      setStatus(nextGame && nextConversation ? "Ready" : "Selected chat was not found");
      setFocusInputRequestNonce(Date.now());
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  async function capturePromptScreenshot() {
    if (!game || isCapturingPromptScreenshot) {
      return;
    }

    setIsCapturingPromptScreenshot(true);
    setStatus("Capturing screenshot");
    try {
      const request = await createGameChatScreenshotCapture(
        game.id,
        screenshotTimestampLabel(new Date())
      );
      const attachmentKey = conversation
        ? chatAttachmentKey(game.id, conversation.id)
        : "";
      setSelectedPromptScreenshotIds((current) => {
        const next = current.includes(request.id) ? current : [...current, request.id];
        if (attachmentKey) {
          pendingScreenshotIdsByChatRef.current[attachmentKey] = next;
        }
        return next;
      });
      setStatus("Screenshot added to current prompt");
      await revealChatOverlayWithoutKeepingFocus();
    } catch (error) {
      setStatus(formatError(error));
      await focusGameChatOverlayWindow().catch(() => false);
    } finally {
      setIsCapturingPromptScreenshot(false);
    }
  }

  async function revealChatOverlayWithoutKeepingFocus() {
    const window = getCurrentWindow();
    await window.setIgnoreCursorEvents(false).catch(() => {});
    await window.show().catch(() => {});
    await window.setAlwaysOnTop(true).catch(() => {});
    await setOverlayWindowOpacity(1).catch(() => {});
    await waitForOverlayRepaint();
    await window.setIgnoreCursorEvents(false).catch(() => {});
    await focusLastGameWindow().catch(() => false);
  }

  async function sendMessage() {
    if (!conversation || !draft.trim() || isSending) {
      return;
    }

    const content = draft.trim();
    const pendingMessage: GameChatMessage = {
      id: Date.now() * -1,
      conversationId: conversation.id,
      role: "user",
      content,
      createdAt: new Date().toISOString()
    };

    setDraft("");
    setIsSending(true);
    setMessages((current) => [...current, pendingMessage]);
    setStatus("Waiting for OpenAI");

    try {
      const nextMessages = await sendGameChatMessage(
        conversation.id,
        content,
        selectedPromptScreenshotIds
      );
      setMessages(nextMessages);
      delete pendingScreenshotIdsByChatRef.current[chatAttachmentKey(conversation.gameId, conversation.id)];
      setSelectedPromptScreenshotIds([]);
      setStatus("Response saved");
    } catch (error) {
      const persistedMessages = await listGameChatMessages(conversation.id).catch(() => null);
      if (persistedMessages) {
        setMessages(persistedMessages);
      } else {
        setMessages((current) => current.filter((message) => message.id !== pendingMessage.id));
      }
      setStatus(formatError(error));
    } finally {
      setIsSending(false);
      setFocusInputRequestNonce(Date.now());
    }
  }

  async function generateBuildGuide() {
    if (!conversation || isGeneratingBuildGuide) {
      return;
    }
    const latestUserMessage = [...messages]
      .reverse()
      .find((message) => message.role === "user");
    const buildGoal = draft.trim() || latestUserMessage?.content.trim() || "";
    if (!buildGoal) {
      setStatus("Type a build goal first");
      return;
    }

    setIsGeneratingBuildGuide(true);
    setStatus("Generating build guide");
    try {
      const generated = await createGameBuildGuideFromChat(conversation.id, buildGoal);
      setDraft("");
      setStatus(`Build guide created: ${generated.guide.title}`);
    } catch (error) {
      setStatus(formatError(error));
    } finally {
      setIsGeneratingBuildGuide(false);
      setFocusInputRequestNonce(Date.now());
    }
  }

  async function closeChatWindow() {
    const window = getCurrentWindow();
    await window.setIgnoreCursorEvents(false).catch(() => {});
    await window.hide();
  }

  function startChatDrag(event: MouseEvent) {
    if (event.detail !== 1 || event.button !== 0) {
      return;
    }
    void startOverlayDrag();
  }

  return (
    <main
      className="overlay-frame overlay-frame-chat-mode standalone-overlay-window game-chat-overlay-frame"
      onMouseDownCapture={() => {
        void getCurrentWindow().setIgnoreCursorEvents(false);
      }}
    >
      <WindowResizeHandles />
      <div
        className="overlay-window-titlebar chat-overlay-titlebar"
        onMouseDown={startChatDrag}
        role="presentation"
      >
        <h1>{conversation?.title ?? status}</h1>
        {game && <span>{game.name}</span>}
      </div>
      <div className="overlay-shell">
        <section className="workspace workspace-chat-overlay-mode" aria-label="Game chat overlay">
          <ChatWorkspace
            conversations={conversations}
            chatOverlayMode
            draft={draft}
            emptyConversationLabel="No selected game chat."
            focusInputRequestNonce={focusInputRequestNonce}
            hideSidebarWhenSelected
            inputPlaceholder={game ? `Ask about ${game.name}...` : "Select a game chat..."}
            inputActionSlot={
              game?.slug === "gearblocks" ? (
                <button
                  className="ghost-button chat-input-extra-action"
                  disabled={!conversation || isSending || isGeneratingBuildGuide}
                  onClick={() => void generateBuildGuide()}
                  type="button"
                >
                  Guide
                </button>
              ) : null
            }
            isSending={isSending}
            messages={messages}
            newConversationTitle=""
            onCaptureScreenshot={() => void capturePromptScreenshot()}
            onCreateConversation={() => {}}
            onDeleteConversation={() => {}}
            onDraftChange={setDraft}
            onExitChatOverlayMode={() => void closeChatWindow()}
            onNewConversationTitleChange={() => {}}
            onSelectConversation={() => {}}
            onSendMessage={() => void sendMessage()}
            promptContextSummary={
              isCapturingPromptScreenshot
                ? "Capturing screenshot..."
                : selectedPromptScreenshotIds.length > 0
                  ? `${selectedPromptScreenshotIds.length} screenshot(s) attached`
                  : "No screenshots attached"
            }
            selectedConversationId={conversation?.id ?? null}
            showFocusedToolbar={false}
            showOverlaySideActions={false}
            status={status}
            title={game ? `${game.name} chat` : "Game chat"}
          />
        </section>
      </div>
    </main>
  );
}

function formatError(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

async function prepareChatOverlayWindow() {
  const window = getCurrentWindow();
  await window.setIgnoreCursorEvents(false).catch(() => {});
  return applyStandaloneOverlayFocusState(window);
}

function waitForOverlayRepaint() {
  return new Promise<void>((resolve) => {
    window.requestAnimationFrame(() => {
      window.setTimeout(resolve, 60);
    });
  });
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

function chatAttachmentKey(gameId: number, conversationId: number) {
  return `${gameId}:${conversationId}`;
}

function padDatePart(value: number) {
  return value.toString().padStart(2, "0");
}
