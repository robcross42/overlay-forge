# Milestone 11 - Bridge File Drafting

## Status

Implemented / Pending User Validation

## Goal

Generate a local Markdown bridge-file draft from a selected project chat conversation.

Milestone 11 proves this workflow:

```text
Project workspace -> Chat -> selected conversation -> attached context/prompt preview -> Draft Bridge File -> saved local draft
```

Milestone 11 creates local bridge drafts only. It does not export Markdown files, hand off to Codex, write to GitHub, or replace user review.

## Implemented Capabilities

- Added a `Draft Bridge File` action inside selected-project Chat.
- Added SQLite-backed bridge-file draft persistence.
- Added backend commands to list, retrieve, create, and delete bridge drafts.
- Added a read-only Bridge Drafts panel inside project Chat.
- Generated Markdown drafts from the selected project, selected conversation, saved messages, and attached context.
- Resolved linked GitHub repository metadata from the selected project when building bridge drafts.
- Included resolved attached context in normal project chat sends.
- Added draft list/reopen behavior scoped to the selected project.
- Added bridge draft deletion that removes only the draft row.
- Preserved existing Prompt Preview behavior.
- Preserved existing manual context attachment behavior.
- Preserved existing project-scoped chat behavior.
- Preserved existing Projects navigation tree/workspace behavior.

## Data Table

Milestone 11 adds:

```text
bridge_file_drafts
- id
- project_id
- conversation_id
- title
- content
- status
- created_at
- updated_at
```

`project_id` links each draft to `projects.id`.

`conversation_id` links each draft to the source `planning_conversations.id`.

`content` stores the full generated Markdown draft.

`status` is `draft` in Milestone 11. Approval, obsolete, sent, and implemented workflows are deferred.

## Generated Draft Structure

Generated bridge drafts include:

```text
Project
Conversation Source
Goal
Relevant Context
Implementation Instructions
Validation Checklist
Deferred Items
Notes
```

When the app cannot safely infer details, the draft uses explicit placeholders such as:

```text
TODO: User review required.
```

## Boundary

Bridge drafts are local SQLite records.

Milestone 11 does not:

- Export drafts to local `.md` files.
- Copy drafts to clipboard.
- Open Codex.
- Send content to Codex.
- Push to GitHub.
- Create commits, branches, pull requests, or issues.
- Approve generated drafts automatically.
- Replace user review.

Project chat sends now include resolved conversation context in the OpenAI request. This includes attached context rows and the selected project's linked GitHub repository metadata when available. The app still does not read GitHub files, inspect repository contents, or claim repository access beyond the metadata provided locally.

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

Expected result:

```text
Frontend builds successfully.
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
Create or select a chat conversation with messages.
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
Open Prompt Preview.
```

Pass criteria:

```text
Prompt Preview still works and does not call OpenAI.
```

Validate:

```text
Click Draft Bridge File.
```

Pass criteria:

```text
A Markdown bridge draft is generated and displayed.
```

Validate:

```text
Review generated bridge draft sections.
```

Pass criteria:

```text
Draft includes project, conversation source, goal, relevant context, implementation instructions, validation checklist, deferred items, and notes.
```

Validate:

```text
Restart the app, select the same project, and return to bridge drafts.
```

Pass criteria:

```text
Generated bridge draft persists in SQLite and can be reopened.
```

Validate:

```text
Generate a second bridge draft from the same conversation.
```

Pass criteria:

```text
The app creates a separate draft without data loss.
```

Validate:

```text
Delete a bridge draft.
```

Pass criteria:

```text
Only the selected bridge draft is deleted and source conversation/context remains intact.
```

Validate:

```text
Return to Scratchpad, Tasks, Notes, Calendar, Projects, GitHub, Chat, References, YouTube, and existing app controls.
```

Pass criteria:

```text
Existing Milestone 0 through Milestone 10 behavior still works.
```

## Deferred Items

- Full bridge-file editor
- Approval workflow
- Obsolete status workflow
- Export to local `.md` file
- Copy-to-clipboard workflow
- Direct Codex handoff
- Project-level local Markdown context
- GitHub commit creation
- GitHub pull request creation
- GitHub file writing
- Chat streaming
- Model picker UI
- Token budgeting
- Vector stores
- Semantic search
- Automatic context attachment
- ChatGPT import

## User Pass/Fail Reporting Format

```markdown
# Milestone 11 Validation Results

## Overall Result

Pass or Fail

## Failed Items

- Item:
  - Expected:
  - Actual:

## Notes

Any extra observations.
```
