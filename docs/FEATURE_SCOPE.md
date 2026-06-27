# Overlay Forge Feature Scope

## Global Scope Guard

Future work should preserve the completed local-first desktop overlay foundation:

- Shell-owned top-level layout and component host.
- Hotkey behavior.
- Always-on-top behavior.
- SQLite persistence.
- Scratchpad, Tasks, Notes, and Calendar organizer behavior.
- Projects navigation tree and focused project chat behavior.
- Backend-owned OpenAI and GitHub token handling.
- User-curated YouTube references.
- Gaming workspace and screenshot metadata path.
- GearBlocks save/runtime data boundaries.
- Smoking Cessation local SQLite ownership.
- Scheduler bounded Rust-handler dispatch.
- SQLite naming conventions.

## Active Project Workspace Scope

Projects are the primary workspace shell.

Current scope:

- Project rows appear in the left navigation tree.
- Conversation rows appear under project rows.
- Project row menu owns Overview, New Chat, References, Edit, and Delete.
- Project Edit owns project metadata, GitHub repository linkage, and Markdown context configuration.
- Focused chat surface owns message history, message input, and Send.
- Right-hand chat pane owns context references and local implementation request drafts.
- Manual context attachments remain conversation-scoped.
- Project Markdown context remains project-scoped.

Do not move conversation-scoped context into project-scoped storage unless a later explicit data-model change requests it.

## OpenAI Boundary

- OpenAI calls are backend-owned.
- React invokes local Tauri commands only.
- React must not read `OPENAI_API_KEY`.
- Model selection, request shape, and assistant instructions should remain centralized in backend OpenAI service code unless a requested model-picker feature changes that boundary.
- Prompt Preview must remain read-only and must not call OpenAI.

## GitHub Boundary

- GitHub token use is backend-owned.
- React must not receive or store `GITHUB_TOKEN`.
- SQLite stores repository linkage and fetched metadata only.
- Current GitHub integration does not read repository files, create branches, create commits, create issues, create pull requests, or perform OAuth.

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

## GearBlocks Boundary

GearBlocks support is local-first and should use the safest available data path for the requested task.

Current data layers:

1. Saved construction decoding from `construction.bytes`.
2. Runtime scene exports reconstructed from GearBlocks script log output.
3. Prompt-time rich full-scene export requests before GearBlocks chat context assembly.
4. SQLite runtime part, property, attachment, API availability, and catalog indexes.
5. Backlog direct BepInEx plugin work for user-controlled temporary visual markers.

GearBlocks runtime API metadata is availability-first by default. Do not invoke getter-heavy or mutating API paths unless the user explicitly asks for a snapshot/control feature.

GearBlocks chat should not request in-game marker placement or emit `overlay-forge-markers` blocks while visual marker support is paused.

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

## Scheduler Boundary

Scheduler jobs must be backend-owned and bounded.

Rules:

- Scheduler rows point to static scheduler type definitions.
- Static scheduler keys map to known Rust handlers.
- SQLite scheduler rows must not execute arbitrary commands, scripts, Lua payloads, or shell commands.
- Jobs should record run status in `obj_scheduler_run`.
- Jobs should avoid blocking the UI or long-running synchronous work.

## Persistence Boundary

SQLite is the source of truth for persisted app data.

Rules:

- Use non-destructive, idempotent migrations.
- Preserve existing user data.
- Use current naming conventions: `obj_`, `def_`, `o2o_`, `n2n_`.
- Avoid table-per-game setting tables. Prefer `obj_game_setting` or normalized feature tables keyed by `game_id` and `id_game`.
- Generated screenshots and local image files are stored outside SQLite; SQLite stores metadata and paths.
