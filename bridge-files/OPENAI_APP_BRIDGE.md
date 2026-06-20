# Overlay Forge - OpenAI App Bridge

## Purpose

This file is the manual bridge between Overlay Forge project state and ChatGPT/Codex conversations.

Provide this file to ChatGPT or Codex when a new chat needs current project context. Keep it concise and update it after meaningful architecture, milestone, or validation changes.

Future changelog and versioning updates must follow `AGENTS.md`. `docs/PROJECT_RULES.md` is retained as a compatibility pointer to that root instruction file.

## Current Milestone

Milestone 13 - Project Workspace UI Consolidation

Status: **Complete / Passed / Successful**

Current user-validated project baseline: **Milestone 13**. Milestone 13 is complete, passed, and successful.

Milestone 0 remains complete, passed, and successful. Milestone 1 adds component navigation plus SQLite-backed Tasks, Notes, and Calendar components. User validation is complete and Milestone 1 passed.

Milestone 2 adds the local Projects component with SQLite persistence. User validation is complete and Milestone 2 passed.

Milestone 3 adds Planning Chat with backend OpenAI Responses API calls and local SQLite conversation/message persistence. User validation is complete and Milestone 3 passed.

Milestone 4 adds project-scoped GitHub repository linkage, SQLite-backed repository metadata/status, backend-only `GITHUB_TOKEN` handling, and a GitHub Repository section inside Projects. User validation is complete and Milestone 4 passed.

Milestone 5 adds controlled user-curated YouTube references, SQLite-backed reference persistence, backend YouTube URL validation, frontend YouTube reference CRUD UI, and external opening of saved URLs. User validation is complete and Milestone 5 passed.

Milestone 6 moves project-scoped chat into the selected Projects workspace. Selecting a project now exposes Overview, GitHub, and Chat sections. The Chat section uses the selected project automatically, preserves existing planning conversation/message persistence, and does not require a separate project selector.
User validation is complete and Milestone 6 passed.

Milestone 7 refines the selected Projects workspace layout. The active project workspace now has a clear project header and Overview, GitHub, Chat, and References sections. References is intentionally minimal and summarizes project-local context categories without implementing manual attachments. YouTube remains a separate app-level component and is not part of the Projects References section.
User validation is complete and Milestone 7 passed.

Milestone 8 is complete, passed, and successful as Projects Navigation Tree Actions. It makes Projects expandable in the left navigation, lists saved projects as children, exposes a compact `+` for new project flow, and exposes compact `...` menus on project rows for edit/delete. This pattern was validated on Projects before applying the pattern to other modules.

Milestone 9 is complete, passed, and successful. It adds manual context attachments for selected project chat conversations. Attachments are stored as conversation-scoped SQLite links in `planning_conversation_context`, can point to project, note, task, calendar event, YouTube reference, or scratchpad context, and can be removed without deleting source records. Linked GitHub repository metadata is automatically attached when a selected project has a repository defined in the GitHub section. Prompt Preview is complete in Milestone 10; attached context inclusion in project chat sends is implemented in Milestone 11.

Milestone 10 is complete, passed, and successful. It adds read-only Prompt Preview inside selected-project Chat. The preview is assembled by the backend from existing local project, conversation, draft message, and attached context data without calling OpenAI. Actual OpenAI sends remain unchanged in Milestone 10; Milestone 11 adds attached context inclusion for project chat sends.

Milestone 11 is complete, passed, and successful. It adds local bridge-file draft generation from selected project chat conversations. Drafts are stored in SQLite as `bridge_file_drafts`, displayed read-only in the Chat section, and remain local. Resolved attached context, including linked GitHub repository metadata, is used by bridge drafts and normal project chat sends. Export, full editing, approval workflow, GitHub writes, and direct Codex handoff remain deferred.

Milestone 12 is complete, passed, and successful. It adds project-level local Markdown context by loading a fresh `README.md` from a configured local project root whenever a project chat starts or loads, then resolving referenced Markdown files inside that project root for chat and bridge drafts. This is project-scoped, not per-conversation attachment.

