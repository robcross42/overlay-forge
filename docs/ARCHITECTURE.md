# Architecture

## Shell

The overlay shell owns the top-level layout, navigation, and component host. Feature modules should render inside the host instead of controlling window behavior directly.

## Frontend

The frontend is a React + TypeScript app organized around feature folders:

```text
src/
├─ app/
├─ components/
├─ features/
├─ services/
├─ styles/
└─ main.tsx
```

Milestone 0 is complete and includes the shell and scratchpad feature. The scratchpad feature passed Milestone 0 validation by saving content to SQLite and restoring it between app sessions.

Milestone 1 is complete, passed, and successful. It adds feature folders for Tasks, Notes, and Calendar while keeping all feature modules inside the shell-owned component host.

Milestone 2 is complete, passed, and successful. It adds a Projects feature folder while preserving the shell-owned component host and Milestone 1 organizer components.

Milestone 3 is complete, passed, and successful. It adds a Planning Chat feature folder inside the shell-owned component host while preserving Scratchpad, Tasks, Notes, Calendar, and Projects.

Milestone 4 - GitHub Integration is complete, passed, and successful. It extends the Projects feature with a project-scoped GitHub Repository section while preserving the existing shell-owned component host and all Milestone 0 through Milestone 3 components.

Milestone 5 - Controlled YouTube Component is complete, passed, and successful. It adds a YouTube feature folder inside the shell-owned component host while preserving Scratchpad, Tasks, Notes, Calendar, Projects, Planning Chat, and GitHub Repository behavior.

Milestone 6 - Project Workspace Chat is complete, passed, and successful. It makes Projects the workspace shell for selected-project planning chat by rendering Overview, GitHub, and Chat sections inside the selected project context. The existing Planning Chat feature is reused in project-bound mode so chat no longer needs a second project selector.

Milestone 7 - Project Workspace Layout Refinement is complete, passed, and successful. It refines Projects as the primary workspace shell by adding a stable active-project header and four selected-project workspace sections: Overview, GitHub, Chat, and References.

Milestone 8 - Projects Navigation Tree Actions is complete, passed, and successful. It moves Projects object-level actions toward the shell navigation by making Projects expandable in the left navigation and listing saved projects as children. This pattern was validated on Projects before generalizing it to other modules.

Milestone 9 - Manual Context Attachments is complete, passed, and successful. It adds a conversation-scoped Attached Context area inside selected-project Chat. Attachments link to existing local app records and store only the link metadata needed to display those attachments.

Milestone 10 - Prompt Preview is complete, passed, and successful. It adds a read-only Prompt Preview surface inside selected-project Chat and a backend preview command that assembles local preview data without calling OpenAI.

Milestone 11 - Bridge File Drafting is complete, passed, and successful. It adds a read-only bridge draft generation surface inside selected-project Chat and backend commands that store generated Markdown drafts locally in SQLite.

Milestone 12 - Project Markdown Context is complete, passed, and successful. It adds a project-scoped local Markdown context configuration so README-driven project documentation can feed project chat and bridge draft generation without per-conversation attachment.

Milestone 13 - Project Workspace UI Consolidation is complete, passed, and successful. It removes redundant project workspace framing, moves project configuration into a clean Project Edit surface, shows conversations in the left navigation hierarchy, keeps the primary chat surface focused, and moves conversation attached context plus local Markdown bridge drafts into a collapsible right-hand chat pane.

Gaming Screenshot Capture is complete, passed, and successful. It adds a Gaming feature surface with a GearBlocks workspace, selected-game screenshot capture controls, thumbnail previews, and screenshot context-menu cleanup while preserving the shell-owned component host.

Overlay Forge 0.2.0 GearBlocks runtime API interface support is complete, passed, and successful. It adds documented construction namespace interface descriptors, Lua exporter getter snapshots, runtime-log import normalization, SQLite persistence of expanded getter data, and catalog display of attribute availability.

## UI Consistency

Organizer components should follow the same interaction pattern unless a milestone explicitly documents a reason to diverge:

- Empty components show the primary New action and keep editor fields hidden.
- New actions reveal the editor for the first item.
- Selecting an existing list item opens that item in selected/read-only mode.
- Selected existing items expose an explicit Edit action before fields become editable.
- Destructive actions are available only inside an edit/selected-item context.
- Active clickable actions use consistent enabled button styling across components.

