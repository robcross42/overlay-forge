import { invoke } from "@tauri-apps/api/core";

export type PlanningConversation = {
  id: number;
  projectId: number;
  title: string;
  createdAt: string;
  updatedAt: string;
};

export type PlanningMessageRole = "user" | "assistant" | "system";

export type PlanningMessage = {
  id: number;
  conversationId: number;
  role: PlanningMessageRole;
  content: string;
  createdAt: string;
};

export type PlanningContextType =
  | "project"
  | "github_repository"
  | "note"
  | "task"
  | "calendar_event"
  | "youtube_reference"
  | "scratchpad";

export type PlanningConversationContext = {
  id: number;
  conversationId: number;
  contextType: PlanningContextType;
  sourceId: number | null;
  label: string;
  createdAt: string;
};

export type PromptPreviewContextItem = {
  id: number;
  contextType: PlanningContextType;
  label: string;
  included: boolean;
  content: string;
  warning: string;
};

export type PlanningPromptPreview = {
  projectLabel: string;
  projectStatus: string;
  projectDescription: string;
  conversationLabel: string;
  messageCount: number;
  draftMessage: string;
  attachedContextItems: PromptPreviewContextItem[];
  assembledPrompt: string;
  warnings: string[];
};

export type BridgeFileDraft = {
  id: number;
  projectId: number;
  conversationId: number;
  title: string;
  content: string;
  status: string;
  createdAt: string;
  updatedAt: string;
};

export function listPlanningConversations(projectId?: number) {
  return invoke<PlanningConversation[]>("list_planning_conversations", {
    projectId: projectId ?? null
  });
}

export function createPlanningConversation(projectId: number, title?: string) {
  return invoke<PlanningConversation>("create_planning_conversation", {
    projectId,
    title: title ?? null
  });
}

export function listPlanningMessages(conversationId: number) {
  return invoke<PlanningMessage[]>("list_planning_messages", { conversationId });
}

export function sendPlanningMessage(conversationId: number, content: string) {
  return invoke<PlanningMessage[]>("send_planning_message", { conversationId, content });
}

export function deletePlanningConversation(conversationId: number) {
  return invoke<void>("delete_planning_conversation", { conversationId });
}

export function listPlanningConversationContext(conversationId: number) {
  return invoke<PlanningConversationContext[]>("list_planning_conversation_context", {
    conversationId
  });
}

export function attachPlanningConversationContext(input: {
  conversationId: number;
  contextType: PlanningContextType;
  sourceId?: number | null;
  label: string;
}) {
  return invoke<PlanningConversationContext>("attach_planning_conversation_context", {
    conversationId: input.conversationId,
    contextType: input.contextType,
    sourceId: input.sourceId ?? null,
    label: input.label
  });
}

export function removePlanningConversationContext(id: number) {
  return invoke<void>("remove_planning_conversation_context", { id });
}

export function previewPlanningChatPrompt(conversationId: number, draftMessage: string) {
  return invoke<PlanningPromptPreview>("preview_planning_chat_prompt", {
    conversationId,
    draftMessage
  });
}

export function listBridgeFileDrafts(projectId: number) {
  return invoke<BridgeFileDraft[]>("list_bridge_file_drafts", { projectId });
}

export function getBridgeFileDraft(id: number) {
  return invoke<BridgeFileDraft>("get_bridge_file_draft", { id });
}

export function createBridgeFileDraftFromConversation(conversationId: number) {
  return invoke<BridgeFileDraft>("create_bridge_file_draft_from_conversation", {
    conversationId
  });
}

export function deleteBridgeFileDraft(id: number) {
  return invoke<void>("delete_bridge_file_draft", { id });
}
