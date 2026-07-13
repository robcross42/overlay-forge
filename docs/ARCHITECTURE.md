# Overlay Forge Architecture

## Application Shape

Overlay Forge is a Tauri desktop application with a React + TypeScript frontend and a Rust/Tauri backend. SQLite is the local source of truth.

The shell owns top-level layout, navigation, component hosting, window behavior, always-on-top behavior, global hotkeys, and overlay show/hide behavior.

Feature modules render inside the shell-owned component host and should not directly own top-level window behavior.

## Architecture Principles

Overlay Forge architecture should prefer reusable domain abstractions over one-off procedural patches when behavior is repeated or state crosses feature boundaries.

Repeated behavior, repeated state shape, duplicated validation, duplicated SQLite mapping, duplicated command payload shaping, and duplicated window setup are regression risks. New features and fixes should first determine whether they belong to an existing domain abstraction, such as a window manager, window config model, window state repository, module manager, settings service, SQLite repository, chat/session model, screenshot/attachment model, export service, log ingestion service, or Tauri command service layer.

If an appropriate abstraction already exists, extend it. If none exists and the concept appears in three or more places, or if two existing places have already diverged and caused a bug, create a shared abstraction before adding more call-site behavior.

The stack-specific model is:

- React + TypeScript renders UI and handles local interaction.
- TypeScript classes may own repeated state plus behavior for domain objects, validation, normalization, serialization, deserialization, command payload creation, and UI view-model mapping.
- TypeScript interfaces and type aliases should remain the default for plain DTOs.
- Rust uses structs, impl blocks, enums, traits, services, repositories, and modules rather than Java-style inheritance.
- Tauri command handlers stay thin and delegate business logic to services, repositories, or domain methods.
- SQLite access and row mapping should be centralized by persisted domain concept.
- Long command, service, and repository argument lists should be converted into typed request, draft, options, or parameter structs when the values describe one domain operation.
- Repeated frontend helpers such as unknown-error formatting, timestamp labels, local storage keys, Markdown cleanup, and command payload normalization should live in shared utilities or domain helpers.
- Large source files should be split by domain boundary before they become dumping grounds for unrelated behavior.

For non-trivial code changes, implementation notes should identify the domain concept involved, the reusable abstraction added or reused, duplicate logic removed, regression risk reduced, and tests added or updated. If no abstraction is needed, the implementation note should explain why.

## Frontend Structure

The frontend is organized around feature folders:

```text
src/
├─ app/
├─ components/
├─ features/
├─ services/
├─ styles/
└─ main.tsx
```

Feature modules should keep UI state local where possible and call backend-owned Tauri commands for persistence, filesystem access, secret-backed requests, game capture, runtime import, and other privileged operations.

React components should remain function components with hooks unless a specific task documents why a class component is needed. Components should not own backend business rules, persistence rules, or Tauri window lifecycle behavior.

Shared frontend utility behavior belongs under focused utility or service modules rather than being redefined inside feature components.

## UI Consistency Rules

Organizer-style components should follow the same interaction pattern unless a task explicitly documents a reason to diverge:

- Empty components show the primary New action and keep editor fields hidden.
- New actions reveal the editor for the first item.
- Selecting an existing list item opens it in selected/read-only mode.
- Selected existing items expose an explicit Edit action before fields become editable.
- Destructive actions are available only inside edit or selected-item context.
- Active clickable actions use consistent enabled button styling.
- Module-level `+` actions create new items for that module.
- Item-level `...` actions open item menus such as Edit and Delete.
- Hover-revealed actions must also be visible or reachable by keyboard focus.
- Workspace surfaces should prioritize selected item content while navigation owns object-level actions.

## Backend Responsibilities

The Tauri backend owns:

- SQLite initialization and migrations.
- Scratchpad persistence commands.
- Task CRUD commands.
- Note CRUD commands.
- Calendar event CRUD commands.
- Backend-only OpenAI Responses API request handling.
- YouTube reference CRUD commands.
- YouTube URL validation and external-open handling.
- Gaming and screenshot capture commands.
- GearBlocks saved construction decoding and runtime import.
- GearBlocks API catalog and runtime API availability indexing.
- Game build guide import, persistence, and overlay-window commands.
- Smoking Cessation event and settings commands.
- Repair Resell source registry, manual import, conservative public refresh, listing persistence, keyword/category rules, watchlist, and manual deal estimates.
- Scheduler commands and backend worker dispatch.
- Global hotkey registration.
- Window show/hide behavior.

The former Projects module command surface has been removed. Legacy project, planning conversation, bridge draft, project Markdown, and project GitHub SQLite tables remain in the database layer only to preserve old local data.

Backend command handlers should receive input, validate it, call a service, repository, or domain method, and return typed results. They should not manually construct complex domain objects inline, duplicate defaults, duplicate SQLite access logic, or own large procedural feature implementations.

Broad backend cleanup should include `cargo clippy --all-targets` when practical. Clear no-risk Clippy warnings should be fixed immediately. Larger warnings, such as high-arity persistence methods that need typed parameter structs, should be captured as explicit refactor work rather than suppressed silently.

## Persistence Boundary

SQLite is the local source of truth. Migrations must be idempotent and non-destructive.

New persistence should follow the current naming convention:

| Prefix | Meaning |
| --- | --- |
| `obj_` | Dynamic object rows. |
| `def_` | Static definition rows. |
| `o2o_` | One-to-one mapping rows. |
| `n2n_` | Many-to-many mapping rows. |

Avoid table-per-game settings. Use `obj_game_setting` or a normalized feature table keyed by `game_id` and `id_game`.

Do not scatter SQL row mapping across unrelated files. Each persisted domain concept should have one canonical mapping path between database rows, domain objects, database insert/update payloads, and frontend DTOs where needed.

SQLite access should surface recoverable infrastructure failures through `Result` values. Repository methods should not panic on normal database lock or mapping failures.

Generated screenshots, manifests, runtime game data, copied DLLs, plugin build output, and local workspaces should remain ignored by git.

## Window Boundary

Overlay Forge treats windows as a first-class domain concept.

All Tauri window creation, configuration, restoration, state persistence, and lifecycle behavior should route through centralized Rust window abstractions. Standalone overlays, the main overlay, and future independent windows should not copy setup logic across commands, React components, utility files, or one-off helpers.

The expected Rust model uses composition:

```text
BaseWindowConfig
  common settings shared by all windows

OverlayWindowConfig
  main overlay-specific configuration

StandaloneWindowConfig
  standalone-specific configuration

WindowKind
  enum describing allowed window types

WindowManager
  centralized creation, open, close, focus, restore, and mutation behavior

WindowStateRepository
  SQLite-backed persistence for size, position, visibility, and related state
```

The primary `Main` overlay is transient: it is not always-on-top and hides when it loses focus. Standalone `GameChat` and `GameBuildGuide` windows remain always-on-top and must not inherit the main window's focus-loss behavior.

`WindowKind` should be an enum, not scattered string labels. `StandaloneWindowConfig` and `OverlayWindowConfig` should compose `BaseWindowConfig` rather than model Java-style inheritance.

Before changing window behavior, inspect all existing creation paths. If more than one file constructs windows, sets default options, generates labels, restores geometry, or handles standalone-window configuration, consolidate the shared path before applying the feature or fix.

## OpenAI Boundary

Game chat and build-guide features call the OpenAI Responses API from the Rust/Tauri backend. React invokes local Tauri commands only and never reads `OPENAI_API_KEY`.

Model selection, request shape, and assistant instructions should stay centralized in backend OpenAI service code so frontend components do not leak API details or secrets.

## Codex Boundary

Codex implementation happens directly in VS Code against the repository.

Overlay Forge repository docs should provide project context, constraints, and validation expectations. The app should not automate VS Code/Codex implementation workflows unless the user explicitly scopes a future feature for that.

## Retired Projects Data Boundary

The former Projects module is retired. Its frontend feature folder, planning-chat UI, project services, project-scoped GitHub service, and active Tauri command registration have been removed.

Retained legacy SQLite data includes:

