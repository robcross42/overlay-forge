# Overlay Forge - OpenAI App Bridge

## Purpose

This file is the manual bridge between Overlay Forge project state and ChatGPT/Codex conversations.

Provide this file to ChatGPT or Codex when a new chat needs current project context. Keep it concise and update it after meaningful architecture, milestone, or validation changes.

## Current Milestone

Milestone 11 - Bridge File Drafting

Status: **Implemented / Pending User Validation**

Current user-validated project baseline: **Milestone 10**. Milestone 11 is implemented and pending user validation.

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

Milestone 11 is implemented and pending user validation. It adds local bridge-file draft generation from selected project chat conversations. Drafts are stored in SQLite as `bridge_file_drafts`, displayed read-only in the Chat section, and remain local. Resolved attached context, including linked GitHub repository metadata, is used by bridge drafts and normal project chat sends. Export, full editing, approval workflow, GitHub writes, and direct Codex handoff remain deferred.

Milestone 12 is planned and not started. It should add project-level local Markdown context by loading a fresh `README.md` from a configured local project root whenever a project chat starts or loads, then resolving referenced Markdown files inside that project root for chat, Prompt Preview, and bridge drafts. This should be project-scoped, not per-conversation attachment.

Milestone numbering note: Milestone 2 is the Local Projects component. Milestone 3 is the OpenAI Planning Chat component. Milestone 4 is GitHub Integration. Milestone 5 is the Controlled YouTube Component. Milestone 6 is Project Workspace Chat. Milestone 7 is Project Workspace Layout Refinement. Milestone 8 is Projects Navigation Tree Actions and is complete, passed, and successful. Milestone 9 is Manual Context Attachments and is complete, passed, and successful. Milestone 10 is Prompt Preview and is complete, passed, and successful. Milestone 11 is Bridge File Drafting and is implemented pending user validation. Milestone 12 is Project Markdown Context and is planned. Do not mistake roadmap list item positions for milestone IDs.

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
- Planned project-level local Markdown context from README and referenced Markdown files

## Hotkey

```text
Ctrl+Shift+Space
```

## Deferred

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
- Pending: Milestone 11 manual validation checklist

## Milestone Validation Workflow

When the user reports that a milestone validation is complete, Codex should update the relevant milestone/docs/changelog statuses to `Complete / Passed / Successful`, run a quick sanity check, review `git status`, commit only the intended milestone changes with a milestone-specific message, and push the current branch. Do not commit unrelated user changes.
