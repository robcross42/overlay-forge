# Changelog

All notable changes to Overlay Forge are documented in this file.

Changelog entries keep local date and time stamps so a day's work can be reviewed quickly.

## Changelog Policy

Overlay Forge uses semantic versioning in `MAJOR.MINOR.PATCH` form. Do not increment the minor version just because a new chat, work session, or calendar day starts. Use `## Unreleased` for active work until a meaningful version is intentionally cut.

Version rules:

- `MAJOR`: incompatible or breaking release changes.
- `MINOR`: substantial new user-visible capabilities.
- `PATCH`: bug fixes, documentation-only changes, validation updates, small UX refinements, and internal refactors.

During `0.x` development, minor versions may still contain breaking early-development changes. Patch versions should remain non-breaking fixes, documentation, and small refinements.

Use local Toronto time for timestamped entries unless otherwise requested.

## Unreleased

## 0.11.0 - 2026-07-22

### 2026-07-20

#### Added

- 19:39:50 EDT - Added a dedicated local-first Books section to Media Library with work/edition modeling, manual books and editions, preferred editions, ownership, Read Next, content-aware reading statuses, page/percent/minute/chapter progress, series overrides, and safe user/provider links.
- 19:39:50 EDT - Added backend-owned Google Books primary search, Open Library fallback/enrichment with documented request identification and throttling, and optional read-only Hardcover exact-ISBN enrichment with partial-provider failure isolation.
- 19:39:50 EDT - Added normalized book work, edition, source identity, author, reader state, link, and series SQLite tables plus fixture-backed provider, ISBN, matching, progress, queue, URL, migration, refresh-preservation, and provider-failure tests.

#### Changed

- 19:39:50 EDT - Changed `obj_media_title.content_type` to accept `BOOK` through an idempotent transaction-safe table rebuild that preserves IDs, existing TMDB identities, indexes, and foreign keys.
- 19:39:50 EDT - Changed all project version metadata from `0.10.0` to `0.11.0` for the Books capability release.

#### Documentation

- 17:34:04 EDT - Deferred backend-owned YouTube full-movie candidate discovery for Media Library titles, including API-only search, official-source preference, completeness/authorization caution, validated direct links, stale-result handling, and *Ghosts of Mars* as an initial user-reported manual validation case.
- 18:29:32 EDT - Deferred a music-specific library for artists, releases, tracks, YouTube music-video/live-performance references, authorized direct MP3 acquisition, and local collection indexing, while explicitly excluding YouTube audio extraction, DRM bypass, and unauthorized downloading or rehosting.
- 19:39:50 EDT - Documented Books architecture, provider/security boundaries, exact matching, progress rules, SQLite ownership, validation matrix, environment configuration, and deferred integrations.

#### Validation

- 20:32:05 EDT - Validated 0.11.0 with `npm run check`, all 43 Rust tests, `cargo clippy --all-targets`, touched-Rust formatting, `git diff --check`, and a logged Tauri desktop startup that completed the live database migration; provider behavior is fixture-tested because Google Books, Hardcover, and Open Library identification environment values were not configured in the validation shell.

### 2026-07-22

#### Validation

- 03:51:01 EDT - User confirmed the full 0.11.0 validation and manual acceptance work complete for release.
- 03:54:18 EDT - Revalidated the release with `npm.cmd run check`, all 43 Rust tests, `npm.cmd run cargo:clippy`, touched-Rust `rustfmt --check`, and `git diff --check`; Clippy reports only the documented pre-existing Repair Resell high-arity warning, while repository-wide formatting remains deferred for untouched legacy Rust files.

## 0.10.0 - 2026-07-20

### 2026-07-20

#### Added

- 14:26:25 EDT - Added the local-first Media Library module with backend-owned TMDB catalogue search, manual entries, movie and episodic progress, Watch Next ordering, tags, Canadian provider availability, manual streaming links, settings, and required attribution.
- 14:26:25 EDT - Added normalized SQLite media title, library entry, season, episode, progress, provider snapshot, availability, streaming link, tag, mapping, and settings tables with transactional refresh preservation.

#### Changed

- 17:08:45 EDT - Changed all project version metadata to `0.10.0` for the Media Library and accumulated post-0.9 capability release cut.

#### Fixed

- 16:38:47 EDT - Fixed Media Library navigation buttons rendering with native browser styles by replacing the retired workspace-tab dependency with shared module-tab styling also used by Repair Resell.

#### Documentation

- 14:26:25 EDT - Added the Media Library guide and updated project architecture, data model, feature scope, deferred scope, validation, README capability, and backend credential documentation.
- 16:12:36 EDT - Deferred backend-owned movie and episode streaming deep links, including TMDB-to-provider mapping, Canadian SQLite caching, clickable provider icons, manual-link fallback, and the no-fabrication/no-scraping boundary.

#### Validation

- 14:30:40 EDT - Validated Media Library with `npm run check`, full Rust tests (13 passed), media-specific Rust tests (8 passed), `cargo clippy --all-targets --no-deps`, Rust formatting, and `git diff --check`; Clippy reports only the 15 documented pre-existing high-arity warnings.
- 15:37:13 EDT - Validated the Media Library merge with current `develop` using `npm run build`, `cargo build`, full Rust tests (16 passed), media-only `rustfmt --check`, `cargo clippy --all-targets`, and `git diff --check`; Clippy reports one pre-existing Repair Resell high-arity warning, while repository-wide `cargo fmt --check` remains blocked by pre-existing formatting drift in the merged upstream changes.
- 16:38:47 EDT - Validated shared Media Library and Repair Resell module-tab styling with `npm run build` and `git diff --check`.
- 16:43:37 EDT - User completed the full Media Library manual acceptance checklist successfully after confirming the module-tab styling fix; the native-styled navigation buttons were the only discovered defect.
- 17:15:25 EDT - Validated the `0.10.0` release metadata with `npm run build`, `cargo build`, and `git diff --check`; the repository stop script released the running debug executable before the successful Rust build.

### 2026-07-11

#### Added

- 18:41:54 EDT - Added The Spell Brigade as a seeded Gaming module with shared chats and screenshots plus Wizards, Spells, Upgrades, Synergies, and Runs planning scaffolds.
- 18:51:50 EDT - Added a game-context picker as the first Gaming workspace toolbar control so games can be switched without expanding the left navigation tree.

#### Changed

- 18:41:54 EDT - Centralized supported game-module section metadata so Path of Exile 2 and The Spell Brigade use the same module navigation and home-scaffold abstraction.
- 19:00:45 EDT - Changed the primary Overlay Forge window to be non-topmost and hide on focus loss while preserving always-on-top behavior for standalone game chat and build-guide windows.

#### Documentation

- 18:41:54 EDT - Documented The Spell Brigade module scope, current scaffold, persistence definition, and deferred game-specific data integrations.
- 18:51:50 EDT - Documented the toolbar game picker as the primary direct path into The Spell Brigade.
- 19:00:45 EDT - Documented the primary-window focus-loss boundary and its manual regression check.

#### Validation

- 18:41:54 EDT - Validated The Spell Brigade module with `npm run build`, `npm run cargo:build`, `npm run cargo:test`, `npm run cargo:clippy`, and `git diff --check`.
- 18:51:50 EDT - Validated the Gaming toolbar game-context picker with `npm run build` and `git diff --check`.
- 19:00:45 EDT - Validated the primary-window focus behavior with `npm run cargo:build`, `npm run cargo:test`, `npm run build`, Windows HWND visibility/topmost inspection, and `git diff --check`; the main window was non-topmost and hid after focus moved, while standalone window policies remained topmost in runtime inspection and unit tests.

### 2026-07-02

#### Added

- 04:18:02 EDT - Added the Repair Resell module with SQLite-backed sources, search profiles, listing imports, snapshots, deterministic keyword/category flags, watchlist toggles, conservative manual source refresh, and manual deal estimates.

#### Changed

- 05:04:12 EDT - Collapsed the Repair Resell tab to a button-only UI shell while leaving the underlying data model and stored records untouched.
- 05:05:49 EDT - Changed the main overlay shell so the left navigation pane starts collapsed by default on app launch.
- 05:09:31 EDT - Unwired Scratchpad, Tasks, and Notes from the main shell navigation/render path while keeping their existing code and data paths intact.
- 05:13:33 EDT - Removed the former Projects module from the active shell, frontend feature/service layer, project chat UI, project-scoped GitHub helper, and registered Tauri command surface while preserving legacy SQLite data.

#### Documentation

