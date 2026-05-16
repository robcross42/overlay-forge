# Milestone 8 - Projects Navigation Tree Actions

## Status

Planned

## Goal

Move project create/select/edit/delete entry points toward the left navigation shell so Projects behaves like an expandable workspace tree.

This milestone should prove the pattern on Projects only before applying it to Tasks, Notes, Calendar, YouTube, or other app modules.

## Design Direction

The left module navigation should become a navigable tree for modules that own saved records.

For Projects:

- The Projects module row can expand and collapse.
- Saved projects appear as children under Projects.
- Selecting a project child opens that project workspace.
- Hovering or focusing the Projects module row reveals a compact `+` action for creating a project.
- Hovering or focusing a project child reveals a compact `...` action for project-specific actions.
- The `...` action opens an action menu with Edit and Delete.
- The same actions must remain keyboard reachable; hover-only controls are not sufficient.

## In Scope

- Refine the left shell navigation for Projects only.
- Load saved projects into the navigation tree.
- Add a Projects module `+` action that starts the new project flow.
- Add per-project `...` actions for Edit and Delete.
- Keep selected project context stable when selecting projects from the nav tree.
- Keep the selected project workspace sections from Milestone 7: Overview, GitHub, Chat, References.
- Move project create/edit/delete triggers out of the main workspace surface where practical.
- Preserve existing SQLite project data and behavior.

## Out Of Scope

- Applying the pattern to Tasks, Notes, Calendar, or YouTube.
- Redesigning the YouTube component.
- Moving task/note/calendar create/edit/delete actions.
- Manual context attachments.
- Prompt preview.
- Bridge-file generation.
- GitHub file browsing.
- Codex handoff.
- ChatGPT import.
- Chat streaming.
- Model picker UI.
- Advanced navigation customization.

## UX Notes

Milestone 8 should use compact symbolic actions in the navigation:

- `+` for creating a new project.
- `...` for project item actions.

Actions should appear on hover and keyboard focus, similar to modern chat/workspace navigation patterns, but they must also be usable without a mouse.

The workspace area should focus on selected project content and section workflows. The navigation tree should own object-level project actions.

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
Navigate to Projects in the left navigation.
```

Pass criteria:

```text
Projects expands to show saved projects.
```

Validate:

```text
Hover or focus the Projects navigation row.
```

Pass criteria:

```text
A compact New Project action appears and can be activated.
```

Validate:

```text
Create a project from the Projects navigation action.
```

Pass criteria:

```text
The project creation flow starts and the saved project appears under Projects.
```

Validate:

```text
Select a project from the navigation tree.
```

Pass criteria:

```text
The selected project workspace opens with Overview, GitHub, Chat, and References.
```

Validate:

```text
Hover or focus a project row.
```

Pass criteria:

```text
A compact project action menu appears.
```

Validate:

```text
Use the project action menu to edit a project.
```

Pass criteria:

```text
The edit flow opens for the selected project and saved changes persist.
```

Validate:

```text
Use the project action menu to delete a project.
```

Pass criteria:

```text
The project is deleted locally and removed from the navigation tree.
```

Validate:

```text
Switch between several projects from the navigation tree.
```

Pass criteria:

```text
Selected project context updates correctly and workspace sections remain stable.
```

Validate:

```text
Use keyboard navigation to reach New Project and project action menus.
```

Pass criteria:

```text
Project actions are usable without relying on mouse hover only.
```

Validate:

```text
Return to Scratchpad, Tasks, Notes, Calendar, YouTube, and existing window controls.
```

Pass criteria:

```text
Existing Milestone 0 through Milestone 7 behavior still works.
```

## Deferred Follow-Up

After Projects navigation tree actions are validated, later milestones can consider extending the same pattern to:

- Notes
- Tasks
- Calendar events
- YouTube library groups or playlists

Do not generalize until the Projects version feels correct.
