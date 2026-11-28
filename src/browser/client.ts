import type { MarketTransport } from "../../modules/hockey-cards/src/ComcMarket";
import type { SalesCaptureTransport, SalesTransport } from "../../modules/hockey-cards/src/ComcSalesHistory";
import type { InventoryCaptureTransport, InventoryTransport } from "../../modules/hockey-cards/src/ComcInventory";
import type { MediaTransport } from "../../modules/media/src/embedded";

export interface BrowserSession {
  email: string;
  name: string | null;
  csrfToken: string;
  navigationCollapsed: boolean;
  localLlmEnabled: boolean;
  hockeyInventoryEnabled: boolean;
  hockeyInventoryCanImport: boolean;
  hockeySalesEnabled: boolean;
  hockeyMarketEnabled: boolean;
}

export interface LocalChatMessage {
  role: "user" | "assistant";
  content: string;
}

/** Same-origin cookies authenticate browser clients; tokens never enter browser storage. */
export class BrowserClient {
  constructor(private readonly expired: () => void) {}

  async session(): Promise<BrowserSession | null> {
    const response = await fetch("/api/session", { credentials: "same-origin", cache: "no-store" });
    if (response.status === 401) return null;
    if (!response.ok) throw new Error("Browser server unavailable. Open the server URL rather than the desktop preview.");
    return response.json() as Promise<BrowserSession>;
  }
  private async post<T>(path: string, csrf?: string, body: unknown = {}): Promise<T> {
    const response = await fetch(path, {
      method: "POST", credentials: "same-origin", cache: "no-store",
      headers: { "Content-Type": "application/json", ...(csrf ? { "X-OF-CSRF": csrf } : {}) },
      body: JSON.stringify(body),
    });
    if (response.status === 401) { this.expired(); throw new Error("Your session ended. Sign in again."); }
    if (!response.ok) {
      const data: unknown = await response.json().catch(() => null);
      const detail = data && typeof data === "object" && "error" in data && typeof data.error === "string" ? data.error : "The request could not be completed.";
      throw new Error(detail);
    }
    return response.status === 204 ? undefined as T : response.json() as Promise<T>;
  }
  async signIn(): Promise<void> {
    const { url } = await this.post<{ url: string }>("/auth/start");
    const destination = new URL(url);
    if (destination.protocol !== "https:" || destination.hostname !== "accounts.google.com") throw new Error("Invalid sign-in destination.");
    window.location.assign(destination.href);
  }
  logout(csrf: string): Promise<void> { return this.post("/auth/logout", csrf); }
  saveNavigation(csrf: string, collapsed: boolean): Promise<void> { return this.post("/api/navigation", csrf, { collapsed }); }

  localChat(csrf: string, messages: LocalChatMessage[]): Promise<{ content: string }> {
    return this.post("/api/local-llm/chat", csrf, { messages });
  }

  hockeyMarketTransport(csrf: string): MarketTransport {
    return (sort, hideAt, hideAuctions, minAsking) => this.post("/api/hockey/market", csrf, { sort, hideAt, hideAuctions, minAsking });
  }

  hockeySalesTransport(csrf: string): SalesTransport {
    return (card) => this.post("/api/hockey/sales", csrf, { card });
  }

  hockeyInventoryCapture(csrf: string): InventoryCaptureTransport {
    return (capture) => this.post("/api/hockey/inventory/import", csrf, capture);
  }

  hockeySalesCapture(csrf: string): SalesCaptureTransport {
    return (observation) => this.post("/api/hockey/sales/observe", csrf, observation);
  }

  hockeyInventoryTransport(csrf: string): InventoryTransport {
    return (query) => this.post("/api/hockey/inventory", csrf, query);
  }

  mediaTransport(csrf: string): MediaTransport {
    return async <T>(command: string, args = {}): Promise<T> => {
      const opensLink = ["open_media_streaming_target", "open_book_link", "open_book_data_source"].includes(command);
      const popup = opensLink ? window.open("about:blank", "_blank") : null;
      if (opensLink && !popup) throw new Error("Allow a new tab to open this link.");
      if (popup) popup.opener = null;
      try {
        const value = await this.post<T>(`/api/media/${encodeURIComponent(command)}`, csrf, args);
        if (popup) {
          if (typeof value !== "string") throw new Error("The link is unavailable.");
          const url = new URL(value);
          if (!["http:", "https:"].includes(url.protocol)) throw new Error("The link is not a web address.");
          popup.location.replace(url.href);
          return undefined as T;
        }
        return value;
      } catch (cause) { popup?.close(); throw cause; }
    };
  }
}
