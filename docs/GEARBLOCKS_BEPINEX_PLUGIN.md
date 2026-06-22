# GearBlocks Direct BepInEx Plugin

Overlay Forge owns a direct GearBlocks BepInEx plugin template for in-game features that are not practical through GearBlocks Lua alone.

The current template lives at:

```text
gearblocks-bepinex-templates/OverlayForgeGearBlocksPlugin/
```

Generated working copies belong under the ignored local workspace:

```text
gearblocks-bepinex-workspace/OverlayForgeGearBlocksPlugin/
```

Do not commit copied reference DLLs, BepInEx DLLs, GearBlocks interop DLLs, Unity DLLs, or compiled plugin output.
The local workspace must include Unity IMGUI references from the user's GearBlocks/BepInEx install because marker labels and screen-space crosshairs are rendered through Unity IMGUI.

## Purpose

The first plugin feature is a file-backed command bridge for temporary visual markers. Overlay Forge can write command JSON files, and the plugin polls those commands from inside GearBlocks.

This is the preferred path for AI-directed temporary block or surface references because the Lua script API is not expected to provide enough control over surface hit points, world-space drawing, or runtime-only marker objects.

GearBlocks chat can request markers by including an `overlay-forge-markers` JSON block in an assistant response. Overlay Forge detects that block and shows a user-controlled `Send Marker(s)` action. Marker commands are not sent automatically.

## Runtime Command Folder

At runtime, the plugin creates and watches:

```text
<GearBlocks persistent data>\OverlayForgePlugin\commands\*.json
```

For a standard Windows GearBlocks install this should resolve under:

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

### spawn_center_marker

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

Markers are runtime-only Unity objects. They are not intended to save into constructions. The plugin also draws a temporary screen-space crosshair and label over the marker while the target is in front of the active camera.

### spawn_world_marker

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

This is the primary path for chat-directed explanations because the latest scene export includes part world coordinates. Markers are runtime-only Unity objects and do not modify saved constructions. The plugin also draws a temporary screen-space crosshair and label over the marker while the target is in front of the active camera.

### clear_markers

Removes all active Overlay Forge temporary markers.

```json
{
  "action": "clear_markers",
  "id": "clear-example"
}
```

### ping

Writes a status file so Overlay Forge can confirm that the plugin command bridge is alive.

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

The ignored workspace `libs` folder must contain the required local GearBlocks/BepInEx references, including:

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

## Chat Marker Block

Assistant responses may include a final marker block shaped as:

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

Overlay Forge strips this raw block from the displayed assistant message and shows a marker action button instead.
