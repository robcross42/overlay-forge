import { useEffect, useMemo, useState } from "react";
import { listCalendarEvents } from "../../services/calendar";
import type { CalendarEvent } from "../../services/calendar";
import { getProjectGitHubRepository } from "../../services/github";
import type { ProjectGitHubRepository } from "../../services/github";
import { listNotes } from "../../services/notes";
import type { Note } from "../../services/notes";
import {
  attachPlanningConversationContext,
  createBridgeFileDraftFromConversation,
  createPlanningConversation,
  deleteBridgeFileDraft,
  deleteProjectMarkdownContext,
  deletePlanningConversation,
  getBridgeFileDraft,
  getProjectMarkdownContext,
  listBridgeFileDrafts,
  listPlanningConversationContext,
  listPlanningConversations,
  listPlanningMessages,
  loadProjectMarkdownContext,
  removePlanningConversationContext,
  saveProjectMarkdownContext,
  sendPlanningMessage
} from "../../services/planningChat";
import type {
  BridgeFileDraft,
  PlanningContextType,
  PlanningConversation,
  PlanningConversationContext,
  PlanningMessage,
  ProjectMarkdownContext,
  ProjectMarkdownContextPayload
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
  focused?: boolean;
  initialConversationId?: number | null;
  onConversationsChanged?: (conversations: PlanningConversation[]) => void;
  startInNewConversation?: boolean;
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

export function PlanningChat({
  project,
  focused = false,
  initialConversationId,
  onConversationsChanged,
  startInNewConversation = false
}: PlanningChatProps) {
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
  const [contextItemsConversationId, setContextItemsConversationId] = useState<number | null>(null);
  const [bridgeDrafts, setBridgeDrafts] = useState<BridgeFileDraft[]>([]);
  const [selectedBridgeDraft, setSelectedBridgeDraft] = useState<BridgeFileDraft | null>(null);
  const [isDraftingBridgeFile, setIsDraftingBridgeFile] = useState(false);
  const [markdownContext, setMarkdownContext] = useState<ProjectMarkdownContext | null>(null);
  const [markdownRootPath, setMarkdownRootPath] = useState("");
  const [markdownReadmePath, setMarkdownReadmePath] = useState("README.md");
  const [markdownPayload, setMarkdownPayload] = useState<ProjectMarkdownContextPayload>({
    files: [],
    warnings: []
  });
  const [isSavingMarkdownContext, setIsSavingMarkdownContext] = useState(false);
  const [isRightPaneOpen, setIsRightPaneOpen] = useState(true);
  const [isMarkdownPaneSectionOpen, setIsMarkdownPaneSectionOpen] = useState(true);
  const [isConversationContextPaneSectionOpen, setIsConversationContextPaneSectionOpen] = useState(true);
  const [isBridgeDraftsPaneSectionOpen, setIsBridgeDraftsPaneSectionOpen] = useState(false);

  const selectedProject = useMemo(
    () => projects.find((project) => project.id === selectedProjectId) ?? null,
    [projects, selectedProjectId]
  );

  const selectedConversation = useMemo(
    () =>
      conversations.find((conversation) => conversation.id === selectedConversationId) ?? null,
    [conversations, selectedConversationId]
  );

  const showConversationSidebar = !focused;

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
      setBridgeDrafts([]);
      setSelectedBridgeDraft(null);
      setMarkdownContext(null);
      setMarkdownRootPath("");
      setMarkdownReadmePath("README.md");
      setMarkdownPayload({ files: [], warnings: [] });
      return;
    }

    Promise.all([
      listPlanningConversations(selectedProjectId),
      listBridgeFileDrafts(selectedProjectId),
      getProjectMarkdownContext(selectedProjectId),
      loadProjectMarkdownContext(selectedProjectId)
    ])
      .then(([nextConversations, nextDrafts, nextMarkdownContext, nextMarkdownPayload]) => {
        setConversations(nextConversations);
        const requestedConversation = initialConversationId
          ? nextConversations.find((conversation) => conversation.id === initialConversationId)
          : null;
        setSelectedConversationId(
          startInNewConversation
            ? null
            : requestedConversation?.id ?? nextConversations[0]?.id ?? null
        );
        setMessages([]);
        setBridgeDrafts(nextDrafts);
        setSelectedBridgeDraft(nextDrafts[0] ?? null);
        setMarkdownContext(nextMarkdownContext);
        setMarkdownRootPath(nextMarkdownContext?.rootPath ?? "");
        setMarkdownReadmePath(nextMarkdownContext?.readmePath ?? "README.md");
        setMarkdownPayload(nextMarkdownPayload);
        setStatus(
          nextConversations.length === 0 ? "No planning conversations yet" : "Ready"
        );
        onConversationsChanged?.(nextConversations);
      })
      .catch((error) => setStatus(formatError(error)));
  }, [selectedProjectId, initialConversationId, startInNewConversation]);

  useEffect(() => {
    if (!initialConversationId) {
      return;
    }

    const conversation = conversations.find((item) => item.id === initialConversationId);
    if (conversation) {
      setSelectedConversationId(conversation.id);
    }
  }, [initialConversationId, conversations]);

  useEffect(() => {
    if (startInNewConversation) {
      setSelectedConversationId(null);
      setMessages([]);
      setContextItems([]);
      setContextItemsConversationId(null);
      setStatus("Ready for a new conversation");
    }
  }, [startInNewConversation]);

  useEffect(() => {
    if (!selectedConversationId) {
      setMessages([]);
      setContextItems([]);
      setContextItemsConversationId(null);
      return;
    }

    setContextItems([]);
    setContextItemsConversationId(null);

    Promise.all([
      listPlanningMessages(selectedConversationId),
      listPlanningConversationContext(selectedConversationId),
      selectedProjectId
        ? loadProjectMarkdownContext(selectedProjectId)
        : Promise.resolve({ files: [], warnings: [] })
    ])
      .then(([nextMessages, nextContext, nextMarkdownPayload]) => {
        setMessages(nextMessages);
        setContextItems(dedupeContextItems(nextContext));
        setContextItemsConversationId(selectedConversationId);
        setMarkdownPayload(nextMarkdownPayload);
        setStatus(nextMessages.length === 0 ? "Conversation ready" : "Ready");
      })
      .catch((error) => setStatus(formatError(error)));
  }, [selectedConversationId, selectedProjectId]);

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

    if (contextItemsConversationId !== selectedConversation.id) {
      return;
    }

    const alreadyAttached = contextItems.some(
      (item) =>
        item.contextType === "github_repository" &&
        (item.sourceId === githubLink.id || item.label === githubLink.repositoryFullName)
    );
    if (alreadyAttached) {
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
          return exists ? dedupeContextItems(current) : dedupeContextItems([...current, attached]);
        });
      })
      .catch((error) => setStatus(formatError(error)));
  }, [selectedConversation?.id, githubLink?.id, contextItems, contextItemsConversationId]);

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
      const nextConversations = [created, ...conversations];
      setConversations(nextConversations);
      onConversationsChanged?.(nextConversations);
      setSelectedConversationId(created.id);
      setMessages([]);
      setContextItems([]);
      setContextItemsConversationId(null);
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
      onConversationsChanged?.(nextConversations);
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
      const nextDrafts = bridgeDrafts.filter(
        (draft) => draft.conversationId !== selectedConversation.id
      );
      setConversations(nextConversations);
      onConversationsChanged?.(nextConversations);
      setSelectedConversationId(nextConversations[0]?.id ?? null);
      setBridgeDrafts(nextDrafts);
      setSelectedBridgeDraft((current) =>
        current?.conversationId === selectedConversation.id ? nextDrafts[0] ?? null : current
      );
      setMessages([]);
      setContextItems([]);
      setContextItemsConversationId(null);
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
      setContextItems((current) => dedupeContextItems([...current, attached]));
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

  async function onSaveMarkdownContext() {
    if (!selectedProject) {
      setStatus("Select a project before configuring Markdown context");
      return;
    }

    const rootPath = markdownRootPath.trim();
    if (!rootPath) {
      setStatus("Markdown context root is required");
      return;
    }

    setIsSavingMarkdownContext(true);
    setStatus("Saving Markdown context root");
    try {
      const saved = await saveProjectMarkdownContext({
        projectId: selectedProject.id,
        rootPath,
        readmePath: markdownReadmePath.trim() || "README.md"
      });
      const payload = await loadProjectMarkdownContext(selectedProject.id);
      setMarkdownContext(saved);
      setMarkdownRootPath(saved.rootPath);
      setMarkdownReadmePath(saved.readmePath);
      setMarkdownPayload(payload);
      setStatus("Markdown context root saved");
    } catch (error) {
      setStatus(formatError(error));
    } finally {
      setIsSavingMarkdownContext(false);
    }
  }

  async function onClearMarkdownContext() {
    if (!selectedProject) {
      return;
    }

    setIsSavingMarkdownContext(true);
    setStatus("Clearing Markdown context root");
    try {
      await deleteProjectMarkdownContext(selectedProject.id);
      setMarkdownContext(null);
      setMarkdownRootPath("");
      setMarkdownReadmePath("README.md");
      setMarkdownPayload({ files: [], warnings: [] });
      setStatus("Markdown context root cleared");
    } catch (error) {
      setStatus(formatError(error));
    } finally {
      setIsSavingMarkdownContext(false);
    }
  }

  async function onReloadMarkdownContext() {
    if (!selectedProject) {
      return;
    }

    setStatus("Reloading Markdown context");
    try {
      const payload = await loadProjectMarkdownContext(selectedProject.id);
      setMarkdownPayload(payload);
      setStatus("Markdown context reloaded");
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  async function onDraftBridgeFile() {
    if (!selectedConversation) {
      setStatus("Create or select a conversation before drafting a bridge file");
      return;
    }

    setIsDraftingBridgeFile(true);
    setStatus("Drafting bridge file");
    try {
      const draft = await createBridgeFileDraftFromConversation(selectedConversation.id);
      setBridgeDrafts((current) => [draft, ...current]);
      setSelectedBridgeDraft(draft);
      setStatus("Bridge draft saved");
    } catch (error) {
      setStatus(formatError(error));
    } finally {
      setIsDraftingBridgeFile(false);
    }
  }

  async function onSelectBridgeDraft(id: number) {
    try {
      const draft = await getBridgeFileDraft(id);
      setSelectedBridgeDraft(draft);
      setStatus("Bridge draft loaded");
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  async function onDeleteBridgeDraft() {
    if (!selectedBridgeDraft) {
      return;
    }

    try {
      await deleteBridgeFileDraft(selectedBridgeDraft.id);
      const nextDrafts = bridgeDrafts.filter((draft) => draft.id !== selectedBridgeDraft.id);
      setBridgeDrafts(nextDrafts);
      setSelectedBridgeDraft(nextDrafts[0] ?? null);
      setStatus("Bridge draft deleted");
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  return (
    <section className={focused ? "planning-panel planning-panel-focused" : "feature-panel planning-panel"}>
      {!focused && (
        <div className="panel-heading">
          <div>
            <p>OpenAI Planning</p>
            <h3>Planning Chat</h3>
          </div>
          <span className={isSending ? "save-pill save-pill-loading" : "save-pill"}>{status}</span>
        </div>
      )}

      <div className={focused ? "planning-layout planning-layout-focused" : "planning-layout"}>
        {showConversationSidebar && (
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

          <section className="markdown-context-config" aria-label="Project Markdown context">
            <div className="markdown-context-heading">
              <div>
                <p>Project Markdown</p>
                <h4>{markdownContext ? "Configured" : "Not configured"}</h4>
              </div>
              <button
                className="ghost-button"
                disabled={!selectedProject || isSavingMarkdownContext}
                onClick={() => void onReloadMarkdownContext()}
                type="button"
              >
                Reload
              </button>
            </div>

            <label className="field-label">
              <span>Root path</span>
              <input
                aria-label="Markdown context root path"
                className="text-input"
                disabled={!selectedProject || isSavingMarkdownContext}
                onChange={(event) => setMarkdownRootPath(event.target.value)}
                placeholder="C:\\Dev\\repos\\overlay-forge"
                value={markdownRootPath}
              />
            </label>

            <label className="field-label">
              <span>README path</span>
              <input
                aria-label="Markdown context README path"
                className="text-input"
                disabled={!selectedProject || isSavingMarkdownContext}
                onChange={(event) => setMarkdownReadmePath(event.target.value)}
                placeholder="README.md"
                value={markdownReadmePath}
              />
            </label>

            <div className="markdown-context-actions">
              <button
                className="primary-button"
                disabled={!selectedProject || isSavingMarkdownContext || markdownRootPath.trim().length === 0}
                onClick={() => void onSaveMarkdownContext()}
                type="button"
              >
                Save
              </button>
              <button
                className="ghost-button"
                disabled={!markdownContext || isSavingMarkdownContext}
                onClick={() => void onClearMarkdownContext()}
                type="button"
              >
                Clear
              </button>
            </div>

            <div className="markdown-context-summary">
              <span>{markdownPayload.files.filter((file) => file.included).length} file(s) loaded</span>
              {markdownPayload.warnings.length > 0 && (
                <span>{markdownPayload.warnings.length} warning(s)</span>
              )}
            </div>
          </section>

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
        )}

        <div
          className={
            focused
              ? isRightPaneOpen
                ? "planning-chat-shell planning-chat-shell-with-right"
                : "planning-chat-shell planning-chat-shell-with-right planning-chat-shell-right-collapsed"
              : "planning-chat-shell"
          }
        >
        <div className="planning-chat-surface">
          {focused && (
            <div className="focused-chat-toolbar">
              <div>
                <p>{selectedProject?.name ?? "Project"}</p>
                <h3>{selectedConversation?.title ?? "Select or create a conversation"}</h3>
              </div>
              <span className={isSending ? "save-pill save-pill-loading" : "save-pill"}>{status}</span>
            </div>
          )}

          {focused && !selectedConversation && (
            <div className="focused-conversation-controls">
              <input
                aria-label="Conversation title"
                className="text-input"
                disabled={!selectedProject || isSending}
                onChange={(event) => setNewConversationTitle(event.target.value)}
                placeholder="New conversation title"
                value={newConversationTitle}
              />
              <button
                className="primary-button"
                disabled={!selectedProject || isSending || newConversationTitle.trim().length === 0}
                onClick={() => void onNewConversation()}
                type="button"
              >
                New
              </button>
            </div>
          )}

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

          {!focused && (
          <section className="project-markdown-panel" aria-label="Project Markdown context sources">
            <div className="project-markdown-heading">
              <div>
                <p>Project Markdown Context</p>
                <h4>Fresh Local Sources</h4>
              </div>
              <button
                className="ghost-button"
                disabled={!selectedProject}
                onClick={() => void onReloadMarkdownContext()}
                type="button"
              >
                Reload
              </button>
            </div>

            <div className="project-markdown-list">
              {markdownPayload.files.length === 0 ? (
                <p>No project Markdown context loaded.</p>
              ) : (
                markdownPayload.files.map((file) => (
                  <div
                    className={
                      file.included
                        ? "project-markdown-item"
                        : "project-markdown-item project-markdown-item-warning"
                    }
                    key={file.relativePath}
                  >
                    <span>{file.relativePath}</span>
                    <strong>{file.included ? "Included" : "Skipped"}</strong>
                    {file.warning && <p>{file.warning}</p>}
                  </div>
                ))
              )}
            </div>

            {markdownPayload.warnings.length > 0 && (
              <div className="project-markdown-warnings">
                {markdownPayload.warnings.slice(0, 3).map((warning) => (
                  <p key={warning}>{warning}</p>
                ))}
              </div>
            )}
          </section>
          )}

          {!focused && (
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
                {isAttachOpen ? "Close Context" : "Conversation Context"}
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

            {isAttachOpen && (
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
            )}
          </section>
          )}

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

          {!focused && (
            <section className="bridge-drafts-panel" aria-label="Bridge drafts">
              <div className="bridge-drafts-heading">
                <div>
                  <p>Bridge Drafts</p>
                  <h4>Local Markdown Drafts</h4>
                </div>
                <button
                  className="ghost-button"
                  disabled={!selectedConversation || isSending || isDraftingBridgeFile}
                  onClick={() => void onDraftBridgeFile()}
                  type="button"
                >
                  Draft Bridge File
                </button>
              </div>

              <div className="bridge-drafts-layout">
                <div className="bridge-drafts-list">
                  {bridgeDrafts.length === 0 ? (
                    <p>No bridge drafts saved for this project.</p>
                  ) : (
                    bridgeDrafts.map((draft) => (
                      <button
                        className={
                          selectedBridgeDraft?.id === draft.id
                            ? "bridge-draft-list-item active"
                            : "bridge-draft-list-item"
                        }
                        key={draft.id}
                        onClick={() => void onSelectBridgeDraft(draft.id)}
                        type="button"
                      >
                        <strong>{draft.title}</strong>
                        <span>{formatDate(draft.updatedAt)}</span>
                      </button>
                    ))
                  )}
                </div>

                <article className="bridge-draft-view">
                  {selectedBridgeDraft ? (
                    <>
                      <div className="bridge-draft-view-heading">
                        <div>
                          <span>{selectedBridgeDraft.status}</span>
                          <strong>{selectedBridgeDraft.title}</strong>
                        </div>
                        <button
                          className="ghost-button"
                          onClick={() => void onDeleteBridgeDraft()}
                          type="button"
                        >
                          Delete
                        </button>
                      </div>
                      <pre>{selectedBridgeDraft.content}</pre>
                    </>
                  ) : (
                    <p>Select or generate a bridge draft to view its Markdown content.</p>
                  )}
                </article>
              </div>
            </section>
          )}
        </div>

        {focused && (
          <aside
            className={
              isRightPaneOpen
                ? "chat-right-pane"
                : "chat-right-pane chat-right-pane-collapsed"
            }
            aria-label="Chat context and drafts"
          >
            <button
              className="chat-right-pane-toggle"
              onClick={() => setIsRightPaneOpen((current) => !current)}
              type="button"
            >
              {isRightPaneOpen ? ">" : "<"}
            </button>

            {isRightPaneOpen && (
              <div className="chat-right-pane-content">
                <div className="chat-right-pane-title">
                  <p>Context References</p>
                  <h4>{selectedProject?.name ?? "Selected Project"}</h4>
                </div>

                <section className="project-markdown-panel" aria-label="Project Markdown context sources">
                  <div className="project-markdown-heading">
                    <div>
                      <p>Project Markdown</p>
                      <h4>Local Repo Context</h4>
                    </div>
                    <div className="right-pane-section-actions">
                      <button
                        className="ghost-button"
                        disabled={!selectedProject}
                        onClick={() => void onReloadMarkdownContext()}
                        type="button"
                      >
                        Reload
                      </button>
                      <button
                        className="ghost-button icon-button"
                        onClick={() => setIsMarkdownPaneSectionOpen((current) => !current)}
                        type="button"
                      >
                        {isMarkdownPaneSectionOpen ? "-" : "+"}
                      </button>
                    </div>
                  </div>

                  {isMarkdownPaneSectionOpen ? (
                    <>
                      <div className="project-markdown-list project-markdown-list-stacked">
                        {markdownPayload.files.length === 0 ? (
                          <p>No project Markdown context loaded.</p>
                        ) : (
                          markdownPayload.files.map((file) => (
                            <div
                              className={
                                file.included
                                  ? "project-markdown-item"
                                  : "project-markdown-item project-markdown-item-warning"
                              }
                              key={file.relativePath}
                            >
                              <span>{file.relativePath}</span>
                              <strong>{file.included ? "Included" : "Skipped"}</strong>
                              {file.warning && <p>{file.warning}</p>}
                            </div>
                          ))
                        )}
                      </div>

                      {markdownPayload.warnings.length > 0 && (
                        <div className="project-markdown-warnings">
                          {markdownPayload.warnings.slice(0, 3).map((warning) => (
                            <p key={warning}>{warning}</p>
                          ))}
                        </div>
                      )}
                    </>
                  ) : (
                    <p className="right-pane-section-summary">
                      {markdownPayload.files.filter((file) => file.included).length} included,
                      {" "}
                      {markdownPayload.files.filter((file) => !file.included).length} skipped
                    </p>
                  )}
                </section>

                <section className="attached-context-panel" aria-label="Attached context">
                  <div className="attached-context-heading">
                    <div>
                      <p>Attached Context</p>
                      <h4>Conversation Links</h4>
                    </div>
                    <div className="right-pane-section-actions">
                      <button
                        className="ghost-button"
                        disabled={!selectedConversation || isSending}
                        onClick={() => setIsAttachOpen((current) => !current)}
                        type="button"
                      >
                        {isAttachOpen ? "Close" : "+ Attach"}
                      </button>
                      <button
                        className="ghost-button icon-button"
                        onClick={() =>
                          setIsConversationContextPaneSectionOpen((current) => !current)
                        }
                        type="button"
                      >
                        {isConversationContextPaneSectionOpen ? "-" : "+"}
                      </button>
                    </div>
                  </div>

                  {isConversationContextPaneSectionOpen && isAttachOpen && (
                    <div className="attach-context-form right-pane-attach-form">
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

                  {isConversationContextPaneSectionOpen ? (
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
                  ) : (
                    <p className="right-pane-section-summary">{contextItems.length} attached</p>
                  )}
                </section>

                <section className="bridge-drafts-panel" aria-label="Bridge drafts">
                  <div className="bridge-drafts-heading">
                    <div>
                      <p>Markdown Drafts</p>
                      <h4>Local Bridge Drafts</h4>
                    </div>
                    <div className="right-pane-section-actions">
                      <button
                        className="ghost-button"
                        disabled={!selectedConversation || isSending || isDraftingBridgeFile}
                        onClick={() => void onDraftBridgeFile()}
                        type="button"
                      >
                        Draft
                      </button>
                      <button
                        className="ghost-button icon-button"
                        onClick={() => setIsBridgeDraftsPaneSectionOpen((current) => !current)}
                        type="button"
                      >
                        {isBridgeDraftsPaneSectionOpen ? "-" : "+"}
                      </button>
                    </div>
                  </div>

                  {isBridgeDraftsPaneSectionOpen ? (
                    <div className="bridge-drafts-layout bridge-drafts-layout-stacked">
                    <div className="bridge-drafts-list">
                      {bridgeDrafts.length === 0 ? (
                        <p>No bridge drafts saved for this project.</p>
                      ) : (
                        bridgeDrafts.map((draft) => (
                          <button
                            className={
                              selectedBridgeDraft?.id === draft.id
                                ? "bridge-draft-list-item active"
                                : "bridge-draft-list-item"
                            }
                            key={draft.id}
                            onClick={() => void onSelectBridgeDraft(draft.id)}
                            type="button"
                          >
                            <strong>{draft.title}</strong>
                            <span>{formatDate(draft.updatedAt)}</span>
                          </button>
                        ))
                      )}
                    </div>

                    <article className="bridge-draft-view">
                      {selectedBridgeDraft ? (
                        <>
                          <div className="bridge-draft-view-heading">
                            <div>
                              <span>{selectedBridgeDraft.status}</span>
                              <strong>{selectedBridgeDraft.title}</strong>
                            </div>
                            <button
                              className="ghost-button"
                              onClick={() => void onDeleteBridgeDraft()}
                              type="button"
                            >
                              Delete
                            </button>
                          </div>
                          <pre>{selectedBridgeDraft.content}</pre>
                        </>
                      ) : (
                        <p>Select or generate a bridge draft to view its Markdown content.</p>
                      )}
                    </article>
                    </div>
                  ) : (
                    <p className="right-pane-section-summary">{bridgeDrafts.length} saved draft(s)</p>
                  )}
                </section>
              </div>
            )}
          </aside>
        )}
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

function contextAttachmentKey(item: PlanningConversationContext) {
  if (item.contextType === "github_repository") {
    return `${item.contextType}:label:${item.label.trim().toLowerCase()}`;
  }

  return item.sourceId === null
    ? `${item.contextType}:label:${item.label.trim().toLowerCase()}`
    : `${item.contextType}:source:${item.sourceId}`;
}

function dedupeContextItems(items: PlanningConversationContext[]) {
  const seen = new Set<string>();
  return items.filter((item) => {
    const key = contextAttachmentKey(item);
    if (seen.has(key)) {
      return false;
    }
    seen.add(key);
    return true;
  });
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

function formatProjectStatus(status: string) {
  return status === "ARCHIVED" ? "Archived" : "Active";
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
