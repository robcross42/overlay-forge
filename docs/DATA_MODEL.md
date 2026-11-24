# Overlay Forge Host Data Model

The host uses `overlay-forge-host.sqlite3`. The legacy `overlay-forge.sqlite3` remains untouched.

## Google identity registry

`overlay-forge-identities.sqlite3` is a separate host-owned SQLite registry in the same app-data directory. Idempotent creation adds `identity_config` (one configured bootstrap email and its pinned Google admin issuer/subject) and `google_profile` (opaque UUID, canonical issuer/subject, current verified email/name labels, with unique issuer/subject). No Google tokens, passwords or client secrets are stored. Bootstrap configuration is read from local `google-auth.json` only before configuration exists; administrator binding and profile upsert commit atomically. The authenticated session and its command-generation ID are memory-only and reset at process start. Role is derived from the pinned issuer/subject, never a renderer-supplied flag. Existing ChatGPT tables and protected payloads are unchanged and remain accessible only through administrator-authorized native commands.

## `obj_host_ui_preferences`

Singleton host presentation preferences, currently the boolean `navigation_collapsed` (default expanded). Idempotent creation and `INSERT OR IGNORE` preserve the saved choice on restart without modifying other tables. Administrator-authorized native commands read and save it through `HostDatabase`; the renderer uses no browser storage. This is a device-local host preference, not window geometry or a product setting.

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