- Project records.
- Planning conversations and messages.
- Conversation context attachment links.
- Bridge file drafts.
- Project Markdown context configuration.
- Project GitHub repository metadata.

Do not restore commands, UI, GitHub token usage, project chat, prompt preview, bridge drafts, or Markdown context loading without an explicit future design.

## YouTube Boundary

YouTube references are local-first and user-curated. SQLite stores title, URL, parsed video id, optional channel name, notes, tags, and timestamps.

No YouTube API key, account login, OAuth flow, watch history, subscription import, playlist sync, comment sync, transcript extraction, recommendations, downloads, scraping, background metadata crawler, or account sync is used.

Saved YouTube URLs open externally in the system browser.

## Gaming Boundary

Gaming is a workspace under the shell-owned component host. Game rows are backed by `def_game` definitions and `obj_game` local sections.

Screenshot image bytes are stored as PNG files under `game-screenshots/`. SQLite stores metadata and local paths only. Tauri asset loading is scoped to `game-screenshots/` for thumbnail previews.

Game build guides are imported from user-selected Markdown files into normalized SQLite rows. The independent build-guide overlay window is shell-owned like the game chat overlay: Rust/Tauri stores the active guide selection, applies persisted bounds, and exposes a keybind-driven show/hide path. The overlay renders narrow, stacked rows rather than wide tables so it can stay pinned to the left or right side of the screen during gameplay.

See `docs/GAMING_SCREENSHOTS.md` for capture behavior.

## GearBlocks Boundary

GearBlocks support is local-first and split across:

- Saved construction decoding.
- Runtime scene export/import.
- Runtime API availability indexing.
- Part catalog display.
- Optional in-game script window tools.
- Backlog direct BepInEx plugin marker support.

Normal chat navigation must not synchronously parse full-scene runtime logs. Runtime import should use explicit refresh/import paths, passive cursor-based log import, or bounded chat-send context assembly.

API availability metadata is excluded from default prompt context unless a future explicit include/snapshot control is added.

GearBlocks chat does not currently request marker placement or emit marker blocks; BepInEx marker work remains backlog.

See `docs/GEARBLOCKS.md`, `docs/GEARBLOCKS_RUNTIME.md`, `docs/GEARBLOCKS_PLUGIN.md`, and `docs/GEARBLOCKS_PARTS_CATALOG.md`.

## Smoking Cessation Boundary

Smoking Cessation is a local-first core feature module. The React feature reads and writes through Tauri commands. Cigarette events and settings are stored in SQLite. Frontend charts are derived from SQLite rows at render time rather than storing aggregate data.

The module may render a narrow Markdown export under app data for personal context review, but SQLite remains the source of truth.

See `docs/SMOKING_CESSATION.md`.

## Repair Resell Boundary

Repair Resell is a local-first utility module for tracking buy -> repair/refurbish -> resell candidates. React renders filters, source controls, manual import, watchlist toggles, and estimate forms. Persistence, source refresh, keyword/category rules, listing snapshots, and deal calculations are backend-owned through Tauri commands.

SQLite stores the source registry, search profiles, listings, snapshots, deterministic keyword/category mappings, watchlist entries, travel profiles, and manual estimates. The scraper layer is allowlist/source-record based, manually triggered, uses normal HTTP fetch for public sources only, and falls back to manual import when a source is disabled, private, login-gated, or unstable.

The module does not perform LLM analysis, OpenAI calls, credentialed scraping, account login, browser automation, bidding, buying, seller messaging, payment, or checkout workflows.

Future pickup planning, repair knowledge-base, inventory, parts, sales, analytics, and restoration-learning features should continue through backend services and SQLite repositories rather than moving business rules into React. See `docs/REPAIR_RESELL.md` for the module vision.

## Scheduler Boundary

The Scheduler framework is a backend-owned worker loop backed by convention-based SQLite tables.

Schedule rows are dynamic `obj_scheduler` records pointing to static `def_scheduler_type` definitions. The dispatcher is closed over known Rust handlers. Scheduler records must not execute arbitrary commands, scripts, or Lua payloads stored in SQLite.
