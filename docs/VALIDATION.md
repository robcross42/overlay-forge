# Overlay Forge Validation

## Default Validation Commands

Use validation appropriate to the changed area.

```powershell
npm install
npm run build
npm run cargo:build
npm run cargo:clippy
npm run tauri:dev
```

Do not run broader validation than needed for a small change unless the request affects shared behavior.

## Validation Matrix

| Changed area | Preferred validation |
| --- | --- |
| React / TypeScript / frontend UI | `npm run build` |
| Rust / Tauri backend | `npm run cargo:build` |
| Frontend + backend behavior | `npm run check` |
| SQLite migrations | both builds plus migration review |
| OpenAI request path | backend build plus chat-send manual check if possible |
| GitHub integration | backend build plus missing-token and valid-token behavior review if possible |
| Screenshot capture | frontend/backend builds plus manual capture/delete flow |
| GearBlocks script exporter | build/type-check plus in-game load/export/import check where possible |
| BepInEx plugin | plugin build plus install/run check where possible |
| Scheduler | backend build plus startup/interval/run-history behavior review |
| Smoking Cessation | frontend/backend builds plus event/keybind/export behavior review |

For broad cleanup and architecture work, run `npm run cargo:clippy` as a review pass. Fix clear no-risk warnings immediately. Record larger warnings as explicit refactor work when they require changing public command shapes, repository APIs, or multiple call sites.

## Core Manual Regression Checklist

Use when changes are broad or touch shell/shared state.

```text
Open the app and reveal the overlay with Ctrl+Shift+Space.
```

Pass criteria:

```text
Overlay appears using existing hotkey behavior.
```

```text
Switch between Scratchpad, Tasks, Notes, Calendar, Projects, YouTube, and Gaming.
```

Pass criteria:

```text
Each module loads without disrupting persisted data.
```

```text
Restart the app.
```

Pass criteria:

```text
Persisted records restore correctly.
```

## Projects And Chat Validation

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
Open the project row ... menu.
```

Pass criteria:

```text
Overview, New Chat, References, Edit, and Delete actions are available.
```

Validate:

```text
Click New Chat from the project row ... menu.
```

Pass criteria:

```text
The focused chat surface opens a new-conversation area and does not auto-select an existing conversation.
```

Validate:

```text
Open and collapse the right-hand chat pane.
```

Pass criteria:

```text
Context references and local implementation request drafts appear in the right-hand pane, and collapsing the pane gives the chat surface more horizontal space.
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
Send a project chat message.
```

Pass criteria:

```text
Existing chat send behavior works and includes project Markdown context when configured.
```

## Project Markdown Context Validation

Validate:

```text
Configure a local Markdown documentation root for the selected project.
```

Pass criteria:

```text
The project stores the configured root without affecting other projects.
```

Validate:

```text
Open or start a project chat where the configured root contains README.md.
```

Pass criteria:

```text
README.md is loaded freshly from the configured local root.
```

Validate:

```text
Use a README.md that references Markdown files inside the configured project root.
```

Pass criteria:

```text
Referenced Markdown files are resolved and included when they remain inside the configured project root.
```

Validate:

```text
Use a README.md that references a missing file or a file outside the configured root.
```

Pass criteria:

```text
The app shows a readable warning and does not crash.
```

## Gaming Screenshot Validation

Validated workflow:

```text
Gaming -> selected game -> Capture Screenshot -> saved PNG -> in-app thumbnail preview -> screenshot context menu -> delete cleanup
```

Validate:

```text
Capture a screenshot while a game is the foreground window.
```

Pass criteria:

```text
Overlay Forge hides before capture, the PNG is saved under game-screenshots/<game-slug>/, and the app restores afterward.
```

Validate:

```text
Preview the screenshot in the selected game pane.
```

Pass criteria:

```text
The thumbnail renders through the Tauri asset path.
```

Validate:

```text
Right-click the screenshot and delete it.
```

Pass criteria:

```text
The screenshot PNG, capture manifest, screenshot metadata row, and matching local-path reference rows are removed.
```

## GearBlocks Validation

When GearBlocks runtime work changes, validate as much of this path as possible:

```text
Install or update the Overlay Forge GearBlocks script.
Launch GearBlocks.
Load the script in Script Mods.
Run Export Scene.
Refresh/import scene context in Overlay Forge.
Open GearBlocks chat or parts details.
```

Pass criteria:

```text
Runtime exports import without reparsing unchanged log prefixes, parts are indexed, and chat context reflects the latest available scene state.
```

For future GearBlocks plugin backlog work:

```text
Build the plugin.
Install the DLL under GearBlocks/BepInEx/plugins.
Restart GearBlocks.
Send a ping command, or a marker command only if marker work has explicitly resumed.
```

Pass criteria:

```text
The plugin processes command files and writes status output.
```

## User Pass/Fail Reporting Format

```markdown
# Validation Results

## Overall Result

Pass or Fail

## Failed Items

- Item:
  - Expected:
  - Actual:

## Notes

Any extra observations.
```
