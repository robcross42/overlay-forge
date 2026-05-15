# Bridge Files

Bridge files are markdown handoff documents that describe project state, constraints, and implementation requests for Codex.

Until Overlay Forge includes its own OpenAI planning module, bridge files should be created and updated manually, then pasted into ChatGPT or Codex chats when context needs to move between sessions.

## Current Manual Bridge

Use:

```text
bridge-files/OPENAI_APP_BRIDGE.md
```

## Current Project State

Milestone 0 is complete, passed, and successful.

The Milestone 0 scratchpad component is complete and passed. Scratchpad content is persisted in SQLite and restored between sessions.

## Intended Future Workflow

1. Select a local project in Overlay Forge.
2. Plan with the in-app OpenAI chat component.
3. Generate a Codex-ready markdown bridge file.
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