Milestone 8 adds navigation action consistency rules:

- Module-level `+` actions create new items for that module.
- Item-level `...` actions open item menus such as Edit and Delete.
- Hover-revealed actions must also be visible or reachable by keyboard focus.
- The workspace surface should prioritize selected item content while navigation owns object-level actions.

Milestone 8 applies these rules to Projects only. The Projects row is expandable/collapsible, the row-level `+` starts project creation, and project child rows can select the active workspace project or open an item menu for Edit/Delete. The main panel remains the selected-project workspace with Overview, GitHub, Chat, and References sections.

Milestone 13 consolidates these rules further for Projects. Conversation rows appear under project rows in the left navigation hierarchy, project row menus route to Overview, New Chat, References, Edit, and Delete, and the main chat surface removes redundant workspace headers so message history and input receive most of the available space. Existing conversations open from their left-nav child rows; `New Chat` opens an empty new-conversation area.

## Backend

The Tauri backend owns:

- SQLite initialization
- Scratchpad persistence commands
- Task CRUD commands
- Note CRUD commands
- Calendar event CRUD commands
- Project CRUD commands
- Planning conversation and message CRUD commands
- Backend-only OpenAI Responses API request handling
- Project-scoped GitHub repository link commands
- Backend-only GitHub repository metadata fetch handling
- YouTube reference CRUD commands
- YouTube URL validation and external-open handling
- Planning conversation context attachment commands
- Planning prompt preview command
- Bridge file draft commands
- Project Markdown context configuration and loading commands
- Gaming and screenshot capture commands
- Global hotkey registration
- Window show/hide behavior

## Persistence

SQLite is the local source of truth. The first schema contains a single-row `scratchpad` table. This Milestone 0 scratchpad persistence path is complete and passed.

Milestone 1 adds idempotent table initialization for `tasks`, `notes`, and `calendar_events`.

Milestone 2 adds idempotent table initialization for `projects`.

Milestone 3 adds idempotent table initialization for `planning_conversations` and `planning_messages`. Later milestones should add tables for bridge file drafts and exported bridge-file workflow state.

Milestone 4 adds idempotent table initialization for `project_github_repositories`. The table stores project repository linkage and fetched metadata/status only. Migrations are non-destructive and must not remove existing Scratchpad, Tasks, Notes, Calendar, Projects, or Planning Chat data.

Milestone 5 adds idempotent table initialization for `youtube_references`. The table stores only user-created YouTube references and user-entered metadata. Migrations are non-destructive and must not remove existing Scratchpad, Tasks, Notes, Calendar, Projects, Planning Chat, or GitHub repository data.

Milestone 6 adds no new tables. It preserves existing `planning_conversations` and `planning_messages` records and scopes the frontend Chat section through the selected Projects workspace.

Milestone 7 adds no new tables. Overview continues to use `projects`, GitHub continues to use `project_github_repositories`, Chat continues to use `planning_conversations` and `planning_messages`, and References only summarizes existing local context categories.

Milestone 8 adds no new tables. The shell-owned Projects navigation tree reads existing `projects` rows and continues to use the same project CRUD commands. Chat and GitHub behavior continue to use the existing selected-project data paths.

Milestone 9 adds idempotent table initialization for `planning_conversation_context`. Attachments are scoped to a single planning conversation, link to existing local records by `context_type` and `source_id`, and store a readable label. Removing an attachment deletes only the attachment link and does not delete the source record.

Milestone 10 adds no new tables. Prompt Preview uses existing project, planning conversation, planning message, and context attachment data.

Milestone 11 adds idempotent table initialization for `bridge_file_drafts`. Drafts are project-scoped, may link to a source planning conversation, and store generated Markdown content locally. Migrations are non-destructive and must not remove existing user data.

Milestone 12 adds idempotent table initialization for `project_markdown_context`. Each row stores one configured local Markdown root per project. Markdown file content is read freshly from disk for chat load, Prompt Preview, project chat sends, and bridge draft generation; file snapshots are not cached in SQLite.

