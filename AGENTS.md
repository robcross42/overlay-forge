# Overlay Forge Codex Instructions

## Authority

This file is the top-level instruction file for Codex work in this repository.

Codex should read this file before planning, editing, validation, documentation updates, or commits. Supporting documentation lives under `docs/` and should be read when relevant to the requested change.

### Instruction Scope And Nesting

The root `AGENTS.md` applies to the entire repository. A nested `AGENTS.md` may add narrower instructions for its own directory subtree when that area genuinely needs local build, validation, generated-file, or domain rules.

When working in a subtree, read the instruction chain from the repository root through the target directory. Deeper instructions supplement the root rules and take precedence only where they explicitly conflict. Do not copy the full root file into nested instruction files, and do not add a nested `AGENTS.md` merely to point back to this file.

Keep cross-repository architecture, safety, documentation, versioning, validation, and commit rules here. Keep nested instructions limited to the owning subtree so the hierarchy remains useful and does not fragment global policy.

## Current Workflow

The user performs code requests directly from Codex chat in VS Code.

This repository documentation is used as local project context for Codex. Do not treat Markdown files as a separate transfer workflow. Do not create external request documents unless the user explicitly asks for one.

## Documentation Map

Read the smallest relevant set before editing:

| Work area | Read first |
| --- | --- |
| General project direction | `docs/PROJECT_OVERVIEW.md`, `docs/PROJECT_HISTORY.md` |
| Frontend/backend architecture | `docs/ARCHITECTURE.md` |
| SQLite tables or migrations | `docs/DATA_MODEL.md` |
| Scope boundaries or deferred work | `docs/FEATURE_SCOPE.md`, `docs/DEFERRED_ITEMS.md` |
| Validation expectations | `docs/VALIDATION.md` |
| Versioning or changelog updates | `docs/VERSIONING.md`, `CHANGELOG.md` |
| Gaming screenshots | `docs/GAMING_SCREENSHOTS.md` |
| Media Library | `docs/MEDIA_LIBRARY.md` |
| GearBlocks feature work | `docs/GEARBLOCKS.md`, then the focused GearBlocks docs |
| Smoking Cessation | `docs/SMOKING_CESSATION.md` |
| Repair Resell | `docs/REPAIR_RESELL.md` |
| The Spell Brigade | `docs/THE_SPELL_BRIGADE.md` |

This map is a routing index, not an inventory of every Markdown file. Add a Markdown file here only when it is an active, authoritative entry point or a required first read for a recurring work area. Supporting, historical, generated, vendor, template, bridge, and narrowly scoped README files should normally be linked from their owning parent document or discovered within their subtree instead of being listed here.

Preserve documentation nesting: route from this map to the broad owning document, then from that document to focused supporting docs. When adding a Markdown file, decide whether it changes task routing. If it does, update this map and the owning documentation index where useful; if it does not, keep the file discoverable from its nearest relevant parent without expanding the root map.

## Reasoning Effort

Default to **Medium** reasoning. Reasoning classification guides execution; it must not become a resubmission gate. Do not stop, pause, or ask the user to repeat a request solely because the active reasoning setting is above or below the level that would have been ideal.

Classify by complexity, uncertainty, and consequence rather than by subsystem name:

| Level | Typical use |
| --- | --- |
| **Light** | Narrow copy, constant, or isolated style changes when conserving usage is important. |
| **Medium** | Default for documentation, research, status reviews, commits and pushes, routine GitHub operations, ordinary fixes and features, and well-understood frontend, Rust, or SQLite work. |
| **High** | Difficult architecture decisions, security-sensitive work, complex migrations, broad refactors, or ambiguous cross-subsystem defects. |
| **Extra High** | Major multi-subsystem changes, risky data transformations, or uncertain reverse engineering. |
| **Ultra** | Exceptional data recovery, highly destructive migrations, serious security incidents, or unusually difficult system-wide work. |

Prefer completing the task over conserving tokens. When the active setting is lower than the ideal level:

1. Continue the task at the active setting with proportionate inspection, validation, and safety checks.
2. When useful, automatically delegate concrete, bounded analysis, implementation, or review subtasks to sub-agents configured with the higher reasoning effort. This repository explicitly authorizes that delegation for reasoning escalation; the primary agent remains responsible for integration, verification, and the final result.
3. If higher-effort delegation is unavailable or would add more coordination than value, proceed at the active setting instead of sending the user away to resubmit.

