# GearBlocks Construction Decoder

Overlay Forge can inspect local GearBlocks saved construction folders from the selected GearBlocks home screen.

GearBlocks' default user data location is:

```text
%USERPROFILE%\AppData\LocalLow\SmashHammer Games\GearBlocks\
```

Overlay Forge uses that root for default GearBlocks paths unless the user configures a feature-specific override.

## Local Save Format

GearBlocks saved construction folders contain `construction.bytes` files. Current local samples decode as raw DEFLATE-compressed BSON:

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

The local save file does not include enough human-readable part metadata to reliably name every part by itself. The GearBlocks Lua scripting API exposes that richer runtime view after a construction is loaded in-game.

Relevant API surfaces:

- `IConstruction` exposes construction part count, composite count, frozen/invulnerable state, mass, active stage, `Parts`, and `GetPart(idx)`: <https://www.gearblocksgame.com/apidoc/interface_smash_hammer_1_1_gear_blocks_1_1_construction_1_1_i_construction.html>
- `IPart` exposes asset GUID, asset name, category, display name, full display name, mass, stage, behaviours, link nodes, transforms, and bounds: <https://www.gearblocksgame.com/apidoc/interface_smash_hammer_1_1_gear_blocks_1_1_construction_1_1_i_part.html>
- `Constructions` is the Lua global for construction instances; it supports `GetInstance(id)` and `Instances` enumeration: <https://www.gearblocksgame.com/apidoc/class_smash_hammer_1_1_gear_blocks_1_1_construction_1_1_construction.html>
- `PopConstructions` is the Lua global for spawning saved constructions, including `SpawnConstruction(savedFolder, saveTypeID)`: <https://www.gearblocksgame.com/apidoc/interface_smash_hammer_1_1_gear_blocks_1_1_construction_1_1_i_populate_constructions.html>

The first implemented decoder stage is local and does not require GearBlocks to be running. The runtime exporter stage is implemented as a GearBlocks script mod template in `gearblocks-script-mods/OverlayForgeConstructionExporter`.

## Lua Exporter

Overlay Forge can install the script mod from Gaming -> GearBlocks -> Home -> Construction Decoder -> Install Exporter.

The installer writes:

```text
%USERPROFILE%\AppData\LocalLow\SmashHammer Games\GearBlocks\ScriptMods\OverlayForgeConstructionExporter\main.lua
%USERPROFILE%\AppData\LocalLow\SmashHammer Games\GearBlocks\ScriptMods\OverlayForgeConstructionExporter\meta.json
```

The script mod install path is always GearBlocks' standard user script directory under the default user data location. It does not follow the configured Save Location or Alternate Data Location.

The installed `main.lua` includes an absolute export directory. If GearBlocks has an Alternate Data Location configured in Overlay Forge, the exporter attempts to write there. Otherwise it targets:

```text
%USERPROFILE%\AppData\LocalLow\SmashHammer Games\GearBlocks\OverlayForgeExports
```

After installing, launch or restart GearBlocks so the locally saved script mod is discovered, then open GearBlocks' Script Mods screen and load `Overlay Forge Construction Exporter`.

The script mod window has two actions:

- `Export Target`: point at any part in a construction, then export that part's parent construction.
- `Export All`: export all currently loaded scene parts from `Parts.Instances`, so parts do not need to be attached into one construction before exporting. If scene-wide part enumeration is unavailable, the exporter falls back to all currently loaded constructions from `Constructions.Instances`.

GearBlocks currently blocks `io.open` in the script context, so the exporter falls back to printing marked JSON chunks into `Player.log`. Use Gaming -> GearBlocks -> Parts -> Import Runtime Log after running `Export Target` or `Export All` to reconstruct those runtime exports in Overlay Forge.

The targeted construction exporter walks `IConstruction.Parts`. The scene-wide exporter walks `Parts.Instances`. Both exports include runtime metadata unavailable in the local BSON save, including part asset name, category, display name, full display name, mass, strength, stage index, transforms, behaviours, parent construction hints, and link nodes where exposed by the runtime API.

Exporter payloads also include `apiAttributes` availability metadata for documented construction interfaces. The parts catalog uses that metadata to show which attributes are available for a part without showing values. Runtime construction exports keep the interface/member availability index in SQLite, but API getter values are not included in default chat prompt context and should require an explicit future include/snapshot control.

On exporter install, Overlay Forge injects the known API index from SQLite into `main.lua`. During export, known parts reuse that index and skip API interface discovery. Unknown parts may be probed once for availability metadata, then cached in the current script session and persisted through the normal runtime-log import path.

After the user imports runtime logs, GearBlocks chat uses the latest reconstructed runtime export indexed in SQLite. Normal game selection and Parts navigation do not automatically scan `Player.log` or `Player-prev.log`; use `Import Runtime Log` after running `Export Target` or `Export All` in GearBlocks. Overlay Forge stores the complete export payload in SQLite and derives a semantic construction understanding model from that indexed export:

- structural frame and connector inventories are aggregated instead of treated as a visual mesh
- functional systems are classified as suspension, steering, drivetrain, engine, brakes/clutches, wheels/tires, controls/data, bodywork, or unknown
- functional parts are listed with inferred purpose, behaviours, link-node counts, local position, and current unit size where available
- structural bounds are summarized as a coarse local chassis envelope

Overlay Forge can install and type-check the script mod from outside the game, but the GearBlocks runtime API can only be exercised after the script is loaded inside GearBlocks.

## Current UI

On Gaming -> GearBlocks -> Home, the Construction Decoder panel lists saved construction folders from the configured GearBlocks Save Location. If no Save Location is configured, Overlay Forge falls back to:

```text
%USERPROFILE%\AppData\LocalLow\SmashHammer Games\GearBlocks\SavedConstructions
```

Selecting Decode inflates and parses the selected `construction.bytes` file, then displays:

- composite count
- part count
- unique asset GUID count
- attachment count
- link count
- decoded payload size
- decoded JSON preview

On Gaming -> GearBlocks -> Constructions, Overlay Forge indexes the saved construction folders into SQLite. Each indexed construction stores the decoded BSON JSON plus summary fields including byte size, decoded byte size, composite count, part count, unique asset GUID count, attachment count, link count, intersection count, and frozen / invulnerable flags.
