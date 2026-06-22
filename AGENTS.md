# AGENTS.md

Agent instructions for Overlay Forge. This file applies to the entire repository.

## Project Baseline

- Current stable app baseline: Milestone 13, complete / passed / successful.
- Current active development bucket: `0.6.0`.
- Overlay Forge is a local-first Tauri v2 desktop overlay app with a React + TypeScript frontend, Rust backend, and SQLite persistence.
- Future work must preserve the completed overlay shell, Projects workspace, project chat, bridge draft, local Markdown context, Gaming, GearBlocks, screenshot, and runtime API indexing workflows unless a user request explicitly changes them.

## Required Context

Before planning or implementing project work, read the relevant repo docs instead of relying only on the README.

- `README.md`
- `CHANGELOG.md`
- `docs/ARCHITECTURE.md`
- `docs/DATA_MODEL.md`
- `docs/PROJECT_PLAN.md`
- `docs/PROJECT_DEFERRED_ITEMS.md`
- GearBlocks work: also read `docs/GEARBLOCKS_CONSTRUCTION_DECODER.md`, `docs/GEARBLOCKS_PARTS_CATALOG.md`, and `docs/GEARBLOCKS_RUNTIME_INTERFACES.md`.
- Gaming screenshot work: also read `docs/GAMING_SCREENSHOT_VALIDATION.md`.
- Bridge workflow work: also read `docs/BRIDGE_FILES.md` and `bridge-files/OPENAI_APP_BRIDGE.md`.

## Versioning

Each new substantive work session uses the next `0.x.0` minor version unless the user says otherwise.

- Compare the current local date to the date of the latest changelog version heading.
- If the latest version bucket only contains the version bump / setup entry and there are no committed or pending substantive changes for that version, keep using that existing version bucket instead of cutting another minor version.
- Treat a version bucket as consumed only after substantive feature, fix, validation, or documentation work for that version has been committed and pushed. A version-only bump commit does not consume the bucket by itself.
- If the current local date is greater than the date of the latest changelog version heading and the latest version bucket is already consumed, create the next minor version bucket before starting project work.
- Increment only the minor version in the `0.x.0` sequence, such as `0.2.0`, `0.3.0`, `0.4.0`, and `0.5.0`.
- Update `package.json` and `package-lock.json` to the new version when cutting a new session bucket.
- Add the new version heading near the top of `CHANGELOG.md` using `## 0.x.0 - YYYY-MM-DD`, then add that session's daily `### YYYY-MM-DD` entries under it.
- Related changes that continue into the early AM hours before the user considers the work session complete may stay in the existing session version bucket.

## Changelog

Keep `CHANGELOG.md` organized so changes can be reviewed by the day they were made.

- Under the active version bucket, add a daily heading before new change entries using ISO format: `### YYYY-MM-DD`.
- Put that day's changes under standard changelog subheadings such as `#### Added`, `#### Changed`, `#### Fixed`, `#### Validation`, or `#### Documentation`.
- For new `#### Changed` and `#### Fixed` bullet entries, prefix each bullet with the current local system timestamp using `HH:MM:SS EDT -`, for example `- 18:34:15 EDT - Changed ...`.
- Add all entries for the same calendar day under the same date heading instead of scattering them through older sections.
- Keep older historical entries as-is unless a separate cleanup task explicitly asks to reorganize them.
- When a milestone validation changes documentation, include the validation/documentation update under that day's heading so the daily work summary stays complete.

## Reasoning Model Selection Rules

Optimize cost, latency, and token usage by defaulting all tasks to Medium Reasoning unless the requested work meets the criteria for High Reasoning or Very High Reasoning.

Codex should automatically evaluate each request before execution and select the lowest reasoning level capable of completing the task correctly.

### Default Rule

Use Medium Reasoning for all tasks unless one or more escalation conditions are met.

Do not escalate reasoning simply because a request is large, contains many files, or involves multiple steps. Escalation should be based on complexity, uncertainty, architectural impact, or risk.

