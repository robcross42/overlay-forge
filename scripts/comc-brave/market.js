import {
  comcRecentlyAddedHockey,
  readRecentlyAddedMarket,
} from "../../modules/hockey-cards/src/comcMarketCapture";

if (location.origin + location.pathname === comcRecentlyAddedHockey &&
    location.hash === "#overlay-forge-market-refresh") {
  let message;
  try {
    message = { type: "recent-market-capture", capture: readRecentlyAddedMarket(document) };
  } catch (error) {
    message = { type: "recent-market-capture", error: error.message };
  }
  void chrome.runtime.sendMessage(message).catch(() => {});
}
