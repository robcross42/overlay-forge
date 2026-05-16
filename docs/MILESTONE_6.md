# Milestone 6 - Project Workspace Chat

## Status

Complete / Passed / Successful

## Goal

Move project-scoped planning chat into the Projects section so chat is attached to the selected project workspace instead of living behind a separate Planning Chat navigation entry.

User validation is complete and Milestone 6 passed.

## Implemented Capabilities

- Projects remains the main navigation entry for selecting a local project.
- Selecting a project establishes the active project workspace context.
- The selected project workspace exposes sections for Overview, GitHub, and Chat.
- Overview preserves the existing project details view, edit, save, and delete behavior.
- GitHub preserves the existing project-scoped repository linkage and metadata fetch behavior.
- Chat reuses the existing planning conversation and message persistence for the selected project.
- Project workspace Chat receives the selected project automatically and does not show a second project selector.
- New chat conversations require a user-entered title before they can be created.
- Standalone Planning Chat navigation is hidden during this migration so project chat is reached through Projects.
- Existing `planning_conversations` and `planning_messages` data is preserved.
- Existing Scratchpad, Tasks, Notes, Calendar, Projects, GitHub, and YouTube surfaces remain available.

## Data Tables

Milestone 6 does not add new SQLite tables.

It continues to use the Milestone 3 planning chat tables:

```text
planning_conversations
- id
- project_id
- title
- created_at
- updated_at
```

```text
planning_messages
- id
- conversation_id
- role
- content
- created_at
```

## Setup Validation

Run:

```powershell
npm install
```

Expected result:

```text
Dependencies install successfully.
```

Run:

```powershell
npm run build
```

Result:

```text
Passed. Frontend builds successfully.
```

Run:

```powershell
cd src-tauri
cargo build
```

Expected result:

```text
Rust backend compiles successfully.
```

Run:

```powershell
npm run tauri:dev
```

Expected result:

```text
App launches successfully in development mode.
```

## Manual Validation Checklist

Validate:

```text
Open Projects.
```

Pass criteria:

```text
Projects loads normally.
```

Validate:

```text
Select a project.
```

Pass criteria:

```text
The selected project becomes the active workspace.
```

Validate:

```text
Open the project Chat section.
```

Pass criteria:

```text
Chat loads without requiring a separate project selector.
```

Validate:

```text
Open Chat and review the new conversation controls.
```

Pass criteria:

```text
A conversation title field is visible and New Conversation is disabled until a title is entered.
```

Validate:

```text
Enter a conversation title, create a new conversation, and send a message.
```

Pass criteria:

```text
The user message and assistant response are shown.
```

Validate:

```text
Restart the app, return to Projects, and select the same project.
```

Pass criteria:

```text
The project's chat conversations and messages restore from SQLite.
```

Validate:

```text
Select a different project.
```

Pass criteria:

```text
The second project has a separate chat context.
```

Validate:

```text
Return to Scratchpad, Tasks, Notes, Calendar, Projects Overview, GitHub, and YouTube.
```

Pass criteria:

```text
Existing Milestone 0 through Milestone 5 behavior still works.
```

## Deferred Items

- Bridge-file generation
- Prompt preview
- Automatic context attachment
- GitHub file reading
- Codex handoff
- ChatGPT import
- Conversation search/filtering
- Chat streaming
- Model picker UI

## User Pass/Fail Reporting Format

```markdown
# Milestone 6 Validation Results

## Overall Result

Pass or Fail

## Failed Items

- Item:
  - Expected:
  - Actual:

## Notes

Any extra observations.
```
