# AGENTS.md

Agent instructions for Overlay Forge. This file applies to the entire repository.

## Project Baseline

- Current stable app baseline: Milestone 13, complete / passed / successful.
- Current active development bucket: `0.4.0`.
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
- Add all entries for the same calendar day under the same date heading instead of scattering them through older sections.
- Keep older historical entries as-is unless a separate cleanup task explicitly asks to reorganize them.
- When a milestone validation changes documentation, include the validation/documentation update under that day's heading so the daily work summary stays complete.

## Implementation Rules

- Prefer existing repo patterns and feature boundaries over new abstractions.
- Keep SQLite migrations non-destructive and idempotent.
- React must call local Tauri commands for backend-owned behavior.
- Backend-only secrets stay in Rust/Tauri. Do not expose `OPENAI_API_KEY` or `GITHUB_TOKEN` to frontend source or SQLite.
- Preserve local-first behavior. Do not introduce cloud sync, broad filesystem indexing, GitHub file browsing, vector stores, semantic search, or direct Codex handoff unless explicitly requested.
- Use structured APIs/parsers when available instead of ad hoc string parsing.
- Keep edits scoped. Do not refactor unrelated modules while implementing a requested change.

## GearBlocks Rules

- GearBlocks save decoding, runtime log import, catalog metadata, screenshots, and chat context are local-first.
- `construction.bytes` decoding reads local raw DEFLATE-compressed BSON and does not require GearBlocks to be running.
- Runtime metadata comes from the installable GearBlocks Lua script mod and explicit `Refresh Runtime Log` imports.
- Normal game selection and Parts navigation must not automatically scan `Player.log` or `Player-prev.log`.
- Runtime API metadata is availability-only by default. Do not execute getter commands or include API getter values in default chat prompt context unless a future explicit user-controlled include/snapshot action is added.
- Prefer normalized GearBlocks runtime/API tables and command-layer payloads over reparsing only the full runtime export JSON blob.
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
