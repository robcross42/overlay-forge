# Milestone 5 - Controlled YouTube Component

## Status

Complete / Passed / Successful

Milestone 5 adds a controlled, local-first YouTube reference component. The user can intentionally save YouTube video references, persist them locally, display them in the app, edit them, delete them, and open saved URLs externally without disrupting the overlay workflow.

User validation is complete and Milestone 5 passed.

## Goal

Prove that Overlay Forge can:

```text
Add a YouTube video reference -> persist it locally -> display it in the app -> open it externally or in a controlled view without disrupting the overlay workflow.
```

## Implemented Capabilities

- Added YouTube to the overlay component navigation.
- Added a YouTube feature component under `src/features/youtube/`.
- Added frontend YouTube service functions under `src/services/youtube.ts`.
- Added SQLite-backed YouTube reference persistence.
- Added Rust/Tauri commands for listing, getting, creating, updating, deleting, and opening YouTube references.
- Added backend validation for common YouTube URL forms.
- Added user-entered metadata fields for channel name, notes, and tags.
- Added selected/read-only/edit UI behavior consistent with Projects and organizer components.
- Added external-open behavior for saved YouTube URLs through the system browser.
- Preserved Scratchpad, Tasks, Notes, Calendar, Projects, Planning Chat, and GitHub Repository behavior.

## Data Table

```text
youtube_references
- id
- title
- url
- video_id
- channel_name
- notes
- tags
- created_at
- updated_at
```

Minimum required fields:

```text
id
title
url
created_at
updated_at
```

## Supported URL Forms

```text
https://www.youtube.com/watch?v=VIDEO_ID
https://youtube.com/watch?v=VIDEO_ID
https://youtu.be/VIDEO_ID
https://www.youtube.com/shorts/VIDEO_ID
```

The backend rejects obviously invalid URLs with readable errors. Milestone 5 does not require a YouTube API key.

## Controlled Component Boundary

Milestone 5 YouTube references are intentionally saved by the user. The component does not auto-recommend videos, pull an infinite feed, require account login, track watch history, scrape YouTube, fetch transcripts, download video/audio, sync playlists, sync comments, or embed an unrestricted browser.

External browser opening is the documented controlled target for Milestone 5.

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
Passed. App launches successfully in development mode when run outside the sandbox with normal app-data write access. The app process started and was stopped after the validation timeout.
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
Navigation shows Scratchpad, Tasks, Notes, Calendar, Projects, Planning Chat, and YouTube.
```

Pass criteria:

```text
All expected component entries are visible.
```

Validate:

```text
Open the YouTube component.
```

Pass criteria:

```text
The YouTube component loads without errors.
```

Validate:

```text
Create a new YouTube reference with a valid YouTube URL and title.
```

Pass criteria:

```text
The reference saves successfully and appears in the list.
```

Validate:

```text
Select the saved YouTube reference.
```

Pass criteria:

```text
The selected reference shows its saved details.
```

Validate:

```text
Edit the saved YouTube reference.
```

Pass criteria:

```text
Updated fields save successfully and remain visible.
```

Validate:

```text
Restart the app and return to the YouTube component.
```

Pass criteria:

```text
The saved YouTube reference is restored from SQLite.
```

Validate:

```text
Attempt to save an invalid YouTube URL.
```

Pass criteria:

```text
The UI shows a readable validation error and the app does not crash.
```

Validate:

```text
Click Open on a saved YouTube reference.
```

Pass criteria:

```text
The saved YouTube URL opens externally in the system browser.
```

Validate:

```text
Delete a saved YouTube reference.
```

Pass criteria:

```text
The reference is removed and does not return after restart.
```

Validate:

```text
Return to Scratchpad, Tasks, Notes, Calendar, Projects, Planning Chat, and GitHub Repository behavior in Projects.
```

Pass criteria:

```text
Existing Milestone 0, Milestone 1, Milestone 2, Milestone 3, and Milestone 4 components still work and persisted data remains available.
```

## User Pass/Fail Reporting Format

```markdown
# Milestone 5 Validation Results

## Overall Result

Pass or Fail

## Failed Items

- Item:
  - Expected:
  - Actual:

## Notes

Any extra observations.
```

## Deferred Items

- YouTube account login
- YouTube API integration
- OAuth flow
- Subscription import
- Watch history import
- Automatic video recommendations
- Transcript retrieval
- Transcript summarization
- Downloading videos or audio
- Embedded unrestricted browser
- Playlist sync
- Comment sync
- Channel scraping
- Background metadata crawler
- Bridge-file generation from videos
- Codex handoff from videos
- Cloud sync
- Multi-user auth
