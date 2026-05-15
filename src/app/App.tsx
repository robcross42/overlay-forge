import { useEffect, useMemo, useState } from "react";
import { ComponentHost } from "../components/ComponentHost";
import { WindowResizeHandles, WindowTitlebar } from "../components/WindowControls";
import { Scratchpad } from "../features/scratchpad/Scratchpad";
import { getMilestoneStatus } from "../services/appStatus";
import type { MilestoneStatus } from "../services/appStatus";

const navItems = [
  { id: "scratchpad", label: "Scratchpad", enabled: true },
  { id: "projects", label: "Projects", enabled: false },
  { id: "calendar", label: "Calendar", enabled: false },
  { id: "planning-chat", label: "Planning Chat", enabled: false },
  { id: "youtube", label: "YouTube", enabled: false },
  { id: "settings", label: "Settings", enabled: false }
];

export default function App() {
  const [status, setStatus] = useState<MilestoneStatus | null>(null);

  useEffect(() => {
    getMilestoneStatus()
      .then(setStatus)
      .catch(() => {
        setStatus({
          milestone: "Milestone 0",
          hotkey: "Ctrl+Shift+Space",
          databaseReady: false
        });
      });
  }, []);

  const activeMeta = useMemo(
    () => ({
      title: "Scratchpad",
      eyebrow: status?.milestone ?? "Milestone 0",
      hotkey: status?.hotkey ?? "Ctrl+Shift+Space"
    }),
    [status]
  );

  return (
    <main className="overlay-frame">
      <WindowResizeHandles />
      <WindowTitlebar />

      <div className="overlay-shell">
        <aside className="sidebar">
          <div className="brand-block">
            <span className="brand-mark">OF</span>
            <div>
              <h1>Overlay Forge</h1>
              <p>Desktop command hub</p>
            </div>
          </div>

          <nav className="component-nav" aria-label="Components">
            {navItems.map((item) => (
              <button
                className={item.enabled ? "nav-item nav-item-active" : "nav-item"}
                disabled={!item.enabled}
                key={item.id}
                type="button"
              >
                {item.label}
              </button>
            ))}
          </nav>

          <div className="shell-status">
            <span className={status?.databaseReady ? "status-dot ready" : "status-dot"} />
            <span>{status?.databaseReady ? "SQLite ready" : "SQLite pending"}</span>
          </div>
        </aside>

        <section className="workspace" aria-label="Active component">
          <header className="workspace-header">
            <div>
              <p>{activeMeta.eyebrow}</p>
              <h2>{activeMeta.title}</h2>
            </div>
            <kbd>{activeMeta.hotkey}</kbd>
          </header>

          <ComponentHost>
            <Scratchpad />
          </ComponentHost>
        </section>
      </div>
    </main>
  );
}
