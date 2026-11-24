import type { ChatgptConnection } from "../ai/hub";
import type { GoogleSession } from "./GoogleSession";

type Tone = "connected" | "attention" | "inactive";
interface ConnectionTile { key: string; name: string; symbol: string; status: string; tone: Tone; account?: string }

class ConnectionStatus {
  static chatgpt(connection?: ChatgptConnection | null): ConnectionTile {
    const base = { key: "chatgpt", name: "ChatGPT", symbol: "✳" };
    if (!connection) return { ...base, status: "Unavailable", tone: "inactive" };
    const connected = ["connected", "verified"].includes(connection.state);
    const attention = ["limited", "failed", "unknown", "authorizing", "verifying", "stopping"].includes(connection.state);
    return { ...base, status: connection.state === "verified" ? "Connected" : connection.state.charAt(0).toUpperCase() + connection.state.slice(1),
      tone: connected ? "connected" : attention ? "attention" : "inactive", account: connection.accountLabel ?? undefined };
  }
  static all(session: GoogleSession, connection?: ChatgptConnection | null): ConnectionTile[] {
    return [
      { key: "google", name: "Google", symbol: "G", status: "Signed in", tone: "connected", account: session.profile?.email ?? undefined },
      this.chatgpt(connection),
      { key: "gmail", name: "Gmail", symbol: "✉", status: "Not connected", tone: "inactive" },
      { key: "calendar", name: "Calendar", symbol: "▦", status: "Not connected", tone: "inactive" },
      { key: "codex", name: "Codex", symbol: "⌘", status: "Unavailable", tone: "inactive" },
      { key: "dot", name: "Dot", symbol: "●", status: "Unavailable", tone: "inactive" },
    ];
  }
}

export function ConnectionDashboard({ session, connection, onAccounts }: {
  session: GoogleSession; connection?: ChatgptConnection | null; onAccounts: () => void;
}) {
  return <div className="connection-dashboard" role="list" aria-label="Connections">
    {ConnectionStatus.all(session, connection).map(tile => <div role="listitem" key={tile.key}>
      <button className={`connection-tile connection-${tile.tone}`} onClick={onAccounts} aria-label={`${tile.name}: ${tile.status}`}>
        <span className="connection-symbol" aria-hidden="true">{tile.symbol}</span>
        <strong>{tile.name}</strong>
        <span className="connection-status"><span className="status-dot" aria-hidden="true" />{tile.status}</span>
        {tile.account && <small>{tile.account}</small>}
      </button>
    </div>)}
  </div>;
}
