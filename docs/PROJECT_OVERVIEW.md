# Overlay Forge Project Overview

## Product Direction

Overlay Forge is a personal desktop command hub built as a local-first Tauri overlay. It floats above the user's workflow and organizes planning, calendar items, references, game context, and focused utility modules.

The active coding workflow is direct Codex chat in VS Code. Repository Markdown files provide local project context and implementation rules.

## Current Project Shape

Current stable project status:

```text
Overlay Forge 0.9.0
Status: Active local-first desktop command hub
```

Overlay Forge organizes calendar items, references, game context, and focused utility modules inside a local-first desktop shell. The retired Projects module is no longer part of the active shell or command surface.

## Major Completed Areas

- Tauri v2 overlay shell.
- React + TypeScript frontend.
- Rust/Tauri backend command layer.
- SQLite local persistence.
- Calendar as the visible main-shell organizer surface.
- Scratchpad, Tasks, and Notes code/data retained for later organizer consolidation review.
- User-curated YouTube references.
- Gaming workspace with screenshot capture.
- GearBlocks save decoding, runtime export import, parts catalog, and in-game script tooling.
- Smoking Cessation module.
- Repair Resell module for local buy-repair-resell listing tracking, source registry, watchlist, deterministic keyword flags, and manual estimates.
- Scheduler framework.
- SQLite naming normalization.
- Path of Exile 2 game module scaffold and local build records.
- The Spell Brigade game module scaffold for chats, screenshots, and co-op run planning.
- Former Projects module removed from active code while legacy project/planning SQLite data remains preserved.

## Active Documentation Structure

| File | Purpose |
| --- | --- |
| `README.md` | Repository entry point and current workflow. |
| `AGENTS.md` | Codex instructions and repo rules. |
| `CHANGELOG.md` | Date/time-stamped change history. |
| `.vscode/CODEX_INSTRUCTIONS.md` | VS Code quick reference. |
| `docs/PROJECT_OVERVIEW.md` | Current project direction and active shape. |
| `docs/PROJECT_HISTORY.md` | Archived early project history. |
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
| `docs/REPAIR_RESELL.md` | Repair Resell restoration, pickup, and learning-path vision. |
| `docs/THE_SPELL_BRIGADE.md` | The Spell Brigade module scope and planning scaffold. |

## Terminology

Use current names in new work:

| Current term | Meaning |
| --- | --- |
| GearBlocks Runtime Export | Runtime scene data reconstructed from GearBlocks script output. |

Do not reintroduce retired external-transfer terminology.
