# GearBlocks Runtime And Construction Data

## Local Save Format

GearBlocks saved construction folders contain `construction.bytes` files.

Current local samples decode as:

```text
construction.bytes -> raw DEFLATE inflate -> BSON document -> JSON summary
```

The decoded BSON exposes construction flags and saved structure including:

- `isFrozen`
- `isInvulnerable`
- `composites`
- per-part `assetGUID`, local position, and local orientation
- `partData`, including dimensions, paint, and behaviour data where saved
- `attachments`
- `links`
- `intersections`

Overlay Forge converts binary vector payloads into JSON arrays, converts asset GUID binary values into unsigned string identifiers, and shows a compact summary plus the raw decoded JSON.

## Runtime API Boundary

The local save file does not include enough human-readable part metadata to reliably name every part by itself. The GearBlocks Lua scripting API exposes a richer runtime view after a construction is loaded in-game.

Relevant runtime surfaces:

- `IConstruction`: construction part count, composite count, frozen/invulnerable state, mass, active stage, `Parts`, and `GetPart(idx)`.
- `IPart`: asset GUID, asset name, category, display name, full display name, mass, stage, behaviours, link nodes, transforms, and bounds.
- `Constructions`: Lua global for construction instances.
- `PopConstructions`: Lua global for spawning saved constructions.

## Lua Exporter

Overlay Forge can install the script mod from:

```text
Gaming -> GearBlocks -> Home -> Construction Decoder -> Install Exporter
```

The installer writes:

```text
%USERPROFILE%\AppData\LocalLow\SmashHammer Games\GearBlocks\ScriptMods\OverlayForgeConstructionExporter\main.lua
%USERPROFILE%\AppData\LocalLow\SmashHammer Games\GearBlocks\ScriptMods\OverlayForgeConstructionExporter\meta.json
```

The script mod install path is always GearBlocks' standard user script directory under the default user data location. It does not follow the configured Save Location or Alternate Data Location.

After installing, launch or restart GearBlocks so the saved script mod is discovered, then open GearBlocks' Script Mods screen and load `Overlay Forge`.

## Export Behavior

The script window exposes `Export Scene` as the default export action.

It exports all currently loaded scene parts from `Parts.Instances`, so parts do not need to be attached into one construction before exporting. If scene-wide part enumeration is unavailable, the exporter falls back to all currently loaded constructions from `Constructions.Instances`.

GearBlocks currently blocks `io.open` in the script context, so the exporter falls back to printing marked JSON chunks into `Player.log`.

Overlay Forge reconstructs completed chunks from:

```text
Player.log
Player-prev.log
```

The backend stores per-log import cursors in SQLite so refresh and send paths read only new log additions when possible.

## Chat Scene Refresh

The GearBlocks script no longer monitors `Parts.Instances` for automatic scene deltas during normal gameplay.

Normal GearBlocks chat Send / Enter does not request a new scene export, parse runtime logs, or include the scene diff. It assembles prompt context from the latest normalized SQLite scene rows already imported into Overlay Forge.

When the user clicks the GearBlocks chat `D↑` action, chat includes the latest scene diff already stored in SQLite. It does not request a new in-game export or parse runtime logs.

When the user refreshes scene context manually, Overlay Forge focuses the remembered GearBlocks window, sends the script's `Ctrl+Shift+E` export shortcut, waits for a completed rich full-scene export in `Player.log`, imports that exact log append into SQLite, computes the latest-vs-previous scene diff, and stores the updated scene context for future chat sends.

The imported full export includes the complete current scene snapshot. Overlay Forge computes the latest-vs-previous runtime export diff in the backend and stores that diff for chat context. Removed parts disappear from latest chat context after the next manual full scene export/import.

Successful imports normalize the payload before storage. Raw full-scene export JSON is not retained in SQLite after the import succeeds; `obj_game_runtime_construction_export` remains an import manifest, `def_gearblocks_part` stores reusable part definitions, and `obj_game_runtime_part_instance` stores the latest scene's repeated physical part instances.

Reusable metadata is normalized into definition tables for part metadata fields, attachment types, part settings, and output/control channels. Runtime value tables map the current observed values back to those definitions so chat context can reference stable DB concepts without retaining the full export payload.

GearBlocks chat prompt context is assembled from normalized SQLite rows through the backend scene-context service. The service reads the latest export manifest, latest runtime part instances, friendly-name aliases, and the normalized definition/value tables, then emits a compact prompt context with stable part keys, instance keys, construction ids, runtime ids, indexes, coordinates, and observed metadata details. Raw export JSON is used only as a legacy fallback for older unnormalized databases.

## Runtime Export Payload

The scene exporter includes runtime metadata unavailable in the local BSON save, including:

- part asset name
- category
- display name
- full display name
- mass
- strength
- stage index
- transforms
- behaviours
- paint target colour where `IPartPaint` exposes it
- material and paintability properties where `IPartProperties` exposes them
- attachment summaries from `IPartAttachments` and `IAttachment`, including owned / associated attachments, attachment type names, lock state, joint flags, and owner / connected positions
- link node summaries from `ILinkNode`, including type names, link availability, and node coordinates
- tweakable and resizable data from `ITweakables` and `IResizable`, including current unit size, resize step, and exposed tweakable values
- controllable behaviour details such as control binding and activation state
- engine relationship details from `IEngineCrank`, `IEngineDrivenCrank`, `IEngineCylinder`, and `IEngineHead`, including crank / cylinder / head links, timing angle, crank angle, linked cylinder count, and current rotation speed
- parent construction hints
- link nodes where exposed by the runtime API
- API availability metadata

