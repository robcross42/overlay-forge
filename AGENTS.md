# Overlay Forge Codex Instructions

## Authority

This file is the top-level instruction file for Codex work in this repository.

Codex should read this file before planning, editing, validation, documentation updates, or commits. Supporting documentation lives under `docs/` and should be read when relevant to the requested change.

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
| GearBlocks feature work | `docs/GEARBLOCKS.md`, then the focused GearBlocks docs |
| Smoking Cessation | `docs/SMOKING_CESSATION.md` |

## Reasoning Model Selection

Mandatory preflight: classify every user request before doing any substantive work, including answering research/design questions, reading external sources, running commands, editing files, or making plans.

Default to **Medium Reasoning**.

Use **Low Reasoning** only for narrow, obvious changes such as small copy edits, simple constant updates, or isolated style tweaks.

Use **High Reasoning** when the request affects:

- architecture
- persistence or migrations
- Rust/Tauri command flow
- OpenAI request assembly
- GitHub integration
- GearBlocks runtime import/export
- scheduler behavior
- security or token handling
- cross-feature UI state
- large refactors

Use **Very High Reasoning** when the request affects multiple major subsystems, requires uncertain reverse engineering, or could damage user data.

If the current Codex reasoning setting is lower than the task requires, stop all processing before reading more context, browsing, running tools, planning, or editing. Tell the user exactly which reasoning level to switch to and wait for the user to resubmit the prompt with the correct reasoning setting.

This includes **Low -> Medium** escalation. If the user is set to Low and the request requires Medium, stop immediately and ask the user to switch to Medium. Do not proceed just because Medium is the default or because the request looks easy to answer.

If the current Codex reasoning setting is **Medium** and the task only requires **Low**, proceed at Medium without stopping.

If the current Codex reasoning setting is **High** or **Very High** and the task only requires **Low** or **Medium**, stop all processing before editing, tell the user the lower recommended reasoning level, and wait for the user to resubmit the prompt.

Do not flag after completion that a lower reasoning level would have been sufficient. Only stop before work begins when the current setting is either too low for the requested task or unnecessarily High / Very High for a Low / Medium task.

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

## Documentation Rules

- Update docs when behavior, scope, validation, or persistence changes.
- Keep active documentation compact and task-facing.
- Put historical milestone details in `docs/PROJECT_HISTORY.md`, not one file per milestone.
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

When the user asks for a commit or milestone completion:

1. Run appropriate validation.
2. Review `git status`.
3. Stage only intended files.
4. Use a specific commit message.
5. Do not include unrelated local changes.
6. Push only when the user requested or the current workflow explicitly requires it.
