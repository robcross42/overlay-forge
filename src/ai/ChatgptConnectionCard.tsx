import { useState } from "react";
import { ChatgptConnection, HubClient } from "./hub";

export function ChatgptConnectionCard({ connection: c, run }: {
  connection: ChatgptConnection | null | undefined;
  run: (command: string, args?: Record<string, unknown>) => Promise<void>;
}) {
  const [model, setModel] = useState("");
  if (!c) return <section className="panel"><h2>ChatGPT live connection</h2><p>Native connection status unavailable.</p></section>;
  return <section className="panel" aria-label="ChatGPT live connection">
    <div className="row"><h2>ChatGPT live connection test</h2><strong>{c.verification.freshness === "stale" || c.verification.freshness === "unknown" ? `Current activity unknown (last state: ${c.state})` : c.state}</strong></div>
    <p role="status">{c.detail}</p>
    <p>Account: {c.accountLabel ?? "not connected"}{c.registration ? ` / Registration: ${c.registration}` : ""}</p>
    <p>Funding: {c.planGranted ? "ChatGPT plan-use permission granted." : "ChatGPT plan use requires permission."} ChatGPT usage and any credits you permit are governed by your ChatGPT settings. API-key billing is disabled.</p>
    <p className="muted">This experimental local connection uses your system browser. Review the identity and plan-use permission there. It does not access existing ChatGPT conversations, Google services, computer capture, or other accounts.</p>
    <div className="actions">
      <button disabled={c.busy} data-oai-siwc-button onClick={() => void run("chatgpt_sign_in")}>Continue with ChatGPT</button>
      <button disabled={c.busy || !c.planGranted} onClick={() => void run("chatgpt_verify", { model: model || null })}>Send harmless verification prompt</button>
      <button disabled={c.busy || !c.registration} onClick={() => void run("chatgpt_disconnect")}>Disconnect ChatGPT</button>
      {c.busy && <button onClick={() => void run("hub_stop_all")}>Stop connection work</button>}
    </div>
    {c.models.length > 0 && <label className="input-label">Verification model <select value={model} onChange={e => setModel(e.target.value)}><option value="">First currently available model</option>{c.models.map(m => <option key={m.slug} value={m.slug}>{m.displayName}</option>)}</select></label>}
    <p className="muted">The live test sends “Say exactly: Overlay Forge connection verified.” It may consume your authorized ChatGPT allowance or credits allowed in ChatGPT settings.</p>
    {c.verificationText && <blockquote className="prompt-text">{c.verificationText}</blockquote>}
    <small>Last native update: {new Date(c.lastUpdate * 1000).toLocaleTimeString()}{c.providerSequence !== null ? ` / Provider event ${c.providerSequence}` : ""}{c.correlation ? ` / ${c.correlation}` : ""}</small>
    <p className="muted">Provider evidence: {c.verification.freshness}. Last verified stream event: {HubClient.timestamp(c.verification.lastVerifiedAt)}. Scheduled remote reconciliation is unsupported; local checks cannot prove continuing remote activity.</p>
    <p className="muted">Tokens stay native. Windows uses current-user DPAPI; Linux uses authenticated encryption with its key in an unlocked desktop Secret Service. Missing or locked secure storage blocks credential operations. Disconnect attempts remote revocation, clears local tokens, and reports whether remote revocation was confirmed.</p>
  </section>;
}