Gaming adds idempotent table initialization for `games`, `game_data_locations`, `game_catalog_objects`, `game_catalog_references`, and `game_catalog_screenshots`. `game_data_locations` stores game-scoped local save or alternate data directories for games that expose the feature, with GearBlocks currently exposing the controls from its selected-game Home screen. Screenshot image bytes are stored as PNG files under `game-screenshots/`, while SQLite stores metadata and local paths only. The screenshot preview path uses Tauri asset loading scoped to `game-screenshots/`.

GearBlocks construction decoding is local-first and file-based. GearBlocks' default user data location is `%USERPROFILE%\AppData\LocalLow\SmashHammer Games\GearBlocks\`. Overlay Forge discovers `construction.bytes` files from the configured GearBlocks Save Location or that default root's `SavedConstructions` folder, inflates the raw DEFLATE payload, parses the BSON document, and renders a JSON-friendly summary on the GearBlocks Home screen. Runtime-only metadata such as display names, categories, mass, active stage, and documented construction namespace getter snapshots is handled through an installable GearBlocks Lua script mod under the default root's `ScriptMods` folder that exports loaded construction metadata through marked `Player.log` JSON chunks when direct Lua file writes are unavailable. GearBlocks chat automatically converts the latest runtime export into a semantic vehicle-build summary that aggregates welded structural pieces and lists functional systems with inferred purposes.

## OpenAI Boundary

Planning Chat calls the OpenAI Responses API from the Rust/Tauri backend. React invokes local Tauri commands only and never reads `OPENAI_API_KEY`. Model selection, request shape, and the planning assistant instruction are centralized in the backend OpenAI service module so later bridge-file generation, tools, streaming, or model changes do not leak through the frontend.

In Milestone 6, the primary UI path for Planning Chat is Projects -> selected project -> Chat. The selected project is passed directly into the chat surface, while the backend continues to enforce conversation ownership through `planning_conversations.project_id`.

Milestone 7 preserves this boundary. Chat remains a selected-project workspace section and remains backed by the existing planning conversation/message tables.

## GitHub Boundary

Milestone 4 GitHub metadata fetches are backend-owned. React invokes local Tauri commands and never receives `GITHUB_TOKEN`. The token is read from the Rust process environment, is not stored in SQLite, and is not passed into frontend state.

SQLite stores repository linkage and fetched metadata/status only:

```text
project_id
repository_full_name
repository_url
default_branch
visibility
last_fetched_at
last_fetch_status
```

The integration is project-scoped and read-only. Milestone 4 does not perform Codex handoff, GitHub write operations, branch creation, commit creation, pull request creation, issue management, repository file browsing, GitHub Actions integration, OAuth, or multi-account workflows.

In Milestone 7, GitHub remains a selected-project workspace section backed by the existing project GitHub repository table. No token exposure or GitHub write behavior is added.

Milestone 8 preserves this boundary. Project selection can now happen from the left navigation tree, but GitHub repository linkage and metadata fetches still happen inside the selected-project GitHub workspace section.

## Context Attachment Boundary

Milestone 9 context attachments are manual and conversation-scoped. The user chooses existing local app context from the selected project Chat section. Supported source types are project details, project GitHub repository metadata, notes, tasks, calendar events, YouTube references, and scratchpad content.

GitHub repository context is the only Milestone 9 automatic attachment path: when a selected project has a repository link defined in the GitHub section, the Chat section adds that repository metadata link to the selected conversation's Attached Context list with a duplicate guard. The repository link is still configured once per project in the GitHub workspace section.

The attachment layer stores links only. It does not count tokens, read GitHub files, fetch YouTube transcripts, export bridge files, or send context outside the local project chat request path. Milestone 10 adds read-only Prompt Preview for these links. Milestone 11 includes resolved attached context in normal project chat sends and bridge draft generation.

Projects remains the primary workspace shell, and the Projects navigation tree remains unchanged from Milestone 8.

## Prompt Preview Boundary

Milestone 10 Prompt Preview is scoped to selected-project Chat. It uses existing local project, conversation, message, and context attachment data to show a read-only preview of the intended prompt/context package.

Prompt Preview must not call OpenAI. It does not send messages, mutate chat history, generate bridge files, count tokens, rewrite prompts, or change the model request path.

In Milestone 10, attached context appears in the preview only. Milestone 11 extends the project chat send path so resolved attached context is included with the selected project and recent conversation messages.

Projects remains the primary workspace shell.

## Bridge Draft Boundary

