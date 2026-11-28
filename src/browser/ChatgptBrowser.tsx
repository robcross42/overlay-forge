import { useState, type FormEvent } from "react";
import { recentContext } from "./chatContext";
import type { BrowserClient, ChatgptStatus, LocalChatMessage } from "./client";

export function ChatgptBrowser({ client, csrf, status, refresh, statusError }: {
  client: BrowserClient;
  csrf: string;
  status?: ChatgptStatus;
  refresh: () => void;
  statusError: string;
}) {
  const [transcript, setTranscript] = useState<LocalChatMessage[]>([]);
  const [draft, setDraft] = useState("");
  const [modelChoice, setModelChoice] = useState("");
  const [pending, setPending] = useState<LocalChatMessage>();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const models = status?.models ?? [];
  const model = models.find((entry) => entry.slug === modelChoice) ?? models[0];
  const connecting = status?.state === "awaitingConsent" || status?.state === "authorizing";

  async function action(operation: () => Promise<void>) {
    setBusy(true);
    setError("");
    try {
      await operation();
      refresh();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "The ChatGPT action failed.");
    } finally {
      setBusy(false);
    }
  }

  async function send(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const content = draft.trim();
    if (!content || !model || pending || busy || connecting) return;
    const prompt: LocalChatMessage = { role: "user", content };
    setPending(prompt);
    setError("");
    try {
      const reply = await client.chatgptReply(csrf, model.slug, recentContext(transcript, prompt));
      setTranscript((current) => [...current, prompt, { role: "assistant", content: reply.content }]);
      setDraft("");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "The ChatGPT reply failed.");
    } finally {
      setPending(undefined);
      refresh();
    }
  }

  return <section className="panel">
    <header className="row">
      <h1>ChatGPT</h1>
      {model && <button type="button" disabled={!!pending || busy} onClick={() => {
        setTranscript([]); setDraft(""); setError("");
      }}>New Chat</button>}
    </header>
    <p role="status">{status?.detail ?? "Checking the ChatGPT connection…"}</p>
    {status?.account && <p>Connected account: {status.account}</p>}
    <div className="actions">
      <button type="button" disabled={busy || !!pending || status?.state === "authorizing"}
        onClick={() => void action(() => client.chatgptSignIn(csrf))}>
        {model ? "Reconnect ChatGPT" : "Continue with ChatGPT"}
      </button>
      {(status?.account || model || connecting) && <button type="button" disabled={busy || !!pending || status?.state === "authorizing"}
        onClick={() => void action(async () => {
          if (connecting) {
            await client.chatgptCancelSignIn(csrf);
            return;
          }
          await client.chatgptDisconnect(csrf);
          setTranscript([]); setDraft("");
        })}>
        {connecting ? "Cancel sign-in" : "Disconnect ChatGPT"}
      </button>}
    </div>
    {(error || statusError) && <p className="error" role="alert">{error || statusError}</p>}
    {model && <>
      <label className="input-label" htmlFor="browser-chatgpt-model">Model</label>
      <select id="browser-chatgpt-model" value={model.slug} disabled={!!pending}
        onChange={(event) => setModelChoice(event.target.value)}>
        {models.map((entry) => <option key={entry.slug} value={entry.slug}>{entry.displayName}</option>)}
      </select>
      <div role="log" aria-label="ChatGPT conversation">
        {(pending ? [...transcript, pending] : transcript).map((message, index) => <article key={index}
          className={message.role === "user" ? "request side-user" : "request"}>
          <h2>{message.role === "user" ? "You" : "ChatGPT"}</h2>
          <div className="prompt-text">{message.content}</div>
        </article>)}
      </div>
      {pending && <p role="status">Waiting for ChatGPT…</p>}
      <form onSubmit={send}>
        <label className="input-label" htmlFor="browser-chatgpt-message">Message</label>
        <textarea id="browser-chatgpt-message" value={draft} onChange={(event) => setDraft(event.target.value)}
          maxLength={4000} readOnly={!!pending} />
        <button type="submit" disabled={!!pending || busy || connecting || !draft.trim()}>Send</button>
        {pending && <button type="button" disabled={busy}
          onClick={() => void action(() => client.chatgptStop(csrf))}>Stop</button>}
      </form>
    </>}
  </section>;
}
