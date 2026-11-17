# AI Anchor Decision and Delivery

## Decision — 14 November 2026

Rob authorizes changing the existing launcher-only main app into the resident personal AI anchor. The existing main app is the anchor; modules depend on its shared AI connections. Preserve the products and their independent data. This supersedes the former host-only restriction for shared AI/account/monitoring responsibilities; it does not authorize product migration, new account access, billing, publishing or hosted deployment.

Source: latest Library `overlay-forge-codex-handoff.md`, version 2, revised 14 November 2026, read before implementation. This supporting decision is routed from Project Overview, Architecture, Scope and Repository Boundaries rather than adding another root documentation entry point.

## Incremental host-services follow-up - 16 November 2026

The host runs five-second local freshness checks and keeps evidence from provider stream events separate. The worker makes no provider status calls. See the [data model](DATA_MODEL.md) for native profile protection and local transcripts.

## Top priority: scheduled, verified progress status

Rob clarified on 16 November 2026 that this is a top-priority delivery requirement: a progress indicator must refresh on a schedule and show which side is pending for each Codex/OpenAI request. Event-driven updates alone are insufficient. This section records required work; scheduled authoritative reconciliation is not claimed to be implemented or verified by the current demo or connection test.

- Keep pending side separate from execution stage and the dependency being awaited. Show user, assistant, neither, or unknown as the pending side; distinguish in progress, waiting for user, waiting for a named tool, complete, failed, and unknown. A tool wait must not falsely imply that the user is blocking work or that the model is generating. Use more specific thinking/streaming/tool labels only when supported events establish them.
- Show the last **verified** update time (including date and time zone), evidence source and freshness alongside the state. Track scheduled local refresh/check time separately: rereading a cached snapshot or repainting elapsed time does not verify continuing provider activity and must not advance the verified timestamp.
- Add a bounded, documented refresh cadence and adapter-specific stale thresholds, with the next scheduled check visible. Rob has not specified an exact interval. Reconcile from supported provider status/events or owned local process/job state, with bounded backoff and no overlapping polls. Local liveness confirms only local liveness; it cannot prove remote Codex/OpenAI work is active.
- When fresh authoritative evidence is unavailable, retain the last known state and its timestamp but clearly mark current activity stale/unknown. Distinguish a failed status check from a confirmed failed request. A verified terminal outcome stays complete/failed/cancelled; stale connection health must not regress it into invented active work or erase its outcome.
- If an adapter exposes no supported status/reconciliation capability, say status verification is unsupported or unavailable. Do not claim access to internal ChatGPT/Codex reasoning, remote jobs, or APIs that have not been established. Do not infer progress from prose, a spinner, elapsed time, open windows or an acknowledged completion marker; no invented percentages or completion estimates.
- Reuse the native request registry and adapter contracts as the single owner of state/freshness classification. Main and compact views must agree, retain request/provider/account ownership, expose user blockers and Stop, and identify the request behind aggregate counts. A scheduled status check must not submit inference, replay work, expand permissions, change billing or restart a task.
- After disconnect, sleep/wake or restart, reconcile before claiming renewed activity. Computer/screen-session resource and indicator evidence remains independently visible; conversational completion never proves capture release.

Acceptance requires a deterministic clock/scheduler and adapter fixtures covering scheduled refresh without new stream events, absent/unsupported status APIs, cached snapshots, tool/user blockers, stale evidence, poll failures, disconnect/reconnect and sleep/wake, concurrent requests, out-of-order events, preserved terminal outcomes, and matching main/compact labels. Each test must distinguish verified source evidence from local display refresh. Real provider claims require an authorized supported-adapter test; local simulations do not establish live API support.

## Initial local slice

| File | Responsibility |
| --- | --- |
| `src-tauri/src/ai_hub.rs` | Bounded native session drafts and request registry; typed providers, pending sides, stages, blockers, action sequence checks and terminal guards |
| `src-tauri/src/computer_session.rs` | Separate capture-resource and indicator evidence, cleanup reasons, stale-evidence guards and metadata-only orphan simulations; opens no capture resources |
| `src-tauri/src/hub_commands.rs` | Thin command boundary and change notifications |
| `src-tauri/src/windows.rs` | Centralized composed window configs, hidden prompt, work-area placement, focus restoration and resident lifecycle |
| `src-tauri/src/lib.rs` | Single-instance, tray and global-shortcut integration; visible startup errors |
| `src/ai/` | Shared DTOs, snapshot subscription, mutation serialization and accessible request cards |
| `src/app/App.tsx` | Dashboard, provider switch, prompt, unavailable account/monitoring surfaces, activity and product launcher |

