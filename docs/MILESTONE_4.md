# Milestone 4 - GitHub Integration

## Status

Complete / Passed / Successful

Milestone 4 adds a local-first GitHub integration foundation. User validation is complete and Milestone 4 passed.

## Goal

Allow a local Overlay Forge project to be linked to a GitHub repository, then fetch and display basic repository metadata/status.

## Implemented Capabilities

- Added project-scoped GitHub repository link storage in SQLite.
- Added a `project_github_repositories` table with non-destructive idempotent initialization.
- Added Rust/Tauri commands to get, save, delete, and fetch GitHub repository metadata for a selected project.
- Added backend-only GitHub token handling through `GITHUB_TOKEN`.
- Added GitHub metadata fetch through the Rust backend without exposing the token to React.
- Added a GitHub Repository section inside the existing Projects component.
- Added readable UI status/error messages for missing token, invalid repository full name, and GitHub request failures.
- Preserved Scratchpad, Tasks, Notes, Calendar, Projects, and Planning Chat behavior.

## Data Table

```text
project_github_repositories
- id
- project_id
- repository_full_name
- repository_url
- default_branch
- visibility
- last_fetched_at
- last_fetch_status
- created_at
- updated_at
```

Each local project can have one linked GitHub repository. SQLite stores repository linkage and fetched metadata only. It does not store GitHub tokens.

## GitHub Token Setup

Milestone 4 reads the GitHub token from:

```text
GITHUB_TOKEN
```

The token is read only by the Rust/Tauri backend. It is not stored in SQLite and is not exposed to React/frontend code. If the token is missing, fetching metadata returns a readable configuration error and the app should not crash.

## Deferred Items

- Automatic Codex handoff
- Pull request creation
- Branch creation
- Commit creation
- File editing through GitHub
- GitHub issue management
- Full repository file browser
- GitHub Actions integration
- Webhook support
- OAuth flow
- Multi-account GitHub support
- Advanced sync engine
- Conflict resolution
- Bridge-file generation UI
- Vector store or repo indexing

## Setup Validation

Run:

```powershell
npm install
```

Result:

```text
Passed. Dependencies install successfully.
```

Run:

```powershell
npm run build
```

Result:

```text
Passed. Frontend builds successfully.
```

Run:

```powershell
cd src-tauri
cargo build
```

Result:

```text
Passed. Rust backend compiles successfully.
```

Run:

```powershell
npm run tauri:dev
```

Result:

```text
Passed. App launches successfully in development mode when run outside the sandbox with normal app-data write access.
```

## Manual Validation Checklist

Validate:

```text
Open the app and reveal the overlay with Ctrl+Shift+Space.
```

Pass criteria:

```text
Overlay appears using existing hotkey behavior.
```

Validate:

```text
Navigation still shows Scratchpad, Tasks, Notes, Calendar, Projects, and Planning Chat.
```

Pass criteria:

```text
All expected component entries are visible.
```

Validate:

```text
Create or select a local project.
```

Pass criteria:

```text
The selected project appears normally and existing project behavior still works.
```

Validate:

```text
Enter and save a GitHub repository full name for the selected project.
```

Pass criteria:

```text
The repository link saves successfully and remains associated with the selected project.
```

Validate:

```text
Restart the app and return to the selected project.
```

Pass criteria:

```text
The linked GitHub repository is restored from SQLite.
```

Validate:

```text
Attempt to fetch GitHub metadata with no GitHub token configured.
```

Pass criteria:

```text
The UI shows a readable missing-configuration error and the app does not crash.
```

Validate:

```text
Configure GITHUB_TOKEN, restart the app, and fetch GitHub metadata for a valid repository.
```

Pass criteria:

```text
The app fetches and displays basic repository metadata.
```

Validate:

```text
Enter an invalid repository full name and attempt to fetch metadata.
```

Pass criteria:

```text
The UI shows a readable fetch/error state and the app does not crash.
```

Validate:

```text
Return to Scratchpad, Tasks, Notes, Calendar, Projects, and Planning Chat.
```

Pass criteria:

```text
Existing Milestone 0, Milestone 1, Milestone 2, and Milestone 3 components still work and persisted data remains available.
```

## User Pass/Fail Reporting Format

```markdown
# Milestone 4 Validation Results

## Overall Result

Pass or Fail

## Failed Items

- Item:
  - Expected:
  - Actual:

## Notes

Any extra observations.
```