## API Availability Metadata

Exporter payloads include availability-only API metadata for documented construction interfaces.

Example shape:

```json
{
  "interface": "IPart",
  "name": "Mass",
  "valueType": "available",
  "availability": "declared"
}
```

The parts catalog uses this metadata to show which attributes are available for a part without showing values.

API getter values are not included in default chat prompt context and should require an explicit future include/snapshot control.

Mutating operation interfaces remain represented as support descriptors and are not called by the exporter.

## Official API Index

Overlay Forge can import the official GearBlocks Doxygen API documentation into the normalized API catalog.

Use:

```text
Gaming -> GearBlocks -> API -> Import Official Docs
```

The importer fetches the official API hierarchy, namespace pages, and type reference pages from:

```text
https://www.gearblocksgame.com/apidoc/
```

The import is metadata-only. It indexes documented types, members, parameters, and namespace enum values into SQLite so chat and UI surfaces can reference the available API shape. It does not execute GearBlocks API getters, setters, or mutating methods.

The original hand-maintained Construction API seed remains as a fallback source, especially for known Lua enum aliases and curated notes.

## Interfaces Covered

Overlay Forge seeds and can refresh the documented `SmashHammer.GearBlocks.Construction` namespace reference interfaces:

- `IAttachment`
- `IAttachmentOperations`
- `ICheckpoint`
- `IConstruction`
- `IConstructionOperations`
- `IControllablePartBehaviour`
- `IEnergyStore`
- `IEngineCrank`
- `IEngineCylinder`
- `IEngineDrivenCrank`
- `IEngineHead`
- `ILink`
- `ILinkNode`
- `IPart`
- `IPartAttachments`
- `IPartBehaviour`
- `IPartBehaviourOperations`
- `IPartPaint`
- `IPartProperties`
- `IPopulateConstructions`
- `IResizable`
- `ITweakables`

Backend support structures live in:

```text
src-tauri/src/gearblocks_api.rs
src-tauri/src/gearblocks_api_scraper.rs
```

## Unified Script Window

Overlay Forge installs one GearBlocks script mod named `Overlay Forge` under:

```text
ScriptMods\OverlayForgeConstructionExporter
```

The script creates one movable, resizable, collapsible GearBlocks window with a compact home menu for:

- Scene
- Builder
- Weld
- Status

Selecting a menu item replaces the window content, changes the window title, and shows a Back button without intentionally recentering the window.

The Scene section exports the live scene. Builder covers common `BuilderToolExt` actions. Weld covers common `WeldTool` actions.

Installing the unified script also removes the older `OverlayForgeTools` user script folder when present.

## Troubleshooting Index

If a future GearBlocks version or a specific part exposes bad, missing, or surprising API data, start with:

- `src-tauri/src/gearblocks_api.rs`: fallback interface descriptors, getter names, value type expectations, and method invocation policy.
- `src-tauri/src/gearblocks_api_scraper.rs`: official Doxygen import parsing for API types, members, parameters, and enum values.
- `gearblocks-script-mods/OverlayForgeConstructionExporter/main.lua`: runtime object export, API availability metadata, JSON emission, and guarded failures from GearBlocks' Lua sandbox.
- `src-tauri/src/commands.rs`: exporter install-time known API index injection, `Player.log` chunk reconstruction, runtime export import, API attribute normalization, and chat-context construction summaries.
- `src-tauri/src/db.rs`: SQLite persistence for runtime export JSON and runtime part JSON.
- `src/features/gaming/Gaming.tsx`: Parts detail display.

Common patterns:

- A getter appears in the catalog without a value: expected; catalog API metadata is availability-only by default.
- Build-guide-relevant live values such as paint/material, attachments, link nodes, tweakables, resizable sizes, controllable state, and engine relationships are captured by the exporter because they are needed for current scene/build-guide reasoning.
- Other future live getter values should still require an explicit user-controlled include/snapshot action.
- Export is slow after an API metadata change: reinstall the exporter so the known API index is regenerated from SQLite, then export again.
- A full scene export appears successful in GearBlocks but chat only sees an old runtime export: check whether `Player.log` rotated after GearBlocks restarted. Overlay Forge treats a shorter replacement log as a rotation, reads that new log from the beginning once, and records a recovery check so early completed export chunks are not skipped by the normal tail-read safety limit. Prompt-triggered rich exports are also read directly from the log offset where the request began so large requested exports are not missed by the normal tail-read safety limit.
- Chat misses repeated structural parts even though the latest export contains them: inspect the parent construction groups in prompt context. Part IDs can repeat across constructions, so chat should use construction id, part id, index, and coordinates together before concluding that plates, beams, or other repeated parts are missing.
- A GearBlocks update changes an interface name or method name: run `Import Official Docs`, then update fallback descriptors in `gearblocks_api.rs` only if the curated seed or Lua aliases need adjustment.
- Import succeeds but chat lacks API details: expected unless a future explicit prompt-inclusion control is enabled.
