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
- Global hotkey registration
- Window show/hide behavior

## Persistence

SQLite is the local source of truth. The first schema contains a single-row `scratchpad` table. This Milestone 0 scratchpad persistence path is complete and passed.

Milestone 1 adds idempotent table initialization for `tasks`, `notes`, and `calendar_events`.

Milestone 2 adds idempotent table initialization for `projects`.

Milestone 3 adds idempotent table initialization for `planning_conversations` and `planning_messages`. Later milestones should add tables for bridge file drafts and exported bridge-file workflow state.

## OpenAI Boundary

Planning Chat calls the OpenAI Responses API from the Rust/Tauri backend. React invokes local Tauri commands only and never reads `OPENAI_API_KEY`. Model selection, request shape, and the planning assistant instruction are centralized in the backend OpenAI service module so later bridge-file generation, tools, streaming, or model changes do not leak through the frontend.

## Bridge Files

Bridge files are markdown documents used to keep ChatGPT and Codex aligned while the in-app OpenAI workflow is deferred.
