import { useState } from "react";

type TabId = "listings" | "watchlist" | "estimates" | "sources" | "settings";

const tabs: TabId[] = ["listings", "watchlist", "estimates", "sources", "settings"];

export function RepairResell() {
  const [activeTab, setActiveTab] = useState<TabId>("listings");

  return (
    <section aria-label="Repair Resell" className="feature-panel repair-resell-panel">
      <div className="module-tabs repair-resell-tabs button-only">
        {tabs.map((tab) => (
          <button
            aria-pressed={activeTab === tab}
            className={activeTab === tab ? "module-tab active" : "module-tab"}
            key={tab}
            onClick={() => setActiveTab(tab)}
            type="button"
          >
            {tabLabel(tab)}
          </button>
        ))}
      </div>
    </section>
  );
}

function tabLabel(tab: TabId) {
  return tab.charAt(0).toUpperCase() + tab.slice(1);
}
