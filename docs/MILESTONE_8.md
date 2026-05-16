# Milestone 8 - Projects Navigation Tree Actions

## Status

Complete / Passed / Successful

## Goal

Move project create, select, edit, and delete entry points into the left navigation shell while preserving the Milestone 7 selected-project workspace.

Milestone 8 validates the navigation-tree pattern on Projects only before applying similar patterns to Tasks, Notes, Calendar, YouTube, or other modules.

## Implemented Capabilities

- Projects is now an expandable and collapsible module row in the left navigation shell.
- Saved projects appear as child rows under Projects.
- The Projects module row includes a compact `+` action for starting the new project flow.
- Project child rows include compact `...` action menus with Edit and Delete actions.
- Selecting a project child row sets the active workspace project.
- The selected project remains visually clear in the nav tree.
- Project delete actions use confirmation before deleting local project data.
- Deleting the active project clears the active workspace cleanly.
- The Milestone 7 selected-project workspace layout is preserved.
- Overview, GitHub, Chat, and References remain selected-project workspace sections.
- Project-scoped Chat continues to use existing planning conversation and message persistence.
- Project-scoped GitHub repository linkage and metadata behavior remain unchanged.
- Existing Scratchpad, Tasks, Notes, Calendar, and YouTube navigation remains unchanged.

## Data Tables

Milestone 8 does not add or change SQLite tables.

It continues to use the existing tables:

```text
projects
project_github_repositories
planning_conversations
planning_messages
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
Navigate to the left navigation shell.
```

Pass criteria:

```text
Projects appears as a module row with an expandable tree pattern.
```

Validate:

```text
Expand Projects.
```

Pass criteria:

```text
Saved projects appear as child rows.
```

Validate:

```text
Select a project child row.
```

Pass criteria:

```text
The selected project becomes the active workspace and its workspace sections appear.
```

Validate:

```text
Switch between Overview, GitHub, Chat, and References for the selected project.
```

Pass criteria:

```text
Each section works and selected project context remains stable.
```

Validate:

```text
Click the Projects + action and create a new project.
```

Pass criteria:

```text
The project saves successfully and appears as a child row under Projects.
```

Validate:

```text
Use a project row ... menu to edit a project.
```

Pass criteria:

```text
Project changes save successfully and the nav tree updates.
```

Validate:

```text
Use a project row ... menu to delete a project.
```

Pass criteria:

```text
Delete uses confirmation, the project is removed, and the app remains stable.
```

Validate:

```text
Select different projects from the nav tree.
```

Pass criteria:

```text
Each project loads its own workspace context.
```

Validate:

```text
Open Chat for two different projects.
```

Pass criteria:

```text
Each project keeps separate chat conversations/messages.
```

Validate:

```text
Open GitHub for a linked project.
```

Pass criteria:

```text
Existing GitHub repository linkage and metadata behavior still works.
```

Validate:

```text
Restart the app and return to Projects.
```

Pass criteria:

```text
Project list, selected-project behavior, workspace layout, GitHub data, and Chat data restore correctly.
```

Validate:

```text
Return to Scratchpad, Tasks, Notes, Calendar, YouTube, and existing app controls.
```

Pass criteria:

```text
Existing Milestone 0 through Milestone 7 behavior still works.
```

## Deferred Items

- Navigation tree refactor for Tasks
- Navigation tree refactor for Notes
- Navigation tree refactor for Calendar
- Navigation tree refactor for YouTube
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
# Milestone 8 Validation Results

## Overall Result

Pass or Fail

## Failed Items

- Item:
  - Expected:
  - Actual:

## Notes

Any extra observations.
```
