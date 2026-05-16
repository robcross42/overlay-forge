# Bridge Files

Bridge files are markdown handoff documents that describe project state, constraints, and implementation requests for Codex.

Milestone 3 introduces an OpenAI Planning Chat foundation, Milestone 4 introduces a GitHub repository-link foundation, Milestone 5 introduces a controlled local YouTube reference foundation, Milestone 6 moves project-scoped chat into the selected Projects workspace, Milestone 7 refines the selected-project workspace layout, and Milestone 8 is planned for Projects navigation tree actions. Full bridge-file generation is still deferred. Bridge files should still be created and updated manually, then pasted into ChatGPT or Codex chats when context needs to move between sessions.

## Current Manual Bridge

Use:

```text
bridge-files/OPENAI_APP_BRIDGE.md
```

## Current Project State

Milestone 0 is complete, passed, and successful.

The Milestone 0 scratchpad component is complete and passed. Scratchpad content is persisted in SQLite and restored between sessions.

Milestone 1 is complete, passed, and successful.

Current user-validated project baseline is Milestone 7. Milestone 4 - GitHub Integration is complete, passed, and successful.

Milestone 5 - Controlled YouTube Component is complete, passed, and successful.

Milestone 6 - Project Workspace Chat is complete, passed, and successful.

Milestone 7 - Project Workspace Layout Refinement is complete, passed, and successful.

Milestone 8 - Projects Navigation Tree Actions is planned.

Milestone 3 is complete, passed, and successful. It adds local planning conversations/messages and backend OpenAI Responses API calls, but it does not generate bridge files yet.

Milestone 4 is complete, passed, and successful. It adds local project GitHub repository linkage and backend GitHub metadata fetches through `GITHUB_TOKEN`, but it does not perform Codex handoff or GitHub write operations.

Milestone 5 is complete, passed, and successful. It adds local user-curated YouTube references with SQLite persistence, backend URL validation, and external URL opening, but it does not use a YouTube API key, YouTube account login, scraping, transcripts, recommendations, downloads, or account sync.

Milestone 6 is complete, passed, and successful. It moves chat into Projects as a selected-project workspace section, while preserving the existing `planning_conversations` and `planning_messages` data.

Milestone 7 is complete, passed, and successful. It refines the selected project workspace with Overview, GitHub, Chat, and References sections. References is minimal and does not generate bridge files or attach context.

Milestone 8 is planned. It should make Projects expandable in the left navigation, list saved projects as children, expose a compact `+` action for new project flow, and expose compact `...` menus on project rows for edit/delete. The pattern should be validated on Projects before applying it to other modules.

Milestone numbering note: Milestone 2 is the Local Projects component and is complete, passed, and successful. Milestone 3 is the OpenAI Planning Chat component. Milestone 4 is GitHub Integration. Milestone 5 is the Controlled YouTube Component. Milestone 6 is Project Workspace Chat. Milestone 7 is Project Workspace Layout Refinement. Milestone 8 is Projects Navigation Tree Actions. Do not infer milestone IDs from numbered-list positions.

## Intended Future Workflow

1. Select a local project in Overlay Forge.
2. Plan with the in-app Chat section inside the selected project workspace.
3. Link the project to a GitHub repository in Projects when repository context is useful.
4. Generate a Codex-ready markdown bridge file in a later milestone.
5. Save the bridge file in SQLite.
6. Export the bridge file to disk.
7. Paste or provide the bridge file to Codex for implementation.

## Milestone Validation Workflow

When the user reports that a milestone validation is complete, Codex should treat that as permission to finish the milestone handoff:

1. Update milestone, changelog, project plan, architecture, data model, and bridge docs from pending validation to `Complete / Passed / Successful`.
2. Run a quick sanity check appropriate to the milestone.
3. Review `git status` and include only intended milestone changes.
4. Commit with a milestone-specific message.
5. Push the current branch.

Codex should not commit unrelated user changes. If unrelated changes are present, leave them unstaged or ask for direction if they block the milestone commit.

## Status Values Planned For Later Milestones

```text
DRAFT
READY_FOR_CODEX
SENT_TO_CODEX
IMPLEMENTED
VALIDATED
ARCHIVED
```
