const comc = "https://www.comc.com/Manage/Inventory";
const forge = "http://127.0.0.1:3000";
const forgePermission = { origins: ["http://127.0.0.1/*"] };
const read = document.getElementById("read");
const save = document.getElementById("save");
const result = document.getElementById("result");
let capture;

read.addEventListener("click", async () => {
  capture = undefined;
  read.disabled = true;
  save.disabled = true;
  result.textContent = "Reading COMC inventory… Keep this popup open.";
  try {
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    const address = new URL(tab?.url ?? "about:blank");
    if (address.origin + address.pathname !== comc) {
      throw new Error("Open COMC Inventory Manager and sign in first.");
    }
    const target = { tabId: tab.id };
    await chrome.scripting.executeScript({ target, files: ["read.js"] });
    const [execution] = await chrome.scripting.executeScript({
      target,
      func: () => {
        const readInventory = globalThis.overlayForgeReadComcInventory;
        delete globalThis.overlayForgeReadComcInventory;
        return readInventory();
      },
    });
    const reading = execution?.result;
    if (!reading) throw new Error("Brave returned no result.");
    capture = reading.capture;
    result.textContent = reading.messages.join("\n");
  } catch (error) {
    result.textContent = error.message;
  } finally {
    read.disabled = false;
    save.disabled = !capture;
  }
});

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

save.addEventListener("click", async () => {
  if (!capture) return;
  read.disabled = true;
  save.disabled = true;
  const summary = result.textContent;
  try {
    if (!await chrome.permissions.request(forgePermission)) {
      throw new Error("Allow the extension to connect to your local Overlay Forge tab.");
    }
    const tabs = await chrome.tabs.query({ url: "http://127.0.0.1/*" });
    const tab = tabs.find((candidate) =>
      candidate.url && new URL(candidate.url).origin === forge);
    if (!tab) throw new Error("Open Overlay Forge at " + forge + " and sign in first.");
    const [execution] = await chrome.scripting.executeScript({
      target: { tabId: tab.id },
      func: saveSnapshot,
      args: [forge, capture],
    });
    const saved = execution?.result;
    if (!saved || saved.error) throw new Error(saved?.error ?? "Overlay Forge returned no result.");
    result.textContent = "Saved " + saved.items + " cards from " + saved.pages + " pages" +
      (saved.partial ? " as a partial snapshot." : ". Refresh Hockey Cards to see it.");
    capture = undefined;
  } catch (error) {
    result.textContent = summary + "\n" + error.message;
  } finally {
    read.disabled = false;
    save.disabled = !capture;
  }
});
