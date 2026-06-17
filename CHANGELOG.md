# Changelog

All notable changes to Overlay Forge will be documented in this file.

Unreleased changes are grouped by day using `YYYY-MM-DD` headings so a single day's work can be reviewed quickly.

## Unreleased

## 0.3.0 - 2026-06-16

Development bucket for work started on 2026-06-16. Related changes that continue into the early AM hours before the work session ends are included in this `0.3.0` bucket.

### 2026-06-16

#### Added

- Added normalized GearBlocks runtime indexes for part API attributes, discovered `value` fields, properties, and attachments so repeated exports update searchable rows instead of relying only on each part's full JSON blob.
- Added canonical GearBlocks API catalog tables for documented construction namespace types, members, and parameters, plus a runtime part/member bridge that maps observed API availability back to canonical member IDs.

#### Changed

- Bumped Overlay Forge from `0.2.0` to `0.3.0` for the next early-development work set.
- Marked the 2026-06-16 work session as the `0.3.0` changelog bucket, including related early-AM rollover changes.
- Added project rules for cutting a new `0.x.0` minor version bucket when a new chat starts on a later date than the latest changelog version heading.

#### Fixed

- Reduced GearBlocks exporter payload bloat by keeping cached known API indexes limited to top-level part availability metadata and removing the remaining behaviour reference value from `apiAttributes`.
- Changed the GearBlocks exporter log payload to emit part API availability metadata once in an `apiAttributeCatalog`, with parts referencing compact `apiAttributeKey` values that Overlay Forge expands during import.

## 0.2.0 - 2026-06-15

Minor version release for the GearBlocks construction runtime API interface inclusion.

### 2026-06-15

#### Changed

- Bumped Overlay Forge from `0.1.0` to `0.2.0` for the new GearBlocks construction runtime API interface feature.
- Marked the GearBlocks parts catalog as complete and validated for game version `0.8.96622`.
- Added GearBlocks catalog version/status metadata to the Parts view.
- Hid the GearBlocks category image import and clear controls from the normal Parts view while keeping the maintenance code path available for future game-version catalog refreshes.
- Hid the GearBlocks Player.log parts import button from the normal Parts view while keeping the import functionality available elsewhere for maintenance.
- Added a blank GearBlocks Constructions top-level view as the future catalog surface for in-game constructions.
- Added a persistent GearBlocks construction index backed by `game_constructions`, populated from `SavedConstructions` by decoding each `construction.bytes` file.
- Changed the GearBlocks Constructions view to list indexed saved constructions with part, composite, and file-size summaries.
- Fully implemented support structures and exporter wiring for the documented `SmashHammer.GearBlocks.Construction` namespace reference interfaces.
- Added persisted GearBlocks runtime construction exports backed by SQLite so the full latest `Player.log` export is available to chat context without relying only on ad hoc log parsing.
- Added explicit GearBlocks runtime log import for refreshing runtime exports and runtime parts after the user runs `Export Target` or `Export All` in GearBlocks.
- Expanded the GearBlocks Lua exporter to emit availability-only `apiAttributes` metadata for documented construction interfaces.
- Changed the GearBlocks part detail view to list available runtime API attributes by name while keeping API metadata out of default chat prompt context.

#### Documentation

- Updated README, bridge, project plan, architecture, and data model notes for the Overlay Forge `0.2.0` GearBlocks runtime API interface release and future troubleshooting path.

#### Validation

- Validated the GearBlocks runtime API interface metadata path in-game on Rob's vehicle: catalog part details show API attributes without values, and DB definitions / runtime export context include indexed interface availability metadata.

#### Fixed

- Removed GearBlocks runtime API metadata from default game-chat prompt context; API details should only be included by a future explicit user-controlled include/snapshot action.
- Changed the GearBlocks Lua exporter API metadata path so `apiAttributes` indexes interface/member availability without executing getter commands to capture values.
- Added install-time and in-session GearBlocks API availability caching so known runtime parts reuse indexed API metadata instead of re-probing interface availability on every export.
- Removed automatic GearBlocks runtime log sync from normal game selection / Parts navigation; runtime logs now import only through explicit user actions such as `Import Runtime Log`.
- Resynced game chat messages after failed sends so persisted user prompts remain accurately reflected instead of disappearing from the local UI state.

