import { useEffect, useRef, useState } from "react";
import { Action, HubClient, Provider, RequestRecord, useHub } from "../ai/hub";
import { RequestCard } from "../ai/RequestCard";
import { ChatgptConnectionCard } from "../ai/ChatgptConnectionCard";
import { StatusCheckSummary } from "../ai/StatusCheckSummary";
import { GoogleSession, GoogleSessionGate, ProfileControls } from "./GoogleSession";
import { AppShell, AppPage } from "./AppShell";
import { ConnectionDashboard } from "./ConnectionDashboard";

const quick = new URLSearchParams(window.location.search).get("surface") === "quick";
const accounts = [
  ["Google primary identity", "Connected · Your verified Google identity controls your Overlay Forge role. Identity alone grants no Gmail or Calendar access."],
  ["Google services", "Disconnected · Gmail read and Calendar scopes require separate supported authorization and actual-grant reconciliation."],
  ["Codex", "Unavailable · Native app-server protocol, workspace permissions, streaming and approval transport required."],
  ["Dot", "Unavailable · No verified native session endpoint or conversation link configured. A new model session is not your Dot. MCP Events remains a feasibility gate."],
];
export default function App() {
  return <GoogleSessionGate>{session => session.profile?.admin
    ? <AdminApp key={session.sessionId} session={session} />
    : <FamilyApp key={session.sessionId} session={session} />}</GoogleSessionGate>;
}
function AdminApp({ session }: { session: GoogleSession }) {
  const [conversationSelection, setConversationSelection] = useState<string | null>();
  const { hub, error, run } = useHub(session.sessionId, conversationSelection);
  const [page, setPage] = useState<AppPage>("Home");
  const [provider, setProvider] = useState<Provider>("chatgpt");
  const [demo, setDemo] = useState(false);
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
  return <AppShell session={session} page={page} onPage={setPage} quick={quick}>
      {(quick || page === "Chat" || page === "Activity & Approvals") && <div className="actions"><button onClick={() => void run("hub_stop_all")}>Stop all work</button></div>}
      {!quick && page === "Connected Accounts" && <ChatgptConnectionCard connection={connection} run={run} />}
      {!hub && !error && <p role="status">Connecting to local hub…</p>}
      {error && <p className="error" role="alert">{error}</p>}
      {hub?.startupMessages.map((message,i) => <p className="error" role="alert" key={i}>{message}</p>)}
      {!quick && page === "Home" && <ConnectionDashboard session={session} connection={connection} onAccounts={() => setPage("Connected Accounts")} />}
      {(quick || page === "Chat") && <section className="panel">
        <div className="row"><span>{HubClient.aggregate(records)}</span></div>
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
      {hub && !quick && page === "Monitoring" && <StatusCheckSummary checks={hub.statusChecks} />}
      {!quick && page === "Connected Accounts" && <section className="panel"><ProfileControls session={session} /><p>OAuth is preferred where supported. App identity, service access, AI disclosure, and action approval are separate permissions.</p>{accounts.map(([title,detail]) => <article className="account" key={title}><strong>{title}</strong><p>{detail}</p></article>)}<p>ChatGPT tokens remain native and platform-protected. ChatGPT access is administrator-only. Codex and Dot remain unavailable.</p></section>}
      {!quick && page === "Monitoring" && <section className="panel"><p>Signing in will not enable monitoring. Rules will require a selected account and resource, purpose, cadence, quiet hours, destination, retention and explicit enablement.</p><p>Durable jobs, retry/backoff, deduplication, cursors, pause and bounded catch-up are not implemented. A local app cannot check accounts while this PC is off. Hosted workers require a separate decision.</p></section>}
      {!quick && page === "Activity & Approvals" && <section className="panel"><p>{HubClient.aggregate(hub?.requests ?? [])}</p><p>Approvals below belong only to the local demo; they cannot grant access or execute external actions.</p>{[...(hub?.requests ?? [])].reverse().map(r => <RequestCard key={r.id} request={r} act={act} />)}{!hub?.requests.length && <p>No request activity yet.</p>}</section>}
      {!quick && page === "Settings" && <section className="panel"><p>Shortcut: Ctrl+Alt+Space. Registration errors appear above. Custom shortcuts are not implemented. Navigation visibility is saved locally.</p><p>Completed ChatGPT conversations are protected and saved on this device. Drafts, unfinished replies and demo activity are cleared on Quit. Each ChatGPT submission includes the text you send and completed turns from the selected conversation.</p><p>ChatGPT and Codex have separate drafts and request histories. Switching provider affects only your next submission.</p><p>Placement uses the cursor display work area on each invocation. Sleep/wake, display-change recovery, and timing targets still require runtime validation.</p></section>}
  </AppShell>;
}

function FamilyApp({ session }: { session: GoogleSession }) {
  const [page, setPage] = useState<AppPage>("Media");
  return <AppShell session={session} page={page} onPage={setPage} quick={quick}>
    {quick ? <p>Open the main app to use Media.</p> : <section className="panel"><ProfileControls session={session} /></section>}
  </AppShell>;
}
