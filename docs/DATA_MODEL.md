# Data Model

## Status

Milestone 1 data model is complete, passed, and successful.

Milestone 2 project data model is complete, passed, and successful.

Milestone 3 planning chat data model is complete, passed, and successful.

Milestone 4 - GitHub Integration data model is complete, passed, and successful.

Milestone 5 - Controlled YouTube Component data model is complete, passed, and successful.

Milestone 6 - Project Workspace Chat data model is complete, passed, and successful. It adds no new tables and reuses the existing project-scoped planning chat tables.

Milestone 9 - Manual Context Attachments data model is complete, passed, and successful. It adds a conversation-scoped context attachment table with a non-destructive migration.

Milestone 11 - Bridge File Drafting data model is complete, passed, and successful. It adds a project-scoped bridge draft table with a non-destructive migration.

Milestone 12 - Project Markdown Context data model is complete, passed, and successful. It adds a project-scoped Markdown context configuration table with a non-destructive migration, and was revalidated after Milestone 13's UI consolidation.

The Gaming data model adds local game sections plus game-scoped catalog tables for objects, references, and screenshot file-path records. Screenshot image bytes are not stored in SQLite.

Gaming Screenshot Capture is complete, passed, and successful for the current GearBlocks workflow. Screenshot metadata, capture manifests, thumbnail preview loading, and screenshot delete cleanup are validated.

## Tables

### scratchpad

```text
id
content
created_at
updated_at
```

Single-row quick capture surface from Milestone 0.

### tasks

```text
id
title
body
deadline
is_completed
created_at
updated_at
```

Local to-do items. Existing Milestone 1 databases are migrated non-destructively by adding `body` and `deadline` columns if they are missing.

### notes

```text
id
title
body
created_at
updated_at
```

Multiple saved note documents, separate from the scratchpad.

### calendar_events

```text
id
title
start_date
start_time
end_date
end_time
notes
created_at
updated_at
```

Simple local event list for Milestone 1. Recurrence, invites, and external calendar sync are deferred.

### projects

```text
id
name
description
status
created_at
updated_at
```

Local project records for Milestone 2. Valid Milestone 2 status values are `ACTIVE` and `ARCHIVED`; complex project lifecycle workflows are deferred.

### planning_conversations

```text
id
project_id
title
created_at
updated_at
```

Local planning chat conversation records for Milestone 3. Each conversation belongs to one local project.

Milestone 6 continues to use this table for Chat inside the selected Projects workspace. The selected project supplies `project_id`; the chat UI no longer asks the user to select the project a second time.

### planning_messages

```text
id
conversation_id
role
content
created_at
```

Local planning chat messages for Milestone 3. Valid role values are:

```text
user
assistant
system
```

Milestone 3 writes `user` and `assistant` messages during normal chat use. `system` is reserved for future workflow needs; the active planning instruction is backend-owned and not stored as a user-visible message.

### planning_conversation_context

```text
id
conversation_id
context_type
source_id
label
created_at
```

Manual context attachment records for Milestone 9. Each row belongs to one `planning_conversations.id` value through `conversation_id`.

Supported `context_type` values are:

```text
project
github_repository
note
task
calendar_event
youtube_reference
scratchpad
```

`source_id` stores the source record ID for local records when applicable. It may be null for singleton context such as Scratchpad. `label` stores the display label captured when the context is attached.

Deleting an attachment removes only the `planning_conversation_context` row. It does not delete the source project, GitHub repository link, note, task, calendar event, YouTube reference, or scratchpad content.

### bridge_file_drafts

```text
id
project_id
conversation_id
title
content
status
created_at
updated_at
```

Local bridge-file draft records for Milestone 11. Each row belongs to one `projects.id` value through `project_id` and links to a source `planning_conversations.id` value through `conversation_id`.

`title` stores a readable draft title.

`content` stores the generated Markdown bridge draft.

`status` defaults to:

```text
draft
```

Milestone 11 only uses `draft`. Approval, obsolete, sent, implemented, validated, and archived workflows are deferred.

Deleting a bridge draft removes only the `bridge_file_drafts` row. It does not delete the source project, planning conversation, planning messages, or attached context.

### project_markdown_context

```text
id
project_id
root_path
readme_path
created_at
updated_at
```

Project Markdown context configuration records for Milestone 12. Each row belongs to one `projects.id` value through `project_id`.

`root_path` stores the configured local project documentation root.

`readme_path` stores the README path relative to the configured root. The default is:

```text
README.md
```

Milestone 12 stores configuration only. It does not cache Markdown file snapshots in SQLite. Markdown files are read freshly from disk when project chat loads, Prompt Preview opens, a project chat message is sent, or a bridge draft is generated.

Deleting a project deletes that project's Markdown context configuration. It does not delete any local filesystem files.

### project_github_repositories

```text
id
project_id
repository_full_name
repository_url
default_branch
visibility
last_fetched_at
last_fetch_status
created_at
updated_at
```

Project-scoped GitHub repository linkage and metadata for Milestone 4. Each project can have one linked repository through the unique `project_id` field.

Minimum required linkage fields:

```text
id
project_id
repository_full_name
created_at
updated_at
```

Fetched metadata fields are populated by the backend GitHub metadata command when `GITHUB_TOKEN` is configured:

```text
repository_url
default_branch
visibility
last_fetched_at
last_fetch_status
```

GitHub tokens are not stored in SQLite.

### youtube_references

```text
id
title
url
video_id
channel_name
notes
tags
created_at
updated_at
```

