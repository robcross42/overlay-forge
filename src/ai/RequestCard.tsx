import { useEffect, useState } from "react";
import { Action, HubClient, RequestRecord } from "./hub";
import { ComputerSessionStatus } from "./ComputerSessionStatus";

export function RequestCard({ request: r, act }: { request: RequestRecord; act: (r: RequestRecord, a: Action) => void }) {
  const [time, setTime] = useState(Date.now());
  const liveChat = r.verification.source === "providerStream";
  useEffect(() => { const timer = window.setInterval(() => setTime(Date.now()), 1000); return () => clearInterval(timer); }, []);
  return <article className={`request side-${r.pendingSide}`}>
    <div className="row"><strong>{r.provider === "chatgpt" ? "ChatGPT" : "Codex"} · #{r.id}</strong><span className="badge">{r.demo ? "LOCAL DEMO" : liveChat ? "CHATGPT PLAN" : "NOT SENT"}</span></div>
    <p className="request-status" role="status">{HubClient.label(r)}</p>
    <small>{r.conversationId} · {r.eventSource} · event {r.sequence}</small>
    <p className="prompt-text">{r.message}</p>{r.response && <p className="prompt-text">{r.response}</p>}
    {r.detail && <p className="error" role="status">{r.detail}</p>}
    <small>{Math.max(0, Math.floor((time - r.stateSince) / 1000))}s in state · Last local observation {HubClient.timestamp(r.lastConfirmedAt)}</small>
    {liveChat ? <p className="muted">Last verified stream event: {HubClient.timestamp(r.verification.lastVerifiedAt)}{r.verification.freshness === "stale" ? " · stale" : ""}</p>
      : <p className="muted">Status evidence: {r.demo ? "local demo only" : "provider verification unsupported"}.</p>}
    {r.requestedAction && <p className="blocker">{r.requestedAction}</p>}
    {r.demo && !r.terminalOutcome && <div className="actions">
      {r.stage === "unknown" ? <button onClick={() => act(r, "reconcile")}>Reconcile local demo</button>
        : r.stage === "stopping" ? <button onClick={() => act(r, "acknowledgeStop")}>Demo: acknowledge cancellation</button>
        : <>
          {r.stage === "approval" ? <><button onClick={() => act(r, "approve")}>Approve demo step</button><button onClick={() => act(r, "deny")}>Deny</button></>
            : <button onClick={() => act(r, "advance")}>{r.stage === "login" ? "Demo: confirm sign-in" : "Demo: next confirmed event"}</button>}
          <button onClick={() => act(r, "stop")}>Stop demo</button>
          <button onClick={() => act(r, "fail")}>Demo: task error</button>
        </>}
      {r.stage !== "unknown" && <button onClick={() => act(r, "loseConnection")}>Demo: lose connection</button>}
    </div>}
    {!liveChat && <ComputerSessionStatus request={r} act={act} />}
  </article>;
}
