# Overlay Forge - OpenAI App Bridge

## Purpose

This file is the manual bridge between Overlay Forge project state and ChatGPT/Codex conversations.

Provide this file to ChatGPT or Codex when a new chat needs current project context. Keep it concise and update it after meaningful architecture, milestone, or validation changes.

## Current Milestone

Milestone 1 - Calendar, To-do, Notes, and Scratchpad Expansion

Status: **Complete / Passed / Successful**

Current project baseline: **Milestone 1**. Future bridge prompts, planning, and implementation should start from Milestone 1.

Milestone 0 remains complete, passed, and successful. Milestone 1 adds component navigation plus SQLite-backed Tasks, Notes, and Calendar components. User validation is complete and Milestone 1 passed.

Milestone numbering note: Milestone 2 is the Local Projects component and has not started. Do not mistake roadmap list item positions for milestone IDs.

## Current Scope

- Tauri v2 desktop app
- React + TypeScript frontend
- Rust backend
- Always-on-top overlay window
- Global hotkey toggle
- Custom draggable titlebar
- Custom minimize, maximize/restore, and hide controls
- Edge and corner resize handles for borderless overlay resizing
- Placeholder component host
- SQLite-backed scratchpad persistence
- SQLite-backed task persistence
- SQLite-backed note persistence
- SQLite-backed calendar event persistence

## Hotkey

```text
Ctrl+Shift+Space
```

## Deferred

- OpenAI API integration
- GitHub integration
- YouTube component
- Calendar UI
- To-do system
- Full notes system
- Local projects component
- Exclusive fullscreen game overlay support

## Latest Validation Notes

Update this section manually after each validation pass.

- Complete: dependency install
- Complete: frontend build with `npm run build`
- Complete: Rust dev compile with `cargo build`
- Complete: production Tauri bundle with `npm run tauri:build`
- Complete: compiled app launched briefly
- Complete: SQLite database created automatically at `%APPDATA%\com.overlayforge.desktop\overlay-forge.sqlite3`
- Complete: scratchpad persistence manually validated by user
- Complete: custom draggable titlebar and window controls build successfully
- Complete: edge/corner resize handle APIs build successfully
- Complete: fixed titlebar drag event interception so window control clicks can fire
- Complete: added explicit Tauri permissions for window control APIs
- Complete: rebuilt release app after window-control click fix
- Passed: Milestone 0 overlay shell validation
- Passed: Milestone 0 scratchpad component validation
- Passed: scratchpad save and restore between sessions
- Implemented: Milestone 1 component navigation
- Implemented: Tasks component and SQLite CRUD commands
- Implemented: Notes component and SQLite CRUD commands
- Implemented: Calendar component and SQLite CRUD commands
- Implemented: hotkey registration guard so app startup survives an already-registered hotkey
- Implemented: task body and deadline fields
- Implemented: task list selection/edit mode with no main-list checkbox or delete button
- Implemented: Notes empty state hides editor controls until a note exists
- Implemented: Calendar date/time field click behavior and automatic end date/time updates
- Implemented: Delete buttons visible only in edit/selected contexts with active styling
- Implemented: Tasks and Calendar empty states now match Notes editor reveal behavior
- Implemented: selected existing tasks show an explicit Edit button before fields become editable
- Implemented: selected existing notes and calendar events show an explicit Edit button before fields become editable
- Documented: organizer components should follow consistent empty-state/edit-mode/destructive-action patterns
- Implemented: overlay starts hidden in the background and is shown with `Ctrl+Shift+Space`
- Implemented: shutdown titlebar control exits the app process
- Deferred cleanup: investigate benign Windows WebView2 shutdown log `Failed to unregister class Chrome_WidgetWin_0. Error = 1412`
- Verified: `npm run tauri:build` passes after Milestone 1 implementation
- Verified: rebuilt release app starts against the existing app-data SQLite database
- Passed: Milestone 1 manual validation checklist
