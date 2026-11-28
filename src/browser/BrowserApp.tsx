import { useEffect, useMemo, useState } from "react";
import { EmbeddedMedia } from "../app/EmbeddedMedia";
import { BrowserClient, type BrowserSession } from "./client";
import { NavigationToggle } from "../app/NavigationToggle";
import { LocalLlmChat } from "./LocalLlmChat";

const pageLabels = { media: "Media", account: "Account", localChat: "Local LLM Chat" };

export function BrowserApp() {
  const [session, setSession] = useState<BrowserSession | null>();
  const [selectedPage, setPage] = useState<keyof typeof pageLabels>("media");
  const [endUserView, setEndUserView] = useState(false);
  const adminView = !!session?.administrator && !endUserView;
  const localLlmVisible = adminView && !!session?.localLlmEnabled;
  const page = selectedPage === "localChat" && !localLlmVisible ? "media" : selectedPage;
  const [error, setError] = useState(() => new URLSearchParams(window.location.search).get("auth") === "denied"
    ? "Sign-in was unsuccessful or this email has not been approved. Contact the app owner." : "");
  const [busy, setBusy] = useState(false);
  const client = useMemo(() => new BrowserClient(() => setSession(null)), []);
  const transport = useMemo(() => session ? client.mediaTransport(session.csrfToken) : undefined, [client, session?.csrfToken]);
  useEffect(() => {
    let active = true;
    const refresh = () => {
      void client.session().then(value => { if (active) setSession(value); })
        .catch(() => { if (active) { setSession(null); setError("Browser server unavailable. Use the server URL, not the desktop development preview."); } });
    };
    refresh();
    window.addEventListener("focus", refresh);
    if (new URLSearchParams(window.location.search).has("auth")) window.history.replaceState(null, "", window.location.pathname);
    return () => { active = false; window.removeEventListener("focus", refresh); };
  }, [client]);
  useEffect(() => {
    setEndUserView(false);
  }, [session?.csrfToken]);
  function switchView() {
    setEndUserView(!endUserView);
    if (!endUserView && selectedPage !== "media" && selectedPage !== "account") setPage("media");
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
  return <main className={`hub-shell${session.navigationCollapsed ? " navigation-collapsed" : ""}`}>
    <aside id="browser-navigation" hidden={session.navigationCollapsed}>
      <nav aria-label="Main navigation">
        <button aria-current={page === "media" ? "page" : undefined} onClick={() => setPage("media")}>Media</button>
        {localLlmVisible && <button aria-current={page === "localChat" ? "page" : undefined}
          onClick={() => setPage("localChat")}>{pageLabels.localChat}</button>}
        <button aria-current={page === "account" ? "page" : undefined} onClick={() => setPage("account")}>Account</button>
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
      {page === "account" && <section className="panel">
        <p>{session.name ?? session.email}</p><p>{session.email}</p>
        <button disabled={busy} onClick={() => void signOut()}>Sign out</button>
      </section>}
    </section></div>
  </main>;
}
