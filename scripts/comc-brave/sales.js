import { comcSalesUrl } from "../../modules/hockey-cards/src/comcCapture";
import { readAllRecentSales } from "./sales-pagination";

async function observeOpenedCard() {
  const opened = new URL(location.href);
  const sourceUrl = comcSalesUrl(opened);
  if (!sourceUrl) return;

  let observation;
  try {
    observation = await readAllRecentSales(document, sourceUrl);
  } catch (error) {
    const reason = error instanceof Error ? error.message : undefined;
    await chrome.runtime.sendMessage({ type: "market-sales-unavailable", sourceUrl, reason });
    return;
  }
  await chrome.runtime.sendMessage({ type: "market-sales", sourceUrl, ...observation });
}

function readCard() {
  if (globalThis.overlayForgeSalesReading) return;
  globalThis.overlayForgeSalesReading = true;
  void observeOpenedCard().catch(() => {}).finally(() => {
    globalThis.overlayForgeSalesReading = false;
  });
}

readCard();
if (!globalThis.overlayForgeSalesPageListener) {
  globalThis.overlayForgeSalesPageListener = true;
  window.addEventListener("pageshow", (event) => {
    if (event.persisted) readCard();
  });
}
