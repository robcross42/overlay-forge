# Overlay Forge Project Plan

## Current Status

**Current project baseline: Milestone 13.**

Milestone 13 is the latest completed, tested, and user-validated milestone.

**Milestone 0 - Overlay Shell Validation is complete, passed, and successful.**

The app has proven a small, reliable desktop overlay shell before expansion into calendar, notes, project planning, OpenAI, GitHub, or YouTube workflows.

The Milestone 0 scratchpad component is also complete and passed. Scratchpad text persists locally in SQLite and restores between app sessions.

**Milestone 1 - Calendar, To-do, Notes, and Scratchpad Expansion is complete, passed, and successful.**

Milestone 1 adds component navigation for Scratchpad, Tasks, Notes, and Calendar, with local SQLite persistence for each data type.

**Milestone 2 - Local Projects component is complete, passed, and successful.**

Milestone 2 adds Projects navigation, a SQLite-backed local project table, Rust/Tauri project CRUD commands, and a Projects component using the same selected/read-only/edit interaction pattern as the organizer components.

**Milestone 3 - OpenAI Planning Chat component is complete, passed, and successful.**

Milestone 3 adds Planning Chat navigation, project-scoped planning conversations, SQLite-backed message persistence, and backend-only OpenAI Responses API calls through `OPENAI_API_KEY`. Milestone 6 later moves the primary chat entry point into Projects.

**Milestone 4 - GitHub Integration is complete, passed, and successful.**

Milestone 4 adds project-scoped GitHub repository linkage, SQLite-backed repository metadata/status, backend-only `GITHUB_TOKEN` handling, and a GitHub Repository section in Projects.

**Milestone 5 - Controlled YouTube Component**

Status: **Complete / Passed / Successful**

Milestone 5 adds YouTube navigation, local SQLite-backed YouTube reference persistence, backend URL validation and CRUD/open commands, and a controlled user-curated frontend component. It does not use a YouTube API key, YouTube account login, scraping, recommendations, transcripts, downloads, or account sync.

**Milestone 6 - Project Workspace Chat**

Status: **Complete / Passed / Successful**

Milestone 6 makes Projects the active workspace shell for project-scoped chat. Selecting a project exposes Overview, GitHub, and Chat sections; Chat uses the selected project automatically and preserves the existing planning conversation/message data.

**Milestone 7 - Project Workspace Layout Refinement**

Status: **Complete / Passed / Successful**

Milestone 7 refines Projects into a clearer selected-project workspace shell with Overview, GitHub, Chat, and References sections. References is intentionally minimal, project-local, and does not implement attachment workflows.

**Milestone 8 - Projects Navigation Tree Actions**

Status: **Complete / Passed / Successful**

Milestone 8 moves project create/select/edit/delete entry points into the left navigation shell. Projects is now an expandable navigation tree with saved projects as children, a compact `+` action on the Projects module row, and compact `...` menus on project rows for edit/delete. User validation is complete and Milestone 8 passed.

**Milestone 9 - Manual Context Attachments**

Status: **Complete / Passed / Successful**

Milestone 9 adds manual context attachments for selected project chat conversations. Attachments are conversation-scoped links to existing local app records and are visible in the Chat section's Attached Context area. Linked GitHub repository metadata is automatically added when a selected project has a repository defined in the GitHub section. User validation is complete and Milestone 9 passed.

**Milestone 10 - Prompt Preview**

Status: **Complete / Passed / Successful**

Milestone 10 adds a read-only Prompt Preview action inside selected-project Chat. It uses existing project, conversation, draft message, and attached context data to show an assembled prompt preview without calling OpenAI. Attached context inclusion in actual sends remains deferred.

**Milestone 11 - Bridge File Drafting**

Status: **Complete / Passed / Successful**

Milestone 11 adds local bridge-file draft generation from selected project chat conversations. Generated Markdown drafts are stored in SQLite, displayed read-only in the Chat section, and remain local until a later export or Codex handoff milestone. Resolved attached context, including linked GitHub repository metadata, is now used by bridge drafts and normal project chat sends.

**Milestone 12 - Project Markdown Context**

Status: **Complete / Passed / Successful**

Milestone 12 adds project-level local Markdown context. Project chat loads a fresh `README.md` from the configured local project root whenever a new chat starts or an existing chat loads, then includes referenced local Markdown files as project context sources for chat, Prompt Preview, and bridge drafts.

**Milestone 13 - Project Workspace UI Consolidation**

Status: **Complete / Passed / Successful**

Milestone 13 consolidates the Projects workspace UI around the left navigation hierarchy, a focused main chat surface, and a collapsible right-hand chat pane. Project conversations appear as child rows under projects, `New Chat` opens an empty new-conversation area from the project row menu, and project editing contains project details, GitHub integration, local Markdown context configuration, and local repo/context settings.

**Gaming Screenshot Capture**

Status: **Complete / Passed / Successful**

The Gaming screenshot feature is validated as a post-Milestone 13 feature addendum. GearBlocks is the initial game workspace, and the selected-game pane can capture the visible foreground game display while hiding Overlay Forge, save unique PNGs and capture manifests under `game-screenshots/`, persist screenshot metadata in SQLite, render thumbnail previews, and delete screenshots with related local capture metadata from the right-click screenshot menu.

**Overlay Forge 0.2.0 - GearBlocks Runtime API Interfaces**

Status: **Complete / Passed / Successful**

Overlay Forge 0.2.0 marks the documented `SmashHammer.GearBlocks.Construction` namespace reference interfaces as implemented for runtime export support, including `apiAttributes` getter snapshots, SQLite persistence of expanded runtime export payloads, catalog attribute availability display, and chat-context handoff of captured getter values.

## Product Direction

