# GearBlocks Plugin And Modding Boundary

## Status

GearBlocks BepInEx plugin work is currently backlog.

The source templates and command code remain in the repository, but active GearBlocks chat should not request marker placement, emit `overlay-forge-markers` blocks, or require BepInEx/GearLib installation while marker rendering is paused.

## Direct Overlay Forge BepInEx Plugin

Overlay Forge owns a direct GearBlocks BepInEx plugin template for future in-game features that are not practical through GearBlocks Lua alone.

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

The first direct plugin experiment was a file-backed command channel for temporary visual markers.

Overlay Forge writes command JSON files. The plugin polls those commands from inside GearBlocks.

If marker work resumes, this remains the likely path for AI-directed temporary block or surface references because the Lua script API is not expected to provide enough control over surface hit points, world-space drawing, or runtime-only marker objects.

Current active behavior: marker commands are not prompted, surfaced, or sent from GearBlocks chat.

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
