# Overlay Forge Host Data Model

The host uses `overlay-forge-host.sqlite3`. The legacy `overlay-forge.sqlite3` remains untouched.

## `obj_host_launcher_state`

Singleton row containing only `last_product_key` and `updated_at`. The product key must map to the compiled `ProductKind` enum.

## `obj_host_window_state`

Host-only geometry and visibility keyed by `window_key`.

## `obj_host_ai_connection`

Singleton row with a stable UUID `host_id`, a nullable `protected_credentials` BLOB, and `updated_at`. The native SecretStore protects the main app's ChatGPT registration before SQLite using current-user DPAPI on Windows or AES-256-GCM with a host-scoped key held in Linux Secret Service. Recovery requires the matching platform profile and protection key. Plain access, refresh and ID tokens must never enter database fields, logs or renderer DTOs.

The protected record retains issued client ID, validated subject/email, granted scopes and expiry together with its tokens. Rotation replaces the encrypted record atomically in one SQLite update. Local disconnect clears tokens inside the protected record while retaining the issued registration and host ID for supported reauthorization. The first slice supports one selected ChatGPT registration; multiple-account storage/selection is not implemented.

Migration uses only `CREATE TABLE IF NOT EXISTS` and lazy `INSERT OR IGNORE`, preserves launcher/window state, and never touches the legacy/product databases. The migration test verifies repeat initialization, stable host identity, preserved last product and credential removal. DPAPI tests use synthetic bytes only.

## `obj_host_chatgpt_conversation`

Each row contains an opaque local conversation ID and a protected transcript BLOB. The native host uses the existing platform protection to encrypt the registration ID, ordered completed prompts and replies, and their local completion and stream-evidence timestamps. It atomically replaces the transcript after each completed reply. The prior saved version survives a failed write, and the request card reports the save failure. On startup the host decrypts transcripts locally and offers conversations for the connected registration. Drafts and unfinished replies remain in memory. The app sends conversation history only when the user submits a follow-up.

No product tables/credentials, plain credentials, arbitrary commands, executable paths, or product release versions belong in this database.
