import { useObservedComcCard } from "./comcSales";
import { useEffect, useMemo, useState } from "react";
import { ComcMarket } from "../../modules/hockey-cards/src/ComcMarket";
import { ComcInventory } from "../../modules/hockey-cards/src/ComcInventory";
import { EmbeddedMedia } from "../app/EmbeddedMedia";
import { BrowserClient, type BrowserSession } from "./client";
import { NavigationToggle } from "../app/NavigationToggle";
import { LocalLlmChat } from "./LocalLlmChat";

const pageLabels = { media: "Media", account: "Account", hockey: "Hockey Cards", localChat: "Local LLM Chat" };

export function BrowserApp() {
  const observedComcCard = useObservedComcCard();
  const [session, setSession] = useState<BrowserSession | null>();
  const [selectedPage, setPage] = useState<keyof typeof pageLabels>("media");
  const page = (selectedPage === "localChat" && !session?.localLlmEnabled) ||
    (selectedPage === "hockey" && !session?.hockeyInventoryEnabled) ? "media" : selectedPage;
  const [error, setError] = useState(() => new URLSearchParams(window.location.search).get("auth") === "denied"
    ? "Sign-in was unsuccessful or this email has not been approved. Contact the app owner." : "");
  const [busy, setBusy] = useState(false);
  const client = useMemo(() => new BrowserClient(() => setSession(null)), []);
  const transport = useMemo(() => session ? client.mediaTransport(session.csrfToken) : undefined, [client, session?.csrfToken]);
  const hockeyInventory = useMemo(() => session?.hockeyInventoryEnabled
    ? client.hockeyInventoryTransport(session.csrfToken) : undefined,
  [client, session?.hockeyInventoryEnabled, session?.csrfToken]);
  const hockeyMarket = useMemo(() => session?.hockeyMarketEnabled
    ? client.hockeyMarketTransport(session.csrfToken) : undefined,
  [client, session?.hockeyMarketEnabled, session?.csrfToken]);
  const hockeySales = useMemo(() => session?.hockeySalesEnabled
    ? client.hockeySalesTransport(session.csrfToken) : undefined,
  [client, session?.hockeySalesEnabled, session?.csrfToken]);
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
        {hockeyInventory && <button aria-current={page === "hockey" ? "page" : undefined}
          onClick={() => setPage("hockey")}>{pageLabels.hockey}</button>}
        {session.localLlmEnabled && <button aria-current={page === "localChat" ? "page" : undefined}
          onClick={() => setPage("localChat")}>{pageLabels.localChat}</button>}
        <button aria-current={page === "account" ? "page" : undefined} onClick={() => setPage("account")}>Account</button>
      </nav>
      <button disabled={busy} onClick={() => void signOut()}>Sign out</button>
    </aside>
    <NavigationToggle collapsed={session.navigationCollapsed} disabled={busy}
      controls="browser-navigation" onToggle={() => void toggleNavigation()} />
    <div className="workspace-scroll"><section className={`workspace${page === "media" ? " workspace-media" : ""}`} aria-label={pageLabels[page]}>
      {error && <p className="error" role="alert">{error}</p>}
      {page === "media" && transport && <EmbeddedMedia sessionId="browser" administrator={false} transport={transport} />}
      {page === "hockey" && hockeySales && <ComcMarket key={"market-" + session.csrfToken} market={hockeyMarket} sales={hockeySales} observedCard={observedComcCard} importSales={client.hockeySalesCapture(session.csrfToken)} />}
      {page === "hockey" && hockeyInventory && <ComcInventory key={session.csrfToken} load={hockeyInventory} importCapture={session.hockeyInventoryCanImport ? client.hockeyInventoryCapture(session.csrfToken) : undefined} />}
      {session.localLlmEnabled && <div hidden={page !== "localChat"}>
        <LocalLlmChat key={session.csrfToken} client={client} csrf={session.csrfToken} />
      </div>}
      {page === "account" && <section className="panel">
        <p>{session.name ?? session.email}</p><p>{session.email}</p>
        <button disabled={busy} onClick={() => void signOut()}>Sign out</button>
      </section>}
    </section></div>
  </main>;
}
