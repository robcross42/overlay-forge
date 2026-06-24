# Overlay Forge Data Model

## Status

SQLite is the local source of truth. Current persistence follows convention-based table naming:

| Prefix | Meaning |
| --- | --- |
| `obj_` | Dynamic object rows. |
| `def_` | Static definition rows. |
| `o2o_` | One-to-one mapping rows. |
| `n2n_` | Many-to-many mapping rows. |

Migrations must be non-destructive and idempotent. Existing `updated_at` columns remain for frontend/API compatibility until a later explicit cleanup can safely remove or rename them.

If older databases contain tables from retired workflows, preserve them unless the user explicitly requests a cleanup migration.

## Core Tables

### `obj_scratchpad`

```text
id
content
created_at
updated_at
```

Single-row quick capture surface.

### `obj_task`

```text
id
title
body
deadline
is_completed
created_at
updated_at
```

Local to-do items. Existing databases are migrated non-destructively by adding missing columns.

### `obj_note`

```text
id
title
body
created_at
updated_at
```

Saved note documents.

### `obj_calendar_event`

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

Simple local event list. Recurrence, invites, and external calendar sync are deferred.

## Project Tables

### `obj_project`

```text
id
name
description
status
created_at
updated_at
```

Local project records. Valid initial status values are `ACTIVE` and `ARCHIVED`.

### `obj_planning_conversation`

```text
id
project_id
title
created_at
updated_at
```

Local planning chat conversations. Each conversation belongs to one project.

### `obj_planning_message`

```text
id
conversation_id
role
content
created_at
```

Local planning chat messages.

Valid role values:

```text
user
assistant
system
```

### `n2n_planning_conversation_context`

```text
id
conversation_id
context_type
source_id
label
created_at
```

Manual conversation-scoped context attachment links.

Supported `context_type` values:

```text
project
github_repository
note
task
calendar_event
youtube_reference
obj_scratchpad
```

Deleting an attachment removes only the attachment row. It must not delete the source record.

### `obj_project_markdown_context`

```text
id
project_id
root_path
readme_path
created_at
updated_at
```

Project Markdown context configuration. Stores only configured paths. Markdown file content is read freshly from disk when needed and is not cached in SQLite.

The default README path is:

```text
README.md
```

### `obj_project_github_repository`

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

Project-scoped GitHub repository linkage and metadata. GitHub tokens are not stored in SQLite.

## YouTube Tables

### `obj_youtube_reference`

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

Local user-curated YouTube references. The backend parses and stores `video_id` from supported URL shapes.

No YouTube API key, account login, scraping, transcript retrieval, recommendations, downloads, or account sync data is stored.

## Gaming Tables

### `def_game`

```text
id_game
game_key
ui_name
summary
schema_json
created_at
modified_at
```

Static game definitions. `GearBlocks` is seeded with `id_game = 1`. `Path of Exile 2` is seeded with `id_game = 2`.

### `obj_game`

```text
id
id_game
name
slug
summary
created_at
updated_at
```

Local game section records. `id_game` links each local section to `def_game.id_game`.

### `obj_game_setting`

```text
id
game_id
id_game
setting_key
setting_value_json
schema_json
created_at
modified_at
```

Generic per-game settings. This avoids table-per-game settings growth.

### `obj_game_data_location`

```text
id
game_id
location_type
label
directory_path
created_at
updated_at
```

Game-scoped local directory records for save folders or alternate game data folders. Current location types include `save` and `alternate`.

### `obj_game_catalog_object`

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

Game-scoped catalog objects for item, block, entity, component, or similar tracking.

### `obj_game_catalog_reference`

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

Game-scoped reference records. `object_id` is optional so references can belong to a whole game or a specific catalog object.

### `obj_game_catalog_screenshot`

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

Screenshot metadata. Screenshot image bytes are stored as PNG files outside SQLite.

## GearBlocks Tables

### `obj_game_construction`

```text
id
game_id
name
folder_path
construction_path
byte_size
decoded_byte_size
composite_count
part_count
unique_asset_guid_count
attachment_count
link_count
intersection_count
is_frozen
is_invulnerable
summary_json
document_json
last_indexed_at
created_at
updated_at
```

GearBlocks saved construction index populated from folders under `SavedConstructions`.

### `obj_game_runtime_construction_export`

```text
id
game_id
export_id
name
export_kind
intended_path
source_log_path
byte_size
construction_id
exported_at
part_count
mass
is_frozen
is_invulnerable
is_player_character
document_json
last_indexed_at
created_at
updated_at
```

GearBlocks runtime construction export index populated from Overlay Forge Lua exporter records reconstructed from `Player.log` / `Player-prev.log`.

### `obj_game_runtime_part`

```text
id
game_id
part_key
asset_guid
asset_name
display_name
full_display_name
category
mass
world_x
world_y
world_z
local_x
local_y
local_z
world_position_json
local_position_json
properties_json
source_export_id
source_construction_id
last_seen_at
display_image_path
source_image_path
notes
created_at
updated_at
```

GearBlocks runtime part index. `part_key` prefers `AssetGUID`, falls back to `AssetName`, then to category plus display name.

### `obj_game_runtime_part_alias`

```text
id
game_id
part_instance_key
friendly_name
asset_guid
asset_name
display_name
full_display_name
category
source_log_path
source_construction_id
world_position_json
local_position_json
current_unit_size_json
payload_json
last_seen_at
created_at
updated_at
```

User-defined GearBlocks friendly names for exact runtime part instances. Rows are imported from the Lua script's `Player.log` alias events and are matched back to latest scene parts by `part_instance_key`.

### `obj_game_runtime_part_api_attribute`

