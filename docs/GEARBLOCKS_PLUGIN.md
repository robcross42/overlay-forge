# GearBlocks Plugin And Modding Boundary

## Status

GearBlocks direct BepInEx plugin work is active for Unity-side part preview rendering.

Marker rendering remains paused. Active GearBlocks chat should not request marker placement, emit `overlay-forge-markers` blocks, or require GearLib installation while marker rendering is paused.

## Direct Overlay Forge BepInEx Plugin

Overlay Forge owns a direct GearBlocks BepInEx plugin template for in-game features that are not practical through GearBlocks Lua alone.

Source template:

```text
gearblocks-bepinex-templates/OverlayForgeGearBlocksPlugin/
```

Generated working copies belong under the ignored local workspace:

```text
gearblocks-bepinex-workspace/OverlayForgeGearBlocksPlugin/
```

Do not commit copied reference DLLs, BepInEx DLLs, GearBlocks interop DLLs, Unity DLLs, or compiled plugin output.

## Backlog Purpose

The current direct plugin experiment is a file-backed Unity runtime bridge.

The active command captures a centered part preview PNG from the object under the active camera crosshair:

```json
{
  "action": "capture_center_part_preview",
  "id": "beam-x3-preview",
  "label": "Beam x3",
  "width": 1024,
  "height": 576,
  "yawDegrees": 35,
  "pitchDegrees": 28,
  "partRotationXDegrees": 0,
  "partRotationYDegrees": 0,
  "partRotationZDegrees": 0
}
```

The plugin raycasts from the center of the active camera, clones the hit object's renderers into an isolated Unity preview layer, applies optional part rotation around the captured preview center, renders only the part/object on a neutral background with thin dark crease/boundary edges for low-contrast faces, and writes:

```text
<GearBlocks persistent data>\OverlayForgePlugin\renders\<id>.png
<GearBlocks persistent data>\OverlayForgePlugin\status\<id>.json
```

`yawDegrees` and `pitchDegrees` control the preview camera view. `partRotationXDegrees`, `partRotationYDegrees`, and `partRotationZDegrees` rotate the cloned part/object itself and can be any degree value, including full 360-degree sweeps on each axis.

The status JSON includes the render path, source object name, label, renderer names/count, edge-line counts, camera view angles, part rotation angles, output dimensions, source bounds, and timestamp.

Overlay Forge can persist a successful status capture as a GearBlocks part render profile through `save_gearblocks_part_render_profile_from_capture`. The profile stores the canonical orientation, source/renderer diagnostics, framing metadata, edge metadata, latest status JSON, and latest render cache path in SQLite while leaving generated PNG files under `OverlayForgePlugin\renders`.

Profiles are intended to support on-demand rotated render requests. Overlay Forge should keep one validated canonical profile per part shape, then request or cache only the specific rotations needed by a build guide instead of pre-rendering every rotation combination.

Individual part preview images intentionally do not include grid planes, axis markers, orientation labels, or rotation guidance. They may include object-edge definition so white or low-contrast parts remain readable. If Unity marks a mesh as non-readable, the plugin skips edge extraction for that mesh and still renders the part material. Build-step image generation owns grid and axis display when composing one or more part previews into a placement diagram.

Preview fallback selection ignores huge environment renderers such as boundary indicators so missed part hierarchy lookups fail clearly instead of producing blank-looking boundary captures.

During the current part-preview iteration loop, Overlay Forge routes the configured Mouse5 shortcut action to this command instead of screenshot capture. Each press writes the next available test command id, such as `test-part-preview-1`, `test-part-preview-2`, and so on. Finalized part captures will use stable part-specific ids later.

Current gap: the active command still starts from the live part under the camera center. A future targetless render command needs either a stable way to re-find a saved source object in the loaded scene, a GearBlocks/Unity path to spawn a part by asset identity into a hidden preview scene, or a cache-on-demand workflow that asks the user to target the part only when a missing profile or rotation is needed.

The earlier direct plugin experiment was a file-backed command channel for temporary visual markers.

Overlay Forge writes command JSON files. The plugin polls those commands from inside GearBlocks.

If marker work resumes, this remains the likely path for AI-directed temporary block or surface references because the Lua script API is not expected to provide enough control over surface hit points, world-space drawing, or runtime-only marker objects.

Current active marker behavior: marker commands are not prompted, surfaced, or sent from GearBlocks chat.

## Runtime Command Folder

At runtime, the plugin creates and watches:

```text
<GearBlocks persistent data>\OverlayForgePlugin\commands\*.json
```

For a standard Windows GearBlocks install this resolves under:

```text
%USERPROFILE%\AppData\LocalLow\SmashHammer Games\GearBlocks\OverlayForgePlugin\
```

Processed commands are moved to:

```text
<GearBlocks persistent data>\OverlayForgePlugin\processed\
```

Status files are written to:

```text
<GearBlocks persistent data>\OverlayForgePlugin\status\
```

## Supported Commands

### `spawn_center_marker`

Raycasts from the center of the active camera and places a temporary crosshair marker on the first hit surface.

