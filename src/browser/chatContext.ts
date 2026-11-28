import type { LocalChatMessage } from "./client";

export function recentContext(transcript: LocalChatMessage[], prompt: LocalChatMessage): LocalChatMessage[] {
  const messages = [...transcript.slice(-12), prompt];
  const encoder = new TextEncoder();
  while (encoder.encode(JSON.stringify({ messages })).byteLength > 60 * 1024) {
    messages.splice(0, 2);
  }
  return messages;
}