```text
id
game_id
part_key
asset_guid
asset_name
display_name
full_display_name
category
interface_name
attribute_name
value_type
availability
source_export_id
source_construction_id
first_seen_at
last_seen_at
created_at
updated_at
```

Runtime API availability index populated from imported runtime export parts after compact API references are hydrated.

### `obj_game_runtime_part_value`

```text
id
game_id
part_key
asset_guid
asset_name
display_name
full_display_name
category
field_path
value_type
value_json
source_export_id
source_construction_id
first_seen_at
last_seen_at
created_at
updated_at
```

Runtime value-field index populated from imported part JSON fields named `value`.

### `obj_game_runtime_part_property`

```text
id
game_id
part_key
asset_guid
asset_name
display_name
full_display_name
category
property_path
value_type
value_json
source_export_id
source_construction_id
first_seen_at
last_seen_at
created_at
updated_at
```

Runtime property index populated from leaf values under each part's `properties` object.

### `obj_game_runtime_part_attachment`

```text
id
game_id
part_key
asset_guid
asset_name
display_name
full_display_name
category
attachment_path
value_type
attachment_json
source_export_id
source_construction_id
first_seen_at
last_seen_at
created_at
updated_at
```

Runtime attachment index populated from direct entries under each part's `attachments` field.

### `def_gearblocks_api_type`

```text
id
namespace
type_name
type_kind
docs_url
source
source_version
notes
created_at
updated_at
```

Canonical GearBlocks API type catalog.

### `def_gearblocks_api_member`

```text
id
type_id
member_key
member_name
signature
member_kind
return_type
is_readable
is_writable
is_invokable
is_mutating
docs_url
source
source_version
notes
created_at
updated_at
```

Canonical GearBlocks API member catalog.

### `def_gearblocks_api_parameter`

```text
id
member_id
position
parameter_name
parameter_type
default_value
is_optional
created_at
updated_at
```

Canonical GearBlocks API method parameter catalog.

### `def_gearblocks_api_enum_value`

```text
id
type_id
position
value_name
numeric_value
lua_name
description
source
source_version
created_at
updated_at
```

Canonical GearBlocks API enum value catalog.

### `n2n_game_runtime_part_api_member`

```text
id
game_id
part_key
api_member_id
availability
source_export_id
source_construction_id
first_seen_at
last_seen_at
created_at
updated_at
```

Runtime link from observed part API availability to canonical GearBlocks API members.

## Game Chat Tables

### `obj_game_chat_conversation`

```text
id
game_id
title
overlay_x
overlay_y
created_at
updated_at
```

Game-scoped chat conversation records. Overlay coordinates are nullable and restore the simple game chat overlay position.

### `obj_game_chat_message`

```text
id
conversation_id
role
content
created_at
```

Game chat messages scoped to `obj_game_chat_conversation`.

## Game Build Guide Tables

### `obj_game_build_guide`

```text
id
game_id
title
source_path
raw_markdown
build_goal
scale_reference
geometry_notes
checklist_json
overlay_x
overlay_y
overlay_width
overlay_height
created_at
updated_at
```

Game-scoped Markdown build guide imports. The raw Markdown is preserved for future parsing improvements while current UI rendering uses structured summary fields plus parsed part and step child rows. Overlay bounds are nullable and restore the independent build-guide overlay position and size.

### `obj_game_build_guide_part`

```text
id
guide_id
section
quantity
part_name
purpose
row_order
created_at
updated_at
```

Parsed build guide part rows grouped by section. Rows are rendered as compact stacked entries in the build-guide overlay to avoid wide table layouts in narrow side-panel use.

### `obj_game_build_guide_step`

```text
id
guide_id
step_number
title
body
row_order
created_at
updated_at
```

Parsed numbered assembly instructions for in-game reference. Automated construction, validation against live scene state, and direct GearBlocks API execution are deferred.

## Smoking Cessation Tables

### `obj_smoking_event`

```text
id
smoked_at
source
notes
created_at
```

Local cigarette event records.

### `obj_smoking_cessation_setting`

```text
id
patch_label
patch_started_at
patch_timezone
current_cigarette_count
created_at
updated_at
```

Singleton settings row. `id` is constrained to `1`.

The initial patch marker is:

```text
patch_label = Nicoderm Step 1
patch_started_at = 2026-06-21 15:00:00
patch_timezone = EDT
```

## Scheduler Tables

### `def_scheduler_type`

```text
id
scheduler_key
label
description
created_at
modified_at
```

Static scheduler type definitions. `scheduler_key` maps a schedule row to a known Rust handler.

Initial seeded value:

```text
1 = smoking_cessation_export
```

### `obj_scheduler`

```text
id
id_type
owner_module
name
is_enabled
interval_seconds
run_on_startup
coalesce_missed_runs
payload_json
next_run_at
last_run_at
last_status
last_error
lease_until
created_at
modified_at
```

Dynamic scheduler records. Scheduler rows must not execute arbitrary commands or scripts.

### `obj_scheduler_run`

```text
id
id_scheduler
id_type
started_at
finished_at
status
message
created_at
modified_at
```

Run history for scheduled events.

## Migration Notes

New or changed tables should be initialized using non-destructive `CREATE TABLE IF NOT EXISTS` and `ALTER TABLE ... ADD COLUMN` style migrations where appropriate.

Renames should preserve data and account for existing legacy names. Deleting a parent object may cascade to local child metadata where the app already owns that data, but must not delete external resources such as GitHub repositories, YouTube data, game save files, or arbitrary local filesystem content.

The local `game-screenshots/` folder is ignored by git because it contains generated capture request files and user screenshot images.