- 04:18:02 EDT - Documented the Repair Resell local-first boundary, scraper safety limits, SQLite table family, and deferred LLM/inventory/repair/sales work.
- 04:34:15 EDT - Documented the Repair Resell future vision around restoration-funded learning, multi-item auction pickups, pickup economics, parts harvesting, trailer-enabled hauling, return-load opportunities, and repair knowledge-base history.
- 04:44:22 EDT - Archived the legacy numbered progress model into project history, removed it from active documentation surfaces, and replaced app status wording with neutral current-state language.
- 05:09:31 EDT - Marked Scratchpad, Tasks, Notes, and Calendar for later organizer consolidation review.
- 05:13:33 EDT - Documented the retired Projects module boundary and added future review items for restore, replacement, migration, export, or deletion decisions.

### 2026-06-29

#### Changed

- 00:59:10 EDT - Changed generated build-guide diagram backgrounds to use barely visible gray transparency.
- 13:53:41 EDT - Removed the standalone build-guide overlay titlebar so guide content uses the full window height.
- 13:57:19 EDT - Changed the standalone build-guide overlay so empty guide space, including the toolbar gap left of the zoom buttons, can drag the window.
- 14:04:33 EDT - Added explicit GearBlocks X/Y/Z coordinate rules and visible X, Y, and Z axis markers to generated build-guide diagrams.
- 14:06:56 EDT - Defined the standard GearBlocks car orientation as length on the Z-axis, front toward +Z, rear toward -Z, width on X, and height on Y.
- 14:19:04 EDT - Added a build-guide overlay action that saves a Step 1 review HTML snapshot with the rendered diagram, caption text, step body, placement cues, and related parts.
- 14:21:35 EDT - Added visible build-guide toolbar feedback for Step 1 review snapshot export success and failure states.
- 14:26:11 EDT - Changed Step 1 review HTML snapshots to override captured app overflow styles so exported review files can scroll.
- 14:38:07 EDT - Moved build-guide diagram axis markers to the far-left grid corner and reduced them to one displayed direction per axis.
- 14:39:31 EDT - Added a Codex workflow rule to reopen edited local dev/review HTML files in the default browser after changes.
- 14:56:24 EDT - Changed build-guide Step 1 visuals to promote a named structural anchor such as `Beam x3` before other parts and number each rendered part label by placement order.
- 15:00:35 EDT - Limited GearBlocks build-guide imports to the first three parsed assembly steps for the current Step 1 review loop while preserving full raw Markdown.
- 15:08:01 EDT - Changed GearBlocks build-guide import buttons to show animated `Importing.`, `Importing..`, and `Importing...` labels on the active import action.
- 15:09:39 EDT - Changed the build-guide review export action to save a 3-step HTML review containing diagrams, captions, bodies, placement cues, and related parts for each imported step.
- 15:24:34 EDT - Changed GearBlocks build-guide display text to strip redundant size-class parentheticals from visual labels, captions, placement cues, and 3-step review snapshots while preserving functional descriptors and tooth counts.
- 16:02:07 EDT - Added a direct GearBlocks BepInEx Unity part-preview renderer command that captures the camera-center hit object into an isolated PNG under `OverlayForgePlugin\renders`.
- 16:39:30 EDT - Changed the configured Mouse5 preview-loop shortcut to write incrementing GearBlocks BepInEx `capture_center_part_preview` test commands such as `test-part-preview-1` instead of triggering screenshot capture.
- 16:43:38 EDT - Changed GearBlocks BepInEx part-preview captures to add dark crease and boundary edges so white or low-contrast part faces remain readable.
- 16:59:59 EDT - Changed GearBlocks BepInEx part-preview edge rendering to use thinner adaptive edge lines and a stricter crease threshold.
- 17:08:22 EDT - Added explicit GearBlocks BepInEx part-preview object rotation fields for X, Y, and Z degree sweeps while keeping camera yaw and pitch separate.
- 17:30:43 EDT - Added persisted GearBlocks part render profiles, rotation snap-angle constants, Tauri commands for saving profiles from BepInEx preview status captures, and frontend quaternion transform helpers for build-guide rotation composition.
- 18:44:12 EDT - Removed the temporary GearBlocks build-guide three-step import cap and added a build-guide overlay phase-2 workflow with a full staging manifest, latest-export import trigger, runtime instance count, and full build-review snapshot export.
- 19:03:47 EDT - Added a GearBlocks build-guide manifest expansion helper that turns high-level combustion-engine guide rows into exact staged part instances with paint slots, relative placement hints, rotation hints, and size/config notes.
- 19:23:50 EDT - Changed GearBlocks build-guide staging manifests so paint slots are assigned only to duplicated paintable parts; unique paintable parts now show no paint needed.
- 19:31:41 EDT - Changed GearBlocks build-guide staging manifests to list duplicated paintable parts first by duplicate count, then solo paintable parts, duplicated unpaintable parts, and solo unpaintable parts.
- 19:43:42 EDT - Changed the GearBlocks build-guide overlay to start without stale Latest Export data, add an in-guide Export All action, remove compact summary/status pills, remove scale and geometry panels, and move Glossary to the bottom.
- 20:03:39 EDT - Changed GearBlocks build-guide staging manifests so paint slot numbers restart at slot 1 for each duplicated part type.
- 20:17:17 EDT - Changed GearBlocks build-guide staging manifests to split slash-separated part names and include Corner 90 Pipe in the combustion-engine intake-manifold expansion.
- 20:52:29 EDT - Changed GearBlocks build-guide staging instructions to allow temporary white jig blocks for initial attachment while excluding them from final build design.
- 21:27:55 EDT - Changed GearBlocks combustion-engine staging manifest `Beam x3` uprights to use `90,0,0` rotation instead of `0,0,90`.
- 21:43:38 EDT - Changed the GearBlocks build-guide staging manifest display to show only render-placement fields: Type, Name, Paint, and Rotation.

#### Fixed

- 16:13:15 EDT - Fixed the GearBlocks BepInEx part-preview renderer to fall back from non-renderable physics composite hit roots to the nearest enabled Unity renderer at the raycast hit point.
- 16:17:53 EDT - Fixed the GearBlocks BepInEx part-preview renderer fallback to always use the nearest enabled Unity renderer when collider hierarchy lookup fails, with selection diagnostics in the status JSON.
- 16:23:32 EDT - Fixed the GearBlocks BepInEx part-preview renderer discovery to scan all loaded Unity renderers and remove the strict MeshRenderer/SkinnedMeshRenderer filter that excluded GearBlocks visual meshes.
- 16:31:54 EDT - Changed GearBlocks BepInEx part-preview captures to render only the isolated part/object on a neutral background, leaving grid and axis composition to build-step images.
- 16:59:59 EDT - Fixed GearBlocks BepInEx part-preview fallback selection so huge environment renderers such as the boundary indicator are ignored, crank sub-renderer hits can promote to a reasonable parent part group, and non-readable Unity meshes skip edge extraction without mesh-access errors.
- 21:33:58 EDT - Fixed the GearBlocks build-guide overlay so unfocused-window translucency applies to gray backgrounds instead of reducing guide text readability.

#### Documentation

- 19:43:42 EDT - Documented the GearBlocks build-guide Export All workflow and the simplified Build Info view.
- 20:10:03 EDT - Documented the 32 default GearBlocks paint palette slots for build-guide staging references.
- 20:17:17 EDT - Documented that combustion-engine staging manifests can use parts from multiple GearBlocks categories, including Pipes.
- 20:52:29 EDT - Documented that temporary white jig blocks are placement aids only and must not be counted as build-guide parts.
- 21:27:55 EDT - Documented observed GearBlocks `Beam x3` rotation behavior for build-guide staging.
- 21:43:38 EDT - Documented that the build-guide staging list is for render capture and shows only part type, instance name, paint slot, and rotation.

#### Validation

