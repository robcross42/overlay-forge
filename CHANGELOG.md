# Changelog

All notable changes to Overlay Forge will be documented in this file.

## Unreleased

### Milestone Status

- Milestone 0 - Overlay Shell Validation is complete, passed, and successful.
- The Milestone 0 scratchpad component is complete and passed.
- Scratchpad content saves to SQLite and restores between app sessions.

### Added

- Created the initial Tauri v2 desktop application scaffold.
- Added React + TypeScript frontend structure.
- Added Rust backend command structure for Tauri.
- Added a dark always-on-top overlay shell.
- Added `Ctrl+Shift+Space` global hotkey registration to toggle the overlay.
- Added a component host with an initial scratchpad component.
- Added SQLite initialization through the Rust backend.
- Added persisted scratchpad storage in SQLite.
- Added automatic scratchpad restoration across app restarts.
- Added a custom draggable titlebar for the borderless overlay window.
- Added custom minimize, maximize/restore, and hide overlay controls.
- Added edge and corner resize handles for adjusting the overlay size and shape.
- Added root documentation for Milestone 0, architecture, project plan, and bridge files.
- Added `bridge-files/OPENAI_APP_BRIDGE.md` as the manual ChatGPT/Codex bridge file.

### Changed

- Configured the app bundle identifier as `com.overlayforge.desktop`.
- Configured the Windows app icon for Tauri bundling.
- Updated the bridge file with current validation status and manual checks.
- Fixed custom window controls so titlebar drag handling no longer intercepts button clicks.
- Added explicit Tauri window permissions for minimize, hide, maximize, drag, and resize commands.

### Validation

- Passed Milestone 0 overlay shell validation.
- Passed Milestone 0 scratchpad component validation.
- Verified `npm install` completes successfully.
- Verified frontend build with `npm run build`.
- Verified Rust development compile with `cargo build`.
- Verified production Tauri build and Windows bundles with `npm run tauri:build`.
- Verified production Tauri build after the window-control click fix.
- Verified the compiled app launches briefly.
- Verified SQLite database creation at `%APPDATA%\com.overlayforge.desktop\overlay-forge.sqlite3`.
- User manually verified scratchpad persistence between app sessions.