Both windows read one native registry. The ChatGPT composer sends explicit submissions through the connected account when ChatGPT plan usage is granted. An unavailable adapter preserves its draft and records a local failure, and the opt-in **local demo** advances only on explicit demo actions. Demo approvals grant no external access. Each submitted turn has its own request within the selected conversation. Completed ChatGPT conversations are protected in the local host database and restore at startup. Drafts and unfinished or demo requests stay in memory, bounded to 50 unsaved requests and 16,000 UTF-8 bytes per draft. Nothing private is logged.

New dependency: pinned `tauri-plugin-single-instance` 2.4.4; existing Tauri tray/global-shortcut capabilities are reused. Frontend packages are restored from the existing lockfile. Existing product identity checks, launch destinations, resume state and independent integrations are preserved. Successful launches no longer exit the anchor. No arbitrary launch path or URL is introduced.

## Task status and computer-session release

Rob reported that the ChatGPT Windows blue border and computer-use banner remained visible after completion was acknowledged. That observation does not establish whether capture continued or the indicator was stale. This project makes no claim about ChatGPT internals and cannot release another app's capture session.

Overlay Forge tracks these independently:

| Observable dimension | Evidence and visible behavior |
| --- | --- |
| Conversation/task | Explicit queued, connecting, in-progress, thinking, tool, replying, waiting-for-user, stopping and terminal outcomes. Thinking requires a supported adapter event; the current thinking event is explicitly a local demo. No private reasoning is exposed. |
| Capture resources | Not started, active, release requested, released, or unknown. Task completion, a cleanup request or a hidden banner cannot prove resources were released. Active is currently available only as simulated metadata. |
| Indicator | Last observed not shown, visible, or cleared, with evidence time. A stale visible indicator can coexist with confirmed resource release; a cleared indicator can coexist with unknown resources. |

The shared request card exposes both dimensions in main and compact views. **Stop all local work / request cleanup** remains available independently of active-conversation state, and per-request cleanup remains available after terminal outcomes. Complete, cancel, error, Stop and app Quit all route through the same native cleanup boundary. The current slice has no capture adapter: synthetic cleanup becomes visibly unverified until separately simulated resource-release and indicator observations arrive. No new computer-use session, screen capture, input hook or OS resource is started by these simulations.

Native session evidence has request ownership, a lease generation and ordered evidence sequence. Old-generation and out-of-order observations cannot clear current uncertainty. Unresolved resources/indicators prevent terminal request eviction and block normal Quit, preserving the warning rather than silently dropping ownership. The two demo confirmation buttons exercise distinct observations; neither is presented as verification of a real screen session.

Before real capture support, introduce a native adapter owning every stream, hook, child process, subscription and indicator. Its cleanup must be idempotent, cancel input first, close each owned resource, then verify resource release and indicator clearance independently; failures and timeouts remain unknown with a retry path. Persist non-secret lease ownership before allocation and reconcile orphaned leases after crash/restart, using process identity and generation checks to avoid affecting unrelated processes. Hook forced shutdown and adapter disconnect into cleanup. The current volatile registry does not provide crash recovery, and no real adapter or release-verification transport is implemented. Do not enable capture until these gates and authorized smoke tests pass.

## Approved architecture and remaining delivery

### Live ChatGPT connection and conversations

The native `chatgpt` module implements the documented local-app Sign in with ChatGPT flow. `oauth.rs` owns random one-use state/nonce/S256 PKCE, exact `127.0.0.1:<available-port>/auth/callback` binding, issued-client registration/reauthorization and RS256 signature/JWKS/issuer/audience/expiry/nonce validation. `secrets.rs` owns the facade for native platform protection. `transport.rs` owns the account-specific model catalog, shared Responses stream decoder and structured failures. The service runs one account or inference operation at a time and keeps OAuth tokens out of the renderer.

Use **Continue with ChatGPT** and review consent in the system browser. In **Chat**, **Send to ChatGPT** streams each reply into its request card using the first model in the account's current catalog. The optional **Send harmless verification prompt** uses the fixed input `Say exactly: Overlay Forge connection verified.` Both actions use `store:false`, `stream:true`, and explicit array input. They require `response.completed`, and partial replies stay visible if a stream ends early. Stop ends work locally, with remote cancellation unconfirmed. No prompt is replayed automatically.

The **Conversation** picker returns to the latest conversation by default. Choose **New chat** for a separate context. Each follow-up sends completed user and assistant turns from the selected conversation, in order, followed by the new prompt. Failed or stopped turns stay visible for reference and are excluded from context. Conversations are bound to their ChatGPT registration and the transport verifies the registration again before sending.

Funding is the authorized ChatGPT registration, subject to eligibility, app/workspace limits and any credits the user permits in ChatGPT settings. A validated identity alone does not grant inference. API-key billing and silent fallback are absent. Configuration was inspected for presence only: `OPENAI_API_KEY` exists but is neither read nor reused; no app-specific client or access-token variables were present. Dynamic registration does not require a client secret. Public OpenID discovery was retrieved without credentials and matched the documented endpoints and RS256 signing algorithm.