Milestone 13 is complete, passed, and successful. It consolidates the Projects workspace UI by moving project/conversation navigation into the left hierarchy, moving GitHub and project Markdown configuration into Project Edit, giving the selected chat conversation most of the main panel, and placing conversation attached context plus local Markdown bridge drafts in a collapsible right-hand chat pane. The project row action is `New Chat`; existing chats open from conversation child rows in the left navigation.

Gaming Screenshot Capture is complete, passed, and successful as a post-Milestone 13 feature addendum. It adds a Gaming workspace with GearBlocks, overlay-hidden screenshot capture, unique PNG and capture manifest output under gitignored `game-screenshots/`, SQLite screenshot metadata, Tauri asset-backed thumbnail previews, and right-click screenshot deletion cleanup. GearBlocks also exposes selected-game Home controls for Save Location and Alternate Data Location records stored locally in SQLite. The GearBlocks Home screen can decode local `construction.bytes` saves by inflating raw DEFLATE and parsing BSON into a JSON summary, and can install a GearBlocks Lua script mod for runtime construction metadata export.

Overlay Forge 0.2.0 is complete, passed, and successful as a GearBlocks runtime API interface release. It implements support structures and exporter wiring for the documented `SmashHammer.GearBlocks.Construction` namespace reference interfaces. Runtime imports create availability-only `apiAttributes`, catalog part details show API attribute availability without values, and DB definitions / runtime construction export context retain indexed interface metadata. API metadata is not included in default game-chat prompt context unless a future explicit include control is added. Future issues with specific interface metadata should start from `docs/GEARBLOCKS_RUNTIME_INTERFACES.md`.

Milestone numbering note: Milestone 2 is the Local Projects component. Milestone 3 is the OpenAI Planning Chat component. Milestone 4 is GitHub Integration. Milestone 5 is the Controlled YouTube Component. Milestone 6 is Project Workspace Chat. Milestone 7 is Project Workspace Layout Refinement. Milestone 8 is Projects Navigation Tree Actions and is complete, passed, and successful. Milestone 9 is Manual Context Attachments and is complete, passed, and successful. Milestone 10 is Prompt Preview and is complete, passed, and successful. Milestone 11 is Bridge File Drafting and is complete, passed, and successful. Milestone 12 is Project Markdown Context and is complete, passed, and successful. Milestone 13 is Project Workspace UI Consolidation and is complete, passed, and successful. Do not mistake roadmap list item positions for milestone IDs.

## Current Scope

- Tauri v2 desktop app
- React + TypeScript frontend
- Rust backend
- Always-on-top overlay window
- Global hotkey toggle
- Custom draggable titlebar
- Custom minimize, maximize/restore, and hide controls
- Edge and corner resize handles for borderless overlay resizing
- Placeholder component host
- SQLite-backed scratchpad persistence
- SQLite-backed task persistence
- SQLite-backed note persistence
- SQLite-backed calendar event persistence
- SQLite-backed local project persistence
- SQLite-backed planning conversation persistence
- SQLite-backed planning message persistence
- Backend-only OpenAI API calls through `OPENAI_API_KEY`
- SQLite-backed project GitHub repository linkage
- Backend-only GitHub metadata fetches through `GITHUB_TOKEN`
- SQLite-backed controlled YouTube reference persistence
- Project workspace Chat inside selected Projects context
- Selected-project workspace sections for Overview, GitHub, Chat, and References
- SQLite-backed manual context attachments for project chat conversations
- Read-only Prompt Preview for project chat conversations
- SQLite-backed local bridge-file drafts for project chat conversations
- Project-level local Markdown context from README and referenced Markdown files
- Implemented Project Workspace UI Consolidation for focused chat layout
- Gaming workspace with GearBlocks as the initial game section
- GearBlocks selected-game Home controls for local Save Location and Alternate Data Location folders
- GearBlocks selected-game Home construction decoder for local `construction.bytes` saves and runtime Lua exporter installation
- GearBlocks runtime API availability metadata for the documented `SmashHammer.GearBlocks.Construction` namespace interfaces
- Validated Gaming screenshot capture, thumbnail preview, context menu, and delete cleanup
- Simple Gaming chat overlay screenshot capture that reuses the regular game screenshot flow and attaches the screenshot to the current prompt
- Centralized Project deferred items in `docs/PROJECT_DEFERRED_ITEMS.md`

