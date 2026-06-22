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

The script mod window exposes `Export Scene` as the default export action. It exports all currently loaded scene parts from `Parts.Instances`, so parts do not need to be attached into one construction before exporting. If scene-wide part enumeration is unavailable, the exporter falls back to all currently loaded constructions from `Constructions.Instances`.

GearBlocks currently blocks `io.open` in the script context, so the exporter falls back to printing marked JSON chunks into `Player.log`. Gaming -> GearBlocks -> Chat -> Refresh Scene Context and Gaming -> GearBlocks -> Parts -> Refresh Scene Context request a fresh in-game scene export through the exporter hotkey before importing new runtime exports in Overlay Forge. GearBlocks chat send reconstructs already-written runtime exports through the same cursor importer. The backend stores per-log import cursors in SQLite so refresh and send paths read only new `Player.log` / `Player-prev.log` additions when possible. Normal chat navigation must not synchronously parse full-scene runtime logs.

After a full scene export establishes a baseline, the loaded GearBlocks script also monitors `Parts.Instances` and emits compact `[OverlayForgeSceneDelta]` records into `Player.log` for added, changed, and removed parts. These records are intended for iterative build changes such as copying a beam after the last full export. Overlay Forge imports those scene deltas before prompt assembly and applies them to the latest indexed runtime export as synthetic `sceneDeltaPatch` snapshots, so chat can see the most recent script-observed scene state without triggering another full export on every prompt.

The scene exporter walks `Parts.Instances`. It includes runtime metadata unavailable in the local BSON save, including part asset name, category, display name, full display name, mass, strength, stage index, transforms, behaviours, parent construction hints, and link nodes where exposed by the runtime API.

Exporter payloads also include `apiAttributes` availability metadata for documented construction interfaces. The parts catalog uses that metadata to show which attributes are available for a part without showing values. Runtime construction exports keep the interface/member availability index in SQLite, but API getter values are not included in default chat prompt context and should require an explicit future include/snapshot control.

On exporter install, Overlay Forge injects the known API index from SQLite into `main.lua`. During export, known parts reuse that index and skip API interface discovery. Unknown parts may be probed once for availability metadata, then cached in the current script session and persisted through the normal runtime-log import path.

After the user exports the scene, GearBlocks chat uses the latest reconstructed runtime export indexed in SQLite. Overlay Forge stores the complete export payload in SQLite and derives a semantic construction understanding model from that indexed export.

## GearBlocks Scale And Units

GearBlocks developer guidance on Steam confirms that `1 unit = 10 cm`. Source: `https://steamcommunity.com/app/1305080/discussions/6/4696784170963236905/`. GearBlocks chat context should use this ratio when discussing part movement, spacing, dimensions, alignment, and vehicle scale. Preferred response units are centimeters and/or GearBlocks units such as `1 unit`, `0.5 units`, or `16 units`; chat should not suggest imperial distances unless the user explicitly asks for an imperial conversion.

Scale exceptions: the developer noted that the player character, wheels, and other parts are slightly oversized so gears and other parts have room to fit inside vehicles. Treat those parts as gameplay-clearance exceptions rather than strict real-world scale references.

The intended iterative workflow is to load the current build in GearBlocks once, make a full scene export, then continue building in-game while the Overlay Forge script remains loaded. Manual script-window exports are picked up by Overlay Forge's passive GearBlocks runtime import monitor, which reads only new completed `Player.log` export chunks by cursor. After the baseline export, the script's scene-delta monitor can append lightweight additions, updates, and removals to `Player.log`; GearBlocks chat send imports those new log additions before creating the prompt context so chat uses the latest indexed scene available at submit time. Runtime part reference rows, API availability rows, value fields, properties, and attachments are upserted by stable keys, so repeated scene exports update the current SQLite reference data instead of requiring a full catalog rebuild. Overlay Forge also stores a compact diff between the latest and previous runtime scene export for chat prompt context. Because the default export covers the whole scene, removed parts disappear from the latest chat context after the next scene export and import or after an imported scene-delta removal.

