import {
  comcRecentlyAddedHockey,
  readRecentlyAddedMarket,
} from "../../modules/hockey-cards/src/comcMarketCapture";

const request = /^#overlay-forge-market-refresh=([0-9a-f-]{36})$/.exec(location.hash);
if (location.origin + location.pathname === comcRecentlyAddedHockey && request) {
  const requestId = request[1];
  let message;
  try {
    message = { type: "recent-market-capture", requestId, capture: readRecentlyAddedMarket(document) };
  } catch (error) {
    message = { type: "recent-market-capture", requestId, error: error.message };
  }
  void chrome.runtime.sendMessage(message).catch(() => {});
}
