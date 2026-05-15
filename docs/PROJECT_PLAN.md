# Overlay Forge Project Plan

## Current Status

**Milestone 0 - Overlay Shell Validation is complete, passed, and successful.**

The app has proven a small, reliable desktop overlay shell before expansion into calendar, notes, project planning, OpenAI, GitHub, or YouTube workflows.

The Milestone 0 scratchpad component is also complete and passed. Scratchpad text persists locally in SQLite and restores between app sessions.

## Product Direction

Overlay Forge is a personal desktop command hub that floats above the user's workflow and eventually helps turn ideas, notes, tasks, and project plans into Codex-ready markdown bridge files.

## Milestone Order

1. Overlay shell validation
2. Calendar, to-do, notes, and scratchpad component
3. Local projects component
4. OpenAI planning chat component
5. GitHub integration
6. Controlled YouTube component

## Scope Guard

Milestone 0 is stable enough to serve as the foundation for later work. Do not implement later milestone features in the Milestone 0 code path; future work should begin from the completed shell, hotkey behavior, always-on-top behavior, component host, and local SQLite scratchpad.