When GearBlocks saves a construction, Overlay Forge can also use the saved `construction.bytes` file as a current-build signal. GearBlocks chat decodes the most recently modified saved construction file before building its prompt context, so saved part additions and removals can be reflected even before a new runtime scene export. This saved-file context complements the runtime log context while runtime exports continue to provide live metadata unavailable in the save file.

The derived semantic model includes:

- structural frame and connector inventories are aggregated instead of treated as a visual mesh
- functional systems are classified as suspension, steering, drivetrain, engine, brakes/clutches, wheels/tires, controls/data, bodywork, or unknown
- functional parts are listed with inferred purpose, behaviours, link-node counts, local position, world position, and current unit size where available
- every runtime-indexed part stores first-class world and local coordinate columns so structural-only parts can still be referenced by temporary marker commands
- structural bounds are summarized as a coarse local chassis envelope

GearBlocks chat prompt context also tells the assistant how to create optional `overlay-forge-markers` JSON blocks for visual explanations. These marker blocks reference GearBlocks world coordinates from the latest runtime export and are only sent to the in-game BepInEx plugin after the user clicks the marker action in Overlay Forge.

Overlay Forge can install and type-check the script mod from outside the game, but the GearBlocks runtime API can only be exercised after the script is loaded inside GearBlocks.

## Unified Script Window

Overlay Forge installs one GearBlocks script mod named `Overlay Forge` under `ScriptMods\OverlayForgeConstructionExporter`. The script creates one movable, resizable, collapsible GearBlocks window with a compact home menu for Scene, Builder, Weld, and Status. Selecting a menu item replaces the window content with that tool view, changes the window title, and shows a Back button at the top without intentionally recentering the window. This keeps scene export, BuilderToolExt helpers, and WeldTool controls in one script window instead of loading multiple GearBlocks sample scripts with separate windows.

The Scene section exports the full live scene for Overlay Forge chat context and displays export progress, success/failure, and exported part count in the same view. The Builder section covers common `BuilderToolExt` actions such as manipulator orientation, step intervals, move-to-ground, pivot snapping, resize clamp, unit sizing, interpenetration, attachment bridging, and show-all-attachments toggles. The Weld section covers common `WeldTool` actions including attachment type selection, start/complete weld, detach targeted part, and targeted part feedback.

Installing the unified script also removes the older `OverlayForgeTools` user script folder when present. Overlay Forge still uses the fixed scene-export hotkey for explicit scene refresh requests, but Builder and Weld controls now run from the in-game script window rather than from arbitrary keyboard input or arbitrary Lua execution sent by Overlay Forge.

## Third-party Mod Dependencies

Overlay Forge may integrate with workflows that require BepInEx and GearLib, but both are third-party dependencies that users must install separately. Overlay Forge must not bundle, redistribute, install, or modify BepInEx or GearLib unless a future explicit license/permission review allows that.

BepInEx is a Unity modding framework. For GearBlocks/GearLib usage, users should follow BepInEx Unity IL2CPP installation guidance and install it into the GearBlocks game root. Overlay Forge detects BepInEx by looking for the `BepInEx` folder and related loader/runtime files under the detected GearBlocks install root. The status check also reads `BepInEx\LogOutput.log` and `BepInEx\ErrorLog.log` when present. `LogOutput.log` is used to report the installed BepInEx version and confirm chainloader activation through `Chainloader initialized` / `Chainloader startup complete` log lines.

GearLib is a third-party GearBlocks modding library by KaBooMa. Its README says GearLib is a requirement for mods made with it and asks modders to tell users about that requirement before distribution. Overlay Forge detects GearLib by looking for `GearLib.dll` under `BepInEx/plugins`.

Overlay Forge also owns a direct BepInEx plugin template that does not depend on GearLib. The direct plugin is intended for Overlay Forge-owned runtime features such as temporary visual markers that cannot be handled cleanly through Lua. See `docs/GEARBLOCKS_BEPINEX_PLUGIN.md`.

Current upstream references:

- BepInEx Unity IL2CPP install docs: `https://docs.bepinex.dev/master/articles/user_guide/installation/unity_il2cpp.html`
- GearLib repository: `https://github.com/KaBooMa/GearLib`

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
