import {
  comcSalesUrl,
  parseRecentSales,
} from "../../modules/hockey-cards/src/comcCapture";

async function observeOpenedCard() {
  const opened = new URL(location.href);
  const sourceUrl = comcSalesUrl(opened);
  if (!sourceUrl) return;

  let sales;
  try {
    sales = parseRecentSales(document);
  } catch (error) {
    const reason = error instanceof Error ? error.message : undefined;
    await chrome.runtime.sendMessage({ type: "market-sales-unavailable", sourceUrl, reason });
    return;
  }
  await chrome.runtime.sendMessage({ type: "market-sales", sourceUrl, sales });
}

function readCard() {
  void observeOpenedCard().catch(() => {});
}

readCard();
if (!globalThis.overlayForgeSalesPageListener) {
  globalThis.overlayForgeSalesPageListener = true;
  window.addEventListener("pageshow", (event) => {
    if (event.persisted) readCard();
  });
}
