# Milestone 3 - OpenAI Planning Chat Component

## Status

Complete / Passed / Successful

The user manually validated Milestone 3 successfully.

## Goal

Add a local-first OpenAI planning chat component that lets the user select a local project and conduct a planning conversation related to that project.

Milestone 3 establishes the foundation for later bridge-file generation but does not implement the full bridge-file workflow.

## Implemented Capabilities

- Added Planning Chat to the overlay component navigation.
- Added a Planning Chat feature component under `src/features/planning-chat/`.
- Added frontend planning chat service functions under `src/services/planningChat.ts`.
- Added project selector using existing local Projects records.
- Added project-scoped conversation list.
- Added New Conversation action.
- Added message history display.
- Added message input and Send action.
- Added loading state while waiting for an assistant response.
- Added readable setup/request error display through the panel status.
- Added backend-only OpenAI Responses API calls through Rust/Tauri.
- Added `OPENAI_API_KEY` environment variable handling in the backend.
- Added backend-owned default model configuration and planning assistant instruction.
- Added SQLite-backed planning conversation and message persistence.
- Preserved Scratchpad, Tasks, Notes, Calendar, and Projects behavior.

## Data Tables

```text
planning_conversations
- id
- project_id
- title
- created_at
- updated_at
```

```text
planning_messages
- id
- conversation_id
- role
- content
- created_at
```

Valid `planning_messages.role` values:

```text
user
assistant
system
```

## API Key Setup

Milestone 3 reads the OpenAI key from:

```text
OPENAI_API_KEY
```

The key is not stored in SQLite and is not exposed to React/frontend code. If the key is missing, the backend returns a readable configuration error and the app should not crash.

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
Navigation shows Scratchpad, Tasks, Notes, Calendar, Projects, and Planning Chat.
```

Pass criteria:

```text
All expected component entries are visible.
```

Validate:

```text
Open Planning Chat with no OPENAI_API_KEY configured.
```

Pass criteria:

```text
The UI shows a readable missing-configuration error after sending a message and the app does not crash.
```

Validate:

```text
Configure OPENAI_API_KEY, restart the app, select an existing project, and create a new planning conversation.
```

Pass criteria:

```text
A new conversation is created and appears in the conversation list.
```

Validate:

```text
Send a planning message.
```

Pass criteria:

```text
The user message is shown, an assistant response is returned, and both messages remain visible.
```

Validate:

```text
Restart the app and return to the selected project's planning conversation.
```

Pass criteria:

```text
The conversation and messages are restored from SQLite.
```

Validate:

```text
Return to Scratchpad, Tasks, Notes, Calendar, and Projects.
```

Pass criteria:

```text
Existing Milestone 0, Milestone 1, and Milestone 2 components still work and persisted data remains available.
```

## Deferred Items

- GitHub integration
- YouTube component
- External calendar integration
- Cloud sync
- Multi-user auth
- Full bridge-file generation UI
- Automatic Codex handoff
- File upload/vector store workflows
- Web search tooling
- Model picker UI
- Streaming responses
- Advanced conversation search/filtering
- Project import/export

## User Pass/Fail Reporting Format

```markdown
# Milestone 3 Validation Results

## Overall Result

Pass or Fail

## Failed Items

- Item:
  - Expected:
  - Actual:

## Notes

Any extra observations.
```
