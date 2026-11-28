import {
  inventoryManagerUrl,
  parseComcHtml,
  parseInventoryPage,
} from "../../modules/hockey-cards/src/comcCapture";

const pageLimit = 25;
const fieldPrefix = "ctl00$ContentPlaceHolder1$";

function nextPageBody(html, page) {
  const form = html.getElementById("aspnetForm");
  const fields = new URLSearchParams();
  const names = [
    "__LASTFOCUS",
    "__VIEWSTATE",
    "__VIEWSTATEGENERATOR",
    "__VIEWSTATEENCRYPTED",
    "__EVENTVALIDATION",
    "ctl00$txtSearch",
    ...["txtFilterResults", "ddlPriceSettings", "ddlOrderBy", "txtGoTo", "ddItems"]
      .map((name) => fieldPrefix + name),
  ];
  for (const name of names) {
    const field = form?.elements.namedItem(name);
    if (field && typeof field.value === "string") fields.set(name, field.value);
  }
  if (!fields.get("__VIEWSTATE") || !fields.get("__EVENTVALIDATION")) {
    throw new Error("COMC did not return the form state needed for pagination.");
  }
  fields.set("__EVENTTARGET", fieldPrefix + "gvInventory");
  fields.set("__EVENTARGUMENT", "Page$" + page);
  return fields;
}

// Injected only after the user clicks Read on an Inventory Manager tab.
async function readComcInventory() {
  const messages = [];
  const pages = [];
  const ids = new Set();
  const signal = AbortSignal.timeout(90_000);
  try {
    if (location.origin + location.pathname !== inventoryManagerUrl) {
      throw new Error("Open the COMC Inventory Manager tab first.");
    }

    let body;
    let context;
    for (let page = 1; page <= pageLimit; page += 1) {
      const response = await fetch(inventoryManagerUrl, {
        method: body ? "POST" : "GET",
        credentials: "same-origin",
        redirect: "error",
        cache: "no-store",
        body,
        signal,
      });
      if (!response.ok) {
        throw new Error("Page " + page + " returned HTTP " + response.status + ".");
      }

      const html = parseComcHtml(await response.text());
      const parsed = parseInventoryPage(html);
      if (!parsed || parsed.captured.page !== page) {
        throw new Error("Expected signed-in Inventory Manager page " + page + ".");
      }
      if (context !== undefined && context !== parsed.context) {
        throw new Error("The COMC account, sort, or page size changed during the read.");
      }
      context = parsed.context;
      const { captured } = parsed;
      for (const { itemId } of captured.rows) {
        if (ids.has(itemId)) throw new Error("COMC repeated an item on page " + page + ".");
        ids.add(itemId);
      }
      pages.push(captured);
      messages.push("Page " + page + ": HTTP " + response.status + " · " + captured.rows.length + " items");
      if (!captured.hasMore) {
        messages.push(ids.size + " unique items across " + pages.length + " pages.");
        return { messages, capture: { pages } };
      }
      if (!parsed.hasNextPage) throw new Error("COMC did not provide the expected next page.");
      if (page < pageLimit) {
        body = nextPageBody(html, page + 1);
        await new Promise((resolve) => setTimeout(resolve, 700));
      }
    }
    throw new Error("Stopped at the " + pageLimit + "-page limit.");
  } catch (error) {
    const detail = error instanceof TypeError
      ? "COMC redirected or the request failed. Check that you are still signed in."
      : error.message;
    messages.push("Stopped after " + pages.length + " verified pages. " + detail);
    return { messages };
  }
}

globalThis.overlayForgeReadComcInventory = readComcInventory;
