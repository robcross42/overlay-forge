# Milestone 10 - Prompt Preview

## Status

Complete / Passed / Successful

## Goal

Add a controlled Prompt Preview feature inside project workspace Chat.

Milestone 10 proves this workflow:

```text
Project workspace -> Chat -> selected conversation -> attached context -> draft message -> prompt preview
```

Milestone 10 is a visibility and validation milestone. It does not generate bridge files.

## Implemented Capabilities

- Added a Prompt Preview action inside selected-project Chat.
- Added a backend preview command that assembles prompt preview data without calling OpenAI.
- Added a read-only frontend Prompt Preview panel.
- Preview shows the selected project, selected conversation, existing message count, current draft message, attached context, and assembled prompt preview.
- Preview resolves attached context content where local data is safely available.
- Preview shows readable warnings when attached source records cannot be resolved.
- Preview handles conversations with no attached context.
- Existing message send behavior is preserved.
- Existing manual context attachment behavior is preserved.
- Existing Projects navigation tree and workspace behavior is preserved.

## Boundary

Prompt Preview does not call OpenAI.

Milestone 10 does not silently change the message send path. The existing OpenAI send behavior remains project context plus recent conversation messages. Attached context inclusion in actual OpenAI sends is deferred beyond Milestone 10.

The assembled preview is the intended local context package for review before later prompt inclusion work.

## Preview Sections

Prompt Preview shows:

```text
Project
Conversation
User Message
Attached Context
Assembled Prompt Preview
Warnings
```

## Data Tables

Milestone 10 does not add or change SQLite tables.

It uses existing tables:

```text
projects
planning_conversations
planning_messages
planning_conversation_context
project_github_repositories
notes
tasks
calendar_events
youtube_references
scratchpad
```

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
Attach at least one context item to the conversation.
```

Pass criteria:

```text
Attached context appears in the Attached Context list.
```

Validate:

```text
Type a draft message and open Prompt Preview.
```

Pass criteria:

```text
Prompt Preview opens without sending a message.
```

Validate:

```text
Review Prompt Preview contents.
```

Pass criteria:

```text
Preview shows selected project, selected conversation, draft message, attached context, and assembled prompt preview.
```

Validate:

```text
Close Prompt Preview.
```

Pass criteria:

```text
Draft message remains available and no message has been sent unless the user explicitly sends it.
```

Validate:

```text
Send the message normally.
```

Pass criteria:

```text
Existing chat send behavior still works.
```

Validate:

```text
Open Prompt Preview with no attached context.
```

Pass criteria:

```text
Preview still works and clearly shows no attached context.
```

Validate:

```text
Restart the app, select the same project, and return to the same conversation.
```

Pass criteria:

```text
Conversation, messages, and attached context still restore correctly.
```

Validate:

```text
Return to Scratchpad, Tasks, Notes, Calendar, Projects, GitHub, Chat, References, YouTube, and existing app controls.
```

Pass criteria:

```text
Existing Milestone 0 through Milestone 9 behavior still works.
```

## Deferred Items

- Bridge-file generation
- Bridge-file editor
- Bridge-file export
- Codex handoff
- GitHub file reading
- YouTube transcript extraction
- Semantic search
- Vector store workflows
- File uploads
- Automatic context attachment
- Token counting
- Token budgeting
- Model picker UI
- Chat streaming
- ChatGPT import
- Automatic prompt rewriting
- Long-term prompt template system
- Attached context inclusion in actual OpenAI sends

## User Pass/Fail Reporting Format

```markdown
# Milestone 10 Validation Results

## Overall Result

Pass or Fail

## Failed Items

- Item:
  - Expected:
  - Actual:

## Notes

Any extra observations.
```
