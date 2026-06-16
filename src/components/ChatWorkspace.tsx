import { useEffect, useRef } from "react";
import type { KeyboardEvent, ReactNode } from "react";
import { startOverlayDrag } from "../services/windowControls";

export type ChatConversation = {
  id: number;
  title: string;
  updatedAt: string;
};

export type ChatMessage = {
  id: number;
  conversationId: number;
  role: "user" | "assistant" | "system";
  content: string;
  createdAt: string;
};

type ChatWorkspaceProps<TConversation extends ChatConversation, TMessage extends ChatMessage> = {
  conversations: TConversation[];
  messages: TMessage[];
  selectedConversationId: number | null;
  draft: string;
  newConversationTitle: string;
  status: string;
  isSending: boolean;
  title: string;
  emptyConversationLabel: string;
  inputPlaceholder: string;
  contextSlot?: ReactNode;
  promptContextSummary?: string;
  focusInputRequestNonce?: number;
  chatOverlayMode?: boolean;
  emptyMainSlot?: ReactNode;
  hideSidebarWhenSelected?: boolean;
  showFocusedToolbar?: boolean;
  onEnterChatOverlayMode?: () => void;
  onExitChatOverlayMode?: () => void;
  onCaptureScreenshot?: () => void;
  onCreateConversation: () => void;
  onDeleteConversation: (conversation: TConversation) => void;
  onDraftChange: (value: string) => void;
  onNewConversationTitleChange: (value: string) => void;
  onSelectConversation: (conversationId: number) => void;
  onSendMessage: () => void;
};

