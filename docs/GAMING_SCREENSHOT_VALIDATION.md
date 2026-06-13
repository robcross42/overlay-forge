# Gaming Screenshot Capture Validation

## Status

Complete / Passed / Successful

## Scope

This validation covers the current Overlay Forge Gaming screenshot workflow for GearBlocks and future game sections.

The validated workflow is:

```text
Gaming -> selected game -> Capture Screenshot -> saved PNG -> in-app thumbnail preview -> screenshot context menu -> delete cleanup
```

## Validated Capabilities

- The left navigation exposes Gaming as an expandable section with game rows nested underneath it.
- GearBlocks is available as the initial game workspace.
- Selecting a game opens a blank game pane with a fixed action toolbar at the top.
- The Capture Screenshot action hides Overlay Forge before capture so the overlay is not included in the saved image.
- The current capture implementation uses Windows GDI `BitBlt` against the foreground window for practical validation.
- Captures are saved as unique PNG files under `game-screenshots/<game-slug>/`.
- Capture request manifests are saved under `game-screenshots/capture-requests/`.
- Saved screenshot metadata is persisted in SQLite through `game_catalog_screenshots`.
- The `game-screenshots/` folder is ignored by git.
- Screenshot previews render in the selected game's collapsible Screenshots section.
- The game pane keeps action buttons fixed while screenshot previews scroll below them.
- Successful captures show a temporary floating `Successful` bubble instead of a blocking alert.
- Right-clicking a screenshot opens an app-owned screenshot context menu.
- The screenshot context menu includes visual-test actions for future object/reference workflows.
- The delete action removes the saved PNG, capture manifest JSON, screenshot database row, and matching local-path reference rows.
- Missing screenshot files are filtered out of the preview list so manually cleaned folders do not leave broken thumbnail cards.

## Validation Notes

- The feature has been manually validated by creating screenshot records, saving PNG output, rendering previews in-app, and confirming the overlay is not included in the captured image.
- The app now enables Tauri asset loading for `game-screenshots/`, which is required for webview thumbnail previews.
- The backend constrains screenshot file deletion to the `game-screenshots/` tree before removing files.
- Build validation passed after implementation with:

```powershell
npm run build
cargo build
```

## Current Capture Method

The current completed validation target uses Windows GDI capture for the visible foreground game display. This was intentionally chosen after initial engine-internal capture planning so the workflow could be tested directly from Overlay Forge.

The capture path:

1. Creates the game screenshot output folder if needed.
2. Creates the capture request manifest folder if needed.
3. Builds a unique request id from timestamp and process information.
4. Writes a JSON capture manifest.
5. Hides Overlay Forge.
6. Captures the foreground window through Windows GDI.
7. Forces all PNG alpha values to 255.
8. Saves the PNG to disk.
9. Restores Overlay Forge.
10. Persists the screenshot metadata row.

## Future Preferred Game-Internal Capture

For GearBlocks and other Unity-rendered games, the preferred long-term capture path remains game-internal frame export:

```text
rendered frame -> read pixels -> Texture2D -> force alpha 255 -> Unity PNG encoding -> save to disk
```

That future path should avoid clipboard captures, `Win+Shift+S`, Snipping Tool dependency, HDR formats, wide-gamut output, and alpha-dependent image files.

