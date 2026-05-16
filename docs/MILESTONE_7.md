# Milestone 7 - Project Workspace Layout Refinement

## Status

Complete / Passed / Successful

## Goal

Refine the Projects section into a clearer workspace shell for the selected project, with predictable internal sections that preserve selected project context.

User validation is complete and Milestone 7 passed.

## Implemented Capabilities

- Projects remains the main navigation entry for selecting a local project.
- Selecting a project establishes the active project workspace context.
- The selected project workspace shows a stable active-workspace header with the project name and status.
- Workspace section navigation now includes Overview, GitHub, Chat, and References.
- Switching workspace sections keeps the selected project context stable.
- Overview preserves existing project details, read-only view, edit, save, and delete behavior.
- GitHub preserves the existing project-scoped repository linkage and metadata fetch behavior.
- Chat preserves the existing selected-project chat behavior from Milestone 6.
- References adds a minimal local context summary without creating attachment workflows.
- Existing Scratchpad, Tasks, Notes, Calendar, Projects, GitHub, Chat, and YouTube behavior remains available.

## Data Tables

Milestone 7 does not add or change SQLite tables.

It continues to use the existing tables:

```text
projects
project_github_repositories
planning_conversations
planning_messages
```

## References Boundary

The Milestone 7 References section is intentionally minimal. It summarizes local reference categories that already exist or are planned:

- Selected project details
- Linked GitHub repository metadata
- Future manual attachments
- Future prompt context previews

Milestone 7 does not implement manual context attachments, prompt preview, bridge-file generation, GitHub file browsing, or AI-generated summaries.

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
Open the app and reveal the overlay with Ctrl+Shift+Space.
```

Pass criteria:

```text
Overlay appears using existing hotkey behavior.
```

Validate:

```text
Navigate to Projects.
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
Switch between Overview, GitHub, Chat, and References.
```

Pass criteria:

```text
Each section loads without losing selected project context.
```

Validate:

```text
Open Overview.
```

Pass criteria:

```text
Project details are visible and existing project edit/view behavior still works.
```

Validate:

```text
Open GitHub.
```

Pass criteria:

```text
Existing repository linkage and metadata-fetch behavior still works.
```

Validate:

```text
Open Chat.
```

Pass criteria:

```text
Existing project-scoped conversations and messages still work.
```

Validate:

```text
Open References.
```

Pass criteria:

```text
References section loads without errors and does not disrupt project context.
```

Validate:

```text
Switch to a different project.
```

Pass criteria:

```text
Workspace sections update to the newly selected project.
```

Validate:

```text
Restart the app and return to Projects.
```

Pass criteria:

```text
Projects, workspace layout, GitHub data, and Chat data restore correctly.
```

Validate:

```text
Return to Scratchpad, Tasks, Notes, Calendar, Projects, YouTube, and existing app controls.
```

Pass criteria:

```text
Existing Milestone 0 through Milestone 6 behavior still works.
```

## Deferred Items

- Manual context attachments
- Prompt preview
- Bridge-file generation
- Bridge-file editor
- Bridge-file export
- GitHub file browsing
- GitHub write operations
- Codex handoff
- ChatGPT import
- Conversation search/filtering
- Chat streaming
- Model picker UI
- AI-generated project summaries
- Advanced project dashboard analytics

## User Pass/Fail Reporting Format

```markdown
# Milestone 7 Validation Results

## Overall Result

Pass or Fail

## Failed Items

- Item:
  - Expected:
  - Actual:

## Notes

Any extra observations.
```
