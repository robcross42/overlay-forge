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

## Scene Delta Behavior

After a full scene export establishes a baseline, the loaded GearBlocks script monitors `Parts.Instances` and emits compact scene-delta records into `Player.log` for added, changed, and removed parts.

Overlay Forge imports those scene deltas before prompt assembly and applies them to the latest indexed runtime export as synthetic scene-delta snapshots. This lets chat see the most recent script-observed scene state without triggering a full export on every prompt.

Because the default export covers the whole scene, removed parts disappear from latest chat context after the next scene export/import or after an imported scene-delta removal.

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
- A future workflow needs live getter values: add an explicit user-controlled include/snapshot action.
- Export is slow after an API metadata change: reinstall the exporter so the known API index is regenerated from SQLite, then export again.
- A GearBlocks update changes an interface name or method name: run `Import Official Docs`, then update fallback descriptors in `gearblocks_api.rs` only if the curated seed or Lua aliases need adjustment.
- Import succeeds but chat lacks API details: expected unless a future explicit prompt-inclusion control is enabled.