Local user-curated YouTube references for Milestone 5. Minimum required fields are:

```text
id
title
url
created_at
updated_at
```

The backend parses and stores `video_id` from supported URL shapes when a reference is created or updated. `channel_name`, `notes`, and `tags` are optional user-entered metadata. No YouTube API key, account login, scraping, transcript retrieval, recommendations, downloads, or account sync data is stored.

### games

```text
id
name
slug
summary
created_at
updated_at
```

Local game section records for the Gaming workspace. `name` and `slug` are unique case-insensitively. `GearBlocks` is inserted automatically if no existing row uses that name or slug.

### game_catalog_objects

```text
id
game_id
name
object_type
category
category_icon
category_icon_path
description
notes
tags
thumbnail_path
source_screenshot_path
created_at
updated_at
```

Game-scoped catalog objects for item, block, entity, component, or similar object tracking. Rows belong to `games.id` through `game_id`.

The Parts cataloger uses this table to store recognizable GearBlocks parts from the screenshot set. `category` preserves the selected left-side GearBlocks category context from the source screenshot, `category_icon_path` stores a cropped image of that selected category icon for the filter button row, `thumbnail_path` points to the current source image used for the chart thumbnail, and `source_screenshot_path` records where the catalog entry came from.

### game_catalog_references

```text
id
game_id
object_id
title
reference_type
url
local_path
notes
tags
created_at
updated_at
```

Game-scoped reference records for future external URLs, local files, notes, and object-linked reference material. `object_id` is optional so references can belong to a whole game or to a specific catalog object.

### game_catalog_screenshots

```text
id
game_id
object_id
title
file_path
request_id
request_path
capture_status
captured_at
notes
tags
created_at
updated_at
```

Game-scoped screenshot metadata. `file_path` stores a local image path; screenshot image bytes are intentionally kept outside SQLite. `request_id` stores the unique capture identifier, `request_path` stores the local capture manifest JSON path, and `capture_status` stores the implementation path/status such as `captured_windows_gdi`. `object_id` is optional so screenshots can attach to a whole game or to a specific catalog object.

Screenshot capture currently supports an experimental Windows GDI capture path for testing. Overlay Forge hides itself, waits briefly, captures the foreground window through GDI `BitBlt`, forces all alpha values to 255, encodes a standard sRGB PNG, saves it under `game-screenshots/<game-slug>/`, and writes a capture manifest under `game-screenshots/capture-requests/`. The intended scope is the visible game display without Overlay Forge.

The preferred GearBlocks-compatible path remains internal game-engine export for future implementation. For GearBlocks and future Unity-rendered games, the ideal workflow is to read pixels from the rendered frame, create a `Texture2D`, force all alpha values to 255, encode as standard sRGB PNG, and save directly to disk using a name such as `GearBlocks_YYYYMMDD_HHMMSS_unique.png`. Avoid clipboard captures, `Win+Shift+S`, Snipping Tool, HDR formats, wide-gamut profiles, and alpha-dependent output.

The selected-game UI lists captured screenshot rows in a collapsible Screenshots section. The toolbar remains fixed at the top of the game pane, while screenshot previews and future catalog content scroll below it. Tauri asset loading is enabled for `game-screenshots/` so saved PNGs render as thumbnails inside the webview.

Right-clicking a screenshot opens the screenshot context menu. The validated delete action removes the saved PNG, the capture manifest JSON, the screenshot metadata row, and any `game_catalog_references` rows for the same game whose `local_path` points at the deleted screenshot or manifest.

## Migration Notes

Milestone 4 uses non-destructive idempotent table initialization:

```text
CREATE TABLE IF NOT EXISTS project_github_repositories
```

Existing user data remains intact. Deleting a local project deletes that project's GitHub repository linkage record, but does not affect any GitHub remote repository.

Milestone 5 uses non-destructive idempotent table initialization:

```text
CREATE TABLE IF NOT EXISTS youtube_references
```

Existing user data remains intact. Deleting a YouTube reference removes only that local SQLite row and does not affect YouTube or any external account.

Milestone 6 does not change the SQLite schema. Existing `projects`, `planning_conversations`, and `planning_messages` rows remain intact.

Milestone 9 uses non-destructive idempotent table initialization:

```text
CREATE TABLE IF NOT EXISTS planning_conversation_context
```

Existing user data remains intact. Deleting a planning conversation deletes that conversation's attachment links along with its messages.

Milestone 11 uses non-destructive idempotent table initialization:

```text
CREATE TABLE IF NOT EXISTS bridge_file_drafts
```

Existing user data remains intact. Bridge drafts are stored locally in SQLite and are not exported to disk by Milestone 11.

Milestone 12 uses non-destructive idempotent table initialization:

```text
CREATE TABLE IF NOT EXISTS project_markdown_context
```

Existing user data remains intact. The table stores only the configured local root and README path for each project. It does not store Markdown file contents.

Gaming uses non-destructive idempotent table initialization:

```text
CREATE TABLE IF NOT EXISTS games
CREATE TABLE IF NOT EXISTS game_catalog_objects
CREATE TABLE IF NOT EXISTS game_catalog_references
CREATE TABLE IF NOT EXISTS game_catalog_screenshots
```

Existing user data remains intact. Deleting a game removes that local game row and its local catalog object/reference/screenshot metadata rows. It does not delete screenshot image files from disk.

The local `game-screenshots/` folder is ignored by git because it contains generated capture request files and user screenshot images.
