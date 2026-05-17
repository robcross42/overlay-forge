import { useEffect, useMemo, useState } from "react";
import { listCalendarEvents } from "../../services/calendar";
import type { CalendarEvent } from "../../services/calendar";
import { getProjectGitHubRepository } from "../../services/github";
import type { ProjectGitHubRepository } from "../../services/github";
import { listNotes } from "../../services/notes";
import type { Note } from "../../services/notes";
import {
  attachPlanningConversationContext,
  createPlanningConversation,
  deletePlanningConversation,
  listPlanningConversationContext,
  listPlanningConversations,
  listPlanningMessages,
  removePlanningConversationContext,
  sendPlanningMessage
} from "../../services/planningChat";
import type {
  PlanningContextType,
  PlanningConversation,
  PlanningConversationContext,
  PlanningMessage
} from "../../services/planningChat";
import { listProjects } from "../../services/projects";
import type { Project } from "../../services/projects";
import { loadScratchpad } from "../../services/scratchpad";
import { listTasks } from "../../services/tasks";
import type { Task } from "../../services/tasks";
import { listYouTubeReferences } from "../../services/youtube";
import type { YouTubeReference } from "../../services/youtube";

type PlanningChatProps = {
  project?: Project;
};

type AttachableOption = {
  contextType: PlanningContextType;
  sourceId: number | null;
  label: string;
};

