# Overlay Forge Host Data Model

The host uses `overlay-forge-host.sqlite3`. The legacy `overlay-forge.sqlite3` remains untouched.

## `obj_host_launcher_state`

Singleton row containing only `last_product_key` and `updated_at`. The product key must map to the compiled `ProductKind` enum.

## `obj_host_window_state`

Host-only geometry and visibility keyed by `window_key`.

No product tables, credentials, arbitrary commands, executable paths, or product release versions belong in this database.
