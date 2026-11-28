import { useState, type FormEvent } from "react";
import type { BrowserClient, LocalChatMessage } from "./client";

import { recentContext } from "./chatContext";

export function LocalLlmChat({ client, csrf }: { client: BrowserClient; csrf: string }) {
  const [transcript, setTranscript] = useState<LocalChatMessage[]>([]);
  const [draft, setDraft] = useState("");
  const [pending, setPending] = useState<LocalChatMessage>();
  const [error, setError] = useState("");

  async function send(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const content = draft.trim();
    if (!content || pending) return;

    const prompt: LocalChatMessage = { role: "user", content };
    setPending(prompt);
    setError("");
    try {
      const reply = await client.localChat(csrf, recentContext(transcript, prompt));
      setTranscript([...transcript, prompt, { role: "assistant", content: reply.content }]);
      setDraft("");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "The local chat request failed.");
    } finally {
      setPending(undefined);
    }
  }

  function newChat() {
    setTranscript([]);
    setDraft("");
    setError("");
  }

  return <section className="panel">
    <header className="row">
      <h1>Local LLM Chat</h1>
      <button type="button" onClick={newChat} disabled={!!pending}>New Chat</button>
    </header>
    <div role="log" aria-label="Conversation">
      {(pending ? [...transcript, pending] : transcript).map((message, index) =>
        <article key={index} className={message.role === "user" ? "request side-user" : "request"}>
          <h2>{message.role === "user" ? "You" : "Qwen"}</h2>
          <div className="prompt-text">{message.content}</div>
        </article>
      )}
    </div>
    {pending && <p role="status">Waiting for Qwen…</p>}
    {error && <p className="error" role="alert">{error}</p>}
    <form onSubmit={send}>
      <label className="input-label" htmlFor="local-llm-message">Message</label>
      <textarea id="local-llm-message" value={draft} onChange={event => setDraft(event.target.value)}
        maxLength={4000} readOnly={!!pending} />
      <button type="submit" disabled={!!pending || !draft.trim()}>Send</button>
    </form>
  </section>;
}
