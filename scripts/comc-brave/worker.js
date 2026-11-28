const forge = "http://127.0.0.1:3000";
const forgePermission = { origins: ["http://127.0.0.1/*"] };
let reading = false;
let saving = false;

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

// Runs inside the isolated world of the selected Overlay Forge tab. Its session
// cookie and CSRF value stay within that origin and never enter the popup.
async function saveSnapshot(origin, inventory) {
  if (location.origin !== origin) return { error: "The Overlay Forge tab has changed." };
  const options = { credentials: "same-origin", cache: "no-store", redirect: "error" };
  const session = await fetch("/api/session", options);
  if (!session.ok) return { error: "Sign in to Overlay Forge as the owner before saving." };
  const identity = await session.json();
  if (!identity.hockeyInventoryCanImport || !identity.csrfToken) {
    return { error: "This Overlay Forge account cannot import inventory." };
  }

  const response = await fetch("/api/hockey/inventory/import", {
    ...options,
    method: "POST",
    headers: { "Content-Type": "application/json", "X-OF-CSRF": identity.csrfToken },
    body: JSON.stringify(inventory),
  });
  if (!response.ok) {
    const detail = await response.json().catch(() => null);
    return { error: detail?.error ?? "Overlay Forge rejected the snapshot." };
  }
  const { items, pages, partial } = await response.json();
  return { items, pages, partial };
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
      const tabs = await chrome.tabs.query({ url: "http://127.0.0.1/*" });
      const tab = tabs.find((candidate) =>
        candidate.url && new URL(candidate.url).origin === forge);
      if (!tab) throw new Error("Open Overlay Forge at " + forge + " and sign in first.");
      const [execution] = await chrome.scripting.executeScript({
        target: { tabId: tab.id },
        func: saveSnapshot,
        args: [forge, scan.capture],
      });
      const saved = execution?.result;
      if (!saved || saved.error) throw new Error(saved?.error ?? "Overlay Forge returned no result.");
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

chrome.runtime.onMessage.addListener((message, sender, respond) => {
  if (sender.url !== chrome.runtime.getURL("popup.html")) return;
  let operation;
  switch (message?.type) {
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
