# Overlay Forge Feature Scope

## Global Scope Guard

Future work should preserve the completed local-first desktop overlay foundation:

- Shell-owned top-level layout and component host.
- Hotkey behavior.
- Always-on-top behavior.
- SQLite persistence.
- Calendar organizer behavior as the visible main-shell organizer surface.
- Scratchpad, Tasks, and Notes implementation/data paths are retained but currently unwired from the main UI pending organizer consolidation review.
- Former Projects module removed from the shell, frontend service layer, and active Tauri command surface.
- Backend-owned OpenAI token handling.
- User-curated YouTube references.
- Gaming workspace and screenshot metadata path.
- GearBlocks save/runtime data boundaries.
- Smoking Cessation local SQLite ownership.
- Repair Resell local listing and estimate ownership.
- Scheduler bounded Rust-handler dispatch.
- SQLite naming conventions.

## Retired Projects Module Boundary

The former Projects module is removed from active code. This includes its main-shell navigation row, feature component, project chat UI, frontend project services, frontend planning-chat services, frontend project GitHub services, and active Tauri command registration.

Retained for data preservation:

- Existing SQLite tables and migrations for legacy project, planning conversation, bridge draft, project Markdown, and project GitHub rows.
- Existing database row-mapping code needed to keep old databases readable.

Future work must explicitly decide whether to restore, replace, migrate, export, or delete those legacy records.

## OpenAI Boundary

- OpenAI calls are backend-owned.
- React invokes local Tauri commands only.
- React must not read `OPENAI_API_KEY`.
- Model selection, request shape, and assistant instructions should remain centralized in backend OpenAI service code unless a requested model-picker feature changes that boundary.
- Retired prompt preview code must not be reintroduced without an explicit replacement design.

## GitHub Boundary

- The former project-scoped GitHub integration is retired with the Projects module.
- React must not receive or store `GITHUB_TOKEN` if a future GitHub feature is reintroduced.
- Existing SQLite GitHub linkage rows are legacy preserved data only.

## YouTube Boundary

YouTube references are local-first and user-curated.

Current scope:

- Save, list, edit, delete, and open user-entered YouTube references.
- Parse and validate supported YouTube URL forms.
- Store title, URL, parsed video id, optional channel name, notes, tags, and timestamps.

Out of scope unless requested:

- Account login.
- API sync.
- Scraping.
- Transcripts.
- Recommendations.
- Downloads.
- Embedded unrestricted browsing.

## Gaming Screenshot Boundary

Screenshot capture is local-first and user-initiated.

Current validated behavior:

- Hide Overlay Forge before capture.
- Capture the visible foreground game display.
- Save unique PNGs under `game-screenshots/<game-slug>/`.
- Save capture manifests under `game-screenshots/capture-requests/`.
- Persist screenshot metadata in SQLite.
- Render thumbnails through Tauri asset loading scoped to `game-screenshots/`.
- Delete screenshot PNG, manifest, metadata row, and matching local-path reference rows.

Avoid clipboard captures, `Win+Shift+S`, Snipping Tool dependency, HDR output, wide-gamut output, and alpha-dependent image files for the long-term capture target.

## Media Library Boundary

Media Library is local-first. Current scope:

- Movie and episodic-series library entries.
- Backend-owned TMDB search and explicit metadata refresh.
- Manual movie and series entries.
- Local notes, rating, favourite, priority, tags, and Watch Next ordering.
- Movie, episode, season, series, and watched-through progress operations.
- Specials-aware automatic completion.
- Cached regional provider availability and user-owned manual links.
- Offline local search, filtering, editing, and progress.

TMDB credentials remain backend-only. Local viewing history and personal fields must not be sent to TMDB.

Current exclusions include account/service sync, automatic playback detection, partial playback, recommendations, background refresh, notifications, provider scraping, direct provider deep-link fabrication, imports/exports, and manual season/episode editing.

## GearBlocks Boundary

GearBlocks support is local-first and should use the safest available data path for the requested task.

Current data layers:

1. Saved construction decoding from `construction.bytes`.
2. Runtime scene exports reconstructed from GearBlocks script log output.
3. Prompt-time rich full-scene export requests before GearBlocks chat context assembly.
4. SQLite runtime part, property, attachment, API availability, and catalog indexes.
5. SQLite GearBlocks part render profiles for validated Unity part-preview captures and canonical rotation metadata.
6. Build-guide staging manifests and a manual phase-2 latest-export import trigger before step/image review.
7. Direct BepInEx plugin work for user-controlled Unity-side part preview rendering.
8. Backlog direct BepInEx marker work for user-controlled temporary visual markers.

GearBlocks runtime API metadata is availability-first by default. Do not invoke getter-heavy or mutating API paths unless the user explicitly asks for a snapshot/control feature.

