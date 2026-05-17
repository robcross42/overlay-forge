# Milestone 13 - Project Workspace UI Consolidation

## Status

Planned / Not Started

## Baseline

Milestone 13 starts from the current Milestone 12 implementation state:

```text
Milestone 12 - Project Markdown Context
Status: Tentatively Validated / Needs UI Follow-up After Milestone 13
```

Milestone 12 successfully adds project-level local Markdown context ingestion, but the current nested Projects workspace UI prevents a clean validation pass. Milestone 13 should remove redundant workspace framing and give project chat most of the main panel.

## Goal

Consolidate the Projects workspace UI so the left navigation hierarchy owns project and conversation selection, while the main panel focuses on the selected task.

Milestone 13 should prove this workflow:

```text
Projects navigation tree -> selected project -> selected conversation -> focused chat surface
```

The user should be able to understand their location from the left navigation tree without redundant project headers, nested workspace shells, or repeated Chat labels in the main panel.

## Planned Capabilities

- Remove redundant Projects workspace containers from the main panel.
- Remove the selected-project Active Workspace header from the chat path.
- Remove the Overview / GitHub / Chat / References tab row from the main chat path.
- Remove the OpenAI Planning / Planning Chat heading from the chat path.
- Give the selected conversation message history and input most of the available main-panel space.
- Show planning conversations as child rows in the far-left Projects navigation hierarchy.
- Add a chat icon or compact chat affordance for conversation child rows.
- Selecting a conversation child row should open that conversation directly in the main chat surface.
- Move project-level secondary surfaces behind the project row `...` menu.
- Move Overview, Chat, References, Edit, and Delete access into the project row `...` menu.
- Move GitHub repository integration into the Project Edit screen.
- Move project Markdown context configuration into the Project Edit screen.
- Move local repo / local Markdown root configuration into the Project Edit screen.
- Move manual context attachment management out of the primary chat surface.
- Preserve existing Milestone 0 through Milestone 12 behavior.

## Navigation Target

The left navigation should become the main project hierarchy:

```text
Projects                         [+]
  overlay-forge                  [...]
    [chat] milestone planning
    [chat] validation notes
  another-project                [...]
```

Expected behavior:

- Clicking `Projects` opens a minimal Projects landing or preserves the current selected project.
- Clicking a project row selects the active project.
- Clicking a project row `...` exposes project-level actions.
- Clicking a conversation child row opens that conversation in the focused chat surface.
- Conversation rows should be visually distinct from project rows and use a chat icon or compact chat marker.

## Project Row Menu

The project row `...` menu should expose:

```text
Overview
Chat
References
Edit
Delete
```

`Edit` should open a clean Project Edit screen.

`Delete` should keep the existing confirmation/safe delete behavior.

## Project Edit Screen

The Project Edit screen should contain project-level configuration and secondary project settings:

- Project name
- Project description
- Project status
- GitHub repository linkage and metadata fetch controls
- Project Markdown context root
- README path
- Local repo / local documentation root configuration
- Project-level context status and warnings

The Project Edit screen should replace the old GitHub workspace tab as the place where GitHub integration is configured.

## Context Attachments Boundary

Milestone 12 project Markdown context is project-scoped and belongs in Project Edit.

Milestone 9 manual context attachments are conversation-scoped. Milestone 13 should move their UI out of the primary chat surface, but it should not silently change the data model or make conversation attachments project-scoped.

Acceptable Milestone 13 options:

- Add a compact Conversation Context / Settings drawer for the selected conversation.
- Add a secondary Context view reachable from the conversation row or project row menu.
- Keep the existing `planning_conversation_context` data model while removing attachment controls from the main chat layout.

Do not convert manual conversation attachments into project-level records unless a later milestone explicitly changes the data model.

## Focused Chat Surface

The focused chat surface should prioritize:

- Selected conversation title or compact label
- Message history
- Message input
- Send action
- Compact Prompt Preview action
- Compact Draft Bridge File action

The main chat surface should not show large project configuration panels, GitHub configuration, Markdown root configuration, manual attachment forms, or redundant workspace labels.

## Out Of Scope

Milestone 13 does not implement:

- New context data model changes
- Project-level conversion of manual conversation attachments
- GitHub file browsing
- GitHub file reading through the GitHub API
- Bridge-file export
- Copy-to-clipboard bridge workflow
- Full bridge-file editor
- Direct Codex handoff
- Chat streaming
- Model picker UI
- Token budgeting
- Vector stores
- Semantic search
- ChatGPT import
- Broad repository indexing

## Data Requirements

Prefer no schema changes.

Milestone 13 should primarily be a frontend navigation and layout refactor using existing data:

```text
projects
planning_conversations
planning_messages
planning_conversation_context
project_github_repositories
project_markdown_context
bridge_file_drafts
```

If a small additive field is absolutely necessary, use a non-destructive migration and update `docs/DATA_MODEL.md`.

## Setup Validation

Run:

```powershell
npm install
```

Expected result:

```text
Dependencies install successfully.
```

Run:

```powershell
npm run build
```

Expected result:

```text
Frontend builds successfully.
```

Run:

```powershell
cd src-tauri
cargo build
```

Expected result:

```text
Rust backend compiles successfully.
```

Run:

```powershell
npm run tauri:dev
```

Expected result:

```text
App launches successfully in development mode.
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
Expand Projects in the left navigation tree.
```

Pass criteria:

```text
Projects show saved project rows and conversation child rows.
```

Validate:

```text
Select a conversation child row.
```

Pass criteria:

```text
The focused chat surface opens directly and gives most of the main panel to messages and input.
```

Validate:

```text
Review the selected conversation chat surface.
```

Pass criteria:

```text
Redundant Projects, Local Projects, Active Workspace, workspace tab, and Planning Chat headers are removed from the main chat path.
```

Validate:

```text
Open the project row ... menu.
```

Pass criteria:

```text
Overview, Chat, References, Edit, and Delete actions are available.
```

Validate:

```text
Open Project Edit.
```

Pass criteria:

```text
Project details, GitHub integration, local Markdown root, README path, and local repo/context configuration are available in a clean edit screen.
```

Validate:

```text
Configure or reload Project Markdown context from Project Edit.
```

Pass criteria:

```text
Markdown context configuration persists and context warnings remain readable.
```

Validate:

```text
Open Prompt Preview from the focused chat surface.
```

Pass criteria:

```text
Prompt Preview still shows project Markdown context and conversation manual attachments.
```

Validate:

```text
Send a project chat message.
```

Pass criteria:

```text
Existing chat send behavior still works and includes project Markdown context.
```

Validate:

```text
Draft a bridge file from the focused chat surface.
```

Pass criteria:

```text
Bridge drafts still include project Markdown context and resolved manual attachments.
```

Validate:

```text
Return to Scratchpad, Tasks, Notes, Calendar, Projects, YouTube, Prompt Preview, Bridge Drafts, and existing app controls.
```

Pass criteria:

```text
Existing Milestone 0 through Milestone 12 behavior still works.
```

## Milestone 12 Revalidation

After Milestone 13 is implemented, run a second Milestone 12 validation pass with the consolidated UI.

Confirm:

- Project Markdown context is easier to configure from Project Edit.
- Chat uses project Markdown context without crowding the main surface.
- Prompt Preview shows loaded Markdown files.
- Bridge drafts include loaded Markdown files.
- Manual context attachments remain scoped correctly.

## User Pass/Fail Reporting Format

```markdown
# Milestone 13 Validation Results

## Overall Result

Pass or Fail

## Failed Items

- Item:
  - Expected:
  - Actual:

## Notes

Any extra observations.
```