const contextTypes = [
  { value: "project", label: "Project details" },
  { value: "github_repository", label: "GitHub repository metadata" },
  { value: "note", label: "Note" },
  { value: "task", label: "Task" },
  { value: "calendar_event", label: "Calendar event" },
  { value: "youtube_reference", label: "YouTube reference" },
  { value: "scratchpad", label: "Scratchpad" }
] satisfies Array<{ value: PlanningContextType; label: string }>;

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
  const [contextItems, setContextItems] = useState<PlanningConversationContext[]>([]);
  const [isAttachOpen, setIsAttachOpen] = useState(false);
  const [attachType, setAttachType] = useState<PlanningContextType>("project");
  const [attachSourceKey, setAttachSourceKey] = useState("");
  const [notes, setNotes] = useState<Note[]>([]);
  const [tasks, setTasks] = useState<Task[]>([]);
  const [calendarEvents, setCalendarEvents] = useState<CalendarEvent[]>([]);
  const [youtubeReferences, setYouTubeReferences] = useState<YouTubeReference[]>([]);
  const [githubLink, setGithubLink] = useState<ProjectGitHubRepository | null>(null);
  const [scratchpadContent, setScratchpadContent] = useState("");
  const [autoAttachedGitHubKeys, setAutoAttachedGitHubKeys] = useState<string[]>([]);

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
      setContextItems([]);
      return;
    }

    Promise.all([
      listPlanningMessages(selectedConversationId),
      listPlanningConversationContext(selectedConversationId)
    ])
      .then(([nextMessages, nextContext]) => {
        setMessages(nextMessages);
        setContextItems(nextContext);
        setStatus(nextMessages.length === 0 ? "Conversation ready" : "Ready");
      })
      .catch((error) => setStatus(formatError(error)));
  }, [selectedConversationId]);

  useEffect(() => {
    Promise.all([listNotes(), listTasks(), listCalendarEvents(), listYouTubeReferences(), loadScratchpad()])
      .then(([nextNotes, nextTasks, nextEvents, nextReferences, scratchpad]) => {
        setNotes(nextNotes);
        setTasks(nextTasks);
        setCalendarEvents(nextEvents);
        setYouTubeReferences(nextReferences);
        setScratchpadContent(scratchpad);
      })
      .catch((error) => setStatus(formatError(error)));
  }, []);

  useEffect(() => {
    if (!selectedProject) {
      setGithubLink(null);
      return;
    }

    getProjectGitHubRepository(selectedProject.id)
      .then(setGithubLink)
      .catch(() => setGithubLink(null));
  }, [selectedProject?.id]);

  const attachableOptions = useMemo(
    () => buildAttachableOptions({
      selectedProject,
      githubLink,
      notes,
      tasks,
      calendarEvents,
      youtubeReferences,
      scratchpadContent
    }),
    [selectedProject, githubLink, notes, tasks, calendarEvents, youtubeReferences, scratchpadContent]
  );

  const activeAttachOptions = useMemo(
    () => attachableOptions.filter((option) => option.contextType === attachType),
    [attachType, attachableOptions]
  );

  useEffect(() => {
    setAttachSourceKey(activeAttachOptions[0] ? optionKey(activeAttachOptions[0]) : "");
  }, [attachType, activeAttachOptions]);

  useEffect(() => {
    if (!selectedConversation || !githubLink) {
      return;
    }

    const autoKey = `${selectedConversation.id}:${githubLink.id}`;
    if (autoAttachedGitHubKeys.includes(autoKey)) {
      return;
    }

    const alreadyAttached = contextItems.some(
      (item) =>
        item.contextType === "github_repository" &&
        (item.sourceId === githubLink.id || item.label === githubLink.repositoryFullName)
    );
    if (alreadyAttached) {
      setAutoAttachedGitHubKeys((current) =>
        current.includes(autoKey) ? current : [...current, autoKey]
      );
      return;
    }

    attachPlanningConversationContext({
      conversationId: selectedConversation.id,
      contextType: "github_repository",
      sourceId: githubLink.id,
      label: githubLink.repositoryFullName
    })
      .then((attached) => {
        setContextItems((current) => {
          const exists = current.some(
            (item) =>
              item.contextType === "github_repository" &&
              (item.sourceId === attached.sourceId || item.label === attached.label)
          );
          return exists ? current : [...current, attached];
        });
        setAutoAttachedGitHubKeys((current) =>
          current.includes(autoKey) ? current : [...current, autoKey]
        );
      })
      .catch((error) => setStatus(formatError(error)));
  }, [selectedConversation?.id, githubLink?.id, contextItems, autoAttachedGitHubKeys]);

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
      setContextItems([]);
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
      setContextItems([]);
      setStatus("Conversation deleted");
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  async function onAttachContext() {
    if (!selectedConversation) {
      setStatus("Create or select a conversation before attaching context");
      return;
    }

    const selectedOption = activeAttachOptions.find(
      (option) => optionKey(option) === attachSourceKey
    );
    if (!selectedOption) {
      setStatus("No context item is available for that type");
      return;
    }

    try {
      const attached = await attachPlanningConversationContext({
        conversationId: selectedConversation.id,
        contextType: selectedOption.contextType,
        sourceId: selectedOption.sourceId,
        label: selectedOption.label
      });
      setContextItems((current) => [...current, attached]);
      setIsAttachOpen(false);
      setStatus("Context attached");
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  async function onRemoveContext(contextItem: PlanningConversationContext) {
    try {
      await removePlanningConversationContext(contextItem.id);
      setContextItems((current) => current.filter((item) => item.id !== contextItem.id));
      setStatus("Context attachment removed");
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

          <section className="attached-context-panel" aria-label="Attached context">
            <div className="attached-context-heading">
              <div>
                <p>Attached Context</p>
                <h4>Conversation Links</h4>
              </div>
              <button
                className="ghost-button"
                disabled={!selectedConversation || isSending}
                onClick={() => setIsAttachOpen((current) => !current)}
                type="button"
              >
                + Attach Context
              </button>
            </div>

            {isAttachOpen && (
              <div className="attach-context-form">
                <label className="field-label">
                  <span>Type</span>
                  <select
                    aria-label="Context type"
                    className="text-input"
                    disabled={!selectedConversation || isSending}
                    onChange={(event) => setAttachType(event.target.value as PlanningContextType)}
                    value={attachType}
                  >
                    {contextTypes.map((item) => (
                      <option key={item.value} value={item.value}>
                        {item.label}
                      </option>
                    ))}
                  </select>
                </label>

                <label className="field-label">
                  <span>Item</span>
                  <select
                    aria-label="Context item"
                    className="text-input"
                    disabled={activeAttachOptions.length === 0 || !selectedConversation || isSending}
                    onChange={(event) => setAttachSourceKey(event.target.value)}
                    value={attachSourceKey}
                  >
                    {activeAttachOptions.length === 0 ? (
                      <option value="">No items available</option>
                    ) : (
                      activeAttachOptions.map((option) => (
                        <option key={optionKey(option)} value={optionKey(option)}>
                          {option.label}
                        </option>
                      ))
                    )}
                  </select>
                </label>

                <button
                  className="primary-button"
                  disabled={!selectedConversation || activeAttachOptions.length === 0 || isSending}
                  onClick={() => void onAttachContext()}
                  type="button"
                >
                  Attach
                </button>
              </div>
            )}

            <div className="attached-context-list">
              {contextItems.length === 0 ? (
                <p>No context attached to this conversation.</p>
              ) : (
                contextItems.map((contextItem) => (
                  <div className="attached-context-item" key={contextItem.id}>
                    <span>
                      {formatContextType(contextItem.contextType)}: {contextItem.label}
                    </span>
                    <button
                      className="ghost-button"
                      disabled={isSending}
                      onClick={() => void onRemoveContext(contextItem)}
                      type="button"
                    >
                      Remove
                    </button>
                  </div>
                ))
              )}
            </div>
          </section>

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

function buildAttachableOptions(input: {
  selectedProject: Project | null;
  githubLink: ProjectGitHubRepository | null;
  notes: Note[];
  tasks: Task[];
  calendarEvents: CalendarEvent[];
  youtubeReferences: YouTubeReference[];
  scratchpadContent: string;
}) {
  const options: AttachableOption[] = [];

  if (input.selectedProject) {
    options.push({
      contextType: "project",
      sourceId: input.selectedProject.id,
      label: input.selectedProject.name
    });
  }

  if (input.githubLink) {
    options.push({
      contextType: "github_repository",
      sourceId: input.githubLink.id,
      label: input.githubLink.repositoryFullName
    });
  }

  input.notes.forEach((note) => {
    options.push({ contextType: "note", sourceId: note.id, label: note.title });
  });

  input.tasks.forEach((task) => {
    options.push({ contextType: "task", sourceId: task.id, label: task.title });
  });

  input.calendarEvents.forEach((event) => {
    options.push({
      contextType: "calendar_event",
      sourceId: event.id,
      label: `${event.title} (${event.startDate})`
    });
  });

  input.youtubeReferences.forEach((reference) => {
    options.push({
      contextType: "youtube_reference",
      sourceId: reference.id,
      label: reference.title
    });
  });

  if (input.scratchpadContent.trim().length > 0) {
    options.push({ contextType: "scratchpad", sourceId: null, label: "Scratchpad" });
  }

  return options;
}

function optionKey(option: AttachableOption) {
  return `${option.contextType}:${option.sourceId ?? "singleton"}`;
}

function formatContextType(contextType: PlanningContextType) {
  switch (contextType) {
    case "project":
      return "Project";
    case "github_repository":
      return "GitHub Repository";
    case "note":
      return "Note";
    case "task":
      return "Task";
    case "calendar_event":
      return "Calendar Event";
    case "youtube_reference":
      return "YouTube Reference";
    case "scratchpad":
      return "Scratchpad";
    default:
      return contextType;
  }
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
