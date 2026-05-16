# Data Model

## Status

Milestone 1 data model is complete, passed, and successful.

Milestone 2 project data model is complete, passed, and successful.

Milestone 3 planning chat data model is complete, passed, and successful.

Milestone 4 - GitHub Integration data model is complete, passed, and successful.

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

## Migration Notes

Milestone 4 uses non-destructive idempotent table initialization:

```text
CREATE TABLE IF NOT EXISTS project_github_repositories
```

Existing user data remains intact. Deleting a local project deletes that project's GitHub repository linkage record, but does not affect any GitHub remote repository.
