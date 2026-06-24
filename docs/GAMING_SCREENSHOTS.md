# Gaming Screenshot Capture

## Status

Complete / Passed / Successful

## Scope

This document covers the current Overlay Forge Gaming screenshot workflow for GearBlocks and future game sections.

Validated workflow:

```text
Gaming -> selected game -> Capture Screenshot -> saved PNG -> in-app thumbnail preview -> screenshot context menu -> delete cleanup
```

## Validated Capabilities

- The left navigation exposes Gaming as an expandable section with game rows nested underneath it.
- GearBlocks is available as the initial game workspace.
- Selecting a game opens a game pane with a fixed action toolbar at the top.
- Capture Screenshot hides Overlay Forge before capture so the overlay is not included in the saved image.
- The current capture implementation uses Windows GDI `BitBlt` against the foreground window.
- Captures are saved as unique PNG files under `game-screenshots/<game-slug>/`.
- Capture request manifests are saved under `game-screenshots/capture-requests/`.
- Screenshot metadata is persisted in SQLite through `obj_game_catalog_screenshot`.
- The `game-screenshots/` folder is ignored by git.
- Screenshot previews render in the selected game's collapsible Screenshots section.
- The game pane keeps action buttons fixed while screenshot previews scroll below them.
- Successful captures show a temporary floating `Successful` bubble instead of a blocking alert.
- Right-clicking a screenshot opens an app-owned screenshot context menu.
- The delete action removes the saved PNG, capture manifest JSON, screenshot database row, and matching local-path reference rows.
- Missing screenshot files are filtered out of the preview list so manually cleaned folders do not leave broken thumbnail cards.

## Current Capture Method

The current implementation uses Windows GDI capture for the visible foreground game display.

Capture path:

1. Create the game screenshot output folder if needed.
2. Create the capture request manifest folder if needed.
3. Build a unique request id from timestamp and process information.
4. Write a JSON capture manifest.
5. Hide Overlay Forge.
6. Capture the foreground window through Windows GDI.
7. Force all PNG alpha values to 255.
8. Save the PNG to disk.
9. Restore Overlay Forge.
10. Persist the screenshot metadata row.

## Future Preferred Game-Internal Capture

For GearBlocks and other Unity-rendered games, the preferred long-term capture path is game-internal frame export:

```text
rendered frame -> read pixels -> Texture2D -> force alpha 255 -> Unity PNG encoding -> save to disk
```

Avoid clipboard captures, `Win+Shift+S`, Snipping Tool dependency, HDR formats, wide-gamut output, and alpha-dependent image files.

## Validation Commands Previously Used

```powershell
npm run build
cargo build
```
