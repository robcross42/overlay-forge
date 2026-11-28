import type { ChatgptStatus } from "./client";

export type Integration = { name: string; detail: string; open?: () => void };

export function BrowserAdminHome({ integrations }: { integrations: Integration[] }) {
  return <section className="panel">
    <h1>Home</h1>
    <h2>Integrations</h2>
    {integrations.map(({ name, detail, open }) => <article className="account" key={name}>
      <strong>{name}</strong>
      <p>{detail}</p>
      {open && <button type="button" onClick={open}>Open {name}</button>}
    </article>)}
  </section>;
}

export function BrowserMonitoring({ chatgpt, error, refresh }: {
  chatgpt?: ChatgptStatus; error: string; refresh: () => void;
}) {
  return <section className="panel">
    <header className="row">
      <h1>Monitoring</h1>
      <button type="button" onClick={refresh}>Refresh status</button>
    </header>
    <article className="account">
      <strong>ChatGPT</strong>
      <p role="status">{chatgpt?.detail ?? "No ChatGPT activity is available."}</p>
      {chatgpt?.observedAt && <p>
        Last recorded update: <time dateTime={new Date(chatgpt.observedAt * 1000).toISOString()}>
          {new Date(chatgpt.observedAt * 1000).toLocaleString()}
        </time>
      </p>}
    </article>
    {error && <p className="error" role="alert">{error}</p>}
  </section>;
}