### 2026-06-14

#### Fixed

- Fixed simple Gaming chat overlay screenshot context feedback so the overlay always shows whether the current prompt has screenshots attached after captures or cleared context.
- Changed Gaming chat screenshot shortcut requests to use a monotonic nonce instead of `Date.now()` so rapid repeated capture requests cannot be missed.
- Fixed overlay shortcut flicker by making React the single owner of show/hide decisions after hotkeys report the pre-shortcut window visibility.
- Changed `Ctrl+Shift+C` into a contextual Gaming chat focus key: it opens the game chat list when no chat is selected, focuses the selected simple chat prompt from game context, and returns focus to the remembered game window from chat context without hiding the chat overlay.

#### Validation

- Validated with `npm run build`, `cargo check`, and `git diff --check`.

### 2026-06-13

#### Added

- Added `Ctrl+Shift+C` as a global shortcut to open or refocus the simplified Gaming chat overlay for the currently selected existing game chat.
- Added a Settings keybind editor with configurable key1/key2/key3 shortcut cells for global app functions.
- Added SQLite-backed keybind persistence and live global shortcut re-registration.
- Added a simple Gaming chat overlay capture button that reuses the working game screenshot capture flow and automatically attaches the saved screenshot to the current prompt.
- Added `Capture Screenshot For Gaming Chat` as a configurable keybind function in Settings.
- Added mouse button support for configurable keybinds, including `Mouse4`, `Mouse5`, and modifier combinations such as `Ctrl+Mouse4`.
- Added Windows native opacity control for the simple Gaming chat overlay.
- Added GearBlocks Home controls for setting game-scoped Save Location and Alternate Data Location folders through a native directory picker.
- Added persisted `game_data_locations` records for game-scoped local data folders.
- Added a GearBlocks Construction Decoder on the selected-game Home screen for local `construction.bytes` files.
- Added raw DEFLATE + BSON decoding for GearBlocks construction saves, including composite, part, asset GUID, attachment, link, and decoded JSON summaries.
- Added a GearBlocks Lua construction exporter installer that writes an Overlay Forge script mod for runtime construction metadata export.
- Added the `OverlayForgeConstructionExporter` script mod template with targeted-construction and all-loaded-constructions JSON export actions.
- Added automatic GearBlocks chat context generation from the latest runtime construction export, including structural aggregation and functional-system purpose inference.

#### Changed

- Made the left navigation pane scroll independently with a compact slider-only scrollbar so smaller overlay window sizes can still access all navigation items.
- Changed the Settings panel to use the same compact slider-only scrollbar styling as the left navigation pane.
- Changed keybind editing so `key1`, `key2`, and `key3` represent the ordered parts of one shortcut, such as `Ctrl`, `Shift`, `Space`.
- Changed Settings keybind capture so mouse clicks can be assigned from the same keybind prompt.
- Changed Gaming chat Enter-submit behavior from a trailing backslash to two trailing spaces before pressing Enter.
- Changed the simple Gaming chat overlay to use a smoky translucent treatment inspired by GearBlocks' in-game control panels.
- Replaced the transparent-window experiment with simple-chat native window opacity so WebView2's black backing still renders translucently over the game.
- Reapplied simple-chat native opacity before showing the hidden chat overlay so `Ctrl+Shift+C` reopen does not return as opaque black.
- Removed the titlebar minimize button so Overlay Forge is not minimized to the taskbar from the app controls.
- Removed the simple chat overlay move and resize buttons; the left rail now drags the overlay, and resizing remains handled by window edges/corners.
- Changed overlay window dragging to use manual positioning on Windows so the main overlay and simple chat overlay can be placed tight against screen borders without triggering Windows auto-snap.
- Changed the simple chat overlay close button to hide the overlay window instead of flashing back to the full Overlay Forge shell.
- Changed `Ctrl+Shift+C` so it opens the current game's Chats page when no game chat is currently selected.
- Made Gaming chat overlay shortcut requests durable so `Ctrl+Shift+C` still routes correctly when Overlay Forge is hidden or waking from focus.

