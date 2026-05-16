# Changelog

All notable changes to Overlay Forge will be documented in this file.

## Unreleased

### Milestone Status

- Milestone 0 - Overlay Shell Validation is complete, passed, and successful.
- The Milestone 0 scratchpad component is complete and passed.
- Scratchpad content saves to SQLite and restores between app sessions.
- Milestone 1 - Calendar, To-do, Notes, and Scratchpad Expansion is complete, passed, and successful.
- Current project baseline is Milestone 2.
- Milestone 2 - Local Projects component is complete, passed, and successful.

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
- Added Scratchpad, Tasks, Notes, and Calendar component navigation.
- Added SQLite `tasks`, `notes`, and `calendar_events` tables.
- Added Tauri CRUD commands for tasks, notes, and calendar events.
- Added Tasks component with create, select, edit, delete, deadline, body, list, and restart restore support.
- Added Notes component with create, select, edit, delete, list, and restart restore support.
- Added Calendar component with create, edit, delete, list, and restart restore support.
- Added `docs/MILESTONE_1.md` with user validation steps.
- Added `docs/DATA_MODEL.md` for the current SQLite schema.
- Added Projects component navigation.
- Added SQLite `projects` table.
- Added Tauri CRUD commands for local projects.
- Added Projects component with create, select, read-only view, edit, delete, list, status, and restart restore support.
- Added `docs/MILESTONE_2.md` with setup validation and manual validation steps.

### Changed

- Configured the app bundle identifier as `com.overlayforge.desktop`.
- Configured the Windows app icon for Tauri bundling.
- Updated the bridge file with current validation status and manual checks.
- Fixed custom window controls so titlebar drag handling no longer intercepts button clicks.
- Added explicit Tauri window permissions for minimize, hide, maximize, drag, and resize commands.
- Updated project documentation for Milestone 1 implementation status.
- Changed global hotkey registration so a taken hotkey logs a warning instead of preventing app startup.
- Changed Tasks to use list selection plus edit mode instead of main-list checkboxes and delete buttons.
- Added task body and deadline support with non-destructive SQLite column migration.
- Changed Notes empty state so editor fields and save/delete controls are hidden until a note exists.
- Changed active Delete buttons to use the same enabled visual treatment as Save.
- Changed Calendar date/time inputs to open native controls when clicking anywhere in the field.
- Changed Calendar start date/time updates to automatically adjust end date/time.
- Changed Calendar Delete visibility so it appears only for selected existing events.
- Changed Tasks and Calendar to match Notes empty-state behavior by hiding editor controls until New or item selection.
- Added UI consistency rules for organizer component empty states, edit modes, destructive actions, and enabled button styling.
- Changed startup behavior so the overlay starts hidden in the background and is shown with the global hotkey.
- Added a shutdown titlebar control that exits the app process.
- Added an explicit Edit button for selected existing tasks.
- Added explicit Edit buttons for selected existing notes and calendar events.
- Documented the Windows WebView2 shutdown class-unregister log as deferred cleanup.
- Clarified milestone numbering in project docs so Milestone 2 is not confused with the second item in the roadmap.
- Updated project documentation for Milestone 2 validation success.

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
- Verified frontend build after Milestone 1 implementation with `npm run build`.
- Verified Rust backend compile after Milestone 1 implementation with `cargo build`.
- Verified production Tauri build after Milestone 1 implementation with `npm run tauri:build`.
- Verified rebuilt release app starts against the existing app-data SQLite database.
- User completed remaining Milestone 1 validation and reported it finished.
- Verified frontend build after Milestone 2 implementation with `npm run build`.
- Verified Rust backend compile after Milestone 2 implementation with `cargo build`.
- Verified production Tauri build after Milestone 2 implementation with `npm run tauri:build`.
- Verified rebuilt release app starts hidden in the background after Milestone 2 implementation.
- Fixed Projects status dropdown option readability.
- User manually validated Milestone 2 successfully.
