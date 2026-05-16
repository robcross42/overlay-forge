# Overlay Forge

Overlay Forge is a local-first desktop overlay shell for planning, notes, tasks, calendar views, and future Codex bridge-file workflows.

**Milestone 0 is complete and passed.** The current app proves the overlay shell, local SQLite initialization, and scratchpad persistence workflow.

**Milestone 1 is complete, passed, and successful.** It adds Tasks, Notes, and Calendar components beside the existing Scratchpad without replacing the Milestone 0 foundation.

**Current project baseline: Milestone 3.** Future bridge prompts, planning, and implementation should treat Milestone 3 as the latest completed and user-validated app state.

**Milestone 2 is complete, passed, and successful.** It adds a local Projects component with SQLite persistence.

**Milestone 3 is complete, passed, and successful.** It adds a backend-mediated OpenAI Planning Chat component with local SQLite conversation and message persistence.

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

Future OpenAI, GitHub, YouTube, advanced calendar/tasks/notes, and full project-planning features are intentionally deferred.

Milestone 1 intentionally keeps OpenAI, GitHub, YouTube, cloud sync, recurring events, calendar invites, and external calendar integrations deferred.

Milestone 2 intentionally keeps OpenAI, GitHub, project import/export, planning chat, bridge-file generation UI, cloud sync, and advanced project lifecycle workflows deferred.

Milestone 3 intentionally keeps GitHub integration, YouTube, external calendar integrations, cloud sync, file upload/vector store workflows, web search tooling, full bridge-file generation UI, and automatic Codex handoff deferred.

## ChatGPT / Codex Bridge Context

When using this repository as context in ChatGPT or Codex, do not rely only on this README. The bridge should explicitly reference every project Markdown file in the repo, including files under `docs/`, because chatgpt.com may not automatically discover nested documentation.

Required Markdown context files:

- `README.md`
- `CHANGELOG.md`
- `bridge-files/OPENAI_APP_BRIDGE.md`
- `docs/ARCHITECTURE.md`
- `docs/BRIDGE_FILES.md`
- `docs/DATA_MODEL.md`
- `docs/MILESTONE_0.md`
- `docs/MILESTONE_1.md`
- `docs/MILESTONE_2.md`
- `docs/MILESTONE_3.md`
- `docs/PROJECT_PLAN.md`

For future bridge prompts, instruct ChatGPT/Codex to read all `*.md` files in the project repo structure before making planning or implementation decisions.

Milestone numbering note: use explicit milestone IDs from the Markdown files. Do not infer milestone numbers from numbered list positions. Milestone 3 is the OpenAI Planning Chat component and is complete, passed, and successful.

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

## OpenAI Planning Chat

Milestone 3 uses the backend environment variable:

```text
OPENAI_API_KEY
```

The key is read only by the Rust/Tauri backend. It is not stored in SQLite and is not exposed to React source code. If the key is missing, Planning Chat shows a readable configuration error when a message is sent.
