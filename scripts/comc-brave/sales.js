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
