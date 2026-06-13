# Bridge Files

Bridge files are markdown handoff documents that describe project state, constraints, and implementation requests for Codex.

Milestone 3 introduces an OpenAI Planning Chat foundation, Milestone 4 introduces a GitHub repository-link foundation, Milestone 5 introduces a controlled local YouTube reference foundation, Milestone 6 moves project-scoped chat into the selected Projects workspace, Milestone 7 refines the selected-project workspace layout, Milestone 8 completes Projects navigation tree actions, Milestone 9 completes manual context attachments, Milestone 10 completes Prompt Preview, Milestone 11 completes local Bridge File Drafting, Milestone 12 completes project-level local Markdown context, and Milestone 13 completes Project Workspace UI Consolidation. Gaming Screenshot Capture is also complete, passed, and successful as a post-Milestone 13 feature addendum. Export and Codex handoff are still deferred.

## Current Manual Bridge

Use:

```text
bridge-files/OPENAI_APP_BRIDGE.md
```

## Current Project State

Milestone 0 is complete, passed, and successful.

The Milestone 0 scratchpad component is complete and passed. Scratchpad content is persisted in SQLite and restored between sessions.

Milestone 1 is complete, passed, and successful.

Current user-validated project baseline is Milestone 13. Milestone 4 - GitHub Integration is complete, passed, and successful.

Milestone 5 - Controlled YouTube Component is complete, passed, and successful.

Milestone 6 - Project Workspace Chat is complete, passed, and successful.

Milestone 7 - Project Workspace Layout Refinement is complete, passed, and successful.

Milestone 8 - Projects Navigation Tree Actions is complete, passed, and successful.

Milestone 9 - Manual Context Attachments is complete, passed, and successful.

Milestone 10 - Prompt Preview is complete, passed, and successful.

Milestone 11 - Bridge File Drafting is complete, passed, and successful.

Milestone 12 - Project Markdown Context is complete, passed, and successful.

Milestone 13 - Project Workspace UI Consolidation is complete, passed, and successful.

Gaming Screenshot Capture is complete, passed, and successful. It adds a Gaming workspace with GearBlocks, overlay-hidden screenshot capture, unique PNG and capture manifest output under `game-screenshots/`, SQLite screenshot metadata, Tauri asset-backed thumbnail previews, and right-click screenshot deletion cleanup.

Milestone 3 is complete, passed, and successful. It adds local planning conversations/messages and backend OpenAI Responses API calls, but it does not generate bridge files yet.

Milestone 4 is complete, passed, and successful. It adds local project GitHub repository linkage and backend GitHub metadata fetches through `GITHUB_TOKEN`, but it does not perform Codex handoff or GitHub write operations.

Milestone 5 is complete, passed, and successful. It adds local user-curated YouTube references with SQLite persistence, backend URL validation, and external URL opening, but it does not use a YouTube API key, YouTube account login, scraping, transcripts, recommendations, downloads, or account sync.

Milestone 6 is complete, passed, and successful. It moves chat into Projects as a selected-project workspace section, while preserving the existing `planning_conversations` and `planning_messages` data.

Milestone 7 is complete, passed, and successful. It refines the selected project workspace with Overview, GitHub, Chat, and References sections. References is minimal and does not generate bridge files or attach context.

Milestone 8 is complete, passed, and successful. It makes Projects expandable in the left navigation, lists saved projects as children, exposes a compact `+` action for new project flow, and exposes compact `...` menus on project rows for edit/delete. The pattern was validated on Projects before applying it to other modules.

Milestone 9 is complete, passed, and successful. It adds manual, conversation-scoped context attachment links inside selected-project Chat. Attachments can link existing local project, note, task, calendar event, YouTube reference, and scratchpad context without deleting source records when attachments are removed. Linked GitHub repository metadata is automatically attached when a selected project has a repository defined in the GitHub section.

Milestone 10 is complete, passed, and successful. It adds a read-only Prompt Preview action inside selected-project Chat. The preview uses existing local project, conversation, draft message, and attached context data and does not call OpenAI. Attached context inclusion in actual sends remains deferred.

Milestone 11 is complete, passed, and successful. It adds local SQLite bridge draft generation from selected project Chat conversations. Drafts use resolved attached context, including linked GitHub repository metadata. Project chat sends also include resolved attached context. Drafts remain in-app and read-only; export, editor, approval workflow, and Codex handoff are deferred.

Milestone 12 is complete, passed, and successful for context ingestion. It adds project-level local Markdown context by loading a fresh `README.md` from a configured project root whenever project chat starts or loads, then resolving referenced Markdown files inside that project root for chat and bridge drafts.

Milestone 13 is complete, passed, and successful. It consolidates the Projects workspace UI so project and conversation selection live in the left navigation hierarchy, while Project Edit owns project details, GitHub integration, local Markdown context configuration, and local repo/context settings. Conversation attached context and local Markdown bridge drafts now live in a collapsible right-hand chat pane. The project row menu uses `New Chat` for new conversations; existing conversations open from their left-nav child rows.

Gaming Screenshot Capture is complete, passed, and successful. Future work should preserve the validated `game-screenshots/` folder layout, metadata table, Tauri asset preview scope, overlay-hidden capture behavior, and screenshot delete cleanup unless a later game-internal capture milestone explicitly supersedes the Windows GDI implementation.

Milestone numbering note: Milestone 2 is the Local Projects component and is complete, passed, and successful. Milestone 3 is the OpenAI Planning Chat component. Milestone 4 is GitHub Integration. Milestone 5 is the Controlled YouTube Component. Milestone 6 is Project Workspace Chat. Milestone 7 is Project Workspace Layout Refinement. Milestone 8 is Projects Navigation Tree Actions and is complete, passed, and successful. Milestone 9 is Manual Context Attachments and is complete, passed, and successful. Milestone 10 is Prompt Preview and is complete, passed, and successful. Milestone 11 is Bridge File Drafting and is complete, passed, and successful. Milestone 12 is Project Markdown Context and is complete, passed, and successful. Milestone 13 is Project Workspace UI Consolidation and is complete, passed, and successful. Do not infer milestone IDs from numbered-list positions.

## Intended Future Workflow

1. Select a local project in Overlay Forge.
2. Plan with the in-app Chat section inside the selected project workspace.
3. Link the project to a GitHub repository in Projects when repository context is useful.
4. Generate a local bridge-file draft from selected-project Chat.
5. Review the saved draft in SQLite.
6. Export the bridge file to disk in a later milestone.
7. Paste or provide the bridge file to Codex for implementation in a later milestone.

## Milestone Validation Workflow

When the user reports that a milestone validation is complete, Codex should treat that as permission to finish the milestone handoff:

1. Update milestone, changelog, project plan, architecture, data model, and bridge docs from pending validation to `Complete / Passed / Successful`.
2. Run a quick sanity check appropriate to the milestone.
3. Review `git status` and include only intended milestone changes.
4. Commit with a milestone-specific message.
5. Push the current branch.

Codex should not commit unrelated user changes. If unrelated changes are present, leave them unstaged or ask for direction if they block the milestone commit.

## Project Deferred Items

See `docs/PROJECT_DEFERRED_ITEMS.md` for the centralized list of deferred items related to Project Chat, Bridge Files, and GitHub integration.

## Status Values Planned For Later Milestones

```text
DRAFT
READY_FOR_CODEX
SENT_TO_CODEX
IMPLEMENTED
VALIDATED
ARCHIVED
```