#### Fixed

- Removed the simple Gaming chat overlay selection step; chat capture now takes a full game screenshot for fast repeated mouse-bound captures.
- Fixed Gaming chat screenshot shortcuts so they no longer focus Overlay Forge before capture, preserving the foreground game target.
- Separated simple chat overlay shortcut behavior from the main Overlay Forge toggle: `Ctrl+Shift+C` toggles the simple chat overlay, while `Ctrl+Shift+Space` restores the main shell.
- Fixed shortcut state handling so `Ctrl+Shift+C` sees the current simple chat overlay state and `Ctrl+Shift+Space` is not double-triggered by key repeat.
- Fixed `Ctrl+Shift+C` reopening a hidden simple chat overlay and immediately hiding it again by preserving the window visibility state from before the shortcut was handled.
- Fixed `Ctrl+Shift+Space` main-shell toggling so React decides whether to hide the main shell or switch out of simple chat mode using the window visibility state from before the shortcut was handled.
- Added a compact screenshot attachment indicator inside the simple Gaming chat overlay prompt area.
- Fixed GearBlocks Lua exporter installation so script mods always install under GearBlocks' standard `AppData\LocalLow` `ScriptMods` folder instead of deriving the script path from configured data locations.
- Fixed the GearBlocks Lua exporter path for GearBlocks' blocked `io.open` sandbox by falling back to marked `Player.log` JSON chunks and adding an Overlay Forge runtime-log importer.

#### Documentation

