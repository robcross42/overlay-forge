import { Action, RequestRecord } from "./hub";

const captureLabels = {
  notStarted: "Capture: not started — no capture adapter is installed",
  active: "Capture: active in local simulation only",
  releaseRequested: "Capture release requested — awaiting verification",
  released: "Capture resources: release verified in local simulation",
  unknown: "Capture: unknown — release is unverified",
};
export function ComputerSessionStatus({ request: r, act }: { request: RequestRecord; act: (r: RequestRecord, a: Action) => void }) {
  const c = r.computer;
  return <section className="computer-session" aria-label={`Computer session for request ${r.id}`}>
    <h3>Computer & screen session</h3>
    <p role="status">{captureLabels[c.captureState]}</p>
    <p>Indicator last observed: {c.indicatorState === "visible" ? "visible (possibly stale)" : c.indicatorState === "cleared" ? "cleared" : "not shown"}.</p>
    {c.demo && <>
      <p className="muted">Metadata-only simulation. No screen capture, input hook or computer-use session was opened.</p>
      <p>Outstanding simulated resources: {c.remainingResources ?? "unknown"}. Task completion and indicator appearance are not release evidence.</p>
      <small>Session {c.generation} · evidence {c.sequence} · Last confirmed {c.lastConfirmedAt ? new Date(c.lastConfirmedAt).toLocaleTimeString() : "never"}</small>
      {c.releaseRequested && <p className="blocker">Cleanup requested ({c.cleanupReason}). {c.captureState === "unknown" ? "No adapter can verify release. This warning remains visible." : "Resource release and indicator clearance are tracked separately."}</p>}
      {c.lastCleanupAttemptAt && <small>Last cleanup request {new Date(c.lastCleanupAttemptAt).toLocaleTimeString()}</small>}
    </>}
    {r.demo && <div className="actions">
      {!c.demo && !r.terminalOutcome && <button onClick={() => act(r,"demoActive")}>Demo: model active capture metadata</button>}
      {!c.demo && <button onClick={() => act(r,"demoOrphan")}>Demo: model orphaned indicator</button>}
      {c.demo && <>
        <button onClick={() => act(r,"cleanup")}>Stop / request session cleanup</button>
        {c.releaseRequested && c.remainingResources !== 0 && <button onClick={() => act(r,"demoVerifyResources")}>Demo: confirm resources released</button>}
        {c.indicatorState === "visible" && <button onClick={() => act(r,"demoClearIndicator")}>Demo: confirm indicator cleared</button>}
      </>}
    </div>}
  </section>;
}
