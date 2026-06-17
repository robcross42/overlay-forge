# Project Rules

These rules guide future Overlay Forge planning, implementation, validation, and documentation updates.

## Changelog Date Tracking

Keep `CHANGELOG.md` organized so changes can be reviewed by the day they were made.

- Under `## Unreleased`, add a daily heading before new change entries using ISO format: `### YYYY-MM-DD`.
- Put that day's changes under standard changelog subheadings such as `#### Added`, `#### Changed`, `#### Fixed`, `#### Validation`, or `#### Documentation`.
- Add all entries for the same calendar day under the same date heading instead of scattering them through the older undated sections.
- Keep older historical entries as-is unless a separate cleanup task explicitly asks to reorganize them.
- When a milestone validation changes documentation, include the validation/documentation update under that day's heading so the daily work summary stays complete.
- If the user explicitly defines a version bucket for a work session, related changes that roll into the early AM hours before the session ends may remain in that same version bucket instead of being split into a new release heading.

## Session Versioning

Each new work session uses the next `0.x.0` minor version until the user says otherwise.

- When reviewing these project rules in a new chat, compare the current local date to the date of the latest changelog version heading.
- If the current local date is greater than the date of the latest changelog version heading, create the next minor version bucket before starting project work.
- Increment only the minor version in the `0.x.0` sequence, such as `0.2.0`, `0.3.0`, `0.4.0`, and `0.5.0`.
- Update `package.json` and `package-lock.json` to the new version when cutting the new session bucket.
- Add the new version heading near the top of `CHANGELOG.md` using `## 0.x.0 - YYYY-MM-DD`, then add that session's daily `### YYYY-MM-DD` entries under it.
- Related changes that continue into the early AM hours before the user considers the work session complete may stay in the existing session version bucket.
