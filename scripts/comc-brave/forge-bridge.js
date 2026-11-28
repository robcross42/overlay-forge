if (location.origin === "http://127.0.0.1:3000") {
  window.addEventListener("message", async (event) => {
    const { type, requestId } = event.data ?? {};
    if (event.source !== window || event.origin !== location.origin ||
        type !== "overlay-forge:market-refresh" ||
        typeof requestId !== "string" || !/^[0-9a-f-]{36}$/.test(requestId)) return;
    window.postMessage({ type: "overlay-forge:market-refresh-ack", requestId }, location.origin);
    let result;
    try {
      result = await chrome.runtime.sendMessage({ type: "refresh-market" });
    } catch (error) {
      result = { error: error.message };
    }
    window.postMessage({ type: "overlay-forge:market-refresh-result", requestId, ...result }, location.origin);
  });
}
