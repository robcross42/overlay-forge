# Overlay Forge Deferred Items

This file centralizes deferred work. Items listed here are not approved scope unless the user explicitly requests them.

## Retired Projects Module Review

- Decide whether a future workspace should restore Projects, replace it with another planning model, migrate old project records into another module, export them, or delete them after explicit approval.
- Review whether project-scoped chat, project Markdown context, bridge drafts, and project GitHub metadata should remain retired or return as a different workflow.
- Preserve legacy project/planning SQLite rows until an explicit migration, export, or deletion plan is requested.
- Avoid reintroducing a top-level Projects module without a new purpose and a clearer relationship to Gaming, Calendar, Repair Resell, and local documentation.

## Chat UX And Controls

- Chat message streaming.
- Model picker UI.
- Conversation search/filtering.
- Long-term prompt template system.
- Automatic prompt rewriting.

## Context, Prompting, And Knowledge

- Automatic context attachment.
- Token counting and budgeting.
- Semantic search.
- Vector stores.
- File uploads.
- Web search tooling.
- Broad repository indexing.
- GitHub repository file reading.
- YouTube transcript extraction.
- ChatGPT import.

## Codex Reasoning And Escalation

- Fix the observed escalation enforcement failure where Codex proceeded on Low reasoning after classifying a request as Medium instead of stopping before tool use and asking the user to switch to Medium.
- Flesh out reasoning-level rules and escalation methods beyond the current project guidance.
- Explore whether reasoning level can be chosen automatically instead of requiring manual VS Code/Codex setting changes.
- Investigate options for modifying or extending the VS Code Codex chat experience to better support reasoning-level selection and escalation workflows.

## Local Markdown Drafts

- Full draft editor.
- Approval or obsolete status workflow.
- Export drafts to local `.md` files.
- Copy-to-clipboard workflow.
- Direct editor automation.
- GitHub commit or pull-request creation from drafts.

## GitHub Integration

- Read or browse repository files through the GitHub API.
- Write operations: commits, branches, pull requests, and issues.
- GitHub Actions integration.
- Webhooks.
- OAuth flow.
- Multi-account support.
- Advanced sync engine.
- Conflict resolution.

## YouTube

- YouTube account login.
- YouTube API integration.
- OAuth.
- Subscription import.
- Watch history import.
- Recommendations.
- Transcript retrieval.
- Transcript summarization.
- Downloads.
- Embedded unrestricted browser.
- Playlist sync.
- Comment sync.
- Channel scraping.
- Background metadata crawler.

## Media Library

- Trakt, Plex, Jellyfin, Kodi, and streaming-account sync.
- Browser-history or service-history import.
- Automatic playback detection and partial playback position.
- Multiple profiles, cloud sync, social sharing, and household watch state.
- Recommendation engine and `Not interested` suppression.
- Provider price comparison and leaving-soon detection.
- New-release notifications, scheduled metadata refresh, calendar integration, and desktop notifications.
- Add a backend-owned streaming-link provider integration for dynamic movie, season, and episode deep links; evaluate the Streaming Availability API first because it can resolve existing TMDB IDs and return region-specific provider links.
- Cache Canadian provider icons and direct links in normalized SQLite rows keyed to movies or exact season/episode coordinates, while preserving user-owned manual links as overrides and fallbacks.
- Render a clickable provider icon beside a movie or episode only when the provider response contains a validated HTTPS deep link; do not fabricate provider URLs, scrape streaming services, automate account login, or infer Netflix content IDs from TMDB IDs.
- JustWatch or streaming-service scraping.
- Torrent, download, or piracy-related functionality.
- Trailer playback and expanded cast/crew browsing.
- Custom lists beyond Watch Next and tags.
- Rewatch history and rewatch counts.
- Manual season and episode editing.
- CSV, Trakt, IMDb, TV Time, Letterboxd, or other service import.
- Export beyond a future basic local backup format.
- Spoiler-protected episode notes.

## Sync And Import

- External/cloud sync.
- Multi-user auth.
- Project import/export.
- External calendar sync.

## Organizer Consolidation Review

- Review whether Scratchpad, Tasks, and Notes should be merged into Calendar, restored as standalone modules, moved under Projects, or replaced with a unified organizer surface.
- Preserve existing Scratchpad, Tasks, and Notes data until an explicit migration or deletion plan is requested.
- Decide whether Calendar should own lightweight reminders, dated notes, task-like items, and quick scratch entries.

