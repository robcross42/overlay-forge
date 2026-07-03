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
Switch between Calendar, Cessation, Repair Resell, Gaming, YouTube, and Settings.
```

Pass criteria:

```text
Each active module loads without disrupting persisted data.
```

```text
Restart the app.
```

Pass criteria:

```text
Persisted records restore correctly.
```

## Retired Projects Module

The former Projects module has no active manual validation path. Legacy SQLite rows are preserved for data safety only.

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
