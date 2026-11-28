type RefreshMessage = {
  type?: string;
  requestId?: string;
  cards?: number;
  error?: string;
};

export function refreshComcMarket(): Promise<number> {
  return new Promise((resolve, reject) => {
    const requestId = crypto.randomUUID();
    const bridgeTimer = window.setTimeout(() => finish(
      new Error("Reload the Overlay Forge Brave extension, then refresh this tab and try again."),
    ), 2_500);
    const requestTimer = window.setTimeout(() => finish(
      new Error("COMC did not return the market page in time."),
    ), 30_000);

    function finish(result: number | Error) {
      window.clearTimeout(bridgeTimer);
      window.clearTimeout(requestTimer);
      window.removeEventListener("message", receive);
      if (result instanceof Error) reject(result);
      else resolve(result);
    }

    function receive(event: MessageEvent<RefreshMessage>) {
      if (event.source !== window || event.origin !== location.origin ||
          event.data?.requestId !== requestId) return;
      if (event.data.type === "overlay-forge:market-refresh-ack") {
        window.clearTimeout(bridgeTimer);
      } else if (event.data.type === "overlay-forge:market-refresh-result") {
        if (typeof event.data.cards === "number") finish(event.data.cards);
        else finish(new Error(event.data.error || "Could not refresh the COMC market."));
      }
    }

    window.addEventListener("message", receive);
    window.postMessage({ type: "overlay-forge:market-refresh", requestId }, location.origin);
  });
}