Token refresh is serialized, retains omitted scopes, rotates token sets together and clears unusable renewable credentials on documented terminal refresh errors. Disconnect attempts the issuer-discovered, HTTPS/OpenAI-only revocation endpoint and distinguishes confirmed revocation from local removal. Global Stop cancels the owned connection attempt; blocking network work may take up to 90 seconds to return, and no remote cancellation is falsely confirmed. Normal Quit pauses while connection work is still owned. This is separate from the computer-session lifecycle; no capture or external account actions are enabled.

Codex app-server transport, Google identity/service grants, multiple ChatGPT accounts, attachments, tools and module broker IPC remain separate work. The fixed ChatGPT verification prompt and a two-turn composer exchange completed successfully in the Windows preview. See the [host data model](DATA_MODEL.md#obj_host_chatgpt_conversation) for the local transcript format.

Official contract checked: [registration/sign-in](https://developers.openai.com/siwc/token-sharing-open-source/sign-in), [models/inference](https://developers.openai.com/siwc/token-sharing-open-source/models-and-inference), [accounts/refresh/revocation](https://developers.openai.com/siwc/token-sharing-open-source/profiles-and-sessions), [preview limitations](https://developers.openai.com/siwc/token-sharing-open-source/preview-limitations). Availability for this local/private app and Rob's account must be established by the live trial; no eligibility promise is made.

### Remaining delivery

Top priority before expanding the AI experience: implement the [scheduled verified status requirement](#top-priority-scheduled-verified-progress-status), including cadence, source-aware timestamps and stale/unknown reconciliation. Existing lifecycle labels and local demo events are a foundation, not acceptance of this requirement.

1. Finish the resident shell: custom shortcut configuration, durable non-secret preferences, `WindowStateRepository` geometry restoration, display/sleep recovery, and measured warm-open/acknowledgment timing. Verify focus and tray behavior on real displays. Provider changes never reroute active requests; hide is distinct from cancellation.
2. Add native identity/account registry, provider-specific capability declarations and Windows-protected secret storage. Google primary identity uses supported desktop authorization code/PKCE, validated loopback/state/nonce and ID-token signature/issuer/audience/expiry. Account identity is issuer plus subject, not email. Separate Google identity from separately connected services and actual granted scopes; do not assume desktop incremental consent behaves like a web-server flow. No token enters React, localStorage, plain SQLite, logs or command arguments.
3. Implement a shared native AI broker: verify current official ChatGPT sign-in eligibility and streaming requirements, explicit history and `store:false`; pin Codex app-server and matching protocol types, manage stdio and refresh, workspace/action approvals and interruption. Use a shared HTTP client with timeouts/cancellation. Show verified plan/API funding; API billing is opt-in and cannot be a silent fallback. Real adapters need provider sequence/correlation IDs, stale thresholds and authoritative reconnect reconciliation before they can drive these status labels.
4. Add provider-specific connected-account adapters with independent identity/read/search/monitor/write capabilities, supported resources/scopes, rate limits and revocation handling. Reconcile actual grants and distinct account IDs. Provider access, sharing selected content with AI and authorizing a write are separate decisions. External content is untrusted data. Disconnect stops new jobs and deletes protected local secrets; partial remote revocation failures remain visible. Validate OAuth denial/state failures, expiry, partial scopes and account isolation with explicit user authorization.
5. Implement durable local monitoring: typed bounded jobs, backoff, per-account cursors, deduplication, checkpoint-after-success, pause/revocation and bounded catch-up after sleep. Rules own scope, purpose, cadence/time zone, quiet hours, destination, retention and last/next outcomes. Gmail push requires a separately authorized Pub/Sub/renewal design; a resident desktop does not imply always-on service. Exact external actions require explicit scoped approval, and uncertain action completion is never automatically replayed.
6. Keep Dot honest: no native session endpoint is configured or claimed. A verified external conversation link can become a labelled handoff. The experimental MCP Events bridge requires separately approved hosted infrastructure, authenticated endpoints, signed delivery, ownership, deduplication, expiry and revocation. A new model session is not the existing Dot.
7. Introduce the versioned authenticated module contract after broker authorization exists. Include request/conversation IDs, selected provider/account, capability, explicit context, cancellation ownership and data isolation. Never expose an unauthenticated local AI endpoint. Existing product integrations continue until deliberately migrated.

No live sign-in button is exposed until an adapter is implemented and verified. Remaining stages are part of the requested product, not removed scope. The current slice is not acceptance of full milestone 1 or any live provider milestone.
