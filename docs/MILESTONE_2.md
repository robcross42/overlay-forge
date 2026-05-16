# Milestone 2 - Local Projects Component

## Status

Complete / Passed / Successful

The user manually validated Milestone 2 successfully.

## Goal

Add a local-first Projects component that establishes the project foundation for later bridge-file, planning-chat, OpenAI, and GitHub workflows.

## Implemented Capabilities

- Added Projects to the overlay component navigation.
- Added a Projects feature component under `src/features/projects/`.
- Added frontend project service functions under `src/services/projects.ts`.
- Added SQLite-backed project persistence.
- Added Rust/Tauri commands for listing, creating, updating, and deleting projects.
- Fixed Projects status dropdown option readability.
- Preserved Scratchpad, Tasks, Notes, and Calendar behavior from Milestone 1.

## Projects Data Table

```text
projects
- id
- name
- description
- status
- created_at
- updated_at
```

Milestone 2 valid status values:

```text
ACTIVE
ARCHIVED
```

## UI Behavior

- Empty Projects state shows only the primary New Project action and the empty-state message.
- Editor fields remain hidden until the user creates or selects a project.
- Selecting an existing project opens it in selected/read-only mode.
- Selected existing projects show an explicit Edit button.
- Edit makes project fields editable.
- Save is available while creating or editing.
- Delete is available only for a selected existing project.
- The main project list does not show direct delete buttons.

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
Overlay appears using the existing hotkey behavior.
```

Validate:

```text
Navigation shows Scratchpad, Tasks, Notes, Calendar, and Projects.
```

Pass criteria:

```text
All five component entries are visible.
```

Validate:

```text
Open Projects with no saved projects.
```

Pass criteria:

```text
Only the New Project action is visible; project editor fields and save/delete controls are hidden.
```

Validate:

```text
Create a new project with name, description, and status.
```

Pass criteria:

```text
The project appears in the project list.
```

Validate:

```text
Select the project from the project list.
```

Pass criteria:

```text
The project opens in selected/read-only mode with an Edit button.
```

Validate:

```text
Click Edit.
```

Pass criteria:

```text
The project fields become editable and Save is available.
```

Validate:

```text
Edit the project name, description, and status.
```

Pass criteria:

```text
The updated project details are shown after saving.
```

Validate:

```text
Review the main project list.
```

Pass criteria:

```text
The main project list does not show direct delete buttons.
```

Validate:

```text
Restart the app and return to Projects.
```

Pass criteria:

```text
The saved project is restored from SQLite.
```

Validate:

```text
Delete the project from the selected/edit context.
```

Pass criteria:

```text
The project is removed from the project list.
```

Validate:

```text
Return to Scratchpad, Tasks, Notes, and Calendar.
```

Pass criteria:

```text
Existing Milestone 1 components still work and persisted data remains available.
```

## User Pass/Fail Reporting Format

```markdown
# Milestone 2 Validation Results

## Overall Result

Pass or Fail

## Failed Items

- Item:
  - Expected:
  - Actual:

## Notes

Any extra observations.
```

## Deferred Items

- OpenAI API integration
- Chat interface
- GitHub integration
- YouTube component
- Bridge-file generation UI
- External sync
- Cloud auth
- Project import/export
- Full project planning workflow
- Advanced project search/filtering
