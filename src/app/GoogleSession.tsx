import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ReactNode, useEffect, useState } from "react";
import { ProductLauncher } from "./ProductLauncher";

export interface GoogleProfile { id: string; email: string | null; name: string | null; admin: boolean }
export interface GoogleSession {
  revision: number; sessionId: string; configured: boolean; busy: boolean;
  profile: GoogleProfile | null; detail: string;
}

export function GoogleSessionGate({ children }: { children: (session: GoogleSession) => ReactNode }) {
  const [session, setSession] = useState<GoogleSession>();
  const [error, setError] = useState("");
  useEffect(() => {
    if (new URLSearchParams(window.location.search).get("surface") !== "quick") return;
    const escape = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        void invoke("hide_quick_prompt").catch(cause => setError(String(cause)));
      }
    };
    window.addEventListener("keydown", escape);
    return () => window.removeEventListener("keydown", escape);
  }, []);
  useEffect(() => {
    let active = true;
    let unlisten: (() => void) | undefined;
    const refresh = async () => {
      try {
        const next = await invoke<GoogleSession>("identity_snapshot");
        if (active) setSession(old => !old || next.revision >= old.revision ? next : old);
      } catch (cause) { if (active) setError(String(cause)); }
    };
    void listen("identity-changed", () => void refresh()).then(stop => {
      if (!active) { stop(); return; }
      unlisten = stop;
      void refresh();
    }).catch(cause => { if (active) setError(String(cause)); });
    return () => { active = false; unlisten?.(); };
  }, []);
  const act = async (command: string) => {
    try { await invoke(command, { sessionId: session?.sessionId }); setError(""); }
    catch (cause) { setError(String(cause)); }
  };
  if (!session || session.busy || !session.profile) return <main className="identity-shell">
    <section className="panel"><h1>Overlay Forge</h1><h2>Sign in with Google</h2>
      <p role="status">{session?.detail ?? "Opening your local session…"}</p>
      {error && <p className="error" role="alert">{error}</p>}
      <p>Use your Google account to open the app. ChatGPT is available only to the administrator.</p>
      <p>Sign-in uses your system browser and requests your name and email. Gmail, Calendar and Drive access are separate permissions.</p>
      {session && (session.busy
        ? <button onClick={() => void act("identity_cancel")}>Cancel sign-in</button>
        : <button disabled={!session.configured} onClick={() => void act("identity_sign_in")}>Continue with Google</button>)}
      {session && !session.configured && <p>Administrator setup is required. See the Google sign-in setup in the project documentation.</p>}
      <button onClick={() => void act("shutdown_app")}>Quit Overlay Forge</button>
    </section>
  </main>;
  if (session.profile.admin) return <>{children(session)}</>;
  return <main className="identity-shell">
    <section className="panel"><h1>Overlay Forge</h1><ProfileControls session={session} />
      <p>ChatGPT is administrator-only. Curated media actions are planned and are not available yet.</p>
      </section>
    <ProductLauncher key={session.sessionId} sessionId={session.sessionId} />
    <button onClick={() => void act("shutdown_app")}>Quit Overlay Forge</button>
  </main>;
}

export function ProfileControls({ session }: { session: GoogleSession }) {
  const [error, setError] = useState("");
  async function act(command: string) {
    try { await invoke(command, { sessionId: session.sessionId }); setError(""); }
    catch (cause) { setError(String(cause)); }
  }
  return <section aria-label="Google account">
    <p role="status">{session.detail}</p>
    <p>{session.profile?.name ?? session.profile?.email ?? "Google account"} · {session.profile?.admin ? "Administrator" : "Family member"}</p>
    {session.profile?.name && session.profile.email && <p>{session.profile.email}</p>}
    <div className="actions"><button onClick={() => void act("identity_sign_in")}>Switch Google account</button>
      <button onClick={() => void act("identity_sign_out")}>Sign out of Overlay Forge</button></div>
    {error && <p className="error" role="alert">{error}</p>}
  </section>;
}
