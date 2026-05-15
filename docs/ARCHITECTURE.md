# Architecture

## Shell

The overlay shell owns the top-level layout, navigation, and component host. Feature modules should render inside the host instead of controlling window behavior directly.

## Frontend

The frontend is a React + TypeScript app organized around feature folders:

```text
src/
├─ app/
├─ components/
├─ features/
├─ services/
├─ styles/
└─ main.tsx
```

Milestone 0 is complete and includes only the shell and scratchpad feature. The scratchpad feature passed Milestone 0 validation by saving content to SQLite and restoring it between app sessions.

## Backend

The Tauri backend owns:

- SQLite initialization
- Scratchpad persistence commands
- Global hotkey registration
- Window show/hide behavior

## Persistence

SQLite is the local source of truth. The first schema contains a single-row `scratchpad` table. This Milestone 0 scratchpad persistence path is complete and passed. Later milestones should add tables for tasks, notes, calendar events, projects, bridge files, and planning chat history.

## Bridge Files

Bridge files are markdown documents used to keep ChatGPT and Codex aligned while the in-app OpenAI workflow is deferred.