Overlay Forge is a personal desktop command hub that floats above the user's workflow and eventually helps turn ideas, notes, tasks, and project plans into Codex-ready markdown bridge files.

## Milestone Order

Use explicit milestone IDs. Do not infer milestone numbers from this list's item positions.

- Milestone 0 - Overlay shell validation - complete and passed
- Milestone 1 - Calendar, to-do, notes, and scratchpad component - complete and passed
- Milestone 2 - Local projects component - complete and passed
- Milestone 3 - OpenAI planning chat component - complete and passed
- Milestone 4 - GitHub integration - complete and passed
- Milestone 5 - Controlled YouTube component - complete and passed
- Milestone 6 - Project workspace chat - complete and passed
- Milestone 7 - Project workspace layout refinement - complete and passed
- Milestone 8 - Projects navigation tree actions - complete and passed
- Milestone 9 - Manual context attachments - complete and passed
- Milestone 10 - Prompt preview - complete, passed, successful
- Milestone 11 - Bridge file drafting - complete, passed, successful
- Milestone 12 - Project markdown context - complete, passed, successful
- Milestone 13 - Project workspace UI consolidation - complete, passed, successful

## Project (Chat/Bridge/GitHub) Deferred Items

Deferred items for the Project workspace (Planning Chat, Bridge Files, GitHub) are centralized here:

- `docs/PROJECT_DEFERRED_ITEMS.md`

## Scope Guard

Milestone 13 is the current passed stable baseline for later work. Do not implement later milestone features by reverting to an earlier code path; future work should begin from the completed overlay shell, hotkey behavior, always-on-top behavior, component host, local SQLite scratchpad, Tasks, Notes, Calendar, Projects navigation tree/workspace, focused Planning Chat persistence, manual context attachments, project Markdown context, Bridge File Drafting, GitHub Integration, YouTube components, and Project Workspace UI Consolidation.

Gaming Screenshot Capture is a completed, passed, and successful feature addendum on top of Milestone 13. Future Gaming work should preserve the validated screenshot folder layout, SQLite screenshot metadata path, thumbnail rendering through the Tauri asset protocol, overlay-hidden capture behavior, and right-click delete cleanup semantics.

Overlay Forge 0.2.0 GearBlocks runtime API interface support is a completed, passed, and successful feature addendum on top of Milestone 13. Future GearBlocks API work should preserve the runtime exporter/importer boundary, keep catalog views value-free, persist full captured getter values in runtime export records, and use `docs/GEARBLOCKS_RUNTIME_INTERFACES.md` as the first troubleshooting index for interface-specific issues.

Milestone 4 remains intentionally small. It does not include automatic Codex handoff, GitHub write operations, pull request creation, branch creation, issue management, full repository browsing, GitHub Actions integration, OAuth, multi-account support, advanced sync, vector store/repo indexing, YouTube integration, external calendar integration, cloud sync, or multi-user auth.

Milestone 5 remains intentionally controlled. It does not include YouTube account login, YouTube API integration, OAuth, subscription import, watch history import, recommendations, transcript retrieval, transcript summarization, video/audio downloads, embedded unrestricted browsing, playlist sync, comment sync, channel scraping, background metadata crawlers, bridge-file generation from videos, Codex handoff from videos, cloud sync, or multi-user auth.

Milestone 6 remains intentionally small. It does not include bridge-file generation, prompt preview, automatic context attachment, GitHub file reading, Codex handoff, ChatGPT import, conversation search/filtering, chat streaming, or model picker UI.

Milestone 7 remains intentionally layout-focused. It does not include manual context attachments, prompt preview, bridge-file generation, GitHub file browsing, Codex handoff, ChatGPT import, conversation search/filtering, chat streaming, model picker UI, AI-generated project summaries, or advanced project dashboard analytics.

Milestone 8 remains intentionally focused on Projects navigation only. It does not refactor Tasks, Notes, Calendar, or YouTube until the Projects navigation tree pattern is validated.

Milestone 9 remains intentionally focused on manual attachment links only. It does not implement automatic context attachment, semantic search, vector stores, file uploads, GitHub file reading, YouTube transcript extraction, prompt preview, token counting, bridge-file generation, Codex handoff, ChatGPT import, chat streaming, or model picker UI.

Milestone 10 remains intentionally focused on read-only prompt visibility only. It does not implement bridge-file generation, bridge-file editing/export, Codex handoff, GitHub file reading, YouTube transcript extraction, semantic search, vector stores, file uploads, automatic context attachment, token counting/budgeting, model picker UI, chat streaming, ChatGPT import, automatic prompt rewriting, long-term prompt templates, or attached context inclusion in actual OpenAI sends.

Milestone 11 remains intentionally focused on local bridge draft creation and attached-context inclusion for project chat sends. It does not implement full bridge-file editing, approval/obsolete workflows, export to local Markdown files, copy-to-clipboard, direct Codex handoff, GitHub writes, chat streaming, model picker UI, token budgeting, vector stores, semantic search, or ChatGPT import.

Milestone 12 is a local-first project Markdown context milestone. It loads `README.md` and referenced local Markdown files from a configured project root as project-level context, not as per-conversation manual attachments. It does not read arbitrary filesystem paths outside the project root, read GitHub files through the GitHub API, add vector stores, add semantic search, export bridge files, or hand off directly to Codex.

Milestone 13 is a UI consolidation milestone. It removes redundant Projects workspace framing, moves project configuration into a clean Project Edit screen, shows conversations in the left navigation hierarchy, moves attached context and local Markdown bridge drafts into a collapsible right-hand pane, and keeps the main chat surface focused. It does not change context data models, convert manual conversation attachments into project-level records, add GitHub file reading, export bridge files, or hand off directly to Codex.
