import { comcRecentlyAddedHockey } from "../../modules/hockey-cards/src/comcMarketCapture";

const pendingKey = "overlay-forge:market-refresh";
const resultKey = "overlay-forge:market-refresh-result";

export function refreshComcMarket(): Promise<number> {
  return new Promise((resolve, reject) => {
    const requestId = crypto.randomUUID();
    sessionStorage.setItem(pendingKey, requestId);
    sessionStorage.removeItem(resultKey);

    const tab = window.open("about:blank", "_blank");
    if (!tab) {
      sessionStorage.removeItem(pendingKey);
      reject(new Error("Allow pop-ups for Overlay Forge before refreshing the COMC market."));
      return;
    }
    tab.opener = null;

    const timer = window.setTimeout(() => finish(new Error(
      "No market results arrived. Check that Brave extension 0.5.1 is enabled and that the COMC tab loaded Recently Added.",
    )), 35_000);

    function finish(result: number | Error) {
      window.clearTimeout(timer);
      window.removeEventListener(resultKey, receive);
      if (sessionStorage.getItem(pendingKey) === requestId) sessionStorage.removeItem(pendingKey);
      sessionStorage.removeItem(resultKey);
      if (result instanceof Error) reject(result);
      else resolve(result);
    }

    function receive() {
      const stored = sessionStorage.getItem(resultKey);
      if (!stored) return;
      const result = JSON.parse(stored) as { requestId?: string; cards?: number; error?: string };
      if (result.requestId !== requestId) return;
      if (typeof result.cards === "number") finish(result.cards);
      else finish(new Error(result.error || "Could not refresh the COMC market."));
    }

    window.addEventListener(resultKey, receive);
    tab.location.replace(comcRecentlyAddedHockey + "#overlay-forge-market-refresh=" + requestId);
  });
}