Do not downgrade or interrupt work when the active setting is higher than necessary. Stop only for a genuine blocker such as missing authority, an unavoidable user decision, unavailable required input, or a safety constraint. Reasoning-level mismatch by itself is never a blocker.

## Architecture And Abstraction Rules

Overlay Forge uses Tauri v2, React + TypeScript, Rust/Tauri commands, and SQLite. Use architecture patterns that fit this stack. Do not apply Java-style inheritance directly to Rust; use Rust-native composition, structs, enums, traits, services, repositories, and thin command handlers.

Do not solve defects or add features with one-off procedural patches when the issue involves reusable behavior, repeated state shape, duplicated validation, duplicated mapping, or inconsistent object handling. Duplicated behavior is a defect risk.

Before implementing a non-trivial feature or fix, identify:

1. The domain concept involved.
2. The abstraction that owns it.
3. Whether a new abstraction is required.
4. What regression-prone duplication this avoids.

If the change is small and does not need a new abstraction, state why. If a concept appears in three or more places, or if two places already diverged and caused a bug, create or extend a reusable abstraction.

Feature work must check whether the behavior belongs in an existing abstraction such as:

- window manager
- window config model
- window state repository
- module manager
- app settings service
- SQLite repository
- chat/session model
- screenshot/attachment model
- export service
- log ingestion service
- Tauri command service layer

If an appropriate abstraction exists, extend it. If none exists, create one before adding isolated call-site behavior.

### Frontend Architecture

React components should be function components with hooks. They should render UI and handle local interaction only. React must not own backend business rules, persistence rules, or Tauri window lifecycle behavior.

Use TypeScript interfaces or type aliases for plain DTOs. Use TypeScript classes when an object has both data and behavior, especially repeated construction, validation, normalization, serialization, deserialization, comparison, default values, state transitions, command payload shaping, SQLite row mapping, or UI view-model mapping.

Move repeated frontend utility behavior such as unknown-error formatting, timestamp labels, local storage key handling, Markdown cleanup, and command payload normalization into shared utilities or domain helpers instead of redefining it in each component.

### Rust And Tauri Architecture

Tauri command handlers must stay thin. They may receive input, validate input, call a service, repository, or domain method, and return a typed result. They must not manually construct complex domain objects inline, duplicate default configuration, duplicate SQLite access logic, own business rules, or contain large procedural feature implementations.

Use Rust `struct` plus `impl` for domain behavior, `enum` for finite variants, `trait` for shared behavior or interchangeable implementations, repository structs for SQLite persistence, service structs for business logic, and modules for domain boundaries.

Avoid long argument lists in commands, services, and repositories. When a command or repository method needs many related values, introduce a typed input, draft, options, or parameter struct so validation and mapping stay coherent.

Avoid large dumping-ground modules. When a Rust or TypeScript file accumulates multiple domains, split by feature, service, repository, parser, or platform boundary before adding more unrelated behavior.

### SQLite Architecture

Do not scatter SQL row mapping across the codebase. Each persisted domain concept should have one canonical mapping path between database rows, domain objects, database insert/update payloads, and frontend DTOs where needed.

Avoid duplicating column names, SQL fragments, and row conversion logic in unrelated files.

Database locks and other recoverable infrastructure failures should return typed errors through the existing result path rather than panicking in normal app operations.

### Window Architecture

Overlay Forge has a first-class window domain model. Do not create Tauri windows ad hoc in commands, React components, utility files, or one-off helpers.

All window creation, configuration, restoration, state persistence, and lifecycle behavior should route through centralized Rust window abstractions. Expected concepts are:

- `WindowKind`
- `BaseWindowConfig`
- `OverlayWindowConfig`
- `StandaloneWindowConfig`
- `WindowManager`
- `WindowStateRepository`

`WindowKind` should be an enum, not scattered strings. Window config should use Rust composition: `StandaloneWindowConfig` and `OverlayWindowConfig` compose shared `BaseWindowConfig`.

`WindowManager` should be the only place that creates, opens, closes, focuses, restores, or mutates Tauri windows. `WindowStateRepository` should be the only place that persists or restores window size, position, visibility, and related SQLite-backed state.

Before changing window behavior, inspect all existing window creation paths. If more than one file constructs windows, sets default options, generates labels, restores geometry, or handles standalone-window configuration, consolidate the shared path first.