- 00:59:10 EDT - Validated the generated diagram background transparency update with `npm run build` and `git diff --check`.
- 13:53:41 EDT - Validated the build-guide overlay titlebar removal with `npm run build` and `git diff --check`.
- 13:57:19 EDT - Validated the build-guide empty-space drag behavior with `npm run build` and `git diff --check`.
- 14:04:33 EDT - Validated GearBlocks coordinate-axis rules and build-guide diagram axis markers with `cargo fmt --manifest-path src-tauri\Cargo.toml`, `npm run build`, `npm run cargo:build`, and `git diff --check`.
- 14:06:56 EDT - Validated the GearBlocks standard car-orientation rule with `npm run cargo:build` and `git diff --check`.
- 14:19:04 EDT - Validated the Step 1 review snapshot export with `npm run build` and `git diff --check`.
- 14:21:35 EDT - Validated Step 1 review snapshot toolbar feedback with `npm run build` and `git diff --check`.
- 14:26:11 EDT - Validated scrollable Step 1 review snapshot exports with `npm run build` and `git diff --check`.
- 14:38:07 EDT - Validated far-corner single-direction build-guide axis markers with `npm run build` and `git diff --check`.
- 14:39:31 EDT - Validated the local dev HTML browser-refresh workflow rule with `git diff --check`.
- 14:56:24 EDT - Validated Step 1 structural-anchor promotion and numbered part labels with `npm run build` and `git diff --check`.
- 15:00:35 EDT - Validated the GearBlocks three-step import cap with `npm run cargo:test`, `npm run cargo:build`, `npm run build`, and `git diff --check`.
- 15:08:01 EDT - Validated GearBlocks build-guide import button progress labels with `npm run build` and `git diff --check`.
- 15:09:39 EDT - Validated the 3-step build-guide review HTML export with `npm run build` and `git diff --check`.
- 15:24:34 EDT - Validated GearBlocks build-guide size-parenthetical display cleanup with `npm run build`, a cleaned local 3-step review HTML text check, and `git diff --check`.
- 16:02:07 EDT - Validated the direct GearBlocks BepInEx part-preview renderer with `dotnet build gearblocks-bepinex-workspace\OverlayForgeGearBlocksPlugin\OverlayForgeGearBlocksPlugin.csproj` and `git diff --check`; install/run validation was skipped because GearBlocks was running.
- 16:13:15 EDT - Validated the GearBlocks BepInEx non-renderable hit-root fallback with `dotnet build gearblocks-bepinex-workspace\OverlayForgeGearBlocksPlugin\OverlayForgeGearBlocksPlugin.csproj`.
- 16:17:53 EDT - Validated the GearBlocks BepInEx nearest-renderer preview fallback with `dotnet build gearblocks-bepinex-workspace\OverlayForgeGearBlocksPlugin\OverlayForgeGearBlocksPlugin.csproj`.
- 16:23:32 EDT - Validated the GearBlocks BepInEx all-loaded-renderers preview discovery with `dotnet build gearblocks-bepinex-workspace\OverlayForgeGearBlocksPlugin\OverlayForgeGearBlocksPlugin.csproj`.
- 16:31:54 EDT - Validated the GearBlocks BepInEx neutral-background part-only preview change with `dotnet build gearblocks-bepinex-workspace\OverlayForgeGearBlocksPlugin\OverlayForgeGearBlocksPlugin.csproj`.
- 16:39:30 EDT - Validated the Mouse5 GearBlocks part-preview command shortcut with `cargo fmt --manifest-path src-tauri\Cargo.toml`, `npm run cargo:build`, and `git diff --check`.
- 16:43:38 EDT - Validated the GearBlocks BepInEx part-preview edge overlay with `dotnet build gearblocks-bepinex-workspace\OverlayForgeGearBlocksPlugin\OverlayForgeGearBlocksPlugin.csproj`, installed the rebuilt DLL into the local GearBlocks BepInEx plugin folder while GearBlocks was closed, and ran `git diff --check`.
- 16:59:59 EDT - Validated the GearBlocks BepInEx part-preview selection and edge-safety fixes with `dotnet build gearblocks-bepinex-workspace\OverlayForgeGearBlocksPlugin\OverlayForgeGearBlocksPlugin.csproj`; install/run validation was skipped because GearBlocks process `9380` was running.
- 17:08:22 EDT - Validated GearBlocks BepInEx part-preview rotation command support with `dotnet build gearblocks-bepinex-workspace\OverlayForgeGearBlocksPlugin\OverlayForgeGearBlocksPlugin.csproj`, `cargo fmt --manifest-path src-tauri\Cargo.toml`, and `npm run cargo:build`, then installed the rebuilt DLL into the local GearBlocks BepInEx plugin folder while GearBlocks was closed.
- 17:31:38 EDT - Validated GearBlocks part render profiles and rotation transform helpers with `cargo fmt --manifest-path src-tauri\Cargo.toml`, `npm run cargo:build`, `npm run cargo:test`, `npm run build`, and `git diff --check`.
- 18:45:15 EDT - Validated the full GearBlocks build-guide import and phase-2 latest-export workflow with `cargo fmt --manifest-path src-tauri\Cargo.toml`, `npm run cargo:build`, `npm run cargo:test`, `npm run build`, and `git diff --check`.
- 19:04:30 EDT - Validated GearBlocks build-guide manifest expansion with `npm run build` and `git diff --check`.
- 19:23:50 EDT - Validated duplicate-only GearBlocks staging-manifest paint slots with `npm run build` and `git diff --check`.
- 19:31:41 EDT - Validated GearBlocks staging-manifest list ordering with `npm run build` and `git diff --check`.
- 19:45:20 EDT - Validated the GearBlocks build-guide Export All and simplified info view changes with `cargo fmt --manifest-path src-tauri\Cargo.toml`, `npm run build`, `npm run cargo:build`, and `git diff --check`.
- 20:04:06 EDT - Validated per-part-type GearBlocks staging-manifest paint slots with `npm run build` and `git diff --check`.
- 20:10:03 EDT - Validated GearBlocks default paint palette documentation with `git diff --check`.
- 20:17:17 EDT - Validated GearBlocks slash-separated pipe staging manifest parsing with `npm run build` and `git diff --check`.
- 20:52:29 EDT - Validated temporary white jig block staging guidance with `npm run build` and `git diff --check`.
- 21:28:14 EDT - Validated GearBlocks `Beam x3` upright rotation correction with `npm run build` and `git diff --check`.
- 21:34:18 EDT - Validated GearBlocks build-guide unfocused background translucency styling with `npm run build` and `git diff --check`.
- 21:44:00 EDT - Validated GearBlocks render-focused staging manifest columns with `npm run build` and `git diff --check`.

### 2026-06-28

#### Changed

- 11:28:32 EDT - Changed the GearBlocks build-guide overlay into a step-focused view with a static Overlay Forge-generated isometric placement diagram, step navigation, current-step instructions, and related placement parts.
- 12:48:56 EDT - Changed the GearBlocks build-guide overlay to separate Build Info from Build Steps, with Build Steps showing generated isometric diagrams and detailed placement/attachment captions only.
- 21:04:32 EDT - Changed GearBlocks build-guide diagrams to use semi-realistic procedural catalog profiles for combustion-engine parts, including a composite cylinder-and-axle rendering for `Engine Rear (Driven) Crank x2 & Axle`.
- 22:58:17 EDT - Added generic local game character build records and changed the Path of Exile 2 Builds section to list, edit, activate, and delete POE2 build records from SQLite.

#### Fixed

- 12:24:13 EDT - Fixed Tauri shutdown lifecycle handling so background GearBlocks import, scheduler, and mouse-shortcut worker loops stop before continuing to access app state during process exit.

#### Documentation

- 11:28:32 EDT - Documented that GearBlocks build-guide visuals are rendered in Overlay Forge from parsed guide steps and part rows without requiring GearBlocks script windows, BepInEx, or a persisted visual-step table.
- 12:48:56 EDT - Documented the Build Info and Build Steps view split for GearBlocks build-guide overlays.
- 21:04:32 EDT - Documented GearBlocks build-guide procedural part profiles for catalog parts whose dimensions should not be interpreted as generic boxes.
- 22:58:17 EDT - Documented the Path of Exile 2 build-planner foundation, `obj_game_character_build`, and the boundary that passive tree, item, gem, and calculation layers remain future work.

#### Validation

- 11:31:38 EDT - Validated the GearBlocks static isometric build-step overlay with `npm run build` and `git diff --check`.
- 12:24:13 EDT - Validated the Tauri shutdown lifecycle fix with `npm run cargo:build`.
- 12:48:56 EDT - Validated the GearBlocks build-guide view split with `npm run build`.
- 21:04:32 EDT - Validated GearBlocks build-guide procedural part profiles with `npm run build`.
- 22:59:10 EDT - Validated the Path of Exile 2 local build-record foundation with `cargo fmt --manifest-path src-tauri\Cargo.toml`, `npm run build`, `npm run cargo:build`, `npm run cargo:test`, `npm run cargo:clippy`, and `git diff --check`.

### 2026-06-26

#### Changed

- 01:49:27 EDT - Changed GearBlocks runtime sync so automatic scene-delta monitoring is disabled and GearBlocks chat submission requests/imports a rich full-scene export before prompt context is assembled.
- 02:10:56 EDT - Added the Mobalytics Ice Shot Deadeye leveling guide as the currently played Path of Exile 2 build and surfaced it in the POE2 Builds section.
- 21:56:15 EDT - Changed GearBlocks runtime export storage so successful imports populate normalized part definition and latest scene instance rows instead of retaining raw full-scene export JSON in SQLite.
- 21:58:50 EDT - Cleaned the local SQLite database by backing up the pre-cleanup file, backfilling latest GearBlocks runtime part instances, clearing historical raw runtime export JSON, checkpointing WAL, and vacuuming the database.
- 22:07:42 EDT - Added normalized GearBlocks definition and value tables for runtime metadata fields, attachment types, part settings, and output/control channels.
- 22:23:44 EDT - Changed GearBlocks chat prompt context to use a backend scene-context service that reads normalized SQLite scene rows and definition/value tables before falling back to legacy raw export JSON.
- 22:51:22 EDT - Changed GearBlocks chat Send / Enter to use the latest normalized SQLite scene context without requesting a runtime export, log import, or scene diff.
- 22:51:22 EDT - Added a short GearBlocks chat `Diff` action that explicitly refreshes the runtime scene, computes the latest scene diff, and includes that diff in the prompt.
- 23:20:50 EDT - Changed the GearBlocks chat diff action to include the latest stored scene diff without requesting a new runtime export or parsing logs.
- 23:20:50 EDT - Changed GearBlocks chat input buttons to compact `G`, `D↑`, and `↑` labels in a shared action strip so controls reserve space instead of overlapping.

