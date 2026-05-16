# Milestone 1 - Calendar, To-do, Notes, and Scratchpad Expansion

## Status

**Complete / Passed / Successful**

Milestone 1 expands the completed Milestone 0 overlay shell into a local-first organizer with four components:

- Scratchpad
- Tasks
- Notes
- Calendar

User validation is complete. Scratchpad, Tasks, Notes, Calendar, persistence, navigation, and overlay shell regression checks passed.

The remaining Windows WebView2 shutdown class-unregister log is documented as deferred cleanup and is not a Milestone 1 blocker.

## Implemented Capabilities

- Component navigation for Scratchpad, Tasks, Notes, and Calendar.
- Active component visual state in the sidebar.
- Scratchpad regression path preserved.
- SQLite-backed task creation, list selection, editing, deletion, deadline storage, body storage, listing, and restart restore.
- SQLite-backed note creation, selection, editing, deletion, listing, and restart restore.
- SQLite-backed calendar event creation, editing, deletion, listing, and restart restore.
- Idempotent SQLite table initialization for `tasks`, `notes`, and `calendar_events`.
- Tauri backend CRUD commands for Milestone 1 data.
- Startup guard for already-registered global hotkey conflicts.
- Overlay starts hidden in the background and is shown with `Ctrl+Shift+Space`.
- Shutdown titlebar control exits the app process.

## Data Tables

```text
scratchpad
tasks
notes
calendar_events
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

## Overlay Shell Regression Validation

Validate:

```text
Ctrl+Shift+Space toggles the overlay open/closed.
```

Pass criteria:

```text
Overlay visibility changes when the hotkey is pressed.
```

Validate:

```text
Overlay appears above normal desktop applications.
```

Pass criteria:

```text
Overlay remains visible above at least one normal desktop app window.
```

Validate:

```text
The custom titlebar can drag the overlay window.
```

Pass criteria:

```text
Window moves when dragged from the custom titlebar.
```

Validate:

```text
Minimize, maximize/restore, and hide controls work.
```

Pass criteria:

```text
Each custom window control performs the expected action.
```

Validate:

```text
Edge/corner resize handles still work.
```

Pass criteria:

```text
Overlay can be resized from at least one edge and one corner.
```

## Component Navigation Validation

Validate:

```text
Navigation shows Scratchpad, Tasks, Notes, and Calendar.
```

Pass criteria:

```text
All four component entries are visible.
```

Validate:

```text
User can switch between Scratchpad, Tasks, Notes, and Calendar.
```

Pass criteria:

```text
Clicking each navigation entry displays the correct component.
```

Validate:

```text
Active component is visually identifiable.
```

Pass criteria:

```text
The currently selected component has a clear active state.
```

## Scratchpad Regression Validation

Validate:

```text
Enter text into the scratchpad and save it.
```

Pass criteria:

```text
Scratchpad accepts and saves user-entered text.
```

Validate:

```text
Restart the app and return to the scratchpad.
```

Pass criteria:

```text
Previously saved scratchpad text is restored.
```

## Tasks Validation

Validate:

```text
Open Tasks with no saved tasks.
```

Pass criteria:

```text
Only the New Task action is visible; title/body/deadline fields and save/delete controls are hidden.
```

Validate:

```text
Create a new task with title, body, and deadline.
```

Pass criteria:

```text
The task appears in the task list with its deadline visible.
```

Validate:

```text
Click an existing task in the task list.
```

Pass criteria:

```text
The task opens in selected/read-only mode with an Edit button.
```

Validate:

```text
Click Edit on a selected task.
```

Pass criteria:

```text
The selected task fields become editable and Save is available.
```

Validate:

```text
Edit the task title, body, and deadline.
```

Pass criteria:

```text
The updated task details are shown after saving.
```

Validate:

```text
Review the main task list.
```

Pass criteria:

```text
The main task list does not show checkboxes or direct delete buttons.
```

Validate:

```text
Delete the task from edit mode.
```

Pass criteria:

```text
The selected task is removed from the task list.
```

Validate:

```text
Create at least one task, restart the app, and return to Tasks.
```

Pass criteria:

```text
The task is restored after restart.
```

## Notes Validation

Validate:

```text
Open Notes with no saved notes.
```

Pass criteria:

```text
Only the New Note action is visible; title/body fields and save/delete controls are hidden.
```

Validate:

```text
Create a new note with a title and body.
```

Pass criteria:

```text
The note appears in the notes list and can be selected.
```

Validate:

```text
Edit the note title and body.
```

Pass criteria:

```text
Selecting an existing note shows an Edit button. Clicking Edit makes the fields editable and Save is available. The updated title and body are shown after saving.
```

Validate:

```text
Restart the app and return to Notes.
```

Pass criteria:

```text
The note title and body are restored after restart.
```

Validate:

```text
Delete the note.
```

Pass criteria:

```text
The note is removed from the notes list.
```

## Calendar Validation

Validate:

```text
Open Calendar with no saved events.
```

Pass criteria:

```text
Only the New Event action is visible; event fields and save/delete controls are hidden.
```

Validate:

```text
Create a calendar event with title, date/time, and notes.
```

Pass criteria:

```text
The event appears in the calendar/event list.
```

Validate:

```text
Click anywhere inside the Calendar start date, end date, start time, and end time fields.
```

Pass criteria:

```text
Each field opens its native date/time control without requiring the small right-side icon.
```

Validate:

```text
Modify the Calendar start date or start time.
```

Pass criteria:

```text
The end date or end time updates automatically.
```

Validate:

```text
Select an existing calendar event.
```

Pass criteria:

```text
Delete is visible only for the selected existing event and uses active button styling.
```

Validate:

```text
Edit the event.
```

Pass criteria:

```text
Selecting an existing event shows an Edit button. Clicking Edit makes the fields editable and Save is available. The updated event details are shown after saving.
```

Validate:

```text
Restart the app and return to Calendar.
```

Pass criteria:

```text
The event is restored after restart.
```

Validate:

```text
Delete the event.
```

Pass criteria:

```text
The event is removed from the calendar/event list.
```

## Persistence Validation

Validate:

```text
Create one scratchpad entry, one task, one note, and one calendar event.
Restart the app.
```

Pass criteria:

```text
All four data types restore correctly after restart.
```

Validate:

```text
Run the app against an existing Milestone 0 database.
```

Pass criteria:

```text
The app starts successfully, existing scratchpad data remains available, and new Milestone 1 tables initialize automatically.
```

## User Pass/Fail Reporting Format

```markdown
# Milestone 1 Validation Results

## Overall Result

Pass or Fail

## Failed Items

- Item:
  - Expected:
  - Actual:

## Notes

Any extra observations.
```

## Deferred Cleanup

- Investigate benign Windows WebView2 shutdown log:

```text
[ERROR:ui\gfx\win\window_impl.cc] Failed to unregister class Chrome_WidgetWin_0. Error = 1412
```

Current assessment: not a Milestone 1 blocker if the app process exits, data persists, and there is no visible crash dialog.