### Regression Prevention

When fixing a bug, first check whether duplicated or inconsistent logic caused it. If yes, refactor the duplicated logic into a shared abstraction, update all call sites, add or update tests around the abstraction, and avoid leaving old duplicate logic behind.

For every non-trivial code change, include a short architecture note in the final response covering the domain concept, reusable abstraction added or reused, duplicate logic removed, regression risk reduced, and tests added or updated. If no abstraction was added, explain why.

Avoid copy/pasted object construction, repeated inline validation, repeated SQLite row mapping, repeated Tauri command payload shaping, business logic inside React components or Tauri command handlers, stringly typed command/status/result handling, large dumping-ground utility files, ad hoc Tauri window creation outside `WindowManager`, and duplicated standalone-window setup or default options.

For broad cleanup or architecture work, run `npm run build`, `cargo build`, `cargo clippy --all-targets`, and `git diff --check` when practical. Treat Clippy warnings as review findings: fix clear no-risk warnings immediately, and document larger refactor warnings instead of suppressing them without a specific reason.

## Coding Rules

- Preserve the local-first design.
- Keep React/frontend code out of token handling.
- Keep OpenAI and GitHub token usage backend-owned.
- Treat SQLite as the source of truth for persisted app data.
- Make migrations non-destructive and idempotent.
- Do not remove existing user data unless the user explicitly requests cleanup.
- Prefer focused changes over broad rewrites.
- Do not commit unrelated user changes.
- Do not introduce arbitrary command execution through SQLite, scheduler rows, Lua payloads, or user-editable config.
- Keep generated local files, screenshots, plugin binaries, third-party DLLs, and machine-specific outputs out of git unless documentation explicitly says otherwise.

## Local Dev HTML Review Rules

- When editing a local dev/review HTML file outside the repository, such as an exported build-guide step review file in Downloads, automatically open it in the default browser after the edit is complete so the latest saved version is visible for review.
- Prefer `Start-Process -FilePath <html-path>` on Windows for this browser refresh/open step.
- Do not add these generated review HTML files to git unless the user explicitly asks to convert one into a committed fixture.

## Documentation Rules

- Update docs when behavior, scope, validation, or persistence changes.
- Review every newly added Markdown file for ownership and discoverability. Add it to the root Documentation Map only when it meets the routing criteria above.
- When cutting a release, update the current stable version in `docs/PROJECT_OVERVIEW.md` alongside the changelog and project version metadata.
- Keep active documentation compact and task-facing.
- Put historical release/checkpoint details in `docs/PROJECT_HISTORY.md`, not separate active tracker files.
- Put deferred items in `docs/DEFERRED_ITEMS.md`.
- Use current terminology from these docs when naming UI, docs, and future features.
- Do not reintroduce retired external-transfer terminology into new documentation or UI.

## Versioning And Changelog Rules

- Use semantic versioning in `MAJOR.MINOR.PATCH` form.
- Do not increment the minor version just because a new chat, work session, or calendar day starts.
- Keep changelog entries date/time-stamped under day headings.
- Use `## Unreleased` for active work until the user intentionally cuts a version.
- Use `PATCH` for fixes, documentation-only changes, validation updates, small UX refinements, and internal refactors.
- Use `MINOR` for substantial new user-visible capabilities.
- Use `MAJOR` for incompatible or breaking release changes.
- Keep `docs/PROJECT_OVERVIEW.md` on the latest released version; do not advance it for work that remains under `## Unreleased`.
- Read `docs/VERSIONING.md` before changing version metadata or changelog structure.

## Validation Rules

Run validation appropriate to the touched area.

Minimum defaults:

| Changed area | Validation |
| --- | --- |
| Frontend / React / TypeScript | `npm run build` |
| Rust / Tauri backend | `cd src-tauri && cargo build` |
| Shared frontend/backend behavior | both commands |
| Persistence changes | both commands plus migration review |
| GearBlocks script/plugin work | build/type-check plus manual game-path validation where possible |
| Scheduler changes | backend build plus bounded-job behavior review |

If validation cannot be run, state that clearly and explain what was not validated.

## Commit Rules

When the user asks for a commit or release/checkpoint completion:

1. Run appropriate validation.
2. Review `git status`.
3. Stage only intended files.
4. Use a specific commit message.
5. Do not include unrelated local changes.
6. Push only when the user requested or the current workflow explicitly requires it.