#### Fixed

- 00:00:07 EDT - Fixed GearBlocks chat context for repeated structural parts by adding parent construction group summaries and duplicate-safe construction/id/index references to runtime prompt context.
- 21:23:23 EDT - Fixed GearBlocks chat submission so prompts are persisted before prompt-time scene refresh and scene-refresh failures are passed to chat as context warnings instead of aborting the response.
- 21:33:47 EDT - Fixed the standalone game chat overlay layout so the message area fills expanded empty space while the screenshot summary and prompt composer stay pinned to the bottom.

#### Documentation

- 00:00:07 EDT - Documented parent construction groups as part of GearBlocks semantic runtime context and troubleshooting for repeated part IDs.
- 00:59:03 EDT - Documented the user-tested GearBlocks player character height as 20 units / 20 blocks / 200 cm for human-scale build design.
- 01:49:27 EDT - Documented the GearBlocks prompt-time full-scene export workflow that replaces passive scene-delta monitoring.
- 02:10:56 EDT - Documented the Path of Exile 2 current-build setting stored in `obj_game_setting`.
- 21:56:15 EDT - Documented normalized GearBlocks runtime export storage, `def_gearblocks_part`, and latest-scene runtime part instances.
- 22:07:42 EDT - Documented GearBlocks metadata, attachment, setting, and output/control channel definition tables.
- 22:23:44 EDT - Documented the GearBlocks DB-backed chat scene-context path and deferred the optional derived scene-facts cache.
- 22:51:22 EDT - Documented that normal GearBlocks chat sends use DB scene context and the `Diff` action is the explicit scene-refresh/diff path.
- 23:20:50 EDT - Documented that the `D↑` chat action includes the latest stored scene diff and manual scene refresh is responsible for computing a fresh diff.

#### Validation

- 00:01:27 EDT - Validated GearBlocks parent construction group prompt context with `npm run build`, `npm run cargo:build`, `npm run cargo:test`, and `git diff --check`.
- 00:59:52 EDT - Validated GearBlocks player-character height prompt context with `npm run build`, `npm run cargo:build`, and `git diff --check`.
- 01:51:29 EDT - Validated GearBlocks prompt-time rich export sync with `npm run build`, `npm run cargo:build`, `npm run cargo:test`, `git diff --check`, and installed-script delta-monitor checks.
- 02:15:07 EDT - Validated the Path of Exile 2 current-build setting and Builds display with `npm run build`, `npm run cargo:build`, and `npm run cargo:test`.
- 21:23:23 EDT - Validated GearBlocks chat submission hardening with `npm run build`, `npm run cargo:build`, and `npm run cargo:test`.
- 21:33:47 EDT - Validated the standalone game chat overlay layout fix with `npm run build`.
- 21:56:15 EDT - Validated normalized GearBlocks runtime export storage with `npm run cargo:build`, `npm run build`, `npm run cargo:test`, and `git diff --check`.
- 21:58:50 EDT - Verified the local SQLite cleanup left zero raw runtime export payload rows, retained 1,465 runtime export manifest rows, backfilled 55 latest runtime part instance rows, and reduced `overlay-forge.sqlite3` to 49,303,552 bytes.
- 22:07:42 EDT - Validated GearBlocks metadata definition tables with `cargo fmt --manifest-path src-tauri/Cargo.toml`, `npm run build`, `npm run cargo:test`, `npm run cargo:build`, and `git diff --check`.
- 22:23:44 EDT - Validated GearBlocks DB-backed chat scene context with `cargo fmt --manifest-path src-tauri/Cargo.toml`, `npm run cargo:build`, `npm run build`, `npm run cargo:test`, `npm run cargo:clippy`, and `git diff --check`.
- 22:51:22 EDT - Validated GearBlocks chat diff gating with `cargo fmt --manifest-path src-tauri/Cargo.toml`, `npm run cargo:build`, `npm run build`, `npm run cargo:test`, and `git diff --check`.
- 23:20:50 EDT - Validated GearBlocks stored-diff chat sends and compact chat controls with `cargo fmt --manifest-path src-tauri/Cargo.toml`, `npm run cargo:build`, `npm run build`, `npm run cargo:test`, and `git diff --check`.
- 23:29:13 EDT - Validated all uncommitted Overlay Forge work with `cargo fmt --manifest-path src-tauri\Cargo.toml`, `npm run build`, `npm run cargo:build`, `npm run cargo:test`, `npm run cargo:clippy`, and `git diff --check`.

### 2026-06-25

#### Fixed

- 23:37:43 EDT - Fixed GearBlocks runtime log imports after `Player.log` rotation so completed full-scene exports near the beginning of a replacement log are recovered instead of being skipped by the incremental tail-read limit.

#### Documentation

- 23:37:43 EDT - Documented GearBlocks runtime export recovery behavior for successful in-game exports that were missed after log rotation.

#### Validation

- 23:38:54 EDT - Validated GearBlocks runtime log rotation recovery with `npm run build`, `npm run cargo:build`, `npm run cargo:test`, and `git diff --check`.

### 2026-06-24

#### Changed

- 19:26:50 EDT - Expanded GearBlocks runtime export and chat context so build-guide reasoning can use paint/material details, attachment and link-node types, tweakables, resizable settings, controllable state, engine relationships, and per-part coordinates from the latest export or scene delta.

#### Documentation

- 19:26:50 EDT - Documented the GearBlocks API surfaces that support build-guide context and clarified which live runtime values are included for chat.

#### Validation

- 19:27:56 EDT - Validated GearBlocks build-guide runtime detail exports with `npm run build`, `npm run cargo:build`, `npm run cargo:test`, and `git diff --check`.

## 0.9.0 - 2026-06-24

### 2026-06-24

#### Changed

- 19:00:46 EDT - Changed project version metadata to `0.9.0` for the stable build-guide release cut.
- 18:23:14 EDT - Associated GearBlocks chat context with the active/latest build guide and changed generated build guides to include glossary and relative-placement instructions.
- 14:52:37 EDT - Tightened the GearBlocks Builder script window with compact numeric fields, small Back/Snap buttons, top-row builder toggles, and a smaller default Builder window size.
- 13:44:03 EDT - Added an explicit GearBlocks API screen import that indexes the official Doxygen API docs into the local normalized API catalog.
- 13:05:09 EDT - Changed standalone overlay scrollbars, prompt/message borders, and build-guide section outlines to use the same focus-aware transparency contract as standalone window backgrounds.
- 12:52:35 EDT - Refactored backend overlay window lifecycle handling into a Rust window domain module with `WindowKind`, composed config structs, and `WindowManager`.
- 12:52:35 EDT - Changed SQLite connection locking to return a repository error instead of panicking on a poisoned database mutex.
- 12:52:35 EDT - Centralized repeated frontend unknown-error formatting and screenshot timestamp label helpers under shared utility modules.
- 12:52:35 EDT - Added root `npm` validation scripts for backend build, backend tests, Clippy, and combined frontend/backend checks.

#### Fixed

- 16:46:30 EDT - Fixed Ctrl+Shift+G build-guide overlay hiding by routing visibility checks and hides through native-aware window manager fallbacks.
- 16:31:46 EDT - Fixed GearBlocks Builder script button and checkbox text disappearing by removing compact font property writes from script controls.
- 13:23:54 EDT - Fixed build-guide standalone overlay focus detection by adding the build-guide window to the default Tauri capability scope so it can read its own focused state.
- 13:21:21 EDT - Fixed standalone overlay focus styling so a build-guide window only becomes opaque when that build-guide window itself has focus, not when chat or the main Overlay Forge window has focus.

#### Documentation

- 13:33:39 EDT - Tightened Codex reasoning preflight rules to explicitly stop before any tool use or context gathering when Low reasoning should escalate to Medium, and documented the observed Low-to-Medium enforcement failure.
- 12:52:35 EDT - Updated architecture and validation rules with shared utility guidance, high-arity typed-parameter guidance, Clippy review expectations, and large-module cleanup guardrails.
- 12:52:35 EDT - Added high-arity repository/command refactors and large backend module splitting to deferred architecture cleanup.

