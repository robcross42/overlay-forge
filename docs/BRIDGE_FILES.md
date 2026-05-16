# Bridge Files

Bridge files are markdown handoff documents that describe project state, constraints, and implementation requests for Codex.

Milestone 3 introduces an OpenAI Planning Chat foundation, but full bridge-file generation is still deferred. Bridge files should still be created and updated manually, then pasted into ChatGPT or Codex chats when context needs to move between sessions.

## Current Manual Bridge

Use:

```text
bridge-files/OPENAI_APP_BRIDGE.md
```

## Current Project State

Milestone 0 is complete, passed, and successful.

The Milestone 0 scratchpad component is complete and passed. Scratchpad content is persisted in SQLite and restored between sessions.

Milestone 1 is complete, passed, and successful.

Current project baseline is Milestone 3. Future bridge files should instruct ChatGPT/Codex to start from the Milestone 3 app state.

Milestone 3 is complete, passed, and successful. It adds local planning conversations/messages and backend OpenAI Responses API calls, but it does not generate bridge files yet.

Milestone numbering note: Milestone 2 is the Local Projects component and is complete, passed, and successful. Milestone 3 is the OpenAI Planning Chat component. Do not infer milestone IDs from numbered-list positions.

## Intended Future Workflow

1. Select a local project in Overlay Forge.
2. Plan with the in-app OpenAI chat component.
3. Generate a Codex-ready markdown bridge file in a later milestone.
4. Save the bridge file in SQLite.
5. Export the bridge file to disk.
6. Paste or provide the bridge file to Codex for implementation.

## Status Values Planned For Later Milestones

```text
DRAFT
READY_FOR_CODEX
SENT_TO_CODEX
IMPLEMENTED
VALIDATED
ARCHIVED
```