## Hotkeys

```text
Ctrl+Shift+Space - toggle Overlay Forge visibility
Ctrl+Shift+C - open or refocus the selected Gaming chat overlay
```

These shortcuts are configurable in Settings -> Keybinds. Each function uses `key1`, `key2`, and `key3` as the ordered parts of one shortcut. Mouse buttons can be used as shortcut parts, including `Mouse4`, `Mouse5`, and modifier combinations such as `Ctrl+Mouse4`.

## Deferred

Project-specific deferred items for Planning Chat, Bridge Files, and GitHub integration are centralized in `docs/PROJECT_DEFERRED_ITEMS.md`.

- YouTube account login, YouTube API integration, scraping, transcripts, recommendations, downloads, and account sync
- Advanced calendar workflows
- Advanced task workflows
- Advanced notes workflows
- Bridge-file export UI
- Full bridge-file editor
- Bridge-file approval workflow
- Bridge-file prompt export
- Automatic context attachment
- Automatic context assembly from manual attachments
- GitHub file reading
- Local arbitrary filesystem reads outside configured project root
- ChatGPT import
- AI-generated project summaries
- Advanced project dashboard analytics
- Exclusive fullscreen game overlay support
- File upload/vector store workflows
- Web search tooling
- Automatic Codex handoff
- GitHub write operations
- Pull request, branch, commit, issue, Actions, webhook, OAuth, and multi-account workflows

## Latest Validation Notes

Update this section manually after each validation pass.

