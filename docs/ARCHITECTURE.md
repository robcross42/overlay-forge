# Overlay Forge Host Architecture

The main app is the resident personal AI host under the [14 November boundary decision](AI_ANCHOR.md). React renders dashboard and compact-prompt views. Rust owns product dispatch, native window lifecycle and per-request state. `ProductKind` remains a closed enum.

`HostDatabase` owns launcher/window state, protected ChatGPT credentials and protected completed conversations. `AiHub` holds provider-separated drafts, active requests and restored conversations. The native ChatGPT service assembles scoped history, owns credentials and streams through one Responses transport. Both windows subscribe before reading versioned snapshots. See the [data model](DATA_MODEL.md).

`ProfileService` and its `ProfileRepository` own Google session identity and the pinned administrator. `google_oauth` verifies browser sign-in and discards Google tokens; `browser_auth` is the shared browser-opening boundary used by both Google and ChatGPT. Every hub and ChatGPT command holds the profile guard through access checks and worker reservation. Session IDs reject delayed renderer commands after sign-out/switch. Profile transitions wait for worker completion and settled resource ownership, clear `AiHub`, and restore saved conversations only for the administrator. The frontend session gate unmounts both windows' private views during transitions; native authorization does not depend on hiding buttons. The singleton ChatGPT connection remains administrator-owned; family access is launcher-only. See [setup and session limits](../README.md#google-sign-in-and-family-access).

`ComputerSession` separately owns capture-resource evidence and last-observed indicator state for each request. Task completion never implies release. All normal task terminal paths, Stop and Quit request cleanup; absent adapter verification remains unknown. Unresolved ownership prevents eviction and normal Quit. The present implementation is metadata-only: no capture API is called. Durable lease recovery and a real native resource-owning adapter remain prerequisites; see [the lifecycle contract](AI_ANCHOR.md#task-status-and-computer-session-release).

`WindowManager` creates both windows from typed composed configs and owns showing, hiding, focusing, placement and Quit. `WindowKind` replaces ad hoc label selection at call sites. The single-instance plugin forwards subsequent typed product invocations; tray controls and the global-shortcut plugin retain access to the resident app. Escape hides without cancelling. Windows focus restoration is best effort and errors are visible. Window-state persistence is not implemented and no `WindowStateRepository` is added until geometry restoration is implemented.

Startup sequence:

1. Resolve an optional typed command-line target.
2. Otherwise resolve the last-used typed product.
3. Launch an installed executable or known sibling development build.
4. Keep the main app resident after successful dispatch.
5. Show the dashboard and product picker, including when a target is unavailable. Show launch failures instead of aborting startup.

The standalone repositories deliberately own their duplicated shell infrastructure for the first split. A shared-core repository is not part of this generation.
