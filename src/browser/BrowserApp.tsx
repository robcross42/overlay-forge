import { useCallback, useEffect, useMemo, useState } from "react";
import { EmbeddedMedia } from "../app/EmbeddedMedia";
import { BrowserClient, type BrowserSession, type ChatgptStatus } from "./client";
import { NavigationToggle } from "../app/NavigationToggle";
import { LocalLlmChat } from "./LocalLlmChat";
import { BrowserAdminHome, BrowserMonitoring, type Integration } from "./BrowserAdminPages";
import { ChatgptBrowser } from "./ChatgptBrowser";

const pageLabels = { home: "Home", media: "Media", chatgpt: "ChatGPT Chat", monitoring: "Monitoring", account: "Account", localChat: "Local LLM Chat" };

export function BrowserApp() {
  const [session, setSession] = useState<BrowserSession | null>();
  const [selectedPage, setPage] = useState<keyof typeof pageLabels>(() =>
    new URLSearchParams(window.location.search).has("chatgpt") ? "chatgpt" : "home");
  const [endUserView, setEndUserView] = useState(false);
  const adminView = !!session?.administrator && !endUserView;
  const localLlmVisible = adminView && !!session?.localLlmEnabled;
  const chatgptVisible = adminView && !!session?.chatgptEnabled;
  const page = !adminView || (selectedPage === "localChat" && !localLlmVisible) ||
    (selectedPage === "chatgpt" && !chatgptVisible) ? "media" : selectedPage;
  const [chatgpt, setChatgpt] = useState<ChatgptStatus>();
  const [chatgptError, setChatgptError] = useState(() =>
    new URLSearchParams(window.location.search).get("chatgpt") === "denied" ? "The ChatGPT sign-in callback could not be verified." : "");
  const [error, setError] = useState(() => new URLSearchParams(window.location.search).get("auth") === "denied"
    ? "Sign-in was unsuccessful or this email has not been approved. Contact the app owner." : "");
  const [busy, setBusy] = useState(false);
  const client = useMemo(() => new BrowserClient(() => setSession(null)), []);
  const transport = useMemo(() => session ? client.mediaTransport(session.csrfToken) : undefined, [client, session?.csrfToken]);
  const csrf = chatgptVisible ? session?.csrfToken : undefined;
  const [statusRevision, setStatusRevision] = useState(0);
  const refreshChatgpt = useCallback(() => setStatusRevision((value) => value + 1), []);
  useEffect(() => {
    let current = true;
    if (!csrf) {
      setChatgpt(undefined);
      return;
    }
    void client.chatgptStatus(csrf).then((value) => {
      if (current) setChatgpt(value);
    }).catch((cause: unknown) => {
      if (current) setChatgptError(cause instanceof Error ? cause.message : "ChatGPT status is unavailable.");
    });
    return () => { current = false; };
  }, [client, csrf, page, statusRevision]);
  useEffect(() => {
    let active = true;
    const refresh = () => {
      void client.session().then(value => { if (active) setSession(value); })
        .catch(() => { if (active) { setSession(null); setError("Browser server unavailable. Use the server URL, not the desktop development preview."); } });
    };
    refresh();
    window.addEventListener("focus", refresh);
    const parameters = new URLSearchParams(window.location.search);
    if (parameters.has("auth") || parameters.has("chatgpt")) window.history.replaceState(null, "", window.location.pathname);
    return () => { active = false; window.removeEventListener("focus", refresh); };
  }, [client]);
  useEffect(() => {
    setEndUserView(false);
  }, [session?.csrfToken]);
  function switchView() {
    setEndUserView(!endUserView);
    setPage(endUserView ? "home" : "media");
  }
  async function signIn() {
    setBusy(true);
    try { await client.signIn(); } catch (cause) { setError(cause instanceof Error ? cause.message : "Could not start sign-in."); setBusy(false); }
  }
  async function toggleNavigation() {
    if (!session || busy) return;
    const previous = session.navigationCollapsed;
    setBusy(true);
    setSession({ ...session, navigationCollapsed: !previous });
    try { await client.saveNavigation(session.csrfToken, !previous); setError(""); }
    catch { setSession(current => current ? { ...current, navigationCollapsed: previous } : null); setError("Could not save navigation preference."); }
    finally { setBusy(false); }
  }
  async function signOut() {
    if (!session) return;
    setBusy(true);
    try { await client.logout(session.csrfToken); setSession(null); setError(""); }
    catch { setError("Could not sign out. Please try again."); }
    finally { setBusy(false); }
  }
  if (!session) return <main className="identity-shell"><section className="panel" aria-label="Sign in">
    {session === undefined ? <p role="status">Opening your session…</p> : <>
      <p>Sign in with your approved Google account.</p>
      <button disabled={busy} onClick={() => void signIn()}>Continue with Google</button>
    </>}
    {error && <p className="error" role="alert">{error}</p>}
  </section></main>;
  const integrations: Integration[] = [
    { name: "Google", detail: "Signed in with an approved Google account.", open: () => setPage("account") },
    { name: "Media", detail: "Shared media library available.", open: () => setPage("media") },
    { name: "ChatGPT", detail: chatgpt?.detail ?? (chatgptVisible ? "Checking connection…" : "Browser ChatGPT is unavailable."), open: chatgptVisible ? () => setPage("chatgpt") : undefined },
    { name: "Local LLM Chat", detail: localLlmVisible ? "Configured for the Ubuntu local model." : "Unavailable.", open: localLlmVisible ? () => setPage("localChat") : undefined },
  ];
  return <main className={`hub-shell${session.navigationCollapsed ? " navigation-collapsed" : ""}`}>
    <aside id="browser-navigation" hidden={session.navigationCollapsed}>
      <nav aria-label="Main navigation">
        {adminView && <button aria-current={page === "home" ? "page" : undefined} onClick={() => setPage("home")}>Home</button>}
        <button aria-current={page === "media" ? "page" : undefined} onClick={() => setPage("media")}>Media</button>
        {localLlmVisible && <button aria-current={page === "localChat" ? "page" : undefined}
          onClick={() => setPage("localChat")}>{pageLabels.localChat}</button>}
        {chatgptVisible && <button aria-current={page === "chatgpt" ? "page" : undefined} onClick={() => setPage("chatgpt")}>ChatGPT Chat</button>}
        {adminView && <button aria-current={page === "monitoring" ? "page" : undefined} onClick={() => setPage("monitoring")}>Monitoring</button>}
        {adminView && <button aria-current={page === "account" ? "page" : undefined} onClick={() => setPage("account")}>Account</button>}
      </nav>
      {session.administrator && <button type="button" onClick={switchView}>
        {endUserView ? "Switch to Admin view" : "Switch to End User view"}
      </button>}
      <button disabled={busy} onClick={() => void signOut()}>Sign out</button>
    </aside>
    <NavigationToggle collapsed={session.navigationCollapsed} disabled={busy}
      controls="browser-navigation" onToggle={() => void toggleNavigation()} />
    <div className="workspace-scroll"><section className={`workspace${page === "media" ? " workspace-media" : ""}`} aria-label={pageLabels[page]}>
      {error && <p className="error" role="alert">{error}</p>}
      {page === "media" && transport && <EmbeddedMedia sessionId="browser" administrator={false} transport={transport} />}
      {localLlmVisible && <div hidden={page !== "localChat"}>
        <LocalLlmChat key={session.csrfToken} client={client} csrf={session.csrfToken} />
      </div>}
      {page === "home" && <BrowserAdminHome integrations={integrations} />}
      {page === "monitoring" && <BrowserMonitoring chatgpt={chatgpt} error={chatgptError} refresh={refreshChatgpt} />}
      {csrf && <div hidden={page !== "chatgpt"}>
        <ChatgptBrowser key={csrf} client={client} csrf={csrf} status={chatgpt} refresh={refreshChatgpt} statusError={chatgptError} />
      </div>}
      {page === "account" && <section className="panel">
        <p>{session.name ?? session.email}</p><p>{session.email}</p>
        <button disabled={busy} onClick={() => void signOut()}>Sign out</button>
      </section>}
    </section></div>
  </main>;
}
