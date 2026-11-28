const cardPath = /^\/Cards\/Hockey\/(?:[^/]+\/){4}\d{1,32}(?:\/(?:Graded|Ungraded|AftermarketAuto)\/[^/]+\/[^/]+)?$/;
const forge = "http://127.0.0.1:3000";
const forgePermission = { origins: ["http://127.0.0.1/*"] };
let reading = false;
let saving = false;
const marketUrl = "https://www.comc.com/Cards/Hockey,sr";

chrome.runtime.onInstalled.addListener(() => {
  // Remove the dynamic bridge used by extension 0.5.0.
  void chrome.scripting.unregisterContentScripts({ ids: ["overlay-forge-market-bridge"] }).catch(() => {});
});

async function readInventory(tabId) {
  reading = true;
  try {
    await chrome.storage.session.remove("saveUntil");
    await chrome.storage.session.set({
      reading: { pending: true, messages: ["Reading COMC inventory… You can reopen this popup to check progress."] },
    });
    const target = { tabId };
    await chrome.scripting.executeScript({ target, files: ["read.js"] });
    const [execution] = await chrome.scripting.executeScript({
      target,
      func: () => {
        const read = globalThis.overlayForgeReadComcInventory;
        delete globalThis.overlayForgeReadComcInventory;
        return read();
      },
    });
    const result = execution?.result;
    if (!result) throw new Error("Brave returned no scan result.");
    if (result.capture?.pages.at(-1)?.hasMore) {
      result.messages.push("You can save the verified pages as a partial snapshot.");
    }
    await chrome.storage.session.set({ reading: result });
    return {};
  } catch (error) {
    await chrome.storage.session.set({ reading: { messages: [error.message] } });
    return {};
  } finally {
    reading = false;
  }
}

// Executed in the local Overlay Forge tab. The session cookie and CSRF
// value stay at that origin and never enter the extension worker.
async function saveInOverlayForge(origin, kind, capture) {
  if (location.origin !== origin) return { error: "The Overlay Forge tab has changed." };
  const options = { credentials: "same-origin", cache: "no-store", redirect: "error" };
  const session = await fetch("/api/session", options);
  if (!session.ok) return { error: "Sign in to Overlay Forge as the owner before saving." };
  const identity = await session.json();
  const inventory = kind === "inventory";
  const market = kind === "market";
  if (!(inventory ? identity.hockeyInventoryCanImport
      : market ? identity.hockeyMarketEnabled : identity.hockeySalesEnabled) ||
      !identity.csrfToken) {
    return { error: "This Overlay Forge account cannot import " + kind + "." };
  }

  if (inventory || market || capture.sales.length) {
    const endpoint = inventory ? "/api/hockey/inventory/import"
      : market ? "/api/hockey/market/observe" : "/api/hockey/sales/observe";
    const response = await fetch(endpoint, {
      ...options,
      method: "POST",
      headers: { "Content-Type": "application/json", "X-OF-CSRF": identity.csrfToken },
      body: JSON.stringify(capture),
    });
    if (!response.ok) {
      const detail = await response.json().catch(() => null);
      return { error: detail?.error ?? "Overlay Forge rejected the capture." };
    }
    const result = await response.json();
    if (inventory) {
      const { items, pages, partial } = result;
      return { items, pages, partial };
    }
    if (market) return { cards: result.cards };
    selectCard();
    return { added: result.added };
  }

  selectCard();
  return { added: 0 };

  function selectCard() {
    sessionStorage.setItem("overlay-forge:comc-sales-card", capture.sourceUrl);
    window.dispatchEvent(new Event("overlay-forge:comc-sales"));
  }
}

async function transferToOverlayForge(kind, capture, windowId) {
  const tabs = await chrome.tabs.query({ url: "http://127.0.0.1/*" });
  const forgeTabs = tabs.filter((tab) => tab.url && new URL(tab.url).origin === forge);
  const tab = forgeTabs.find((candidate) => candidate.windowId === windowId) ?? forgeTabs[0];
  if (!tab) throw new Error("Open Overlay Forge at " + forge + " and sign in first.");
  return transferToForgeTab(kind, capture, tab.id);
}

async function transferToForgeTab(kind, capture, tabId) {
  const [execution] = await chrome.scripting.executeScript({
    target: { tabId },
    func: saveInOverlayForge,
    args: [forge, kind, capture],
  });
  const saved = execution?.result;
  if (!saved || saved.error) throw new Error(saved?.error ?? "Overlay Forge returned no result.");
  return saved;
}

async function captureRecentSales(message, sender) {
  let status;
  try {
    const opened = new URL(sender.url);
    const source = new URL(message.sourceUrl);
    if (source.origin !== opened.origin || source.pathname !== opened.pathname ||
        !cardPath.test(source.pathname) || message.sourceUrl !== source.origin + source.pathname) return {};
    if (message.type === "market-sales-unavailable") {
      const reason = typeof message.reason === "string" && message.reason.length <= 160
        ? message.reason : "COMC’s Recent Sales data could not be read for this card.";
      throw new Error(reason);
    }
    if (!Array.isArray(message.sales) || message.sales.length > 2_000 ||
        !Number.isInteger(message.pages) || message.pages < 0 || message.pages > 100) {
      throw new Error("The Recent Sales table was too large or unreadable.");
    }
    if (!await chrome.permissions.contains(forgePermission)) {
      throw new Error("Allow the local Overlay Forge connection in this extension.");
    }
    const capture = { sourceUrl: message.sourceUrl, sales: message.sales };
    const { added } = await transferToOverlayForge("sales", capture, sender.tab.windowId);
    const catalog = source.pathname.split("/")[7];
    const scanned = capture.sales.length + " sales across " + message.pages + " page(s)";
    status = "Catalog " + catalog + " · " + (capture.sales.length
      ? scanned + " · " + (added ? added + " new sales saved." : "All sales already saved.")
      : "COMC showed no Recent Sales.");
  } catch (error) {
    status = error.message;
  }
  await chrome.storage.session.set({ salesStatus: status });
  return {};
}

