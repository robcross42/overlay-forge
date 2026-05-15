# Milestone 0 - Overlay Shell Validation

## Status

**Complete / Passed / Successful**

Milestone 0 is complete. The desktop overlay shell launches, hosts the initial component, initializes SQLite automatically, and persists scratchpad content between sessions.

The scratchpad component is also complete for Milestone 0. It can save user-entered text and restore that text after the app is restarted.

## Goal

Prove that a Tauri v2 desktop overlay shell can host a component and persist local data.

## Implemented Capabilities

- Tauri v2 shell
- React + TypeScript frontend
- Rust command backend
- Dark overlay UI
- Always-on-top main window configuration
- Rust-registered global hotkey toggle
- Custom draggable titlebar
- Custom minimize, maximize/restore, and hide controls
- Borderless overlay resize handles
- Component host layout
- Placeholder component
- Scratchpad editor
- SQLite database initialization
- Scratchpad save and restore commands

## Hotkey

```text
Ctrl+Shift+Space
```

## Validation Checklist

- Passed: App launches successfully.
- Passed: Global hotkey toggles the overlay open and closed.
- Passed: Overlay appears above normal desktop applications.
- Passed: Placeholder component renders inside the shell.
- Passed: Scratchpad text can be saved.
- Passed: Scratchpad text is restored after restarting the app.
- Passed: SQLite initializes without manual setup.

## Scratchpad Component Result

**Complete / Passed**

The Milestone 0 scratchpad is the first persisted component in Overlay Forge. It stores content in SQLite through Tauri backend commands and restores that content on the next app launch.

## Explicit Non-Goals

- Calendar UI
- To-do system
- Notes system beyond the scratchpad
- OpenAI API integration
- GitHub integration
- YouTube component
- Exclusive fullscreen game overlay support
- Multi-window workflows
- Plugin marketplace or external plugin loading