#### Validation

- 19:03:00 EDT - Validated the `0.9.0` stable build-guide release cut with `npm run build`, `npm run cargo:build`, `npm run cargo:test`, and `git diff --check`.
- 12:53:33 EDT - Validated architecture cleanup with `npm run check`, `npm run cargo:test`, `npm run cargo:clippy`, and `git diff --check`; Clippy now reports only documented high-arity refactor warnings.

## 0.8.0 - 2026-06-24

### 2026-06-24

#### Added

- 09:52:14 EDT - Added GearBlocks chat-to-build-guide generation so a chat draft or latest user message can produce a Markdown guide, save it under app data, and import it into the Build Guides list automatically.

#### Changed

- 10:21:59 EDT - Changed chat and build-guide overlay title bars to remain pinned and draggable while removing the chat overlay side controls.
- 10:42:42 EDT - Changed standalone overlay windows to use shared focus-aware opacity so unfocused chat and build-guide windows are twice as transparent as their focused state.
- 10:46:28 EDT - Changed standalone overlay unfocused opacity to be barely visible while the game has focus.
- 10:50:38 EDT - Changed standalone overlay focus dimming to use CSS background transparency so chat and build-guide text stays fully opaque while the game has focus.
- 10:55:45 EDT - Added a shared `standalone-overlay-window` class so chat, build guides, and future standalone overlays use the same focused and unfocused background transparency contract.
- 11:09:07 EDT - Changed standalone overlay focus detection to follow whether any Overlay Forge window is focused and route chat and build-guide backgrounds through shared standalone window variables.
- 11:14:12 EDT - Changed standalone overlay focus detection to use the OS foreground window instead of Tauri's app-local focused window state.
- 11:26:47 EDT - Changed Tauri standalone overlay windows to use transparent window backing and full native opacity so CSS background alpha can reveal the game while text remains opaque.
- 11:34:09 EDT - Changed standalone overlay transparency to default to the near-transparent state and only switch opaque when Overlay Forge foreground focus is positively identified.
- 11:49:17 EDT - Changed the root HTML background fallback to transparent so standalone overlay window transparency is not blocked by the document backing layer.
- 12:17:37 EDT - Changed project version metadata to `0.8.0` for the stable build cut.

#### Fixed

- 10:31:23 EDT - Fixed the chat overlay layout so the pinned title bar no longer pushes the prompt input below the visible window area.
- 10:55:45 EDT - Fixed standalone overlay transparency regressions so chat no longer stays fully opaque and build guides no longer fall back to native low-opacity text.
- 11:09:07 EDT - Fixed standalone window transparency state so chat and build-guide overlays share the same near-transparent game-focus background behavior.
- 11:14:12 EDT - Fixed standalone overlays staying opaque while GearBlocks has focus by comparing the actual Windows foreground window against Overlay Forge windows.
- 11:26:47 EDT - Fixed standalone overlays appearing fully opaque in-game by enabling transparent Tauri window surfaces and removing native low-opacity restores from standalone windows.
- 11:34:09 EDT - Fixed stale foreground detection by returning the foreground Overlay Forge window label and ignoring stale current-standalone foreground results when the standalone webview is not focused.
- 11:49:17 EDT - Fixed the remaining opaque standalone overlay backing layer caused by `index.html` forcing `html`, `body`, and `#root` to a solid background color.

#### Documentation

- 09:52:14 EDT - Documented the GearBlocks chat-generated build guide workflow.
- 10:56:45 EDT - Added a design notes document for recording future design thoughts without creating implementation commitments.
- 11:04:29 EDT - Updated reasoning model selection rules so Codex stops only when the current setting is too low for the task or unnecessarily High / Very High for Low / Medium work.
- 11:49:17 EDT - Added a known issues document and recorded the deferred Codex reasoning escalation enforcement issue.
- 12:14:48 EDT - Updated Codex and architecture rules with abstraction-first implementation guidance, stack-appropriate Rust/TypeScript patterns, and first-class window domain requirements.

#### Validation

- 09:52:14 EDT - Validated GearBlocks chat-to-build-guide generation with `npm run build`, `cargo check`, `cargo build`, and `git diff --check`.
- 10:22:24 EDT - Validated overlay titlebar and scrollbar updates with `npm run build` and `git diff --check`.
- 10:31:23 EDT - Validated the chat overlay height fix with `npm run build`.
- 10:42:42 EDT - Validated standalone overlay focus-opacity handling with `npm run build`.
- 10:46:28 EDT - Validated the lowered unfocused standalone overlay opacity with `npm run build`.
- 10:50:38 EDT - Validated CSS-based standalone overlay focus dimming with `npm run build`.
- 10:55:45 EDT - Validated the shared standalone overlay class and transparency regression fix with `npm run build`.
- 11:09:07 EDT - Validated global standalone overlay focus-state handling with `npm run build`.
- 11:14:12 EDT - Validated OS foreground based standalone overlay focus handling with `npm run build` and `cargo check`.
- 11:26:47 EDT - Validated transparent standalone window backing and native opacity cleanup with `npm run build` and `cargo check`.
- 11:34:09 EDT - Validated safe-default standalone transparency and foreground label handling with `npm run build` and `cargo check`.
- 11:49:17 EDT - Validated root HTML transparency and known-issue documentation with `npm run build` and `git diff --check`.
- 12:15:10 EDT - Validated architecture-rule documentation updates with `git diff --check -- AGENTS.md docs/ARCHITECTURE.md CHANGELOG.md`.
- 12:18:57 EDT - Validated the `0.8.0` stable build cut with `npm run build`, `cargo build`, and `git diff --check`.

### 2026-06-23

#### Changed

- 00:30:00 EDT - Changed GearBlocks chat so in-game marker prompting and marker action buttons are disabled while the marker feature is paused.
- 00:30:00 EDT - Changed GearBlocks runtime coordinate context to support spatial reasoning without asking chat to emit marker blocks.
- 00:47:51 EDT - Changed GearBlocks runtime prompt context to include user-defined friendly names for exact part instances when those aliases are imported from the in-game script.
- 23:06:16 EDT - Changed the GearBlocks build-guide overlay to support three local text sizes with Ctrl+Shift+mousewheel.

#### Fixed

- 22:33:00 EDT - Fixed the GearBlocks Build Guides view so it no longer collides with the Path of Exile 2 Builds section and added a Ctrl+Shift+G fallback that opens the Build Guides panel when no guide overlay is active.
- 22:47:43 EDT - Fixed the GearBlocks build-guide overlay so it loads the selected imported guide reliably, removes the redundant titlebar buttons, and uses the guide title box as the window drag handle.
- 22:53:40 EDT - Fixed the GearBlocks build-guide overlay selection handoff by storing the clicked guide ID before opening the separate overlay window and removing the duplicate empty-state title text.
- 23:06:16 EDT - Fixed GearBlocks build-guide Markdown cleanup so table separators, horizontal rules, and fenced code markers are filtered from parts, steps, and section text.

#### Added

- 00:47:51 EDT - Added a GearBlocks script `Names` section that emits targeted or pivot-part friendly name aliases into `Player.log`.
- 00:47:51 EDT - Added SQLite persistence for GearBlocks runtime part aliases keyed by physical part instance.
- 22:08:09 EDT - Added GearBlocks Markdown build guide import, normalized build-guide persistence, and an independent narrow build-guide overlay window with persisted bounds.

#### Documentation

- 00:30:00 EDT - Documented GearBlocks markers, BepInEx plugin status UI, and GearLib plugin work as backlog items until the marker feature is explicitly resumed.
- 00:47:51 EDT - Documented the GearBlocks friendly part naming workflow and runtime alias table.
- 22:08:09 EDT - Documented the GearBlocks build-guide overlay workflow and SQLite build-guide tables.

#### Validation

- 00:30:58 EDT - Validated the GearBlocks marker unwiring with `npm run build`, `cargo build`, and `git diff --check`.
- 00:48:56 EDT - Validated GearBlocks friendly part names with `npm run build`, `cargo check`, `cargo build`, `git diff --check`, and installed-script marker checks.
- 22:09:34 EDT - Validated the GearBlocks build-guide overlay with `npm run build`, `cargo check`, `cargo build`, and `git diff --check`.

### 2026-06-22

#### Documentation

- 22:12:09 EDT - Reworked README, changelog, and repository Markdown structure around direct Codex use in VS Code.
- 22:12:09 EDT - Added semantic versioning rules so version numbers change for meaningful release scope rather than each work session.

## 0.7.0 - 2026-06-22

Version section for the Path of Exile 2 game module work.

### 2026-06-22

#### Added

- 21:23:27 EDT - Added a seeded Path of Exile 2 game module scaffold with Home, Chats, Builds, Skill Tree, Items, Skill Gems, Support Gems, Loot Filter, and Trade sections.

