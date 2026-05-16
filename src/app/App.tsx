import { useEffect, useMemo, useState } from "react";
import { ComponentHost } from "../components/ComponentHost";
import { WindowResizeHandles, WindowTitlebar } from "../components/WindowControls";
import { Calendar } from "../features/calendar/Calendar";
import { Notes } from "../features/notes/Notes";
import { PlanningChat } from "../features/planning-chat/PlanningChat";
import { Projects } from "../features/projects/Projects";
import { Scratchpad } from "../features/scratchpad/Scratchpad";
import { Tasks } from "../features/tasks/Tasks";
import { getMilestoneStatus } from "../services/appStatus";
import type { MilestoneStatus } from "../services/appStatus";

type ComponentId = "scratchpad" | "tasks" | "notes" | "calendar" | "projects" | "planning-chat";

const navItems = [
  { id: "scratchpad", label: "Scratchpad" },
  { id: "tasks", label: "Tasks" },
  { id: "notes", label: "Notes" },
  { id: "calendar", label: "Calendar" },
  { id: "projects", label: "Projects" },
  { id: "planning-chat", label: "Planning Chat" }
] satisfies Array<{ id: ComponentId; label: string }>;

export default function App() {
  const [status, setStatus] = useState<MilestoneStatus | null>(null);
  const [activeComponent, setActiveComponent] = useState<ComponentId>("scratchpad");

  useEffect(() => {
    getMilestoneStatus()
      .then(setStatus)
      .catch(() => {
        setStatus({
          milestone: "Milestone 2",
          hotkey: "Ctrl+Shift+Space",
          databaseReady: false
        });
      });
  }, []);

  const activeMeta = useMemo(
    () => ({
      title: navItems.find((item) => item.id === activeComponent)?.label ?? "Scratchpad",
      eyebrow: status?.milestone ?? "Milestone 2",
      hotkey: status?.hotkey ?? "Ctrl+Shift+Space"
    }),
    [activeComponent, status]
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
                className={item.id === activeComponent ? "nav-item nav-item-active" : "nav-item"}
                key={item.id}
                onClick={() => setActiveComponent(item.id)}
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
            {activeComponent === "scratchpad" && <Scratchpad />}
            {activeComponent === "tasks" && <Tasks />}
            {activeComponent === "notes" && <Notes />}
            {activeComponent === "calendar" && <Calendar />}
            {activeComponent === "projects" && <Projects />}
            {activeComponent === "planning-chat" && <PlanningChat />}
          </ComponentHost>
        </section>
      </div>
    </main>
  );
}
