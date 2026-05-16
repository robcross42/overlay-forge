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

Milestone 0 is complete and includes the shell and scratchpad feature. The scratchpad feature passed Milestone 0 validation by saving content to SQLite and restoring it between app sessions.

Milestone 1 is complete, passed, and successful. It adds feature folders for Tasks, Notes, and Calendar while keeping all feature modules inside the shell-owned component host.

Milestone 2 is complete, passed, and successful. It adds a Projects feature folder while preserving the shell-owned component host and Milestone 1 organizer components.

Milestone 3 is complete, passed, and successful. It adds a Planning Chat feature folder inside the shell-owned component host while preserving Scratchpad, Tasks, Notes, Calendar, and Projects.

Milestone 4 - GitHub Integration is complete, passed, and successful. It extends the Projects feature with a project-scoped GitHub Repository section while preserving the existing shell-owned component host and all Milestone 0 through Milestone 3 components.

Milestone 5 - Controlled YouTube Component is complete, passed, and successful. It adds a YouTube feature folder inside the shell-owned component host while preserving Scratchpad, Tasks, Notes, Calendar, Projects, Planning Chat, and GitHub Repository behavior.

## UI Consistency

Organizer components should follow the same interaction pattern unless a milestone explicitly documents a reason to diverge:

- Empty components show the primary New action and keep editor fields hidden.
- New actions reveal the editor for the first item.
- Selecting an existing list item opens that item in selected/read-only mode.
- Selected existing items expose an explicit Edit action before fields become editable.
- Destructive actions are available only inside an edit/selected-item context.
- Active clickable actions use consistent enabled button styling across components.

## Backend

The Tauri backend owns:

- SQLite initialization
- Scratchpad persistence commands
- Task CRUD commands
- Note CRUD commands
- Calendar event CRUD commands
- Project CRUD commands
- Planning conversation and message CRUD commands
- Backend-only OpenAI Responses API request handling
- Project-scoped GitHub repository link commands
- Backend-only GitHub repository metadata fetch handling
- YouTube reference CRUD commands
- YouTube URL validation and external-open handling
- Global hotkey registration
- Window show/hide behavior

## Persistence

SQLite is the local source of truth. The first schema contains a single-row `scratchpad` table. This Milestone 0 scratchpad persistence path is complete and passed.

Milestone 1 adds idempotent table initialization for `tasks`, `notes`, and `calendar_events`.

Milestone 2 adds idempotent table initialization for `projects`.

Milestone 3 adds idempotent table initialization for `planning_conversations` and `planning_messages`. Later milestones should add tables for bridge file drafts and exported bridge-file workflow state.

Milestone 4 adds idempotent table initialization for `project_github_repositories`. The table stores project repository linkage and fetched metadata/status only. Migrations are non-destructive and must not remove existing Scratchpad, Tasks, Notes, Calendar, Projects, or Planning Chat data.

Milestone 5 adds idempotent table initialization for `youtube_references`. The table stores only user-created YouTube references and user-entered metadata. Migrations are non-destructive and must not remove existing Scratchpad, Tasks, Notes, Calendar, Projects, Planning Chat, or GitHub repository data.

## OpenAI Boundary

Planning Chat calls the OpenAI Responses API from the Rust/Tauri backend. React invokes local Tauri commands only and never reads `OPENAI_API_KEY`. Model selection, request shape, and the planning assistant instruction are centralized in the backend OpenAI service module so later bridge-file generation, tools, streaming, or model changes do not leak through the frontend.

## GitHub Boundary

Milestone 4 GitHub metadata fetches are backend-owned. React invokes local Tauri commands and never receives `GITHUB_TOKEN`. The token is read from the Rust process environment, is not stored in SQLite, and is not passed into frontend state.

SQLite stores repository linkage and fetched metadata/status only:

```text
project_id
repository_full_name
repository_url
default_branch
visibility
last_fetched_at
last_fetch_status
```

The integration is project-scoped and read-only. Milestone 4 does not perform Codex handoff, GitHub write operations, branch creation, commit creation, pull request creation, issue management, repository file browsing, GitHub Actions integration, OAuth, or multi-account workflows.

## YouTube Boundary

Milestone 5 YouTube references are local-first and user-curated. React invokes local Tauri commands to save, list, edit, delete, and open references. SQLite stores the title, URL, parsed video id, optional channel name, notes, tags, and timestamps.

No YouTube API key is required. No YouTube account login, OAuth flow, watch history, subscription import, playlist sync, comment sync, transcript extraction, recommendations, downloads, scraping, background metadata crawler, or account sync is used.

Saved YouTube URLs open externally in the system browser. This is preferred over an unrestricted embedded browser so the overlay workflow remains controlled.

## Bridge Files

Bridge files are markdown documents used to keep ChatGPT and Codex aligned while the in-app OpenAI workflow is deferred.
