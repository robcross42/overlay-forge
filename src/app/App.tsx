import { useEffect, useRef, useState } from "react";
import { Action, HubClient, Provider, RequestRecord, useHub } from "../ai/hub";
import { RequestCard } from "../ai/RequestCard";
import { ChatgptConnectionCard } from "../ai/ChatgptConnectionCard";
import { StatusCheckSummary } from "../ai/StatusCheckSummary";
import { GoogleSession, GoogleSessionGate, ProfileControls } from "./GoogleSession";
import { ProductLauncher } from "./ProductLauncher";

const pages = ["Home", "Chat", "Connected Accounts", "Monitoring", "Activity & Approvals", "Settings"] as const;
const quick = new URLSearchParams(window.location.search).get("surface") === "quick";
const accounts = [
  ["Google primary identity", "Connected · Your verified Google identity controls your Overlay Forge role. Identity alone grants no Gmail or Calendar access."],
  ["Google services", "Disconnected · Gmail read and Calendar scopes require separate supported authorization and actual-grant reconciliation."],
  ["Codex", "Unavailable · Native app-server protocol, workspace permissions, streaming and approval transport required."],
  ["Dot", "Unavailable · No verified native session endpoint or conversation link configured. A new model session is not your Dot. MCP Events remains a feasibility gate."],
];
export default function App() {
  return <GoogleSessionGate>{session => <AdminApp key={session.sessionId} session={session} />}</GoogleSessionGate>;
}
function AdminApp({ session }: { session: GoogleSession }) {
  const [conversationSelection, setConversationSelection] = useState<string | null>();
  const { hub, error, run } = useHub(session.sessionId, conversationSelection);
  const [page, setPage] = useState<typeof pages[number]>("Home");
  const [provider, setProvider] = useState<Provider>("chatgpt");
  const [demo, setDemo] = useState(false);
  const [quit, setQuit] = useState(false);
  const input = useRef<HTMLTextAreaElement>(null);
  const pendingEdits = useRef(0);
  const [drafts, setDrafts] = useState({ chatgpt: "", codex: "" });
  useEffect(() => {
    if (hub && pendingEdits.current === 0) setDrafts({ chatgpt: hub.chatgptDraft, codex: hub.codexDraft });
  }, [hub]);
  const editDraft = (text: string) => {
    setDrafts(old => ({ ...old, [provider]: text }));
    pendingEdits.current++;
    void run("hub_draft", { provider, text }).finally(() => { pendingEdits.current--; });
  };
  useEffect(() => {
    if (!quick) return;
    const focus = () => input.current?.focus();
    window.addEventListener("focus", focus);
    return () => window.removeEventListener("focus", focus);
  }, []);
  const connection = hub?.chatgpt;
  const liveChat = provider === "chatgpt" && !demo;
  const conversations = hub?.chatgptConversations ?? [];
  const conversationId = conversationSelection === undefined ? (conversations[0]?.id ?? null) : conversationSelection;
  const conversationAvailable = conversationId === null || conversations.some(chat => chat.id === conversationId);
  const records = hub?.requests.filter(r => r.provider === provider && (!liveChat || r.conversationId === conversationId)) ?? [];
  const draft = drafts[provider];
  const canSubmit = !!hub && !!draft.trim() && (!liveChat || (!!connection?.planGranted && !connection.busy && conversationAvailable));
  const act = (r: RequestRecord, action: Action) => void run("hub_action", { input: { id: r.id, sequence: r.sequence, action } });
  const submit = async () => {
    if (!canSubmit) return;
    const submittedConversation = await run<string | null>("hub_submit", {
      provider, demo, text: liveChat ? draft : null, conversationId: liveChat ? conversationId : null,
    });
    if (submittedConversation) {
      setConversationSelection(current => current === conversationSelection ? submittedConversation : current);
    }
  };
  return <main className={quick ? "quick-shell" : "hub-shell"}>
    {!quick && <aside><div className="brand">◈ Overlay Forge</div><p className="muted">Your resident AI anchor</p><nav>{pages.map(p => <button aria-current={page === p ? "page" : undefined} key={p} onClick={() => setPage(p)}>{p}</button>)}</nav><small>Local session / ChatGPT: {hub?.chatgpt?.state ?? "unavailable"}</small><button onClick={() => setQuit(true)}>Quit Overlay Forge</button></aside>}
    <section className="workspace">
      <header className="row"><div><p className="eyebrow">PERSONAL AI HUB · FIRST SLICE</p><h1>{quick ? "Overlay Forge" : page}</h1></div><button onClick={() => void run(quick ? "show_main" : "show_quick_prompt")}>{quick ? "Open main app" : "Open quick prompt"}</button>{quick && <button onClick={() => void run("hide_quick_prompt")}>Hide</button>}</header>
      <div className="actions"><button onClick={() => void run("hub_stop_all")}>Stop all local work / request cleanup</button></div>
      {!quick && (page === "Home" || page === "Connected Accounts") && <ChatgptConnectionCard connection={connection} run={run} />}
      {hub && !quick && page !== "Chat" && <StatusCheckSummary checks={hub.statusChecks} />}
      {!hub && !error && <p role="status">Connecting to local hub…</p>}
      {error && <p className="error" role="alert">{error}</p>}
      {hub?.startupMessages.map((message,i) => <p className="error" role="alert" key={i}>{message}</p>)}
      {!quick && page === "Home" && <section className="summary"><strong>{HubClient.aggregate(hub?.requests ?? [])}</strong><p>Resident session active. Closing a window hides it; use the tray or Ctrl+Alt+Space to return.</p><p>Monitoring is off. ChatGPT connection status is shown separately above.</p><button onClick={() => setPage("Activity & Approvals")}>Inspect all requests and blockers</button></section>}
      {(quick || page === "Home" || page === "Chat") && <section className="panel">
        <div className="row"><h2>{quick ? "Quick prompt" : "Conversations"}</h2><span>{HubClient.aggregate(records)}</span></div>
        <div className="provider-switch" role="group" aria-label="AI provider">{(["chatgpt", "codex"] as const).map(p => <button key={p} aria-pressed={provider === p} onClick={() => setProvider(p)}>{p === "chatgpt" ? "ChatGPT" : "Codex"}</button>)}</div>
        <label className="demo-toggle"><input type="checkbox" checked={demo} onChange={e => setDemo(e.target.checked)} /> Use local demo (no AI or account access)</label>
        {liveChat && <label className="input-label">Conversation <select value={conversationId ?? ""} onChange={e => setConversationSelection(e.target.value || null)}>
          <option value="">New chat</option>
          {!conversationAvailable && <option value={conversationId ?? ""} disabled>Conversation unavailable</option>}
          {conversations.map(chat => <option key={chat.id} value={chat.id}>{chat.title}</option>)}
        </select></label>}
        <label className="input-label" htmlFor="prompt">{provider === "chatgpt" ? "ChatGPT" : "Codex"} draft</label>
        <textarea ref={input} id="prompt" value={draft} disabled={!hub} placeholder="Write a prompt…" onChange={e => editDraft(e.target.value)} onKeyDown={e => { if (e.key === "Enter" && !e.shiftKey && !e.nativeEvent.isComposing) { e.preventDefault(); submit(); } }} />
        <div className="row"><small>Enter sends · Shift+Enter adds a line{quick ? " · Escape hides" : ""}</small><button disabled={!canSubmit} onClick={submit}>{demo ? "Start local demo" : liveChat ? "Send to ChatGPT" : "Check provider availability"}</button></div>
        {liveChat && <p className="muted" role="status">{!conversationAvailable ? "Choose New chat or another conversation to send your draft." : connection?.busy ? "ChatGPT is busy. You can draft your next prompt while it runs." : connection?.planGranted ? "Uses your ChatGPT plan. Follow-ups include completed turns from this conversation." : "Connect ChatGPT and grant plan use to send."}</p>}
        {!demo && provider === "codex" && <p className="muted">Codex transport is unavailable. Your draft stays local.</p>}
        {[...records].reverse().map(r => <RequestCard key={r.id} request={r} act={act} />)}{!records.length && <p className="empty" role="status">Ready · No active request for this provider.</p>}
      </section>}
      {(quick || page === "Chat") && <ChatgptConnectionCard connection={connection} run={run} />}
      {hub && (quick || page === "Chat") && <StatusCheckSummary checks={hub.statusChecks} />}
      {!quick && page === "Connected Accounts" && <section className="panel"><h2>Accounts & capabilities</h2><ProfileControls session={session} /><p>OAuth is preferred where supported. App identity, service access, AI disclosure, and action approval are separate permissions.</p>{accounts.map(([title,detail]) => <article className="account" key={title}><h3>{title}</h3><p>{detail}</p></article>)}<p>ChatGPT tokens remain native and platform-protected. ChatGPT access is administrator-only. Codex and Dot remain unavailable.</p></section>}
      {!quick && page === "Monitoring" && <section className="panel"><h2>Monitoring is not running</h2><p>Signing in will not enable monitoring. Rules will require a selected account and resource, purpose, cadence, quiet hours, destination, retention and explicit enablement.</p><p>Durable jobs, retry/backoff, deduplication, cursors, pause and bounded catch-up are not implemented. A local app cannot check accounts while this PC is off. Hosted workers require a separate decision.</p></section>}
      {!quick && page === "Activity & Approvals" && <section className="panel"><h2>All request activity</h2><p>{HubClient.aggregate(hub?.requests ?? [])}</p><p>Approvals below belong only to the local demo; they cannot grant access or execute external actions.</p>{[...(hub?.requests ?? [])].reverse().map(r => <RequestCard key={r.id} request={r} act={act} />)}{!hub?.requests.length && <p>No request activity yet.</p>}</section>}
      {!quick && page === "Settings" && <section className="panel"><h2>Session & privacy</h2><p>Shortcut: Ctrl+Alt+Space. Registration errors appear above. Custom shortcuts and durable preferences are not implemented.</p><p>Completed ChatGPT conversations are protected and saved on this device. Drafts, unfinished replies and demo activity are cleared on Quit. Each ChatGPT submission includes the text you send and completed turns from the selected conversation.</p><p>ChatGPT and Codex have separate drafts and request histories. Switching provider affects only your next submission.</p><p>Placement uses the cursor display work area on each invocation. Sleep/wake, display-change recovery, and timing targets still require runtime validation.</p></section>}
      {!quick && page === "Home" && <><section className="panel"><ProfileControls session={session} /></section><ProductLauncher sessionId={session.sessionId} /></>}
      {quit && <section className="panel" role="alertdialog" aria-label="Quit Overlay Forge"><h2>Quit Overlay Forge?</h2><p>Completed ChatGPT conversations are saved on this device. Drafts, unfinished replies and demo activity clear on exit.</p><button onClick={() => void run("shutdown_app")}>Quit Overlay Forge</button><button onClick={() => setQuit(false)}>Keep running</button></section>}
    </section>
  </main>;
}