### Medium Reasoning

Use for:

- Bug fixes with a clearly identified cause.
- Small feature additions within an existing architecture.
- UI adjustments.
- Refactoring isolated code.
- Documentation updates.
- Unit test additions.
- Data model additions that follow existing patterns.
- Configuration changes.
- Log analysis with obvious findings.
- Simple database updates.
- Export/import enhancements following existing implementation patterns.
- Iterative tuning and refinement of existing functionality.
- Game module feature development where requirements are already understood.

Examples:

- Add a button.
- Fix a null pointer exception.
- Add a database field.
- Update export behavior.
- Adjust overlay positioning.
- Improve error handling.
- Add a new API endpoint following existing conventions.

### High Reasoning

Use when the request requires substantial analysis, design decisions, or understanding of multiple interconnected systems.

Escalate to High Reasoning when any of the following apply:

- Architectural changes affecting multiple modules.
- Database schema redesign.
- Significant refactoring across multiple files.
- New subsystem creation.
- Complex debugging where root cause is unknown.
- Performance optimization requiring investigation.
- State synchronization issues.
- Multi-step workflows spanning several components.
- Security-sensitive changes.
- Concurrency or threading concerns.
- Data migration planning.
- New integration with external services.
- Ambiguous requirements requiring design decisions.

Examples:

- Design a new overlay subsystem.
- Implement a Graph API integration.
- Investigate data inconsistencies across modules.
- Rework export and database synchronization logic.
- Design a plugin architecture.
- Create a screenshot ingestion pipeline.
- Diagnose intermittent synchronization failures.

### Very High Reasoning

Use only when failure would be costly or when extensive planning and deep analysis are required before implementation.

Escalate to Very High Reasoning when any of the following apply:

- Designing entirely new platform architecture.
- Major framework-level changes.
- Complex cross-module redesigns.
- Large-scale data model redesigns.
- Multi-phase migration strategies.
- High-risk security architecture.
- Complex AI-agent workflows.
- Large project planning requiring extensive decomposition.
- Requests where multiple valid architectural approaches exist and tradeoff analysis is required.
- Situations where implementation without significant planning would likely create technical debt.

Examples:

- Designing Overlay Forge core architecture.
- Defining a new module framework.
- Planning a major persistence redesign.
- Designing agent-to-agent coordination systems.
- Reworking application lifecycle management.
- Establishing project-wide standards affecting all modules.

### Escalation Process

Before beginning work:

1. Classify the task.
2. Select the lowest reasoning level capable of completing it.
3. Briefly state the selected reasoning level.
4. If the selected level is Medium, proceed with execution.
5. If the selected level is High or Very High, stop before execution and ask the user to switch the VS Code/Codex reasoning setting to that level. Do not continue until the user confirms the setting has been changed or explicitly tells Codex to proceed anyway.

After completing a Medium Reasoning task, briefly flag the user in the final response if Low Reasoning would likely have been sufficient. Do not stop or delay Medium Reasoning work for this; use it only as post-implementation calibration so the project can tune reasoning defaults over time.

Reasoning levels should be selected conservatively.

If uncertain between two levels:

- Prefer Medium over High.
- Prefer High over Very High.

Escalate only when the complexity genuinely requires it.

### Overlay Forge Specific Guidance

Medium:

- Routine feature additions.
- UI changes.
- Database field additions.
- Export/import fixes.
- Module-specific enhancements.
- Iterative improvements discovered through gameplay.

High:

- New modules.
- Cross-module integrations.
- Database architecture changes.
- State synchronization redesign.
- External service integrations.
- Complex troubleshooting with unknown root causes.

Very High:

- Core framework redesign.
- Agent architecture changes.
- Major persistence redesign.
- Module framework redesign.
- Changes affecting all modules and future development patterns.

Goal: maintain maximum development velocity by using Medium Reasoning whenever practical while automatically escalating to High or Very High Reasoning only when complexity, risk, uncertainty, or architectural impact justify the additional cost.

