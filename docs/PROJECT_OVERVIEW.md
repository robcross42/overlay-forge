# Overlay Forge Project Overview

## Product Direction

Overlay Forge is a personal desktop command hub built as a local-first Tauri overlay. It floats above the user's workflow and organizes project planning, notes, tasks, references, game context, and focused utility modules.

The active coding workflow is direct Codex chat in VS Code. Repository Markdown files provide local project context and implementation rules.

## Current Baseline

Current stable project baseline:

```text
Milestone 13 - Project Workspace UI Consolidation
Status: Complete / Passed / Successful
```

Milestone 13 consolidates Projects around the left navigation hierarchy and focused chat surfaces. Project and conversation selection live in the navigation tree. Project details, GitHub configuration, and Markdown context settings live in Project Edit.

## Major Completed Areas

- Tauri v2 overlay shell.
- React + TypeScript frontend.
- Rust/Tauri backend command layer.
- SQLite local persistence.
- Scratchpad, Tasks, Notes, Calendar.
- Local Projects.
- Project-scoped chat.
- GitHub repository metadata linkage.
- User-curated YouTube references.
- Manual conversation context attachments.
- Prompt Preview.
- Local Markdown implementation request drafts.
- Project-level Markdown context.
- Consolidated Projects navigation tree and focused chat UI.
- Gaming workspace with screenshot capture.
- GearBlocks save decoding, runtime export import, parts catalog, and in-game script tooling.
- Smoking Cessation module.
- Scheduler framework.
- SQLite naming normalization.
- Path of Exile 2 game module scaffold.

## Active Documentation Structure

| File | Purpose |
| --- | --- |
| `README.md` | Repository entry point and current workflow. |
| `AGENTS.md` | Codex instructions and repo rules. |
| `CHANGELOG.md` | Date/time-stamped change history. |
| `.vscode/CODEX_INSTRUCTIONS.md` | VS Code quick reference. |
| `docs/PROJECT_OVERVIEW.md` | Current project direction and baseline. |
| `docs/PROJECT_HISTORY.md` | Condensed milestone history. |
| `docs/ARCHITECTURE.md` | Frontend/backend/module architecture. |
| `docs/DATA_MODEL.md` | SQLite schema and naming conventions. |
| `docs/FEATURE_SCOPE.md` | Scope boundaries and guardrails. |
| `docs/DEFERRED_ITEMS.md` | Centralized deferred work. |
| `docs/VALIDATION.md` | Build and manual validation expectations. |
| `docs/VERSIONING.md` | Semantic versioning and changelog rules. |
| `docs/GAMING_SCREENSHOTS.md` | Gaming screenshot workflow. |
| `docs/GEARBLOCKS.md` | GearBlocks module overview. |
| `docs/GEARBLOCKS_RUNTIME.md` | GearBlocks save/runtime data flow. |
| `docs/GEARBLOCKS_PLUGIN.md` | BepInEx, GearLib, and marker plugin boundaries. |
| `docs/GEARBLOCKS_PARTS_CATALOG.md` | Validated GearBlocks parts vocabulary. |
| `docs/SMOKING_CESSATION.md` | Smoking Cessation module scope. |

## Terminology

Use current names in new work:

| Current term | Meaning |
| --- | --- |
| Project Chat | Project-scoped OpenAI chat inside Overlay Forge. |
| Project Markdown Context | Project-level local Markdown files loaded from a configured root. |
| Conversation Context Attachments | Conversation-scoped links to local app records. |
| Implementation Request Draft | Local Markdown draft generated from project/chat/context data for user review. |
| GearBlocks Runtime Export | Runtime scene data reconstructed from GearBlocks script output. |

Do not reintroduce retired external-transfer terminology.
