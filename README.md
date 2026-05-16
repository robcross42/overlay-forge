# Overlay Forge

Overlay Forge is a local-first desktop overlay shell for planning, notes, tasks, calendar views, and future Codex bridge-file workflows.

**Milestone 0 is complete and passed.** The current app proves the overlay shell, local SQLite initialization, and scratchpad persistence workflow.

**Milestone 1 is complete, passed, and successful.** It adds Tasks, Notes, and Calendar components beside the existing Scratchpad without replacing the Milestone 0 foundation.

**Current project baseline: Milestone 6.** Future bridge prompts, planning, and implementation should treat Milestone 6 as the latest completed and user-validated app state.

**Milestone 2 is complete, passed, and successful.** It adds a local Projects component with SQLite persistence.

**Milestone 3 is complete, passed, and successful.** It adds a backend-mediated OpenAI Planning Chat component with local SQLite conversation and message persistence.

**Milestone 4 - GitHub Integration** is complete, passed, and successful. It adds project-scoped GitHub repository linkage, backend-only `GITHUB_TOKEN` metadata fetches, and local SQLite storage for repository metadata/status.

**Milestone 5 - Controlled YouTube Component**

Status: **Complete / Passed / Successful**

Milestone 5 adds a local-first YouTube component for intentionally saved, user-curated video references. It stores references in SQLite, validates common YouTube URL shapes, and opens saved URLs externally in the system browser without requiring a YouTube API key or YouTube account login.

**Milestone 6 - Project Workspace Chat**

Status: **Complete / Passed / Successful**

Milestone 6 moves project-scoped chat into the Projects section. Selecting a project now establishes the active project workspace, with Overview, GitHub, and Chat sections scoped to that selected project. The Chat section reuses existing `planning_conversations` and `planning_messages` data and no longer requires a second project selector.

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

Advanced YouTube workflows, advanced calendar/tasks/notes, and full bridge-file/project-planning automation are intentionally deferred.

Milestone 1 intentionally keeps OpenAI, GitHub, YouTube, cloud sync, recurring events, calendar invites, and external calendar integrations deferred.

Milestone 2 intentionally keeps OpenAI, GitHub, project import/export, planning chat, bridge-file generation UI, cloud sync, and advanced project lifecycle workflows deferred.

Milestone 3 intentionally kept GitHub integration, YouTube, external calendar integrations, cloud sync, file upload/vector store workflows, web search tooling, full bridge-file generation UI, and automatic Codex handoff deferred.

Milestone 4 intentionally keeps automatic Codex handoff, GitHub write operations, pull request creation, branch creation, issue management, repository file browsing, GitHub Actions integration, OAuth, multi-account support, advanced sync, vector store/repo indexing, YouTube, external calendar integrations, cloud sync, and multi-user auth deferred.

Milestone 5 intentionally keeps YouTube account login, YouTube API integration, OAuth, subscription import, watch history import, recommendations, transcripts, summarization, video/audio downloads, playlist sync, comment sync, scraping, unrestricted embedded browsing, background crawlers, bridge-file generation from videos, cloud sync, and multi-user auth deferred.

Milestone 6 intentionally keeps bridge-file generation, prompt preview, automatic context attachment, GitHub file reading, Codex handoff, ChatGPT import, conversation search/filtering, chat streaming, and model picker UI deferred.

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
- `docs/MILESTONE_4.md`
- `docs/MILESTONE_5.md`
- `docs/MILESTONE_6.md`
- `docs/PROJECT_PLAN.md`

For future bridge prompts, instruct ChatGPT/Codex to read all `*.md` files in the project repo structure before making planning or implementation decisions.

Milestone numbering note: use explicit milestone IDs from the Markdown files. Do not infer milestone numbers from numbered list positions. Milestone 3 is the OpenAI Planning Chat component and is complete, passed, and successful. Milestone 4 is GitHub Integration and is complete, passed, and successful. Milestone 5 is the Controlled YouTube Component and is complete, passed, and successful. Milestone 6 is Project Workspace Chat and is complete, passed, and successful.

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

## Project Workspace Chat

Milestone 6 makes Projects the primary workspace for project-scoped chat. Open Projects, select a project, then use the Chat section inside that selected project workspace. Chat data continues to persist through the Milestone 3 planning chat tables.

The OpenAI planning chat backend uses the environment variable:

```text
OPENAI_API_KEY
```

The key is read only by the Rust/Tauri backend. It is not stored in SQLite and is not exposed to React source code. If the key is missing, Planning Chat shows a readable configuration error when a message is sent.

## GitHub Integration

Milestone 4 uses the backend environment variable:

```text
GITHUB_TOKEN
```

The token is read only by the Rust/Tauri backend. It is not stored in SQLite and is not exposed to React source code. SQLite stores project repository linkage and fetched metadata/status only. If the token is missing, the Projects GitHub section shows a readable configuration error when metadata fetch is attempted.

## YouTube References

Milestone 5 does not use a YouTube API key, YouTube login, scraping, recommendations, transcripts, downloads, or account sync. Users intentionally create local references with a title, YouTube URL, and optional user-entered metadata. Saved URLs open externally in the system browser.