export function ChatWorkspace<TConversation extends ChatConversation, TMessage extends ChatMessage>({
  conversations,
  messages,
  selectedConversationId,
  draft,
  newConversationTitle,
  status,
  isSending,
  title,
  emptyConversationLabel,
  inputPlaceholder,
  contextSlot,
  promptContextSummary,
  focusInputRequestNonce = 0,
  chatOverlayMode = false,
  emptyMainSlot,
  hideSidebarWhenSelected = false,
  showFocusedToolbar = true,
  onEnterChatOverlayMode,
  onExitChatOverlayMode,
  onCaptureScreenshot,
  onCreateConversation,
  onDeleteConversation,
  onDraftChange,
  onNewConversationTitleChange,
  onSelectConversation,
  onSendMessage
}: ChatWorkspaceProps<TConversation, TMessage>) {
  const messageListRef = useRef<HTMLDivElement | null>(null);
  const inputRef = useRef<HTMLTextAreaElement | null>(null);
  const selectedConversation =
    conversations.find((conversation) => conversation.id === selectedConversationId) ?? null;
  const showEmptyMainSlot = !selectedConversation && emptyMainSlot;
  const showSidebar = !chatOverlayMode && (!hideSidebarWhenSelected || !selectedConversation);
  const showSideActions =
    selectedConversation && !showSidebar && (chatOverlayMode || onEnterChatOverlayMode);

  useEffect(() => {
    const messageList = messageListRef.current;
    if (!messageList || !selectedConversation) {
      return;
    }

    messageList.scrollTo({
      top: messageList.scrollHeight,
      behavior: isSending ? "auto" : "smooth"
    });
  }, [isSending, messages.length, selectedConversation?.id]);

  useEffect(() => {
    if (!selectedConversation || focusInputRequestNonce <= 0) {
      return;
    }

    inputRef.current?.focus();
  }, [focusInputRequestNonce, selectedConversation?.id]);

  function handleDraftKeyDown(event: KeyboardEvent<HTMLTextAreaElement>) {
    if (
      event.key !== "Enter" ||
      event.shiftKey ||
      !selectedConversation ||
      isSending ||
      !draft.endsWith("  ")
    ) {
      return;
    }

    event.preventDefault();
    onSendMessage();
  }

  return (
    <section
      className={
        showSidebar
          ? "chat-workspace"
          : chatOverlayMode
            ? "chat-workspace chat-workspace-focused chat-workspace-overlay-mode"
            : showSideActions
              ? "chat-workspace chat-workspace-focused chat-workspace-with-side-actions"
              : "chat-workspace chat-workspace-focused"
      }
      aria-label={title}
    >
      {showSidebar && (
        <aside className="chat-workspace-sidebar" aria-label={`${title} conversations`}>
          <div className="chat-workspace-new">
            <input
              aria-label="New conversation title"
              className="text-input"
              onChange={(event) => onNewConversationTitleChange(event.target.value)}
              placeholder="New chat title"
              value={newConversationTitle}
            />
            <button className="primary-button" onClick={onCreateConversation} type="button">
              New Chat
            </button>
          </div>

          <div className="sub-list">
            {conversations.length === 0 ? (
              <p>{emptyConversationLabel}</p>
            ) : (
              conversations.map((conversation) => (
                <div className="chat-conversation-row" key={conversation.id}>
                  <button
                    className={
                      conversation.id === selectedConversationId
                        ? "sub-list-item active"
                        : "sub-list-item"
                    }
                    onClick={() => onSelectConversation(conversation.id)}
                    type="button"
                  >
                    <strong>{conversation.title}</strong>
                    <span>{formatDate(conversation.updatedAt)}</span>
                  </button>
                  <button
                    aria-label={`Delete ${conversation.title}`}
                    className="ghost-button chat-conversation-delete"
                    disabled={isSending}
                    onClick={() => onDeleteConversation(conversation)}
                    type="button"
                  >
                    Delete
                  </button>
                </div>
              ))
            )}
          </div>
        </aside>
      )}

      {showSideActions && (
        <div
          className="chat-overlay-controls"
          aria-label="Chat controls"
          onMouseDown={(event) => {
            if (chatOverlayMode && event.button === 0 && event.target === event.currentTarget) {
              event.preventDefault();
              void startOverlayDrag();
            }
          }}
        >
          {chatOverlayMode ? (
            <div className="chat-overlay-exit-slot">
              <button
                aria-label="Exit chat overlay mode"
                className="chat-overlay-control"
                onClick={(event) => {
                  event.stopPropagation();
                  onExitChatOverlayMode?.();
                }}
                onMouseDown={(event) => event.stopPropagation()}
                title="Exit"
                type="button"
              >
                x
              </button>
              {onCaptureScreenshot && (
                <button
                  aria-label="Capture screenshot for current chat prompt"
                  className="chat-overlay-control"
                  onClick={(event) => {
                    event.stopPropagation();
                    onCaptureScreenshot();
                  }}
                  onMouseDown={(event) => event.stopPropagation()}
                  title="Capture screenshot"
                  type="button"
                >
                  Cap
                </button>
              )}
            </div>
          ) : (
            <button
              aria-label="Enter chat overlay mode"
              className="chat-overlay-control chat-overlay-entry-control"
              disabled={isSending}
              onClick={onEnterChatOverlayMode}
              title="Overlay"
              type="button"
            >
              Overlay
            </button>
          )}
        </div>
      )}

      <div
        className={
          showEmptyMainSlot
            ? "chat-workspace-main chat-workspace-main-empty"
            : showFocusedToolbar
              ? "chat-workspace-main"
              : "chat-workspace-main chat-workspace-main-no-toolbar"
        }
      >
        {showEmptyMainSlot ? (
          emptyMainSlot
        ) : (
          <>
            {showFocusedToolbar && (
              <div className="focused-chat-toolbar">
                <div>
                  <p>{status}</p>
                  <h3>{selectedConversation?.title ?? "Select or create a chat"}</h3>
                </div>
                <button
                  className="ghost-button"
                  disabled={!selectedConversation || isSending}
                  onClick={() => selectedConversation && onDeleteConversation(selectedConversation)}
                  type="button"
                >
                  Delete
                </button>
              </div>
            )}

            <div className="message-list" ref={messageListRef}>
              {!selectedConversation ? (
                <p>Create or select a chat to start.</p>
              ) : messages.length === 0 ? (
                <p>No messages yet.</p>
              ) : (
                messages.map((message) => (
                  <article className={`chat-message chat-message-${message.role}`} key={message.id}>
                    <span>{message.role === "assistant" ? "Assistant" : "You"}</span>
                    <p>{message.content}</p>
                  </article>
                ))
              )}
            </div>

            {chatOverlayMode && promptContextSummary && (
              <div className="chat-overlay-context-summary">{promptContextSummary}</div>
            )}

            <form className="chat-input-row">
              <textarea
                ref={inputRef}
                aria-label="Chat message"
                className="body-input compact"
                disabled={!selectedConversation || isSending}
                onChange={(event) => onDraftChange(event.target.value)}
                onKeyDown={handleDraftKeyDown}
                placeholder={inputPlaceholder}
                value={draft}
              />
              <button
                className="primary-button"
                disabled={!selectedConversation || isSending || draft.trim().length === 0}
                onClick={() => onSendMessage()}
                type="button"
              >
                Send
              </button>
            </form>

            {!chatOverlayMode && contextSlot}
          </>
        )}
      </div>
    </section>
  );
}

function formatDate(value: string) {
  if (!value) {
    return "Saved";
  }

  return value.replace("T", " ").slice(0, 16);
}
