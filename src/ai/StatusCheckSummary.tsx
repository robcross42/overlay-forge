import { useEffect, useState } from "react";
import { HubClient, StatusChecks } from "./hub";

export function StatusCheckSummary({ checks }: { checks: StatusChecks }) {
  const [now, setNow] = useState(Date.now());
  useEffect(() => {
    const timer = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(timer);
  }, []);
  const overdue = now > checks.nextCheckAt + checks.intervalMs;
  return <section className="panel" aria-label="Scheduled status checks">
    <strong>Local status checks every {checks.intervalMs / 1000} seconds</strong>
    {overdue && <p role="status">Local checks are overdue. Current remote activity is unverified.</p>}
    <p>Last check: {HubClient.timestamp(checks.checkedAt)}. Next check: {HubClient.timestamp(checks.nextCheckAt)}.</p>
    <p className="muted">Checks inspect native status and stream freshness. They do not call a provider or restart work. Cached reads do not verify remote progress. ChatGPT and Codex status polling is unsupported.</p>
  </section>;
}
