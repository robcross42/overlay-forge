import { invoke } from "@tauri-apps/api/core";
import { useCallback } from "react";
import {
  ComcInventory,
  type InventoryTransport,
} from "../../modules/hockey-cards/src/ComcInventory";

export function EmbeddedHockeyInventory({ sessionId }: { sessionId: string }) {
  const load = useCallback<InventoryTransport>(
    (query) => invoke("hockey_inventory_query", { sessionId, query }),
    [sessionId],
  );
  return <ComcInventory load={load} />;
}