## Implementation Rules

- Prefer existing repo patterns and feature boundaries over new abstractions.
- Keep SQLite migrations non-destructive and idempotent.
- Use normalized SQLite names: `obj_` for dynamic object tables, `def_` for static definition tables, `o2o_` for one-to-one mapping tables, and `n2n_` for many-to-many mapping tables. Include `created_at`, `modified_at`, and a `schema_json` field on normalized tables unless there is a concrete compatibility reason not to. Keep legacy table renames non-destructive and idempotent.
- For game-specific persistence, prefer `def_game`, `obj_game`, `obj_game_setting`, or normalized feature tables keyed by `game_id` and `id_game`. Do not create per-game physical tables such as `obj_game_gearblocks` unless a future design explicitly justifies the migration cost.
- Scheduler rows must map to explicit Rust handlers through `def_scheduler_type`; do not store or execute arbitrary commands, script bodies, Lua payloads, shell commands, or frontend-controlled executable strings from SQLite.
- React must call local Tauri commands for backend-owned behavior.
- Backend-only secrets stay in Rust/Tauri. Do not expose `OPENAI_API_KEY` or `GITHUB_TOKEN` to frontend source or SQLite.
- Preserve local-first behavior. Do not introduce cloud sync, broad filesystem indexing, GitHub file browsing, vector stores, semantic search, or direct Codex handoff unless explicitly requested.
- Use structured APIs/parsers when available instead of ad hoc string parsing.
- Keep edits scoped. Do not refactor unrelated modules while implementing a requested change.

## GearBlocks Rules

- GearBlocks save decoding, runtime log import, catalog metadata, screenshots, and chat context are local-first.
- `construction.bytes` decoding reads local raw DEFLATE-compressed BSON and does not require GearBlocks to be running.
- Runtime metadata comes from the installable GearBlocks Lua script mod, which now exports the whole live scene by default.
- Do not trigger GearBlocks runtime log parsing from normal chat navigation; use explicit refresh or backend chat-send context assembly until a true background/latest-export importer is added.
- Runtime API metadata is availability-only by default. Do not execute getter commands or include API getter values in default chat prompt context unless a future explicit user-controlled include/snapshot action is added.
- Prefer normalized GearBlocks runtime/API tables and command-layer payloads over reparsing only the full runtime export JSON blob.
- BepInEx and GearLib are third-party, user-installed dependencies. Overlay Forge may detect and document their presence, but must not bundle, redistribute, install, or modify them unless a future explicit license / permission review allows it.
- GearBlocks scale guidance must use metric units: 1 GearBlocks unit equals 10 cm. Chat should answer GearBlocks build-distance advice in centimeters and/or GearBlocks units such as 1 unit, 0.5 units, or 16 units, and should not suggest imperial distances unless the user explicitly asks for imperial conversion.
- GearBlocks scale caveat: the developer noted that the player character, wheels, and other parts are slightly oversized to allow room for gears and other parts inside vehicles. Treat those as gameplay-clearance exceptions, not strict real-world scale references.
- Preserve the validated `game-screenshots/` folder layout, Tauri asset preview scope, overlay-hidden capture behavior, and screenshot delete cleanup semantics.

## Validation

Use focused validation that matches the change.

- Frontend-only changes: run `npm run build` when practical.
- Rust/backend changes: run `cargo check` or `cargo build` from `src-tauri` when practical.
- Full app changes: run both frontend and Rust validation when practical.
- Mention any validation that could not be run.

## Milestone Handoff

When the user reports that a milestone validation is complete:

1. Update milestone, changelog, project plan, architecture, data model, and bridge docs from pending validation to `Complete / Passed / Successful`.
2. Run a quick sanity check appropriate to the milestone.
3. Review `git status` and include only intended milestone changes.
4. Commit with a milestone-specific message.
5. Push the current branch.

Do not commit unrelated user changes.