async function finishSave() {
  const { reading: scan, saveUntil } = await chrome.storage.session.get(["reading", "saveUntil"]);
  if (!scan?.capture || !(saveUntil > Date.now()) ||
      !await chrome.permissions.contains(forgePermission) || saving || reading) return;

  saving = true;
  try {
    await chrome.storage.session.remove("saveUntil");
    await chrome.storage.session.set({
      reading: { ...scan, pending: true, messages: ["Saving to Overlay Forge…"] },
    });
    try {
      const saved = await transferToOverlayForge("inventory", scan.capture);
      const confirmation = "Saved " + saved.items + " cards from " + saved.pages + " pages" +
        (saved.partial ? " as a partial snapshot." : ". Refresh Hockey Cards to see it.");
      await chrome.storage.session.set({ reading: { messages: [confirmation] } });
    } catch (error) {
      await chrome.storage.session.set({
        reading: { ...scan, messages: [...scan.messages, error.message] },
      });
    }
  } finally {
    saving = false;
  }
}

async function queueSave() {
  await chrome.storage.session.set({ saveUntil: Date.now() + 120_000 });
  await finishSave();
  return {};
}

chrome.permissions.onAdded.addListener(() => {
  void finishSave().catch(() => {});
});

async function readRecentSalesFromTab(tabId) {
  const tab = await chrome.tabs.get(tabId);
  const url = new URL(tab.url ?? "about:blank");
  if (url.origin !== "https://www.comc.com" || !cardPath.test(url.pathname)) {
    return { error: "Switch to a COMC Hockey card tab first." };
  }
  await chrome.scripting.executeScript({ target: { tabId }, files: ["sales.js"] });
  return {};
}

async function receiveMarketCapture(message, sender) {
  const requestId = message.requestId;
  const opened = new URL(sender.url);
  if (opened.origin + opened.pathname !== marketUrl ||
      typeof requestId !== "string" || !/^[0-9a-f-]{36}$/.test(requestId) ||
      (opened.hash && opened.hash !== "#overlay-forge-market-refresh=" + requestId)) return;

  const tabs = await chrome.tabs.query({ url: "http://127.0.0.1/*" });
  for (const tab of tabs) {
    if (!tab.url || new URL(tab.url).origin !== forge) continue;
    const target = { tabId: tab.id };
    const [match] = await chrome.scripting.executeScript({
      target,
      func: (id) => sessionStorage.getItem("overlay-forge:market-refresh") === id,
      args: [requestId],
    }).catch(() => []);
    if (match?.result !== true) continue;

    let outcome;
    try {
      if (message.error) throw new Error(message.error);
      outcome = await transferToForgeTab("market", message.capture, tab.id);
    } catch (error) {
      outcome = { error: error.message };
    }
    const [delivered] = await chrome.scripting.executeScript({
      target,
      func: (id, result) => {
        if (sessionStorage.getItem("overlay-forge:market-refresh") !== id) return false;
        const key = "overlay-forge:market-refresh-result";
        sessionStorage.setItem(key, JSON.stringify({ requestId: id, ...result }));
        window.dispatchEvent(new Event(key));
        return true;
      },
      args: [requestId, outcome],
    }).catch(() => []);
    if (delivered?.result && typeof outcome.cards === "number") {
      const marketTab = await chrome.tabs.get(sender.tab.id).catch(() => null);
      if (marketTab?.url === marketUrl + "#overlay-forge-market-refresh=" + requestId) {
        await chrome.tabs.remove(marketTab.id).catch(() => {});
      }
    }
    return;
  }
}

chrome.runtime.onMessage.addListener((message, sender, respond) => {
  if (message?.type === "recent-market-capture") {
    if (!sender.tab || sender.frameId !== 0 || !sender.url?.startsWith(marketUrl)) return;
    receiveMarketCapture(message, sender).then(respond, () => respond({ error: "Could not deliver the market results." }));
    return true;
  }
  if (message?.type === "market-sales" || message?.type === "market-sales-unavailable") {
    if (!sender.tab || sender.frameId !== 0 || !sender.url?.startsWith("https://www.comc.com/")) return;
    captureRecentSales(message, sender).then(respond, () => respond({ error: "Could not record sales." }));
    return true;
  }
  if (sender.url !== chrome.runtime.getURL("popup.html")) return;
  let operation;
  switch (message?.type) {
    case "read-market-sales":
      if (!Number.isInteger(message.tabId)) return;
      operation = readRecentSalesFromTab(message.tabId);
      break;
    case "read-inventory":
      if (!Number.isInteger(message.tabId)) return;
      if (reading || saving) {
        respond({ error: "An inventory operation is already running." });
        return;
      }
      operation = readInventory(message.tabId);
      break;
    case "save-inventory":
      operation = queueSave();
      break;
    case "cancel-save":
      operation = chrome.storage.session.remove("saveUntil");
      break;
    default:
      return;
  }
  operation.then(respond, () => respond({ error: "Brave could not finish the inventory operation." }));
  return true;
});