#### Changed

- 21:23:27 EDT - Changed project version metadata to `0.7.0`.

#### Documentation

- 21:23:27 EDT - Documented the Path of Exile 2 game module scaffold and updated the active version reference to `0.7.0`.

#### Validation

- 21:27:10 EDT - Validated the Path of Exile 2 game module scaffold with `npm run build`, `cargo check`, `cargo test`, and `git diff --check`.

## 0.6.1 - 2026-06-21

Version section for the SQLite naming normalization and scheduler-backed persistence migration.

### 2026-06-22

#### Changed

- 00:06:33 EDT - Changed GearBlocks chat-send runtime importing to reconcile the full `Player.log` and `Player-prev.log` after cursor-based diff imports so missed completed exports are persisted before prompt context is assembled.
- 00:06:33 EDT - Changed GearBlocks runtime prompt context to include the exact latest export ID and exported timestamp used by chat.
- 00:18:24 EDT - Changed GearBlocks chat submission to avoid triggering a full in-game scene export and instead only import/reconcile completed `Player.log` additions before prompt context is assembled.
- 00:35:49 EDT - Changed the GearBlocks Lua script to emit compact scene-delta log records for added, changed, and removed parts after a full scene export baseline.
- 01:18:15 EDT - Changed the `npm run tauri:dev` workflow to write a PowerShell transcript log for each dev session under `logs\tauri-dev`.
- 01:24:28 EDT - Changed the Tauri dev logging wrapper to redirect the full `tauri dev` output to a session log and mirror new log lines back to the visible terminal.
- 01:36:26 EDT - Changed the GearBlocks BepInEx marker plugin template to include the Unity IMGUI reference used for screen-space marker crosshairs and labels.

#### Fixed

- 00:35:49 EDT - Fixed GearBlocks chat context missing copied parts after the last full export by importing scene deltas from `Player.log` and applying them as synthetic `sceneDeltaPatch` runtime snapshots before prompt context is assembled.
- 00:48:08 EDT - Fixed GearBlocks Chats shortcut freezes by keeping the passive runtime import monitor cursor-only, seeding missing monitor cursors at the log tail, and reserving full-log reconciliation for explicit refresh/import actions.
- 00:59:31 EDT - Fixed the remaining first-open GearBlocks Chats freeze by removing the chat-view runtime import polling loop, capping stale runtime log catch-up reads, filtering player-character scene deltas, and quieting the Lua delta monitor so player animation and world-position drift are not logged as scene edits.
- 01:09:43 EDT - Fixed intermittent chat input lag by removing the dedicated chat overlay's GearBlocks runtime import polling loop and the main app's 500ms pending-shortcut polling fallback.
- 01:13:20 EDT - Fixed the debug restart helper so automated Overlay Forge restarts launch the visible app window instead of starting the process hidden.
- 01:36:26 EDT - Fixed GearBlocks marker visibility by adding a screen-space Unity crosshair and label over spawned marker world positions in addition to the runtime scene marker objects.

#### Known Issues

- 01:48:22 EDT - GearBlocks marker commands are reaching the BepInEx plugin, but visual markers are still not appearing in-game; marker rendering work is paused while in-game chat usage is prioritized.

#### Documentation

- 00:35:49 EDT - Documented GearBlocks baseline export plus scene-delta prompt context behavior in the runtime exporter and data model docs.
- 01:12:02 EDT - Documented that Overlay Forge restarts should be automated with the repo stop/start scripts when safe, while GearBlocks and other stateful user apps still require explicit user action or warning.
- 01:16:34 EDT - Documented that automated Overlay Forge restarts during development should prefer the visible `npm run tauri:dev` workflow so both Vite and Tauri are running.
- 01:18:15 EDT - Documented the Tauri dev-session terminal log location and latest-log pointer file.
- 01:36:26 EDT - Documented the GearBlocks marker plugin's Unity IMGUI reference requirement and screen-space marker behavior.

#### Validation

- 00:06:33 EDT - Validated GearBlocks chat-send runtime reconciliation with `cargo check`, `cargo test`, `npm run build`, and `git diff --check`.
- 00:18:24 EDT - Validated diff-only GearBlocks chat submission importing with `cargo check`, `npm run build`, and `git diff --check`.
- 00:35:49 EDT - Validated GearBlocks scene-delta importing with `cargo check`, `cargo test`, `npm run build`, and `git diff --check`; regenerated the installed GearBlocks script with the new delta monitor.
- 00:48:08 EDT - Validated the GearBlocks passive import freeze fix with `cargo check`, `cargo test`, `npm run build`, and `git diff --check`.
- 00:59:31 EDT - Validated the GearBlocks chat-view polling and scene-delta spam fix with `cargo check`, `cargo test`, `npm run build`, and `git diff --check`; regenerated the installed GearBlocks script with the quieter delta monitor.
- 01:09:43 EDT - Validated the chat input lag fix with `npm run build`, `cargo check`, and `git diff --check`.
- 01:13:20 EDT - Validated the visible Overlay Forge restart by stopping the hidden debug process, rebuilding, starting a visible debug process, and confirming PID `8072`.
- 01:16:34 EDT - Validated the corrected dev restart by stopping the raw debug executable, launching `npm run tauri:dev` in a visible PowerShell session, and confirming Node/Vite plus `overlay-forge.exe` child processes.
- 01:18:15 EDT - Validated the dev terminal logging wrapper with PowerShell parser syntax validation, package script inspection, and `git diff --check`.
- 01:24:28 EDT - Validated the corrected dev terminal logging wrapper by launching `npm run tauri:dev`, confirming Vite/Tauri output in `logs\tauri-dev\latest.txt`, and confirming the `overlay-forge.exe` dev process.
- 01:36:26 EDT - Validated the screen-space GearBlocks marker plugin update with `dotnet build` and installed the rebuilt BepInEx plugin DLL into the local GearBlocks install.

### 2026-06-21

#### Added

- 20:07:26 EDT - Added normalized `def_game`, `obj_game`, and `obj_game_setting` persistence so static game definitions, local game rows, and deep per-game settings do not require table-per-game schemas.
- 21:06:34 EDT - Added a source-only GearBlocks BepInEx/GearLib plugin template folder with local reference guidance and ignored workspace conventions.
- 22:11:56 EDT - Added a direct Overlay Forge GearBlocks BepInEx plugin template with a file-backed command channel and temporary center-raycast marker command.
- 22:47:46 EDT - Added a general GearBlocks chat marker capability where assistant responses can include user-approved `overlay-forge-markers` JSON plans that Overlay Forge sends to the BepInEx plugin as temporary world-coordinate marker commands.

#### Changed

- 19:56:38 EDT - Changed project version metadata to `0.6.1`.
- 20:07:26 EDT - Changed SQLite initialization to non-destructively rename legacy tables into the `obj_`, `def_`, and `n2n_` naming convention and add `schema_json` plus `modified_at` metadata columns.
- 22:47:46 EDT - Changed GearBlocks runtime prompt context to include marker guidance and functional-part world coordinates so chat can point at connection points without using custom parts.
- 23:38:51 EDT - Changed GearBlocks runtime part indexing to store first-class world and local coordinates for every exported part and backfill existing rows from stored runtime JSON.
- 23:47:57 EDT - Changed GearBlocks marker commands and the BepInEx plugin template to use larger default markers with runtime Unity sphere objects plus thicker crosshair lines.

#### Fixed

- 20:15:11 EDT - Fixed partially migrated SQLite databases by copying legacy rows into normalized tables and dropping copied legacy duplicates so normalized unique indexes can be recreated for `ON CONFLICT` upserts.
- 22:23:52 EDT - Fixed the direct GearBlocks BepInEx plugin command parser to avoid Newtonsoft's IL2CPP generic deserialize path and remove the unsupported custom command parameter warning.
- 23:47:57 EDT - Fixed chat-authored GearBlocks markers being too small and line-only to reliably see in the Unity scene.

#### Documentation

- 20:07:26 EDT - Documented the 0.6.1 SQLite naming normalization, game definition model, and normalized-table project rules.
- 21:06:34 EDT - Documented GearLib as a third-party user-installed dependency and linked the new BepInEx template guidance from project docs.
- 22:11:56 EDT - Documented the direct GearBlocks BepInEx plugin path, runtime marker command folder, supported commands, and install location.
- 22:47:46 EDT - Documented chat-authored marker blocks, world-coordinate marker commands, and the user-approved marker dispatch workflow.
- 23:15:00 EDT - Documented that one `npm run tauri:dev` run hit a Rust `STATUS_ACCESS_VIOLATION` and required exiting GearBlocks before continuing BepInEx plugin development.
- 23:38:51 EDT - Documented GearBlocks runtime part world/local coordinate columns and marker coordinate context for structural-only parts.
- 23:47:57 EDT - Documented the larger GearBlocks marker size and runtime Unity object marker behavior.
- 23:55:22 EDT - Documented that restart, relaunch, and reload requirements must be shown as bold red user-facing warnings.

