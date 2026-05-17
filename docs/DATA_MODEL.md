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
