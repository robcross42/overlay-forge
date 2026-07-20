# Overlay Forge Markdown Restructure Manifest

This generated set replaces separate active progress-tracker files with a smaller repo-facing documentation structure.

## Files

```text
README.md
CHANGELOG.md
AGENTS.md
.vscode/CODEX_INSTRUCTIONS.md
docs/PROJECT_OVERVIEW.md
docs/PROJECT_HISTORY.md
docs/ARCHITECTURE.md
docs/DATA_MODEL.md
docs/FEATURE_SCOPE.md
docs/DEFERRED_ITEMS.md
docs/VALIDATION.md
docs/VERSIONING.md
docs/GAMING_SCREENSHOTS.md
docs/GEARBLOCKS.md
docs/GEARBLOCKS_RUNTIME.md
docs/GEARBLOCKS_PLUGIN.md
docs/GEARBLOCKS_PARTS_CATALOG.md
docs/SMOKING_CESSATION.md
MANIFEST.md
```

## Merge Decisions

- `README.md` is now a compact repository entry point instead of an archive.
- `CHANGELOG.md` keeps date/time-stamped history and now follows semantic versioning.
- Root instructions live in `AGENTS.md`.
- VS Code reminder content lives in `.vscode/CODEX_INSTRUCTIONS.md`.
- Early project history is consolidated into `docs/PROJECT_HISTORY.md`.
- Current behavior and baseline live in `docs/PROJECT_OVERVIEW.md`.
- Current architecture and module ownership live in `docs/ARCHITECTURE.md`.
- Current schema and persistence rules live in `docs/DATA_MODEL.md`.
- Scope guardrails live in `docs/FEATURE_SCOPE.md`.
- Deferred work lives in `docs/DEFERRED_ITEMS.md`.
- Build/manual checks live in `docs/VALIDATION.md` and `docs/VERSIONING.md`.
- GearBlocks remains split because runtime, plugin/modding, and parts catalog details are large and operationally distinct.

## Files Not Retained As Active Docs

The old one-file-per-checkpoint structure is not retained as active documentation. Its useful content is merged into:

- `docs/PROJECT_HISTORY.md`
- `docs/VALIDATION.md`
- `docs/VERSIONING.md`
- `docs/FEATURE_SCOPE.md`
- `docs/DEFERRED_ITEMS.md`

Historical detail can still be recovered from git history if needed.
