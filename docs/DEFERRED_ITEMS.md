# Overlay Forge Deferred Items

This file centralizes deferred work. Items listed here are not approved scope unless the user explicitly requests them.

## Project Chat UX And Controls

- Chat message streaming.
- Model picker UI.
- Conversation search/filtering.
- Advanced project dashboard analytics.
- AI-generated project summaries.
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
- Direct provider deep links beyond user-owned manual links.
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

## Architecture Cleanup Backlog

- Split large Rust backend modules by domain boundary, starting with `commands.rs` and `db.rs`, so Tauri command routing, domain services, repositories, parsers, and platform window code are easier to review independently.
- Convert high-arity command and repository methods into typed request, draft, options, or parameter structs. Clippy currently flags several calendar, YouTube, screenshot, build guide, GearBlocks runtime, and catalog persistence methods for this pattern.
- Continue moving repeated frontend helpers into shared utilities or domain helpers when a behavior appears in more than one component.
- Add stricter lint gates only after the current broad Clippy findings have been reduced enough that the checks can run cleanly in normal development.

## GearBlocks Markers And Plugins

- In-game visual marker rendering.
- Chat-authored `overlay-forge-markers` response blocks.
- BepInEx plugin status UI in the active GearBlocks workspace.
- GearLib-based GearBlocks plugin work.
- Any user-facing requirement to install BepInEx or GearLib.

## GearBlocks Build Guide Construction Generation

- Generate a GearBlocks construction file from a build guide that contains all required parts as loose, unconnected parts.
- Load the generated construction into the world as a parts staging set so the needed parts are available before manual assembly.
- Keep placement, alignment, and connection work manual for the foreseeable future; this is not automated construction.
