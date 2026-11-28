const comc = "https://www.comc.com";
const forgePermission = { origins: ["http://127.0.0.1/*"] };
const read = document.getElementById("read");
const save = document.getElementById("save");
const result = document.getElementById("result");
const salesStatus = document.getElementById("sales-status");
let capture;

function showReading(reading) {
  capture = reading?.capture;
  read.disabled = Boolean(reading?.pending);
  save.disabled = !capture || read.disabled;
  result.textContent = reading?.messages.join("\n") ?? "Open a COMC tab to read your inventory.";
}

chrome.storage.onChanged.addListener((changes, area) => {
  if (area !== "session") return;
  if (changes.reading) showReading(changes.reading.newValue);
  if (changes.salesStatus) showSales(changes.salesStatus.newValue);
});
chrome.storage.session.get(["reading", "salesStatus"]).then((state) => {
  showReading(state.reading);
  showSales(state.salesStatus);
});

function showSales(status) {
  salesStatus.textContent = status ?? "Recent Sales sync runs when you open a COMC Hockey card.";
}

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

save.addEventListener("click", async () => {
  if (!capture) return;
  save.disabled = true;
  result.textContent = "Saving to Overlay Forge…";
  try {
    // Dispatch the request before Brave opens its permission prompt. The
    // worker can then finish the save if the prompt closes this popup.
    const queued = chrome.runtime.sendMessage({ type: "save-inventory" });
    const allowed = chrome.permissions.request(forgePermission);
    const [status, granted] = await Promise.all([queued, allowed]);
    if (status?.error) throw new Error(status.error);
    if (!granted) {
      await chrome.runtime.sendMessage({ type: "cancel-save" });
      throw new Error("Allow the extension to connect to your local Overlay Forge tab.");
    }
  } catch (error) {
    result.textContent = error.message;
    save.disabled = !capture;
  }
});
