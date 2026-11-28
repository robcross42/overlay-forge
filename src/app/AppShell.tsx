import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ReactNode, useEffect, useState } from "react";
import type { GoogleSession } from "./GoogleSession";
import { useNavigationPreference } from "./useNavigationPreference";
import { EmbeddedMedia } from "./EmbeddedMedia";
import { EmbeddedHockeyInventory } from "./EmbeddedHockeyInventory";
import { NavigationToggle } from "./NavigationToggle";
import { ProjectsModule } from "../projects/ProjectsModule";

export const adminPages = ["Home", "Projects", "Hockey Cards", "Chat", "Connected Accounts", "Monitoring", "Activity & Approvals", "Settings"] as const;
export type AppPage = typeof adminPages[number] | "Media";
interface Product { key: string; name: string; available: boolean }

export function AppShell({ session, page, onPage, quick, children }: {
  session: GoogleSession; page: AppPage; onPage: (page: AppPage) => void;
  quick: boolean; children: ReactNode;
}) {
  const navigation = useNavigationPreference(session.sessionId, !quick);
  const [productRevision, setProductRevision] = useState(0);
  const [products, setProducts] = useState<Product[]>([]);
  const [error, setError] = useState("");
  const [confirmQuit, setConfirmQuit] = useState(false);
  useEffect(() => {
    if (quick) return;
    let active = true;
    void invoke<Product[]>("list_products").then(items => { if (active) setProducts(items); })
      .catch(() => { if (active) setError("Could not load modules."); });
    const subscription = listen<string>("product-selected", event => {
      if (active && event.payload === "media") onPage("Media");
    });
    return () => { active = false; void subscription.then(stop => stop()).catch(() => undefined); };
  }, [quick, onPage, productRevision]);
  async function selectProduct(product: Product) {
    if (product.key === "media") { onPage("Media"); return; }
    try { await invoke("launch_product", { productKey: product.key, sessionId: session.sessionId }); setError(""); }
    catch { setError(`Could not open ${product.name}.`); }
  }
  const administrator = !!session.profile?.admin;
  let content = children;
  if (!quick && page === "Media") {
    content = <EmbeddedMedia sessionId={session.sessionId} administrator={administrator} />;
  }
  if (!quick && page === "Projects") {
    content = administrator && <ProjectsModule key={session.sessionId} sessionId={session.sessionId} />;
  }
  if (!quick && page === "Hockey Cards") {
    content = administrator && <EmbeddedHockeyInventory key={session.sessionId} sessionId={session.sessionId} />;
  }
  return <main className={quick ? "quick-shell" : `hub-shell${navigation.collapsed ? " navigation-collapsed" : ""}`}>
    {!quick && <aside id="main-navigation" hidden={navigation.collapsed}>
      <nav aria-label="Main navigation">
        {(session.profile?.admin ? adminPages : ["Settings"] as const).map(item => <button key={item}
          aria-current={page === item ? "page" : undefined} onClick={() => onPage(item)}>{item}</button>)}
        <div className="module-navigation">{products.map(product => <button key={product.key}
          aria-current={page === "Media" && product.key === "media" ? "page" : undefined}
          disabled={!product.available} onClick={() => void selectProduct(product)}>{product.name}</button>)}</div>
      </nav>
      <button aria-label="Refresh modules" title="Refresh modules" onClick={() => setProductRevision(value => value + 1)}>↻</button>
      <button onClick={() => setConfirmQuit(true)}>Quit</button>
    </aside>}
    {!quick && <NavigationToggle collapsed={navigation.collapsed} disabled={navigation.disabled}
      controls="main-navigation" onToggle={() => void navigation.toggle()} />}
    <div className="workspace-scroll">
      <section className={`workspace${page === "Media" && !quick ? " workspace-media" : ""}`} aria-label={quick ? "Quick prompt" : page}>
        <div className="workspace-tools">
          {(quick || session.profile?.admin) && <button onClick={() => void invoke(quick ? "show_main" : "show_quick_prompt").catch(() => setError("Could not open the requested view."))}>
            {quick ? "Open main app" : "Open quick prompt"}</button>}
          {quick && <button onClick={() => void invoke("hide_quick_prompt").catch(() => setError("Could not hide the prompt."))}>Hide</button>}
        </div>
        {(error || navigation.error) && <p className="error" role="alert">{error || navigation.error}</p>}
        {content}
        {confirmQuit && <section className="panel" role="alertdialog" aria-label="Quit Overlay Forge">
          <p>Quit Overlay Forge? Unsent drafts will be cleared.</p>
          <div className="actions"><button onClick={() => void invoke("shutdown_app").catch(() => setError("Could not quit the app."))}>Quit</button>
          <button onClick={() => setConfirmQuit(false)}>Keep running</button></div>
        </section>}
      </section>
    </div>
  </main>;
}