GearBlocks chat should not request in-game marker placement or emit `overlay-forge-markers` blocks while visual marker support is paused.

Build-guide staging manifests can expand high-level guide rows into exact staged part instances for known patterns, starting with the combustion-engine starter guide. Step/image generation currently refreshes the latest exported runtime scene before showing the step view, but persisted step-to-instance matching and automated construction remain out of scope until the staged workflow has enough real exports to validate matching rules.

## Smoking Cessation Boundary

Smoking Cessation is local-first.

Current scope:

- SQLite cigarette event records.
- Current cigarette inventory count.
- `Nicoderm Step 1` marker started at `2026-06-21 15:00:00 EDT`.
- Configurable record-cigarette keybind.
- Derived frontend charts and predictions.
- Narrow Markdown export for external review.

Out of scope unless requested:

- Cloud sync.
- Health-provider integration.
- Medical advice automation.
- Sharing records externally.

## Repair Resell Boundary

Repair Resell is local-first.

Long-term direction:

- Treat the module as a restoration-funded learning path, not only a flipping surface.
- Use profit to help pay down debt, reduce mortgage pressure, and fund increasingly ambitious repair projects.
- Prefer workflows that build repair knowledge: find, diagnose, learn, repair/restore, keep/sell/part out, and record what was learned.
- Model pickup economics around the user's available Wednesday, Saturday, and Sunday pickup windows, with special value for Wednesday business-hour pickups.
- Prefer multi-item auction pickups and regional auction clusters when long trips would make single-item fuel costs unattractive.

Current scope:

- SQLite-backed source registry for surplus, auction, marketplace, and manual-import sources.
- Manual source refresh for enabled public HTTP sources using conservative backend fetch and static parsing.
- Manual listing import for sources that are private, login-gated, unstable, or inappropriate to scrape directly.
- Listing persistence, snapshots, deterministic keyword flags, rule-derived categories, watchlist status, travel profiles, and manual deal estimates.
- External listing links opened by user action.

Out of scope unless explicitly requested later:

- LLM listing analysis.
- OpenAI calls.
- Scheduled/background source imports.
- Automated alerts/notifications.
- Route, load, trailer, or return-load planning.
- Inventory, inspection, repair, parts, sales, analytics, and repair knowledge-base workflows.
- Credentialed scraping.
- Login automation.
- Browser automation that bypasses anti-bot controls.
- Scraping private/account-only content.
- Automated bidding, buying, checkout, seller messaging, or payments.
- Cloud sync, push notifications, or social posting.

## Scheduler Boundary

Scheduler jobs must be backend-owned and bounded.

Rules:

- Scheduler rows point to static scheduler type definitions.
- Static scheduler keys map to known Rust handlers.
- SQLite scheduler rows must not execute arbitrary commands, scripts, Lua payloads, or shell commands.
- Jobs should record run status in `obj_scheduler_run`.
- Jobs should avoid blocking the UI or long-running synchronous work.

## Path Of Exile 2 Module Scope

Path of Exile 2 should build toward local build planning without requiring Path of Building to run beside Overlay Forge.

Current in-scope foundation:

- Store editable local character build records in `obj_game_character_build`.
- Keep build records generic and keyed by `game_id` / `id_game` so future games can reuse the same build-planning path.
- Treat passive tree snapshots, item indexes, skill gems, support gems, loot filters, trade lookups, and calculated values as separate capability layers attached to build records.

Current out-of-scope boundaries:

- Do not copy or depend on Path of Building runtime code.
- Do not create POE2-only settings tables for build data.
- Do not claim full POE2 calculation parity until passive tree, item, gem, ailment, damage, defense, and condition modeling are represented and validated.

## The Spell Brigade Module Scope

The Spell Brigade is a local-first Gaming module scaffold.

Current in-scope foundation:

- Seed a stable `def_game` definition and matching local `obj_game` section.
- Reuse shared Gaming chat and screenshot workflows.
- Provide focused Wizards, Spells, Upgrades, Synergies, and Runs planning sections.

Out of scope unless explicitly requested:

- Persisted wizard, spell, upgrade, synergy, or run-history domain records.
- Automated build or tier-list recommendations.
- Save-file parsing, process inspection, telemetry, or live game integration.
- Automatic imports from community wikis or other third-party sources.

## Persistence Boundary

SQLite is the source of truth for persisted app data.

Rules:

- Use non-destructive, idempotent migrations.
- Preserve existing user data.
- Use current naming conventions: `obj_`, `def_`, `o2o_`, `n2n_`.
- Avoid table-per-game setting tables. Prefer `obj_game_setting` or normalized feature tables keyed by `game_id` and `id_game`.
- Generated screenshots and local image files are stored outside SQLite; SQLite stores metadata and paths.