#### Validation

- 20:09:58 EDT - Validated the 0.6.1 database migration with `cargo test`, `cargo check`, `npm run build`, and `git diff --check`.
- 20:15:30 EDT - Validated the partial SQLite migration repair with `cargo test`, `cargo check`, `npm run build`, `git diff --check`, and a live startup check against the app-data database.
- 21:08:49 EDT - Validated the GearBlocks BepInEx/GearLib template scaffold with .NET SDK and BepInEx template checks plus `git diff --check`.
- 22:11:56 EDT - Validated the direct GearBlocks BepInEx plugin template by building the ignored working copy with `dotnet build` and installing the compiled DLL into the local GearBlocks BepInEx plugins folder.
- 22:47:46 EDT - Validated the chat marker capability with `npm run build`, `cargo check`, and `dotnet build`; live DLL replacement is pending until GearBlocks releases the loaded plugin file.
- 23:38:51 EDT - Validated runtime coordinate indexing with `cargo test`, `cargo check`, `npm run build`, and `git diff --check`.
- 23:47:57 EDT - Validated visible GearBlocks marker updates with `dotnet build`, `cargo check`, `npm run build`, and `git diff --check`; installing the rebuilt DLL is pending because GearBlocks has the current plugin DLL locked.
- 23:54:33 EDT - Installed the rebuilt visible-marker GearBlocks BepInEx plugin DLL after GearBlocks released the loaded file lock.

## 0.6.0 - 2026-06-21

Version section for work starting on 2026-06-21 after the completed `0.5.0` GearBlocks scene-context workflow session.

### 2026-06-21

#### Added

- 15:37:18 EDT - Added a local Smoking Cessation module with SQLite cigarette event tracking, a configurable record-cigarette keybind, Nicoderm Step 1 patch marker, and day/week/month/year charts.
- 19:44:22 EDT - Added a local scheduler framework with `def_scheduler_type`, `obj_scheduler`, and `obj_scheduler_run` tables, safe Rust-handler dispatch, run history, and a seeded Smoking Cessation export refresh job.

#### Changed

- 12:09:22 EDT - Changed project version metadata to `0.6.0`.
- 12:30:17 EDT - Changed GearBlocks script integration so scene export, BuilderToolExt helpers, and WeldTool controls share one resizable Overlay Forge script window with clickable section navigation.
- 12:49:09 EDT - Changed the GearBlocks script window navigation to start on a compact vertical home menu, replace content per tool view, and expose a top Back button for returning home.
- 14:06:17 EDT - Changed GearBlocks scene export UI feedback to show export progress, success, and part count in the Scene view while preserving window position across section changes.
- 14:22:08 EDT - Changed GearBlocks script window positioning so the initial home window opens centered before section navigation starts remembering moved positions.
- 14:28:28 EDT - Changed GearBlocks section navigation to avoid reapplying window alignment after initial load so moved script windows no longer jump to the top-left corner.
- 14:51:50 EDT - Changed GearBlocks runtime context importing so manual script exports are indexed from `Player.log` by a passive monitor and before chat responses, and changed Ctrl+Shift+C to toggle the GearBlocks chat overlay window.
- 14:53:46 EDT - Changed GearBlocks runtime log importing to reconcile completed full-log exports newer than the database latest when the incremental cursor has already advanced past them.
- 16:07:40 EDT - Changed game chat overlay placement so each chat persists its last window coordinates in SQLite and restores them across app restarts.
- 16:42:40 EDT - Changed Ctrl+Shift+C chat overlay cycling so selected chats are focused from game context and hidden only when the chat overlay is already foreground.
- 17:21:53 EDT - Changed GearBlocks chat context and rules to use the developer-confirmed scale of 1 GearBlocks unit = 10 cm and avoid imperial-distance build advice unless requested.
- 17:41:21 EDT - Added a GearBlocks Home status skeleton for detecting user-installed BepInEx and GearLib third-party dependencies without bundling or installing them.
- 18:21:15 EDT - Changed the GearBlocks BepInEx status check to read local BepInEx logs and report installed version, install correctness, and chainloader activation.
- 18:46:40 EDT - Changed the Smoking Cessation record/delete workflows to maintain a ChatGPT-readable Markdown export of patch status, cigarette counts, and event history.
- 18:54:36 EDT - Changed the Smoking Cessation module to track current cigarette inventory, decrement it when recording cigarettes, graph recent minutes-between-cigarettes spacing, and estimate run-out time.
- 19:16:24 EDT - Changed module headers to remove the global shortcut label from Cessation and other module views.
- 19:44:22 EDT - Changed the Smoking Cessation predicted run-out display to refresh on a one-minute UI clock while the scheduler keeps the ChatGPT export fresh in the background.

#### Fixed

- 15:44:33 EDT - Fixed the Ctrl+Shift+C chat overlay shortcut so it does not re-show the main Overlay Forge window when an active game chat context is already selected.
- 16:14:26 EDT - Fixed Ctrl+Shift+C so it toggles the active chat overlay directly from the backend even when the main Overlay Forge window is hidden.
- 18:56:56 EDT - Fixed the Smoking Cessation minutes-between-cigarettes graph so it stays clipped within its card and no longer overlaps the history panel.
- 19:01:52 EDT - Fixed the Smoking Cessation dashboard layout so the rate graph keeps its full height and pushes content into vertical scrolling instead of shrinking at the bottom.

#### Documentation

- Added project reasoning model selection rules that default work to Medium Reasoning and define High / Very High escalation criteria.
- Added a reasoning escalation gate requiring Codex to stop and ask the user to switch VS Code/Codex reasoning settings before High or Very High tasks.
- Added a reasoning calibration rule requiring Codex to flag after Medium Reasoning tasks when Low Reasoning would likely have been sufficient.
- Added the forward SQLite table naming convention for new `obj_`, `def_`, `o2o_`, and `n2n_` tables plus scheduler safety rules.

## 0.5.0 - 2026-06-20

Version section for GearBlocks scene-context workflow work starting on 2026-06-20 after the completed `0.4.0` documentation rules session.

### 2026-06-20

#### Changed

- Changed the GearBlocks Lua exporter workflow so the default export path is the full live scene instead of a targeted construction.
- Removed the GearBlocks targeted export button binding from the installed exporter UI.
- Added the backend/runtime plumbing needed to fingerprint GearBlocks scene context changes after new `Player.log` exporter chunks appear.
- Added explicit `Refresh Scene Context` controls to GearBlocks Chat and Parts so scene-specific chat context can be updated after running `Export Scene`.
- Changed GearBlocks runtime export import to use per-log SQLite cursors so explicit refresh reads only new `Player.log` / `Player-prev.log` additions when possible.
- Added a compact runtime scene diff summary between the latest and previous scene exports and included it in GearBlocks chat prompt context.
- 18:45:48 EDT - Changed chat prompt keyboard submission so pressing Enter on a blank line submits the prompt.
- 18:55:37 EDT - Changed GearBlocks tool access by adding an Overlay Forge Tools view, installer, no-window script channel, and whitelisted BuilderToolExt / WeldTool actions.
- 20:37:52 EDT - Changed GearBlocks scene context refresh so it requests a fresh in-game scene export before importing new `Player.log` context.

#### Fixed

- Disabled GearBlocks runtime context polling from the React chat entry path so opening GearBlocks Chats does not synchronously parse and index full-scene `Player.log` exports.
- Changed GearBlocks chat send context to import new `Player.log` additions through the runtime cursor before using the latest indexed runtime scene snapshot.

## 0.4.0 - 2026-06-17

Version section for work starting on 2026-06-17 after the completed `0.3.0` session.

### 2026-06-18

#### Documentation

- Clarified versioning so version metadata changes only when a meaningful version is intentionally cut.
- Converted the project rules into a root `AGENTS.md` instruction file and kept `AGENTS.md` as a compatibility pointer.

### 2026-06-17

#### Changed

- Set Overlay Forge version metadata from `0.3.0` to `0.4.0` for a new capability group.

## 0.3.0 - 2026-06-16

Version section for work started on 2026-06-16. Related changes that continued into the early AM hours are retained in this `0.3.0` section.

### 2026-06-16

#### Added

