# Versioning And Changelog Rules

Overlay Forge uses a traceability-first `MAJOR.MINOR.PATCH` project release sequence. It uses the familiar three-part version form, but its patch segment identifies independently completed changes rather than only bug fixes.

## Traceability Contract

The normal relationship is:

```text
one completed change -> one change commit -> one version -> one changelog section -> one annotated Git tag
```

Each completed change commit must be discoverable by its version. The changelog explains the issue and user impact, the version tag identifies the exact commit, and Git shows every file included in that change.

Merge commits that only integrate already-versioned commits do not receive another application version. Avoid separate cleanup or metadata-only commits for the same issue; amend the versioned change before pushing when practical. Never rewrite shared history merely to repair an older unversioned commit. Instead, let the next version explicitly adopt and document the accumulated unversioned work as a transition.

## Version Number Rules

Choose the smallest version level that honestly represents the final cohesive diff:

| Segment | Use for |
| --- | --- |
| `PATCH` | Default for one independently completed change: fixes, refinements, documentation, validation, internal work, and modest additions to existing features. |
| `MINOR` | A substantial cohesive capability, significant feature expansion, or major rework within the current product generation. Reset patch to zero. |
| `MAJOR` | A new broad product generation or fundamental change spanning product identity, major workflows, compatibility, or several capability groups. Reset minor and patch to zero. |

Version selection follows scope, not elapsed time, session count, calendar day, file count, or raw line count. Changes to previously introduced features normally increment patch unless the actual work becomes a substantial expansion or major rework.

## Version Circuit Breaker

Apply the circuit breaker after implementation and validation, immediately before committing, when the actual diff is known:

1. Review the complete staged scope against the intended issue.
2. Split unrelated or independently releasable work into separate commits and sequential versions.
3. Keep inseparable work together when it represents one cohesive change.
4. Automatically promote patch to minor, or minor to major, when that cohesive change exceeds the planned level.
5. Never downgrade a requested level or force oversized work through a lower version.

Typical patch-to-minor triggers include a materially expanded user workflow, a substantial architectural or persistence change, or a large cohesive feature rework. Typical minor-to-major triggers include a new product generation, broad incompatibility, or fundamental changes across several major capabilities. Promotion should continue automatically without interrupting the user; explain it in the final response.

## Changelog Workflow

Keep `## Unreleased` only for work that has not yet been committed. Before creating the completed change commit:

1. Select the next version from the latest `vMAJOR.MINOR.PATCH` tag.
2. Move the relevant Unreleased entries into a matching version section.
3. Update every project version location.
4. Run `npm.cmd run version:check` on Windows (`npm run version:check` elsewhere) and the validation required for the changed area.
5. Commit using `vMAJOR.MINOR.PATCH: description`.
6. Create an annotated tag using `git tag -a vMAJOR.MINOR.PATCH -m "Overlay Forge MAJOR.MINOR.PATCH"`.
7. When publishing, push both the branch and the specific version tag.

Use this version heading format:

```markdown
## MAJOR.MINOR.PATCH - YYYY-MM-DD
```

Within the version section, retain Toronto-local day and time entries:

```markdown
### YYYY-MM-DD

#### Changed

- HH:MM:SS EDT - Changed ...
```

Preferred categories are Added, Changed, Fixed, Removed, Documentation, Validation, and Known Issues. Include an issue or pull-request number when one exists and is known before the versioned commit; the Git tag remains the canonical commit link.

## Project Version Locations

Keep the same version in:

- `package.json`
- the root package and empty-package entries in `package-lock.json`
- `src-tauri/Cargo.toml`
- the `overlay-forge` package entry in `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`
- the current stable release in `README.md`
- the current stable release in `docs/PROJECT_OVERVIEW.md`
- the newest version heading in `CHANGELOG.md`

Run `npm.cmd run version:check` on Windows (`npm run version:check` elsewhere) after every version update. After the commit is tagged, `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-project-version.ps1 -RequireTag` also verifies that the matching local tag exists.

## Validation Notes

Changelog validation entries must record only checks that were actually completed. If validation cannot be run, report that in the final response instead of inventing a changelog validation entry.
