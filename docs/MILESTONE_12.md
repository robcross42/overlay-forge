# Milestone 12 - Project Markdown Context

## Status

Complete / Passed / Successful

## Goal

Add project-level local Markdown context for project chat, Prompt Preview, and bridge-file draft workflows.

Milestone 12 should prove this workflow:

```text
Project workspace -> configured local project root -> README.md -> referenced Markdown files -> fresh project context -> chat / Prompt Preview / bridge drafts
```

Milestone 12 is project-scoped, not conversation-scoped. The user should not need to manually attach the same repository documentation to each chat conversation.

Milestone 12 remains local-first. It reads Markdown documentation from a configured local project root only.

Validation note: Milestone 12 context ingestion is validated. Project Markdown files load from the configured local root and are used by chat and bridge draft workflows after the Milestone 13 UI consolidation pass.

## Implemented Capabilities

- Add a project-level local Markdown context configuration.
- Allow a project to store or configure a local documentation root path.
- Load a fresh copy of `README.md` when a project chat starts or loads.
- Parse `README.md` for referenced local Markdown files.
- Resolve referenced Markdown files only when they are inside the configured project root.
- Read resolved local Markdown files as project-level context.
- Include loaded project Markdown context in project chat sends.
- Include loaded project Markdown context in Prompt Preview.
- Include loaded project Markdown context in bridge-file draft generation.
- Preserve conversation-level manual attachments as an additional context layer.
- Show readable warnings for missing, unreadable, or skipped Markdown files.
- Prevent crashes when configured files are missing or invalid.
- Preserve existing Milestone 0 through Milestone 11 behavior.

## Implementation Notes

- Project Chat now includes a Project Markdown configuration panel for the selected project.
- The backend stores one Markdown context root per project in SQLite.
- Markdown context is loaded from disk on project chat load, conversation selection, explicit reload, Prompt Preview, project chat send, and bridge draft generation.
- Prompt Preview displays project Markdown sources separately from conversation manual attachments.
- Project chat sends include project Markdown context before conversation manual attachments.
- Bridge drafts include project Markdown context before conversation manual attachments.
- Missing, unreadable, skipped, unsafe, or truncated Markdown sources produce readable warnings instead of crashing the app.

## Initial Markdown Sources

Milestone 12 should prioritize these local Markdown sources:

```text
README.md
CHANGELOG.md
docs/*.md
bridge-files/*.md
```

The first implementation can support both:

```text
Explicit Markdown links found in README.md
Known project documentation paths
```

The implementation should avoid broad unrestricted filesystem scans.

## Data Tables

Milestone 12 should prefer a small additive SQLite model.

Recommended table:

```text
project_markdown_context
- id
- project_id
- root_path
- readme_path
- created_at
- updated_at
```

`project_id` links the Markdown context configuration to `projects.id`.

`root_path` stores the configured local project documentation root.

`readme_path` stores the README path relative to the configured root when useful.

If cached file snapshots are needed, they should use a separate non-destructive table. The default behavior should be fresh reads from disk on chat load or new chat so local documentation changes are reflected.

## Context Assembly Rules

Project Markdown context should be assembled before conversation-level manual attachments.

The context package should preserve source visibility by identifying each included Markdown file by relative path.

A safe assembled context order is:

```text
Project Markdown Context
Conversation Manual Attachments
Selected Project Metadata
Selected Conversation Messages
Current Draft User Message
```

Project Markdown context should not silently override manual attachments. Conversation-level manual attachments remain valid and should act as an additional layer.

## Boundary

Milestone 12 reads local Markdown documentation only from the configured project root.

Milestone 12 does not:

- Read arbitrary filesystem paths outside the configured project root.
- Follow unsafe path traversal references.
- Read GitHub repository file contents through the GitHub API.
- Upload files.
- Add file uploads.
- Add vector stores.
- Add semantic search.
- Add broad repository indexing.
- Add token budgeting beyond a basic size guard if needed.
- Export bridge files.
- Copy bridge files to clipboard.
- Hand off directly to Codex.
- Push to GitHub.
- Create commits, branches, pull requests, or issues.
- Automatically trust generated or external content without user visibility.
- Replace conversation-level manual attachments.

## Setup Validation

Run:

```powershell
npm install
```

Expected result:

```text
Passed. Dependencies install successfully.
```

Run:

```powershell
npm run build
```

Expected result:

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
Passed. Rust backend compiles successfully.
```

Run:

```powershell
npm run tauri:dev
```

Expected result:

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
Configure a local Markdown documentation root for the selected project.
```

Pass criteria:

```text
The project stores the configured local documentation root without affecting other projects.
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
Open or start a project chat where the configured root contains README.md.
```

Pass criteria:

```text
README.md is loaded freshly from the configured local root.
```

Validate:

```text
Use a README.md that references other Markdown files inside the configured project root.
```

Pass criteria:

```text
Referenced Markdown files are resolved and included when they remain inside the configured project root.
```

Validate:

```text
Use a README.md that references a missing Markdown file.
```

Pass criteria:

```text
The app shows a readable warning and does not crash.
```

Validate:

```text
Use a README.md that references a file outside the configured project root.
```

Pass criteria:

```text
The app skips the unsafe reference and shows a readable warning.
```

Validate:

```text
Open Prompt Preview for the selected project chat.
```

Pass criteria:

```text
Prompt Preview shows the loaded project Markdown context and identifies included files by relative path.
```

Validate:

```text
Attach at least one manual context item to the same conversation.
```

Pass criteria:

```text
Manual context still appears and is included as an additional context layer.
```

Validate:

```text
Send a project chat message.
```

Pass criteria:

```text
The chat send path includes project Markdown context and preserves existing chat behavior.
```

Validate:

```text
Click Draft Bridge File.
```

Pass criteria:

```text
Generated bridge drafts include relevant project Markdown context.
```

Validate:

```text
Edit a local Markdown file, then reopen or reload the project chat.
```

Pass criteria:

```text
The app reads the fresh local Markdown content rather than relying on a stale snapshot.
```

Validate:

```text
Restart the app, select the same project, and return to the Chat section.
```

Pass criteria:

```text
The configured Markdown context root persists and Markdown context loads again.
```

Validate:

```text
Return to Scratchpad, Tasks, Notes, Calendar, Projects, GitHub, Chat, References, YouTube, Prompt Preview, Bridge Drafts, and existing app controls.
```

Pass criteria:

```text
Existing Milestone 0 through Milestone 11 behavior still works.
```

## Deferred Items

- GitHub repository file reading
- GitHub file browsing
- GitHub writes
- GitHub commit creation
- GitHub pull request creation
- Direct Codex handoff
- Bridge-file export to local `.md` files
- Copy-to-clipboard workflow
- Full bridge-file editor
- Bridge-file approval workflow
- File uploads
- Vector stores
- Semantic search
- Broad repository indexing
- Advanced token budgeting
- Chat streaming
- Model picker UI
- ChatGPT import
- Automatic prompt rewriting
- Long-term prompt template system
- External/cloud sync

## User Pass/Fail Reporting Format

```markdown
# Milestone 12 Validation Results

## Overall Result

Pass or Fail

## Failed Items

- Item:
  - Expected:
  - Actual:

## Notes

Any extra observations.
```
