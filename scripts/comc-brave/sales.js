import {
  comcBaseSalesUrl,
  parseComcHtml,
  parseRecentSales,
} from "../../modules/hockey-cards/src/comcCapture";

async function observeOpenedCard() {
  const opened = new URL(location.href);
  const sourceUrl = comcBaseSalesUrl(opened);
  if (!sourceUrl) return;

  let sales;
  try {
    let html = document;
    // A graded listing has a separate view. Read the base card's Recent
    // Sales so every observation has the same scope for its catalog ID.
    if (sourceUrl !== opened.origin + opened.pathname) {
      const response = await fetch(sourceUrl, {
        credentials: "same-origin",
        redirect: "error",
        cache: "no-store",
        signal: AbortSignal.timeout(12_000),
      });
      if (!response.ok) throw new Error("COMC did not return the base card.");
      html = parseComcHtml(await response.text());
    }
    sales = parseRecentSales(html);
  } catch {
    await chrome.runtime.sendMessage({ type: "market-sales-unavailable", sourceUrl });
    return;
  }
  await chrome.runtime.sendMessage({ type: "market-sales", sourceUrl, sales });
}

function readCard() {
  void observeOpenedCard().catch(() => {});
}

readCard();
window.addEventListener("pageshow", (event) => {
  if (event.persisted) readCard();
});
