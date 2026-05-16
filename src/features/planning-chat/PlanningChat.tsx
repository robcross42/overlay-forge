import { useEffect, useMemo, useState } from "react";
import {
  createPlanningConversation,
  deletePlanningConversation,
  listPlanningConversations,
  listPlanningMessages,
  sendPlanningMessage
} from "../../services/planningChat";
import type { PlanningConversation, PlanningMessage } from "../../services/planningChat";
import { listProjects } from "../../services/projects";
import type { Project } from "../../services/projects";

type PlanningChatProps = {
  project?: Project;
};

export function PlanningChat({ project }: PlanningChatProps) {
  const isWorkspaceChat = Boolean(project);
  const [projects, setProjects] = useState<Project[]>(project ? [project] : []);
  const [selectedProjectId, setSelectedProjectId] = useState<number | null>(project?.id ?? null);
  const [conversations, setConversations] = useState<PlanningConversation[]>([]);
  const [selectedConversationId, setSelectedConversationId] = useState<number | null>(null);
  const [messages, setMessages] = useState<PlanningMessage[]>([]);
  const [newConversationTitle, setNewConversationTitle] = useState("");
  const [draft, setDraft] = useState("");
  const [status, setStatus] = useState(project ? "Select a conversation" : "Loading");
  const [isSending, setIsSending] = useState(false);

  const selectedProject = useMemo(
    () => projects.find((project) => project.id === selectedProjectId) ?? null,
    [projects, selectedProjectId]
  );

  const selectedConversation = useMemo(
    () =>
      conversations.find((conversation) => conversation.id === selectedConversationId) ?? null,
    [conversations, selectedConversationId]
  );

  useEffect(() => {
    if (project) {
      setProjects([project]);
      setSelectedProjectId(project.id);
      setStatus("Select a conversation");
      return;
    }

    listProjects()
      .then((nextProjects) => {
        setProjects(nextProjects);
        setSelectedProjectId(nextProjects[0]?.id ?? null);
        setStatus(nextProjects.length === 0 ? "Create a project first" : "Select a conversation");
      })
      .catch((error) => setStatus(formatError(error)));
  }, [project]);

  useEffect(() => {
    if (!selectedProjectId) {
      setConversations([]);
      setSelectedConversationId(null);
      setMessages([]);
      return;
    }

    listPlanningConversations(selectedProjectId)
      .then((nextConversations) => {
        setConversations(nextConversations);
        setSelectedConversationId(nextConversations[0]?.id ?? null);
        setMessages([]);
        setStatus(
          nextConversations.length === 0 ? "No planning conversations yet" : "Ready"
        );
      })
      .catch((error) => setStatus(formatError(error)));
  }, [selectedProjectId]);

  useEffect(() => {
    if (!selectedConversationId) {
      setMessages([]);
      return;
    }

    listPlanningMessages(selectedConversationId)
      .then((nextMessages) => {
        setMessages(nextMessages);
        setStatus(nextMessages.length === 0 ? "Conversation ready" : "Ready");
      })
      .catch((error) => setStatus(formatError(error)));
  }, [selectedConversationId]);

  async function onNewConversation() {
    if (!selectedProject) {
      setStatus("Create a project before starting planning chat");
      return;
    }

    const title = newConversationTitle.trim();
    if (!title) {
      setStatus("Conversation title is required");
      return;
    }

    try {
      const created = await createPlanningConversation(selectedProject.id, title);
      setConversations((current) => [created, ...current]);
      setSelectedConversationId(created.id);
      setMessages([]);
      setNewConversationTitle("");
      setStatus("Conversation created");
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  async function onSendMessage() {
    if (!selectedConversation) {
      setStatus("Create or select a conversation first");
      return;
    }

    const content = draft.trim();
    if (!content) {
      setStatus("Message is required");
      return;
    }

    const pendingMessage: PlanningMessage = {
      id: Date.now() * -1,
      conversationId: selectedConversation.id,
      role: "user",
      content,
      createdAt: new Date().toISOString()
    };

    setDraft("");
    setIsSending(true);
    setMessages((current) => [...current, pendingMessage]);
    setStatus("Waiting for OpenAI");

    try {
      const nextMessages = await sendPlanningMessage(selectedConversation.id, content);
      setMessages(nextMessages);
      const nextConversations = await listPlanningConversations(selectedConversation.projectId);
      setConversations(nextConversations);
      setStatus("Response saved");
    } catch (error) {
      setMessages((current) => current.filter((message) => message.id !== pendingMessage.id));
      setStatus(formatError(error));
    } finally {
      setIsSending(false);
    }
  }

  async function onDeleteConversation() {
    if (!selectedConversation) {
      return;
    }

    try {
      await deletePlanningConversation(selectedConversation.id);
      const nextConversations = conversations.filter(
        (conversation) => conversation.id !== selectedConversation.id
      );
      setConversations(nextConversations);
      setSelectedConversationId(nextConversations[0]?.id ?? null);
      setMessages([]);
      setStatus("Conversation deleted");
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  return (
    <section className="feature-panel planning-panel">
      <div className="panel-heading">
        <div>
          <p>OpenAI Planning</p>
          <h3>Planning Chat</h3>
        </div>
        <span className={isSending ? "save-pill save-pill-loading" : "save-pill"}>{status}</span>
      </div>

      <div className="planning-layout">
        <aside className="planning-sidebar" aria-label="Planning chat controls">
          {isWorkspaceChat ? (
            <div className="workspace-project-context" aria-label="Workspace project">
              <span>Workspace Project</span>
              <strong>{selectedProject?.name ?? "No project selected"}</strong>
            </div>
          ) : (
            <label className="field-label">
              <span>Project</span>
              <select
                aria-label="Planning project"
                className="text-input"
                disabled={projects.length === 0 || isSending}
                onChange={(event) => setSelectedProjectId(Number(event.target.value))}
                value={selectedProjectId ?? ""}
              >
                {projects.length === 0 ? (
                  <option value="">No projects available</option>
                ) : (
                  projects.map((project) => (
                    <option key={project.id} value={project.id}>
                      {project.name}
                    </option>
                  ))
                )}
              </select>
            </label>
          )}

          <label className="field-label">
            <span>Conversation title</span>
            <input
              aria-label="Conversation title"
              className="text-input"
              disabled={!selectedProject || isSending}
              onChange={(event) => setNewConversationTitle(event.target.value)}
              placeholder="Planning topic"
              value={newConversationTitle}
            />
          </label>

          <button
            className="primary-button full-width"
            disabled={!selectedProject || isSending || newConversationTitle.trim().length === 0}
            onClick={() => void onNewConversation()}
            type="button"
          >
            New Conversation
          </button>

          <div className="sub-list" aria-label="Planning conversations">
            {conversations.map((conversation) => (
              <button
                className={
                  conversation.id === selectedConversationId
                    ? "sub-list-item active"
                    : "sub-list-item"
                }
                disabled={isSending}
                key={conversation.id}
                onClick={() => setSelectedConversationId(conversation.id)}
                type="button"
              >
                <strong>{conversation.title}</strong>
                <span>{formatDate(conversation.updatedAt)}</span>
              </button>
            ))}
          </div>

          {selectedConversation && (
            <button
              className="ghost-button full-width"
              disabled={isSending}
              onClick={() => void onDeleteConversation()}
              type="button"
            >
              Delete Conversation
            </button>
          )}
        </aside>

        <div className="planning-chat-surface">
          <div className="message-list" aria-label="Planning messages">
            {projects.length === 0 && (
              <div className="empty-editor-state">
                <p>Create a local project in Projects before using Planning Chat.</p>
              </div>
            )}

            {projects.length > 0 && !selectedConversation && (
              <div className="empty-editor-state">
                <p>Create a new conversation for the selected project.</p>
              </div>
            )}

            {messages.map((message) => (
              <article
                className={`chat-message chat-message-${message.role}`}
                key={message.id}
              >
                <span>{message.role}</span>
                <p>{message.content}</p>
              </article>
            ))}

            {isSending && (
              <article className="chat-message chat-message-assistant">
                <span>assistant</span>
                <p>Waiting for response...</p>
              </article>
            )}
          </div>

          <form className="chat-input-row">
            <textarea
              aria-label="Planning message"
              className="body-input compact"
              disabled={!selectedConversation || isSending}
              onChange={(event) => setDraft(event.target.value)}
              placeholder="Ask for project planning help..."
              value={draft}
            />
            <button
              className="primary-button"
              disabled={!selectedConversation || isSending || draft.trim().length === 0}
              onClick={() => void onSendMessage()}
              type="button"
            >
              Send
            </button>
          </form>
        </div>
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

function formatError(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
