import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
export type Provider = "chatgpt" | "codex";
export type Stage = "queued" | "connecting" | "inProgress" | "thinking" | "tool" | "streaming" | "approval" | "login" | "input" | "resuming" | "stopping" | "complete" | "cancelled" | "failed" | "unknown";
export type Action = "advance" | "approve" | "deny" | "stop" | "acknowledgeStop" | "loseConnection" | "reconcile" | "fail" | "demoOrphan" | "demoActive" | "demoVerifyResources" | "demoClearIndicator" | "cleanup";
export interface ComputerSession {
  captureState: "notStarted" | "active" | "releaseRequested" | "released" | "unknown";
  indicatorState: "notShown" | "visible" | "cleared";
  cleanupReason: "completed" | "cancelled" | "failed" | "stop" | "quit" | null;
  releaseRequested: boolean; remainingResources: number | null; generation: number; sequence: number;
  lastConfirmedAt: number | null; lastCleanupAttemptAt: number | null; demo: boolean;
}
export interface Verification {
  freshness: "localOnly" | "fresh" | "stale" | "unknown" | "unsupported" | "terminal";
  source: "localDemo" | "localCapabilityCheck" | "providerStream";
  lastVerifiedAt: number | null; lastObservedAt: number; staleAfterMs: number;
  reconciliationSupported: boolean;
}
export interface StatusChecks {
  checkedAt: number | null; nextCheckAt: number; intervalMs: number; checkSequence: number;
}
export interface RequestRecord {
  id: number; conversationId: string; provider: Provider; pendingSide: "user" | "assistant" | "neither" | "unknown";
  stage: Stage; blockerKind: string | null; requestedAction: string | null; eventSource: string;
  sequence: number; stateSince: number; lastConfirmedAt: number; terminalOutcome: Stage | null;
  demo: boolean; message: string; response: string; detail: string | null;
  computer: ComputerSession; verification: Verification;
}
export interface ChatgptConnection {
  state: "disconnected" | "authorizing" | "connected" | "limited" | "verifying" | "verified" | "stopping" | "unknown" | "failed";
  accountLabel: string | null; registration: string | null; detail: string;
  models: { slug: string; displayName: string }[]; verificationText: string; planGranted: boolean;
  verification: Verification;
  lastUpdate: number; providerSequence: number | null; correlation: string | null; busy: boolean;
}
export interface ChatConversation { id: string; title: string }
export interface Snapshot { statusChecks: StatusChecks; revision: number; requests: RequestRecord[]; chatgptDraft: string; chatgptConversations: ChatConversation[]; codexDraft: string; startupMessages: string[]; chatgpt: ChatgptConnection | null }
const labels: Record<Stage, string> = {
  queued: "Assistant: queued locally", connecting: "Assistant: connecting", inProgress: "Assistant: in progress",
  thinking: "Assistant: thinking (explicit demo event)",
  tool: "Waiting for the demo tool", streaming: "Assistant: replying", approval: "Waiting for you: approve access",
  login: "Waiting for you: sign in", input: "Waiting for your answer", resuming: "Assistant: resuming",
  stopping: "Stopping: awaiting acknowledgment", complete: "Task finished", cancelled: "Task cancelled", failed: "Task failed: inspect details",
  unknown: "Connection lost: status unknown",
};
export class HubClient {
  private static queue: Promise<unknown> = Promise.resolve();
  static mutate<T = void>(command: string, args?: Record<string, unknown>): Promise<T> {
    const result = this.queue.then(() => invoke<T>(command, args));
    this.queue = result.catch(() => undefined);
    return result;
  }
  static timestamp(at: number | null): string {
    return at === null ? "none" : new Date(at).toLocaleString(undefined, { timeZoneName: "short" });
  }
  static label(request: RequestRecord): string { return labels[request.stage]; }
  static aggregate(requests: RequestRecord[]): string {
    const count = (side: RequestRecord["pendingSide"]) => requests.filter(r => r.pendingSide === side).length;
    return `${count("user")} needs you · ${count("assistant")} working${count("unknown") ? ` · ${count("unknown")} unknown` : ""}`;
  }
}
export function useHub(sessionId: string, conversationId?: string | null) {
  const [hub, setHub] = useState<Snapshot>();
  const [error, setError] = useState("");
  useEffect(() => {
    let active = true;
    let unlisten: (() => void) | undefined;
    const refresh = async () => {
      try {
        const snapshot = await invoke<Snapshot>("hub_snapshot", { conversationId: conversationId ?? null, sessionId });
        if (active) setHub(old => !old || snapshot.revision >= old.revision ? snapshot : old);
      } catch (e) { if (active) setError(String(e)); }
    };
    void listen("hub-changed", () => void refresh()).then(stop => {
      if (!active) { stop(); return; }
      unlisten = stop; void refresh();
    }).catch(e => { if (active) setError(String(e)); });
    return () => { active = false; unlisten?.(); };
  }, [conversationId, sessionId]);
  const run = async <T = void>(command: string, args?: Record<string, unknown>): Promise<T | undefined> => {
    try { const result = await HubClient.mutate<T>(command, { ...args, sessionId }); setError(""); return result; }
    catch (e) { setError(String(e)); }
  };
  return { hub, error, run };
}
