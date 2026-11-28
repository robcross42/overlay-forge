let reading = false;

async function readInventory(tabId) {
  reading = true;
  try {
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

chrome.runtime.onMessage.addListener((message, sender, respond) => {
  if (sender.url !== chrome.runtime.getURL("popup.html") ||
      message?.type !== "read-inventory" || !Number.isInteger(message.tabId)) return;
  if (reading) {
    respond({ error: "An inventory read is already running." });
    return;
  }
  readInventory(message.tabId)
    .then(respond, () => respond({ error: "The browser could not store the scan result." }));
  return true;
});