```json
{
  "action": "spawn_center_marker",
  "id": "example-marker",
  "label": "Front axle reference",
  "color": "#55f0c8",
  "durationSeconds": 20,
  "size": 4.0
}
```

### `spawn_world_marker`

Places a temporary center sphere plus crosshair marker at explicit GearBlocks world coordinates.

```json
{
  "action": "spawn_world_marker",
  "id": "rear-diff-connection",
  "label": "Rear diff input",
  "reason": "Connect the motor output shaft here.",
  "x": 2.6,
  "y": 1.1,
  "z": -0.4,
  "color": "#55f0c8",
  "durationSeconds": 45,
  "size": 4.0
}
```

This is the primary path for chat-directed explanations because the latest scene export includes part world coordinates.

### `clear_markers`

Removes all active Overlay Forge temporary markers.

```json
{
  "action": "clear_markers",
  "id": "clear-example"
}
```

### `ping`

Writes a status file so Overlay Forge can confirm that the plugin command channel is alive.

```json
{
  "action": "ping",
  "id": "ping-example"
}
```

## Installation

Build the ignored working copy with:

```powershell
dotnet build gearblocks-bepinex-workspace\OverlayForgeGearBlocksPlugin\OverlayForgeGearBlocksPlugin.csproj
```

The ignored workspace `libs` folder must contain required local GearBlocks/BepInEx references, including:

```text
UnityEngine.IMGUIModule.dll
UnityEngine.ImageConversionModule.dll
```

For a standard BepInEx install, this can be copied from:

```text
C:\Program Files (x86)\Steam\steamapps\common\GearBlocks\BepInEx\interop\UnityEngine.IMGUIModule.dll
```

Install the built DLL to:

```text
C:\Program Files (x86)\Steam\steamapps\common\GearBlocks\BepInEx\plugins\OverlayForgeGearBlocksPlugin\OverlayForgeGearBlocksPlugin.dll
```

Restart GearBlocks after installing or replacing the DLL.

## Development Troubleshooting

When iterating on the BepInEx plugin and Overlay Forge at the same time, close GearBlocks before replacing the plugin DLL or starting a fresh `npm run tauri:dev` session if the dev run fails during Rust compilation with:

```text
STATUS_ACCESS_VIOLATION
```

One observed run failed while compiling `overlay-forge` through `cargo run` with `rustc.exe` exiting as `0xc0000005, STATUS_ACCESS_VIOLATION`. Exiting GearBlocks was required before continuing the development run. Treat this as an operational workaround while the exact cause is unconfirmed.

## Paused Marker Response Block

This response shape is retained for future marker work only. Assistant responses should not currently include this block:

````text
```overlay-forge-markers
{
  "markers": [
    {
      "label": "Rear motor to axle",
      "reason": "Add the missing coupling at this connection point.",
      "x": 2.6,
      "y": 1.1,
      "z": -0.4,
      "color": "#55f0c8",
      "durationSeconds": 45,
      "size": 4.0
    }
  ]
}
```
````

Active GearBlocks chat strips legacy marker blocks from displayed assistant messages and does not show marker action buttons while marker support is paused.

## GearLib Boundary

GearLib is a third-party GearBlocks modding library by KaBooMa. Overlay Forge may use GearLib for future BepInEx-based GearBlocks features, but Overlay Forge does not vendor GearLib or claim ownership of it.

Overlay Forge documentation and any future install UI must state that GearLib and BepInEx are separate user-installed requirements. Do not make either one an active user-facing requirement until plugin work is explicitly resumed.

Upstream references:

- GearLib repository: `https://github.com/KaBooMa/GearLib`
- GearLib docs: `https://kabooma.github.io/GearLib/`
- BepInEx builds: `https://builds.bepinex.dev/projects/bepinex_be`

GearLib-based source template:

```text
gearblocks-bepinex-templates/OverlayForgeGearLibPlugin/
```

The direct non-GearLib plugin template remains:

```text
gearblocks-bepinex-templates/OverlayForgeGearBlocksPlugin/
```

Use the direct plugin path for runtime-only Overlay Forge features that do not need GearLib.

## Required User Installation For GearLib-Based Work

Before a GearLib-based Overlay Forge plugin can run, the user must install:

1. BepInEx for GearBlocks.
2. GearLib into `GearBlocks\BepInEx\plugins`.
3. Any Overlay Forge GearBlocks plugin DLL built from this template into the GearBlocks BepInEx plugins folder.

## GearLib Development Setup

The local machine should have .NET SDK 6.0 and the BepInEx templates installed.

Current working template package:

```powershell
dotnet new install BepInEx.Templates@2.0.0-be.4 --nuget-source https://nuget.bepinex.dev/v3/index.json
```

After copying the repo template to `gearblocks-bepinex-workspace`, copy local reference DLLs from:

```text
C:\Program Files (x86)\Steam\steamapps\common\GearBlocks\BepInEx\plugins
C:\Program Files (x86)\Steam\steamapps\common\GearBlocks\BepInEx\interop
```

Only copy them into ignored local working folders.