- Complete: dependency install
- Complete: frontend build with `npm run build`
- Complete: Rust dev compile with `cargo build`
- Complete: production Tauri bundle with `npm run tauri:build`
- Complete: compiled app launched briefly
- Complete: SQLite database created automatically at `%APPDATA%\com.overlayforge.desktop\overlay-forge.sqlite3`
- Complete: scratchpad persistence manually validated by user
- Complete: Gaming Screenshot Capture validated successfully for capture, preview, and delete cleanup
- Complete: Overlay Forge 0.2.0 GearBlocks runtime API interface inclusion validated in-game on Rob's vehicle
- Complete: custom draggable titlebar and window controls build successfully
- Complete: edge/corner resize handle APIs build successfully
- Complete: fixed titlebar drag event interception so window control clicks can fire
- Complete: added explicit Tauri permissions for window control APIs
- Complete: rebuilt release app after window-control click fix
- Passed: Milestone 0 overlay shell validation
- Passed: Milestone 0 scratchpad component validation
- Passed: scratchpad save and restore between sessions
- Implemented: Milestone 1 component navigation
- Implemented: Tasks component and SQLite CRUD commands
- Implemented: Notes component and SQLite CRUD commands
- Implemented: Calendar component and SQLite CRUD commands
- Implemented: hotkey registration guard so app startup survives an already-registered hotkey
- Implemented: task body and deadline fields
- Implemented: task list selection/edit mode with no main-list checkbox or delete button
- Implemented: Notes empty state hides editor controls until a note exists
- Implemented: Calendar date/time field click behavior and automatic end date/time updates
- Implemented: Delete buttons visible only in edit/selected contexts with active styling
- Implemented: Tasks and Calendar empty states now match Notes editor reveal behavior
- Implemented: selected existing tasks show an explicit Edit button before fields become editable
- Implemented: selected existing notes and calendar events show an explicit Edit button before fields become editable
- Documented: organizer components should follow consistent empty-state/edit-mode/destructive-action patterns
- Implemented: overlay starts hidden in the background and is shown with `Ctrl+Shift+Space`
- Implemented: shutdown titlebar control exits the app process
- Deferred cleanup: investigate benign Windows WebView2 shutdown log `Failed to unregister class Chrome_WidgetWin_0. Error = 1412`
- Verified: `npm run tauri:build` passes after Milestone 1 implementation
- Verified: rebuilt release app starts against the existing app-data SQLite database
- Passed: Milestone 1 manual validation checklist
- Implemented: Milestone 2 Projects navigation entry
- Implemented: local Projects component with create, select, read-only view, edit, delete, status, and list behavior
- Implemented: SQLite `projects` table and project CRUD commands
- Verified: `npm run build` passes after Milestone 2 implementation
- Verified: `cargo build` passes after Milestone 2 implementation
- Verified: `npm run tauri:build` passes after Milestone 2 implementation
- Verified: rebuilt release app starts hidden in the background after Milestone 2 implementation
- Fixed: Projects status dropdown option readability
- Passed: Milestone 2 manual validation checklist
- Implemented: Milestone 3 Planning Chat navigation entry
- Implemented: local project selector for planning chat context
- Implemented: SQLite `planning_conversations` and `planning_messages` tables
- Implemented: backend OpenAI Responses API service using `OPENAI_API_KEY`
- Implemented: conversation create/list/delete and message list/send commands
- Implemented: readable missing `OPENAI_API_KEY` error path
- Verified: `npm install` passes after Milestone 3 implementation
- Verified: `npm run build` passes after Milestone 3 implementation
- Verified: `cargo build` passes after Milestone 3 implementation
- Passed: Milestone 3 manual validation checklist
- Fixed: scrollable component lists show scrollbars when content exceeds visible space
- Fixed: list item title/date rows keep stable spacing during overlay resizing
- Implemented: Milestone 4 GitHub repository linkage inside Projects
- Implemented: SQLite `project_github_repositories` table
- Implemented: backend GitHub commands for get/save/delete/fetch metadata
- Implemented: backend-only `GITHUB_TOKEN` handling
- Implemented: readable missing-token, invalid repository name, and GitHub request error states
- Verified: `npm install` passes after Milestone 4 implementation
- Verified: `npm run build` passes after Milestone 4 implementation
- Verified: `cargo build` passes after Milestone 4 implementation
- Verified: `npm run tauri:dev` launches after Milestone 4 implementation when run outside the sandbox with app-data write access
- Passed: Milestone 4 manual validation checklist
- Implemented: Milestone 5 YouTube navigation entry
- Implemented: SQLite `youtube_references` table
- Implemented: Rust/Tauri YouTube reference list/get/create/update/delete/open commands
- Implemented: backend YouTube URL validation for common watch, short link, and shorts URL forms
- Implemented: frontend YouTube reference CRUD UI with selected/read-only/edit behavior
- Implemented: external opening of saved YouTube URLs through the system browser
- Verified: `npm install` passes after Milestone 5 implementation
- Verified: `npm run build` passes after Milestone 5 implementation
- Verified: `cargo build` passes after Milestone 5 implementation
- Verified: `npm run tauri:dev` launches after Milestone 5 implementation when run outside the sandbox with app-data write access
- Passed: Milestone 5 manual validation checklist
- Implemented: Milestone 6 Project workspace sections for Overview, GitHub, and Chat
- Implemented: project-scoped Chat inside the selected Projects workspace without a second project selector
- Implemented: standalone Planning Chat navigation hidden during the workspace migration
- Verified: `npm run build` passes after Milestone 6 implementation
- Verified: `cargo build` passes after Milestone 6 implementation
- Passed: Milestone 6 manual validation checklist
- Implemented: Milestone 7 selected-project workspace header
- Implemented: Overview, GitHub, Chat, and References workspace sections
- Implemented: minimal References summary without attachment workflows
- Verified: `npm install` passes after Milestone 7 implementation
- Verified: `npm run build` passes after Milestone 7 implementation
- Verified: `cargo build` passes after Milestone 7 implementation
- Verified: `npm run tauri:dev` launches after Milestone 7 implementation
- Passed: Milestone 7 manual validation checklist
- Implemented: Milestone 8 Projects navigation tree actions
- Implemented: expandable Projects row, saved project child rows, compact `+` create action, and compact `...` edit/delete menus
- Preserved: selected-project Overview, GitHub, Chat, and References workspace sections
- Verified: `npm install` passes after Milestone 8 implementation
- Verified: `npm run build` passes after Milestone 8 implementation
- Verified: `cargo build` passes after Milestone 8 implementation
- Verified: `npm run tauri:dev` launches after Milestone 8 implementation
- Passed: Milestone 8 manual validation checklist
- Implemented: Milestone 9 Manual Context Attachments
- Implemented: SQLite `planning_conversation_context` table
- Implemented: backend list/add/remove context attachment commands
- Implemented: Attached Context area inside selected-project Chat
- Implemented: automatic GitHub repository metadata attachment from the selected project's linked repository
- Preserved: existing project-scoped chat and Projects navigation tree behavior
- Verified: `npm install` passes after Milestone 9 implementation
- Verified: `npm run build` passes after Milestone 9 implementation
- Verified: `cargo build` passes after Milestone 9 implementation
- Verified: `npm run tauri:dev` launches after Milestone 9 implementation
- Passed: Milestone 9 manual validation checklist
- Implemented: Milestone 10 Prompt Preview
- Implemented: backend preview command that does not call OpenAI
- Implemented: read-only Prompt Preview panel inside selected-project Chat
- Implemented: attached context inclusion for project chat sends
- Verified: `npm install` passes after Milestone 10 implementation
- Verified: `npm run build` passes after Milestone 10 implementation
- Verified: `cargo build` passes after Milestone 10 implementation
- Verified: `npm run tauri:dev` launches after Milestone 10 implementation
- Complete: Milestone 10 manual validation checklist passed successfully
- Implemented: Milestone 11 Bridge File Drafting
- Implemented: SQLite `bridge_file_drafts` table
- Implemented: backend bridge draft creation/list/retrieve/delete commands
- Implemented: read-only Bridge Drafts panel inside selected-project Chat
- Preserved: Prompt Preview, manual attachments, project chat, and Projects navigation behavior
- Verified: `npm install` passes after Milestone 11 implementation
- Verified: `npm run build` passes after Milestone 11 implementation
- Verified: `cargo build` passes after Milestone 11 implementation
- Verified: `npm run tauri:dev` launches after Milestone 11 implementation
- Complete: Milestone 11 manual validation checklist passed successfully
- Implemented: Milestone 12 Project Markdown Context
- Implemented: SQLite `project_markdown_context` table
- Implemented: backend project Markdown context configuration and loading commands
- Implemented: safe local Markdown reads from configured project roots for chat, Prompt Preview, and bridge drafts
- Verified: `npm install` passes after Milestone 12 implementation
- Verified: `npm run build` passes after Milestone 12 implementation
- Verified: `cargo build` passes after Milestone 12 implementation
- Verified: `npm run tauri:dev` launches after Milestone 12 implementation
- Validated: Milestone 12 context ingestion works after Milestone 13 UI consolidation
- Implemented: Milestone 13 Project Workspace UI Consolidation
- Implemented: conversation child rows in the Projects left navigation hierarchy
- Implemented: Project Edit owns GitHub linkage and project Markdown context configuration
- Implemented: focused chat surface removes redundant workspace and Planning Chat headers
- Implemented: collapsible right-hand pane for attached context and local Markdown bridge drafts
- Implemented: `New Chat` project menu action that does not auto-select an existing conversation
- Verified: `npm install` passes after Milestone 13 implementation
- Verified: `npm run build` passes after Milestone 13 implementation
- Verified: `cargo build` passes after Milestone 13 implementation
- Verified: `npm run tauri:dev` launches after Milestone 13 implementation
- Validated: Milestone 13 Project Workspace UI Consolidation is complete, passed, and successful

## Milestone Validation Workflow

When the user reports that a milestone validation is complete, Codex should update the relevant milestone/docs/changelog statuses to `Complete / Passed / Successful`, run a quick sanity check, review `git status`, commit only the intended milestone changes with a milestone-specific message, and push the current branch. Do not commit unrelated user changes.
