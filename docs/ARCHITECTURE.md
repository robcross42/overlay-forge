# Overlay Forge Host Architecture

## Projects catalog

Projects is a host-owned module rendered in `AppShell`, alongside embedded Media. It adds no
repository, package, window, runtime, login or integration service. `src/projects/` contains the
presentational list/editor and typed session-bound command client. Native `projects/` owns the
draft/stage model, validation, initial catalog and canonical SQLite repository. `HostDatabase`
supplies its existing connection and initializes the additive schema.

Every Projects command requires the existing pinned-administrator session guard and retains it
through the operation. Family sessions have neither navigation nor data access; stale session
IDs fail the same guard. The browser server exposes no Projects endpoint.
Catalog data is local host data, not transmitted to AI or metadata providers. It is not an
encrypted vault; OS-account/database access still governs local files.

References are inert absolute paths or HTTPS URLs. Paths are displayed as selectable text and
never opened, executed, traversed or changed. Explicit HTTPS clicks reuse `browser_auth`.
Archive is separate from stage; permanent deletion checks archived state and exact current title
inside one repository transaction. A SQLite trigger independently rejects active-record deletion.
Initial seeding uses a transactional ledger separate from records, so edits, archives and deleted
seed entries survive future initialization. This is a curated snapshot, not live project scanning.

The main app is the resident personal AI host under the [14 November boundary decision](AI_ANCHOR.md). React renders dashboard and compact-prompt views. Rust owns product dispatch, native window lifecycle and per-request state. `ProductKind` remains a closed enum.

`HostDatabase` owns launcher/window state, protected ChatGPT credentials and protected completed conversations. `AiHub` holds provider-separated drafts, active requests and restored conversations. The native ChatGPT service assembles scoped history, owns credentials and streams through one Responses transport. Both windows subscribe before reading versioned snapshots. See the [data model](DATA_MODEL.md).

`ProfileService` and its `ProfileRepository` own Google session identity and the pinned administrator. `google_oauth` verifies browser sign-in and discards Google tokens; `browser_auth` is the shared browser-opening boundary used by both Google and ChatGPT. Every hub and ChatGPT command holds the profile guard through access checks and worker reservation. Session IDs reject delayed renderer commands after sign-out/switch. Profile transitions wait for worker completion and settled resource ownership, clear `AiHub`, and restore saved conversations only for the administrator. The frontend session gate unmounts both windows' private views during transitions; native authorization does not depend on hiding buttons. The singleton ChatGPT connection remains administrator-owned; family access uses the integrated Media shell and existing product launch targets; host AI remains administrator-only. See [setup and session limits](../README.md#google-sign-in-and-family-access).

`ComputerSession` separately owns capture-resource evidence and last-observed indicator state for each request. Task completion never implies release. All normal task terminal paths, Stop and Quit request cleanup; absent adapter verification remains unknown. Unresolved ownership prevents eviction and normal Quit. The present implementation is metadata-only: no capture API is called. Durable lease recovery and a real native resource-owning adapter remain prerequisites; see [the lifecycle contract](AI_ANCHOR.md#task-status-and-computer-session-release).

`WindowManager` creates both windows from typed composed configs and owns showing, hiding, focusing, placement and Quit. `WindowKind` replaces ad hoc label selection at call sites. The single-instance plugin forwards subsequent typed product invocations; tray controls and the global-shortcut plugin retain access to the resident app. Escape hides without cancelling. Windows focus restoration is best effort and errors are visible. Window-state persistence is not implemented and no `WindowStateRepository` is added until geometry restoration is implemented.

Startup sequence:

1. Show the main host's Google sign-in surface.
2. Verify identity and establish the host role/session.
3. Restore host chat data only for the administrator and focus the connection dashboard; family sessions open Media. Authentication never launches a product or resumes the last-used module.
4. Open products only after an explicit picker selection or typed target forwarded to the already signed-in host.
5. Keep the host resident and show product availability/launch errors without moving identity ownership into a module. Last-used product state remains stored but is not an automatic post-authentication launch instruction.

## Integrated Media

`AppShell` owns navigation, independent scrolling, persisted collapse state and the common Quit flow for administrator and family views. Home renders connection status tiles only. `EmbeddedMedia` mounts the owning product's React UI in a shadow root to isolate its stylesheet; product content titles remain visible while repeated page/section headings are omitted. Product links are direct navigation entries.

`modules/media` pins the Media repository via Git submodule. Cargo links its crate with the standalone feature disabled. `integrated_media` initializes the same product-owned database path and forwards only its shared command registry, rejecting missing/stale sessions and non-main-window requests. Settings writes and legacy import are administrator-only. Authorization guards remain held through synchronous Media commands so a session cannot switch during a mutation. Product services, mapping, schema and provider behavior remain owned by Media; no source copy or second process/server is used. The injected renderer transport adds the session ID before calling native commands. The existing Media data is device-local and shared across host profiles.

The host still launches Gaming, Retirement, and other standalone products through their existing paths. Media's standalone developer bootstrap uses the same UI, registry and services.

## Integrated Hockey Cards

The pinned `modules/hockey-cards` repository exports a window-free Rust query service and the React inventory view. Native `hockey_inventory_query` requires the main window and an active administrator session. The opt-in browser route uses approved Google identity, the configured owner, exact Origin and CSRF checks, and the existing bounded blocking-worker pool. Both adapters call the same product query, which selects the latest complete saved USD snapshot and applies combined filters and stable pagination in SQLite.

Desktop looks for `collection.sqlite3` in the sibling product app-data directory `com.robcross42.hockeycardcompanion`. Browser mode uses its explicitly configured product database or the default path in its private service data directory. Both open the database read-only. A missing product database shows an empty state and leaves the host available.

## Browser/server mode

The headless `server` crate is a separate process entry point which requires no Tauri or desktop libraries. It serves the built frontend and same-origin HTTP API behind an owner-managed HTTPS reverse proxy. `BrowserApp` is selected outside Tauri; `BrowserClient` supplies the existing Media renderer with an HTTP transport instead of native invoke. The MediaRuntime adapter in the pinned product crate maps typed commands to its existing services and database. No library business rules are copied into HTTP handlers. Client link actions resolve safe URLs and open them in that client's browser.

`AccessRepository` owns manually approved emails, first-login Google subject binding, session hashes, CSRF tokens and per-account navigation preference in a separate server SQLite database. This does not use the desktop singleton ProfileService or its administrator binding. Google Web authorization uses browser-bound, single-use state, PKCE and nonce; both desktop and server share `crates/google-identity` signature/claim verification. Browser email-first approval additionally requires verified Gmail/Workspace email authority. Google tokens are discarded after identity verification.

Every HTTP Media command checks an unexpired approved session and exact Origin/CSRF before dispatching the fixed family command registry. Settings mutation, file imports, process launching and native host/ChatGPT commands are absent from HTTP routing. Revocation deletes sessions and future data requests fail. Shared Media data stays on the server. The server's loopback listener does not trust forwarded-origin headers; TLS/proxy configuration uses its explicit public origin. See [Browser server setup](BROWSER_SERVER.md) for data paths, configuration and remaining runtime acceptance.

Local LLM Chat extends the same browser navigation. Its visibility and HTTP endpoint use one owner check against the configured email after the existing repository confirms an approved, Google-subject-bound session. The endpoint applies the same Origin and CSRF checks as Media. The `LocalLlm` service owns input validation and a separate, bounded Ollama transport pinned to Ubuntu loopback and `qwen3:8b`. The browser sends recent turns with each request and keeps its transcript in memory for the active session.
