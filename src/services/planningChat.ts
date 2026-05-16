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
