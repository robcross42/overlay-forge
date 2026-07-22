# Codex VS Code Quick Reference

This file is a VS Code-side reminder. The authoritative instruction file is the repository-root `AGENTS.md`.

## Direct Codex Workflow

Use Codex chat directly in VS Code against the repository.

Repository Markdown is context. It is not a separate external transfer system.

## Read Before Work

- `AGENTS.md`
- `docs/PROJECT_OVERVIEW.md`
- `docs/PROJECT_HISTORY.md` when archived project history matters
- `docs/ARCHITECTURE.md` for architecture
- `docs/DATA_MODEL.md` for SQLite or migrations
- `docs/FEATURE_SCOPE.md` and `docs/DEFERRED_ITEMS.md` for scope
- `docs/VALIDATION.md` for test expectations
- `docs/VERSIONING.md` for changelog/versioning rules

## Default Validation

```powershell
npm run build
cd src-tauri
cargo build
```

Use only the validation needed for the actual change when the task is narrow.

## Versioning Reminder

Do not bump a minor version for a new chat, work session, or date. Use `## Unreleased` until the user intentionally cuts a semantic version. Keep changelog entries timestamped, and update the current stable version in `docs/PROJECT_OVERVIEW.md` whenever a release is cut.
