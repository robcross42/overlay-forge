# Milestone 9 - Manual Context Attachments

## Status

Complete / Passed / Successful

## Goal

Let the user manually attach existing local app context to a selected project chat conversation.

Milestone 9 proves this workflow:

```text
Project workspace -> Chat -> select conversation -> attach local context -> context persists with that conversation
```

This milestone stores visible attachment links only. It does not automatically assemble prompts, preview prompt context, count tokens, or send attached context to OpenAI.

## Implemented Capabilities

- Added an Attached Context area inside the project Chat section.
- Added manual attachment controls for the selected chat conversation.
- Added SQLite-backed conversation context attachment persistence.
- Added backend commands for listing, adding, and removing context attachments.
- Added frontend support for displaying attached context items on the selected conversation.
- Added frontend support for removing attached context links without deleting source records.
- Added automatic GitHub repository context attachment when the selected project has a linked repository.
- Preserved attachment links across app restart through SQLite persistence.
- Preserved Projects navigation tree behavior from Milestone 8.
- Preserved existing selected-project workspace behavior.
- Preserved existing project-scoped chat behavior.

## Supported Context Types

Milestone 9 supports:

```text
project
github_repository
note
task
calendar_event
youtube_reference
scratchpad
```

The attachment stores a readable label, source type, and source record ID when one exists. Scratchpad is a singleton and uses a nullable source ID.

When a selected project has a linked GitHub repository, the Chat section automatically adds that repository metadata link to the selected conversation's Attached Context list. The duplicate guard prevents repeatedly adding the same repository attachment to the same conversation.

## Data Table

Milestone 9 adds:

```text
planning_conversation_context
- id
- conversation_id
- context_type
- source_id
- label
- created_at
```

`conversation_id` links each attachment to `planning_conversations.id`. Attachments are scoped to a single conversation.

Removing an attachment deletes only the attachment row. It does not delete the source project, repository link, note, task, calendar event, YouTube reference, or scratchpad content.

## Setup Validation

Run:

```powershell
npm install
```

Result:

```text
Passed. Dependencies install successfully.
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

Result:

```text
Passed. Rust backend compiles successfully.
```

Run:

```powershell
npm run tauri:dev
```

Result:

```text
Passed. App launches successfully in development mode. The app process started and was stopped after the validation timeout.
```

## Manual Validation Checklist

Validate:

```text
Open the app and reveal the overlay with Ctrl+Shift+Space.
```

Pass criteria:

```text
Overlay appears using existing hotkey behavior.
```

Validate:

```text
Expand Projects in the left navigation tree and select a project.
```

Pass criteria:

```text
The selected project becomes the active workspace.
```

Validate:

```text
Open the selected project's Chat section.
```

Pass criteria:

```text
Project-scoped chat loads normally.
```

Validate:

```text
Create or select a chat conversation.
```

Pass criteria:

```text
The conversation loads and messages remain functional.
```

Validate:

```text
Attach project details to the conversation.
```

Pass criteria:

```text
Project details appear in the Attached Context list.
```

Validate:

```text
Attach GitHub repository metadata to the conversation, if the selected project has a linked repository.
```

Pass criteria:

```text
The GitHub repository appears automatically in the Attached Context list.
```

Validate:

```text
Attach a note or task to the conversation.
```

Pass criteria:

```text
The selected note or task appears in the Attached Context list.
```

Validate:

```text
Restart the app, select the same project, and return to the same conversation.
```

Pass criteria:

```text
Attached context items are restored from SQLite.
```

Validate:

```text
Remove an attached context item.
```

Pass criteria:

```text
The attachment is removed from the list, but the source record is not deleted.
```

Validate:

```text
Restart the app again.
```

Pass criteria:

```text
Removed attachment does not return.
```

Validate:

```text
Select a different project and chat conversation.
```

Pass criteria:

```text
Context attachments remain scoped to their original conversation.
```

Validate:

```text
Return to Scratchpad, Tasks, Notes, Calendar, Projects, GitHub, Chat, References, YouTube, and existing app controls.
```

Pass criteria:

```text
Existing Milestone 0 through Milestone 8 behavior still works.
```

## Deferred Items

- Automatic context attachment
- Semantic search
- Vector store workflows
- File uploads
- GitHub file reading
- YouTube transcript extraction
- Prompt preview
- Token counting
- Token budgeting
- Bridge-file generation
- Bridge-file editor
- Bridge-file export
- Codex handoff
- ChatGPT import
- Conversation search/filtering
- Chat streaming
- Model picker UI

## User Pass/Fail Reporting Format

```markdown
# Milestone 9 Validation Results

## Overall Result

Pass or Fail

## Failed Items

- Item:
  - Expected:
  - Actual:

## Notes

Any extra observations.
```
