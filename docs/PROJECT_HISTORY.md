# Overlay Forge Project History

This file keeps milestone history in one compact reference. It replaces separate active `MILESTONE_*.md` files.

Historical milestone notes may mention old table names or old UI names. For current implementation work, prefer:

- `AGENTS.md`
- `docs/PROJECT_OVERVIEW.md`
- `docs/ARCHITECTURE.md`
- `docs/DATA_MODEL.md`
- `docs/FEATURE_SCOPE.md`
- `docs/VALIDATION.md`

## Milestone Summary

| Milestone | Status | Result |
| --- | --- | --- |
| 0 - Overlay Shell Validation | Complete / Passed / Successful | Proved Tauri overlay shell, global hotkey, component host, SQLite init, and scratchpad persistence. |
| 1 - Calendar, To-do, Notes, and Scratchpad Expansion | Complete / Passed / Successful | Added local Scratchpad, Tasks, Notes, and Calendar navigation, CRUD, and persistence. |
| 2 - Local Projects Component | Complete / Passed / Successful | Added local Projects records, project CRUD, and project UI behavior. |
| 3 - OpenAI Planning Chat Component | Complete / Passed / Successful | Added backend-owned OpenAI Responses API calls, project-scoped conversations, and persisted messages. |
| 4 - GitHub Integration | Complete / Passed / Successful | Added project-scoped GitHub repository metadata linkage through backend-owned `GITHUB_TOKEN`. |
| 5 - Controlled YouTube Component | Complete / Passed / Successful | Added user-curated YouTube reference storage, validation, edit/delete, and external open behavior. |
| 6 - Project Workspace Chat | Complete / Passed / Successful | Moved planning chat into the selected Projects workspace and removed the separate project selector from chat. |
| 7 - Project Workspace Layout Refinement | Complete / Passed / Successful | Added Overview, GitHub, Chat, and References sections for the selected project workspace. |
| 8 - Projects Navigation Tree Actions | Complete / Passed / Successful | Moved project create/select/edit/delete actions into the left navigation tree. |
| 9 - Manual Context Attachments | Complete / Passed / Successful | Added conversation-scoped links to local context records. |
| 10 - Prompt Preview | Complete / Passed / Successful | Added a read-only local prompt preview surface without calling OpenAI. |
| 11 - Local Markdown Implementation Request Drafts | Complete / Passed / Successful | Added project-scoped local Markdown draft generation from project chat and resolved context. |
| 12 - Project Markdown Context | Complete / Passed / Successful | Added project-level Markdown context loading from configured local roots. |
| 13 - Project Workspace UI Consolidation | Complete / Passed / Successful | Consolidated Projects around the left navigation tree, focused chat surface, Project Edit, and right-hand context pane. |

## Current Baseline Detail

Milestone 13 is the active stable baseline.

Current project behavior:

- Projects navigation owns project selection.
- Conversation rows appear under project rows.
- Selecting a conversation opens a focused chat surface.
- Project row actions route to Overview, New Chat, References, Edit, and Delete.
- Project Edit owns project details, GitHub repository configuration, and project Markdown context configuration.
- Conversation-scoped context attachments remain conversation-scoped.
- The right-hand chat pane owns context references and local implementation request drafts.
- The focused chat composer keeps only the draft field and Send action.
- No schema changes were required for Milestone 13.

## Addenda After Milestone 13

### Gaming Screenshot Capture

Status: Complete / Passed / Successful.

Validated workflow:

```text
Gaming -> selected game -> Capture Screenshot -> saved PNG -> in-app thumbnail preview -> screenshot context menu -> delete cleanup
```

The validated implementation hides Overlay Forge before capture, captures the visible foreground game display through Windows GDI, forces PNG alpha values to 255, stores PNG files under `game-screenshots/<game-slug>/`, writes capture manifests under `game-screenshots/capture-requests/`, persists screenshot metadata in SQLite, renders thumbnails through Tauri asset loading, and cleans related files/rows on delete.

### GearBlocks Runtime API Interface Support

Status: Complete / Passed / Successful.

Overlay Forge indexes documented GearBlocks construction runtime interface/member availability, stores availability metadata from runtime exports, and displays API attribute availability in catalog details. Default chat context excludes API values unless a future explicit include/snapshot control is added.

### Smoking Cessation

Status: Implemented / Pending Validation.

Smoking Cessation records cigarette events locally in SQLite, tracks current cigarette inventory, displays the `Nicoderm Step 1` patch marker started at `2026-06-21 15:00:00 EDT`, supports a configurable record-cigarette keybind, and keeps a narrow Markdown export current for external review.

### Scheduler Framework

Status: Implemented / Pending Validation.

Scheduler adds local-first, backend-owned recurring work through explicit Rust handlers, scheduler rows, leases, and run history. Scheduler rows must not execute arbitrary commands or scripts.

### SQLite Naming Normalization

Status: Implemented / Pending Validation.

SQLite persistence now follows:

```text
obj_ = dynamic object rows
def_ = static definition rows
o2o_ = one-to-one mappings
n2n_ = many-to-many mappings
```

### Path of Exile 2 Game Module

Status: Scaffolded / Pending Capability Buildout.

Path of Exile 2 is seeded as a Gaming module with frontend section targets for Home, Chats, Builds, Skill Tree, Items, Skill Gems, Support Gems, Loot Filter, and Trade. Add persistence only when a concrete capability requires it.

## Historical Validation Pattern

Most completed milestones used the same setup validation pattern:

```powershell
npm install
npm run build
cd src-tauri
cargo build
npm run tauri:dev
```

Use `docs/VALIDATION.md` for current validation expectations.
