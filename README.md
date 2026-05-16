# Overlay Forge

Overlay Forge is a local-first desktop overlay shell for planning, notes, tasks, calendar views, and future Codex bridge-file workflows.

**Milestone 0 is complete and passed.** The current app proves the overlay shell, local SQLite initialization, and scratchpad persistence workflow.

**Milestone 1 is complete, passed, and successful.** It adds Tasks, Notes, and Calendar components beside the existing Scratchpad without replacing the Milestone 0 foundation.

Completed Milestone 0 capabilities:

- Tauri v2 desktop shell
- React + TypeScript frontend
- Rust backend commands
- Always-on-top dark overlay window
- Global hotkey toggle
- Draggable borderless titlebar
- Custom minimize, maximize/restore, and hide controls
- Edge and corner resizing
- Placeholder component host
- SQLite-backed scratchpad persistence
- Manual Markdown bridge files for ChatGPT/Codex alignment

The scratchpad component is complete for Milestone 0: entered text saves locally and restores between app sessions.

Future OpenAI, GitHub, YouTube, calendar, tasks, notes, and project-planning features are intentionally deferred.

Milestone 1 intentionally keeps OpenAI, GitHub, YouTube, cloud sync, recurring events, calendar invites, and external calendar integrations deferred.

## Development

Install dependencies:

```powershell
npm install
```

Run the Tauri app:

```powershell
npm run tauri:dev
```

Build the frontend:

```powershell
npm run build
```

## Hotkey

The Milestone 0 overlay toggle is registered in Rust as:

```text
Ctrl+Shift+Space
```

## Local Data

The SQLite database is created automatically in the app data directory as:

```text
overlay-forge.sqlite3
```
