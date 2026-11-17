# Overlay Forge Host Scope

In scope: private product discovery, typed target dispatch, last-used resume, host window state, resident lifecycle, and the approved personal AI anchor described in [AI Anchor Decision and Delivery](AI_ANCHOR.md).

The AI anchor includes Google primary identity, separately authorized service access, extensible connected accounts, ChatGPT/Codex shared AI connections, monitoring, action approvals, and Dot feasibility. The resident shell, local request-state demonstration, native ChatGPT sign-in, fixed streaming account verification and multi-turn ChatGPT composer with protected local history are implemented. Account read/write, scheduled remote monitoring, hosted services and module connection APIs remain unavailable.

Out of scope: embedded product UI or business logic, product settings/data migration, dynamic code loading, marketplace behavior, automatic updates, and executable commands stored in SQLite or editable configuration. Hosted monitoring requires separate authorization.

The main and compact views report the last verified ChatGPT stream event alongside a five-second local status check. See [verified progress status](AI_ANCHOR.md#top-priority-scheduled-verified-progress-status) for freshness and remote reconciliation limits.

Product work belongs in the Media, Gaming, Retirement, or Journal repository and changes only that product's version. Journal adds only a fixed local browser target to the host; its responsive UI, server, SQLite storage, and OpenAI integration remain independently owned. Hosting and sign-in are deferred.

Deferred module concepts are not current scope. `LanInputBridge` is recorded in
[Future Module Concepts](FUTURE_FEATURES.md); implementing it requires an explicit decision to
change the current host-only repository boundary.