- Documented the Gaming chat overlay shortcut in README and bridge context.
- Documented Settings as the place to configure Overlay Forge keybinds.
- Documented game data-location persistence in the data model and architecture notes.
- Documented the GearBlocks construction decoder, local save format discovery, and runtime API boundary.
- Documented the GearBlocks Lua exporter install path, export directory behavior, and in-game validation boundary.
- Documented `%USERPROFILE%\AppData\LocalLow\SmashHammer Games\GearBlocks\` as GearBlocks' default user data location and the source for default subpaths.
- Documented the GearBlocks runtime construction understanding model used for chat context.

#### Validation

- Marked the 2026-06-13 Overlay Forge, Gaming chat overlay, screenshot capture, keybind, and window behavior changes as implemented, successful, and validated.
- Validated with `npm run build`, `cargo build`, and runtime confirmation that simple chat translucency renders correctly and screenshots transmit correctly.
- Validated the GearBlocks data-location implementation with `npm run build` and `cargo build`.
- Validated the GearBlocks construction decoder implementation with `npm run build` and `cargo build`.
- Validated the GearBlocks Lua exporter installer and command wiring with `npm run build` and `cargo check`; `cargo build` was blocked by Windows denying replacement of the locked `target\debug\overlay-forge.exe`.
- Validated the GearBlocks runtime-log importer with `npm run build`, `cargo check`, and `git diff --check`.
- Validated the GearBlocks construction understanding context with `npm run build`, `cargo check`, and `git diff --check`.

### 2026-06-06

#### Documentation

- Added project rules requiring future changelog entries to be grouped by change date.
- Added `docs/PROJECT_RULES.md` to document the daily changelog tracking convention.
- Added `docs/GAMING_SCREENSHOT_VALIDATION.md` to document Gaming Screenshot Capture as complete, passed, and successful.
- Added `docs/GEARBLOCKS_PARTS_CATALOG.md` as a shareable ChatGPT reference for GearBlocks categories and cataloged parts.
- Updated README, project plan, architecture, data model, and bridge docs with the validated screenshot capture status.

#### Added

- Added a top-level Gaming section with add/remove controls for game workspace sections.
- Added the initial GearBlocks game section.
- Added expandable Gaming child rows in the left navigation for game sections.
- Added persisted SQLite tables for games, game catalog objects, game catalog references, and game screenshot file-path metadata.
- Added Tauri commands and frontend services for listing, creating, and deleting persisted games.
- Added selected-game toolbar buttons for screenshot capture, object creation, and reference creation.
- Documented the internal game-engine PNG screenshot workflow for GearBlocks-compatible captures.
- Wired the Capture Screenshot button to create a capture manifest JSON and unique PNG path under gitignored `game-screenshots/`.
- Changed Capture Screenshot to test Windows GDI foreground-window capture while hiding Overlay Forge before saving the PNG.
- Added a scrollable selected-game content area with a collapsible Screenshots thumbnail section showing capture date/time.
- Replaced the screenshot success alert with a temporary floating `Successful` bubble.
- Added a right-click screenshot context menu with visual test actions and a delete action.
- Added screenshot deletion that removes the saved PNG, capture manifest JSON, and screenshot database row.
- Changed the selected-game toolbar's second action to `Parts`.
- Added a GearBlocks parts catalog import that upserts recognizable parts from the screenshot set into `game_catalog_objects`.
- Added a selected-game Parts chart showing category icon, thumbnail source, part name, and practical physics-use description.
- Doubled the Parts chart text size and widened chart rows/columns to support the larger typography.
- Replaced Parts chart category text indicators with a filter button row that uses cropped GearBlocks category icons from the source screenshots.
- Rebuilt GearBlocks Parts filters around all 21 left-panel part categories in source screenshot order, including selectable categories with no cataloged rows yet.
- Changed selected game sections to open a blank main pane without Gaming or game-title labels.
- Removed the selected-game blank pane border so no top separator line appears.

#### Fixed

- Enabled Tauri asset loading for the `game-screenshots/` folder so saved screenshots can render as in-app thumbnails.
- Filtered missing screenshot files out of the preview list so manually deleted captures do not leave broken thumbnail cards.

#### Validation

- Documented Gaming Screenshot Capture as complete, passed, and successful after validating capture, saved files, thumbnail previews, and screenshot delete cleanup.

### Milestone Status

- Milestone 0 - Overlay Shell Validation is complete, passed, and successful.
- The Milestone 0 scratchpad component is complete and passed.
- Scratchpad content saves to SQLite and restores between app sessions.
- Milestone 1 - Calendar, To-do, Notes, and Scratchpad Expansion is complete, passed, and successful.
- Current user-validated project baseline is Milestone 13.
- Milestone 2 - Local Projects component is complete, passed, and successful.
- Milestone 3 - OpenAI Planning Chat component is complete, passed, and successful.
- Milestone 4 - GitHub Integration is complete, passed, and successful.
- Milestone 5 - Controlled YouTube Component is complete, passed, and successful.
- Milestone 6 - Project Workspace Chat is complete, passed, and successful.
- Milestone 7 - Project Workspace Layout Refinement is complete, passed, and successful.
- Milestone 8 - Projects Navigation Tree Actions is complete, passed, and successful.
- Milestone 9 - Manual Context Attachments is complete, passed, and successful.
- Milestone 10 - Prompt Preview is complete, passed, and successful.
- Milestone 11 - Bridge File Drafting is complete, passed, and successful.
- Milestone 12 - Project Markdown Context is complete, passed, and successful.
- Milestone 13 - Project Workspace UI Consolidation is complete, passed, and successful.
- Milestone 13 refinement moves conversation attached context and local Markdown bridge drafts into a collapsible right-hand chat pane.
- Milestone 13 refinement changes the project row `...` Chat action to `New Chat`, which opens a new-conversation area instead of auto-selecting an existing chat.

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
- Added Planning Chat component navigation.
- Added SQLite `planning_conversations` and `planning_messages` tables.
- Added Tauri planning chat commands for conversation listing, creation, deletion, message listing, and backend message sending.
- Added a backend OpenAI Responses API integration using `OPENAI_API_KEY`.
- Added Planning Chat project selection, conversation list, new conversation action, message history, message input, loading state, and readable error display.
- Added `docs/MILESTONE_3.md` with setup validation and manual validation steps.
- Added SQLite `project_github_repositories` table initialization for project-scoped GitHub repository linkage and metadata/status.
- Added Rust/Tauri GitHub integration commands for getting, saving, deleting, and fetching project repository metadata.
- Added backend-only GitHub metadata fetch behavior using `GITHUB_TOKEN`.
- Added frontend GitHub project-link UI inside the Projects component.
- Added readable missing-token, invalid repository full-name, and GitHub request error handling.
- Added `docs/MILESTONE_4.md` with setup validation and manual validation steps.
- Added YouTube component navigation.
- Added SQLite `youtube_references` table initialization for local YouTube reference persistence.
- Added Rust/Tauri YouTube reference commands for list, get, create, update, delete, and external open.
- Added frontend YouTube reference CRUD UI with create, selected read-only view, edit, delete, list, and optional user-entered metadata.
- Added YouTube URL validation for common watch, short link, and shorts URL forms.
- Added external-open behavior for saved YouTube URLs through the system browser.
- Added `docs/MILESTONE_5.md` with setup validation and manual validation steps.
- Added Project workspace sections for Overview, GitHub, and Chat.
- Added project-scoped Chat inside the selected project workspace.
- Added a required conversation title field before creating a project workspace chat conversation.
- Added `docs/MILESTONE_6.md` with setup validation and manual validation steps.
- Added a refined selected-project workspace header in Projects.
- Added Overview, GitHub, Chat, and References workspace sections.
- Added a minimal References section with project-local context category summaries.
- Added `docs/MILESTONE_7.md` with setup validation and manual validation steps.
- Added `docs/MILESTONE_8.md` for Projects navigation tree actions.
- Added the Projects navigation tree pattern in the left navigation shell.
- Added an expandable Projects module row.
- Added saved project child rows under Projects.
- Added a compact Projects `+` creation action.
- Added compact project row `...` edit/delete actions.
- Added `docs/MILESTONE_9.md` for Manual Context Attachments.
- Added manual context attachment support for project chat conversations.
- Added SQLite `planning_conversation_context` table initialization for conversation-scoped context attachment links.
- Added backend commands for listing, adding, and removing planning conversation context attachments.
- Added a frontend Attached Context area inside project Chat.
- Added support for attaching project, GitHub repository, note, task, calendar event, YouTube reference, and scratchpad context.
- Added automatic GitHub repository context attachment for selected project chat conversations when a repository is linked in the GitHub section.
- Added Prompt Preview action in project workspace Chat.
- Added backend prompt preview command that assembles local preview data without calling OpenAI.
- Added display of selected project, selected conversation, draft message, and attached context in Prompt Preview.
- Added display of assembled prompt preview.
- Added bridge-file draft generation from selected project chat conversations.
- Added SQLite `bridge_file_drafts` table initialization for local bridge draft persistence.
- Added backend commands for bridge draft creation, listing, retrieval, and deletion.
- Added frontend `Draft Bridge File` action in project workspace Chat.
- Added read-only Bridge Drafts panel with saved draft list and generated Markdown content display.
- Added generated Markdown draft structure with project, conversation source, goal, relevant context, implementation instructions, validation checklist, deferred items, and notes.
- Added attached context inclusion for project chat sends, including linked GitHub repository metadata when available.
- Added project-level local Markdown context configuration for selected projects.
- Added SQLite `project_markdown_context` table initialization for per-project local Markdown context roots.
- Added backend commands for getting, saving, clearing, and loading project Markdown context.
- Added safe local Markdown loading from configured project roots, including `README.md`, `CHANGELOG.md`, `docs/*.md`, `bridge-files/*.md`, and explicit Markdown references found in `README.md`.
- Added readable warnings for missing, unreadable, unsafe, skipped, and truncated Markdown files.
- Added Project Markdown context display in selected-project Chat.
- Added project Markdown context inclusion for Prompt Preview, project chat sends, and bridge draft generation.
- Added `docs/MILESTONE_13.md` as the baseline plan for Project Workspace UI Consolidation.
- Added planning conversation child rows under project rows in the left navigation hierarchy.
- Added compact chat markers for conversation rows.
- Added direct conversation-row navigation into the focused project chat surface.
- Added project row `...` menu routing for Overview, Chat, References, Edit, and Delete.
- Added a focused chat surface that removes redundant Projects, Active Workspace, tab, and Planning Chat headings.
- Added Project Edit as the consolidated surface for project details, GitHub repository linkage, and project Markdown context configuration.
- Added `docs/PROJECT_DEFERRED_ITEMS.md` to centralize Project workspace deferred items.

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
- Updated project documentation for Milestone 3 validation success.
- Updated project documentation for Milestone 4 validation success.
- Updated project documentation for Milestone 5 validation success.
- Updated project documentation for Milestone 6 validation success.
- Changed Planning Chat access so project-scoped chat is reached through Projects instead of standalone navigation during the workspace migration.
- Updated project documentation for Milestone 7 validation success.
- Confirmed selected project context remains stable across Overview, GitHub, Chat, and References.
- Preserved existing GitHub repository linkage behavior inside the selected project workspace.
- Preserved existing project-scoped chat behavior inside the selected project workspace.
- Implemented the Projects navigation tree pattern with module-level `+` and item-level `...` actions.
- Confirmed selected project workspace behavior is preserved.
- Confirmed existing project-scoped chat behavior is preserved.
- Confirmed existing GitHub behavior is preserved.
- Confirmed attachment removal deletes only the attachment link and does not delete source records.
- Confirmed existing project-scoped chat behavior is preserved after adding manual context attachments.
- Confirmed linked GitHub repository metadata only needs to be defined once per project before it appears in project chat Attached Context.
- Confirmed Prompt Preview does not send to OpenAI.
- Clarified attached context is included in Prompt Preview only; actual OpenAI sends remain unchanged in Milestone 10.
- Confirmed bridge drafts persist locally in SQLite.
- Confirmed bridge draft deletion removes only the selected draft and does not delete source conversations, messages, or context records.
- Confirmed linked GitHub repository metadata is resolved from the selected project for bridge drafts, prompt previews, and project chat sends.
- Confirmed export, full editor workflows, and direct Codex handoff remain deferred beyond Milestone 11.
- Confirmed project Markdown context is project-scoped, local-first, and not stored as per-conversation manual attachments.
- Confirmed conversation manual attachments remain an additional context layer after project Markdown context.
- Confirmed GitHub file APIs, broad repository indexing, bridge export, and direct Codex handoff remain deferred beyond Milestone 12.
- Confirmed that Milestone 12 is complete, passed, and successful after the Milestone 13 UI consolidation pass.
- Clarified that Milestone 13 should preserve conversation-scoped manual attachment data semantics while moving attachment controls out of the primary chat surface.
- Added the Milestone 13 right-hand pane for conversation attached context and local Markdown bridge drafts.
- Added a `New Chat` project action that starts on an empty new-conversation area.
- Preserved left-navigation conversation child-row selection as the only path for opening existing chats directly.
- Confirmed Milestone 13 did not require schema changes.
- Confirmed conversation manual attachments remain conversation-scoped after UI consolidation.
- Synced `docs/DATA_MODEL.md` to mark the Milestone 12 data model as complete and revalidated after Milestone 13.
- Linked `README.md`, `docs/BRIDGE_FILES.md`, and `docs/PROJECT_PLAN.md` to the centralized Project deferred items doc.

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
- Verified `npm install` completes successfully after Milestone 3 implementation.
- Verified frontend build after Milestone 3 implementation with `npm run build`.
- Verified Rust backend compile after Milestone 3 implementation with `cargo build`.
- User manually validated Milestone 3 successfully.
- Verified `npm install` completes successfully after Milestone 4 implementation.
- Verified frontend build after Milestone 4 implementation with `npm run build`.
- Verified Rust backend compile after Milestone 4 implementation with `cargo build`.
- Verified development launch after Milestone 4 implementation with `npm run tauri:dev` outside the sandbox so the app could write the app-data SQLite database.
- User manually validated Milestone 4 successfully.
- Verified `npm install` completes successfully after Milestone 5 implementation.
- Verified frontend build after Milestone 5 implementation with `npm run build`.
- Verified Rust backend compile after Milestone 5 implementation with `cargo build`.
- Verified development launch after Milestone 5 implementation with `npm run tauri:dev` outside the sandbox; the app process started and was stopped after the validation timeout.
- User manually validated Milestone 5 successfully.
- Verified frontend build after Milestone 6 implementation with `npm run build`.
- Verified Rust backend compile after Milestone 6 implementation with `cargo build`.
- User manually validated Milestone 6 successfully.
- Verified frontend build after Milestone 7 implementation with `npm run build`.
- Verified `npm install` completes successfully after Milestone 7 implementation.
- Verified Rust backend compile after Milestone 7 implementation with `cargo build`.
- Verified development launch after Milestone 7 implementation with `npm run tauri:dev`; the app process started and was stopped after the validation timeout.
- User manually validated Milestone 7 successfully.
- Verified `npm install` completes successfully after Milestone 8 implementation.
- Verified frontend build after Milestone 8 implementation with `npm run build`.
- Verified Rust backend compile after Milestone 8 implementation with `cargo build`.
- Verified development launch after Milestone 8 implementation with `npm run tauri:dev`; the app process started and was stopped after the validation timeout.
- User manually validated Milestone 8 successfully.
- Verified `npm install` completes successfully after Milestone 9 implementation.
- Verified frontend build after Milestone 9 implementation with `npm run build`.
- Verified Rust backend compile after Milestone 9 implementation with `cargo build`.
- Verified development launch after Milestone 9 implementation with `npm run tauri:dev`; the app process started and was stopped after the validation timeout.
- User manually validated Milestone 9 successfully.
- Verified `npm install` completes successfully after Milestone 10 implementation.
- Verified frontend build after Milestone 10 implementation with `npm run build`.
- Verified Rust backend compile after Milestone 10 implementation with `cargo build`.
- Verified development launch after Milestone 10 implementation with `npm run tauri:dev`; the app process started and was stopped after the validation timeout.
- Manual validation for Milestone 10 is complete, passed, and successful.
- Verified `npm install` completes successfully after Milestone 11 implementation.
- Verified frontend build after Milestone 11 implementation with `npm run build`.
- Verified Rust backend compile after Milestone 11 implementation with `cargo build`.
- Verified development launch after Milestone 11 implementation with `npm run tauri:dev`; the app process started and was stopped after the validation timeout.
- Manual validation for Milestone 11 is complete, passed, and successful.
- Verified `npm install` completes successfully after Milestone 12 implementation.
- Verified frontend build after Milestone 12 implementation with `npm run build`.
- Verified Rust backend compile after Milestone 12 implementation with `cargo build`.
- Verified development launch after Milestone 12 implementation with `npm run tauri:dev`; the app process started and was stopped after the validation timeout.
- Manual validation for Milestone 12 is complete, passed, and successful.
- Verified frontend build after Milestone 13 implementation with `npm run build`.
- Verified `npm install` completes successfully after Milestone 13 implementation.
- Verified Rust backend compile after Milestone 13 implementation with `cargo build`.
- Verified development launch after Milestone 13 implementation with `npm run tauri:dev`; the app process started and was stopped after the validation timeout.
- Manual validation for Milestone 13 is complete, passed, and successful.
