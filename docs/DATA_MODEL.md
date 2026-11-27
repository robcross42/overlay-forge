# Overlay Forge Host Data Model

## Projects

`obj_host_project` stores an opaque ID, the canonical serialized `ProjectDraft` body (title,
description, typed stage, next step, Waiting on Rob, labelled references), and an independent
boolean `archived`. The body is plain local catalog data and contains no financial figures or
credentials. Only the administrator's authorized native commands access these records.

`obj_host_project_seed` records the applied `initial-catalog-v1` seed. Both the ledger marker and
initial rows are committed atomically. Stable seed IDs and `INSERT OR IGNORE` preserve existing
rows; the ledger persists after deletion, preventing resurrection. No product/legacy data is
migrated. `project_delete_requires_archive` rejects deletion of active records even through SQL.

The initial catalog was curated on 27 November 2026 from the named local workspaces' README and
status documentation, plus Rob's supplied stage/review instructions. Retirement workspaces are
one entry; HouseReady/Real Estate and Contractor Simulator/Unity construction sim are aliases.
The fictional lake map has no confirmed implementation/workspace and remains Planned. Ruckus's
local main has no commits and contains staged/untracked prototype work; no new owner playtest
is assigned. Media is pinned at 0.3.1, with Google host sign-in and UI/browser refinement ongoing.
Stages are initial evidence-based notes and are editable; they do not automatically follow Git.

The host uses `overlay-forge-host.sqlite3`. The legacy `overlay-forge.sqlite3` remains untouched.

## Google identity registry

`overlay-forge-identities.sqlite3` is a separate host-owned SQLite registry in the same app-data directory. Idempotent creation adds `identity_config` (one configured bootstrap email and its pinned Google admin issuer/subject) and `google_profile` (opaque UUID, canonical issuer/subject, current verified email/name labels, with unique issuer/subject). No Google tokens, passwords or client secrets are stored. Bootstrap configuration is read from local `google-auth.json` only before configuration exists; administrator binding and profile upsert commit atomically. The authenticated session and its command-generation ID are memory-only and reset at process start. Role is derived from the pinned issuer/subject, never a renderer-supplied flag. Existing ChatGPT tables and protected payloads are unchanged and remain accessible only through administrator-authorized native commands.

## `obj_host_ui_preferences`

Singleton host presentation preferences, currently the boolean `navigation_collapsed` (default expanded). Idempotent creation and `INSERT OR IGNORE` preserve the saved choice on restart without modifying other tables. Signed-in native commands read and save it through `HostDatabase`; the renderer uses no browser storage. This is a device-local preference shared by administrator and family navigation, not window geometry or a product setting.

## `obj_host_launcher_state`

Singleton row containing only `last_product_key` and `updated_at`. The product key must map to the compiled `ProductKind` enum.

## `obj_host_window_state`

Host-only geometry and visibility keyed by `window_key`.

## `obj_host_google_client`

The companion `obj_host_google_client` singleton holds a platform-protected Google Desktop application configuration BLOB, including its client secret and exact configured client ID. Native import uses the existing SecretStore and stable host protection identity. It stores no Google user access, refresh or ID tokens. Existing ChatGPT credentials and transcripts are unchanged.

## `obj_host_ai_connection`

Singleton row with a stable UUID `host_id`, a nullable `protected_credentials` BLOB, and `updated_at`. The native SecretStore protects the main app's ChatGPT registration before SQLite using current-user DPAPI on Windows or AES-256-GCM with a host-scoped key held in Linux Secret Service. Recovery requires the matching platform profile and protection key. Plain access, refresh and ID tokens must never enter database fields, logs or renderer DTOs.

The protected record retains issued client ID, validated subject/email, granted scopes and expiry together with its tokens. Rotation replaces the encrypted record atomically in one SQLite update. Local disconnect clears tokens inside the protected record while retaining the issued registration and host ID for supported reauthorization. The first slice supports one selected ChatGPT registration; multiple-account storage/selection is not implemented.

Migration uses only `CREATE TABLE IF NOT EXISTS` and lazy `INSERT OR IGNORE`, preserves launcher/window state, and never touches the legacy/product databases. The migration test verifies repeat initialization, stable host identity, preserved last product and credential removal. DPAPI tests use synthetic bytes only.

## `obj_host_chatgpt_conversation`

Each row contains an opaque local conversation ID and a protected transcript BLOB. The native host uses the existing platform protection to encrypt the registration ID, ordered completed prompts and replies, and their local completion and stream-evidence timestamps. It atomically replaces the transcript after each completed reply. The prior saved version survives a failed write, and the request card reports the save failure. On startup the host decrypts transcripts locally and offers conversations for the connected registration. Drafts and unfinished replies remain in memory. The app sends conversation history only when the user submits a follow-up.

No product tables/credentials, plain credentials, arbitrary commands, executable paths, or product release versions belong in this database.

## Browser server access database

The headless server owns `browser-access.sqlite3` in its configured private data directory, separate from desktop host/profile databases. Idempotent creation adds `browser_access` (canonical approved email, approval flag, first-login pinned Google subject and navigation visibility) and `browser_session` (SHA-256 session-token hash, approved email/subject, display name, CSRF token, absolute expiry and last authenticated request). No plain session identifiers, Google tokens or client secrets are stored here. Approval/revocation uses parameterized SQLite operations; revocation atomically disables the email and removes its sessions. Reapproval retains the pinned subject. Session lookups join the current approval/subject on every request and prune expired rows. The Google Web application's downloaded client configuration stays in a private file outside Git and the static frontend directory.

The headless Media database path is explicitly configured or defaults to a separate `overlay-forge-media.sqlite3` in that server data directory. It uses the product's existing idempotent initialization. The server never copies databases or credentials from another machine. All approved browser accounts share this family library; per-profile library isolation is not implemented. [Browser setup](BROWSER_SERVER.md) documents data-path selection and access management.
