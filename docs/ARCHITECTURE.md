# Overlay Forge Host Architecture

The React frontend is a minimal fallback picker. Rust owns product identity, executable discovery, process launch, and last-used state. `ProductKind` is the closed domain enum; no executable path or shell command is read from SQLite or user-editable configuration.

`HostDatabase` is the only persistence abstraction. It owns `obj_host_launcher_state` and `obj_host_window_state`. Product data and credentials are forbidden in the host database.

Startup sequence:

1. Resolve an optional typed command-line target.
2. Otherwise resolve the last-used typed product.
3. Launch an installed executable or known sibling development build.
4. Exit the host after successful dispatch.
5. Show the fallback picker only when no valid executable is available.

The standalone repositories deliberately own their duplicated shell infrastructure for the first split. A shared-core repository is not part of this generation.
