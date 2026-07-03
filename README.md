# Overlay Forge

Overlay Forge is a local-first Tauri desktop overlay for planning, calendar items, game context, and focused utility modules.

The active coding workflow is direct Codex chat in VS Code. Repository Markdown files provide local context and implementation rules for that workflow.

## Current Status

```text
Current stable app release: 0.9.0
Status: Active local-first desktop command hub
```

Overlay Forge is now maintained as an evolving local-first command hub. Current stable capabilities include functional GearBlocks build guides, chat-to-guide generation, official API indexing, standalone overlay-window behavior, Smoking Cessation tracking, and Repair Resell planning.

## Core Capabilities

- Tauri v2 desktop overlay shell.
- React + TypeScript frontend.
- Rust/Tauri backend command layer.
- SQLite local persistence.
- Calendar as the visible main-shell organizer surface.
- Scratchpad, Tasks, and Notes code/data retained for later organizer consolidation review.
- Backend-owned OpenAI Responses API use through `OPENAI_API_KEY`.
- User-curated YouTube references.
- Gaming workspace and screenshot capture.
- GearBlocks save decoding, runtime export import, parts catalog, script tooling, and backlog BepInEx plugin templates.
- GearBlocks build guide import, chat-generated guide creation, in-game build guide overlay, and active guide chat context.
- Smoking Cessation module.
- Repair Resell module for local listing tracking, source registry, watchlist, deterministic flags, and manual deal estimates.
- Scheduler framework.
- SQLite naming normalization.
- Path of Exile 2 game module scaffold.
- Former Projects module removed from active code; legacy project/planning SQLite rows are retained for data preservation and future review.

## Documentation Map

| File | Purpose |
| --- | --- |
| `AGENTS.md` | Codex instructions and repository rules. |
| `.vscode/CODEX_INSTRUCTIONS.md` | VS Code quick reference. |
| `CHANGELOG.md` | Date/time-stamped change history. |
| `docs/VERSIONING.md` | Semantic versioning and changelog rules. |
| `docs/PROJECT_OVERVIEW.md` | Current project direction and active shape. |
| `docs/PROJECT_HISTORY.md` | Archived early project history. |
| `docs/ARCHITECTURE.md` | Frontend/backend/module ownership. |
| `docs/DATA_MODEL.md` | SQLite schema and naming conventions. |
| `docs/FEATURE_SCOPE.md` | Scope boundaries and guardrails. |
| `docs/DEFERRED_ITEMS.md` | Centralized deferred work. |
| `docs/VALIDATION.md` | Build and manual validation expectations. |
| `docs/GAMING_SCREENSHOTS.md` | Gaming screenshot workflow. |
| `docs/GEARBLOCKS.md` | GearBlocks module overview. |
| `docs/GEARBLOCKS_RUNTIME.md` | GearBlocks save/runtime data flow. |
| `docs/GEARBLOCKS_PLUGIN.md` | BepInEx, GearLib, and marker plugin boundaries. |
| `docs/GEARBLOCKS_PARTS_CATALOG.md` | Validated GearBlocks parts vocabulary. |
| `docs/SMOKING_CESSATION.md` | Smoking Cessation module scope. |
| `docs/REPAIR_RESELL.md` | Repair Resell restoration, pickup, and learning-path vision. |

## Development

Install dependencies:

```powershell
npm install
```

Run the Tauri app:

```powershell
npm run tauri:dev
```

The dev command writes a terminal transcript to:

```text
logs\tauri-dev\tauri-dev_YYYYMMDD_HHMMSS.log
```

`logs\tauri-dev\latest.txt` contains the path to the latest dev-session log.

Build the frontend:

```powershell
npm run build
```

Build the Rust/Tauri backend:

```powershell
cd src-tauri
cargo build
```

## Versioning

Overlay Forge uses semantic versioning in `MAJOR.MINOR.PATCH` form.

Do not increment the minor version just because a new chat, work session, or calendar day starts.

Use:

- `MAJOR` for incompatible or breaking release changes.
- `MINOR` for substantial new user-visible capabilities.
- `PATCH` for bug fixes, documentation-only changes, validation updates, small UX refinements, and internal refactors that do not introduce a major capability.

During `0.x` development, minor versions may still contain breaking early-development changes. Patch versions should remain non-breaking fixes, documentation, and small refinements.

See `docs/VERSIONING.md` for the full policy.

## Hotkeys

The main overlay toggle is registered in Rust as:

```text
Ctrl+Shift+Space
```

The Gaming chat overlay focus shortcut is registered in Rust as:

```text
Ctrl+Shift+C
```

Global shortcuts can be configured from Settings -> Keybinds. Each function uses `key1`, `key2`, and `key3` as the ordered parts of one shortcut, such as `Ctrl`, `Shift`, `Space`.

Mouse buttons are supported for shortcut parts, including `Mouse4`, `Mouse5`, and modifier combinations such as `Ctrl+Mouse4`.

The Smoking Cessation module adds a `Record Cigarette` shortcut action. It starts unassigned and can be mapped from Settings.

## Local Data

The SQLite database is created automatically in the app data directory as:

```text
overlay-forge.sqlite3
```

Generated local screenshots, capture manifests, dev logs, plugin working copies, and third-party DLLs should remain outside committed source unless a document explicitly says otherwise.

## Environment Variables

Project chat uses:

```text
OPENAI_API_KEY
```

GitHub repository metadata fetches use:

```text
GITHUB_TOKEN
```

Both tokens are read only by the Rust/Tauri backend. They are not stored in SQLite and are not exposed to React/frontend code.

## Current Workflow

Use Codex chat directly in VS Code against the repository.

Before implementation work, read `AGENTS.md` and the smallest relevant set of supporting docs from `docs/`.
