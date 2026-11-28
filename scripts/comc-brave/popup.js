const comc = "https://www.comc.com";
const forge = "http://127.0.0.1:3000";
const forgePermission = { origins: ["http://127.0.0.1/*"] };
const read = document.getElementById("read");
const save = document.getElementById("save");
const result = document.getElementById("result");
let capture;

function showReading(reading) {
  capture = reading?.capture;
  read.disabled = Boolean(reading?.pending);
  save.disabled = !capture || read.disabled;
  result.textContent = reading?.messages.join("\n") ?? "Open a COMC tab to read your inventory.";
}

chrome.storage.onChanged.addListener((changes, area) => {
  if (area === "session" && changes.reading) showReading(changes.reading.newValue);
});
chrome.storage.session.get("reading").then(({ reading }) => showReading(reading));

read.addEventListener("click", async () => {
  try {
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    if (!tab?.url || new URL(tab.url).origin !== comc) {
      throw new Error("Switch to your www.comc.com tab, then open this popup again.");
    }
    read.disabled = true;
    save.disabled = true;
    const status = await chrome.runtime.sendMessage({ type: "read-inventory", tabId: tab.id });
    if (status?.error) throw new Error(status.error);
  } catch (error) {
    result.textContent = error.message;
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
    const confirmation = "Saved " + saved.items + " cards from " + saved.pages + " pages" +
      (saved.partial ? " as a partial snapshot." : ". Refresh Hockey Cards to see it.");
    await chrome.storage.session.set({ reading: { messages: [confirmation] } });
  } catch (error) {
    result.textContent = summary + "\n" + error.message;
  } finally {
    read.disabled = false;
    save.disabled = !capture;
  }
});