Milestone 11 bridge drafts are local SQLite records generated from selected-project Chat data. The generator uses the selected project, source planning conversation, saved conversation messages, and resolved attached context where safely available. Linked GitHub repository metadata is resolved from the selected project if the attachment row is stale or missing a source id.

Bridge drafts are project-scoped through `project_id` and may link to the source conversation through `conversation_id`. Deleting a bridge draft removes only the draft row; it does not delete the project, conversation, messages, or attached context.

Milestone 11 does not export Markdown files, copy drafts to the clipboard, open Codex, send content to Codex, write to GitHub, create commits, create pull requests, or approve generated drafts. User review remains required before a draft is used outside Overlay Forge.

Milestone 12 extends bridge draft generation by adding project Markdown context before conversation manual attachments. Drafts may include local README-driven project documentation, but they remain local SQLite records and still require user review.

## Project Markdown Context Boundary

Milestone 12 project Markdown context is project-scoped. A selected project can store a configured local documentation root and README path in SQLite. The backend reads a fresh copy of `README.md`, known local documentation paths, and explicit Markdown references found in README whenever the context is loaded.

Markdown resolution is constrained to the configured local project root. Unsafe path traversal, absolute paths, external URLs, missing files, unreadable files, non-Markdown files, and files that resolve outside the root are skipped or warned about instead of crashing the app.

Project Markdown context is assembled before conversation manual attachments for project chat sends, Prompt Preview, and bridge draft generation. Manual attachments remain conversation-scoped and continue to act as an additional context layer.

Milestone 12 does not read GitHub repository file contents through the GitHub API, upload files, add vector stores, add semantic search, broadly index repositories, export bridge files, or hand off directly to Codex.

## Project Workspace UI Boundary

Milestone 13 is a frontend workspace consolidation. It does not add tables or change context ownership. Project Markdown context remains project-scoped. Manual context attachments remain conversation-scoped through `planning_conversation_context`.

GitHub repository linkage and project Markdown context configuration now live in Project Edit instead of occupying the primary chat path. The focused chat surface keeps message history and message input close to the selected conversation while moving manual attachment controls and local Markdown bridge drafts into a collapsible right-hand pane.

## YouTube Boundary

Milestone 5 YouTube references are local-first and user-curated. React invokes local Tauri commands to save, list, edit, delete, and open references. SQLite stores the title, URL, parsed video id, optional channel name, notes, tags, and timestamps.

No YouTube API key is required. No YouTube account login, OAuth flow, watch history, subscription import, playlist sync, comment sync, transcript extraction, recommendations, downloads, scraping, background metadata crawler, or account sync is used.

Saved YouTube URLs open externally in the system browser. This is preferred over an unrestricted embedded browser so the overlay workflow remains controlled.

## Gaming Screenshot Boundary

Gaming Screenshot Capture is local-first and user-initiated. React invokes local Tauri commands to list games, create/delete game sections, capture screenshots, list screenshot metadata, and delete screenshot records/files.

The validated capture path hides Overlay Forge before capture, captures the visible foreground game display through Windows GDI for the current implementation, forces PNG alpha values to 255, saves unique PNG files under `game-screenshots/<game-slug>/`, writes capture manifests under `game-screenshots/capture-requests/`, and then restores the overlay.

The webview may render screenshot thumbnails only through the Tauri asset protocol scoped to `game-screenshots/`. Deletion is constrained to that folder and removes the PNG, capture manifest, screenshot metadata row, and local-path reference rows that point at either deleted file.

The preferred future GearBlocks path remains game-internal rendered-frame export from the game engine. Clipboard capture, `Win+Shift+S`, Snipping Tool dependency, HDR output, wide-gamut output, and alpha-dependent image files remain avoided for the long-term capture target.

## References Boundary

Milestone 7 References are intentionally minimal. The References workspace section summarizes selected project details, linked GitHub metadata, future attachment availability, and future prompt context availability. It does not attach context to chat, generate prompt previews, browse GitHub files, generate bridge files, or include unrelated app-level YouTube library data.

## Bridge Files

Bridge files are markdown documents used to keep ChatGPT and Codex aligned while the in-app OpenAI workflow is deferred.

Milestone 11 introduces local bridge-file drafts as SQLite records. These drafts are generated from selected-project Chat and remain in-app for review. They are not exported to disk and are not sent to Codex automatically.
