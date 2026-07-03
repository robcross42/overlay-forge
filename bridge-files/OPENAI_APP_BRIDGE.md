# Overlay Forge Bridge Context

## Purpose

This file is a compact local context summary for external review. Current workflow is direct Codex chat in VS Code against this repository; repository Markdown remains local project context.

Future changelog and versioning updates must follow `AGENTS.md`. `docs/PROJECT_HISTORY.md` holds archived early project history only.

## Current App Shape

Overlay Forge is a local-first Tauri v2 desktop command hub with a React + TypeScript frontend, Rust/Tauri backend commands, and SQLite persistence. The shell supports always-on-top desktop use, configurable hotkeys, custom window controls, game context, and utility modules.

## Current Capabilities

- Calendar as the visible main-shell organizer surface.
- Scratchpad, Tasks, and Notes code/data retained for later organizer consolidation review.
- Backend-owned OpenAI Responses API calls through `OPENAI_API_KEY`.
- User-curated YouTube references.
- Gaming workspace with GearBlocks and Path of Exile 2 sections.
- GearBlocks save decoding, runtime export import, API indexing, parts catalog, build guides, script tooling, and in-game guide overlay.
- Gaming screenshot capture, thumbnail preview, chat attachment flow, and delete cleanup.
- Smoking Cessation tracking with local cigarette events, inventory, patch marker, keybind support, and narrow Markdown export.
- Repair Resell planning with local listing tracking, source registry, watchlist, deterministic flags, manual estimates, and future repair-learning vision.
- Scheduler framework for explicit backend-owned recurring work.
- SQLite naming normalization using `obj_`, `def_`, `o2o_`, and `n2n_` table families.
- Former Projects module removed from active code; legacy project/planning SQLite rows are retained for preservation and future review.

## Boundaries

- Preserve the local-first design.
- Keep OpenAI and GitHub token usage backend-owned.
- Treat SQLite as the source of truth for persisted app data.
- Make migrations non-destructive and idempotent.
- Do not introduce arbitrary command execution through SQLite, scheduler rows, Lua payloads, or user-editable config.
- Do not add external transfer workflow language to UI or active docs.

## Deferred

Deferred work is centralized in `docs/DEFERRED_ITEMS.md`. Scope boundaries are in `docs/FEATURE_SCOPE.md`.

## Validation

Use `docs/VALIDATION.md` for current validation expectations. Typical broad validation is:

```powershell
npm run build
cd src-tauri
cargo build
```
