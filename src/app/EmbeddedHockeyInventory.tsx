import { invoke } from "@tauri-apps/api/core";
import { useCallback } from "react";
import type { InventoryTransport } from "../../modules/hockey-cards/src/ComcInventory";
import { ComcWorkspace } from "../../modules/hockey-cards/src/ComcWorkspace";

export function EmbeddedHockeyInventory({ sessionId }: { sessionId: string }) {
  const load = useCallback<InventoryTransport>(
    (query) => invoke("hockey_inventory_query", { sessionId, query }),
    [sessionId],
  );
  return <ComcWorkspace inventory={load} />;
}
