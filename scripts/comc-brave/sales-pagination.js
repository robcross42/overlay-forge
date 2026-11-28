import {
  parseComcHtml,
  parseRecentSalesPage,
} from "../../modules/hockey-cards/src/comcCapture";

const pageLimit = 100;
const saleLimit = 2000;
const salesControl = "ctl00$ContentPlaceHolder1$gvItemsSold";
const postback = /^javascript:__doPostBack\('ctl00\$ContentPlaceHolder1\$gvItemsSold','(Page\$(?:[1-9]\d{0,2}|Next|Prev|First|Last))'\);?$/;

function selectedPage(pager) {
  const numbers = [...pager.querySelectorAll("[aria-current='page'], span, b, strong")]
    .filter((node) => !node.closest("a"))
    .map((node) => node.textContent.trim())
    .filter((text) => /^[1-9]\d{0,2}$/.test(text));
  const pages = [...new Set(numbers)];
  if (pages.length !== 1) throw new Error("Could not verify the current COMC Recent Sales page.");
  return Number(pages[0]);
}

function nextArgument(pager, page) {
  const links = [...pager.querySelectorAll("a[href]")];
  const argumentsByLink = links.map((link) =>
    postback.exec(link.getAttribute("href")?.trim() ?? "")?.[1]);
  if (argumentsByLink.some((argument) => argument === undefined)) {
    throw new Error("COMC returned an unfamiliar Recent Sales pager.");
  }
  const next = "Page$" + (page + 1);
  if (argumentsByLink.includes(next)) return next;
  if (argumentsByLink.includes("Page$Next")) return "Page$Next";
  if (argumentsByLink.some((argument) =>
    argument === "Page$Last" || Number(argument.slice(5)) > page)) {
    throw new Error("COMC did not provide the next consecutive Recent Sales page.");
  }
  return null;
}

function pageTurn(pager, sourceUrl, argument) {
  const form = pager.closest("form");
  if (!form || form.method !== "post") throw new Error("The Recent Sales pager has no POST form.");
  const action = new URL(form.getAttribute("action") || sourceUrl, sourceUrl);
  const source = new URL(sourceUrl);
  if (action.origin !== source.origin || action.pathname !== source.pathname ||
      action.username || action.password) {
    throw new Error("The Recent Sales pager points outside this COMC card.");
  }

  // Send only ASP.NET page state and the verified sales pagination command.
  // Never include any listing, purchase, or offer controls from the card form.
  const fields = new URLSearchParams();
  const field = (name) => form.elements.namedItem(name)?.value;
  const stateCount = Number(field("__VIEWSTATEFIELDCOUNT") ?? "1");
  if (!Number.isInteger(stateCount) || stateCount < 1 || stateCount > 20) {
    throw new Error("COMC returned unsupported Recent Sales page state.");
  }
  for (const name of ["__VIEWSTATEFIELDCOUNT", "__VIEWSTATEGENERATOR",
    "__VIEWSTATEENCRYPTED", "__EVENTVALIDATION", "__LASTFOCUS"]) {
    const value = field(name);
    if (typeof value === "string") fields.set(name, value);
  }
  for (let index = 0; index < stateCount; index += 1) {
    const name = "__VIEWSTATE" + (index || "");
    const value = field(name);
    if (!value) throw new Error("COMC did not return the state needed for the next sales page.");
    fields.set(name, value);
  }
  fields.set("__EVENTTARGET", salesControl);
  fields.set("__EVENTARGUMENT", argument);
  return { url: action.origin + action.pathname + action.search, body: fields };
}

export async function readAllRecentSales(initial, sourceUrl) {
  const signal = AbortSignal.timeout(150_000);
  const options = { credentials: "same-origin", redirect: "error", cache: "no-store", signal };
  async function load(url, request = {}) {
    const response = await fetch(url, { ...options, ...request });
    if (!response.ok) throw new Error("COMC returned HTTP " + response.status + " while reading Recent Sales.");
    return parseRecentSalesPage(parseComcHtml(await response.text()));
  }

  let currentPage = parseRecentSalesPage(initial);
  if (!currentPage) return { sales: [], pages: 0 };
  if (currentPage.pager && selectedPage(currentPage.pager) !== 1) {
    // The toolbar action can start on a page the user previously paged to.
    currentPage = await load(sourceUrl);
  }

  const sales = [];
  for (let page = 1; page <= pageLimit; page += 1) {
    if (!currentPage || (currentPage.pager ? selectedPage(currentPage.pager) !== page : page !== 1)) {
      throw new Error("COMC did not return the expected Recent Sales page " + page + ".");
    }
    sales.push(...currentPage.sales);
    if (sales.length > saleLimit) throw new Error("The Recent Sales table exceeds the 2,000-sale scan limit.");
    const argument = currentPage.pager && nextArgument(currentPage.pager, page);
    if (!argument) return { sales, pages: page };
    if (page === pageLimit) break;
    const { url, body } = pageTurn(currentPage.pager, sourceUrl, argument);
    await new Promise((resolve) => setTimeout(resolve, 500));
    currentPage = await load(url, { method: "POST", body });
  }
  throw new Error("COMC Recent Sales exceeds the 100-page scan limit.");
}