- Added normalized GearBlocks runtime indexes for part API attributes, discovered `value` fields, properties, and attachments so repeated exports update searchable rows instead of relying only on each part's full JSON blob.
- Added canonical GearBlocks API catalog tables for documented construction namespace types, members, and parameters, plus a runtime part/member channel that maps observed API availability back to canonical member IDs.
- Expanded the canonical GearBlocks API catalog seed to include documented construction namespace classes and enum values, including Lua constant names where documented.
- Added a backend command and TypeScript service payload for browsing the indexed GearBlocks API catalog types, members, parameters, and enum values.
- Added a GearBlocks API tab for browsing indexed API types, members, method parameters, and enum values from the Gaming workspace.
- Added a GearBlocks runtime part API member detail query and changed the part detail view to show canonical API member availability from the indexed part/member channel when available.

#### Changed

- Set Overlay Forge version metadata from `0.2.0` to `0.3.0` for a new capability group.
- Marked 2026-06-16 work under the `0.3.0` changelog section, including related early-AM rollover changes.
- Added project rules for cutting a new semantic version only when a meaningful version boundary is intentionally cut.

#### Fixed

- Reduced GearBlocks exporter payload bloat by keeping cached known API indexes limited to top-level part availability metadata and removing the remaining behaviour reference value from `apiAttributes`.
- Changed the GearBlocks exporter log payload to emit part API availability metadata once in an `apiAttributeCatalog`, with parts referencing compact `apiAttributeKey` values that Overlay Forge expands during import.

### 2026-06-17

#### Changed

- Renamed the GearBlocks runtime log action to `Refresh Runtime Log` so the workflow is clear: export the current in-game build state, then refresh Overlay Forge's indexed runtime context without rebuilding the whole reference catalog.
- Restored GearBlocks saved construction refresh for chat context by decoding the latest modified `construction.bytes` before building the prompt, including saved-file part additions/removals alongside runtime export context.

#### Documentation

- Documented the GearBlocks runtime refresh workflow for syncing current build changes into chat context after an explicit in-game export.
- Documented that saved construction refresh reflects removals after the build is saved, while runtime log refresh remains the source for live runtime metadata.

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

- Updated README, project plan, architecture, and data model notes for the Overlay Forge `0.2.0` GearBlocks runtime API interface release and future troubleshooting path.

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

- Documented the Gaming chat overlay shortcut in README and project context.
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
- Added `AGENTS.md` to document the daily changelog tracking convention.
- Added `docs/GAMING_SCREENSHOT_VALIDATION.md` to document Gaming Screenshot Capture as complete, passed, and successful.
- Added `docs/GEARBLOCKS_PARTS_CATALOG.md` as a shareable ChatGPT reference for GearBlocks categories and cataloged parts.
- Updated README, project plan, architecture, data model, and project docs with the validated screenshot capture status.

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
- Milestone 11 - Implementation Request Drafts is complete, passed, and successful.
- Milestone 12 - Project Markdown Context is complete, passed, and successful.
- Milestone 13 - Project Workspace UI Consolidation is complete, passed, and successful.
- Milestone 13 refinement moves conversation attached context and local Markdown implementation request drafts into a collapsible right-hand chat pane.
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
- Added root documentation for Milestone 0, architecture, project plan, and implementation requests.
- Added `docs/PROJECT_OVERVIEW.md` as the manual repository context file.
- Added Scratchpad, Tasks, Notes, and Calendar component navigation.
- Added SQLite `tasks`, `notes`, and `calendar_events` tables.
- Added Tauri CRUD commands for tasks, notes, and calendar events.
- Added Tasks component with create, select, edit, delete, deadline, body, list, and restart restore support.
- Added Notes component with create, select, edit, delete, list, and restart restore support.
- Added Calendar component with create, edit, delete, list, and restart restore support.
- Added `docs/DATA_MODEL.md` for the current SQLite schema.
- Added Projects component navigation.
- Added SQLite `projects` table.
- Added Tauri CRUD commands for local projects.
- Added Projects component with create, select, read-only view, edit, delete, list, status, and restart restore support.
- Added Planning Chat component navigation.
- Added SQLite `planning_conversations` and `planning_messages` tables.
- Added Tauri planning chat commands for conversation listing, creation, deletion, message listing, and backend message sending.
- Added a backend OpenAI Responses API integration using `OPENAI_API_KEY`.
- Added Planning Chat project selection, conversation list, new conversation action, message history, message input, loading state, and readable error display.
- Added SQLite `project_github_repositories` table initialization for project-scoped GitHub repository linkage and metadata/status.
- Added Rust/Tauri GitHub integration commands for getting, saving, deleting, and fetching project repository metadata.
- Added backend-only GitHub metadata fetch behavior using `GITHUB_TOKEN`.
- Added frontend GitHub project-link UI inside the Projects component.
- Added readable missing-token, invalid repository full-name, and GitHub request error handling.
- Added YouTube component navigation.
- Added SQLite `youtube_references` table initialization for local YouTube reference persistence.
- Added Rust/Tauri YouTube reference commands for list, get, create, update, delete, and external open.
- Added frontend YouTube reference CRUD UI with create, selected read-only view, edit, delete, list, and optional user-entered metadata.
- Added YouTube URL validation for common watch, short link, and shorts URL forms.
- Added external-open behavior for saved YouTube URLs through the system browser.
- Added Project workspace sections for Overview, GitHub, and Chat.
- Added project-scoped Chat inside the selected project workspace.
- Added a required conversation title field before creating a project workspace chat conversation.
- Added a refined selected-project workspace header in Projects.
- Added Overview, GitHub, Chat, and References workspace sections.
- Added a minimal References section with project-local context category summaries.
- Added the Projects navigation tree pattern in the left navigation shell.
- Added an expandable Projects module row.
- Added saved project child rows under Projects.
- Added a compact Projects `+` creation action.
- Added compact project row `...` edit/delete actions.
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
- Added implementation request draft generation from selected project chat conversations.
- Added SQLite `obj_implementation_request_draft` table initialization for local implementation request draft persistence.
- Added backend commands for implementation request draft creation, listing, retrieval, and deletion.
- Added frontend `Draft Implementation Request` action in project workspace Chat.
- Added read-only Implementation Request Drafts panel with saved draft list and generated Markdown content display.
- Added generated Markdown draft structure with project, conversation source, goal, relevant context, implementation instructions, validation checklist, deferred items, and notes.
- Added attached context inclusion for project chat sends, including linked GitHub repository metadata when available.
- Added project-level local Markdown context configuration for selected projects.
- Added SQLite `project_markdown_context` table initialization for per-project local Markdown context roots.
- Added backend commands for getting, saving, clearing, and loading project Markdown context.
- Added safe local Markdown loading from configured project roots, including `README.md`, `CHANGELOG.md`, `docs/*.md`, `docs/*.md`, and explicit Markdown references found in `README.md`.
- Added readable warnings for missing, unreadable, unsafe, skipped, and truncated Markdown files.
- Added Project Markdown context display in selected-project Chat.
- Added project Markdown context inclusion for Prompt Preview, project chat sends, and implementation request draft generation.
- Added planning conversation child rows under project rows in the left navigation hierarchy.
- Added compact chat markers for conversation rows.
- Added direct conversation-row navigation into the focused project chat surface.
- Added project row `...` menu routing for Overview, Chat, References, Edit, and Delete.
- Added a focused chat surface that removes redundant Projects, Active Workspace, tab, and Planning Chat headings.
- Added Project Edit as the consolidated surface for project details, GitHub repository linkage, and project Markdown context configuration.

### Changed

- Configured the app bundle identifier as `com.overlayforge.desktop`.
- Configured the Windows app icon for Tauri bundling.
- Updated the implementation request with current validation status and manual checks.
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
- Confirmed implementation request drafts persist locally in SQLite.
- Confirmed implementation request draft deletion removes only the selected draft and does not delete source conversations, messages, or context records.
- Confirmed linked GitHub repository metadata is resolved from the selected project for implementation request drafts, prompt previews, and project chat sends.
- Confirmed export, full editor workflows, and direct Codex automation remain deferred beyond Milestone 11.
- Confirmed project Markdown context is project-scoped, local-first, and not stored as per-conversation manual attachments.
- Confirmed conversation manual attachments remain an additional context layer after project Markdown context.
- Confirmed GitHub file APIs, broad repository indexing, draft export, and direct Codex automation remain deferred beyond Milestone 12.
- Confirmed that Milestone 12 is complete, passed, and successful after the Milestone 13 UI consolidation pass.
- Clarified that Milestone 13 should preserve conversation-scoped manual attachment data semantics while moving attachment controls out of the primary chat surface.
- Added the Milestone 13 right-hand pane for conversation attached context and local Markdown implementation request drafts.
- Added a `New Chat` project action that starts on an empty new-conversation area.
- Preserved left-navigation conversation child-row selection as the only path for opening existing chats directly.
- Confirmed Milestone 13 did not require schema changes.
- Confirmed conversation manual attachments remain conversation-scoped after UI consolidation.
- Synced `docs/DATA_MODEL.md` to mark the Milestone 12 data model as complete and revalidated after Milestone 13.

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