## Repair Resell Future Work

- Multi-item auction pickup planning that groups watchlisted lots by source, region, pickup window, closing date, and vehicle/trailer capacity.
- Pickup economics that spread fuel and travel time across a whole load instead of evaluating each listing in isolation.
- Wednesday/business-hour pickup planning, plus Saturday/Sunday longer-trip planning, based on the user's work schedule.
- Regional arbitrage tracking between lower-cost inventory areas and Kitchener/GTA resale opportunities.
- Estate-auction bundle evaluation for hand tools, woodworking equipment, mechanics' tools, lawn equipment, bicycles, and shop equipment.
- Parts-harvesting workflows for donor machines, part-out value, combined repairs, and remaining-parts resale.
- Trailer-enabled hauling workflows for riding mowers, utility trailers, motorcycles, ATVs, compact tractors, shop machinery, and other larger-margin items.
- Optional return-load opportunity tracking to offset fuel costs when a truck/trailer trip can transport something back for someone else.
- Repair/restoration knowledge base records for symptoms, diagnosis, root cause, parts used, cost, time spent, photos, manuals, videos, lessons learned, and final outcome.
- Model, brand, engine-family, and failure-mode history so prior repairs surface when similar listings appear later.
- Phase-based learning progression from bicycles and small engines toward riding mowers, trailers, motorcycles, ATVs, compact tractors, engine rebuilding, vehicle restoration, and eventually a ground-up car build.
- LLM listing analysis and repair/resale estimate enrichment.
- OpenAI-backed category, make/model/year, red-flag, failure-mode, parts, and max-safe-bid suggestions.
- Listing photo ingestion, local image storage, and optional future vision support.
- Inventory, purchase, inspection, repair task, parts, sale listing, sale, analytics, and skill-area tracking tables.
- Comparable resale data collection.
- Scheduled/background source refreshes.
- Preferred-brand and model alerts for brands such as John Deere, Honda, Scott, Stihl, and other user-selected interests.
- Any credentialed marketplace access, login automation, seller messaging, automated bidding, automated buying, checkout, payment, or social posting workflow.

## Architecture Cleanup Backlog

- Split large Rust backend modules by domain boundary, starting with `commands.rs` and `db.rs`, so Tauri command routing, domain services, repositories, parsers, and platform window code are easier to review independently.
- Convert high-arity command and repository methods into typed request, draft, options, or parameter structs. Clippy currently flags several calendar, YouTube, screenshot, build guide, GearBlocks runtime, and catalog persistence methods for this pattern.
- Continue moving repeated frontend helpers into shared utilities or domain helpers when a behavior appears in more than one component.
- Add stricter lint gates only after the current broad Clippy findings have been reduced enough that the checks can run cleanly in normal development.

## GearBlocks Markers And GearLib

- In-game visual marker rendering.
- Chat-authored `overlay-forge-markers` response blocks.
- BepInEx plugin status UI in the active GearBlocks workspace.
- GearLib-based GearBlocks plugin work.
- Any user-facing requirement to install GearLib.

## GearBlocks Build Guide Construction Generation

- Generate a GearBlocks construction file from a build guide that contains all required parts as loose, unconnected parts.
- Load the generated construction into the world as a parts staging set so the needed parts are available before manual assembly.
- Keep placement, alignment, and connection work manual for the foreseeable future; this is not automated construction.
- Persist step-to-runtime-instance matching after staged build-guide exports have enough real data to validate matching rules for duplicate unpaintable parts. Initial matching signals should include part key/name, relative position from the first anchor, current unit size, paint target colour when present, source construction id, and optional friendly names.

## GearBlocks Part Preview Rendering

- Add targetless part-preview rendering after a stable GearBlocks/Unity source identity path is confirmed. Current BepInEx preview rendering can rotate and capture the live part under the camera center, and Overlay Forge can persist a validated render profile from that capture, but the plugin cannot yet re-find or spawn a part solely from a saved profile key without user targeting or a loaded live source object.
- Add an on-demand rotated preview cache for build-guide composition. Cache keys should include profile key, render profile version, camera preset, and composed part rotation so Overlay Forge renders only rotations that a guide actually needs.

## GearBlocks Runtime Scene Context

- Add a derived `obj_game_runtime_scene_summary` or similar scene-facts cache after normalized runtime imports if prompt assembly becomes expensive.
