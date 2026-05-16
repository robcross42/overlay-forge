# Overlay Forge Project Plan

## Current Status

**Current project baseline: Milestone 3.**

Milestone 3 is the latest completed, tested, and user-validated milestone. Future planning and implementation should start from the Milestone 3 app state.

**Milestone 0 - Overlay Shell Validation is complete, passed, and successful.**

The app has proven a small, reliable desktop overlay shell before expansion into calendar, notes, project planning, OpenAI, GitHub, or YouTube workflows.

The Milestone 0 scratchpad component is also complete and passed. Scratchpad text persists locally in SQLite and restores between app sessions.

**Milestone 1 - Calendar, To-do, Notes, and Scratchpad Expansion is complete, passed, and successful.**

Milestone 1 adds component navigation for Scratchpad, Tasks, Notes, and Calendar, with local SQLite persistence for each data type.

**Milestone 2 - Local Projects component is complete, passed, and successful.**

Milestone 2 adds Projects navigation, a SQLite-backed local project table, Rust/Tauri project CRUD commands, and a Projects component using the same selected/read-only/edit interaction pattern as the organizer components.

**Milestone 3 - OpenAI Planning Chat component is complete, passed, and successful.**

Milestone 3 adds Planning Chat navigation, project-scoped planning conversations, SQLite-backed message persistence, and backend-only OpenAI Responses API calls through `OPENAI_API_KEY`.

## Product Direction

Overlay Forge is a personal desktop command hub that floats above the user's workflow and eventually helps turn ideas, notes, tasks, and project plans into Codex-ready markdown bridge files.

## Milestone Order

Use explicit milestone IDs. Do not infer milestone numbers from this list's item positions.

- Milestone 0 - Overlay shell validation - complete and passed
- Milestone 1 - Calendar, to-do, notes, and scratchpad component - complete and passed
- Milestone 2 - Local projects component - complete and passed
- Milestone 3 - OpenAI planning chat component - complete and passed
- Milestone 4 - GitHub integration - not started
- Milestone 5 - Controlled YouTube component - not started

## Scope Guard

Milestone 3 is the current passed stable baseline for later work. Do not implement later milestone features by reverting to the Milestone 0, Milestone 1, or Milestone 2 code path; future work should begin from the completed overlay shell, hotkey behavior, always-on-top behavior, component host, local SQLite scratchpad, Tasks, Notes, Calendar, Projects, and Planning Chat components.
