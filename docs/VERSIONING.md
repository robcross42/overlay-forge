# Versioning And Changelog Rules

Overlay Forge uses semantic versioning in `MAJOR.MINOR.PATCH` form.

## Version Number Rules

Do not increment the minor version just because a new chat, work session, or calendar day starts.

Use version numbers for release scope:

| Segment | Use for |
| --- | --- |
| `MAJOR` | Incompatible or breaking release changes. |
| `MINOR` | Substantial new user-visible features or capability groups. |
| `PATCH` | Bug fixes, documentation-only changes, validation updates, small UX refinements, and internal refactors that do not introduce a major capability. |

During `0.x` development, minor versions may still contain breaking early-development changes. Patch versions should remain non-breaking fixes, documentation, and small refinements.

## When To Open A New Version Section

Create a new version heading only when preparing or recording a meaningful version cut.

Do not create a new version heading for:

- a new Codex chat
- a new work session
- a new calendar day
- an uncommitted local experiment
- documentation-only updates that fit the current unreleased work

Use `## Unreleased` for active work until a version is intentionally cut.

## Changelog Entry Format

Keep date and time logging.

Use day headings under `## Unreleased` or a version section:

```markdown
### YYYY-MM-DD

#### Added

- HH:MM:SS EDT - Added ...
```

Use local Toronto time unless the user explicitly asks for another timezone.

Preferred categories:

```text
Added
Changed
Fixed
Removed
Documentation
Validation
Known Issues
```

## Version Heading Format

Use:

```markdown
## MAJOR.MINOR.PATCH - YYYY-MM-DD
```

The date is the date the version is cut, not necessarily the first date work started.

## Project Metadata Updates

When cutting a version, update all project metadata that stores the app version.

Also update the current stable version in `docs/PROJECT_OVERVIEW.md` as part of the same release change. The overview should identify the latest released version, not an in-progress version still recorded under `## Unreleased`.

When making ordinary unreleased changes, do not update project version metadata unless the user explicitly asks to cut a version.

## Validation Notes

Changelog validation entries should record the exact commands or manual checks performed.

If validation was not run, document that in the final response to the user instead of inventing a changelog validation entry.
